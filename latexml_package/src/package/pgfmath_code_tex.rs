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

/// Format a pgfmath result: 5 decimal places, strip trailing zeros
/// Perl: sub pgfmathresult { sprintf("%.5f", $value); $value =~ s/0+$//; }
fn pgfmath_result_str(value: f64) -> String {
  if value.is_nan() || value.is_infinite() {
    return "0.0".to_string();
  }
  let clamped = value.clamp(-MAX_PGF_NUMBER, MAX_PGF_NUMBER);
  if clamped == (clamped as i64) as f64 {
    // Integer result — pgf prints with .0 suffix
    return format!("{}.0", clamped as i64);
  }
  let mut s = format!("{:.5}", clamped);
  // Strip trailing zeros after decimal point (keep at least one)
  if s.contains('.') {
    while s.ends_with('0') && !s.ends_with(".0") {
      s.pop();
    }
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
    // Original Perl returned 0 (Pgfmath::FALSE); Rust still computes the
    // recursive product (matches what Perl did just-prior-to-error in some
    // versions) but emits the same diagnostic. f64 will saturate to inf
    // around n>=170, so the diagnostic is the user-visible signal.
    let _ = (|| -> Result<()> {
      Error!(
        "pgfmath",
        "overflow",
        format!("Arithmetic overflow: {n_orig}! is too large.")
      );
      Ok(())
    })();
    FACTS[21] * ((22..=n).fold(1.0, |acc, i| acc * i as f64))
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
      // Try user-defined function
      pgfmath_apply_user(name, args).unwrap_or(0.0)
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
  };
  if state::lookup_definition(&tok).ok().flatten().is_none() {
    return false;
  }
  // Check arity — must be 0 for constant
  let arity_cs = format!("\\pgfmath@operation@{}@arity", name);
  let arity_tok = Token {
    text: arena::pin(&arity_cs),
    code: Catcode::CS,
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
  };
  if state::lookup_definition(&tok).ok().flatten().is_none() {
    return false;
  }
  let arity_cs = format!("\\pgfmath@operation@{}@arity", name);
  let arity_tok = Token {
    text: arena::pin(&arity_cs),
    code: Catcode::CS,
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

struct PgfMathParser<'a> {
  input:          &'a [u8],
  pos:            usize,
  /// Set to true when the parsed expression contains dimension units (pt, cm, etc.)
  /// Used by tikz's \tikz@checkunit via \ifpgfmathunitsdeclared
  units_declared: bool,
}

impl<'a> PgfMathParser<'a> {
  fn new(input: &'a str) -> Self {
    Self {
      input:          input.as_bytes(),
      pos:            0,
      units_declared: false,
    }
  }

  /// Skip whitespace and braces (Perl: <skip:'[\s\{\}]*'>)
  fn skip(&mut self) {
    while self.pos < self.input.len() {
      match self.input[self.pos] {
        b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'}' => self.pos += 1,
        _ => break,
      }
    }
  }

  fn try_char(&mut self, c: u8) -> bool {
    self.skip();
    if self.pos < self.input.len() && self.input[self.pos] == c {
      self.pos += 1;
      true
    } else {
      false
    }
  }

  fn remaining_trimmed(&self) -> &str {
    let mut p = self.pos;
    while p < self.input.len() {
      match self.input[p] {
        b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'}' => p += 1,
        _ => break,
      }
    }
    std::str::from_utf8(&self.input[p..]).unwrap_or("")
  }

  // ---- Grammar Rules ----

  /// formula: expr ? expr : expr | expr CMP expr | expr
  fn formula(&mut self) -> Option<f64> {
    let left = self.expr()?;
    self.skip();
    // Ternary: expr ? expr : expr
    if self.try_char(b'?') {
      let then_val = self.expr()?;
      self.skip();
      self.try_char(b':');
      let else_val = self.expr()?;
      return Some(if left != 0.0 { then_val } else { else_val });
    }
    // Comparison operators
    let saved = self.pos;
    if let Some(op) = self.try_cmp_op() {
      if let Some(right) = self.expr() {
        return Some(pgfmath_cmp_op(&op, left, right));
      }
      self.pos = saved;
    }
    Some(left)
  }

  /// Try to match a comparison operator (Perl L726)
  fn try_cmp_op(&mut self) -> Option<String> {
    self.skip();
    if self.pos >= self.input.len() {
      return None;
    }
    // Two-char operators
    if self.pos + 1 < self.input.len() {
      let two = [self.input[self.pos], self.input[self.pos + 1]];
      let op = match &two {
        b"==" | b"!=" | b">=" | b"<=" | b"&&" | b"||" => {
          Some(std::str::from_utf8(&two).unwrap().to_string())
        },
        _ => None,
      };
      if let Some(op) = op {
        self.pos += 2;
        return Some(op);
      }
    }
    // Single-char: > <
    match self.input[self.pos] {
      b'>' | b'<' => {
        let op = (self.input[self.pos] as char).to_string();
        self.pos += 1;
        Some(op)
      },
      _ => None,
    }
  }

  /// expr: term (ADDOP term)*
  fn expr(&mut self) -> Option<f64> {
    let mut result = self.term()?;
    loop {
      self.skip();
      if self.pos >= self.input.len() {
        break;
      }
      match self.input[self.pos] {
        b'+' => {
          self.pos += 1;
          result += self.term()?;
        },
        b'-' => {
          self.pos += 1;
          result -= self.term()?;
        },
        _ => break,
      }
    }
    Some(result)
  }

  /// term: factor (MULOP factor)*
  fn term(&mut self) -> Option<f64> {
    let mut result = self.factor()?;
    loop {
      self.skip();
      if self.pos >= self.input.len() {
        break;
      }
      match self.input[self.pos] {
        b'*' => {
          self.pos += 1;
          result *= self.factor()?;
        },
        b'/' => {
          self.pos += 1;
          result /= pgfmath_divisor(self.factor()?);
        },
        _ => break,
      }
    }
    Some(result)
  }

  /// factor: simplefactor ^ simplefactor | simplefactor postfix*
  fn factor(&mut self) -> Option<f64> {
    let base = self.simplefactor()?;
    self.skip();
    // Power
    if self.try_char(b'^') {
      let exp = self.simplefactor()?;
      return Some(base.powf(exp));
    }
    // Postfix operators
    let mut result = base;
    loop {
      self.skip();
      if self.pos >= self.input.len() {
        break;
      }
      match self.input[self.pos] {
        b'!' => {
          // Avoid matching != (comparison)
          if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == b'=' {
            break;
          }
          self.pos += 1;
          result = pgfmath_factorial(result as i64);
        },
        b'r' => {
          // Only if not followed by letter (avoid matching identifiers)
          if self.pos + 1 < self.input.len() && self.input[self.pos + 1].is_ascii_alphabetic() {
            break;
          }
          self.pos += 1;
          result = result.to_degrees();
        },
        _ => break,
      }
    }
    Some(result)
  }

  /// simplefactor — the core atom parser
  fn simplefactor(&mut self) -> Option<f64> {
    self.skip();
    if self.pos >= self.input.len() {
      return None;
    }

    // 1. Parenthesized expression: ( formula )
    if self.input[self.pos] == b'(' {
      self.pos += 1;
      let val = self.formula()?;
      self.skip();
      self.try_char(b')'); // Perl: allow unclosed ) at end
      return Some(val);
    }

    // 2. Prefix operators: - + !
    match self.input[self.pos] {
      b'-' => {
        self.pos += 1;
        return Some(-self.simplefactor()?);
      },
      b'+' => {
        self.pos += 1;
        return self.simplefactor();
      },
      b'!' => {
        // Only prefix ! if not != (but != is at CMP level, unlikely here)
        if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == b'=' {
          return None; // not a prefix
        }
        self.pos += 1;
        let val = self.simplefactor()?;
        return Some(if val == 0.0 { 1.0 } else { 0.0 });
      },
      _ => {},
    }

    // 3. Try number first
    if let Some(num) = self.try_number() {
      self.skip();
      // NUMBER UNIT
      if let Some(unit) = self.try_unit() {
        self.units_declared = true;
        return Some(pgfmath_convert(num, &unit));
      }
      // NUMBER REGISTER
      if self.pos < self.input.len() && self.input[self.pos] == b'\\' {
        let saved = self.pos;
        if let Some(reg) = self.try_cs_register() {
          return Some(num * reg);
        }
        self.pos = saved;
      }
      return Some(num);
    }

    // 4. CS register: \something
    if self.pos < self.input.len() && self.input[self.pos] == b'\\' {
      if let Some(reg) = self.try_cs_register() {
        self.skip();
        // REGISTER REGISTER
        if self.pos < self.input.len() && self.input[self.pos] == b'\\' {
          let saved = self.pos;
          if let Some(reg2) = self.try_cs_register() {
            return Some(reg * reg2);
          }
          self.pos = saved;
        }
        return Some(reg);
      }
    }

    // 5. Identifier: function, constant, or user-defined
    if self.pos < self.input.len() && self.input[self.pos].is_ascii_alphabetic() {
      let name = self.read_identifier();

      // Built-in function?
      if is_builtin_function(&name) {
        return self.parse_function_call(&name);
      }
      // Built-in constant?
      if is_builtin_constant(&name) {
        return Some(pgfmath_apply_fn(&name, &[]));
      }
      // User-defined function?
      if is_user_function(&name) {
        return self.parse_function_call(&name);
      }
      // User-defined constant?
      if is_user_constant(&name) {
        return Some(pgfmath_apply_fn(&name, &[]));
      }
      // Unknown — might be 0
      return None;
    }

    // 6. Single dot = 0.0 (Perl L712)
    if self.pos < self.input.len() && self.input[self.pos] == b'.' {
      self.pos += 1;
      return Some(0.0);
    }

    None
  }

  /// Parse a function call with parenthesized or bare args
  fn parse_function_call(&mut self, name: &str) -> Option<f64> {
    self.skip();
    // FUNCTION ( formula (, formula)* )
    if self.pos < self.input.len() && self.input[self.pos] == b'(' {
      self.pos += 1;
      let first = self.formula()?;
      let mut args = vec![first];
      loop {
        self.skip();
        if self.try_char(b',') {
          if let Some(arg) = self.formula() {
            args.push(arg);
          } else {
            break;
          }
        } else {
          break;
        }
      }
      self.skip();
      self.try_char(b')');
      return Some(pgfmath_apply_fn(name, &args));
    }
    // FUNCTION simplefactor
    let arg = self.simplefactor()?;
    Some(pgfmath_apply_fn(name, &[arg]))
  }

  /// Try to parse a number (Perl L707-712)
  fn try_number(&mut self) -> Option<f64> {
    self.skip();
    if self.pos >= self.input.len() {
      return None;
    }

    // Hex: 0x...
    if self.pos + 1 < self.input.len()
      && self.input[self.pos] == b'0'
      && self.input[self.pos + 1] == b'x'
    {
      self.pos += 2;
      let start = self.pos;
      while self.pos < self.input.len() && self.input[self.pos].is_ascii_hexdigit() {
        self.pos += 1;
      }
      if self.pos == start {
        return None;
      }
      let s = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
      return Some(i64::from_str_radix(s, 16).unwrap_or(0) as f64);
    }

    // Binary: 0b...
    if self.pos + 1 < self.input.len()
      && self.input[self.pos] == b'0'
      && self.input[self.pos + 1] == b'b'
    {
      self.pos += 2;
      let start = self.pos;
      while self.pos < self.input.len()
        && (self.input[self.pos] == b'0' || self.input[self.pos] == b'1')
      {
        self.pos += 1;
      }
      if self.pos == start {
        return None;
      }
      let s = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
      return Some(i64::from_str_radix(s, 2).unwrap_or(0) as f64);
    }

    // Regular number: (\d+\.?\d*|\d*\.?\d+)([eE][+-]?\d+)?
    let start = self.pos;
    let mut has_digit = false;

    // Integer part
    while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
      self.pos += 1;
      has_digit = true;
    }

    // Decimal point
    if self.pos < self.input.len() && self.input[self.pos] == b'.' {
      // Consume dot if we have leading digits OR next is digit
      if has_digit || (self.pos + 1 < self.input.len() && self.input[self.pos + 1].is_ascii_digit())
      {
        self.pos += 1; // consume dot
        // Fractional digits (Perl [\d.]* — allows embedded dots like 1.2.3)
        while self.pos < self.input.len()
          && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == b'.')
        {
          self.pos += 1;
        }
        has_digit = true; // ".5" counts as having digits
      } else if has_digit {
        // "3." — trailing dot is part of the number
        self.pos += 1;
      }
    }

    if !has_digit || self.pos == start {
      return None;
    }

    // Scientific notation: [eE][+-]?\d+
    if self.pos < self.input.len() && (self.input[self.pos] == b'e' || self.input[self.pos] == b'E')
    {
      let saved = self.pos;
      self.pos += 1;
      if self.pos < self.input.len()
        && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-')
      {
        self.pos += 1;
      }
      if self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
          self.pos += 1;
        }
      } else {
        self.pos = saved; // Not scientific notation, restore
      }
    }

    let s = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
    Some(s.parse::<f64>().unwrap_or(0.0))
  }

  /// Try to match a TeX unit (Perl L714-715)
  fn try_unit(&mut self) -> Option<String> {
    self.skip();
    if self.pos >= self.input.len() {
      return None;
    }
    for &unit in PGF_UNITS {
      let bytes = unit.as_bytes();
      if self.pos + bytes.len() <= self.input.len()
        && &self.input[self.pos..self.pos + bytes.len()] == bytes
      {
        // Don't match if followed by a letter (e.g. "ptx" is not "pt")
        let end = self.pos + bytes.len();
        if end < self.input.len() && self.input[end].is_ascii_alphabetic() {
          continue;
        }
        self.pos += bytes.len();
        return Some(unit.to_string());
      }
    }
    None
  }

  /// Try to read a CS register (\name) and return its value
  fn try_cs_register(&mut self) -> Option<f64> {
    self.skip();
    if self.pos >= self.input.len() || self.input[self.pos] != b'\\' {
      return None;
    }
    let start = self.pos;
    self.pos += 1; // skip backslash
    while self.pos < self.input.len()
      && (self.input[self.pos].is_ascii_alphabetic() || self.input[self.pos] == b'@')
    {
      self.pos += 1;
    }
    if self.pos == start + 1 {
      self.pos = start;
      return None;
    }
    let cs = std::str::from_utf8(&self.input[start..self.pos]).unwrap();
    Some(pgfmath_register_lookup(cs))
  }

  /// Read an identifier: [a-zA-Z][a-zA-Z0-9_]*
  fn read_identifier(&mut self) -> String {
    let start = self.pos;
    while self.pos < self.input.len()
      && (self.input[self.pos].is_ascii_alphanumeric() || self.input[self.pos] == b'_')
    {
      self.pos += 1;
    }
    std::str::from_utf8(&self.input[start..self.pos])
      .unwrap()
      .to_string()
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

/// Main pgfmathparse evaluation function
/// Perl: sub pgfmathparse (L316-394)
/// Returns (result_string, units_declared)
pub(crate) fn pgfmathparse_eval_with_units(raw_input: &str) -> (String, bool) {
  // Normalize whitespace
  let input: String = raw_input.split_whitespace().collect::<Vec<_>>().join(" ");
  let input = input.trim();

  // 1. Simple number check (Perl L332-345)
  if let Some(result) = try_simple_number(input) {
    return (result, false);
  }

  // 2. Unit expression check (Perl L352-354): /^([+-]?[\d\.]+)(UNIT)$/ Handled by the parser below,
  //    since it's a subset of the grammar.

  // 3. Parse with recursive descent (replaces both Perl eval and RecDescent)
  let mut parser = PgfMathParser::new(input);
  if let Some(result) = parser.formula() {
    // Perl L378: forgive trailing ) or ]
    let remaining = parser.remaining_trimmed().trim_start_matches([')', ']']);
    if !remaining.is_empty() {
      // Partial parse — still return what we got
    }
    return (format_parse_result(result, input), parser.units_declared);
  }

  // 4. Fallback
  ("0.0".to_string(), false)
}

/// Convenience wrapper that returns only the result string
pub fn pgfmathparse_eval(raw_input: &str) -> String { pgfmathparse_eval_with_units(raw_input).0 }

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

  // Perl pgfmath.code.tex.ltxml L321-327: inside pgfmathparse, seven calc
  // package CSes are Let'd to the pgfmath@calc@* internals each time the
  // parser runs. Rust's pgfmathparse is a native function, so we can't
  // re-bind them per-call. Register the aliases at package-load time so
  // users who call `\real{3.14}` or `\widthof{\hbox{foo}}` outside a
  // pgfmathparse context still resolve the CS. If calc.sty is loaded
  // first, pgfmath intentionally shadows its copies — matching Perl's
  // "last definition wins" runtime semantics (calc's `\real` would be
  // replaced on the first pgfmathparse call anyway).
  Let!("\\real",     "\\pgfmath@calc@real");
  Let!("\\minof",    "\\pgfmath@calc@minof");
  Let!("\\maxof",    "\\pgfmath@calc@maxof");
  Let!("\\ratio",    "\\pgfmath@calc@ratio");
  Let!("\\widthof",  "\\pgfmath@calc@widthof");
  Let!("\\heightof", "\\pgfmath@calc@heightof");
  Let!("\\depthof",  "\\pgfmath@calc@depthof");

  // ==================== pgfmathparse override ====================
  // Perl L401-403: DefMacro('\lx@pgfmath@parse{}', sub { ... })
  DefMacro!("\\lx@pgfmath@parse {}", sub[(tokens)] {
    // Expand tokens and convert to string
    let expanded = gullet::do_expand(tokens).unwrap_or_default();
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
    let expanded = gullet::do_expand(tokens).unwrap_or_default();
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
