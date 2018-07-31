use common::error::*;
use common::object::Object;
use definition::{BeforeDigestClosure, ConditionalClosure, Definition, DigestionClosure};
use document::Document;
use gullet::Gullet;
use parameter::Parameters;
use state::State;
use std::rc::Rc;
use stomach::Stomach;
use token::*;
use tokens::Tokens;
use whatsit::Whatsit;
use Digested;

#[derive(Debug, Clone)]
pub struct Register {
  pub cs: Token,
  pub parameters: Option<Parameters>,
  pub register_type: Option<String>,
  pub readonly: bool,
  // pub traits: PrimitiveOptions,
}
impl Default for Register {
  fn default() -> Self {
    Register {
      cs: T_CS!(s!("Register")),
      parameters: None,
      register_type: None,
      readonly: false,
    }
  }
}
impl PartialEq for Register {
  fn eq(&self, other: &Register) -> bool { self.cs == other.cs }
}

impl Register {
  // `is_register` begs to be refactored into a better naming scheme
  fn is_register(&self) -> Option<String> { self.register_type.clone() }
  fn is_readonly(&self) -> bool { self.readonly }
  fn is_prefix(&self) -> bool { false }
}

impl Object for Register {}
impl Definition for Register {
  // No before/after daemons ???
  // (other than afterassign)
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  // TODO:
  fn get_parameters(&self) -> &Option<Parameters> { &self.parameters }
  fn get_cs(&self) -> Token { self.cs.clone() }

  fn get_cs_name(&self) -> String { self.cs.get_cs_name() }

  fn get_locator(&self) -> String { String::from("Locator is TODO") }

  // Not implemented for expandable
  fn invoke_primitive(
    &self,
    _gullet: &mut Stomach,
    _caller: Rc<Definition>,
    _state: &mut State,
  ) -> Result<Vec<Digested>>
  {
    Ok(Vec::new())
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
    _state: &mut State,
  ) -> Result<()>
  {
    Ok(())
  }
}

//impl Register {}
