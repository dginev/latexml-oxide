use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

use common::dimension::Dimension;
use common::font::Font;
use common::glue::{Glue, MuGlue};
use common::number::Number;
use definition::conditional::{Conditional, IfFrame};
use definition::constructor::Constructor;
use definition::expandable::Expandable;
use definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use definition::primitive::Primitive;
use definition::register::{Register, RegisterValue};
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
  Glue(Glue),
  MuGlue(MuGlue),
  Dimension(Dimension),
  // LaTeXML objects (Rc-wrapped)
  Expandable(Rc<Expandable>),
  Conditional(Rc<Conditional>),
  Primitive(Rc<Primitive>),
  MathPrimitive(Rc<MathPrimitive>),
  // WALL OF SHAME (interior mutability)
  Register(Rc<RefCell<Register>>),
  IfFrame(Rc<RefCell<IfFrame>>),
  /////// MathPrimitiveOptions(MathPrimitiveOptions), // Maybe later
  Constructor(Rc<Constructor>),
  Digested(Rc<::Digested>),
  Parameter(Parameter),
  Font(Rc<Font>),
}

impl fmt::Debug for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use Stored::*;
    match *self {
      String(ref s) => write!(f, "{:?}", s),
      Int(ref num) => write!(f, "Stored::Int[{:?}]", num),
      VecChar(ref vs) => write!(f, "Stored::VecChar[{:?}]", vs),
      VecString(ref vs) => write!(f, "Stored::VecString[{:?}]", vs),
      Bool(ref b) => write!(f, "Stored::Bool[{:?}]", b),
      Token(ref t) => write!(f, "Stored::Token[{:?}]", t),
      Tokens(ref t) => write!(f, "Stored::Tokens[{:?}]", t),
      Catcode(ref cc) => write!(f, "Stored::Catcode[{:?}]", cc),
      Mathcode(ref cc) => write!(f, "Stored::Mathcode[{:?}]", cc),
      IfFrame(ref fr) => write!(f, "Stored::IfFrame[{:?}]", fr),
      Expandable(ref _expandable) => write!(f, "Stored::Expandable[]"),
      Conditional(ref _conditional) => write!(f, "Stored::Conditional[]"),
      Primitive(ref _primitive) => write!(f, "Stored::Primitive[]"),
      MathPrimitive(ref _primitive) => write!(f, "Stored::MathPrimitive[]"),
      // MathPrimitiveOptions(ref _primitive) => write!(f, "<math primitive options>"),
      Constructor(ref _constructor) => write!(f, "Stored::Constructor[]"),
      Digested(ref digested) => write!(f, "Stored::Digested[{:?}]", digested),
      Parameter(ref parameter) => write!(f, "Stored::Parameter[{:?}]", parameter),
      Register(ref register) => write!(f, "Stored::Register[{:?}]", register.borrow().cs),
      Font(ref font) => write!(f, "Stored::Font[{:?}]", font),
      Number(ref number) => write!(f, "Stored::Number[{:?}]", number),
      Glue(ref glue) => write!(f, "Stored::Glue[{:?}]", glue),
      MuGlue(ref glue) => write!(f, "Stored::MuGlue[{:?}]", glue),
      Dimension(ref dimension) => write!(f, "Stored::Dimension[{:?}]", dimension),
      VecToken(ref token_vec) => write!(f, "VecToken[{:?}]", token_vec),
      VecDigested(ref digested_vec) => write!(f, "VecDigested[{:?}]", digested_vec),
      VecDequeStored(ref vec) => write!(f, "VecDequeStored[{:?}]", vec),
      HashStored(ref hos) => write!(f, "HashStored[{:?}]", hos),
      HashTagData(ref htd) => write!(f, "HashTagData[{:?}]", htd),
      HashStr(ref hstr) => write!(f, "HashStr[{:?}]", hstr),
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

impl<'a> From<&'a str> for Stored {
  fn from(value: &'a str) -> Self { value.to_owned().into() }
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
  fn from(font: Rc<Font>) -> Self { Stored::Font(font) }
}

impl From<Rc<RefCell<Register>>> for Stored {
  fn from(register: Rc<RefCell<Register>>) -> Self { Stored::Register(register) }
}
impl From<Register> for Stored {
  fn from(register: Register) -> Self { Rc::new(RefCell::new(register)).into() }
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

impl<'a> From<Vec<&'a str>> for Stored {
  fn from(value: Vec<&'a str>) -> Self {
    Stored::VecString(value.iter().map(|x| x.to_string()).collect::<Vec<String>>())
  }
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

impl From<RegisterValue> for Stored {
  fn from(rv: RegisterValue) -> Self {
    match rv {
      RegisterValue::Number(v) => Stored::Number(v),
      RegisterValue::Dimension(v) => Stored::Dimension(v),
      RegisterValue::Glue(v) => Stored::Glue(v),
      RegisterValue::MuGlue(v) => Stored::MuGlue(v),
      RegisterValue::Token(v) => Stored::Token(v),
      RegisterValue::Tokens(v) => Stored::Tokens(v),
    }
  }
}

impl From<Rc<RefCell<IfFrame>>> for Stored {
  fn from(frame: Rc<RefCell<IfFrame>>) -> Stored { Stored::IfFrame(frame) }
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

impl<'a> From<&'a Stored> for Option<Rc<RefCell<Register>>> {
  fn from(value: &'a Stored) -> Option<Rc<RefCell<Register>>> {
    match value {
      Stored::Register(ref reg) => Some(reg.clone()),
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

impl<'a> From<&'a Stored> for Option<RegisterValue> {
  fn from(value: &'a Stored) -> Option<RegisterValue> {
    match value {
      Stored::Number(v) => Some(RegisterValue::Number(v.clone())),
      Stored::Dimension(v) => Some(RegisterValue::Dimension(v.clone())),
      Stored::Glue(v) => Some(RegisterValue::Glue(v.clone())),
      Stored::MuGlue(v) => Some(RegisterValue::MuGlue(v.clone())),
      Stored::Token(v) => Some(RegisterValue::Token(v.clone())),
      Stored::Tokens(v) => Some(RegisterValue::Tokens(v.clone())),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<::Digested> {
  fn from(value: &'a Stored) -> Option<::Digested> {
    match value {
      Stored::Digested(digested) => Some((**digested).clone()),
      _ => None,
    }
  }
}

impl<'a, 'b> From<&'a &'b Stored> for Option<::Digested> {
  fn from(value: &'a &'b Stored) -> Option<::Digested> { (*value).into() }
}

impl<'a> From<&'a Stored> for Token {
  fn from(value: &'a Stored) -> Token {
    match value {
      Stored::Tokens(ts) => ts.into(),
      Stored::Token(t) => (*t).clone(),
      Stored::String(t) => T_CS!(t),
      t => T_CS!(t.to_string()), /* TODO, is this the right place to default to CS? Do we need a
                                  * custom method instead? */
    }
  }
}
