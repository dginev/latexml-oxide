#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Glue {
  number: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MuGlue {
  number: i32,
}

impl Glue {
  pub fn value_of(self) -> i32 { self.number }
}

impl MuGlue {
  pub fn value_of(self) -> i32 { self.number }
}
