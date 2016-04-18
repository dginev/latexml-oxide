#[macro_use]
extern crate lazy_static;

extern crate glob;
extern crate libxml;
extern crate libc;
extern crate regex;
extern crate Archive;
extern crate rustc_serialize;
extern crate rand;
extern crate tempfile;
extern crate time;

#[macro_use]pub mod aux_macros;
#[macro_use]pub mod token;
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

pub struct Core {
  pub state: State,
  pub stomach: Stomach,
  pub preload: Vec<String>,
}
pub trait Digested {
  fn unlist(&self) -> Vec<&TBox>;
  fn to_string(&self) -> String {
    "Vec<TBox> for now ".to_string()
  }
  fn stringify(&self) -> String {
    "Vec<TBox> for now ".to_string()
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
