use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use string_interner::symbol::SymbolU32;

use crate::common::arena;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::glue::Glue;
use crate::common::locator::Locator;
use crate::common::mudimension::MuDimension;
use crate::common::muglue::MuGlue;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::alignment::Alignment;
use crate::definition::argument::ArgWrap;
use crate::definition::conditional::{Conditional, IfFrame};
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use crate::definition::primitive::Primitive;
use crate::definition::register::{Register, RegisterValue};
use crate::definition::FontDirective;
use crate::definition::Reversion;
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

// Basic principles:
// 1. If the type is `Copy`, store directly
// 2. If the type is intended as State-exclusive, store in a Box
//      (or directly if any already Boxed datatype such as Vec, VecDeque, HashMap)
// 3. If the struct is intended for reuse in digestion components, store it in an Rc,
//    e.g. Rc<Font>
// 4. In the very unfortunate cases where we have to mutate items while they are stored,
//    we may consider a RefCell<> wrapper for interior mutability.
//      BUT it is often possible to take the value out to mutate, and re-insert.
//      State::checkout_value(key) + mutate + State::checkin_value(key,val)
//      The only cases where this isn't straightforward is for deep recursive callchains,
//      as in the ones where we rely on Stomach or a Mouth being in State.
/// The original global State (in Perl) allowed arbitrary values. To stay consistent, we create an
/// extremely permissive struct that affords all essential kinds of values that appear essential.
#[derive(Default, Clone)]
pub enum Stored {
  /// if we want to keep a key but make it 'undef', set it to None
  #[default]
  None,
  // Primitives (Copy types, or cheap Clone)
  /// atomic data (Copy)
  Bool(bool),
  /// atomic data (Clone)
  String(SymbolU32),
  /// atomic data (Copy)
  Charcode(u16),
  /// atomic data (Copy)
  /// note that we currently work with 64-bit integers
  Int(i64),
  /// atomic data (Clone)
  Node(Node),
  // Collections (boxed)
  /// boxed collection
  VecChar(Vec<char>),
  /// boxed collection
  VecOptionChar(Vec<Option<char>>),
  /// boxed collection
  VecString(Vec<String>),
  /// boxed collection (latexml)
  VecTokens(Vec<crate::Tokens>),
  /// boxed collection (latexml)
  VecDigested(Vec<crate::Digested>),
  /// the heart of state - a stored Stash table
  Stash(StashTable),
  /// boxed map
  HashString(HashMap<String, String>),
  /// boxed collection - Stored
  VecDequeStored(VecDeque<Stored>),
  /// boxed map - Stored
  HashStored(HashMap<String, Stored>),
  /// boxed map (latexml)
  HashTagData(HashMap<String, Vec<TagData>>),
  // LaTeXML primitives (Copy types)
  /// latexml object
  Catcode(Catcode),
  /// latexml object
  Token(Token),
  /// latexml object
  Tokens(Tokens),
  /// latexml object
  Number(Number),
  /// latexml object
  Glue(Glue),
  /// latexml object
  MuGlue(MuGlue),
  /// latexml object
  Dimension(Dimension),
  /// latexml object
  MuDimension(MuDimension),
  /// metadata object
  Locator(Box<Locator>),
  /// latexml object
  Rewrite(Box<Rewrite>),
  /// latexml object
  Ligature(Box<Ligature>),
  /// latexml object
  Reversion(Reversion),
  // LaTeXML objects (Rc-wrapped)
  /// latexml object (Rc-wrapped)
  Register(Rc<Register>),
  /// latexml object (Rc-wrapped)
  Expandable(Rc<Expandable>),
  /// latexml object (Rc-wrapped)
  Conditional(Rc<Conditional>),
  /// latexml object (Rc-wrapped)
  Primitive(Rc<Primitive>),
  /// latexml object (Rc-wrapped)
  MathPrimitive(Rc<MathPrimitive>),
  //  MathPrimitiveOptions(MathPrimitiveOptions), // Maybe later
  /// latexml object (Rc-wrapped)
  Constructor(Rc<Constructor>),
  /// latexml object (Rc-wrapped)
  Digested(crate::Digested),
  /// latexml object (Rc-wrapped)
  Parameter(Rc<Parameter>),
  /// latexml object (Rc-wrapped)
  Font(Rc<Font>),
  /// a stored FontDirective (Font or closure building a Font)
  FontDirective(FontDirective),
  /// WALL OF SHAME (interior mutability) -- can we dispense with these?
  Mouth(Rc<RefCell<Mouth>>),
  /// WALL OF SHAME (interior mutability) -- can we dispense with these?
  IfFrame(Rc<RefCell<IfFrame>>),
}

impl fmt::Debug for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use crate::Stored::*;
    match *self {
      None => write!(f, "None"),
      String(ref s) => arena::with(*s, |str| write!(f, "{str}")),
      Int(ref num) => write!(f, "Stored::Int[{num:?}]"),
      Node(ref n) => write!(f, "Stored::Node[{n:?}]"),
      VecChar(ref vs) => write!(f, "Stored::VecChar[{vs:?}]"),
      VecOptionChar(ref vs) => write!(f, "Stored::VecOptionChar[{vs:?}]"),
      Stash(ref vs) => write!(f, "Stored::Stash[{vs:?}]"),
      VecString(ref vs) => write!(f, "Stored::VecString[{vs:?}]"),
      Bool(ref b) => write!(f, "Stored::Bool[{b:?}]"),
      Token(ref t) => write!(f, "Stored::Token[{t:?}]"),
      Tokens(ref t) => write!(f, "Stored::Tokens[{t:?}]"),
      Locator(ref t) => write!(f, "Stored::Locator[{t:?}]"),
      Reversion(ref _t) => write!(f, "Stored::Reversion[TODO]"),
      Catcode(ref cc) => write!(f, "Stored::Catcode[{cc:?}]"),
      Charcode(ref cc) => write!(f, "Stored::Charcode[{cc:?}]"),
      IfFrame(ref fr) => write!(f, "Stored::IfFrame[{fr:?}]"),
      Expandable(ref expandable) => write!(f, "Stored::Expandable[{expandable:?}]"),
      Conditional(ref _conditional) => write!(f, "Stored::Conditional[TODO]"),
      Primitive(ref _primitive) => write!(f, "Stored::Primitive[TODO]"),
      MathPrimitive(ref _primitive) => write!(f, "Stored::MathPrimitive[TODO]"),
      // MathPrimitiveOptions(ref _primitive) => write!(f, "<math primitive options>"),
      Constructor(ref _constructor) => write!(f, "Stored::Constructor[TODO]"),
      Digested(ref digested) => write!(f, "Stored::Digested[{digested:?}]"),
      Parameter(ref parameter) => write!(f, "Stored::Parameter[{parameter:?}]"),
      Register(ref register) => write!(f, "Stored::Register[{:?}]", register.cs),
      Rewrite(ref rewrite) => write!(f, "Stored::Rewrite[{rewrite:?}]"),
      Mouth(ref mouth) => write!(f, "Stored::Mouth[{:?}]", mouth.borrow().get_source()),
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
      String(ref s) => arena::with(*s, |str| write!(f, "{str}")),
      Int(ref s) => write!(f, "{s}"),
      Bool(ref s) => write!(f, "{s}"),
      Tokens(ref s) => write!(f, "{s}"),
      Token(ref s) => write!(f, "{s}"),
      Conditional(ref s) => write!(f, "{s}"),
      ref variant => {
        panic!("TODO: implement Display for Stored variant {variant:?}");
        // write!(f, "{:?}", self)
      },
    }
  }
}

/// We can not simply derive PartialEq since it is not obvious (to rust, or to me)
/// if it is safe to eagerly borrow() from the RefCell guards over fields with interior
/// mutability.
/// Worse: some conditions depend on the Stateful meaning of Token's,
///        so the perfect equality check would need State as an argument.
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
      Node(ref n) => {
        if let Node(n2) = other {
          *n == *n2
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
      Reversion(ref t) => {
        if let Reversion(t2) = other {
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
          *fr.borrow() == *fr2.borrow()
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
          *d == *d2
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
          *m.borrow() == *m2.borrow()
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
            && hs.iter().all(|(key, value)| {
              if let Some(item2) = hs2.get(key) {
                value == item2
              } else {
                false
              }
            })
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
  /// helper method that uses `ToString::to_string` to flatten a map with Stored values
  pub fn cast_to_string_hash(in_map: &HashMap<String, Stored>) -> HashMap<String, String> {
    let mut out_map: HashMap<String, String> = HashMap::default();
    for (key, val) in in_map.iter() {
      out_map.insert(key.to_owned(), val.to_string());
    }
    out_map
  }
  /// Dynamic dispatch for Definition's `read_arguments`,
  /// to circumvent the limitations of using trait objects with `Rc<Definition>`
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
  /// Uses `NumericOps::to_attribute` for Stored values supporting it, otherwise
  /// `ToString::to_string`
  pub fn to_attribute(&self) -> String {
    match self {
      Stored::Dimension(ref v) => v.to_attribute(),
      Stored::Number(ref v) => v.to_attribute(),
      //Stored::MuDimension(ref v) => v.to_attribute(),
      Stored::Glue(ref v) => v.to_attribute(),
      Stored::MuGlue(ref v) => v.to_attribute(),
      other => other.to_string(),
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
  fn from(value: String) -> Self { Stored::String(arena::pin(value)) }
}
impl From<SymbolU32> for Stored {
  fn from(value: SymbolU32) -> Self { Stored::String(value) }
}

impl From<char> for Stored {
  fn from(value: char) -> Self { Stored::String(arena::pin_char(value)) }
}

impl<'a> From<&'a String> for Stored {
  fn from(value: &'a String) -> Self { Stored::String(arena::pin(value)) }
}

impl From<&'static str> for Stored {
  fn from(value: &'static str) -> Self { Stored::String(arena::pin_static(value)) }
}

impl From<usize> for Stored {
  fn from(value: usize) -> Self { Stored::Int(value as i64) }
}

impl From<i32> for Stored {
  fn from(value: i32) -> Self { Stored::Int(value as i64) }
}
impl From<i64> for Stored {
  fn from(value: i64) -> Self { Stored::Int(value) }
}

impl From<f64> for Stored {
  fn from(value: f64) -> Self { Stored::Number(Number::new(value.floor() as i64)) }
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

impl From<Locator> for Stored {
  fn from(value: Locator) -> Self { Stored::Locator(Box::new(value)) }
}

impl From<Mouth> for Stored {
  fn from(value: Mouth) -> Self { Stored::Mouth(Rc::new(RefCell::new(value))) }
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

impl From<List> for Stored {
  fn from(value: List) -> Self { crate::Digested::from(value).into() }
}

impl From<Node> for Stored {
  fn from(value: Node) -> Self { Stored::Node(value) }
}

impl From<crate::Digested> for Stored {
  fn from(value: crate::Digested) -> Self { crate::Stored::Digested(value) }
}

impl From<&crate::Digested> for Stored {
  fn from(value: &crate::Digested) -> Self { crate::Stored::Digested(value.clone()) }
}

impl<T> From<Option<T>> for Stored where
  T: Into<Stored> + Sized {
  fn from(value_opt: Option<T>) -> Self {
    match value_opt {
      None => Stored::None,
      Some(v) => v.into()
    }
  }
}

impl<'a> From<Cow<'a, crate::Digested>> for Stored {
  fn from(value: Cow<'a, crate::Digested>) -> Self { crate::Stored::Digested(value.into_owned()) }
}

impl From<Box<crate::Digested>> for Stored {
  fn from(value: Box<crate::Digested>) -> Self { Stored::Digested(*value) }
}

impl From<Parameter> for Stored {
  fn from(value: Parameter) -> Self { Stored::Parameter(Rc::new(value)) }
}

impl From<Rc<Font>> for Stored {
  fn from(font: Rc<Font>) -> Self { Stored::Font(font) }
}

impl From<Rc<Register>> for Stored {
  fn from(register: Rc<Register>) -> Self { Stored::Register(register) }
}
impl From<Register> for Stored {
  fn from(register: Register) -> Self { Rc::new(register).into() }
}

impl From<Rewrite> for Stored {
  fn from(value: Rewrite) -> Self { Stored::Rewrite(Box::new(value)) }
}

impl From<Font> for Stored {
  fn from(value: Font) -> Self { Rc::new(value).into() }
}

impl<'a> From<Cow<'a, Font>> for Stored {
  fn from(value: Cow<Font>) -> Self { Rc::new((*value).clone()).into() }
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

impl From<Alignment> for Stored {
  fn from(a: Alignment) -> Self { Stored::Digested(crate::Digested::from(a)) }
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
  fn from(value: Vec<&'a str>) -> Self {
    Stored::VecString(
      value
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>(),
    )
  }
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

impl From<Rc<RefCell<IfFrame>>> for Stored {
  fn from(frame: Rc<RefCell<IfFrame>>) -> Stored { Stored::IfFrame(frame) }
}

impl From<Ligature> for Stored {
  fn from(lig: Ligature) -> Stored { Stored::Ligature(Box::new(lig)) }
}

impl From<Reversion> for Stored {
  fn from(rev: Reversion) -> Stored { Stored::Reversion(rev) }
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
      Stored::String(v) => arena::to_string(*v),
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

impl<'a> From<&'a Stored> for Option<Rc<Font>> {
  fn from(value: &'a Stored) -> Option<Rc<Font>> {
    match value {
      Stored::Font(ref f) => Some(Rc::clone(f)),
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
      Stored::String(ref sym) => Some(mouth::tokenize_internal(&arena::to_string(*sym))),
      Stored::Token(ref ts) => Some(Tokens::new(vec![ts.clone()])),
      Stored::Tokens(ref ts) => Some(ts.clone()),
      Stored::VecDequeStored(vdq) => {
        // Each item in the queue can be unlisted into a Vec<Token>
        // and then the result can be re-cast as a single Tokens
        let mut collected: Vec<Token> = Vec::new();
        for item in vdq {
          let item_tokens_opt: Option<Tokens> = item.into();
          if let Some(item_tokens) = item_tokens_opt {
            collected.extend(item_tokens.unlist());
          }
        }
        if collected.is_empty() {
          None
        } else {
          Some(Tokens::new(collected))
        }
      },
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Rc<Register>> {
  fn from(value: &'a Stored) -> Option<Rc<Register>> {
    match value {
      Stored::Register(ref reg) => Some(Rc::clone(reg)),
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
      Stored::Digested(digested) => Some(digested),
      Stored::String(text) => Some(text.into()),
      Stored::Int(text) => Some(text.to_string().into()),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<crate::Digested> {
  fn from(value: &'a Stored) -> Option<crate::Digested> {
    match value {
      Stored::Digested(digested) => Some((*digested).clone()),
      Stored::String(text) => Some((*text).into()),
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
      Stored::String(text) => Token {
        text: *text,
        code: Catcode::CS,
        smuggled: None,
      },
      t => {
        let message = s!("dangerous cast to CS for {:?}", t);
        Warn!("Stored", "cast", None, None, message);
        T_CS!(t.to_string())
      }, /* TODO, is this the right place to default to CS? Do we need a
          * custom method instead? */
    }
  }
}
