use std::hash::Hash;
use common::model::{Model};
// use core::stomach::{Stomach};
use core::token::{Catcode, Token};
use core::definition::Definition;

pub struct State {
  pub verbosity : i32,
  pub map : Vec<String>,
  pub status_code : usize,
  pub model : Model
}

impl Default for State {
  fn default() -> Self {
    State {
      // stomach : Stomach::default(),
      verbosity : 0,
      status_code: 0,
      model : Model::default(),
      map : Vec::new(),
    }
  }
}

impl State {// TODO for all
  pub fn lookup_catcode<'lc>(&'lc mut self, c: &'lc char) -> Option<Catcode> {
    Some(Catcode::LETTER)
  }
  pub fn lookup_value<'lv, T: Hash>(&'lv mut self, key: &'lv str) -> Option<Box<T>> {
    None
  }
  pub fn lookup_definition<'def>(&'def mut self, key: &'def Token) -> Option<Box<Definition>> {
    None
  }
  pub fn assign_value<'av, T: Hash>(&'av mut self, key: &'av str, value: Box<T>) {}
  pub fn assign_catcode<'ac>(&'ac mut self, c: &'ac char, cc : Catcode) {}
}