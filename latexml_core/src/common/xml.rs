use crate::common::error::Result;
use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;

pub const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";
pub const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

pub struct XPath {
  context: Context,
}

// pub type XPathClosure = Rc<Fn(&mut Gullet, Tokens) -> bool>;
impl XPath {
  pub fn new(doc: &Document, _mappings: HashMap<String, String>) -> Self {
    let context = Context::new(doc).unwrap();
    XPath { context }
  }

  pub fn register_namespace(&mut self, codeprefix: &str, namespace: &str) -> Result<()> {
    match self.context.register_namespace(codeprefix, namespace) {
      Ok(()) => {},
      Err(_) => {
        let message = s!(
          "Failed to register an XPath namespace: prefix {:?} and href {:?}",
          codeprefix,
          namespace
        );
        Error!("expected", "XPath", message);
      },
    };
    Ok(())
  }

  pub fn findnodes(&mut self, xpath: &str, node: Option<&Node>) -> Vec<Node> {
    match self.context.findnodes(xpath, node) {
      Ok(nodes) => nodes,
      Err(e) => {
        let message = s!("{:?}", e);
        let err = || {
          Error!("xpath", "findnodes", message);
          Ok(())
        };
        err().ok();
        // libxml2 XPath failures (invalid context node, growth limit
        // hit, malformed expression) used to panic and abort the run.
        // Treat as "no matches" instead — the conversion can usually
        // recover and produce most of the document. Drivers:
        // 2105.04174, 2304.07380, 1904.02716 all aborted here.
        Vec::new()
      },
    }
  }

  pub fn findvalues(&mut self, xpath: &str, node: Option<&Node>) -> Vec<String> {
    match self.context.findvalues(xpath, node) {
      Ok(vals) => vals,
      Err(e) => {
        let message = s!("{:?}", e);
        let err = || {
          Error!("xpath", "findvalues", message);
          Ok(())
        };
        err().ok();
        Vec::new()
      },
    }
  }

  pub fn findvalue(&mut self, xpath: &str, node: Option<&Node>) -> String {
    self.context.findvalue(xpath, node).unwrap_or_default()
  }
}

//======================================================================
// XML Utilities
/// gets the following `Element` sibling of `node` (skipping over non-element nodes)
pub fn get_next_element(node_in: &Node) -> Option<Node> {
  let mut node = Cow::Borrowed(node_in);
  while let Some(next) = node.get_next_sibling() {
    if next.get_type() == Some(NodeType::ElementNode) {
      return Some(next);
    } else {
      node = Cow::Owned(next);
    }
  }
  None
}
/// gets the previous `Element` sibling of `node` (skipping over non-element nodes)
pub fn get_prev_element(node_in: &Node) -> Option<Node> {
  let mut node = Cow::Borrowed(node_in);
  while let Some(next) = node.get_prev_sibling() {
    if next.get_type() == Some(NodeType::ElementNode) {
      return Some(next);
    } else {
      node = Cow::Owned(next);
    }
  }
  None
}
/// obtains all `Element` children of `node`, ignoring all other node types
pub fn element_nodes(node: &Node) -> Vec<Node> {
  node
    .get_child_nodes()
    .into_iter()
    .filter(|n| matches!(n.get_type(), Some(NodeType::ElementNode)))
    .collect()
}

/// obtains all content children of `node` (`Element` and `Text`), ignoring all other node types
pub fn content_nodes(node: &Node) -> Vec<Node> {
  node
    .get_child_nodes()
    .into_iter()
    .filter(|n| {
      matches!(
        n.get_type(),
        Some(NodeType::ElementNode) | Some(NodeType::TextNode)
      )
    })
    .collect()
}

pub fn closest_element(node: &Node) -> Option<Node> {
  if node.get_type() == Some(NodeType::ElementNode) {
    return Some(node.clone());
  }
  while let Some(parent) = node.get_parent() {
    if parent.get_type() == Some(NodeType::ElementNode) {
      return Some(parent);
    }
  }
  None
}

/// Is `child` the same as `parent`, or a descendent of `parent`?
pub fn is_descendant_or_self(child: &Node, parent: &Node) -> bool {
  let mut p = Some(child);
  let mut parent_opt;
  while let Some(p_node) = p {
    // if p.is_same_node(parent) {
    if p_node == parent {
      return true;
    }
    if let Some(parent_node) = p_node.get_parent() {
      parent_opt = Some(parent_node);
      p = parent_opt.as_ref();
    } else {
      break;
    }
  }
  false
}

#[cfg(test)]
mod tests {
  use super::*;
  use libxml::tree::Document;

  #[test]
  fn namespace_constants() {
    assert_eq!(XML_NS, "http://www.w3.org/XML/1998/namespace");
    assert_eq!(XMLNS_NS, "http://www.w3.org/2000/xmlns/");
  }

  fn build_tree() -> (Document, Node) {
    let mut doc = Document::new().unwrap();
    let mut root = Node::new("root", None, &doc).unwrap();
    doc.set_root_element(&root);
    // Mix of element + text siblings:
    //   root: <a/> "text1" <b/> "text2" <c/>
    let mut a = Node::new("a", None, &doc).unwrap();
    let mut t1 = Node::new_text("text1", &doc).unwrap();
    let mut b = Node::new("b", None, &doc).unwrap();
    let mut t2 = Node::new_text("text2", &doc).unwrap();
    let mut c = Node::new("c", None, &doc).unwrap();
    root.add_child(&mut a).unwrap();
    root.add_child(&mut t1).unwrap();
    root.add_child(&mut b).unwrap();
    root.add_child(&mut t2).unwrap();
    root.add_child(&mut c).unwrap();
    (doc, root)
  }

  #[test]
  fn element_nodes_skips_text() {
    let (_doc, root) = build_tree();
    let children = element_nodes(&root);
    assert_eq!(children.len(), 3);
    assert_eq!(children[0].get_name(), "a");
    assert_eq!(children[1].get_name(), "b");
    assert_eq!(children[2].get_name(), "c");
  }

  #[test]
  fn content_nodes_includes_text() {
    let (_doc, root) = build_tree();
    let children = content_nodes(&root);
    assert_eq!(children.len(), 5, "3 elements + 2 text nodes");
  }

  #[test]
  fn get_next_element_skips_text() {
    let (_doc, root) = build_tree();
    let a = element_nodes(&root)[0].clone();
    let next = get_next_element(&a).expect("a has a next element");
    assert_eq!(
      next.get_name(),
      "b",
      "<a> next element must be <b>, skipping the text node"
    );
  }

  #[test]
  fn get_next_element_none_at_end() {
    let (_doc, root) = build_tree();
    let c = element_nodes(&root)[2].clone();
    assert!(get_next_element(&c).is_none(), "last element has no next");
  }

  #[test]
  fn get_prev_element_skips_text() {
    let (_doc, root) = build_tree();
    let b = element_nodes(&root)[1].clone();
    let prev = get_prev_element(&b).expect("b has a prev element");
    assert_eq!(
      prev.get_name(),
      "a",
      "<b> prev element must be <a>, skipping text"
    );
  }

  #[test]
  fn get_prev_element_none_at_start() {
    let (_doc, root) = build_tree();
    let a = element_nodes(&root)[0].clone();
    assert!(get_prev_element(&a).is_none(), "first element has no prev");
  }

  #[test]
  fn is_descendant_or_self_true_for_self() {
    let (_doc, root) = build_tree();
    assert!(is_descendant_or_self(&root, &root));
  }

  #[test]
  fn is_descendant_or_self_true_for_child() {
    let (_doc, root) = build_tree();
    let a = element_nodes(&root)[0].clone();
    assert!(is_descendant_or_self(&a, &root));
  }

  #[test]
  fn is_descendant_or_self_false_for_sibling() {
    let (_doc, root) = build_tree();
    let kids = element_nodes(&root);
    assert!(
      !is_descendant_or_self(&kids[0], &kids[1]),
      "a is not a descendant of b"
    );
  }
}
