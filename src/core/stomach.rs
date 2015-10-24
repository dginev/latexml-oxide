use core::{Digested};
use core::gullet::{Gullet};

pub struct Stomach {
  pub gullet : Gullet
}

impl Default for Stomach {
  fn default() -> Self {
    Stomach {
      gullet : Gullet::default()
    }
  }
}

impl Stomach {
  pub fn get_gullet(&self) -> &Gullet {
    &self.gullet
  }

  pub fn digest_next_body(&self) -> String {
    // TODO
    "a body?".to_string()
  }
}