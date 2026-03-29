//! Generate Rust source code from a kernel dump file.
//!
//! Reads the text dump produced by `dump_writer.rs` and emits a `.rs` file
//! containing direct `state::assign_value` / `state::install_definition` calls.
//! The generated module compiles with `latexml_package` and loads via
//! `LoadDefinitions!()`.
//!
//! Usage:
//!   1. Generate dump: `latexml_oxide --init=latex.ltx --dest=/tmp/latex_dump.oxide`
//!   2. Generate Rust: `dump_codegen::generate_rs("/tmp/latex_dump.oxide", "latex_dump.rs")`
//!   3. Place in `latexml_package/src/engine/latex_dump.rs`
//!   4. Load at runtime via `latex_dump::load_definitions()`

use std::io::{BufRead, Write};
use std::path::Path;

/// Value entries to skip (runtime-specific or cause regressions).
/// Must match dump_reader.rs::SKIP_VALUES.
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
  SKIP_VALUES.iter().any(|skip| {
    key == *skip || key.starts_with(skip) || key.ends_with(skip) || key.contains(skip)
  })
}

/// Escape a string for Rust source code (inside a raw string r#"..."#).
fn rust_escape(s: &str) -> String {
  // Raw strings can contain anything except the closing delimiter "# sequence.
  // If the string contains "# we fall back to standard string with escaping.
  if s.contains("\"#") || s.contains('\r') || s.chars().any(|c| c.is_ascii_control() && c != '\n' && c != '\t') {
    // Use normal string literal with escapes
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

/// Generate a Rust source file that embeds the dump data as a `const &str`.
/// The data is loaded at runtime via `dump_reader::load_from_str()`.
///
/// Trade-off analysis: Native Rust statements (20K `assign_value` calls) cause
/// compiler OOM in release mode. The text-embed approach compiles instantly and
/// loads in ~50ms at runtime — acceptable for a hot path that runs once per invocation.
pub fn generate_rs(dump_path: &Path, output_path: &Path) -> Result<usize, String> {
  let content = std::fs::read_to_string(dump_path)
    .map_err(|e| format!("Failed to read dump: {}", e))?;

  let count = content.lines()
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .count();

  // Ensure the content doesn't contain the raw string delimiter
  let delim = if content.contains("\"####") { "\"#####" } else { "\"####" };
  let open_delim = delim.replace('"', "");
  let raw_open = format!("r{open_delim}\"");
  let raw_close = format!("\"{open_delim}");

  let mut out = std::fs::File::create(output_path)
    .map_err(|e| format!("Failed to create output: {}", e))?;

  writeln!(out, r#"//! Auto-generated LaTeX kernel dump.
//! DO NOT EDIT — regenerate with: `cargo run --release --bin latexml_oxide -- --init=latex.ltx`
//!
//! Embedded as const string, loaded at startup via dump_reader (~50ms).
//! Native Rust statements would be faster (0ms) but cause compiler OOM for 20K+ entries.

/// Embedded kernel dump data ({count} entries).
const DUMP_DATA: &str = {raw_open}"#).map_err(|e| format!("Write error: {}", e))?;

  // Write the dump content directly
  out.write_all(content.as_bytes())
    .map_err(|e| format!("Write error: {}", e))?;

  writeln!(out, r#"{raw_close};

/// Load the precompiled LaTeX kernel definitions into the global state.
pub fn load_definitions() -> latexml_core::common::error::Result<()> {{
  latexml_core::dump_reader::load_from_str(DUMP_DATA)
    .map_err(latexml_core::common::error::Error::from)?;
  Ok(())
}}"#).map_err(|e| format!("Write error: {}", e))?;

  log::info!(
    "[dump_codegen] Generated {} entries to {} (embedded string)",
    count, output_path.display()
  );

  Ok(count)
}

/// Generate a single Rust statement from a dump line.
/// Returns None if the entry should be skipped.
fn generate_entry(line: &str) -> Result<Option<String>, String> {
  let parts: Vec<&str> = line.splitn(3, '\t').collect();
  if parts.len() < 2 {
    return Ok(None);
  }

  let table = parts[0];
  let key = url_decode(parts[1]);
  let data = if parts.len() > 2 { parts[2] } else { "" };

  match table {
    "V" => generate_value(&key, data),
    "M" => generate_meaning(&key, data),
    _ => Ok(None),
  }
}

/// Generate a value assignment statement.
fn generate_value(key: &str, data: &str) -> Result<Option<String>, String> {
  if should_skip_value(key) {
    return Ok(None);
  }

  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Ok(None);
  }

  let key_lit = rust_escape(key);
  let value_expr = match parts[0] {
    "N" => "Stored::None".to_string(),
    "B" => {
      let v = parts.get(1).map(|s| *s == "1").unwrap_or(false);
      format!("Stored::Bool({})", v)
    }
    "I" => {
      let n: i64 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
      format!("Stored::Int({})", n)
    }
    "S" => {
      let s = url_decode(parts.get(1).unwrap_or(&""));
      format!("Stored::from({})", rust_escape(&s))
    }
    "CH" => {
      let n: u16 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
      format!("Stored::Charcode({})", n)
    }
    "CC" => {
      let n: u8 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
      format!("Stored::Catcode(Catcode::from({}))", n)
    }
    "T" => {
      let tok_s = parts.get(1).unwrap_or(&"");
      match generate_token_expr(tok_s) {
        Some(expr) => format!("Stored::Token({})", expr),
        None => return Ok(None),
      }
    }
    "TK" => {
      let tok_s = parts.get(1).unwrap_or(&"");
      if tok_s.is_empty() {
        "Stored::Tokens(Tokens::default())".to_string()
      } else {
        let tok_exprs: Vec<String> = tok_s.split(',').filter_map(generate_token_expr).collect();
        format!("Stored::Tokens(Tokens::from(vec![{}]))", tok_exprs.join(", "))
      }
    }
    "D" => {
      let n: i64 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
      format!("Stored::Dimension(Dimension({}))", n)
    }
    "G" => {
      let g = parts.get(1).unwrap_or(&"0");
      generate_glue_expr("Glue", g)
    }
    "MD" => {
      let n: i64 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
      format!("Stored::MuDimension(MuDimension({}))", n)
    }
    "MG" => {
      let g = parts.get(1).unwrap_or(&"0");
      generate_glue_expr("MuGlue", g)
    }
    "VD" => "Stored::VecDequeStored(VecDeque::new())".to_string(),
    _ => return Ok(None),
  };

  Ok(Some(format!(
    "state::assign_value({}, {}, Some(Scope::Global));",
    key_lit, value_expr
  )))
}

/// Generate a meaning (definition) statement.
fn generate_meaning(key: &str, data: &str) -> Result<Option<String>, String> {
  let key_lit = rust_escape(key);
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Ok(None);
  }

  match parts[0] {
    "N" => Ok(None), // Skip None meanings
    "E" => {
      // Expandable: E\tCSNAME\tNARGS\tFLAGS\tTOKENS
      let eparts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(4, '\t').collect();
      if eparts.len() < 4 {
        return Ok(None);
      }
      let nargs: usize = eparts[1].parse().unwrap_or(0);
      let flags = eparts[2];
      let tok_data = eparts[3];

      let is_long = flags.contains('L');
      let is_protected = flags.contains('P');

      let tok_exprs: Vec<String> = tok_data.split(',').filter_map(generate_token_expr).collect();
      if tok_exprs.is_empty() && !tok_data.is_empty() {
        return Ok(None);
      }

      let mut code = String::new();
      code.push_str(&format!(
        "{{ let cs = Token {{ text: arena::pin({}), code: Catcode::CS }}; ",
        key_lit
      ));
      code.push_str("if !state::has_meaning(&cs) { ");
      code.push_str(&format!(
        "let toks = Tokens::from(vec![{}]); ",
        tok_exprs.join(", ")
      ));

      // Build parameter spec
      if nargs > 0 {
        code.push_str(&format!(
          "let params = parse_parameters(&{}, &cs, false).ok(); ",
          rust_escape(&"{}".repeat(nargs))
        ));
      } else {
        code.push_str("let params = None; ");
      }

      code.push_str(&format!(
        "let opts = Some(ExpandableOptions {{ long: {}, protected: {}, nopack_parameters: true, ..ExpandableOptions::default() }}); ",
        is_long, is_protected
      ));
      code.push_str(
        "if let Ok(exp) = Expandable::new(cs, params, Some(toks.into()), opts) { \
         state::install_definition(exp, Some(Scope::Global)); } ",
      );
      code.push_str("} }");

      Ok(Some(code))
    }
    "T" => {
      // Token meaning (let-assignment)
      let tok_s = parts.get(1).unwrap_or(&"");
      match generate_token_expr(tok_s) {
        Some(tok_expr) => {
          let code = format!(
            "{{ let cs = Token {{ text: arena::pin({}), code: Catcode::CS }}; \
             if !state::has_meaning(&cs) {{ \
             state::assign_meaning(&cs, {}, Some(Scope::Global)); }} }}",
            key_lit, tok_expr
          );
          Ok(Some(code))
        }
        None => Ok(None),
      }
    }
    _ => Ok(None),
  }
}

/// Generate a Rust Token expression from "CC:TEXT" format.
fn generate_token_expr(s: &str) -> Option<String> {
  let (cc_str, text) = s.split_once(':')?;
  let cc: u8 = cc_str.parse().ok()?;
  let decoded = url_decode(text);
  Some(format!(
    "Token {{ text: arena::pin({}), code: Catcode::from({}) }}",
    rust_escape(&decoded),
    cc
  ))
}

/// Generate a Glue or MuGlue expression from serialized format.
fn generate_glue_expr(type_name: &str, s: &str) -> String {
  let mut skip = "0".to_string();
  let mut plus = "None".to_string();
  let mut pfill = "None".to_string();
  let mut minus = "None".to_string();
  let mut mfill = "None".to_string();

  for (i, part) in s.split(',').enumerate() {
    if i == 0 {
      skip = part.to_string();
    } else if let Some(rest) = part.strip_prefix("pf") {
      pfill = format!("FillCode::new({})", rest);
    } else if let Some(rest) = part.strip_prefix('p') {
      plus = format!("Some({})", rest);
    } else if let Some(rest) = part.strip_prefix("mf") {
      mfill = format!("FillCode::new({})", rest);
    } else if let Some(rest) = part.strip_prefix('m') {
      minus = format!("Some({})", rest);
    }
  }

  let stored_type = if type_name == "Glue" {
    "Stored::Glue"
  } else {
    "Stored::MuGlue"
  };

  format!(
    "{}({} {{ skip: {}, plus: {}, pfill: {}, minus: {}, mfill: {} }})",
    stored_type, type_name, skip, plus, pfill, minus, mfill
  )
}

const MODULE_HEADER: &str = r#"//! Auto-generated LaTeX kernel dump.
//! Generated by dump_codegen from a latexml-oxide format dump.
//! DO NOT EDIT — regenerate with `latexml_oxide --init=latex.ltx --codegen`.
#![allow(unused)]

use std::collections::VecDeque;

use latexml_core::common::arena;
use latexml_core::common::dimension::Dimension;
use latexml_core::common::glue::{FillCode, Glue};
use latexml_core::common::mudimension::MuDimension;
use latexml_core::common::muglue::MuGlue;
use latexml_core::common::store::Stored;
use latexml_core::definition::expandable::{Expandable, ExpandableOptions};
use latexml_core::state;
use latexml_core::state::Scope;
use latexml_core::token::{Catcode, Token};
use latexml_core::tokens::Tokens;

pub fn load_definitions() -> latexml_core::common::error::Result<()> {
"#;

const MODULE_FOOTER: &str = r#"  Ok(())
}
"#;

/// Generate a compact Rust module that embeds the dump as a static string
/// and loads it via the existing dump_reader at startup.
/// This approach is much faster to compile than 20K individual Rust statements.
pub fn generate_embed_rs(dump_path: &Path, output_dir: &Path) -> Result<usize, String> {
  // Copy the dump file to the output directory
  let dump_filename = dump_path.file_name()
    .ok_or("Invalid dump filename")?
    .to_string_lossy();
  let dump_dest = output_dir.join(&*dump_filename);
  std::fs::copy(dump_path, &dump_dest)
    .map_err(|e| format!("Failed to copy dump: {}", e))?;

  // Generate the wrapper module
  let rs_path = output_dir.join("latex_dump.rs");
  let mut out = std::fs::File::create(&rs_path)
    .map_err(|e| format!("Failed to create: {}", e))?;

  let code = format!(
    r#"//! Auto-generated LaTeX kernel dump loader.
//! Generated by dump_codegen. DO NOT EDIT.
//! Regenerate with: `latexml_oxide --init=latex.ltx` then `--codegen=...`

/// The embedded kernel dump data.
const DUMP_DATA: &str = include_str!("{}");

/// Load the precompiled LaTeX kernel definitions into the global state.
/// This replaces processing latex.ltx at every test run.
pub fn load_definitions() -> latexml_core::common::error::Result<()> {{
  latexml_core::dump_reader::load_from_str(DUMP_DATA)
    .map_err(|e| latexml_core::common::error::Error::from(e))?;
  Ok(())
}}
"#,
    dump_filename
  );

  write!(out, "{}", code).map_err(|e| format!("Write error: {}", e))?;

  // Count entries in the dump for reporting
  let content = std::fs::read_to_string(dump_path)
    .map_err(|e| format!("Failed to read dump: {}", e))?;
  let count = content.lines().filter(|l| !l.is_empty() && !l.starts_with('#')).count();

  log::info!("[dump_codegen] Generated embed module at {}", rs_path.display());
  Ok(count)
}
