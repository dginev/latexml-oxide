///======================================================================
/// Exported generic functions for dealing with `LaTeXML`'s objects
///======================================================================
use std::borrow::Cow;
use crate::common::error::*;
use crate::common::locator::Locator;
use crate::tbox::Tbox;

// ======================================================================
// LaTeXML Object
//  Base object for all LaTeXML Objects;
// Defines basic default methods for comparison, printing
// Tried to use overloading, but the Magic methods lead to hard-to-find
// (and occasionally quite serious) performance issues -- at least, if you
// try to have stringify do too much.
// ======================================================================
pub trait Object {
  fn stringify(&self) -> String { unimplemented!(); }

  // Since the next two are used in debugging and error messages,
  // be careful to avoid recursive errors

  // Just how deep of an equality test should this be?
  fn equals<T>(&self, _other: T) -> bool
  where Self: Sized {
    unimplemented!()
  }

  fn to_attribute(&self) -> String { unimplemented!() }

  fn notequals<T>(&self, other: &T) -> bool
  where Self: Sized {
    !self.equals(other)
  }

  fn isa_token(&self) -> bool { false }
  fn isa_box(&self) -> bool { false }
  fn is_expandable(&self) -> bool { false }
  fn is_definition(&self) -> bool { false }

  // These should really only make sense for Data objects within the
  // processing stream.
  // Defaults (probably poor)
  fn be_digested(&self) -> Result<Tbox> { Ok(Tbox::default()) }

  // fn be_absorbed(&self, _document: Document) { unimplemented!() }
  fn get_locator(&self) -> Cow<Locator> { unimplemented!(); }
  fn get_location(&self) -> String {
    let loc = self.get_locator();
    if *loc == Locator::default() {
      String::new() 
    } else {
      s!("at {}", loc)
    }
  }

  // fn unlist<T>(&self) -> Vec<T>
  // where Self: Sized {
  //   unimplemented!()
  // }
}
