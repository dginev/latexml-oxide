//! Unit tests for `Document::modify_id` — the core ID collision
//! resolver. Returns the input unchanged if not in idstore; otherwise
//! appends alphabetic suffixes (`a`, `b`, …, `aa`, …) until a free
//! slot is found. If `state::ID_SUFFIX` is set, that suffix is tried
//! first.

use latexml_core::document::Document;
use libxml::tree::Node;

fn fresh_doc_with_node(id: &str) -> (Document, Node) {
  let mut doc = Document::new();
  let xml_doc = doc.get_document_mut();
  let root = Node::new("root", None, xml_doc).unwrap();
  xml_doc.set_root_element(&root);
  doc.set_node(&root);
  let root = doc.get_document().get_root_element().unwrap();
  let mut root_mut = root.clone();
  let mut n = Node::new("N", None, doc.get_document_mut()).unwrap();
  root_mut.add_child(&mut n).unwrap();
  doc.set_attribute(&mut n, "xml:id", id).unwrap();
  (doc, n)
}

#[test]
fn modify_id_passes_through_when_free() {
  let mut doc = Document::new();
  assert_eq!(
    doc.modify_id("free_id".to_string()),
    "free_id",
    "modify_id on a free id returns it unchanged"
  );
}

#[test]
fn modify_id_appends_alpha_suffix_on_collision() {
  let (mut doc, _) = fresh_doc_with_node("X");
  assert_eq!(
    doc.modify_id("X".to_string()),
    "Xa",
    "first collision gets 'a' suffix"
  );
}

#[test]
fn modify_id_progresses_through_alphabet() {
  // Seed idstore with X, Xa, Xb so modify_id must return Xc.
  let (mut doc, _) = fresh_doc_with_node("X");
  let root = doc.get_document().get_root_element().unwrap();
  let mut root_mut = root.clone();
  for id in ["Xa", "Xb"] {
    let mut n = Node::new("N", None, doc.get_document_mut()).unwrap();
    root_mut.add_child(&mut n).unwrap();
    doc.set_attribute(&mut n, "xml:id", id).unwrap();
  }
  assert_eq!(
    doc.modify_id("X".to_string()),
    "Xc",
    "third collision gets 'c' suffix"
  );
}

#[test]
fn modify_id_no_state_contamination_between_calls() {
  // modify_id itself is pure with respect to idstore — it only reads,
  // never writes. Two back-to-back calls on the same clashing id
  // yield the same result.
  let (mut doc, _) = fresh_doc_with_node("Y");
  let first = doc.modify_id("Y".to_string());
  let second = doc.modify_id("Y".to_string());
  assert_eq!(
    first, second,
    "modify_id is read-only over idstore; repeated calls are stable"
  );
  assert_eq!(first, "Ya");
}
