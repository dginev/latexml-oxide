use libxml::tree::Node;
use std::borrow::Borrow;
use std::borrow::Cow;
use std::fmt;
use std::rc::Rc;
use rustc_hash::FxHashMap as HashMap;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::glue::Glue;
use crate::common::font;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::arena;
use crate::definition::{BeforeDigestClosure, Definition, DigestionClosure};
use crate::document::Document;
use crate::parameter::Parameters;
use crate::state::{Scope};
use crate::tbox::Tbox;
use crate::gullet;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{Digested, Locator, state};

use super::argument::ArgWrap;

/// The values that can be read by, and stored in, a Register
#[derive(Clone, PartialEq)]
pub enum RegisterValue {
  ///
  Number(Number),
  ///
  Dimension(Dimension),
  ///
  MuDimension(MuDimension),
  ///
  Glue(Glue),
  ///
  MuGlue(MuGlue),
  ///
  Token(Token),
  ///
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

impl Default for RegisterValue {
  fn default() -> Self { RegisterValue::Number(Number::new(0)) }
}
impl Object for RegisterValue {
  fn stringify(&self) -> String { s!("RegisterValue[{}]", self) }

  fn revert(&self) -> Result<Tokens> {
    match self {
      // ExplodeText($self->toString);
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

impl NumericOps for RegisterValue {
  fn new(number: i64) -> Self { RegisterValue::Number(Number::new(number)) }
  fn new_f64(number: f64) -> Self { RegisterValue::Number(Number::new_f64(number)) }
  fn value_of(self) -> i64 {
    match self {
      RegisterValue::Number(v) => v.value_of(),
      RegisterValue::Dimension(v) => v.value_of(),
      RegisterValue::MuDimension(v) => v.value_of(),
      RegisterValue::Glue(v) => v.value_of(),
      RegisterValue::MuGlue(v) => v.value_of(),
      RegisterValue::Token(v) => {
        let message = s!(".value_of called on Token {:?}", v);
        Warn!("register", "value_of", message);
        -1
      },
      RegisterValue::Tokens(v) => {
        let message = s!(".value_of called on Tokens {:?}", v);
        Warn!("register", "value_of", message);
        -1
      },
    }
  }
  fn register_type(&self) -> RegisterType {
    match self {
      RegisterValue::Number(_) => RegisterType::Number,
      RegisterValue::Dimension(_) => RegisterType::Dimension,
      RegisterValue::MuDimension(_) => RegisterType::MuDimension,
      RegisterValue::Glue(_) => RegisterType::Glue,
      RegisterValue::MuGlue(_) => RegisterType::MuGlue,
      RegisterValue::Token(_) => RegisterType::Token,
      RegisterValue::Tokens(_) => RegisterType::Tokens,
    }
  }
  fn add<T: NumericOps>(self, other: T) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.add(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.add(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.add(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.add(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.add(other)),
      RegisterValue::Token(_v) => unimplemented!(),
      RegisterValue::Tokens(_v) => unimplemented!(),
    }
  }
  fn subtract<T: NumericOps>(self, other: T) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.subtract(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.subtract(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.subtract(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.subtract(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.subtract(other)),
      RegisterValue::Token(_v) => unimplemented!(),
      RegisterValue::Tokens(_v) => unimplemented!(),
    }
  }
  fn multiply<T: NumericOps>(self, other: T) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.multiply(other)),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.multiply(other)),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.multiply(other)),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.multiply(other)),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.multiply(other)),
      RegisterValue::Token(_v) => unimplemented!(),
      RegisterValue::Tokens(_v) => unimplemented!(),
    }
  }
  fn negate(self) -> Self {
    match self {
      RegisterValue::Number(v) => RegisterValue::Number(v.negate()),
      RegisterValue::Dimension(v) => RegisterValue::Dimension(v.negate()),
      RegisterValue::MuDimension(v) => RegisterValue::MuDimension(v.negate()),
      RegisterValue::Glue(v) => RegisterValue::Glue(v.negate()),
      RegisterValue::MuGlue(v) => RegisterValue::MuGlue(v.negate()),
      RegisterValue::Token(_v) => unimplemented!(),
      RegisterValue::Tokens(_v) => unimplemented!(),
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
      RegisterValue::Token(_v) => unimplemented!(),
      RegisterValue::Tokens(_v) => unimplemented!(),
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
  fn to_attribute(&self) -> String
  where Self: fmt::Display {
    match self {
      RegisterValue::Number(v) => v.to_attribute(),
      RegisterValue::Dimension(v) => v.to_attribute(),
      RegisterValue::MuDimension(v) => v.to_attribute(),
      RegisterValue::Glue(v) => v.to_attribute(),
      RegisterValue::MuGlue(v) => v.to_attribute(),
      // Token, Tokens?
      other => other.to_string(),
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
impl From<RegisterValue> for MuGlue {
  fn from(v: RegisterValue) -> MuGlue { (&v).into() }
}
impl From<RegisterValue> for f64 {
  fn from(v: RegisterValue) -> f64 { v.value_of() as f64 }
}
impl From<RegisterValue> for i64 {
  fn from(v: RegisterValue) -> i64 { v.value_of() }
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
  fn from(v: RegisterValue) -> Tokens { Tokens!(T_OTHER!(v.value_of().to_string())) }
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
        let message = s!(
          "Token register can not be cast into a dimension: {:?}",
          other
        );
        // silence a potential Fatal from 100 errors,
        // until a better place is reached in the high-level conversion logic
        let err = || {Error!("expected", "dimension", message); Ok(()) };
        err().ok();
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
        let message = s!("Token register can not be cast into a Glue: {other:?}");
        // silence a potential Fatal from 100 errors,
        // until a better place is reached in the high-level conversion logic
        let err = || { Error!("expected", "dimension", message); Ok(()) };
        err().ok();
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
        let err = || {Error!("expected", "dimension", message); Ok(())};
        err().ok();
        MuGlue::new(0)
      },
    }
  }
}

// passthrough the Debug print to the inner value, RegisterValue is transparent
impl fmt::Debug for RegisterValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegisterValue::Number(n) => write!(f, "{n:?}"),
      RegisterValue::Dimension(d) => write!(f, "{d:?}"),
      RegisterValue::MuDimension(d) => write!(f, "{d:?}"),
      RegisterValue::Glue(g) => write!(f, "{g:?}"),
      RegisterValue::MuGlue(g) => write!(f, "{g:?}"),
      RegisterValue::Tokens(t) => write!(f, "{t:?}"),
      RegisterValue::Token(t) => write!(f, "{t:?}"),
    }
  }
}

// passthrough the Display print to the inner value, RegisterValue is transparent
impl fmt::Display for RegisterValue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegisterValue::Number(n) => write!(f, "{n}"),
      RegisterValue::Dimension(d) => write!(f, "{d}"),
      RegisterValue::MuDimension(d) => write!(f, "{d}"),
      RegisterValue::Glue(g) => write!(f, "{g}"),
      RegisterValue::MuGlue(g) => write!(f, "{g}"),
      RegisterValue::Tokens(t) => write!(f, "{t}"),
      RegisterValue::Token(t) => write!(f, "{t}"),
    }
  }
}

/// The type of a TeX Register
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegisterType {
  /// simple scalar number
  Number,
  /// a TeX dimension
  Dimension,
  /// a TeX mu dimension
  MuDimension,
  /// a TeX glue
  Glue,
  /// a TeX mu glue
  MuGlue,
  /// a TeX Token
  Token,
  /// multiple Tokens
  Tokens,
  /// a character definition
  CharDef,
  /// Placeholder for any argument accepted
  Any,
}

/// looks up a stored value from the state frame (at a constant key, or key based on the arguments)
pub type RegisterGetterClosure = Rc<dyn Fn(Vec<ArgWrap>) -> Option<RegisterValue>>;
/// sets a register value in the state::frame
pub type RegisterSetterClosure = Rc<dyn Fn(RegisterValue, Option<Scope>, Vec<ArgWrap>)>;

/// A struct representing a TeX register
#[derive(Clone)]
pub struct Register {
  /// the public command sequence for this register
  pub cs: Token,
  /// the internal address for this register
  pub address: String,
  /// associated parameters, if any
  pub parameters: Option<Parameters>,
  /// the type of values accepted by this register (Number, Dimension, ...)
  pub register_type: RegisterType,
  /// read-only flag (default: false)
  pub readonly: bool,
  /// the current value
  pub value: Option<RegisterValue>,
  /// reader for a value
  pub getter: Option<RegisterGetterClosure>,
  /// setter for a value
  pub setter: Option<RegisterSetterClosure>,
  /// a default value
  pub default: Option<RegisterValue>,
  /// the unicode corresponding to the \mathchar of `value` (for chardef)
  pub mathglyph: Option<char>,
  /// the source point of origin for this register definition
  pub locator: Locator
}
impl Default for Register {
  fn default() -> Self {
    Register {
      cs: T_CS!("Register"),
      address: String::from("Register"),
      locator: Locator::default(),
      parameters: None,
      register_type: RegisterType::Number,
      getter: None,
      setter: None,
      readonly: false,
      value: None,
      mathglyph: None,
      default: None
    }
  }
}
impl PartialEq for Register {
  fn eq(&self, other: &Register) -> bool {
    self.register_type == other.register_type
      && self.parameters == other.parameters
      && self.value == other.value
      && self.address == other.address
  }
}
impl fmt::Debug for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Register[cs:{:?}, address:{:?}, parameters:{:?}, type:{:?}, readonly:{:?}, value:{:?}, default:{:?}]",
      self.cs, self.address, self.parameters, self.register_type, self.readonly, self.value,
      self.default,
    )
  }
}
impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{self:?}") }
}

impl Object for Register {
  fn stringify(&self) -> String { format!("{self:?}") }

}

impl Definition for Register {
  fn is_register(&self) -> bool { true }
  fn is_prefix(&self) -> bool { false }
  fn is_readonly(&self) -> bool { self.readonly }
  // not implemented for primitives
  fn invoke(&self, _once_only: bool) -> Result<Tokens> {
    unimplemented!()
  }
  fn get_parameters(&self) -> Option<&Parameters> { unimplemented!() }
  fn get_cs(&self) -> Cow<Token> { Cow::Owned(self.cs) }
  fn get_cs_name(&self) -> Cow<str> { Cow::Owned(self.cs.with_cs_name(ToString::to_string)) }
  fn get_alias(&self) -> Option<&String> { None }

  fn set_value(&self, value: RegisterValue, scope: Option<Scope>, args: Vec<ArgWrap>) {
    if self.register_type == RegisterType::CharDef {
      let message = self
        .cs
        .with_cs_name(|cs_str| s!("Can't assign to chardef {}", cs_str));
      let err = || {Error!("unexpected", "chardef", message); Ok(()) };
      err().ok();
    } else if let Some(setter) = &self.setter {
      setter(value, scope, args);
    } else {
      // default setter
      if self.readonly {
        let message = s!("Can't assign to register {}", self.address);
        Warn!("unexpected", self.address, message);
      } else {
        let loc = if args.is_empty() { Cow::Borrowed(&self.address) } else {
          let args_string: String = args
            .into_iter()
            .map(|a| {
              a.as_tokens()
               .expect("TODO: handle malformed values here.")
               .unwrap()
               .to_string()
           })
           .collect::<Vec<String>>()
           .join("");
          Cow::Owned(format!("{}{args_string}",self.address))
        };
        state::assign_value(&loc, value, scope);
      }
    }
  }
  // No before/after daemons ???
  // (other than afterassign)
  fn invoke_primitive(&self) -> Result<Vec<Digested>> {
    // CharDef case
    if self.register_type == RegisterType::CharDef {
      return Ok(vec![Digested::from(
        Tbox::new(arena::pin_char(font::decode(self.value.clone().unwrap().value_of() as u8, None,false).unwrap()), None, None,
          Tokens!(T_CS!("\\char"), self.value.as_ref().unwrap().revert()?, T_CS!("\\relax")), HashMap::default()))]);
    }

    // my $profiled = $state->lookupValue('PROFILING') && ($LaTeXML::CURRENT_TOKEN || $$self{cs});
    // LaTeXML::Core::Definition::startProfiling($profiled, 'digest') if $profiled;
    let args = self.read_arguments()?;
    gullet::read_keyword(&["="])?;
    let value = gullet::read_value(self.register_type().unwrap())?;

    self.borrow().set_value(value, None, args);

    state::after_assignment();
    // # Tracing ?
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'digest') if $profiled;

    Ok(Vec::new())
  }

  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn read_arguments(&self) -> Result<Vec<ArgWrap>> {
    let params = &self.parameters;
    match params {
      None => Ok(Vec::new()),
      Some(ref params) => params.read_arguments(Some(self)),
    }
  }

  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
  ) -> Result<Vec<Node>> {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Primitive should never be called!"
    );
  }
  fn value_of(&self, args: Vec<ArgWrap>) -> Option<RegisterValue> {
    if self.register_type == RegisterType::CharDef {
      self.value.clone()
    } else if let Some(ref getter) = self.getter {
      getter(args)
    } else {
      let key = if args.is_empty() {
        Cow::Borrowed(&self.address)
      } else {
        let args_string: String = args
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join("");
        Cow::Owned(format!("{}{args_string}",self.address))
      };
      state::with_value(&key,|v_opt| match v_opt {
        Some(v) => v.into(),
        None => self.default.clone(),
      })
    }
  }
  fn register_type(&self) -> Option<RegisterType> { Some(self.register_type) }
}

impl Register {
  /// checks the readonly flag
  pub fn is_readonly(&self) -> bool { self.readonly }
  /// creates a CharDef type register
  pub fn new_chardef(cs: Token, value: Option<RegisterValue>, mathglyph:Option<char>) -> Self {
    Register {
      cs,
      parameters: None,
      value,
      mathglyph,
      register_type: RegisterType::CharDef,
      readonly: true,
      locator: gullet::get_locator(),
      ..Register::default()
    }
  }

  pub fn get_address(&self) -> Cow<str> {
    if self.address.is_empty() {
      self.get_cs_name()
    } else {
      Cow::Borrowed(&self.address)
    }
  }
}

//===============================================================================
