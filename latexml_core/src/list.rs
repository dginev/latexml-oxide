use libxml::tree::Node;
use std::borrow::Cow;
use std::fmt;

use crate::common::arena::SymHashMap as HashMap;
use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::document::Document;

use crate::common::numeric_ops::NumericOps;
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, TexMode};

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Clone, Default)]
pub struct List {
  pub boxes:      Vec<Digested>,
  pub mode:       Option<TexMode>,
  pub font:       Option<Font>,
  pub locator:    Locator,
  pub properties: HashMap<Stored>,
}

impl fmt::Debug for List {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      self
        .boxes
        .iter()
        .map(|d| d.stringify())
        .collect::<Vec<_>>()
        .join(", ")
    )
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
    self.boxes.len() == other.boxes.len()
      && self
        .boxes
        .iter()
        .zip(other.boxes.iter())
        .all(|(box1, box2)| box1 == box2)
  }
}

impl Object for List {
  fn stringify(&self) -> String { format!("List[{self:?}]") }
  fn get_locator(&self) -> Locator { self.locator }

  fn revert(&self) -> Result<Tokens> {
    let mut reverted = Vec::new();
    for tbox in self.boxes.iter() {
      reverted.extend(tbox.revert()?.unlist());
    }
    Ok(Tokens::new(reverted))
  }
}
impl BoxOps for List {
  fn unlist(&self) -> Vec<Digested> { self.boxes.clone() }
  fn unlist_ref(&self) -> Vec<Cow<'_, Digested>> { self.boxes.iter().map(Cow::Borrowed).collect() }
  fn get_properties(&self) -> &HashMap<Stored> { &self.properties }
  fn get_property(&self, key: &str) -> Option<Cow<'_, Stored>> {
    self.properties.get(key).map(Cow::Borrowed)
  }
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<Stored>) -> R {
    caller(&self.properties)
  }
  fn get_properties_mut(&mut self) -> &mut HashMap<Stored> { &mut self.properties }
  fn set_property<T: Into<Stored>>(&mut self, key: &str, value: T) {
    self.properties.insert(key, value.into());
  }
  fn get_string(&self) -> Result<Cow<'_, str>> { Ok(Cow::Owned(self.to_string())) }
  /// NOTE: No longer used; Document->absorb bypasses this for stack efficiency.
  fn be_absorbed(&self, _document: &mut Document) -> Result<Vec<Node>> { todo!() }

  fn get_font(&self) -> Result<Option<Cow<'_, Font>>> { Ok(self.font.as_ref().map(Cow::Borrowed)) }
  fn compute_size(&self, options: HashMap<Stored>) -> Result<(Dimension, Dimension, Dimension)> {
    let font = self.font.as_ref().cloned().unwrap_or_else(Font::text_default);
    // Perl: horizontal mode Lists use paragraph layout with wrapwidth = width property
    let is_paragraph = matches!(self.mode, Some(TexMode::Text));
    let (wd, ht, dp) = font.compute_boxes_size(&self.boxes, options)?;
    if is_paragraph {
      // Perl: for paragraph layout, line width = stored width property (\hsize)
      if let Some(Stored::Dimension(d)) = self.properties.get("width") {
        return Ok((Dimension::new(d.value_of()), ht, dp));
      }
    }
    Ok((wd, ht, dp))
  }
}

impl List {
  pub fn new(boxes: Vec<Digested>) -> Self {
    // while (defined($bx = shift(@bxs)) && (!defined $locator)) {
    //   $locator = $bx->getLocator unless defined $locator; }
    // TODO: Should the locators be an Option<> type? Or can we test for the default here, since
    // it's rare? Hmmmm
    let mut locator: Locator = Locator::default();
    for bx in boxes.iter().rev() {
      let bx_locator = bx.get_locator();
      if bx_locator != locator {
        // not the default!
        locator = bx_locator;
        break;
      }
    }
    // Maybe the most representative font for a List is the font of the LAST box (that _has_ a
    // font!) ???
    let mut font: Option<Font> = None;
    for bx in boxes.iter().rev() {
      if let Some(bx_font) = bx
        .get_font()
        .expect("getting a font should go well during List construction")
      {
        font = Some(bx_font.into_owned());
        break;
      }
    }
    List {
      boxes,
      font,
      mode: None,
      locator,
      properties: HashMap::default(),
    }
  }

  pub fn is_empty(&self) -> bool {
    // 1. A space-like thing
    // 2. empty contents
    self.get_property_bool("isEmpty")
      || self.get_property_bool("isSpace")
      || self
        .boxes
        .iter()
        .all(|item| item.is_empty().unwrap_or(false))
  }
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
