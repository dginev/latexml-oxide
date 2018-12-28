use libxml::tree::{Document, Node, NodeType};
use libxml::xpath::Context;
use std::collections::HashMap;

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
      Err(_) => error!(target: "expected:XPath", "Failed to register an XPath namespace: prefix {:?} and href {:?}", codeprefix, namespace),
    };
  }

  pub fn findnodes(&mut self, xpath: &str, node: Option<&Node>) -> Vec<Node> {
    match self.context.findnodes(xpath, node) {
      Ok(nodes) => nodes,
      Err(e) => {
        error!(target: "xpath:findnodes", "{:?}", e);
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
