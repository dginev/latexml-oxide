use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, RwLock};

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::glue::Glue;
use crate::common::locator::Locator;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::definition::argument::ArgWrap;
use crate::definition::conditional::{Conditional, IfFrame};
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use crate::definition::primitive::Primitive;
use crate::definition::register::{Register, RegisterCell, RegisterValue};
use crate::definition::FontDirective;
use crate::document::tag::TagData;
use crate::gullet::Gullet;
use crate::ligature::Ligature;
use crate::list::List;
use crate::mouth;
use crate::mouth::Mouth;
use crate::parameter::Parameter;
use crate::rewrite::Rewrite;
use crate::state::{StashTable, State};
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;

const STORED_TRUE: Stored = Stored::Bool(true);
const STORED_FALSE: Stored = Stored::Bool(false);

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
//    rather than requiring writing Stored::Primitive(Arc::new(p)) each time
//    and equally importantly for unwrapping.

// Basic principles:
// 1. If the type is `Copy`, store directly
// 2. If the type is intended as State-exclusive, store in a Box
//      (or directly if any already Boxed datatype such as Vec, VecDeque)
// 3. If the struct is intended for reuse/(mutation?!) in digestion components, store it in an Rc,
//    e.g. Rc<Font>

#[derive(Clone)]
pub enum Stored {
  /// if we want to keep a key but make it 'undef', set it to None
  None,
  // Primitives (Copy types, or cheap Clone)
  Bool(bool),
  String(String),
  Charcode(u16),
  Int(i32),
  // Collections (boxed)
  VecChar(Vec<char>),
  VecOptionChar(Vec<Option<char>>),
  VecString(Vec<String>),
  VecTokens(Vec<crate::Tokens>),
  VecDigested(Vec<crate::Digested>),
  Stash(StashTable),
  HashString(HashMap<String, String>),
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
  MuDimension(MuDimension),
  Locator(Locator),
  Rewrite(Rewrite),
  Ligature(Ligature),
  // LaTeXML objects (Arc-wrapped)
  Expandable(Arc<Expandable>),
  Conditional(Arc<Conditional>),
  Primitive(Arc<Primitive>),
  MathPrimitive(Arc<MathPrimitive>),
  /////// MathPrimitiveOptions(MathPrimitiveOptions), // Maybe later
  Constructor(Arc<Constructor>),
  Digested(Box<crate::Digested>), // todo: should this be an Arc<> to make it shareable?
  Parameter(Arc<Parameter>),
  Font(Arc<Font>),
  FontDirective(FontDirective),
  // WALL OF SHAME (interior mutability) -- can we dispense with these?
  Mouth(Arc<RwLock<Mouth>>),
  Register(Arc<RegisterCell>),
  IfFrame(Arc<RwLock<IfFrame>>),
}

impl fmt::Debug for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use crate::Stored::*;
    match *self {
      None => write!(f, "None"),
      String(ref s) => write!(f, "{s}"),
      Int(ref num) => write!(f, "Stored::Int[{num:?}]"),
      VecChar(ref vs) => write!(f, "Stored::VecChar[{vs:?}]"),
      VecOptionChar(ref vs) => write!(f, "Stored::VecOptionChar[{vs:?}]"),
      Stash(ref vs) => write!(f, "Stored::Stash[{vs:?}]"),
      VecString(ref vs) => write!(f, "Stored::VecString[{vs:?}]"),
      Bool(ref b) => write!(f, "Stored::Bool[{b:?}]"),
      Token(ref t) => write!(f, "Stored::Token[{t:?}]"),
      Tokens(ref t) => write!(f, "Stored::Tokens[{t:?}]"),
      Locator(ref t) => write!(f, "Stored::Locator[{t:?}]"),
      Catcode(ref cc) => write!(f, "Stored::Catcode[{cc:?}]"),
      Charcode(ref cc) => write!(f, "Stored::Charcode[{cc:?}]"),
      IfFrame(ref fr) => write!(f, "Stored::IfFrame[{fr:?}]"),
      Expandable(ref expandable) => write!(f, "Stored::Expandable[{expandable:?}]"),
      Conditional(ref conditional) => write!(f, "Stored::Conditional[TODO]"),
      Primitive(ref primitive) => write!(f, "Stored::Primitive[TODO]"),
      MathPrimitive(ref _primitive) => write!(f, "Stored::MathPrimitive[TODO]"),
      // MathPrimitiveOptions(ref _primitive) => write!(f, "<math primitive options>"),
      Constructor(ref _constructor) => write!(f, "Stored::Constructor[TODO]"),
      Digested(ref digested) => write!(f, "Stored::Digested[{digested:?}]"),
      Parameter(ref parameter) => write!(f, "Stored::Parameter[{parameter:?}]"),
      Register(ref register) => write!(f, "Stored::Register[{:?}]", register.borrow().cs),
      Rewrite(ref rewrite) => write!(f, "Stored::Rewrite[{rewrite:?}]"),
      Mouth(ref mouth) => write!(f, "Stored::Mouth[{:?}]", mouth.read().unwrap().get_source()),
      Font(ref font) => write!(f, "Stored::Font[{font:?}]"),
      FontDirective(ref font) => write!(f, "Stored::FontDirective[{font:?}]"),
      Number(ref number) => write!(f, "Stored::Number[{number:?}]"),
      Glue(ref glue) => write!(f, "Stored::Glue[{glue:?}]"),
      MuGlue(ref glue) => write!(f, "Stored::MuGlue[{glue:?}]"),
      Dimension(ref dimension) => write!(f, "Stored::Dimension[{dimension:?}]"),
      MuDimension(ref dimension) => write!(f, "Stored::MuDimension[{dimension:?}]"),
      VecDigested(ref digested_vec) => write!(f, "VecDigested{digested_vec:?}"),
      VecTokens(ref vec) => write!(f, "VecTokens{vec:?}"),
      VecDequeStored(ref vec) => write!(f, "VecDequeStored{vec:?}"),
      HashStored(ref hos) => write!(f, "HashStored{hos:?}"),
      HashTagData(ref htd) => write!(f, "HashTagData[{htd:?}]"),
      HashString(ref hstr) => write!(f, "HashStr[{hstr:?}]"),
      Ligature(ref lig) => write!(f, "Ligature[{lig:?}]"),
    }
  }
}
impl fmt::Display for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use crate::Stored::*;
    match *self {
      Digested(ref digested) => write!(f, "{digested}"),
      Dimension(ref v) => write!(f, "{v}"),
      Number(ref v) => write!(f, "{v}"),
      Glue(ref v) => write!(f, "{v}"),
      MuGlue(ref v) => write!(f, "{v}"),
      MuDimension(ref v) => write!(f, "{v}"),
      Font(ref font) => write!(f, "{font}"),
      String(ref s) => write!(f, "{s}"),
      Int(ref s) => write!(f, "{s}"),
      Bool(ref s) => write!(f, "{s}"),
      Tokens(ref s) => write!(f, "{s}"),
      Token(ref s) => write!(f, "{s}"),
      ref variant => {
        panic!("TODO: implement Display for Stored variant {variant:?}");
        // write!(f, "{:?}", self)
      },
    }
  }
}
/// We can not simply derive PartialEq since it is not obvious (to rust, or to me)
/// if it is safe to carelessly lock the RwLock guards of the suspect fields with interior mutability
/// Worse: some conditions depend on the Stateful meaning of Token's,
///        so the perfect equality check would need State as an argument :(
impl PartialEq for Stored {
  fn eq(&self, other: &Stored) -> bool {
    use crate::Stored::*;
    match *self {
      Stored::None => matches!(other, Stored::None),
      String(ref s) => {
        if let String(s2) = other {
          *s == *s2
        } else {
          false
        }
      },
      Int(ref num) => {
        if let Int(num2) = other {
          *num == *num2
        } else {
          false
        }
      },
      VecChar(ref vs) => {
        if let VecChar(vs2) = other {
          *vs == *vs2
        } else {
          false
        }
      },
      VecOptionChar(ref vs) => {
        if let VecOptionChar(vs2) = other {
          *vs == *vs2
        } else {
          false
        }
      },
      VecString(ref vs) => {
        if let VecString(vs2) = other {
          *vs == *vs2
        } else {
          false
        }
      },
      Bool(ref b) => {
        if let Bool(b2) = other {
          *b == *b2
        } else {
          false
        }
      },
      Token(ref t) => {
        if let Token(t2) = other {
          *t == *t2
        } else {
          false
        }
      },
      Tokens(ref t) => {
        if let Tokens(t2) = other {
          *t == *t2
        } else {
          false
        }
      },
      Locator(ref t) => {
        if let Locator(t2) = other {
          *t == *t2
        } else {
          false
        }
      },
      Catcode(ref cc) => {
        if let Catcode(cc2) = other {
          *cc == *cc2
        } else {
          false
        }
      },
      Charcode(ref cc) => {
        if let Charcode(cc2) = other {
          *cc == *cc2
        } else {
          false
        }
      },
      IfFrame(ref fr) => {
        if let IfFrame(fr2) = other {
          *fr.read().unwrap() == *fr2.read().unwrap()
        } else {
          false
        }
      },
      Expandable(ref e) => {
        if let Expandable(e2) = other {
          **e == **e2
        } else {
          false
        }
      },
      Conditional(ref c) => {
        if let Conditional(c2) = other {
          **c == **c2
        } else {
          false
        }
      },
      Primitive(ref p) => {
        if let Primitive(p2) = other {
          **p == **p2
        } else {
          false
        }
      },
      MathPrimitive(ref p) => {
        if let MathPrimitive(p2) = other {
          **p == **p2
        } else {
          false
        }
      },
      // MathPrimitiveOptions(ref _primitive) =>
      Constructor(ref c) => {
        if let Constructor(c2) = other {
          **c == **c2
        } else {
          false
        }
      },
      Digested(ref d) => {
        if let Digested(d2) = other {
          **d == **d2
        } else {
          false
        }
      },
      Parameter(ref p) => {
        if let Parameter(p2) = other {
          **p == **p2
        } else {
          false
        }
      },
      Register(ref r) => {
        if let Register(r2) = other {
          **r == **r2
        } else {
          false
        }
      },
      Rewrite(ref r) => {
        if let Rewrite(r2) = other {
          *r == *r2
        } else {
          false
        }
      },
      Mouth(ref m) => {
        if let Mouth(m2) = other {
          *m.read().unwrap() == *m2.read().unwrap()
        } else {
          false
        }
      },
      Font(ref f) => {
        if let Font(f2) = other {
          **f == **f2
        } else {
          false
        }
      },
      FontDirective(ref fd) => {
        if let FontDirective(fd2) = other {
          fd == fd2
        } else {
          false
        }
      },
      Number(ref n) => {
        if let Number(n2) = other {
          *n == *n2
        } else {
          false
        }
      },
      Glue(ref g) => {
        if let Glue(g2) = other {
          *g == *g2
        } else {
          false
        }
      },
      MuGlue(ref mg) => {
        if let MuGlue(mg2) = other {
          *mg == *mg2
        } else {
          false
        }
      },
      Dimension(ref d) => {
        if let Dimension(d2) = other {
          *d == *d2
        } else {
          false
        }
      },
      MuDimension(ref md) => {
        if let MuDimension(md2) = other {
          *md == *md2
        } else {
          false
        }
      },
      VecDigested(ref vd) => {
        if let VecDigested(vd2) = other {
          *vd == *vd2
        } else {
          false
        }
      },
      VecTokens(ref vd) => {
        if let VecTokens(vd2) = other {
          *vd == *vd2
        } else {
          false
        }
      },
      VecDequeStored(ref v) => {
        if let VecDequeStored(v2) = other {
          v.len() == v2.len() && v.iter().zip(v2.iter()).all(|(item1, item2)| item1 == item2)
        } else {
          false
        }
      },
      Stash(ref v) => {
        if let Stash(v2) = other {
          v.len() == v2.len() // TODO: Do we need accuracy on stash comparisons?
        } else {
          false
        }
      },
      HashStored(ref hs) => {
        if let HashStored(hs2) = other {
          hs.len() == hs2.len()
            && hs
              .iter()
              .all(|(key, value)| if let Some(item2) = hs2.get(key) { value == item2 } else { false })
        } else {
          false
        }
      },
      HashTagData(ref htd) => {
        if let HashTagData(htd2) = other {
          *htd == *htd2
        } else {
          false
        }
      },
      HashString(ref hstr) => {
        if let HashString(hstr2) = other {
          *hstr == *hstr2
        } else {
          false
        }
      },
      Ligature(ref lig) => {
        if let Ligature(lig2) = other {
          *lig == *lig2
        } else {
          false
        }
      },
    }
  }
}

unsafe impl Send for Stored {}
unsafe impl Sync for Stored {}
impl Stored {
  pub fn cast_to_string_hash(in_map: &HashMap<String, Stored>) -> HashMap<String, String> {
    let mut out_map: HashMap<String, String> = HashMap::new();
    for (key, val) in in_map.iter() {
      out_map.insert(key.to_owned(), val.to_string());
    }
    out_map
  }
  /// Dynamic dispatch for Definition's `read_arguments`,
  /// to circumvent the limitations of using trait objects with `Arc<Definition>`
  pub fn read_arguments(&self, gullet: &mut Gullet, state: &mut State) -> Result<Vec<ArgWrap>> {
    use crate::definition::Definition;
    match self {
      Stored::Conditional(ref entry) => entry.read_arguments(gullet, state),
      Stored::Constructor(ref entry) => entry.read_arguments(gullet, state),
      Stored::Expandable(ref entry) => entry.read_arguments(gullet, state),
      Stored::MathPrimitive(ref entry) => entry.read_arguments(gullet, state),
      Stored::Primitive(ref entry) => entry.read_arguments(gullet, state),
      Stored::Register(ref entry) => entry.read_arguments(gullet, state),
      e => Err(s!(".read_arguments not defined for stored variant {:?}", e).into()),
    }
  }
  pub fn to_attribute(&self) -> String {
    match self {
      Stored::Dimension(ref v) => v.to_attribute(),
      Stored::Number(ref v) => v.to_attribute(),
      //Stored::MuDimension(ref v) => v.to_attribute(),
      Stored::Glue(ref v) => v.to_attribute(),
      Stored::MuGlue(ref v) => v.to_attribute(),
      other => s!("{}", other),
    }
  }
}

impl From<bool> for Stored {
  fn from(value: bool) -> Self { Stored::Bool(value) }
}

impl<'a> From<bool> for &'a Stored {
  fn from(value: bool) -> Self {
    if value {
      &STORED_TRUE
    } else {
      &STORED_FALSE
    }
  }
}

impl From<String> for Stored {
  fn from(value: String) -> Self { Stored::String(value) }
}

impl From<char> for Stored {
  fn from(value: char) -> Self { Stored::String(value.to_string()) }
}

impl<'a> From<&'a String> for Stored {
  fn from(value: &'a String) -> Self { Stored::String(value.clone()) }
}

impl<'a> From<&'a str> for Stored {
  fn from(value: &'a str) -> Self { value.to_owned().into() }
}

impl From<usize> for Stored {
  fn from(value: usize) -> Self { Stored::Int(value as i32) }
}

impl From<i32> for Stored {
  fn from(value: i32) -> Self { Stored::Int(value) }
}

impl From<f32> for Stored {
  fn from(value: f32) -> Self { Stored::Number(Number::new(value.floor() as i32)) }
}

impl From<Catcode> for Stored {
  fn from(value: Catcode) -> Self { Stored::Catcode(value) }
}

impl From<Token> for Stored {
  fn from(value: Token) -> Self { Stored::Token(value) }
}

// Storing all definitions is expected - Arc<Expandable> case

impl From<Tokens> for Stored {
  fn from(value: Tokens) -> Self { Stored::Tokens(value) }
}

impl From<Locator> for Stored {
  fn from(value: Locator) -> Self { Stored::Locator(value) }
}

impl From<Mouth> for Stored {
  fn from(value: Mouth) -> Self { Stored::Mouth(Arc::new(RwLock::new(value))) }
}

impl From<Arc<Expandable>> for Stored {
  fn from(definition: Arc<Expandable>) -> Self { Stored::Expandable(definition) }
}
/// Storing all definitions is expected - Expandable case
impl From<Expandable> for Stored {
  fn from(definition: Expandable) -> Self { Arc::new(definition).into() }
}

impl From<Arc<Conditional>> for Stored {
  fn from(definition: Arc<Conditional>) -> Self { Stored::Conditional(definition) }
}
impl From<Conditional> for Stored {
  fn from(value: Conditional) -> Self { Arc::new(value).into() }
}

impl From<Arc<Primitive>> for Stored {
  fn from(definition: Arc<Primitive>) -> Self { Stored::Primitive(definition) }
}
impl From<Primitive> for Stored {
  fn from(value: Primitive) -> Self { Arc::new(value).into() }
}

impl From<Arc<MathPrimitive>> for Stored {
  fn from(definition: Arc<MathPrimitive>) -> Self { Stored::MathPrimitive(definition) }
}
impl From<MathPrimitive> for Stored {
  fn from(value: MathPrimitive) -> Self { Arc::new(value).into() }
}

impl From<Arc<Constructor>> for Stored {
  fn from(definition: Arc<Constructor>) -> Self { Stored::Constructor(definition) }
}
impl From<Constructor> for Stored {
  fn from(value: Constructor) -> Self { Arc::new(value).into() }
}

impl From<List> for Stored {
  fn from(value: List) -> Self { crate::Digested::from(value).into() }
}

impl From<crate::Digested> for Stored {
  fn from(value: crate::Digested) -> Self { Box::new(value).into() }
}

impl From<Box<crate::Digested>> for Stored {
  fn from(value: Box<crate::Digested>) -> Self { Stored::Digested(value) }
}

impl From<Parameter> for Stored {
  fn from(value: Parameter) -> Self { Stored::Parameter(Arc::new(value)) }
}

impl From<Arc<Font>> for Stored {
  fn from(font: Arc<Font>) -> Self { Stored::Font(font) }
}

impl From<Arc<RegisterCell>> for Stored {
  fn from(register: Arc<RegisterCell>) -> Self { Stored::Register(register) }
}
impl From<Register> for Stored {
  fn from(register: Register) -> Self { Arc::new(RegisterCell::new(RwLock::new(register))).into() }
}

impl From<Rewrite> for Stored {
  fn from(value: Rewrite) -> Self { Stored::Rewrite(value) }
}

impl From<Font> for Stored {
  fn from(value: Font) -> Self { Arc::new(value).into() }
}

impl<'a> From<Cow<'a, Font>> for Stored {
  fn from(value: Cow<Font>) -> Self { Arc::new((*value).clone()).into() }
}

impl From<Number> for Stored {
  fn from(value: Number) -> Self { Stored::Number(value) }
}
impl From<Dimension> for Stored {
  fn from(value: Dimension) -> Self { Stored::Dimension(value) }
}

impl<'a> From<&'a Token> for Stored {
  fn from(value: &'a Token) -> Self { Stored::Token(value.clone()) }
}

impl From<Vec<char>> for Stored {
  fn from(value: Vec<char>) -> Self { Stored::VecChar(value) }
}
impl From<Vec<Option<char>>> for Stored {
  fn from(value: Vec<Option<char>>) -> Self { Stored::VecOptionChar(value) }
}

impl From<Vec<String>> for Stored {
  fn from(value: Vec<String>) -> Self { Stored::VecString(value) }
}

impl<'a> From<Vec<&'a str>> for Stored {
  fn from(value: Vec<&'a str>) -> Self { Stored::VecString(value.iter().map(ToString::to_string).collect::<Vec<String>>()) }
}

impl From<Vec<Token>> for Stored {
  fn from(value: Vec<Token>) -> Self { Stored::Tokens(Tokens::new(value)) }
}

impl From<Vec<crate::Digested>> for Stored {
  fn from(value: Vec<crate::Digested>) -> Self { Stored::VecDigested(value) }
}

impl From<Vec<crate::Tokens>> for Stored {
  fn from(value: Vec<crate::Tokens>) -> Self { Stored::VecTokens(value) }
}

impl From<HashMap<String, String>> for Stored {
  fn from(value: HashMap<String, String>) -> Self { Stored::HashString(value) }
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
      RegisterValue::MuDimension(v) => Stored::MuDimension(v),
      RegisterValue::Glue(v) => Stored::Glue(v),
      RegisterValue::MuGlue(v) => Stored::MuGlue(v),
      RegisterValue::Token(v) => Stored::Token(v),
      RegisterValue::Tokens(v) => Stored::Tokens(v),
    }
  }
}

impl From<Arc<RwLock<IfFrame>>> for Stored {
  fn from(frame: Arc<RwLock<IfFrame>>) -> Stored { Stored::IfFrame(frame) }
}

impl From<Ligature> for Stored {
  fn from(lig: Ligature) -> Stored { Stored::Ligature(lig) }
}

impl From<Option<&Stored>> for Stored {
  fn from(stored_opt: Option<&Stored>) -> Stored {
    match stored_opt {
      Some(val) => val.clone(),
      None => Stored::Bool(false),
    }
  }
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
      Stored::String(v) => v.to_owned(),
      v => s!("{v:?}"),
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

impl<'a> From<&'a Stored> for Option<Arc<Font>> {
  fn from(value: &'a Stored) -> Option<Arc<Font>> {
    match value {
      Stored::Font(ref f) => Some(Arc::clone(f)),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Number> {
  fn from(value: &'a Stored) -> Option<Number> {
    match value {
      Stored::Number(ref n) => Some(*n),
      Stored::Dimension(ref n) => Some(Number::new(n.value_of())),
      Stored::Glue(ref n) => Some(Number::new(n.value_of())),
      Stored::MuDimension(ref n) => Some(Number::new(n.value_of())),
      Stored::MuGlue(ref n) => Some(Number::new(n.value_of())),
      other => {
        eprintln!("TODO: auto-cast of Stored to Number attempted on {other:?}");
        None
      },
    }
  }
}

impl<'a> From<&'a Stored> for Option<Dimension> {
  fn from(value: &'a Stored) -> Option<Dimension> {
    match value {
      Stored::Dimension(ref n) => Some(*n),
      Stored::Number(ref n) => Some(Dimension::new(n.value_of())),
      Stored::Glue(ref n) => Some(Dimension::new(n.value_of())),
      Stored::MuDimension(ref n) => Some(Dimension::new(n.value_of())),
      Stored::MuGlue(ref n) => Some(Dimension::new(n.value_of())),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Glue> {
  fn from(value: &'a Stored) -> Option<Glue> {
    match value {
      Stored::Dimension(ref n) => Some(Glue::new(n.value_of())),
      Stored::Number(ref n) => Some(Glue::new(n.value_of())),
      Stored::MuDimension(ref n) => Some(Glue::new(n.value_of())),
      Stored::MuGlue(ref n) => Some(Glue::new(n.value_of())),
      Stored::Glue(ref n) => Some(*n),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Tokens> {
  fn from(value: &'a Stored) -> Option<Tokens> {
    match value {
      Stored::String(ref text) => Some(mouth::tokenize_internal(text)),
      Stored::Token(ref ts) => Some(Tokens::new(vec![ts.clone()])),
      Stored::Tokens(ref ts) => Some(ts.clone()),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Arc<RegisterCell>> {
  fn from(value: &'a Stored) -> Option<Arc<RegisterCell>> {
    match value {
      Stored::Register(ref reg) => Some(Arc::clone(reg)),
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

impl<'a> From<&'a Stored> for Option<&'a Vec<Option<char>>> {
  fn from(value: &'a Stored) -> Option<&'a Vec<Option<char>>> {
    match value {
      Stored::VecOptionChar(ref cc) => Some(cc),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<RegisterValue> {
  fn from(value: &'a Stored) -> Option<RegisterValue> {
    match value {
      Stored::Number(v) => Some(RegisterValue::Number(*v)),
      Stored::Dimension(v) => Some(RegisterValue::Dimension(*v)),
      Stored::Glue(v) => Some(RegisterValue::Glue(*v)),
      Stored::MuGlue(v) => Some(RegisterValue::MuGlue(*v)),
      Stored::Token(v) => Some(RegisterValue::Token(v.clone())),
      Stored::Tokens(v) => Some(RegisterValue::Tokens(v.clone())),
      _ => None,
    }
  }
}

impl From<Stored> for Option<crate::Digested> {
  fn from(value: Stored) -> Option<crate::Digested> {
    match value {
      Stored::Digested(digested) => Some(*digested),
      Stored::String(text) => Some(text.into()),
      Stored::Int(text) => Some(text.to_string().into()),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<crate::Digested> {
  fn from(value: &'a Stored) -> Option<crate::Digested> {
    match value {
      Stored::Digested(digested) => Some((**digested).clone()),
      Stored::String(text) => Some(text.into()),
      Stored::Int(text) => Some(text.to_string().into()),
      _ => None,
    }
  }
}

impl<'a, 'b> From<&'a &'b Stored> for Option<crate::Digested> {
  fn from(value: &'a &'b Stored) -> Option<crate::Digested> { (*value).into() }
}

impl<'a> From<&'a Stored> for Token {
  fn from(value: &'a Stored) -> Token {
    match value {
      Stored::Tokens(ts) => ts.into(),
      Stored::Token(t) => (*t).clone(),
      Stored::String(t) => T_CS!(t),
      t => {
        let message = s!("dangerous cast to CS for {:?}", t);
        Warn!("Stored", "cast", None, None, message);
        T_CS!(t.to_string())
      }, /* TODO, is this the right place to default to CS? Do we need a
          * custom method instead? */
    }
  }
}
