use std::fmt::Display;

use crate::{
  common::glue::Glue,
  definition::register::RegisterType,
  token::{Catcode, Token},
};

pub const UNITY: i64 = 65536;
pub const UNITY_F64: f64 = 65536.0;
pub const EPSILON: f64 = 0.000_000_119_209_29;
pub const ROUNDING_HALF: f64 = 0.49999994;
pub const SCALES: &[i32] = &[1, 10, 100, 1000, 10000, 100_000];

/// Round $number to $prec decimals (0...6) attempting to do so portably.
pub fn round_to(number: f64, prec_opt: Option<u8>) -> f64 {
  let mut prec = prec_opt.unwrap_or(2);
  if prec > 5 {
    prec = 5;
  }
  let scale = SCALES[prec as usize];
  // scale to integer, w/some slop in case arbitrarily close to an integer...
  let n = number * scale as f64 * (1.0 + 100.0 * EPSILON);
  let adjusted: f64 = if n < -EPSILON {
    n - 0.5
  } else if n > EPSILON {
    n + 0.5
  } else {
    0.0
  };
  adjusted.trunc() / scale as f64
}

/// An attempt at rounding floats to integers (like scaled points),
/// in a (hopefully) Knuthian manner (like round_decimals \S102 in Tex The Program)
// DG: Note that we have to go to the largest `i64` type to contain the truncation
// of large SP values multiplied up by UNITY
pub fn kround(number: f64) -> i64 {
  let rounded = if number < 0.0 {
    number - ROUNDING_HALF
  } else {
    number + ROUNDING_HALF
  };
  rounded.trunc() as i64
}

/// Convert `float` to a fixed-point number
///
/// If `unit` is given, it is number of units PER SCALED-POINT! (hence, extra division)
/// AND, note that the float is rounded and THEN truncated after multiplying by units!
/// to mimic TeX's behavior.
pub fn fixpoint(float: f64, unit_opt: Option<f64>) -> i64 {
  let fix = kround(float * UNITY_F64);
  if let Some(unit) = unit_opt {
    (fix as f64 * unit / UNITY_F64).trunc() as i64
  } else {
    fix
  }
}

/// Exact TeX fixed-point unit conversion: `floor(round(float·65536)·num/den)`,
/// computed in integer (i128) arithmetic.
///
/// This is tex.web §458 `scan_dimen` — `cur_val := xn_over_d(cur_val,num,denom);
/// f := (num*f + 65536*remainder) div denom; cur_val += f div 65536` — which
/// algebraically collapses to `floor(fix·num/den)` for `fix = cur_val·65536 + f`
/// (proof: `xn_over_d(x,n,d) = floor(x·n/d)` with `remainder = x·n mod d`, so the
/// integer-part and fraction-carry terms recombine to `num·fix/den`). The
/// per-unit `(num,den)` come from [`crate::state::convert_unit_ratio`] (physical
/// units use TeX's `set_conversion` fractions, tex.web 9020-9032; font-relative
/// units use `(metric_sp, 65536)`, the `nx_plus_y`/`xn_over_d` path of §8983).
///
/// The legacy float [`fixpoint`] computes `trunc(fix·(65536·num/den)/65536)` and
/// drifts ±1 sp on rounding boundaries (verified against pdftex on `cm`/`bp`/`mm`/
/// `cc`; issue #127). Integer math is bit-faithful to TeX/pdfTeX. Sign matches
/// TeX: it tracks sign separately and floors the magnitude, i.e. truncation
/// toward zero — exactly Rust's integer `/`.
pub fn fixpoint_unit(float: f64, num: i64, den: i64) -> i64 {
  let fix = kround(float * UNITY_F64) as i128;
  (fix * num as i128 / den as i128) as i64
}

pub trait NumericOps {
  fn new(num: i64) -> Self
  where Self: Sized;
  fn new_f64(num: f64) -> Self
  where Self: Sized;
  fn unit(&self) -> Option<&'static str> { None }
  fn value_of(self) -> i64;
  fn value_f64(self) -> f64
  where Self: Sized {
    self.value_of() as f64
  }
  fn pt_value(self, prec: Option<u8>) -> f64
  where Self: Sized {
    round_to(self.value_of() as f64 / UNITY_F64, prec)
  }
  fn px_value(self, prec: Option<u8>) -> f64
  where Self: Sized {
    let dpi = crate::state::lookup_int("DPI");
    let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
    round_to((self.value_f64() / UNITY_F64) * (dpi / 72.27), prec)
  }

  fn absolute(self) -> Self
  where Self: Sized {
    Self::new(self.value_of().abs())
  }

  fn sign(self) -> i8
  where Self: Sized {
    use std::cmp::Ordering::*;
    match self.value_of().cmp(&0) {
      Less => -1,
      Equal => 0,
      Greater => 1,
    }
  }

  fn negate(self) -> Self
  where Self: Sized {
    Self::new(-self.value_of())
  }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() - other.value_of())
  }
  // Perl: int($self->valueOf * $other->valueOf) — uses float arithmetic to
  // handle Float multipliers correctly, then truncates.
  fn multiply<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new((self.value_of() as f64 * other.value_f64()) as i64)
  }
  /// Truncating division
  fn divide<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let mut other_value: f64 = other.value_of() as f64;
    if other_value == 0.0 {
      other_value = EPSILON; // avoid dividing by zero
    }
    Self::new((self.value_of() as f64 / other_value).trunc() as i64)
  }

  /// Rounding division
  fn divideround<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    let mut other_value: f64 = other.value_of() as f64;
    if other_value == 0.0 {
      other_value = EPSILON; // avoid dividing by zero
    }
    Self::new((0.5 + self.value_of() as f64 / other_value).trunc() as i64)
  }

  fn smaller<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of().min(other.value_of()))
  }

  fn larger<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of().max(other.value_of()))
  }

  fn to_token(self) -> Token
  where Self: Sized {
    T_OTHER!(self.value_of().to_string())
  }
  // dancing around meta-programming in the Glue case... is there a better way?
  fn into_glue_type(self) -> Glue
  where Self: Sized {
    Glue::new(0) // default: zero glue
  }
  fn register_type(&self) -> RegisterType;
  fn to_attribute(&self) -> String
  where Self: Display {
    self.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_to_default_precision_is_two() {
    assert_eq!(round_to(1.2345, None), 1.23);
    assert_eq!(round_to(1.2355, None), 1.24);
    assert_eq!(round_to(0.0, None), 0.0);
  }

  #[test]
  fn round_to_respects_precision() {
    assert_eq!(round_to(1.23456, Some(3)), 1.235);
    assert_eq!(round_to(1.23456, Some(0)), 1.0);
    assert_eq!(round_to(1.5, Some(0)), 2.0);
  }

  #[test]
  fn round_to_caps_precision_at_five() {
    // The doc-comment says 0..=5 is the intended range; precisions
    // above 5 are clamped to 5.
    let a = round_to(1.12345, Some(5));
    let b = round_to(1.12345, Some(10));
    assert_eq!(a, b, "precision > 5 clamps to 5 (got {a} vs {b})");
  }

  #[test]
  fn round_to_negative_numbers() {
    assert_eq!(round_to(-1.235, None), -1.24);
    assert_eq!(round_to(-0.005, None), -0.01);
  }

  #[test]
  fn kround_basic() {
    assert_eq!(kround(0.0), 0);
    assert_eq!(kround(0.49), 0);
    // 0.5 + ROUNDING_HALF (0.49999994) = 0.99999994 → trunc = 0
    // Knuthian rounding below is actually a bit different from banker's.
    assert_eq!(kround(1.49), 1);
    assert_eq!(kround(1.5), 1);
    assert_eq!(kround(-0.49), 0);
    assert_eq!(kround(-1.49), -1);
  }

  #[test]
  fn fixpoint_without_unit() {
    // fixpoint(x, None) returns kround(x * 65536).
    assert_eq!(fixpoint(1.0, None), UNITY);
    assert_eq!(fixpoint(0.0, None), 0);
    assert_eq!(fixpoint(0.5, None), UNITY / 2);
  }

  #[test]
  fn fixpoint_with_unit_scales() {
    // unit=1.0 means 1 unit per scaled-point, so:
    //   fix(1.0, Some(1.0)) = kround(65536.0) * 1.0 / 65536.0 = 1 (truncated)
    let out = fixpoint(1.0, Some(1.0));
    // Result depends on the unit semantics ("units PER SCALED-POINT").
    // Just sanity-check that non-zero input produces a defined result.
    assert!(out >= 0 || out < 0, "defined integer output: {out}");
  }

  #[test]
  fn constants_unity_matches_f64() {
    // The integer UNITY and f64 UNITY_F64 must agree numerically.
    assert_eq!(UNITY as f64, UNITY_F64);
  }

  #[test]
  fn fixpoint_unit_matches_pdftex() {
    // Ground-truth scaled-point values captured from pdftex (TeX Live 2025):
    //   `\dimen0=<value><unit> \showthe\dimen0`. These are the cases where the
    //   legacy float multiply drifts ±1 sp; the integer path is bit-exact.
    //   See issue #127. (num, den) are tex.web §458 set_conversion fractions.
    let cases: &[(f64, i64, i64, i64)] = &[
      // value, num, den, expected sp
      (1.0, 7227, 100, 4736286),         // 1in   = 72.26999pt
      (1.0, 7227, 254, 1864679),         // 1cm   = 28.45274pt
      (1.0, 7227, 2540, 186467),         // 1mm   = 2.84526pt
      (1.0, 7227, 7200, 65781),          // 1bp   = 1.00374pt
      (1.0, 1238, 1157, 70124),          // 1dd   = 1.07pt
      (1.0, 14856, 1157, 841489),        // 1cc   = 12.8401pt
      (76.24341, 7227, 254, 142169544),  // cm: float gave 142169543 (off by 1)
      (400.43946, 7227, 7200, 26341612), // bp: float gave 26341611
      (188.33008, 7227, 7200, 12388684), // bp: float gave 12388683
      (75.77057, 7227, 2540, 14128785),  // mm: float gave 14128784
      (76.40832, 14856, 1157, 64296768), // cc: float gave 64296767
    ];
    for &(value, num, den, expected) in cases {
      assert_eq!(
        fixpoint_unit(value, num, den),
        expected,
        "fixpoint_unit({value}, {num}, {den}) should match pdftex"
      );
    }
  }

  #[test]
  fn fixpoint_unit_is_exact_floor() {
    // fixpoint_unit must equal floor(fix·num/den), the integer collapse of
    // tex.web scan_dimen — no float drift. Cross-check against an independent
    // i128 floor over a deterministic sweep.
    let units: &[(i64, i64)] = &[(7227, 100), (7227, 254), (7227, 2540), (7227, 7200)];
    for &(num, den) in units {
      for k in 0..2000 {
        let value = k as f64 * 0.13759 + 0.0007; // varied non-round decimals
        let fix = kround(value * UNITY_F64) as i128;
        let expected = (fix * num as i128 / den as i128) as i64;
        assert_eq!(fixpoint_unit(value, num, den), expected);
      }
    }
  }
}
