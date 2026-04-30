//! Unit tests for `Document::set_attribute` xml:id de-duplication —
//! the session-128 record_id_with_node shadow-variable fix
//! (commit bab8beb53). Before the fix, a shadow binding inside an
//! `if let Some(prev) = prev_opt` scope meant the deduped id
//! never reached the caller: `idstore.insert` and the return both
//! referenced the OUTER (original) `id`, so the caller wrote a
//! duplicate xml:id to the DOM and libxml2 validation spent
//! O(n²) on the resulting cascade.

use latexml_core::document::Document;
use libxml::tree::Node;

fn fresh_doc() -> Document {
  let mut doc = Document::new();
  let xml_doc = doc.get_document_mut();
  let root = Node::new("root", None, xml_doc).unwrap();
  xml_doc.set_root_element(&root);
  doc.set_node(&root);
  doc
}

#[test]
fn set_attribute_xml_id_deduplicates_on_conflict() {
  let mut doc = fresh_doc();
  let xml_doc = doc.get_document_mut();
  let mut node1 = Node::new("A", None, xml_doc).unwrap();
  let mut node2 = Node::new("B", None, xml_doc).unwrap();

  // Record both on the DOM so rebuild has something to iterate later.
  let root = doc.get_document().get_root_element().unwrap();
  let mut root_mut = root.clone();
  root_mut.add_child(&mut node1).unwrap();
  root_mut.add_child(&mut node2).unwrap();

  // First set: accepted as-is.
  doc.set_attribute(&mut node1, "xml:id", "X").unwrap();
  assert_eq!(
    node1
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .as_deref(),
    Some("X"),
    "first node gets its requested id"
  );

  // Second set on a DIFFERENT node with the SAME requested id:
  // record_id_with_node must deduplicate. Before bab8beb53 the
  // shadow-variable bug silently dropped the rename and BOTH nodes
  // ended up with xml:id=X (+ idstore pointing at node2 only).
  doc.set_attribute(&mut node2, "xml:id", "X").unwrap();
  assert_ne!(
    node2
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .as_deref(),
    Some("X"),
    "conflicting id must be deduplicated to a different value"
  );

  // node1 keeps its original id.
  assert_eq!(
    node1
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .as_deref(),
    Some("X"),
    "non-conflicting node's id is untouched"
  );

  // idstore tracks BOTH nodes — the deduped id and the original.
  assert!(doc.lookup_id("X").is_some(), "original id still indexed");
  let node2_id = node2
    .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
    .unwrap();
  assert!(
    doc.lookup_id(&node2_id).is_some(),
    "deduped id also indexed"
  );
  assert_ne!(node2_id, "X", "deduped id must differ from original");
}

#[test]
fn set_attribute_xml_id_idempotent_on_self() {
  // Setting xml:id on a node that already has the same id is a
  // no-op — the gate `if node != prev` in record_id_with_node
  // prevents spurious dedup when the same node is re-visited.
  let mut doc = fresh_doc();
  let xml_doc = doc.get_document_mut();
  let mut node = Node::new("A", None, xml_doc).unwrap();
  let root = doc.get_document().get_root_element().unwrap();
  let mut root_mut = root.clone();
  root_mut.add_child(&mut node).unwrap();

  doc.set_attribute(&mut node, "xml:id", "Y").unwrap();
  doc.set_attribute(&mut node, "xml:id", "Y").unwrap();
  assert_eq!(
    node
      .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
      .as_deref(),
    Some("Y"),
    "re-setting same id on same node is a no-op"
  );
}

#[test]
fn rebuild_idstore_finds_all_ids_post_dedup() {
  // End-to-end: after a dedup, rebuild should pick up both the
  // original (node1's X) and the deduped (node2's Xa-ish) id.
  let mut doc = fresh_doc();
  let xml_doc = doc.get_document_mut();
  let mut n1 = Node::new("A", None, xml_doc).unwrap();
  let mut n2 = Node::new("B", None, xml_doc).unwrap();
  let mut n3 = Node::new("C", None, xml_doc).unwrap();
  let root = doc.get_document().get_root_element().unwrap();
  let mut root_mut = root.clone();
  root_mut.add_child(&mut n1).unwrap();
  root_mut.add_child(&mut n2).unwrap();
  root_mut.add_child(&mut n3).unwrap();

  doc.set_attribute(&mut n1, "xml:id", "dup").unwrap();
  doc.set_attribute(&mut n2, "xml:id", "dup").unwrap();
  doc.set_attribute(&mut n3, "xml:id", "dup").unwrap();

  doc.rebuild_idstore_from_dom().unwrap();

  // The DOM has three distinct xml:ids after dedup. idstore should
  // index all three.
  let id1 = n1
    .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
    .unwrap();
  let id2 = n2
    .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
    .unwrap();
  let id3 = n3
    .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
    .unwrap();
  assert!(
    id1 != id2 && id1 != id3 && id2 != id3,
    "three nodes must end up with distinct xml:ids after dedup; got {id1}, {id2}, {id3}"
  );
  assert!(doc.lookup_id(&id1).is_some());
  assert!(doc.lookup_id(&id2).is_some());
  assert!(doc.lookup_id(&id3).is_some());
}
