#![allow(dead_code, unused_variables, unused_mut, unused_macros, clippy::trivial_regex)]

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
use crate::common::object::Object;
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
impl Object for Core {}
#[derive(Default)]
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

impl Default for Core {
  fn default() -> Self {
    let stomach = Rc::new(RefCell::new(Stomach::default()));
    let mut state = State::new(StateOptions::default());
    state.stomach = Rc::clone(&stomach);
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
    state.stomach = Rc::clone(&stomach);

    Core { state, stomach, preload }
  }

  pub fn get_state(&self) -> &State { &self.state }
  pub fn get_state_mut(&mut self) -> &mut State { &mut self.state }
}

pub trait BoxOps: Object {
  fn unlist(&self) -> Vec<Digested>;
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()>;
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored>;
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    let mut props = self.get_properties_mut();
    props.insert(key.to_string(), value.into());
  }
  fn get_property(&self, _key: &str, state: &mut State) -> Option<Cow<Stored>> {
    Error!(
      "boxops",
      "get_property",
      self,
      state,
      "Generic BoxOps::get_property should never be called!"
    );
    None
  }
  fn get_body(&self) -> Option<Digested> {
    Error!("boxops", "get_body", self, None, "Generic BoxOps::get_body should never be called!");
    None
  }
  fn get_font(&self) -> Option<Cow<Font>>;

  fn set_width<T: Into<Stored>>(&mut self, width: T) {
    let mut props = self.get_properties_mut();
    props.insert("width".to_string(), width.into());
  }
  fn get_width(&self, state: &mut State) -> Option<RegisterValue> {
    // why is clippy intent (&*val).into() is needless?
    #[allow(clippy::needless_borrow)]
    match self.get_property("width", state) {
      None => Some(Number::new(0.0).into()),
      Some(val) => (&*val).into(),
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
  RegisterValue(Rc<RegisterValue>),
}

impl<'a> From<&'a String> for Digested {
  fn from(value: &'a String) -> Digested {
    Digested::TBox(Rc::new(Tbox {
      text: value.to_string(),
      ..Tbox::default()
    }))
  }
}
impl From<String> for Digested {
  fn from(value: String) -> Digested {
    Digested::TBox(Rc::new(Tbox {
      text: value,
      ..Tbox::default()
    }))
  }
}

impl From<Tbox> for Digested {
  fn from(value: Tbox) -> Digested { Digested::TBox(Rc::new(value)) }
}
impl From<List> for Digested {
  fn from(value: List) -> Digested { Digested::List(Rc::new(value)) }
}
impl From<Whatsit> for Digested {
  fn from(value: Whatsit) -> Digested { Digested::Whatsit(Rc::new(RefCell::new(value))) }
}
impl From<KeyVals> for Digested {
  fn from(value: KeyVals) -> Digested { Digested::KeyVals(Rc::new(value)) }
}
impl From<RegisterValue> for Digested {
  fn from(value: RegisterValue) -> Digested { Digested::RegisterValue(Rc::new(value)) }
}

impl<'a> From<&'a Digested> for Option<crate::Digested> {
  fn from(value: &'a Digested) -> Option<crate::Digested> { Some(value.clone()) }
}
// impl<'a> From<&'a Digested> for Tokens {
//   fn from(value: &'a Digested) -> Tokens { value.revert(state).unwrap() }
// }
// impl From<Digested> for Tokens {
//   fn from(value: Digested) -> Tokens { value.revert(state).unwrap() }
// }
impl From<Digested> for Result<Digested> {
  fn from(value: Digested) -> Result<Digested> { Ok(value) }
}
impl From<Digested> for Result<Vec<Digested>> {
  fn from(value: Digested) -> Result<Vec<Digested>> { Ok(vec![value]) }
}
impl From<Digested> for Result<Option<Digested>> {
  fn from(value: Digested) -> Result<Option<Digested>> { Ok(Some(value)) }
}

impl Default for Digested {
  fn default() -> Self { Digested::TBox(Rc::new(Tbox::default())) }
}

impl fmt::Display for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Digested::TBox(ref b) => write!(f, "{}", b),
      Digested::List(ref l) => write!(f, "{}", l),
      Digested::Whatsit(ref w) => write!(f, "{}", w.borrow()),
      Digested::Postponed(ref t) => write!(f, "{}", t),
      Digested::KeyVals(ref kvs) => write!(f, "{}", kvs),
      Digested::RegisterValue(ref rv) => write!(f, "{}", rv),
    }
  }
}
impl Object for Digested {
  fn stringify(&self) -> String {
    match *self {
      Digested::TBox(ref b) => b.stringify(),
      Digested::List(ref l) => l.stringify(),
      Digested::Whatsit(ref w) => w.borrow().stringify(),
      Digested::Postponed(ref t) => (*t).stringify(),
      Digested::KeyVals(ref kvs) => kvs.stringify(),
      Digested::RegisterValue(ref rv) => (*rv).stringify(),
    }
  }
  fn get_locator(&self) -> Cow<Locator> {
    match *self {
      Digested::TBox(ref b) => b.get_locator(),
      Digested::List(ref l) => l.get_locator(),
      Digested::Whatsit(ref w) => Cow::Owned(w.borrow().get_locator().into_owned()),
      _ => unimplemented!(),
    }
  }
  fn revert(&self, state:&mut State) -> Result<Tokens> {
    match *self {
      Digested::TBox(ref b) => b.revert(state),
      Digested::List(ref l) => l.revert(state),
      Digested::Whatsit(ref w) => w.borrow().revert(state),
      Digested::Postponed(ref t) => Ok((**t).clone()),
      Digested::KeyVals(ref kvs) => kvs.revert(state),
      Digested::RegisterValue(ref rv) => (**rv).revert(state),
    }
  }
}

impl BoxOps for Digested {
  fn unlist(&self) -> Vec<Digested> {
    match self {
      Digested::TBox(ref b) => b.unlist(),
      Digested::List(ref l) => l.unlist(),
      Digested::Whatsit(ref w) => w.borrow().unlist(),
      Digested::KeyVals(ref kvs) => kvs.unlist(),
      Digested::Postponed(ref _t) => unimplemented!(),
      Digested::RegisterValue(ref _rv) => unimplemented!(),
    }
  }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> {
    match self {
      Digested::TBox(b) => b.be_absorbed(document, state),
      Digested::List(l) => l.be_absorbed(document, state),
      Digested::Whatsit(w) => w.borrow().be_absorbed(document, state),
      Digested::KeyVals(kvs) => kvs.be_absorbed(document, state),
      Digested::Postponed(_) => unimplemented!(),
      Digested::RegisterValue(ref _rv) => unimplemented!(),
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

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    match *self {
      Digested::TBox(ref b) => Error!("digested", "set_property", self, None, s!("Called set_property on Box: {:?}", b)),
      Digested::List(ref l) => Error!("digested", "set_property", self, None, s!("Called set_property on List: {:?}", l)),
      Digested::Whatsit(ref w) => w.borrow_mut().set_property(key, value), // TODO
      _ => unimplemented!(),
    }
  }

  fn get_property(&self, key: &str, state: &mut State) -> Option<Cow<Stored>> {
    match *self {
      Digested::TBox(ref b) => b.get_property(key, state),
      Digested::List(ref l) => {
        Error!("digested", "get_property", self, state, "Called get_property on List: {:?}", l);
        None
      },
      Digested::Whatsit(ref w) => w.borrow().get_property(key, state).map(|v| Cow::Owned(v.into_owned())),
      _ => unimplemented!(),
    }
  }
  fn get_body(&self) -> Option<Digested> {
    match *self {
      Digested::TBox(ref b) => {
        Error!("digested", "get_body", self, None, s!("Called get_body on Box: {:?}", b));
        None
      },
      Digested::List(ref l) => {
        Error!("digested", "get_body", self, None, s!("Called get_body on List: {:?}", l));
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
      Digested::Whatsit(ref w) => w.borrow().get_font().map(|t| Cow::Owned(t.into_owned())),
      _ => unimplemented!(),
    }
  }
}

impl Digested {
  // convenience subset of NumericOps, added here for now as an experiment:
  pub fn value_of(&self) -> f32 {
    match self {
      Digested::RegisterValue(rv) => (**rv).clone().value_of(),
      _ => 0.0,
    }
  }
  pub fn pt_value(&self, prec: Option<u8>) -> f32 {
    match self {
      Digested::RegisterValue(rv) => (**rv).clone().pt_value(prec),
      _ => 0.0,
    }   
  }
}
