//! The on-disk spill area for fragmented conversion.
//!
//! A segment has two lifecycle stages, and the file's content differs between
//! them on purpose:
//!
//! 1. **Spilled** ([`SegmentStore::write_segment`]) — pass 1 stores the
//!    pre-finalize serialization of one or more sibling subtrees, wrapped in a
//!    `<_lxfragment>` root that carries the namespace declarations recorded in
//!    the segment's [`SegmentMeta`]. Wrapped, the file is well-formed XML that
//!    [`super::FragmentReader`] can stream.
//! 2. **Processed** ([`SegmentStore::finalize_segment`]) — pass 2 replaces the
//!    file with the fragment's FINAL output text (post-rewrite, post-finalize,
//!    correctly indented, no wrapper). From then on the file is raw text that
//!    the assembly splice appends verbatim where the fragment's placeholder
//!    sits.
//!
//! The directory lives beside the destination file — same volume, so
//! [`crate::watchdog::available_disk_bytes`] measures the filesystem the spill
//! actually consumes — and is removed on [`Drop`]. After a hard kill the
//! directory survives; its name (`.latexml-spill-<pid>`) is deliberately
//! self-describing so a user knows what to delete.

use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use crate::common::error::{Error, ErrorCategory, ErrorTarget, Result};

/// Identifies one spilled segment within its [`SegmentStore`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentId(pub u32);

impl std::fmt::Display for SegmentId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

/// Per-segment facts recorded at spill time and consumed when the segment is
/// re-materialized in pass 2.
#[derive(Debug, Clone, Default)]
pub struct SegmentMeta {
  /// Spine depth of the spilled subtree's insertion point (the depth the
  /// serializer must resume at for indentation to match the eager output).
  pub depth:      usize,
  /// The ancestor font context at the spill point, in `Font::to_string` form —
  /// the seed `finalize_rec` needs so per-fragment finalize resolves fonts
  /// exactly as the whole-document walk would have.
  pub font:       Option<String>,
  /// Namespace declarations (`(prefix, uri)`) the wrapper must carry so the
  /// segment parses stand-alone. In the eager DOM these live on the document
  /// root (hoisted during build); a spilled subtree's own serialization does
  /// not repeat them.
  pub namespaces: Vec<(String, String)>,
}

/// The element wrapping a spilled segment's sibling subtrees. Matches the
/// existing fragment-parsing convention (`common/xml.rs::FRAGMENT_WRAPPER`):
/// never part of a document, only a parse vehicle.
const SEGMENT_WRAPPER: &str = "_lxfragment";

/// The on-disk spill area: numbered segment files plus their in-RAM metadata.
#[derive(Debug)]
pub struct SegmentStore {
  dir:   PathBuf,
  metas: Vec<SegmentMeta>,
}

impl SegmentStore {
  /// Create the spill directory beside `dest` (the conversion's output file or
  /// directory), so disk-headroom checks and the spill share a volume.
  pub fn create(dest: &Path) -> Result<Self> {
    let parent = if dest.is_dir() {
      dest
    } else {
      dest.parent().unwrap_or_else(|| Path::new("."))
    };
    let dir = parent.join(format!(".latexml-spill-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| store_error(format!("create {}: {e}", dir.display())))?;
    Ok(SegmentStore { dir, metas: Vec::new() })
  }

  /// Number of segments written so far.
  pub fn len(&self) -> usize { self.metas.len() }

  /// True when nothing has been spilled.
  pub fn is_empty(&self) -> bool { self.metas.is_empty() }

  /// The segment ids in spill (= document) order.
  pub fn ids(&self) -> impl Iterator<Item = SegmentId> {
    (0..self.metas.len() as u32).map(SegmentId)
  }

  /// The path of a segment's file (exists only after `write_segment`).
  pub fn segment_path(&self, id: SegmentId) -> PathBuf {
    self.dir.join(format!("segment-{:06}.xml", id.0))
  }

  /// Spill one or more serialized sibling subtrees as a new segment, wrapped
  /// for stand-alone parsing with `meta`'s namespace declarations.
  pub fn write_segment(&mut self, xml: &str, meta: SegmentMeta) -> Result<SegmentId> {
    let id = SegmentId(self.metas.len() as u32);
    let path = self.segment_path(id);
    let file = fs::File::create(&path)
      .map_err(|e| store_error(format!("create {}: {e}", path.display())))?;
    let mut w = std::io::BufWriter::new(file);
    let write = (|| -> std::io::Result<()> {
      write!(w, "<{SEGMENT_WRAPPER}")?;
      for (prefix, uri) in &meta.namespaces {
        if prefix.is_empty() {
          write!(w, " xmlns=\"{uri}\"")?;
        } else {
          write!(w, " xmlns:{prefix}=\"{uri}\"")?;
        }
      }
      write!(w, ">{xml}</{SEGMENT_WRAPPER}>")?;
      w.flush()
    })();
    write.map_err(|e| store_error(format!("write {}: {e}", path.display())))?;
    self.metas.push(meta);
    Ok(id)
  }

  /// Replace a spilled segment with its processed, splice-ready output text
  /// (raw — no wrapper; appended verbatim at the placeholder during assembly).
  pub fn finalize_segment(&mut self, id: SegmentId, output: &str) -> Result<()> {
    let _ = self.meta(id)?; // reject unknown ids before touching the disk
    let path = self.segment_path(id);
    fs::write(&path, output).map_err(|e| store_error(format!("write {}: {e}", path.display())))
  }

  /// The file's current content, whichever lifecycle stage it is in.
  pub fn read_segment(&self, id: SegmentId) -> Result<String> {
    let _ = self.meta(id)?;
    let path = self.segment_path(id);
    fs::read_to_string(&path).map_err(|e| store_error(format!("read {}: {e}", path.display())))
  }

  /// The metadata recorded when the segment was spilled.
  pub fn meta(&self, id: SegmentId) -> Result<&SegmentMeta> {
    self
      .metas
      .get(id.0 as usize)
      .ok_or_else(|| store_error(format!("unknown segment {id}")))
  }
}

impl Drop for SegmentStore {
  fn drop(&mut self) {
    // Best-effort: a failure to clean up must never mask the conversion's own
    // outcome. After a hard kill the directory simply survives under its
    // self-describing name.
    let _ = fs::remove_dir_all(&self.dir);
  }
}

fn store_error(details: String) -> Error {
  Error {
    target:   ErrorTarget::Internal,
    category: ErrorCategory::Unexpected,
    message:  format!("segment-store: {details}"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn meta_with_ns() -> SegmentMeta {
    SegmentMeta {
      depth:      2,
      font:       Some(String::from("italic")),
      namespaces: vec![(
        String::from("ltx"),
        String::from("http://dlmf.nist.gov/LaTeXML"),
      )],
    }
  }

  #[test]
  fn segment_round_trip_and_lifecycle() {
    let tmp = std::env::temp_dir().join(format!("lxsxml-store-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let dest = tmp.join("doc.xml");
    let mut store = SegmentStore::create(&dest).expect("create store");
    assert!(store.is_empty());

    // Two sibling subtrees, non-ASCII content, in one segment.
    let xml = "<ltx:para xml:id=\"p1\"><ltx:p>Gr\u{00fc}\u{00df}e \u{6570}\u{5b66}</ltx:p></ltx:para><ltx:para xml:id=\"p2\"/>";
    let id = store.write_segment(xml, meta_with_ns()).expect("write");
    assert_eq!(store.len(), 1);

    // Spilled form: wrapped, carries the namespace declaration, parseable.
    let spilled = store.read_segment(id).expect("read");
    assert!(spilled.starts_with("<_lxfragment"), "wrapped: {spilled}");
    assert!(spilled.contains("xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\""));
    assert!(spilled.contains("Gr\u{00fc}\u{00df}e \u{6570}\u{5b66}"));

    // Metadata survives.
    let meta = store.meta(id).expect("meta");
    assert_eq!(meta.depth, 2);
    assert_eq!(meta.font.as_deref(), Some("italic"));

    // Processed form replaces the file verbatim, no wrapper.
    store
      .finalize_segment(id, "  <final>out</final>\n")
      .expect("finalize");
    assert_eq!(store.read_segment(id).unwrap(), "  <final>out</final>\n");

    // Unknown ids are refused, not silently empty (fail toward flagging).
    assert!(store.read_segment(SegmentId(7)).is_err());
    assert!(store.finalize_segment(SegmentId(7), "x").is_err());

    // Drop removes the spill dir but never the destination's directory.
    let spill_dir = store.segment_path(id).parent().unwrap().to_path_buf();
    drop(store);
    assert!(!spill_dir.exists(), "spill dir cleaned on drop");
    assert!(tmp.exists());
    let _ = fs::remove_dir_all(&tmp);
  }

  #[test]
  fn segment_ids_iterate_in_document_order() {
    let tmp = std::env::temp_dir().join(format!("lxsxml-order-{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let mut store = SegmentStore::create(&tmp).expect("create store");
    for k in 0..3 {
      store
        .write_segment(&format!("<ltx:p>{k}</ltx:p>"), meta_with_ns())
        .expect("write");
    }
    let order: Vec<u32> = store.ids().map(|s| s.0).collect();
    assert_eq!(order, vec![0, 1, 2]);
    drop(store);
    let _ = fs::remove_dir_all(&tmp);
  }
}
