use std::sync::Arc;
use state::State;
use common::object::Object;

use core::token::*;
use core::tbox::TBox;
use core::gullet::Gullet;
use core::stomach::Stomach;
// use core::whatsit::Whatsit;
use core::parameter::Parameters;
use core::definition::Definition;
use core::definition::expandable::ExpansionClosure;
use core::definition::constructor::DigestionClosure;
use core::document::Document;


pub struct PrimitiveOptions {
  pub bounded : bool,
  pub mode : String, // TODO
  pub before_digest : Option<ExpansionClosure>,
  pub after_digest : Option<DigestionClosure>,
}
impl Default for PrimitiveOptions {
  fn default() -> Self { 
    PrimitiveOptions {
      bounded : false,
      before_digest : None,
      after_digest : None,
      mode : String::new()
    }
  }
}

pub type PrimitiveClosure = Arc<Box<Fn(&mut Document, &mut State)>>;
#[derive(Clone)]
pub struct Primitive {
  pub cs : Token,
  pub paramlist : Option<Parameters>,
  pub nargs : Option<usize>,
  pub replacement : String
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs : T_CS("Primitive".to_string()),
      paramlist : None,
      nargs : None,
      replacement : String::new()
    }
  }
}

impl Object for Primitive {}
impl Definition for Primitive {
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
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_num_args(&self) -> usize {
    let nargs = match self.nargs {
      Some(n) => n,
      None => {
        match &self.paramlist {
          &Some(ref params) => params.get_num_args(),
          &None => 0
        }
      }
    };
    // TODO: Rethink the memoize in this immutable setting
    // self.nargs = Some(nargs);
    nargs
  }


}