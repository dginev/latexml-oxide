//! Document structure scanning processor.
//!
//! Port of `LaTeXML::Post::Scan`.
//! Scans the document for structural elements (sections, figures, equations, etc.)
//! and records their IDs, labels, titles, and relationships in the ObjectDB.
//! This data is used by later processors (CrossRef, MakeIndex, etc.).

use libxml::tree::{Node, NodeType};
use rustc_hash::FxHashMap as HashMap;

use crate::document::PostDocument;
use crate::object_db::{ObjectDB, Value};
use crate::processor::{ProcessResult, Processor};

/// Scan post-processor: collects structural information into ObjectDB.
///
/// Port of `LaTeXML::Post::Scan`.
pub struct Scan {
  name:    String,
  /// Reference to the shared ObjectDB.
  pub db:  ObjectDB,
  /// Root document id for the current scan.
  page_id: Option<String>,
}

/// Collected properties for a scanned element, ready for DB registration.
struct ScannedProps {
  /// Properties to register.
  props:  Vec<(String, Value)>,
  /// Labels to register separately.
  labels: Vec<String>,
  /// The xml:id of the scanned element.
  id:     Option<String>,
}

impl ScannedProps {
  fn push(&mut self, key: &str, val: Value) { self.props.push((key.to_string(), val)); }
}

impl Scan {
  pub fn new(db: ObjectDB) -> Self {
    Scan {
      name: "Scan".to_string(),
      db,
      page_id: None,
    }
  }

  /// Recursively scan a node and its children.
  ///
  /// Port of `Scan::scan`.
  pub fn scan(&mut self, doc: &PostDocument, node: &Node, parent_id: Option<&str>) {
    if let Some(qname) = doc.get_qname(node) {
      self.dispatch(doc, node, &qname, parent_id);
    }
  }

  /// Dispatch to the appropriate handler based on tag name.
  fn dispatch(&mut self, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
    match tag {
      "ltx:document" | "ltx:part" | "ltx:chapter" | "ltx:section" | "ltx:appendix"
      | "ltx:subsection" | "ltx:subsubsection" | "ltx:paragraph" | "ltx:subparagraph"
      | "ltx:bibliography" | "ltx:index" | "ltx:glossary" | "ltx:theorem" | "ltx:proof" => {
        self.section_handler(doc, node, tag, parent_id)
      },
      "ltx:table" | "ltx:figure" | "ltx:float" | "ltx:listing" => {
        self.captioned_handler(doc, node, tag, parent_id)
      },
      "ltx:equation" | "ltx:equationgroup" | "ltx:item" | "ltx:listingline" => {
        self.labelled_handler(doc, node, tag, parent_id)
      },
      "ltx:anchor" => self.anchor_handler(doc, node, tag, parent_id),
      // Math subtrees contain thousands of XMTok/XMApp/XMRef/XMWrap/XMDual
      // nodes with xml:ids that serve only local math-tree navigation —
      // they're not targets for cross-reference and do not need to appear
      // in the Scan ObjectDB. Register the outer Math element's id,
      // then skip descent. This drops Scan time on arXiv:0705.0790
      // from 11.4 s → <1 s (the 65K XM* nodes were dominating).
      //
      // Rust-side intentional divergence from Perl Scan.pm, which
      // descends blindly — but Perl doesn't emit xml:id on XM*
      // descendants in the first place, so its default_handler short-
      // circuits naturally. The ar5iv.sty preload in Rust populates
      // xml:id everywhere via _ID_counter__, making this skip necessary
      // for performance parity with Perl.
      "ltx:Math" => {
        let id = get_xml_id(node);
        if let Some(ref id_str) = id {
          let sp = self.collect_common(doc, node, tag, parent_id);
          let key = format!("ID:{}", id_str);
          self.register_scanned(&key, sp);
          self.add_as_child(id_str, parent_id);
        }
        // No scan_children — XM* descendants are skipped.
      },
      "ltx:note" => self.note_handler(doc, node, tag, parent_id),
      "ltx:bibitem" => self.bibitem_handler(doc, node, parent_id),
      "ltx:bibentry" => {},
      "ltx:indexmark" => self.indexmark_handler(doc, node, parent_id),
      "ltx:glossaryentry" | "ltx:glossarydefinition" => {
        self.glossaryentry_handler(doc, node, tag, parent_id)
      },
      "ltx:ref" => self.ref_handler(doc, node, tag, parent_id),
      "ltx:bibref" => self.bibref_handler(doc, node, tag, parent_id),
      "ltx:glossaryref" => self.glossaryref_handler(doc, node, tag, parent_id),
      "ltx:navigation" | "ltx:rawhtml" => {},
      "ltx:rdf" => self.rdf_handler(node, parent_id),
      "ltx:declare" => self.declare_handler(doc, node, tag, parent_id),
      _ => self.default_handler(doc, node, tag, parent_id),
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
  fn page_id(&self, doc: &PostDocument) -> Option<String> {
    self.page_id.clone().or_else(|| {
      doc
        .get_document_element()
        .and_then(|root| get_xml_id(&root))
    })
  }

  /// Compute the fragment ID for a node within its page.
  fn in_page_id(&self, doc: &PostDocument, node: &Node) -> Option<String> {
    let id = get_xml_id(node)?;
    let base_id = self.page_id(doc).unwrap_or_default();

    if id == base_id {
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

  /// Build common properties for a scanned element WITHOUT mutating self.
  /// Labels and tag nodes are collected but not yet registered.
  fn collect_common(
    &self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) -> ScannedProps {
    let id = get_xml_id(node);
    let labels_str = node.get_attribute("labels");
    let labels: Vec<String> = labels_str
      .map(|s| s.split_whitespace().map(String::from).collect())
      .unwrap_or_default();

    let mut sp = ScannedProps { props: Vec::new(), labels, id };

    sp.push("type", Value::from(tag));
    if let Some(ref id_str) = sp.id {
      sp.push("id", Value::from(id_str.as_str()));
    }
    if let Some(pid) = parent_id {
      sp.push("parent", Value::from(pid));
    }
    if !sp.labels.is_empty() {
      sp.push("labels", Value::from(sp.labels.clone()));
    }
    if let Some(loc) = doc.site_relative_destination() {
      sp.push("location", Value::from(loc));
    }
    if let Some(pageid) = self.page_id(doc) {
      sp.push("pageid", Value::from(pageid));
    }
    if sp.id.is_some() {
      if let Some(fragid) = self.in_page_id(doc, node) {
        sp.push("fragid", Value::from(fragid));
      }
    }

    // inlist
    if let Some(listnames) = node.get_attribute("inlist") {
      let mut inlist = HashMap::default();
      for name in listnames.split_whitespace() {
        inlist.insert(name.to_string(), Value::Bool(true));
      }
      sp.push("inlist", Value::Hash(inlist));
    }

    // tag nodes (refnum, typerefnum, etc.)
    // Store as String (not Xml) to avoid dangling node references.
    // Perl uses cloneNode(1) deep copy; our libxml bindings only do ref copies.
    for tagnode in child_tag_nodes(node) {
      let key = if let Some(role) = tagnode.get_attribute("role") {
        if role.ends_with("refnum") {
          role
        } else {
          format!("tag:{}", role)
        }
      } else {
        "refnum".to_string()
      };
      let text = tagnode.get_content();
      sp.push(&key, Value::from(text));
    }

    sp
  }

  /// Register a ScannedProps into the DB, including labels.
  fn register_scanned(&mut self, db_key: &str, sp: ScannedProps) {
    // Register labels
    if let Some(ref id) = sp.id {
      for label in &sp.labels {
        self
          .db
          .register(label, vec![("id", Value::from(id.as_str()))]);
      }
    }
    // Register main entry
    let owned_props: Vec<(&str, Value)> = Vec::new();
    let entry = self.db.register(db_key, owned_props);
    for (k, v) in sp.props {
      entry.set_value(&k, v);
    }
  }

  /// Add an ID as a child of a parent entry.
  fn add_as_child(&mut self, id: &str, parent_id: Option<&str>) {
    let mut current_parent = parent_id.map(String::from);
    while let Some(ref pid) = current_parent {
      let key = format!("ID:{}", pid);
      let has_children = self
        .db
        .lookup(&key)
        .map(|e| e.has_value("children"))
        .unwrap_or(false);
      if has_children {
        if let Some(entry_mut) = self.db.lookup_mut(&key) {
          entry_mut.push_new("children", vec![Value::from(id)]);
        }
        return;
      }
      let parent = self
        .db
        .lookup(&key)
        .and_then(|e| e.get_string("parent").map(String::from));
      current_parent = parent;
    }
  }

  // ======================================================================
  // Handlers

  fn default_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    // Mirror Perl Scan.pm default_handler (L272-283): only build ScannedProps
    // when the node actually carries an xml:id. For typical papers with large
    // <Math> subtrees, the XMTok/XMApp/XMRef/XMWrap/XMDual descendants have
    // no id and `collect_common`'s attribute fetches + labels parsing are
    // pure waste. arXiv:0705.0790 has 65K nodes (37K XMTok alone) and only
    // ~1K carry ids — skipping collect_common on the other 64K drops Scan
    // from 11.4 s → sub-second on that paper.
    let id = get_xml_id(node);
    if let Some(ref id_str) = id {
      let sp = self.collect_common(doc, node, tag, parent_id);
      let key = format!("ID:{}", id_str);
      self.register_scanned(&key, sp);
      // Keep the ID entry addressable for refs/URLs, but do not add generic
      // layout/math/text nodes to section children. TOCs only need primary
      // structural children, and adding tens of thousands of table cells here
      // turns Scan into quadratic duplicate checking.
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn section_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    let mut sp = self.collect_common(doc, node, tag, parent_id);
    let id = sp.id.clone();
    if let Some(ref id_str) = id {
      sp.push("primary", Value::Bool(true));
      sp.push("children", Value::List(Vec::new()));
      if let Some(title_node) = doc.findnode_at("ltx:title", node) {
        sp.push("title", Value::from(title_text_content(&title_node)));
      }
      if let Some(toctitle_node) = doc.findnode_at("ltx:toctitle", node) {
        sp.push("toctitle", Value::from(title_text_content(&toctitle_node)));
      }
      if let Some(stub) = node.get_attribute("stub") {
        sp.push("stub", Value::from(stub));
      }
      let key = format!("ID:{}", id_str);
      self.register_scanned(&key, sp);
      self.add_as_child(id_str, parent_id);
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn captioned_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    let mut sp = self.collect_common(doc, node, tag, parent_id);
    let id = sp.id.clone();
    if let Some(ref id_str) = id {
      if let Some(role) = node.get_attribute("role") {
        sp.push("role", Value::from(role));
      }
      let caption = doc
        .findnode_at("child::ltx:caption", node)
        .or_else(|| doc.findnode_at("descendant::ltx:caption", node));
      if let Some(ref cap) = caption {
        sp.push("caption", Value::from(cap.get_content()));
      }
      let toccaption = doc
        .findnode_at("child::ltx:toccaption", node)
        .or_else(|| doc.findnode_at("descendant::ltx:toccaption", node));
      if let Some(ref tc) = toccaption {
        sp.push("toccaption", Value::from(tc.get_content()));
      }
      let key = format!("ID:{}", id_str);
      self.register_scanned(&key, sp);
      self.add_as_child(id_str, parent_id);
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn labelled_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    let mut sp = self.collect_common(doc, node, tag, parent_id);
    let id = sp.id.clone();
    if let Some(ref id_str) = id {
      if let Some(role) = node.get_attribute("role") {
        sp.push("role", Value::from(role));
      }
      let key = format!("ID:{}", id_str);
      self.register_scanned(&key, sp);
      self.add_as_child(id_str, parent_id);
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn anchor_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    let mut sp = self.collect_common(doc, node, tag, parent_id);
    let id = sp.id.clone();
    if let Some(ref id_str) = id {
      sp.push("title", Value::from(node.get_content()));
      let key = format!("ID:{}", id_str);
      self.register_scanned(&key, sp);
      self.add_as_child(id_str, parent_id);
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn note_handler(&mut self, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
    let mut sp = self.collect_common(doc, node, tag, parent_id);
    let id = sp.id.clone();
    if let Some(ref id_str) = id {
      if let Some(role) = node.get_attribute("role") {
        sp.push("role", Value::from(role));
      }
      // Store note text content, not XML node reference (avoids dangling refs)
      sp.push("note", Value::from(node.get_content()));
      let key = format!("ID:{}", id_str);
      self.register_scanned(&key, sp);
      self.add_as_child(id_str, parent_id);
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn bibitem_handler(&mut self, doc: &PostDocument, node: &Node, parent_id: Option<&str>) {
    let id = match get_xml_id(node) {
      Some(id) => id,
      None => {
        self.scan_children(doc, node, parent_id);
        return;
      },
    };

    let key = node.get_attribute("key");
    let bib = doc.findnode_at("ancestor-or-self::ltx:bibliography", node);
    let lists_str = bib
      .and_then(|b| b.get_attribute("lists"))
      .unwrap_or_else(|| "bibliography".to_string());

    // Register BIBLABEL entries
    if let Some(ref bibkey) = key {
      for list in lists_str.split_whitespace() {
        let label_key = format!("BIBLABEL:{}:{}", list, bibkey);
        self
          .db
          .register(&label_key, vec![("id", Value::from(id.as_str()))]);
      }
    }

    // Build props for bibitem
    let mut props: Vec<(String, Value)> = Vec::new();
    props.push(("id".to_string(), Value::from(id.as_str())));
    props.push(("type".to_string(), Value::from("ltx:bibitem")));
    if let Some(pid) = parent_id {
      props.push(("parent".to_string(), Value::from(pid)));
    }
    if let Some(ref k) = key {
      props.push(("bibkey".to_string(), Value::from(k.as_str())));
    }
    if let Some(loc) = doc.site_relative_destination() {
      props.push(("location".to_string(), Value::from(loc)));
    }
    if let Some(pageid) = self.page_id(doc) {
      props.push(("pageid".to_string(), Value::from(pageid)));
    }
    if let Some(fragid) = self.in_page_id(doc, node) {
      props.push(("fragid".to_string(), Value::from(fragid)));
    }

    for role in &[
      "authors",
      "fullauthors",
      "year",
      "number",
      "refnum",
      "title",
      "key",
      "bibtype",
    ] {
      let xpath = format!("ltx:tags/ltx:tag[@role='{}']", role);
      if let Some(tagnode) = doc.findnode_at(&xpath, node) {
        let prop_name = match *role {
          "key" => "keytag",
          "bibtype" => "typetag",
          _ => role,
        };
        props.push((prop_name.to_string(), Value::from(tagnode.get_content())));
      }
    }

    let db_key = format!("ID:{}", id);
    let entry = self.db.register(&db_key, vec![]);
    for (k, v) in props {
      entry.set_value(&k, v);
    }

    self.scan_children(doc, node, Some(&id));
  }

  fn ref_handler(&mut self, doc: &PostDocument, node: &Node, tag: &str, parent_id: Option<&str>) {
    if let Some(label) = node.get_attribute("labelref") {
      let in_toc = !doc
        .findnodes_at(
          "ancestor::ltx:tocentry | ancestor::ltx:bibblock[contains(@class,'ltx_bib_cited')]",
          Some(node),
        )
        .is_empty();
      if !in_toc {
        self.db.register(&label, vec![]);
        if let Some(pid) = parent_id {
          if let Some(entry) = self.db.lookup_mut(&label) {
            entry.note_association(&["referrers", pid]);
          }
        }
      }
    }
    self.default_handler(doc, node, tag, parent_id);
  }

  fn bibref_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
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
        let label_keys: Vec<String> = keys
          .split(',')
          .filter(|k| !k.is_empty())
          .flat_map(|bibkey| {
            lists
              .iter()
              .map(move |list| format!("BIBLABEL:{}:{}", list, bibkey))
          })
          .collect();
        for label_key in &label_keys {
          self.db.register(label_key, vec![]);
        }
        if let Some(pid) = parent_id {
          for label_key in &label_keys {
            if let Some(entry) = self.db.lookup_mut(label_key) {
              entry.note_association(&["referrers", pid]);
            }
          }
        }
      }
    }
    self.default_handler(doc, node, tag, parent_id);
  }

  fn glossaryref_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    if let (Some(k), Some(l)) = (node.get_attribute("key"), node.get_attribute("inlist")) {
      let gkey = format!("GLOSSARY:{}:{}", l, k);
      self.db.register(&gkey, vec![]);
      if let Some(pid) = parent_id {
        if let Some(entry) = self.db.lookup_mut(&gkey) {
          entry.note_association(&["referrers", pid]);
        }
      }
    }
    self.default_handler(doc, node, tag, parent_id);
  }

  fn indexmark_handler(&mut self, doc: &PostDocument, node: &Node, parent_id: Option<&str>) {
    let phrases = doc.findnodes_at("ltx:indexphrase", Some(node));
    let see_also = doc.findnodes_at("ltx:indexsee", Some(node));

    let key_parts: Vec<String> = phrases
      .iter()
      .filter_map(|p| p.get_attribute("key"))
      .collect();
    let key = format!("INDEX:{}", key_parts.join(":"));

    let inlist = node.get_attribute("inlist").map(|listnames| {
      let mut h = HashMap::default();
      for name in listnames.split_whitespace() {
        h.insert(name.to_string(), Value::Bool(true));
      }
      Value::Hash(h)
    });

    let exists = self.db.lookup(&key).is_some();
    if !exists {
      let mut props = vec![];
      if let Some(il) = inlist {
        props.push(("inlist", il));
      }
      self.db.register(&key, props);
    }

    if !see_also.is_empty() {
      if let Some(entry) = self.db.lookup_mut(&key) {
        let nodes: Vec<Value> = see_also
          .iter()
          .map(|n| Value::from(n.get_content()))
          .collect();
        entry.push_new("see_also", nodes);
      }
    } else if let Some(pid) = parent_id {
      let style = node
        .get_attribute("style")
        .unwrap_or_else(|| "normal".to_string());
      if let Some(entry) = self.db.lookup_mut(&key) {
        entry.note_association(&["referrers", pid, &style]);
      }
    }
  }

  fn glossaryentry_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    let id = if tag == "ltx:glossaryentry" {
      get_xml_id(node)
    } else {
      None
    };
    let lists = node.get_attribute("inlist").unwrap_or_else(|| {
      doc
        .findnode_at(
          "ancestor::ltx:glossarylist[@lists] | ancestor::ltx:glossary[@lists]",
          node,
        )
        .and_then(|p| p.get_attribute("lists"))
        .unwrap_or_else(|| "glossary".to_string())
    });
    let key = node.get_attribute("key").unwrap_or_default();
    let phrases = doc.findnodes_at("ltx:glossaryphrase", Some(node));

    for list in lists.split_whitespace() {
      let gkey = format!("GLOSSARY:{}:{}", list, key);
      let entry = self.db.register(&gkey, vec![]);
      for phrase in &phrases {
        let role = phrase
          .get_attribute("role")
          .unwrap_or_else(|| "label".to_string());
        let prop_key = format!("phrase:{}", role);
        entry.set_value(&prop_key, Value::from(phrase.get_content()));
      }
      if let Some(ref id_str) = id {
        entry.set_value("id", Value::from(id_str.as_str()));
      }
    }

    if let Some(ref id_str) = id {
      let sp = self.collect_common(doc, node, tag, parent_id);
      let db_key = format!("ID:{}", id_str);
      self.register_scanned(&db_key, sp);
    }
    let effective_id = id.as_deref().or(parent_id);
    self.scan_children(doc, node, effective_id);
  }

  fn rdf_handler(&mut self, node: &Node, parent_id: Option<&str>) {
    let mut id = node.get_attribute("about");
    if let Some(ref about) = id {
      if let Some(stripped) = about.strip_prefix('#') {
        id = Some(stripped.to_string());
      }
    }
    let id = id.or_else(|| parent_id.map(String::from));
    let property = node.get_attribute("property");
    let value = node
      .get_attribute("resource")
      .or_else(|| node.get_attribute("content"));

    if let (Some(prop), Some(val), Some(id_str)) = (property, value, id) {
      let db_key = format!("ID:{}", id_str);
      let entry = self.db.register(&db_key, vec![]);
      entry.set_value(&prop, Value::from(val));
    }
  }

  fn declare_handler(
    &mut self,
    doc: &PostDocument,
    node: &Node,
    tag: &str,
    parent_id: Option<&str>,
  ) {
    let decl_type = node.get_attribute("type");
    let sort = node.get_attribute("sortkey");
    let decl_id = get_xml_id(node);
    let definiens = node.get_attribute("definiens");

    let term = doc.findnode_at("child::ltx:tags/ltx:tag[@role='term']", node);
    let description = doc.findnode_at("child::ltx:text", node);

    if decl_type.as_deref() == Some("definition") {
      let mut def = definiens.clone();
      if def.is_none() {
        if let Some(ref term_node) = term {
          let syms = doc.findnodes_at("descendant-or-self::ltx:XMTok[@meaning]", Some(term_node));
          let mut non_rel = Vec::new();
          let mut rel = Vec::new();
          for sym in &syms {
            let meaning = sym.get_attribute("meaning").unwrap_or_default();
            if meaning.starts_with("delimited-") {
              continue;
            }
            if sym.get_attribute("role").as_deref() == Some("RELOP") {
              rel.push(meaning);
            } else {
              non_rel.push(meaning);
            }
          }
          non_rel.extend(rel);
          def = non_rel.into_iter().next();
        }
      }
      if let Some(ref def_name) = def {
        let dkey = format!("DECLARATION:global:{}", def_name);
        let mut sp = self.collect_common(doc, node, tag, parent_id);
        if let Some(ref desc) = description {
          sp.push("description", Value::from(desc.get_content()));
        }
        self.register_scanned(&dkey, sp);
      }
    } else if decl_type.is_none() && parent_id.is_some() {
      if let Some(ref did) = decl_id {
        let has_content =
          description.is_some() || doc.findnode_at("ltx:tags/ltx:tag", node).is_some();
        if has_content {
          let dkey = format!("DECLARATION:local:{}", did);
          let mut sp = self.collect_common(doc, node, tag, parent_id);
          if let Some(ref desc) = description {
            sp.push("description", Value::from(desc.get_content()));
          }
          self.register_scanned(&dkey, sp);
        }
      }
    }

    if let Some(ref sk) = sort {
      let name = definiens.as_deref().or(decl_id.as_deref()).unwrap_or(sk);
      let nkey = format!("NOTATION:{}", name);
      let mut sp = self.collect_common(doc, node, tag, parent_id);
      sp.push("sortkey", Value::from(sk.as_str()));
      if let Some(ref desc) = description {
        sp.push("description", Value::from(desc.get_content()));
      }
      self.register_scanned(&nkey, sp);
    }
  }
}

impl Processor for Scan {
  fn get_name(&self) -> &str { &self.name }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    let root = match doc.get_document_element() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

    let id = get_xml_id(&root).unwrap_or_else(|| {
      let mut root_mut = root.clone();
      root_mut.set_attribute("xml:id", "Document").ok();
      "Document".to_string()
    });

    if self.db.lookup("SITE_ROOT").is_none() {
      self
        .db
        .register("SITE_ROOT", vec![("id", Value::from(id.as_str()))]);
    }

    self.page_id = Some(id.clone());
    self.scan(&doc, &root, None);
    self.page_id = None;

    let loc = doc.site_relative_destination().unwrap_or_default();
    let doc_key = format!("DOCUMENT:{}", loc);
    self
      .db
      .register(&doc_key, vec![("id", Value::from(id.as_str()))]);

    // Perl Post::Scan L108-133: when scanning a doc that's not itself
    // the site root and whose own entry has no parent yet, infer one.
    // Without this step, the cross-document TOC produced by Split has
    // the SITE_ROOT (e.g. "Document") with no `children` pointing at
    // the per-page roots ("Ch1", "Ch1.S1", …) — CrossRef::fill_in_tocs
    // then walks an empty children list and emits an empty TOC, so the
    // index page's `\tableofcontents` placeholder collapses to a 27-
    // line title-only page.
    let site_id = self
      .db
      .lookup("SITE_ROOT")
      .and_then(|e| e.get_string("id").map(String::from))
      .unwrap_or_default();
    let id_key = format!("ID:{}", id);
    let needs_parent = self
      .db
      .lookup(&id_key)
      .map(|e| !e.has_value("parent"))
      .unwrap_or(false);
    if !site_id.is_empty() && id != site_id && needs_parent {
      // 1) Strip ".suffix" iteratively to find an ancestor id already in DB.
      let mut parent_id: Option<String> = None;
      let mut upid = id.clone();
      while let Some(dot) = upid.rfind('.') {
        upid.truncate(dot);
        if !upid.is_empty() && self.db.lookup(&format!("ID:{}", upid)).is_some() {
          parent_id = Some(upid.clone());
          break;
        }
      }
      // 2) Fallback to the site root.
      if parent_id.is_none() {
        parent_id = Some(site_id);
      }
      if let Some(pid) = parent_id {
        if pid != id {
          if let Some(entry_mut) = self.db.lookup_mut(&id_key) {
            entry_mut.set_values(vec![("parent", Value::from(pid.as_str()))]);
          }
          self.add_as_child(&id, Some(&pid));
        }
      }
    }

    Info!("scan", "db_status", "Scan: DBStatus: {}", self.db.status());
    Ok(vec![doc])
  }
}

// ======================================================================
// Helpers

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

fn child_tag_nodes(node: &Node) -> Vec<Node> {
  let mut result = Vec::new();
  let mut child = node.get_first_child();
  while let Some(c) = child {
    if c.get_type() == Some(NodeType::ElementNode) && c.get_name() == "tags" {
      let mut tag_child = c.get_first_child();
      while let Some(t) = tag_child {
        if t.get_type() == Some(NodeType::ElementNode) && t.get_name() == "tag" {
          result.push(t.clone());
        }
        tag_child = t.get_next_sibling();
      }
    }
    child = c.get_next_sibling();
  }
  result
}

/// Get xml:id from a node, trying both attribute forms.
fn get_xml_id(node: &Node) -> Option<String> {
  node
    .get_attribute("xml:id")
    .or_else(|| node.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace"))
}

/// Extract text content from a node tree, honoring `open`/`close` attributes on `ltx:tag`.
///
/// Perl uses cloneNode(1) + full DOM rendering, so `<tag close=" ">A</tag>GPT-4o`
/// renders as "A GPT-4o". Our get_content() would give "AGPT-4o".
/// This function inserts the `open`/`close` attribute values around tag elements.
fn title_text_content(node: &Node) -> String {
  let mut result = String::new();
  let mut child = node.get_first_child();
  while let Some(c) = child {
    match c.get_type() {
      Some(NodeType::TextNode) => {
        result.push_str(&c.get_content());
      },
      Some(NodeType::ElementNode) => {
        let name = c.get_name();
        if name == "tag" {
          // Honor open/close attributes on <ltx:tag>
          if let Some(open) = c.get_attribute("open") {
            result.push_str(&open);
          }
          result.push_str(&title_text_content(&c));
          if let Some(close) = c.get_attribute("close") {
            result.push_str(&close);
          }
        } else {
          result.push_str(&title_text_content(&c));
        }
      },
      _ => {},
    }
    child = c.get_next_sibling();
  }
  result
}

// NOTE: Perl uses cloneNode(1) for deep DOM copies. Our libxml bindings only
// provide reference copies via Clone. All Value::Xml uses were converted to
// Value::String (using get_content()) to avoid dangling node references.
// TODO: Add deep clone support to rust-libxml (wrapping xmlCopyNode).
