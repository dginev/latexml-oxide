//! Reader for Rust-native kernel dump files (produced by dump_writer.rs).
//!
//! Loads a dump file produced by `latexml_oxide --init=latex.ltx --dest=dump`
//! and replays the state assignments into the engine.
//!
//! Format: tab-separated lines:
//!   V\tKEY\tTYPE\tDATA             — value assignment
//!   M\tKEY\tE\tCS\tNARGS\tFLAGS\tTOKENS — Expandable definition
//!   M\tKEY\tN                       — None meaning (undefined)
//!   M\tKEY\tT\tCC:TEXT              — Token meaning (let-assignment)

use std::path::Path;

use crate::common::arena;
use crate::common::store::Stored;
use crate::definition::expandable::{Expandable, ExpandableOptions};
use crate::state::{self, Scope};
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;


/// Load a Rust-native dump file into the current State.
/// Returns the number of entries loaded.
pub fn load_native_dump(path: &Path) -> Result<usize, String> {
  let content = std::fs::read_to_string(path)
    .map_err(|e| format!("Failed to read dump file {}: {}", path.display(), e))?;

  let mut count = 0;
  let mut errors = 0;

  for (lineno, line) in content.lines().enumerate() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    match parse_and_load(line) {
      Ok(true) => count += 1,
      Ok(false) => {} // skipped (already defined)
      Err(e) => {
        errors += 1;
        if errors <= 10 {
          log::warn!(
            "[dump_reader] Line {}: {}: {}",
            lineno + 1,
            e,
            &line[..line.len().min(60)]
          );
        }
      }
    }
  }

  if errors > 10 {
    log::warn!("[dump_reader] ... and {} more errors", errors - 10);
  }
  log::info!(
    "[dump_reader] Loaded {} entries from {} ({} errors)",
    count,
    path.display(),
    errors
  );

  Ok(count)
}

/// Parse a single dump line and load it. Returns Ok(true) if loaded,
/// Ok(false) if skipped (already defined), Err on parse error.
fn parse_and_load(line: &str) -> Result<bool, String> {
  let parts: Vec<&str> = line.splitn(3, '\t').collect();
  if parts.len() < 2 {
    return Err("Too few fields".into());
  }

  let table = parts[0];
  let key = url_decode(parts[1]);
  let data = if parts.len() > 2 { parts[2] } else { "" };

  match table {
    "V" => load_value(&key, data),
    "M" => load_meaning(&key, data),
    _ => Ok(false), // Skip unknown table types for now
  }
}

/// Value entries to skip (cause test regressions or are runtime-specific)
/// Value entries to skip from dump loading (runtime-specific or cause regressions)
const SKIP_VALUES: &[&str] = &[
  "INTERPRETING_DEFINITIONS", // Runtime flag
  "if_count",                 // Runtime counter
  "absorb_count",             // Runtime counter
  "_loaded",                  // Package loading flags (e.g., expl3-code.tex_loaded)
                              // These must be set by actual loading, not pre-set by dump,
                              // otherwise packages think they're loaded and skip initialization
  "INCLUDE_COMMENTS",         // Runtime config
  "INCLUDE_STYLES",           // Runtime config
  "\\everyjob",               // Token register — affects startup behavior
  "\\toks",                   // Token registers
  "input_file:",              // File tracking
  "output_file:",             // File tracking
  "texsys",                   // System config
];

/// Load a value entry: V\tKEY\tTYPE\tDATA
fn load_value(key: &str, data: &str) -> Result<bool, String> {
  // Skip values that cause regressions or are runtime-specific
  for skip in SKIP_VALUES {
    if key == *skip || key.starts_with(skip) || key.ends_with(skip) || key.contains(skip) {
      return Ok(false);
    }
  }
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Err("Missing value type".into());
  }

  let value = match parts[0] {
    "N" => Stored::None,
    "B" => Stored::Bool(parts.get(1).map(|s| *s == "1").unwrap_or(false)),
    "I" => {
      let n: i64 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad int: {}", e))?;
      Stored::Int(n)
    }
    "S" => Stored::from(url_decode(parts.get(1).unwrap_or(&""))),
    "CH" => {
      let n: u16 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad charcode: {}", e))?;
      Stored::Charcode(n)
    }
    "CC" => {
      let n: u8 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad catcode: {}", e))?;
      Stored::Catcode(Catcode::from(n))
    }
    "T" => {
      let tok = parse_token(parts.get(1).unwrap_or(&""))?;
      Stored::Token(tok)
    }
    "TK" => {
      let toks = parse_token_list(parts.get(1).unwrap_or(&""))?;
      Stored::Tokens(Tokens::from(toks))
    }
    "D" => {
      let n: i64 = parts.get(1).unwrap_or(&"0").parse()
        .map_err(|e| format!("Bad dimension: {}", e))?;
      Stored::Dimension(crate::common::dimension::Dimension(n))
    }
    "G" => {
      Stored::Glue(parse_glue(parts.get(1).unwrap_or(&"0"))?)
    }
    "MD" => {
      let n: i64 = parts.get(1).unwrap_or(&"0").parse()
        .map_err(|e| format!("Bad mudimension: {}", e))?;
      Stored::MuDimension(crate::common::mudimension::MuDimension(n))
    }
    "MG" => {
      Stored::MuGlue(parse_muglue(parts.get(1).unwrap_or(&"0"))?)
    }
    "VD" => Stored::VecDequeStored(std::collections::VecDeque::new()),
    _ => return Ok(false), // Unknown value type
  };

  state::assign_value(key, value, Some(Scope::Global));
  Ok(true)
}

/// Load a meaning entry: M\tKEY\tTYPE\t...
fn load_meaning(key: &str, data: &str) -> Result<bool, String> {
  let cs_tok = Token {
    text: arena::pin(key),
    code: Catcode::CS,
  };

  // Add-only policy: don't override ANY existing definition.
  if state::has_meaning(&cs_tok) {
    return Ok(false);
  }

  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Err("Missing meaning type".into());
  }

  match parts[0] {
    "N" => {
      // None meaning — skip (don't define as undefined)
      Ok(false)
    }
    "E" => {
      // Expandable: E\tCSNAME\tNARGS\tFLAGS\tTOKENS
      let eparts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(4, '\t').collect();
      if eparts.len() < 4 {
        return Err("Incomplete Expandable entry".into());
      }

      let _cs_name = url_decode(eparts[0]);
      let nargs: usize = eparts[1].parse().unwrap_or(0);
      let flags = eparts[2];
      let tok_data = eparts[3];

      let is_long = flags.contains('L');
      let is_protected = flags.contains('P');

      let expansion = parse_token_list(tok_data)?;

      // Build parameter spec from nargs
      let paramlist = if nargs > 0 {
        let proto = "{}".repeat(nargs);
        crate::common::def_parser::parse_parameters(&proto, &cs_tok, false)
          .map_err(|e| format!("Param parse: {}", e))?
      } else {
        None
      };

      let options = Some(ExpandableOptions {
        long: is_long,
        protected: is_protected,
        nopack_parameters: true, // tokens already have ARG catcode
        ..ExpandableOptions::default()
      });

      let expansion_body = Tokens::from(expansion).into();
      match Expandable::new(cs_tok, paramlist, Some(expansion_body), options) {
        Ok(exp) => {
          state::install_definition(exp, Some(Scope::Global));
          Ok(true)
        }
        Err(e) => Err(format!("Expandable creation failed: {}", e)),
      }
    }
    "T" => {
      // Token meaning (let-assignment)
      let tok = parse_token(parts.get(1).unwrap_or(&""))?;
      state::assign_meaning(&cs_tok, tok, Some(Scope::Global));
      Ok(true)
    }
    _ => Ok(false),
  }
}

/// Parse a single token from "CC:TEXT" format
fn parse_token(s: &str) -> Result<Token, String> {
  let (cc_str, text) = s.split_once(':').ok_or("Missing ':' in token")?;
  let cc: u8 = cc_str.parse().map_err(|e| format!("Bad CC: {}", e))?;
  Ok(Token {
    text: arena::pin(url_decode(text)),
    code: Catcode::from(cc),
  })
}

/// Parse comma-separated token list
fn parse_token_list(s: &str) -> Result<Vec<Token>, String> {
  if s.is_empty() {
    return Ok(Vec::new());
  }
  s.split(',').map(parse_token).collect()
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

/// Parse a serialized Glue value: "skip,pN,pfN,mN,mfN"
fn parse_glue(s: &str) -> Result<crate::common::glue::Glue, String> {
  use crate::common::glue::{FillCode, Glue};
  let mut skip = 0i64;
  let mut plus = None;
  let mut pfill = None;
  let mut minus = None;
  let mut mfill = None;
  for (i, part) in s.split(',').enumerate() {
    if i == 0 {
      skip = part.parse().map_err(|e| format!("Bad glue skip: {}", e))?;
    } else if let Some(rest) = part.strip_prefix("pf") {
      pfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('p') {
      plus = Some(rest.parse().map_err(|e| format!("Bad glue plus: {}", e))?);
    } else if let Some(rest) = part.strip_prefix("mf") {
      mfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('m') {
      minus = Some(rest.parse().map_err(|e| format!("Bad glue minus: {}", e))?);
    }
  }
  Ok(Glue { skip, plus, pfill, minus, mfill })
}

/// Parse a serialized MuGlue value (same format as Glue)
fn parse_muglue(s: &str) -> Result<crate::common::muglue::MuGlue, String> {
  use crate::common::glue::FillCode;
  use crate::common::muglue::MuGlue;
  let mut skip = 0i64;
  let mut plus = None;
  let mut pfill = None;
  let mut minus = None;
  let mut mfill = None;
  for (i, part) in s.split(',').enumerate() {
    if i == 0 {
      skip = part.parse().map_err(|e| format!("Bad muglue skip: {}", e))?;
    } else if let Some(rest) = part.strip_prefix("pf") {
      pfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('p') {
      plus = Some(rest.parse().map_err(|e| format!("Bad muglue plus: {}", e))?);
    } else if let Some(rest) = part.strip_prefix("mf") {
      mfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('m') {
      minus = Some(rest.parse().map_err(|e| format!("Bad muglue minus: {}", e))?);
    }
  }
  Ok(MuGlue { skip, plus, pfill, minus, mfill })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_load_native_dump() {
    let path = std::path::Path::new("/tmp/latex_dump.oxide");
    if !path.exists() {
      println!("No dump file at /tmp/latex_dump.oxide, skipping");
      return;
    }
    let count = load_native_dump(path).unwrap();
    assert!(count > 0, "Expected entries loaded");
    println!("Loaded {} entries from native dump", count);
  }
}
