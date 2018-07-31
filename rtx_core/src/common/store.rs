use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

use common::font::Font;
use common::number::Number;
use definition::conditional::Conditional;
use definition::constructor::Constructor;
use definition::expandable::Expandable;
use definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use definition::primitive::Primitive;
use definition::register::Register;
use document::tag::TagData;
use parameter::Parameter;
use token::{Catcode, Token};
use tokens::Tokens;

// TODO: Some design decisions need to be finalzed w.r.t Stored
// 1. which types are allowed,
// 2. should the store be made generic? e.g. T:Clone ? I would lean towards no, since it is very
// easy to mistakenly insert the wrongly wrapped data e.g. Stored(Primitive(x)) vs
// Stored(Rc(Primitive(x))) ... best to enforce explicitly 3. consistent type principles - should
// we always demand an Rc<T> for a complex struct T? do we ever want to mutate the values in
// place? if so, would also need to consider a RefCell<> if not (which is what I lean to), a
// simple Box<> may suffice, HOWEVER, there are cases where definitions need to be passed on
// and shared, and especially so for Font      in which case an Rc<()> may make more sense.
// 4. API ergonomics - we need From/Into implementations for ALL enum variants, to avoid explicitly
//    writing a wrapper every time. if there is a primitive p, it should be accepted as an argument,
//    rather than requiring writing Stored::Primitive(Rc::new(p)) each time
//    and equally importantly for unwrapping.

// Basic principles:
// 1. If the type is `Copy`, store directly
// 2. If the type is intended as State-exclusive, store in a Box
//      (or directly if any already Boxed datatype such as Vec, VecDeque)
// 3. If the struct is intended for reuse/(mutation?!) in digestion components, store it in an Rc,
//    e.g. Rc<Font>

#[derive(Clone, PartialEq)]
pub enum Stored {
  // Primitives (Copy types, or cheap Clone)
  Bool(bool),
  String(String),
  Mathcode(usize),
  Int(i32),
  // Collections (boxed)
  VecChar(Vec<char>),
  VecString(Vec<String>),
  VecToken(Vec<Token>),
  VecDigested(Vec<::Digested>),
  HashStr(HashMap<String, String>),
  VecDequeStored(VecDeque<Stored>),
  HashStored(HashMap<String, Stored>),
  HashTagData(HashMap<String, Vec<TagData>>),
  // LaTeXML primitives (Copy types)
  Catcode(Catcode),
  Token(Token),
  Tokens(Tokens),
  Number(Number),
  // LaTeXML objects (Rc-wrapped)
  Expandable(Rc<Expandable>),
  Conditional(Rc<Conditional>),
  Primitive(Rc<Primitive>),
  MathPrimitive(Rc<MathPrimitive>),
  Register(Rc<Register>),
  // MathPrimitiveOptions(MathPrimitiveOptions), // Maybe later
  Constructor(Rc<Constructor>),
  Digested(Rc<::Digested>),
  Parameter(Parameter),
  Font(Rc<Font>),
}

impl fmt::Debug for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Stored::*;
    match *self {
      String(ref s) => write!(f, "{}", s),
      Int(ref num) => write!(f, "{}", num),
      VecChar(ref vs) => write!(f, "{:?}", vs),
      VecString(ref vs) => write!(f, "{:?}", vs),
      Bool(ref b) => write!(f, "{:?}", b),
      Token(ref t) => write!(f, "{:?}", t),
      Tokens(ref t) => write!(f, "{:?}", t),
      Catcode(ref cc) => write!(f, "{:?}", cc),
      Mathcode(ref cc) => write!(f, "{:?}", cc),
      Expandable(ref _expandable) => write!(f, "<closure for expandable definition>"),
      Conditional(ref _conditional) => write!(f, "<closure for conditional definition>"),
      Primitive(ref _primitive) => write!(f, "<closure for primitive definition>"),
      MathPrimitive(ref _primitive) => write!(f, "<closure for math primitive definition>"),
      // MathPrimitiveOptions(ref _primitive) => write!(f, "<math primitive options>"),
      Constructor(ref _constructor) => write!(f, "<closure for constructor definition>"),
      Digested(ref digested) => write!(f, "{:?}", digested),
      Parameter(ref parameter) => write!(f, "{:?}", parameter),
      Register(ref register) => write!(f, "{:?}", register),
      Font(ref font) => write!(f, "{:?}", font),
      Number(ref number) => write!(f, "{:?}", number),
      VecToken(ref token_vec) => write!(f, "{:?}", token_vec),
      VecDigested(ref digested_vec) => write!(f, "{:?}", digested_vec),
      VecDequeStored(ref vec) => write!(f, "VecDequeStored({:?})", vec),
      HashStored(ref hos) => write!(f, "HashStored({:?})", hos),
      HashTagData(ref htd) => write!(f, "HashTagData({:?})", htd),
      HashStr(ref hstr) => write!(f, "HashStr({:?})", hstr),
    }
  }
}
impl fmt::Display for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

impl From<bool> for Stored {
  fn from(value: bool) -> Self { Stored::Bool(value) }
}

impl From<String> for Stored {
  fn from(value: String) -> Self { Stored::String(value) }
}

impl From<i32> for Stored {
  fn from(value: i32) -> Self { Stored::Int(value) }
}

impl From<Catcode> for Stored {
  fn from(value: Catcode) -> Self { Stored::Catcode(value) }
}

impl From<Token> for Stored {
  fn from(value: Token) -> Self { Stored::Token(value) }
}

// Storing all definitions is expected - Rc<Expandable> case

impl From<Tokens> for Stored {
  fn from(value: Tokens) -> Self { Stored::Tokens(value) }
}

impl From<Rc<Expandable>> for Stored {
  fn from(definition: Rc<Expandable>) -> Self { Stored::Expandable(definition) }
}
/// Storing all definitions is expected - Expandable case
impl From<Expandable> for Stored {
  fn from(definition: Expandable) -> Self { Rc::new(definition).into() }
}

impl From<Rc<Conditional>> for Stored {
  fn from(definition: Rc<Conditional>) -> Self { Stored::Conditional(definition) }
}
impl From<Conditional> for Stored {
  fn from(value: Conditional) -> Self { Rc::new(value).into() }
}

impl From<Rc<Primitive>> for Stored {
  fn from(definition: Rc<Primitive>) -> Self { Stored::Primitive(definition) }
}
impl From<Primitive> for Stored {
  fn from(value: Primitive) -> Self { Rc::new(value).into() }
}

impl From<Rc<MathPrimitive>> for Stored {
  fn from(definition: Rc<MathPrimitive>) -> Self { Stored::MathPrimitive(definition) }
}
impl From<MathPrimitive> for Stored {
  fn from(value: MathPrimitive) -> Self { Rc::new(value).into() }
}

impl From<Rc<Constructor>> for Stored {
  fn from(definition: Rc<Constructor>) -> Self { Stored::Constructor(definition) }
}
impl From<Constructor> for Stored {
  fn from(value: Constructor) -> Self { Rc::new(value).into() }
}

impl From<Rc<::Digested>> for Stored {
  fn from(definition: Rc<::Digested>) -> Self { Stored::Digested(definition) }
}
impl From<::Digested> for Stored {
  fn from(value: ::Digested) -> Self { Rc::new(value).into() }
}

impl From<Parameter> for Stored {
  fn from(value: Parameter) -> Self { Stored::Parameter(value) }
}

impl From<Rc<Font>> for Stored {
  fn from(definition: Rc<Font>) -> Self { Stored::Font(definition) }
}
impl From<Font> for Stored {
  fn from(value: Font) -> Self { Rc::new(value).into() }
}

impl From<Number> for Stored {
  fn from(value: Number) -> Self { Stored::Number(value) }
}

impl<'a> From<&'a Token> for Stored {
  fn from(value: &'a Token) -> Self { Stored::Token(value.clone()) }
}

impl From<Vec<char>> for Stored {
  fn from(value: Vec<char>) -> Self { Stored::VecChar(value) }
}

impl From<Vec<String>> for Stored {
  fn from(value: Vec<String>) -> Self { Stored::VecString(value) }
}

impl From<Vec<Token>> for Stored {
  fn from(value: Vec<Token>) -> Self { Stored::VecToken(value) }
}

impl From<Vec<::Digested>> for Stored {
  fn from(value: Vec<::Digested>) -> Self { Stored::VecDigested(value) }
}

impl From<HashMap<String, String>> for Stored {
  fn from(value: HashMap<String, String>) -> Self { Stored::HashStr(value) }
}

impl From<VecDeque<Stored>> for Stored {
  fn from(value: VecDeque<Stored>) -> Self { Stored::VecDequeStored(value) }
}

impl From<HashMap<String, Stored>> for Stored {
  fn from(value: HashMap<String, Stored>) -> Self { Stored::HashStored(value) }
}

impl From<HashMap<String, Vec<TagData>>> for Stored {
  fn from(value: HashMap<String, Vec<TagData>>) -> Self { Stored::HashTagData(value) }
}

// Reverse direction -- cast Stored back into concrete types, with meaningfull fallbacks where
// impossible

impl<'a> From<&'a Stored> for bool {
  fn from(value: &Stored) -> bool {
    match value {
      Stored::Bool(b) => *b,
      _ => true,
    }
  }
}

impl<'a> From<&'a Stored> for String {
  fn from(value: &Stored) -> String {
    match value {
      &Stored::String(ref v) => v.to_owned(),
      v => s!("{:?}", v),
    }
  }
}

impl<'a> From<&'a Stored> for Option<&'a VecDeque<Stored>> {
  fn from(value: &'a Stored) -> Option<&'a VecDeque<Stored>> {
    match value {
      Stored::VecDequeStored(ref v) => Some(v),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Rc<Font>> {
  fn from(value: &'a Stored) -> Option<Rc<Font>> {
    match value {
      Stored::Font(ref f) => Some(f.clone()),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Number> {
  fn from(value: &'a Stored) -> Option<Number> {
    match value {
      Stored::Number(ref n) => Some(*n),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Tokens> {
  fn from(value: &'a Stored) -> Option<Tokens> {
    match value {
      Stored::Tokens(ref ts) => Some(ts.clone()),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Catcode> {
  fn from(value: &'a Stored) -> Option<Catcode> {
    match value {
      Stored::Catcode(ref cc) => Some(*cc),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<&'a Vec<char>> {
  fn from(value: &'a Stored) -> Option<&'a Vec<char>> {
    match value {
      Stored::VecChar(ref cc) => Some(cc),
      _ => None,
    }
  }
}
