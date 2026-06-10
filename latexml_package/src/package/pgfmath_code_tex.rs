//! pgfmath.code.tex — PGF math operations accelerator
//! Perl: pgfmath.code.tex.ltxml (737 lines)
//!
//! Replaces TeX-level pgfmath operations with native Rust implementations
//! for better precision and performance. Without this, pgf math runs in
//! TeX's fixed-point arithmetic which has different rounding behavior.
use crate::prelude::*;

const PI: f64 = std::f64::consts::PI;
const E_CONST: f64 = std::f64::consts::E;
const LOG2: f64 = std::f64::consts::LN_2;
const LOG10: f64 = std::f64::consts::LN_10;
const EPSILON: f64 = 0.00001;
const MAX_PGF_NUMBER: f64 = 16383.99998;

// ==================== Result Formatting ====================

/// Format a RAW pgfmath result, as set by pgf's internal `@`-suffixed
/// functions (e.g. `\pgfmathmod@`, the trig used by
/// `\pgfmathanglebetweenpoints`). Perl LaTeXML / real pgf:
///   `sub pgfmathresult { sprintf("%.5f", $value); $value =~ s/0+$//; ... }`
/// — i.e. it STRIPS trailing zeros AND the now-bare decimal point, so an
/// integer prints as `10`, not `10.0`. (The PUBLIC `\pgfmathparse` path keeps
/// the `.0` — that is `format_parse_result`, deliberately separate.)
///
/// Faithfulness matters here beyond cosmetics: pgf's
/// `\pgfmathpointintersectionoflineandarc` runs an UNBOUNDED bisection
/// (`pgfmathcalc.code.tex`) whose only exit is the exact comparison
/// `\ifdim\x pt=\q pt`. When the `@`-functions append a spurious `.0`, the
/// probe angle's string never matches the target the way TeX's raw fixed-point
/// output does, the bisection over-narrows to `\p == \s` (no break there) and
/// spins forever — each iteration opening a `{` group that accumulates as an
/// empty box → OOM (witness 2201.09268, a `rounded rectangle`/callout node
/// boundary).
fn pgfmath_result_str(value: f64) -> String {
  if value.is_nan() || value.is_infinite() {
    return "0".to_string();
  }
  let clamped = value.clamp(-MAX_PGF_NUMBER, MAX_PGF_NUMBER);
  if clamped == (clamped as i64) as f64 {
    // Integer result — raw, NO `.0` (matches Perl `@`-function output).
    return format!("{}", clamped as i64);
  }
  let mut s = format!("{:.5}", clamped);
  // Strip trailing zeros after the decimal point (the fractional case always
  // keeps at least one non-zero fractional digit, so this never bares the dot).
  while s.ends_with('0') {
    s.pop();
  }
  s
}

/// Return tokens that \def\pgfmathresult{<value>}
fn pgfmath_result_tokens(value: f64) -> Vec<Token> {
  let s = pgfmath_result_str(value);
  let mut toks = vec![T_CS!("\\def"), T_CS!("\\pgfmathresult"), T_BEGIN!()];
  toks.extend(Explode!(s));
  toks.push(T_END!());
  toks
}

/// Return tokens that \def\pgfmathresult{<string>}
fn pgfmath_result_tokens_str(s: &str) -> Vec<Token> {
  let mut toks = vec![T_CS!("\\def"), T_CS!("\\pgfmathresult"), T_BEGIN!()];
  toks.extend(Explode!(s));
  toks.push(T_END!());
  toks
}

/// Safe divisor — avoid division by zero
fn pgfmath_divisor(v: f64) -> f64 { if v == 0.0 { EPSILON } else { v } }

/// Convert degrees to radians (pgf default is degrees)
fn pgfmath_arg_radians(arg: f64) -> f64 { arg.to_radians() }

/// Factorial (matches Perl's memoized_pgf_factorial)
fn pgfmath_factorial(n: i64) -> f64 {
  static FACTS: [f64; 22] = [
    1.0,
    1.0,
    2.0,
    6.0,
    24.0,
    120.0,
    720.0,
    5040.0,
    40320.0,
    362880.0,
    3628800.0,
    39916800.0,
    479001600.0,
    6227020800.0,
    87178291200.0,
    1307674368000.0,
    20922789888000.0,
    355687428096000.0,
    6402373705728000.0,
    121645100408832000.0,
    2432902008176640000.0,
    51090942171709440000.0,
  ];
  let n_orig = n;
  let n = n.unsigned_abs() as usize;
  if n >= FACTS.len() {
    // Perl pgfmath.code.tex.ltxml L68:
    //   Error("pgfmath", "overflow", undef,
    //     "Arithmetic overflow: $arg! is too large.");
    //   return Pgfmath::FALSE;   # i.e. 0
    // Emit the diagnostic and return 0 exactly like Perl. We must NOT run the
    // product loop: `n` is a saturating `f64 as i64 as usize`, so a user
    // `\pgfmathparse{1e20!}` makes `n ≈ 9.2e18` and `(22..=n).fold(..)` spins
    // ~9.2e18 iterations — a hang reachable from plain document TeX (review
    // B2). 0 is also benign downstream and any n ≥ 22! already overflows pgf's
    // MAX_PGF_NUMBER range, so the prior `clamp`-to-max output was meaningless.
    let _ = (|| -> Result<()> {
      Error!(
        "pgfmath",
        "overflow",
        format!("Arithmetic overflow: {n_orig}! is too large.")
      );
      Ok(())
    })();
    0.0
  } else {
    FACTS[n]
  }
}

/// Parse a pgfNumber argument: expand macros, then convert to f64
/// Perl: pgfNumber parameter type reads and evaluates the number
fn parse_pgf_number(arg: &Tokens) -> f64 {
  // First try direct string (avoids expansion overhead for literal numbers)
  let s = arg.to_string();
  let s = strip_leading_double_negation(s.trim());
  if s == "." {
    return 0.0;
  }
  if let Ok(v) = s.parse::<f64>() {
    return v;
  }
  // If direct parse fails (e.g. unexpanded macro), expand and retry
  if let Ok(expanded) = gullet::do_expand(arg.clone()) {
    let s = expanded.to_string();
    let s = strip_leading_double_negation(s.trim());
    if s == "." {
      return 0.0;
    }
    s.parse::<f64>().unwrap_or(0.0)
  } else {
    0.0
  }
}

// Perl #2711 uses `s/^\-\-//g`, which strips every leading `--` pair
// (e.g. `----5` → `5`). `strip_prefix` only removes one pair.
fn strip_leading_double_negation(mut s: &str) -> &str {
  while let Some(rest) = s.strip_prefix("--") {
    s = rest;
  }
  s
}

// ==================== Simple Number Check ====================
// Perl: /^([-+])?(\d+)(\.[\d.]*)?$/

/// Try to parse input as a simple number and format per Perl rules.
/// Returns None if input is not a simple number.
fn try_simple_number(input: &str) -> Option<String> {
  let bytes = input.as_bytes();
  let mut pos = 0;

  // Optional sign
  let sign: Option<char> = if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
    let c = bytes[pos] as char;
    pos += 1;
    Some(c)
  } else {
    None
  };

  // Required: at least one digit
  let int_start = pos;
  while pos < bytes.len() && bytes[pos].is_ascii_digit() {
    pos += 1;
  }
  if pos == int_start {
    return None;
  }
  let integer = &input[int_start..pos];

  // Optional decimal part: \.[\d.]*
  let decimals: Option<&str> = if pos < bytes.len() && bytes[pos] == b'.' {
    let dec_start = pos;
    pos += 1;
    while pos < bytes.len() && (bytes[pos].is_ascii_digit() || bytes[pos] == b'.') {
      pos += 1;
    }
    Some(&input[dec_start..pos])
  } else {
    None
  };

  // Must be at end of string
  if pos != bytes.len() {
    return None;
  }

  // Format result (Perl L332-344)
  let mut result = input.to_string();
  if let Some(s) = sign {
    if s == '+' {
      result = match decimals {
        Some(d) => format!("{}{}", integer, d),
        None => integer.to_string(),
      };
    } else {
      // sign == '-'
      if !result.contains(|c: char| ('1'..='9').contains(&c)) {
        result = "0.0".to_string();
      } else if decimals.is_none() {
        result = format!("{}.0", result);
      }
      // Overflow check for negative
      if let Ok(numval) = result.parse::<f64>() {
        if numval < -MAX_PGF_NUMBER {
          result = format!("{}", -MAX_PGF_NUMBER);
        }
      }
    }
  }
  Some(result)
}

// ==================== Unit Conversion ====================

/// Convert number with unit to points
/// Perl: sub pgfmath_convert (L476-485)
fn pgfmath_convert(number: f64, unit: &str) -> f64 {
  let sp = state::convert_unit(unit);
  number * sp / 65536.0
}

// ==================== Register Lookup ====================

/// Look up a TeX register value, return as points
/// Perl: sub pgfmath_register (L487-493)
fn pgfmath_register_lookup(cs: &str) -> f64 {
  if let Ok(Some(reg)) = state::lookup_register(cs, vec![]) {
    match reg {
      RegisterValue::Number(n) => n.value_of() as f64,
      RegisterValue::Dimension(d) => d.value_of() as f64 / 65536.0,
      RegisterValue::Glue(g) => g.value_of() as f64 / 65536.0,
      _ => 0.0,
    }
  } else {
    0.0
  }
}

// ==================== Built-in Function Application ====================

/// TeX units recognized by the parser (Perl L714-715)
const PGF_UNITS: &[&str] = &[
  "ex", "em", "pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp",
];

/// Check if name is a known built-in function (Perl L720)
fn is_builtin_function(name: &str) -> bool {
  matches!(
    name,
    "abs"
      | "acos"
      | "asin"
      | "atan2"
      | "atan"
      | "angle"
      | "bin"
      | "ceil"
      | "cos"
      | "cosec"
      | "cosh"
      | "cot"
      | "deg"
      | "exp"
      | "factorial"
      | "floor"
      | "frac"
      | "gcd"
      | "hex"
      | "Hex"
      | "int"
      | "ifthenelse"
      | "iseven"
      | "isodd"
      | "isprime"
      | "ln"
      | "log10"
      | "log2"
      | "max"
      | "min"
      | "mod"
      | "Mod"
      | "neg"
      | "not"
      | "oct"
      | "pow"
      | "rad"
      | "random"
      | "real"
      | "round"
      | "scalar"
      | "sec"
      | "sign"
      | "sin"
      | "sinh"
      | "sqrt"
      | "subtract"
      | "tan"
      | "tanh"
      | "add"
      | "and"
      | "divide"
      | "div"
      | "equal"
      | "greater"
      | "less"
      | "multiply"
      | "notequal"
      | "notgreater"
      | "notless"
      | "or"
      | "veclen"
  )
}

/// Check if name is a known constant (zero-arity function) (Perl L717)
fn is_builtin_constant(name: &str) -> bool {
  matches!(
    name,
    "e" | "pi" | "false" | "rand" | "rnd" | "true" | "axis_height" | "rule_thickness"
  )
}

/// Apply a built-in pgf math function (Perl L556-647)
fn pgfmath_apply_fn(name: &str, args: &[f64]) -> f64 {
  let a = args.first().copied().unwrap_or(0.0);
  let b = args.get(1).copied().unwrap_or(0.0);
  match name {
    // Constants
    "e" => E_CONST,
    "pi" => PI,
    "false" => 0.0,
    "true" => 1.0,
    "rand" => 0.0, // deterministic
    "rnd" => 0.5,  // deterministic
    "axis_height" => 2.5,
    "rule_thickness" => 0.39998,
    // Arithmetic
    "add" => a + b,
    "subtract" => a - b,
    "neg" => -a,
    "multiply" => a * b,
    "divide" => a / pgfmath_divisor(b),
    "div" => (a / pgfmath_divisor(b)) as i64 as f64,
    "pow" => a.powf(b),
    "abs" => a.abs(),
    "round" => a.round(),
    "floor" => a.floor(),
    "ceil" => a.ceil(),
    "int" => a as i64 as f64,
    "real" => a,
    "mod" | "Mod" => {
      // Perl: pgfmath_mod_trunc for mod, pgfmath_mod_floor for Mod
      if name == "mod" {
        a % pgfmath_divisor(b) // truncated mod (like Perl %)
      } else {
        // floor mod
        let b_abs = b.abs();
        if a / pgfmath_divisor(b) < 0.0 {
          -(a.abs() % b_abs) + b_abs
        } else {
          a.abs() % b_abs
        }
      }
    },
    "sign" => {
      if a > 0.0 {
        1.0
      } else if a < 0.0 {
        -1.0
      } else {
        0.0
      }
    },
    "factorial" => pgfmath_factorial(a as i64),
    // Trigonometric (input in degrees)
    "sin" => pgfmath_arg_radians(a).sin(),
    "cos" => pgfmath_arg_radians(a).cos(),
    "tan" => pgfmath_arg_radians(a).tan(),
    "cot" => {
      let r = pgfmath_arg_radians(a);
      r.cos() / pgfmath_divisor(r.sin())
    },
    "sec" => 1.0 / pgfmath_divisor(pgfmath_arg_radians(a).cos()),
    "cosec" => 1.0 / pgfmath_divisor(pgfmath_arg_radians(a).sin()),
    "asin" => a.asin().to_degrees(),
    "acos" => a.acos().to_degrees(),
    "atan" => a.atan().to_degrees(),
    "atan2" | "angle" => a.atan2(b).to_degrees(),
    "sinh" => a.sinh(),
    "cosh" => a.cosh(),
    "tanh" => a.tanh(),
    // Exponential/logarithmic
    "exp" => a.exp(),
    "ln" => a.ln(),
    "log10" => a.ln() / LOG10,
    "log2" => a.ln() / LOG2,
    "sqrt" => a.sqrt(),
    // Conversion
    "deg" => a.to_degrees(),
    "rad" => a.to_radians(),
    // Comparison/logic
    "equal" => {
      if a == b {
        1.0
      } else {
        0.0
      }
    },
    "greater" => {
      if a > b {
        1.0
      } else {
        0.0
      }
    },
    "less" => {
      if a < b {
        1.0
      } else {
        0.0
      }
    },
    "notequal" => {
      if a != b {
        1.0
      } else {
        0.0
      }
    },
    "notgreater" => {
      if a <= b {
        1.0
      } else {
        0.0
      }
    },
    "notless" => {
      if a >= b {
        1.0
      } else {
        0.0
      }
    },
    "and" => {
      if a != 0.0 && b != 0.0 {
        1.0
      } else {
        0.0
      }
    },
    "or" => {
      if a != 0.0 || b != 0.0 {
        1.0
      } else {
        0.0
      }
    },
    "not" => {
      if a == 0.0 {
        1.0
      } else {
        0.0
      }
    },
    // Misc
    "max" => a.max(b),
    "min" => a.min(b),
    "iseven" => {
      if (a as i64) % 2 == 0 {
        1.0
      } else {
        0.0
      }
    },
    "isodd" => {
      if (a as i64) % 2 != 0 {
        1.0
      } else {
        0.0
      }
    },
    "hex" | "Hex" | "oct" | "bin" => a, // formatting functions — return value
    "ifthenelse" => {
      let c = args.get(2).copied().unwrap_or(0.0);
      if a != 0.0 { b } else { c }
    },
    "veclen" => (a * a + b * b).sqrt(),
    "scalar" => a,                   // just returns the value
    "frac" => a - (a as i64 as f64), // fractional part
    _ => {
      // Try user-defined function; if neither built-in nor user-defined,
      // Perl pgfmath.code.tex.ltxml L457-459:
      //   Error('unexpected', $op, undef, "Unimplemented pgfmath operator '$op'");
      //   return 0;
      pgfmath_apply_user(name, args).unwrap_or_else(|| {
        let _ = (|| -> Result<()> {
          Error!(
            "unexpected",
            name,
            format!("Unimplemented pgfmath operator '{name}'")
          );
          Ok(())
        })();
        0.0
      })
    },
  }
}

/// Apply a user-defined pgf math function by calling TeX
/// Perl: sub pgfmath_apply (L447-459)
fn pgfmath_apply_user(name: &str, args: &[f64]) -> Option<f64> {
  let cs_name = format!("\\pgfmath{}@", name);
  let cs_tok = Token {
    text: arena::pin(&cs_name),
    code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    };
  // Check if the function is defined
  state::lookup_definition(&cs_tok).ok()?.as_ref()?;
  // Build invocation tokens: \pgfmath{name}@{arg1}{arg2}...
  let mut tokens: Vec<Token> = vec![cs_tok];
  for arg in args {
    let s = pgfmath_result_str(*arg);
    tokens.push(T_BEGIN!());
    tokens.extend(Explode!(s));
    tokens.push(T_END!());
  }
  // Reset the global `\ifpgfmathunitsdeclared` flag BEFORE digesting the body
  // so that, afterwards, the flag reflects ONLY whether THIS function body
  // parsed a unit'd value. A user function body may run a nested
  // `\pgfmathparse` (e.g. pgfplots' `pgfplotsbarwidthgeneric` body parses the
  // `<bar width>pt` dimension); the outer parser must inherit that via
  // `absorb_units_flag` so it doesn't clobber the flag back to false on exit.
  // Without the reset a prior parse's stale flag would leak in for bodies that
  // set `\pgfmathresult` directly (e.g. `\def\pgfmathresult{42}`). Mirrors
  // Perl, where `\ifpgfmathunitsdeclared` is a persistent global threaded
  // through nested evaluations. Fixes `bar shift={\pgfplotbarwidth}` under
  // `symbolic x coords` (2110.14597).
  let _ = stomach::digest(Tokens::from(vec![T_CS!("\\pgfmathunitsdeclaredfalse")]));
  // Digest the invocation (sets \pgfmathresult)
  let _ = stomach::digest(Tokens::from(tokens));
  // Read back \pgfmathresult via expansion
  let result_tokens = gullet::do_expand(Tokens::from(vec![T_CS!("\\pgfmathresult")])).ok()?;
  let s = result_tokens.to_string();
  s.trim().parse::<f64>().ok()
}

/// Check if a name is a user-defined pgf constant (arity 0)
/// Perl: sub pgfmath_checkuserconstant (L540-546)
fn is_user_constant(name: &str) -> bool {
  let cs = format!("\\pgfmath@function@{}", name);
  let tok = Token {
    text: arena::pin(&cs),
    code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    };
  if state::lookup_definition(&tok).ok().flatten().is_none() {
    return false;
  }
  // Check arity — must be 0 for constant
  let arity_cs = format!("\\pgfmath@operation@{}@arity", name);
  let arity_tok = Token {
    text: arena::pin(&arity_cs),
    code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    };
  if let Ok(Some(_)) = state::lookup_definition(&arity_tok) {
    if let Ok(expanded) = gullet::do_expand(Tokens::from(vec![arity_tok])) {
      let s = expanded.to_string();
      let arity: usize = s.trim().parse().unwrap_or(0);
      return arity == 0;
    }
  }
  // No arity defined — treat as constant
  true
}

/// Check if a name is a user-defined pgf function (arity > 0)
/// Perl: sub pgfmath_checkuserfunction (L548-554)
fn is_user_function(name: &str) -> bool {
  let cs = format!("\\pgfmath@function@{}", name);
  let tok = Token {
    text: arena::pin(&cs),
    code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    };
  if state::lookup_definition(&tok).ok().flatten().is_none() {
    return false;
  }
  let arity_cs = format!("\\pgfmath@operation@{}@arity", name);
  let arity_tok = Token {
    text: arena::pin(&arity_cs),
    code: Catcode::CS,
      #[cfg(feature = "token-locators")] loc: 0
    };
  if let Ok(Some(_)) = state::lookup_definition(&arity_tok) {
    if let Ok(expanded) = gullet::do_expand(Tokens::from(vec![arity_tok])) {
      let s = expanded.to_string();
      let arity: usize = s.trim().parse().unwrap_or(0);
      return arity > 0;
    }
  }
  false
}

// ==================== Recursive Descent Parser ====================
// Faithful translation of the Parse::RecDescent grammar from
// pgfmath.code.tex.ltxml L653-732
//
// Grammar rules:
//   formula     : expr ? expr : expr | expr CMP expr | expr
//   expr        : term (ADDOP term)*
//   term        : factor (MULOP factor)*
//   factor      : simplefactor ^ simplefactor | simplefactor postfix*
//   simplefactor: ( formula ) | PREFIX simplefactor | FUNCTION(...) |
//                 FUNCTION0 | NUMBER UNIT | NUMBER | REGISTER

/// Read the global `\ifpgfmathunitsdeclared` boolean — true iff the most
/// recent (possibly nested) evaluation parsed a unit'd value. The winnow
/// grammar inherits this after a user function/constant via
/// `pgfmath_grammar::absorb_units_flag`; see `pgfmath_apply_user`, which resets
/// the flag before digesting each user body. Mirrors Perl, where
/// `\ifpgfmathunitsdeclared` is a persistent global threaded through nested
/// evaluations. Lives at file scope (not in the grammar module) because
/// `if_condition` / `T_CS!` come from the crate prelude here.
fn pgfmath_units_declared_global() -> bool {
  if_condition(&T_CS!("\\ifpgfmathunitsdeclared")).unwrap_or(None) == Some(true)
}

// ── pgfmath grammar (winnow; #171 family) ────────────────────────────────
// A 1:1 port of the battle-tested hand-written recdescent above-replaced
// (which itself mirrored Perl pgfmath.code.tex.ltxml's Parse::RecDescent
// grammar, `<skip:'[\s\{\}]*'>`), preserving its post-port improvements —
// gated by the 64-expression golden corpus + the 85_pgf/86_tikz suites.
// State (`units_declared`) rides the winnow `Stateful` input and is NOT
// rolled back on backtrack, matching the original's flag semantics.
mod pgfmath_grammar {
  use super::{pgfmath_apply_fn, pgfmath_cmp_op, pgfmath_convert, pgfmath_divisor,
              pgfmath_factorial, pgfmath_register_lookup, is_builtin_constant,
              is_builtin_function, is_user_constant, is_user_function, PGF_UNITS};
  use winnow::error::{ContextError, ErrMode};
  use winnow::prelude::*;
  use winnow::stream::Stream;

  pub(super) type In<'a> = winnow::Stateful<&'a str, Flags>;

  #[derive(Debug, Default, Clone)]
  pub(super) struct Flags {
    pub units_declared: bool,
  }

  fn fail<T>() -> ModalResult<T> { Err(ErrMode::Backtrack(ContextError::new())) }

  /// The grammar-wide skip: `[\s\{\}]*`.
  fn sb(i: &mut In) {
    let n = i
      .input
      .bytes()
      .take_while(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'}'))
      .count();
    i.input = &i.input[n..];
  }
  fn first(i: &In) -> Option<u8> { i.input.as_bytes().first().copied() }
  fn second(i: &In) -> Option<u8> { i.input.as_bytes().get(1).copied() }
  fn bump(i: &mut In, n: usize) { i.input = &i.input[n..]; }
  fn eat(i: &mut In, b: u8) -> bool {
    sb(i);
    if first(i) == Some(b) {
      bump(i, 1);
      true
    } else {
      false
    }
  }

  /// formula := expr ('?' expr ':' expr)? | expr CMPOP expr
  pub(super) fn formula(i: &mut In) -> ModalResult<f64> {
    let left = expr(i)?;
    sb(i);
    if eat(i, b'?') {
      let then_val = expr(i)?;
      sb(i);
      eat(i, b':');
      let else_val = expr(i)?;
      return Ok(if left != 0.0 { then_val } else { else_val });
    }
    let cp = i.checkpoint();
    if let Some(op) = cmp_op(i) {
      if let Ok(right) = expr(i) {
        return Ok(pgfmath_cmp_op(&op, left, right));
      }
      i.reset(&cp);
    }
    Ok(left)
  }

  fn cmp_op(i: &mut In) -> Option<String> {
    sb(i);
    let b = i.input.as_bytes();
    if b.len() >= 2 {
      if let two @ (b"==" | b"!=" | b">=" | b"<=" | b"&&" | b"||") = &[b[0], b[1]] {
        let op = std::str::from_utf8(two.as_slice()).unwrap().to_string();
        bump(i, 2);
        return Some(op);
      }
    }
    match b.first() {
      Some(c @ (b'>' | b'<')) => {
        let op = (*c as char).to_string();
        bump(i, 1);
        Some(op)
      },
      _ => None,
    }
  }

  /// expr := term (('+'|'-') term)*
  fn expr(i: &mut In) -> ModalResult<f64> {
    let mut result = term(i)?;
    loop {
      sb(i);
      match first(i) {
        Some(b'+') => {
          bump(i, 1);
          result += term(i)?;
        },
        Some(b'-') => {
          bump(i, 1);
          result -= term(i)?;
        },
        _ => break,
      }
    }
    Ok(result)
  }

  /// term := factor (('*'|'/') factor)*
  fn term(i: &mut In) -> ModalResult<f64> {
    let mut result = factor(i)?;
    loop {
      sb(i);
      match first(i) {
        Some(b'*') => {
          bump(i, 1);
          result *= factor(i)?;
        },
        Some(b'/') => {
          bump(i, 1);
          result /= pgfmath_divisor(factor(i)?);
        },
        _ => break,
      }
    }
    Ok(result)
  }

  /// factor := simplefactor '^' simplefactor | simplefactor ('!'|'r')*
  fn factor(i: &mut In) -> ModalResult<f64> {
    let base = simplefactor(i)?;
    sb(i);
    if eat(i, b'^') {
      let exp = simplefactor(i)?;
      return Ok(base.powf(exp));
    }
    let mut result = base;
    loop {
      sb(i);
      match first(i) {
        Some(b'!') if second(i) != Some(b'=') => {
          bump(i, 1);
          result = pgfmath_factorial(result as i64);
        },
        Some(b'r') if !second(i).map(|c| c.is_ascii_alphabetic()).unwrap_or(false) => {
          bump(i, 1);
          result = result.to_degrees();
        },
        _ => break,
      }
    }
    Ok(result)
  }

  /// simplefactor — the atom: parens, prefix ops, numbers (with unit/register
  /// suffix), registers, identifiers (functions/constants), lone dot.
  fn simplefactor(i: &mut In) -> ModalResult<f64> {
    sb(i);
    match first(i) {
      None => return fail(),
      Some(b'(') => {
        bump(i, 1);
        let val = formula(i)?;
        sb(i);
        eat(i, b')'); // forgive unclosed at end (Perl)
        return Ok(val);
      },
      Some(b'-') => {
        bump(i, 1);
        return Ok(-simplefactor(i)?);
      },
      Some(b'+') => {
        bump(i, 1);
        return simplefactor(i);
      },
      Some(b'!') => {
        if second(i) == Some(b'=') {
          return fail();
        }
        bump(i, 1);
        let val = simplefactor(i)?;
        return Ok(if val == 0.0 { 1.0 } else { 0.0 });
      },
      _ => {},
    }

    if let Some(num) = try_number(i) {
      sb(i);
      if let Some(unit) = try_unit(i) {
        i.state.units_declared = true;
        return Ok(pgfmath_convert(num, &unit));
      }
      if first(i) == Some(b'\\') {
        let cp = i.checkpoint();
        if let Some(reg) = try_cs_register(i) {
          return Ok(num * reg);
        }
        i.reset(&cp);
      }
      return Ok(num);
    }

    if first(i) == Some(b'\\') {
      if let Some(reg) = try_cs_register(i) {
        sb(i);
        if first(i) == Some(b'\\') {
          let cp = i.checkpoint();
          if let Some(reg2) = try_cs_register(i) {
            return Ok(reg * reg2);
          }
          i.reset(&cp);
        }
        return Ok(reg);
      }
    }

    if first(i).map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
      let name = read_identifier(i);
      if is_builtin_function(&name) {
        return function_call(i, &name);
      }
      if is_builtin_constant(&name) {
        return Ok(pgfmath_apply_fn(&name, &[]));
      }
      if is_user_function(&name) {
        // A user function whose TeX body may declare units — inherit them
        // afterwards (see `absorb_units_flag`). Builtin functions above must
        // NOT absorb: they compute purely in Rust and never touch the global
        // `\ifpgfmathunitsdeclared`, so reading it could only pick up a STALE
        // true from an earlier evaluation and wrongly mark e.g. `sin(30)` as
        // unit'd. (Real `\pgfmathparse` resets the global at parse-start, which
        // is why #249's recdescent — which absorbed for builtins too — never
        // tripped; the golden corpus runs evals back-to-back without that
        // reset and exposes the staleness.)
        let v = function_call(i, &name)?;
        absorb_units_flag(i);
        return Ok(v);
      }
      if is_user_constant(&name) {
        // A 0-arg user pseudo-constant whose body may parse a unit'd value
        // (e.g. `pgfplotsbarwidthgeneric`). Inherit its units flag — see
        // `absorb_units_flag`. Builtin constants above are pure and need no
        // absorb.
        let v = pgfmath_apply_fn(&name, &[]);
        absorb_units_flag(i);
        return Ok(v);
      }
      return fail();
    }

    if first(i) == Some(b'.') {
      bump(i, 1);
      return Ok(0.0);
    }
    fail()
  }

  /// After evaluating a USER-declared pgfmath function/constant (NOT a
  /// builtin), inherit the global `\ifpgfmathunitsdeclared` flag that its body
  /// may have set (its body runs with the flag reset by `pgfmath_apply_user`,
  /// then a nested `\pgfmathparse` of a unit'd value sets it). Mirrors Perl,
  /// where the flag is a persistent global threaded through nested evaluations.
  /// Monotonic — only ever sets the flag, matching `try_cs_register`'s
  /// unconditional set. Call ONLY from the user-function / user-constant
  /// dispatch arms: builtins never touch the global, so absorbing after one
  /// would leak a stale true into a dimensionless result. Fixes
  /// `bar shift={\pgfplotbarwidth}` under `symbolic x coords` (2110.14597):
  /// `\pgfplotbarwidth` → `pgfplotsbarwidthgeneric` (a 0-arg pseudo-constant)
  /// whose body parses the bar-width dimension.
  fn absorb_units_flag(i: &mut In) {
    if super::pgfmath_units_declared_global() {
      i.state.units_declared = true;
    }
  }

  /// FUNCTION '(' formula (',' formula)* ')' | FUNCTION simplefactor
  fn function_call(i: &mut In, name: &str) -> ModalResult<f64> {
    sb(i);
    if first(i) == Some(b'(') {
      bump(i, 1);
      let mut args = vec![formula(i)?];
      loop {
        sb(i);
        if eat(i, b',') {
          if let Ok(arg) = formula(i) {
            args.push(arg);
          } else {
            break;
          }
        } else {
          break;
        }
      }
      sb(i);
      eat(i, b')');
      return Ok(pgfmath_apply_fn(name, &args));
    }
    let arg = simplefactor(i)?;
    Ok(pgfmath_apply_fn(name, &[arg]))
  }

  /// `(\d+\.?[\d.]*|\d*\.?\d+)([eE][+-]?\d+)?` with hex/binary forms and
  /// the embedded-dot tolerance ("1.2.3"), exactly as the recdescent had it.
  fn try_number(i: &mut In) -> Option<f64> {
    sb(i);
    let b = i.input.as_bytes();
    if b.is_empty() {
      return None;
    }
    if b.len() >= 2 && b[0] == b'0' && b[1] == b'x' {
      let n = b[2..].iter().take_while(|c| c.is_ascii_hexdigit()).count();
      if n == 0 {
        return None;
      }
      let v = i64::from_str_radix(&i.input[2..2 + n], 16).unwrap_or(0) as f64;
      bump(i, 2 + n);
      return Some(v);
    }
    if b.len() >= 2 && b[0] == b'0' && b[1] == b'b' {
      let n = b[2..].iter().take_while(|c| matches!(c, b'0' | b'1')).count();
      if n == 0 {
        return None;
      }
      let v = i64::from_str_radix(&i.input[2..2 + n], 2).unwrap_or(0) as f64;
      bump(i, 2 + n);
      return Some(v);
    }
    let mut pos = 0;
    let mut has_digit = false;
    while pos < b.len() && b[pos].is_ascii_digit() {
      pos += 1;
      has_digit = true;
    }
    if pos < b.len() && b[pos] == b'.' {
      if has_digit || (pos + 1 < b.len() && b[pos + 1].is_ascii_digit()) {
        pos += 1;
        while pos < b.len() && (b[pos].is_ascii_digit() || b[pos] == b'.') {
          pos += 1;
        }
        has_digit = true;
      } else if has_digit {
        pos += 1;
      }
    }
    if !has_digit || pos == 0 {
      return None;
    }
    if pos < b.len() && (b[pos] == b'e' || b[pos] == b'E') {
      let saved = pos;
      pos += 1;
      if pos < b.len() && (b[pos] == b'+' || b[pos] == b'-') {
        pos += 1;
      }
      if pos < b.len() && b[pos].is_ascii_digit() {
        while pos < b.len() && b[pos].is_ascii_digit() {
          pos += 1;
        }
      } else {
        pos = saved;
      }
    }
    let v = i.input[..pos].parse::<f64>().unwrap_or(0.0);
    bump(i, pos);
    Some(v)
  }

  fn try_unit(i: &mut In) -> Option<String> {
    sb(i);
    for &unit in PGF_UNITS {
      if let Some(rest) = i.input.strip_prefix(unit) {
        // not a unit if a letter follows ("ptx" is not "pt")
        if rest.as_bytes().first().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
          continue;
        }
        bump(i, unit.len());
        return Some(unit.to_string());
      }
    }
    None
  }

  /// `\\[a-zA-Z@]+` — a TeX register; reading one declares units (see the
  /// pgfplots ybar witnesses on the original implementation).
  fn try_cs_register(i: &mut In) -> Option<f64> {
    sb(i);
    let b = i.input.as_bytes();
    if b.first() != Some(&b'\\') {
      return None;
    }
    let n = b[1..].iter().take_while(|c| c.is_ascii_alphabetic() || **c == b'@').count();
    if n == 0 {
      return None;
    }
    let cs = &i.input[..1 + n];
    let value = pgfmath_register_lookup(cs);
    bump(i, 1 + n);
    i.state.units_declared = true;
    Some(value)
  }

  fn read_identifier(i: &mut In) -> String {
    let n = i
      .input
      .bytes()
      .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
      .count();
    let name = i.input[..n].to_string();
    bump(i, n);
    name
  }
}

/// Apply a comparison operator
fn pgfmath_cmp_op(op: &str, left: f64, right: f64) -> f64 {
  let result = match op {
    "==" => left == right,
    "!=" => left != right,
    ">" => left > right,
    "<" => left < right,
    ">=" => left >= right,
    "<=" => left <= right,
    "&&" => left != 0.0 && right != 0.0,
    "||" => left != 0.0 || right != 0.0,
    _ => false,
  };
  if result { 1.0 } else { 0.0 }
}

// ==================== Main pgfmathparse ====================

/// Format the result of pgfmathparse for output
/// Perl: L383-393 of pgfmath.code.tex.ltxml
fn format_parse_result(result: f64, input: &str) -> String {
  // A non-finite result — NaN from `sqrt(-1)`/`ln(-1)`/`asin(5)`, or ±inf from
  // an overflow — must NOT serialize as the literal "NaN"/"inf": that string
  // poisons every downstream `\pgfmathresult` dimension read. Mirror the
  // sibling `pgfmath_result_str` guard (L21) and degrade to 0; the public path
  // keeps the `.0`. NB `clamp` below does NOT rescue NaN (`NaN.clamp(..) ==
  // NaN`), so this must come first. Review M4.
  if result.is_nan() || result.is_infinite() {
    return "0.0".to_string();
  }
  // Overflow check
  let result = result.clamp(-MAX_PGF_NUMBER, MAX_PGF_NUMBER);

  // Integer result
  if result == (result as i64) as f64 {
    let i = result as i64;
    // Perl: $result .= '.0' unless $input =~ /^int\(/;
    if input.starts_with("int(") {
      return format!("{}", i);
    }
    return format!("{}.0", i);
  }

  // Decimal result: sprintf("%.5f"), strip trailing zeros
  let mut s = format!("{:.5}", result);
  if s.contains('.') {
    while s.ends_with('0') && !s.ends_with(".0") {
      s.pop();
    }
  }
  s
}

/// Expand the argument to a pgfmath parse with the seven calc-package
/// compatibility CSes (`\real`, `\minof`, `\maxof`, `\ratio`, `\widthof`,
/// `\heightof`, `\depthof`) transiently `\let` to the `\pgfmath@calc@*`
/// internals, then restored to their prior meanings.
///
/// Perl `pgfmath.code.tex.ltxml` L320-327 performs these `Let`s at the *start
/// of `sub pgfmathparse`* — i.e. only while the pgfmath argument is being
/// expanded; being local (non-`global`) `Let`s they revert with the enclosing
/// tikz/pgf group. The earlier Rust port hoisted them to package-load time
/// (because the native parser "can't re-bind per call"), which globally
/// clobbered `\real` (and the six siblings) for the *whole document*. That
/// broke the very common use of `\real` as the blackboard-ℝ symbol: e.g.
/// `\int_\real p_m`, where the 1-argument `\pgfmath@calc@real` ate the
/// following `p` as its argument and yielded a spurious "Double subscript".
/// Witness: 1608.06741 (`\newcommand\real{\mathbb{R}}`, ignored because
/// mathtools→calc already defined `\real`, then pgfmath via todonotes→tikz
/// clobbered it). We restore Perl's exact scope with a tight
/// save → let → expand → restore around just this one expansion, so `\real`
/// used as an ordinary math macro elsewhere is never touched.
fn expand_pgfmath_arg<T: Into<Tokens>>(tokens: T) -> Tokens {
  const PAIRS: [(&str, &str); 7] = [
    ("\\real", "\\pgfmath@calc@real"),
    ("\\minof", "\\pgfmath@calc@minof"),
    ("\\maxof", "\\pgfmath@calc@maxof"),
    ("\\ratio", "\\pgfmath@calc@ratio"),
    ("\\widthof", "\\pgfmath@calc@widthof"),
    ("\\heightof", "\\pgfmath@calc@heightof"),
    ("\\depthof", "\\pgfmath@calc@depthof"),
  ];
  let saved: Vec<(Token, Stored)> = PAIRS
    .iter()
    .map(|(cs, _)| {
      let t = T_CS!(*cs);
      let m = state::lookup_meaning(&t).unwrap_or(Stored::None);
      (t, m)
    })
    .collect();
  for (cs, helper) in PAIRS.iter() {
    state::let_i(&T_CS!(*cs), &T_CS!(*helper), None);
  }
  let expanded = gullet::do_expand(tokens).unwrap_or_default();
  for (t, m) in saved {
    state::assign_meaning(&t, m, None);
  }
  expanded
}

/// Main pgfmathparse evaluation function
/// Perl: sub pgfmathparse (L316-394)
/// Returns (result_string, units_declared)
pub(crate) fn pgfmathparse_eval_with_units(raw_input: &str) -> (String, bool) {
  // Normalize whitespace
  let input: String = raw_input.split_whitespace().collect::<Vec<_>>().join(" ");
  let input = input.trim();

  // 0. String-valued ternary / ifthenelse — Perl pgfmath uses untyped
  //    scalars so the `?:` operator and `ifthenelse(...)` can return
  //    string values like `"black"` or `"pgreen!50"` for color names.
  //    Our parser is f64-only, so detect the pattern
  //      <test> ? "a" : "b"     OR     ifthenelse(<test>, "a", "b")
  //    early, evaluate the test as a number, and return the chosen
  //    string literal. Drivers: 2601.14798 (color jitter heatmap),
  //    Stage-20 v6 ~1579 errors from \pgfmathsetmacro{\clr}{...}.
  if let Some(result) = try_string_ternary(input) {
    return (result, false);
  }

  // 1. Simple number check (Perl L332-345)
  if let Some(result) = try_simple_number(input) {
    return (result, false);
  }

  // 2. Unit expression check (Perl L352-354): /^([+-]?[\d\.]+)(UNIT)$/ Handled by the parser below,
  //    since it's a subset of the grammar.

  // 3. Parse with the winnow grammar (replaces both Perl eval and RecDescent;
  //    partial parses still return what was read, as before).
  let mut stream = winnow::Stateful { input, state: pgfmath_grammar::Flags::default() };
  if let Ok(result) = pgfmath_grammar::formula(&mut stream) {
    return (format_parse_result(result, input), stream.state.units_declared);
  }

  // 4. Fallback
  ("0.0".to_string(), false)
}

/// Convenience wrapper that returns only the result string
pub fn pgfmathparse_eval(raw_input: &str) -> String { pgfmathparse_eval_with_units(raw_input).0 }

/// Detect string-valued pgfmath ternaries `<test> ? "a" : "b"` or
/// `ifthenelse(<test>, "a", "b")` and return the chosen literal.
/// Returns None if the pattern doesn't match or contains no string args.
fn try_string_ternary(input: &str) -> Option<String> {
  // Strip ifthenelse(...) wrapper if present
  let inner = if let Some(rest) = input.strip_prefix("ifthenelse(") {
    let body = rest.strip_suffix(')')?;
    // Re-shape as ternary: split on top-level commas (depth 0 of nested parens/quotes)
    let parts = split_top_commas(body)?;
    if parts.len() != 3 { return None; }
    format!("{} ? {} : {}", parts[0].trim(), parts[1].trim(), parts[2].trim())
  } else {
    input.to_string()
  };
  // Find top-level `?` and `:` (skipping inside quotes and parens)
  let (cond, then_part, else_part) = split_ternary(&inner)?;
  // Branches must include at least one quoted string for this path to make sense
  let then_is_str = then_part.trim().starts_with('"');
  let else_is_str = else_part.trim().starts_with('"');
  if !then_is_str && !else_is_str {
    return None;
  }
  // Evaluate the test as a number via the regular f64 grammar
  let mut stream =
    winnow::Stateful { input: cond.trim(), state: pgfmath_grammar::Flags::default() };
  let test_val = pgfmath_grammar::formula(&mut stream).ok()?;
  let chosen = if test_val != 0.0 { then_part } else { else_part };
  // Strip outer quotes if present; otherwise return as-is
  let trimmed = chosen.trim();
  let unq = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
    &trimmed[1..trimmed.len()-1]
  } else {
    trimmed
  };
  Some(unq.to_string())
}

/// Split a string on commas at the top level (depth 0), skipping
/// nested parens/brackets and quoted substrings.
fn split_top_commas(s: &str) -> Option<Vec<&str>> {
  let bytes = s.as_bytes();
  let mut parts = Vec::new();
  let mut start = 0usize;
  let mut depth: i32 = 0;
  let mut in_quote = false;
  for (i, &b) in bytes.iter().enumerate() {
    match b {
      b'"' => in_quote = !in_quote,
      b'(' | b'[' | b'{' if !in_quote => depth += 1,
      b')' | b']' | b'}' if !in_quote => depth -= 1,
      b',' if !in_quote && depth == 0 => {
        parts.push(&s[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  parts.push(&s[start..]);
  Some(parts)
}

/// Split `cond ? then : else` at the top-level `?` and `:`.
/// Skips inside quotes and parens. Returns (cond, then, else).
fn split_ternary(s: &str) -> Option<(&str, &str, &str)> {
  let bytes = s.as_bytes();
  let mut depth: i32 = 0;
  let mut in_quote = false;
  let mut q_pos: Option<usize> = None;
  for (i, &b) in bytes.iter().enumerate() {
    match b {
      b'"' => in_quote = !in_quote,
      b'(' | b'[' | b'{' if !in_quote => depth += 1,
      b')' | b']' | b'}' if !in_quote => depth -= 1,
      b'?' if !in_quote && depth == 0 => { q_pos = Some(i); break; },
      _ => {},
    }
  }
  let qp = q_pos?;
  // Now find the matching `:` after qp at depth 0
  let mut depth: i32 = 0;
  let mut in_quote = false;
  for (j, &b) in bytes.iter().enumerate().skip(qp + 1) {
    match b {
      b'"' => in_quote = !in_quote,
      b'(' | b'[' | b'{' if !in_quote => depth += 1,
      b')' | b']' | b'}' if !in_quote => depth -= 1,
      b':' if !in_quote && depth == 0 => {
        return Some((&s[..qp], &s[qp+1..j], &s[j+1..]));
      },
      _ => {},
    }
  }
  None
}

// ==================== Macro Definitions ====================

#[rustfmt::skip]
LoadDefinitions!({
  // Load pgf's raw TeX math code first (reloadable to bypass the loaded guard,
  // since our binding was already marked as loaded by the dispatch system)
  InputDefinitions!("pgfmath.code", extension => Some(Cow::Borrowed("tex")), noltxml => true, reloadable => true);

  // Then override math operations with native Rust implementations.
  // Perl: "using pgflibraryluamath.code.tex as a guide for what needs doing"

  // Conditionals needed by pgfmath
  RawTeX!(r"\lx@ifundefined{pgfmathunitsdeclaredtrue}{\newif\ifpgfmathunitsdeclared}{}");
  RawTeX!(r"\lx@ifundefined{pgfmathmathunitsdeclaredtrue}{\newif\ifpgfmathmathunitsdeclared}{}");

  // Constants
  DefMacro!("\\pgfmathpi@", sub[_args] {
    pgfmath_result_tokens(PI)
  });
  DefMacro!("\\pgfmathe@", sub[_args] {
    pgfmath_result_tokens(std::f64::consts::E)
  });

  // Basic arithmetic
  DefMacro!("\\pgfmathadd@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a) + parse_pgf_number(&b))
  });
  DefMacro!("\\pgfmathsubtract@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a) - parse_pgf_number(&b))
  });
  DefMacro!("\\pgfmathneg@ {}", sub[(a)] {
    pgfmath_result_tokens(-parse_pgf_number(&a))
  });
  DefMacro!("\\pgfmathmultiply@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a) * parse_pgf_number(&b))
  });
  DefMacro!("\\pgfmathdivide@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a) / pgfmath_divisor(parse_pgf_number(&b)))
  });
  DefMacro!("\\pgfmathpow@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a).powf(parse_pgf_number(&b)))
  });

  // Rounding and comparison
  DefMacro!("\\pgfmathabs@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).abs())
  });
  DefMacro!("\\pgfmathround@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).round())
  });
  DefMacro!("\\pgfmathfloor@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).floor())
  });
  DefMacro!("\\pgfmathceil@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).ceil())
  });

  // Trigonometric (input in degrees, output as float)
  DefMacro!("\\pgfmathsin@ {}", sub[(a)] {
    pgfmath_result_tokens(pgfmath_arg_radians(parse_pgf_number(&a)).sin())
  });
  DefMacro!("\\pgfmathcos@ {}", sub[(a)] {
    pgfmath_result_tokens(pgfmath_arg_radians(parse_pgf_number(&a)).cos())
  });
  DefMacro!("\\pgfmathtan@ {}", sub[(a)] {
    pgfmath_result_tokens(pgfmath_arg_radians(parse_pgf_number(&a)).tan())
  });
  DefMacro!("\\pgfmathatan@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).atan().to_degrees())
  });
  DefMacro!("\\pgfmathatantwo@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a).atan2(parse_pgf_number(&b)).to_degrees())
  });
  Let!("\\pgfmathatan2@", "\\pgfmathatantwo@");
  DefMacro!("\\pgfmathasin@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).asin().to_degrees())
  });
  DefMacro!("\\pgfmathacos@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).acos().to_degrees())
  });
  DefMacro!("\\pgfmathcot@ {}", sub[(a)] {
    let r = pgfmath_arg_radians(parse_pgf_number(&a));
    pgfmath_result_tokens(r.cos() / pgfmath_divisor(r.sin()))
  });
  DefMacro!("\\pgfmathsec@ {}", sub[(a)] {
    pgfmath_result_tokens(1.0 / pgfmath_divisor(pgfmath_arg_radians(parse_pgf_number(&a)).cos()))
  });
  DefMacro!("\\pgfmathcosec@ {}", sub[(a)] {
    pgfmath_result_tokens(1.0 / pgfmath_divisor(pgfmath_arg_radians(parse_pgf_number(&a)).sin()))
  });

  // Exponential and logarithmic
  DefMacro!("\\pgfmathexp@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).exp())
  });
  DefMacro!("\\pgfmathln@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).ln())
  });
  DefMacro!("\\pgfmathlogten@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).ln() / LOG10)
  });
  DefMacro!("\\pgfmathsqrt@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).sqrt())
  });

  // Random
  DefMacro!("\\pgfmathrnd@", sub[_args] {
    pgfmath_result_tokens(0.5) // deterministic for reproducibility
  });
  DefMacro!("\\pgfmathrand@", sub[_args] {
    pgfmath_result_tokens(0.0) // deterministic
  });

  DefMacro!("\\pgfmathreciprocal@ {}", sub[(a)] {
    pgfmath_result_tokens(1.0 / pgfmath_divisor(parse_pgf_number(&a)))
  });

  // Modular arithmetic
  DefMacro!("\\pgfmathmod@ {} {}", sub[(a, b)] {
    let a = parse_pgf_number(&a);
    let b = parse_pgf_number(&b);
    // Perl: pgfmath_mod_floor — mod towards -infinity
    let result = if a / b < 0.0 {
      -(a.abs() % b.abs()) + b.abs()
    } else {
      a.abs() % b.abs()
    };
    pgfmath_result_tokens(result)
  });

  // Degree/radian conversion
  DefMacro!("\\pgfmathrad@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).to_radians())
  });
  DefMacro!("\\pgfmathdeg@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).to_degrees())
  });

  // Integer operations
  DefMacro!("\\pgfmathint@ {}", sub[(a)] {
    let v = parse_pgf_number(&a) as i64;
    pgfmath_result_tokens(v as f64)
  });

  // Calc package compatibility
  DefMacro!("\\pgfmath@calc@real{}", "#1");
  DefMacro!("\\pgfmath@calc@minof{}{}", "min(#1,#2)");
  DefMacro!("\\pgfmath@calc@maxof{}{}", "max(#1,#2)");
  DefMacro!("\\pgfmath@calc@ratio{}{}", "#1/#2");
  DefMacro!("\\pgfmath@calc@widthof{}", "width(\"#1\")");
  DefMacro!("\\pgfmath@calc@heightof{}", "height(\"#1\")");
  DefMacro!("\\pgfmath@calc@depthof{}", "depth(\"#1\")");

  // Perl pgfmath.code.tex.ltxml L320-327: the seven calc-package CSes
  // (`\real`, `\minof`, `\maxof`, `\ratio`, `\widthof`, `\heightof`,
  // `\depthof`) are `Let` to the `\pgfmath@calc@*` internals *inside*
  // `sub pgfmathparse` — transiently, per parse, with local scope. They are
  // deliberately NOT bound at package-load time: doing so globally clobbers
  // `\real` (commonly `\mathbb{R}`) and friends for the whole document. See
  // `expand_pgfmath_arg`, which replicates the exact transient scope around
  // the pgfmath argument expansion.

  // ==================== pgfmathparse override ====================
  // Perl L401-403: DefMacro('\lx@pgfmath@parse{}', sub { ... })
  DefMacro!("\\lx@pgfmath@parse {}", sub[(tokens)] {
    // Expand tokens and convert to string. The seven calc-compat CSes are
    // transiently bound only for this expansion (Perl `sub pgfmathparse`).
    let expanded = expand_pgfmath_arg(tokens);
    let input = expanded.to_string();
    let (result, units) = pgfmathparse_eval_with_units(&input);
    let mut toks = pgfmath_result_tokens_str(&result);
    // Perl: \ifpgfmathunitsdeclared flag — used by tikz's \tikz@checkunit
    if units {
      toks.push(T_CS!("\\pgfmathunitsdeclaredtrue"));
    } else {
      toks.push(T_CS!("\\pgfmathunitsdeclaredfalse"));
    }
    toks
  });

  // Perl L442: Let('\pgfmathparse', '\lx@pgfmath@parse');
  Let!("\\pgfmathparse", "\\lx@pgfmath@parse");

  // Perl L396-399: \lx@pgfmath@parseX (alternative that uses \lx@pgfmathresult)
  DefMacro!("\\lx@pgfmath@parseX {}", sub[(tokens)] {
    let expanded = expand_pgfmath_arg(tokens);
    let input = expanded.to_string();
    let result = pgfmathparse_eval(&input);
    let mut toks = vec![T_CS!("\\def"), T_CS!("\\lx@pgfmathresult"), T_BEGIN!()];
    toks.extend(Explode!(result));
    toks.push(T_END!());
    toks
  });

  // Perl L425: save original \pgfmathparse
  Let!("\\@orig@pgfmathparse", "\\pgfmathparse");

  // Override \pgfmathsincos to use our native sin/cos (for direct calls)
  DefMacro!("\\pgfmathsincos{}",
    "\\pgfmathparse{sin(#1)}\\let\\pgfmathresulty\\pgfmathresult\\pgfmathparse{cos(#1)}\\let\\pgfmathresultx\\pgfmathresult");

  // ==================== \pgfmathsetlength ====================
  // Perl L405-423: DefPrimitive('\pgfmathsetlength DefToken {}', sub { ... })
  //
  // The raw TeX \pgfmathsetlength uses \pgfmath@onquick with \pgfmath@ as a
  // delimiter token. Our engine can choke on this delimiter pattern, causing
  // "undefined:\pgfmath@" errors that cascade into group mismatches. The Perl
  // binding overrides this with a native implementation that:
  //   1. Strips leading spaces from #2
  //   2. If #2 starts with '+', treats it as plain glue (readGlue)
  //   3. Otherwise, evaluates via pgfmathparse and converts to Dimension
  //   4. Assigns the result to register #1
  DefPrimitive!("\\pgfmathsetlength DefToken {}", sub[(register, tokens)] {
    let toks: Vec<Token> = tokens.unlist_ref().iter()
      .skip_while(|t| t.get_catcode() == Catcode::SPACE)
      .cloned()
      .collect();

    if toks.first().is_some_and(|t| t.text == pin!("+")) {
      // Starts with '+' → treat as plain glue
      // Unread the tokens and read as glue
      gullet::unread(Tokens::new(toks));
      let glue = gullet::read_glue()?;
      let cs = register.to_string();
      state::assign_register(&cs, glue.into(), None, vec![])?;
    } else {
      // Evaluate via pgfmathparse
      let tok_str = Tokens::new(toks);
      let expanded = gullet::do_expand(tok_str).unwrap_or_default();
      let input = expanded.to_string();
      let (result_str, _units) = pgfmathparse_eval_with_units(&input);
      let value: f64 = result_str.parse().unwrap_or(0.0);

      // Perl L419-422: check \ifpgfmathmathunitsdeclared for mu units
      let cs = register.to_string();
      let is_mu = if_condition(&T_CS!("\\ifpgfmathmathunitsdeclared"))
        .unwrap_or(None) == Some(true);
      if is_mu {
        let mu_sp = state::convert_unit("mu");
        let mu_dim = Dimension((value * mu_sp as f64).round() as i64);
        state::assign_register(&cs, mu_dim.into(), None, vec![])?;
      } else {
        // Convert pgfmath result (in pt) to sp (× 65536), round to nearest
        let dim = Dimension((value * 65536.0).round() as i64);
        state::assign_register(&cs, dim.into(), None, vec![])?;
      }
    }
  });

  // Also override \pgfmathsetlengthmacro (Perl L426 area — uses same approach
  // but assigns to a macro instead of a register; we approximate by just doing
  // pgfmathparse and \def-ing the result)

  // ==================== \pgfmath@smuggleone ====================
  // Perl L285-299: DefMacro('\pgfmath@smuggleone Until:\endgroup', sub { ... })
  //
  // The raw TeX definition is: \def\pgfmath@smuggleone#1\endgroup{...}
  // It "smuggles" a definition out of a group by expanding before \endgroup.
  // Perl overrides: for expandables, emit the expansion chain; for primitives,
  // just emit \endgroup (bindings are already global, no smuggling needed).
  DefMacro!("\\pgfmath@smuggleone Until:\\endgroup", sub[(arg)] {
    // The arg is everything up to \endgroup. Extract the first meaningful token.
    let first_tok = arg.unlist_ref().iter()
      .find(|t| {
        let cc = t.get_catcode();
        cc != Catcode::SPACE && cc != Catcode::COMMENT && cc != Catcode::MARKER
      })
      .cloned();
    let mut smuggle = false;
    let mut first_cs = T_CS!("\\relax"); // placeholder
    if let Some(first) = first_tok {
      if let Ok(Some(defn)) = state::lookup_definition(&first) {
        if defn.is_expandable() {
          smuggle = true;
          first_cs = first;
        }
      }
    }
    if smuggle {
      // Texlive 2020 definition: smuggle by expanding before endgroup
      vec![
        T_CS!("\\expandafter"), T_CS!("\\endgroup"),
        T_CS!("\\expandafter"), T_CS!("\\def"),
        T_CS!("\\expandafter"), first_cs,
        T_CS!("\\expandafter"), T_BEGIN!(),
        first_cs,
        T_END!(),
      ]
    } else {
      // For primitives/bindings: already global, just close the group
      vec![T_CS!("\\endgroup")]
    }
  });

});

#[cfg(test)]
mod pgfmath_golden_tests {
  fn describe(expr: &str) -> String {
    let (result, units) = super::pgfmathparse_eval_with_units(expr);
    format!("{result}@{}", if units { "u" } else { "-" })
  }

  /// Golden corpus pinning the battle-tested recdescent's behavior
  /// (captured 2026-06-10) — incl. its deliberate quirks (ternary dropped
  /// after a comparison parse, pass-through formatting of "+3"/"3.").
  /// The gate for the winnow grammar: divergence = regression.
  #[test]
  fn golden_pgfmath_corpus() {
    latexml_core::state::set_state(latexml_core::state::State::new(
      latexml_core::state::StateOptions::default(),
    ));
    let golden: &[(&str, &str)] = &[
      ("1+1", "2.0@-"),
      ("2+3*4", "14.0@-"),
      ("2*3+4", "10.0@-"),
      ("10/4", "2.5@-"),
      ("2^10", "1024.0@-"),
      ("(1+2)*3", "9.0@-"),
      ("-5+2", "-3.0@-"),
      ("+3", "3@-"),
      ("--2", "2.0@-"),
      ("3!", "6.0@-"),
      ("1.5^2", "2.25@-"),
      ("7/2", "3.5@-"),
      ("1/3", "0.33333@-"),
      ("57.29577951r", "3282.80635@-"),
      ("0x1F", "31.0@-"),
      ("0b101", "5.0@-"),
      (".5", "0.5@-"),
      ("3.", "3.@-"),
      ("1.2e3", "1200.0@-"),
      ("1e-2", "0.01@-"),
      ("2.5E+2", "250.0@-"),
      (".", "0.0@-"),
      ("2pt", "2.0@u"),
      ("1cm", "28.45276@u"),
      ("10mm", "28.45276@u"),
      ("1in", "72.27@u"),
      ("3bp", "3.01125@u"),
      ("1dd", "1.07001@u"),
      ("1cc", "12.8401@u"),
      ("100sp", "0.00153@u"),
      ("3>2", "1.0@-"),
      ("2>=3", "0.0@-"),
      ("1==1", "1.0@-"),
      ("1!=2", "1.0@-"),
      ("1&&0", "0.0@-"),
      ("0||1", "1.0@-"),
      ("3>2 ? 10 : 20", "1.0@-"),
      ("(1<2)&&(3<4)", "1.0@-"),
      ("!1", "0.0@-"),
      ("!0", "1.0@-"),
      ("sin(30)", "0.5@-"),
      ("cos(60)", "0.5@-"),
      ("tan(45)", "1.0@-"),
      ("min(3,5)", "3.0@-"),
      ("max(3,5)", "5.0@-"),
      ("mod(7,3)", "1.0@-"),
      ("abs(-4)", "4.0@-"),
      ("sqrt(16)", "4.0@-"),
      ("floor(1.7)", "1.0@-"),
      ("ceil(1.2)", "2.0@-"),
      ("round(2.5)", "3.0@-"),
      ("int(3.9)", "3@-"),
      ("exp(1)", "2.71828@-"),
      ("ln(2.718281828)", "1.0@-"),
      ("pow(2,8)", "256.0@-"),
      ("veclen(3,4)", "5.0@-"),
      ("atan(1)", "45.0@-"),
      ("pi", "3.14159@-"),
      ("e", "2.71828@-"),
      ("pi*2", "6.28319@-"),
      ("sin 30", "0.5@-"),
      ("{1}+{2}", "3.0@-"),
      (" 1 + { 2 } ", "3.0@-"),
      ("2*3pt", "6.0@u"),
    ];
    for (expr, expected) in golden {
      assert_eq!(&describe(expr), expected, "pgfmath diverged on {expr:?}");
    }
  }

  /// Review B2: a huge factorial argument must NOT spin the product loop.
  /// `1e20!` saturates `n` to ~9.2e18; the pre-fix code looped that many
  /// times (hang/DoS reachable from `\pgfmathparse{1e20!}`). It must now
  /// return promptly with the Perl overflow value (0). The test itself
  /// completing IS the DoS assertion.
  #[test]
  fn factorial_overflow_is_bounded() {
    latexml_core::state::set_state(latexml_core::state::State::new(
      latexml_core::state::StateOptions::default(),
    ));
    assert_eq!(super::pgfmathparse_eval("1e20!"), "0.0");
    // A few more saturating / large arguments — all overflow → 0, no spin.
    assert_eq!(super::pgfmathparse_eval("1000000!"), "0.0");
    assert_eq!(super::pgfmathparse_eval("100!"), "0.0");
    // Small factorials still compute (table path).
    assert_eq!(super::pgfmathparse_eval("5!"), "120.0");
  }

  /// Review M4: an invalid op yields NaN; the PUBLIC parse path must degrade
  /// it to "0.0", never serialize the literal "NaN" (which would poison a
  /// downstream `\pgfmathresult` dimension read).
  #[test]
  fn nan_results_format_as_zero() {
    latexml_core::state::set_state(latexml_core::state::State::new(
      latexml_core::state::StateOptions::default(),
    ));
    for expr in ["sqrt(-1)", "ln(-1)", "asin(5)", "acos(-5)"] {
      assert_eq!(
        super::pgfmathparse_eval(expr),
        "0.0",
        "non-finite {expr} must format as 0.0, not the literal NaN/inf"
      );
    }
  }
}
