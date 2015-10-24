use core::mouth::{Mouth};

pub struct Gullet {
  pub mouth : Mouth
}

impl Default for Gullet {
  fn default() -> Self {
    Gullet {
      mouth : Mouth::default()
    }
  }
}

impl Gullet {
  pub fn get_mouth(&self) -> &Mouth {
    &self.mouth
  }

  pub fn flush(&self) {
    // TODO
  }
}