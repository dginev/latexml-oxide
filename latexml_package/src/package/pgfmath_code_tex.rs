//! pgfmath.code.tex — PGF math operations accelerator
//! Perl: pgfmath.code.tex.ltxml (737 lines)
//!
//! Replaces TeX-level pgfmath operations with native Rust implementations
//! for better precision and performance. Without this, pgf math runs in
//! TeX's fixed-point arithmetic which has different rounding behavior.
use crate::prelude::*;

const PI: f64 = std::f64::consts::PI;
const LOG10: f64 = std::f64::consts::LN_10;
const EPSILON: f64 = 0.00001;
const MAX_PGF_NUMBER: f64 = 16383.99998;

/// Format a pgfmath result: 5 decimal places, strip trailing zeros
/// Perl: sub pgfmathresult { sprintf("%.5f", $value); $value =~ s/0+$//; }
fn pgfmath_result_str(value: f64) -> String {
  if value.is_nan() || value.is_infinite() {
    return "0.0".to_string();
  }
  let clamped = value.max(-MAX_PGF_NUMBER).min(MAX_PGF_NUMBER);
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

/// Safe divisor — avoid division by zero
fn pgfmath_divisor(v: f64) -> f64 {
  if v == 0.0 { EPSILON } else { v }
}

/// Convert degrees to radians (pgf default is degrees)
fn pgfmath_arg_radians(arg: f64) -> f64 {
  arg.to_radians()
}

/// Factorial (matches Perl's memoized_pgf_factorial)
fn pgfmath_factorial(n: i64) -> f64 {
  static FACTS: [f64; 8] = [1.0, 1.0, 2.0, 6.0, 24.0, 120.0, 720.0, 5040.0];
  let n = n.unsigned_abs() as usize;
  if n >= FACTS.len() { FACTS[7] } else { FACTS[n] }
}

/// Parse a pgfNumber argument: expand, convert to f64
fn parse_pgf_number(arg: &Tokens) -> f64 {
  let s = arg.to_string();
  let s = s.trim();
  // Drop leading double negation
  let s = if s.starts_with("--") { &s[2..] } else { s };
  // "." is a valid number meaning 0
  if s == "." { return 0.0; }
  s.parse::<f64>().unwrap_or(0.0)
}

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

  // Perl L285-299: \pgfmath@smuggleone — smuggle a macro out of a group
  // Simplified: the raw TeX definition handles this, we don't need to override.
});
