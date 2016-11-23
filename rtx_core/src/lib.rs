#![feature(custom_derive)]
#[macro_use]
extern crate lazy_static;

extern crate glob;
extern crate libxml;
extern crate libc;
extern crate regex;
extern crate Archive;
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

use state::State;
use stomach::Stomach;
use tbox::TBox;
use list::List;
use whatsit::Whatsit;
use document::Document;

pub struct Core {
  pub state: State,
  pub stomach: Stomach,
  pub preload: Vec<String>,
}

pub trait BoxOps {
  fn unlist(self) -> Vec<Digested>;
  fn be_absorbed(&mut self, document: &mut Document, state: &mut State);
  fn to_string(&self) -> String {
    "Vec<TBox> for now ".to_string()
  }
  fn stringify(&self) -> String {
    "Vec<TBox> for now ".to_string()
  }
}

#[derive(Debug)]
pub enum Digested {
  BoxObj(TBox),
  ListObj(List),
  WhatsitObj(Whatsit),
}

impl BoxOps for Digested {
  fn unlist(self) -> Vec<Digested> {
    match self {
      Digested::BoxObj(b) => b.unlist(),
      Digested::ListObj(l) => l.unlist(),
      Digested::WhatsitObj(w) => w.unlist(),
    }
  }

  fn be_absorbed(&mut self, document: &mut Document, state: &mut State) {
    match self {
      &mut Digested::BoxObj(ref mut b) => b.be_absorbed(document, state),
      &mut Digested::ListObj(ref mut l) => l.be_absorbed(document, state),
      &mut Digested::WhatsitObj(ref mut w) => w.be_absorbed(document, state),
    }
  }

  fn to_string(&self) -> String {
    match self {
      &Digested::BoxObj(ref b) => b.to_string(),
      &Digested::ListObj(ref l) => l.to_string(),
      &Digested::WhatsitObj(ref w) => w.to_string(),
    }
  }

  fn stringify(&self) -> String {
    match self {
      &Digested::BoxObj(ref b) => b.stringify(),
      &Digested::ListObj(ref l) => l.stringify(),
      &Digested::WhatsitObj(ref w) => w.stringify(),
    }
  }
}

impl Default for Core {
  fn default() -> Self {
    Core {
      preload: Vec::new(),
      stomach: Stomach::default(),
      state: State::new(),
    }
  }
}
