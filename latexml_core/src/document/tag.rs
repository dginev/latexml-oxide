use std::rc::Rc;

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::{Digested, common::error::*, document::Document};

/// Perl's Tag afterOpen/afterClose closures receive ($document, $node, $box).
/// The $box is the Digested (whatsit) associated with the node, retrieved via getNodeBox.
pub type TagConstructionClosure =
  Rc<dyn Fn(&mut Document, &mut Node, Option<&Digested>) -> Result<()>>;

/// Attribute map of a frontmatter entry (Perl: the `{%attr}` hashref).
pub type TagAttrs = HashMap<String, String>;

/// One frontmatter entry: Perl's `[$tag, {%attr}, @content]`
/// (Base_Utility.pool.ltxml, PR #2767 frontmatter refactor).
#[derive(Debug, Clone, PartialEq)]
pub struct TagData {
  pub tag:     String,
  pub attr:    TagAttrs,
  pub content: Vec<TagContent>,
}

/// The attribute key of the author block's own creator, which the frontmatter emitter writes below a flush's names: no
/// annotation's parent by place ([`TagList::parent_indices`]). The same key as the engine's `frontmatter_api::BLOCK_CREATOR`,
/// which declares it for the frontmatter API.
pub const BLOCK_CREATOR: &str = "_blockcreator";

/// One tag's frontmatter entries, Perl's `@{$frontmatter{$tag}}` (Base_Utility.pool.ltxml), with the indices of the
/// entries an annotation may attach to kept as the list changes — Perl's `grep { ($$_[1]{role} || '') ne 'pending' }`
/// (Base_Utility.pool.ltxml:492), read per annotation instead of rescanned over every entry. The
/// entries are a slice to read; they change only through the list's own methods, which keep the index exact: `push`,
/// `remove`, `retain`, and [`TagList::get_mut`]/[`TagList::last_mut`], whose [`EntryMut`] re-files its entry when an edit
/// changed its kind (debug builds also check the index against the scan at every read).
#[derive(Clone, Default, PartialEq)]
pub struct TagList {
  entries: Vec<TagData>,
  /// the entries not `role=pending`, in order, without the block creators
  parents: Vec<usize>,
  /// the entries not `role=pending` with the [`BLOCK_CREATOR`] key, in order
  block:   Vec<usize>,
}

/// Which of [`TagList`]'s indices an entry goes to: none (a pending stub), the parents, or the block creators.
fn parent_kind(entry: &TagData) -> Option<bool> {
  (entry.attr.get("role").map(String::as_str) != Some("pending"))
    .then(|| entry.attr.contains_key(BLOCK_CREATOR))
}

impl TagList {
  /// The list of `entries`, indexed.
  pub fn from_entries(entries: Vec<TagData>) -> Self {
    let mut list = TagList {
      entries,
      parents: Vec::new(),
      block: Vec::new(),
    };
    list.reindex();
    list
  }
  fn reindex(&mut self) {
    self.parents.clear();
    self.block.clear();
    for (i, entry) in self.entries.iter().enumerate() {
      match parent_kind(entry) {
        Some(false) => self.parents.push(i),
        Some(true) => self.block.push(i),
        None => {},
      }
    }
  }
  /// Append `entry` (Perl's `push(@{$frontmatter{$tag}}, $entry)`).
  pub fn push(&mut self, entry: TagData) {
    let i = self.entries.len();
    match parent_kind(&entry) {
      Some(false) => self.parents.push(i),
      Some(true) => self.block.push(i),
      None => {},
    }
    self.entries.push(entry);
  }
  /// Remove and return the entry at `index` (panics if out of bounds, as `Vec::remove`).
  pub fn remove(&mut self, index: usize) -> TagData {
    let entry = self.entries.remove(index);
    for indices in [&mut self.parents, &mut self.block] {
      if let Ok(at) = indices.binary_search(&index) {
        indices.remove(at);
      }
      let from = indices.partition_point(|&i| i < index);
      for i in &mut indices[from..] {
        *i -= 1;
      }
    }
    entry
  }
  /// Keep only the entries `keep` holds (`Vec::retain`).
  pub fn retain(&mut self, keep: impl FnMut(&TagData) -> bool) {
    self.entries.retain(keep);
    self.reindex();
  }
  /// The entries an annotation may attach to: those not `role=pending`, in order — without the block creators when
  /// `exclude_block` (once a flush wrote one).
  pub fn parent_indices(&self, exclude_block: bool) -> Vec<usize> {
    debug_assert!(
      {
        let scan = TagList::from_entries(self.entries.clone());
        scan.parents == self.parents && scan.block == self.block
      },
      "TagList index out of step with its entries"
    );
    if exclude_block || self.block.is_empty() {
      return self.parents.clone();
    }
    let mut all = Vec::with_capacity(self.parents.len() + self.block.len());
    all.extend_from_slice(&self.parents);
    all.extend_from_slice(&self.block);
    all.sort_unstable();
    all
  }
  /// The entries, as a `Vec`.
  pub fn into_entries(self) -> Vec<TagData> { self.entries }
  /// The entry at `index` to edit in place, `None` out of bounds (`slice::get_mut`): the index follows whatever the
  /// edit does to its role or [`BLOCK_CREATOR`] key.
  pub fn get_mut(&mut self, index: usize) -> Option<EntryMut<'_>> {
    let entry = self.entries.get_mut(index)?;
    Some(EntryMut {
      kind: parent_kind(entry),
      entry,
      index,
      parents: &mut self.parents,
      block: &mut self.block,
    })
  }
  /// The last entry to edit in place, as [`TagList::get_mut`].
  pub fn last_mut(&mut self) -> Option<EntryMut<'_>> {
    let last = self.entries.len().checked_sub(1)?;
    self.get_mut(last)
  }
}

/// One entry of a [`TagList`] edited in place ([`TagList::get_mut`]): when it is dropped, an entry whose kind (pending
/// stub, parent, block creator) the edit changed moves to the index of its new kind — an O(1) check when unchanged.
pub struct EntryMut<'a> {
  entry:   &'a mut TagData,
  index:   usize,
  kind:    Option<bool>,
  parents: &'a mut Vec<usize>,
  block:   &'a mut Vec<usize>,
}

impl std::ops::Deref for EntryMut<'_> {
  type Target = TagData;
  fn deref(&self) -> &TagData { self.entry }
}

impl std::ops::DerefMut for EntryMut<'_> {
  fn deref_mut(&mut self) -> &mut TagData { self.entry }
}

impl Drop for EntryMut<'_> {
  fn drop(&mut self) {
    let kind = parent_kind(self.entry);
    if kind == self.kind {
      return;
    }
    let index = self.index;
    match self.kind {
      Some(false) => sorted_remove(self.parents, index),
      Some(true) => sorted_remove(self.block, index),
      None => {},
    }
    match kind {
      Some(false) => sorted_insert(self.parents, index),
      Some(true) => sorted_insert(self.block, index),
      None => {},
    }
  }
}

fn sorted_remove(indices: &mut Vec<usize>, index: usize) {
  if let Ok(at) = indices.binary_search(&index) {
    indices.remove(at);
  }
}

fn sorted_insert(indices: &mut Vec<usize>, index: usize) {
  if let Err(at) = indices.binary_search(&index) {
    indices.insert(at, index);
  }
}

impl std::ops::Deref for TagList {
  type Target = [TagData];
  fn deref(&self) -> &[TagData] { &self.entries }
}

impl std::fmt::Debug for TagList {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.entries.fmt(f) }
}

impl From<Vec<TagData>> for TagList {
  fn from(entries: Vec<TagData>) -> Self { TagList::from_entries(entries) }
}

impl<'a> IntoIterator for &'a TagList {
  type Item = &'a TagData;
  type IntoIter = std::slice::Iter<'a, TagData>;
  fn into_iter(self) -> Self::IntoIter { self.entries.iter() }
}

/// A content item within a frontmatter entry: digested material (Box, List,
/// Whatsit), recursively a nested `[tag, {attr}, @content]` annotation entry,
/// or the `'place_keeper'` marker used while the entry's own content is still
/// being digested (Perl uses the literal string `'place_keeper'`).
#[derive(Debug, Clone, PartialEq)]
pub enum TagContent {
  PlaceKeeper,
  Box(Digested),
  Entry(TagData),
}

/// A queued-but-not-yet-digested frontmatter command: Perl's
/// `[$tag, {%attr}, $command]` entries in the `frontmatter_raw` state value.
/// Digestion is deferred until `\maketitle` (or similar) so that entries
/// (eg. `\title`, `\author`) can still be redefined & replaced.
pub type RawFrontmatter = (String, TagAttrs, crate::tokens::Tokens);

// Specify the properties of a Node tag.
pub enum TagOptionName {
  AfterOpen,
  AfterOpenEarly,
  AfterOpenLate,
  AfterClose,
  AfterCloseEarly,
  AfterCloseLate,
}

impl TagOptionName {
  pub fn all() -> Vec<TagOptionName> {
    use self::TagOptionName::*;
    vec![
      AfterOpen,
      AfterOpenEarly,
      AfterOpenLate,
      AfterClose,
      AfterCloseEarly,
      AfterCloseLate,
    ]
  }
  pub fn is_prepend(&self) -> bool {
    use self::TagOptionName::*;
    matches!(*self, AfterOpenEarly | AfterCloseEarly)
  }
  pub fn is_append(&self) -> bool {
    use self::TagOptionName::*;
    matches!(
      *self,
      AfterOpen | AfterClose | AfterOpenLate | AfterCloseLate
    )
  }
}

/// How readily an element is auto-opened to hold a child it can contain — Perl's
/// `Tag($tag, autoOpen => $x)`, where any true value counts as 1 and a fraction
/// (`ltx:picture`'s 0.5, latex_constructs.pool.ltxml:4943) makes the element's paths
/// lose to those through a full auto-opener (Document.pm:196-199, 220). Kept as a
/// percentage (100 = 1, 50 = 0.5) for the indirect model's integer arithmetic
/// (`compute_indirect_model`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoOpen(pub u32);

impl From<bool> for AutoOpen {
  fn from(open: bool) -> Self { AutoOpen(if open { 100 } else { 0 }) }
}

impl From<f64> for AutoOpen {
  fn from(fraction: f64) -> Self { AutoOpen((fraction.clamp(0.0, 1.0) * 100.0).round() as u32) }
}

#[derive(Clone, Default)]
pub struct TagOptions {
  pub auto_open:         Option<AutoOpen>,
  pub auto_close:        Option<bool>,
  pub after_open:        Option<Vec<TagConstructionClosure>>,
  pub after_close:       Option<Vec<TagConstructionClosure>>,
  pub after_open_early:  Option<Vec<TagConstructionClosure>>,
  pub after_close_early: Option<Vec<TagConstructionClosure>>,
  pub after_open_late:   Option<Vec<TagConstructionClosure>>,
  pub after_close_late:  Option<Vec<TagConstructionClosure>>,
}

impl TagOptions {
  pub fn get(&self, name: &TagOptionName) -> Option<&Vec<TagConstructionClosure>> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => self.after_open.as_ref(),
      AfterOpenEarly => self.after_open_early.as_ref(),
      AfterOpenLate => self.after_open_late.as_ref(),
      AfterClose => self.after_close.as_ref(),
      AfterCloseEarly => self.after_close_early.as_ref(),
      AfterCloseLate => self.after_close_late.as_ref(),
    }
  }

  pub fn get_mut(&mut self, name: &TagOptionName) -> Option<&mut Vec<TagConstructionClosure>> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => self.after_open.as_mut(),
      AfterOpenEarly => self.after_open_early.as_mut(),
      AfterOpenLate => self.after_open_late.as_mut(),
      AfterClose => self.after_close.as_mut(),
      AfterCloseEarly => self.after_close_early.as_mut(),
      AfterCloseLate => self.after_close_late.as_mut(),
    }
  }

  pub fn remove(&mut self, name: &TagOptionName) -> Vec<TagConstructionClosure> {
    use self::TagOptionName::*;
    let field = match *name {
      AfterOpen => &mut self.after_open,
      AfterOpenEarly => &mut self.after_open_early,
      AfterOpenLate => &mut self.after_open_late,
      AfterClose => &mut self.after_close,
      AfterCloseEarly => &mut self.after_close_early,
      AfterCloseLate => &mut self.after_close_late,
    };
    match field {
      Some(vec) => std::mem::take(vec),
      None => Vec::new(),
    }
  }

  pub fn set(&mut self, name: &TagOptionName, value: Option<Vec<TagConstructionClosure>>) {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => {
        self.after_open = value;
      },
      AfterOpenEarly => {
        self.after_open_early = value;
      },
      AfterOpenLate => {
        self.after_open_late = value;
      },
      AfterClose => {
        self.after_close = value;
      },
      AfterCloseEarly => {
        self.after_close_early = value;
      },
      AfterCloseLate => {
        self.after_close_late = value;
      },
    }
  }

  pub fn prepend(&mut self, name: &TagOptionName, mut value: Vec<TagConstructionClosure>) {
    let drained: Vec<TagConstructionClosure> = match self.get_mut(name) {
      Some(vec) => std::mem::take(vec),
      None => Vec::new(),
    };

    value.extend(drained);

    self.set(name, Some(value));
  }

  pub fn append(&mut self, name: &TagOptionName, value: Vec<TagConstructionClosure>) {
    if self.get(name).is_none() {
      // initialize if needed
      self.set(name, Some(Vec::new()));
    }

    if let Some(vec) = self.get_mut(name) {
      vec.extend(value);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tag_list_entry_edit_refiles_its_kind() {
    let entry = |attrs: &[(&str, &str)]| TagData {
      tag:     "ltx:creator".to_string(),
      attr:    attrs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect(),
      content: Vec::new(),
    };
    let mut list = TagList::from_entries(vec![
      entry(&[]),
      entry(&[("role", "pending")]),
      entry(&[(BLOCK_CREATOR, "1")]),
    ]);
    assert_eq!(list.parent_indices(true), vec![0]);
    assert_eq!(list.parent_indices(false), vec![0, 2]);
    // a stub made a parent, a parent made a stub, a block creator made a parent: each re-filed on drop
    list.get_mut(1).unwrap().attr.remove("role");
    list
      .get_mut(0)
      .unwrap()
      .attr
      .insert("role".into(), "pending".into());
    list.last_mut().unwrap().attr.remove(BLOCK_CREATOR);
    assert_eq!(list.parent_indices(true), vec![1, 2]);
    list
      .get_mut(1)
      .unwrap()
      .attr
      .insert(BLOCK_CREATOR.into(), "1".into());
    assert_eq!(list.parent_indices(true), vec![2]);
    assert_eq!(list.parent_indices(false), vec![1, 2]);
    assert!(list.get_mut(3).is_none());
  }

  #[test]
  fn tag_option_name_all_has_six() {
    let all = TagOptionName::all();
    assert_eq!(all.len(), 6);
  }

  #[test]
  fn tag_option_name_is_prepend_vs_append() {
    assert!(TagOptionName::AfterOpenEarly.is_prepend());
    assert!(TagOptionName::AfterCloseEarly.is_prepend());
    assert!(!TagOptionName::AfterOpen.is_prepend());
    assert!(!TagOptionName::AfterClose.is_prepend());

    assert!(TagOptionName::AfterOpen.is_append());
    assert!(TagOptionName::AfterClose.is_append());
    assert!(TagOptionName::AfterOpenLate.is_append());
    assert!(TagOptionName::AfterCloseLate.is_append());
    assert!(!TagOptionName::AfterOpenEarly.is_append());
    assert!(!TagOptionName::AfterCloseEarly.is_append());
  }

  #[test]
  fn tag_option_name_prepend_and_append_are_disjoint() {
    // Every variant is either prepend OR append, not both.
    for name in TagOptionName::all() {
      let p = name.is_prepend();
      let a = name.is_append();
      assert!(
        p != a,
        "variant classification should be exactly one of prepend/append"
      );
    }
  }

  #[test]
  fn tag_options_default_all_none() {
    let t = TagOptions::default();
    assert!(t.auto_open.is_none());
    assert!(t.auto_close.is_none());
    assert!(t.after_open.is_none());
    assert!(t.after_close.is_none());
    assert!(t.after_open_early.is_none());
    assert!(t.after_close_early.is_none());
    assert!(t.after_open_late.is_none());
    assert!(t.after_close_late.is_none());
  }

  #[test]
  fn tag_options_get_returns_stored_vec() {
    let mut t = TagOptions::default();
    // Initially all None.
    assert!(t.get(&TagOptionName::AfterOpen).is_none());
    // After set, get returns the slot.
    t.after_open = Some(Vec::new());
    assert!(t.get(&TagOptionName::AfterOpen).is_some());
    assert_eq!(t.get(&TagOptionName::AfterOpen).unwrap().len(), 0);
  }

  #[test]
  fn tag_options_get_independent_per_variant() {
    // Setting one slot doesn't populate others.
    let t = TagOptions {
      after_open: Some(Vec::new()),
      ..Default::default()
    };
    assert!(t.get(&TagOptionName::AfterOpen).is_some());
    assert!(t.get(&TagOptionName::AfterClose).is_none());
    assert!(t.get(&TagOptionName::AfterOpenLate).is_none());
    assert!(t.get(&TagOptionName::AfterCloseEarly).is_none());
  }
}
