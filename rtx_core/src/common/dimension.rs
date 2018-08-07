#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension {
  number: i32,
}

impl Dimension {
  pub fn value_of(&self) -> i32 { self.number }
}
