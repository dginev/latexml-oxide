use std::hash::Hash;
use std::collections::HashMap;

use common::model::{Model};
use common::object::Object;
// use core::stomach::{Stomach};
use core::token::{Catcode, Token};
use core::definition::{Definition, Expandable};

pub enum Scope {
  Global,
  Local
}

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
  pub fn lookup_value<'lv, T: Hash>(&'lv mut self, key: &'lv str) -> Option<Box<T>>{
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
  pub fn assign_definition<'def, T: Definition + Hash>(&'def mut self, key: &'def Token, definition : Box<T>) { }
  pub fn assign_internal<'ai>(&'ai mut self, table : &'ai str, key : &'ai str, definition : Expandable, scope : &'ai Option<Scope>) {}
  pub fn clear_prefixes<'cp>(&'cp mut self) {}

  /// And a shorthand for installing definitions
  pub fn install_definition<'id>(&'id mut self, definition: Expandable, scope: &'id Option<Scope>) {
    // Locked definitions!!! (or should this test be in assignMeaning?)
    // Ignore attempts to (re)define $cs from tex sources
    //  my $cs = $definition->getCS->getCSName;
    let token = definition.get_cs();
    let cs = token.get_cs_name();
    let cs_locked = cs.clone() + ":locked";
    // TODO, .is_none() should be a real false check
    let is_cs_locked : Option<Box<bool>> = self.lookup_value(&cs_locked);
    let is_state_unlocked : Option<Box<bool>> = self.lookup_value("UNLOCKED");
    if is_cs_locked.is_some() && is_state_unlocked.is_none() {
      match self.lookup_value("SOURCEFILE") {
        Some(s) => {
          let tex_or_bib_ext_regex = regex!(r"/\.(tex|bib)$/");
          let code_tex_ext_regex = regex!(r"/\.code\.tex$/");
          // report if the redefinition seems to come from document source
          if ((*s == "Anonymous String") || tex_or_bib_ext_regex.is_match(*s)) && (! code_tex_ext_regex.is_match(*s)) {
            // TODO:
            //  info("ignore", cs, self.get_stomach(), "Ignoring redefinition of $cs");
          }
          return;
        },
        None => {}
      };
    }
    self.assign_internal("meaning", &cs, definition, scope);
    return;
  }
}