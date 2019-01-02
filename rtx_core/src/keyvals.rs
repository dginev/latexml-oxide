use std::borrow::Cow;
use std::collections::HashMap;
// use std::fmt;
// use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::store::Stored;
// use crate::definition::expandable::Expandable;
// use crate::definition::Definition;
use crate::document::Document;
// use crate::list::List;
use crate::state::State;
use crate::token::Token;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested};

#[derive(Debug, Clone)]
pub struct KeyVals {
  // which KeyVals are we parsing and how do we behave?
  prefix: String,
  keysets: Vec<String>,
  skip: Vec<String>,
  set_all: bool,
  set_internals: bool,
  skip_missing: bool,
  hook_missing: Option<Token>,
  // all the internal representations
  tuples: Vec<(String, String)>,
  cached_pairs: Vec<(String, String)>,
  cached_hash: HashMap<String, Vec<String>>,
  // all the character tokens we used
  punct: Vec<char>,
  assign: Vec<char>,
}

impl Default for KeyVals {
  fn default() -> Self {
    KeyVals {
      prefix: "KV".to_string(),
      keysets: vec!["_anonymous_".to_string()],
      skip: Vec::new(),
      set_all: false,
      set_internals: false,
      skip_missing: false,
      hook_missing: None,
      tuples: Vec::new(),
      cached_pairs: Vec::new(),
      cached_hash: HashMap::new(),
      punct: Vec::new(),
      assign: Vec::new(),
    }
  }
}

impl PartialEq for KeyVals {
  fn eq(&self, _other: &KeyVals) -> bool {
    false // TODO ?
  }
}

impl KeyVals {
  //======================================================================
  // Public accessors of all the values
  //======================================================================
  // Note: The API of this need to be stable, as people may be using it

  /// return the value of a given key. If multiple values are given, return the last one.
  pub fn get_value(&self, key: &str) -> Option<&String> {
    // Since we (by default) accumulate lists of values when repeated,
    // we need to provide the "common" thing: return the last value given.
    match self.cached_hash.get(key) {
      None => None,
      Some(value) => value.last(),
    }
  }
}

impl BoxOps for KeyVals {
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { unimplemented!() }
  fn to_string(&self) -> String { String::new() } // TODO
  fn unlist(&self) -> Vec<Digested> { Vec::new() } // TODO
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> { Ok(()) } // TODO
  fn revert(&self) -> Tokens { Tokens::new(Vec::new()) } // TODO
  fn get_locator(&self) -> Option<Locator> {
    // TODO
    None
  }
  fn get_font(&self) -> Option<Cow<Font>> { None } // TODO
}
