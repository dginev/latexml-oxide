#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue {
  number: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue {
  number: f32,
}

impl Glue {
  pub fn value_of(self) -> f32 { self.number }
  pub fn new(number: f32) -> Self { Glue { number } }
}

impl MuGlue {
  pub fn value_of(self) -> f32 { self.number }
}
