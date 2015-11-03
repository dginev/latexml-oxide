use std::hash::Hash;
use std::collections::HashMap;

use common::model::{Model};
// use core::stomach::{Stomach};
use core::token::{Catcode, Token};
use core::definition::Definition;


pub struct State {
  pub verbosity : i32,
  pub map : Vec<String>,
  pub catcode : HashMap<char, Catcode>,
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
      catcode : HashMap::new()
    }
  }
}

impl State {// TODO for all
  pub fn new() -> Self {
    use core::token::Catcode::*;
    // TODO: Only standard catcodes for now.
    
    // Setup default catcodes.
    let mut std_catcodes : HashMap<char,Catcode> = HashMap::new();
    std_catcodes.insert('\\', ESCAPE);
    std_catcodes.insert('{', BEGIN);
    std_catcodes.insert('}', END);
    std_catcodes.insert('$', MATH);
    std_catcodes.insert('&', ALIGN);
    std_catcodes.insert('\r', EOL);
    std_catcodes.insert('#', PARAM);
    std_catcodes.insert('^', SUPER);
    std_catcodes.insert('_', SUB);
    std_catcodes.insert(' ', SPACE);
    std_catcodes.insert('\t', SPACE);
    std_catcodes.insert('%', COMMENT);
    std_catcodes.insert('~', ACTIVE);
    std_catcodes.insert('\0', IGNORE);
    for c in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
      std_catcodes.insert(c, LETTER);
    }

    State {
      catcode : std_catcodes,
      ..State::default()   
    }
  }
  // $$self{value}{SPECIALS} = [['^', '_', '@', '~', '&', '$', '#', '%', "'"]];
  // if ($options{catcodes} eq 'style') {
  //   $$self{catcode}{'@'} = [LETTER]; }
  // }

  pub fn lookup_catcode<'lc>(&'lc mut self, c: &'lc char) -> Option<Catcode> {
    match self.catcode.get(c) {
      None => None,
      Some(&c) => Some(c.clone())
    }
  }
  pub fn lookup_value<'lv, T: Hash>(&'lv mut self, key: &'lv str) -> Option<Box<T>> {
    None
  }
  pub fn lookup_definition<'def>(&'def mut self, key: &'def Token) -> Option<Box<Definition>> {
    None
  }
  pub fn lookup_digestable_definition<'def>(&'def mut self, key: &'def Token) -> Option<Box<Definition>> {
    None 
  }
  pub fn assign_value<'av, T: Hash>(&'av mut self, key: &'av str, value: Box<T>) {}
  pub fn assign_catcode<'ac>(&'ac mut self, c: &'ac char, cc : Catcode) {}
  pub fn clear_prefixes<'ac>(&'ac mut self) {}
}