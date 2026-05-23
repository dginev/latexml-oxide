//! Interface layer for the full range of digested objects
use libxml::tree::Node;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::alignment::Alignment;
use crate::comment::Comment;
use crate::common::arena::SymHashMap as HashMap;
use crate::common::arena::{self, SymStr};
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
use crate::tbox::Tbox;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::{BoxOps, NO_PROPERTIES};

/// An `Rc`-guarded abstraction for any object encountered at the "digested" phase of processing
// Each variant is wrapped in an `Rc`, for cheap(er) cloning when passing around
// these objects to various auxiliary state (e.g. bookkeeping current box),
// but also for repeatedly passing them as owned into binding closures
// while also storing them in their owner Box.
//
// This model is incredibly hard to achieve with lifetimes, so
// we employ reference counting instead (close to their original Perl design).
// A strict OO-hierarchy of object ownership (with no auxiliary state metadata)
// would allow a Rust-like redesign. But it could be too hard to achieve in practice.
#[derive(Clone)]
pub struct Digested(Rc<DigestedData>);
/// These are all kinds of data which we consider officially supported
/// as outputs from the digestion phase of TeX, i.e. from invoking a token.
#[allow(clippy::large_enum_variant)]
// TODO: Investigate if the outer Rc<> wrap of Digested is enough to avoid performance penalties
// from having       the concrete structs in DigestedData vary a lot in size.
pub enum DigestedData {
  /// A TeX Box
  TBox(RefCell<Tbox>),
  /// A TeX Whatsit (with interior mutability, for setters invoked while stored in state)
  Whatsit(RefCell<Whatsit>),
  /// A TeX Alignment (with interior mutability, for setters invoked while stored in state)
  Alignment(Box<RefCell<Alignment>>),
  /// A list of Digested data
  List(RefCell<List>),
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
      Alignment(a) => write!(f, "{a:?}"),
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
          *tb.borrow() == *tb2.borrow()
        } else {
          false
        }
      },
      Alignment(ref tb) => {
        if let Alignment(ref tb2) = *other.0 {
          *tb.borrow() == *tb2.borrow()
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
    Digested(Rc::new(DigestedData::Postponed(Tokens::new(ExplodeText!(
      value
    )))))
  }
}
impl From<String> for Digested {
  fn from(value: String) -> Digested {
    Digested(Rc::new(DigestedData::Postponed(Tokens::new(ExplodeText!(
      value
    )))))
  }
}
impl From<SymStr> for Digested {
  fn from(sym: SymStr) -> Digested {
    let tks = SymExplodeText!(sym);
    Digested(Rc::new(DigestedData::Postponed(Tokens::new(tks))))
  }
}

impl From<Tokens> for Digested {
  fn from(value: Tokens) -> Digested { Digested(Rc::new(DigestedData::Postponed(value))) }
}
impl From<Tbox> for Digested {
  fn from(value: Tbox) -> Digested { Digested(Rc::new(DigestedData::TBox(RefCell::new(value)))) }
}
impl From<List> for Digested {
  fn from(value: List) -> Digested { Digested(Rc::new(DigestedData::List(RefCell::new(value)))) }
}
impl From<Whatsit> for Digested {
  fn from(value: Whatsit) -> Digested {
    Digested(Rc::new(DigestedData::Whatsit(RefCell::new(value))))
  }
}
impl From<Alignment> for Digested {
  fn from(value: Alignment) -> Digested {
    Digested(Rc::new(DigestedData::Alignment(Box::new(RefCell::new(
      value,
    )))))
  }
}
impl From<KeyVals> for Digested {
  fn from(value: KeyVals) -> Digested { Digested(Rc::new(DigestedData::KeyVals(value))) }
}
impl From<RegisterValue> for Digested {
  fn from(value: RegisterValue) -> Digested {
    Digested(Rc::new(DigestedData::RegisterValue(value)))
  }
}
impl From<Comment> for Digested {
  fn from(value: Comment) -> Digested { Digested(Rc::new(DigestedData::Comment(value))) }
}

impl<'a> From<&'a Digested> for Option<crate::Digested> {
  fn from(value: &'a Digested) -> Option<crate::Digested> { Some(value.clone()) }
}

// impl<'a> From<&'a Digested> for Tokens {
//   fn from(value: &'a Digested) -> Tokens { value.revert().unwrap() }
// }
// impl From<Digested> for Tokens {
//   fn from(value: Digested) -> Tokens { value.revert().unwrap() }
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
  fn default() -> Self { Digested(Rc::new(DigestedData::TBox(RefCell::new(Tbox::default())))) }
}

impl fmt::Display for Digested {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => write!(f, "{}", b.borrow()),
      List(ref l) => write!(f, "{}", l.borrow()),
      Whatsit(ref w) => write!(f, "{}", w.borrow()),
      Alignment(ref a) => write!(f, "{}", a.borrow()),
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
      TBox(ref b) => b.borrow().stringify(),
      List(ref l) => l.borrow().stringify(),
      Whatsit(ref w) => w.borrow().stringify(),
      Alignment(ref w) => w.borrow().stringify(),
      Postponed(ref t) => (*t).stringify(),
      KeyVals(ref kvs) => kvs.stringify(),
      Comment(ref c) => c.stringify(),
      RegisterValue(ref rv) => (*rv).stringify(),
    }
  }
  fn get_locator(&self) -> Locator {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.borrow().get_locator(),
      List(ref l) => l.borrow().get_locator(),
      Comment(ref c) => c.get_locator(),
      Whatsit(ref w) => w.borrow().get_locator(),
      Alignment(ref w) => w.borrow().get_locator(),
      KeyVals(ref kvs) => kvs.get_locator(), // KeyVals locator?
      RegisterValue(ref rv) => rv.get_locator(),
      Postponed(ref _t) => Locator::default(), // Tokens locator?
    }
  }
  fn revert(&self) -> Result<Tokens> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.borrow().revert(),
      List(ref l) => l.borrow().revert(),
      Whatsit(ref w) => w.borrow().revert(),
      Alignment(ref w) => w.borrow().revert(),
      Postponed(ref t) => Ok(t.clone()),
      KeyVals(ref kvs) => kvs.revert(),
      Comment(ref c) => c.revert(),
      RegisterValue(ref rv) => rv.revert(),
    }
  }
}

impl BoxOps for Digested {
  fn unlist(&self) -> Vec<Digested> {
    use DigestedData::*;
    match *self.0 {
      TBox(_) | Whatsit(_) | Alignment(_) | KeyVals(_) | Comment(_) | Postponed(_)
      | RegisterValue(_) => {
        vec![self.clone()]
      },
      List(ref l) => l.borrow().unlist(),
    }
  }
  fn unlist_ref(&self) -> Vec<Cow<'_, Digested>> {
    use DigestedData::*;
    match *self.0 {
      TBox(_) | Whatsit(_) | Alignment(_) | KeyVals(_) | Comment(_) | Postponed(_)
      | RegisterValue(_) => {
        vec![Cow::Borrowed(self)]
      },
      List(ref l) => l.borrow().unlist().into_iter().map(Cow::Owned).collect(),
    }
  }

  fn be_absorbed(&self, document: &mut Document) -> Result<Vec<Node>> {
    use DigestedData::*;
    match &*self.0 {
      TBox(b) => b.borrow().be_absorbed(document),
      List(l) => l.borrow().be_absorbed(document),
      Comment(c) => c.be_absorbed(document),
      Whatsit(w) => w.borrow().be_absorbed(document),
      Alignment(w) => w.borrow_mut().be_absorbed_mut(document),
      KeyVals(kvs) => kvs.be_absorbed(document),
      Postponed(_) => Ok(Vec::new()), // Postponed items absorbed silently
      RegisterValue(ref _rv) => Ok(Vec::new()), // Register values not absorbable
    }
  }

  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<Stored>) -> R {
    use DigestedData::*;
    // Defensive `try_borrow`: when a Digested wrapper is mid-`be_absorbed_mut`
    // (which holds an exclusive `borrow_mut`) and document construction
    // recursively asks the SAME node for its properties, an infallible
    // `.borrow()` panics with "RefCell already mutably borrowed". Fall
    // back to NO_PROPERTIES instead — property access during the
    // mid-absorption window is read-only and a missing-properties result
    // is benign (matches Perl: properties default to empty in this state).
    // Witness: 1205.0376 (article + plain-TeX `\AND`/`\at` redefs +
    // align environment) — previously FATAL_101 panic at digested.rs:329,
    // now succeeds.
    match &*self.0 {
      TBox(b) => match b.try_borrow() {
        Ok(b) => caller(b.get_properties()),
        Err(_) => caller(&NO_PROPERTIES),
      },
      List(l) => match l.try_borrow() {
        Ok(l) => caller(l.get_properties()),
        Err(_) => caller(&NO_PROPERTIES),
      },
      Comment(c) => caller(c.get_properties()),
      Whatsit(w) => match w.try_borrow() {
        Ok(w) => caller(w.get_properties()),
        Err(_) => caller(&NO_PROPERTIES),
      },
      Alignment(w) => match w.try_borrow() {
        Ok(w) => caller(w.get_properties()),
        Err(_) => caller(&NO_PROPERTIES),
      },
      KeyVals(_) | Postponed(_) | RegisterValue(_) => caller(&NO_PROPERTIES),
    }
  }
  // Note: get_properties_mut is not implemented, as it would generically require a RefCell
  // around each type of DigestedData. Currently we are trying to keep some immutability guarantees.
  // at the Digested interface

  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    use DigestedData::*;
    match *self.0 {
      // TODO: This is only possible if we have interior mutability for *ALL* Digested variants
      // i.e. Rc<RefCell<Tbox>>, Rc<RefCell<List>>, etc.
      TBox(ref b) => b.borrow_mut().set_property(key, value),
      List(ref l) => l.borrow_mut().set_property(key, value),
      Whatsit(ref w) => w.borrow_mut().set_property(key, value),
      _ => { /* no-op for Comment/Postponed/RegisterValue/KeyVals/Alignment */ },
    }
  }

  fn get_property(&self, key: &str) -> Option<Cow<'_, Stored>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b
        .borrow()
        .get_property(key)
        .map(|v| Cow::Owned(v.into_owned())),
      List(ref l) => l
        .borrow()
        .get_property(key)
        .map(|v| Cow::Owned(v.into_owned())),
      Whatsit(ref w) => w
        .borrow()
        .get_property(key)
        .map(|v| Cow::Owned(v.into_owned())),
      _ => None,
    }
  }
  fn get_string(&self) -> Result<Cow<'_, str>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.borrow().get_string().map(|v| Cow::Owned(v.into_owned())),
      List(ref l) => l.borrow().get_string().map(|v| Cow::Owned(v.into_owned())),
      Whatsit(ref w) => w.borrow().get_string().map(|v| Cow::Owned(v.into_owned())),
      _ => Ok(Cow::Borrowed("")),
    }
  }
  fn has_property(&self, key: &str) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.borrow().has_property(key),
      List(ref l) => l.borrow().has_property(key),
      Whatsit(ref w) => w.borrow().has_property(key),
      _ => false,
    }
  }
  fn get_body(&self) -> Result<Option<Digested>> {
    use DigestedData::*;
    match *self.0 {
      // Perl: Box::getBody returns $self; List::getBody returns $self
      TBox(_) | List(_) => Ok(Some(self.clone())),
      Whatsit(ref w) => w.borrow().get_body(),
      _ => Ok(None),
    }
  }
  fn get_property_bool(&self, key: &str) -> bool {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.borrow().get_property_bool(key),
      List(ref l) => l.borrow().get_property_bool(key),
      Whatsit(ref w) => w.borrow().get_property_bool(key),
      Alignment(_) | KeyVals(_) | Comment(_) | Postponed(_) | RegisterValue(_) => false,
    }
  }
  fn get_font(&self) -> Result<Option<Cow<'_, Font>>> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => Ok(b.borrow().get_font()?.map(|v| Cow::Owned(v.into_owned()))),
      List(ref l) => Ok(l.borrow().get_font()?.map(|v| Cow::Owned(v.into_owned()))),
      Whatsit(ref w) => Ok(w.borrow().get_font()?.map(|t| Cow::Owned(t.into_owned()))),
      Postponed(ref _tks) => Ok(None),
      _ => Ok(None),
    }
  }

  /// Note the difference between calling `compute_size` on a Digested object, and calling it on a
  /// concrete box type. When called on `Digested` it will opt for caching the computed sizes,
  /// but when called on the concrete types it will always compute sizes fresh.
  fn compute_size(&self, options: HashMap<Stored>) -> Result<(Dimension, Dimension, Dimension)> {
    use DigestedData::*;
    match *self.0 {
      TBox(ref b) => b.borrow_mut().compute_size_and_cache(options),
      List(ref l) => l.borrow_mut().compute_size_and_cache(options),
      KeyVals(ref kvs) => kvs.compute_size(options),
      Whatsit(ref w) => w.borrow_mut().compute_size_and_cache(options),
      Alignment(ref w) => w.borrow_mut().compute_size_and_cache(options),
      Postponed(_) | RegisterValue(_) | Comment(_) => {
        Ok((Dimension::new(0), Dimension::new(0), Dimension::new(0)))
      },
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
  /// Obtain a Dimension from the digested object, iff it wraps a `RegisterValue`
  pub fn get_dimension(&self) -> Option<Dimension> {
    match &*self.0 {
      DigestedData::RegisterValue(rv) => Some(Dimension::from(rv)),
      _ => None,
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
      TBox(_) | Whatsit(_) | Alignment(_) | Postponed(_) | KeyVals(_) | RegisterValue(_) => {
        check(self)
      },
      Comment(_) => true,
      List(l) => l.borrow().boxes.iter().any(check),
    }
  }

  /// Predicate check - true if `all` elements of the current object passes the check
  pub fn all<F>(&self, mut check: F) -> bool
  where F: FnMut(&Self) -> bool {
    use DigestedData::*;
    match &*self.0 {
      TBox(_) | Whatsit(_) | Alignment(_) | Postponed(_) | KeyVals(_) | RegisterValue(_) => {
        check(self)
      },
      Comment(_) => true,
      List(l) => l.borrow().boxes.iter().all(check),
    }
  }

  /// Predicate check - delegates to `.is_empty()` of the underlying data
  pub fn is_empty(&self) -> Result<bool> {
    use DigestedData::*;
    Ok(match *self.0 {
      TBox(ref b) => b.borrow().is_empty(),
      List(ref l) => l.borrow().is_empty(),
      Whatsit(ref w) => w.borrow().is_empty()?,
      Postponed(ref tks) => tks.is_empty(),
      _ => false, // Comments, RegisterValues, Alignments, KeyVals are non-empty
    })
  }

  /// Check if all items are "empty" or only spaces or otherwise skippable in a table cell.
  /// Perl: isSkippable (Alignment.pm L484-508)
  pub fn is_skippable(&self) -> bool {
    use DigestedData::*;
    match *self.0 {
      Comment(_) => true,
      TBox(ref b) => {
        let b = b.borrow();
        if b.get_property_bool("isEmpty")
          || b.get_property_bool("isSpace")
          || b.get_property_bool("alignmentSkippable")
        {
          true
        } else {
          // Perl: getString, check if only whitespace
          b.get_string()
            .ok()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
        }
      },
      List(ref l) => l.borrow().boxes.iter().all(|d| d.is_skippable()),
      Whatsit(ref w) => {
        let w = w.borrow();
        if w.get_property_bool("isEmpty")
          || w.get_property_bool("isSpace")
          || w.get_property_bool("alignmentSkippable")
        {
          true
        } else if let Ok(Some(body)) = w.get_body() {
          body.is_skippable()
        } else if let Some(ref prop) = w.get_property("content_box") {
          // Perl: $thing->getProperty('content_box') — for \hbox etc.
          match &**prop {
            Stored::Digested(cb) => cb.is_skippable(),
            _ => false,
          }
        } else {
          false
        }
      },
      Postponed(ref tks) => {
        // Perl checks token catcodes: letters, others, active, CS are NOT skippable
        tks.unlist_ref().iter().all(|t| {
          let cc = t.get_catcode();
          !matches!(
            cc,
            crate::token::Catcode::LETTER
              | crate::token::Catcode::OTHER
              | crate::token::Catcode::ACTIVE
              | crate::token::Catcode::CS
          )
        })
      },
      _ => false,
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
  pub fn untex(&self) -> Result<String> { Ok(self.revert()?.untex()) }

  pub fn alignment_cell(&self) -> Option<&RefCell<Alignment>> {
    if let DigestedData::Alignment(ref alignment) = *self.0 {
      Some(alignment)
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn digested_from_tokens_roundtrip() {
    let ts = Tokens::new(vec![]);
    let d: Digested = ts.into();
    // Default variant is Postponed for raw Tokens.
    match &*d.0 {
      DigestedData::Postponed(_) => {},
      other => panic!("expected Postponed, got {other:?}"),
    }
  }

  #[test]
  fn digested_from_string_is_postponed_tokens() {
    let d: Digested = "abc".to_string().into();
    match &*d.0 {
      DigestedData::Postponed(_) => {},
      other => panic!("expected Postponed, got {other:?}"),
    }
  }

  #[test]
  fn digested_from_tbox_is_tbox_variant() {
    let tb = Tbox::default();
    let d: Digested = tb.into();
    match &*d.0 {
      DigestedData::TBox(_) => {},
      other => panic!("expected TBox, got {other:?}"),
    }
  }

  #[test]
  fn digested_from_list_is_list_variant() {
    let l = List::default();
    let d: Digested = l.into();
    match &*d.0 {
      DigestedData::List(_) => {},
      other => panic!("expected List, got {other:?}"),
    }
  }

  #[test]
  fn digested_from_whatsit_is_whatsit_variant() {
    let w = Whatsit::default();
    let d: Digested = w.into();
    match &*d.0 {
      DigestedData::Whatsit(_) => {},
      other => panic!("expected Whatsit, got {other:?}"),
    }
  }

  #[test]
  fn digested_from_keyvals_is_keyvals_variant() {
    let kv = KeyVals::default();
    let d: Digested = kv.into();
    match &*d.0 {
      DigestedData::KeyVals(_) => {},
      other => panic!("expected KeyVals, got {other:?}"),
    }
  }

  #[test]
  fn digested_clone_shares_rc() {
    // Digested is Rc-wrapped; clone should share the same underlying
    // RefCell, not a deep copy.
    let tb = Tbox::default();
    let a: Digested = tb.into();
    let b = a.clone();
    // Rc strong count is at least 2 now.
    assert!(Rc::strong_count(&a.0) >= 2);
    assert!(Rc::strong_count(&b.0) >= 2);
  }

  #[test]
  fn digested_ref_to_option_some() {
    let d: Digested = Tbox::default().into();
    let o: Option<Digested> = (&d).into();
    assert!(o.is_some());
  }
}
