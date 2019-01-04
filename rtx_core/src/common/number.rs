use crate::definition::register::NumericOps;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Number {
  number: f32,
}

impl Default for Number {
  fn default() -> Self { Number::new(0.0) }
}

impl NumericOps for Number {
  fn new<T: Into<f32>>(number: T) -> Self { Number { number: number.into() } }
  fn value_of(self) -> f32 { self.number }
}

const SCALES: &'static [f32] = &[1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0];
// smallest number that makes a difference added to 1 in Rust's float format.
// my $EPSILON = 1.0;
// while (1.0 + $EPSILON / 2 != 1) {
//   $EPSILON /= 2.0; }
const EPSILON: f32 = 0.00000011920929;

impl Number {
  /// Round $number to $prec decimals (0...6) attempting to do so portably.
  pub fn round_to(number: f32, prec_opt: Option<usize>) -> f32 {
    let mut prec = prec_opt.unwrap_or(2);
    if prec > 5 {
      prec = 5;
    }
    let scale = SCALES.get(prec).unwrap();
    // scale to integer, w/some slop in case arbitrarily close to an integer...
    let n = number * scale * (1.0 + 100.0 * EPSILON);
    let adjusted: f32 = if n < -EPSILON {
      n - 0.5
    } else if n > EPSILON {
      n + 0.5
    } else {
      0.0
    };
    adjusted.floor() / scale
  }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {{
    use ::rtx_core::definition::register::NumericOps;
    ::rtx_core::common::number::Number::new($number as f32)
  }};
}
