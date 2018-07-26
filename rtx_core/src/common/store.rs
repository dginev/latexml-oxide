use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::rc::Rc;

use common::font::Font;
use common::number::Number;
use definition::conditional::Conditional;
use definition::constructor::Constructor;
use definition::expandable::Expandable;
use definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use definition::primitive::Primitive;
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

#[derive(Clone, PartialEq)]
pub enum Stored {
  // Primitives
  Bool(bool),
  String(String),
  Mathcode(usize),
  Int(i32),
  // LaTeXML objects
  Catcode(Catcode),
  Token(Token),
  Tokens(Tokens),
  Expandable(Rc<Expandable>),
  Conditional(Rc<Conditional>),
  Primitive(Rc<Primitive>),
  MathPrimitive(Rc<MathPrimitive>),
  // MathPrimitiveOptions(MathPrimitiveOptions), // Maybe later
  Constructor(Rc<Constructor>),
  Digested(Rc<::Digested>),
  Parameter(Parameter),
  Font(Rc<Font>),
  Number(Number),
  // Collections
  VecChar(Vec<char>),
  VecString(Vec<String>),
  VecToken(Vec<Token>),
  VecDigested(Vec<::Digested>),
  HashStr(HashMap<String, String>),
  VecDequeOS(VecDeque<Stored>),
  HashOS(HashMap<String, Stored>),
  HashTagData(HashMap<String, Vec<TagData>>),
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
      Font(ref font) => write!(f, "{:?}", font),
      Number(ref number) => write!(f, "{:?}", number),
      VecToken(ref token_vec) => write!(f, "{:?}", token_vec),
      VecDigested(ref digested_vec) => write!(f, "{:?}", digested_vec),
      VecDequeOS(ref vec) => write!(f, "VecDequeOS({:?})", vec),
      HashOS(ref hos) => write!(f, "HashOS({:?})", hos),
      HashTagData(ref htd) => write!(f, "HashTagData({:?})", htd),
      HashStr(ref hstr) => write!(f, "HashStr({:?})", hstr),
    }
  }
}
impl fmt::Display for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}
