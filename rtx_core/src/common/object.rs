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

/// Base object for all LaTeXML Objects.
///
/// Defines basic default methods for comparison, printing
// TODO: Reconsider if this is still the right organization
//       since Rust provides more benefit to decoupling e.g. PartialEq from Display
pub trait Object {
  fn stringify(&self) -> String {
    // TODO: remove this and make sure all structs implement
    // something reasonable
    unimplemented!();
  }

  fn isa_box(&self) -> bool { false }
  fn is_expandable(&self) -> bool { false }
  fn is_definition(&self) -> bool { false }
  fn is_comment(&self) -> bool { false }

  // These should really only make sense for Data objects within the
  // processing stream.
  fn be_digested(self, _stomach: &mut Stomach, _state: &mut State) -> Result<Digested>
  where
    Self: Sized,
    Self: Debug,
  {
    panic!("Was it really intended to digest? We don't know how! {self:?} {:?}", self.get_locator());
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
  fn revert(&self, _state: &State) -> Result<Tokens> {
    unimplemented!();
  }
}
