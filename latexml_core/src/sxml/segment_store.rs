//! The on-disk spill area for fragmented conversion.
//!
//! A segment file holds RAW, splice-ready text at every lifecycle stage —
//! never a wrapper element. An enclosing subtree that spills LATER keeps its
//! children's `<_spilled_ ref=…/>` placeholders as LITERAL elements in its
//! own file (inlining them once rebuilt multi-GB segments out of chapter
//! shells); the final assembly resolves placeholders recursively
//! (`Document::splice_segment_text`).
//!
//! 1. **Spilled** ([`SegmentStore::write_segment`]) — pass 1 stores the
//!    pre-finalize serialization of one or more sibling subtrees.
//! 2. **Processed** ([`SegmentStore::finalize_segment`]) — pass 2 replaces
//!    the file with the fragment's FINAL output text (post-rewrite,
//!    post-finalize, correctly indented).
//!
//! Parsing a segment (pass 2) goes through [`SegmentStore::wrapped_segment`],
//! which wraps the raw text in a `<_lxfragment>` root carrying the recorded
//! namespace declarations — in memory, never on disk.
//!
//! The directory lives beside the destination file — same volume, so
//! [`crate::watchdog::available_disk_bytes`] measures the filesystem the spill
//! actually consumes — and is removed on [`Drop`]. After a hard kill the
//! directory survives; its name (`.latexml-spill-<pid>-<seq>`) is deliberately
//! self-describing so a user knows what to delete.

use std::{
  fs,
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
  /// The spill parent's `noindent_children` — the `noindent` value the eager
  /// serializer would have passed when recursing into these children
  /// (schema-driven: whether the parent can contain `#PCDATA`). Pass-2
  /// re-serialization must use the same value or indentation diverges.
  pub noindent:   bool,
  /// The ancestor font context at the spill point, in `Font::to_string` form —
  /// the seed `finalize_rec` needs so per-fragment finalize resolves fonts
  /// exactly as the whole-document walk would have.
  pub font:       Option<String>,
  /// Namespace declarations (`(prefix, uri)`) the wrapper must carry so the
  /// segment parses stand-alone. In the eager DOM these live on the document
  /// root (hoisted during build); a spilled subtree's own serialization does
  /// not repeat them.
  pub namespaces: Vec<(String, String)>,
  /// The nearest ancestor SECTION's `xml:id` on the spine at the spill point.
  /// Scope-gated processing (`\lxDeclare` section scoping) resolves a token's
  /// section by walking ancestors — which a spilled-from-inside-a-section
  /// fragment no longer has.
  pub section_id: Option<String>,
  /// The qname of the spilled run's PARENT element (the spine node the
  /// segment splices back under). Pass-2 finalize consults the parent for
  /// schema decisions — e.g. collapsing an attribute-less `ltx:text` font
  /// wrapper requires `can_contain(parent, grandchild)` — and the parse
  /// wrapper (`ltx:_lxfragment`) is not in the model, so the recorded real
  /// parent substitutes for it (witness: tests/digestion/dollar.tex kept an
  /// empty `<text>` around an inline-block that eager collapsed).
  pub parent:     Option<String>,
  /// Every `xml:id` on the spilled run's ANCESTOR chain at the spill point.
  /// A `label:`/`id:`-scoped rewrite whose scope node is one of these
  /// ancestors covers the WHOLE fragment (the fragment sits inside the
  /// scope subtree), but the node itself lives in another fragment — the
  /// selection would come up empty (witness: tests/math/simplemath.tex,
  /// where `label:sec:restricted` stamps `role=FUNCTION` inside a section
  /// whose shell spills separately from its paragraphs).
  pub ancestors:  Vec<String>,
}

/// The element wrapping a spilled segment's sibling subtrees. Matches the
/// existing fragment-parsing convention (`common/xml.rs::FRAGMENT_WRAPPER`):
/// never part of a document, only a parse vehicle.
const SEGMENT_WRAPPER: &str = "_lxfragment";

/// The on-disk spill area: numbered segment files plus their in-RAM metadata.
#[derive(Debug)]
pub struct SegmentStore {
  dir:     PathBuf,
  metas:   Vec<SegmentMeta>,
  /// Historical: segments inlined into an enclosing one. Nested spills now
  /// stay nested (literal placeholders + recursive assembly splice), so
  /// nothing retires in the current pipeline; the mechanism remains for the
  /// store's API stability.
  retired: Vec<bool>,
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
    // Pid alone is NOT unique: two concurrent streaming conversions in one
    // process with destinations in the same parent (in-process test harnesses;
    // any embedder running conversions on several threads) would share the
    // directory, and the first store's Drop removes it under the other
    // (witness: 113_streaming_core's two byte-identity tests under plain
    // `cargo test`, which shares one process — segment-store write ENOENT).
    // A process-wide sequence keeps the name self-describing AND unique.
    static SPILL_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SPILL_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = parent.join(format!(".latexml-spill-{}-{seq}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| store_error(format!("create {}: {e}", dir.display())))?;
    Ok(SegmentStore {
      dir,
      metas: Vec::new(),
      retired: Vec::new(),
    })
  }

  /// The spill directory (for disk-headroom checks against its volume).
  pub fn dir(&self) -> &Path { &self.dir }

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

  /// Spill one or more serialized sibling subtrees as a new segment (raw,
  /// splice-ready text — see the module doc for why no wrapper is written).
  pub fn write_segment(&mut self, xml: &str, meta: SegmentMeta) -> Result<SegmentId> {
    let id = SegmentId(self.metas.len() as u32);
    let path = self.segment_path(id);
    fs::write(&path, xml).map_err(|e| store_error(format!("write {}: {e}", path.display())))?;
    self.metas.push(meta);
    self.retired.push(false);
    Ok(id)
  }

  /// The segment's content wrapped for stand-alone PARSING: a `_lxfragment`
  /// root carrying the namespace declarations recorded at spill time. Built
  /// in memory — the file itself stays raw and splice-ready.
  pub fn wrapped_segment(&self, id: SegmentId) -> Result<String> {
    let meta = self.meta(id)?;
    let mut out = String::with_capacity(256);
    out.push('<');
    out.push_str(SEGMENT_WRAPPER);
    for (prefix, uri) in &meta.namespaces {
      if prefix.is_empty() {
        out.push_str(&format!(" xmlns=\"{uri}\""));
      } else {
        out.push_str(&format!(" xmlns:{prefix}=\"{uri}\""));
      }
    }
    out.push('>');
    out.push_str(&self.read_segment(id)?);
    out.push_str(&format!("</{SEGMENT_WRAPPER}>"));
    Ok(out)
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

  /// Mark a segment retired: its text was inlined into an enclosing segment,
  /// so pass 2 skips it and assembly never asks for it. The file is truncated
  /// (the content lives in the outer segment now).
  pub fn retire_segment(&mut self, id: SegmentId) -> Result<()> {
    let _ = self.meta(id)?;
    self.retired[id.0 as usize] = true;
    let path = self.segment_path(id);
    fs::write(&path, "").map_err(|e| store_error(format!("truncate {}: {e}", path.display())))
  }

  /// Was this segment inlined into an enclosing one?
  pub fn is_retired(&self, id: SegmentId) -> bool {
    self.retired.get(id.0 as usize).copied().unwrap_or(false)
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
      noindent:   false,
      section_id: None,
      parent:     None,
      ancestors:  vec![],
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

    // The FILE stays raw and splice-ready; the wrapped form is in-memory.
    let raw = store.read_segment(id).expect("read");
    assert_eq!(raw, xml, "file content is exactly the spilled text");
    let wrapped = store.wrapped_segment(id).expect("wrap");
    assert!(wrapped.starts_with("<_lxfragment"), "wrapped: {wrapped}");
    assert!(wrapped.contains("xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\""));
    assert!(wrapped.ends_with("</_lxfragment>"));
    assert!(wrapped.contains("Gr\u{00fc}\u{00df}e \u{6570}\u{5b66}"));

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
