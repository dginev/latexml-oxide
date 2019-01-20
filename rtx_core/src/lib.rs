#![allow(dead_code, unused_variables, unused_mut, unused_macros)]

use log::error;

#[macro_use]
pub mod aux_macros;
#[macro_use]
pub mod token;
#[macro_use]
pub mod common;
#[macro_use]
pub mod tokens;
#[macro_use]
pub mod definition;
pub mod document;
pub mod gullet;
pub mod keyval;
pub mod keyvals;
pub mod list;
pub mod mouth;
pub mod parameter;
pub mod state;
pub mod stomach;
pub mod tbox;
pub mod util;
pub mod whatsit;

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::model::Model;
use crate::common::number::Number;
use crate::common::store::Stored;
use crate::definition::register::{NumericOps, RegisterValue};
use crate::document::Document;
use crate::keyvals::KeyVals;
use crate::list::List;
use crate::state::{State, StateOptions};
use crate::stomach::Stomach;
use crate::tbox::Tbox;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;

pub struct Core {
  pub state: State,
  pub stomach: Rc<RefCell<Stomach>>,
  pub preload: Vec<String>,
}
pub struct CoreOptions {
  // First, state-related options:
  pub model: Option<Model>,
  pub verbosity: Option<i32>,
  pub strict: Option<bool>,
  pub include_comments: Option<bool>,
  pub include_styles: Option<bool>,
  pub nomathparse: Option<bool>,
  pub documentid: Option<String>,
  pub search_paths: Option<Vec<String>>,
  pub graphics_paths: Option<Vec<String>>,
  pub input_encoding: Option<String>,
  // The core-related
  pub preload: Option<Vec<String>>,
}
impl Default for CoreOptions {
  fn default() -> Self {
    CoreOptions {
      model: None,
      verbosity: None,
      strict: None,
      include_comments: None,
      include_styles: None,
      nomathparse: None,
      documentid: None,
      search_paths: None,
      graphics_paths: None,
      input_encoding: None,
      preload: None,
    }
  }
}

impl Default for Core {
  fn default() -> Self {
    let stomach = Rc::new(RefCell::new(Stomach::default()));
    let mut state = State::new(StateOptions::default());
    state.stomach = stomach.clone();
    Core {
      preload: Vec::new(),
      stomach,
      state,
    }
  }
}
impl Core {
  pub fn new(options: CoreOptions) -> Self {
    let preload = match options.preload {
      None => Vec::new(),
      Some(p) => p,
    };

    // pass on the state options, defaults are handled in State::new
    let state_options = StateOptions {
      model: options.model,
      verbosity: options.verbosity,
      strict: options.strict,
      include_comments: options.include_comments,
      documentid: options.documentid,
      search_paths: options.search_paths,
      graphics_paths: options.graphics_paths,
      include_styles: options.include_styles,
      input_encoding: options.input_encoding,
      nomathparse: options.nomathparse,
      ..StateOptions::default()
    };
    let stomach = Rc::new(RefCell::new(Stomach::default()));
    let mut state = State::new(state_options);
    state.stomach = stomach.clone();

    Core { state, preload, stomach }
  }

  pub fn get_state(&self) -> &State { &self.state }
  pub fn get_state_mut(&mut self) -> &mut State { &mut self.state }
}

pub trait BoxOps {
  fn unlist(&self) -> Vec<Digested>;
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()>;
  fn to_string(&self) -> String;
  fn stringify(&self) -> String { s!("Vec<Tbox> for now ") }
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored>;
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    let mut props = self.get_properties_mut();
    props.insert(key.to_string(), value.into());
  }
  fn get_property(&self, _key: &str, _state: &mut State) -> Option<Cow<Stored>> {
    error!(target: "boxops:get_property", "Generic BoxOps::get_property should never be called!");
    None
  }
  fn get_body(&self) -> Option<Digested> {
    error!(target: "boxops:get_body", "Generic BoxOps::get_body should never be called!");
    None
  }
  fn get_font(&self) -> Option<Cow<Font>>;
  fn get_locator(&self) -> Option<Locator>;
  fn revert(&self) -> Result<Tokens>;

  fn set_width<T: Into<Stored>>(&mut self, width: T) {
    let mut props = self.get_properties_mut();
    props.insert("width".to_string(), width.into());
  }
  fn get_width(&self, state: &mut State) -> Option<RegisterValue> {
    match self.get_property("width", state) {
      None => Some(Number::new(0.0).into()),
      Some(val) => (&(*val)).into(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TexMode {
  Math,
  Text,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Digested {
  TBox(Rc<Tbox>),
  Whatsit(Rc<RefCell<Whatsit>>),
  List(Rc<List>),
  Postponed(Rc<Tokens>),
  KeyVals(Rc<KeyVals>),
}

impl<'a> From<&'a String> for Digested {
  fn from(value: &'a String) -> Digested {
    Digested::TBox(Rc::new(Tbox {
      text: value.to_string(),
      ..Tbox::default()
    }))
  }
}

impl From<Tbox> for Digested {
  fn from(value: Tbox) -> Digested { Digested::TBox(Rc::new(value)) }
}

impl<'a> From<&'a Digested> for Option<crate::Digested> {
  fn from(value: &'a Digested) -> Option<crate::Digested> { Some(value.clone()) }
}
impl<'a> From<&'a Digested> for Tokens {
  fn from(value: &'a Digested) -> Tokens { value.revert().unwrap() }
}
impl<'a> From<Digested> for Tokens {
  fn from(value: Digested) -> Tokens { value.revert().unwrap() }
}

impl Default for Digested {
  fn default() -> Self { Digested::TBox(Rc::new(Tbox::default())) }
}

impl BoxOps for Digested {
  fn unlist(&self) -> Vec<Digested> {
    match self {
      Digested::TBox(ref b) => b.unlist(),
      Digested::List(ref l) => l.unlist(),
      Digested::Whatsit(ref w) => w.borrow().unlist(),
      Digested::KeyVals(ref kvs) => kvs.unlist(),
      Digested::Postponed(ref _t) => unimplemented!(),
    }
  }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    match self {
      Digested::TBox(b) => b.be_absorbed(document, state),
      Digested::List(l) => l.be_absorbed(document, state),
      Digested::Whatsit(w) => w.borrow().be_absorbed(document, state),
      Digested::KeyVals(kvs) => kvs.be_absorbed(document, state),
      Digested::Postponed(_) => unimplemented!(),
    }
  }

  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> {
    unimplemented!()
    // match self {
    //   Digested::TBox(ref mut b) => b.get_properties_mut(),
    //   Digested::List(ref mut l) => l.get_properties_mut(),
    //   Digested::Whatsit(ref mut w) => unimplemented!(), //w.borrow_mut().get_properties_mut(),
    //   Digested::KeyVals(ref mut kvs) => kvs.get_properties_mut(),
    //   Digested::Postponed(_) => unimplemented!(),
    // }
  }

  fn to_string(&self) -> String {
    match *self {
      Digested::TBox(ref b) => b.to_string(),
      Digested::List(ref l) => l.to_string(),
      Digested::Whatsit(ref w) => w.borrow().to_string(),
      Digested::Postponed(ref t) => t.to_string(),
      Digested::KeyVals(ref kvs) => kvs.to_string(),
    }
  }

  fn stringify(&self) -> String {
    match *self {
      Digested::TBox(ref b) => b.stringify(),
      Digested::List(ref l) => l.stringify(),
      Digested::Whatsit(ref w) => w.borrow().stringify(),
      Digested::Postponed(ref t) => (*t).stringify(),
      Digested::KeyVals(ref kvs) => kvs.stringify(),
    }
  }

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    match *self {
      Digested::TBox(ref b) => error!(target: "digested:set_property", "Called set_property on Box: {:?}", b),
      Digested::List(ref l) => error!(target: "digested:set_property", "Called set_property on List: {:?}", l),
      Digested::Whatsit(ref w) => w.borrow_mut().set_property(key, value), // TODO
      _ => unimplemented!(),
    }
  }

  fn get_property(&self, key: &str, state: &mut State) -> Option<Cow<Stored>> {
    match *self {
      Digested::TBox(ref b) => b.get_property(key, state),
      Digested::List(ref l) => {
        error!(target: "digested:get_property", "Called get_property on List: {:?}", l);
        None
      },
      Digested::Whatsit(ref w) => match w.borrow().get_property(key, state) {
        None => None,
        Some(v) => Some(Cow::Owned(v.into_owned())),
      },
      _ => unimplemented!(),
    }
  }
  fn get_body(&self) -> Option<Digested> {
    match *self {
      Digested::TBox(ref b) => {
        error!(target: "digested:get_body", "Called get_body on Box: {:?}", b);
        None
      },
      Digested::List(ref l) => {
        error!(target: "digested:get_body", "Called get_body on List: {:?}", l);
        None
      },
      Digested::Whatsit(ref w) => w.borrow().get_body(),
      _ => unimplemented!(),
    }
  }

  fn get_font(&self) -> Option<Cow<Font>> {
    match *self {
      Digested::TBox(ref b) => b.get_font(),
      Digested::List(ref l) => l.get_font(),
      Digested::Whatsit(ref w) => match w.borrow().get_font() {
        None => None,
        Some(t) => Some(Cow::Owned(t.into_owned())),
      },
      _ => unimplemented!(),
    }
  }

  fn get_locator(&self) -> Option<Locator> {
    match *self {
      Digested::TBox(ref b) => b.get_locator(),
      Digested::List(ref l) => l.get_locator(),
      Digested::Whatsit(ref w) => w.borrow().get_locator(),
      _ => unimplemented!(),
    }
  }

  fn revert(&self) -> Result<Tokens> {
    match *self {
      Digested::TBox(ref b) => b.revert(),
      Digested::List(ref l) => l.revert(),
      Digested::Whatsit(ref w) => w.borrow().revert(),
      Digested::Postponed(ref t) => Ok((**t).clone()),
      Digested::KeyVals(ref kvs) => kvs.revert(),
    }
  }
}
