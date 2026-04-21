use crate::Digested;
use crate::common::error::*;
use crate::common::locator::Locator;
use crate::tokens::Tokens;
///======================================================================
/// Exported generic functions for dealing with `LaTeXML`'s objects
///======================================================================
use std::fmt::Debug;

/// Base object for all LaTeXML Objects.
///
/// Defines basic default methods for comparison, printing
pub trait Object {
  fn stringify(&self) -> String { String::from("<unknown>") }

  fn isa_box(&self) -> bool { false }
  fn is_expandable(&self) -> bool { false }
  fn is_definition(&self) -> bool { false }
  fn is_comment(&self) -> bool { false }

  // These should really only make sense for Data objects within the
  // processing stream.
  fn be_digested(self) -> Result<Digested>
  where
    Self: Sized,
    Self: Debug,
  {
    panic!("Was it really intended to digest? We don't know how! {self:?}");
  }
  fn get_locator(&self) -> Locator { Locator::default() }

  /// each concrete object needs to provide its own path back to tokens
  fn revert(&self) -> Result<Tokens> { Ok(Tokens::new(vec![])) }
}
