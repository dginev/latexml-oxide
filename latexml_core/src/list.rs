use std::{borrow::Cow, fmt};

use libxml::tree::Node;

use crate::{
  BoxOps, Digested, TexMode,
  common::{
    arena::SymHashMap as HashMap, dimension::Dimension, error::*, font::Font, locator::Locator,
    object::Object, store::Stored,
  },
  document::Document,
  pin,
  tokens::Tokens,
};

/// Lists can contain any Digested items, such as boxes, whatsits or other lists
#[derive(Clone, Default)]
pub struct List {
  pub boxes:      Vec<Digested>,
  pub mode:       Option<TexMode>,
  pub font:       Option<Font>,
  pub locator:    Option<Locator>,
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
  fn get_locator(&self) -> Option<Locator> { self.locator }

  fn revert(&self) -> Result<Tokens> {
    // Seed with one-token-per-box (a lower bound) so the per-box `extend` loop
    // starts past the first few doubling reallocations (a `grow_one` site).
    let mut reverted = Vec::with_capacity(self.boxes.len());
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
  /// If called directly, absorb each box individually.
  fn be_absorbed(&self, document: &mut Document) -> Result<Vec<Node>> {
    for box_item in &self.boxes {
      document.absorb(box_item, None)?;
    }
    Ok(Vec::new())
  }

  fn get_font(&self) -> Result<Option<Cow<'_, Font>>> { Ok(self.font.as_ref().map(Cow::Borrowed)) }
  fn compute_size(
    &self,
    mut options: HashMap<Stored>,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    let font = self
      .font
      .as_ref()
      .cloned()
      .unwrap_or_else(Font::text_default);
    // Perl: pass mode, vattach, and width from List properties through options
    // so that compute_boxes_size can determine layout mode
    //
    // In Perl, List stores mode as a property string ("horizontal", "restricted_horizontal",
    // "internal_vertical"). In Rust, we have: list.mode = TexMode::Text for horizontal modes.
    // The actual mode string may be stored as a property, OR we infer from context:
    //  - If "width" property is set, this is a horizontal-mode List (paragraph layout)
    //  - Otherwise, default to "restricted_horizontal"
    if let Some(mode_str) = self.properties.get("mode") {
      if let Stored::String(s) = mode_str {
        options.insert("mode", Stored::String(*s));
      }
    } else if self.properties.get("width").is_some() && matches!(self.mode, Some(TexMode::Text)) {
      // Lists with width property set are from horizontal mode (paragraph layout)
      options.insert("mode", Stored::String(pin!("horizontal")));
    }
    if let Some(Stored::String(s)) = self.properties.get("vattach") {
      options.insert("vattach", Stored::String(*s));
    }
    if let Some(width) = self.properties.get("width")
      && options.get("width").is_none()
    {
      options.insert("width", width.clone());
    }
    // Perl #2798 (S6): pass the List's recorded \baselineskip (set by S4 in
    // repack_horizontal) so compute_boxes_size can stack lines with the right
    // inter-line spacing.
    if let Some(baseline) = self.properties.get("baseline")
      && options.get("baseline").is_none()
    {
      options.insert("baseline", baseline.clone());
    }
    font.compute_boxes_size(&self.boxes, options)
  }
}

impl List {
  pub fn new(boxes: Vec<Digested>) -> Self {
    // Perl: `$locator = $bx->getLocator unless defined $locator` — the first
    // box (walking back-to-front) that has a locator. Now that locators are
    // `Option<Locator>`, this is a clean `find_map` (no default-sentinel hack).
    //
    // Under the `token-locators` precision build we instead want the run's full
    // *extent* — the span from the first contributing box's start to the last
    // box's end — so a text run carries its true `(from..to)` range
    // (docs/SOURCE_PROVENANCE.md §3.1.1, Tbox consumer). Boxes are in source
    // order, so folding `new_range` keeps the first `from` and extends to the
    // latest `to`. Gated at *compile time* (not the runtime `source_map`
    // switch): `List::new` runs deep in digestion where `State` is already
    // mutably borrowed, so a `state!()` read here double-borrows the RefCell.
    // Off the feature this stays the byte-identical Perl representative-locator
    // behavior and never touches `State`.
    #[cfg(feature = "token-locators")]
    let locator: Option<Locator> = boxes.iter().fold(None, |acc, bx| match bx.get_locator() {
      Some(l) if l.from_line != 0 => match acc {
        None => Some(l),
        Some(a) => Locator::new_range(a, l).or(Some(a)),
      },
      _ => acc,
    });
    #[cfg(not(feature = "token-locators"))]
    let locator: Option<Locator> = boxes.iter().rev().find_map(|bx| bx.get_locator());
    // Maybe the most representative font for a List is the font of the LAST box (that _has_ a
    // font!) ???
    // Walk boxes back-to-front for the most representative font.
    // A single box whose font resolution errors (e.g. FontDirective::Closure
    // returning Err) shouldn't crash the whole List; treat it as "no font"
    // and keep walking.
    let mut font: Option<Font> = None;
    for bx in boxes.iter().rev() {
      if let Ok(Some(bx_font)) = bx.get_font() {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_default_is_empty() {
    let l = List::default();
    assert!(l.is_empty());
    assert_eq!(l.boxes.len(), 0);
    assert_eq!(l.mode, None);
    assert_eq!(l.font, None);
  }

  #[test]
  fn list_new_from_empty_vec() {
    let l = List::new(vec![]);
    assert!(l.is_empty());
    assert_eq!(l.boxes.len(), 0);
  }

  #[test]
  fn list_display_empty_is_empty_string() {
    let l = List::default();
    assert_eq!(format!("{l}"), "");
  }

  #[test]
  fn list_equality_same_empty() {
    let a = List::default();
    let b = List::default();
    assert_eq!(a, b);
  }

  #[test]
  fn list_stringify_wraps_in_brackets() {
    let l = List::default();
    let s = l.stringify();
    assert!(s.starts_with("List["), "got {s:?}");
    assert!(s.ends_with(']'));
  }

  #[test]
  fn list_get_properties_empty_by_default() {
    let l = List::default();
    assert_eq!(l.get_properties().len(), 0);
  }

  #[test]
  fn list_set_property_persists() {
    let mut l = List::default();
    l.set_property("testkey", Stored::Bool(true));
    assert!(l.get_properties().contains_key("testkey"));
  }

  #[test]
  fn list_revert_empty_is_empty_tokens() {
    let l = List::default();
    let t = l.revert().expect("empty list reverts cleanly");
    assert_eq!(t.len(), 0);
  }

  #[test]
  fn list_unlist_ref_returns_borrowed_boxes() {
    let l = List::default();
    let refs = l.unlist_ref();
    assert_eq!(refs.len(), 0);
  }

  #[test]
  fn list_get_font_default_none() {
    let l = List::default();
    let f = l.get_font().unwrap();
    assert!(f.is_none());
  }
}
