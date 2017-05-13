use std::rc::Rc;
use state::State;
use Digested;
use token::*;
use tokens::Tokens;
use gullet::Gullet;
use stomach::Stomach;
use parameter::Parameters;
use common::object::Object;
use common::error::*;
use definition::{Definition, ExpansionClosure, BeforeDigestClosure, DigestionClosure};
use whatsit::Whatsit;
use document::Document;

#[derive(Clone)]
pub struct Expandable {
  pub is_protected: bool,
  pub alias: Option<String>,
  pub locator: String,
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub expansion: ExpansionClosure,
  pub trivial_expansion: Option<Vec<Token>>,
}
impl Default for Expandable {
  fn default() -> Self {
    Expandable {
      is_protected: false,
      trivial_expansion: None,
      alias: None,
      locator: String::new(),
      cs: T_CS!("Expandable".to_string()),
      paramlist: None,
      expansion: Rc::new(|_gullet, _args, _state| Ok(Vec::new())),
    }
  }
}
impl Object for Expandable {
  fn is_definition(&self) -> bool {
    true
  }
  fn is_expandable(&self) -> bool {
    true
  }
}
impl Definition for Expandable {
  fn is_protected(&self) -> bool {
    self.is_protected
  }
  fn get_parameters(&self) -> &Option<Parameters> {
    &self.paramlist
  }
  fn get_cs(&self) -> Token {
    self.cs.clone()
  }

  fn get_cs_name(&self) -> String {
    match &self.alias {
      &Some(ref alias) => alias.clone(),
      &None => self.cs.get_cs_name(),
    }
  }

  fn get_locator(&self) -> String {
    self.locator.clone()
  }

  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Vec<Token>> {
    // Expand the expandable control sequence. This should be carried out by the Gullet.
    // log!("-- expandable invoke for {:?}", self.get_cs());
    if self.trivial_expansion.is_some() {
      match &self.trivial_expansion {
        &Some(ref expansion) => Ok(expansion.clone()),
        &None => Ok(Vec::new()),
      }
    } else {
      let args = try!(self.read_arguments(gullet, state));
      self.do_invocation(gullet, args, state)
    }
  }

  // Not implemented for expandable
  fn invoke_primitive(&self, _gullet: &mut Stomach, _caller: Rc<Definition>, _state: &mut State) -> Result<Vec<Digested>> {
    Ok(Vec::new())
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> {
    None
  }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> {
    None
  }
  fn capture_body(&self) -> bool {false}
  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) {
    panic!("do_absorbtion on Primitive should never be called!");
  }
}

impl Expandable {
  fn do_invocation(&self, gullet: &mut Gullet, args: Vec<Tokens>, state: &mut State) -> Result<Vec<Token>> {
    let closure: &ExpansionClosure = &self.expansion;
    closure(gullet, args, state)
  }
}
