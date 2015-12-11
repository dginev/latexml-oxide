use std::collections::HashMap;
use common::xml::XPath;
use common::font::*;

const LTX_NAMESPACE : &'static str = "http://dlmf.nist.gov/LaTeXML";

pub struct Model {
  xpath : XPath,
  code_namespace_prefixes : HashMap<String, String>,
  code_namespaces : HashMap<String, String>,
  doctype_namespaces : HashMap<String, String>,
  namespace_errors : usize
}
impl Default for Model {
  fn default() -> Self {
    Model {
      xpath : XPath::default(),
      code_namespace_prefixes : HashMap::new(),
      code_namespaces : HashMap::new(),
      doctype_namespaces : HashMap::new(),
      namespace_errors : 0
    }
  }
}

impl Model {
  pub fn new() -> Self {
    let mut model = Model::default();
    // model.xpath.register_function("match-font", |x, y| {font::match_font(x,y)})
    model.register_namespace("xml".to_string(),Some("http://www.w3.org/XML/1998/namespace".to_string()));
    model.register_document_namespace("xml".to_string(), Some("http://www.w3.org/XML/1998/namespace".to_string()));
    model
  }
  pub fn load_schema(&mut self) {}
  pub fn register_namespace(&mut self, codeprefix : String, namespace_opt : Option<String>) {
    match namespace_opt {
      Some(namespace) => {
        self.code_namespace_prefixes.insert(namespace.clone(), codeprefix.clone());
        self.code_namespaces.insert(codeprefix.clone(), namespace.clone());
        self.xpath.register_ns(codeprefix, namespace); 
      },
      None => {
        match self.code_namespaces.get(&codeprefix) {
          Some(prev) => self.code_namespace_prefixes.remove(prev),
          None => None
        };
        self.code_namespaces.remove(&codeprefix);
      }
    };
    return;
  }

  pub fn register_document_namespace(&mut self, codeprefix : String, namespace_opt : Option<String>) {

  }
}