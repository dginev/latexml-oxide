use std::collections::HashMap;
use libxml::tree::{Document, Node};
use libxml::xpath::Context;

pub struct XPath<'xp> {
  context: Context<'xp>
}

// pub type XPathClosure = Arc<Fn(&mut Gullet, Vec<Token>, &mut State) -> bool>;
impl<'xp> XPath<'xp> {
  pub fn new(doc: &'xp Document, _mappings: HashMap<String, String>) -> Self {
    let context = Context::new(doc).unwrap();
    XPath {
      context: context
    }
  }

  pub fn register_namespace(&mut self, codeprefix: &str, namespace: &str) {
    match self.context.register_namespace(codeprefix, namespace) {
      Ok(()) => {},
      Err(_) => println_stderr!("Error:expected:XPath Failed to register an XPath namespace: prefix {:?} and href {:?}", codeprefix, namespace)
    };
  }

  // TODO: top-level findnodes so far, add rust-libxml support for per-node xpaths
  pub fn findnodes(&self, xpath: &str, _node: Node) -> Vec<Node> {
    println!("-- expression : {:?}", xpath);
    let results = self.context.evaluate(xpath).unwrap().get_nodes_as_vec();
    println!("-- found {:?} nodes", results.len());
    results
  }

  pub fn findvalue(&self, xpath: &str, _node: Node) -> String {
    self.context.evaluate(xpath).unwrap().to_string()
  }

}
