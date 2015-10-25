use core::gullet::{Gullet};

pub struct Stomach<'stomach> {
  pub gullet : Gullet<'stomach>
}

impl<'stomach> Default for Stomach<'stomach> {
  fn default() -> Self {
    Stomach {
      gullet : Gullet::default()
    }
  }
}

impl<'stomach> Stomach<'stomach> {
  pub fn get_gullet(&self) -> &Gullet {
    &self.gullet
  }

  pub fn digest_next_body(&self) -> String {
    // TODO
    "a body?".to_string()
  }
}