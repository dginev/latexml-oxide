//! Reader for Rust-native kernel dump files (produced by dump_writer.rs).
//!
//! Loads a dump file produced by `latexml_oxide --init=latex.ltx --dest=dump`
//! and replays the state assignments into the engine.
//!
//! **Loading policy:** The dump loads AFTER the compiled engine definitions.
//! Meanings (M entries) use add-only policy: skip if the CS is already defined.
//! Values (V entries) use add-only policy: skip if the key already has a value.
//! This ensures compiled engine semantics (constructors, etc.) take priority
//! over raw TeX definitions from the dump, matching Perl's approach where
//! `latex_constructs` overrides the dump.
//!
//! Format: tab-separated lines:
//!   V\tKEY\tTYPE\tDATA             — value assignment
//!   M\tKEY\tE\tCS\tNARGS\tFLAGS\tTOKENS — Expandable definition
//!   M\tKEY\tN                       — None meaning (undefined)
//!   M\tKEY\tT\tCC:TEXT              — Token meaning (let-assignment)
//!   C\tCHAR\tCC\tVALUE             — catcode assignment
//!   LC\tCHAR\tCH\tVALUE            — lccode assignment
//!   UC\tCHAR\tCH\tVALUE            — uccode assignment
//!   SC\tCHAR\tCH\tVALUE            — sfcode assignment
//!   DC\tCHAR\tCH\tVALUE            — delcode assignment
//!   MC\tCHAR\tCH\tVALUE            — mathcode assignment

use std::path::Path;

use crate::common::arena;
use crate::common::numeric_ops::NumericOps;
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
  let count = load_from_str_internal(&content, &path.display().to_string())?;
  Ok(count)
}

/// Load dump data from a string (used by embedded dump modules).
/// Returns the number of entries loaded.
pub fn load_from_str(content: &str) -> Result<usize, String> {
  load_from_str_internal(content, "<embedded>")
}

fn load_from_str_internal(content: &str, source_name: &str) -> Result<usize, String> {
  let mut count = 0;
  let mut skipped = 0;
  let mut errors = 0;

  for (lineno, line) in content.lines().enumerate() {
    // Trim only CR (from CRLF line endings); `lines()` already strips LF.
    // Do NOT use `trim()` here — it strips trailing tabs, which are part of
    // the tab-separated format for entries with empty trailing fields (e.g.
    // E-entries with empty body: `E\t<cs>\t<nargs>\t<flags>\t`).
    let line = line.trim_end_matches('\r');
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    match parse_and_load(line) {
      Ok(true) => count += 1,
      Ok(false) => skipped += 1,
      Err(e) => {
        errors += 1;
        if errors <= 10 {
          log::warn!(
            "[dump_reader] Line {}: {}: {}",
            lineno + 1,
            e,
            &line[..line.len().min(80)]
          );
        }
      }
    }
  }

  if errors > 10 {
    log::warn!("[dump_reader] ... and {} more errors", errors - 10);
  }
  log::info!(
    "[dump_reader] Loaded {} entries from {} ({} skipped, {} errors)",
    count,
    source_name,
    skipped,
    errors
  );

  Ok(count)
}

/// Parse a single dump line and load it. Returns Ok(true) if loaded,
/// Ok(false) if skipped (already defined or filtered), Err on parse error.
fn parse_and_load(line: &str) -> Result<bool, String> {
  let parts: Vec<&str> = line.splitn(3, '\t').collect();
  if parts.len() < 2 {
    return Err("Too few fields".into());
  }

  let table = parts[0];
  let key = url_decode(parts[1]);
  let data = if parts.len() > 2 { parts[2] } else { "" };

  match table {
    // V: Value entries (registers, fontdimen, font metadata).
    // Add-only policy: only loads if key has no existing value.
    "V" => load_value(&key, data),
    // M: Meaning entries (expandable definitions + primitive aliases).
    //
    // Current policy: only route `@`-internal entries (whose body does
    // not reference the expl3 hook system) to `load_meaning`. Those
    // include `@`-internal PA/MPA aliases, which DO get consumed via
    // the PA arm of `load_meaning`. The @-internal gate is still the
    // right safety fence: it keeps us out of the expl3 short-circuit
    // hazard zone.
    //
    // `:`-style expl3 Meanings and public-CS PAs (like
    // `\tex_let:D → \let`) remain gated off because enabling them
    // causes two failure modes, both observed in earlier experiments:
    //
    //   - PA alone: `\tex_let:D` becomes let-aliased to `\let` via
    //     the dump → `expl3.sty`'s own guard fires → raw
    //     `\input expl3-code.tex` is skipped → post-guard code hits
    //     `\__kernel_dependency_version_check:Nn`, `\ProcessOptions`,
    //     `\keys_define:nn { sys }`, which are `:`-style macros we
    //     don't load → undefined-CS recovery loop (60 s timeout,
    //     memory climbing, SIGTERM-by-watchdog).
    //
    //   - PA + `:`-style M: the `:`-style bodies themselves trigger
    //     the same pattern via cross-references.
    //
    // Both must be unblocked TOGETHER, in coordination with
    // `expl3_sty.rs` short-circuiting its whole `load_definitions`
    // when the dump already supplies expl3 state. See SYNC_STATUS
    // D0 (d.5).
    "M" => {
      let name = key.trim_start_matches('\\');
      let is_at_internal = name.contains('@') && !name.contains(':');
      if is_at_internal && !data.contains("\\\\hook") && !data.contains("16:\\hook") {
        load_meaning(&key, data)
      } else {
        Ok(false)
      }
    },
    // LC/UC: case-mapping codes — safe, always load
    "LC" => load_lccode(&key, data),
    "UC" => load_uccode(&key, data),
    // SC: space factor codes — safe, always load
    "SC" => load_sfcode(&key, data),
    // C: catcodes — only for non-ASCII (>127). ASCII catcodes are set by
    // the engine; loading from dump would conflict.
    "C" => {
      let ch = decode_char_key(&key);
      if ch.is_some_and(|c| c as u32 > 127) {
        load_catcode(&key, data)
      } else {
        Ok(false)
      }
    }
    // MC/DC: mathcodes and delcodes from the dump are corrupted by expl3
    // format initialization (e.g., mathcode('v')=618 maps to '|').
    // The engine sets correct math/delcodes. Skip.
    "MC" | "DC" => Ok(false),
    _ => Ok(false),
  }
}

/// V entries to unconditionally skip (runtime state, never useful from dump).
const SKIP_VALUE_KEYS: &[&str] = &[
  "INTERPRETING_DEFINITIONS",
  "if_count",
  "absorb_count",
  "if_stack",
  "INCLUDE_COMMENTS",
  "INCLUDE_STYLES",
  "INPUT_ENCODING",
  "CURRENT_INPUT_ENCODING",
  "SUPPRESS_UNEXPECTED_ERRORS",
  "SUPPRESS_UNDEFINED_ERRORS",
];

/// V entry key prefixes to skip.
const SKIP_VALUE_PREFIXES: &[&str] = &[
  "input_file:",
  "output_file:",
  "texsys",
];

/// V entry key substrings to skip.
///
/// Note: `_loaded` / `_found_loaded` flags are present in the dump (correctly,
/// since `--init=latex.ltx` sees expl3-code.tex, hyphenation patterns, and
/// hundreds of other raw-loaded files). But carrying them through into state
/// at dump-load time blows up in practice:
///
///  - Hyphenation `loadhyph-*.tex_loaded` flags make subsequent raw-loading of
///    babel's language.def skip files that set `\l@<lang>` registers our
///    engine then discovers aren't present, triggering a flood of error
///    recovery that can consume gigabytes of RAM.
///  - `expl3.ltx_loaded=1` plus `expl3.sty_loaded=` NOT being set means
///    `\usepackage{expl3}` doesn't short-circuit AT the .sty layer, but the
///    raw .ltx re-load now enters a stranger code path with partial flags.
///
/// The proper fix, tracked as the "dump/_base mutual-exclusivity" item in
/// SYNC_STATUS D0, is to have exactly ONE loading path (dump-cache or raw-load)
/// active at a time, mirroring Perl's `LoadFormat` branching. Until that lands,
/// keep the skip list conservative so mixed paths don't trigger recovery loops.
const SKIP_VALUE_CONTAINS: &[&str] = &[
  "_loaded",       // Package loading flags — see doc comment above
  "_found_loaded", // Package found+loaded flags
];

/// Load a value entry: V\tKEY\tTYPE\tDATA
///
/// Uses add-only policy: only loads if the key does not already have a value.
/// This ensures compiled engine state takes priority over dump state.
fn load_value(key: &str, data: &str) -> Result<bool, String> {
  // Skip unconditional keys
  for skip in SKIP_VALUE_KEYS {
    if key == *skip {
      return Ok(false);
    }
  }
  // Skip by prefix
  for prefix in SKIP_VALUE_PREFIXES {
    if key.starts_with(prefix) {
      return Ok(false);
    }
  }
  // Skip by substring.
  for substr in SKIP_VALUE_CONTAINS {
    if key.contains(substr) {
      return Ok(false);
    }
  }

  // Add-only policy: don't overwrite any existing value.
  // This preserves engine-configured state (e.g., \everymath, \everypar,
  // named skips, etc.) while filling in gaps from the dump (e.g., expl3
  // fontdimen intarrays, font metadata).
  if state::has_value(key) {
    return Ok(false);
  }

  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Err("Missing value type".into());
  }

  let value = match parts[0] {
    "N" => return Ok(false), // Don't load None values (would erase existing)
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
    "VD" => return Ok(false), // Don't load empty VecDeque (runtime state)
    _ => return Ok(false),    // Unknown value type
  };

  state::assign_value(key, value, Some(Scope::Global));
  Ok(true)
}

/// Load a meaning entry: M\tKEY\tTYPE\t...
///
/// Uses add-only policy: skip if the CS already has a meaning.
/// Additionally, only loads "safe" definitions — those that won't interfere
/// with the compiled engine's processing during normal LaTeX operation:
/// - expl3 internals (contain `:`) — safe because `:` is OTHER under normal catcodes
/// - Private LaTeX internals (contain `@`) — only invoked by other macros
/// - Skip all "public" macros that could be invoked during normal expansion
///   and might reference hooks/primitives not supported by our engine
fn load_meaning(key: &str, data: &str) -> Result<bool, String> {
  let cs_tok = Token {
    text: arena::pin(key),
    code: Catcode::CS,
  };

  // Add-only policy: don't override ANY existing definition.
  if state::has_meaning(&cs_tok) {
    return Ok(false);
  }

  // Safety filter: only load definitions that won't interfere with normal
  // LaTeX processing. Public macros from the dump (like \document, \hook,
  // \UseOneTimeHook) can reference expl3 hooks and internal state that
  // our engine doesn't fully support, causing cascading errors.
  //
  // Safe: expl3 internals (with `:` or `__`), LaTeX internals (with `@`)
  // Unsafe: public macros without `:` or `@` (e.g., \document, \hook)
  let name = key.trim_start_matches('\\');
  let is_internal = name.contains(':') || name.contains('@');
  if !is_internal {
    // This is a "public" macro. Skip it — our engine either already defines
    // it (caught by has_meaning above) or it's a raw TeX definition that
    // would interfere with our LaTeXML-specific processing.
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
    "PA" | "MPA" => {
      // Primitive alias: PA\t<target_cs> — the entry's meaning is an
      // Rc<Primitive> whose own cs is <target_cs>. If <target_cs> == key
      // this is the "primary" entry (already provided by compiled bindings
      // in _base.rs etc.); skip. Otherwise, replay `\let <key> <target>`
      // so the Rc<Primitive> is shared just as it was when the dump was
      // generated. This is how \tex_let:D, \tex_def:D, etc. survive the
      // dump — without this the expl3.sty short-circuit guard
      // `\ifx\csname tex_let:D\endcsname\relax` never fires and the 36k
      // lines of expl3-code.tex get reprocessed on every run.
      let target_cs_raw = url_decode(parts.get(1).unwrap_or(&""));
      if target_cs_raw == key {
        return Ok(false);
      }
      let target_tok = Token {
        text: arena::pin(&target_cs_raw),
        code: Catcode::CS,
      };
      // Skip silently if the target isn't in bindings — rare and means
      // the alias points at a CS only defined during raw-load that we
      // also lost. The engine's undefined-CS handler will cope at runtime.
      if !state::has_meaning(&target_tok) {
        return Ok(false);
      }
      state::let_i(&cs_tok, &target_tok, Some(Scope::Global));
      Ok(true)
    }
    "R" => {
      // Register: R\tCS\tTYPE\tVALUE[\tMATHGLYPH]
      let rparts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(4, '\t').collect();
      if rparts.len() < 3 {
        return Err("Incomplete Register entry".into());
      }
      let _cs_name = url_decode(rparts[0]);
      let rtype = rparts[1];
      let value_str = rparts[2];
      let mathglyph = rparts.get(3).and_then(|s| s.parse::<u32>().ok()).and_then(char::from_u32);

      use crate::definition::register::{Register, RegisterType, RegisterValue};
      use crate::common::number::Number;

      let (reg_type, reg_value) = match rtype {
        "N" | "CD" => {
          let n: i64 = value_str.parse().unwrap_or(0);
          let rt = if rtype == "CD" { RegisterType::CharDef } else { RegisterType::Number };
          (rt, Some(RegisterValue::Number(Number::new(n))))
        }
        "D" => {
          let n: i64 = value_str.parse().unwrap_or(0);
          (RegisterType::Dimension, Some(RegisterValue::Dimension(
            crate::common::dimension::Dimension(n))))
        }
        "G" => {
          (RegisterType::Glue, Some(RegisterValue::Glue(
            parse_glue(value_str)?)))
        }
        "MG" => {
          (RegisterType::MuGlue, Some(RegisterValue::MuGlue(
            parse_muglue(value_str)?)))
        }
        "TK" => {
          // Token register: value is comma-separated token list, or "0" for empty
          let toks = if value_str == "0" || value_str.is_empty() {
            Vec::new()
          } else {
            parse_token_list(value_str)?
          };
          (RegisterType::Tokens, Some(RegisterValue::Tokens(
            Tokens::from(toks))))
        }
        _ => return Ok(false),
      };

      let mut reg = Register {
        cs: cs_tok,
        register_type: reg_type,
        value: reg_value,
        mathglyph,
        ..Register::default()
      };
      // Set address from CS name
      reg.address = key.to_string();
      state::install_definition(reg, Some(Scope::Global));
      Ok(true)
    }
    _ => Ok(false),
  }
}

/// Decode a character key from the dump. Handles:
/// - Single characters: "A", "è"
/// - URL-encoded control chars: "%19" (→ char 0x19), "%0A" (→ char 0x0A)
fn decode_char_key(key: &str) -> Option<char> {
  let decoded = url_decode(key);
  decoded.chars().next()
}

/// Load a catcode entry: C\tCHAR\tCC\tVALUE
fn load_catcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad catcode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CC" {
    return Err(format!("Bad catcode data: {}", data));
  }
  let cc: u8 = parts[1].parse().map_err(|e| format!("Bad catcode value: {}", e))?;
  state::assign_catcode(ch, Catcode::from(cc), Some(Scope::Global));
  Ok(true)
}

/// Load a lccode entry: LC\tCHAR\tCH\tVALUE
fn load_lccode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad lccode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CH" {
    return Err(format!("Bad lccode data: {}", data));
  }
  let val: u16 = parts[1].parse().map_err(|e| format!("Bad lccode value: {}", e))?;
  state::assign_lccode(ch, val, Some(Scope::Global));
  Ok(true)
}

/// Load a uccode entry: UC\tCHAR\tCH\tVALUE
fn load_uccode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad uccode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CH" {
    return Err(format!("Bad uccode data: {}", data));
  }
  let val: u16 = parts[1].parse().map_err(|e| format!("Bad uccode value: {}", e))?;
  state::assign_uccode(ch, val, Some(Scope::Global));
  Ok(true)
}

/// Load a sfcode entry: SC\tCHAR\tCH\tVALUE
fn load_sfcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad sfcode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CH" {
    return Err(format!("Bad sfcode data: {}", data));
  }
  let val: u16 = parts[1].parse().map_err(|e| format!("Bad sfcode value: {}", e))?;
  state::assign_sfcode(ch, val, Some(Scope::Global));
  Ok(true)
}

// load_delcode and load_mathcode were implemented but never wired — the
// "MC"/"DC" arm in parse_entry returns Ok(false) because the dumped values
// are corrupted by expl3 format init (see comment on that arm). If we
// eventually harvest clean delcode/mathcode data, restore them from git
// history and point the "MC"/"DC" arm at them.

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

pub(crate) fn url_decode(s: &str) -> String {
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
  fn test_load_native_dump_inline() {
    // Test with inline tab-separated dump content (no external file dependency)
    let content = "V\tcount@\tI\t42\nM\t\\mymacro\tE\t\\mymacro\t1\t\t6:1,6:2\n";
    let count = load_from_str(content).unwrap();
    assert!(count > 0, "Expected entries loaded from inline dump, got {}", count);
  }

  #[test]
  fn test_catcode_loading_nonascii() {
    // Only non-ASCII catcodes are loaded from the dump
    let content = "C\t\u{00e8}\tCC\t12\n"; // è → catcode 12 (OTHER)
    let count = load_from_str(content).unwrap();
    assert!(count > 0, "Expected non-ASCII catcode entry loaded");
  }

  #[test]
  fn test_lccode_loading() {
    let content = "LC\t\u{00c8}\tCH\t232\n"; // È → lccode 232 (è)
    let count = load_from_str(content).unwrap();
    assert!(count > 0, "Expected lccode entry loaded");
  }
}
