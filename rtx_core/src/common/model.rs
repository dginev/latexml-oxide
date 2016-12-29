use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::{HashMap, HashSet};
use regex::Regex;

use libxml::tree::{Node};
use libxml::tree::Document as XmlDoc;
use common::relaxng::Relaxng;
use document::Document;
use common::xml::XPath;
use util::pathname;
use common::error::*;

// use common::font::*;

const LTX_NAMESPACE: &'static str = "http://dlmf.nist.gov/LaTeXML";
pub type IndirectModel = HashMap<String, HashMap<String, String>>;

lazy_static! {
  // static ref OPTIONAL_RE : Regex = Regex::new(r"^Optional(.+)$").unwrap();
  static ref PREFIXED_LOCALNAME_RE : Regex = Regex::new(r"^([^:]+):(.+)$").unwrap();
  static ref LEAD_DEFAULT_RE : Regex = Regex::new(r"^DEFAULT#").unwrap();
  static ref CAPTURE_TAG_RE : Regex = Regex::new(r"(.*?:)?_Capture_$").unwrap();
  static ref TAG_MODEL_LINE : Regex = Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap();
  static ref CLASS_MODEL_LINE : Regex = Regex::new(r"^([^:=]+):=(.*?)$").unwrap();
  static ref NAMESPACE_MODEL_LINE : Regex = Regex::new(r"^([^=]+)=(.*?)$").unwrap();
}

pub struct TagFrame {
  model: HashSet<String>,
  attributes: HashSet<String>,
}
impl Default for TagFrame {
  fn default() -> Self {
    TagFrame {
      model: HashSet::new(),
      attributes: HashSet::new(),
    }
  }
}

pub struct Model {
  pub schema: Option<Relaxng>,
  pub schema_data: Option<Vec<String>>,
  pub schema_class: HashMap<String, HashSet<String>>,
  pub code_namespace_prefixes: HashMap<String, String>,
  pub code_namespaces: HashMap<String, String>,
  pub document_namespace_prefixes: HashMap<String, String>,
  pub document_namespaces: HashMap<String, String>,
  // doctype_namespaces: HashMap<String, String>,
  // namespace_errors: usize,
  pub permissive: bool,
  pub no_compiled: bool,
  pub debug_mode: bool,
  pub namespace_errors: u8,
  pub tagprop: HashMap<String, TagFrame>
}
impl Default for Model {
  fn default() -> Self {
    Model {
      schema: None,
      schema_data: None,
      schema_class: HashMap::new(),
      code_namespace_prefixes: HashMap::new(),
      code_namespaces: HashMap::new(),
      document_namespace_prefixes: HashMap::new(),
      document_namespaces: HashMap::new(),
      // doctype_namespaces: HashMap::new(),
      namespace_errors: 0,
      permissive: false,
      no_compiled: false,
      debug_mode: false,
      tagprop: HashMap::new()
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
  }

  pub fn set_relaxng_schema(&mut self, schema: String) {
    self.schema_data = Some(vec!["RelaxNG".to_string(), schema]);
  }
  pub fn add_schema_declaration(&self, document: &mut Document) {
    if let &Some(ref schema) = &self.schema {
      schema.add_schema_declaration(document);
    }
  }

  pub fn load_schema(&mut self, search_paths: Option<Vec<String>>) -> &Option<Relaxng> {
    // Only load once
    if self.schema.is_some() {
      return &self.schema;
    }
    let mut name = String::new();
    if self.schema_data.is_none() {
      // TODO: Return this code path to normal once we properly load schemas
      println_stderr!("Warn:expected:<model> TODO");
      // Warn('expected', '<model>', undef, "No Schema Model has been declared; assuming LaTeXML");
      // // article ??? or what ? undef gives problems!

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
    } // Actually, they could have declared all sorts of Tags....
    let mut schema_type = String::new();
    match self.schema_data {
      None => {},
      Some(ref data) => {
        schema_type = data[0].clone();
        match schema_type.as_ref() {
          "DTD" => {
            println_stderr!("Error:TODO:DTD not yet supported");
            // my ($roottag, $publicid, $systemid) = @data;
            // require LaTeXML::Common::Model::DTD;
            // $name = $systemid;
            // $$self{schema} = LaTeXML::Common::Model::DTD->new($self, $roottag, $publicid, $systemid);
          }
          "RelaxNG" => {
            name = data[1].to_string();
            self.schema = Some(Relaxng{ name: name.clone(), ..Relaxng::default()});
          }
          _ => {}
        };
      }
    };

    if !self.no_compiled {
      let pathname_opt = pathname::find(&name, pathname::FindOptions{
        paths: search_paths,
        types: Some(vec!["model".to_string()]),
        installation_subdir: Some(format!("resources/{}", schema_type))
      });

      match pathname_opt {
        Some(compiled_path) =>self.load_compiled_schema(&compiled_path),
        None => self.schema.as_mut().unwrap().load_schema()
      };
    }

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
    // double-check empty strings are None
    let namespace_opt_checked = match namespace_opt {
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
        if let Some(prev) = self.document_namespaces.get(docprefix) {
          self.document_namespace_prefixes.remove(prev);
        };
        self.document_namespaces.remove(docprefix);
      }
    };
    return;
  }

  pub fn get_document_namespace_prefix(&mut self, namespace: &str, forattribute: bool, probe: bool) -> Option<String> {
   // Get the prefix associated with the namespace url, noting that for elements, it might by "#default",
   // but for attributes would never be.
    // println_stderr!("Searching for {:?} in {:?}", namespace, self.document_namespace_prefixes);
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
        // TODO: Mock for now, add namespace_uri capability to rust-libxml next
        format!("ltx:{}",node.get_name())
      },
      // Need others?
      t =>  panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {:?}", t)
        // Fatal('misdefined', '<caller>', undef,
        //   "Should not ask for Qualified Name for node of type $type: " . Stringify($node));
    }
  }

  /// Same as get_node_qname, but using the Document namespace prefixes
  pub fn get_node_document_qname(&self, node: &Node) -> String {
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

      // TODO
      // elsif ($type == XML_NAMESPACE_DECL) {
      //   my $ns = $node->declaredURI;
      //   my $prefix = $ns && $self->getDocumentNamespacePrefix($ns, 0, 1);
      //   return ($prefix ? 'xmlns:' . $prefix : 'xmlns'); }
      NamespaceDecl => "xmlns".to_string(),

      ElementNode | AttributeNode => {
        // TODO
        // my $ns = $node->namespaceURI;
        // my $prefix = $ns && $self->getDocumentNamespacePrefix($ns, 0, 1);
        // return ($prefix ? $prefix . ":" . $node->localname : $node->localname); } }
        format!("ltx:{}",node.get_name())
      },
      // Need others?
      t =>  panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {:?}", t)
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


  //**********************************************************************
  // Document Structure Queries
  //**********************************************************************
  // NOTE: These are public, but perhaps should be passed
  // to submodel, in case it can evolve to more precision?
  // However, it would need more context to do that.

  /// Can an element with (qualified name) `tag` contain a `child` element?
  pub fn can_contain(&mut self, tag: &str, child: &str) -> bool {
    // Handle obvious cases explicitly.
    match tag {
      "#PCDATA" => return false,
      "#Comment" => return false,
      "_WildCard_" => return true,
      _ => {}
    };
    if CAPTURE_TAG_RE.is_match(tag) || CAPTURE_TAG_RE.is_match(child) { // with or without namespace prefix
      return true;
    }

    match child {
      "_WildCard_" => return true,
      "#Comment" => return true,
      "#ProcessingInstruction" => return true,
      "#DTD" => return true,
      _ => {}
    };
    if self.permissive && tag == "#Document" && child != "#PCDATA" {
      return true; // No DTD? Punt!
    }

    // Else query tag properties.
    let ref mut model = self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default).model;

    model.contains("ANY") || model.contains(child)
  }

  /// TODO: This is another component that would fit perfectly as a compiler plugin,
  ///       which generates a rust objects from all available schemas and has them directly available at runtime
  ///       For now, simply reimplementing the runtime loading of LaTeXML.model as-is from Model.pm
  pub fn load_compiled_schema(&mut self, path: &str) {
    note_begin(&(format!("Loading compiled schema {}", path)));
    let compiled_fh = File::open(path).unwrap();
    let compiled_reader = BufReader::new(&compiled_fh);
    for line_result in compiled_reader.lines() {
      if let Ok(line) = line_result {
        if let Some(caps) = TAG_MODEL_LINE.captures(&line) {
          let tag = caps.at(1).unwrap();
          let attr = caps.at(2).unwrap();
          let children = caps.at(3).unwrap();
          self.add_tag_attribute(tag, attr.split(",").collect());
          self.add_tag_content(tag, children.split(",").collect());

        } else if let Some(caps) = CLASS_MODEL_LINE.captures(&line) {
          let classname = caps.at(1).unwrap();
          let elements = caps.at(2).unwrap();
          let mut class_set = HashSet::new();
          for set_element in elements.split(",").collect::<Vec<&str>>() {
            class_set.insert(set_element.to_owned());
          }
          self.set_schema_class(classname, class_set);

        } else if let Some(caps) = NAMESPACE_MODEL_LINE.captures(&line) {
          let prefix = caps.at(1).unwrap();
          let namespace = caps.at(2).unwrap();
          self.register_document_namespace(prefix, Some(namespace.to_owned()));
        } else {
          panic!("Fatal:internal:{:?} Compiled model '{:?}' is malformatted at \"{:?}\"", path, path, line);
        }
      }
    }

    note_end(&(format!("Loading compiled schema {}", path)));
    return;
  }

  //**********************************************************************
  // Accessors
  //**********************************************************************

  pub fn get_tags(&self) -> Vec<String> {
    let mut keys : Vec<String> = self.tagprop.keys().map(|k| k.as_str().to_owned()).collect();
    keys.sort();
    keys
  }

  pub fn get_tag_contents(&self, tag :&str) -> Vec<&str> {
    match self.tagprop.get(tag) {
      Some(h) => {
        let mut keys : Vec<&str> = h.model.iter().map(|k| k.as_str()).collect();
        keys.sort();
        keys
      },
      None => Vec::new()
    }
  }

  pub fn add_tag_content(&mut self, tag: &str, elements: Vec<&str>) {
    let frame = self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default);

    for element in elements {
      frame.model.insert(element.to_owned());
    }
  }

  pub fn get_tag_attributes(&self, tag :&str) -> Vec<&str> {
    match self.tagprop.get(tag) {
      Some(h) => {
        let mut keys : Vec<&str> = h.attributes.iter().map(|k| k.as_str()).collect();
        keys.sort();
        keys
      },
      None => Vec::new()
    }
  }

  pub fn add_tag_attribute(&mut self, tag: &str, attributes: Vec<&str>) {
    let frame = self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default);

    for attribute in attributes {
      frame.attributes.insert(attribute.to_owned());
    }
  }

  pub fn set_schema_class(&mut self, classname: &str, content: HashSet<String>) {
    self.schema_class.insert(classname.to_owned(), content);
  }
}
