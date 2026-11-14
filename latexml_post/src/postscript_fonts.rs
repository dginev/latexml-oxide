//! The fonts a MetaPost figure leaves to dvips.
//!
//! MetaPost writes PostScript with `prologues:=0` (its default) that is not self-contained: the text is
//! `(S) cmmi10 9.96265 fshow`, the fonts are named by their TeX names, and the file only lists them in `%*Font:`
//! header comments. dvips reads those comments when it includes the figure, downloads the fonts and supplies `fshow`
//! (`finclude.pro`), which is how arXiv's PDF build renders them; Ghostscript alone stops at `/undefined in cmmi10`, so
//! the figure is missing (Perl LaTeXML's ImageMagick path fails the same way). Before converting such a file we do
//! what dvips does: a copy gets, in its prolog, each listed font's Type1 program (found through TeX Live's
//! `pdftex.map`), the TeX name bound to the font, and MetaPost's own `fshow` (what it writes with `prologues:=1`).
//! OXIDIZED_DESIGN #587; witnesses gr-qc0001079 (`foliation.ps`), math0102124 (`sch.13`).
use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::OnceLock,
};

/// The header comment dvips reads for a font a figure uses: `%*Font: <texname> <size> <designsize> <chars>`.
const FONT_COMMENT: &str = "%*Font:";

/// How much of a PostScript file is read to look for `%*Font:` comments.
const HEADER_BYTES: u64 = 64 * 1024;

/// MetaPost's `fshow` for `prologues:=1` output (string, font name, size).
const FSHOW: &str = "/fshow {exch findfont exch scalefont setfont show} bind def\n";

/// A TeX font's PostScript font name and its Type1 file, from `pdftex.map`.
struct MappedFont {
  ps_name: String,
  file:    String,
}

/// `pdftex.map`'s plain entries (`cmmi10 CMMI10 <cmmi10.pfb`), read once. Entries that re-encode or slant a font are
/// left out: their glyphs are not at the TeX positions MetaPost's strings use without the encoding applied.
fn font_map() -> &'static HashMap<String, MappedFont> {
  static MAP: OnceLock<HashMap<String, MappedFont>> = OnceLock::new();
  MAP.get_or_init(|| {
    latexml_core::util::pathname::kpsewhich(&["pdftex.map"])
      .and_then(|path| std::fs::read_to_string(path).ok())
      .map(|text| parse_font_map(&text))
      .unwrap_or_default()
  })
}

fn parse_font_map(text: &str) -> HashMap<String, MappedFont> {
  let mut map = HashMap::new();
  for line in text.lines() {
    let line = line.trim();
    if line.is_empty()
      || line.starts_with(['%', '#'])
      || line.contains('"')
      || line.contains(".enc")
    {
      continue;
    }
    let mut fields = line.split_whitespace();
    let (Some(tex_name), Some(ps_name)) = (fields.next(), fields.next()) else {
      continue;
    };
    let file = fields
      .filter_map(|f| {
        f.trim_start_matches(['<', '['])
          .strip_suffix(".pfb")
          .map(|stem| format!("{stem}.pfb"))
      })
      .next();
    if let Some(file) = file
      && !ps_name.starts_with('<')
    {
      map.entry(tex_name.to_string()).or_insert(MappedFont {
        ps_name: ps_name.to_string(),
        file,
      });
    }
  }
  map
}

/// The TeX names in a PostScript file's `%*Font:` header comments, in order, each once.
fn declared_fonts(header: &str) -> Vec<String> {
  let mut names: Vec<String> = Vec::new();
  for line in header.lines() {
    if let Some(rest) = line.strip_prefix(FONT_COMMENT)
      && let Some(name) = rest.split_whitespace().next()
      && !names.iter().any(|n| n == name)
    {
      names.push(name.to_string());
    }
  }
  names
}

/// A `.pfb` font program as the PFA text PostScript reads inline: ASCII segments as they are, binary (eexec)
/// segments as hex (PFB segment headers: 0x80, type 1 ASCII / 2 binary / 3 end, 4-byte little-endian length).
fn pfb_to_pfa(bytes: &[u8]) -> Option<String> {
  let mut out = String::new();
  let mut at = 0;
  while at < bytes.len() {
    if bytes[at] != 0x80 || at + 2 > bytes.len() {
      return None;
    }
    let kind = bytes[at + 1];
    if kind == 3 {
      break;
    }
    let len_bytes = bytes.get(at + 2..at + 6)?;
    let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
    let segment = bytes.get(at + 6..at + 6 + len)?;
    match kind {
      1 => out.push_str(&String::from_utf8_lossy(segment).replace('\r', "\n")),
      2 => {
        for (i, b) in segment.iter().enumerate() {
          out.push_str(&format!("{b:02x}"));
          if i % 32 == 31 {
            out.push('\n');
          }
        }
        out.push('\n');
      },
      _ => return None,
    }
    at += 6 + len;
  }
  Some(out)
}

/// The prolog dvips would give a figure that uses `fonts`: each font's program, its TeX name bound to its PostScript
/// name, and `fshow` unless the file defines its own. `None` when a font cannot be found — the figure would still fail.
fn download_prolog(fonts: &[String], defines_fshow: bool) -> Option<String> {
  let map = font_map();
  let mut prolog = String::from("%%BeginProcSet: latexml-dvips-fonts\n");
  for name in fonts {
    let mapped = map.get(name)?;
    let path = latexml_core::util::pathname::kpsewhich(&[mapped.file.as_str()])?;
    prolog.push_str(&pfb_to_pfa(&std::fs::read(path).ok()?)?);
    prolog.push_str(&format!("/{name} /{} def\n", mapped.ps_name));
  }
  if !defines_fshow {
    prolog.push_str(FSHOW);
  }
  prolog.push_str("%%EndProcSet\n");
  Some(prolog)
}

/// A converted figure's temporary copy, removed when dropped — on every return and on panic unwinding (the canvas
/// `catch_unwind`). A `panic = "abort"` build (`maxperf`) or a process killed by a timeout runs no drop and leaves the
/// copy behind.
pub(crate) struct FontCopy(PathBuf);

impl FontCopy {
  pub(crate) fn path(&self) -> &Path { &self.0 }
}

impl Drop for FontCopy {
  fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); }
}

/// A copy of the PostScript file `source`, beside `dest`, holding the fonts its `%*Font:` comments ask dvips for —
/// `None` when it declares none or one is not available. The caller converts the copy; dropping it removes it.
pub(crate) fn with_downloaded_fonts(source: &str, dest: &str) -> Option<FontCopy> {
  // Only the header is read for the check; a figure that declares fonts is read whole.
  let mut head = Vec::new();
  std::io::Read::read_to_end(
    &mut std::io::Read::take(std::fs::File::open(source).ok()?, HEADER_BYTES),
    &mut head,
  )
  .ok()?;
  if !head.starts_with(b"%!")
    || !head
      .windows(FONT_COMMENT.len())
      .any(|w| w == FONT_COMMENT.as_bytes())
  {
    return None;
  }
  let bytes = std::fs::read(source).ok()?;
  // The header comments end at `%%EndProlog` or at the first line that is not a comment.
  let mut header_end = 0;
  let mut insert_at = None;
  for line in bytes.split_inclusive(|b| *b == b'\n') {
    if line.starts_with(b"%%EndProlog") {
      insert_at = Some(header_end);
      break;
    }
    if !line.starts_with(b"%") {
      break;
    }
    header_end += line.len();
  }
  let insert_at = insert_at.unwrap_or(header_end);
  let header = String::from_utf8_lossy(&bytes[..insert_at]);
  let fonts = declared_fonts(&header);
  if fonts.is_empty() {
    return None;
  }
  let defines_fshow = bytes.windows(6).any(|w| w == b"/fshow");
  let prolog = download_prolog(&fonts, defines_fshow)?;
  let parent = Path::new(dest).parent().unwrap_or_else(|| Path::new("."));
  let stem = Path::new(dest)
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("figure");
  let unique = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
  let copy = parent.join(format!(".{stem}.{unique}.fonts.eps"));
  let mut out = Vec::with_capacity(bytes.len() + prolog.len());
  out.extend_from_slice(&bytes[..insert_at]);
  out.extend_from_slice(prolog.as_bytes());
  out.extend_from_slice(&bytes[insert_at..]);
  let copy = FontCopy(copy);
  std::fs::write(copy.path(), out).ok()?;
  Some(copy)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn declared_fonts_read_from_the_header_comments() {
    let header = "%!PS\n%%BoundingBox: 0 0 10 10\n%*Font: cmmi10 9.96265 9.96265 53:a\n%*Font: cmr5 4.98 4.98 31:c\n\
                  %*Font: cmmi10 6.97 9.96 1c:8\n";
    assert_eq!(declared_fonts(header), vec!["cmmi10", "cmr5"]);
  }

  #[test]
  fn plain_map_entries_only() {
    let map = parse_font_map(
      "cmmi10 CMMI10 <cmmi10.pfb\nptmr8r NimbusRomNo9L-Regu \" TeXBase1Encoding ReEncodeFont \" <8r.enc <utmr8a.pfb\n\
       % comment\nmsam10 MSAM10 <msam10.pfb\n",
    );
    assert_eq!(map.len(), 2);
    assert_eq!(map["cmmi10"].ps_name, "CMMI10");
    assert_eq!(map["msam10"].file, "msam10.pfb");
    assert!(!map.contains_key("ptmr8r"));
  }

  #[test]
  fn pfb_segments_become_pfa_text() {
    let mut pfb = vec![0x80, 1, 4, 0, 0, 0];
    pfb.extend_from_slice(b"%!a\n");
    pfb.extend_from_slice(&[0x80, 2, 2, 0, 0, 0, 0xab, 0x01]);
    pfb.extend_from_slice(&[0x80, 3]);
    assert_eq!(pfb_to_pfa(&pfb).as_deref(), Some("%!a\nab01\n"));
  }
  /// A MetaPost figure (`prologues:=0`, gr-qc0001079 `foliation.ps`) gets, before its `%%EndProlog`, the program of each
  /// font its `%*Font:` comments list, its TeX name bound to the font, and `fshow`; the original stays untouched.
  #[test]
  fn metapost_figure_gets_the_fonts_dvips_would_download() {
    let dir = std::env::temp_dir().join(format!("lx_mpfonts_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("fig.1");
    let figure = "%!PS\n%%BoundingBox: -1 -1 20 12\n%%Creator: MetaPost\n%%Pages: 1\n\
                  %*Font: cmmi10 9.96265 9.96265 53:a\n%*Font: cmr5 4.98132 4.98132 31:c\n%%EndProlog\n%%Page: 1 1\n\
                  0 0 moveto\n(S) cmmi10 9.96265 fshow\n(1) cmr5 4.98132 fshow\nshowpage\n%%EOF\n";
    std::fs::write(&source, figure).unwrap();
    let dest = dir.join("fig.png");
    let copy = with_downloaded_fonts(source.to_str().unwrap(), dest.to_str().unwrap())
      .expect("cmmi10 and cmr5 are found through pdftex.map");
    let text = String::from_utf8_lossy(&std::fs::read(copy.path()).unwrap()).into_owned();
    let prolog_at = text.find("%%BeginProcSet: latexml-dvips-fonts").unwrap();
    let end_prolog_at = text.find("%%EndProlog").unwrap();
    assert!(
      text.starts_with("%!PS\n%%BoundingBox: -1 -1 20 12\n"),
      "{text}"
    );
    assert!(prolog_at < end_prolog_at, "{text}");
    assert_eq!(text.matches("currentfile eexec").count(), 2);
    assert!(text.contains("/FontName /CMMI10 def"));
    assert!(text.contains("/cmmi10 /CMMI10 def\n"));
    assert!(text.contains("/cmr5 /CMR5 def\n"));
    assert!(text.contains(FSHOW));
    assert!(text.ends_with("(S) cmmi10 9.96265 fshow\n(1) cmr5 4.98132 fshow\nshowpage\n%%EOF\n"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), figure);
    let copy_path = copy.path().to_path_buf();
    drop(copy);
    assert!(!copy_path.exists());
    // A figure with no `%*Font:` comments, or one that is not PostScript, is converted as it is.
    std::fs::write(
      &source,
      "%!PS\n%%BoundingBox: 0 0 1 1\n%%EndProlog\nshowpage\n",
    )
    .unwrap();
    assert!(with_downloaded_fonts(source.to_str().unwrap(), dest.to_str().unwrap()).is_none());
    std::fs::write(
      &source,
      "\\documentstyle{article}\n%*Font: cmmi10 9.96 9.96 53:a\n",
    )
    .unwrap();
    assert!(with_downloaded_fonts(source.to_str().unwrap(), dest.to_str().unwrap()).is_none());
    let _ = std::fs::remove_dir_all(&dir);
  }
}
