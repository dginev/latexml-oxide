use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;
use rustc_hash::FxHashMap as HashMap;
use crate::common::error::Result;
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
        let err = || {Error!("xpath", "findnodes", message); Ok(()) };
        err().ok();
        panic!("this is an external libxml2 error; unwinding...");
      },
    }
  }

  pub fn findvalues(&mut self, xpath: &str, node: Option<&Node>) -> Vec<String> {
    match self.context.findvalues(xpath, node) {
      Ok(vals) => vals,
      Err(e) => {
        let message = s!("{:?}", e);
        let err = || {Error!("xpath", "findvalues", message); Ok(())};
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
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
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
