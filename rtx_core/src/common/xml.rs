use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;
use std::collections::HashMap;
use std::borrow::Cow;

pub const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";
pub const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

pub struct XPath {
  context: Context,
}

// pub type XPathClosure = Rc<Fn(&mut Gullet, Tokens, &mut State) -> bool>;
impl XPath {
  pub fn new(doc: &Document, _mappings: HashMap<String, String>) -> Self {
    let context = Context::new(doc).unwrap();
    XPath { context }
  }

  pub fn register_namespace(&mut self, codeprefix: &str, namespace: &str) {
    match self.context.register_namespace(codeprefix, namespace) {
      Ok(()) => {},
      Err(_) => {
        let message = s!("Failed to register an XPath namespace: prefix {:?} and href {:?}", codeprefix, namespace);
        Error!("expected", "XPath", None, None, message);
      },
    };
  }

  pub fn findnodes(&mut self, xpath: &str, node: Option<&Node>) -> Vec<Node> {
    match self.context.findnodes(xpath, node) {
      Ok(nodes) => nodes,
      Err(e) => {
        let message = s!("{:?}", e);
        Error!("xpath", "findnodes", None, None, message);
        Vec::new()
      },
    }
  }

  pub fn findvalue(&mut self, xpath: &str, node: Option<&Node>) -> String {
    match self.context.findvalue(xpath, node) {
      Ok(v) => v,
      _ => String::new(),
    }
  }
}

//======================================================================
// XML Utilities
pub fn element_nodes(node: &Node) -> Vec<Node> {
  node
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect()
}

pub fn closest_element(mut node: &Node) -> Option<Node> {
  if node.get_type() == Some(NodeType::ElementNode) {
    return Some(node.clone())
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
