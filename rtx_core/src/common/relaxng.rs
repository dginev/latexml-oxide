use std::collections::HashMap;
use common::model::Model;
use document::Document;


pub struct Relaxng<'a> {
  pub name: String,
  pub model: Option<&'a Model>,
  pub modules: Vec<String>,
  pub elementdefs: HashMap<String, String>,
  pub defs: HashMap<String, String>,
  pub elements: HashMap<String, String>,
  pub internal_grammars: u8,
}

impl<'a> Default for Relaxng<'a> {
  fn default() -> Self {
    Relaxng {
      name: "LaTeXML".to_string(),
      model: None,
      modules: Vec::new(),
      elementdefs: HashMap::new(),
      defs: HashMap::new(),
      elements: HashMap::new(),
      internal_grammars: 0
    }
  }
}
impl<'a> Relaxng<'a> {
  pub fn add_schema_declaration(&self, document: &mut Document) {
    document.insert_pi("latexml", vec!["RelaxNGSchema".to_string()], vec![self.name.clone()]);
  }
}