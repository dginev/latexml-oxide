//! BibTeX.pool.ltxml — bibliographic-entry processing for LaTeXML.
//!
//! Perl: `LaTeXML/blib/lib/LaTeXML/Engine/BibTeX.pool.ltxml`. Loaded
//! via `LoadPool('BibTeX')` (e.g. from `amsrefs.sty.ltxml`) or as a
//! preload when the conversion mode is BibTeX
//! (`Common/Config.pm:406`: `unshift(... 'BibTeX.pool')`).
//!
//! ## Status (2026-05-15): Phase 1 foundation
//!
//! Per [`docs/BIBTEX_PORT_PLAN.md`](../../../docs/BIBTEX_PORT_PLAN.md),
//! the full port is split into 6 phases. This file is at **Phase 1**:
//! the `BibEntry` data structure + current-entry tracking + field
//! accessors are in place; no Def\* bindings yet.
//!
//! The Perl pool stores entries in the State Value table under keys
//! like `BIBENTRY@<normalized-key>`. Rust uses a thread-local
//! registry instead to avoid threading a custom `Stored::BibEntry`
//! variant through dump_writer / dump_reader / Stored::PartialEq.
//! (BibEntries don't round-trip through dumps anyway — they're
//! created and consumed entirely within a single conversion.)
//!
//! Bind‐wise still TODO (Phases 2-6):
//! - `\bib` / `\bibitem` family (Perl ~L80-200)
//! - Bib entry-type constructors (`@article`, `@book`, ... ~L220-500)
//! - Field handlers (`bib@field@*` family ~60 macros)
//! - BibTeX special-character handling (~L800-956)
//! - `bibAddToContainer` / `processBibNameList` (need Document API
//!   integration)

use crate::prelude::*;
use latexml_core::tokens::Tokens;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// One BibTeX-style bibliography entry. Mirrors Perl's per-entry hash
/// object (`LaTeXML/Engine/BibTeX.pool.ltxml` uses
/// `$entry->getField(...)`, `$entry->addField(...)` etc.).
///
/// `fields` are *processed* token values (the post-digestion form).
/// `raw_fields` are the *raw* verbatim source strings as they
/// appeared in the input (used by BibTeX-flavoured re-rendering in
/// `\bib@@title`'s case-folding and the `\bib@@origbibentry`
/// roundtrip).
#[derive(Debug, Clone, Default)]
pub struct BibEntry {
  pub key:        String,
  pub entry_type: String,
  fields:         Vec<(String, Tokens)>,
  raw_fields:     Vec<(String, String)>,
}

impl BibEntry {
  pub fn new(key: impl Into<String>, entry_type: impl Into<String>) -> Self {
    Self { key: key.into(), entry_type: entry_type.into(), ..Self::default() }
  }

  /// Perl: `$entry->getField($name)` — first match wins (Perl uses
  /// the same first-set-wins semantics; multiple `addField` calls
  /// for the same name are stored in order, and `getField` returns
  /// the first).
  pub fn get_field(&self, name: &str) -> Option<&Tokens> {
    self.fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
  }

  /// Perl: `$entry->addField($name, $value)`. If the field already
  /// exists, the new value is *appended* — Perl's behaviour for
  /// authors/editors that name-merge across entries. For typical
  /// single-value fields the caller adds at most once.
  pub fn add_field(&mut self, name: impl Into<String>, value: Tokens) {
    self.fields.push((name.into(), value));
  }

  /// Perl: `$entry->getRawField($name)` — raw source string.
  pub fn get_raw_field(&self, name: &str) -> Option<&str> {
    self
      .raw_fields
      .iter()
      .find(|(k, _)| k == name)
      .map(|(_, v)| v.as_str())
  }

  pub fn add_raw_field(&mut self, name: impl Into<String>, value: impl Into<String>) {
    self.raw_fields.push((name.into(), value.into()));
  }

  /// Iterate all field names (in insertion order). Used by
  /// crossref-copy and entry-completion logic.
  pub fn field_names(&self) -> impl Iterator<Item = &str> {
    self.fields.iter().map(|(k, _)| k.as_str())
  }
}

thread_local! {
  /// Map from normalized bibkey (Perl: `NormalizeBibKey(<raw-key>)`)
  /// to the registered entry. Populated by `\bib`'s entry-create
  /// path (Phase 4); read by `current_entry`-based field helpers.
  static BIB_ENTRIES: RefCell<HashMap<String, Rc<RefCell<BibEntry>>>> =
    RefCell::new(HashMap::new());

  /// The normalized key of the entry currently being processed.
  /// Set by the per-entry pipeline at `\bib@entry@<type>@prepare`
  /// time; cleared after `\bib@entry@<type>@complete`.
  static CURRENT_ENTRY_KEY: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Register a fresh `BibEntry` under its normalized key, and set it
/// as the current entry. Returns an `Rc<RefCell<...>>` so callers
/// can hold a handle while the registry retains its copy.
///
/// Perl roughly: `AssignValue('BIBENTRY@'.NormalizeBibKey($key),
/// $entry, 'global'); AssignValue('current_bib_entry', $entry,...)`.
pub fn register_entry(key: &str, entry: BibEntry) -> Rc<RefCell<BibEntry>> {
  use latexml_core::common::cleaners::normalize_bib_key;
  let normkey = normalize_bib_key(key);
  let cell = Rc::new(RefCell::new(entry));
  BIB_ENTRIES.with(|m| {
    m.borrow_mut().insert(normkey.clone(), cell.clone());
  });
  CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = Some(normkey));
  cell
}

/// Switch the "current" pointer to a previously-registered entry.
/// Returns `false` if `key` isn't registered.
pub fn set_current_entry(key: &str) -> bool {
  use latexml_core::common::cleaners::normalize_bib_key;
  let normkey = normalize_bib_key(key);
  let exists = BIB_ENTRIES.with(|m| m.borrow().contains_key(&normkey));
  if exists {
    CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = Some(normkey));
  }
  exists
}

/// Clear the "current" pointer. Called at entry-complete time so a
/// stray `currentBibEntry()` outside a `\bib{...}` block returns
/// `None` instead of leaking the previous entry.
pub fn clear_current_entry() {
  CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = None);
}

/// Perl: `currentBibEntry()` — return a handle to the entry being
/// processed, or `None` if not inside a `\bib{...}` block.
pub fn current_entry() -> Option<Rc<RefCell<BibEntry>>> {
  CURRENT_ENTRY_KEY.with(|k| {
    let key = k.borrow();
    let key_ref = key.as_deref()?;
    BIB_ENTRIES.with(|m| m.borrow().get(key_ref).cloned())
  })
}

/// Look up a *registered* entry by raw (un-normalized) key. Perl
/// equivalent: `LookupValue('BIBENTRY@'.NormalizeBibKey($k))`. Used
/// by `\bib@@field`'s crossref path (`\bib@entry@default@prepare`
/// pulls listed fields from the crossref'd parent).
pub fn lookup_entry(key: &str) -> Option<Rc<RefCell<BibEntry>>> {
  use latexml_core::common::cleaners::normalize_bib_key;
  let normkey = normalize_bib_key(key);
  BIB_ENTRIES.with(|m| m.borrow().get(&normkey).cloned())
}

/// Perl: `currentBibEntryField('fieldname')` — get the *processed*
/// token value of a field on the current entry.
pub fn current_entry_field(name: &str) -> Option<Tokens> {
  current_entry()?.borrow().get_field(name).cloned()
}

/// Perl: `currentBibEntryRawField('fieldname')` — get the *raw*
/// source string of a field on the current entry.
pub fn current_entry_raw_field(name: &str) -> Option<String> {
  current_entry()?
    .borrow()
    .get_raw_field(name)
    .map(str::to_string)
}

/// Perl: `copyCrossrefFields(@fields)` — for each named field, if
/// the current entry doesn't already have it but its crossref'd
/// parent does, copy the value over (processed and raw paths). The
/// crossref field itself is read from `current_entry().get_field`.
pub fn copy_crossref_fields(fields: &[&str]) {
  let Some(current) = current_entry() else { return; };
  // Get the crossref target's raw key; if no crossref, nothing to do.
  let xref_key = current.borrow().get_raw_field("crossref").map(str::to_string);
  let Some(xref_key) = xref_key else { return; };
  let Some(parent) = lookup_entry(&xref_key) else { return; };
  // Self-crossref is a paper bug; skip silently to avoid infinite
  // looping if a user writes `crossref={selfkey}`.
  if Rc::ptr_eq(&current, &parent) {
    return;
  }
  let parent_b = parent.borrow();
  let mut current_b = current.borrow_mut();
  for field in fields {
    if current_b.get_field(field).is_none() {
      if let Some(v) = parent_b.get_field(field) {
        current_b.add_field(*field, v.clone());
      }
    }
    if current_b.get_raw_field(field).is_none() {
      if let Some(v) = parent_b.get_raw_field(field) {
        current_b.add_raw_field(*field, v.to_string());
      }
    }
  }
}

/// Reset all bibtex thread-local state. Used between conversions so
/// one paper's entries don't leak into the next. Future hook for
/// `Converter::reset_session` integration.
pub fn reset() {
  BIB_ENTRIES.with(|m| m.borrow_mut().clear());
  CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = None);
}

LoadDefinitions!({
  // Perl BibTeX.pool.ltxml L19: `LoadPool('LaTeX')` — BibTeX
  // pool is built on top of the full LaTeX format, since bib
  // entries digest LaTeX-flavored markup in titles/authors/etc.
  LoadPool!("LaTeX");

  // TODO: port the remaining 936+ lines of BibTeX entry-type
  // constructors, field handlers, key normalization, and
  // special-character handling from `BibTeX.pool.ltxml`
  // L20-955. See module docstring above and
  // `docs/BIBTEX_PORT_PLAN.md` for the phase plan.
});

#[cfg(test)]
mod tests {
  use super::*;

  fn fresh() {
    reset();
  }

  #[test]
  fn entry_round_trips_fields_and_raw_fields() {
    fresh();
    let mut e = BibEntry::new("Smith2020", "article");
    e.add_field("title", Tokens::new(Vec::new())); // placeholder Tokens
    e.add_raw_field("title", "On Examples");
    e.add_raw_field("year", "2020");
    assert_eq!(e.key, "Smith2020");
    assert_eq!(e.entry_type, "article");
    assert_eq!(e.get_raw_field("title"), Some("On Examples"));
    assert_eq!(e.get_raw_field("year"), Some("2020"));
    assert_eq!(e.get_raw_field("missing"), None);
    let names: Vec<&str> = e.field_names().collect();
    assert_eq!(names, vec!["title"]);
  }

  #[test]
  fn current_entry_round_trip() {
    fresh();
    let mut e = BibEntry::new("MyKey", "book");
    e.add_raw_field("year", "1999");
    register_entry("MyKey", e);
    let cur = current_entry().expect("current entry registered");
    assert_eq!(cur.borrow().key, "MyKey");
    assert_eq!(current_entry_raw_field("year").as_deref(), Some("1999"));
    clear_current_entry();
    assert!(current_entry().is_none());
    // Re-set via set_current_entry
    assert!(set_current_entry("MyKey"));
    assert!(current_entry().is_some());
    // Unknown key
    assert!(!set_current_entry("Nope"));
  }

  #[test]
  fn normalized_key_makes_lookup_case_insensitive() {
    fresh();
    register_entry("FooBar", BibEntry::new("FooBar", "article"));
    // Perl NormalizeBibKey lowercases; both `foobar` and `FOOBAR`
    // resolve to the same entry.
    assert!(lookup_entry("foobar").is_some());
    assert!(lookup_entry("FOOBAR").is_some());
    assert!(lookup_entry("foo bar").is_some()); // SPACES_RE strips ws
    assert!(lookup_entry("Other").is_none());
  }

  #[test]
  fn copy_crossref_fields_pulls_from_parent() {
    fresh();
    // Set up a parent with author + journal, then a child with
    // crossref=parent. copy_crossref_fields should copy author into
    // child but not overwrite child's existing journal.
    let mut parent = BibEntry::new("Parent2020", "article");
    parent.add_raw_field("author", "Parent A.");
    parent.add_raw_field("journal", "Parent Journal");
    register_entry("Parent2020", parent);

    let mut child = BibEntry::new("Child2020", "article");
    child.add_raw_field("crossref", "Parent2020");
    child.add_raw_field("journal", "Child Journal");
    register_entry("Child2020", child);

    copy_crossref_fields(&["author", "journal"]);

    let c = current_entry().unwrap();
    assert_eq!(c.borrow().get_raw_field("author"), Some("Parent A."));
    // child's journal NOT overwritten
    assert_eq!(c.borrow().get_raw_field("journal"), Some("Child Journal"));
  }

  #[test]
  fn copy_crossref_handles_missing_crossref() {
    fresh();
    register_entry("Solo", BibEntry::new("Solo", "article"));
    // No-op: no crossref → no panic, no copy
    copy_crossref_fields(&["author", "title"]);
    let c = current_entry().unwrap();
    assert_eq!(c.borrow().get_raw_field("author"), None);
  }

  #[test]
  fn copy_crossref_handles_self_crossref() {
    fresh();
    let mut e = BibEntry::new("Loop", "article");
    e.add_raw_field("crossref", "Loop");
    e.add_raw_field("title", "Self-referential");
    register_entry("Loop", e);
    // Should NOT recurse / overwrite. Silently skip.
    copy_crossref_fields(&["title"]);
    let c = current_entry().unwrap();
    assert_eq!(c.borrow().get_raw_field("title"), Some("Self-referential"));
  }

  #[test]
  fn current_entry_returns_none_outside_block() {
    fresh();
    assert!(current_entry().is_none());
    assert!(current_entry_field("title").is_none());
    assert!(current_entry_raw_field("title").is_none());
  }
}
