//! pgfmath.code.tex — PGF math operations accelerator
//! Perl: pgfmath.code.tex.ltxml (737 lines)
//!
//! Replaces TeX-level pgfmath operations with native Rust implementations
//! for better precision and performance. Without this, pgf math runs in
//! TeX's fixed-point arithmetic which has different rounding behavior.
use latexml_core::common::{dimension::fixedformat, numeric_ops::fixpoint};

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
/// `\pgfmathanglebetweenpoints`). Real pgf's `\pgfmath@tonumber`
/// (`\expandafter\Pgf@geT\the<dimen>`) converts a dimension (e.g. `0.0pt`) to
/// `"0.0"` preserving the decimal dot. Delimited parameter scanners like
/// `\pgfplotscolormap@floor@unforgiving#1.#2\relax` require the dot.
/// Perl pgfmath.code.tex.ltxml:85-90 strips `.0`, which latexml-oxide diverges
/// from (surpass, OXIDIZED_DESIGN) to match real pgf.
///
/// Note: previously 2201.09268's bisection in `\pgfmathpointintersectionoflineandarc`
/// needed bare integers, but batch 56ac replaced that bisection with a closed-form
/// binding (KNOWN_PERL_ERRORS #210).
fn pgfmath_result_str(value: f64) -> String {
  if value.is_nan() || value.is_infinite() {
    return "0.0".to_string();
  }
  let clamped = value.clamp(-MAX_PGF_NUMBER, MAX_PGF_NUMBER);
  if clamped == (clamped as i64) as f64 {
    return format!("{}.0", clamped as i64);
  }
  let mut s = format!("{:.5}", clamped);
  // Strip trailing zeros after the decimal point, preserving at least one digit after the dot.
  if s.contains('.') {
    while s.ends_with('0') && !s.ends_with(".0") {
      s.pop();
    }
  }
  s
}

/// pgf's `@` arithmetic is TeX dimension-register arithmetic (pgfmathfunctions.basic.code.tex:16-25 `add`:
/// `\pgfmath@x=#1pt\relax \pgfmath@y=#2pt\relax \advance\pgfmath@x by\pgfmath@y`; `subtract`, `neg` alike): an
/// operand is scanned to scaled points as `#1pt` is (tex.web §102 `round_decimals`), the arithmetic is on integers,
/// and the result is printed by `\the` (`\pgfmath@returnone`, §103 `print_scaled`). Done in f64 and rounded to five
/// decimals, as Perl's binding does (pgfmath.code.tex.ltxml:85-90, :155-156), the cloud shape's border-anchor binary
/// search (pgflibraryshapes.symbols.code.tex:1236-1292, which ends on `\ifdim\p pt=\s pt`) could round its midpoint up
/// to its end forever: Fatal on 2606.26406 and 2508.07407 (KNOWN_PERL_ERRORS #461).
/// A value past TeX's largest dimension is clamped to it, as TeX's `arith_error` clamps a scanned `#1pt` (§460) — and
/// so the integer arithmetic below cannot overflow.
fn pgfmath_sp(text: &Tokens) -> i64 {
  fixpoint(
    parse_pgf_number(text).clamp(-MAX_PGF_NUMBER, MAX_PGF_NUMBER),
    None,
  )
}

/// pgf's `divide` (pgfmathfunctions.basic.code.tex:66-100): a whole divisor divides the scaled points as TeX's
/// `\divide` does, truncating toward zero (§1240 → §106 `x_over_n`), so an interval halved to 1sp has its midpoint at
/// its start; any other divisor is pgf's long division, its quotient on the same grid. A zero divisor leaves the
/// dividend ([`pgfmath_divide`]).
fn pgfmath_divide_sp(x: i64, y: i64) -> i64 {
  if y == 0 {
    fixpoint(pgfmath_divide(x as f64 / 65536.0, 0.0), None)
  } else if y % 65536 == 0 {
    x / (y / 65536)
  } else {
    fixpoint(x as f64 / y as f64, None)
  }
}

/// `\def\pgfmathresult{<the value>}` for a value in scaled points, printed as `\the` prints a dimension, without
/// its `pt` (§103).
fn pgfmath_sp_result_tokens(sp: i64) -> Vec<Token> {
  let mut toks = vec![T_CS!("\\def"), T_CS!("\\pgfmathresult"), T_BEGIN!()];
  toks.extend(Explode!(fixedformat(sp, None)));
  toks.push(T_END!());
  toks
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
///
/// pgfmathparser.code.tex:392-396 keeps a `"…"` string operand as the REAL
/// tokens it was written with, so a ternary like braids.sty:276
/// `\pgfmathparse{\value{c} < \n ? "\noexpand\setcounter{c}{\n}" : ""}` followed
/// by `\pgfmathresult` EXECUTES the `\setcounter`. Our evaluator carries the
/// string through Rust, so a result holding a control sequence is re-tokenized
/// (at the caller's `@` catcode — braids.sty runs under `\makeatletter`) instead
/// of exploded into dead `\`+letters. Numbers explode as before. Perl's pgfmath
/// cannot parse the ternary at all (SHARED). Witness: braids/braids.
/// Guard: `perfect_kernel_batch56::pgfmath_string_result_keeps_control_sequences`.
fn pgfmath_result_tokens_str(s: &str) -> Vec<Token> {
  let mut toks = vec![T_CS!("\\def"), T_CS!("\\pgfmathresult"), T_BEGIN!()];
  // pgfmathparser.code.tex:35-40 `\pgfmath@catcodes` re-catcodes only `= , | &`,
  // so a string operand's LETTERS stay catcode 11: tikz's path dispatcher
  // `\tikz@handle` (tikz.code.tex:2134-2163) `\ifx`-matches the letter `a` of
  // an `arc[…]` result against a catcode-11 `{a}` — exploded OTHER letters never
  // dispatch and the path gives up (zx-calculus curved wires,
  // tikzlibraryzx-calculus.code.tex:56-63; Perl's `ExplodeText` shares the
  // gap). Guard: `perfect_kernel_batch56::pgfmath_string_result_keeps_letter_catcodes`.
  if s.contains('\\') || s.chars().any(|c| c.is_ascii_alphabetic()) {
    // A control word with `@` in the string came from expanding an internal macro (a string
    // operand is expanded first: `\,` is `\lx@thinspace` here, robust and kept in LaTeX), so
    // it is read back with `@` a letter even in the document body; read as `\lx` and
    // `@thinspace` it was an undefined `\lx` (wheelchart.tex's `5>10?"#1":"\,\%"` labels,
    // 29 errors).
    let internal_name = s.split('\\').skip(1).any(|rest| {
      let name: &str = rest
        .split(|c: char| !(c.is_ascii_alphabetic() || c == '@'))
        .next()
        .unwrap_or("");
      // Only a defined `@`-name: a user's `"\LaTeX@home"` stays `\LaTeX` then `@home`.
      name.contains('@')
        && lookup_definition(&T_CS!(&s!("\\{name}")))
          .ok()
          .flatten()
          .is_some()
    });
    let at_letter = internal_name || matches!(lookup_catcode('@'), Some(Catcode::LETTER));
    let src = TeXString::assembled(s.to_string());
    let retok = if at_letter {
      TokenizeInternal!(src)
    } else {
      Tokenize!(src)
    };
    toks.extend(retok.unlist());
  } else {
    toks.extend(Explode!(s));
  }
  toks.push(T_END!());
  toks
}

/// pgf's `mod` (pgfmathfunctions.basic.code.tex:263-278): truncated, as TeX's `\divide` truncates, and a zero divisor
/// leaves the dividend (`mod(x, 0)` is `x`, :271). Its `Mod` (:286-297) is that `mod` plus the divisor when it is
/// negative, so a multiple of the divisor is 0 (`Mod(-5,5)`: 1811.01404's `style\st`, style5 for style0). Perl's
/// floor mod (pgfmath.code.tex.ltxml:196-200) makes it the divisor, and its second `\pgfmathmod@` (:206) defines
/// `mod` as that floor mod (KNOWN_PERL_ERRORS #944). Guard `perfect_kernel_batch71a::pgfmath_mod_of_a_multiple_is_zero`.
fn pgfmath_mod(a: f64, b: f64, never_negative: bool) -> f64 {
  let truncated = if b == 0.0 {
    pgfmath_divide(a, b)
  } else {
    a % b
  } + 0.0;
  if never_negative && truncated < 0.0 {
    truncated + b
  } else {
    truncated
  }
}

/// Safe divisor — avoid division by zero. Perl `pgfmath.code.tex.ltxml:255`
/// warns on every zero divisor ("pgfmath: divisor should never be zero!"),
/// as pgf itself raises a Math Error; this port had gone silent.
fn pgfmath_divisor(v: f64) -> f64 {
  if v == 0.0 {
    Warn!(
      "unexpected",
      "<number>",
      "pgfmath: divisor should never be zero!"
    );
    EPSILON
  } else {
    v
  }
}

/// pgf's `divide`: `\pgfmath@x` is divided by the integer divisor with TeX's
/// `\divide` (pgfmathfunctions.basic.code.tex:66-80), and a zero divisor
/// leaves the register UNCHANGED — tex.web §107 `x_over_n` sets `arith_error`
/// and §1240 `do_register_command` returns after the error without storing —
/// so `divide(x, 0)` is `x` (pdflatex: "You've asked me to divide 'x' by
/// '0.0'", recoverable). The EPSILON divisor gave `x / 0.00001` instead: a
/// decoration whose `segment length` had degraded to 0 (carbohydrates.sty:158
/// reads chemfig's renamed `\CF@atom@sep`) then set its step from
/// `len / int(len / 0)` — ~1 sp instead of pgf's ~1 pt — and pgf's decoration
/// automaton walked a 16 pt path a million steps into the pushback limit
/// (batch 56gb; RUST-ONLY, Perl never draws it).
/// Also pgf's `mod`: the same `\divide` leaves its dividend, so `mod(x, 0)` is `x`.
/// Perl's port returned `x / epsilon` here (and warned); the dividend is the
/// pgf/pdflatex value — OXIDIZED_DESIGN #259.
fn pgfmath_divide(a: f64, b: f64) -> f64 {
  if b == 0.0 {
    Warn!(
      "unexpected",
      "<number>",
      "pgfmath: divisor should never be zero!"
    );
    a
  } else {
    a / b
  }
}

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
  match do_expand(arg.clone()) {
    Ok(expanded) => {
      let s = expanded.to_string();
      let s = strip_leading_double_negation(s.trim());
      if s == "." {
        return 0.0;
      }
      s.parse::<f64>().unwrap_or(0.0)
    },
    _ => 0.0,
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
      // Overflow check for negative (Perl .ltxml:342-344)
      if let Ok(numval) = result.parse::<f64>()
        && numval < -MAX_PGF_NUMBER
      {
        report_pgfmath_error(
          "pgfmath",
          "overflow",
          &format!("Dimension too large: {result}. Input was: {input}"),
        );
        result = format!("{}", -MAX_PGF_NUMBER);
      }
    }
  }
  Some(result)
}

// ==================== Unit Conversion ====================

/// Convert number with unit to points
/// Perl: sub pgfmath_convert (L476-485)
///
/// `em` and `ex` are the quad and x-height of the font `\pgfmath@selectfont` selects, as pgf parses after it
/// (pgfmathparser.code.tex:132): inside a picture that is the document's font, not the `\nullfont` in force.
fn pgfmath_convert(number: f64, unit: &str) -> f64 {
  let sp = match unit.to_lowercase().as_str() {
    "em" => pgfmath_font().map_or_else(|| convert_unit(unit), |font| font.get_em_width() as f64),
    "ex" => pgfmath_font().map_or_else(|| convert_unit(unit), |font| font.get_ex_height() as f64),
    _ => convert_unit(unit),
  };
  number * sp / 65536.0
}

/// The font `\pgfmath@selectfont` (= `\selectfont`, pgfmathutil.code.tex:167-174) puts in force: the current font with
/// LaTeX's family, series and shape, so a picture's `\nullfont` gives way to the document's font. pgf evaluates every
/// length under it (pgfmathcalc.code.tex:30-38, pgfmathparser.code.tex:132): `8em` at `\footnotesize` is cmr8's quad,
/// 68.00098pt, where the `\nullfont` in force gave 64.00012pt and a node's frame came out narrower than its `text
/// width` (consort-flowchart; repro graphics-tikz/pgfmath_em_selectfont).
fn pgfmath_font() -> Option<Font> {
  let current = lookup_font()?;
  // Plain TeX has no NFSS: pgf's `\pgfmath@selectfont` is `\rm` there (pgfmathutil.code.tex:169), for which the font
  // in force stands in (the same at 10pt; not after a raw `\font\x=cmr12 \x`).
  if lookup_definition(&T_CS!("\\f@family"))
    .ok()
    .flatten()
    .is_none()
  {
    return Some((*current).clone());
  }
  latexml_engine::latex_constructs::nfss_selected_font(&current, false).ok()
}

// ==================== Register Lookup ====================

/// Look up a TeX register value, return as points, and whether it is a count
/// Perl: sub pgfmath_register (L487-493)
fn pgfmath_register_lookup(cs: &str) -> (f64, bool) {
  match lookup_register(cs, vec![]) {
    Ok(Some(reg)) => match reg {
      RegisterValue::Number(n) => (n.value_of() as f64, true),
      RegisterValue::Dimension(d) => (d.value_of() as f64 / 65536.0, false),
      RegisterValue::Glue(g) => (g.value_of() as f64 / 65536.0, false),
      _ => (0.0, false),
    },
    _ => (0.0, false),
  }
}

/// The e-TeX expression `text` opens (`\dimexpr`, `\glueexpr`, `\numexpr`, `\muexpr`), read as TeX reads pgf's
/// `\pgfmath@dimen\dimexpr…` (pgfmathparser.code.tex:571-600): its value (points, the integer, or the mu) and the
/// bytes of `text` before the text TeX left (1711.06128, 2206.04438).
fn pgfmath_read_expr(primitive: &str, text: &str) -> Option<(f64, usize)> {
  let tokens = mouth::tokenize_internal(TeXString::assembled(text.to_string()));
  let integer = primitive == "\\numexpr";
  let mu = primitive == "\\muexpr";
  let (value, left) = Mouth::new("", None)
    .and_then(|mouth| {
      reading_from_mouth(mouth, move || {
        unread(tokens);
        let value = if integer {
          read_number()?.value_of() as f64
        } else if mu {
          read_mu_glue()?.value_of() as f64 / 65536.0
        } else {
          read_dimension()?.value_of() as f64 / 65536.0
        };
        let mut left = Vec::new();
        while let Some(token) = read_token()? {
          left.push(token);
        }
        Ok((value, Tokens::new(left).to_string()))
      })
    })
    .ok()?;
  // Where the text left starts in `text`, matched from the end, spaces aside.
  let text = text.trim_end();
  let mut chars = text
    .char_indices()
    .rev()
    .filter(|(_, c)| !c.is_whitespace());
  let mut start = text.len();
  for wanted in left.chars().rev().filter(|c| !c.is_whitespace()) {
    let (at, c) = chars.next()?;
    if c != wanted {
      return None;
    }
    start = at;
  }
  Some((value, start))
}

/// `\wd`/`\ht`/`\dp` of box `number`, in points (Perl `pgfmath_getwidth`/`getheight`/`getdepth`,
/// pgfmath.code.tex.ltxml:499-521): the box register's own getter, a void box 0.
fn pgfmath_box_dimension(dimension: &str, number: i64) -> f64 {
  match lookup_register(dimension, vec![ArgWrap::Number(Number::new(number))]) {
    Ok(Some(RegisterValue::Dimension(d))) => d.value_of() as f64 / 65536.0,
    _ => 0.0,
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
      | "array"
      | "dim"
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
/// pgf's `min`/`max` list fold (pgfmathfunctions.misc.code.tex:292-336): every
/// argument against the sentinel; a lone comma-less argument is scanned one
/// token at a time by `\pgfmathmin@@#1{}` — its decimal digits.
fn pgfmath_fold_list(args: &[f64], sentinel: f64, pick: fn(f64, f64) -> f64) -> f64 {
  if args.len() == 1 {
    let text = format!("{}", args[0].trunc().abs() as i64);
    return text
      .bytes()
      .map(|b| f64::from(b - b'0'))
      .fold(sentinel, pick);
  }
  args.iter().copied().fold(sentinel, pick)
}

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
    "divide" => pgfmath_divide(a, b),
    "div" => pgfmath_divide(a, b) as i64 as f64,
    "pow" => a.powf(b),
    "abs" => a.abs(),
    "round" => a.round(),
    "floor" => a.floor(),
    "ceil" => a.ceil(),
    "int" => a as i64 as f64,
    "real" => a,
    "mod" | "Mod" => pgfmath_mod(a, b, name == "Mod"),
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
    // Misc — variadic, as pgfmathfunctions.misc.code.tex:292-336 declares
    // `min`/`max` `{...}` and folds `\pgfmathmin@@`/`\pgfmathmax@@` over the
    // whole list from the `±16383pt` sentinel; the binary `a.max(b)` dropped
    // every argument past the second (ribbonproofs.sty:1213
    // `min(\@leftPositions)` → wrong `\@stepLeft`, a ribbon re-started
    // "already active" and a `\fi`/`\iffalse` cascade inside the tikz
    // `\foreach`; ribbonproofsmanual 3, pdflatex clean). A SINGLE comma-less
    // argument reaches `\pgfmathmin@@#1{}` unbraced, so pgf consumes it one
    // TOKEN at a time — `min(20)` is `min(2,0)` = 0, `max(20)` = 2 (pdflatex
    // verified); reproduced over the integer's digits.
    "max" => pgfmath_fold_list(args, -16383.0, f64::max),
    "min" => pgfmath_fold_list(args, 16383.0, f64::min),
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
      pgfmath_apply_user_reported(name, args, &[]).0
    },
  }
}

/// Apply a user-defined pgf math function by calling TeX
/// Perl: sub pgfmath_apply (L447-459)
///
/// An argument in integer form (`arg_ints`) is passed as an integer (`7`, not `7.0`), and the
/// result's own form is returned with it: a body whose `\pgfmathresult` is an integer literal
/// (`\pgfmathparse{#1>0}`, `\def\pgfmathresult{1}`) yields an integer, as pgf leaves it.
fn pgfmath_apply_user(name: &str, args: &[f64], arg_ints: &[bool]) -> UserApply {
  let cs_name = format!("\\pgfmath{}@", name);
  let cs_tok = T_CS!(&cs_name);
  // Check if the function is defined
  if !matches!(lookup_definition(&cs_tok), Ok(Some(_))) {
    return UserApply::Undefined;
  }
  // Build invocation tokens: \pgfmath{name}@{arg1}{arg2}...
  let mut tokens: Vec<Token> = vec![cs_tok];
  for (k, arg) in args.iter().enumerate() {
    let s = if arg_ints.get(k).copied().unwrap_or(false) && *arg == (*arg as i64) as f64 {
      format!("{}", *arg as i64)
    } else {
      pgfmath_result_str(*arg)
    };
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
  let _ = digest(Tokens::from(vec![T_CS!("\\pgfmathunitsdeclaredfalse")]));
  // Digest the invocation (sets \pgfmathresult) inside the parse's group
  // (`pgfmath_grammar::evaluate`): what the body defines locally — a tikzmath function's
  // parameters `\cx=#1` — is seen by later calls of the same parse and gone after it. Ungrouped
  // (Perl pgfmath.code.tex.ltxml:452-455 too, KPE #421), 2605.28612's indexed variable
  // `\CircleCenterX1` read the leaked parameter: `0`, `1` → `01`, then 11, 111 cm.
  let _ = digest(Tokens::from(tokens));
  // Read back \pgfmathresult via expansion
  let result = do_expand(Tokens::from(vec![T_CS!("\\pgfmathresult")]));
  let Ok(result_tokens) = result else {
    return UserApply::NotNumeric(String::new());
  };
  let s = result_tokens.to_string();
  let s = s.trim();
  match s.parse::<f64>() {
    Ok(value) => UserApply::Value(value, s.parse::<i64>().is_ok()),
    Err(_) => UserApply::NotNumeric(s.to_string()),
  }
}

/// The outcome of calling a pgfmath function through its TeX body.
enum UserApply {
  /// The result and whether it is in integer form.
  Value(f64, bool),
  /// The body left a `\pgfmathresult` that is not a number.
  NotNumeric(String),
  /// No `\pgfmath<name>@` is defined.
  Undefined,
}

/// [`pgfmath_apply_user`], its body run once, an unusable result reported (Perl
/// pgfmath.code.tex.ltxml:457-459 for an undefined operator) and read as 0.
fn pgfmath_apply_user_reported(name: &str, args: &[f64], arg_ints: &[bool]) -> (f64, bool) {
  let message = match pgfmath_apply_user(name, args, arg_ints) {
    UserApply::Value(value, int) => return (value, int),
    UserApply::NotNumeric(text) => {
      format!("pgfmath function '{name}' returned a non-numeric result '{text}'")
    },
    UserApply::Undefined => format!("Unimplemented pgfmath operator '{name}'"),
  };
  let _ = (|| -> Result<()> {
    Error!("unexpected", name, message);
    Ok(())
  })();
  (0.0, false)
}

/// Check if a name is a user-defined pgf constant (arity 0)
/// Perl: sub pgfmath_checkuserconstant (L540-546)
fn is_user_constant(name: &str) -> bool {
  let cs = format!("\\pgfmath@function@{}", name);
  let tok = T_CS!(&cs);
  if lookup_definition(&tok).ok().flatten().is_none() {
    return false;
  }
  // Check arity — must be 0 for constant
  let arity_cs = format!("\\pgfmath@operation@{}@arity", name);
  let arity_tok = T_CS!(&arity_cs);
  if let Ok(Some(_)) = lookup_definition(&arity_tok)
    && let Ok(expanded) = do_expand(Tokens::from(vec![arity_tok]))
  {
    let s = expanded.to_string();
    let arity: usize = s.trim().parse().unwrap_or(0);
    return arity == 0;
  }
  // No arity defined — treat as constant
  true
}

/// Check if a name is a user-defined pgf function (arity > 0)
/// Perl: sub pgfmath_checkuserfunction (L548-554)
fn is_user_function(name: &str) -> bool {
  let cs = format!("\\pgfmath@function@{}", name);
  let tok = T_CS!(&cs);
  if lookup_definition(&tok).ok().flatten().is_none() {
    return false;
  }
  let arity_cs = format!("\\pgfmath@operation@{}@arity", name);
  let arity_tok = T_CS!(&arity_cs);
  if let Ok(Some(_)) = lookup_definition(&arity_tok)
    && let Ok(expanded) = do_expand(Tokens::from(vec![arity_tok]))
  {
    let s = expanded.to_string();
    let arity: usize = s.trim().parse().unwrap_or(0);
    return arity > 0;
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
//   factor      : simplefactor (^ simplefactor | !)*   (postfix ! and r wait for the ^ chain after them)
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
  use latexml_core::{T_CS, stomach::digest, tokens::Tokens};
  use winnow::{
    error::{ContextError, ErrMode},
    prelude::*,
    stream::Stream,
  };

  use super::{
    PGF_UNITS, is_builtin_constant, is_builtin_function, is_user_constant, is_user_function,
    pgfmath_apply_fn, pgfmath_cmp_op, pgfmath_convert, pgfmath_divide, pgfmath_factorial,
    pgfmath_register_lookup,
  };

  pub(super) type In<'a> = winnow::Stateful<&'a str, Flags>;

  #[derive(Debug, Default, Clone)]
  pub(super) struct Flags {
    pub units_declared: bool,
    /// arXiv-fork c52d3cf6: comparison operators return INTEGERS
    /// (`2 == 2` -> "1", not "1.0") so `\ifnum\pgfmathresult=1`-style
    /// tests work downstream.
    pub int_result:     bool,
    /// A user function was called: the parse's `\begingroup` is open (see `evaluate`).
    pub group_open:     bool,
    /// The value is not a number: an array's text element, or an array, as pgf's `\pgfmathresult` holds it.
    pub text_result:    Option<String>,
  }

  /// The literal [`try_number`] read: digits and points, a base prefix's digits, or an exponent's.
  enum Literal {
    Decimal(String),
    Based,
    Scientific,
  }

  /// A control sequence's operand ([`try_cs_register`]): a dimension, which scales a number before it; a count, an
  /// integer that takes its place; an expression's value, which takes its place.
  enum CsOperand {
    Scales(f64),
    Count(f64),
    Replaces(f64),
  }

  impl CsOperand {
    /// The operand's value after `number`, the parse's integer flag set from it.
    fn after(self, i: &mut In, number: f64) -> f64 {
      i.state.int_result = matches!(self, CsOperand::Count(_));
      match self {
        CsOperand::Scales(value) => number * value,
        CsOperand::Count(value) | CsOperand::Replaces(value) => value,
      }
    }
  }

  /// An element of a pgfmath array, `{"a",1+1,{2,3}}` (pgfmathparser.code.tex:951-954, `"` :392, :931): pgf keeps each
  /// as its result, in a parser that has only numbers these are kept apart.
  enum ArrayElement {
    Number(f64, bool),
    Text(String),
    Array(Vec<ArrayElement>),
  }

  impl ArrayElement {
    /// As `\pgfmathresult` holds it: `2`, `2.0`, `up`, an array as its elements' groups, `{1}{2}`.
    fn result_text(&self) -> String {
      match self {
        ArrayElement::Number(value, true) => format!("{}", *value as i64),
        ArrayElement::Number(value, false) => super::format_parse_result(*value, ""),
        ArrayElement::Text(text) => text.clone(),
        ArrayElement::Array(elements) => elements
          .iter()
          .map(|element| format!("{{{}}}", element.result_text()))
          .collect(),
      }
    }
  }

  /// The index of a `{`-group's closing brace in `s`, which starts with it (a quoted string's braces count for nothing).
  pub(super) fn matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    for (at, byte) in s.bytes().enumerate() {
      match byte {
        b'"' => quoted = !quoted,
        b'{' if !quoted => depth += 1,
        b'}' if !quoted => {
          depth -= 1;
          if depth == 0 {
            return Some(at);
          }
        },
        _ => {},
      }
    }
    None
  }

  /// Whether the `{`-group `s` starts with is an array, not the grouping the grammar skips: one indexed, `{…}[i]`, or a
  /// list that is the rest of the formula, `{1,2,3}`.
  fn opens_array(s: &str) -> bool {
    let Some(end) = matching_brace(s) else {
      return false;
    };
    let after = s[end + 1..].trim_start();
    after.starts_with('[')
      || (after.is_empty() && {
        let inner = &s[1..end];
        inner.trim_start().starts_with('"')
          || super::split_top_commas(inner).is_some_and(|l| l.len() > 1)
      })
  }

  /// A number is wanted here, where the value so far may be text. pgf's arithmetic reads an operand as TeX reads
  /// `\pgfmath@x=#1pt` (pgfmathfunctions.basic.code.tex), so [`text_as_number`] gave the value and text a number can't
  /// be read from is "Missing number, treated as zero" (live pgf, `{"a","b"}[0]+1` is 1.0 and that error).
  fn number_wanted(i: &mut In) {
    if i
      .state
      .text_result
      .take()
      .is_some_and(|text| !text_as_number(&text).1)
    {
      super::report_pgfmath_error(
        "expected",
        "<number>",
        "Missing number, treated as zero: a pgfmath string in arithmetic",
      );
    }
  }

  /// The number TeX reads from a text value followed by `pt`, and whether it reads all it should: `1` and `1.5`, a unit
  /// of its own (`1pt` and `1cm`, whose `pt` left over prints in pgf's null font), but `1e2` is 1 and an illegal unit,
  /// `a` is a missing number.
  fn text_as_number(text: &str) -> (f64, bool) {
    let sign = text
      .bytes()
      .take_while(|b| matches!(b, b'+' | b'-'))
      .count();
    let mut point = false;
    let digits = text[sign..]
      .bytes()
      .take_while(|b| b.is_ascii_digit() || (*b == b'.' && !std::mem::replace(&mut point, true)))
      .count();
    let (number, unit) = text.split_at(sign + digits);
    let negative = number.bytes().filter(|b| *b == b'-').count() % 2 == 1;
    let Ok(value) = number[sign..].parse::<f64>() else {
      return (0.0, false);
    };
    let value = if negative { -value } else { value };
    if unit.is_empty() {
      return (value, true);
    }
    match PGF_UNITS.iter().find(|known| unit.starts_with(**known)) {
      Some(known) => (pgfmath_convert(value, known), true),
      None => (value, false),
    }
  }

  /// The bytes of the token `text` opens with: a character, or a control sequence.
  fn token_width(text: &str) -> usize {
    match text.strip_prefix('\\') {
      Some(name) => {
        let letters = name.bytes().take_while(u8::is_ascii_alphabetic).count();
        1 + if letters > 0 {
          letters
        } else {
          name.chars().next().map_or(0, char::len_utf8)
        }
      },
      None => text.chars().next().map_or(0, char::len_utf8),
    }
  }

  /// The tokens of a value, as elements: `\pgfmatharray@@`'s undelimited argument skips spaces, and takes a group whole.
  fn text_tokens(text: &str) -> Vec<ArrayElement> {
    let mut tokens = Vec::new();
    let mut rest = text.trim_start();
    while let Some(c) = rest.chars().next() {
      let width = if c == '{' {
        matching_brace(rest).map_or(1, |end| end + 1)
      } else {
        token_width(rest)
      };
      let (token, tail) = rest.split_at(width);
      tokens.push(ArrayElement::Text(
        token
          .strip_prefix('{')
          .and_then(|t| t.strip_suffix('}'))
          .unwrap_or(token)
          .to_string(),
      ));
      rest = tail.trim_start();
    }
    tokens
  }

  /// What `\pgfmath@collect@`'s `\pgfmath@bgroup@strip` leaves of a string in a list of two or more
  /// (pgfmathparser.code.tex:963, :984-1009): its re-read `\pgfmath@bgroup@@@strip#1#2#3` takes two undelimited arguments,
  /// which skip the spaces before them, so a space before the first token or the second goes and any later one stays
  /// (`"a b"` is `ab`, `"a ? b"` is `a? b`, `"ab c"` stays). A string that opens with a group is kept whole.
  fn collected_string(text: &str) -> String {
    let rest = text.trim_start();
    if rest.starts_with('{') {
      return text.to_string();
    }
    let first = token_width(rest);
    let (head, tail) = rest.split_at(first);
    // (a control word keeps the space that ends it, which is no token)
    let word = head.len() > 2 && head.starts_with('\\');
    if word {
      rest.to_string()
    } else {
      format!("{head}{}", tail.trim_start())
    }
  }

  /// The elements of the array `{…}` the input starts at (each a string, a nested array, or a formula), consumed.
  fn array_literal(i: &mut In) -> ModalResult<Vec<ArrayElement>> {
    let text = i.input;
    let Some(end) = matching_brace(text) else {
      return fail();
    };
    let inner = &text[1..end];
    let mut elements = Vec::new();
    if !inner.trim().is_empty() {
      let pieces = super::split_top_commas(inner).unwrap_or_default();
      let collected = pieces.len() > 1;
      for piece in pieces {
        let piece = piece.trim();
        let quoted = piece
          .strip_prefix('"')
          .and_then(|rest| rest.strip_suffix('"'))
          .filter(|text| !text.contains('"'));
        if let Some(string) = quoted {
          elements.push(ArrayElement::Text(if collected {
            collected_string(string)
          } else {
            string.to_string()
          }));
        } else if piece.is_empty() {
          elements.push(ArrayElement::Text(String::new()));
        } else if piece.starts_with('{') && matching_brace(piece) == Some(piece.len() - 1) {
          i.input = piece;
          let mut inner = array_literal(i)?;
          // (a list's strip takes a lone string's braces too: `{{"a b"},"d"}[0]` is `ab`)
          elements.push(match inner.as_mut_slice() {
            [ArrayElement::Text(text)] if collected => ArrayElement::Text(collected_string(text)),
            _ => ArrayElement::Array(inner),
          });
        } else {
          i.input = piece;
          let value = formula(i);
          let number = value.unwrap_or(0.0);
          elements.push(match i.state.text_result.take() {
            Some(string) => ArrayElement::Text(string),
            None => ArrayElement::Number(number, i.state.int_result),
          });
        }
      }
    }
    i.input = &text[end + 1..];
    Ok(elements)
  }

  /// `elements[index]`, `index` truncated as `\pgfmath@count=#2` does; out of bounds is pgf's error and result 0.
  fn array_element(elements: Vec<ArrayElement>, index: f64) -> ArrayElement {
    let chosen = if index.trunc() >= 0.0 {
      elements.into_iter().nth(index.trunc() as usize)
    } else {
      None
    };
    chosen.unwrap_or_else(|| {
      super::report_pgfmath_error(
        "unexpected",
        "array",
        "PGF Math Error: Array index out of bounds.",
      );
      ArrayElement::Number(0.0, true)
    })
  }

  /// `elements[index]` of a list as pgf holds it. A list of one is no group of groups: `\pgfmatharray@@` indexes the tokens
  /// of its value, `{"ab"}[1]` is `b`, `{1cm}[0]` is `2` (pgfmathfunctions.misc.code.tex:243-262), and one such list inside
  /// another is that value's group, `{{"ab"}}[0]` is `ab`.
  fn list_element(mut elements: Vec<ArrayElement>, index: f64) -> ArrayElement {
    let is_value = |element: &ArrayElement| !matches!(element, ArrayElement::Array(_));
    match elements.as_mut_slice() {
      [ArrayElement::Array(inner)] if inner.len() == 1 && is_value(&inner[0]) => {
        array_element(std::mem::take(inner), index)
      },
      [one] if is_value(one) => array_element(text_tokens(&one.result_text()), index),
      _ => array_element(elements, index),
    }
  }

  /// The value of a whole array element: a number as it is, text in the result, an array as its text.
  fn array_value(i: &mut In, value: ArrayElement) -> f64 {
    match value {
      ArrayElement::Number(number, int) => {
        i.state.int_result = int;
        number
      },
      other => {
        let text = other.result_text();
        let number = text_as_number(&text).0;
        i.state.text_result = Some(text);
        i.state.int_result = false;
        number
      },
    }
  }

  /// `{…}` and its accesses `[i][j]…` (pgfmathparser.code.tex:943-949 `\pgfmath@startindex@`, which gives an array
  /// element back in a group of its own: `{{1,2},{3,4}}[1]` is `{{3}{4}}`). Live pgf for each form, repro
  /// graphics-tikz/pgfmath_array_primary_and_strings.
  fn array_primary(i: &mut In) -> ModalResult<f64> {
    let mut value = ArrayElement::Array(array_literal(i)?);
    let mut indexed = false;
    loop {
      let spaces = i.input.bytes().take_while(u8::is_ascii_whitespace).count();
      if i.input.as_bytes().get(spaces) != Some(&b'[') {
        break;
      }
      bump(i, spaces + 1);
      let index = formula(i)?;
      number_wanted(i);
      eat(i, b']');
      value = match value {
        ArrayElement::Array(elements) => list_element(elements, index),
        // an index into what is no array: pgf reads its one token as the array
        other => array_element(vec![other], index),
      };
      indexed = true;
    }
    // (a whole formula's braces go before pgf reads it, pgfmathparser.code.tex:139: `{"a"}` is `"a"`)
    if !indexed
      && let ArrayElement::Array(elements) = &mut value
      && matches!(elements.as_slice(), [ArrayElement::Text(_)])
    {
      value = elements.remove(0);
    }
    let array = indexed && matches!(value, ArrayElement::Array(_));
    let number = array_value(i, value);
    if array {
      i.state.text_result = i.state.text_result.take().map(|text| format!("{{{text}}}"));
    }
    Ok(number)
  }

  fn fail<T>() -> ModalResult<T> { Err(ErrMode::Backtrack(ContextError::new())) }

  /// `\pgfmathparse` evaluates in one group (pgfmathparser.code.tex:21, :145-148,
  /// `\pgfmath@smuggleone\pgfmathresult\endgroup`): a user function's body runs in it, so what
  /// one call defines locally is seen by a later call of the same parse and gone after it. Opened
  /// on the first user call (a parse of builtins digests nothing), closed by `evaluate`.
  fn open_parse_group(i: &mut In) {
    if !i.state.group_open {
      let _ = digest(Tokens::from(vec![T_CS!("\\begingroup")]));
      i.state.group_open = true;
    }
  }

  /// Parse `input` as a formula, closing the parse's group if a user function opened it; the
  /// units flag, global in pgf (pgfmathparser.code.tex:51-52), is set after the group from the
  /// parse's own.
  pub(super) fn evaluate(input: &str) -> (ModalResult<f64>, Flags) {
    let mut stream = winnow::Stateful { input, state: Flags::default() };
    let result = formula(&mut stream);
    if stream.state.group_open {
      // The parse's own units flag (each user body reset the global one before it ran).
      let units = if stream.state.units_declared {
        "\\pgfmathunitsdeclaredtrue"
      } else {
        "\\pgfmathunitsdeclaredfalse"
      };
      let _ = digest(Tokens::from(vec![T_CS!("\\endgroup")]));
      let _ = digest(Tokens::from(vec![T_CS!(units)]));
    }
    (result, stream.state)
  }

  /// The grammar-wide skip: `[\s\{\}]*`, but for the brace of an array.
  fn sb(i: &mut In) {
    let bytes = i.input.as_bytes();
    let mut n = 0;
    while let Some(byte) = bytes.get(n) {
      match byte {
        b' ' | b'\t' | b'\n' | b'\r' | b'}' => n += 1,
        b'{' if !opens_array(&i.input[n..]) => n += 1,
        _ => break,
      }
    }
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

  /// formula := logic ('?' logic ':' logic)*
  ///
  /// pgf's precedences (pgfmathparser.code.tex:898-916): `?` 100 and `:` 101 are the loosest,
  /// then `&&`/`||` 200, the comparisons 250, `+`/`-` 500; so chained ternaries group to the
  /// left, `a?b:c?d:e` is `(a?b:c)?d:e`. The ternary yields its chosen branch as that branch
  /// left the result (an integer literal stays `7`, arithmetic gives a real); a missing `:`
  /// branch is 0.
  pub(super) fn formula(i: &mut In) -> ModalResult<f64> {
    let mut condition = logic(i)?;
    loop {
      sb(i);
      if !eat(i, b'?') {
        return Ok(condition);
      }
      number_wanted(i);
      let then_val = logic(i)?;
      let then_int = i.state.int_result;
      let then_text = i.state.text_result.take();
      sb(i);
      let (else_val, else_int) = if eat(i, b':') {
        (logic(i)?, i.state.int_result)
      } else {
        (0.0, false)
      };
      let else_text = i.state.text_result.take();
      i.state.text_result = if condition != 0.0 {
        then_text
      } else {
        else_text
      };
      let (value, int) = if condition != 0.0 {
        (then_val, then_int)
      } else {
        (else_val, else_int)
      };
      i.state.int_result = int;
      condition = value;
    }
  }

  /// logic := relation (('&&'|'||') relation)*, an integer.
  fn logic(i: &mut In) -> ModalResult<f64> {
    let mut left = relation(i)?;
    loop {
      sb(i);
      let op = match i.input.as_bytes() {
        [b'&', b'&', ..] => "&&",
        [b'|', b'|', ..] => "||",
        _ => break,
      };
      let cp = i.checkpoint();
      bump(i, 2);
      number_wanted(i);
      match relation(i) {
        Ok(right) => {
          number_wanted(i);
          left = pgfmath_cmp_op(op, left, right);
          i.state.int_result = true;
        },
        Err(_) => {
          i.reset(&cp);
          break;
        },
      }
    }
    Ok(left)
  }

  /// relation := expr (CMPOP expr)*, left-associative, an integer.
  fn relation(i: &mut In) -> ModalResult<f64> {
    let mut left = expr(i)?;
    loop {
      let cp = i.checkpoint();
      let Some(op) = cmp_op(i) else {
        break;
      };
      number_wanted(i);
      match expr(i) {
        Ok(right) => {
          number_wanted(i);
          left = pgfmath_cmp_op(&op, left, right);
          i.state.int_result = true;
        },
        Err(_) => {
          i.reset(&cp);
          break;
        },
      }
    }
    Ok(left)
  }

  fn cmp_op(i: &mut In) -> Option<String> {
    sb(i);
    let b = i.input.as_bytes();
    if b.len() >= 2
      && let two @ (b"==" | b"!=" | b">=" | b"<=") = &[b[0], b[1]]
    {
      let op = std::str::from_utf8(two.as_slice()).unwrap().to_string();
      bump(i, 2);
      return Some(op);
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
          number_wanted(i);
          result += term(i)?;
          number_wanted(i);
          // Arithmetic yields a real: `2+(1<2)` → "3.0", NOT the integer a bare
          // comparison produces (pgfmath ground truth). A comparison anywhere in
          // an operand set int_result; combining it arithmetically clears that.
          i.state.int_result = false;
        },
        Some(b'-') => {
          bump(i, 1);
          number_wanted(i);
          result -= term(i)?;
          number_wanted(i);
          i.state.int_result = false;
        },
        _ => break,
      }
    }
    Ok(result)
  }

  /// term := factor (('*'|'/') factor | 'r' power_tail)* — postfix `r` (deg, 600) is below `*`/`/` (700), `!` (800)
  /// and `^` (900): pgf's operator stack holds it until the term ends, so it applies to the product so far and to the
  /// powers and factorials after it (`4/2r` (4/2)r, `3*2r^2` ((3*2)^2)r; pgfmathparser.code.tex:765-790, 900-919).
  fn term(i: &mut In) -> ModalResult<f64> {
    let mut result = factor(i)?;
    let mut degrees = 0;
    loop {
      sb(i);
      match first(i) {
        Some(b'r') if !second(i).map(|c| c.is_ascii_alphabetic()).unwrap_or(false) => {
          bump(i, 1);
          degrees += 1;
          result = power_tail(i, result)?;
          i.state.int_result = false;
        },
        Some(b'*') => {
          bump(i, 1);
          number_wanted(i);
          result *= factor(i)?;
          number_wanted(i);
          // See expr(): arithmetic demotes a comparison's integer to a real
          // (`2*(1<2)` → "2.0", not "2").
          i.state.int_result = false;
        },
        Some(b'/') => {
          bump(i, 1);
          number_wanted(i);
          // Infix `/` is pgf's `divide`: a zero divisor leaves the dividend.
          result = pgfmath_divide(result, factor(i)?);
          number_wanted(i);
          i.state.int_result = false;
        },
        _ => break,
      }
    }
    for _ in 0..degrees {
      result = result.to_degrees();
    }
    Ok(result)
  }

  /// factor := simplefactor power_tail
  fn factor(i: &mut In) -> ModalResult<f64> {
    let base = simplefactor(i)?;
    power_tail(i, base)
  }

  /// power_tail := ('^' simplefactor | '!')* — `^` (900) left-associative (`2^3^2` 64.0); a postfix `!` (800) waits
  /// on the stack for the `^` chain after it, so it applies to the whole power (`2^2!` 24.0, `2!^2` (2^2)! 24.0).
  fn power_tail(i: &mut In, mut result: f64) -> ModalResult<f64> {
    let mut factorials = 0;
    loop {
      sb(i);
      match first(i) {
        Some(b'^') => {
          bump(i, 1);
          number_wanted(i);
          let exp = simplefactor(i)?;
          number_wanted(i);
          i.state.int_result = false; // exponentiation yields a real
          result = result.powf(exp);
        },
        Some(b'!') if second(i) != Some(b'=') => {
          bump(i, 1);
          factorials += 1;
          i.state.int_result = false;
        },
        _ => break,
      }
    }
    for _ in 0..factorials {
      result = pgfmath_factorial(result as i64);
    }
    Ok(result)
  }

  /// simplefactor — the atom: parens, prefix ops, numbers (with unit/register
  /// suffix), registers, identifiers (functions/constants), lone dot.
  fn simplefactor(i: &mut In) -> ModalResult<f64> {
    sb(i);
    // A value is a real unless it is an integer literal, a comparison, or an integer function
    // (pgf keeps `7` and `(7)` as typed; `-7`, `\pgf@x`, `2pt`, `max(1,2)` print `.0`).
    i.state.int_result = false;
    match first(i) {
      None => return fail(),
      Some(b'{') => return array_primary(i),
      Some(b'(') => {
        bump(i, 1);
        let val = formula(i)?;
        sb(i);
        eat(i, b')'); // forgive unclosed at end (Perl)
        return Ok(val);
      },
      Some(b'-') => {
        // Unary minus is pgf's `neg` at precedence 500 (pgfmathfunctions.basic.code.tex:46-49), below `r` (600),
        // `*` and `/` (700), `!` (800) and `^` (900): it negates the whole term after it (`-2^2` -4.0, `-3!` -6.0,
        // `8/-2*2` -2.0, `2^-1*3` 0.125 in pdflatex; witness 2311.02367's `exp(-(\y)^2)`).
        bump(i, 1);
        let value = -term(i)?;
        number_wanted(i);
        i.state.int_result = false;
        return Ok(value);
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
        i.state.int_result = true; // `!0` → `1` (live pgf), like `not(0)`
        return Ok(if val == 0.0 { 1.0 } else { 0.0 });
      },
      _ => {},
    }

    if let Some((num, literal)) = try_number(i) {
      sb(i);
      if let Some(unit) = try_unit(i) {
        i.state.units_declared = true;
        return Ok(pgfmath_convert(num, &unit));
      }
      if first(i) == Some(b'\\') {
        let cp = i.checkpoint();
        if let Some(operand) = try_cs_register(i) {
          return Ok(operand.after(i, num));
        }
        i.reset(&cp);
      }
      // An integer literal keeps its form: decimal digits, or a hex/binary literal (`0x1F` 31); one opening with `0`
      // is octal (`010+1` is 9.0, pgfmathparser.code.tex:355-358, 486-497).
      return Ok(match literal {
        Literal::Based => {
          i.state.int_result = true;
          num
        },
        Literal::Decimal(digits) if digits.bytes().all(|b| b.is_ascii_digit()) => {
          i.state.int_result = true;
          match digits.strip_prefix('0') {
            Some(octal) if !octal.is_empty() => {
              i64::from_str_radix(octal, 8).map_or(num, |v| v as f64)
            },
            _ => num,
          }
        },
        _ => num,
      });
    }

    if first(i) == Some(b'\\')
      && let Some(operand) = try_cs_register(i)
    {
      let value = operand.after(i, 1.0);
      sb(i);
      if first(i) == Some(b'\\') {
        let cp = i.checkpoint();
        if let Some(operand) = try_cs_register(i) {
          return Ok(operand.after(i, value));
        }
        i.reset(&cp);
      }
      return Ok(value);
    }

    if first(i).map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
      let name = read_identifier(i);
      if matches!(name.as_str(), "width" | "height" | "depth")
        && let Some(text) = quoted_argument(i)
      {
        return Ok(super::pgfmath_sizer(&name, text));
      }
      if is_builtin_function(&name) {
        return function_call(i, &name);
      }
      if is_builtin_constant(&name) {
        set_int_result_for_fn(i, &name);
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
        open_parse_group(i);
        let (v, int) = super::pgfmath_apply_user_reported(&name, &[], &[]);
        i.state.int_result = int;
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

  /// `("…")`, the quoted text argument of `width`/`height`/`depth` (pgf's SIZER, Perl
  /// pgfmath.code.tex.ltxml:682, :724), consumed; nothing is consumed when it is absent. pgf takes a `{"…"}` as well, its
  /// operand one group: circuitikz's `height{"Q"}` (pgfcircvoltage.tex:192, circuitikzmanual's `raised voltages`), 0 and
  /// no Error before arrays, "Missing number" with them.
  fn quoted_argument<'a>(i: &mut In<'a>) -> Option<&'a str> {
    let input: &'a str = i.input;
    let rest = input.trim_start();
    let close = match rest.as_bytes().first() {
      Some(b'(') => ')',
      Some(b'{') => '}',
      _ => return None,
    };
    let inner = rest[1..].trim_start().strip_prefix('"')?;
    let end = inner.find('"')?;
    let after = inner[end + 1..].trim_start().strip_prefix(close)?;
    bump(i, input.len() - after.len());
    Some(&inner[..end])
  }

  /// After evaluating a USER-declared pgfmath function/constant (NOT a
  /// builtin), inherit the global `\ifpgfmathunitsdeclared` flag that its body
  /// may have set (its body runs with the flag reset by `pgfmath_apply_user`,
  /// then a nested `\pgfmathparse` of a unit'd value sets it). Mirrors Perl,
  /// where the flag is a persistent global threaded through nested evaluations.
  /// Monotonic — only ever sets the flag, as `try_cs_register` does for a
  /// dimension operand (a count leaves it). Call ONLY from the user-function / user-constant
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

  /// Whether a `{` follows blanks (which are consumed): the first argument of `array` and `dim`.
  fn brace_follows(i: &mut In) -> bool {
    let spaces = i.input.bytes().take_while(u8::is_ascii_whitespace).count();
    bump(i, spaces);
    first(i) == Some(b'{')
  }

  /// FUNCTION '(' formula (',' formula)* ')' | FUNCTION simplefactor
  fn function_call(i: &mut In, name: &str) -> ModalResult<f64> {
    sb(i);
    // `array({e0,e1,…}, index)` — the one builtin whose FIRST argument is a
    // brace-delimited expression list, not a formula (pgfmathfunctions.code
    // `\pgfmatharray@`). Parse the elements in place with the shared parser,
    // then select by the (0-based) index; out-of-bounds yields 0 like an
    // empty result. Witness colorblind.sty L433-449
    // `round(255*array({\colorspec},0))`.
    // `dim({…})`, the number of its elements (pgfmathfunctions.misc.code.tex:269-290): an array's, or 1 for any other
    // value (tikzpingus.sty's lists; `dim(5)`).
    if name == "dim" && first(i) == Some(b'(') {
      bump(i, 1);
      let count = if brace_follows(i) {
        array_literal(i)?.len()
      } else {
        formula(i)?;
        i.state.text_result = None;
        1
      };
      sb(i);
      eat(i, b')');
      i.state.int_result = true;
      return Ok(count as f64);
    }
    if name == "array" && first(i) == Some(b'(') {
      bump(i, 1);
      if brace_follows(i) {
        let elements = array_literal(i)?;
        sb(i);
        let index = if eat(i, b',') { formula(i)? } else { 0.0 };
        number_wanted(i);
        sb(i);
        eat(i, b')');
        let element = list_element(elements, index);
        return Ok(array_value(i, element));
      }
      sb(i);
      // Brace-less form: pgfmath's own macro layers can strip the list
      // braces before the expression reaches this evaluator, leaving
      // `array(3,7,5,1)` — the LAST value is the index, the rest the
      // elements.
      let mut elems: Vec<f64> = Vec::new();
      // Each element's integer form: `array` returns the chosen element as it is.
      let mut elem_ints: Vec<bool> = Vec::new();
      while let Ok(v) = formula(i) {
        elems.push(v);
        elem_ints.push(i.state.int_result);
        sb(i);
        if !eat(i, b',') {
          break;
        }
      }
      elem_ints.pop();
      let idx = elems.pop().unwrap_or(0.0);
      sb(i);
      eat(i, b')');
      let idx = idx.round();
      let (val, int) = if idx >= 0.0 {
        (
          elems.get(idx as usize).copied().unwrap_or(0.0),
          elem_ints.get(idx as usize).copied().unwrap_or(false),
        )
      } else {
        (0.0, false)
      };
      i.state.int_result = int;
      return Ok(val);
    }
    if first(i) == Some(b'(') {
      bump(i, 1);
      let mut args = vec![formula(i)?];
      number_wanted(i);
      // Each argument's integer form: `ifthenelse` returns its chosen argument as it is
      // (pgfmathfunctions.comparison.code.tex:77-86), text too, so its branches want no number.
      let mut arg_ints = vec![i.state.int_result];
      let mut branch_texts = Vec::new();
      loop {
        sb(i);
        if eat(i, b',') {
          match formula(i) {
            Ok(arg) => {
              if name == "ifthenelse" {
                branch_texts.push(i.state.text_result.take());
              } else {
                number_wanted(i);
              }
              args.push(arg);
              arg_ints.push(i.state.int_result);
            },
            _ => {
              break;
            },
          }
        } else {
          break;
        }
      }
      sb(i);
      eat(i, b')');
      if !is_builtin_function(name) {
        open_parse_group(i);
        let (value, int) = super::pgfmath_apply_user_reported(name, &args, &arg_ints);
        i.state.int_result = int;
        return Ok(value);
      }
      if name == "ifthenelse" && args.len() == 3 {
        let chosen = if args[0] != 0.0 { 1 } else { 2 };
        i.state.int_result = arg_ints[chosen];
        i.state.text_result = branch_texts.swap_remove(chosen - 1);
      } else {
        set_int_result_for_fn(i, name);
      }
      return Ok(pgfmath_apply_fn(name, &args));
    }
    let arg = simplefactor(i)?;
    number_wanted(i);
    if !is_builtin_function(name) {
      open_parse_group(i);
      let (value, int) = super::pgfmath_apply_user_reported(name, &[arg], &[i.state.int_result]);
      i.state.int_result = int;
      return Ok(value);
    }
    set_int_result_for_fn(i, name);
    Ok(pgfmath_apply_fn(name, &[arg]))
  }

  /// Real pgfmath returns BARE integers from the comparison/logic function
  /// forms — probed against pdflatex/pgf TL2025 (2026-08-31): `notless(5,3)`
  /// → `1`, `equal(2,2)` → `1`, `and(1,1)` → `1`, `not(1)` → `0`,
  /// `int(5/2)` → `2` — while arithmetic keeps the fixed-point `.0`
  /// (`2*3` → `6.0`). Without this, `\edef`'d results like `1.0` broke every
  /// downstream `\ifnum\pgfmathresult=1` (pgf-spectra.sty L490/L503 spectral
  /// loop: 7k `expected:<relationaltoken>` errors PER DOC, ~18k corpus-wide;
  /// witnesses pgf-spectra/pgf-spectraManual, *PreviewDataLSE, *PreviewDataNIST).
  /// Same flag the infix comparison path sets; surrounding arithmetic still
  /// clears it (`2+(equal(1,1))` → `3.0`). `ifthenelse` returns its chosen argument as
  /// it is, and `function_call` takes that argument's form instead.
  fn set_int_result_for_fn(i: &mut In, name: &str) {
    i.state.int_result = matches!(
      name,
      "equal"
        | "greater"
        | "less"
        | "notequal"
        | "notgreater"
        | "notless"
        | "and"
        | "or"
        | "not"
        | "int"
        // Integer-literal results in pgf (`\def\pgfmathresult{1}`, no
        // `\pgfmath@tonumber`): `sign` (pgfmathfunctions.basic.code.tex:312-
        // 325), `iseven`/`isodd`/`isprime`/`gcd`/`div` (integerarithmetics),
        // `scalar`, `true`/`false`. Probed against pdflatex/pgf TL2025
        // (2026-09-03): `sign(-2.5)`=`-1`, `iseven(4)`=`1`, `gcd(12,18)`=`6`,
        // `div(7,2)`=`3`, `scalar(3)`=`3`, `true`=`1` — while `floor`/`ceil`/
        // `round`/`abs`/`max`/`min`/`factorial` keep the `.0`. tikzbricks.sty:
        // 146-151 does `\ifnum\brick@sin<0` on `sign(sin(…))` and the former
        // `-1.0` broke at the `.` ("Expected a relational token", tikzbricks
        // doc 90 errors; Perl identical). Guard:
        // `perfect_kernel_batch54::pgfmath_integer_functions_yield_integers`.
        | "sign"
        | "iseven"
        | "isodd"
        | "isprime"
        | "gcd"
        | "div"
        | "scalar"
        | "true"
        | "false"
    );
  }

  /// pgf's number (pgfmathparser.code.tex:426-540): digits and points, with the spaces between them skipped as pgf's
  /// argument reading skips them (`-4 .5` is -4.5, 1612.03043); then an exponent `e`/`E` and its integer, or after a
  /// lone `0` a base prefix `x`/`X`, `b`/`B` and its digits, unless the letter opens a unit (`0bp`, 1106.3771,
  /// 1104.5553, 1503.00895). Returns the value and the literal as pgf collected it.
  fn try_number(i: &mut In) -> Option<(f64, Literal)> {
    sb(i);
    let b = i.input.as_bytes();
    let skip = |mut at: usize| {
      while b.get(at) == Some(&b' ') {
        at += 1;
      }
      at
    };
    let mut pos = 0;
    let mut number = String::new();
    loop {
      let at = skip(pos);
      match b.get(at) {
        Some(c) if c.is_ascii_digit() || *c == b'.' => {
          number.push(*c as char);
          pos = at + 1;
        },
        _ => break,
      }
    }
    if !number.bytes().any(|c| c.is_ascii_digit()) {
      return None;
    }
    let at = skip(pos);
    let next = b.get(at).copied();
    let radix = match next {
      Some(b'x' | b'X') => 16,
      Some(b'b' | b'B') => 2,
      _ => 0,
    };
    if number == "0" && radix != 0 && !opens_unit(&i.input[at..]) {
      let mut digits = String::new();
      let mut end = at + 1;
      loop {
        let at = skip(end);
        match b.get(at) {
          Some(c) if (*c as char).is_digit(radix) => {
            digits.push(*c as char);
            end = at + 1;
          },
          _ => break,
        }
      }
      let value = i64::from_str_radix(&digits, radix).ok()? as f64;
      bump(i, end);
      return Some((value, Literal::Based));
    }
    if matches!(next, Some(b'e' | b'E')) {
      // `\pgfmath@count` reads the exponent as TeX reads an integer: spaces, a sign, digits.
      let mut end = skip(at + 1);
      let negative = b.get(end) == Some(&b'-');
      if matches!(b.get(end), Some(b'+' | b'-')) {
        end = skip(end + 1);
      }
      let digits = b[end.min(b.len())..]
        .iter()
        .take_while(|c| c.is_ascii_digit())
        .count();
      if digits > 0 {
        let exponent = &i.input[end..end + digits];
        let sign = if negative { "-" } else { "" };
        let value = format!("{number}e{sign}{exponent}")
          .parse::<f64>()
          .unwrap_or(0.0);
        bump(i, end + digits);
        return Some((value, Literal::Scientific));
      }
    }
    let value = number.parse::<f64>().unwrap_or(0.0);
    bump(i, pos);
    Some((value, Literal::Decimal(number)))
  }

  /// Whether `text` opens with one of pgf's two-letter units, the letters read as pgf reads them, spaces between them
  /// skipped, and no letter after them.
  fn unit_at(text: &str) -> Option<(&'static str, usize)> {
    let b = text.as_bytes();
    PGF_UNITS.iter().find_map(|unit| {
      let u = unit.as_bytes();
      if b.first() != Some(&u[0]) {
        return None;
      }
      let second = 1 + b[1..].iter().take_while(|c| **c == b' ').count();
      let end = second + 1;
      (b.get(second) == Some(&u[1]) && !b.get(end).is_some_and(|c| c.is_ascii_alphabetic()))
        .then_some((*unit, end))
    })
  }

  fn opens_unit(text: &str) -> bool { unit_at(text).is_some() }

  fn try_unit(i: &mut In) -> Option<String> {
    sb(i);
    let (unit, end) = unit_at(i.input)?;
    bump(i, end);
    Some(unit.to_string())
  }

  /// `\\[a-zA-Z@]+` — a TeX register; reading a dimension declares units (see the
  /// pgfplots ybar witnesses on the original implementation). An expl3 name
  /// (`\l_tmpa_dim`, `\c_pgfinterference_delta_dim`) is ONE control sequence
  /// in the tokens `\pgfmathparse` received; the string round-trip
  /// (`pgfmath_input_text`) plus this alphabetic-only scan split it at `_`
  /// and looked up `\l`/`\c` — 200k "not a register" warnings and zero
  /// geometry per pixel on pgf-interference (412 s; Perl pgfmath.code.tex
  /// .ltxml:704 splits the same way, pdflatex reads the register token). When
  /// the maximal `[a-zA-Z@_:]` run names a live register, that is the name.
  /// Guard: `perfect_kernel_batch56::pgfmath_reads_expl3_register_names`.
  ///
  /// A dimension scales a number before it (`2\mydim`); a count or an e-TeX expression takes its place, as pgf
  /// reads them (pgfmathparser.code.tex:430-476): `2\mycount` is the count, `0.5\dimexpr 4pt` is 4.0.
  fn try_cs_register(i: &mut In) -> Option<CsOperand> {
    sb(i);
    let n = cs_name_len(i.input)?;
    let cs = &i.input[..1 + n];
    if matches!(cs, "\\wd" | "\\ht" | "\\dp") {
      let dimension = cs;
      let cp = i.checkpoint();
      bump(i, 1 + n);
      if let Some(number) = box_number(i) {
        i.state.units_declared = true;
        return Some(CsOperand::Scales(super::pgfmath_box_dimension(
          dimension, number,
        )));
      }
      i.reset(&cp);
    }
    if matches!(cs, "\\dimexpr" | "\\glueexpr" | "\\numexpr" | "\\muexpr") {
      let (value, read) = super::pgfmath_read_expr(cs, i.input)?;
      bump(i, read);
      if matches!(cs, "\\numexpr") {
        return Some(CsOperand::Count(value));
      }
      i.state.units_declared |= matches!(cs, "\\dimexpr" | "\\glueexpr");
      return Some(CsOperand::Replaces(value));
    }
    let (value, count) = pgfmath_register_lookup(cs);
    bump(i, 1 + n);
    if count {
      return Some(CsOperand::Count(value));
    }
    i.state.units_declared = true;
    Some(CsOperand::Scales(value))
  }

  /// The name's length after the `\` of the control sequence `input` starts with: the `[a-zA-Z@]` run, or the longer
  /// `[a-zA-Z@_:]` run when that names a live register (an expl3 name, see `try_cs_register`); `None` for no name.
  fn cs_name_len(input: &str) -> Option<usize> {
    let b = input.as_bytes().strip_prefix(b"\\")?;
    let n = b
      .iter()
      .take_while(|c| c.is_ascii_alphabetic() || **c == b'@')
      .count();
    if n == 0 {
      return None;
    }
    let n_long = n
      + b[n..]
        .iter()
        .take_while(|c| c.is_ascii_alphabetic() || matches!(**c, b'@' | b'_' | b':'))
        .count();
    let long_register =
      n_long > n && ::latexml_core::state::lookup_register_quiet(&input[..1 + n_long]).is_some();
    Some(if long_register { n_long } else { n })
  }

  /// The box after `\wd`/`\ht`/`\dp`: a register holding its number (`\wd\mybox`, an expl3 `\wd\l_my_box`; Perl's
  /// `/\\wd/ CS`, pgfmath.code.tex.ltxml:696-701; tikzscale.sty:522-524), or decimal digits (`\wd0`, which TeX and pgf
  /// read; Perl's grammar reads none and returns 0).
  fn box_number(i: &mut In) -> Option<i64> {
    let spaces = i.input.bytes().take_while(|c| *c == b' ').count();
    let rest = &i.input[spaces..];
    if let Some(n) = cs_name_len(rest) {
      let number = pgfmath_register_lookup(&rest[..1 + n]).0 as i64;
      bump(i, spaces + 1 + n);
      return Some(number);
    }
    if rest.starts_with('\\') {
      return None;
    }
    let digits = rest.bytes().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
      return None;
    }
    let number = rest[..digits].parse().ok()?;
    bump(i, spaces + digits);
    Some(number)
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

/// pgf's `width("…")`/`height("…")`/`depth("…")`: the text typeset in a box, its dimension in
/// points (pgfmathfunctions.misc.code.tex `\pgfmathwidth@`; Perl `pgfmath_sizer`,
/// pgfmath.code.tex.ltxml:523-537 digests the text). Measured in a fresh restricted
/// horizontal box, as calc's `\widthof`. wheelchart's `value=width("\WCvarC")` (wheelchart.tex)
/// sized every slice 0.
fn pgfmath_sizer(dimension: &str, text: &str) -> f64 {
  let measure = || -> Result<f64> {
    begin_mode("restricted_horizontal")?;
    // Typeset after `\pgfmath@selectfont` (pgfmathparser.code.tex:132): in a picture the document's font, not the
    // `\nullfont` that measures every character 0.
    if let Some(font) = pgfmath_font() {
      assign_font(Rc::new(font), Some(Scope::Local));
    }
    let boxed = digest(mouth::tokenize_internal(TeXString::assembled(
      text.to_string(),
    )));
    end_mode("restricted_horizontal")?;
    let boxed = boxed?;
    let value = match dimension {
      "height" => boxed.get_height(),
      "depth" => boxed.get_depth(),
      _ => boxed.get_width(None)?,
    };
    Ok(value.map_or(0.0, |v| v.value_of() as f64 / 65536.0))
  };
  with_own_adjust_queue(measure).unwrap_or(0.0)
}

/// An Error of the native parser (`pgfmath_grammar`).
fn report_pgfmath_error(category: &str, object: &str, message: &str) {
  let _ = (|| -> Result<()> {
    Error!(category, object, message);
    Ok(())
  })();
}

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
  // pgf's `\pgfmath@x=…pt` stops on "Dimension too large" (tex.web §460); Perl errs too (.ltxml:391-393), on the
  // five-decimal result, so `\maxdimen` (16383.99998474…) is in range.
  if ((result * 1e5).round() / 1e5).abs() > MAX_PGF_NUMBER {
    report_pgfmath_error(
      "pgfmath",
      "overflow",
      &format!("Dimension too large: {result}. Input was: {input}"),
    );
  }
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

/// The seven calc-package names pgf `\let`s to its own for a parse (pgfmathparser.code.tex:124-130).
const CALC_COMPAT: [(&str, &str); 7] = [
  ("\\real", "\\pgfmath@calc@real"),
  ("\\minof", "\\pgfmath@calc@minof"),
  ("\\maxof", "\\pgfmath@calc@maxof"),
  ("\\ratio", "\\pgfmath@calc@ratio"),
  ("\\widthof", "\\pgfmath@calc@widthof"),
  ("\\heightof", "\\pgfmath@calc@heightof"),
  ("\\depthof", "\\pgfmath@calc@depthof"),
];

/// pgf `\let`s the calc names in `\pgfmathparse@@` (pgfmathparser.code.tex:124-130), after its edef expanded the
/// argument (:54): a user's `\newcommand\ratio{0.5}` reads as 0.5, and only a calc name the edef left reads as pgf's.
/// Perl lets them first (KNOWN_PERL_ERRORS #1225). Per parse, never for the document: `\real` as ℝ (1608.06741).
/// TL 2025's pgf errs on every such name ("Unknown function", "Unknown operator"); calc's value is semantic: a Warning.
fn calc_compat_expand(expanded: Tokens) -> Result<Tokens> {
  let names: [Token; 7] = CALC_COMPAT.map(|(cs, _)| T_CS!(cs));
  let Some(name) = expanded.unlist_ref().iter().find(|t| names.contains(t)) else {
    return Ok(expanded);
  };
  Warn!(
    "unexpected",
    name.to_string(),
    s!("pgf's parser errs on calc's {name} in an expression; calc's value is used")
  );
  let saved: Vec<(Token, Stored)> = names
    .iter()
    .map(|t| (*t, lookup_meaning(t).unwrap_or(Stored::None)))
    .collect();
  for (name, (_, helper)) in names.iter().zip(CALC_COMPAT) {
    let_i(name, &T_CS!(helper), None);
  }
  let reexpanded = do_expand(expanded);
  for (t, m) in saved {
    assign_meaning(&t, m, None);
  }
  reexpanded
}

/// Perl's `sub pgfmathparse` (pgfmath.code.tex.ltxml:316-394), which `\pgfmathparse`, `\lx@pgfmath@parseX` and
/// `\pgfmathsetmacro` share: expand, read the text, evaluate; returns the result and whether units were declared.
pub(crate) fn pgfmathparse(tokens: Tokens) -> Result<(String, bool)> {
  pgfmathparse_expanded(do_expand(tokens)?)
}

/// `pgfmathparse` on an argument its caller expanded (`\pgfmathsetlength` looks for pgf's `+` first). Sets both
/// units flags globally, as pgf's parse does (pgfmathparser.code.tex:51-52, 443-503); no `mu` is read yet.
pub(crate) fn pgfmathparse_expanded(expanded: Tokens) -> Result<(String, bool)> {
  let (result, units) =
    pgfmathparse_eval_with_units(&pgfmath_input_text(calc_compat_expand(expanded)?));
  set_condition(
    &T_CS!("\\ifpgfmathunitsdeclared"),
    units,
    Some(Scope::Global),
  );
  set_condition(
    &T_CS!("\\ifpgfmathmathunitsdeclared"),
    false,
    Some(Scope::Global),
  );
  Ok((result, units))
}

/// The parser's text of an expanded argument: its string, keeping the space TeX's mouth skipped after a control word
/// where a letter follows (`\mydim r` is `\mydim` then `r`, not `\mydimr` as Perl's pgfmath.code.tex.ltxml:328 reads;
/// KNOWN_PERL_ERRORS #1226).
fn pgfmath_input_text(tokens: Tokens) -> String {
  let mut spaced = Vec::with_capacity(tokens.unlist_ref().len());
  let mut after_control_word = false;
  for t in tokens.unlist() {
    let code = t.get_catcode();
    if after_control_word && code == Catcode::LETTER {
      spaced.push(T_SPACE!());
    }
    // A control word as tex.web's print_cs (§262) reads one: a name of two or more characters, or a single letter.
    after_control_word = code == Catcode::CS
      && t.with_str(|name| {
        let mut chars = name.strip_prefix('\\').unwrap_or(name).chars();
        match (chars.next(), chars.next()) {
          (Some(c), None) => lookup_catcode(c) == Some(Catcode::LETTER),
          (first, _) => first.is_some(),
        }
      });
    spaced.push(t);
  }
  Tokens::new(spaced).to_string()
}

/// Main pgfmathparse evaluation function
/// Perl: sub pgfmathparse (L316-394)
/// Returns (result_string, units_declared)
pub(crate) fn pgfmathparse_eval_with_units(raw_input: &str) -> (String, bool) {
  // Normalize whitespace
  let input: String = raw_input.split_whitespace().collect::<Vec<_>>().join(" ");
  let mut input = input.trim();
  // `\pgfmath@bgroup@strip` takes a whole formula's braces (pgfmathparser.code.tex:139); the group under the last pair
  // the grammar reads, an array or a grouping (`{{"a","b"}}` is `{a}{b}`)
  while let Some(inner) = input
    .strip_prefix('{')
    .and_then(|rest| rest.strip_suffix('}'))
    && pgfmath_grammar::matching_brace(input) == Some(input.len() - 1)
    && inner.trim_start().starts_with('{')
    && pgfmath_grammar::matching_brace(inner.trim()) == Some(inner.trim().len() - 1)
  {
    input = inner.trim();
  }

  // 0. String-valued ternary / ifthenelse — Perl pgfmath uses untyped scalars so the `?:` operator
  //    and `ifthenelse(...)` can return string values like `"black"` or `"pgreen!50"` for color
  //    names. Our parser is f64-only, so detect the pattern <test> ? "a" : "b"     OR
  //    ifthenelse(<test>, "a", "b") early, evaluate the test as a number, and return the chosen
  //    string literal. Drivers: 2601.14798 (color jitter heatmap), Stage-20 v6 ~1579 errors from
  //    \pgfmathsetmacro{\clr}{...}.
  if let Some(result) = try_string_ternary(input) {
    return (result, false);
  }

  // 0b. Bare string literal (arXiv-fork e0dd3e33/aeb24f2d: string results
  //     pass through untouched, including the empty string) — PGF strings
  //     like `\pgfmathparse{"south east"}` feed anchors/colors verbatim.
  {
    let trimmed = input.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
      let inner = &trimmed[1..trimmed.len() - 1];
      if !inner.contains('"') {
        return (inner.to_string(), false);
      }
    }
  }

  // 1. Simple number check (Perl L332-345)
  if let Some(result) = try_simple_number(input) {
    return (result, false);
  }

  // 2. Unit expression check (Perl L352-354): /^([+-]?[\d\.]+)(UNIT)$/ Handled by the parser below,
  //    since it's a subset of the grammar.

  // 3. Parse with the winnow grammar (replaces both Perl eval and RecDescent; partial parses still
  //    return what was read, as before).
  let (parsed, state) = pgfmath_grammar::evaluate(input);
  if parsed.is_ok()
    && let Some(text) = state.text_result
  {
    return (text, false);
  }
  if let Ok(result) = parsed {
    // arXiv-fork c52d3cf6: a top-level comparison returns an integer string.
    if state.int_result && result == (result as i64) as f64 {
      return (format!("{}", result as i64), state.units_declared);
    }
    return (format_parse_result(result, input), state.units_declared);
  }

  // 4. Fallback
  ("0.0".to_string(), false)
}

/// The result string alone, for the golden rows.
#[cfg(test)]
fn pgfmathparse_eval(raw_input: &str) -> String { pgfmathparse_eval_with_units(raw_input).0 }

/// Detect string-valued pgfmath ternaries `<test> ? "a" : "b"` or
/// `ifthenelse(<test>, "a", "b")` and return the chosen literal.
/// Returns None if the pattern doesn't match or contains no string args.
fn try_string_ternary(input: &str) -> Option<String> {
  // Strip ifthenelse(...) wrapper if present
  let inner = if let Some(rest) = input.strip_prefix("ifthenelse(") {
    let body = rest.strip_suffix(')')?;
    // Re-shape as ternary: split on top-level commas (depth 0 of nested parens/quotes)
    let parts = split_top_commas(body)?;
    if parts.len() != 3 {
      return None;
    }
    format!(
      "{} ? {} : {}",
      parts[0].trim(),
      parts[1].trim(),
      parts[2].trim()
    )
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
  let test_val = pgfmath_grammar::evaluate(cond.trim()).0.ok()?;
  let chosen = if test_val != 0.0 {
    then_part
  } else {
    else_part
  };
  // Strip outer quotes if present; a branch reading an array (`{"a","b"}[0]`) is a formula; otherwise return as-is
  let trimmed = chosen.trim();
  if trimmed.starts_with('{') {
    return Some(pgfmathparse_eval_with_units(trimmed).0);
  }
  let unq = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
    &trimmed[1..trimmed.len() - 1]
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
      b'?' if !in_quote && depth == 0 => {
        q_pos = Some(i);
        break;
      },
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
        return Some((&s[..qp], &s[qp + 1..j], &s[j + 1..]));
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
    pgfmath_sp_result_tokens(pgfmath_sp(&a) + pgfmath_sp(&b))
  });
  DefMacro!("\\pgfmathsubtract@ {} {}", sub[(a, b)] {
    pgfmath_sp_result_tokens(pgfmath_sp(&a) - pgfmath_sp(&b))
  });
  DefMacro!("\\pgfmathneg@ {}", sub[(a)] {
    pgfmath_sp_result_tokens(-pgfmath_sp(&a))
  });
  DefMacro!("\\pgfmathmultiply@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(parse_pgf_number(&a) * parse_pgf_number(&b))
  });
  DefMacro!("\\pgfmathdivide@ {} {}", sub[(a, b)] {
    pgfmath_sp_result_tokens(pgfmath_divide_sp(pgfmath_sp(&a), pgfmath_sp(&b)))
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
    pgfmath_result_tokens(pgfmath_mod(parse_pgf_number(&a), parse_pgf_number(&b), false))
  });
  DefMacro!("\\pgfmathMod@ {} {}", sub[(a, b)] {
    pgfmath_result_tokens(pgfmath_mod(parse_pgf_number(&a), parse_pgf_number(&b), true))
  });

  // Degree/radian conversion
  DefMacro!("\\pgfmathrad@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).to_radians())
  });
  DefMacro!("\\pgfmathdeg@ {}", sub[(a)] {
    pgfmath_result_tokens(parse_pgf_number(&a).to_degrees())
  });

  // Integer operations
  // `\pgfmathint@` is the one `@`-function that does NOT go through
  // `\pgfmath@returnone`/`\pgfmath@tonumber`: pgfmathfunctions.round.code.tex:83-96
  // sets `\pgfmathresult` to the bare integer (`6`, not `6.0`; verified with
  // pdflatex — Gemini round-10 N1 review).
  DefMacro!("\\pgfmathint@ {}", sub[(a)] {
    let v = parse_pgf_number(&a) as i64;
    let mut toks = vec![T_CS!("\\def"), T_CS!("\\pgfmathresult"), T_BEGIN!()];
    toks.extend(Explode!(v.to_string()));
    toks.push(T_END!());
    toks
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
  // `calc_compat_expand`, which binds them for the one parse, after its
  // expansion as pgf does.

  // ==================== pgfmathparse override ====================
  // Perl L401-403: DefMacro('\lx@pgfmath@parse{}', sub { ... })
  DefMacro!("\\lx@pgfmath@parse {}", sub[(tokens)] {
    let (result, _units) = pgfmathparse(tokens)?;
    pgfmath_result_tokens_str(&result)
  });

  // Perl L442: Let('\pgfmathparse', '\lx@pgfmath@parse');
  Let!("\\pgfmathparse", "\\lx@pgfmath@parse");

  // Perl L396-399: \lx@pgfmath@parseX (alternative that uses \lx@pgfmathresult)
  DefMacro!("\\lx@pgfmath@parseX {}", sub[(tokens)] {
    let (result, _units) = pgfmathparse(tokens)?;
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
    // EXPAND before testing the leading-'+' fast-path. pgf fast-paths a
    // '+'-prefixed (relative/glue) value through native glue reading, which CAN
    // read box dimension registers like `\wd<box>` that the pgfmath expression
    // parser cannot (pgfmath returns 0 for `\wd`/`\ht`/`\dp`). The value is very
    // often delivered via a MACRO — e.g. the pgf decoration automaton calls
    // `\pgfmathsetlength\pgf@decorate@distancetomove{\pgf@decorate@width}` where
    // `\pgf@decorate@width` expands to `+.5\wd\pgf@lib@dec@text@box`. Testing the
    // *raw* first token sees the macro (not '+'), so we fell to pgfmathparse,
    // `\wd<box>` → 0, the decoration never advanced along the path, and
    // `text along path` (decorations.text) looped forever placing boxes → RSS
    // runaway. Expanding first lets the '+' surface (pdflatex ground truth:
    // `\pgfmathsetlength\d{+.5\wd0}` = 3.75pt). Witnesses: arXiv 1709.07916 /
    // 1912.13052 / 2004.14791 / 1312.6499 (STABILITY_WITNESSES Cluster E).
    let toks: Vec<Token> =
      do_expand(Tokens::new(tokens.unlist_ref().clone()))?
        .unlist_ref()
        .iter()
        .skip_while(|t| t.get_catcode() == Catcode::SPACE)
        .cloned()
        .collect();

    if toks.first().is_some_and(|t| t.text == pin!("+")) {
      // Starts with '+' → read as TeX reads the assignment, from the argument alone: pgf
      // assigns `#1#2\unskip` (pgfmathcalc.code.tex:30-38), so the scan ends with the
      // argument. Read from the live input, the optional space and `plus` keyword scan after
      // the unit expanded the next token — `\ifdim\pgfdecoratedremainingdistance<\pgf@x`
      // (pgfmoduledecorations.code.tex:1020-1035) then compared the stale `\pgf@x`, and text
      // along a left-to-right path stopped short (wheelchart `arc data`). A dimen register
      // takes a dimension (no `plus`/`minus`), a skip register glue. Perl reads the live input
      // too (pgfmath.code.tex.ltxml:412-416).
      // Tokens after the value stay in the input, typeset after the assignment as in TeX
      // (`\pgfmathsetlength\d{+3pt XY}` prints "XY", a dimen's `plus 2pt` is text) — here after the group,
      // where pgf sets them inside it (KNOWN_PERL_ERRORS #437 residual).
      // That quick path reads in a group under `\pgfmath@selectfont`: `em` is the document font's, not a picture's
      // `\nullfont` (KNOWN_PERL_ERRORS #437); the register is assigned after it, and the group closes on an error.
      let register_type = lookup_register_definition(&register).and_then(|defn| defn.register_type());
      begingroup();
      if let Some(font) = pgfmath_font() {
        assign_font(Rc::new(font), Some(Scope::Local));
      }
      let read = Mouth::new("", None).and_then(|mouth| reading_from_mouth(mouth, move || {
        unread(Tokens::new(toks));
        let value: RegisterValue = match register_type {
          Some(RegisterType::Glue) => read_glue()?.into(),
          Some(RegisterType::MuGlue) => read_mu_glue()?.into(),
          Some(RegisterType::MuDimension) => read_mu_dimension()?.into(),
          _ => read_dimension()?.into(),
        };
        let mut rest = Vec::new();
        while let Some(token) = read_token()? {
          rest.push(token);
        }
        // pgf's `\unskip` after `#2` removes a trailing space (pgf's decoration states pass
        // `+.5\pgfdecorationsegmentlength ` with one).
        while rest.last().is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
          rest.pop();
        }
        Ok((value, rest))
      }));
      endgroup()?;
      let (value, rest): (RegisterValue, Vec<Token>) = read?;
      let cs = register.to_string();
      assign_register(&cs, value, None, vec![])?;
      if !rest.is_empty() {
        unread(Tokens::new(rest));
      }
    } else {
      // pgf's `\pgfmathparse{#2}` (pgfmathcalc.code.tex:40) on the tokens expanded above; its units are
      // `\pgfmath@selectfont`'s (`pgfmath_convert`), and it sets both units flags globally.
      let (result_str, _units) = pgfmathparse_expanded(Tokens::new(toks))?;
      let value: f64 = result_str.parse().unwrap_or(0.0);

      // Perl L419-422: check \ifpgfmathmathunitsdeclared for mu units
      let cs = register.to_string();
      let is_mu = if_condition(&T_CS!("\\ifpgfmathmathunitsdeclared"))
        .unwrap_or(None) == Some(true);
      if is_mu {
        let mu_sp = convert_unit("mu");
        let mu_dim = Dimension((value * mu_sp as f64).round() as i64);
        assign_register(&cs, mu_dim.into(), None, vec![])?;
      } else {
        // Convert pgfmath result (in pt) to sp (× 65536), round to nearest
        let dim = Dimension((value * 65536.0).round() as i64);
        assign_register(&cs, dim.into(), None, vec![])?;
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
  // The definition smuggled is the whole argument, as pgfmathutil.code.tex:295-296 writes `#1` twice: a
  // `\pgfmathtruncatemacro{\y0}` defines `\y` delimited by `0\expandafter` and never expands it. Perl took only the
  // argument's first token, so `\y` was expanded without its `0` (KNOWN_PERL_ERRORS #464; witness 2606.15113, six
  // "Missing argument" errors, pdflatex clean).
  DefMacro!("\\pgfmath@smuggleone Until:\\endgroup", sub[(arg)] {
    let toks: Vec<Token> = arg.unlist_ref().iter()
      .filter(|t| !matches!(t.get_catcode(), Catcode::COMMENT | Catcode::MARKER))
      .copied()
      .collect();
    let first = toks.iter().find(|t| t.get_catcode() != Catcode::SPACE).copied();
    let smuggle = first.is_some_and(|first| {
      matches!(lookup_definition(&first), Ok(Some(defn)) if defn.is_expandable())
    });
    if smuggle {
      // Texlive 2020 definition: smuggle by expanding before endgroup
      let mut out = vec![
        T_CS!("\\expandafter"), T_CS!("\\endgroup"),
        T_CS!("\\expandafter"), T_CS!("\\def"),
        T_CS!("\\expandafter"),
      ];
      out.extend(toks.iter().copied());
      out.push(T_CS!("\\expandafter"));
      out.push(T_BEGIN!());
      out.extend(toks.iter().copied());
      out.push(T_END!());
      out
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
  /// (captured 2026-06-10) — incl. its deliberate quirks (pass-through
  /// formatting of "+3"/"3."); rows re-blessed against live pgf say so.
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
      // Hex and binary literals are integers in pgf (lualatex/pgf TL2025, 2026-10-01).
      ("0x1F", "31@-"),
      ("0b101", "5@-"),
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
      ("3>2", "1@-"),
      ("2>=3", "0@-"),
      ("1==1", "1@-"),
      ("1!=2", "1@-"),
      ("1&&0", "0@-"),
      ("0||1", "1@-"),
      // `?:` binds looser than the comparisons (pgfmathparser.code.tex:898-916); the integer
      // literal branch is kept as typed (lualatex/pgf TL2025, 2026-10-01).
      ("3>2 ? 10 : 20", "10@-"),
      ("(1<2)&&(3<4)", "1@-"),
      // `!` yields an integer literal in real pgf (`!0` → `1`), like `not()`;
      // re-blessed 2026-09-03 against pdflatex/pgf TL2025 (batch 54o).
      ("!1", "0@-"),
      ("!0", "1@-"),
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
      // DTL1: pgf's operator precedences (pgfmathparser.code.tex:898-919, `neg` 500 at
      // pgfmathfunctions.basic.code.tex:46-49): unary minus negates the whole term after it, `^` is
      // left-associative, `r` applies to the product so far. pdflatex/pgf TL2025
      // values; where pgf's own series rounds the last digit the row says so (2311.02367).
      ("-2^2", "-4.0@-"),
      ("-3!", "-6.0@-"),
      ("2^-1", "0.5@-"),
      ("-(2)^2", "-4.0@-"),
      ("exp(-(1)^2)", "0.36788@-"), // pdflatex 0.36787 (pgf's exp series), Perl 0.36788
      ("8/-2*2", "-2.0@-"),
      ("2^-1*3", "0.125@-"),
      ("2*-3^2", "-18.0@-"),
      ("-2^2+1", "-3.0@-"),
      ("3--2^2", "7.0@-"),
      ("-2^-2", "-0.25@-"),
      ("-2pt^2", "-4.0@u"),
      ("2^3^2", "64.0@-"),
      ("2^2!", "24.0@-"),
      ("-2^2!", "-24.0@-"),
      // `r`'s last digit: our f64 against pgf's fixed-point TeX dimens (pdflatex 114.59155, 344.77466).
      ("4/2r", "114.59156@-"),
      ("1+2*3r", "344.77468@-"),
      // DTL1e: a postfix `!` or `r` waits for the stronger operators after it (pgfmathparser.code.tex:765-790); the
      // pdflatex values of `graphics-tikz/pgfmath_postfix_before_power.tex` and its siblings (revDTL1b S1, P3).
      ("2!^2", "24.0@-"),
      ("2!^2*3", "72.0@-"),
      ("-2!^2", "-24.0@-"),
      ("2r^2", "229.18312@-"),    // pdflatex 229.1831
      ("-2r^2", "-229.18312@-"),  // pdflatex -229.1831
      ("3r^2", "515.66202@-"),    // pdflatex 515.66199
      ("3*2r^2", "2062.64806@-"), // pdflatex 2062.64795
      ("2r^2!", "1375.09871@-"),  // pdflatex 1375.09863
      ("3r!", "343.77468@-"),     // pdflatex 343.77466
      ("-1r!", "-57.29578@-"),
      // pgf's number (pgfmathparser.code.tex:426-540; pdflatex TL 2025, 2026-11-12): a unit before a base prefix
      // (`0bp`, 1106.3771), the spaces inside a number skipped (`-4 .5`, 1612.03043), either case of prefix, octal.
      ("0bp+1pt", "1.0@u"),
      ("0bp", "0.0@u"),
      ("2*0bp", "0.0@u"),
      ("0B101", "5@-"),
      ("0X1F", "31@-"),
      ("0x 1F", "31@-"),
      ("0 b101", "5@-"),
      ("0b1 01", "5@-"),
      ("010+1", "9.0@-"),
      ("-010+0", "-8.0@-"),
      ("(010)", "8@-"),
      ("0777+0", "511.0@-"),
      ("010e1", "100.0@-"),
      ("010pt", "10.0@u"),
      ("-4 .5", "-4.5@-"),
      ("1 2", "12@-"),
      ("1. 5", "1.5@-"),
      ("1 .5e 2", "150.0@-"),
      ("1 e2", "100.0@-"),
      ("1 0pt", "10.0@u"),
      ("2 p t", "2.0@u"),
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

  /// A comparison yields an INTEGER string, but arithmetic that consumes a
  /// comparison yields a REAL — `int_result` must be scoped to the actual
  /// result, not left globally set by a nested comparison. Ground truth from
  /// real pgfmath (`\pgfmathparse` + `\pgfmathresult`, TeX Live): `2==2`→1,
  /// `(1<2)`→1 (parens preserve), but `2*(1<2)`→2.0 and `2+(1<2)`→3.0.
  /// Regression guard for the arXiv-fork c52d3cf6 comparison port, which set a
  /// stream-global flag so `2*(1<2)` wrongly truncated to "2".
  #[test]
  fn comparison_int_result_is_scoped_not_global() {
    latexml_core::state::set_state(latexml_core::state::State::new(
      latexml_core::state::StateOptions::default(),
    ));
    // Bare / parenthesised comparison → integer.
    assert_eq!(super::pgfmathparse_eval("2==2"), "1");
    assert_eq!(super::pgfmathparse_eval("1<2"), "1");
    assert_eq!(super::pgfmathparse_eval("(1<2)"), "1");
    // Arithmetic over a comparison → real (the leak this guards against).
    assert_eq!(super::pgfmathparse_eval("2*(1<2)"), "2.0");
    assert_eq!(super::pgfmathparse_eval("2+(1<2)"), "3.0");
    // A comparison that evaluates false, then multiplied, is still a real.
    assert_eq!(super::pgfmathparse_eval("5*(3<2)"), "0.0");
    // Plain arithmetic is unaffected (no comparison anywhere).
    assert_eq!(super::pgfmathparse_eval("1+1"), "2.0");
  }

  /// pgfmath precedence (pgfmathparser.code.tex:898-916): `?:` is loosest, then `&&`/`||`,
  /// then the comparisons, then `+`/`-`. The ternary yields its chosen branch as pgf leaves it:
  /// an integer literal keeps its form (`7`, also parenthesized), arithmetic and real
  /// functions give a real (`5.0`, `-1.0`, `2.0`), a comparison or `int` an integer. Ground
  /// truth: lualatex/pgf TL2025 `\pgfmathparse` probes (2026-10-01). wheelchart's
  /// `arc data dir={\WCmidangle<180?1:-1}` (wheelchart.tex:714) gave 0 and walked a
  /// zero-length arc forever.
  #[test]
  fn ternary_and_logic_follow_pgf_precedence() {
    latexml_core::state::set_state(latexml_core::state::State::new(
      latexml_core::state::StateOptions::default(),
    ));
    for (expr, pgf) in [
      ("1>0?7:5", "7"),
      ("1<0?7:5", "5"),
      ("1>0?7.5:5", "7.5"),
      ("1>0?2+3:5", "5.0"),
      ("230.5<180?1:-1", "-1.0"),
      ("1<2&&3<2?1:0", "0"),
      ("1<2||3<2?1:0", "1"),
      ("1<2==1", "1"),
      ("2+(1<2)", "3.0"),
      ("1>0?(0>1?3:4):5", "4"),
      ("0>1?3:0>1?4:5", "5"),
      ("1+1>1?10:20", "10"),
      ("3<2+2", "1"),
      ("!(1<2)", "0"),
      ("(7)", "7"),
      ("7*1", "7.0"),
      ("-7", "-7.0"),
      ("3^1", "3.0"),
      ("max(1,2)", "2.0"),
      ("sin(0)", "0.0"),
      ("0.5*2", "1.0"),
      ("1?7:5", "7"),
      ("0?7:5", "5"),
      ("1>0?-1:1", "-1.0"),
      ("1>0?max(1,2):0", "2.0"),
      ("int(3.7)", "3"),
      ("1>0?int(3.7):0", "3"),
      ("abs(-3)", "3.0"),
      ("1>0?7", "7"),
      ("(1<2)", "1"),
      ("1<2?1>0:5", "1"),
      // `?` 100 / `:` 101: chained ternaries group to the left, `(a?b:c)?d:e`.
      ("1?2:3?4:5", "4"),
      ("1?0:3?4:5", "5"),
      ("1<2?3:4?5:6", "5"),
      ("-5<0?-1:-5>0?1:0", "1"),
      // `ifthenelse` and `array` return the chosen argument as it is.
      ("ifthenelse(3>2,1,0)", "1"),
      ("ifthenelse(2>1,1<2,9)", "1"),
      ("ifthenelse(1,(7),9)", "7"),
      ("ifthenelse(1<2,7,9)", "7"),
      ("ifthenelse(1,2*3,9)", "6.0"),
      ("ifthenelse(1,-7,9)", "-7.0"),
      ("array({2*3,8},0)", "6.0"),
      ("array({-7,8},0)", "-7.0"),
      ("array({7,8},1+0)", "8"),
      // Hex and binary literals are integers.
      ("0x1F", "31"),
      ("(0x1F)", "31"),
      ("0b101", "5"),
    ] {
      assert_eq!(
        super::pgfmathparse_eval(expr),
        pgf,
        "\\pgfmathparse{{{expr}}}"
      );
    }
  }

  /// Comparison/logic FUNCTION forms return bare integers, matching live
  /// pgfmath probes (pdflatex/pgf TL2025, 2026-08-31). The `1.0`-vs-`1`
  /// mismatch broke `\ifnum\pgfmathresult=1` in pgf-spectra's spectral
  /// loop — ~7k `expected:<relationaltoken>` errors per manual (witnesses
  /// pgf-spectra/pgf-spectraManual, *PreviewDataLSE, *PreviewDataNIST).
  #[test]
  fn min_max_fold_over_every_argument() {
    // pgfmathfunctions.misc.code.tex:292-336: `min`/`max` are variadic.
    assert_eq!(super::pgfmath_apply_fn("min", &[55.0, 74.0, 35.0]), 35.0);
    assert_eq!(super::pgfmath_apply_fn("max", &[10.0, 20.0, 30.0]), 30.0);
    // A lone comma-less argument is scanned token by token: `min(2,0)`.
    assert_eq!(super::pgfmath_apply_fn("min", &[20.0]), 0.0);
    assert_eq!(super::pgfmath_apply_fn("max", &[20.0]), 2.0);
    assert_eq!(
      super::pgfmath_apply_fn("max", &[1.0, 2.0, 3.0, 4.0, 5.0]),
      5.0
    );
    assert_eq!(super::pgfmath_apply_fn("min", &[55.0, 35.0]), 35.0);
  }

  #[test]
  fn boolean_function_forms_return_bare_integers() {
    latexml_core::state::set_state(latexml_core::state::State::new(
      latexml_core::state::StateOptions::default(),
    ));
    assert_eq!(super::pgfmathparse_eval("notless(5,3)"), "1");
    assert_eq!(super::pgfmathparse_eval("notless(1,3)"), "0");
    assert_eq!(super::pgfmathparse_eval("notgreater(5,3)"), "0");
    assert_eq!(super::pgfmathparse_eval("equal(2,2)"), "1");
    assert_eq!(super::pgfmathparse_eval("greater(5,3)"), "1");
    assert_eq!(super::pgfmathparse_eval("less(5,3)"), "0");
    assert_eq!(super::pgfmathparse_eval("and(1,1)"), "1");
    assert_eq!(super::pgfmathparse_eval("or(0,0)"), "0");
    assert_eq!(super::pgfmathparse_eval("not(1)"), "0");
    assert_eq!(super::pgfmathparse_eval("int(5/2)"), "2");
    // Arithmetic over a boolean function demotes to real, same as infix.
    assert_eq!(super::pgfmathparse_eval("2*equal(1,1)"), "2.0");
    // Arithmetic keeps its fixed-point .0 (probed: 2*3 → 6.0).
    assert_eq!(super::pgfmathparse_eval("2*3"), "6.0");
  }
}
