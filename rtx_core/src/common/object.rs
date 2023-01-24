use crate::common::error::*;
use crate::common::locator::Locator;
use crate::state::State;
use crate::stomach::Stomach;
use crate::tokens::Tokens;
use crate::Digested;
///======================================================================
/// Exported generic functions for dealing with `LaTeXML`'s objects
///======================================================================
use std::borrow::Cow;
use std::fmt::Debug;

// ======================================================================
// LaTeXML Object
//  Base object for all LaTeXML Objects;
// Defines basic default methods for comparison, printing
// Tried to use overloading, but the Magic methods lead to hard-to-find
// (and occasionally quite serious) performance issues -- at least, if you
// try to have stringify do too much.
// ======================================================================
pub trait Object {
  fn stringify(&self) -> String {
    unimplemented!();
  }

  // Since the next two are used in debugging and error messages,
  // be careful to avoid recursive errors

  // Just how deep of an equality test should this be?
  fn equals<T>(&self, _other: T) -> bool
  where Self: Sized {
    unimplemented!()
  }

  fn notequals<T>(&self, other: &T) -> bool
  where Self: Sized {
    !self.equals(other)
  }

  fn isa_token(&self) -> bool { false }
  fn isa_box(&self) -> bool { false }
  fn is_expandable(&self) -> bool { false }
  fn is_definition(&self) -> bool { false }
  fn is_comment(&self) -> bool { false }

  // These should really only make sense for Data objects within the
  // processing stream.
  fn be_digested(self, stomach: &mut Stomach, state: &mut State) -> Result<Digested>
  where Self: Sized, Self: Debug {
    panic!("Was it really intended to digest? We don't know how! {self:?} {:?}",self.get_locator());
  }

  // fn be_absorbed(&self, _document: Document) { unimplemented!() }
  fn get_locator(&self) -> Option<Cow<Locator>>;
  fn get_location(&self) -> String {
    if let Some(loc) = self.get_locator() {
      if *loc == Locator::default() {
        String::new()
      } else {
        s!("at {}", loc)
      }
    } else {
      String::new()
    }
  }
  /// each concrete object needs to provide its own path back to tokens
  fn revert(&self, state: &mut State) -> Result<Tokens> {
    unimplemented!();
  }
}
