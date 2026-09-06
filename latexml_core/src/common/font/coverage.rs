//! Per-font glyph coverage — the answer `\iffontchar` gives.
//!
//! eTeX's `\iffontchar <font><charcode>` (etex_man §3.7) is true iff the
//! font's metric file has a character in that slot: for a TFM the
//! `char_info` word's width index is non-zero (tftopl.web §"char_info"),
//! for an OpenType/TrueType font (LuaTeX/XeTeX) the `cmap` maps the code
//! point to a non-zero glyph. Perl LaTeXML leaves `\iffontchar` undefined
//! (eTeX.pool.ltxml:335) and our former stub answered TRUE for every code
//! point, which made unicodefonttable's `\displayfonttable` emit one cell
//! per code point of the requested range (262k cells for the samples doc:
//! `Fatal:Timeout:MemoryBudget` at 4.8 GB) instead of one per glyph the
//! font actually covers (~6,500 across its five tables). Real coverage is
//! the only faithful bound — a constant FALSE would skip every row.
//!
//! Font files are read from the host TeX Live tree (the ecosystem is in
//! scope; owned resources are embedded). Coverage is parsed once per file
//! and cached for the thread.

use std::{cell::RefCell, rc::Rc};

use rustc_hash::{FxHashMap, FxHashSet};

/// The set of code points (TFM: character codes) a font file covers.
#[derive(Debug)]
pub struct GlyphCoverage {
  set: FxHashSet<u32>,
}

impl GlyphCoverage {
  /// Does the font have a glyph in this slot?
  pub fn contains(&self, code: u32) -> bool { self.set.contains(&code) }

  /// Number of covered slots.
  pub fn len(&self) -> usize { self.set.len() }

  /// True when no slot is covered.
  pub fn is_empty(&self) -> bool { self.set.is_empty() }
}

thread_local! {
  static COVERAGE_CACHE: RefCell<FxHashMap<String, Option<Rc<GlyphCoverage>>>> =
    RefCell::new(FxHashMap::default());
  static FONT_INDEX: RefCell<Option<Rc<FontIndex>>> = const { RefCell::new(None) };
}

/// Coverage of the font file at `path` (`.tfm`, `.otf`, `.ttf`, `.ttc`);
/// `None` when the file is missing or unparseable. Cached per path.
pub fn coverage_for_file(path: &str) -> Option<Rc<GlyphCoverage>> {
  COVERAGE_CACHE.with(|cache| {
    if let Some(hit) = cache.borrow().get(path) {
      return hit.clone();
    }
    let parsed = std::fs::read(path).ok().and_then(|bytes| {
      let lower = path.to_ascii_lowercase();
      if lower.ends_with(".tfm") {
        parse_tfm(&bytes)
      } else {
        parse_sfnt_cmap(&bytes)
      }
    });
    let entry = parsed.map(|set| Rc::new(GlyphCoverage { set }));
    cache.borrow_mut().insert(path.to_string(), entry.clone());
    entry
  })
}

// ---------------------------------------------------------------------------
// TFM (tftopl.web: the header is 12 halfwords lf lh bc ec nw nh nd ni nl nk ne
// np; char_info starts after the `lh`-word header and has one 4-byte word per
// character bc..ec whose first byte is the width index — zero means "no
// character in this slot").
fn parse_tfm(b: &[u8]) -> Option<FxHashSet<u32>> {
  if b.len() < 24 {
    return None;
  }
  let hw = |i: usize| u16::from_be_bytes([b[2 * i], b[2 * i + 1]]) as usize;
  let (lf, lh, bc, ec) = (hw(0), hw(1), hw(2), hw(3));
  if lf * 4 != b.len() || bc > ec + 1 || ec > 255 {
    return None; // not a TFM (JFM/OFM headers differ) — no coverage claim
  }
  let mut set = FxHashSet::default();
  let start = 24 + 4 * lh;
  if bc <= ec {
    for c in bc..=ec {
      let off = start + 4 * (c - bc);
      if off + 4 > b.len() {
        break;
      }
      if b[off] != 0 {
        set.insert(c as u32);
      }
    }
  }
  Some(set)
}

// ---------------------------------------------------------------------------
// sfnt (OpenType/TrueType/TrueType-collection): locate the `cmap` table and
// read its best Unicode subtable (format 12 full-repertoire, else 4 BMP,
// else 6/0 byte tables).
fn rd16(b: &[u8], o: usize) -> Option<u16> {
  Some(u16::from_be_bytes([*b.get(o)?, *b.get(o + 1)?]))
}
fn rd32(b: &[u8], o: usize) -> Option<u32> {
  Some(u32::from_be_bytes([
    *b.get(o)?,
    *b.get(o + 1)?,
    *b.get(o + 2)?,
    *b.get(o + 3)?,
  ]))
}

/// Offset of the table directory of the first font in `b` (TTC → first face).
fn sfnt_base(b: &[u8]) -> Option<usize> {
  if b.get(0..4)? == b"ttcf" {
    Some(rd32(b, 12)? as usize)
  } else {
    Some(0)
  }
}

/// `(offset, length)` of the named table.
fn sfnt_table(b: &[u8], tag: &[u8; 4]) -> Option<(usize, usize)> {
  let base = sfnt_base(b)?;
  let num_tables = rd16(b, base + 4)? as usize;
  for i in 0..num_tables {
    let rec = base + 12 + 16 * i;
    if b.get(rec..rec + 4)? == tag {
      let off = rd32(b, rec + 8)? as usize;
      let len = rd32(b, rec + 12)? as usize;
      return if off.checked_add(len)? <= b.len() {
        Some((off, len))
      } else {
        None
      };
    }
  }
  None
}

fn parse_sfnt_cmap(b: &[u8]) -> Option<FxHashSet<u32>> {
  let (off, len) = sfnt_table(b, b"cmap")?;
  let c = &b[off..off + len];
  let n = rd16(c, 2)? as usize;
  let mut subtables: Vec<(u8, u16, u16, usize)> = Vec::with_capacity(n);
  for i in 0..n {
    let rec = 4 + 8 * i;
    let platform = rd16(c, rec)?;
    let encoding = rd16(c, rec + 2)?;
    let sub_off = rd32(c, rec + 4)? as usize;
    // Preference: Unicode full repertoire, Unicode BMP, then anything.
    let rank = match (platform, encoding) {
      (3, 10) => 0,
      (0, 4) | (0, 6) => 1,
      (3, 1) => 2,
      (0, _) => 3,
      _ => 4,
    };
    subtables.push((rank, platform, encoding, sub_off));
  }
  subtables.sort_by_key(|s| s.0);
  for (_, _, _, sub_off) in subtables {
    if let Some(set) = parse_cmap_subtable(c, sub_off) {
      return Some(set);
    }
  }
  None
}

fn parse_cmap_subtable(c: &[u8], off: usize) -> Option<FxHashSet<u32>> {
  let mut set = FxHashSet::default();
  match rd16(c, off)? {
    0 => {
      for code in 0..256usize {
        if *c.get(off + 6 + code)? != 0 {
          set.insert(code as u32);
        }
      }
    },
    4 => {
      let seg_x2 = rd16(c, off + 6)? as usize;
      let segs = seg_x2 / 2;
      let end_codes = off + 14;
      let start_codes = end_codes + seg_x2 + 2;
      let deltas = start_codes + seg_x2;
      let range_offsets = deltas + seg_x2;
      for s in 0..segs {
        let end = rd16(c, end_codes + 2 * s)? as u32;
        let start = rd16(c, start_codes + 2 * s)? as u32;
        let delta = rd16(c, deltas + 2 * s)?;
        let range_offset = rd16(c, range_offsets + 2 * s)? as usize;
        if start > end || start == 0xFFFF {
          continue;
        }
        for code in start..=end {
          let glyph = if range_offset == 0 {
            (code as u16).wrapping_add(delta)
          } else {
            let addr = range_offsets + 2 * s + range_offset + 2 * (code - start) as usize;
            let g = rd16(c, addr)?;
            if g == 0 { 0 } else { g.wrapping_add(delta) }
          };
          if glyph != 0 {
            set.insert(code);
          }
        }
      }
    },
    6 => {
      let first = rd16(c, off + 6)? as u32;
      let count = rd16(c, off + 8)? as usize;
      for i in 0..count {
        if rd16(c, off + 10 + 2 * i)? != 0 {
          set.insert(first + i as u32);
        }
      }
    },
    12 => {
      let groups = rd32(c, off + 12)? as usize;
      for g in 0..groups {
        let rec = off + 16 + 12 * g;
        let start = rd32(c, rec)?;
        let end = rd32(c, rec + 4)?;
        let start_glyph = rd32(c, rec + 8)?;
        if start > end || end > 0x10FFFF {
          continue;
        }
        for code in start..=end {
          if start_glyph + (code - start) != 0 {
            set.insert(code);
          }
        }
      }
    },
    _ => return None,
  }
  Some(set)
}

// ---------------------------------------------------------------------------
// Resolving a fontspec font name to a file.
//
// fontspec looks a family name ("Latin Modern Sans") up in luaotfload's
// name database, or takes a file name ("NewCMMath-Regular.otf") directly
// (fontspec.pdf §3 "Font selection"). We do not read luaotfload's cache;
// instead the ls-R database of the TeX Live tree gives every font file, and
// a family name is matched (1) as a file stem, (2) as the OpenType `name`
// table's family (name IDs 1/16), through an index built once from every
// font file's `name` table (two small reads per file — luaotfload's own
// database is built the same way).

struct FontIndex {
  /// lowercase basename → full path, for `.otf/.ttf/.ttc/.tfm`
  files: FxHashMap<String, String>,
}

fn texmf_trees() -> Vec<String> {
  let mut trees = Vec::new();
  for var in ["TEXMFDIST", "TEXMFLOCAL"] {
    if let Ok(out) = std::process::Command::new("kpsewhich")
      .arg(format!("-var-value={var}"))
      .output()
    {
      let dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
      if !dir.is_empty() && std::path::Path::new(&format!("{dir}/ls-R")).exists() {
        trees.push(dir);
      }
    }
  }
  trees
}

fn font_index() -> Rc<FontIndex> {
  FONT_INDEX.with(|slot| {
    if let Some(idx) = slot.borrow().as_ref() {
      return Rc::clone(idx);
    }
    let mut files = FxHashMap::default();
    for tree in texmf_trees() {
      let Ok(text) = std::fs::read_to_string(format!("{tree}/ls-R")) else {
        continue;
      };
      let mut dir = String::new();
      for line in text.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('%') {
          continue;
        }
        if let Some(d) = l.strip_suffix(':') {
          dir = format!("{tree}/{}", d.trim_start_matches("./"));
          continue;
        }
        let lower = l.to_ascii_lowercase();
        if lower.ends_with(".otf")
          || lower.ends_with(".ttf")
          || lower.ends_with(".ttc")
          || lower.ends_with(".tfm")
        {
          files.entry(lower).or_insert_with(|| format!("{dir}/{l}"));
        }
      }
    }
    let idx = Rc::new(FontIndex { files });
    *slot.borrow_mut() = Some(Rc::clone(&idx));
    idx
  })
}

/// Full path of a font file given its basename (any case), from ls-R.
pub fn font_file_path(basename: &str) -> Option<String> {
  font_index()
    .files
    .get(&basename.to_ascii_lowercase())
    .cloned()
}

fn squash(s: &str) -> String {
  s.chars()
    .filter(|c| c.is_ascii_alphanumeric())
    .map(|c| c.to_ascii_lowercase())
    .collect()
}

/// Resolve a fontspec font specification (family name or file name,
/// optionally bracketed as `[file.otf]`) to a font file on disk.
pub fn resolve_fontspec_file(spec: &str) -> Option<String> {
  let name = spec
    .trim()
    .trim_start_matches('[')
    .trim_end_matches(']')
    .trim();
  // fontspec's `Name/B`, `Name/I` style modifiers select faces of the same family.
  let name = name.split('/').next().unwrap_or(name).trim();
  if name.is_empty() {
    return None;
  }
  let lower = name.to_ascii_lowercase();
  if [".otf", ".ttf", ".ttc"].iter().any(|e| lower.ends_with(e)) {
    return font_file_path(name);
  }
  let idx = font_index();
  // (1) file-stem match: "TeX Gyre Pagella" → texgyrepagella-regular.otf.
  let key = squash(name);
  let mut best: Option<(u8, &String, &String)> = None;
  for (base, path) in &idx.files {
    if base.ends_with(".tfm") {
      continue;
    }
    let stem = squash(base.rsplit_once('.').map(|(s, _)| s).unwrap_or(base));
    let rank = if stem == key {
      0
    } else if stem == format!("{key}regular") || stem == format!("{key}r") {
      1
    } else if stem == format!("{key}book") || stem == format!("{key}roman") {
      2
    } else {
      continue;
    };
    if best.is_none_or(|(r, b, _)| rank < r || (rank == r && base.len() < b.len())) {
      best = Some((rank, base, path));
    }
  }
  if let Some((_, _, path)) = best {
    return Some(path.clone());
  }
  // (2) OpenType `name` table match through the family index.
  let families = family_index();
  let mut best: Option<(u8, &String)> = None;
  if let Some(entries) = families.get(&key) {
    for (path, regular) in entries {
      let rank = if *regular { 0 } else { 1 };
      if best.is_none_or(|(r, b)| rank < r || (rank == r && path.len() < b.len())) {
        best = Some((rank, path));
      }
    }
  }
  best.map(|(_, p)| p.clone())
}

/// squashed family name → `(path, is_regular_face)` entries.
type FamilyIndex = FxHashMap<String, Vec<(String, bool)>>;

thread_local! {
  static FAMILY_INDEX: RefCell<Option<Rc<FamilyIndex>>> = const { RefCell::new(None) };
}

/// squashed family name → `(path, is_regular_face)` for every OpenType/TrueType
/// font in the tree; built on first use from each file's `name` table.
fn family_index() -> Rc<FamilyIndex> {
  FAMILY_INDEX.with(|slot| {
    if let Some(idx) = slot.borrow().as_ref() {
      return Rc::clone(idx);
    }
    let mut map: FamilyIndex = FxHashMap::default();
    for (base, path) in &font_index().files {
      if base.ends_with(".tfm") {
        continue;
      }
      let Some((families, subfamilies)) = sfnt_names_at(path) else {
        continue;
      };
      let regular = subfamilies.iter().any(|s| {
        matches!(
          s.to_ascii_lowercase().as_str(),
          "regular" | "book" | "roman" | "normal"
        )
      });
      let mut seen: Vec<String> = Vec::new();
      for f in families {
        let k = squash(&f);
        if k.is_empty() || seen.contains(&k) {
          continue;
        }
        seen.push(k.clone());
        map.entry(k).or_default().push((path.clone(), regular));
      }
    }
    let idx = Rc::new(map);
    *slot.borrow_mut() = Some(Rc::clone(&idx));
    idx
  })
}

/// The `name` table of the font file at `path`, read with two small reads
/// (table directory, then the table) rather than the whole file.
fn sfnt_names_at(path: &str) -> Option<(Vec<String>, Vec<String>)> {
  use std::io::{Read, Seek, SeekFrom};
  let mut f = std::fs::File::open(path).ok()?;
  let mut head = [0u8; 16];
  f.read_exact(&mut head).ok()?;
  let base = if &head[0..4] == b"ttcf" {
    rd32(&head, 12)? as u64
  } else {
    0
  };
  let mut dir_head = [0u8; 12];
  f.seek(SeekFrom::Start(base)).ok()?;
  f.read_exact(&mut dir_head).ok()?;
  let num_tables = rd16(&dir_head, 4)? as usize;
  if num_tables == 0 || num_tables > 64 {
    return None;
  }
  let mut dir = vec![0u8; 16 * num_tables];
  f.read_exact(&mut dir).ok()?;
  for i in 0..num_tables {
    let rec = 16 * i;
    if &dir[rec..rec + 4] == b"name" {
      let off = rd32(&dir, rec + 8)? as u64;
      let len = rd32(&dir, rec + 12)? as usize;
      if len > 1 << 20 {
        return None;
      }
      let mut table = vec![0u8; len];
      f.seek(SeekFrom::Start(off)).ok()?;
      f.read_exact(&mut table).ok()?;
      return parse_name_table(&table);
    }
  }
  None
}

/// Family (name IDs 1, 16) and subfamily (2, 17) strings of an sfnt font.
#[cfg(test)]
fn sfnt_names(b: &[u8]) -> Option<(Vec<String>, Vec<String>)> {
  let (off, len) = sfnt_table(b, b"name")?;
  parse_name_table(&b[off..off + len])
}

/// Family (1, 16) and subfamily (2, 17) strings of a `name` table.
fn parse_name_table(t: &[u8]) -> Option<(Vec<String>, Vec<String>)> {
  let count = rd16(t, 2)? as usize;
  let strings = rd16(t, 4)? as usize;
  let mut families = Vec::new();
  let mut subfamilies = Vec::new();
  for i in 0..count {
    let rec = 6 + 12 * i;
    let platform = rd16(t, rec)?;
    let name_id = rd16(t, rec + 6)?;
    let slen = rd16(t, rec + 8)? as usize;
    let soff = rd16(t, rec + 10)? as usize;
    if !matches!(name_id, 1 | 2 | 16 | 17) {
      continue;
    }
    let Some(raw) = t.get(strings + soff..strings + soff + slen) else {
      continue;
    };
    let text = if platform == 1 {
      raw.iter().map(|&c| c as char).collect::<String>()
    } else {
      let units: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_be_bytes(*p))
        .collect();
      String::from_utf16_lossy(&units)
    };
    if matches!(name_id, 1 | 16) {
      families.push(text)
    } else {
      subfamilies.push(text)
    }
  }
  Some((families, subfamilies))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tfm_coverage_of_cmr10_is_the_128_slot_ot1_font() {
    let Some(path) = font_file_path("cmr10.tfm") else {
      return;
    }; // no TeX Live: self-skip
    let cov = coverage_for_file(&path).expect("cmr10.tfm parses");
    assert_eq!(cov.len(), 128);
    assert!(cov.contains(b'A' as u32));
    assert!(!cov.contains(200));
  }

  #[test]
  fn opentype_cmap_bounds_coverage_to_real_glyphs() {
    let Some(path) = resolve_fontspec_file("Latin Modern Sans") else {
      return;
    };
    let cov = coverage_for_file(&path).expect("otf parses");
    assert!(cov.contains(b'A' as u32));
    assert!(!cov.contains(0x0000), "control block is not covered");
    assert!(!cov.contains(0x0080));
    assert!(
      cov.len() > 200 && cov.len() < 2000,
      "lmsans10-regular has ~800 glyphs: {}",
      cov.len()
    );
  }

  #[test]
  fn name_table_reader_matches_whole_file_parse() {
    let Some(path) = font_file_path("lmsans10-regular.otf") else {
      return;
    };
    let whole = std::fs::read(&path)
      .map(|b| sfnt_names(&b).expect("names"))
      .expect("read");
    let partial = sfnt_names_at(&path).expect("partial names");
    assert_eq!(whole, partial);
    assert!(
      partial.0.iter().any(|f| f == "Latin Modern Sans"),
      "{partial:?}"
    );
  }

  #[test]
  fn fontspec_file_names_and_family_names_resolve() {
    if font_file_path("cmr10.tfm").is_none() {
      return;
    }
    let direct = resolve_fontspec_file("NewCMMath-Regular.otf");
    assert!(direct.is_some_and(|p| p.ends_with("NewCMMath-Regular.otf")));
    let stem = resolve_fontspec_file("TeX Gyre Pagella");
    assert!(
      stem
        .as_deref()
        .is_some_and(|p| p.to_ascii_lowercase().contains("texgyrepagella-regular")),
      "{stem:?}"
    );
  }
}
