use std::hash::Hash;
use core::stomach::{Stomach};
use core::token::{Catcode};

pub struct State<'state> {
  pub verbosity : i32,
  pub map : Vec<String>,
  pub status_code : usize,
  pub stomach : Stomach<'state>
}

impl<'state> State<'state> {
  pub fn get_stomach(&mut self) -> &'state mut Stomach {
    &mut self.stomach
  }
  pub fn lookup_catcode(&self, c: &char) -> Option<Catcode> {
    Some(Catcode::LETTER)
  }
  pub fn lookup_value<T: Hash>(&self, key: &str) -> Option<Box<T>> {
    None
  }
  pub fn assign_value<T: Hash>(&self, key: &str, value: Box<T>) {}
  pub fn assign_catcode(&self, c: &char, cc : Catcode) {}
}