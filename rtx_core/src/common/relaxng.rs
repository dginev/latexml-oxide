use std::collections::HashMap;
use document::Document;


pub struct Relaxng {
  pub name: String,
  pub modules: Vec<String>,
  pub elementdefs: HashMap<String, String>,
  pub defs: HashMap<String, String>,
  pub elements: HashMap<String, String>,
  pub internal_grammars: u8,
}

impl Default for Relaxng {
  fn default() -> Self {
    Relaxng {
      name: "LaTeXML".to_string(),
      modules: Vec::new(),
      elementdefs: HashMap::new(),
      defs: HashMap::new(),
      elements: HashMap::new(),
      internal_grammars: 0
    }
  }
}
impl Relaxng {
  pub fn add_schema_declaration(&self, document: &mut Document) {
    document.insert_pi("latexml", vec!["RelaxNGSchema".to_string()], vec![self.name.clone()]);
  }
}