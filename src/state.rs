use std::hash::Hash;
use common::model::{Model};
use core::stomach::{Stomach};
use core::token::{Catcode};

pub struct State<'state> {
  pub verbosity : i32,
  pub map : Vec<String>,
  pub status_code : usize,
  pub stomach : Stomach<'state>,
  pub model : Model
}

impl<'state> Default for State<'state> {
  fn default() -> Self {
    State {
      stomach : Stomach::default(),
      verbosity : 0,
      status_code: 0,
      model : Model::default(),
      map : Vec::new(),
    }
  }
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