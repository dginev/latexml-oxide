use std::sync::Arc;
use state::State;
use common::object::Object;
use Digested;
use token::*;
use gullet::Gullet;
use stomach::Stomach;
use parameter::Parameters;
use definition::{Definition, BeforeDigestClosure, DigestionClosure};
use document::Document;

#[derive(Clone)]
pub struct PrimitiveOptions {
  pub bounded: bool,
  pub mode: String, // TODO
  pub before_digest: Vec<BeforeDigestClosure>,
  pub after_digest: Vec<DigestionClosure>,
}
impl Default for PrimitiveOptions {
  fn default() -> Self {
    PrimitiveOptions {
      bounded: false,
      before_digest: Vec::new(),
      after_digest: Vec::new(),
      mode: String::new(),
    }
  }
}

pub type PrimitiveClosure = Arc<Fn(&mut Document, &mut State)>;
#[derive(Clone)]
pub struct Primitive {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub nargs: Option<usize>,
  pub replacement: String,
  pub options: PrimitiveOptions,
}
impl Default for Primitive {
  fn default() -> Self {
    Primitive {
      cs: T_CS!("Primitive".to_string()),
      paramlist: None,
      nargs: None,
      replacement: String::new(),
      options: PrimitiveOptions::default(),
    }
  }
}

impl Object for Primitive {}
impl Definition for Primitive {
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> {
    Some(&self.options.before_digest)
  }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> {
    Some(&self.options.after_digest)
  }

  fn invoke(&self, _gullet: &mut Gullet, _state: &mut State) -> Vec<Token> {
    Vec::new()
  }
  fn invoke_primitive(&self, _gullet: &mut Stomach, _caller: Arc<Definition>, _state: &mut State) -> Vec<Digested> {
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
  fn get_parameters(&self) -> &Option<Parameters> {
    &self.paramlist
  }
  fn get_num_args(&self) -> usize {
    let nargs = match self.nargs {
      Some(n) => n,
      None => {
        match &self.paramlist {
          &Some(ref params) => params.get_num_args(),
          &None => 0,
        }
      }
    };
    // TODO: Rethink the memoize in this immutable setting
    // self.nargs = Some(nargs);
    nargs
  }
}