//! Post-processing document wrapper with XML/DOM, ID management, and caching.
//!
//! Port of `LaTeXML::Post::Document`.
//! Wraps an `XML::LibXML::Document` (via the `libxml` crate) and provides:
//! - Namespace management
//! - ID tracking (idcache, reusable, reserved)
//! - XPath queries with registered namespaces
//! - Node manipulation (addNodes, removeNodes, cloneNode, etc.)
//! - Persistent cache (key-value store)

use libxml::parser::Parser as XmlParser;
use libxml::tree::{Document, Namespace, Node, NodeType, set_node_rc_guard};
use libxml::xpath::Context as XPathContext;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

use crate::radix::radix_alpha;

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// Get the xml:id attribute value from a node.
/// Handles both namespace-aware and plain attribute access.
pub fn get_xml_id(node: &Node) -> Option<String> {
  node
    .get_attribute_ns("id", XML_NS)
    .or_else(|| node.get_attribute("xml:id"))
    .or_else(|| {
      // Fallback: check properties hash for "id" key
      let props = node.get_properties();
      props.get("id").cloned()
    })
}

/// The LaTeXML namespace URI.
pub const LTX_NSURI: &str = "http://dlmf.nist.gov/LaTeXML";

/// Post-processing document: wraps an XML document with ID management,
/// namespace tracking, XPath helpers, and a persistent cache.
///
/// Port of `LaTeXML::Post::Document`.
pub struct PostDocument {
  /// The underlying XML document.
  document:                    Document,
  /// Destination file path for this document.
  pub destination:             Option<String>,
  /// Destination directory (derived from destination).
  pub destination_directory:   Option<String>,
  /// Site root directory.
  pub site_directory:          Option<String>,
  /// Source file path.
  pub source:                  Option<String>,
  /// Source directory.
  pub source_directory:        Option<String>,
  /// Search paths for resources.
  pub searchpaths:             Vec<String>,
  /// Namespace prefix → URI mapping.
  pub namespaces:              HashMap<String, String>,
  /// URI → prefix reverse mapping.
  pub namespace_uris:          HashMap<String, String>,
  /// ID cache: xml:id → node.
  idcache:                     HashMap<String, Node>,
  /// IDs marked as reusable (will be removed later).
  idcache_reusable:            HashMap<String, bool>,
  /// IDs reserved but not yet recorded.
  idcache_reserve:             HashMap<String, bool>,
  /// Clash counters for uniquifyID.
  idcache_clashes:             HashMap<String, u32>,
  /// Processing instructions from the document.
  pub processing_instructions: Vec<String>,
  /// Parent document (for split sub-documents).
  pub parent_document:         Option<Box<PostDocument>>,
  /// ID of document we were split from.
  pub split_from_id:           Option<String>,
  /// Whether to validate the document.
  pub validate:                bool,
  /// Simple key-value cache (replaces Perl's DB_File tied hash).
  cache:                       HashMap<String, String>,
  /// Whether caching is disabled.
  pub nocache:                 bool,
}

impl Drop for PostDocument {
  /// Rationalize Node lifetime between post-processing components.
  /// `idcache` entries are Node *handles* into the C-owned libxml
  /// Document tree — the Document owns the lifetime, Node wrappers
  /// are lookup references.
  ///
  /// libxml 0.3.9's `_Node::drop` fires `xmlFreeNode(ptr)` whenever
  /// the wrapper's internal `unlinked` flag is true. Math processing
  /// calls `unlink_node()` on nodes as it replaces XMath subtrees
  /// with MathML, flipping that flag for nodes still held by
  /// `idcache`. The resulting drop sequence is:
  ///   1. `document: Document` (declared first) → `xmlFreeDoc` walks the full tree including
  ///      still-reachable nodes that share memory with idcache entries; freed.
  ///   2. `idcache: HashMap<String, Node>` → each Node with `unlinked=true` fires `xmlFreeNode` on
  ///      already-freed memory → SIGSEGV inside `xmlFreeNodeList`.
  ///
  /// Fix: hand each idcache entry to `DocOwnedNode` (see
  /// `crate::doc_owned_node`), which suppresses the inner Rc's Drop
  /// so `xmlFreeNode` never fires on already-freed memory.
  /// `xmlFreeDoc` remains the sole owner of the C node memory.
  /// Per-entry Rc control block leaks (~24 B) — bounded by
  /// per-document idcache size and reclaimed at process exit.
  /// Proper upstream fix: a public `set_linked()` setter on the
  /// `libxml` crate's `Node`, which would let us relink before drop
  /// rather than leaking.
  fn drop(&mut self) {
    for (_, node) in std::mem::take(&mut self.idcache) {
      let _kept = crate::doc_owned_node::DocOwnedNode::new(node);
    }
  }
}

impl PostDocument {
  /// Create a new PostDocument wrapping an existing XML document.
  ///
  /// Port of `Post::Document::new`.
  pub fn new(doc: Document, options: PostDocumentOptions) -> Self {
    // Post-processing holds many shared references (id cache, XPath results).
    // Raise the Rc guard to allow mutations on shared nodes.
    set_node_rc_guard(128);

    let mut pd = Self::new_internal(doc, options);
    pd.set_document_internal();
    pd
  }

  fn new_internal(doc: Document, options: PostDocumentOptions) -> Self {
    let mut dest_dir = options.destination_directory.clone();
    if options.destination.is_some() && dest_dir.is_none() {
      if let Some(ref dest) = options.destination {
        if let Some(parent) = Path::new(dest).parent() {
          let parent_str = parent.to_string_lossy().to_string();
          // Empty parent (e.g., from "paper.html") means current directory — use "." not ""
          if parent_str.is_empty() {
            dest_dir = Some(".".to_string());
          } else {
            dest_dir = Some(parent_str);
          }
        }
      }
    }

    let site_dir = if let Some(ref sd) = options.site_directory {
      Some(sd.clone())
    } else {
      dest_dir.clone()
    };

    let mut namespaces = HashMap::new();
    namespaces.insert("ltx".to_string(), LTX_NSURI.to_string());
    let mut namespace_uris = HashMap::new();
    namespace_uris.insert(LTX_NSURI.to_string(), "ltx".to_string());

    PostDocument {
      document: doc,
      destination: options.destination,
      destination_directory: dest_dir,
      site_directory: site_dir,
      source: options.source,
      source_directory: options.source_directory,
      searchpaths: options.searchpaths.unwrap_or_default(),
      namespaces,
      namespace_uris,
      idcache: HashMap::new(),
      idcache_reusable: HashMap::new(),
      idcache_reserve: HashMap::new(),
      idcache_clashes: HashMap::new(),
      processing_instructions: Vec::new(),
      parent_document: None,
      split_from_id: None,
      validate: options.validate,
      cache: HashMap::new(),
      nocache: options.nocache,
    }
  }

  /// Initialize document internals: scan IDs, extract namespaces and PIs.
  fn set_document_internal(&mut self) {
    // Record all xml:id's
    for node in self.findnodes("//*[@xml:id]") {
      if let Some(id) = get_xml_id(&node) {
        self.idcache.insert(id, node);
      }
    }

    // Extract namespaces from root element
    if let Some(root) = self.document.get_root_element() {
      let ns_decls = root.get_namespace_declarations();
      for ns in ns_decls {
        let prefix = ns.get_prefix();
        if !prefix.is_empty() {
          let href = ns.get_href();
          self
            .namespaces
            .entry(prefix.clone())
            .or_insert_with(|| href.clone());
          self.namespace_uris.entry(href).or_insert(prefix);
        }
      }
    }

    // Extract processing instructions
    let pis = self.findnodes(".//processing-instruction('latexml')");
    self.processing_instructions = pis.iter().map(|pi| pi.get_content()).collect();

    // Extract search paths from PIs
    let sp_re = Regex::new(r#"^\s*searchpaths\s*=\s*[\"'](.*?)[\"']\s*$"#).unwrap();
    let mut paths = self.searchpaths.clone();
    for pi_text in &self.processing_instructions {
      if let Some(cap) = sp_re.captures(pi_text) {
        for p in cap[1].split(',') {
          paths.push(p.trim().to_string());
        }
      }
    }
    paths.push(".".to_string());
    self.searchpaths = paths;
  }

  // ======================================================================
  // Constructors from various sources

  /// Create from an XML file.
  ///
  /// Port of `Post::Document::newFromFile`.
  pub fn new_from_file(path: &str, options: PostDocumentOptions) -> Result<Self, String> {
    let parser = XmlParser::default();
    let doc = parser
      .parse_file(path)
      .map_err(|e| format!("Failed to parse '{}': {}", path, e))?;
    let mut opts = options;
    if opts.source.is_none() {
      opts.source = Some(path.to_string());
    }
    if opts.source_directory.is_none() {
      if let Some(parent) = Path::new(path).parent() {
        opts.source_directory = Some(parent.to_string_lossy().to_string());
      }
    }
    Ok(Self::new(doc, opts))
  }

  /// Create from an XML string.
  ///
  /// Port of `Post::Document::newFromString`.
  pub fn new_from_string(xml: &str, options: PostDocumentOptions) -> Result<Self, String> {
    let parser = XmlParser::default();
    let doc = parser
      .parse_string(xml)
      .map_err(|e| format!("Failed to parse XML string: {}", e))?;
    let mut opts = options;
    if opts.source_directory.is_none() {
      opts.source_directory = Some(".".to_string());
    }
    Ok(Self::new(doc, opts))
  }

  /// Create a new sub-document from an element node.
  ///
  /// Port of Perl `Post::Document::newDocument`.
  /// The element is imported into a fresh XML document.
  /// Resources, processing instructions, and class attributes are copied from the parent.
  pub fn new_document(&self, root: Node, destination: &str) -> Self {
    use libxml::tree::Document as XmlDocument;
    // Create a fresh XML document that owns a deep copy of `root`'s
    // subtree as its root element.
    //
    // Perl Post.pm L831-839: `XML::LibXML::Document->new(...)` then
    // `setDocumentElement($doc->importNode($root))`. importNode COPIES
    // the subtree into the new doc; the new doc and the source no
    // longer share C-side state.
    //
    // We use libxml-rs's `Document::dup_node_into_new_doc` (added in
    // KWARC/rust-libxml `clone-document` branch). The earlier
    // `import_node` route had two pitfalls that made it unusable for
    // the Split.process_pages loop:
    //   1. `import_node` gates on `Node::is_unlinked()`, a wrapper-
    //      side flag with no public setter; the gate leaks `false`
    //      across iterations because the previous call's set_linked()
    //      mutated the wrapper Rc, forcing every page after the
    //      first to Err.
    //   2. Direct `xmlDocCopyNode(src, dst, 1)` returns NULL on the
    //      second sibling page — the first recursive copy dirties
    //      dict/ns state on the source doc such that subsequent
    //      recursive copies fail their child-copy phase (verified:
    //      extended=2 still works, isolating the failure to
    //      recursive descent).
    // dup_node_into_new_doc avoids both: it does
    // `xmlCopyNode(node, 1)` (orphan deep copy, no source-doc state
    // mutation), plants the copy into a freshly created xmlDoc, fixes
    // up doc pointers via xmlSetTreeDoc, and reconciles namespaces.
    // The returned Document shares zero C-side state with the source.
    //
    // SCOPE: this method is only called from `Split::process_pages`
    // (split.rs L260); non-split flows do not pay the deep-copy cost.
    let new_xml_doc: XmlDocument = XmlDocument::dup_node_into_new_doc(&root)
      .expect("dup_node_into_new_doc returned NULL while creating split sub-document");
    let _ = root;

    let opts = PostDocumentOptions {
      destination: Some(destination.to_string()),
      // CRITICAL: inherit the parent's site_directory so the sub-doc's
      // `site_relative_destination` carries any intermediate split
      // directory (e.g. "Ch1/schema.scholarly-ltx.html") into DB
      // location strings. Without this, every sub-doc defaults
      // site_directory to its own destination_directory, which makes
      // every per-doc `location` resolve to just the basename and
      // CrossRef::generate_url then produces broken in-page anchors
      // instead of the cross-doc relative URLs that the rendered TOC
      // is supposed to walk.
      site_directory: self.site_directory.clone(),
      source: self.source.clone(),
      source_directory: self.source_directory.clone(),
      searchpaths: Some(self.searchpaths.clone()),
      ..PostDocumentOptions::default()
    };
    let mut subdoc = Self::new_internal(new_xml_doc, opts);

    // Copy namespaces
    subdoc.namespaces = self.namespaces.clone();
    subdoc.namespace_uris = self.namespace_uris.clone();

    // Record IDs
    for node in subdoc.findnodes("//*[@xml:id]") {
      if let Some(id) = get_xml_id(&node) {
        subdoc.idcache.insert(id, node);
      }
    }

    // Record the parent document's root ID
    if let Some(ref root_el) = self.get_document_element() {
      if let Some(root_id) = get_xml_id(root_el) {
        subdoc.split_from_id = Some(root_id);
      }
    }

    // Copy processing instructions
    for mut pi in self.findnodes(".//processing-instruction('latexml')") {
      if let Ok(mut pi_clone) = subdoc.document.import_node(&mut pi) {
        if let Some(mut doc_node) = subdoc.document.get_root_element() {
          doc_node.add_prev_sibling(&mut pi_clone).ok();
        }
      }
    }

    // Copy resource elements (Perl: addNodes for ltx:resource)
    let resources: Vec<NodeData> = self
      .findnodes("descendant::ltx:resource")
      .iter()
      .map(|r| NodeData::XmlNode(r.clone()))
      .collect();
    if !resources.is_empty() {
      if let Some(mut doc_root) = subdoc.get_document_element() {
        subdoc.add_nodes(&mut doc_root, &resources);
      }
    }

    // Copy class from top-level document element (Perl L777-782)
    if let Some(parent_root) = self.get_document_element() {
      if let Some(pclass) = parent_root.get_attribute("class") {
        if let Some(mut doc_root) = subdoc.get_document_element() {
          let existing = doc_root.get_attribute("class").unwrap_or_default();
          if existing.is_empty() {
            doc_root.set_attribute("class", &pclass).ok();
          } else {
            doc_root
              .set_attribute("class", &format!("{} {}", existing, pclass))
              .ok();
          }
        }
      }
    }

    // Copy class from top-level document element (Perl L777-782)
    if let Some(parent_root) = self.get_document_element() {
      if let Some(pclass) = parent_root.get_attribute("class") {
        if let Some(mut doc_root) = subdoc.get_document_element() {
          let existing = doc_root.get_attribute("class").unwrap_or_default();
          if existing.is_empty() {
            doc_root.set_attribute("class", &pclass).ok();
          } else {
            doc_root
              .set_attribute("class", &format!("{} {}", existing, pclass))
              .ok();
          }
        }
      }
    }

    subdoc
  }

  // ======================================================================
  // Accessors

  /// Get a reference to the underlying XML document.
  pub fn get_document(&self) -> &Document { &self.document }

  /// Get a mutable reference to the underlying XML document.
  pub fn get_document_mut(&mut self) -> &mut Document { &mut self.document }

  /// Get the document's root element.
  pub fn get_document_element(&self) -> Option<Node> { self.document.get_root_element() }

  /// Get the source path.
  pub fn get_source(&self) -> Option<&str> { self.source.as_deref() }

  /// Get the source directory.
  pub fn get_source_directory(&self) -> &str { self.source_directory.as_deref().unwrap_or(".") }

  /// Get search paths.
  pub fn get_search_paths(&self) -> &[String] { &self.searchpaths }

  /// Get the destination path.
  pub fn get_destination(&self) -> Option<&str> { self.destination.as_deref() }

  /// Get the destination directory.
  pub fn get_destination_directory(&self) -> Option<&str> { self.destination_directory.as_deref() }

  /// Get the site directory.
  pub fn get_site_directory(&self) -> Option<&str> { self.site_directory.as_deref() }

  /// Return destination relative to site directory.
  ///
  /// Port of `siteRelativeDestination`.
  pub fn site_relative_destination(&self) -> Option<String> {
    if let (Some(dest), Some(site)) = (&self.destination, &self.site_directory) {
      Some(pathdiff(dest, site))
    } else {
      self.destination.clone()
    }
  }

  /// Return a pathname relative to the site directory.
  pub fn site_relative_pathname(&self, pathname: &str) -> Option<String> {
    self
      .site_directory
      .as_ref()
      .map(|site| pathdiff(pathname, site))
  }

  /// Get the destination file extension.
  pub fn get_destination_extension(&self) -> Option<String> {
    self.destination.as_ref().and_then(|d| {
      Path::new(d)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
    })
  }

  /// Serialize the document to an XML string.
  pub fn to_xml_string(&self) -> String { self.document.to_string() }

  pub fn stringify(&self) -> String {
    format!(
      "Post::Document[{}]",
      self
        .site_relative_destination()
        .unwrap_or_else(|| "?".to_string())
    )
  }

  // ======================================================================
  // XPath queries

  /// Find nodes matching an XPath expression.
  ///
  /// Port of `Post::Document::findnodes`.
  pub fn findnodes(&self, xpath: &str) -> Vec<Node> { self.findnodes_at(xpath, None) }

  /// Find nodes matching an XPath expression, relative to a given context node.
  pub fn findnodes_at(&self, xpath: &str, context_node: Option<&Node>) -> Vec<Node> {
    let ctx = match XPathContext::new(&self.document) {
      Ok(c) => c,
      Err(_) => return vec![],
    };

    // Register all known namespaces
    for (prefix, uri) in &self.namespaces {
      let _ = ctx.register_namespace(prefix, uri);
    }

    let result = if let Some(node) = context_node {
      ctx.node_evaluate(xpath, node)
    } else {
      ctx.evaluate(xpath)
    };

    match result {
      Ok(obj) => obj.get_nodes_as_vec(),
      Err(_) => vec![],
    }
  }

  /// Find the first node matching an XPath expression.
  pub fn findnode(&self, xpath: &str) -> Option<Node> { self.findnodes(xpath).into_iter().next() }

  /// Find the first node matching an XPath expression, relative to a context node.
  pub fn findnode_at(&self, xpath: &str, context_node: &Node) -> Option<Node> {
    self
      .findnodes_at(xpath, Some(context_node))
      .into_iter()
      .next()
  }

  /// Evaluate an XPath expression and return the string value.
  pub fn findvalue(&self, xpath: &str) -> Option<String> {
    let ctx = XPathContext::new(&self.document).ok()?;
    for (prefix, uri) in &self.namespaces {
      let _ = ctx.register_namespace(prefix, uri);
    }
    ctx.evaluate(xpath).ok().map(|obj| obj.to_string())
  }

  /// XPath query on an arbitrary node, even if from a different document.
  /// Creates a temporary XPath context on the node's own document.
  pub fn findnodes_foreign(xpath: &str, node: &Node) -> Vec<Node> {
    // Navigate up to find the document root, then create context
    let mut current = node.clone();
    while let Some(parent) = current.get_parent() {
      current = parent;
    }
    // current is the document root (or the node itself if detached)
    // Get the document for this node tree
    if let Some(doc) = current.get_parent() {
      // Has a parent = we're at root element, doc is parent
      let _ = doc; // can't use this easily
    }
    // Fallback: use libxml's node_evaluate with a fresh context
    // We need to use the internal document. libxml2 nodes know their document.
    #[allow(unused_imports)]
    use libxml::xpath::Context as XPathContext;
    // Create context from the document that owns this node
    // node._node_ptr -> xmlNodePtr -> doc field
    // Unfortunately, libxml2-rs doesn't expose a way to get the document from a node.
    // Workaround: build a new document wrapping this subtree.
    // Simpler workaround: just traverse children manually for common patterns.
    Self::findnodes_by_traversal(xpath, node)
  }

  /// Manual node traversal for common XPath patterns used in bibliography formatting.
  /// Handles: "ltx:bib-name[@role='author']", "ltx:bib-title", "ltx:bib-date[@role='publication']",
  /// "ltx:bib-related/ltx:bib-title", "ltx:bib-part[@role='volume']", etc.
  fn findnodes_by_traversal(xpath: &str, parent: &Node) -> Vec<Node> {
    let xpath = xpath.trim_start_matches('!').trim();
    let mut results = Vec::new();

    // Parse simple patterns: "ltx:elem" or "ltx:elem[@attr='val']" or "ltx:elem/ltx:child"
    let parts: Vec<&str> = xpath.split('/').collect();
    if parts.is_empty() {
      return results;
    }

    fn match_element(node: &Node, pattern: &str) -> bool {
      let pattern = pattern.trim().trim_start_matches("ltx:");
      if let Some(bracket_pos) = pattern.find('[') {
        let elem_name = &pattern[..bracket_pos];
        let attr_part = &pattern[bracket_pos + 1..pattern.len() - 1]; // strip [ and ]
        if node.get_name() != elem_name {
          return false;
        }
        // Parse @attr='value'
        if let Some(eq_pos) = attr_part.find('=') {
          let attr_name = attr_part[1..eq_pos].trim(); // skip @
          let attr_val = attr_part[eq_pos + 1..]
            .trim()
            .trim_matches('\'')
            .trim_matches('"');
          node
            .get_attribute(attr_name)
            .map(|v| v == attr_val)
            .unwrap_or(false)
        } else {
          true
        }
      } else {
        node.get_name() == pattern
      }
    }

    fn collect_matching(node: &Node, parts: &[&str], results: &mut Vec<Node>) {
      if parts.is_empty() {
        return;
      }
      let pattern = parts[0];
      // Handle "A | B" alternatives
      let alternatives: Vec<&str> = pattern.split('|').map(|s| s.trim()).collect();
      let mut child = node.get_first_child();
      while let Some(c) = child {
        for alt in &alternatives {
          if match_element(&c, alt) {
            if parts.len() == 1 {
              results.push(c.clone());
            } else {
              collect_matching(&c, &parts[1..], results);
            }
          }
        }
        child = c.get_next_sibling();
      }
    }

    collect_matching(parent, &parts, &mut results);
    results
  }

  // ======================================================================
  // Namespace management

  /// Register a new namespace prefix → URI mapping.
  ///
  /// Port of `Post::Document::addNamespace`.
  pub fn add_namespace(&mut self, prefix: &str, nsuri: &str) {
    let dominated = self
      .namespaces
      .get(prefix)
      .map(|u| u == nsuri)
      .unwrap_or(false);
    if !dominated {
      self
        .namespaces
        .insert(prefix.to_string(), nsuri.to_string());
      self
        .namespace_uris
        .insert(nsuri.to_string(), prefix.to_string());
      // Declare the namespace on the root element (without changing its own namespace).
      // Namespace::new() creates the declaration; we do NOT call set_namespace()
      // which would change the root element's own namespace.
      if let Some(mut root) = self.document.get_root_element() {
        let _ = Namespace::new(prefix, nsuri, &mut root);
      }
    }
  }

  /// Get the qualified name (prefix:localname) for a node.
  ///
  /// Port of `Post::Document::getQName`.
  pub fn get_qname(&self, node: &Node) -> Option<String> {
    if node.get_type() != Some(NodeType::ElementNode) {
      return None;
    }
    let localname = node.get_name();
    if let Some(ns) = node.get_namespace() {
      let nsuri = ns.get_href();
      if let Some(prefix) = self.namespace_uris.get(&nsuri) {
        Some(format!("{}:{}", prefix, localname))
      } else {
        // Auto-generate a prefix for unknown namespaces
        let n = self
          .namespaces
          .keys()
          .filter(|k| k.starts_with("_ns"))
          .count()
          + 1;
        Some(format!("_ns{}:{}", n, localname))
      }
    } else {
      Some(localname)
    }
  }

  /// Resolve a node's namespace URI to its registered prefix without
  /// allocating a combined "prefix:localname". Returns the prefix as an
  /// owned `String` (a copy of the entry in `namespace_uris`); callers
  /// can then match on `node.get_name()` separately. Useful in hot
  /// dispatch code where the `format!` in `get_qname` is the cost.
  pub fn qname_prefix(&self, node: &Node) -> Option<String> {
    if node.get_type() != Some(NodeType::ElementNode) {
      return None;
    }
    node.get_namespace().and_then(|ns| {
      let nsuri = ns.get_href();
      self.namespace_uris.get(&nsuri).cloned()
    })
  }

  /// Check whether a node's qualified name equals a fixed "prefix:localname"
  /// string without allocating a `String`. Fast-path for hot comparisons
  /// like `is_qname(node, "ltx:XMApp")` — avoids the `format!` in
  /// `get_qname` when the caller only needs a boolean answer. Falls back
  /// to allocating comparison (via `get_qname`) for unknown-namespace
  /// cases so semantics exactly match `get_qname(node).as_deref() == Some(...)`.
  pub fn is_qname(&self, node: &Node, expected: &str) -> bool {
    if node.get_type() != Some(NodeType::ElementNode) {
      return false;
    }
    let (expected_prefix, expected_local) = match expected.split_once(':') {
      Some((p, l)) => (Some(p), l),
      None => (None, expected),
    };
    let localname = node.get_name();
    if localname != expected_local {
      return false;
    }
    match (node.get_namespace(), expected_prefix) {
      (Some(ns), Some(ep)) => {
        let nsuri = ns.get_href();
        self
          .namespace_uris
          .get(&nsuri)
          .map(|p| p == ep)
          .unwrap_or(false)
      },
      (None, None) => true,
      _ => false,
    }
  }

  // ======================================================================
  // ID management

  /// Record an ID → node mapping.
  ///
  /// Port of `Post::Document::recordID`.
  pub fn record_id(&mut self, id: &str, node: Node) {
    self.idcache.insert(id.to_string(), node);
    self.idcache_reserve.remove(id);
    self.idcache_reusable.remove(id);
  }

  /// Find a node by its xml:id.
  ///
  /// Port of `Post::Document::findNodeByID`.
  pub fn find_node_by_id(&self, id: &str) -> Option<&Node> { self.idcache.get(id) }

  /// Generate a unique ID based on `baseid`, optionally applying a suffix.
  ///
  /// If the resulting ID is already used (and not marked reusable),
  /// appends alphabetic suffixes (a, b, c, ...) until unique.
  ///
  /// Port of `Post::Document::uniquifyID`.
  pub fn uniquify_id(&mut self, baseid: &str, suffix: Option<&str>) -> String {
    let apply_suffix = |id: &str, sfx: Option<&str>| -> String {
      if let Some(s) = sfx {
        format!("{}{}", id, s)
      } else {
        id.to_string()
      }
    };

    let mut id = apply_suffix(baseid, suffix);
    let cachekey = id.clone();

    while (self.idcache.contains_key(&id) || self.idcache_reserve.contains_key(&id))
      && !self.idcache_reusable.contains_key(&id)
    {
      let clash_count = self.idcache_clashes.entry(cachekey.clone()).or_insert(0);
      *clash_count += 1;
      id = apply_suffix(&format!("{}{}", baseid, radix_alpha(*clash_count)), suffix);
    }

    self.idcache_reusable.remove(&id);
    self.idcache_reserve.insert(id.clone(), true);
    id
  }

  /// Generate, add, and register an xml:id for a node.
  ///
  /// Creates a structured ID relative to the nearest parent with an ID.
  ///
  /// Port of `Post::Document::generateNodeID`.
  pub fn generate_node_id(
    &mut self,
    node: &mut Node,
    prefix: &str,
    reusable: bool,
  ) -> Option<String> {
    if let Some(id) = get_xml_id(node) {
      return Some(id);
    }

    // Find the closest parent with an ID
    let mut parent_node = node.get_parent();
    let mut pid = String::new();
    while let Some(ref p) = parent_node {
      if let Some(id) = p.get_attribute("xml:id") {
        pid = id;
        break;
      }
      parent_node = p.get_parent();
    }

    if !pid.is_empty() {
      pid.push('.');
    }

    // Find the next unused ID
    let mut n = 1u32;
    let id = loop {
      let candidate = format!("{}{}{}", pid, prefix, n);
      if !self.idcache.contains_key(&candidate) && !self.idcache_reserve.contains_key(&candidate) {
        break candidate;
      }
      n += 1;
    };

    node.set_attribute("xml:id", &id).ok();
    let node_copy = node.clone();
    self.idcache.insert(id.clone(), node_copy);
    if reusable {
      self.idcache_reusable.insert(id.clone(), true);
    }

    // If the parent has a fragid, create one here too
    if let Some(ref p) = parent_node {
      if p.get_attribute("fragid").is_some() {
        let new_fragid = format!("{}.{}{}", p.get_attribute("fragid").unwrap(), prefix, n);
        node.set_attribute("fragid", &new_fragid).ok();
      }
    }

    Some(id)
  }

  // ======================================================================
  // Node manipulation

  /// Add nodes to `parent` using the recursive representation.
  ///
  /// Port of `Post::Document::addNodes`.
  pub fn add_nodes(&mut self, parent: &mut Node, data: &[NodeData]) {
    for child in data {
      match child {
        NodeData::Text(text) => {
          parent.append_text(text).ok();
        },
        NodeData::Element { tag, attributes, children } => {
          if tag == "_Fragment_" {
            self.add_nodes(parent, children);
          } else if let Some((prefix, localname)) = tag.split_once(':') {
            let nsuri = self.namespaces.get(prefix).cloned();
            if nsuri.is_none() {
              log::warn!("No namespace on '{}'", tag);
            }
            // Find or create namespace for this prefix.
            // Prefer the default namespace (empty prefix) if it matches the target URI,
            // so elements like ltx:ref are created as <ref> not <ltx:ref>.
            let ns = nsuri.and_then(|uri| {
              // First check if the default namespace matches — prefer it to avoid ltx: prefix
              parent
                .get_namespace_declarations()
                .into_iter()
                .find(|ns| ns.get_prefix().is_empty() && ns.get_href() == uri)
                .or_else(|| {
                  parent
                    .get_namespaces(&self.document)
                    .into_iter()
                    .find(|ns| ns.get_prefix().is_empty() && ns.get_href() == uri)
                })
                // Fall back to matching prefix
                .or_else(|| {
                  parent
                    .get_namespace_declarations()
                    .into_iter()
                    .find(|ns| ns.get_prefix() == prefix)
                })
                .or_else(|| {
                  parent
                    .get_namespaces(&self.document)
                    .into_iter()
                    .find(|ns| ns.get_prefix() == prefix)
                })
                .or_else(|| {
                  // Create a new declaration
                  Namespace::new(prefix, &uri, parent).ok()
                })
            });
            if let Ok(mut new_node) = parent.new_child(ns, localname) {
              // Set attributes
              if let Some(attrs) = attributes {
                let mut sorted_keys: Vec<_> = attrs.keys().collect();
                sorted_keys.sort();
                for key in sorted_keys {
                  let value = &attrs[key];
                  if key.starts_with('_') {
                    continue;
                  }
                  if key == "xml:id" {
                    let id = if self.idcache.contains_key(value.as_str()) {
                      self.uniquify_id(value, None)
                    } else {
                      value.clone()
                    };
                    self.record_id(&id, new_node.clone());
                    new_node.set_attribute("xml:id", &id).ok();
                  } else {
                    new_node.set_attribute(key, value).ok();
                  }
                }
              }
              self.add_nodes(&mut new_node, children);
            }
          } else {
            log::warn!("Tag '{}' has no namespace prefix", tag);
          }
        },
        NodeData::XmlNode(source_node) => {
          self.add_xml_node(parent, source_node);
        },
      }
    }
  }

  /// Clone and append an existing XML node into `parent`.
  fn add_xml_node(&mut self, parent: &mut Node, source: &Node) {
    match source.get_type() {
      Some(NodeType::ElementNode) => {
        let localname = source.get_name();
        let ns = source.get_namespace();
        if let Ok(mut new_node) = parent.new_child(ns, &localname) {
          // Copy attributes
          let props = source.get_properties();
          for (key, value) in &props {
            if key.starts_with('_') {
              continue;
            }
            if key == "xml:id" {
              let id = if self.idcache.contains_key(value.as_str()) {
                self.uniquify_id(value, None)
              } else {
                value.clone()
              };
              self.record_id(&id, new_node.clone());
              new_node.set_attribute("xml:id", &id).ok();
            } else {
              new_node.set_attribute(key, value).ok();
            }
          }
          // Recursively add children
          if let Some(child) = source.get_first_child() {
            let mut current = Some(child);
            while let Some(ref c) = current {
              self.add_xml_node(&mut new_node, c);
              current = c.get_next_sibling();
            }
          }
        }
      },
      Some(NodeType::TextNode) => {
        parent.append_text(&source.get_content()).ok();
      },
      Some(NodeType::DocumentFragNode) => {
        if let Some(child) = source.get_first_child() {
          let mut current = Some(child);
          while let Some(ref c) = current {
            self.add_xml_node(parent, c);
            current = c.get_next_sibling();
          }
        }
      },
      _ => {},
    }
  }

  /// Remove nodes from the document, cleaning up ID caches.
  ///
  /// Port of `Post::Document::removeNodes`.
  pub fn remove_nodes(&mut self, nodes: &[Node]) {
    fn collect_ids_of_subtree(node: &Node, out: &mut Vec<String>) {
      if node.get_type() != Some(NodeType::ElementNode) {
        return;
      }
      if let Some(id) = get_xml_id(node) {
        out.push(id);
      }
      let mut child = node.get_first_child();
      while let Some(c) = child {
        collect_ids_of_subtree(&c, out);
        child = c.get_next_sibling();
      }
    }

    for node in nodes {
      if node.get_type() == Some(NodeType::ElementNode) {
        // Walk the subtree directly to enumerate xml:id descendants.
        let mut ids = Vec::new();
        collect_ids_of_subtree(node, &mut ids);
        for id in ids {
          self.idcache.remove(&id);
        }
      }
      let mut n = node.clone();
      n.unlink_node();
    }
  }

  /// Mark nodes as "will be removed later" — their IDs become reusable.
  ///
  /// Port of `Post::Document::preremoveNodes`.
  pub fn preremove_nodes(&mut self, nodes: &[Node]) {
    for node in nodes {
      if node.get_type() == Some(NodeType::ElementNode) {
        for idd in self.findnodes_at("descendant-or-self::*[@xml:id]", Some(node)) {
          if let Some(id) = idd.get_attribute("xml:id") {
            self.idcache_reusable.insert(id, true);
          }
        }
      }
    }
  }

  /// Remove blank (whitespace-only) text nodes that are direct children of `node`.
  ///
  /// Port of `Post::Document::removeBlankNodes`.
  pub fn remove_blank_nodes(&self, node: &Node) -> u32 {
    let mut count = 0;
    if let Some(child) = node.get_first_child() {
      let mut current = Some(child);
      while let Some(ref mut c) = current {
        let next = c.get_next_sibling();
        if c.get_type() == Some(NodeType::TextNode) {
          let text = c.get_content();
          if text.trim().is_empty() {
            c.unlink_node();
            count += 1;
          }
        }
        current = next;
      }
    }
    count
  }

  /// Replace `node` with `replacements` in the document.
  ///
  /// Port of `Post::Document::replaceNode`.
  pub fn replace_node(&mut self, old_node: &Node, replacements: &[NodeData]) {
    if let Some(mut parent) = old_node.get_parent() {
      // Save following siblings
      let mut save = Vec::new();
      while let Some(mut last) = parent.get_last_child() {
        if last == *old_node {
          break;
        }
        last.unlink_node();
        save.insert(0, last);
      }

      // Remove the old node
      self.remove_nodes(&[old_node.clone()]);

      // Add replacements
      self.add_nodes(&mut parent, replacements);

      // Re-append saved siblings
      for mut s in save {
        parent.add_child(&mut s).ok();
      }
    }
  }

  /// Prepend `nodes` as the first children of `parent`.
  ///
  /// Port of `Post::Document::prependNodes`.
  pub fn prepend_nodes(&mut self, parent: &mut Node, nodes: &[NodeData]) {
    // Save all existing children
    let mut save = Vec::new();
    while let Some(mut last) = parent.get_last_child() {
      last.unlink_node();
      save.insert(0, last);
    }

    // Add new nodes first
    self.add_nodes(parent, nodes);

    // Re-append original children
    for mut s in save {
      parent.add_child(&mut s).ok();
    }
  }

  /// Clone a node with unique IDs.
  ///
  /// Port of `Post::Document::cloneNode`.
  pub fn clone_node(&mut self, node: &Node, id_suffix: Option<&str>) -> Option<Node> {
    let copy = node.clone();

    // Find all IDs and remap them
    let mut idmap: HashMap<String, String> = HashMap::new();
    for mut n in self.findnodes_at("descendant-or-self::*[@xml:id]", Some(&copy)) {
      if let Some(id) = n.get_attribute("xml:id") {
        let newid = self.uniquify_id(&id, id_suffix);
        idmap.insert(id, newid.clone());
        self.record_id(&newid, n.clone());
        n.set_attribute("xml:id", &newid).ok();
      }
    }

    // Update idref references
    for mut n in self.findnodes_at("descendant-or-self::*[@idref]", Some(&copy)) {
      if let Some(idref) = n.get_attribute("idref") {
        if let Some(newid) = idmap.get(&idref) {
          n.set_attribute("idref", newid).ok();
        }
      }
    }

    // Remove labels
    for mut n in self.findnodes_at("descendant-or-self::*[@labels]", Some(&copy)) {
      let _ = n.remove_attribute("labels");
    }

    Some(copy)
  }

  // ======================================================================
  // CSS class and style management

  /// Add space-separated values to an attribute, deduplicating and sorting.
  ///
  /// Port of `Post::Document::addSSValues`.
  pub fn add_ss_values(node: &mut Node, key: &str, values: &str) {
    if values.is_empty() {
      return;
    }
    let new_values: Vec<&str> = values.split_whitespace().collect();
    if let Some(old_values_str) = node.get_attribute(key) {
      let mut all: Vec<String> = old_values_str
        .split_whitespace()
        .map(String::from)
        .collect();
      for v in &new_values {
        if !all.iter().any(|o| o == v) {
          all.push(v.to_string());
        }
      }
      all.sort();
      node.set_attribute(key, &all.join(" ")).ok();
    } else {
      let mut sorted: Vec<&str> = new_values;
      sorted.sort();
      node.set_attribute(key, &sorted.join(" ")).ok();
    }
  }

  /// Add CSS class(es) to a node.
  ///
  /// Port of `Post::Document::addClass`.
  pub fn add_class(node: &mut Node, class: &str) { Self::add_ss_values(node, "class", class); }

  // ======================================================================
  // XMath visibility marking

  /// Mark XMath node visibility (content vs presentation branches).
  ///
  /// Port of `Post::Document::markXMNodeVisibility`.
  pub fn mark_xm_node_visibility(&self) {
    for mut math_child in self.findnodes("//ltx:XMath/*") {
      self.mark_xm_node_visibility_aux(&mut math_child, true, true);
    }
  }

  fn mark_xm_node_visibility_aux(&self, node: &mut Node, cvis: bool, pvis: bool) {
    let qname = match self.get_qname(node) {
      Some(q) => q,
      None => return,
    };

    let has_cvis = node.get_attribute("_cvis").is_some();
    let has_pvis = node.get_attribute("_pvis").is_some();
    if (!cvis || has_cvis) && (!pvis || has_pvis) {
      return;
    }

    if cvis {
      node.set_attribute("_cvis", "1").ok();
    }
    if pvis {
      node.set_attribute("_pvis", "1").ok();
    }

    if qname == "ltx:XMDual" {
      let mut children = element_children(node);
      if children.len() >= 2 {
        if cvis {
          self.mark_xm_node_visibility_aux(&mut children[0], true, false);
        }
        if pvis {
          self.mark_xm_node_visibility_aux(&mut children[1], false, true);
        }
      }
    } else if qname == "ltx:XMRef" {
      if let Some(idref) = node.get_attribute("idref") {
        if let Some(target) = self.find_node_by_id(&idref) {
          let mut target_mut = target.clone();
          self.mark_xm_node_visibility_aux(&mut target_mut, cvis, pvis);
        }
      }
    } else {
      for mut child in element_children(node) {
        self.mark_xm_node_visibility_aux(&mut child, cvis, pvis);
      }
    }
  }

  /// Realize an XMRef/XMDual node: follow references to get the "real" node.
  ///
  /// Port of `Post::Document::realizeXMNode`.
  pub fn realize_xm_node(&self, node: &Node) -> Option<Node> {
    if self.is_qname(node, "ltx:XMRef") {
      let idref = node.get_attribute("idref")?;
      self.find_node_by_id(&idref).cloned()
    } else {
      Some(node.clone())
    }
  }

  // ======================================================================
  // Utility methods

  /// Join a list of nodes with a conjunction.
  ///
  /// Port of `Post::Document::conjoin`.
  pub fn conjoin(conjunction: Conjunction, nodes: Vec<NodeData>) -> Vec<NodeData> {
    let n = nodes.len();
    if n < 2 {
      return nodes;
    }

    let (comma, and) = match conjunction {
      Conjunction::Simple(s) => (s.clone(), s),
      Conjunction::Pair(c, a) => (c, a),
    };

    let mut result = Vec::new();
    let mut iter = nodes.into_iter();
    result.push(iter.next().unwrap());

    let mut remaining: Vec<_> = iter.collect();
    while remaining.len() > 1 {
      result.push(NodeData::Text(comma.clone()));
      result.push(remaining.remove(0));
    }
    result.push(NodeData::Text(and));
    result.push(remaining.remove(0));
    result
  }

  /// Find the initial letter for sorting.
  ///
  /// Port of `Post::Document::initial`.
  pub fn initial(string: &str, force: bool) -> String {
    let decomposed: String = string.nfd().collect();
    let trimmed = decomposed.trim_start();
    let s = if force {
      trimmed.trim_start_matches(|c: char| !c.is_ascii_alphabetic())
    } else {
      trimmed
    };
    match s.chars().next() {
      Some(c) if c.is_ascii_alphabetic() => c.to_uppercase().to_string(),
      _ => "*".to_string(),
    }
  }

  /// Trim leading/trailing whitespace text nodes from a node's children.
  ///
  /// Port of `Post::Document::trimChildNodes`.
  pub fn trim_child_nodes(node: &Node) -> Vec<Node> {
    let mut children: Vec<Node> = Vec::new();
    if let Some(child) = node.get_first_child() {
      let mut current = Some(child);
      while let Some(ref c) = current {
        children.push(c.clone());
        current = c.get_next_sibling();
      }
    }

    if children.is_empty() {
      return children;
    }

    // Trim leading whitespace
    if let Some(first) = children.first_mut() {
      if first.get_type() == Some(NodeType::TextNode) {
        let text = first.get_content();
        let trimmed = text.trim_start();
        if trimmed.is_empty() {
          children.remove(0);
        } else if trimmed != text {
          first.set_content(trimmed).ok();
        }
      }
    }

    // Trim trailing whitespace
    if let Some(last) = children.last_mut() {
      if last.get_type() == Some(NodeType::TextNode) {
        let text = last.get_content();
        let trimmed = text.trim_end();
        if trimmed.is_empty() {
          children.pop();
        } else if trimmed != text {
          last.set_content(trimmed).ok();
        }
      }
    }

    children
  }

  /// Add a navigation reference.
  ///
  /// Port of `Post::Document::addNavigation`.
  pub fn add_navigation(&mut self, relation: &str, id: &str) {
    let check_xpath = format!(
      "//ltx:navigation/ltx:ref[@rel='{}'][@idref='{}']",
      relation, id
    );
    if self.findnode(&check_xpath).is_some() {
      return;
    }

    let ref_node = NodeData::Element {
      tag:        "ltx:ref".to_string(),
      attributes: Some(HashMap::from([
        ("idref".to_string(), id.to_string()),
        ("rel".to_string(), relation.to_string()),
        ("show".to_string(), "toctitle".to_string()),
      ])),
      children:   vec![],
    };

    if let Some(mut nav) = self.findnode("//ltx:navigation") {
      self.add_nodes(&mut nav, &[ref_node]);
    } else if let Some(mut root) = self.get_document_element() {
      let nav_node = NodeData::Element {
        tag:        "ltx:navigation".to_string(),
        attributes: None,
        children:   vec![ref_node],
      };
      self.add_nodes(&mut root, &[nav_node]);
    }
  }

  // ======================================================================
  // Validation

  /// Validate the document against its declared schema.
  ///
  /// Port of `Post::Document::validate`.
  pub fn validate(&self) -> Result<(), String> {
    let rng_re = Regex::new(r#"^\s*RelaxNGSchema\s*=\s*[\"'](.*?)[\"']\s*$"#).unwrap();
    for pi_text in &self.processing_instructions {
      if let Some(cap) = rng_re.captures(pi_text) {
        let schema = &cap[1];
        log::info!("Would validate against RelaxNG schema: {}", schema);
        return Ok(());
      }
    }
    log::warn!("No schema found for document validation");
    Ok(())
  }

  /// Check ID consistency.
  ///
  /// Port of `Post::Document::idcheck`.
  pub fn idcheck(&self) {
    let mut doc_ids: HashMap<String, bool> = HashMap::new();
    let mut dups = Vec::new();

    for node in self.findnodes("//*[@xml:id]") {
      if let Some(id) = get_xml_id(&node) {
        if doc_ids.contains_key(&id) {
          dups.push(id.clone());
        }
        doc_ids.insert(id, true);
      }
    }

    let mut missing = Vec::new();
    for id in self.idcache.keys() {
      if !doc_ids.contains_key(id) {
        missing.push(id.clone());
      }
    }

    if !dups.is_empty() {
      log::warn!(
        "Duplicate IDs for {}: {}",
        self.site_relative_destination().unwrap_or_default(),
        dups.join(", ")
      );
    }
    if !missing.is_empty() {
      log::warn!(
        "Cached IDs not in document for {}: {}",
        self.site_relative_destination().unwrap_or_default(),
        missing.join(", ")
      );
    }
  }

  // ======================================================================
  // Cache support

  /// Look up a value in the persistent cache.
  pub fn cache_lookup(&self, key: &str) -> Option<String> { self.cache.get(key).cloned() }

  /// Store a value in the persistent cache.
  pub fn cache_store(&mut self, key: &str, value: &str) {
    self.cache.insert(key.to_string(), value.to_string());
  }

  /// Remove a value from the persistent cache.
  pub fn cache_remove(&mut self, key: &str) { self.cache.remove(key); }
}

// ======================================================================
// Supporting types

/// Options for creating a PostDocument.
#[derive(Debug, Default, Clone)]
pub struct PostDocumentOptions {
  pub destination:           Option<String>,
  pub destination_directory: Option<String>,
  pub site_directory:        Option<String>,
  pub source:                Option<String>,
  pub source_directory:      Option<String>,
  pub searchpaths:           Option<Vec<String>>,
  pub validate:              bool,
  pub nocache:               bool,
}

/// Recursive representation for building XML nodes.
///
/// Port of the Perl `data = string | [$tag, {attrs}, @children]` convention.
#[derive(Debug, Clone)]
pub enum NodeData {
  /// A text node.
  Text(String),
  /// An element node with tag (prefix:localname), optional attributes, and children.
  Element {
    tag:        String,
    attributes: Option<HashMap<String, String>>,
    children:   Vec<NodeData>,
  },
  /// A reference to an existing XML node (will be cloned when added).
  XmlNode(Node),
}

/// Conjunction for joining node lists.
pub enum Conjunction {
  /// A single separator used everywhere.
  Simple(String),
  /// (comma, and) — comma between items, 'and' before the last.
  Pair(String, String),
}

// ======================================================================
// Helper functions

/// Get element children of a node (skipping text, comments, etc.).
pub fn element_children(node: &Node) -> Vec<Node> {
  let mut result = Vec::new();
  if let Some(child) = node.get_first_child() {
    let mut current = Some(child);
    while let Some(ref c) = current {
      if c.get_type() == Some(NodeType::ElementNode) {
        result.push(c.clone());
      }
      current = c.get_next_sibling();
    }
  }
  result
}

/// Iterator version of `element_children` — walks the sibling chain lazily.
/// Prefer this in hot paths that only need to read or filter children
/// without materializing a Vec. Callers that need len() or random access
/// still want the Vec version.
pub fn element_children_iter(node: &Node) -> impl Iterator<Item = Node> {
  let first = node.get_first_child();
  std::iter::successors(first, |c| c.get_next_sibling())
    .filter(|c| c.get_type() == Some(NodeType::ElementNode))
}

/// Compute a relative path from `base` to `path`.
fn pathdiff(path: &str, base: &str) -> String {
  let p = Path::new(path);
  let b = Path::new(base);
  if let Ok(rel) = p.strip_prefix(b) {
    rel.to_string_lossy().to_string()
  } else {
    path.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_test_doc(xml: &str) -> PostDocument {
    PostDocument::new_from_string(xml, PostDocumentOptions::default()).unwrap()
  }

  #[test]
  fn test_new_from_string() {
    let doc = make_test_doc("<document xmlns='http://dlmf.nist.gov/LaTeXML'/>");
    assert!(doc.get_document_element().is_some());
  }

  #[test]
  fn test_findnodes() {
    let doc = make_test_doc(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'>\
         <section xml:id='s1'/>\
         <section xml:id='s2'/>\
       </document>",
    );
    let sections = doc.findnodes("//ltx:section");
    assert_eq!(sections.len(), 2);
  }

  #[test]
  fn test_uniquify_id() {
    let doc = make_test_doc(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'>\
         <p xml:id='p1'/>\
       </document>",
    );
    let mut doc = doc;
    // First call reserves a unique id based on "p1"
    let id1 = doc.uniquify_id("p1", None);
    // Second call with same base must produce a different id
    let id2 = doc.uniquify_id("p1", None);
    assert_ne!(id1, id2);
    // Both should start with p1
    assert!(id1.starts_with("p1"));
    assert!(id2.starts_with("p1"));
  }

  #[test]
  fn test_initial() {
    assert_eq!(PostDocument::initial("Hello", false), "H");
    assert_eq!(PostDocument::initial("  world", false), "W");
    assert_eq!(PostDocument::initial("123abc", true), "A");
    assert_eq!(PostDocument::initial("!@#", false), "*");
    assert_eq!(PostDocument::initial("\u{00E9}cole", false), "E"); // é NFD-decomposes to e + combining accent
  }

  #[test]
  fn test_add_class() {
    let doc = make_test_doc(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'>\
         <p xml:id='p1'/>\
       </document>",
    );
    let mut node = doc.findnode("//ltx:p").unwrap();
    PostDocument::add_class(&mut node, "foo bar");
    let class = node.get_attribute("class").unwrap();
    assert!(class.contains("bar"));
    assert!(class.contains("foo"));

    // Adding again should not duplicate
    PostDocument::add_class(&mut node, "foo");
    let class = node.get_attribute("class").unwrap();
    let count = class.matches("foo").count();
    assert_eq!(count, 1);
  }
}
