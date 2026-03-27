//! Loader for Perl LaTeXML kernel dump files (latex_dump.pool.ltxml).
//!
//! The dump file is Perl source code with shorthand function calls that reconstruct
//! the State after processing latex.ltx. By parsing and replaying these calls in Rust,
//! we get the precompiled LaTeX kernel without processing latex.ltx ourselves.
//!
//! Format: Each line is one of:
//!   Lt('\\csA','\\csB')           — let assignment (copy meaning)
//!   V('key',value)                — assign value
//!   Cc('char',catcode)            — assign catcode
//!   Im('key',meaning)             — assign meaning
//!   I(E(cs,params,expansion,...)) — install Expandable definition
//!   I(CD(cs,value,...))           — install CharDef
//!   I(R(cs,value,...))            — install Register
//!   I(FD(cs,...))                 — install FontDef
//!
//! Token format within definitions:
//!   C('\\csname')  — CS token
//!   L('x')         — letter token
//!   O('.')         — other token
//!   A(n)           — parameter #n
//!   $TB/$TE        — begin/end group
//!   $TS            — space
//!   $TM            — math shift
//!   $TP            — parameter (#)
//!   $TSP/$TSB      — superscript/subscript
//!   $TA            — alignment tab
//!   $CR            — newline (catcode 10)
//!   T(tok,tok,...) — token list
//!   UTF(n)         — character by codepoint

use std::path::Path;

use crate::common::arena;
use crate::state;
use crate::token::{Catcode, Token};

/// Load a Perl kernel dump file into the current State.
/// Returns the number of entries successfully loaded, or an error.
pub fn load_dump(path: &Path) -> Result<usize, String> {
  let content = std::fs::read_to_string(path)
    .map_err(|e| format!("Failed to read dump file {}: {}", path.display(), e))?;

  let mut count = 0;
  let mut errors = 0;

  // Join multi-line entries: lines not starting with an entry type are continuations
  let entry_prefixes = ["Lt(", "V(", "Cc(", "I(", "Im(", "Mc(", "Sc(", "Lc(", "Uc(", "Dc("];
  let mut current_entry = String::new();
  let mut entry_start_line = 0;

  let mut process_entry =
    |entry: &str, start_line: usize, count: &mut usize, errors: &mut usize| {
      match parse_and_execute(entry) {
        Ok(()) => *count += 1,
        Err(e) => {
          *errors += 1;
          if *errors <= 10 {
            eprintln!(
              "[dump_loader] Line {}: {}: {}",
              start_line,
              e,
              &entry[..entry.len().min(80)]
            );
          }
        }
      }
    };

  for (lineno, line) in content.lines().enumerate() {
    let line = line.trim();
    if line.is_empty()
      || line.starts_with('#')
      || line.starts_with("package ")
      || line.starts_with("use ")
      || line == "1;"
    {
      continue;
    }

    let is_new_entry = entry_prefixes.iter().any(|p| line.starts_with(p));
    if is_new_entry {
      // Process previous entry if any
      if !current_entry.is_empty() {
        process_entry(&current_entry, entry_start_line, &mut count, &mut errors);
      }
      current_entry = line.to_string();
      entry_start_line = lineno + 1;
    } else if !current_entry.is_empty() {
      // Continuation line
      current_entry.push(' ');
      current_entry.push_str(line);
    } else {
      // Orphan line (not a continuation, not an entry) — skip
      errors += 1;
    }
  }
  // Process final entry
  if !current_entry.is_empty() {
    process_entry(&current_entry, entry_start_line, &mut count, &mut errors);
  }

  if errors > 10 {
    eprintln!("[dump_loader] ... and {} more errors", errors - 10);
  }
  eprintln!(
    "[dump_loader] Loaded {} entries from {} ({} errors)",
    count,
    path.display(),
    errors
  );

  Ok(count)
}

/// Parse a single dump line and execute it.
fn parse_and_execute(line: &str) -> Result<(), String> {
  let line = line.strip_suffix(';').unwrap_or(line).trim();

  if let Some(rest) = line.strip_prefix("Lt(") {
    parse_let(rest)
  } else if let Some(rest) = line.strip_prefix("Cc(") {
    parse_catcode(rest)
  } else if let Some(rest) = line.strip_prefix("V(") {
    parse_value(rest)
  } else if let Some(rest) = line.strip_prefix("I(") {
    parse_install(rest)
  } else if let Some(rest) = line.strip_prefix("Im(") {
    parse_meaning(rest)
  } else if let Some(rest) = line.strip_prefix("Mc(") {
    parse_code_table(rest, "mathcode")
  } else if let Some(rest) = line.strip_prefix("Sc(") {
    parse_code_table(rest, "sfcode")
  } else if let Some(rest) = line.strip_prefix("Lc(") {
    parse_code_table(rest, "lccode")
  } else if let Some(rest) = line.strip_prefix("Uc(") {
    parse_code_table(rest, "uccode")
  } else if let Some(rest) = line.strip_prefix("Dc(") {
    parse_code_table(rest, "delcode")
  } else {
    Err(format!("Unknown dump entry type"))
  }
}

//======================================================================
// String parsing helpers
//======================================================================

/// Parse a Perl string literal: 'single quoted' or "double quoted" or UTF(n)
fn parse_perl_string(input: &str) -> Result<(String, &str), String> {
  let input = input.trim();
  if let Some(rest) = input.strip_prefix('\'') {
    // Single-quoted string
    let mut result = String::new();
    let mut chars = rest.chars();
    loop {
      match chars.next() {
        Some('\\') => match chars.next() {
          Some('\\') => result.push('\\'),
          Some('\'') => result.push('\''),
          Some(c) => {
            result.push('\\');
            result.push(c);
          }
          None => return Err("Unterminated single-quoted string".into()),
        },
        Some('\'') => {
          let remaining = chars.as_str();
          return Ok((result, remaining));
        }
        Some(c) => result.push(c),
        None => return Err("Unterminated single-quoted string".into()),
      }
    }
  } else if let Some(rest) = input.strip_prefix('"') {
    // Double-quoted string
    let mut result = String::new();
    let mut chars = rest.chars();
    loop {
      match chars.next() {
        Some('\\') => match chars.next() {
          Some('n') => result.push('\n'),
          Some('t') => result.push('\t'),
          Some('\\') => result.push('\\'),
          Some('"') => result.push('"'),
          Some('@') => result.push('@'),
          Some('$') => result.push('$'),
          Some('\'') => result.push('\''),
          Some('x') => {
            // \x{HHHH} Unicode escape
            if chars.next() == Some('{') {
              let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
              if let Ok(code) = u32::from_str_radix(&hex, 16) {
                if let Some(c) = char::from_u32(code) {
                  result.push(c);
                }
              }
            }
          }
          Some(c) => {
            result.push('\\');
            result.push(c);
          }
          None => return Err("Unterminated double-quoted string".into()),
        },
        Some('"') => {
          let remaining = chars.as_str();
          // Handle string concatenation: "abc".UTF(n)."def"
          let remaining = remaining.trim();
          if let Some(rest2) = remaining.strip_prefix('.') {
            let (next_str, rest3) = parse_perl_string(rest2.trim())?;
            result.push_str(&next_str);
            return Ok((result, rest3));
          }
          return Ok((result, remaining));
        }
        Some(c) => result.push(c),
        None => return Err("Unterminated double-quoted string".into()),
      }
    }
  } else if let Some(rest) = input.strip_prefix("UTF(") {
    // UTF(n) — character by codepoint
    let end = rest.find(')').ok_or("Unterminated UTF()")?;
    let code: u32 = rest[..end].trim().parse().map_err(|e| format!("Bad UTF code: {}", e))?;
    let ch = char::from_u32(code).ok_or("Invalid Unicode codepoint")?;
    let remaining = &rest[end + 1..];
    // Handle concatenation: UTF(n)."string"
    let remaining = remaining.trim();
    if let Some(rest2) = remaining.strip_prefix('.') {
      let (next_str, rest3) = parse_perl_string(rest2.trim())?;
      let mut s = String::new();
      s.push(ch);
      s.push_str(&next_str);
      return Ok((s, rest3));
    }
    Ok((ch.to_string(), remaining))
  } else {
    Err(format!("Expected string, got: {}", &input[..input.len().min(30)]))
  }
}

/// Skip a comma and whitespace
fn skip_comma(input: &str) -> &str {
  let input = input.trim();
  input.strip_prefix(',').unwrap_or(input).trim()
}

/// Skip closing paren
fn skip_close_paren(input: &str) -> Result<&str, String> {
  let input = input.trim();
  input
    .strip_prefix(')')
    .ok_or_else(|| format!("Expected ')', got: {}", &input[..input.len().min(20)]))
    .map(|s| s.trim())
}

//======================================================================
// Entry parsers
//======================================================================

/// Parse Lt('\\csA','\\csB') — let assignment
fn parse_let(input: &str) -> Result<(), String> {
  let (key, rest) = parse_perl_string(input)?;
  let rest = skip_comma(rest);
  let (target, _rest) = parse_perl_string(rest)?;

  // Look up the target's definition and assign it to the key
  let target_tok = make_cs_token(&target);
  let key_tok = make_cs_token(&key);

  if let Some(defn) = state::lookup_meaning(&target_tok) {
    state::assign_meaning(&key_tok, defn, Some(state::Scope::Global));
  }
  // If target has no meaning yet, store as a token reference (lazy let)
  // This handles forward references in the dump
  else {
    state::assign_meaning(&key_tok, target_tok, Some(state::Scope::Global));
  }
  Ok(())
}

/// Parse Cc('char',catcode) or Cc(num,catcode) — catcode assignment
fn parse_catcode(input: &str) -> Result<(), String> {
  let input = input.trim();
  // Key can be a string ('char') or bare number (e.g., Cc(2,12))
  let (ch, rest) = if input.starts_with('\'') || input.starts_with('"') || input.starts_with("UTF(")
  {
    let (s, rest) = parse_perl_string(input)?;
    (s.chars().next(), rest)
  } else {
    // Bare number — character code
    let end = input.find(',').ok_or("No comma in Cc()")?;
    let code: u32 = input[..end]
      .trim()
      .parse()
      .map_err(|e| format!("Bad char code: {}", e))?;
    (char::from_u32(code), &input[end..])
  };
  let rest = skip_comma(rest);
  let end = rest.find(')').unwrap_or(rest.len());
  let code: u8 = rest[..end]
    .trim()
    .parse()
    .map_err(|e| format!("Bad catcode: {}", e))?;

  if let Some(ch) = ch {
    let cc = Catcode::from(code);
    state::assign_catcode(ch, cc, Some(state::Scope::Global));
  }
  Ok(())
}

/// Parse V('key',value) — value assignment
fn parse_value(input: &str) -> Result<(), String> {
  let (key, rest) = parse_perl_string(input)?;
  let rest = skip_comma(rest);
  // Value can be a string, number, or complex object
  // For now, handle strings and numbers; skip complex objects
  let rest = rest.trim();
  if rest.starts_with('\'') || rest.starts_with('"') || rest.starts_with("UTF(") {
    let (val, _) = parse_perl_string(rest)?;
    state::assign_value(
      &key,
      crate::common::store::Stored::from(val),
      Some(state::Scope::Global),
    );
    Ok(())
  } else if rest.starts_with("undef") || rest.starts_with("$") {
    // Skip undef values and variable references
    Ok(())
  } else {
    // Try parsing as number
    let end = rest.find(')').unwrap_or(rest.len());
    let val_str = rest[..end].trim();
    if let Ok(n) = val_str.parse::<i64>() {
      state::assign_value(
        &key,
        crate::common::store::Stored::Int(n),
        Some(state::Scope::Global),
      );
      Ok(())
    } else {
      // Skip complex values we can't parse yet
      Ok(())
    }
  }
}

/// Parse I(E(...)) or I(CD(...)) or I(R(...)) — install definition
fn parse_install(input: &str) -> Result<(), String> {
  let input = input.trim();
  if input.starts_with("E(") {
    parse_install_expandable(&input[2..])
  } else if input.starts_with("CD(") {
    // CharDef — skip for now (requires more complex parsing)
    Ok(())
  } else if input.starts_with("R(") {
    // Register — skip for now
    Ok(())
  } else if input.starts_with("FD(") {
    // FontDef — skip for now
    Ok(())
  } else {
    Err(format!("Unknown I() type: {}", &input[..input.len().min(20)]))
  }
}

/// Parse Im('key', meaning) — assign meaning
fn parse_meaning(_input: &str) -> Result<(), String> {
  // Im is rare (116 entries) and complex; skip for initial prototype
  Ok(())
}

/// Parse code table entries (Mc, Sc, Lc, Uc, Dc)
fn parse_code_table(_input: &str, _table: &str) -> Result<(), String> {
  // These set mathcode, sfcode, lccode, uccode, delcode.
  // TODO: implement once the Rust State has these tables exposed.
  // For now, skip — these affect hyphenation/case-change, not macro processing.
  Ok(())
}

//======================================================================
// Expandable definition parser
//======================================================================

/// Parse E(cs_token, params, expansion, options...)
fn parse_install_expandable(input: &str) -> Result<(), String> {
  // Parse the CS token (first argument)
  let input = input.trim();
  let (cs_token, rest) = parse_dump_token(input)?;
  let cs_name = token_to_string(&cs_token);

  let rest = skip_comma(rest);

  // Parse parameters (second argument) — can be undef, $P, Ps(...), or P(...)
  let (params, rest) = parse_parameters(rest)?;
  let rest = skip_comma(rest);

  // Parse expansion (third argument) — T(tok,tok,...) token list
  let (expansion, rest) = parse_token_list(rest)?;

  // Parse optional trailing key=>value pairs (isLong, isProtected, etc.)
  let rest = rest.trim();
  let mut is_long = false;
  let mut is_protected = false;
  if rest.contains("isLong=>1") {
    is_long = true;
  }
  if rest.contains("isProtected=>1") {
    is_protected = true;
  }

  // Create and install the Expandable definition
  install_expandable(&cs_name, params, expansion, is_long, is_protected)
}

/// Parse a single token from the dump format
fn parse_dump_token(input: &str) -> Result<(Token, &str), String> {
  let input = input.trim();

  // Variable tokens
  if let Some(rest) = input.strip_prefix("$TB") {
    return Ok((Token { text: arena::pin_static("{"), code: Catcode::BEGIN }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TE") {
    return Ok((Token { text: arena::pin_static("}"), code: Catcode::END }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TM") {
    return Ok((Token { text: arena::pin_static("$"), code: Catcode::MATH }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TA") {
    return Ok((Token { text: arena::pin_static("&"), code: Catcode::ALIGN }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TP") {
    return Ok((Token { text: arena::pin_static("#"), code: Catcode::PARAM }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TSP") {
    return Ok((Token { text: arena::pin_static("^"), code: Catcode::SUPER }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TSB") {
    return Ok((Token { text: arena::pin_static("_"), code: Catcode::SUB }, rest));
  }
  if let Some(rest) = input.strip_prefix("$TS") {
    return Ok((Token { text: arena::pin_static(" "), code: Catcode::SPACE }, rest));
  }
  if let Some(rest) = input.strip_prefix("$CR") {
    return Ok((
      Token { text: arena::pin_static("\n"), code: Catcode::SPACE },
      rest,
    ));
  }

  // Function-form tokens
  if let Some(rest) = input.strip_prefix("C(") {
    let (s, rest) = parse_perl_string(rest)?;
    let rest = skip_close_paren(rest)?;
    return Ok((Token { text: arena::pin(&s), code: Catcode::CS }, rest));
  }
  if let Some(rest) = input.strip_prefix("L(") {
    let (s, rest) = parse_perl_string(rest)?;
    let rest = skip_close_paren(rest)?;
    return Ok((Token { text: arena::pin(&s), code: Catcode::LETTER }, rest));
  }
  if let Some(rest) = input.strip_prefix("O(") {
    let rest = rest.trim();
    // O() can have a string or a bare number
    let (s, rest) = if rest.starts_with('\'') || rest.starts_with('"') || rest.starts_with("UTF(") {
      parse_perl_string(rest)?
    } else {
      // Bare number or character — read until ')'
      let end = rest.find(')').ok_or("Unterminated O()")?;
      let val = rest[..end].trim();
      (val.to_string(), &rest[end..])
    };
    let rest = skip_close_paren(rest)?;
    return Ok((Token { text: arena::pin(&s), code: Catcode::OTHER }, rest));
  }
  if let Some(rest) = input.strip_prefix("A(") {
    let end = rest.find(')').ok_or("Unterminated A()")?;
    let n: u8 = rest[..end]
      .trim()
      .parse()
      .map_err(|e| format!("Bad arg number: {}", e))?;
    let s = format!("#{}", n);
    let rest = &rest[end + 1..];
    return Ok((Token { text: arena::pin(&s), code: Catcode::ARG }, rest));
  }
  if let Some(rest) = input.strip_prefix("TA(") {
    let (s, rest) = parse_perl_string(rest)?;
    let rest = skip_close_paren(rest)?;
    return Ok((Token { text: arena::pin(&s), code: Catcode::ACTIVE }, rest));
  }
  if let Some(rest) = input.strip_prefix("TM(") {
    let (s, rest) = parse_perl_string(rest)?;
    let rest = skip_close_paren(rest)?;
    return Ok((Token { text: arena::pin(&s), code: Catcode::MARKER }, rest));
  }

  Err(format!(
    "Unknown token format: {}",
    &input[..input.len().min(30)]
  ))
}

/// Parse T(tok,tok,...) — a token list
fn parse_token_list(input: &str) -> Result<(Vec<Token>, &str), String> {
  let input = input.trim();
  if let Some(rest) = input.strip_prefix("T(") {
    let mut tokens = Vec::new();
    let mut rest = rest.trim();
    while !rest.starts_with(')') && !rest.is_empty() {
      let (tok, r) = parse_dump_token(rest)?;
      tokens.push(tok);
      rest = skip_comma(r).trim();
    }
    let rest = skip_close_paren(rest)?;
    Ok((tokens, rest))
  } else {
    Err(format!(
      "Expected T(...), got: {}",
      &input[..input.len().min(30)]
    ))
  }
}

/// Parse parameter specification
fn parse_parameters(input: &str) -> Result<(Option<Vec<String>>, &str), String> {
  let input = input.trim();
  if input.starts_with("undef") {
    return Ok((None, &input[5..]));
  }
  if input.starts_with("$P") && !input[2..].starts_with('s') {
    // Single mandatory parameter $P
    return Ok((Some(vec!["{}".to_string()]), &input[2..]));
  }
  if let Some(rest) = input.strip_prefix("Ps(") {
    // Ps($P,$P,...) — list of parameters
    let mut params = Vec::new();
    let mut rest = rest.trim();
    while !rest.starts_with(')') && !rest.is_empty() {
      if rest.starts_with("$P") {
        params.push("{}".to_string());
        rest = skip_comma(&rest[2..]);
      } else if let Some(r) = rest.strip_prefix("P(") {
        // P(type, spec) — complex parameter
        let end = r.find(')').unwrap_or(r.len());
        let param_str = &r[..end];
        params.push(param_str.to_string());
        rest = skip_comma(&r[end + 1..]);
      } else {
        rest = skip_comma(&rest[1..]); // skip unknown
      }
    }
    let rest = skip_close_paren(rest)?;
    return Ok((Some(params), rest));
  }
  // Unknown parameter format — skip
  Ok((None, input))
}

//======================================================================
// Helpers
//======================================================================

fn make_cs_token(name: &str) -> Token {
  Token { text: arena::pin(name), code: Catcode::CS }
}

fn token_to_string(tok: &Token) -> String {
  arena::with(tok.text, |s| s.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_perl_string() {
    let (s, rest) = parse_perl_string("'hello')").unwrap();
    assert_eq!(s, "hello");
    assert_eq!(rest.trim(), ")");

    let (s, _) = parse_perl_string("'\\\\foo'").unwrap();
    assert_eq!(s, "\\foo");

    let (s, _) = parse_perl_string("UTF(13)").unwrap();
    assert_eq!(s, "\r");
  }

  #[test]
  fn test_load_dump_file() {
    let path = std::path::Path::new(
      "/home/deyan/perl5/lib/perl5/LaTeXML/Engine/latex_dump.pool.ltxml",
    );
    if !path.exists() {
      eprintln!("Dump file not found, skipping test");
      return;
    }
    let count = load_dump(path).unwrap();
    assert!(count > 1000, "Expected >1000 entries, got {}", count);
    eprintln!("Loaded {} entries from dump", count);
  }
}

/// Create and install an Expandable definition
fn install_expandable(
  cs_name: &str,
  _params: Option<Vec<String>>,
  expansion: Vec<Token>,
  _is_long: bool,
  _is_protected: bool,
) -> Result<(), String> {
  use crate::definition::expandable::Expandable;
  use crate::tokens::Tokens;

  let cs = make_cs_token(cs_name);
  let expansion_tokens = Tokens::from(expansion);

  // Create Expandable with no parameters (simplified — full param parsing needed later)
  // For the initial prototype, we install macros without parameter handling
  match Expandable::new(cs, None, expansion_tokens.into(), None) {
    Ok(exp) => {
      state::install_definition(exp, Some(state::Scope::Global));
      Ok(())
    }
    Err(e) => Err(format!("Failed to create Expandable for {}: {}", cs_name, e)),
  }
}
