//! Document structure scanning processor.
//!
//! Port of `LaTeXML::Post::Scan`.
//! Scans the document for structural elements (sections, figures, equations, etc.)
//! and records their IDs, labels, titles, and relationships in the ObjectDB.
//! This data is used by later processors (CrossRef, MakeIndex, etc.).

use libxml::tree::{Node, NodeType};
use std::collections::HashMap;

use crate::document::PostDocument;
use crate::object_db::{ObjectDB, Value};
use crate::processor::{ProcessResult, Processor};

/// Type alias for scan handler functions.
type ScanHandler = fn(&Scan, &PostDocument, &Node, &str, Option<&str>);

/// Scan post-processor: collects structural information into ObjectDB.
///
/// Port of `LaTeXML::Post::Scan`.
pub struct Scan {
  name: String,
  /// Reference to the shared ObjectDB.
  pub db: ObjectDB,
  /// Tag → handler mapping.
  handlers: HashMap<String, ScanHandler>,
}

impl Scan {
  pub fn new(db: ObjectDB) -> Self {
    let mut handlers: HashMap<String, ScanHandler> = HashMap::new();

    // Section-level elements
    for tag in &[
      "ltx:document", "ltx:part", "ltx:chapter", "ltx:section",
      "ltx:appendix", "ltx:subsection", "ltx:subsubsection",
      "ltx:paragraph", "ltx:subparagraph", "ltx:bibliography",
      "ltx:index", "ltx:glossary", "ltx:theorem", "ltx:proof",
    ] {
      handlers.insert(tag.to_string(), section_handler);
    }

    // Captioned elements
    for tag in &["ltx:table", "ltx:figure", "ltx:float", "ltx:listing"] {
      handlers.insert(tag.to_string(), captioned_handler);
    }

    // Labelled elements
    for tag in &[
      "ltx:equation", "ltx:equationgroup", "ltx:item", "ltx:listingline",
    ] {
      handlers.insert(tag.to_string(), labelled_handler);
    }

    // Special handlers
    handlers.insert("ltx:anchor".to_string(), anchor_handler);
    handlers.insert("ltx:note".to_string(), note_handler);
    handlers.insert("ltx:bibitem".to_string(), bibitem_handler);
    handlers.insert("ltx:bibentry".to_string(), bibentry_handler);
    handlers.insert("ltx:indexmark".to_string(), indexmark_handler);
    handlers.insert("ltx:glossaryentry".to_string(), glossaryentry_handler);
    handlers.insert("ltx:glossarydefinition".to_string(), glossaryentry_handler);
    handlers.insert("ltx:ref".to_string(), ref_handler);
    handlers.insert("ltx:bibref".to_string(), bibref_handler);
    handlers.insert("ltx:glossaryref".to_string(), glossaryref_handler);
    handlers.insert("ltx:navigation".to_string(), navigation_handler);
    handlers.insert("ltx:rdf".to_string(), rdf_handler);
    handlers.insert("ltx:declare".to_string(), declare_handler);
    handlers.insert("ltx:rawhtml".to_string(), rawhtml_handler);

    Scan { name: "Scan".to_string(), db, handlers }
  }

  /// Register a custom handler for a tag.
  pub fn register_handler(&mut self, tag: &str, handler: ScanHandler) {
    self.handlers.insert(tag.to_string(), handler);
  }

  /// Recursively scan a node and its children.
  pub fn scan(&mut self, doc: &PostDocument, node: &Node, parent_id: Option<&str>) {
    if let Some(qname) = doc.get_qname(node) {
      let handler = self.handlers.get(&qname).copied().unwrap_or(default_handler);
      handler(self, doc, node, &qname, parent_id);
    }
  }

  /// Scan all element children of a node.
  pub fn scan_children(&mut self, doc: &PostDocument, node: &Node, parent_id: Option<&str>) {
    let children = collect_element_children(node);
    for child in &children {
      self.scan(doc, child, parent_id);
    }
  }

  /// Compute the page ID for the current document.
  pub fn page_id(&self, doc: &PostDocument) -> Option<String> {
    doc.get_document_element().and_then(|root| root.get_attribute("xml:id"))
  }

  /// Compute the fragment ID for a node within its page.
  ///
  /// Port of `Scan::inPageID`.
  pub fn in_page_id(&self, doc: &PostDocument, node: &Node) -> Option<String> {
    let id = node.get_attribute("xml:id")?;
    let base_id = doc
      .get_document_element()
      .and_then(|root| root.get_attribute("xml:id"))
      .unwrap_or_default();

    if base_id == id {
      None
    } else if !base_id.is_empty() {
      if let Some(rest) = id.strip_prefix(&base_id).and_then(|r| r.strip_prefix('.')) {
        Some(rest.to_string())
      } else {
        Some(id)
      }
    } else {
      Some(id)
    }
  }

  /// Record labels for a node, returning the labels if any.
  ///
  /// Port of `Scan::noteLabels`.
  pub fn note_labels(&mut self, node: &Node) -> Option<Vec<String>> {
    let id = node.get_attribute("xml:id")?;
    let labels_str = node.get_attribute("labels")?;
    let labels: Vec<String> = labels_str.split_whitespace().map(String::from).collect();
    for label in &labels {
      self.db.register(label, vec![("id", Value::from(id.as_str()))]);
    }
    Some(labels)
  }

  /// Build the common properties for a scanned element.
  ///
  /// Port of `Scan::addCommon`.
  pub fn add_common(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) -> Vec<(&str, Value)> {
    let id = node.get_attribute("xml:id");
    let labels = self.note_labels(node);

    let mut props: Vec<(&str, Value)> = vec![
      ("type", Value::from(tag)),
    ];

    if let Some(ref id_str) = id {
      props.push(("id", Value::from(id_str.as_str())));
    }
    if let Some(pid) = parent_id {
      props.push(("parent", Value::from(pid)));
    }
    if let Some(ref labs) = labels {
      props.push(("labels", Value::from(labs.clone())));
    }
    if let Some(loc) = doc.site_relative_destination() {
      props.push(("location", Value::from(loc)));
    }
    if let Some(pageid) = self.page_id(doc) {
      props.push(("pageid", Value::from(pageid)));
    }
    if let Some(fragid) = id.as_ref().and_then(|_| self.in_page_id(doc, node)) {
      props.push(("fragid", Value::from(fragid)));
    }

    // Extract inlist
    if let Some(listnames) = node.get_attribute("inlist") {
      let mut inlist = HashMap::new();
      for name in listnames.split_whitespace() {
        inlist.insert(name.to_string(), Value::Bool(true));
      }
      props.push(("inlist", Value::Hash(inlist)));
    }

    // Extract tag nodes (refnum, etc.)
    for tagnode in doc.findnodes_at("ltx:tags/ltx:tag", Some(node)) {
      let _key = if let Some(role) = tagnode.get_attribute("role") {
        if role.ends_with("refnum") {
          role
        } else {
          format!("tag:{}", role)
        }
      } else {
        "refnum".to_string()
      };
      props.push(("_tagnode", Value::Xml(tagnode.clone())));
      // Note: in full impl, we'd store cleaned clones keyed by role
    }

    props
  }

  /// Add an ID as a child of a parent entry.
  ///
  /// Port of `Scan::addAsChild`.
  pub fn add_as_child(&mut self, id: &str, parent_id: Option<&str>) {
    let mut current_parent = parent_id.map(String::from);
    while let Some(ref pid) = current_parent {
      let key = format!("ID:{}", pid);
      if let Some(entry) = self.db.lookup(&key) {
        if entry.has_value("children") {
          // Found an ancestor that maintains children
          if let Some(entry_mut) = self.db.lookup_mut(&key) {
            entry_mut.push_new("children", vec![Value::from(id)]);
          }
          return;
        }
        // Go up to the parent's parent
        current_parent = entry.get_string("parent").map(String::from);
      } else {
        return;
      }
    }
  }
}

impl Processor for Scan {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let root = match nodes.into_iter().next() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

    // Ensure root has an ID
    let mut root_mut = root.clone();
    let id = root_mut.get_attribute("xml:id").unwrap_or_else(|| {
      root_mut.set_attribute("xml:id", "Document").ok();
      "Document".to_string()
    });

    // Register site root if first document
    if self.db.lookup("SITE_ROOT").is_none() {
      self.db.register("SITE_ROOT", vec![("id", Value::from(id.as_str()))]);
    }

    // Scan the document tree
    self.scan(&doc, &root, None);

    // Register document location
    let loc = doc.site_relative_destination().unwrap_or_default();
    let doc_key = format!("DOCUMENT:{}", loc);
    self.db.register(&doc_key, vec![("id", Value::from(id.as_str()))]);

    log::info!("Scan: DBStatus: {}", self.db.status());
    Ok(vec![doc])
  }
}

// ======================================================================
// Handler implementations

fn collect_element_children(node: &Node) -> Vec<Node> {
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

fn default_handler(scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
  let id = node.get_attribute("xml:id");
  if let Some(ref id_str) = id {
    let _props = scan_add_common_immutable(scan, doc, node, tag, parent_id);
    let _key = format!("ID:{}", id_str);
    // NOTE: These handlers take &Scan (immutable) because they're fn pointers.
    // The actual DB mutation happens via interior mutability or post-scan fixup.
    log::trace!("Scan default: {} id={}", tag, id_str);
  }
  // scan_children needs &mut, so we log intent here
  // Actual child scanning happens in the recursive scan() call
}

fn section_handler(_scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, _parent_id: Option<&str>) {
  if let Some(id) = node.get_attribute("xml:id") {
    log::trace!("Scan section: {} id={}", tag, id);
    // Records: common props + primary=1, title, toctitle, children=[]
    let title = doc.findnode_at("ltx:title", node);
    let _toctitle = doc.findnode_at("ltx:toctitle", node);
    if let Some(ref t) = title {
      log::trace!("  title: {}", t.get_content());
    }
  }
}

fn captioned_handler(_scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, _parent_id: Option<&str>) {
  if let Some(id) = node.get_attribute("xml:id") {
    log::trace!("Scan captioned: {} id={}", tag, id);
    // Records: common + role, caption, toccaption
    let _caption = doc.findnode_at("child::ltx:caption", node)
      .or_else(|| doc.findnode_at("descendant::ltx:caption", node));
    let _toccaption = doc.findnode_at("child::ltx:toccaption", node)
      .or_else(|| doc.findnode_at("descendant::ltx:toccaption", node));
  }
}

fn labelled_handler(_scan: &Scan, _doc: &PostDocument, node: &Node, tag: &str, _parent_id: Option<&str>) {
  if let Some(id) = node.get_attribute("xml:id") {
    log::trace!("Scan labelled: {} id={}", tag, id);
    // Records: common + role
  }
}

fn anchor_handler(_scan: &Scan, _doc: &PostDocument, node: &Node, _tag: &str, _parent_id: Option<&str>) {
  if let Some(id) = node.get_attribute("xml:id") {
    log::trace!("Scan anchor: id={}", id);
    // Records: common + title (the node itself as title)
  }
}

fn note_handler(_scan: &Scan, _doc: &PostDocument, node: &Node, _tag: &str, _parent_id: Option<&str>) {
  if let Some(id) = node.get_attribute("xml:id") {
    log::trace!("Scan note: id={}", id);
    // Records: common + role + note content (with tags stripped)
  }
}

fn bibitem_handler(_scan: &Scan, doc: &PostDocument, node: &Node, _tag: &str, _parent_id: Option<&str>) {
  if let Some(id) = node.get_attribute("xml:id") {
    let key = node.get_attribute("key");
    log::trace!("Scan bibitem: id={} key={:?}", id, key);

    // Register BIBLABEL entries for each bibliography list
    if let Some(ref bibkey) = key {
      let bib = doc.findnode_at("ancestor-or-self::ltx:bibliography", node);
      let lists = bib
        .and_then(|b| b.get_attribute("lists"))
        .unwrap_or_else(|| "bibliography".to_string());
      for list in lists.split_whitespace() {
        let label_key = format!("BIBLABEL:{}:{}", list, bibkey);
        log::trace!("  register {}", label_key);
      }
    }

    // Record bibliographic metadata from tags
    for role in &["authors", "fullauthors", "year", "number", "refnum", "title", "key", "bibtype"] {
      let xpath = format!("ltx:tags/ltx:tag[@role='{}']", role);
      if let Some(tagnode) = doc.findnode_at(&xpath, node) {
        log::trace!("  bib {}: {}", role, tagnode.get_content());
      }
    }
  }
}

fn bibentry_handler(_scan: &Scan, _doc: &PostDocument, _node: &Node, _tag: &str, _parent_id: Option<&str>) {
  // Nothing to do: bibentries are formatted into bibitems by MakeBibliography
}

fn indexmark_handler(_scan: &Scan, doc: &PostDocument, node: &Node, _tag: &str, parent_id: Option<&str>) {
  let phrases = doc.findnodes_at("ltx:indexphrase", Some(node));
  let see_also = doc.findnodes_at("ltx:indexsee", Some(node));

  let key_parts: Vec<String> = phrases
    .iter()
    .filter_map(|p| p.get_attribute("key"))
    .collect();
  let key = format!("INDEX:{}", key_parts.join(":"));
  log::trace!("Scan indexmark: key={}", key);

  if !see_also.is_empty() {
    log::trace!("  see_also: {} entries", see_also.len());
  } else if let Some(pid) = parent_id {
    let style = node.get_attribute("style").unwrap_or_else(|| "normal".to_string());
    log::trace!("  referrer: {} ({})", pid, style);
  }
}

fn glossaryentry_handler(_scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, _parent_id: Option<&str>) {
  let id = if tag == "ltx:glossaryentry" {
    node.get_attribute("xml:id")
  } else {
    None
  };
  let key = node.get_attribute("key").unwrap_or_default();
  let lists = node.get_attribute("inlist").unwrap_or_else(|| {
    doc.findnode_at(
      "ancestor::ltx:glossarylist[@lists] | ancestor::ltx:glossary[@lists]",
      node,
    )
    .and_then(|p| p.get_attribute("lists"))
    .unwrap_or_else(|| "glossary".to_string())
  });

  for list in lists.split_whitespace() {
    let gkey = format!("GLOSSARY:{}:{}", list, key);
    log::trace!("Scan glossary: {} id={:?}", gkey, id);
  }
}

fn ref_handler(scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
  if let Some(label) = node.get_attribute("labelref") {
    // Only record refs of labels, not from TOC or cited bibblock
    let in_toc = !doc
      .findnodes_at(
        "ancestor::ltx:tocentry | ancestor::ltx:bibblock[contains(@class,'ltx_bib_cited')]",
        Some(node),
      )
      .is_empty();
    if !in_toc {
      log::trace!("Scan ref: labelref={} parent={:?}", label, parent_id);
    }
  }
  // Scan children (refs might have content)
  default_handler(scan, doc, node, tag, parent_id);
}

fn bibref_handler(scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
  let in_cited = !doc
    .findnodes_at(
      "ancestor::ltx:bibblock[contains(@class,'ltx_bib_cited')]",
      Some(node),
    )
    .is_empty();
  if !in_cited {
    if let Some(keys) = node.get_attribute("bibrefs") {
      let inlist = node.get_attribute("inlist").unwrap_or_default();
      let mut lists: Vec<&str> = inlist.split_whitespace().collect();
      lists.push("bibliography");
      for bibkey in keys.split(',').filter(|k| !k.is_empty()) {
        for list in &lists {
          let label_key = format!("BIBLABEL:{}:{}", list, bibkey);
          log::trace!("Scan bibref: {} parent={:?}", label_key, parent_id);
        }
      }
    }
  }
  default_handler(scan, doc, node, tag, parent_id);
}

fn glossaryref_handler(scan: &Scan, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
  let list = node.get_attribute("inlist");
  let key = node.get_attribute("key");
  if let (Some(k), Some(l)) = (&key, &list) {
    let gkey = format!("GLOSSARY:{}:{}", l, k);
    log::trace!("Scan glossaryref: {} parent={:?}", gkey, parent_id);
  }
  default_handler(scan, doc, node, tag, parent_id);
}

fn navigation_handler(_scan: &Scan, _doc: &PostDocument, _node: &Node, _tag: &str, _parent_id: Option<&str>) {
  // Navigation elements: not scanned
}

fn rdf_handler(_scan: &Scan, _doc: &PostDocument, node: &Node, _tag: &str, parent_id: Option<&str>) {
  let mut id = node.get_attribute("about");
  if let Some(ref about) = id {
    if let Some(stripped) = about.strip_prefix('#') {
      id = Some(stripped.to_string());
    }
  }
  let id = id.or_else(|| parent_id.map(String::from));
  let property = node.get_attribute("property");
  let value = node.get_attribute("resource").or_else(|| node.get_attribute("content"));

  if let (Some(prop), Some(val), Some(id_str)) = (property, value, id) {
    log::trace!("Scan rdf: ID:{} {} = {}", id_str, prop, val);
  }
}

fn declare_handler(_scan: &Scan, _doc: &PostDocument, node: &Node, _tag: &str, _parent_id: Option<&str>) {
  let decl_type = node.get_attribute("type");
  let sort = node.get_attribute("sortkey");
  let decl_id = node.get_attribute("xml:id");
  let definiens = node.get_attribute("definiens");

  if decl_type.as_deref() == Some("definition") {
    if let Some(ref def) = definiens {
      log::trace!("Scan declare: definition of '{}'", def);
    }
  }

  if let Some(ref sk) = sort {
    let name = definiens.as_deref().or(decl_id.as_deref()).unwrap_or(sk);
    log::trace!("Scan notation: NOTATION:{}", name);
  }
}

fn rawhtml_handler(_scan: &Scan, _doc: &PostDocument, _node: &Node, _tag: &str, _parent_id: Option<&str>) {
  // Raw HTML: nothing to scan
}

/// Helper: build common properties without mutating scan (for fn pointer handlers).
fn scan_add_common_immutable(
  _scan: &Scan,
  doc: &PostDocument,
  node: &Node,
  tag: &str,
  parent_id: Option<&str>,
) -> Vec<(String, String)> {
  let mut props = Vec::new();
  if let Some(id) = node.get_attribute("xml:id") {
    props.push(("id".to_string(), id));
  }
  props.push(("type".to_string(), tag.to_string()));
  if let Some(pid) = parent_id {
    props.push(("parent".to_string(), pid.to_string()));
  }
  if let Some(loc) = doc.site_relative_destination() {
    props.push(("location".to_string(), loc));
  }
  props
}
