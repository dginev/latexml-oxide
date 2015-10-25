use common::model::Model;
use core::Digested;

pub struct Document<'doc> {
  pub model : &'doc Model
}

impl<'doc> Document<'doc> {

  pub fn finalize(&self) -> String {
    "fake finalize".to_string()
  }

  pub fn absorb(&self, digested : Digested) -> String {
    "absorbed".to_string()
  }

  pub fn insert_pi(&self, which : &str, paths : Vec<String>) {
    // TODO
  }
}