use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use crate::common::dimension::Dimension;
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
#[derive(Clone, Default)]
pub struct List {
  pub boxes: Vec<Digested>,
  pub mode: Option<TexMode>,
  pub font: Option<Font>,
  pub locator: Locator,
  pub properties: HashMap<String, Stored>,
}

impl fmt::Debug for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\nList[")?;
    for tbox in &self.boxes {
      writeln!(f, "  {tbox:?}")?;
    }
    writeln!(f, "]({:?})", self.mode)
  }
}

impl fmt::Display for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for inner in self.boxes.iter() {
      write!(f, "{inner}")?;
    }
    Ok(())
  }
}

impl PartialEq for List {
  fn eq(&self, other: &Self) -> bool {
    self.boxes.len() == other.boxes.len() && self.boxes.iter().zip(other.boxes.iter()).all(|(box1, box2)| box1 == box2)
  }
}

impl Object for List {
  fn get_locator(&self) -> Option<Cow<Locator>> { Some(Cow::Borrowed(&self.locator)) }

  fn revert(&self, state: &State) -> Result<Tokens> {
    let mut reverted = Vec::new();
    for tbox in self.boxes.iter() {
      reverted.extend(tbox.revert(state)?.unlist());
    }
    Ok(Tokens::new(reverted))
  }
}
impl BoxOps for List {
  fn unlist(&self) -> Vec<Digested> { self.boxes.clone() }
  fn has_property(&self, key: &str) -> bool { self.properties.contains_key(key) }
  fn get_property_bool(&self, key: &str) -> bool { matches!(self.properties.get(key), Some(Stored::Bool(true))) }
  fn get_properties(&self) -> &HashMap<String, Stored> { &self.properties }
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) { self.properties.insert(key.to_string(), value.into()); }
  fn get_string(&self, state: &State) -> Result<Cow<str>> { Ok(Cow::Owned(self.to_string())) }
  /// NOTE: No longer used; Document->absorb bypasses this for stack efficiency.
  fn be_absorbed(&self, document: &mut Document, state: &mut State) -> Result<()> { unimplemented!() }

  fn get_font(&self, _: &mut State) -> Result<Option<Cow<Font>>> { Ok(self.font.as_ref().map(Cow::Borrowed)) }
  fn compute_size(&self, options: HashMap<String, Stored>, state: &mut State) -> Result<(Dimension, Dimension, Dimension)> {
    Ok(match &self.font {
      Some(f) => f.compute_boxes_size(&self.boxes, options, state)?,
      _ => Font::text_default().compute_boxes_size(&self.boxes, options, state)?,
    })
  }
}

impl List {
  pub fn new(boxes: Vec<Digested>, state: &mut State) -> Self {
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
      if let Some(bx_font) = bx.get_font(state).expect("getting a font should go well during List construction") {
        font = Some(bx_font.into_owned());
        break;
      }
    }
    List {
      boxes,
      font,
      mode: None,
      locator,
      properties: HashMap::new(),
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
