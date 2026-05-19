//! Cross-reference resolution processor.
//!
//! Port of `LaTeXML::Post::CrossRef` (946 lines of Perl).
//! Resolves cross-references (`ltx:ref`, `ltx:bibref`, etc.) by looking up
//! referenced IDs in the ObjectDB and filling in the reference text,
//! titles, and navigation links.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::document::{get_xml_id, NodeData, PostDocument};
use crate::object_db::{ObjectDB, Value};
use crate::processor::{ProcessResult, Processor};

/// Sectional element types that appear in TOCs.
const NORMAL_TOC_TYPES: &[&str] = &[
  "ltx:document",
  "ltx:part",
  "ltx:chapter",
  "ltx:section",
  "ltx:subsection",
  "ltx:subsubsection",
  "ltx:paragraph",
  "ltx:subparagraph",
  "ltx:index",
  "ltx:bibliography",
  "ltx:glossary",
  "ltx:appendix",
];

/// Fallback fields when a requested ref show key is not found.
fn ref_fallbacks(key: &str) -> &'static [&'static str] {
  match key {
    "typerefnum" => &["refnum"],
    "toctitle" => &["title", "toccaption"],
    "title" => &["toccaption"],
    "rawtoctitle" => &["toctitle", "title", "toccaption"],
    "rawtitle" => &["title", "toccaption"],
    _ => &[],
  }
}

/// URL style for cross-references.
#[derive(Debug, Clone)]
pub enum UrlStyle {
  /// Use file.html#fragment
  File,
  /// Use server-side paths (strip trailing index.ext)
  Server,
  /// Negotiated: strip file extension and trailing index
  Negotiated,
}

/// CrossRef post-processor.
///
/// Port of `LaTeXML::Post::CrossRef`.
pub struct CrossRef {
  name:           String,
  /// Reference to the shared ObjectDB.
  pub db:         ObjectDB,
  /// URL style for cross-references.
  url_style:      UrlStyle,
  /// File extension used for output (e.g. "html", "xml").
  extension:      String,
  /// Default show format for TOC refs.
  toc_show:       String,
  /// Default show format for regular refs.
  ref_show:       String,
  /// Minimum useful content length for refs.
  min_ref_length: usize,
  /// Join string between parent+child ref text.
  ref_join:       String,
  /// Type of navigation TOC to add (e.g. "context").
  navigation_toc: Option<String>,
  /// Track missing references for reporting.
  missing:        HashMap<String, HashMap<String, HashMap<String, u32>>>,
}

impl CrossRef {
  pub fn new(db: ObjectDB, url_style: UrlStyle, number_sections: bool) -> Self {
    CrossRef {
      name: "CrossRef".to_string(),
      db,
      url_style,
      extension: "xml".to_string(),
      toc_show: "toctitle".to_string(),
      ref_show: if number_sections {
        "refnum".to_string()
      } else {
        "title".to_string()
      },
      min_ref_length: 1,
      ref_join: " \u{2023} ".to_string(), // TRIANGULAR BULLET
      navigation_toc: None,
      missing: HashMap::default(),
    }
  }

  /// Set the file extension for URL generation.
  pub fn set_extension(&mut self, ext: &str) { self.extension = ext.to_string(); }

  /// Set the navigation TOC format.
  pub fn set_navigation_toc(&mut self, format: &str) {
    self.navigation_toc = Some(format.to_string());
  }

  /// Note a missing reference.
  fn note_missing(&mut self, severity: &str, ref_type: &str, key: &str) {
    self
      .missing
      .entry(severity.to_string())
      .or_default()
      .entry(ref_type.to_string())
      .or_default()
      .entry(key.to_string())
      .and_modify(|c| *c += 1)
      .or_insert(1);
  }

  /// Generate a URL for a referenced ID.
  ///
  /// Port of `CrossRef::generateURL`.
  fn generate_url(&mut self, doc: &PostDocument, id: &str) -> Option<String> {
    let entry = self.db.lookup(&format!("ID:{}", id))?;
    let location = entry.get_string("location")?;

    let doc_location = doc.site_relative_destination().unwrap_or_default();
    let mut url = relative_url(location, &doc_location);

    // Apply URL style
    match self.url_style {
      UrlStyle::Server => {
        let index_suffix = format!("index.{}", self.extension);
        if url.ends_with(&index_suffix) {
          let prefix = &url[..url.len() - index_suffix.len()];
          url = if prefix.is_empty() {
            "./".to_string()
          } else {
            prefix.to_string()
          };
        }
      },
      UrlStyle::Negotiated => {
        let ext_suffix = format!(".{}", self.extension);
        if url.ends_with(&ext_suffix) {
          url = url[..url.len() - ext_suffix.len()].to_string();
        }
        if url.ends_with("/index") {
          url = url[..url.len() - 5].to_string();
        }
      },
      UrlStyle::File => {},
    }

    if url.is_empty() {
      url = ".".to_string();
    }

    // Add fragment ID
    let fragid = entry.get_string("fragid").map(String::from);
    let loc = location.to_string();
    if let Some(fid) = fragid {
      if url == "." || loc == doc_location {
        url = String::new();
      }
      url = format!("{}#{}", url, fid);
    } else if loc == doc_location {
      url = String::new();
    }

    Some(url)
  }

  /// Generate a title string for a referenced ID, traversing parents for context.
  ///
  /// Port of `CrossRef::generateTitle`.
  fn generate_title(&self, _doc: &PostDocument, id: &str, shown: &str) -> Option<String> {
    let mut current_id = id.to_string();
    let mut result = String::new();
    let mut prefix = String::new();
    let mut shown_so_far = shown.to_string();

    while let Some(entry) = self.db.lookup(&format!("ID:{}", current_id)) {
      let mut pieces = Vec::new();
      let mut is_dup = false;

      // Try title, then typerefnum, then refnum
      if let Some(title_val) = entry.get_value("title") {
        if title_val.is_truthy() {
          is_dup = shown_so_far.contains("title");
          pieces.push(title_val.to_string());
        }
      }
      if pieces.is_empty() {
        let has_type = entry
          .get_value("tag:creftypecap")
          .or_else(|| entry.get_value("tag:creftype"));
        let has_refnum = entry.get_value("refnum");
        if has_type.is_some() && has_refnum.is_some() {
          is_dup = shown_so_far.contains("type") && shown_so_far.contains("refnum");
          if let Some(t) = has_type {
            pieces.push(t.to_string());
          }
          if let Some(r) = has_refnum {
            pieces.push(r.to_string());
          }
        } else if let Some(tr) = entry.get_value("typerefnum") {
          is_dup = shown_so_far.contains("type") && shown_so_far.contains("refnum");
          pieces.push(tr.to_string());
        } else if let Some(r) = has_refnum {
          is_dup = shown_so_far.contains("refnum");
          pieces.push(r.to_string());
        }
      }

      if is_dup {
        prefix = "In ".to_string();
        shown_so_far.clear();
      } else {
        let title = pieces.join(" ");
        let title = title.trim();
        if !title.is_empty() {
          result.push_str(&prefix);
          prefix = self.ref_join.clone();
          result.push_str(title);
        }
      }

      // Walk to parent for more context
      match entry.get_string("parent").map(String::from) {
        Some(pid) => current_id = pid,
        None => break,
      }
    }

    if result.is_empty() {
      None
    } else {
      Some(result)
    }
  }

  /// Generate a title for the document itself.
  ///
  /// Port of `CrossRef::generateDocumentTitle`.
  fn generate_document_title(&self, doc: &PostDocument) -> Option<String> {
    // Try to generate from the document's root ID. Use `get_xml_id` so we
    // pick up ids stored in the xml namespace (Scan's default placement)
    // as well as the bare "xml:id" attribute form.
    if let Some(docid) = doc
      .get_document_element()
      .as_ref()
      .and_then(get_xml_id)
    {
      let title = self.generate_title(doc, &docid, "toctitle");
      if title.as_ref().map(|t| !t.is_empty()).unwrap_or(false) {
        return title;
      }
    }
    // Fallback: look for a title element in the document
    if let Some(node) =
      doc.findnode("//ltx:title | //ltx:toctitle | //ltx:caption | //ltx:toccaption")
    {
      let text = get_text_content_node(&node);
      if !text.is_empty() {
        return Some(text);
      }
    }
    None
  }

  /// Generate content for a glossary reference.
  ///
  /// Port of `CrossRef::generateGlossaryRefTitle`.
  fn generate_glossary_ref_title(&self, entry_key: &str, show: &str) -> Vec<NodeData> {
    let entry = match self.db.lookup(entry_key) {
      Some(e) => e,
      None => return vec![],
    };

    let phrase_key = format!("phrase:{}", show);
    if let Some(val) = entry.get_value(&phrase_key) {
      return vec![NodeData::Element {
        tag:        "ltx:text".to_string(),
        attributes: Some(HashMap::from_iter([(
          "class".to_string(),
          format!("ltx_glossary_{}", show),
        )])),
        children:   vec![NodeData::Text(val.to_string())],
      }];
    }

    // Handle -plural and -indefinite suffixes
    if let Some(base_show) = show.strip_suffix("-plural") {
      let base_key = format!("phrase:{}", base_show);
      if let Some(val) = entry.get_value(&base_key) {
        return vec![NodeData::Element {
          tag:        "ltx:text".to_string(),
          attributes: Some(HashMap::from_iter([(
            "class".to_string(),
            format!("ltx_glossary_{}", show),
          )])),
          children:   vec![NodeData::Text(format!("{}s", val))],
        }];
      }
    }
    if let Some(base_show) = show.strip_suffix("-indefinite") {
      let base_key = format!("phrase:{}", base_show);
      if let Some(val) = entry.get_value(&base_key) {
        let text = val.to_string();
        let article = if text.starts_with(|c: char| "aeiouAEIOU".contains(c)) {
          "an "
        } else {
          "a "
        };
        return vec![NodeData::Element {
          tag:        "ltx:text".to_string(),
          attributes: Some(HashMap::from_iter([(
            "class".to_string(),
            format!("ltx_glossary_{}", show),
          )])),
          children:   vec![NodeData::Text(article.to_string()), NodeData::Text(text)],
        }];
      }
    }

    vec![]
  }

  /// Copy linked resources (non-idref hrefs) to the destination.
  ///
  /// Port of `CrossRef::copy_resources`.
  fn copy_resources(&self, doc: &PostDocument) {
    let refs = doc.findnodes("//ltx:ref[@href and not(@idref) and not(@labelref)]");
    for ref_node in &refs {
      if let Some(url) = ref_node.get_attribute("href") {
        // Only copy relative URLs (no protocol, not absolute)
        if !url.contains("://") && !url.starts_with('/') {
          // Would copy resource from search path to destination
          log::trace!("CrossRef: would copy resource '{}'", url);
        }
      }
    }
  }

  /// Generate reference content for a given ID and show pattern.
  ///
  /// Port of `CrossRef::generateRef`.
  fn generate_ref(&mut self, _doc: &PostDocument, req_id: &str, req_show: &str) -> Vec<NodeData> {
    let show_options = if !req_show.contains("title") {
      vec![req_show.to_string(), "title".to_string()]
    } else {
      vec![req_show.to_string(), "refnum".to_string()]
    };

    for show in &show_options {
      let mut stuff = Vec::new();
      let mut id = req_id.to_string();
      let mut pending = String::new();
      loop {
        let entry_exists = self.db.lookup(&format!("ID:{}", id)).is_some();
        if !entry_exists {
          break;
        }
        let s = self.generate_ref_aux(&id, show);
        if !s.is_empty() {
          if !pending.is_empty() {
            stuff.push(NodeData::Text(pending.clone()));
          }
          stuff.extend(s);
          if self.check_ref_content(&stuff) {
            return stuff;
          }
          pending = self.ref_join.clone();
        }
        let parent = self
          .db
          .lookup(&format!("ID:{}", id))
          .and_then(|e| e.get_string("parent").map(String::from));
        match parent {
          Some(pid) => id = pid,
          None => break,
        }
      }
      if !stuff.is_empty() {
        return stuff;
      }
    }

    self.note_missing("info", "Usable title for ID", req_id);
    vec![NodeData::Text(req_id.to_string())]
  }

  /// Generate ref content from a single DB entry.
  fn generate_ref_aux(&self, id: &str, show: &str) -> Vec<NodeData> {
    let entry = match self.db.lookup(&format!("ID:{}", id)) {
      Some(e) => e,
      None => return vec![],
    };

    let mut stuff = Vec::new();
    let mut ok = false;
    let mut remaining = show.to_string();

    while !remaining.is_empty() {
      if remaining.starts_with(|c: char| c.is_alphanumeric()) {
        let keyword: String = remaining
          .chars()
          .take_while(|c| c.is_alphanumeric())
          .collect();
        remaining = remaining[keyword.len()..].to_string();
        let key = keyword.to_lowercase();
        let class = if key.contains("title") {
          "ltx_ref_title"
        } else {
          "ltx_ref_tag"
        };

        let mut keys_to_try = vec![key.clone(), format!("tag:{}", key)];
        keys_to_try.extend(ref_fallbacks(&key).iter().map(|s| s.to_string()));

        for k in &keys_to_try {
          if let Some(val) = entry.get_value(k) {
            if val.is_truthy() {
              ok = true;
              let text = val.to_string();
              stuff.push(NodeData::Element {
                tag:        "ltx:text".to_string(),
                attributes: Some(HashMap::from_iter([("class".to_string(), class.to_string())])),
                children:   vec![NodeData::Text(text)],
              });
              break;
            }
          }
        }
      } else if remaining.starts_with('{') {
        if let Some(end) = remaining[1..].find('}') {
          let literal = &remaining[1..1 + end];
          if !literal.is_empty() {
            stuff.push(NodeData::Text(literal.to_string()));
          }
          remaining = remaining[2 + end..].to_string();
        } else {
          remaining.clear();
        }
      } else if remaining.starts_with('~') {
        remaining = remaining[1..].to_string();
        if !stuff.is_empty() {
          stuff.push(NodeData::Text("\u{00A0}".to_string()));
        }
      } else if remaining.starts_with(|c: char| c.is_whitespace()) {
        let ws: String = remaining
          .chars()
          .take_while(|c| c.is_whitespace())
          .collect();
        remaining = remaining[ws.len()..].to_string();
        if !stuff.is_empty() {
          stuff.push(NodeData::Text(ws));
        }
      } else {
        let sym: String = remaining
          .chars()
          .take_while(|c| !c.is_alphanumeric() && *c != '{' && *c != '~')
          .collect();
        remaining = remaining[sym.len()..].to_string();
        stuff.push(NodeData::Text(sym));
      }
    }

    if ok { stuff } else { vec![] }
  }

  /// Check if ref content is "good enough".
  fn check_ref_content(&self, stuff: &[NodeData]) -> bool {
    let text = text_content(stuff);
    let cleaned = text.replace("in ", "");
    cleaned.chars().any(|c| c.is_alphanumeric())
  }

  // ======================================================================
  // Fill-in methods

  fn fill_in_relations(&mut self, doc: &mut PostDocument) {
    // Same get_xml_id trick as generate_document_title: Scan stores ids
    // in the xml namespace by default; without this, sub-docs would skip
    // relation filling and never gain the prev/next/up navigation.
    let page_id = match doc.get_document_element().as_ref().and_then(get_xml_id) {
      Some(id) => id,
      None => return,
    };

    // 1. up / "up up" / "up up up" — walk ancestors that have a title.
    let mut current_id = page_id.clone();
    let mut rel = "up".to_string();
    let mut topmost = current_id.clone();
    loop {
      let parent_id = self
        .db
        .lookup(&format!("ID:{}", current_id))
        .and_then(|e| e.get_string("parent").map(String::from));
      match parent_id {
        Some(pid) => {
          let has_title = self
            .db
            .lookup(&format!("ID:{}", pid))
            .and_then(|e| e.get_value("title"))
            .map(|v| v.is_truthy())
            .unwrap_or(false);
          if has_title {
            doc.add_navigation(&rel, &pid);
            rel = format!("{} up", rel);
          }
          current_id = pid.clone();
          topmost = pid;
        },
        None => break,
      }
    }

    // 2. start — the topmost ancestor (root page), if different from us.
    if topmost != page_id {
      if let Some(top_pageid) = self
        .db
        .lookup(&format!("ID:{}", topmost))
        .and_then(|e| e.get_string("pageid").map(String::from))
      {
        doc.add_navigation("start", &top_pageid);
      }
    }

    // 3. prev / next — walk the page tree.
    if let Some(prev) = self.find_previous_page_id(&page_id) {
      doc.add_navigation("prev", &prev);
    }
    if let Some(next) = self.find_next_page_id(&page_id) {
      doc.add_navigation("next", &next);
    }
  }

  /// Return whether the given xml:id is registered as a primary page.
  /// Port of `$entry->getValue('primary')`.
  fn is_primary_page(&self, page_id: &str) -> bool {
    self
      .db
      .lookup(&format!("ID:{}", page_id))
      .and_then(|e| e.get_value("primary"))
      .map(|v| v.is_truthy())
      .unwrap_or(false)
  }

  /// Resolve `entry_id` to the pageid of the page that *contains* its
  /// parent. Port of Perl `CrossRef::getParentPage`.
  fn get_parent_page_id(&self, entry_id: &str) -> Option<String> {
    let entry = self.db.lookup(&format!("ID:{}", entry_id))?;
    let pageid = entry.get_string("pageid")?.to_string();
    let page_entry = self.db.lookup(&format!("ID:{}", pageid))?;
    let parent_id = page_entry.get_string("parent")?.to_string();
    let parent_entry = self.db.lookup(&format!("ID:{}", parent_id))?;
    Some(parent_entry.get_string("pageid")?.to_string())
  }

  /// Recursively collect distinct child page ids under `entry_id`.
  /// Port of Perl `CrossRef::getChildPages`.
  fn get_child_page_ids(&self, entry_id: &str) -> Vec<String> {
    let entry = match self.db.lookup(&format!("ID:{}", entry_id)) {
      Some(e) => e,
      None => return Vec::new(),
    };
    let here_pageid = entry.get_string("pageid").map(String::from);
    let children = entry.get_children();
    let mut out = Vec::new();
    for ch in children {
      let ch_entry = match self.db.lookup(&format!("ID:{}", ch)) {
        Some(e) => e,
        None => continue,
      };
      let ch_pageid = match ch_entry.get_string("pageid") {
        Some(p) => p.to_string(),
        None => continue,
      };
      if here_pageid.as_deref() != Some(&ch_pageid) {
        out.push(ch_pageid);
      } else {
        out.extend(self.get_child_page_ids(&ch));
      }
    }
    out
  }

  /// Page immediately preceding `page_id` in tree order, restricted to
  /// `primary` pages. Port of Perl `CrossRef::findPreviousPage`: previous
  /// sibling if any, drilled into rightmost descendant.
  fn find_previous_page_id(&self, page_id: &str) -> Option<String> {
    let parent_id = self.get_parent_page_id(page_id)?;
    let mut sibs = self.get_child_page_ids(&parent_id);
    // Drop following sibs (rightward) until we hit ourselves.
    while sibs.last().map(|s| s.as_str()) != Some(page_id) {
      sibs.pop()?;
    }
    sibs.pop(); // remove ourselves
    sibs.retain(|s| self.is_primary_page(s));
    let mut current = sibs.pop()?;
    loop {
      let deepest = self
        .get_child_page_ids(&current)
        .into_iter()
        .rev()
        .find(|s| self.is_primary_page(s));
      match deepest {
        Some(deepest) => current = deepest,
        None => break,
      }
    }
    Some(current)
  }

  /// Page immediately following `page_id` in tree order, restricted to
  /// `primary` pages. Port of Perl `CrossRef::findNextPage`: first child,
  /// else walk up to find next sibling at progressively higher levels.
  fn find_next_page_id(&self, page_id: &str) -> Option<String> {
    if let Some(first) = self
      .get_child_page_ids(page_id)
      .into_iter()
      .find(|s| self.is_primary_page(s))
    {
      return Some(first);
    }
    let mut current = page_id.to_string();
    loop {
      let parent = self.get_parent_page_id(&current)?;
      let mut sibs = self.get_child_page_ids(&parent);
      while sibs.first().map(|s| s.as_str()) != Some(&current) {
        if sibs.is_empty() {
          return None;
        }
        sibs.remove(0);
      }
      sibs.remove(0); // drop ourselves
      if let Some(first) = sibs.into_iter().find(|s| self.is_primary_page(s)) {
        return Some(first);
      }
      current = parent;
    }
  }

  fn fill_in_tocs(&mut self, doc: &mut PostDocument) {
    // Perl Post.pm L946-948: Document::findnodes defaults the XPath
    // context to documentElement. oxide's `findnodes(None)` defaults to
    // the XML document node, where libxml2's `descendant::` axis evaluates
    // differently — `descendant::ltx:TOC` matches zero from the doc node
    // even though `//ltx:TOC` matches one. Pin the root explicitly so the
    // user's `\tableofcontents` placeholder is reachable.
    let tocs = match doc.get_document_element() {
      Some(root) => doc.findnodes_at("descendant::ltx:TOC[not(ltx:toclist)]", Some(&root)),
      None => Vec::new(),
    };
    for toc in &tocs {
      // Use the unified get_xml_id helper: Scan's `Document` fallback
      // assigns xml:id via the xml namespace, which is invisible to a
      // bare `get_attribute("xml:id")` lookup but is found by
      // `get_attribute_ns("id", XML_NS)` (which get_xml_id tries first).
      let mut id = doc
        .get_document_element()
        .as_ref()
        .and_then(get_xml_id)
        .unwrap_or_default();
      // `scope="global"` retargets the TOC to the topmost ancestor — used
      // by the persistent sidebar so every split page shows the same tree.
      // Default scope (`current` or absent) keeps the current-page id, so
      // the inline `\tableofcontents` placeholder still produces a
      // page-local TOC.
      if toc.get_attribute("scope").as_deref() == Some("global") {
        let mut root = id.clone();
        loop {
          let parent = self
            .db
            .lookup(&format!("ID:{}", root))
            .and_then(|e| e.get_string("parent").map(String::from));
          match parent {
            Some(p) => root = p,
            None => break,
          }
        }
        id = root;
      }
      let show = toc
        .get_attribute("show")
        .unwrap_or_else(|| self.toc_show.clone());

      let list = self.gen_toc(&id, &show);
      if !list.is_empty() {
        let toclist = NodeData::Element {
          tag:        "ltx:toclist".to_string(),
          attributes: None,
          children:   list,
        };
        let mut toc_mut = toc.clone();
        doc.add_nodes(&mut toc_mut, &[toclist]);
      }
    }
  }

  fn gen_toc(&self, id: &str, show: &str) -> Vec<NodeData> {
    let entry = match self.db.lookup(&format!("ID:{}", id)) {
      Some(e) => e,
      None => return vec![],
    };

    let children = entry.get_children();
    let kids: Vec<NodeData> = children
      .iter()
      .flat_map(|child_id| self.gen_toc(child_id, show))
      .collect();

    let entry_type = entry.get_string("type").unwrap_or("");
    let is_toc_type = NORMAL_TOC_TYPES.contains(&entry_type);
    let in_toc = entry
      .get_value("inlist")
      .map(|v| match v {
        Value::Hash(h) => h.contains_key("toc"),
        _ => false,
      })
      .unwrap_or(false);

    if is_toc_type && in_toc {
      let type_name = entry_type.strip_prefix("ltx:").unwrap_or(entry_type);
      let mut toc_children = vec![NodeData::Element {
        tag:        "ltx:ref".to_string(),
        attributes: Some(HashMap::from_iter([
          ("show".to_string(), show.to_string()),
          ("idref".to_string(), id.to_string()),
        ])),
        children:   vec![],
      }];
      if !kids.is_empty() {
        toc_children.push(NodeData::Element {
          tag:        "ltx:toclist".to_string(),
          attributes: Some(HashMap::from_iter([(
            "class".to_string(),
            format!("ltx_toclist_{}", type_name),
          )])),
          children:   kids,
        });
      }
      vec![NodeData::Element {
        tag:        "ltx:tocentry".to_string(),
        attributes: Some(HashMap::from_iter([(
          "class".to_string(),
          format!("ltx_tocentry_{}", type_name),
        )])),
        children:   toc_children,
      }]
    } else {
      kids
    }
  }

  fn fill_in_frags(&self, doc: &PostDocument) {
    // Invert loop: iterate DB entries (~1K on arXiv:0705.0790) and look up
    // each node via the idcache, instead of iterating every xml:id-bearing
    // node in the DOM (60K+ on math-heavy papers, ~98% of which map to
    // XM* descendants with no DB entry). This avoids the XPath `//*[@xml:id]`
    // result-set clone and 60K hashmap misses per paper.
    //
    // Correctness: fragids are only assigned to nodes that have a DB entry;
    // the old DOM-iteration variant early-exited on `lookup` miss for every
    // non-DB node, so the outputs match exactly.
    for key in self.db.get_keys() {
      let id = match key.strip_prefix("ID:") {
        Some(rest) => rest,
        None => continue,
      };
      let entry = match self.db.lookup(key) {
        Some(e) => e,
        None => continue,
      };
      let fragid = match entry.get_string("fragid") {
        Some(f) => f,
        None => continue,
      };
      if let Some(node) = doc.find_node_by_id(id) {
        let mut n = node.clone();
        n.set_attribute("fragid", fragid).ok();
      }
    }
  }

  fn fill_in_refs(&mut self, doc: &mut PostDocument) {
    let mut refs = doc.findnodes("//*[@idref]");
    refs.extend(doc.findnodes("//*[@labelref]"));
    for ref_node in &refs {
      let tag = doc.get_qname(ref_node).unwrap_or_default();
      if tag == "ltx:XMRef" {
        continue;
      }

      let mut ref_mut = ref_node.clone();
      let mut id = ref_node.get_attribute("idref");
      let show = ref_node
        .get_attribute("show")
        .unwrap_or_else(|| self.ref_show.clone());

      if id.is_none() {
        if let Some(label) = ref_node.get_attribute("labelref") {
          if let Some(entry) = self.db.lookup(&label) {
            if let Some(resolved_id) = entry.get_string("id") {
              ref_mut.set_attribute("idref", resolved_id).ok();
              id = Some(resolved_id.to_string());
            }
          }
          if id.is_none() {
            self.note_missing("warn", "Target for Label", &label);
            PostDocument::add_class(&mut ref_mut, "ltx_missing_label");
          }
        }
      }

      if let Some(ref id_str) = id {
        if ref_mut.get_attribute("href").is_none() {
          if let Some(url) = self.generate_url(doc, id_str) {
            ref_mut.set_attribute("href", &url).ok();
          }
        }
        if ref_mut.get_attribute("title").is_none() {
          if let Some(titlestring) = self.generate_title(doc, id_str, &show) {
            ref_mut.set_attribute("title", &titlestring).ok();
          }
        }
        if ref_mut.get_first_child().is_none() && tag != "ltx:graphics" && tag != "ltx:picture" {
          let content = self.generate_ref(doc, id_str, &show);
          doc.add_nodes(&mut ref_mut, &content);
        }
      }
    }
  }

  fn fill_in_glossaryrefs(&mut self, doc: &mut PostDocument) {
    // Mirrors Perl CrossRef.pm L454-481 fill_in_glossaryrefs:
    //   - resolve `<ltx:glossaryref key=… inlist=…>` against the
    //     GLOSSARY:list:key DB entry registered by Scan + MakeIndex,
    //   - copy the entry's id into `idref` so a later fill_in_refs
    //     pass converts it to `href`,
    //   - copy `phrase:description` into `title` so the XSLT inline
    //     template renders a tooltip,
    //   - fall back to the bare key + `ltx_missing` class when the
    //     entry is not in the DB or has no displayable content.
    for ref_node in &doc.findnodes("descendant::ltx:glossaryref") {
      let mut ref_mut = ref_node.clone();
      let key = ref_node.get_attribute("key").unwrap_or_default();
      let list = ref_node.get_attribute("inlist").unwrap_or_default();

      let gkey = format!("GLOSSARY:{}:{}", list, key);
      if let Some(entry) = self.db.lookup(&gkey) {
        if let Some(id) = entry.get_string("id") {
          ref_mut.set_attribute("idref", id).ok();
        }
        // Perl L465-467: copy phrase:definition (Rust schema uses
        // phrase:description) into `title` if not already set.
        if ref_mut.get_attribute("title").is_none() {
          if let Some(desc) = entry.get_string("phrase:description") {
            if !desc.is_empty() {
              ref_mut.set_attribute("title", desc).ok();
            }
          }
        }
      } else {
        self.note_missing("warn", "Glossary Entry for key", &key);
      }

      if ref_mut.get_first_child().is_none() {
        doc.add_nodes(&mut ref_mut, &[NodeData::Text(key.clone())]);
        PostDocument::add_class(&mut ref_mut, "ltx_missing");
      }
    }
  }

  fn fill_in_bibrefs(&mut self, doc: &mut PostDocument) {
    let bibrefs = doc.findnodes("//ltx:bibref");
    for bibref in &bibrefs {
      let keys_str = bibref.get_attribute("bibrefs").unwrap_or_default();
      let show = bibref
        .get_attribute("show")
        .unwrap_or_else(|| "refnum".to_string());
      // natbib emits show patterns like:
      //   "AuthorsPhrase1Year"           → \citep{X} → "Author (Year)"
      //   "Authors Phrase1YearPhrase2"  → \citet{X} → "Author (Year)"
      //   "refnum"                       → numeric / default
      // Anything containing "Author" or "Year" wants the author-year
      // text built from the bibentry's `authors`/`year` fields; the
      // legacy refnum-only path serves the numeric case.
      let want_authoryear = show.contains("Author") || show.contains("Year");
      let sep = bibref
        .get_attribute("separator")
        .unwrap_or_else(|| ",".to_string());
      let lists_str = bibref
        .get_attribute("inlist")
        .unwrap_or_else(|| "bibliography".to_string());
      let lists: Vec<&str> = lists_str.split_whitespace().collect();

      let mut refs: Vec<NodeData> = Vec::new();
      for key in keys_str.split(',').filter(|k| !k.is_empty()) {
        let mut found_id = None;
        for list in &lists {
          let bkey = format!("BIBLABEL:{}:{}", list, key);
          if let Some(bentry) = self.db.lookup(&bkey) {
            found_id = bentry.get_string("id").map(String::from);
            if found_id.is_some() {
              break;
            }
          }
        }
        if !refs.is_empty() {
          refs.push(NodeData::Text(format!("{} ", sep)));
        }
        if let Some(id) = found_id {
          let mut attrs = HashMap::default();
          attrs.insert("idref".to_string(), id.clone());
          if let Some(url) = self.generate_url(doc, &id) {
            attrs.insert("href".to_string(), url);
          }
          // Build the display text: author-year when natbib's `show`
          // requests it AND the bibentry has the author/year metadata;
          // otherwise fall back to the numeric `number`/`refnum`
          // (matches the legacy path).
          // Perl: use 'number' field for numeric citations (bare number without brackets).
          // The 'refnum' field includes brackets like "[13]", causing double brackets [[13]].
          let entry = self.db.lookup(&format!("ID:{}", id));
          let display = if want_authoryear {
            let authors = entry
              .and_then(|e| {
                e.get_value("authors")
                  .or_else(|| e.get_value("fullauthors"))
                  .or_else(|| e.get_value("keytag"))
              })
              .map(|v| v.to_string())
              .map(|s| s.trim().to_string())
              .filter(|s| !s.is_empty());
            let year = entry
              .and_then(|e| e.get_value("year").or_else(|| e.get_value("typetag")))
              .map(|v| v.to_string())
              .map(|s| s.trim().to_string())
              .filter(|s| !s.is_empty());
            match (authors, year) {
              (Some(a), Some(y)) => {
                // `Phrase1`/`Phrase2` in the show string mark where
                // open/close paren or yyseparator usually go in Perl's
                // bibrefphrase markup. We collapse to a simple
                // "Authors Year" form: `\citep` (AuthorsPhrase1Year)
                // becomes "Author Year" inside the surrounding
                // parens emitted by the citemacro; `\citet`
                // (Authors Phrase1YearPhrase2) becomes "Author Year"
                // with the macro adding the year-parens. Good enough
                // for visual parity with the PDF in the common case.
                format!("{} {}", a, y)
              },
              (Some(a), None) => a,
              (None, Some(y)) => y,
              (None, None) => {
                // No author/year metadata → fall back to refnum.
                entry
                  .and_then(|e| e.get_value("number").or_else(|| e.get_value("refnum")))
                  .map(|v| v.to_string())
                  .unwrap_or_else(|| key.to_string())
              },
            }
          } else {
            entry
              .and_then(|e| e.get_value("number").or_else(|| e.get_value("refnum")))
              .map(|v| v.to_string())
              .unwrap_or_else(|| key.to_string())
          };
          refs.push(NodeData::Element {
            tag:        "ltx:ref".to_string(),
            attributes: Some(attrs),
            children:   vec![NodeData::Text(display)],
          });
        } else {
          self.note_missing("warn", "Entry for citation", key);
          refs.push(NodeData::Element {
            tag:        "ltx:ref".to_string(),
            attributes: Some(HashMap::from_iter([
              ("idref".to_string(), key.to_string()),
              ("class".to_string(), "ltx_missing_citation".to_string()),
            ])),
            children:   vec![NodeData::Text(key.to_string())],
          });
        }
      }
      if !refs.is_empty() {
        doc.replace_node(bibref, &refs);
      }
    }
  }

  fn fill_in_mathlinks(&mut self, doc: &PostDocument) {
    for sym in &doc.findnodes("descendant::*[@decl_id or @meaning]") {
      let tag = doc.get_qname(sym).unwrap_or_default();
      if tag == "ltx:XMRef" || sym.get_attribute("href").is_some() {
        continue;
      }
      let entry_key = sym
        .get_attribute("decl_id")
        .map(|did| format!("DECLARATION:local:{}", did))
        .or_else(|| {
          sym
            .get_attribute("meaning")
            .map(|m| format!("DECLARATION:global:{}", m))
        });
      let parent_id = entry_key
        .as_ref()
        .and_then(|ek| self.db.lookup(ek))
        .and_then(|entry| entry.get_string("parent").map(String::from));
      if let Some(pid) = parent_id {
        if let Some(url) = self.generate_url(doc, &pid) {
          let mut sym_mut = sym.clone();
          sym_mut.set_attribute("href", &url).ok();
        }
      }
    }
  }

  fn report_missing(&self) {
    for (severity, types) in &self.missing {
      for (ref_type, items) in types {
        let keys: Vec<&String> = items.keys().collect();
        let msg = format!(
          "Missing {}: {}",
          ref_type,
          keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(",")
        );
        // Perl CrossRef.pm L72-75: structured Error/Warn/Info with
        // class='expected', object='ids'. Use harness-friendly target.
        match severity.as_str() {
          "error" => log_post_error!("expected", "ids", "{}", msg),
          "warn" => log_post_warn!("expected", "ids", "{}", msg),
          _ => log_post_info!("expected", "ids", "{}", msg),
        }
      }
    }
  }
}

impl Processor for CrossRef {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    match doc.get_document_element() {
      Some(el) => vec![el],
      None => vec![],
    }
  }

  fn process(&mut self, mut doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    self.missing.clear();

    // Generate document title and add navigation
    let doc_title = self.generate_document_title(&doc);
    let navtoc = self.navigation_toc.clone();

    if (navtoc.is_some() || doc_title.is_some()) && doc.findnode("//ltx:navigation").is_none() {
      if let Some(mut root) = doc.get_document_element() {
        doc.add_nodes(&mut root, &[NodeData::Element {
          tag:        "ltx:navigation".to_string(),
          attributes: None,
          children:   vec![],
        }]);
      }
    }
    if let Some(ref format) = navtoc {
      if let Some(mut nav) = doc.findnode("//ltx:navigation") {
        // `scope="global"` so fill_in_tocs walks from the root page,
        // producing the full TOC on every split sub-page (rustdoc-style
        // persistent sidebar).
        doc.add_nodes(&mut nav, &[NodeData::Element {
          tag:        "ltx:TOC".to_string(),
          attributes: Some(HashMap::from_iter([
            ("format".to_string(), format.clone()),
            ("scope".to_string(), "global".to_string()),
          ])),
          children:   vec![],
        }]);
      }
    }
    if let Some(ref title) = doc_title {
      if let Some(mut nav) = doc.findnode("//ltx:navigation") {
        doc.add_nodes(&mut nav, &[NodeData::Element {
          tag:        "ltx:title".to_string(),
          attributes: None,
          children:   vec![NodeData::Text(title.clone())],
        }]);
      }
    }

    self.fill_in_relations(&mut doc);
    self.fill_in_tocs(&mut doc);
    self.fill_in_frags(&doc);
    self.fill_in_glossaryrefs(&mut doc);
    self.fill_in_refs(&mut doc);
    self.fill_in_bibrefs(&mut doc);
    self.fill_in_mathlinks(&doc);
    self.copy_resources(&doc);
    self.report_missing();
    Ok(vec![doc])
  }
}

// ======================================================================
// Helpers

fn relative_url(target: &str, base: &str) -> String {
  if target == base {
    return ".".to_string();
  }
  let target_parts: Vec<&str> = target.split('/').collect();
  let base_parts: Vec<&str> = base.split('/').collect();
  let common = target_parts
    .iter()
    .zip(base_parts.iter())
    .take_while(|(a, b)| a == b)
    .count();
  let mut result = String::new();
  for _ in common..base_parts.len().saturating_sub(1) {
    result.push_str("../");
  }
  result.push_str(&target_parts[common..].join("/"));
  if result.is_empty() {
    ".".to_string()
  } else {
    result
  }
}

/// Get text content from an XML node, normalizing whitespace.
///
/// Port of `getTextContent`.
fn get_text_content_node(node: &Node) -> String {
  let text = node.get_content();
  let trimmed = text.trim();
  // Normalize whitespace
  trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn text_content(nodes: &[NodeData]) -> String {
  nodes
    .iter()
    .map(|n| match n {
      NodeData::Text(s) => s.clone(),
      NodeData::Element { children, .. } => text_content(children),
      NodeData::XmlNode(n) => n.get_content(),
    })
    .collect::<Vec<_>>()
    .join("")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn relative_url_identical_paths_is_dot() {
    assert_eq!(relative_url("a/b.html", "a/b.html"), ".");
  }

  #[test]
  fn relative_url_same_dir() {
    // Both live under a/, so target becomes simply the sibling filename.
    assert_eq!(relative_url("a/other.html", "a/index.html"), "other.html");
  }

  #[test]
  fn relative_url_sibling_dir() {
    // From a/index.html to b/x.html: up once, then down.
    assert_eq!(relative_url("b/x.html", "a/index.html"), "../b/x.html");
  }

  #[test]
  fn relative_url_deeply_nested_base() {
    // Up past each intermediate dir of the base, then into the new path.
    assert_eq!(
      relative_url("top/sibling.html", "top/deep/nested/page.html"),
      "../../sibling.html"
    );
  }

  #[test]
  fn relative_url_same_prefix_different_file() {
    assert_eq!(
      relative_url("a/b/c/target.html", "a/b/c/source.html"),
      "target.html"
    );
  }

  #[test]
  fn ref_fallbacks_typerefnum_goes_to_refnum() {
    assert_eq!(ref_fallbacks("typerefnum"), &["refnum"]);
  }

  #[test]
  fn ref_fallbacks_title_chain() {
    assert_eq!(ref_fallbacks("title"), &["toccaption"]);
    assert_eq!(ref_fallbacks("toctitle"), &["title", "toccaption"]);
    assert_eq!(ref_fallbacks("rawtoctitle"), &[
      "toctitle",
      "title",
      "toccaption"
    ]);
    assert_eq!(ref_fallbacks("rawtitle"), &["title", "toccaption"]);
  }

  #[test]
  fn ref_fallbacks_unknown_key_is_empty() {
    let empty: &[&str] = &[];
    assert_eq!(ref_fallbacks("nonexistent"), empty);
    assert_eq!(ref_fallbacks(""), empty);
  }

  #[test]
  fn text_content_flattens_text() {
    let nodes = vec![
      NodeData::Text("hello ".to_string()),
      NodeData::Text("world".to_string()),
    ];
    assert_eq!(text_content(&nodes), "hello world");
  }

  #[test]
  fn text_content_recurses_into_elements() {
    let nodes = vec![NodeData::Element {
      tag:        "span".to_string(),
      attributes: None,
      children:   vec![
        NodeData::Text("inner ".to_string()),
        NodeData::Text("text".to_string()),
      ],
    }];
    assert_eq!(text_content(&nodes), "inner text");
  }

  #[test]
  fn text_content_empty_list_is_empty_string() {
    assert_eq!(text_content(&[]), "");
  }

  #[test]
  fn text_content_mixed_text_and_nested_element() {
    let nodes = vec![
      NodeData::Text("outer ".to_string()),
      NodeData::Element {
        tag:        "em".to_string(),
        attributes: None,
        children:   vec![NodeData::Text("inner".to_string())],
      },
      NodeData::Text(" tail".to_string()),
    ];
    assert_eq!(text_content(&nodes), "outer inner tail");
  }
}
