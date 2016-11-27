use std::collections::HashMap;
use common::relaxng::Relaxng;
use document::Document;
// use common::font::*;

const LTX_NAMESPACE: &'static str = "http://dlmf.nist.gov/LaTeXML";

pub struct Model {
  schema: Option<Relaxng>,
  schema_data: Option<Vec<String>>,
  code_namespace_prefixes: HashMap<String, String>,
  code_namespaces: HashMap<String, String>,
  document_namespace_prefixes: HashMap<String, String>,
  document_namespaces: HashMap<String, String>,
  // doctype_namespaces: HashMap<String, String>,
  // namespace_errors: usize,
  permissive: bool,
  // no_compiled: bool,
  debug_mode: bool,
}
impl Default for Model {
  fn default() -> Self {
    Model {
      schema: None,
      schema_data: None,
      code_namespace_prefixes: HashMap::new(),
      code_namespaces: HashMap::new(),
      document_namespace_prefixes: HashMap::new(),
      document_namespaces: HashMap::new(),
      // doctype_namespaces: HashMap::new(),
      // namespace_errors: 0,
      permissive: true,
      // no_compiled: true,
      debug_mode: false,
    }
  }
}

impl Model {
  pub fn new() -> Self {
    let mut model = Model::default();
    // model.xpath.register_function("match-font", |x, y| {font::match_font(x,y)})
    model.register_namespace("xml".to_string(),
                             Some("http://www.w3.org/XML/1998/namespace".to_string()));
    model.register_document_namespace("xml".to_string(),
                                      Some("http://www.w3.org/XML/1998/namespace".to_string()));
    model
  }

  pub fn set_doc_type(&mut self, roottag: String, publicid: String, systemid: String) {
    self.schema_data = Some(vec!["DTD".to_string(), roottag, publicid, systemid]);
    return;
  }

  pub fn set_relaxng_schema(&mut self, schema: String) {
    self.schema_data = Some(vec!["RelaxNG".to_string(), schema]);
    return;
  }
  pub fn add_schema_declaration(&self, document: &mut Document) {
    if let &Some(ref schema) = &self.schema {
      schema.add_schema_declaration(document);
    }
  }

  pub fn load_schema(&mut self) -> &Option<Relaxng> {
    // Only load once
    if self.schema.is_some() {
      return &self.schema;
    }
    let name;
    if self.schema_data.is_none() {
      // Warn('expected', '<model>', undef, "No Schema Model has been declared; assuming LaTeXML");
      // // article ??? or what ? undef gives problems!
      self.set_relaxng_schema("LaTeXML".to_string());
      self.register_namespace("ltx".to_string(), Some(LTX_NAMESPACE.to_string()));
      self.register_namespace("svg".to_string(),
                              Some("http://www.w3.org/2000/svg".to_string()));
      self.register_namespace("xlink".to_string(),
                              Some("http://www.w3.org/1999/xlink".to_string())); // Needed for SVG
      self.register_namespace("m".to_string(),
                              Some("http://www.w3.org/1998/Math/MathML".to_string()));
      self.register_namespace("xhtml".to_string(),
                              Some("http://www.w3.org/1999/xhtml".to_string()));
      self.permissive = true;
      return &self.schema;
    } // Actually, they could have declared all sorts of Tags....
    match self.schema_data {
      None => {}
      Some(ref data) => {
        let schema_type = &data[0];
        match schema_type.as_ref() {
          "DTD" => {
            // my ($roottag, $publicid, $systemid) = @data;
            // require LaTeXML::Common::Model::DTD;
            // $name = $systemid;
            // $$self{schema} = LaTeXML::Common::Model::DTD->new($self, $roottag, $publicid, $systemid);
          }
          "RelaxNG" => {
            name = data[1].to_string();
            self.schema = Some(Relaxng{ name: name, ..Relaxng::default()});
          }
          _ => {}
        };
      }
    };

    // if (let compiled = ! self.no_compiled)
    //   && pathname_find($name, paths => $STATE->lookupValue('SEARCHPATHS'),
    //   types => ['model'], installation_subdir => "resources/$type")) {
    // $self->loadCompiledSchema($compiled); }
    // else {
    //   $$self{schema}->loadSchema; }
    if self.debug_mode {
      self.describe_model()
    }
    return &self.schema;
  }
  pub fn describe_model(&self) {}
  ///**********************************************************************
  /// Namespaces
  ///**********************************************************************
  /// There are TWO namespace mappings!!!
  /// One for coding, one for the DocType.
  ///
  /// Coding: this namespace mapping associates prefixes to namespace URIs for
  ///   use in the latexml code, constructors and such.
  ///   This must be a one to one mapping and there are no default namespaces.
  /// Document: this namespace mapping associates prefixes to namespace URIs
  ///   as used in the generated document, and will be the
  ///   set of prefixes used in the generated output.
  ///   This mapping may also use a prefix of "#default" which is for
  ///   the unprefixed form of elements (not used for attributes!)
  pub fn register_namespace(&mut self, codeprefix: String, namespace_opt: Option<String>) {
    let namespace_opt_checked = match namespace_opt { // double-check empty strings are None
      None => None,
      Some(val) => {
        if val.is_empty() {
          None
        } else {
          Some(val)
        }
      }
    };
    match namespace_opt_checked {
      Some(namespace) => {
        self.code_namespace_prefixes.insert(namespace.clone(), codeprefix.clone());
        self.code_namespaces.insert(codeprefix.clone(), namespace.clone());
        // self.xpath.register_ns(codeprefix, namespace);
      }
      None => {
        match self.code_namespaces.get(&codeprefix) {
          Some(prev) => self.code_namespace_prefixes.remove(prev),
          None => None,
        };
        self.code_namespaces.remove(&codeprefix);
      }
    };
    return;
  }

  pub fn register_document_namespace(&mut self, mut docprefix: String, namespace_opt: Option<String>) {
    if docprefix.is_empty() {
      docprefix = "#default".to_string();
    }
    match namespace_opt {
      Some(namespace) => {
        // Since the default namespace url can still ALSO have a prefix associated,
        // we prepend "DEFAULT#url" when using as a hash key in the prefixes table.
        let regnamespace = if docprefix == "#default" {
          "DEFAULT#".to_string() + &namespace
        } else {
          namespace.to_string()
        };
        self.document_namespace_prefixes.insert(regnamespace, docprefix.clone());
        self.document_namespaces.insert(docprefix, namespace);
      }
      None => {
        match self.document_namespaces.get(&docprefix) {
          Some(prev) => self.document_namespace_prefixes.remove(prev),
          None => None,
        };
        self.document_namespaces.remove(&docprefix);
      }
    };
    return;
  }
}
