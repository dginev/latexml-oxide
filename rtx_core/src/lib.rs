#[macro_use]
extern crate lazy_static;

extern crate glob;
extern crate libxml;
extern crate libc;
extern crate regex;
extern crate rand;
extern crate tempfile;
extern crate time;

#[macro_use]pub mod aux_macros;
#[macro_use]pub mod token;
pub mod tokens;
pub mod stomach;
pub mod gullet;
pub mod mouth;
pub mod definition;
pub mod parameter;
pub mod tbox;
pub mod list;
pub mod document;
pub mod whatsit;
pub mod common;
pub mod state;
pub mod util;

use std::fmt;
use state::{State, StateOptions};
use common::model::Model;
use stomach::Stomach;
use tbox::Tbox;
use list::List;
use whatsit::Whatsit;
use document::Document;

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
  pub preload: Option<Vec<String>>
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
      Some(p) => p
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

    Core{
      state: state,
      preload: preload,
      .. Core::default()
    }
  }

  pub fn state_mut(&mut self) -> &mut State {
    &mut self.state
  }
}

pub trait BoxOps {
  fn unlist(self) -> Vec<Digested>;
  fn be_absorbed(self, document: &mut Document, state: &mut State);
  fn to_string(&self) -> String {
    "Vec<Tbox> for now ".to_string()
  }
  fn stringify(&self) -> String {
    "Vec<Tbox> for now ".to_string()
  }
}

#[derive(Debug, Clone)]
pub enum TexMode {
  Math,
  Text
}

#[derive(Clone)]
pub enum Digested {
  Box(Tbox),
  List(List),
  Whatsit(Whatsit),
}
impl fmt::Debug for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &Digested::Box(ref v) => write!(f, "{:?}", v),
      &Digested::List(ref v) => write!(f, "{:?}", v),
      &Digested::Whatsit(ref v) => write!(f, "{:?}", v),
    }
  }
}

impl BoxOps for Digested {
  fn unlist(self) -> Vec<Digested> {
    match self {
      Digested::Box(b) => b.unlist(),
      Digested::List(l) => l.unlist(),
      Digested::Whatsit(w) => w.unlist(),
    }
  }

  fn be_absorbed(self, document: &mut Document, state: &mut State) {
    match self {
      Digested::Box(b) => b.be_absorbed(document, state),
      Digested::List(l) => l.be_absorbed(document, state),
      Digested::Whatsit(w) => w.be_absorbed(document, state),
    }
  }

  fn to_string(&self) -> String {
    match self {
      &Digested::Box(ref b) => b.to_string(),
      &Digested::List(ref l) => l.to_string(),
      &Digested::Whatsit(ref w) => w.to_string(),
    }
  }

  fn stringify(&self) -> String {
    match self {
      &Digested::Box(ref b) => b.stringify(),
      &Digested::List(ref l) => l.stringify(),
      &Digested::Whatsit(ref w) => w.stringify(),
    }
  }
}
