use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::cell::{Ref, RefMut, RefCell};
use std::rc::Rc;
use std::fmt;

use crate::common::dimension::{Dimension, MuDimension};
use crate::common::error::*;
use crate::common::glue::{Glue, MuGlue};
use crate::common::number::Number;
use crate::common::object::Object;
use crate::common::store::Stored;
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

lazy_static! {
  static ref SPEC_RE: Regex = Regex::new(r"^(-?\d*\.?\d*)([a-zA-Z][a-zA-Z])$").unwrap();
}

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

impl Default for RegisterValue {
  fn default() -> Self { RegisterValue::Number(Number::new(0.0)) }
}
impl Object for RegisterValue {
  fn stringify(&self) -> String { s!("RegisterValue[{}]", self) }
  fn revert(&self) -> Result<Tokens> {
    match self { // ExplodeText($self->toString);
      RegisterValue::Number(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::Dimension(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::MuDimension(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::Glue(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::MuGlue(ref value) => Ok(Tokens::new(ExplodeText!(value))),
      RegisterValue::Token(ref value) => Ok(Tokens!(value.revert())),
      RegisterValue::Tokens(ref value) => Ok(Tokens::new(value.clone().revert())), // clone?
    }
  }
}

const SCALES: &[f32] = &[1.0, 10.0, 100.0, 1000.0, 10000.0, 100_000.0];
// smallest number that makes a difference added to 1 in Rust's float format.
// my $EPSILON = 1.0;
// while (1.0 + $EPSILON / 2 != 1) {
//   $EPSILON /= 2.0; }
const EPSILON: f32 = 0.000_000_119_209_29;

/// Round $number to $prec decimals (0...6) attempting to do so portably.
pub fn round_to(number: f32, prec_opt: Option<u8>) -> f32 {
  let mut prec = prec_opt.unwrap_or(2);
  if prec > 5 {
    prec = 5;
  }
  let scale = SCALES[prec as usize];
  // scale to integer, w/some slop in case arbitrarily close to an integer...
  let n = number * scale * (1.0 + 100.0 * EPSILON);
  let adjusted: f32 = if n < -EPSILON {
    n - 0.5
  } else if n > EPSILON {
    n + 0.5
  } else {
    0.0
  };
  adjusted.floor() / scale
}

pub trait NumericOps {
  fn new<T: Into<f32>>(number: T) -> Self;
  fn value_of(self) -> f32;
  fn pt_value(self, prec: Option<u8>) -> f32
  where Self: Sized {
    round_to(self.value_of() / 65536.0, prec)
  }
  fn add<T: NumericOps>(self, other: T) -> Self
  where Self: Sized {
    Self::new(self.value_of() + other.value_of())
  }
  fn negate(self) -> Self
  where Self: Sized {
    let value = self.value_of();
    if value > 0.0 {
      Self::new(-value)
    } else {
      Self::new(value)
    }
  }
  fn multiply<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() * other).floor())
  }
  fn divide<T: Into<f32>>(self, other: T) -> Self
  where Self: Sized {
    let other: f32 = other.into();
    Self::new((self.value_of() / other).floor())
  }

  fn spec_to_f32(spec: &str, state: &State) -> Result<f32> {
    if spec.is_empty() {
      Ok(0.0)
    } else if let Some(cap) = SPEC_RE.captures(spec) {
      // Dimensions given.
      let num_str = cap.get(1).map_or(String::new(), |m| m.as_str().to_string());
      let num: f32 = num_str.parse::<f32>()?;
      let unit = cap.get(2).map_or(String::new(), |m| m.as_str().to_string());
      Ok(num * state.convert_unit(unit))
    } else {
      Ok(0.0)
    }
  }
  fn to_token(self) -> Token
  where Self: Sized {
    T_OTHER!(self.value_of().to_string())
  }
  // dancing around meta-programming in the Glue case... is there a better way?
  fn to_glue_type(self) -> Glue
  where Self: Sized {
    unimplemented!()
  }
  fn register_type(&self) -> RegisterType;
}

impl NumericOps for RegisterValue {
  fn new<T: Into<f32>>(number: T) -> Self { RegisterValue::Number(Number::new(number)) }
  fn value_of(self) -> f32 {
    match self {
      RegisterValue::Number(v) => v.value_of(),
      RegisterValue::Dimension(v) => v.value_of(),
      RegisterValue::MuDimension(v) => v.value_of(),
      RegisterValue::Glue(v) => v.value_of(),
      RegisterValue::MuGlue(v) => v.value_of(),
      RegisterValue::Token(v) => {
        let message = s!(".value_of called on Token {:?}", v);
        Warn!("register","value_of", None, None, message);
        -1.0
      },
      RegisterValue::Tokens(v) => {
        let message = s!(".value_of called on Tokens {:?}", v);
        Warn!("register","value_of", None, None, message);
        -1.0
      },
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
  /// For now only meant as a type cast, unimplemented in other cases
  /// DO NOT use this method to cast into a Glue object, define a `.to_glue()` instead
  fn to_glue_type(self) -> Glue {
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
  fn from(v: RegisterValue) -> Number { (&v).into() }
}
impl From<RegisterValue> for Dimension {
  fn from(v: RegisterValue) -> Dimension { (&v).into() }
}
impl From<RegisterValue> for Glue {
  fn from(v: RegisterValue) -> Glue { (&v).into() }
}
impl From<RegisterValue> for f32 {
  fn from(v: RegisterValue) -> f32 { v.value_of() }
}

impl From<Number> for Dimension {
  fn from(n: Number) -> Dimension { Dimension::new(n.value_of()) }
}
impl From<Number> for Glue {
  fn from(n: Number) -> Glue { Glue::new(n.value_of()) }
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
        Error!("expected","dimension", None, None, message);
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
      RegisterValue::MuDimension(other) => Glue::new(other.value_of()),
      RegisterValue::MuGlue(other) => Glue::new(other.value_of()),
      RegisterValue::Token(other) => other.to_number().into(),
      RegisterValue::Tokens(other) => {
        let message = s!("Token register can not be cast into a Glue: {:?}", other);
        Error!("expected","dimension", None, None, message);
        Glue::new(0.0)
      },
    }
  }
}

impl fmt::Display for RegisterValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegisterValue::Dimension(d) => write!(f,"{}", d),
      RegisterValue::Glue(g) => write!(f,"{}", g),
      other => write!(f,"{}", self.clone().value_of()),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

/// The only purpose of RegisterCell is to provide us with a place to implement fmt::Display over
/// a `RefCell<Register>`.
#[derive(PartialEq)]
pub struct RegisterCell(RefCell<Register>);
impl fmt::Display for RegisterCell {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unimplemented!();
  }
}
impl Object for RegisterCell {
  fn stringify(&self) -> String { Definition::stringify_type(self, "RegisterCell") }
}
impl RegisterCell {
  pub fn new(cell: RefCell<Register>) -> Self { RegisterCell(cell) }
  pub fn borrow(&self) -> Ref<Register> { self.0.borrow() }
  pub fn borrow_mut(&self) -> RefMut<Register> { self.0.borrow_mut() }
}
impl Definition for RegisterCell {
  fn is_register(&self) -> bool { true }
  fn is_prefix(&self) -> bool { false }
  fn is_readonly(&self) -> bool { self.borrow().readonly }
  // not implemented for primitives
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> { unimplemented!() }
  fn get_parameters(&self) -> Option<&Parameters> { unimplemented!() } // TODO: How do we do this with a RefCell ?!
  fn get_cs(&self) -> Cow<Token> { Cow::Owned(self.borrow().cs.clone()) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Owned(self.borrow().cs.get_cs_name().to_owned()) }
  fn get_alias(&self) -> Option<&String> { None }
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
    gullet.read_keyword(&["="], state)?;
    let value = gullet.read_value(self.register_type().unwrap(), state)?;

    self.borrow_mut().set_value(value, args, state);

    if let Some(after) = state.remove_value("afterAssignment") {
      match after {
        // primitive returns boxes, so these need to be digested!
        Stored::Token(t) => gullet.unread(Tokens!(t)),
        Stored::Tokens(tks) => gullet.unread(tks),
        other => {
          let message = s!("expected tokens, found: {:?}", other);
          Error!("unexpected","afterassignment", stomach, state, message)
        }
      };
    }
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
      let message = s!("Can't assign to chardef {}", self.cs.get_cs_name());
      Error!("unexpected","chardef", None, state, message);
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
