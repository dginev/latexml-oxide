use std::sync::Arc;
use state::State;
use common::object::Object;

use core::token::*;
use core::tbox::TBox;
use core::gullet::Gullet;
use core::stomach::Stomach;
use core::whatsit::Whatsit;
use core::parameter::Parameters;
use core::definition::Definition;
use core::definition::expandable::ExpansionClosure;
use core::document::Document;


pub struct ConstructorOptions {
  pub bounded : bool,
  pub mode : String, // TODO
  pub beforeDigest : Option<ExpansionClosure>,
  pub afterDigest : Option<DigestionClosure>,
}
impl Default for ConstructorOptions {
  fn default() -> Self { 
    ConstructorOptions {
      bounded : false,
      beforeDigest : None,
      afterDigest : None,
      mode : String::new()
    }
  }
}

pub type ConstructionClosure = Arc<Box<Fn(&mut Document, &mut State)>>;
pub type DigestionClosure = Arc<Box<Fn(&mut Stomach, &mut Whatsit, &mut State)>>;
#[derive(Clone)]
pub struct Constructor {
  pub cs : Token,
  pub paramlist : Option<Parameters>,
  pub nargs : Option<usize>,
  pub replacement : String
}
impl Default for Constructor {
  fn default() -> Self {
    Constructor {
      cs : T_CS("Constructor".to_string()),
      paramlist : None,
      nargs : None,
      replacement : String::new()
    }
  }
}

impl Object for Constructor {}
impl Definition for Constructor {
  fn invoke(&self, gullet : &mut Gullet, state : &mut State) -> Vec<Token> {
    Vec::new()
  }
  fn invoke_primitive(&self, gullet : &mut Stomach, state : &mut State) -> Vec<TBox> {
    Vec::new()
  }

  fn get_cs(&self) -> Token {
    self.cs.clone()
  }
  fn get_cs_name(&self) -> String {
    self.cs.get_cs_name()
  }
  fn get_locator(&self) -> String {
    unimplemented!()
  }
  fn get_num_args(&mut self) -> usize {
    let nargs = match self.nargs {
      Some(n) => n,
      None => {
        match &self.paramlist {
          &Some(ref params) => params.get_num_args(),
          &None => 0
        }
      }
    };
    self.nargs = Some(nargs);
    nargs
  }


}