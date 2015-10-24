pub struct Mouth {
  pub todo : i32
}

impl Default for Mouth {
  fn default() -> Self {
    Mouth {
      todo : 1
    }
  }
}

impl Mouth {
  pub fn has_more_input(&self) -> bool {
    false
  }
}