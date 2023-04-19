use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::Cow;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::string::ToString;
use string_interner::symbol::SymbolU32;

use crate::common::arena::{self, ANY_SYM};
use crate::common::error::*;
use crate::common::object::Object;
use crate::common::relaxng::Relaxng;
use crate::common::xml::{XPath, XML_NS};
use crate::document::Document;
use crate::util::pathname;
use crate::Locator;
use libxml::tree::Document as XmlDoc;
use libxml::tree::Node;

use super::arena::{H_PCDATA_SYM, H_COMMENT_SYM,EMPTY_SYM,WILD_CARD_SYM,H_PI_SYM,DTD_SYM,H_DTD_SYM,H_DOC_SYM,RELAXNG_SYM,H_DEFAULT_SYM};

// use common::font::*;

pub const LTX_NAMESPACE: &str = "http://dlmf.nist.gov/LaTeXML";
pub type IndirectModel = HashMap<SymbolU32, HashMap<SymbolU32, SymbolU32>>;

static PREFIXED_LOCALNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^:]+):(.+)$").unwrap());
static CAPTURE_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.*?:)?_Capture_$").unwrap());
static TAG_MODEL_LINE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap());
static CLASS_MODEL_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^:=]+):=(.*?)$").unwrap());
static NAMESPACE_MODEL_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^=]+)=(.*?)$").unwrap());

#[derive(Default)]
pub struct TagFrame {
  model: HashSet<SymbolU32>,
  attributes: HashSet<SymbolU32>,
}

static DEFAULT_TAG_FRAME :Lazy<TagFrame> = Lazy::new(TagFrame::default);

#[derive(Default)]
pub struct Model {
  pub schema: Option<Relaxng>,
  pub schema_data: Option<Vec<SymbolU32>>,
  pub schema_class: HashMap<SymbolU32, HashSet<SymbolU32>>,
  pub code_namespace_prefixes: HashMap<SymbolU32, SymbolU32>,
  pub code_namespaces: HashMap<SymbolU32, SymbolU32>,
  pub document_namespace_prefixes: HashMap<SymbolU32, SymbolU32>,
  pub document_namespaces: HashMap<SymbolU32, SymbolU32>,
  // doctype_namespaces: HashMap<SymbolU32, SymbolU32>,
  // namespace_errors: usize,
  pub permissive: bool,
  pub no_compiled: bool,
  pub debug_mode: bool,
  pub namespace_errors: u8,
  pub tagprop: HashMap<SymbolU32, TagFrame>,
}

impl Object for Model {
  fn get_locator(&self) -> Option<Cow<Locator>> { None }
}
impl Model {
  pub fn new() -> Self {
    let mut model = Model::default();
    // model.xpath.register_function("match-font", |x, y| {font::match_font(x,y)})
    model.register_namespace("xml", Some(XML_NS));
    model.register_document_namespace("xml", Some(XML_NS));
    model
  }

  pub fn set_doc_type(&mut self, roottag: &str, publicid: &str, systemid: &str) {
    self.schema_data = Some(vec![
      DTD_SYM.with(|sym| *sym),
      arena::pin(roottag),
      arena::pin(publicid),
      arena::pin(systemid)]);
  }

  pub fn set_relaxng_schema(&mut self, schema: &str) {
    self.schema_data = Some(vec![RELAXNG_SYM.with(|sym| *sym), arena::pin(schema)]);
  }
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

      self.register_document_namespace("ltx", Some(LTX_NAMESPACE));
      self.set_relaxng_schema("LaTeXML");
      self.register_namespace("ltx", Some(LTX_NAMESPACE));
      self.register_namespace("svg", Some("http://www.w3.org/2000/svg"));
      self.register_namespace("xlink", Some("http://www.w3.org/1999/xlink")); // Needed for SVG
      self.register_namespace("m", Some("http://www.w3.org/1998/Math/MathML"));
      self.register_namespace("xhtml", Some("http://www.w3.org/1999/xhtml"));
      self.permissive = true;
    } // Actually, they could have declared all sorts of Tags....
    let mut schema_type = EMPTY_SYM.with(|sym| *sym);
    match self.schema_data {
      None => {},
      Some(ref data) => {
        schema_type = data[0];
        if schema_type == DTD_SYM.with(|sym| *sym) {
            // NOTE: This is a hack, as DTD should be deprecated, just making xii test work for now
            // ($roottag, $publicid, $systemid) = @data;
            name = arena::with(*data.last().unwrap(), |data_str| data_str.replace(".dtd", ""));
            self.schema = Some(Relaxng {
              name: "DTD".to_string(), // HACK, phase out DTD support!
              ..Relaxng::default()
            });
            // $systemid);
        } else if schema_type == RELAXNG_SYM.with(|sym| *sym) {
          name = arena::to_string(data[1]);
          self.schema = Some(Relaxng {
            name: name.clone(),
            ..Relaxng::default()
          });
        } else {
          let message = arena::with(schema_type, |schema_type_str| s!("Can't load a schema of type {schema_type_str:?}"));
          Error!("unknown", "schematype", self, None, message)
        }
      }
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
          extensions: Some(vec![s!("model")]),
          installation_subdir: Some(arena::with(schema_type, |str|
            s!("resources/{str}"))),
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
    let mut context = XPath::new(document, HashMap::default());
    for (prefix, ns) in &self.code_namespaces {
      // TODO: Is this too slow? We may need to store an active context in the State as an
      // alternative
      arena::with(*prefix, |p_str|
        arena::with(*ns, |ns_str|
          context.register_namespace(p_str, ns_str)));
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
  pub fn register_namespace(&mut self, codeprefix: &str, namespace_opt: Option<&str>) {
    self.register_namespace_sym(arena::pin(codeprefix), namespace_opt.map(arena::pin))
  }
  pub fn register_namespace_sym(&mut self, codeprefix: SymbolU32, namespace_opt: Option<SymbolU32>) {
    // double-check empty strings are None
    let namespace_opt_checked = namespace_opt.filter(|val| *val != EMPTY_SYM.with(|sym| *sym));
    match namespace_opt_checked {
      Some(namespace) => {
        self
          .code_namespace_prefixes
          .insert(namespace, codeprefix);
        self
          .code_namespaces
          .insert(codeprefix, namespace);
      },
      None => {
        match self.code_namespaces.get(&codeprefix) {
          Some(prev) => self.code_namespace_prefixes.remove(prev),
          None => None,
        };
        self.code_namespaces.remove(&codeprefix);
      },
    };
  }

  pub fn register_document_namespace(
    &mut self,
    docprefix: &str,
    namespace_opt: Option<&str>,
  ) {
    let default_sym = H_DEFAULT_SYM.with(|sym| *sym);
    let docprefix_sym = if docprefix.is_empty() {
      default_sym
    } else {
      arena::pin(docprefix)
    };

    match namespace_opt {
      Some(namespace) => {
        // Since the default namespace url can still ALSO have a prefix associated,
        // we prepend "DEFAULT#url" when using as a hash key in the prefixes table.
        let ns_sym = arena::pin(namespace);
        let regnamespace = if docprefix_sym == default_sym {
          arena::pin(s!("DEFAULT#{namespace}"))
        } else {
          ns_sym
        };
        self
          .document_namespace_prefixes
          .insert(regnamespace, docprefix_sym);
        self
          .document_namespaces
          .insert(docprefix_sym, ns_sym);
      },
      None => {
        if let Some(prev) = self.document_namespaces.get(&docprefix_sym) {
          self.document_namespace_prefixes.remove(prev);
        };
        self.document_namespaces.remove(&docprefix_sym);
      },
    };
  }

  pub fn get_document_namespace_prefix(
    &mut self,
    namespace: &str,
    forattribute: bool,
    probe: bool,
  ) -> Option<SymbolU32> {
    // Get the prefix associated with the namespace url, noting that for elements, it might by
    // "#default", but for attributes would never be.
    // log!("Searching for {:?} in {:?}", namespace, self.document_namespace_prefixes);
    let mut docprefix = if !forattribute {
      self.document_namespace_prefixes
        .get(&arena::pin(s!("DEFAULT#{namespace}")))
        .copied()
    } else {
      None
    };
    let ns_sym = arena::pin(namespace);
    if docprefix.is_none() {
      docprefix = self.document_namespace_prefixes
        .get(&ns_sym)
        .copied();
    }

    if docprefix.is_none() && !probe {
      self.namespace_errors += 1;
      let ns_err = s!("namespace{}", &self.namespace_errors.to_string());
      docprefix = Some(arena::pin(&ns_err));
      self.register_document_namespace(&ns_err, Some(namespace));
      let message2 = if let Some(dp) = docprefix {
        arena::with(dp, |dp_str| s!("Using '{dp_str}' instead"))
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
    let default_sym = H_DEFAULT_SYM.with(|sym| *sym);
    docprefix.filter(|p| p != &default_sym)
  }

  pub fn get_document_namespace(&mut self, docprefix: &str, probe: bool) -> Option<String> {
    let h_default_sym = H_DEFAULT_SYM.with(|sym| *sym);
    let docprefix_sym = if docprefix.is_empty() {
      h_default_sym
    } else {
      arena::pin(docprefix)
    };
    let ns_str = match self.document_namespaces.get(&docprefix_sym) {
      None => String::new(),
      Some(sym) => {
        arena::with(*sym, |s|
        if s.starts_with("DEFAULT#") {
          s.replacen("DEFAULT#", "", 1)
        } else {
          s.to_string()
        })
      },
    };

    if docprefix_sym != h_default_sym && ns_str.is_empty() && !probe {
      self.namespace_errors += 1;
      let ns_error = s!(
        "http://example.com/namespace{}",
        &self.namespace_errors.to_string()
      );
      self.register_document_namespace(docprefix, Some(&ns_error));
      let msg1 = arena::with(docprefix_sym, |dp_str| s!(
        "No namespace has been registered for prefix '{dp_str}' (in document)"));
      let msg2 = s!("Using '{ns_str}' instead");
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
  pub fn get_namespace_prefix(
    &mut self,
    namespace: &str,
    _forattribute: bool,
    probe: bool,
  ) -> Option<SymbolU32> {
    let mut codeprefix: Option<SymbolU32> = None;
    let ns_sym = arena::pin(namespace);
    if !namespace.is_empty() {
      codeprefix = self.code_namespace_prefixes.get(&ns_sym).copied();

      if codeprefix.is_some() && !probe {
        {
          let docprefix = self.document_namespace_prefixes.get(&ns_sym);
          // if there's a doc prefix and it's NOT already used in code namespace mapping
          if docprefix.is_some() && !self.code_namespaces.contains_key(docprefix.unwrap()) {
            codeprefix = docprefix.copied();
          }
        }
      } else {
        // Else synthesize one
        self.namespace_errors += 1;
        let auto_prefix = arena::pin(s!("namespace{}", &self.namespace_errors.to_string()));
        codeprefix = Some(auto_prefix);
      }
      self.register_namespace_sym(codeprefix.unwrap(), Some(arena::pin(namespace)));
      // Warn!('malformed', $namespace, undef,
      //   "No prefix has been registered for namespace '$namespace' (in code)",
      //   "Using '$codeprefix' instead"); }
    }

    codeprefix
  }

  pub fn get_namespace(&mut self, codeprefix: &str, probe: bool) -> Option<SymbolU32> {
    let mut ns: Option<SymbolU32> = self
      .code_namespaces
      .get(&arena::pin(codeprefix))
      .copied();
    if ns.is_none() && !probe {
      self.namespace_errors += 1;
      let example_namespace = s!(
        "http://example.com/namespace{}",
        &self.namespace_errors.to_string()
      );
      ns = Some(arena::pin(&example_namespace));
      self.register_namespace(codeprefix, Some(&example_namespace));
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
  pub fn get_node_qname<'a>(&'a self, node: &'a Node) -> SymbolU32 {
    use libxml::tree::NodeType::*;
    let node_type = node.get_type();
    if node_type.is_none() {
      return arena::pin("#BrokenNode");
    }
    match node_type.unwrap() {
      TextNode => arena::pin_static("#PCDATA"),
      DocumentNode => arena::pin_static("#Document"),
      CommentNode => arena::pin_static("#Comment"),
      PiNode => arena::pin_static("#ProcessingInstruction"),
      DTDNode => arena::pin_static("#DTD"),
      NamespaceDecl => {
        // match node.declared_uri() {
        //   Some(ns) => match self.get_namespace_prefix(ns, false, true) {
        //     Some(prefix) => s!("xmlns:")+prefix,
        //     None => s!("xmlns")
        //   },
        //   None => s!("xmlns")
        // }
        arena::pin_static("xmlns")
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
        let name_str = node.get_name();
        match name_str.as_str() {
          "song" | "verse" | "line" => arena::pin(name_str),
          regular => arena::pin(s!("ltx:{}", regular)),
        }
      },
      // Need others?
      t => {
        panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {t:?}")
      },
    }
  }

  pub fn with_node_qname<R,FnR>(&self, node: &Node, caller: FnR) -> R
  where FnR: FnOnce(&str) -> R  {
    arena::with(self.get_node_qname(node), |qname_str|
      caller(qname_str))
  }

  /// Same as get_node_qname, but using the Document namespace prefixes
  pub fn get_node_document_qname(&mut self, node: &Node) -> SymbolU32 {
    use libxml::tree::NodeType::*;
    let node_type = node.get_type();
    if node_type.is_none() {
      return arena::pin_static("#BrokenNode");
    }

    match node_type.unwrap() {
      TextNode => arena::pin_static("#PCDATA"),
      DocumentNode => arena::pin_static("#Document"),
      CommentNode => arena::pin_static("#Comment"),
      PiNode => arena::pin_static("#ProcessingInstruction"),
      DTDNode => arena::pin_static("#DTD"),

      // TODO
      // elsif ($type == XML_NAMESPACE_DECL) {
      //   my $ns = $node->declaredURI;
      //   my $prefix = $ns && $self->getDocumentNamespacePrefix($ns, 0, 1);
      //   return ($prefix ? 'xmlns:' . $prefix : 'xmlns'); }
      NamespaceDecl => arena::pin_static("xmlns"),

      ElementNode | AttributeNode => {
        let empty_sym = EMPTY_SYM.with(|sym| *sym);
        let mut prefix = empty_sym;
        if let Some(ns) = node.get_namespace() {
          let href = ns.get_href();
          if !href.is_empty() {
            prefix = self
              .get_document_namespace_prefix(&href, false, true)
              .unwrap_or(empty_sym);
          }
        }
        if prefix == empty_sym {
          arena::pin(node.get_name())
        } else {
          arena::pin(arena::with(prefix, |prefix_str|
            s!("{}:{}", prefix_str, node.get_name())))
        }
      },
      // Need others?
      t => {
        panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {t:?}")
      },
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
          (self.get_namespace(prefix, false)
            .map(arena::to_string),
          localname.to_string())
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

  pub fn can_contain_sym(&mut self, tag:SymbolU32, child:SymbolU32) -> bool {
    // Handle obvious cases explicitly.
    if H_PCDATA_SYM.with(|sym| tag == *sym) ||
       H_COMMENT_SYM.with(|sym| tag == *sym) ||
       EMPTY_SYM.with(|sym| tag == *sym) {
      return false
    } else if WILD_CARD_SYM.with(|sym| tag == *sym) {
      return true
    };
    if arena::with(tag, |tag_str| CAPTURE_TAG_RE.is_match(tag_str)) ||
      arena::with(child, |child_str| CAPTURE_TAG_RE.is_match(child_str)) {
      // with or without namespace prefix
      return true;
    }

    if WILD_CARD_SYM.with(|sym| child == *sym) ||
      H_COMMENT_SYM.with(|sym| child == *sym) ||
      H_PI_SYM.with(|sym| child == *sym) ||
      H_DTD_SYM.with(|sym| child == *sym) {
      return true
    }

    if self.permissive && H_DOC_SYM.with(|sym| tag == *sym) &&
      H_PCDATA_SYM.with(|sym| child != *sym) {
      return true; // No DTD? Punt!
    }

    // Else query tag properties.
    let model = &mut self
      .tagprop
      .entry(tag)
      .or_insert_with(TagFrame::default)
      .model;
    ANY_SYM.with(|sym| model.contains(sym)) || model.contains(&child)
  }

  /// Can an element with (qualified name) `tag` contain a `child` element?
  pub fn can_contain(&self, tag: &str, child: &str) -> bool {
    // Handle obvious cases explicitly.
    match tag {
      "#PCDATA" | "#Comment" | "" => return false,
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
    let model = &self
      .tagprop
      .get(&arena::pin(tag))
      .unwrap_or(&*DEFAULT_TAG_FRAME)
      .model;
    ANY_SYM.with(|sym| model.contains(sym)) || model.contains(&arena::pin(child))
  }

  pub fn can_have_attribute(&self, tag: SymbolU32, attrib: SymbolU32) -> bool {
    // Handle obvious cases explicitly.
    if let Some(early_choice) = arena::with(tag, |tag_str| match tag_str {
      "#PCDATA" | "#Comment" | "#Document" | "#ProcessingInstruction" | "#DTD" => Some(false),
      "_WildCard_" => Some(true),
      other if CAPTURE_TAG_RE.is_match(other) => Some(true),
      _ => None
    }) {
      return early_choice
    };
    if self.permissive {
      return true;
    }

    // Else query tag properties.
    let attributes = &self
      .tagprop
      .get(&tag)
      .unwrap_or(&*DEFAULT_TAG_FRAME)
      .attributes;
    attributes.contains(&attrib)
  }

  pub fn is_node_in_schema_class(&self, class_name: &str, tag: &Node) -> bool {
    let tag = self.get_node_qname(tag);
    self.is_in_schema_class(&arena::pin(class_name), &tag)
  }
  pub fn is_in_schema_class(&self, class_name: &SymbolU32, tag: &SymbolU32) -> bool {
    if let Some(class) = self.schema_class.get(class_name) {
      class.contains(tag)
    } else {
      false
    }
  }

  /// TODO: This is another component that would fit perfectly as a compiler plugin.
  /// For now, simply reimplementing the runtime loading of
  /// LaTeXML.model as-is from Model.pm
  pub fn load_compiled_schema(&mut self, path: &str) {
    note_begin(&s!("Loading compiled schema {}\n", path));
    let compiled_fh = File::open(path).unwrap();
    let compiled_reader = BufReader::new(&compiled_fh);
    for line in compiled_reader.lines().flatten() {
      if let Some(caps) = TAG_MODEL_LINE_RE.captures(&line) {
        let tag = caps.get(1).map_or("", |m| m.as_str());
        let attr = caps.get(2).map_or("", |m| m.as_str());
        let children = caps.get(3).map_or("", |m| m.as_str());
        self.add_tag_attribute(tag, attr.split(',').collect());
        self.add_tag_content(tag, children.split(',').collect());
      } else if let Some(caps) = CLASS_MODEL_LINE_RE.captures(&line) {
        let classname = caps.get(1).map_or("", |m| m.as_str());
        let elements = caps.get(2).map_or("", |m| m.as_str());
        let mut class_set = HashSet::default();
        for set_element in elements.split(',').collect::<Vec<&str>>() {
          class_set.insert(arena::pin(set_element));
        }
        self.set_schema_class(classname, class_set);
      } else if let Some(caps) = NAMESPACE_MODEL_LINE_RE.captures(&line) {
        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let namespace = caps.get(2).map_or("", |m| m.as_str());
        self.register_document_namespace(prefix, Some(namespace));
      } else {
        panic!("Fatal:internal:{path} Compiled model '{path}' is malformatted at \"{line}\"");
      }
    }

    note_end(&s!("Loading compiled schema {path}\n"));
  }

  //**********************************************************************
  // Accessors
  //**********************************************************************

  pub fn get_tags(&self) -> Vec<&SymbolU32> {
    self
      .tagprop
      .keys()
      .collect()
  }
  pub fn get_sym_tags(&self) -> Vec<SymbolU32> { self.tagprop.keys().copied().collect() }

  pub fn get_tag_contents(&self, tag: &SymbolU32) -> Vec<SymbolU32> {
    match self.tagprop.get(tag) {
      Some(h) => h.model.iter().copied().collect(),
      None => Vec::new(),
    }
  }

  pub fn add_tag_content(&mut self, tag: &str, elements: Vec<&str>) {
    let frame = self
      .tagprop
      .entry(arena::pin(tag))
      .or_insert_with(TagFrame::default);

    for element in elements {
      frame.model.insert(arena::pin(element));
    }
  }

  // pub fn get_tag_attributes(&self, tag: &str) -> Vec<&str> {
  //   match self.tagprop.get(&arena::pin(tag)) {
  //     Some(h) => {
  //       let mut keys: Vec<&str> = h.attributes.iter().map(|s| arena::resolve(*s)).collect();
  //       keys.sort_unstable();
  //       keys
  //     },
  //     None => Vec::new(),
  //   }
  // }

  pub fn add_tag_attribute(&mut self, tag: &str, attributes: Vec<&str>) {
    let frame = self
      .tagprop
      .entry(arena::pin(tag))
      .or_insert_with(TagFrame::default);

    for attribute in attributes {
      frame.attributes.insert(arena::pin(attribute));
    }
  }

  pub fn set_schema_class(&mut self, classname: &str, content: HashSet<SymbolU32>) {
    self.schema_class.insert(arena::pin(classname), content);
  }
}
