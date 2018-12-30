use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::glue::{Glue, MuGlue};
use crate::common::number::Number;
use crate::common::object::Object;
use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure};
use crate::document::Document;
use crate::gullet::Gullet;
use crate::parameter::Parameters;
use crate::state::State;
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

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

impl Default for RegisterValue {
  fn default() -> Self { RegisterValue::Number(Number::new(0.0)) }
}

impl<'a> From<&'a RegisterValue> for Number {
  fn from(v: &RegisterValue) -> Number {
    match v {
      RegisterValue::Number(n) => *n,
      RegisterValue::Dimension(other) => Number::new(other.value_of()),
      RegisterValue::Glue(other) => Number::new(other.value_of()),
      RegisterValue::MuGlue(other) => Number::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number(),
      RegisterValue::Tokens(other) => {
        error!(target:"expected:number", "Token register can not be cast into a number: {:?}", other);
        Number::new(0.0)
      },
    }
  }
}
impl From<RegisterValue> for Number {
  fn from(v: RegisterValue) -> Number { (&v).into() }
}
impl From<RegisterValue> for Dimension {
  fn from(v: RegisterValue) -> Dimension { (&v).into() }
}
impl From<RegisterValue> for Glue {
  fn from(v: RegisterValue) -> Glue { (&v).into() }
}

impl From<Number> for Dimension {
  fn from(n: Number) -> Dimension { Dimension::new(n.value_of()) }
}
impl From<Number> for Glue {
  fn from(n: Number) -> Glue { Glue::new(n.value_of()) }
}

impl<'a> From<&'a RegisterValue> for Dimension {
  fn from(v: &RegisterValue) -> Dimension {
    match v {
      RegisterValue::Dimension(n) => *n,
      RegisterValue::Number(other) => Dimension::new(other.value_of()),
      RegisterValue::Glue(other) => Dimension::new(other.value_of()),
      RegisterValue::MuGlue(other) => Dimension::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number().into(),
      RegisterValue::Tokens(other) => {
        error!(target:"expected:dimension", "Token register can not be cast into a dimension: {:?}", other);
        Dimension::new(0.0)
      },
    }
  }
}
impl<'a> From<&'a RegisterValue> for Glue {
  fn from(v: &RegisterValue) -> Glue {
    match v {
      RegisterValue::Glue(n) => *n,
      RegisterValue::Number(other) => Glue::new(other.value_of()),
      RegisterValue::Dimension(other) => Glue::new(other.value_of()),
      RegisterValue::MuGlue(other) => Glue::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number().into(),
      RegisterValue::Tokens(other) => {
        error!(target:"expected:dimension", "Token register can not be cast into a Glue: {:?}", other);
        Glue::new(0.0)
      },
    }
  }
}

impl RegisterValue {
  pub fn value_of(&self) -> f32 {
    match self {
      RegisterValue::Number(v) => v.value_of(),
      RegisterValue::Dimension(v) => v.value_of(),
      RegisterValue::Glue(v) => v.value_of(),
      RegisterValue::MuGlue(v) => v.value_of(),
      RegisterValue::Token(v) => {
        warn!(target: "register:value_of", ".value_of called on Token {:?}", v);
        -1.0
      },
      RegisterValue::Tokens(v) => {
        warn!(target: "register:value_of", ".value_of called on Tokens {:?}", v);
        -1.0
      },
    }
  }

  pub fn to_string(&self) -> String { self.value_of().to_string() }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RegisterType {
  Number,
  Dimension,
  Glue,
  MuGlue,
  Token,
  Tokens,
  CharDef,
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
  pub internalcs: Option<Token>,
  pub internalvalue: Option<RegisterValue>,
  pub getter: RegisterGetterClosure,
  pub setter: RegisterSetterClosure,
  // pub traits: PrimitiveOptions,
}
impl Default for Register {
  fn default() -> Self {
    Register {
      cs: T_CS!("Register"),
      parameters: None,
      register_type: RegisterType::Number,
      getter: Rc::new(|_: Vec<Token>, _: &State| Some(RegisterValue::Number(Number::new(0.0)))),
      setter: Rc::new(|_: RegisterValue, _: Vec<Tokens>, _: &mut State| {}),
      readonly: false,
      internalcs: None,
      internalvalue: None,
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
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> { unimplemented!() }
  fn get_parameters(&self) -> &Option<Parameters> { unimplemented!() } // TODO: How do we do this with a RefCell ?!
  fn get_cs(&self) -> Cow<Token> { Cow::Owned(self.borrow().cs.clone()) }

  fn get_cs_name(&self) -> Cow<str> { Cow::Owned(self.borrow().cs.get_cs_name().to_owned()) }

  fn get_locator(&self) -> String { String::from("Locator is TODO") }

  // No before/after daemons ???
  // (other than afterassign)
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Rc<Definition>, state: &mut State) -> Result<Vec<Digested>> {
    // CharDef case
    if self.borrow().register_type == RegisterType::CharDef {
      return match self.borrow().internalcs {
        // Tracing ?
        None => Ok(Vec::new()),
        Some(ref cs) => stomach.invoke_token(cs, state),
      };
    }

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

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<()> {
    fatal!(Definition, Unexpected, "do_absorbtion on Primitive should never be called!");
  }
  fn value_of(&self, args: Vec<Token>, state: &State) -> Option<RegisterValue> {
    if self.borrow().register_type == RegisterType::CharDef {
      self.borrow().internalvalue.clone()
    } else {
      (self.borrow().getter)(args, state)
    }
  }
  fn register_type(&self) -> Option<RegisterType> { Some(self.borrow().register_type) }
}

impl Register {
  pub fn is_readonly(&self) -> bool { self.readonly }
  pub fn set_value(&mut self, value: RegisterValue, args: Vec<Tokens>, state: &mut State) {
    if self.register_type == RegisterType::CharDef {
      error!(target:"unexpected:chardef", "Can't assign to chardef {}",self.cs.get_cs_name());
    } else {
      (self.setter)(value, args, state);
    }
  }

  pub fn new_chardef(cs: Token, internalvalue: Option<RegisterValue>, internalcs: Option<Token>) -> Self {
    Register {
      cs,
      parameters: None,
      internalvalue,
      internalcs,
      register_type: RegisterType::CharDef,
      readonly: true,
      //locator => $STATE->getStomach->getGullet->getMouth->getLocator,
      ..Register::default()
    }
  }
}

//===============================================================================
