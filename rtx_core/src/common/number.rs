#[derive(Debug, Clone, PartialEq)]
pub struct Number {
  number: i32,
}

impl Number {
  pub fn new(number: i32) -> Self { Number { number: number } }
  pub fn value_of(&self) -> i32 { self.number }
  pub fn add(&self, other: Number) -> Self { Number::new(self.value_of() + other.value_of()) }
}

#[macro_export]
macro_rules! Number {
  ($number:expr) => {
    ::rtx_core::common::number::Number::new($number)
  };
}
