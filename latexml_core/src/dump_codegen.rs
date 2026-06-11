//! Generate Rust source code from a kernel dump file.
//!
//! Reads the text dump produced by `dump_writer.rs` and emits a `.rs` file
//! containing typed static data tables. The compiler checks every entry at
//! build time — no runtime parsing, no text format errors possible.
//!
//! Usage:
//!   1. Generate dump: `latexml_oxide --init=latex.ltx --dest=/tmp/dump.tmp`
//!   2. Generate Rust: `dump_codegen::generate_rs("/tmp/dump.tmp", "latex_dump.rs")`
//!   3. Place in `latexml_package/src/engine/latex_dump.rs`
//!   4. Load at runtime via `latex_dump::load_definitions()`

use std::{io::Write, path::Path};

/// Value entries to skip (runtime-specific or cause regressions).
const SKIP_VALUES: &[&str] = &[
  "INTERPRETING_DEFINITIONS",
  "if_count",
  "absorb_count",
  "_loaded",
  "INCLUDE_COMMENTS",
  "INCLUDE_STYLES",
  "\\everyjob",
  "\\toks",
  "input_file:",
  "output_file:",
  "texsys",
];

fn should_skip_value(key: &str) -> bool {
  SKIP_VALUES
    .iter()
    .any(|skip| key == *skip || key.starts_with(skip) || key.ends_with(skip) || key.contains(skip))
}

/// Escape a string for use as a Rust string literal.
fn rust_escape(s: &str) -> String {
  if s.contains("\"#")
    || s.contains('\r')
    || s
      .chars()
      .any(|c| c.is_ascii_control() && c != '\n' && c != '\t')
  {
    let mut out = String::with_capacity(s.len() + 8);
    out.push('"');
    for ch in s.chars() {
      match ch {
        '\\' => out.push_str("\\\\"),
        '"' => out.push_str("\\\""),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        c if c.is_ascii_control() => out.push_str(&format!("\\x{:02x}", c as u8)),
        c => out.push(c),
      }
    }
    out.push('"');
    out
  } else {
    format!("r#\"{}\"#", s)
  }
}

fn url_decode(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars();
  while let Some(ch) = chars.next() {
    if ch == '%' {
      let hex: String = chars.by_ref().take(2).collect();
      if let Ok(byte) = u8::from_str_radix(&hex, 16) {
        result.push(byte as char);
      }
    } else {
      result.push(ch);
    }
  }
  result
}

// ---- Parsed entry types ----

struct BoolEntry {
  key: String,
  val: bool,
}
struct IntEntry {
  key: String,
  val: i64,
}
struct StrEntry {
  key: String,
  val: String,
}
struct DimEntry {
  key: String,
  val: i64,
}
struct GlueEntry {
  key:   String,
  skip:  i64,
  plus:  Option<i64>,
  pfill: u8, // 0=None, 1=Fil, 2=Fill, 3=Filll
  minus: Option<i64>,
  mfill: u8,
}
struct CharcodeEntry {
  key: String,
  val: u16,
}
struct CatcodeEntry {
  key: String,
  val: u8,
}
struct TokenEntry {
  key:  String,
  cc:   u8,
  text: String,
}
struct TokenListEntry {
  key:       String,
  tok_start: u32,
  tok_count: u16,
}
struct ExpandEntry {
  cs:        String,
  nargs:     u8,
  flags:     u8, // bit 0=long, bit 1=protected
  tok_start: u32,
  tok_count: u16,
}
struct LetAliasEntry {
  key:     String, // e.g. "\\tex_let:D"
  target:  String, // e.g. "\\let"
  // Reserved for future use: when we start emitting MathPrimitive-specific
  // assignment calls, this flag distinguishes PA (ordinary Primitive alias)
  // from MPA (MathPrimitive alias). Today `state::let_i` handles both by
  // copying the target's Stored meaning, so the flag is advisory.
  #[allow(dead_code)]
  is_math: bool,
}
struct TokData {
  cc:   u8,
  text: String,
}

/// All categorized entries from a dump file.
#[derive(Default)]
struct DumpData {
  // Value table
  bools:         Vec<BoolEntry>,
  ints:          Vec<IntEntry>,
  strings:       Vec<StrEntry>,
  dims:          Vec<DimEntry>,
  glues:         Vec<GlueEntry>,
  mudims:        Vec<DimEntry>,
  muglues:       Vec<GlueEntry>,
  charcodes:     Vec<CharcodeEntry>,
  catcode_vals:  Vec<CatcodeEntry>,
  single_tokens: Vec<TokenEntry>,
  token_lists:   Vec<TokenListEntry>,
  nones:         Vec<String>,
  vecdeques:     Vec<String>,

  // Code tables
  catcodes:  Vec<CatcodeEntry>,
  lccodes:   Vec<CharcodeEntry>,
  uccodes:   Vec<CharcodeEntry>,
  sfcodes:   Vec<CharcodeEntry>,
  delcodes:  Vec<CharcodeEntry>,
  mathcodes: Vec<CharcodeEntry>,

  // Meaning table
  let_defs:    Vec<TokenEntry>,
  expandables: Vec<ExpandEntry>,
  let_aliases: Vec<LetAliasEntry>, // PA / MPA entries

  // Token pools (flattened)
  value_tokens:  Vec<TokData>,
  expand_tokens: Vec<TokData>,
}

impl DumpData {
  fn total_entries(&self) -> usize {
    self.bools.len()
      + self.ints.len()
      + self.strings.len()
      + self.dims.len()
      + self.glues.len()
      + self.mudims.len()
      + self.muglues.len()
      + self.charcodes.len()
      + self.catcode_vals.len()
      + self.single_tokens.len()
      + self.token_lists.len()
      + self.nones.len()
      + self.vecdeques.len()
      + self.catcodes.len()
      + self.lccodes.len()
      + self.uccodes.len()
      + self.sfcodes.len()
      + self.delcodes.len()
      + self.mathcodes.len()
      + self.let_defs.len()
      + self.expandables.len()
      + self.let_aliases.len()
  }
}

/// Parse dump content into categorized entries.
fn parse_dump(content: &str) -> DumpData {
  let mut data = DumpData::default();

  for line in content.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let parts: Vec<&str> = line.splitn(3, '\t').collect();
    if parts.len() < 2 {
      continue;
    }
    let table = parts[0];
    let key = url_decode(parts[1]);
    let rest = if parts.len() > 2 { parts[2] } else { "" };

    match table {
      "V" => parse_value(&mut data, &key, rest),
      "M" => parse_meaning(&mut data, &key, rest),
      "C" => {
        let val_parts: Vec<&str> = rest.splitn(2, '\t').collect();
        if val_parts.len() >= 2
          && let Ok(v) = val_parts[1].parse::<u8>()
        {
          data.catcodes.push(CatcodeEntry { key, val: v });
        }
      },
      "LC" => {
        let val_parts: Vec<&str> = rest.splitn(2, '\t').collect();
        if val_parts.len() >= 2
          && let Ok(v) = val_parts[1].parse::<u16>()
        {
          data.lccodes.push(CharcodeEntry { key, val: v });
        }
      },
      "UC" => {
        let val_parts: Vec<&str> = rest.splitn(2, '\t').collect();
        if val_parts.len() >= 2
          && let Ok(v) = val_parts[1].parse::<u16>()
        {
          data.uccodes.push(CharcodeEntry { key, val: v });
        }
      },
      "SC" => {
        let val_parts: Vec<&str> = rest.splitn(2, '\t').collect();
        if val_parts.len() >= 2
          && let Ok(v) = val_parts[1].parse::<u16>()
        {
          data.sfcodes.push(CharcodeEntry { key, val: v });
        }
      },
      "DC" => {
        let val_parts: Vec<&str> = rest.splitn(2, '\t').collect();
        if val_parts.len() >= 2
          && let Ok(v) = val_parts[1].parse::<u16>()
        {
          data.delcodes.push(CharcodeEntry { key, val: v });
        }
      },
      "MC" => {
        let val_parts: Vec<&str> = rest.splitn(2, '\t').collect();
        if val_parts.len() >= 2
          && let Ok(v) = val_parts[1].parse::<u16>()
        {
          data.mathcodes.push(CharcodeEntry { key, val: v });
        }
      },
      _ => {}, // Skip unknown tables
    }
  }

  data
}

fn parse_value(data: &mut DumpData, key: &str, rest: &str) {
  // Skip \ver@ and other runtime-specific entries
  if key.starts_with("\\ver@") || should_skip_value(key) {
    return;
  }
  let parts: Vec<&str> = rest.splitn(2, '\t').collect();
  if parts.is_empty() {
    return;
  }
  let val_data = parts.get(1).copied().unwrap_or("");

  match parts[0] {
    "N" => data.nones.push(key.to_string()),
    "B" => data.bools.push(BoolEntry {
      key: key.to_string(),
      val: val_data == "1",
    }),
    "I" => {
      if let Ok(v) = val_data.parse::<i64>() {
        data.ints.push(IntEntry { key: key.to_string(), val: v });
      }
    },
    "S" => data.strings.push(StrEntry {
      key: key.to_string(),
      val: url_decode(val_data),
    }),
    "CH" => {
      if let Ok(v) = val_data.parse::<u16>() {
        data
          .charcodes
          .push(CharcodeEntry { key: key.to_string(), val: v });
      }
    },
    "CC" => {
      if let Ok(v) = val_data.parse::<u8>() {
        data
          .catcode_vals
          .push(CatcodeEntry { key: key.to_string(), val: v });
      }
    },
    "D" => {
      if let Ok(v) = val_data.parse::<i64>() {
        data.dims.push(DimEntry { key: key.to_string(), val: v });
      }
    },
    "G" => {
      if let Some(g) = parse_glue_data(key, val_data) {
        data.glues.push(g);
      }
    },
    "MD" => {
      if let Ok(v) = val_data.parse::<i64>() {
        data.mudims.push(DimEntry { key: key.to_string(), val: v });
      }
    },
    "MG" => {
      if let Some(g) = parse_glue_data(key, val_data) {
        data.muglues.push(g);
      }
    },
    "T" => {
      if let Some((cc, text)) = parse_tok(val_data) {
        data
          .single_tokens
          .push(TokenEntry { key: key.to_string(), cc, text });
      }
    },
    "TK" => {
      if val_data.is_empty() {
        data.token_lists.push(TokenListEntry {
          key:       key.to_string(),
          tok_start: data.value_tokens.len() as u32,
          tok_count: 0,
        });
      } else {
        let start = data.value_tokens.len() as u32;
        let mut count = 0u16;
        for tok_s in val_data.split(',') {
          if let Some((cc, text)) = parse_tok(tok_s) {
            data.value_tokens.push(TokData { cc, text });
            count += 1;
          }
        }
        data.token_lists.push(TokenListEntry {
          key:       key.to_string(),
          tok_start: start,
          tok_count: count,
        });
      }
    },
    "VD" => data.vecdeques.push(key.to_string()),
    _ => {},
  }
}

fn parse_meaning(data: &mut DumpData, key: &str, rest: &str) {
  let parts: Vec<&str> = rest.splitn(2, '\t').collect();
  if parts.is_empty() {
    return;
  }
  match parts[0] {
    "N" => {}, // Skip None meanings
    "E" => {
      let eparts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(4, '\t').collect();
      if eparts.len() < 4 {
        return;
      }
      let nargs: u8 = eparts[1].parse().unwrap_or(0);
      let flags_str = eparts[2];
      let tok_data = eparts[3];

      let mut flags: u8 = 0;
      if flags_str.contains('L') {
        flags |= 1;
      }
      if flags_str.contains('P') {
        flags |= 2;
      }

      let start = data.expand_tokens.len() as u32;
      let mut count = 0u16;
      if !tok_data.is_empty() {
        for tok_s in tok_data.split(',') {
          if let Some((cc, text)) = parse_tok(tok_s) {
            data.expand_tokens.push(TokData { cc, text });
            count += 1;
          }
        }
      }
      data.expandables.push(ExpandEntry {
        cs: key.to_string(),
        nargs,
        flags,
        tok_start: start,
        tok_count: count,
      });
    },
    "T" => {
      let tok_s = parts.get(1).unwrap_or(&"");
      if let Some((cc, text)) = parse_tok(tok_s) {
        data
          .let_defs
          .push(TokenEntry { key: key.to_string(), cc, text });
      }
    },
    "PA" | "MPA" => {
      // Primitive alias: \let <key> = <target>.
      // Emit as a call to state::let_i at load time; target must already
      // be defined (guaranteed by the (d.2) early/late ordering).
      let target = url_decode(parts.get(1).unwrap_or(&""));
      if target.is_empty() || target == key {
        return; // self-alias or malformed: skip
      }
      data.let_aliases.push(LetAliasEntry {
        key: key.to_string(),
        target,
        is_math: parts[0] == "MPA",
      });
    },
    _ => {},
  }
}

fn parse_tok(s: &str) -> Option<(u8, String)> {
  let (cc_str, text) = s.split_once(':')?;
  let cc: u8 = cc_str.parse().ok()?;
  Some((cc, url_decode(text)))
}

fn parse_glue_data(key: &str, s: &str) -> Option<GlueEntry> {
  let mut skip = 0i64;
  let mut plus = None;
  let mut pfill = 0u8;
  let mut minus = None;
  let mut mfill = 0u8;

  for (i, part) in s.split(',').enumerate() {
    if i == 0 {
      skip = part.parse().ok()?;
    } else if let Some(rest) = part.strip_prefix("pf") {
      pfill = rest.parse().unwrap_or(0);
    } else if let Some(rest) = part.strip_prefix('p') {
      plus = Some(rest.parse().ok()?);
    } else if let Some(rest) = part.strip_prefix("mf") {
      mfill = rest.parse().unwrap_or(0);
    } else if let Some(rest) = part.strip_prefix('m') {
      minus = Some(rest.parse().ok()?);
    }
  }

  Some(GlueEntry {
    key: key.to_string(),
    skip,
    plus,
    pfill,
    minus,
    mfill,
  })
}

// ---- Code generation ----

/// Generate a Rust source file with typed static data tables from a text dump.
/// Every entry is compiler-checked. Zero runtime parsing needed.
pub fn generate_rs(dump_path: &Path, output_path: &Path) -> Result<usize, String> {
  let content =
    std::fs::read_to_string(dump_path).map_err(|e| format!("Failed to read dump: {}", e))?;

  let data = parse_dump(&content);
  let total = data.total_entries();

  let mut out =
    std::fs::File::create(output_path).map_err(|e| format!("Failed to create output: {}", e))?;

  // Header
  writeln!(
    out,
    "//! Auto-generated kernel dump — static data tables.\n\
     //! DO NOT EDIT — regenerate with: `cargo run --release --bin latexml_oxide -- --init=latex.ltx`\n\
     //!\n\
     //! {total} entries, compiler-checked at build time. Zero runtime parsing.\n\
     #![allow(unused)]\n\
     #![allow(clippy::type_complexity)]\n"
  )
  .map_err(we)?;

  // Imports
  writeln!(
    out,
    "use std::collections::VecDeque;\n\
     \n\
     use latexml_core::common::arena;\n\
     use latexml_core::common::def_parser::parse_parameters;\n\
     use latexml_core::common::dimension::Dimension;\n\
     use latexml_core::common::glue::{{FillCode, Glue}};\n\
     use latexml_core::common::mudimension::MuDimension;\n\
     use latexml_core::common::muglue::MuGlue;\n\
     use latexml_core::common::store::Stored;\n\
     use latexml_core::definition::expandable::{{Expandable, ExpandableOptions}};\n\
     use latexml_core::state;\n\
     use latexml_core::state::Scope;\n\
     use latexml_core::token::{{Catcode, Token}};\n\
     use latexml_core::tokens::Tokens;\n"
  )
  .map_err(we)?;

  // Local struct for expandable definitions
  writeln!(
    out,
    "struct ExpandDef {{\n\
     {s}cs: &'static str,\n\
     {s}nargs: u8,\n\
     {s}flags: u8, // bit 0 = long, bit 1 = protected\n\
     {s}tok_start: u32,\n\
     {s}tok_count: u16,\n\
     }}\n",
    s = "  "
  )
  .map_err(we)?;

  // Emit static arrays
  emit_bool_array(&mut out, "BOOL_VALUES", &data.bools)?;
  emit_int_array(&mut out, "INT_VALUES", &data.ints)?;
  emit_str_array(&mut out, "STRING_VALUES", &data.strings)?;
  emit_dim_array(&mut out, "DIMENSION_VALUES", &data.dims)?;
  emit_glue_array(&mut out, "GLUE_VALUES", &data.glues)?;
  emit_dim_array(&mut out, "MUDIM_VALUES", &data.mudims)?;
  emit_glue_array(&mut out, "MUGLUE_VALUES", &data.muglues)?;
  emit_charcode_array(&mut out, "CHARCODE_VALUES", &data.charcodes)?;
  emit_catcode_array(&mut out, "CATCODE_VALUE_ENTRIES", &data.catcode_vals)?;
  emit_token_array(&mut out, "SINGLE_TOKEN_VALUES", &data.single_tokens)?;
  emit_none_array(&mut out, "NONE_VALUES", &data.nones)?;
  emit_none_array(&mut out, "VECDEQUE_VALUES", &data.vecdeques)?;

  // Token list values (V/TK) — entries reference VALUE_TOKENS pool
  emit_token_list_array(&mut out, "TOKEN_LIST_VALUES", &data.token_lists)?;
  emit_tok_pool(&mut out, "VALUE_TOKENS", &data.value_tokens)?;

  // Code tables
  emit_catcode_array(&mut out, "CATCODE_TABLE", &data.catcodes)?;
  emit_charcode_array(&mut out, "LCCODE_TABLE", &data.lccodes)?;
  emit_charcode_array(&mut out, "UCCODE_TABLE", &data.uccodes)?;
  emit_charcode_array(&mut out, "SFCODE_TABLE", &data.sfcodes)?;
  emit_charcode_array(&mut out, "DELCODE_TABLE", &data.delcodes)?;
  emit_charcode_array(&mut out, "MATHCODE_TABLE", &data.mathcodes)?;

  // Meaning table
  emit_token_array(&mut out, "LET_DEFINITIONS", &data.let_defs)?;
  emit_expand_array(&mut out, "EXPANDABLE_DEFS", &data.expandables)?;
  emit_tok_pool(&mut out, "EXPAND_TOKENS", &data.expand_tokens)?;
  emit_let_alias_array(&mut out, "LET_ALIAS_DEFS", &data.let_aliases)?;

  // load_definitions function
  emit_load_fn(&mut out, &data)?;

  Info!(
    "dump_codegen",
    "generated",
    s!(
      "Generated {} entries to {} (static data tables)",
      total,
      output_path.display()
    )
  );

  Ok(total)
}

fn we(e: std::io::Error) -> String { format!("Write error: {}", e) }

fn emit_bool_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[BoolEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, bool)] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  ({}, {}),", rust_escape(&e.key), e.val).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_int_array(out: &mut std::fs::File, name: &str, entries: &[IntEntry]) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, i64)] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  ({}, {}),", rust_escape(&e.key), e.val).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_str_array(out: &mut std::fs::File, name: &str, entries: &[StrEntry]) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, &str)] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  ({}, {}),", rust_escape(&e.key), rust_escape(&e.val)).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_dim_array(out: &mut std::fs::File, name: &str, entries: &[DimEntry]) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, i64)] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  ({}, {}),", rust_escape(&e.key), e.val).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_glue_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[GlueEntry],
) -> Result<(), String> {
  // (key, skip, plus, pfill, minus, mfill)
  writeln!(
    out,
    "static {name}: &[(&str, i64, Option<i64>, u8, Option<i64>, u8)] = &["
  )
  .map_err(we)?;
  for e in entries {
    let plus_s = match e.plus {
      Some(v) => format!("Some({v})"),
      None => "None".to_string(),
    };
    let minus_s = match e.minus {
      Some(v) => format!("Some({v})"),
      None => "None".to_string(),
    };
    writeln!(
      out,
      "  ({}, {}, {}, {}, {}, {}),",
      rust_escape(&e.key),
      e.skip,
      plus_s,
      e.pfill,
      minus_s,
      e.mfill
    )
    .map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_charcode_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[CharcodeEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, u16)] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  ({}, {}),", rust_escape(&e.key), e.val).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_catcode_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[CatcodeEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, u8)] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  ({}, {}),", rust_escape(&e.key), e.val).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_token_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[TokenEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, u8, &str)] = &[").map_err(we)?;
  for e in entries {
    writeln!(
      out,
      "  ({}, {}, {}),",
      rust_escape(&e.key),
      e.cc,
      rust_escape(&e.text)
    )
    .map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

/// Emit a static array of (key, target) pairs for primitive let-aliases.
/// The `is_math` field on `LetAliasEntry` is currently advisory only —
/// `state::let_i` installs whatever meaning the target token has, so the
/// runtime distinction between Primitive and MathPrimitive is handled by
/// `Stored` unification rather than the loader.
fn emit_let_alias_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[LetAliasEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, &str)] = &[").map_err(we)?;
  for e in entries {
    writeln!(
      out,
      "  ({}, {}),",
      rust_escape(&e.key),
      rust_escape(&e.target)
    )
    .map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_none_array(out: &mut std::fs::File, name: &str, entries: &[String]) -> Result<(), String> {
  writeln!(out, "static {name}: &[&str] = &[").map_err(we)?;
  for e in entries {
    writeln!(out, "  {},", rust_escape(e)).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_token_list_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[TokenListEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[(&str, u32, u16)] = &[").map_err(we)?;
  for e in entries {
    writeln!(
      out,
      "  ({}, {}, {}),",
      rust_escape(&e.key),
      e.tok_start,
      e.tok_count
    )
    .map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_tok_pool(out: &mut std::fs::File, name: &str, tokens: &[TokData]) -> Result<(), String> {
  writeln!(out, "static {name}: &[(u8, &str)] = &[").map_err(we)?;
  for t in tokens {
    writeln!(out, "  ({}, {}),", t.cc, rust_escape(&t.text)).map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

fn emit_expand_array(
  out: &mut std::fs::File,
  name: &str,
  entries: &[ExpandEntry],
) -> Result<(), String> {
  writeln!(out, "static {name}: &[ExpandDef] = &[").map_err(we)?;
  for e in entries {
    writeln!(
      out,
      "  ExpandDef {{ cs: {}, nargs: {}, flags: {}, tok_start: {}, tok_count: {} }},",
      rust_escape(&e.cs),
      e.nargs,
      e.flags,
      e.tok_start,
      e.tok_count
    )
    .map_err(we)?;
  }
  writeln!(out, "];\n").map_err(we)
}

/// Emit the load_definitions() function that iterates over all static arrays.
fn emit_load_fn(out: &mut std::fs::File, data: &DumpData) -> Result<(), String> {
  let s = "  ";
  writeln!(
    out,
    "/// Load the precompiled kernel definitions into the global state.\n\
     /// Perl equivalent: LoadFormat → LoadPool(format_dump)\n\
     pub fn load_definitions() -> latexml_core::common::error::Result<()> {{"
  )
  .map_err(we)?;

  // Bool values
  if !data.bools.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in BOOL_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::Bool(val), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Int values
  if !data.ints.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in INT_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::Int(val), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // String values
  if !data.strings.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in STRING_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::from(val), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Dimension values
  if !data.dims.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in DIMENSION_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::Dimension(Dimension(val)), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Glue values
  if !data.glues.is_empty() {
    writeln!(
      out,
      "{s}for &(key, skip, plus, pfill, minus, mfill) in GLUE_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::Glue(Glue {{\n\
       {s}{s}{s}skip, plus, pfill: FillCode::new(pfill as usize),\n\
       {s}{s}{s}minus, mfill: FillCode::new(mfill as usize),\n\
       {s}{s}}}), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // MuDimension values
  if !data.mudims.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in MUDIM_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::MuDimension(MuDimension(val)), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // MuGlue values
  if !data.muglues.is_empty() {
    writeln!(
      out,
      "{s}for &(key, skip, plus, pfill, minus, mfill) in MUGLUE_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::MuGlue(MuGlue {{\n\
       {s}{s}{s}skip, plus, pfill: FillCode::new(pfill as usize),\n\
       {s}{s}{s}minus, mfill: FillCode::new(mfill as usize),\n\
       {s}{s}}}), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Charcode values
  if !data.charcodes.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in CHARCODE_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::Charcode(val), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Catcode values (in value table, not catcode table)
  if !data.catcode_vals.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in CATCODE_VALUE_ENTRIES {{\n\
       {s}{s}state::assign_value(key, Stored::Catcode(Catcode::from(val)), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Single token values
  if !data.single_tokens.is_empty() {
    writeln!(
      out,
      "{s}for &(key, cc, text) in SINGLE_TOKEN_VALUES {{\n\
       {s}{s}let tok = Token {{ text: arena::pin(text), code: Catcode::from(cc) }};\n\
       {s}{s}state::assign_value(key, Stored::Token(tok), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Token list values
  if !data.token_lists.is_empty() {
    writeln!(
      out,
      "{s}for &(key, tok_start, tok_count) in TOKEN_LIST_VALUES {{\n\
       {s}{s}let toks: Vec<Token> = VALUE_TOKENS[tok_start as usize..][..tok_count as usize]\n\
       {s}{s}{s}.iter()\n\
       {s}{s}{s}.map(|&(cc, text)| Token {{ text: arena::pin(text), code: Catcode::from(cc) }})\n\
       {s}{s}{s}.collect();\n\
       {s}{s}state::assign_value(key, Stored::Tokens(Tokens::from(toks)), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // None values
  if !data.nones.is_empty() {
    writeln!(
      out,
      "{s}for &key in NONE_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::None, Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // VecDeque values
  if !data.vecdeques.is_empty() {
    writeln!(
      out,
      "{s}for &key in VECDEQUE_VALUES {{\n\
       {s}{s}state::assign_value(key, Stored::VecDequeStored(VecDeque::new()), Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Catcode table
  if !data.catcodes.is_empty() {
    writeln!(
      out,
      "{s}for &(key, val) in CATCODE_TABLE {{\n\
       {s}{s}if let Some(ch) = key.chars().next() {{\n\
       {s}{s}{s}state::assign_catcode(ch, Catcode::from(val), Some(Scope::Global));\n\
       {s}{s}}}\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // LC/UC/SF/DC/MC code tables
  for (array_name, fn_name) in [
    ("LCCODE_TABLE", "assign_lccode"),
    ("UCCODE_TABLE", "assign_uccode"),
    ("SFCODE_TABLE", "assign_sfcode"),
    ("DELCODE_TABLE", "assign_delcode"),
    ("MATHCODE_TABLE", "assign_mathcode"),
  ] {
    let entries_exist = match array_name {
      "LCCODE_TABLE" => !data.lccodes.is_empty(),
      "UCCODE_TABLE" => !data.uccodes.is_empty(),
      "SFCODE_TABLE" => !data.sfcodes.is_empty(),
      "DELCODE_TABLE" => !data.delcodes.is_empty(),
      "MATHCODE_TABLE" => !data.mathcodes.is_empty(),
      _ => false,
    };
    if entries_exist {
      writeln!(
        out,
        "{s}for &(key, val) in {array_name} {{\n\
         {s}{s}if let Some(ch) = key.chars().next() {{\n\
         {s}{s}{s}state::{fn_name}(ch, val, Some(Scope::Global));\n\
         {s}{s}}}\n\
         {s}}}"
      )
      .map_err(we)?;
    }
  }

  // Let definitions (M/T)
  if !data.let_defs.is_empty() {
    writeln!(
      out,
      "{s}for &(cs_name, cc, text) in LET_DEFINITIONS {{\n\
       {s}{s}let cs = Token {{ text: arena::pin(cs_name), code: Catcode::CS }};\n\
       {s}{s}if !state::has_meaning(&cs) {{\n\
       {s}{s}{s}let tok = Token {{ text: arena::pin(text), code: Catcode::from(cc) }};\n\
       {s}{s}{s}state::assign_meaning(&cs, tok, Some(Scope::Global));\n\
       {s}{s}}}\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Expandable definitions (M/E) — the bulk (77% of entries)
  if !data.expandables.is_empty() {
    writeln!(
      out,
      "{s}for def in EXPANDABLE_DEFS {{\n\
       {s}{s}let cs = Token {{ text: arena::pin(def.cs), code: Catcode::CS }};\n\
       {s}{s}if state::has_meaning(&cs) {{ continue; }}\n\
       {s}{s}let toks: Vec<Token> = EXPAND_TOKENS[def.tok_start as usize..][..def.tok_count as usize]\n\
       {s}{s}{s}.iter()\n\
       {s}{s}{s}.map(|&(cc, text)| Token {{ text: arena::pin(text), code: Catcode::from(cc) }})\n\
       {s}{s}{s}.collect();\n\
       {s}{s}let params = if def.nargs > 0 {{\n\
       {s}{s}{s}let proto = \"{{}}\".repeat(def.nargs as usize);\n\
       {s}{s}{s}// init_flag=true: engine is up when this table is consumed\n\
       {s}{s}{s}// at runtime, so Parameter::init() can resolve readers\n\
       {s}{s}{s}// via PARAMETER_TYPES; without init the Plain reader\n\
       {s}{s}{s}// falls back to mock_reader and invocation fails.\n\
       {s}{s}{s}parse_parameters(&proto, &cs, true).ok().flatten()\n\
       {s}{s}}} else {{ None }};\n\
       {s}{s}let opts = Some(ExpandableOptions {{\n\
       {s}{s}{s}long: def.flags & 1 != 0,\n\
       {s}{s}{s}protected: def.flags & 2 != 0,\n\
       {s}{s}{s}nopack_parameters: true,\n\
       {s}{s}{s}..ExpandableOptions::default()\n\
       {s}{s}}});\n\
       {s}{s}if let Ok(exp) = Expandable::new(cs, params, Some(Tokens::from(toks).into()), opts) {{\n\
       {s}{s}{s}state::install_definition(exp, Some(Scope::Global));\n\
       {s}{s}}}\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  // Primitive-alias entries (M/PA, M/MPA). The write pass ordered these
  // so that `target` is either a bootstrap primitive (always present) or
  // an entry defined earlier in this pass. Skip self-aliases and any
  // alias whose key already has a meaning (add-only parity).
  if !data.let_aliases.is_empty() {
    writeln!(
      out,
      "{s}for &(key_s, target_s) in LET_ALIAS_DEFS {{\n\
       {s}{s}if key_s == target_s {{ continue; }}\n\
       {s}{s}let key_tok = Token {{ text: arena::pin(key_s), code: Catcode::CS }};\n\
       {s}{s}if state::has_meaning(&key_tok) {{ continue; }}\n\
       {s}{s}let target_tok = Token {{ text: arena::pin(target_s), code: Catcode::CS }};\n\
       {s}{s}if !state::has_meaning(&target_tok) {{ continue; }}\n\
       {s}{s}state::let_i(&key_tok, &target_tok, Some(Scope::Global));\n\
       {s}}}"
    )
    .map_err(we)?;
  }

  writeln!(out, "\n{s}Ok(())\n}}").map_err(we)?;

  Ok(())
}
