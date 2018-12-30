#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension {
  number: f32,
}

impl Dimension {
  pub fn value_of(self) -> f32 { self.number }
}

impl Dimension {
  pub fn new<T: Into<f32>>(number: T) -> Self { Dimension { number: number.into() } }
  pub fn negate(self) -> Dimension {
    if self.number > 0.0 {
      Dimension::new(-self.number)
    } else {
      self
    }
  }
}
