///! Extract Model information from a RelaxNG schema
use crate::document::Document;
use std::collections::HashMap;
/// an internal representation for a RelaxNG schema
pub struct Relaxng {
  /// the schema name
  pub name: String,
  ///
  pub modules: Vec<String>,
  ///
  pub elementdefs: HashMap<String, String>,
  ///
  pub defs: HashMap<String, String>,
  ///
  pub elements: HashMap<String, String>,
  ///
  pub internal_grammars: u8,
}

impl Default for Relaxng {
  fn default() -> Self {
    Relaxng {
      name: s!("LaTeXML"),
      modules: Vec::new(),
      elementdefs: HashMap::new(),
      defs: HashMap::new(),
      elements: HashMap::new(),
      internal_grammars: 0,
    }
  }
}
impl Relaxng {
  /// declare the schema on a given `Document`
  pub fn add_schema_declaration(&self, document: &mut Document) {
    let mut attributes = HashMap::new();
    if self.name != "DTD" {
      // provisions for phasing out DTD
      attributes.insert(s!("RelaxNGSchema"), self.name.clone());
      document.insert_pi("latexml", Some(attributes)).expect("should never fail");
    }
  }

  /// build the internal representation
  pub fn load_schema(&self) {
    unimplemented!()
  }
}
