use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashSet as HashSet;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use crate::common::arena::{self, SymStr};
use crate::common::error::*;
use crate::common::relaxng::Relaxng;
use crate::common::xml::XML_NS;
use crate::document::Document;
use crate::util::pathname;
use libxml::tree::Node;

use super::arena::SymHashMap;
use crate::pin;

// use common::font::*;

pub const LTX_NAMESPACE: &str = "http://dlmf.nist.gov/LaTeXML";
pub type IndirectModel = SymHashMap<SymHashMap<SymStr>>;

static PREFIXED_LOCALNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^:]+):(.+)$").unwrap());
static TAG_MODEL_LINE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^\{]+)\{(.*?)\}\((.*?)\)$").unwrap());
// Mirrors Perl Model.pm L149: `m/^([^:=]+):=\(?([^)]*?)\)?$/` — the
// `\(?…\)?` pair strips the surrounding parens from
// `classname:=(elt1,elt2,...)` so the elements split cleanly.
static CLASS_MODEL_LINE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^([^:=]+):=\(?([^)]*?)\)?$").unwrap());
static NAMESPACE_MODEL_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^=]+)=(.*?)$").unwrap());

#[derive(Default, Debug)]
pub struct TagFrame {
  model:      HashSet<SymStr>,
  attributes: HashSet<SymStr>,
}

static DEFAULT_TAG_FRAME: Lazy<TagFrame> = Lazy::new(TagFrame::default);

#[derive(Default, Debug)]
pub struct Model {
  pub schema:                      Option<Relaxng>,
  pub schema_data:                 Option<Vec<SymStr>>,
  pub schema_class:                SymHashMap<HashSet<SymStr>>,
  pub code_namespace_prefixes:     SymHashMap<SymStr>,
  pub code_namespaces:             SymHashMap<SymStr>,
  pub document_namespace_prefixes: SymHashMap<SymStr>,
  pub document_namespaces:         SymHashMap<SymStr>,
  // doctype_namespaces: SymHashMap<SymStr>,
  // namespace_errors: usize,
  pub permissive:                  bool,
  pub no_compiled:                 bool,
  pub debug_mode:                  bool,
  pub namespace_errors:            u8,
  pub tagprop:                     SymHashMap<TagFrame>,
}

#[thread_local]
pub static MODEL: Lazy<RefCell<Model>> = Lazy::new(|| RefCell::new(Model::new()));

macro_rules! model {
  () => {
    (*MODEL).borrow()
  };
}
macro_rules! model_mut {
  () => {
    (*MODEL).borrow_mut()
  };
}

pub fn initialize_model() {
  let mut global_model = MODEL.borrow_mut();
  *global_model = Model::new();
}

impl Model {
  pub fn new() -> Self {
    let mut model = Model::default();
    // model.xpath.register_function("match-font", |x, y| {font::match_font(x,y)})
    model.register_namespace("xml", Some(XML_NS));
    model.register_document_namespace("xml", Some(XML_NS));
    model
  }
  ///**********************************************************************
  /// Namespaces
  ///**********************************************************************
  /// There are TWO namespace mappings!!!
  /// One for coding, one for the document output.
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
  pub fn register_namespace_sym(&mut self, codeprefix: SymStr, namespace_opt: Option<SymStr>) {
    // double-check empty strings are None
    let namespace_opt_checked = namespace_opt.filter(|val| *val != pin!(""));
    match namespace_opt_checked {
      Some(namespace) => {
        self
          .code_namespace_prefixes
          .insert_sym(namespace, codeprefix);
        self.code_namespaces.insert_sym(codeprefix, namespace);
      },
      None => {
        if let Some(prev) = self.code_namespaces.get_sym(codeprefix) {
          self.code_namespace_prefixes.remove_sym(*prev);
        };
        self.code_namespaces.remove_sym(codeprefix);
      },
    };
  }

  pub fn register_document_namespace(&mut self, docprefix: &str, namespace_opt: Option<&str>) {
    let default_sym = pin!("#default");
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
          .insert_sym(regnamespace, docprefix_sym);
        self.document_namespaces.insert_sym(docprefix_sym, ns_sym);
      },
      None => {
        if let Some(prev) = self.document_namespaces.get_sym(docprefix_sym) {
          self.document_namespace_prefixes.remove_sym(*prev);
        };
        self.document_namespaces.remove_sym(docprefix_sym);
      },
    };
  }

  pub fn set_relaxng_schema(&mut self, schema: &str) {
    self.schema_data = Some(vec![pin!("RelaxNG"), arena::pin(schema)]);
  }
  /// TODO: This is another component that would fit perfectly as a compiler plugin.
  /// For now, simply reimplementing the runtime loading of
  /// LaTeXML.model as-is from Model.pm
  pub fn load_compiled_schema(&mut self, path: &str) {
    note_begin(&s!("Loading compiled schema {}\n", path));
    let compiled_fh = File::open(path).unwrap();
    let compiled_reader = BufReader::new(&compiled_fh);
    for line_item in compiled_reader.lines().map_while(std::result::Result::ok) {
      let line: String = line_item;
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
  pub fn add_tag_content(&mut self, tag: &str, elements: Vec<&str>) {
    let frame = self.tagprop.entry(tag).or_default();

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
    let frame = self.tagprop.entry(tag).or_default();

    for attribute in attributes {
      frame.attributes.insert(arena::pin(attribute));
    }
  }

  pub fn set_schema_class(&mut self, classname: &str, content: HashSet<SymStr>) {
    self.schema_class.insert(classname, content);
  }

  /// Serialise the loaded schema into the `.model` plain-text format
  /// emitted by Perl `LaTeXML::Common::Model::compileSchema`
  /// (Model.pm L121-136). Three kinds of lines, all newline-separated:
  ///
  /// * `prefix=namespace` for every entry in `document_namespaces` (sorted by prefix).
  /// * `classname:=(elt1,elt2,...)` for every entry in `schema_class` (sorted by classname; each
  ///   element list sorted).
  /// * `tag{attr1,attr2}(child1,child2)` for every entry in `tagprop` (sorted by tag; attrs and
  ///   children sorted; tags whose name starts with `!` are skipped — they are content-model-only
  ///   negations).
  ///
  /// Output is identical to the Perl tool so a downstream
  /// `tools/compileschema.sh` can diff Rust vs. Perl-generated
  /// `LaTeXML.model` files byte-for-byte (modulo schema content).
  pub fn dump_compiled_schema(&self) -> String {
    fn sym_to_string(sym: SymStr) -> String { arena::with(sym, |s| s.to_string()) }
    fn syms_sorted(set: impl IntoIterator<Item = SymStr>) -> Vec<String> {
      let mut v: Vec<String> = set.into_iter().map(sym_to_string).collect();
      v.sort();
      v
    }
    let mut out = String::new();
    let prefixes = syms_sorted(self.document_namespaces.keys().copied());
    for prefix in &prefixes {
      let ns_opt = self
        .document_namespaces
        .get_sym(arena::pin(prefix.as_str()));
      let ns = match ns_opt {
        Some(v) => sym_to_string(*v),
        None => continue,
      };
      out.push_str(prefix);
      out.push('=');
      out.push_str(&ns);
      out.push('\n');
    }
    let classnames = syms_sorted(self.schema_class.keys().copied());
    for classname in &classnames {
      let elements = match self.schema_class.get_sym(arena::pin(classname.as_str())) {
        Some(set) => set,
        None => continue,
      };
      let elt_names = syms_sorted(elements.iter().copied());
      out.push_str(classname);
      out.push_str(":=(");
      out.push_str(&elt_names.join(","));
      out.push_str(")\n");
    }
    let tags = syms_sorted(self.tagprop.keys().copied());
    for tag in &tags {
      if tag.starts_with('!') {
        continue;
      }
      let frame = match self.tagprop.get_sym(arena::pin(tag.as_str())) {
        Some(f) => f,
        None => continue,
      };
      let attrs = syms_sorted(frame.attributes.iter().copied());
      let children = syms_sorted(frame.model.iter().copied());
      out.push_str(tag);
      out.push('{');
      out.push_str(&attrs.join(","));
      out.push_str("}(");
      out.push_str(&children.join(","));
      out.push_str(")\n");
    }
    out
  }
  pub fn describe_model(&self) {}
  fn load_internal_extensions(&mut self) {
    if !self.tagprop.contains_key("ltx:_CaptureBlock_") {
      // Synthesize ltx:_CaptureBlock_ to act like the union of ltx:block, ltx:para,
      self.synthesize_element("ltx:_CaptureBlock_", &[
        "ltx:block",
        "ltx:logical-block",
        "ltx:sectional-block",
        "Caption",
      ]);
      let cb_entry = self.tagprop.entry("ltx:_CaptureBlock_").or_default();
      cb_entry.model.insert(arena::pin_static("svg:g"));
      cb_entry
        .model
        .insert(arena::pin_static("svg:foreignObject"));
    }
  }

  /// Clone the tagprop's (allowed content & attributes) of @other to $tag
  fn synthesize_element(&mut self, tag: &str, others: &[&str]) {
    let mut to_add_in_model = Vec::new();
    let mut to_add_in_attrs = Vec::new();
    for other in others {
      if let Some(content) = self.schema_class.get(other) {
        for child in content {
          to_add_in_model.push(*child);
        }
      } else if let Some(entry) = self.tagprop.get(other) {
        for child in &entry.model {
          to_add_in_model.push(*child);
        }
        for attr in &entry.attributes {
          to_add_in_attrs.push(*attr);
        }
      }
    }
    let capture = self.tagprop.entry(tag).or_default();
    for child in to_add_in_model {
      capture.model.insert(child);
    }
    for attr in to_add_in_attrs {
      capture.attributes.insert(attr);
    }
  }
}

pub fn set_relaxng_schema(schema: &str) { model_mut!().set_relaxng_schema(schema) }
pub fn add_schema_declaration(document: &mut Document) {
  if let Some(ref schema) = model!().schema {
    schema.add_schema_declaration(document);
  }
}

pub fn load_schema(search_paths: &[&str]) -> Result<()> {
  // Only load once
  let mut model = model_mut!();
  if model.schema.is_some() {
    return Ok(());
  }
  let mut name = String::new();
  if model.schema_data.is_none() {
    // TODO: Return this code path to normal once we properly load schemas
    Warn!("expected", "<model>", "TODO");
    // Warn('expected', '<model>', undef, "No Schema Model has been declared; assuming LaTeXML");
    // // article ??? or what ? undef gives problems!

    model.register_document_namespace("ltx", Some(LTX_NAMESPACE));
    model.set_relaxng_schema("LaTeXML");
    model.register_namespace("ltx", Some(LTX_NAMESPACE));
    model.register_namespace("svg", Some("http://www.w3.org/2000/svg"));
    model.register_namespace("xlink", Some("http://www.w3.org/1999/xlink")); // Needed for SVG
    model.register_namespace("m", Some("http://www.w3.org/1998/Math/MathML"));
    model.register_namespace("xhtml", Some("http://www.w3.org/1999/xhtml"));
    model.permissive = true;
  } // Actually, they could have declared all sorts of Tags....
  // Only RelaxNG schemas are supported (DTD support removed from Rust port)
  if let Some(ref data) = model.schema_data {
    if data[0] == pin!("RelaxNG") {
      name = arena::to_string(data[1]);
      model.schema = Some(Relaxng {
        name: name.clone(),
        ..Relaxng::default()
      });
    } else {
      let message = arena::with(data[0], |schema_type_str| {
        s!("Can't load a schema of type {schema_type_str:?}")
      });
      Error!("unknown", "schematype", message)
    }
  }

  if !model.no_compiled && model.schema.is_some() {
    let paths: Option<Vec<String>> = if search_paths.is_empty() {
      None
    } else {
      Some(search_paths.iter().map(ToString::to_string).collect())
    };
    let pathname_opt = pathname::find(&name, pathname::PathnameFindOptions {
      paths,
      extensions: Some(vec![s!("model")]),
      installation_subdir: Some(s!("resources/RelaxNG")),
      ..Default::default()
    });

    match pathname_opt {
      Some(compiled_path) => model.load_compiled_schema(&compiled_path),
      None => model.schema.as_mut().unwrap().load_schema(),
    };
  }
  model.load_internal_extensions();
  if model.debug_mode {
    model.describe_model()
  }

  Ok(())
}

pub fn get_document_namespace_prefix(
  namespace: &str,
  forattribute: bool,
  probe: bool,
) -> Option<SymStr> {
  // Get the prefix associated with the namespace url, noting that for elements, it might by
  // "#default", but for attributes would never be.
  // log!("Searching for {:?} in {:?}", namespace, self.document_namespace_prefixes);
  let mut docprefix = if !forattribute {
    model!()
      .document_namespace_prefixes
      .get(&s!("DEFAULT#{namespace}"))
      .copied()
  } else {
    None
  };
  let ns_sym = arena::pin(namespace);
  if docprefix.is_none() {
    docprefix = model!()
      .document_namespace_prefixes
      .get_sym(ns_sym)
      .copied();
  }

  if docprefix.is_none() && !probe {
    {
      model_mut!().namespace_errors += 1;
    }
    let ns_err = s!("namespace{}", &model!().namespace_errors.to_string());
    docprefix = Some(arena::pin(&ns_err));
    {
      model_mut!().register_document_namespace(&ns_err, Some(namespace));
    }
    let message2 = if let Some(dp) = docprefix {
      arena::with(dp, |dp_str| s!("Using '{dp_str}' instead"))
    } else {
      String::from("No prefix to fall back on.")
    };
    Warn!(
      "malformed",
      namespace,
      "No prefix has been registered for namespace (in document)",
      message2
    );
  }
  let default_sym = pin!("#default");
  docprefix.filter(|p| p != &default_sym)
}

pub fn get_document_namespace(docprefix: &str, probe: bool) -> Option<String> {
  let h_default_sym = pin!("#default");
  let docprefix_sym = if docprefix.is_empty() {
    h_default_sym
  } else {
    arena::pin(docprefix)
  };
  let ns_str = match model!().document_namespaces.get_sym(docprefix_sym) {
    None => String::new(),
    Some(sym) => arena::with(*sym, |s| {
      if s.starts_with("DEFAULT#") {
        s.replacen("DEFAULT#", "", 1)
      } else {
        s.to_string()
      }
    }),
  };

  if docprefix_sym != h_default_sym && ns_str.is_empty() && !probe {
    {
      model_mut!().namespace_errors += 1;
    }
    let ns_error = s!(
      "http://example.com/namespace{}",
      &model!().namespace_errors.to_string()
    );
    {
      model_mut!().register_document_namespace(docprefix, Some(&ns_error));
    }
    let msg1 = arena::with(docprefix_sym, |dp_str| {
      s!("No namespace has been registered for prefix '{dp_str}' (in document)")
    });
    let msg2 = s!("Using '{ns_str}' instead");
    let err = || {
      Error!("malformed", docprefix, msg1, msg2);
      Ok(())
    };
    err().ok();
  }
  if ns_str.is_empty() {
    None
  } else {
    Some(ns_str)
  }
}

/// Get the (code) prefix associated with $namespace,
/// creating a dummy prefix and signalling an error if none has been registered.
///
/// In the following:
/// $forattribute is 1 if the namespace is for an attribute (in which case, there must be a
/// non-empty prefix) $probe, if non 0, just test for namespace, without creating an entry
/// if missing.
pub fn get_namespace_prefix(namespace: &str, _forattribute: bool, probe: bool) -> Option<SymStr> {
  let mut codeprefix: Option<SymStr> = None;
  let ns_sym = arena::pin(namespace);
  let mut model = model_mut!();
  if !namespace.is_empty() {
    codeprefix = model.code_namespace_prefixes.get_sym(ns_sym).copied();

    if codeprefix.is_some() && !probe {
      {
        let docprefix = model.document_namespace_prefixes.get_sym(ns_sym);
        // if there's a doc prefix and it's NOT already used in code namespace mapping
        if docprefix.is_some() && !model.code_namespaces.contains_key_sym(docprefix.unwrap()) {
          codeprefix = docprefix.copied();
        }
      }
    } else {
      // Else synthesize one
      model.namespace_errors += 1;
      let auto_prefix = arena::pin(s!("namespace{}", &model.namespace_errors.to_string()));
      codeprefix = Some(auto_prefix);
    }
    model.register_namespace_sym(codeprefix.unwrap(), Some(arena::pin(namespace)));
    // Warn!('malformed', $namespace, undef,
    //   "No prefix has been registered for namespace '$namespace' (in code)",
    //   "Using '$codeprefix' instead"); }
  }

  codeprefix
}

pub fn get_namespace(codeprefix: &str, probe: bool) -> Result<Option<SymStr>> {
  let mut model = model_mut!();
  let mut ns: Option<SymStr> = model.code_namespaces.get(codeprefix).copied();
  if ns.is_none() && !probe {
    model.namespace_errors += 1;
    let example_namespace = s!(
      "http://example.com/namespace{}",
      &model.namespace_errors.to_string()
    );
    ns = Some(arena::pin(&example_namespace));
    model.register_namespace(codeprefix, Some(&example_namespace));
    Error!(
      "malformed",
      codeprefix,
      s!("No namespace has been registered for prefix '{codeprefix}' (in code)"),
      s!("Using '{example_namespace}' instead")
    );
  }
  Ok(ns)
}

/// Get the node's qualified name in standard form
/// Ie. using the registered (code) prefix for that namespace.
/// NOTE: Reconsider how _Capture_ & _WildCard_ should be integrated!?!
pub fn get_node_qname(node: &Node) -> SymStr {
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
      let name_str = node.get_name();
      // Use the actual namespace prefix from the node when available.
      // For elements in the default (ltx) namespace, the prefix is empty
      // so we prepend "ltx:". For SVG/MathML/etc with explicit prefix,
      // use that prefix directly.
      if let Some(ns) = node.get_namespace() {
        let prefix = ns.get_prefix();
        if prefix.is_empty() {
          // Default namespace — use ltx: prefix
          arena::pin(s!("ltx:{}", name_str))
        } else {
          // Explicit prefix (e.g., "svg", "m") — use it
          arena::pin(s!("{}:{}", prefix, name_str))
        }
      } else {
        // No namespace — special cases for non-namespaced elements
        match name_str.as_str() {
          "song" | "verse" => arena::pin(name_str),
          regular => arena::pin(s!("ltx:{}", regular)),
        }
      }
    },
    // Need others?
    t => {
      panic!("Fatal:misdefined:<caller> should not ask for qualified name for node of type {t:?}")
    },
  }
}

pub fn with_node_qname<R, FnR>(node: &Node, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  let qsym = get_node_qname(node);
  arena::with(qsym, |qname_str| caller(qname_str))
}

/// Same as get_node_qname, but using the Document namespace prefixes
pub fn get_node_document_qname(node: &Node) -> SymStr {
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
      let empty_sym = pin!("");
      let mut prefix = empty_sym;
      if let Some(ns) = node.get_namespace() {
        let href = ns.get_href();
        if !href.is_empty() {
          prefix = get_document_namespace_prefix(&href, false, true).unwrap_or(empty_sym);
        }
      }
      if prefix == empty_sym {
        arena::pin(node.get_name())
      } else {
        arena::pin(arena::with(prefix, |prefix_str| {
          s!("{}:{}", prefix_str, node.get_name())
        }))
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
pub fn decode_qname(codetag: &str) -> Result<(Option<String>, String)> {
  match PREFIXED_LOCALNAME_RE.captures(codetag) {
    Some(captures) => {
      let prefix = captures.get(1).map_or("", |m| m.as_str());
      let localname = captures.get(2).map_or("", |m| m.as_str());

      if prefix == "xml" {
        Ok((None, codetag.to_string()))
      } else {
        Ok((
          get_namespace(prefix, false)?.map(arena::to_string),
          localname.to_string(),
        ))
      }
    },
    None => Ok((None, codetag.to_string())),
  }
}

/// TODO: We need a proper data model to deal with the Symbol - String distinction.
/// For now let's allocate strings and release the arena, but this is a CODE SMELL!
pub fn decode_qname_sym(sym: SymStr) -> Result<(Option<String>, String)> {
  let codetag = arena::to_string(sym);
  decode_qname(&codetag)
}

//**********************************************************************
// Document Structure Queries
//**********************************************************************
// NOTE: These are public, but perhaps should be passed
// to submodel, in case it can evolve to more precision?
// However, it would need more context to do that.

/// A check for allowed direct element containment, using ticket-based `SymStr` names.
///
/// TODO: This is a major code smell, experimental prototyping to see how to interoperate
/// strings with the inerned arena.
/// `can_contain` and `can_contain_sym` should be implemented once, and one should be an
/// interning-only helper.
pub fn can_contain_sym(tag: SymStr, child: SymStr) -> bool {
  // Handle obvious cases explicitly.
  if tag == pin!("#PCDATA") || tag == pin!("#Comment") || tag == pin!("") {
    return false;
  } else if tag == pin!("_WildCard_") {
    return true;
  };
  if arena::with(tag, |tag_str| tag_str.ends_with("_Capture_"))
    || arena::with(child, |child_str| {
      child_str.ends_with("_Capture_") || child_str.ends_with("_CaptureBlock_")
    })
  {
    // with or without namespace prefix
    return true;
  }

  if child == pin!("_WildCard_")
    || child == pin!("#Comment")
    || child == pin!("#ProcessingInstruction")
    || child == pin!("#DTD")
  {
    return true;
  }

  let mut model = model_mut!();
  if model.permissive && tag == pin!("#Document") && child != pin!("#PCDATA") {
    return true; // No schema? Punt!
  }

  // Else query tag properties.
  let model_entry = &mut model.tagprop.entry_sym(tag).or_default().model;
  model_entry.contains(&pin!("ANY")) || model_entry.contains(&child)
}

/// Can an element with (qualified name) `tag` contain a `child` element?
pub fn can_contain(tag: &str, child: &str) -> bool {
  // Handle obvious cases explicitly.
  match tag {
    "#PCDATA" | "#Comment" | "" => return false,
    "_WildCard_" => return true,
    _ => {},
  };
  if tag.ends_with("_Capture_") || child.ends_with("_Capture_") || tag.ends_with("_CaptureBlock_") {
    // with or without namespace prefix
    return true;
  }

  match child {
    "_WildCard_" | "#Comment" | "#ProcessingInstruction" | "#DTD" => return true,
    _ => {},
  };
  let model = model!();
  if model.permissive && tag == "#Document" && child != "#PCDATA" {
    return true; // No schema? Punt!
  }

  // Else query tag properties.
  let model = &model.tagprop.get(tag).unwrap_or(&*DEFAULT_TAG_FRAME).model;
  model.contains(&pin!("ANY")) || model.contains(&arena::pin(child))
}

pub fn can_have_attribute(tag: SymStr, attrib: SymStr) -> bool {
  // Handle obvious cases explicitly.
  if let Some(early_choice) = arena::with(tag, |tag_str| match tag_str {
    "#PCDATA" | "#Comment" | "#Document" | "#ProcessingInstruction" | "#DTD" => Some(false),
    "_WildCard_" => Some(true),
    other if other.ends_with("_Capture_") => Some(true),
    _ => None,
  }) {
    return early_choice;
  };
  let model = model!();
  if model.permissive {
    return true;
  }

  // Else query tag properties.
  let attributes = &model
    .tagprop
    .get_sym(tag)
    .unwrap_or(&*DEFAULT_TAG_FRAME)
    .attributes;
  attributes.contains(&attrib)
}

pub fn is_node_in_schema_class(class_name: &str, tag: &Node) -> bool {
  let tag = get_node_qname(tag);
  is_in_schema_class(arena::pin(class_name), tag)
}
pub fn is_in_schema_class(class_name: SymStr, tag: SymStr) -> bool {
  if let Some(class) = model!().schema_class.get_sym(class_name) {
    class.contains(&tag)
  } else {
    false
  }
}

//**********************************************************************
// Accessors
//**********************************************************************

pub fn get_tags() -> Vec<SymStr> { model!().tagprop.keys().copied().collect() }

pub fn get_tag_contents(tag: SymStr) -> Vec<SymStr> {
  match model!().tagprop.get_sym(tag) {
    Some(h) => h.model.iter().copied().collect(),
    None => Vec::new(),
  }
}
pub fn set_model(new_model: Model) {
  let mut model = model_mut!();
  *model = new_model;
}
pub fn is_permissive() -> bool { model!().permissive }

pub fn with_schema_data<FnR, R>(caller: FnR) -> R
where FnR: FnOnce(Option<&Vec<SymStr>>) -> R {
  caller(model!().schema_data.as_ref())
}
pub fn set_schema(schema: Relaxng) {
  let mut model = model_mut!();
  model.schema = Some(schema);
}
pub fn set_schema_class(classname: &str, content: HashSet<SymStr>) {
  model_mut!().set_schema_class(classname, content)
}
pub fn add_tag_content(tag: &str, elements: Vec<&str>) {
  model_mut!().add_tag_content(tag, elements)
}
pub fn add_tag_attribute(tag: &str, attributes: Vec<&str>) {
  model_mut!().add_tag_attribute(tag, attributes)
}

pub(crate) fn compute_indirect_model_aux(
  tag: SymStr,
  start_opt: Option<SymStr>,
  desirability: usize,
  openability: &mut SymHashMap<u32>,
  desc: &mut SymHashMap<SymHashMap<usize>>,
) {
  let start = match start_opt {
    Some(s) => s,
    None => pin!(""),
  };

  // A bit tricky here, we need to release the model_mut!() borrow immediately, which is why we
  // move ownership of the tag strings into the tag_contents vector.
  // That leads to a bunch of .clone()s later one, but stays close to the original algorithm
  let tag_contents: Vec<SymStr> = get_tag_contents(tag);

  for kid in tag_contents {
    // Memoise on (kid, start) to bound recursion in cyclic schemas, but
    // retain the *maximum* desirability observed across paths — the
    // outer loop in compute_indirect_model picks the highest-scoring
    // starting tag, so the score stored here must reflect the best path,
    // not the first one the hashmap iteration happened to surface.
    //
    // The prior "first visit wins" behavior (WISDOM #49) caused paralists
    // test-harness runs to assign `desc[#PCDATA][ltx:text] = 50` when
    // `contents(text)` iterated `ltx:picture` before `#PCDATA`: the
    // sub-recursion `text → picture → #PCDATA` inserted 50 first and the
    // direct `text → #PCDATA` path was skipped, forcing the auto-open
    // path to pick `<ltx:picture>` instead of `<ltx:text>`.
    let prior = desc.entry_sym(kid).or_default().get_sym(start).copied();
    if let Some(prior_d) = prior {
      if prior_d >= desirability {
        continue;
      }
    }

    if start != pin!("") {
      desc
        .entry_sym(kid)
        .or_default()
        .insert_sym(start, desirability);
    }

    if kid != pin!("#PCDATA") {
      if let Some(priority) = openability.get_sym(kid).copied() {
        let inner = if start != pin!("") { start } else { kid };
        // Perl Document.pm L220: `$desirability * $x`. We keep integer
        // arithmetic (priorities scaled by 100), so this is a scaled multiply.
        let next_desirability = desirability * (priority as usize) / 100;
        compute_indirect_model_aux(kid, Some(inner), next_desirability, openability, desc);
      }
    }
  }
}
pub fn register_document_namespace(docprefix: &str, namespace_opt: Option<&str>) {
  model_mut!().register_document_namespace(docprefix, namespace_opt)
}

/// Returns all registered document namespace prefixes and their URIs.
pub fn get_document_namespace_prefixes() -> Vec<(String, String)> {
  model!()
    .document_namespace_prefixes
    .iter()
    .map(|(ns_sym, prefix_sym)| {
      let prefix = arena::with(*prefix_sym, |s| s.to_string());
      let ns = arena::with(*ns_sym, |s| s.to_string());
      (prefix, ns)
    })
    .collect()
}

pub fn register_namespace(codeprefix: &str, namespace_opt: Option<&str>) {
  model_mut!().register_namespace(codeprefix, namespace_opt)
}

pub fn with_code_namespaces<FnR, R>(caller: FnR) -> R
where FnR: FnOnce(&SymHashMap<SymStr>) -> R {
  caller(&model!().code_namespaces)
}
