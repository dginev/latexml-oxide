#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate ansi_term;
extern crate glob;
extern crate libxml;
extern crate quote;
extern crate rand;
extern crate regex;
extern crate tempfile;
extern crate time;

#[macro_use]
pub mod aux_macros;
#[macro_use]
pub mod common;
#[macro_use]
pub mod token;
#[macro_use]
pub mod tokens;
#[macro_use]
pub mod definition;
pub mod document;
pub mod gullet;
pub mod list;
pub mod mouth;
pub mod parameter;
pub mod state;
pub mod stomach;
pub mod tbox;
pub mod util;
pub mod whatsit;

use common::error::*;
use common::font::Font;
use common::model::Model;
use document::Document;
use list::List;
use state::{ObjectStore, State, StateOptions};
use std::fmt;
use stomach::Stomach;
use tbox::Tbox;
use tokens::Tokens;
use whatsit::Whatsit;

pub struct Core {
  pub state: State,
  pub stomach: Stomach,
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
    Core {
      preload: Vec::new(),
      stomach: Stomach::default(),
      state: State::new(StateOptions::default()),
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

    let state = State::new(state_options);

    Core {
      state: state,
      preload: preload,
      ..Core::default()
    }
  }

  pub fn state_mut(&mut self) -> &mut State { &mut self.state }
}

pub trait BoxOps {
  fn unlist(self) -> Vec<Digested>;
  fn be_absorbed(self, document: &mut Document, state: &mut State) -> Result<()>;
  fn to_string(&self) -> String;
  fn stringify(&self) -> String { s!("Vec<Tbox> for now ") }
  fn set_property(&mut self, _key: &str, _value: ObjectStore) {}
  fn get_property(&self, _key: &str) -> Option<&ObjectStore> {
    error!(target: "boxops:get_property", "Generic BoxOps::get_property should never be called!");
    None
  }
  fn get_body(&self) -> Option<&Digested> {
    error!(target: "boxops:get_body", "Generic BoxOps::get_body should never be called!");
    None
  }
  fn get_font(&self) -> Option<&Font>;
  fn revert(&self) -> Tokens;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TexMode {
  Math,
  Text,
}

#[derive(Clone, PartialEq)]
pub enum Digested {
  Box(Tbox),
  List(List),
  Whatsit(Whatsit),
}
impl fmt::Debug for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Digested::Box(ref v) => write!(f, "{:?}", v),
      Digested::List(ref v) => write!(f, "{:?}", v),
      Digested::Whatsit(ref v) => write!(f, "{:?}", v),
    }
  }
}

impl Default for Digested {
  fn default() -> Self { Digested::Box(Tbox::default()) }
}

impl BoxOps for Digested {
  fn unlist(self) -> Vec<Digested> {
    match self {
      Digested::Box(b) => b.unlist(),
      Digested::List(l) => l.unlist(),
      Digested::Whatsit(w) => w.unlist(),
    }
  }

  fn be_absorbed(self, document: &mut Document, state: &mut State) -> Result<()> {
    match self {
      Digested::Box(b) => b.be_absorbed(document, state),
      Digested::List(l) => l.be_absorbed(document, state),
      Digested::Whatsit(w) => w.be_absorbed(document, state),
    }
  }

  fn to_string(&self) -> String {
    match *self {
      Digested::Box(ref b) => b.to_string(),
      Digested::List(ref l) => l.to_string(),
      Digested::Whatsit(ref w) => w.to_string(),
    }
  }

  fn stringify(&self) -> String {
    match *self {
      Digested::Box(ref b) => b.stringify(),
      Digested::List(ref l) => l.stringify(),
      Digested::Whatsit(ref w) => w.stringify(),
    }
  }

  fn set_property(&mut self, key: &str, value: ObjectStore) {
    match *self {
      Digested::Box(ref b) => {
        error!(target: "digested:set_property", "Called set_property on Box: {:?}", b)
      },
      Digested::List(ref l) => {
        error!(target: "digested:set_property", "Called set_property on List: {:?}", l)
      },
      Digested::Whatsit(ref mut w) => w.set_property(key, value),
    }
  }

  fn get_property(&self, key: &str) -> Option<&ObjectStore> {
    match *self {
      Digested::Box(ref b) => {
        error!(target: "digested:get_property", "Called get_property on Box: {:?}", b);
        None
      },
      Digested::List(ref l) => {
        error!(target: "digested:get_property", "Called get_property on List: {:?}", l);
        None
      },
      Digested::Whatsit(ref w) => w.get_property(key),
    }
  }
  fn get_body(&self) -> Option<&Digested> {
    match *self {
      Digested::Box(ref b) => {
        error!(target: "digested:get_body", "Called get_body on Box: {:?}", b);
        None
      },
      Digested::List(ref l) => {
        error!(target: "digested:get_body", "Called get_body on List: {:?}", l);
        None
      },
      Digested::Whatsit(ref w) => w.get_body(),
    }
  }

  fn get_font(&self) -> Option<&Font> {
    match *self {
      Digested::Box(ref b) => b.get_font(),
      Digested::List(ref l) => l.get_font(),
      Digested::Whatsit(ref w) => w.get_font(),
    }
  }

  fn revert(&self) -> Tokens {
    match *self {
      Digested::Box(ref b) => b.revert(),
      Digested::List(ref l) => l.revert(),
      Digested::Whatsit(ref w) => w.revert(),
    }
  }
}
