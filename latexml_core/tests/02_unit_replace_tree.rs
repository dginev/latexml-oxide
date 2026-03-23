use latexml_core::document::Document;
use libxml::tree::Node;

/// Helper: create a Document and build a simple XML tree for testing.
fn setup_doc_with_tree() -> (Document, Node) {
  let mut doc = Document::new();
  let xml_doc = doc.get_document_mut();
  let mut root = Node::new("root", None, xml_doc).unwrap();
  xml_doc.set_root_element(&root);

  let mut a = Node::new("A", None, xml_doc).unwrap();
  a.set_content("content_a").ok();
  root.add_child(&mut a).unwrap();

  let mut b = Node::new("B", None, xml_doc).unwrap();
  b.set_content("content_b").ok();
  root.add_child(&mut b).unwrap();

  let mut c = Node::new("C", None, xml_doc).unwrap();
  c.set_content("content_c").ok();
  root.add_child(&mut c).unwrap();

  doc.set_node(&root);
  (doc, root)
}

/// Helper: get element child names
fn child_names(node: &Node) -> Vec<String> {
  node
    .get_child_elements()
    .iter()
    .map(|n| n.get_name())
    .collect()
}

#[test]
fn replace_tree_basic() {
  let (mut doc, root) = setup_doc_with_tree();
  let b = root.get_child_elements()[1].clone();
  assert_eq!(b.get_name(), "B");

  let mut d = Node::new("D", None, doc.get_document()).unwrap();
  d.set_content("content_d").ok();

  let result = doc.replace_tree(d, b).unwrap();
  assert!(result.is_some());
  assert_eq!(result.unwrap().get_name(), "D");
  assert_eq!(child_names(&root), vec!["A", "D", "C"]);
}

#[test]
fn replace_tree_first_child() {
  let (mut doc, root) = setup_doc_with_tree();
  let a = root.get_child_elements()[0].clone();

  let x = Node::new("X", None, doc.get_document()).unwrap();
  doc.replace_tree(x, a).unwrap();
  assert_eq!(child_names(&root), vec!["X", "B", "C"]);
}

#[test]
fn replace_tree_last_child() {
  let (mut doc, root) = setup_doc_with_tree();
  let c = root.get_child_elements()[2].clone();

  let y = Node::new("Y", None, doc.get_document()).unwrap();
  doc.replace_tree(y, c).unwrap();
  assert_eq!(child_names(&root), vec!["A", "B", "Y"]);
}

#[test]
fn replace_tree_preserves_following_siblings() {
  let (mut doc, root) = setup_doc_with_tree();
  let b = root.get_child_elements()[1].clone();

  let d = Node::new("D", None, doc.get_document()).unwrap();
  doc.replace_tree(d, b).unwrap();

  let children = root.get_child_elements();
  assert_eq!(children.len(), 3);
  assert_eq!(children[2].get_name(), "C");
  assert_eq!(children[2].get_content(), "content_c");
}

#[test]
fn replace_tree_replacement_reuses_old_children() {
  // CRITICAL test: The replacement node contains children that were
  // originally part of the old node. After replace_tree, those children
  // must still be accessible (no use-after-free from xmlFreeNode).
  let (mut doc, root) = setup_doc_with_tree();
  let xml_doc_ptr = doc.get_document() as *const _;

  // Build: root -> A, Wrapper(B1, B2), C
  let b = root.get_child_elements()[1].clone(); // B
  let mut wrapper = Node::new("Wrapper", None, doc.get_document()).unwrap();
  let mut b1 = Node::new("B1", None, doc.get_document()).unwrap();
  b1.set_content("hello").ok();
  let mut b2 = Node::new("B2", None, doc.get_document()).unwrap();
  b2.set_content("world").ok();
  wrapper.add_child(&mut b1).unwrap();
  wrapper.add_child(&mut b2).unwrap();

  // Replace B with Wrapper in root
  doc.replace_tree(wrapper, b).unwrap();
  assert_eq!(child_names(&root), vec!["A", "Wrapper", "C"]);

  // Now replace Wrapper with a Replacement that contains Wrapper's child B1
  let wrapper_node = root.get_child_elements()[1].clone();
  let b1_ref = wrapper_node.get_child_elements()[0].clone();
  assert_eq!(b1_ref.get_name(), "B1");

  // Build replacement containing B1 (moved from Wrapper)
  let mut replacement = Node::new("Repl", None, doc.get_document()).unwrap();
  let mut b1_move = b1_ref.clone();
  b1_move.unlink();
  replacement.add_child(&mut b1_move).unwrap();

  // This is the critical operation: Wrapper will be freed (mem::forget),
  // but B1 is now a child of Replacement, so it must survive.
  let result = doc.replace_tree(replacement, wrapper_node);
  assert!(result.is_ok());
  let result_node = result.unwrap().unwrap();
  assert_eq!(result_node.get_name(), "Repl");

  // Verify B1 survived inside the replacement
  let inner = result_node.get_child_elements();
  assert_eq!(inner.len(), 1);
  assert_eq!(inner[0].get_name(), "B1");
  assert_eq!(inner[0].get_content(), "hello");

  // Verify overall tree structure
  assert_eq!(child_names(&root), vec!["A", "Repl", "C"]);
}

#[test]
fn replace_tree_detached_node_returns_none() {
  let (mut doc, _root) = setup_doc_with_tree();
  let mut detached = Node::new("Detached", None, doc.get_document()).unwrap();
  detached.unlink();

  let new = Node::new("New", None, doc.get_document()).unwrap();
  let result = doc.replace_tree(new, detached).unwrap();
  assert!(result.is_none());
}
