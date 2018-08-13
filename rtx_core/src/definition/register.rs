use std::cell::RefCell;
use std::rc::Rc;

use common::dimension::Dimension;
use common::error::*;
use common::glue::{Glue, MuGlue};
use common::number::Number;
use common::object::Object;
use definition::{BeforeDigestClosure, Definition, DigestionClosure};
use document::Document;
use gullet::Gullet;
use parameter::Parameters;
use state::State;
use stomach::Stomach;
use token::*;
use tokens::Tokens;
use whatsit::Whatsit;
use Digested;

#[derive(Clone)]
pub enum RegisterValue {
  Number(Number),
  Dimension(Dimension),
  Glue(Glue),
  MuGlue(MuGlue),
  Token(Token),
  Tokens(Tokens),
}
impl From<Number> for RegisterValue {
  fn from(n: Number) -> RegisterValue { RegisterValue::Number(n) }
}
impl From<Number> for Option<RegisterValue> {
  fn from(n: Number) -> Option<RegisterValue> { Some(n.into()) }
}
impl From<Dimension> for RegisterValue {
  fn from(n: Dimension) -> RegisterValue { RegisterValue::Dimension(n) }
}
impl From<Glue> for RegisterValue {
  fn from(n: Glue) -> RegisterValue { RegisterValue::Glue(n) }
}
impl From<MuGlue> for RegisterValue {
  fn from(n: MuGlue) -> RegisterValue { RegisterValue::MuGlue(n) }
}
impl From<Token> for RegisterValue {
  fn from(n: Token) -> RegisterValue { RegisterValue::Token(n) }
}
impl From<Tokens> for RegisterValue {
  fn from(n: Tokens) -> RegisterValue { RegisterValue::Tokens(n) }
}
impl<'a> From<&'a RegisterValue> for RegisterType {
  fn from(v: &RegisterValue) -> RegisterType {
    match *v {
      RegisterValue::Number(_) => RegisterType::Number,
      RegisterValue::Dimension(_) => RegisterType::Dimension,
      RegisterValue::Glue(_) => RegisterType::Glue,
      RegisterValue::MuGlue(_) => RegisterType::MuGlue,
      RegisterValue::Token(_) => RegisterType::Token,
      RegisterValue::Tokens(_) => RegisterType::Tokens,
    }
  }
}

impl<'a> From<&'a RegisterValue> for Number {
  fn from(v: &RegisterValue) -> Number {
    match v {
      RegisterValue::Number(n) => n.clone(),
      RegisterValue::Dimension(other) => Number::new(other.value_of()),
      RegisterValue::Glue(other) => Number::new(other.value_of()),
      RegisterValue::MuGlue(other) => Number::new(other.value_of()),
      RegisterValue::Token(other) => {
        error!(target:"expected:number", "Token register can not be cast into a number: {:?}", other);
        Number::new(0)
      },
      RegisterValue::Tokens(other) => {
        error!(target:"expected:number", "Token register can not be cast into a number: {:?}", other);
        Number::new(0)
      },
    }
  }
}
impl From<RegisterValue> for Number {
  fn from(v: RegisterValue) -> Number { (&v).into() }
}

impl RegisterValue {
  pub fn value_of(&self) -> i32 {
    match self {
      RegisterValue::Number(v) => v.value_of(),
      RegisterValue::Dimension(v) => v.value_of(),
      RegisterValue::Glue(v) => v.value_of(),
      RegisterValue::MuGlue(v) => v.value_of(),
      RegisterValue::Token(v) => {
        warn!(target: "register:value_of", ".value_of called on Token {:?}", v);
        -1
      },
      RegisterValue::Tokens(v) => {
        warn!(target: "register:value_of", ".value_of called on Tokens {:?}", v);
        -1
      },
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RegisterType {
  Number,
  Dimension,
  Glue,
  MuGlue,
  Token,
  Tokens,
  Any, // Placeholder for any argument accepted
}

pub type RegisterGetterClosure = Rc<Fn(Vec<Token>, &State) -> Option<RegisterValue>>;
pub type RegisterSetterClosure = Rc<Fn(RegisterValue, Vec<Tokens>, &mut State)>;

#[derive(Clone)]
pub struct Register {
  pub cs: Token,
  pub parameters: Option<Parameters>,
  pub register_type: RegisterType,
  pub readonly: bool,
  pub getter: RegisterGetterClosure,
  pub setter: RegisterSetterClosure,
  // pub traits: PrimitiveOptions,
}
impl Default for Register {
  fn default() -> Self {
    Register {
      cs: T_CS!(s!("Register")),
      parameters: None,
      register_type: RegisterType::Number,
      getter: Rc::new(|_: Vec<Token>, _: &State| Some(RegisterValue::Number(Number::new(0)))),
      setter: Rc::new(|_: RegisterValue, _: Vec<Tokens>, _: &mut State| {}),
      readonly: false,
    }
  }
}
impl PartialEq for Register {
  fn eq(&self, other: &Register) -> bool { self.cs == other.cs }
}

impl Object for RefCell<Register> {}
impl Definition for RefCell<Register> {
  fn is_register(&self) -> bool { true }
  fn is_prefix(&self) -> bool { false }
  // not implemented for primitives
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> { Ok(Tokens!()) }
  // TODO:
  fn get_parameters(&self) -> &Option<Parameters> { &None } // TODO: How do we do this with a RefCell ?!
  fn get_cs(&self) -> Token { self.borrow().cs.clone() }

  fn get_cs_name(&self) -> String { self.borrow().cs.get_cs_name() }

  fn get_locator(&self) -> String { String::from("Locator is TODO") }

  // No before/after daemons ???
  // (other than afterassign)
  fn invoke_primitive(
    &self,
    stomach: &mut Stomach,
    _caller: Rc<Definition>,
    state: &mut State,
  ) -> Result<Vec<Digested>>
  {
    // my $profiled = $STATE->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;

    let gullet = stomach.get_gullet_mut();
    let args = self.read_arguments(gullet, state)?;
    gullet.read_keyword(&["="], state)?; // Ignore
    let value = gullet.read_value(self.register_type().unwrap(), state)?;

    self.borrow_mut().set_value(value, args, state);

    // if (my $after = $STATE->lookupValue('afterAssignment')) {
    //   $STATE->assignValue(afterAssignment => undef, 'global');
    //   $gullet->unread($after); }    # primitive returns boxes, so these need to be digested!
    // # Tracing ?
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;

    Ok(Vec::new())
  }

  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn read_arguments(&self, gullet: &mut Gullet, state: &mut State) -> Result<Vec<Tokens>> {
    match self.borrow().parameters {
      None => Ok(Vec::new()),
      Some(ref params) => params.read_arguments(gullet, self, state),
    }
  }

  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
    _state: &mut State,
  ) -> Result<()>
  {
    Ok(())
  }
  fn value_of(&self, args: Vec<Token>, state: &State) -> Option<RegisterValue> {
    (self.borrow().getter)(args, state)
  }
  fn register_type(&self) -> Option<RegisterType> { Some(self.borrow().register_type) }
}

impl Register {
  fn is_readonly(&self) -> bool { self.readonly }
  fn set_value(&mut self, value: RegisterValue, args: Vec<Tokens>, state: &mut State) {
    (self.setter)(value, args, state);
  }
}
