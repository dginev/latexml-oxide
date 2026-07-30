//! Document-global facts that must survive a spill.
//!
//! When a subtree is spilled its DOM nodes are freed, so every registry that
//! held a handle into it (`Document::idstore`'s `Node`s, `node_boxes`' raw
//! pointers) must be purged — the historical finalize-SIGSEGV class. What the
//! later phases still need from spilled content is retained HERE, as plain
//! strings:
//!
//! * which segment holds a spilled `xml:id` — for `id:`-scoped rewrites, math
//!   `XMRef` existence checks, and global id-collision dedup;
//! * `label → id` — for `label:`-scoped rewrites (built eagerly by
//!   `Document::load_labels_for_rewrite` today);
//! * RDFa prefix declarations seen in spilled content — `set_rdfa_prefixes`
//!   scans the whole DOM in eager mode, so spilled fragments must contribute
//!   at spill time.
//!
//! The index is append-only during pass 1 and read-only afterwards. It can be
//! saved beside the segments for diagnostics ([`FragmentIndex::save`] /
//! [`FragmentIndex::load`]) in a dependency-free line format.

use std::{fmt::Write as _, fs, path::Path};

use rustc_hash::FxHashMap;

use super::SegmentId;
use crate::common::error::{Error, ErrorCategory, ErrorTarget, Result};

/// The string-only registry of what spilled content still owes the rest of the
/// conversion.
#[derive(Debug, Default)]
pub struct FragmentIndex {
  ids:           FxHashMap<String, SegmentId>,
  labels:        FxHashMap<String, String>,
  rdfa_prefixes: FxHashMap<String, String>,
}

impl FragmentIndex {
  /// A spilled node's `xml:id`, and the segment now holding it.
  pub fn record_id(&mut self, id: &str, segment: SegmentId) {
    self.ids.insert(id.to_string(), segment);
  }

  /// A `label → xml:id` association from spilled content.
  pub fn record_label(&mut self, label: &str, id: &str) {
    self.labels.insert(label.to_string(), id.to_string());
  }

  /// An RDFa prefix declaration (`prefix → uri`) seen in spilled content.
  pub fn record_rdfa_prefix(&mut self, prefix: &str, uri: &str) {
    self
      .rdfa_prefixes
      .insert(prefix.to_string(), uri.to_string());
  }

  /// Which segment holds `id`, if it was spilled.
  pub fn id_segment(&self, id: &str) -> Option<SegmentId> { self.ids.get(id).copied() }

  /// Is `id` claimed by any spilled fragment? (The global half of id-collision
  /// dedup; the live spine's half stays in `Document::idstore`.)
  pub fn contains_id(&self, id: &str) -> bool { self.ids.contains_key(id) }

  /// The `xml:id` a spilled `label` resolves to, if any.
  pub fn label_id(&self, label: &str) -> Option<&str> { self.labels.get(label).map(String::as_str) }

  /// All spilled `label → id` associations (merged into each fragment's — and
  /// the spine's — `rewrite_labels` before rules run, so a `label:`-scoped
  /// rule resolves no matter which fragment holds the label).
  pub fn labels(&self) -> impl Iterator<Item = (&str, &str)> {
    self.labels.iter().map(|(l, i)| (l.as_str(), i.as_str()))
  }

  /// Registry sizes `(ids, labels, rdfa)` — pass-1 telemetry.
  pub fn sizes(&self) -> (usize, usize, usize) {
    (self.ids.len(), self.labels.len(), self.rdfa_prefixes.len())
  }

  /// All RDFa prefix declarations contributed by spilled content.
  pub fn rdfa_prefixes(&self) -> impl Iterator<Item = (&str, &str)> {
    self
      .rdfa_prefixes
      .iter()
      .map(|(p, u)| (p.as_str(), u.as_str()))
  }

  /// Number of spilled ids recorded.
  pub fn id_count(&self) -> usize { self.ids.len() }

  /// Persist beside the segments (diagnostics / crash inspection). Line
  /// format, one record per line: `kind\tkey\tvalue`, keys percent-escaped
  /// for the three bytes that would break the framing (`%`, tab, newline).
  pub fn save(&self, path: &Path) -> Result<()> {
    let mut out = String::new();
    for (id, seg) in &self.ids {
      let _ = writeln!(out, "id\t{}\t{}", escape(id), seg.0);
    }
    for (label, id) in &self.labels {
      let _ = writeln!(out, "label\t{}\t{}", escape(label), escape(id));
    }
    for (prefix, uri) in &self.rdfa_prefixes {
      let _ = writeln!(out, "rdfa\t{}\t{}", escape(prefix), escape(uri));
    }
    fs::write(path, out).map_err(|e| index_error(format!("write {}: {e}", path.display())))
  }

  /// Load a saved index. Unknown record kinds are an error, not a skip — a
  /// partially understood index would silently drop facts (fail toward
  /// flagging).
  pub fn load(path: &Path) -> Result<Self> {
    let text =
      fs::read_to_string(path).map_err(|e| index_error(format!("read {}: {e}", path.display())))?;
    let mut index = FragmentIndex::default();
    for (n, line) in text.lines().enumerate() {
      if line.is_empty() {
        continue;
      }
      let mut parts = line.splitn(3, '\t');
      let (kind, key, value) = match (parts.next(), parts.next(), parts.next()) {
        (Some(k), Some(key), Some(v)) => (k, unescape(key), unescape(v)),
        _ => {
          return Err(index_error(format!(
            "malformed line {} in {}",
            n + 1,
            path.display()
          )));
        },
      };
      match kind {
        "id" => {
          let seg: u32 = value
            .parse()
            .map_err(|_| index_error(format!("bad segment number {value:?} at line {}", n + 1)))?;
          index.ids.insert(key, SegmentId(seg));
        },
        "label" => {
          index.labels.insert(key, value);
        },
        "rdfa" => {
          index.rdfa_prefixes.insert(key, value);
        },
        other => {
          return Err(index_error(format!(
            "unknown record kind {other:?} at line {} in {}",
            n + 1,
            path.display()
          )));
        },
      }
    }
    Ok(index)
  }
}

fn escape(s: &str) -> String {
  // Only the three bytes that would break the line framing.
  s.replace('%', "%25")
    .replace('\t', "%09")
    .replace('\n', "%0A")
}

fn unescape(s: &str) -> String {
  s.replace("%0A", "\n")
    .replace("%09", "\t")
    .replace("%25", "%")
}

fn index_error(details: String) -> Error {
  Error {
    target:   ErrorTarget::Internal,
    category: ErrorCategory::Unexpected,
    message:  format!("fragment-index: {details}"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn records_and_queries() {
    let mut index = FragmentIndex::default();
    index.record_id("S1.p3", SegmentId(0));
    index.record_id("S2.p1", SegmentId(4));
    index.record_label("LABEL:intro", "S1.p3");
    index.record_rdfa_prefix("dct", "http://purl.org/dc/terms/");

    assert_eq!(index.id_segment("S1.p3"), Some(SegmentId(0)));
    assert_eq!(index.id_segment("S2.p1"), Some(SegmentId(4)));
    assert_eq!(index.id_segment("missing"), None);
    assert!(index.contains_id("S1.p3"));
    assert!(!index.contains_id("LABEL:intro"));
    assert_eq!(index.label_id("LABEL:intro"), Some("S1.p3"));
    assert_eq!(index.id_count(), 2);
    let prefixes: Vec<_> = index.rdfa_prefixes().collect();
    assert_eq!(prefixes, vec![("dct", "http://purl.org/dc/terms/")]);
  }

  #[test]
  fn save_load_round_trip_with_awkward_keys() {
    let mut index = FragmentIndex::default();
    // Keys exercising the escaping: tab, newline, percent, non-ASCII.
    index.record_id("id\twith\ttabs", SegmentId(1));
    index.record_id("id\nnewline", SegmentId(2));
    index.record_label("LABEL:100%\u{6570}", "S1.E5");
    index.record_rdfa_prefix("foaf", "http://xmlns.com/foaf/0.1/");

    let tmp = std::env::temp_dir().join(format!("lxsxml-index-{}.tsv", std::process::id()));
    index.save(&tmp).expect("save");
    let loaded = FragmentIndex::load(&tmp).expect("load");
    let _ = fs::remove_file(&tmp);

    assert_eq!(loaded.id_segment("id\twith\ttabs"), Some(SegmentId(1)));
    assert_eq!(loaded.id_segment("id\nnewline"), Some(SegmentId(2)));
    assert_eq!(loaded.label_id("LABEL:100%\u{6570}"), Some("S1.E5"));
    assert_eq!(loaded.rdfa_prefixes().collect::<Vec<_>>(), vec![(
      "foaf",
      "http://xmlns.com/foaf/0.1/"
    )]);
  }

  #[test]
  fn load_rejects_unknown_kinds_instead_of_skipping() {
    let tmp = std::env::temp_dir().join(format!("lxsxml-badidx-{}.tsv", std::process::id()));
    fs::write(&tmp, "mystery\tkey\tvalue\n").unwrap();
    let result = FragmentIndex::load(&tmp);
    let _ = fs::remove_file(&tmp);
    assert!(result.is_err(), "unknown record kinds must fail loudly");
  }
}
