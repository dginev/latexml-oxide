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

/// FxHashMap alias used by `bib_add_to_container` — matches
/// `latexml_core::document::Document::insert_element`'s expected
/// attribute-map type. Distinct from the `HashMap` (std) used by
/// the thread-local BibEntry registry above.
type FxAttrMap = rustc_hash::FxHashMap<String, String>;

/// Build the find-or-create XPath used by `bib_add_to_container`.
/// Perl `BibTeX.pool.ltxml:248-250`: `$tag[@k1='v1' and @k2='v2']`
/// with the attribute keys sorted so identical attr sets always
/// produce identical xpaths (cache hit on the second call).
fn bib_container_xpath(tag: &str, attrs: &FxAttrMap) -> String {
  if attrs.is_empty() {
    return tag.to_string();
  }
  let mut keys: Vec<&String> = attrs.keys().collect();
  keys.sort();
  let preds: Vec<String> = keys
    .iter()
    .map(|k| format!("@{}='{}'", k, attrs.get(*k).unwrap()))
    .collect();
  format!("{tag}[{}]", preds.join(" and "))
}

/// Perl `bibAddToContainer($document, $tag, $data, %attr)` —
/// `BibTeX.pool.ltxml:242-257`. Find-or-create a child of the
/// current `<ltx:bibentry>` / `<ltx:bib-related>` ancestor with
/// matching tag+attrs. If found, absorb `data` into it; if not,
/// insert a fresh element with `data` as content.
///
/// Used by `\bib@addto@related` (deduplicates same-(type,role)
/// bib-related groups, e.g. all authors collected in one
/// `<ltx:bib-related role='authors'>`).
pub fn bib_add_to_container(
  doc: &mut latexml_core::document::Document,
  tag: &str,
  data: Option<&latexml_core::digested::Digested>,
  attrs: FxAttrMap,
) -> latexml_core::common::error::Result<()> {
  let current = doc.get_node().clone();
  let entry = doc.findnode(
    "ancestor-or-self::ltx:bibentry | ancestor-or-self::ltx:bib-related",
    Some(&current),
  );
  let xpath = bib_container_xpath(tag, &attrs);
  if let Some(rel) = doc.findnode(&xpath, entry.as_ref()) {
    doc.set_node(&rel);
    if let Some(d) = data {
      doc.absorb(d, None)?;
    }
    doc.set_node(&current);
  } else {
    let content: Vec<&latexml_core::digested::Digested> = match data {
      Some(d) => vec![d],
      None => vec![],
    };
    doc.insert_element(tag, content, Some(attrs))?;
  }
  Ok(())
}

/// A single parsed BibTeX-style author/editor name.
///
/// Perl returns this implicitly inside `Invocation(T_CS('\bib@surname'),
/// Tokenize($surname))` etc. We expose the parsed triple so later
/// phases can wrap it however they need (Tokens, XML, post-processed
/// metadata) — splitting concerns at the parser/output boundary.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BibName {
  pub given:   String,
  pub surname: String,
  /// "Jr.", "Sr.", "III" — Perl `$jr`, `\bib@lineage` in TeX output.
  pub lineage: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BibNameList {
  pub names: Vec<BibName>,
  /// True if the input ended with `and others` / `and et al.` —
  /// Perl's `$etal` flag, post-processed to append a final
  /// `\bib@surname{others}` invocation.
  pub etal:  bool,
}

/// Perl `splitWords` (`BibTeX.pool.ltxml` L921-949). Split a name
/// string into words on whitespace / comma / tilde, but treat
/// `{balanced}` groups as a single word. Commas survive as their
/// own tokens. `\~` is preserved (Perl protects it via the
/// `####` placeholder trick — we use a sentinel `\x00` byte
/// which can't appear in TeX source).
fn split_words(input: &str) -> Vec<String> {
  // 1. Protect `\~`, normalise leading whitespace + `%\n` line continuations.
  const PLACEHOLDER: &str = "\x00";
  let s = input.replace("\\~", PLACEHOLDER);
  let s = s.trim_start_matches(|c: char| c.is_whitespace() || c == '~');
  let s = s.replace("%\n", "");

  let mut words: Vec<String> = Vec::new();
  let mut word = String::new();
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let b = bytes[i];
    // Check for `(comma?) whitespace+` separators. Perl regex:
    // s/^(,?)[\s~]+//
    if b == b',' || b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == b'~' {
      let had_comma = b == b',';
      let mut j = i + 1;
      // Either a leading comma we just consumed, or the whole run is
      // pure whitespace/tilde — collect the trailing whitespace.
      if had_comma {
        while j < bytes.len()
          && matches!(bytes[j], b' ' | b'\t' | b'\n' | b'\r' | b'~')
        {
          j += 1;
        }
      } else {
        // Pure whitespace run; skip them.
        while j < bytes.len() && matches!(bytes[j], b' ' | b'\t' | b'\n' | b'\r' | b'~') {
          j += 1;
        }
        // If we didn't actually see any whitespace beyond this one
        // char, we still advance (the `[\s~]+` pattern matches `+`
        // which requires ≥1; we have 1 = the current char).
      }
      // Flush accumulated word
      if !word.is_empty() {
        words.push(std::mem::take(&mut word));
      }
      if had_comma {
        words.push(",".to_string());
      }
      i = j;
    } else if b == b'{' {
      // Extract balanced group; include the braces.
      let start = i;
      let mut depth = 0i32;
      while i < bytes.len() {
        match bytes[i] {
          b'{' => depth += 1,
          b'}' => {
            depth -= 1;
            if depth == 0 {
              i += 1;
              break;
            }
          },
          _ => {},
        }
        i += 1;
      }
      // Append the entire braced chunk (including the braces) to the
      // current word so it stays atomic across word splits — matches
      // Perl `$word .= $t`.
      word.push_str(&s[start..i]);
    } else {
      // Greedily accumulate until the next separator / `{`.
      let start = i;
      while i < bytes.len()
        && !matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r' | b'~' | b',' | b'{')
      {
        i += 1;
      }
      word.push_str(&s[start..i]);
    }
  }
  if !word.is_empty() {
    words.push(word);
  }
  // 6. Restore `\~`.
  for w in words.iter_mut() {
    if w.contains(PLACEHOLDER) {
      *w = w.replace(PLACEHOLDER, "\\~");
    }
  }
  words
}

/// Perl `processBibNameList` (`BibTeX.pool.ltxml` L872-918): parse a
/// BibTeX-style author/editor list into structured names. The input
/// is a raw string (Perl uses `UnTeX($names, 1)` to flatten Tokens
/// first). Returns the parsed names plus an `etal` flag.
///
/// Name shapes recognised (BibTeX `bibtex.web` convention):
///   - `First Last` — given before surname; the surname starts at
///     the first lowercase word (e.g. `von`, `de la`) or, in the
///     no-lowercase case, at the LAST word.
///   - `Last, First` — surname first; one comma.
///   - `Last, Jr., First` — surname + lineage + given; two commas.
///
/// Multiple names separated by ` and ` (case-insensitive). A
/// trailing `and others` / `and et al.` sets `etal = true`.
pub fn process_bib_name_list(input: &str) -> BibNameList {
  let mut words = split_words(input);
  let mut etal = false;
  // Detect trailing `and others` / `and et al(.)?` and strip the
  // last two words if present. Perl L878.
  if words.len() >= 2 {
    let last = &words[words.len() - 1];
    let prev = &words[words.len() - 2];
    if prev.eq_ignore_ascii_case("and") {
      let last_lc = last.to_ascii_lowercase();
      let last_lc = last_lc.trim_end_matches('.');
      if matches!(last_lc, "others" | "et al" | "etal") {
        words.pop();
        words.pop();
        etal = true;
      }
    }
  }
  let mut names: Vec<BibName> = Vec::new();
  while !words.is_empty() {
    // Collect words for one name, splitting comma-delimited phrases.
    let mut phrases: Vec<Vec<String>> = Vec::new();
    let mut phrase: Vec<String> = Vec::new();
    while !words.is_empty() {
      let word = words.remove(0);
      if word.eq_ignore_ascii_case("and") {
        break;
      }
      if word == "," {
        phrases.push(std::mem::take(&mut phrase));
      } else {
        phrase.push(word);
      }
    }
    if phrase.is_empty() && phrases.is_empty() {
      // Empty name (consecutive `and`s, or stray comma+and). Perl
      // emits Warn; we silently skip — the caller's Tokens output
      // would be a no-op anyway.
      continue;
    }
    if !phrase.is_empty() {
      phrases.push(phrase);
    }

    let (given, surname, lineage) = match phrases.len() {
      3 => {
        // "von Last, Jr, First"
        let surname = phrases[0].join(" ");
        let lineage = phrases[1].join(" ");
        let given = phrases[2].join(" ");
        (given, surname, lineage)
      },
      2 => {
        // "von Last, First"
        let surname = phrases[0].join(" ");
        let given = phrases[1].join(" ");
        (given, surname, String::new())
      },
      _ => {
        // "First [von] Last" — words before the FIRST lowercase
        // word are given; the rest are surname. If no lowercase
        // word, the LAST word is surname and the rest are given.
        let pwords = &phrases[0];
        let mut first: Vec<String> = Vec::new();
        let mut rest: Vec<String> = pwords.clone();
        while !rest.is_empty() && !starts_lowercase(&rest[0]) {
          first.push(rest.remove(0));
        }
        if rest.is_empty() && !first.is_empty() {
          // No lowercase word — move the last `first` word into rest.
          rest.push(first.pop().unwrap());
        }
        let given = first.join(" ");
        let surname = rest.join(" ");
        (given, surname, String::new())
      },
    };
    names.push(BibName { given, surname, lineage });
  }
  BibNameList { names, etal }
}

/// Helper for `process_bib_name_list`: does a word start with a
/// lowercase letter? Perl test: `$word !~ /^[a-z]/` (NOT lowercase).
/// We invert that. Word may have a leading `{...}` group — peek past
/// it to find the first actual letter. Words starting with a control
/// sequence like `\von` are treated as lowercase by inspecting the
/// CS name's first letter (matches Perl behaviour for `\von Last`
/// where the regex sees `\` which IS lowercase via the
/// `[a-z]` heuristic — Perl's comment "This case test is not
/// correct!!!" acknowledges this is approximate).
fn starts_lowercase(word: &str) -> bool {
  // Strip a single leading `{...}` envelope (BibTeX convention for
  // forcing capitalisation: `{Smith}` should be opaque).
  let stripped = if let Some(rest) = word.strip_prefix('{') {
    rest.trim_end_matches('}')
  } else {
    word
  };
  // Skip a leading `\` (control sequence) — peek at the next char.
  let stripped = stripped.strip_prefix('\\').unwrap_or(stripped);
  stripped
    .chars()
    .next()
    .is_some_and(|c| c.is_lowercase())
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

  // --- process_bib_name_list tests (Perl `processBibNameList`) ---

  #[test]
  fn name_first_last() {
    let r = process_bib_name_list("Jane Smith");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "Jane");
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[0].lineage, "");
    assert!(!r.etal);
  }

  #[test]
  fn name_last_comma_first() {
    let r = process_bib_name_list("Smith, Jane");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "Jane");
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[0].lineage, "");
  }

  #[test]
  fn name_with_lineage() {
    let r = process_bib_name_list("Smith, Jr., Bob");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[0].lineage, "Jr.");
    assert_eq!(r.names[0].given, "Bob");
  }

  #[test]
  fn name_with_von_particle() {
    // "First von Last" — lowercase `von` triggers the split:
    // first = ["Ludwig"], rest = ["von", "Beethoven"] → surname.
    let r = process_bib_name_list("Ludwig von Beethoven");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "Ludwig");
    assert_eq!(r.names[0].surname, "von Beethoven");
  }

  #[test]
  fn name_all_capital_falls_back_to_last_word() {
    // No lowercase word — last word becomes surname per Perl
    // L909 `push(@pwords, pop(@first)) unless @pwords;`.
    let r = process_bib_name_list("John Q Public");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "John Q");
    assert_eq!(r.names[0].surname, "Public");
  }

  #[test]
  fn multiple_names_separated_by_and() {
    let r = process_bib_name_list("Jane Smith and John Doe and Alice Brown");
    assert_eq!(r.names.len(), 3);
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[1].surname, "Doe");
    assert_eq!(r.names[2].surname, "Brown");
    assert!(!r.etal);
  }

  #[test]
  fn etal_others_marker() {
    let r = process_bib_name_list("Jane Smith and others");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].surname, "Smith");
    assert!(r.etal);
  }

  #[test]
  fn etal_etal_single_word_marker() {
    // Perl's etal regex (`^(others|et\s*al\.?)$/i`) requires the
    // whole "et al" token to be ONE word — splitWords splits on
    // whitespace so "et al." becomes two words and the regex
    // doesn't fire. Match Perl-faithful: only `etal` / `etal.` as
    // a single word triggers etal detection. The multi-word
    // "et al." case becomes a second author with surname="et al."
    // — same as Perl LaTeXML.
    let r = process_bib_name_list("Smith and etal.");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].surname, "Smith");
    assert!(r.etal);
  }

  #[test]
  fn et_al_multi_word_is_not_etal_per_perl() {
    // Perl-faithful: split words case stays as a second "author".
    let r = process_bib_name_list("Smith and et al.");
    assert_eq!(r.names.len(), 2);
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[1].surname, "et al.");
    assert!(!r.etal);
  }

  #[test]
  fn braced_group_stays_atomic() {
    // BibTeX convention: `{De Long}` is a single surname token, the
    // braces protect "De" from being treated as a separate (capital)
    // word. After unwrapping for the lowercase check, it starts with
    // 'D' which is uppercase, so it falls into the "First Last"
    // branch and our heuristic puts the whole `{De Long}` as
    // surname.
    let r = process_bib_name_list("John {De Long}");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "John");
    assert_eq!(r.names[0].surname, "{De Long}");
  }

  #[test]
  fn tilde_treated_as_space() {
    // `~` is the LaTeX non-breaking space; BibTeX names use it for
    // initials. Perl's splitWords treats it as a hard space.
    let r = process_bib_name_list("J.~K. Rowling");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "J. K.");
    assert_eq!(r.names[0].surname, "Rowling");
  }

  #[test]
  fn empty_input_returns_empty_list() {
    let r = process_bib_name_list("");
    assert!(r.names.is_empty());
    assert!(!r.etal);
  }

  // --- bib_container_xpath (find-or-create xpath construction) ---

  #[test]
  fn xpath_bare_tag_when_no_attrs() {
    let attrs = FxAttrMap::default();
    assert_eq!(bib_container_xpath("ltx:bib-name", &attrs), "ltx:bib-name");
  }

  #[test]
  fn xpath_single_attr() {
    let mut attrs = FxAttrMap::default();
    attrs.insert("role".to_string(), "authors".to_string());
    assert_eq!(
      bib_container_xpath("ltx:bib-related", &attrs),
      "ltx:bib-related[@role='authors']"
    );
  }

  #[test]
  fn xpath_sorts_attr_keys_for_cache_stability() {
    // Perl's `sort keys %attr` ensures the same (type,role) pair
    // always produces the same xpath, regardless of hash iteration
    // order. We mirror that — alphabetic order by attribute name.
    let mut attrs = FxAttrMap::default();
    attrs.insert("type".to_string(), "book".to_string());
    attrs.insert("role".to_string(), "host".to_string());
    let xpath = bib_container_xpath("ltx:bib-related", &attrs);
    assert_eq!(xpath, "ltx:bib-related[@role='host' and @type='book']");
  }
}
