//! Unit tests for `Document::rebuild_idstore_from_dom` — the
//! belt-and-suspenders fix for the 1605.08055 Finalizing-phase UAF
//! class (session 128, commit 337c1ef52). A stale `idstore` entry
//! pointing at freed libxml2 memory SIGSEGVs when downstream code
//! dereferences the dangling `Node` wrapper. The rebuild drops all
//! cache entries and repopulates from a fresh DOM walk.

use latexml_core::document::Document;
use libxml::tree::Node;

fn build_doc_with_ids() -> (Document, Node) {
  let mut doc = Document::new();
  let xml_doc = doc.get_document_mut();
  let mut root = Node::new("root", None, xml_doc).unwrap();
  xml_doc.set_root_element(&root);

  let mut a = Node::new("A", None, xml_doc).unwrap();
  a.set_attribute("xml:id", "a1").ok();
  root.add_child(&mut a).unwrap();

  let mut b = Node::new("B", None, xml_doc).unwrap();
  b.set_attribute("xml:id", "b1").ok();
  root.add_child(&mut b).unwrap();

  doc.set_node(&root);
  (doc, root)
}

#[test]
fn rebuild_idstore_clears_stale_entries() {
  // Construct a document where idstore contains an entry for a node
  // that no longer lives in the DOM — the exact dangling-reference
  // condition that caused 1605.08055.
  let (mut doc, _root) = build_doc_with_ids();

  // Manually record a node that is then never attached. In real usage
  // this comes from math-parser `replace_tree` or `unbind_node()` paths
  // that fail to call unrecord_id. Here we simulate by creating an
  // orphan.
  let xml_doc = doc.get_document_mut();
  let mut orphan = Node::new("Orphan", None, xml_doc).unwrap();
  orphan.set_attribute("xml:id", "orphan_id").ok();
  doc.record_node_ids(&orphan).unwrap();

  // Now look up the orphan through idstore. lookup_id returns it —
  // but in production the orphan's libxml2 memory could be freed.
  assert!(
    doc.lookup_id("orphan_id").is_some(),
    "pre-rebuild: stale entry is in idstore"
  );

  // Rebuild from DOM. The orphan is NOT in the DOM root, so the
  // rebuild should drop its entry. The attached-to-DOM entries
  // (a1, b1) should survive.
  doc.rebuild_idstore_from_dom().unwrap();

  assert!(
    doc.lookup_id("orphan_id").is_none(),
    "post-rebuild: stale entry purged"
  );
  assert!(
    doc.lookup_id("a1").is_some(),
    "post-rebuild: live entry retained"
  );
  assert!(
    doc.lookup_id("b1").is_some(),
    "post-rebuild: live entry retained"
  );
}

#[test]
fn rebuild_idstore_preserves_live_entries() {
  // Control case — with no stale entries, a rebuild is idempotent for
  // all live ids.
  let (mut doc, _root) = build_doc_with_ids();
  doc.rebuild_idstore_from_dom().unwrap();
  assert!(doc.lookup_id("a1").is_some());
  assert!(doc.lookup_id("b1").is_some());
  // Double rebuild: still stable.
  doc.rebuild_idstore_from_dom().unwrap();
  assert!(doc.lookup_id("a1").is_some());
  assert!(doc.lookup_id("b1").is_some());
}

#[test]
fn rebuild_idstore_handles_empty_document() {
  // Edge case — a Document with no root element yet (pre-absorb state)
  // should not panic. `finalize` guards this with `if let Some(root)`
  // anyway, but the function itself must be tolerant.
  let mut doc = Document::new();
  // No root element set.
  doc.rebuild_idstore_from_dom().unwrap();
}
