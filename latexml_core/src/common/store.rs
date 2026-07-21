use std::{borrow::Cow, cell::RefCell, collections::VecDeque, fmt, rc::Rc};

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::definition::math_primitive::MathPrimitive; //MathPrimitiveOptions
use crate::{
  alignment::Alignment,
  common::{
    arena::{self, SymStr, data::SymHashMap},
    dimension::Dimension,
    error::*,
    float::Float,
    font::Font,
    glue::Glue,
    locator::Locator,
    mudimension::MuDimension,
    muglue::MuGlue,
    number::Number,
    numeric_ops::NumericOps,
  },
  definition::{
    Definition, FontDirective, Reversion,
    argument::ArgWrap,
    conditional::{Conditional, IfFrame},
    constructor::Constructor,
    expandable::Expandable,
    primitive::Primitive,
    register::{Register, RegisterValue},
  },
  document::tag::{RawFrontmatter, TagData},
  keyval::KeyVal,
  keyvals::KeyVals,
  ligature::Ligature,
  list::List,
  mouth,
  mouth::Mouth,
  parameter::Parameter,
  rewrite::Rewrite,
  state::StashTable,
  token::{Catcode, Token},
  tokens::Tokens,
};

const STORED_TRUE: Stored = Stored::Bool(true);
const STORED_FALSE: Stored = Stored::Bool(false);

// Basic principles:
// 1. If the type is `Copy`, store directly
// 2. If the type is intended as state-exclusive, store in a Box (or directly if any already Boxed
//    datatype such as Vec, VecDeque, HashMap)
// 3. If the struct is intended for reuse in digestion components, store it in an Rc, e.g. Rc<Font>
// 4. In the very unfortunate cases where we have to mutate items while they are stored, we may
//    consider a RefCell<> wrapper for interior mutability. BUT it is often possible to take the
//    value out to mutate, and re-insert. state::checkout_value(key) + mutate +
//    state::checkin_value(key,val) The only cases where this isn't straightforward is for deep
//    recursive callchains, as in the ones where we rely on Stomach or a Mouth being in state.
/// The original global state (in Perl) allowed arbitrary values. To stay consistent, we create an
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
  String(SymStr),
  /// atomic data (Copy)
  Charcode(u16),
  /// atomic data (Copy)
  /// note that we currently work with 64-bit integers
  Int(i64),
  /// atomic data (Clone)
  Node(Node),
  // Collections (boxed)
  /// boxed [char]
  Chars(Box<[char]>),
  /// boxed `[Option<char>]`
  Fontmap(Rc<[Option<char>]>),
  /// boxed collection
  Strings(Rc<[SymStr]>),
  /// boxed collection (latexml)
  VecDigested(Vec<crate::Digested>),
  /// the heart of state - a stored Stash table
  Stash(StashTable),
  /// boxed map
  HashString(HashMap<String, String>),
  /// boxed collection - Stored
  VecDequeStored(VecDeque<Stored>),
  /// boxed map - Stored
  HashStored(SymHashMap<Stored>),
  /// boxed map (latexml)
  HashTagData(HashMap<String, Vec<TagData>>),
  /// queued-but-undigested frontmatter commands (Perl: `frontmatter_raw`)
  FrontmatterRaw(Vec<RawFrontmatter>),
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
  Float(Float),
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
  /// latexml object (Rc-wrapped from within)
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
  /// latexml keyval definition
  KeyVal(KeyVal),
  /// latexml keyvals collection
  KeyVals(Rc<KeyVals>),
  /// alignment template (for storing between macros, e.g. deluxetable)
  Template(Rc<crate::alignment::template::Template>),
}

impl fmt::Debug for Stored {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use crate::Stored::*;
    match *self {
      None => write!(f, "None"),
      String(ref s) => arena::with(*s, |str| write!(f, "{str}")),
      Int(ref num) => write!(f, "Stored::Int[{num:?}]"),
      Node(ref n) => write!(f, "Stored::Node[{n:?}]"),
      Chars(ref vs) => write!(f, "Stored::Chars[{vs:?}]"),
      Fontmap(ref vs) => write!(f, "Stored::Fontmap[{vs:?}]"),
      Stash(ref vs) => write!(f, "Stored::Stash[{vs:?}]"),
      Strings(ref vs) => write!(f, "Stored::Strings[{vs:?}]"),
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
      Float(ref float) => write!(f, "Stored::Float[{float:?}]"),
      Glue(ref glue) => write!(f, "Stored::Glue[{glue:?}]"),
      MuGlue(ref glue) => write!(f, "Stored::MuGlue[{glue:?}]"),
      Dimension(ref dimension) => write!(f, "Stored::Dimension[{dimension:?}]"),
      MuDimension(ref dimension) => write!(f, "Stored::MuDimension[{dimension:?}]"),
      VecDigested(ref digested_vec) => write!(f, "VecDigested{digested_vec:?}"),
      VecDequeStored(ref vec) => write!(f, "VecDequeStored{vec:?}"),
      HashStored(ref hos) => write!(f, "HashStored{hos:?}"),
      HashTagData(ref htd) => write!(f, "HashTagData[{htd:?}]"),
      FrontmatterRaw(ref raw) => write!(f, "FrontmatterRaw[{raw:?}]"),
      HashString(ref hstr) => write!(f, "HashStr[{hstr:?}]"),
      Ligature(ref lig) => write!(f, "Ligature[{lig:?}]"),
      KeyVal(ref kv) => write!(f, "KeyVal[{kv:?}]"),
      KeyVals(ref kvs) => write!(f, "KeyVals[{kvs:?}]"),
      Template(ref t) => write!(f, "Template[{t}]"),
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
      Float(ref v) => write!(f, "{v}"),
      Glue(ref v) => write!(f, "{v}"),
      MuGlue(ref v) => write!(f, "{v}"),
      MuDimension(ref v) => write!(f, "{v}"),
      Font(ref font) => write!(f, "{font}"),
      FontDirective(ref font) => write!(f, "{font:?}"),
      String(ref s) => arena::with(*s, |str| write!(f, "{str}")),
      Int(ref s) => write!(f, "{s}"),
      Bool(ref s) => write!(f, "{s}"),
      Tokens(ref s) => write!(f, "{s}"),
      Token(ref s) => write!(f, "{s}"),
      Conditional(ref s) => write!(f, "{s}"),
      Constructor(ref c) => write!(f, "Constructor[{}]", c.get_cs_name()),
      Strings(ref vs) => write!(f, "{}", arena::join(vs, ",")),
      KeyVals(ref kvs) => write!(f, "{kvs}"),
      Template(ref t) => write!(f, "{t}"),
      VecDequeStored(ref v) => {
        // A pushed list renders as its items joined — replacing the Debug repr
        // `VecDequeStored[...]` that leaked into Rhai's LookupString (#315).
        // Option lists (String items, e.g. class_options) are COMMA-separated,
        // faithful to Perl `join(',', @options)` (\@classoptionslist,
        // Package.pm L2457/L2581); hook lists (Token/Digested items:
        // \AtBeginDocument, afterGroup) concatenate with no separator, since a
        // stray comma would corrupt the executed token stream. Kept in lockstep
        // with the `Stored -> Option<Tokens>` reversion below.
        let mut first = true;
        for item in v {
          if !first && matches!(item, String(_)) {
            write!(f, ",")?;
          }
          write!(f, "{item}")?;
          first = false;
        }
        Ok(())
      },
      None => write!(f, "Stored[None]"),
      _ => write!(f, "Stored[??]"),
    }
  }
}

/// Note: PartialEq on Stored is *structural*. See WISDOM.md for RegisterType trap.
impl PartialEq for Stored {
  fn eq(&self, other: &Stored) -> bool {
    use crate::Stored::*;
    match *self {
      None => matches!(other, None),
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
      Chars(ref vs) => {
        if let Chars(vs2) = other {
          **vs == **vs2
        } else {
          false
        }
      },
      Fontmap(ref vs) => {
        if let Fontmap(vs2) = other {
          *vs == *vs2
        } else {
          false
        }
      },
      Strings(ref vs) => {
        if let Strings(vs2) = other {
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
      Float(ref n) => {
        if let Float(n2) = other {
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
              if let Some(item2) = hs2.get_sym(*key) {
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
      FrontmatterRaw(ref raw) => {
        if let FrontmatterRaw(raw2) = other {
          *raw == *raw2
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
      KeyVal(ref kv) => {
        if let KeyVal(kv2) = other {
          *kv == *kv2
        } else {
          false
        }
      },
      KeyVals(ref kvs) => {
        if let KeyVals(kvs2) = other {
          Rc::ptr_eq(kvs, kvs2)
        } else {
          false
        }
      },
      Template(ref t) => {
        if let Template(t2) = other {
          Rc::ptr_eq(t, t2)
        } else {
          false
        }
      },
    }
  }
}

// SAFETY: `Stored` contains `Rc`/`RefCell` (which are !Send/!Sync by default)
// because it embeds libxml::tree::Node and other reference-counted values.
// This crate's convention is that `State` (and therefore all `Stored`
// values held inside it) is thread-local — each conversion job is pinned
// to exactly one OS thread via `use_{main,std,sty}_state()` in state.rs.
// No `Stored` instance is ever moved across threads at runtime.
// These impls exist to satisfy trait bounds on error paths (e.g. Box<dyn
// Error + Send + Sync>) that transitively require Send/Sync on all inner
// types. The invariant is maintained by construction, not the type system.
unsafe impl Send for Stored {}
unsafe impl Sync for Stored {}
impl Stored {
  /// Zero-alloc equivalent of `self.to_string() == target` for the two
  /// string-carrying variants (`String`, `Tokens`). Falls back to
  /// `to_string()` for everything else, where the Display impl allocates
  /// anyway.
  pub fn eq_text(&self, target: &str) -> bool {
    match self {
      Stored::String(s) => arena::with(*s, |v| v == target),
      Stored::Tokens(t) => t.eq_text(target),
      Stored::Token(t) => t.with_str(|v| v == target),
      other => other.to_string() == target,
    }
  }

  /// Zero-alloc `self.to_string().starts_with(prefix)` for the
  /// string-carrying variants; falls back to `to_string()` for others.
  pub fn starts_with_text(&self, prefix: &str) -> bool {
    match self {
      Stored::String(s) => arena::with(*s, |v| v.starts_with(prefix)),
      Stored::Tokens(t) => t.starts_with_text(prefix),
      Stored::Token(t) => t.with_str(|v| v.starts_with(prefix)),
      other => other.to_string().starts_with(prefix),
    }
  }

  /// Zero-alloc `self.to_string().ends_with(suffix)` for the String /
  /// Token variants (where the entire Display output equals the
  /// interned text). For Tokens, we still walk into a small owned
  /// String — ends_with requires anchoring at the tail and rolling
  /// backward, which needs random access. Others fall back to
  /// `to_string()` (cost paid anyway via the Display impl).
  pub fn ends_with_text(&self, suffix: &str) -> bool {
    match self {
      Stored::String(s) => arena::with(*s, |v| v.ends_with(suffix)),
      Stored::Token(t) => t.with_str(|v| v.ends_with(suffix)),
      other => other.to_string().ends_with(suffix),
    }
  }

  /// helper method that uses `ToString::to_string` to flatten a map with Stored values
  // TODO: Obviously a performance issue, find a way to unify the interfaces where string allocation
  // is completely avoided until serialization in the XML.
  // libxml can accept &str, so as long as we can stay within the interner arena paradigm,
  // we should be allocation free. [end TODO]
  pub fn cast_to_string_hash(in_map: &SymHashMap<Stored>) -> HashMap<String, String> {
    let mut out_map: HashMap<String, String> = HashMap::default();
    for (key, val) in in_map {
      // Use to_attribute() so MuGlue/MuDimension widths are converted to pt
      // (e.g. `3.0mu` → `1.66663pt`) before becoming XML attribute strings.
      // Mirror Perl `attributeformat` which uses `ptValue` for mu-typed
      // lengths in attribute context.
      out_map.insert(arena::to_string(*key), val.to_attribute());
    }
    out_map
  }
  /// Dynamic dispatch for Definition's `read_arguments`,
  /// to circumvent the limitations of using trait objects with `Rc<Definition>`
  pub fn read_arguments(&self) -> Result<Vec<ArgWrap>> {
    match self {
      Stored::Conditional(entry) => entry.read_arguments(),
      Stored::Constructor(entry) => entry.read_arguments(),
      Stored::Expandable(entry) => entry.read_arguments(),
      Stored::MathPrimitive(entry) => entry.read_arguments(),
      Stored::Primitive(entry) => entry.read_arguments(),
      Stored::Register(entry) => entry.read_arguments(),
      e => Err(s!(".read_arguments not defined for stored variant {:?}", e).into()),
    }
  }
  /// Uses `NumericOps::to_attribute` for Stored values supporting it, otherwise
  /// `ToString::to_string`
  pub fn to_attribute(&self) -> String {
    match self {
      Stored::Dimension(v) => v.to_attribute(),
      Stored::Number(v) => v.to_attribute(),
      Stored::MuDimension(v) => v.to_attribute(),
      Stored::Glue(v) => v.to_attribute(),
      Stored::MuGlue(v) => v.to_attribute(),
      other => other.to_string(),
    }
  }
  pub fn to_definition(&self) -> Option<Rc<dyn Definition>> {
    match self {
      Stored::Primitive(defn) => Some(defn.clone()),
      Stored::MathPrimitive(defn) => Some(defn.clone()),
      Stored::Conditional(defn) => Some(defn.clone()),
      Stored::Register(defn) => Some(defn.clone()),
      Stored::Expandable(defn) => Some(defn.clone()),
      Stored::Constructor(defn) => Some(defn.clone()),
      _ => None,
    }
  }
}

impl From<bool> for Stored {
  fn from(value: bool) -> Self { Stored::Bool(value) }
}

impl From<bool> for &Stored {
  fn from(value: bool) -> Self { if value { &STORED_TRUE } else { &STORED_FALSE } }
}

impl From<Cow<'_, str>> for Stored {
  fn from(value: Cow<'_, str>) -> Self { Stored::String(arena::pin(value)) }
}
impl From<String> for Stored {
  fn from(value: String) -> Self { Stored::String(arena::pin(value)) }
}
impl From<SymStr> for Stored {
  fn from(value: SymStr) -> Self { Stored::String(value) }
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

// TODO: Should we add a lot more numeric nuance to the Store ?
impl From<u8> for Stored {
  fn from(value: u8) -> Self { Stored::Int(value as i64) }
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
  fn from(value: crate::Digested) -> Self { Stored::Digested(value) }
}

impl From<&crate::Digested> for Stored {
  fn from(value: &crate::Digested) -> Self { Stored::Digested(value.clone()) }
}

impl<T> From<Option<T>> for Stored
where T: Into<Stored> + Sized
{
  fn from(value_opt: Option<T>) -> Self {
    match value_opt {
      None => Stored::None,
      Some(v) => v.into(),
    }
  }
}

impl<'a> From<Cow<'a, crate::Digested>> for Stored {
  fn from(value: Cow<'a, crate::Digested>) -> Self { Stored::Digested(value.into_owned()) }
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

impl From<Cow<'_, Font>> for Stored {
  fn from(value: Cow<Font>) -> Self { Rc::new((*value).clone()).into() }
}

impl From<Number> for Stored {
  fn from(value: Number) -> Self { Stored::Number(value) }
}
impl From<Dimension> for Stored {
  fn from(value: Dimension) -> Self { Stored::Dimension(value) }
}
impl From<MuDimension> for Stored {
  fn from(value: MuDimension) -> Self { Stored::MuDimension(value) }
}
impl From<Glue> for Stored {
  fn from(value: Glue) -> Self { Stored::Glue(value) }
}
impl From<MuGlue> for Stored {
  fn from(value: MuGlue) -> Self { Stored::MuGlue(value) }
}

impl<'a> From<&'a Token> for Stored {
  fn from(value: &'a Token) -> Self { Stored::Token(*value) }
}

impl From<Alignment> for Stored {
  fn from(a: Alignment) -> Self { Stored::Digested(crate::Digested::from(a)) }
}

impl From<Box<[char]>> for Stored {
  fn from(value: Box<[char]>) -> Self { Stored::Chars(value) }
}
impl From<Rc<[Option<char>]>> for Stored {
  fn from(value: Rc<[Option<char>]>) -> Self { Stored::Fontmap(value) }
}

impl From<Rc<[SymStr]>> for Stored {
  fn from(value: Rc<[SymStr]>) -> Self { Stored::Strings(value) }
}

impl From<Vec<String>> for Stored {
  fn from(value: Vec<String>) -> Self { Stored::Strings(value.iter().map(arena::pin).collect()) }
}

impl<'a> From<Vec<&'a str>> for Stored {
  fn from(value: Vec<&'a str>) -> Self { Stored::Strings(value.iter().map(arena::pin).collect()) }
}

impl From<Vec<Token>> for Stored {
  fn from(value: Vec<Token>) -> Self { Stored::Tokens(Tokens::new(value)) }
}

impl From<Vec<crate::Digested>> for Stored {
  fn from(value: Vec<crate::Digested>) -> Self { Stored::VecDigested(value) }
}

impl From<HashMap<String, String>> for Stored {
  fn from(value: HashMap<String, String>) -> Self { Stored::HashString(value) }
}

impl From<VecDeque<Stored>> for Stored {
  fn from(value: VecDeque<Stored>) -> Self { Stored::VecDequeStored(value) }
}

impl From<HashMap<SymStr, Stored>> for Stored {
  fn from(value: HashMap<SymStr, Stored>) -> Self { Stored::HashStored(SymHashMap(value)) }
}
impl From<SymHashMap<Stored>> for Stored {
  fn from(value: SymHashMap<Stored>) -> Self { Stored::HashStored(value) }
}

// TODO: What is the right interface here? Should we really commit to SymStr?
// Or is it too distracting from a developer perspective and String should be allowed more widely?
impl From<HashMap<String, Stored>> for Stored {
  fn from(str_hash: HashMap<String, Stored>) -> Self {
    let mut arena_value = HashMap::default();
    for (key, value) in str_hash {
      arena_value.insert(arena::pin(key), value);
    }
    Stored::HashStored(SymHashMap(arena_value))
  }
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
      RegisterValue::Pair(_) => Stored::None, // TODO: add Stored::Pair
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

impl From<KeyVal> for Stored {
  fn from(kv: KeyVal) -> Stored { Stored::KeyVal(kv) }
}

impl From<KeyVals> for Stored {
  fn from(kvs: KeyVals) -> Stored { Stored::KeyVals(Rc::new(kvs)) }
}

impl From<crate::alignment::template::Template> for Stored {
  fn from(t: crate::alignment::template::Template) -> Stored { Stored::Template(Rc::new(t)) }
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

impl From<&Stored> for bool {
  fn from(value: &Stored) -> bool {
    // Mirror Perl's `if ($val)` truthiness: defined-and-nonzero is true,
    // numeric-zero is false. Without the numeric-zero check, registers
    // initialized to 0 (e.g. `\globaldefs` default, `\count255` unset)
    // would read as "set" via `lookup_bool`, breaking flag-style probes.
    match value {
      Stored::Bool(b) => *b,
      Stored::Int(0) => false,
      Stored::Number(n) if n.0 == 0 => false,
      _ => true,
    }
  }
}

impl From<&Stored> for String {
  fn from(value: &Stored) -> String {
    match value {
      Stored::String(v) => arena::to_string(*v),
      // Flatten a pushed list to the concatenation of its item strings (via the
      // `Display` arm) rather than leaking the `{v:?}` Debug repr
      // `VecDequeStored[...]` into `lookup_string` / Rhai's LookupString (#315).
      Stored::VecDequeStored(_) => s!("{value}"),
      v => s!("{v:?}"),
    }
  }
}

impl<'a> From<&'a Stored> for Option<&'a VecDeque<Stored>> {
  fn from(value: &'a Stored) -> Option<&'a VecDeque<Stored>> {
    match value {
      Stored::VecDequeStored(v) => Some(v),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Rc<Font>> {
  fn from(value: &'a Stored) -> Option<Rc<Font>> {
    match value {
      Stored::Font(f) => Some(Rc::clone(f)),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Number> {
  fn from(value: &'a Stored) -> Option<Number> {
    match value {
      Stored::Number(n) => Some(*n),
      Stored::Dimension(n) => Some(Number::new(n.value_of())),
      Stored::Glue(n) => Some(Number::new(n.value_of())),
      Stored::MuDimension(n) => Some(Number::new(n.value_of())),
      Stored::MuGlue(n) => Some(Number::new(n.value_of())),
      other => {
        eprintln!("TODO: auto-cast of Stored to Number attempted on {other:?}");
        None
      },
    }
  }
}

// MuGlue/MuDimension store raw mu in fixpoint units (1mu = 1/18 em).
// Convert to scaled-pt by mirroring Perl `MuGlue::spValue` →
// `fixpoint(mu/UNITY, MUWidth)` where `MUWidth = int(size * emwidth /
// 18)`. The two-step integer truncation in Perl is load-bearing: a
// single-step `(mu * size / 18)` gives a slightly larger value (109226
// vs 109219 for 3mu at 10pt), and Knuth's `print_scaled` then formats
// "1.66666pt" instead of the expected "1.66663pt". See
// LaTeXML/lib/LaTeXML/Common/Font.pm:580 (getMUWidth) and
// Core/MuGlue.pm spValue.
fn mu_to_pt_value(mu_val: i64) -> i64 {
  let fs = crate::state::lookup_font()
    .and_then(|f| f.get_size())
    .unwrap_or(10.0);
  let unity = crate::common::numeric_ops::UNITY_F64;
  // MUWidth = int(font_size * emwidth(=1.0*UNITY) / 18)
  let muwidth = (fs * unity / 18.0) as i64;
  // fixpoint(mu/UNITY, MUWidth) ≈ (mu_val * muwidth / UNITY).trunc()
  ((mu_val as f64 * muwidth as f64 / unity).trunc()) as i64
}

impl<'a> From<&'a Stored> for Option<Dimension> {
  fn from(value: &'a Stored) -> Option<Dimension> {
    match value {
      Stored::Dimension(n) => Some(*n),
      Stored::Number(n) => Some(Dimension::new(n.value_of())),
      Stored::Glue(n) => Some(Dimension::new(n.value_of())),
      Stored::MuDimension(n) => Some(Dimension::new(mu_to_pt_value(n.value_of()))),
      Stored::MuGlue(n) => Some(Dimension::new(mu_to_pt_value(n.value_of()))),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Glue> {
  fn from(value: &'a Stored) -> Option<Glue> {
    match value {
      Stored::Dimension(n) => Some(Glue::new(n.value_of())),
      Stored::Number(n) => Some(Glue::new(n.value_of())),
      Stored::MuDimension(n) => Some(Glue::new(mu_to_pt_value(n.value_of()))),
      Stored::MuGlue(n) => Some(Glue::new(mu_to_pt_value(n.value_of()))),
      Stored::Glue(n) => Some(*n),
      _ => None,
    }
  }
}

impl From<Stored> for Option<Tokens> {
  fn from(value: Stored) -> Option<Tokens> {
    match value {
      Stored::String(sym) => Some(mouth::tokenize_internal(&arena::to_string(sym))),
      Stored::Token(ts) => Some(Tokens::new(vec![ts])),
      Stored::Tokens(ts) => Some(ts),
      // Digested: revert to tokens (needed for \AtBeginDocument hooks that
      // store Digested arguments via push_value)
      Stored::Digested(d) => {
        use crate::common::object::Object;
        d.revert().ok()
      },
      Stored::VecDequeStored(vdq) => {
        // Revert the queue to a single Tokens. Option lists (String items, e.g.
        // class_options) are COMMA-separated — faithful to Perl
        // `Explode(join(',', @options))` (\@classoptionslist, Package.pm
        // L2457/L2581). Token/Digested items (hook lists like
        // @at@begin@document / afterGroup, pushed as `$op->unlist` token
        // objects) concatenate with NO separator — a stray comma would corrupt
        // the executed token stream. (The two never mix in one key: only the
        // Rhai LookupTokens/LookupString bindings revert a String-item queue;
        // internal option processing reads it via `lookup_vecdeque`.)
        let mut collected: Vec<Token> = Vec::new();
        for item in vdq {
          if matches!(item, Stored::String(_)) && !collected.is_empty() {
            collected.push(T_OTHER!(","));
          }
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
      Stored::Register(reg) => Some(Rc::clone(reg)),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<Catcode> {
  fn from(value: &'a Stored) -> Option<Catcode> {
    match value {
      Stored::Catcode(cc) => Some(*cc),
      _ => None,
    }
  }
}

impl<'a> From<&'a Stored> for Option<&'a [char]> {
  fn from(value: &'a Stored) -> Option<&'a [char]> {
    match value {
      Stored::Chars(cc) => Some(cc),
      _ => None,
    }
  }
}

impl From<&Stored> for Option<Rc<[Option<char>]>> {
  fn from(value: &Stored) -> Option<Rc<[Option<char>]>> {
    match value {
      Stored::Fontmap(cc) => Some(Rc::clone(cc)),
      _ => None,
    }
  }
}
impl From<Stored> for Option<Rc<[Option<char>]>> {
  fn from(value: Stored) -> Option<Rc<[Option<char>]>> { (&value).into() }
}

impl<'a> From<&'a Stored> for Option<RegisterValue> {
  fn from(value: &'a Stored) -> Option<RegisterValue> {
    match value {
      Stored::Number(v) => Some(RegisterValue::Number(*v)),
      Stored::Dimension(v) => Some(RegisterValue::Dimension(*v)),
      Stored::Glue(v) => Some(RegisterValue::Glue(*v)),
      Stored::MuGlue(v) => Some(RegisterValue::MuGlue(*v)),
      Stored::Token(v) => Some(RegisterValue::Token(*v)),
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
      Stored::Token(t) => *t,
      Stored::String(text) => Token {
        text: *text,
        code: Catcode::CS,
        #[cfg(feature = "token-locators")]
        loc: 0,
      },
      t => {
        let message = s!("dangerous cast to CS for {:?}", t);
        Warn!("stored", "cast", message);
        T_CS!(t.to_string())
      }, /* TODO, is this the right place to default to CS? Do we need a
          * custom method instead? */
    }
  }
}
