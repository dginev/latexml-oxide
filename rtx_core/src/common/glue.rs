use crate::definition::register::NumericOps;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue {
  number: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue {
  number: f32,
}

impl NumericOps for Glue {
  fn value_of(self) -> f32 { self.number }
  fn new<T: Into<f32>>(number: T) -> Self { Glue { number: number.into() } }
}

impl NumericOps for MuGlue {
  fn new<T: Into<f32>>(number: T) -> Self { MuGlue { number: number.into() } }
  fn value_of(self) -> f32 { self.number }
}
