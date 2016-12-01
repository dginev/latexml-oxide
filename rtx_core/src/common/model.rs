use regex::Regex;
use std::collections::HashMap;
use libxml::tree::{Node};
use libxml::tree::Document as XmlDoc;
use common::relaxng::Relaxng;
use document::Document;
use common::xml::XPath;
// use common::font::*;

const LTX_NAMESPACE: &'static str = "http://dlmf.nist.gov/LaTeXML";

lazy_static! {
  static ref OPTIONAL_RE : Regex = Regex::new(r"^Optional(.+)$").unwrap();
  static ref PREFIXED_LOCALNAME_RE : Regex = Regex::new(r"^([^:]+):(.+)$").unwrap();
  static ref LEAD_DEFAULT_RE : Regex = Regex::new(r"^DEFAULT#").unwrap();
}

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
  namespace_errors: u8,
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
      namespace_errors: 0,
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
    model.register_namespace("xml",
                             Some("http://www.w3.org/XML/1998/namespace".to_string()));
    model.register_document_namespace("xml",
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
    // if self.schema.is_some() {
    //   return &self.schema;
    // }
    let name;
    // if self.schema_data.is_none() {
      // Warn('expected', '<model>', undef, "No Schema Model has been declared; assuming LaTeXML");
      // // article ??? or what ? undef gives problems!
      // TODO: Return this code path to normal once we properly load schemas
      self.register_document_namespace("ltx", Some(LTX_NAMESPACE.to_string()));
      self.set_relaxng_schema("LaTeXML".to_string());
      self.register_namespace("ltx", Some(LTX_NAMESPACE.to_string()));
      self.register_namespace("svg",
                              Some("http://www.w3.org/2000/svg".to_string()));
      self.register_namespace("xlink",
                              Some("http://www.w3.org/1999/xlink".to_string())); // Needed for SVG
      self.register_namespace("m",
                              Some("http://www.w3.org/1998/Math/MathML".to_string()));
      self.register_namespace("xhtml",
                              Some("http://www.w3.org/1999/xhtml".to_string()));
      self.permissive = true;
    // } // Actually, they could have declared all sorts of Tags....
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

  pub fn get_xpath<'o>(&'o self, document: &'o XmlDoc) -> XPath {
    let mut context = XPath::new(document, HashMap::new());
    for (prefix, ns) in self.code_namespaces.iter() {
      // TODO: Is this too slow? We may need to store an active context in the State as an alternative
      context.register_namespace(prefix, ns);
    }
    context
  }
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
  pub fn register_namespace(&mut self, codeprefix: &str, namespace_opt: Option<String>) {
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
        self.code_namespace_prefixes.insert(namespace.clone(), codeprefix.to_string());
        self.code_namespaces.insert(codeprefix.to_string(), namespace.clone());
        // self.xpath.register_ns(codeprefix, namespace);
      }
      None => {
        match self.code_namespaces.get(codeprefix) {
          Some(prev) => self.code_namespace_prefixes.remove(prev),
          None => None,
        };
        self.code_namespaces.remove(codeprefix);
      }
    };
    return;
  }

  pub fn register_document_namespace(&mut self, mut docprefix: &str, namespace_opt: Option<String>) {
    if docprefix.is_empty() {
      docprefix = "#default";
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
        self.document_namespace_prefixes.insert(regnamespace, docprefix.to_string());
        self.document_namespaces.insert(docprefix.to_string(), namespace);
      }
      None => {
        match self.document_namespaces.get(docprefix) {
          Some(prev) => self.document_namespace_prefixes.remove(prev),
          None => None,
        };
        self.document_namespaces.remove(docprefix);
      }
    };
    return;
  }

  pub fn get_document_namespace_prefix(&mut self, namespace: &str, forattribute: bool, probe: bool) -> Option<String> {
   // Get the prefix associated with the namespace url, noting that for elements, it might by "#default",
   // but for attributes would never be.
    let mut docprefix = if !forattribute {
      match self.document_namespace_prefixes.get(&("DEFAULT#".to_string() + &namespace)) {
        Some(prefix) => Some(prefix.to_string()),
        None => None
      }
    } else { None };
    if docprefix.is_none() {
      docprefix = match self.document_namespace_prefixes.get(namespace) {
        Some(prefix) => Some(prefix.to_string()),
        None => None
      };
    }

    if docprefix.is_none() && !probe {
      self.namespace_errors += 1;
      docprefix = Some("namespace".to_string() + &self.namespace_errors.to_string());
      self.register_document_namespace(docprefix.as_ref().unwrap(), Some(namespace.to_string()));
      println_stderr!("Warn:malformed:{:?}: No prefix has been registered for namespace.", namespace );
      // Warn('malformed', $namespace, undef,
        // "No prefix has been registered for namespace '$namespace' (in document)",
        // "Using '$docprefix' instead"); }
    }
    match docprefix {
      None => None,
      Some(p) => {
        if p == "#default" {
          None
        } else {
          Some(p)
        }
      }
    }
  }

  pub fn get_document_namespace(&mut self, mut docprefix: &str, probe: bool) -> Option<String> {
    if docprefix.is_empty() {
      docprefix = "#default";
    }
    let ns_str = match self.document_namespaces.get(docprefix) {
      None => String::new(),
      Some(s) => {
        LEAD_DEFAULT_RE.replace(s,"")
      }
    };

    if docprefix != "#default" && ns_str.is_empty() && !probe {
      self.namespace_errors += 1;
      let ns_error = "http://example.com/namespace".to_string() + &self.namespace_errors.to_string();
      self.register_document_namespace(&docprefix, Some(ns_error));
      println_stderr!("Error:malformed:{:?}: No namespace has been registered for prefix.", docprefix);
      // Error('malformed', $docprefix, undef,
      //   "No namespace has been registered for prefix '$docprefix' (in document)",
      //   "Using '$ns' instead"); }
    }
    if ns_str.is_empty() {
      None
    } else {
      Some(ns_str.to_string())
    }
  }

  /// In the following:
  ///    $forattribute is 1 if the namespace is for an attribute (in which case, there must be a non-empty prefix)
  ///    $probe, if non 0, just test for namespace, without creating an entry if missing.
  /// Get the (code) prefix associated with $namespace,
  /// creating a dummy prefix and signalling an error if none has been registered.
  pub fn get_namespace_prefix(&mut self, namespace: &str, _forattribute: bool, probe: bool) -> Option<String> {
    let mut codeprefix : Option<String> = None;

    if !namespace.is_empty() {
      codeprefix = match self.code_namespace_prefixes.get(namespace) {
        None => None,
        Some(p) => Some(p.clone())
      };

      if codeprefix.is_some() && !probe {
        {
          let docprefix = self.document_namespace_prefixes.get(namespace);
          // if there's a doc prefix and it's NOT already used in code namespace mapping
          if docprefix.is_some() && self.code_namespaces.get(docprefix.unwrap()).is_none() {
            codeprefix = match docprefix {
              None => None,
              Some(p) => Some(p.to_string())
            };
          }
        }
      } else { // Else synthesize one
        self.namespace_errors += 1;
        let auto_prefix = "namespace".to_string() + &self.namespace_errors.to_string();
        codeprefix = Some(auto_prefix);
      }
      self.register_namespace(codeprefix.as_ref().unwrap(), Some(namespace.to_string()));
      // Warn!('malformed', $namespace, undef,
      //   "No prefix has been registered for namespace '$namespace' (in code)",
      //   "Using '$codeprefix' instead"); }
    }

    match codeprefix {
      None => None,
      Some(cp) => Some(cp.to_string())
    }
  }

  pub fn get_namespace(&mut self, codeprefix: &str, probe: bool) -> Option<String> {
    let mut ns : Option<String> = match self.code_namespaces.get(codeprefix) {
      None => None,
      Some(ns) => Some(ns.to_string())
    };
    if ns.is_none() && !probe {
      self.namespace_errors += 1;
      let example_namespace = "http://example.com/namespace".to_string() + &self.namespace_errors.to_string();
      ns = Some(example_namespace.clone());
      self.register_namespace(codeprefix, Some(example_namespace));
      // Error!('malformed', $codeprefix, undef,
      //   "No namespace has been registered for prefix '$codeprefix' (in code)",
      //   "Using '$ns' isntead");
    }
    match ns {
      None => None,
      Some(ns) => Some(ns.to_string())
    }
  }

  /// Get the node's qualified name in standard form
  /// Ie. using the registered (code) prefix for that namespace.
  /// NOTE: Reconsider how _Capture_ & _WildCard_ should be integrated!?!
  pub fn get_node_qname(&self, node: &Node) -> String {
    use libxml::tree::NodeType::*;
    let node_type = node.get_type();
    if node_type.is_none() {
      return "#BrokenNode".to_string()
    }
    match node_type.unwrap() {
      TextNode => "#PCDATA".to_string(),
      DocumentNode => "#Document".to_string(),
      CommentNode => "#Comment".to_string(),
      PiNode => "#ProcessingInstruction".to_string(),
      DTDNode => "#DTD".to_string(),
      NamespaceDecl => {
        // match node.declared_uri() {
        //   Some(ns) => match self.get_namespace_prefix(ns, false, true) {
        //     Some(prefix) => "xmlns:".to_string()+prefix,
        //     None => "xmlns".to_string()
        //   },
        //   None => "xmlns".to_string()
        // }
        "xmlns".to_string()
      },
      ElementNode | AttributeNode => {
        // match node.namespace_uri() {
        //   Some(ns) => match self.get_namespace_prefix(ns, false, true) {
        //     Some(prefix) => prefx+":"+node.get_name(),
        //     None => node.get_name()
        //   },
        //   None => node.get_name()
        // }
        node.get_name()
      }
      // Need others?
      t =>  panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {:?}", t)
        // Fatal('misdefined', '<caller>', undef,
        //   "Should not ask for Qualified Name for node of type $type: " . Stringify($node));
    }
  }

  /// Given a Qualified name, possibly prefixed with a namespace prefix,
  /// as defined by the code namespace mapping,
  /// return the NamespaceURI and localname.
  pub fn decode_qname(&mut self, codetag: &str) -> (Option<String>, String) {
    match PREFIXED_LOCALNAME_RE.captures(codetag) {
      Some(captures) => {
        let prefix = captures.at(1).unwrap();
        let localname = captures.at(2).unwrap();

        if prefix == "xml" {
          (None, codetag.to_string())
        } else {
          (self.get_namespace(prefix, false), localname.to_string())
        }
      },
      None => (None, codetag.to_string())
    }
  }
}
