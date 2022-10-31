use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::string::ToString;

use crate::common::error::*;
use crate::common::object::Object;
use crate::common::relaxng::Relaxng;
use crate::common::xml::{XPath, XML_NS};
use crate::document::Document;
use crate::util::pathname;
use crate::Locator;
use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;

// use common::font::*;

pub const LTX_NAMESPACE: &str = "http://dlmf.nist.gov/LaTeXML";
pub type IndirectModel = HashMap<String, HashMap<String, String>>;

lazy_static! {
  // static ref OPTIONAL_RE : Regex = Regex::new(r"^Optional(.+)$").unwrap();
  static ref PREFIXED_LOCALNAME_RE : Regex = Regex::new(r"^([^:]+):(.+)$").unwrap();
  static ref CAPTURE_TAG_RE : Regex = Regex::new(r"(.*?:)?_Capture_$").unwrap();
  static ref TAG_MODEL_LINE : Regex = Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap();
  static ref CLASS_MODEL_LINE : Regex = Regex::new(r"^([^:=]+):=(.*?)$").unwrap();
  static ref NAMESPACE_MODEL_LINE : Regex = Regex::new(r"^([^=]+)=(.*?)$").unwrap();
}

#[derive(Default)]
pub struct TagFrame {
  model: HashSet<String>,
  attributes: HashSet<String>,
}

#[derive(Default)]
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
  pub tagprop: HashMap<String, TagFrame>,
}

impl Object for Model {
  fn get_locator(&self) -> Option<Cow<Locator>> { unimplemented!() }
}
impl Model {
  pub fn new() -> Self {
    let mut model = Model::default();
    // model.xpath.register_function("match-font", |x, y| {font::match_font(x,y)})
    model.register_namespace("xml", Some(XML_NS.to_string()));
    model.register_document_namespace("xml", Some(XML_NS.to_string()));
    model
  }

  pub fn set_doc_type(&mut self, roottag: String, publicid: String, systemid: String) {
    self.schema_data = Some(vec![s!("DTD"), roottag, publicid, systemid]);
  }

  pub fn set_relaxng_schema(&mut self, schema: String) { self.schema_data = Some(vec![s!("RelaxNG"), schema]); }
  pub fn add_schema_declaration(&self, document: &mut Document) {
    if let Some(ref schema) = self.schema {
      schema.add_schema_declaration(document);
    }
  }

  pub fn load_schema(&mut self, search_paths: &[&str]) -> &Option<Relaxng> {
    // Only load once
    if self.schema.is_some() {
      return &self.schema;
    }
    let mut name = String::new();
    if self.schema_data.is_none() {
      // TODO: Return this code path to normal once we properly load schemas
      Warn!("expected", "<model>", None, None, "TODO");
      // Warn('expected', '<model>', undef, "No Schema Model has been declared; assuming LaTeXML");
      // // article ??? or what ? undef gives problems!

      self.register_document_namespace("ltx", Some(LTX_NAMESPACE.to_string()));
      self.set_relaxng_schema(s!("LaTeXML"));
      self.register_namespace("ltx", Some(LTX_NAMESPACE.to_string()));
      self.register_namespace("svg", Some(s!("http://www.w3.org/2000/svg")));
      self.register_namespace("xlink", Some(s!("http://www.w3.org/1999/xlink"))); // Needed for SVG
      self.register_namespace("m", Some(s!("http://www.w3.org/1998/Math/MathML")));
      self.register_namespace("xhtml", Some(s!("http://www.w3.org/1999/xhtml")));
      self.permissive = true;
    } // Actually, they could have declared all sorts of Tags....
    let mut schema_type = String::new();
    match self.schema_data {
      None => {},
      Some(ref data) => {
        schema_type = data[0].clone();
        match schema_type.as_ref() {
          "DTD" => {
            // NOTE: This is a hack, as DTD should be deprecated, just making xii test work for now
            // ($roottag, $publicid, $systemid) = @data;
            name = data.last().unwrap().replace(".dtd", "");
            self.schema = Some(Relaxng {
              name: "DTD".to_string(), // HACK, phase out DTD support!
              ..Relaxng::default()
            });
            // $systemid);
          },
          "RelaxNG" => {
            name = data[1].to_string();
            self.schema = Some(Relaxng {
              name: name.clone(),
              ..Relaxng::default()
            });
          },
          e => {
            let message = s!("Can't load a schema of type {:?}", e);
            Error!("unknown", "schematype", self, None, message)
          },
        };
      },
    };

    if !self.no_compiled {
      let paths: Option<Vec<String>> = if search_paths.is_empty() {
        None
      } else {
        Some(search_paths.iter().map(ToString::to_string).collect())
      };
      let pathname_opt = pathname::find(
        &name,
        pathname::PathnameFindOptions {
          paths,
          types: Some(vec![s!("model")]),
          installation_subdir: Some(s!("resources/{}", schema_type)),
        },
      );

      match pathname_opt {
        Some(compiled_path) => self.load_compiled_schema(&compiled_path),
        None => self.schema.as_mut().unwrap().load_schema(),
      };
    }

    if self.debug_mode {
      self.describe_model()
    }

    &self.schema
  }
  pub fn describe_model(&self) {}

  pub fn get_xpath<'o>(&'o self, document: &'o XmlDoc) -> XPath {
    let mut context = XPath::new(document, HashMap::new());
    for (prefix, ns) in &self.code_namespaces {
      // TODO: Is this too slow? We may need to store an active context in the State as an
      // alternative
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
    let namespace_opt_checked = namespace_opt.filter(|val| !val.is_empty());

    match namespace_opt_checked {
      Some(namespace) => {
        self.code_namespace_prefixes.insert(namespace.clone(), codeprefix.to_string());
        self.code_namespaces.insert(codeprefix.to_string(), namespace);
      },
      None => {
        match self.code_namespaces.get(codeprefix) {
          Some(prev) => self.code_namespace_prefixes.remove(prev),
          None => None,
        };
        self.code_namespaces.remove(codeprefix);
      },
    };
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
          s!("DEFAULT#{}", &namespace)
        } else {
          namespace.to_string()
        };
        self.document_namespace_prefixes.insert(regnamespace, docprefix.to_string());
        self.document_namespaces.insert(docprefix.to_string(), namespace);
      },
      None => {
        if let Some(prev) = self.document_namespaces.get(docprefix) {
          self.document_namespace_prefixes.remove(prev);
        };
        self.document_namespaces.remove(docprefix);
      },
    };
  }

  pub fn get_document_namespace_prefix(&mut self, namespace: &str, forattribute: bool, probe: bool) -> Option<String> {
    // Get the prefix associated with the namespace url, noting that for elements, it might by
    // "#default", but for attributes would never be.
    // log!("Searching for {:?} in {:?}", namespace, self.document_namespace_prefixes);
    let mut docprefix = if !forattribute {
      self
        .document_namespace_prefixes
        .get(&s!("DEFAULT#{}", namespace))
        .map(|prefix| prefix.to_string())
    } else {
      None
    };
    if docprefix.is_none() {
      docprefix = self.document_namespace_prefixes.get(namespace).map(|prefix| prefix.to_string());
    }

    if docprefix.is_none() && !probe {
      self.namespace_errors += 1;
      docprefix = Some(s!("namespace{}", &self.namespace_errors.to_string()));
      self.register_document_namespace(docprefix.as_ref().unwrap(), Some(namespace.to_string()));
      let message2 = if let Some(ref dp) = docprefix {
        s!("Using '{}' instead", dp)
      } else {
        String::from("No prefix to fall back on.")
      };
      Warn!(
        "malformed",
        namespace,
        self,
        None,
        "No prefix has been registered for namespace (in document)",
        message2
      );
    }
    docprefix.filter(|p| p != "#default")
  }

  pub fn get_document_namespace(&mut self, mut docprefix: &str, probe: bool) -> Option<String> {
    if docprefix.is_empty() {
      docprefix = "#default";
    }
    let ns_str = match self.document_namespaces.get(docprefix) {
      None => String::new(),
      Some(s) => {
        if s.starts_with("DEFAULT#") {
          s.replacen("DEFAULT#", "", 1)
        } else {
          s.to_string()
        }
      },
    };

    if docprefix != "#default" && ns_str.is_empty() && !probe {
      self.namespace_errors += 1;
      let ns_error = s!("http://example.com/namespace{}", &self.namespace_errors.to_string());
      self.register_document_namespace(docprefix, Some(ns_error));
      let msg1 = s!("No namespace has been registered for prefix '{}' (in document)", docprefix);
      let msg2 = s!("Using '{}' instead", ns_str);
      Error!("malformed", docprefix, self, None, msg1, msg2);
    }
    if ns_str.is_empty() {
      None
    } else {
      Some(ns_str)
    }
  }

  /// In the following:
  /// $forattribute is 1 if the namespace is for an attribute (in which case, there must be a
  /// non-empty prefix) $probe, if non 0, just test for namespace, without creating an entry
  /// if missing. Get the (code) prefix associated with $namespace,
  /// creating a dummy prefix and signalling an error if none has been registered.
  pub fn get_namespace_prefix(&mut self, namespace: &str, _forattribute: bool, probe: bool) -> Option<String> {
    let mut codeprefix: Option<String> = None;

    if !namespace.is_empty() {
      codeprefix = self.code_namespace_prefixes.get(namespace).cloned();

      if codeprefix.is_some() && !probe {
        {
          let docprefix = self.document_namespace_prefixes.get(namespace);
          // if there's a doc prefix and it's NOT already used in code namespace mapping
          if docprefix.is_some() && !self.code_namespaces.contains_key(docprefix.unwrap()) {
            codeprefix = docprefix.map(|p| p.to_string());
          }
        }
      } else {
        // Else synthesize one
        self.namespace_errors += 1;
        let auto_prefix = s!("namespace{}", &self.namespace_errors.to_string());
        codeprefix = Some(auto_prefix);
      }
      self.register_namespace(codeprefix.as_ref().unwrap(), Some(namespace.to_string()));
      // Warn!('malformed', $namespace, undef,
      //   "No prefix has been registered for namespace '$namespace' (in code)",
      //   "Using '$codeprefix' instead"); }
    }

    codeprefix
  }

  pub fn get_namespace(&mut self, codeprefix: &str, probe: bool) -> Option<String> {
    let mut ns: Option<String> = self.code_namespaces.get(codeprefix).map(|ns| ns.to_string());
    if ns.is_none() && !probe {
      self.namespace_errors += 1;
      let example_namespace = s!("http://example.com/namespace{}", &self.namespace_errors.to_string());
      ns = Some(example_namespace.clone());
      self.register_namespace(codeprefix, Some(example_namespace));
      Error!(
        "malformed",
        codeprefix,
        self,
        None,
        "No namespace has been registered for prefix '$codeprefix' (in code)",
        "Using '$ns' isntead"
      );
    }
    ns
  }

  /// Get the node's qualified name in standard form
  /// Ie. using the registered (code) prefix for that namespace.
  /// NOTE: Reconsider how _Capture_ & _WildCard_ should be integrated!?!
  pub fn get_node_qname(&self, node: &Node) -> String {
    use libxml::tree::NodeType::*;
    let node_type = node.get_type();
    if node_type.is_none() {
      return s!("#BrokenNode");
    }
    match node_type.unwrap() {
      TextNode => s!("#PCDATA"),
      DocumentNode => s!("#Document"),
      CommentNode => s!("#Comment"),
      PiNode => s!("#ProcessingInstruction"),
      DTDNode => s!("#DTD"),
      NamespaceDecl => {
        // match node.declared_uri() {
        //   Some(ns) => match self.get_namespace_prefix(ns, false, true) {
        //     Some(prefix) => s!("xmlns:")+prefix,
        //     None => s!("xmlns")
        //   },
        //   None => s!("xmlns")
        // }
        s!("xmlns")
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
        match node.get_name().as_str() {
          "song" | "verse" | "line" => node.get_name(),
          regular => s!("ltx:{}", regular),
        }
      },
      // Need others?
      t => panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {:?}", t), /* Fatal('misdefined', '<caller>',
                                                                                                            * undef,
                                                                                                            *   "Should not ask for Qualified Name
                                                                                                            * for node of type $type: " .
                                                                                                            * Stringify($node)); */
    }
  }

  /// Same as get_node_qname, but using the Document namespace prefixes
  pub fn get_node_document_qname(&mut self, node: &Node) -> String {
    use libxml::tree::NodeType::*;
    let node_type = node.get_type();
    if node_type.is_none() {
      return s!("#BrokenNode");
    }

    match node_type.unwrap() {
      TextNode => s!("#PCDATA"),
      DocumentNode => s!("#Document"),
      CommentNode => s!("#Comment"),
      PiNode => s!("#ProcessingInstruction"),
      DTDNode => s!("#DTD"),

      // TODO
      // elsif ($type == XML_NAMESPACE_DECL) {
      //   my $ns = $node->declaredURI;
      //   my $prefix = $ns && $self->getDocumentNamespacePrefix($ns, 0, 1);
      //   return ($prefix ? 'xmlns:' . $prefix : 'xmlns'); }
      NamespaceDecl => s!("xmlns"),

      ElementNode | AttributeNode => {
        let mut prefix = String::new();
        if let Some(ns) = node.get_namespace() {
          let href = ns.get_href();
          if !href.is_empty() {
            prefix = self.get_document_namespace_prefix(&href, false, true).unwrap_or_default();
          }
        }
        if prefix.is_empty() {
          node.get_name()
        } else {
          s!("{}:{}", prefix, node.get_name())
        }
      },
      // Need others?
      t => panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {:?}", t),
    }
  }

  /// Given a Qualified name, possibly prefixed with a namespace prefix,
  /// as defined by the code namespace mapping,
  /// return the NamespaceURI and localname.
  pub fn decode_qname(&mut self, codetag: &str) -> (Option<String>, String) {
    match PREFIXED_LOCALNAME_RE.captures(codetag) {
      Some(captures) => {
        let prefix = captures.get(1).map_or("", |m| m.as_str());
        let localname = captures.get(2).map_or("", |m| m.as_str());

        if prefix == "xml" {
          (None, codetag.to_string())
        } else {
          (self.get_namespace(prefix, false), localname.to_string())
        }
      },
      None => (None, codetag.to_string()),
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
      "#PCDATA" | "#Comment" => return false,
      "_WildCard_" => return true,
      _ => {},
    };
    if CAPTURE_TAG_RE.is_match(tag) || CAPTURE_TAG_RE.is_match(child) {
      // with or without namespace prefix
      return true;
    }

    match child {
      "_WildCard_" | "#Comment" | "#ProcessingInstruction" | "#DTD" => return true,
      _ => {},
    };
    if self.permissive && tag == "#Document" && child != "#PCDATA" {
      return true; // No DTD? Punt!
    }

    // Else query tag properties.
    let model = &mut self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default).model;
    model.contains("ANY") || model.contains(child)
  }

  pub fn can_have_attribute(&mut self, tag: &str, attrib: &str) -> bool {
    // Handle obvious cases explicitly.
    match tag {
      "#PCDATA" | "#Comment" | "#Document" | "#ProcessingInstruction" | "#DTD" => return false,
      "_WildCard_" => return true,
      _ => {},
    };

    if CAPTURE_TAG_RE.is_match(tag) {
      return true;
    }

    if self.permissive {
      return true;
    }

    // Else query tag properties.
    let attributes = &mut self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default).attributes;

    attributes.contains(attrib)
  }

  pub fn is_node_in_schema_class(&self, class_name: &str, tag: &Node) -> bool {
    let tag = self.get_node_qname(tag);
    self.is_in_schema_class(class_name, &tag)
  }
  pub fn is_in_schema_class(&self, class_name: &str, tag: &str) -> bool {
    if let Some(class) = self.schema_class.get(class_name) {
      class.contains(tag)
    } else {
      false
    }
  }

  /// TODO: This is another component that would fit perfectly as a compiler plugin,
  /// which generates a rust objects from all available schemas and has them directly
  /// available at runtime For now, simply reimplementing the runtime loading of
  /// LaTeXML.model as-is from Model.pm
  pub fn load_compiled_schema(&mut self, path: &str) {
    note_begin(&s!("Loading compiled schema {}\n", path));
    let compiled_fh = File::open(path).unwrap();
    let compiled_reader = BufReader::new(&compiled_fh);
    for line in compiled_reader.lines().flatten() {
      if let Some(caps) = TAG_MODEL_LINE.captures(&line) {
        let tag = caps.get(1).map_or("", |m| m.as_str());
        let attr = caps.get(2).map_or("", |m| m.as_str());
        let children = caps.get(3).map_or("", |m| m.as_str());
        self.add_tag_attribute(tag, attr.split(',').collect());
        self.add_tag_content(tag, children.split(',').collect());
      } else if let Some(caps) = CLASS_MODEL_LINE.captures(&line) {
        let classname = caps.get(1).map_or("", |m| m.as_str());
        let elements = caps.get(2).map_or("", |m| m.as_str());
        let mut class_set = HashSet::new();
        for set_element in elements.split(',').collect::<Vec<&str>>() {
          class_set.insert(set_element.to_owned());
        }
        self.set_schema_class(classname, class_set);
      } else if let Some(caps) = NAMESPACE_MODEL_LINE.captures(&line) {
        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let namespace = caps.get(2).map_or("", |m| m.as_str());
        self.register_document_namespace(prefix, Some(namespace.to_owned()));
      } else {
        panic!("Fatal:internal:{} Compiled model '{}' is malformatted at \"{}\"", path, path, line);
      }
    }

    note_end(&s!("Loading compiled schema {}\n", path));
  }

  //**********************************************************************
  // Accessors
  //**********************************************************************

  pub fn get_tags(&self) -> Vec<String> {
    let mut keys: Vec<String> = self.tagprop.keys().map(String::as_str).map(str::to_owned).collect();
    keys.sort();
    keys
  }

  pub fn get_tag_contents(&self, tag: &str) -> Vec<&str> {
    match self.tagprop.get(tag) {
      Some(h) => {
        let mut keys: Vec<&str> = h.model.iter().map(String::as_str).collect();
        keys.sort_unstable();
        keys
      },
      None => Vec::new(),
    }
  }

  pub fn add_tag_content(&mut self, tag: &str, elements: Vec<&str>) {
    let frame = self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default);

    for element in elements {
      frame.model.insert(element.to_owned());
    }
  }

  pub fn get_tag_attributes(&self, tag: &str) -> Vec<&str> {
    match self.tagprop.get(tag) {
      Some(h) => {
        let mut keys: Vec<&str> = h.attributes.iter().map(String::as_str).collect();
        keys.sort_unstable();
        keys
      },
      None => Vec::new(),
    }
  }

  pub fn add_tag_attribute(&mut self, tag: &str, attributes: Vec<&str>) {
    let frame = self.tagprop.entry(tag.to_owned()).or_insert_with(TagFrame::default);

    for attribute in attributes {
      frame.attributes.insert(attribute.to_owned());
    }
  }

  pub fn set_schema_class(&mut self, classname: &str, content: HashSet<String>) { self.schema_class.insert(classname.to_owned(), content); }
}
