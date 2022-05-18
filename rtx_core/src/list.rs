use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::document::Document;
use crate::state::State;

use crate::tokens::Tokens;
use crate::{BoxOps, Digested, TexMode};

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Clone, Default, PartialEq)]
pub struct List {
  pub boxes: Vec<Digested>,
  pub mode: Option<TexMode>,
  pub font: Option<Font>,
  pub locator: Locator,
}

impl fmt::Debug for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\nList[")?;
    for tbox in &self.boxes {
      writeln!(f, "  {:?}", tbox)?;
    }
    writeln!(f, "]({:?})", self.mode)
  }
}

impl fmt::Display for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for inner in self.boxes.iter() {
      write!(f, "{}", inner)?;
    }
    Ok(())
  }
}

impl Object for List {
  fn get_locator(&self) -> Option<Cow<Locator>> { Some(Cow::Borrowed(&self.locator)) }

  fn revert(&self, state: &mut State) -> Result<Tokens> {
    let mut reverted = Vec::new();
    for tbox in self.boxes.iter() {
      reverted.extend(tbox.revert(state)?.unlist());
    }
    Ok(Tokens::new(reverted))
  }
}
impl BoxOps for List {
  fn unlist(&self) -> Vec<Digested> { self.boxes.clone() }
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { unimplemented!() }
  fn get_property(&self, _key: &str, _state: &mut State) -> Option<Cow<Stored>> { unimplemented!() }
  fn get_property_bool(&self, _key: &str) -> bool { false }

  /// NOTE: No longer used; Document->absorb bypasses this for stack efficiency.
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> { unimplemented!() }

  fn get_font(&self) -> Option<Cow<Font>> { self.font.as_ref().map(Cow::Borrowed) }
}

impl List {
  pub fn new(boxes: Vec<Digested>) -> Self {
    // while (defined($bx = shift(@bxs)) && (!defined $locator)) {
    //   $locator = $bx->getLocator unless defined $locator; }
    // TODO: Should the locators be an Option<> type? Or can we test for the default here, since it's rare? Hmmmm
    let mut locator: Locator = Locator::default();
    for bx in boxes.iter().rev() {
      if let Some(bx_locator) = bx.get_locator() {
        if *bx_locator != locator {
          // not the default!
          locator = bx_locator.into_owned();
          break;
        }
      }
    }
    // Maybe the most representative font for a List is the font of the LAST box (that _has_ a
    // font!) ???
    let mut font: Option<Font> = None;
    for bx in boxes.iter().rev() {
      if let Some(bx_font) = bx.get_font() {
        font = Some(bx_font.into_owned());
        break;
      }
    }
    List {
      boxes,
      font,
      mode: None,
      locator,
    }
  }

  pub fn is_empty(&self) -> bool { self.boxes.is_empty() }
}

impl From<List> for Result<Vec<Digested>> {
  fn from(list: List) -> Result<Vec<Digested>> {
    let tmp: Digested = list.into();
    tmp.into()
  }
}

impl From<List> for Result<Digested> {
  fn from(value: List) -> Result<Digested> {
    let tmp: Digested = value.into();
    tmp.into()
  }
}
