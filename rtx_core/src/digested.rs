//! Interface layer for the full range of digested objects
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, RwLock}; //,RwLockReadGuard,RwLockWriteGuard};
                              //use once_cell::sync::Lazy;
use libxml::tree::Node;
use string_interner::symbol::SymbolU32;

use crate::comment::Comment;
use crate::common::arena;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::register::RegisterValue;
use crate::document::Document;
use crate::keyvals::KeyVals;
use crate::list::List;
use crate::state::State;
use crate::tbox::Tbox;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::BoxOps;

/// An `Arc`-guarded abstraction for any object encountered at the "digested" phase of processing
// Each variant is wrapped in an `Arc`, for cheap(er) cloning when passing around
// these objects to various auxiliary state (e.g. bookkeeping current box),
// but also for repeatedly passing them as owned into binding closures
// while also storing them in their owner Box.
//
// This model is incredibly hard to achieve with lifetimes, so
// we employ reference counting instead (close to their original Perl design).
// A strict OO-hierarchy of object ownership (with no auxiliary state metadata)
// would allow a Rust-like redesign. But it could be too hard to achieve in practice.
#[derive(Clone)]
pub struct Digested(Arc<DigestedData>);
/// These are all kinds of data which we consider officially supported
/// as outputs from the digestion phase of TeX, i.e. from invoking a token.
pub enum DigestedData {
  /// A TeX Box
  TBox(Tbox),
  /// A TeX Whatsit (with interior mutability, for setters invoked while stored in state)
  Whatsit(RwLock<Whatsit>),
  /// A list of Digested data
  List(List),
  /// Raw Tokens that were postponed to the digestion phase uninvoked/undigested
  Postponed(Tokens),
  /// A LaTeX-like digested key-value map
  KeyVals(KeyVals),
  /// A TeX-like `RegisterValue` (e.g. a Dimension or Glue)
  RegisterValue(RegisterValue),
  /// A TeX comment
  Comment(Comment),
}

// Digested and DigestedData are transparent for debugging -- just show the inner data
impl fmt::Debug for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", *self.0) }
}
impl fmt::Debug for DigestedData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use DigestedData::*;
    match self {
      TBox(v) => write!(f, "{v:?}"),
      Whatsit(v) => write!(f, "{v:?}"),
      List(v) => write!(f, "{v:?}"),
      Postponed(v) => write!(f, "{v:?}"),
      KeyVals(v) => write!(f, "{v:?}"),
      RegisterValue(v) => write!(f, "{v:?}"),
      Comment(v) => write!(f, "{v:?}"),
    }
  }
}

impl PartialEq for Digested {
  fn eq(&self, other: &Digested) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref tb) => {
        if let TBox(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      Whatsit(ref tb) => {
        if let Whatsit(ref tb2) = *other.0 {
          *tb.read().unwrap() == *tb2.read().unwrap()
        } else {
          false
        }
      },
      List(ref tb) => {
        if let List(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      Postponed(ref tb) => {
        if let Postponed(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      KeyVals(ref tb) => {
        if let KeyVals(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      RegisterValue(ref tb) => {
        if let RegisterValue(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
      Comment(ref tb) => {
        if let Comment(ref tb2) = *other.0 {
          tb == tb2
        } else {
          false
        }
      },
    }
  }
}

// Important: we need to postpone the creation of a box until a time where
// we have the most current font information
impl<'a> From<&'a String> for Digested {
  fn from(value: &'a String) -> Digested {
    Digested(Arc::new(DigestedData::Postponed(Tokens::new(
      ExplodeText!(value),
    ))))
  }
}
impl From<String> for Digested {
  fn from(value: String) -> Digested {
    Digested(Arc::new(DigestedData::Postponed(Tokens::new(
      ExplodeText!(value),
    ))))
  }
}
impl From<SymbolU32> for Digested {
  fn from(sym: SymbolU32) -> Digested {
    let allocated = arena::to_string(sym);
    Digested(Arc::new(DigestedData::Postponed(Tokens::new(
      ExplodeText!(allocated),
    ))))
  }
}

impl From<Tokens> for Digested {
  fn from(value: Tokens) -> Digested { Digested(Arc::new(DigestedData::Postponed(value))) }
}
impl From<Tbox> for Digested {
  fn from(value: Tbox) -> Digested { Digested(Arc::new(DigestedData::TBox(value))) }
}
impl From<List> for Digested {
  fn from(value: List) -> Digested { Digested(Arc::new(DigestedData::List(value))) }
}
impl From<Whatsit> for Digested {
  fn from(value: Whatsit) -> Digested {
    Digested(Arc::new(DigestedData::Whatsit(RwLock::new(value))))
  }
}
impl From<KeyVals> for Digested {
  fn from(value: KeyVals) -> Digested { Digested(Arc::new(DigestedData::KeyVals(value))) }
}
impl From<RegisterValue> for Digested {
  fn from(value: RegisterValue) -> Digested {
    Digested(Arc::new(DigestedData::RegisterValue(value)))
  }
}

impl<'a> From<&'a Digested> for Option<crate::Digested> {
  fn from(value: &'a Digested) -> Option<crate::Digested> { Some(value.clone()) }
}

// impl<'a> From<&'a Digested> for Tokens {
//   fn from(value: &'a Digested) -> Tokens { value.revert(state).unwrap() }
// }
// impl From<Digested> for Tokens {
//   fn from(value: Digested) -> Tokens { value.revert(state).unwrap() }
// }
impl From<Digested> for Result<Digested> {
  fn from(value: Digested) -> Result<Digested> { Ok(value) }
}
impl From<Digested> for Result<Vec<Digested>> {
  fn from(value: Digested) -> Result<Vec<Digested>> { Ok(vec![value]) }
}
impl From<Digested> for Result<Option<Digested>> {
  fn from(value: Digested) -> Result<Option<Digested>> { Ok(Some(value)) }
}

impl Default for Digested {
  fn default() -> Self { Digested(Arc::new(DigestedData::TBox(Tbox::default()))) }
}

impl fmt::Display for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => write!(f, "{b}"),
      List(ref l) => write!(f, "{l}"),
      Whatsit(ref w) => write!(f, "{}", w.read().unwrap()),
      Postponed(ref t) => write!(f, "{t}"),
      KeyVals(ref kvs) => write!(f, "{kvs}"),
      Comment(ref c) => write!(f, "{c}"),
      RegisterValue(ref rv) => write!(f, "{rv}"),
    }
  }
}
impl Object for Digested {
  fn stringify(&self) -> String {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.stringify(),
      List(ref l) => l.stringify(),
      Whatsit(ref w) => w.read().unwrap().stringify(),
      Postponed(ref t) => (*t).stringify(),
      KeyVals(ref kvs) => kvs.stringify(),
      Comment(ref c) => c.stringify(),
      RegisterValue(ref rv) => (*rv).stringify(),
    }
  }
  fn get_locator(&self) -> Option<Cow<Locator>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_locator(),
      List(ref l) => l.get_locator(),
      Comment(ref c) => c.get_locator(),
      Whatsit(ref w) => w
        .read()
        .unwrap()
        .get_locator()
        .map(|l| Cow::Owned(l.into_owned())),
      KeyVals(ref kvs) => kvs.get_locator(), // KeyVals locator?
      RegisterValue(ref rv) => rv.get_locator(),
      Postponed(ref _t) => None, // Tokens locator?
    }
  }
  fn revert(&self, state: &State) -> Result<Tokens> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.revert(state),
      List(ref l) => l.revert(state),
      Whatsit(ref w) => w.read().unwrap().revert(state),
      Postponed(ref t) => Ok(t.clone()),
      KeyVals(ref kvs) => kvs.revert(state),
      Comment(ref c) => c.revert(state),
      RegisterValue(ref rv) => rv.revert(state),
    }
  }
}

impl BoxOps for Digested {
  fn unlist(&self) -> Vec<Digested> {
    use DigestedData::*;
    match *self.0 {
      TBox(_) | Whatsit(_) | KeyVals(_) | Comment(_) | Postponed(_) | RegisterValue(_) => {
        vec![self.clone()]
      },
      List(ref l) => l.unlist(),
    }
  }

  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<Vec<Node>> {
    use DigestedData::*;
    match &*self.0 {
      TBox(b) => b.be_absorbed(document, state),
      List(l) => l.be_absorbed(document, state),
      Comment(c) => c.be_absorbed(document, state),
      Whatsit(w) => w.read().unwrap().be_absorbed(document, state),
      KeyVals(kvs) => kvs.be_absorbed(document, state),
      Postponed(_) => unimplemented!(),
      RegisterValue(ref _rv) => unimplemented!(),
    }
  }

  fn get_properties(&self) -> &HashMap<String, Stored> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_properties(),
      List(ref l) => l.get_properties(),
      KeyVals(ref kvs) => kvs.get_properties(),
      Whatsit(ref _w) => unimplemented!(), // Oooof; w.read().unwrap().get_properties(),
      Postponed(_) | RegisterValue(_) | Comment(_) => unimplemented!(),
    }
  }

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    match *self.0 {
      // TODO: This is only possible if we have interior mutability for *ALL* Digested variants
      // i.e. Arc<RwLock<Tbox>>, Arc<RwLock<List>>, etc.
      //
      // Digested::TBox(ref b) => b.set_property(key, value),
      // Digested::List(ref l) => l.set_property(key, value),
      DigestedData::Whatsit(ref w) => w.write().unwrap().set_property(key, value),
      DigestedData::List(ref _l) => Debug!(
        "ignore",
        "set_property",
        None,
        None,
        format!("List::set_property({key},_)")
      ),
      _ => unimplemented!(),
    }
  }

  fn get_property(&self, key: &str) -> Option<Cow<Stored>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_property(key),
      List(ref l) => l.get_property(key),
      Whatsit(ref w) => w
        .read()
        .unwrap()
        .get_property(key)
        .map(|v| Cow::Owned(v.into_owned())),
      _ => unimplemented!(),
    }
  }
  fn get_string(&self, state: &State) -> Result<Cow<str>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_string(state),
      List(ref l) => l.get_string(state),
      Whatsit(ref w) => match w.read().unwrap().get_string(state) {
        Ok(v) => Ok(Cow::Owned(v.into_owned())),
        Err(e) => Err(format!("failed Whatsit get_string: {e}").into()),
      },
      _ => unimplemented!(),
    }
  }
  fn has_property(&self, key: &str) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.has_property(key),
      List(ref l) => l.has_property(key),
      Whatsit(ref w) => w.read().unwrap().has_property(key),
      _ => unimplemented!(),
    }
  }
  fn get_body(&self) -> Option<Digested> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => {
        Error!(
          "digested",
          "get_body",
          self,
          None,
          s!("Called get_body on Box: {:?}", b)
        );
        None
      },
      List(ref l) => {
        Error!(
          "digested",
          "get_body",
          self,
          None,
          s!("Called get_body on List: {:?}", l)
        );
        None
      },
      Whatsit(ref w) => w.read().unwrap().get_body(),
      _ => unimplemented!(),
    }
  }
  fn get_property_bool(&self, key: &str) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_property_bool(key),
      List(ref l) => l.get_property_bool(key),
      Whatsit(ref w) => w.read().unwrap().get_property_bool(key),
      _ => unimplemented!(),
    }
  }
  fn get_font(&self, state: &mut State) -> Result<Option<Cow<Font>>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.get_font(state),
      List(ref l) => l.get_font(state),
      Whatsit(ref w) => Ok(
        w.read()
          .unwrap()
          .get_font(state)?
          .map(|t| Cow::Owned(t.into_owned())),
      ),
      Postponed(ref _tks) => Ok(None),
      _ => unimplemented!(),
    }
  }

  fn compute_size(
    &self,
    options: HashMap<String, Stored>,
    state: &mut State,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.compute_size(options, state),
      List(ref l) => l.compute_size(options, state),
      KeyVals(ref kvs) => kvs.compute_size(options, state),
      Whatsit(ref w) => w.read().unwrap().compute_size(options, state),
      Postponed(_) | RegisterValue(_) | Comment(_) => unimplemented!(),
    }
  }
}

impl Digested {
  /// immutably borrow the inner Digested data
  pub fn data(&self) -> &DigestedData { &self.0 }
  // convenience subset of NumericOps, added here for now as an experiment:
  /// Obtain the i64 value of the digested object, iff it wraps a `RegisterValue`
  pub fn value_of(&self) -> i64 {
    match &*self.0 {
      DigestedData::RegisterValue(rv) => rv.clone().value_of(),
      _ => 0,
    }
  }
  /// Obtain the f64 value of the digested object, iff it wraps a `RegisterValue`
  pub fn pt_value(&self, prec: Option<u8>) -> f64 {
    match &*self.0 {
      DigestedData::RegisterValue(rv) => rv.clone().pt_value(prec),
      _ => 0.0,
    }
  }
  /// Predicate check - true if `any` element of the current object passes the check
  pub fn any<F>(&self, mut check: F) -> bool
  where F: FnMut(&Self) -> bool {
    use DigestedData::*;
    match &*self.0 {
      TBox(_) | Whatsit(_) | Postponed(_) | KeyVals(_) | RegisterValue(_) => check(self),
      Comment(_) => true,
      List(l) => l.boxes.iter().any(check),
    }
  }

  /// Predicate check - true if `all` elements of the current object passes the check
  pub fn all<F>(&self, mut check: F) -> bool
  where F: FnMut(&Self) -> bool {
    use DigestedData::*;
    match &*self.0 {
      TBox(_) | Whatsit(_) | Postponed(_) | KeyVals(_) | RegisterValue(_) => check(self),
      Comment(_) => true,
      List(l) => l.boxes.iter().all(check),
    }
  }

  /// Predicate check - delegates to `.is_empty()` of the underlying data
  pub fn is_empty(&self) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.is_empty(),
      List(ref l) => l.is_empty(),
      Whatsit(ref w) => w.read().unwrap().is_empty(),
      Postponed(ref tks) => tks.is_empty(),
      _ => unimplemented!(),
    }
  }

  /// Provide a way of emulating an `Undigested` argument, by requesting
  /// raw tokens, only when they are preserved -- empty otherwise.
  pub fn raw_tokens(&self) -> Option<&Tokens> {
    match *self.0 {
      DigestedData::Postponed(ref tks) => Some(tks),
      _ => None,
    }
  }

  /// builds an attribute-friendly String form of the digested object, suitable for XML attributes
  pub fn to_attribute(&self) -> String {
    match *self.0 {
      DigestedData::RegisterValue(ref v) => v.to_attribute(),
      _ => self.to_string(),
    }
  }

  /// Reverts a digested object to `Tokens` and extracts a TeX-near string representation of its
  /// content
  pub fn untex(&self, state: &mut State) -> Result<String> { Ok(self.revert(state)?.untex()) }
}
