#[derive(Debug, Clone, PartialEq)]
pub struct Number {
  number: usize,
}

impl Number {
  pub fn new(number: usize) -> Self { Number { number: number } }
  pub fn value_of(&self) -> usize { self.number }
}
