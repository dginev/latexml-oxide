use crate::definition::register::NumericOps;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue(pub f32);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue(pub f32);

impl NumericOps for Glue {
  fn value_of(self) -> f32 { self.0 }
  fn new<T: Into<f32>>(number: T) -> Self { Glue(number.into()) }
}

impl NumericOps for MuGlue {
  fn new<T: Into<f32>>(number: T) -> Self { MuGlue(number.into()) }
  fn value_of(self) -> f32 { self.0 }
}
