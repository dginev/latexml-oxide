use crate::definition::register::NumericOps;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension {
  number: f32,
}

impl NumericOps for Dimension {
  fn value_of(self) -> f32 { self.number }
  fn new<T: Into<f32>>(number: T) -> Self { Dimension { number: number.into() } }
}
