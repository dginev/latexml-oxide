use std::borrow::Borrow;
use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::glue::Glue;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::numeric_ops::{NumericOps};
use crate::common::number::{Number};
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
use crate::{Digested, Locator};

use super::argument::ArgWrap;

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterValue {
  Number(Number),
  Dimension(Dimension),
  MuDimension(MuDimension),
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
impl From<MuDimension> for RegisterValue {
  fn from(n: MuDimension) -> RegisterValue { RegisterValue::MuDimension(n) }
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
      RegisterValue::MuDimension(_) => RegisterType::MuDimension,
      RegisterValue::Glue(_) => RegisterType::Glue,
      RegisterValue::MuGlue(_) => RegisterType::MuGlue,
      RegisterValue::Token(_) => RegisterType::Token,
      RegisterValue::Tokens(_) => RegisterType::Tokens,
    }
  }
}
impl From<RegisterValue> for RegisterType {
  fn from(v: RegisterValue) -> RegisterType { v.borrow().into() }
}

impl Default for RegisterValue {
  fn default() -> Self { RegisterValue::Number(Number::new(0)) }
}
impl Object for RegisterValue {
  fn stringify(&self) -> String { s!("RegisterValue[{}]", self) }
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
  fn revert(&self, state: &mut State) -> Result<Tokens> {
    match self {
      // ExplodeText($self->toString);
      RegisterValue::Number(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::Dimension(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::MuDimension(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::Glue(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::MuGlue(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::Token(ref value) => Ok(Tokens!(value.clone().revert())),
      RegisterValue::Tokens(ref value) => Ok(Tokens::new(value.clone().revert())), // clone?
    }
  }
}

impl NumericOps for RegisterValue {
  fn new(number: i32) -> Self { RegisterValue::Number(Number::new(number)) }
  fn new_f32(number: f32) -> Self { RegisterValue::Number(Number::new_f32(number)) }
  fn value_of(self) -> i32 {
    match self {
      RegisterValue::Number(v) => v.value_of(),
      RegisterValue::Dimension(v) => v.value_of(),
      RegisterValue::MuDimension(v) => v.value_of(),
      RegisterValue::Glue(v) => v.value_of(),
      RegisterValue::MuGlue(v) => v.value_of(),
      RegisterValue::Token(v) => {
        let message = s!(".value_of called on Token {:?}", v);
        Warn!("register", "value_of", None, None, message);
        -1
      },
      RegisterValue::Tokens(v) => {
        let message = s!(".value_of called on Tokens {:?}", v);
        Warn!("register", "value_of", None, None, message);
        -1
      },
    }
  }
  fn register_type(&self) -> RegisterType {
    match self {
      RegisterValue::Number(v) => RegisterType::Number,
      RegisterValue::Dimension(v) => RegisterType::Dimension,
      RegisterValue::MuDimension(v) => RegisterType::MuDimension,
      RegisterValue::Glue(v) => RegisterType::Glue,
      RegisterValue::MuGlue(v) => RegisterType::MuGlue,
      RegisterValue::Token(v) => RegisterType::Token,
      RegisterValue::Tokens(v) => RegisterType::Tokens,
    }
  }
  fn add<T: NumericOps>(self, other: T) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.add(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.add(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.add(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.add(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.add(other)),
      RegisterValue::Token(v) => unimplemented!(),
      RegisterValue::Tokens(v) => unimplemented!(),
    }
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.subtract(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.subtract(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.subtract(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.subtract(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.subtract(other)),
      RegisterValue::Token(v) => unimplemented!(),
      RegisterValue::Tokens(v) => unimplemented!(),
    }
  }
  fn multiply<T: NumericOps>(self, other: T) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.multiply(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.multiply(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.multiply(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.multiply(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.multiply(other)),
      RegisterValue::Token(v) => unimplemented!(),
      RegisterValue::Tokens(v) => unimplemented!(),
    }
  }
  fn negate(self) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.negate()),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.negate()),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.negate()),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.negate()),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.negate()),
      RegisterValue::Token(v) => unimplemented!(),
      RegisterValue::Tokens(v) => unimplemented!(),
    }
  }
  fn divide<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.divide(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.divide(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.divide(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.divide(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.divide(other)),
      RegisterValue::Token(v) => unimplemented!(),
      RegisterValue::Tokens(v) => unimplemented!(),
    }
  }
  /// For now only meant as a type cast, unimplemented in other cases
  /// DO NOT use this method to cast into a Glue object, define a `.to_glue()` instead
  fn into_glue_type(self) -> Glue {
    match self {
      RegisterValue::Glue(v) => v,
      _ => unimplemented!(),
    }
  }
}

impl<'a> From<&'a RegisterValue> for Number {
  fn from(v: &RegisterValue) -> Number {
    match v {
      RegisterValue::Number(n) => *n,
      RegisterValue::Dimension(other) => Number::new(other.value_of()),
      RegisterValue::MuDimension(other) => Number::new(other.value_of()),
      RegisterValue::Glue(other) => Number::new(other.value_of()),
      RegisterValue::MuGlue(other) => Number::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number(),
      RegisterValue::Tokens(other) => other.to_number(),
    }
  }
}
impl From<RegisterValue> for Number {
  fn from(v: RegisterValue) -> Number { v.borrow().into() }
}
impl From<RegisterValue> for Dimension {
  fn from(v: RegisterValue) -> Dimension { v.borrow().into() }
}
impl From<RegisterValue> for Glue {
  fn from(v: RegisterValue) -> Glue { v.borrow().into() }
}
impl From<RegisterValue> for MuGlue {
  fn from(v: RegisterValue) -> MuGlue { v.borrow().into() }
}
impl From<RegisterValue> for f32 {
  fn from(v: RegisterValue) -> f32 { v.value_of() as f32 }
}
impl From<RegisterValue> for i32 {
  fn from(v: RegisterValue) -> i32 { v.value_of() }
}

impl From<Number> for Dimension {
  fn from(n: Number) -> Dimension { Dimension::new(n.value_of()) }
}
impl From<Number> for Glue {
  fn from(n: Number) -> Glue { Glue::new(n.value_of()) }
}
impl From<Number> for MuGlue {
  fn from(n: Number) -> MuGlue { MuGlue::new(n.value_of()) }
}

// TODO: Does this successfully emulate the behavior in latexml?
//   see example use in gullet::read_tokens_value
impl From<RegisterValue> for Tokens {
  fn from(v: RegisterValue) -> Tokens { Tokens!(T_OTHER!(v.value_of())) }
}

impl<'a> From<&'a RegisterValue> for Dimension {
  fn from(v: &RegisterValue) -> Dimension {
    match v {
      RegisterValue::Dimension(n) => *n,
      RegisterValue::MuDimension(other) => Dimension::new(other.value_of()),
      RegisterValue::Number(other) => Dimension::new(other.value_of()),
      RegisterValue::Glue(other) => Dimension::new(other.value_of()),
      RegisterValue::MuGlue(other) => Dimension::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number().into(),
      RegisterValue::Tokens(other) => {
        let message = s!("Token register can not be cast into a dimension: {:?}", other);
        Error!("expected", "dimension", None, None, message);
        Dimension::new(0)
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
      RegisterValue::MuDimension(other) => Glue::new(other.value_of()),
      RegisterValue::MuGlue(other) => Glue::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number().into(),
      RegisterValue::Tokens(other) => {
        let message = s!("Token register can not be cast into a Glue: {:?}", other);
        Error!("expected", "dimension", None, None, message);
        Glue::new(0)
      },
    }
  }
}

impl<'a> From<&'a RegisterValue> for MuGlue {
  fn from(v: &RegisterValue) -> MuGlue {
    match v {
      RegisterValue::MuGlue(n) => *n,
      RegisterValue::Number(other) => MuGlue::new(other.value_of()),
      RegisterValue::Dimension(other) => MuGlue::new(other.value_of()),
      RegisterValue::MuDimension(other) => MuGlue::new(other.value_of()),
      RegisterValue::Glue(other) => MuGlue {
        skip: other.skip,
        plus: other.plus,
        pfill: other.pfill,
        minus: other.minus,
        mfill: other.mfill,
      },
      RegisterValue::Token(other) => other.to_number().into(),
      RegisterValue::Tokens(other) => {
        let message = s!("Token register can not be cast into a Glue: {:?}", other);
        Error!("expected", "dimension", None, None, message);
        MuGlue::new(0)
      },
    }
  }
}

impl fmt::Display for RegisterValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegisterValue::Dimension(d) => write!(f, "{}", d),
      RegisterValue::MuDimension(d) => write!(f, "{}", d),
      RegisterValue::Glue(g) => write!(f, "{}", g),
      RegisterValue::MuGlue(g) => write!(f, "{}", g),
      other => write!(f, "{}", self.clone().value_of()),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegisterType {
  Number,
  Dimension,
  MuDimension,
  Glue,
  MuGlue,
  Token,
  Tokens,
  CharDef,
  Any, // Placeholder for any argument accepted
}

pub type RegisterGetterClosure = Arc<dyn Fn(Vec<ArgWrap>, &State) -> Option<RegisterValue>>;
pub type RegisterSetterClosure = Arc<dyn Fn(RegisterValue, Vec<ArgWrap>, &mut State)>;

#[derive(Clone)]
pub struct Register {
  pub cs: Token,
  pub parameters: Option<Parameters>,
  pub register_type: RegisterType,
  pub readonly: bool,
  pub internalcs: Option<Token>,
  pub value: Option<RegisterValue>,
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
      getter: Arc::new(|_: Vec<ArgWrap>, _: &State| Some(RegisterValue::Number(Number::new(0)))),
      setter: Arc::new(|_: RegisterValue, _: Vec<ArgWrap>, _: &mut State| {}),
      readonly: false,
      internalcs: None,
      value: None,
    }
  }
}
impl PartialEq for Register {
  fn eq(&self, other: &Register) -> bool { self.cs == other.cs }
}
impl fmt::Debug for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Register[cs:{:?}, parameters:{:?}, type:{:?}, readonly:{:?}, internalcs:{:?}, value:{:?}]",
      self.cs, self.parameters, self.register_type, self.readonly, self.internalcs, self.value
    )
  }
}
impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}
/// The only purpose of RegisterCell is to provide us with a place to implement fmt::Display over
/// a `RefCell<Register>`.
pub struct RegisterCell(RwLock<Register>);
impl fmt::Debug for RegisterCell {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0.read().unwrap()) }
}
impl fmt::Display for RegisterCell {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0.read().unwrap()) }
}
impl Object for RegisterCell {
  fn stringify(&self) -> String { Definition::stringify_type(self, "RegisterCell") }
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}
impl PartialEq for RegisterCell {
  fn eq(&self, other: &RegisterCell) -> bool { *self.0.read().unwrap() == *other.0.read().unwrap() }
}
impl RegisterCell {
  pub fn new(cell: RwLock<Register>) -> Self { RegisterCell(cell) }
  pub fn borrow(&self) -> RwLockReadGuard<Register> { self.0.read().unwrap() }
  pub fn borrow_mut(&self) -> RwLockWriteGuard<Register> { self.0.write().unwrap() }
}
impl Definition for RegisterCell {
  fn is_register(&self) -> bool { true }
  fn is_prefix(&self) -> bool { false }
  fn is_readonly(&self) -> bool { self.borrow().readonly }
  // not implemented for primitives
  fn invoke(&self, gullet: &mut Gullet, _once_only: bool, state: &mut State) -> Result<Tokens> { unimplemented!() }
  fn get_parameters(&self) -> Option<&Parameters> { unimplemented!() } // TODO: How do we do this with a RefCell ?!
  fn get_cs(&self) -> Cow<Token> { Cow::Owned(self.borrow().cs.clone()) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Owned(self.borrow().cs.get_cs_name().to_owned()) }
  fn get_alias(&self) -> Option<&String> { None }
  // No before/after daemons ???
  // (other than afterassign)
  fn invoke_primitive(&self, stomach: &mut Stomach, _caller: Arc<dyn Definition>, state: &mut State) -> Result<Vec<Digested>> {
    // CharDef case
    if self.borrow().register_type == RegisterType::CharDef {
      let internalcs = &self.borrow().internalcs;
      return match internalcs {
        // Tracing ?
        None => Ok(Vec::new()),
        Some(ref cs) => stomach.invoke_token(cs, state),
      };
    }

    // my $profiled = $STATE->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    let gullet = stomach.get_gullet_mut();
    let args = self.read_arguments(gullet, state)?;
    gullet.read_keyword(&["="], state)?;
    let value = gullet.read_value(self.register_type().unwrap(), state)?;

    self.borrow_mut().set_value(value, args, state);

    state.after_assignment(gullet);
    // # Tracing ?
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;

    Ok(Vec::new())
  }

  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn read_arguments(&self, gullet: &mut Gullet, state: &mut State) -> Result<Vec<ArgWrap>> {
    let params = &self.borrow().parameters;
    match params {
      None => Ok(Vec::new()),
      Some(ref params) => params.read_arguments(gullet, self, state),
    }
  }

  fn do_absorbtion(&self, _document: &mut Document, _whatsit: &Whatsit, _state: &mut State) -> Result<()> {
    fatal!(Definition, Unexpected, "do_absorbtion on Primitive should never be called!");
  }
  fn value_of(&self, args: Vec<ArgWrap>, state: &State) -> Option<RegisterValue> {
    if self.borrow().register_type == RegisterType::CharDef {
      self.borrow().value.clone()
    } else {
      (self.borrow().getter)(args, state)
    }
  }
  fn register_type(&self) -> Option<RegisterType> { Some(self.borrow().register_type) }
}

impl Register {
  pub fn is_readonly(&self) -> bool { self.readonly }
  pub fn set_value(&mut self, value: RegisterValue, args: Vec<ArgWrap>, state: &mut State) {
    if self.register_type == RegisterType::CharDef {
      let message = s!("Can't assign to chardef {}", self.cs.get_cs_name());
      Error!("unexpected", "chardef", None, state, message);
    } else {
      (self.setter)(value, args, state);
    }
  }

  pub fn new_chardef(cs: Token, value: Option<RegisterValue>, internalcs: Option<Token>) -> Self {
    Register {
      cs,
      parameters: None,
      value,
      internalcs,
      register_type: RegisterType::CharDef,
      readonly: true,
      //locator => $STATE->getStomach->getGullet->getMouth->getLocator,
      ..Register::default()
    }
  }
}

//===============================================================================
