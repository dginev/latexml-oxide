//! Document splitting processor.
//!
//! Port of `LaTeXML::Post::Split`.
//! Splits a document into multiple pages based on an XPath expression
//! that identifies section-level elements to extract as separate documents.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  document::{NodeData, PostDocument, get_xml_id},
  processor::{ProcessResult, Processor},
};

/// Page naming strategy for split documents.
#[derive(Debug, Clone)]
pub enum SplitNaming {
  /// Use xml:id attribute
  Id,
  /// Use xml:id, relative to parent
  IdRelative,
  /// Use labels attribute
  Label,
  /// Use labels, relative to parent
  LabelRelative,
}

/// A tree node used to track the hierarchy of pages during splitting.
struct PageEntry {
  node:     Node,
  id:       Option<String>,
  upid:     Option<String>,
  name:     String,
  children: Vec<PageEntry>,
  document: Option<PostDocument>,
}

/// Split post-processor: splits a document into multiple pages.
///
/// Port of `LaTeXML::Post::Split`.
pub struct Split {
  name:          String,
  /// XPath expression to find elements that become pages.
  split_xpath:   String,
  /// Whether to suppress navigation links.
  no_navigation: bool,
  /// Names the pages: Perl's `splitnaming` and `unnamed_page_counter`.
  namer:         PageNamer,
}

impl Split {
  pub fn new(split_xpath: &str, split_naming: SplitNaming, no_navigation: bool) -> Self {
    Split {
      name: "Split".to_string(),
      split_xpath: split_xpath.to_string(),
      no_navigation,
      namer: PageNamer::new(split_naming),
    }
  }

  /// Get the nodes that will become separate pages, via the limit-safe
  /// [`PostDocument::find_split_pages`] walk (not XPath, which silently splits
  /// nothing on very large documents). A hidden element (`ltx_nodisplay`, such as the bibliography only a
  /// `\fullcite` reads) stays in its parent's page: a page of its own would show nothing.
  fn get_pages(&self, doc: &PostDocument) -> Vec<Node> {
    doc
      .find_split_pages(&self.split_xpath)
      .into_iter()
      .filter(|page| {
        !page
          .get_attribute("class")
          .is_some_and(|class| class.split_whitespace().any(|c| c == "ltx_nodisplay"))
      })
      .collect()
  }

  /// Sort pages into a tree hierarchy.
  ///
  /// Port of Perl `presortPages`.
  /// If a page is a descendant of another page, it becomes a child in the tree.
  fn presort_pages(
    tree: &mut PageEntry,
    haschildren: &mut HashMap<String, bool>,
    pages: Vec<Node>,
  ) {
    // We track the "current" position in the tree by maintaining a path of ancestors.
    // Since we can't have mutable borrows at multiple tree levels simultaneously,
    // we use an index-based approach to walk the tree.
    let mut path: Vec<usize> = Vec::new(); // indices into children arrays

    for page in pages {
      // Walk back up the tree until we find an ancestor of `page`
      loop {
        let current_node = Self::get_node_at(tree, &path);
        if is_child(&page, &current_node) {
          break;
        }
        if path.is_empty() {
          break;
        }
        path.pop();
      }

      let current_node = Self::get_node_at(tree, &path);
      let current_id = get_xml_id(&current_node);
      let localname = current_node.get_name();
      haschildren.insert(localname, true);

      let page_id = get_xml_id(&page);
      let entry = PageEntry {
        node:     page,
        id:       page_id,
        upid:     current_id,
        name:     String::new(),
        children: Vec::new(),
        document: None,
      };

      // Add as child of current position
      let parent = Self::get_entry_at_mut(tree, &path);
      parent.children.push(entry);
      let new_idx = parent.children.len() - 1;

      // Go "down" — following pages may be children of this one
      path.push(new_idx);
    }
  }

  /// Get the node at a given path in the tree.
  fn get_node_at(tree: &PageEntry, path: &[usize]) -> Node {
    let mut current = tree;
    for &idx in path {
      current = &current.children[idx];
    }
    current.node.clone()
  }

  /// Get a mutable reference to the entry at a given path.
  fn get_entry_at_mut<'a>(tree: &'a mut PageEntry, path: &[usize]) -> &'a mut PageEntry {
    let mut current = tree;
    for &idx in path {
      current = &mut current.children[idx];
    }
    current
  }

  /// Compute destination pathnames for each page in the tree.
  ///
  /// Port of Perl `prenamePages`.
  fn prename_pages(
    &mut self,
    doc: &PostDocument,
    tree: &mut PageEntry,
    haschildren: &HashMap<String, bool>,
  ) {
    for i in 0..tree.children.len() {
      let (parent_name, parent_node) = (tree.name.clone(), tree.node.clone());
      let child = &tree.children[i];
      let child_localname = child.node.get_name();
      let recursive = haschildren.get(&child_localname).copied().unwrap_or(false);
      let name = self.get_page_name(doc, &child.node, &parent_node, &parent_name, recursive);
      tree.children[i].name = name;
    }
    // Recurse into children
    for child in &mut tree.children {
      self.prename_pages(doc, child, haschildren);
    }
  }

  /// Process a sequence of page entries, removing them from the document
  /// and generating sub-documents for each.
  ///
  /// Port of Perl `processPages`.
  fn process_pages(
    &mut self,
    doc: &mut PostDocument,
    entries: &mut Vec<PageEntry>,
    nav_holders: &[Node],
  ) -> Vec<PostDocument> {
    // Before any document surgery, copy inheritable attributes.
    let mut intoc = false;
    for entry in entries.iter() {
      let node = &entry.node;
      if let Some(inlist) = node.get_attribute("inlist") {
        if inlist.contains("toc") {
          intoc = true;
        }
      }
      // Copy xml:lang and backgroundcolor from ancestors (Perl Split.pm's
      // inheritable-attribute loop). `get_attribute("xml:lang")` alone can
      // miss the namespaced form (`xml:` attributes live in the XML
      // namespace — the same trap `get_xml_id` guards), which silently
      // skipped the copy: the XPath found the ancestor, the read returned
      // None, and split pages lost their inherited language. Caught by the
      // streaming-split parity gate (118_streaming_split_parity).
      for attr in &["xml:lang", "backgroundcolor"] {
        let xpath = format!("ancestor-or-self::*[@{}][1]", attr);
        if let Some(anc) = doc.findnode_at(&xpath, node) {
          let val = anc.get_attribute(attr).or_else(|| {
            attr
              .strip_prefix("xml:")
              .and_then(|local| anc.get_attribute_ns(local, "http://www.w3.org/XML/1998/namespace"))
          });
          if let Some(val) = val {
            let mut node_mut = node.clone();
            node_mut.set_attribute(attr, &val).ok();
          }
        }
      }
    }

    let mut docs = Vec::new();
    while !entries.is_empty() {
      let parent = match entries[0].node.get_parent() {
        Some(p) => p,
        None => {
          entries.remove(0);
          continue;
        },
      };

      // Remove page & ALL following siblings (backwards).
      let mut removed: Vec<Node> = Vec::new();
      loop {
        let last = parent.get_last_child();
        match last {
          Some(mut sib) => {
            sib.unlink_node();
            removed.insert(0, sib.clone());
            if sib == entries[0].node {
              break;
            }
          },
          None => break,
        }
      }

      // Build TOC from adjacent nodes being extracted.
      let mut toc: Vec<NodeData> = Vec::new();

      // Process a sequence of adjacent pages that share the same parent.
      while !entries.is_empty() && !removed.is_empty() && entries[0].node == removed[0] {
        let mut entry = entries.remove(0);
        let page = entry.node.clone();

        // If any pages go in toc, assume siblings should too
        if intoc && !page.has_attribute("inlist") {
          let mut page_mut = page.clone();
          page_mut.set_attribute("inlist", "toc").ok();
        }

        // Remove this page from the removed list and from the document's ID cache
        let removed_node = removed.remove(0);
        doc.remove_nodes(&[removed_node]);

        // Build TOC entry
        if let Some(id) = get_xml_id(&page) {
          let mut toc_attrs = HashMap::default();
          toc_attrs.insert("idref".to_string(), id);
          toc_attrs.insert("show".to_string(), "toctitle".to_string());
          let tocentry = NodeData::Element {
            tag:        "ltx:tocentry".to_string(),
            attributes: None,
            children:   vec![NodeData::Element {
              tag:        "ltx:ref".to_string(),
              attributes: Some(toc_attrs),
              children:   vec![],
            }],
          };
          toc.push(tocentry);
        }

        // Process children pages BEFORE this page (Perl: "Due to the way document building works")
        let mut child_docs = self.process_pages(doc, &mut entry.children, nav_holders);

        // Create sub-document from the extracted page
        let subdoc = doc.new_document(page.clone(), &entry.name, entry.upid.clone());
        // The page now lives in its copy; free the source subtree (Perl's GC does at
        // the last reference). Left unlinked, every page leaked: the whole source DOM
        // stayed live through post, 818 MB of C heap by page 1024 of a 1,600-subsection
        // book. A page holding a navigation stays: the nav may use its namespaces.
        if !nav_holders.contains(&page) {
          doc.discard_subtree(page);
        }
        entry.document = Some(subdoc);
        // Take the document out to push to the results
        docs.push(entry.document.take().unwrap());
        docs.append(&mut child_docs);
      }

      // Add TOC to reflect the extracted pages
      if !toc.is_empty() {
        // Only add if parent doesn't already have a TOC with lists='toc'
        let has_toc = !doc
          .findnodes_at("descendant::ltx:TOC[@lists='toc']", Some(&parent))
          .is_empty();
        if !has_toc {
          let parent_type = parent.get_name();
          let mut toclist_attrs = HashMap::default();
          toclist_attrs.insert("class".to_string(), format!("ltx_toclist_{}", parent_type));
          let toc_node = NodeData::Element {
            tag:        "ltx:TOC".to_string(),
            attributes: None,
            children:   vec![NodeData::Element {
              tag:        "ltx:toclist".to_string(),
              attributes: Some(toclist_attrs),
              children:   toc,
            }],
          };
          let mut parent_mut = parent.clone();
          doc.add_nodes(&mut parent_mut, &[toc_node]);
        }
      }

      // Re-add remaining siblings
      let mut parent_mut = parent;
      for mut child in removed {
        parent_mut.add_child(&mut child).ok();
      }
    }
    docs
  }

  /// Add navigation elements to all documents in the tree.
  ///
  /// Port of Perl `addNavigation`.
  fn add_navigation(entry: &mut PageEntry, nav_nodes: &[Node]) {
    if let Some(ref mut doc) = entry.document {
      if let Some(mut root) = doc.get_document_element() {
        let nav_data: Vec<NodeData> = nav_nodes
          .iter()
          .map(|n| NodeData::XmlNode(n.clone()))
          .collect();
        doc.add_nodes(&mut root, &nav_data);
      }
    }
    for child in &mut entry.children {
      Self::add_navigation(child, nav_nodes);
    }
  }

  /// Compute the destination pathname for a page.
  ///
  /// Port of `Split::getPageName`, through [`PageNamer`].
  fn get_page_name(
    &mut self,
    doc: &PostDocument,
    page: &Node,
    parent: &Node,
    parent_path: &str,
    recursive: bool,
  ) -> String {
    let (labels, id) = (page.get_attribute("labels"), get_xml_id(page));
    let (parent_labels, parent_id) = (parent.get_attribute("labels"), get_xml_id(parent));
    let ext = doc
      .get_destination_extension()
      .unwrap_or_else(|| "xml".to_string());
    self.namer.page_name(
      PageKeys {
        labels: labels.as_deref(),
        id:     id.as_deref(),
      },
      PageKeys {
        labels: parent_labels.as_deref(),
        id:     parent_id.as_deref(),
      },
      parent_path,
      &ext,
      recursive,
    )
  }
}

/// A page's `labels` and `xml:id`, the keys `Split::getPageName` names a page and its parent by.
#[derive(Clone, Copy)]
pub(crate) struct PageKeys<'a> {
  pub(crate) labels: Option<&'a str>,
  pub(crate) id:     Option<&'a str>,
}

impl PageKeys<'_> {
  /// The first key of `attr` less `LABEL:` (`Split.pm` L164-166, L177-179).
  fn key(&self, attr: &str) -> &str {
    let value = if attr == "labels" {
      self.labels
    } else {
      self.id
    };
    let first = value
      .and_then(|v| v.split_whitespace().next())
      .unwrap_or("");
    first.strip_prefix("LABEL:").unwrap_or(first)
  }
}

/// Why a page cannot have the pathname its key gives.
enum Refusal {
  /// Outside the destination's directory.
  Outside,
  /// Issued to another page (`None`: the document's own).
  Taken(Option<String>),
}

/// Perl `Split::getPageName` (`Split.pm` L158-191) for both split paths, and the pathnames one split has issued.
pub(crate) struct PageNamer {
  /// Perl's `splitnaming`.
  naming:                SplitNaming,
  /// Perl's `unnamed_page_counter`, which no split resets.
  unnamed_page_counter:  u32,
  /// The destination's directory, which every page lies in.
  destination_directory: String,
  /// Each pathname issued in this split, absolute, with what named it (`None`: the document's own page).
  issued:                HashMap<String, Option<String>>,
}

impl PageNamer {
  pub(crate) fn new(naming: SplitNaming) -> Self {
    PageNamer {
      naming,
      unnamed_page_counter: 0,
      destination_directory: String::new(),
      issued: HashMap::default(),
    }
  }

  /// Start the split of a document written to `destination`.
  pub(crate) fn start(&mut self, destination: &str) {
    self.destination_directory = latexml_core::util::pathname::directory(destination);
    self.issued.clear();
    self
      .issued
      .insert(latexml_core::util::pathname::absolute(destination), None);
  }

  /// Perl `generateUnnamedPageName`.
  fn generate_unnamed_page_name(&mut self) -> String {
    self.unnamed_page_counter += 1;
    format!("FOO{}", self.unnamed_page_counter)
  }

  /// A page with no usable `attr` key takes its `xml:id`, and is then named relative to its parent's `xml:id`
  /// (`Split.pm` L167-171); without one, a generated name.
  fn fallback_name(&mut self, page: PageKeys, attr: &'static str) -> (&'static str, String) {
    match page.id.filter(|_| attr == "labels") {
      Some(id) => ("xml:id", id.to_string()),
      None => (attr, self.generate_unnamed_page_name()),
    }
  }

  /// The pathname for `page`, a child of `parent` written to `parent_path`. A page that would leave the destination's
  /// directory (an Error, as Perl's site check `Post.pm` L700-704) or take an issued pathname (a Warning) takes its
  /// fallback name instead (OD #1164).
  pub(crate) fn page_name(
    &mut self,
    page: PageKeys,
    parent: PageKeys,
    parent_path: &str,
    ext: &str,
    recursive: bool,
  ) -> String {
    use latexml_core::util::pathname;
    let mut attr = match self.naming {
      SplitNaming::Id | SplitNaming::IdRelative => "xml:id",
      SplitNaming::Label | SplitNaming::LabelRelative => "labels",
    };
    let mut name = page_name_segments(page.key(attr));
    if name.is_empty() {
      let missing = attr;
      (attr, name) = self.fallback_name(page, attr);
      Info!(
        "split",
        "pathname",
        "Using '{}' to create page pathname, instead of missing '{}'",
        name,
        missing
      );
    }
    // The first pathname refused, the key that named it, and why.
    let mut refused: Option<(String, String, Refusal)> = None;
    loop {
      let pathname = self.pathname(attr, &name, parent, parent_path, ext, recursive);
      let key = pathname::absolute(&pathname);
      let by = match page.key(attr) {
        "" => format!("'{name}'"),
        key if attr == "labels" => format!("label '{key}'"),
        key => format!("id '{key}'"),
      };
      let refusal = if name.split('/').next() == Some("..")
        && pathname::is_contained(&pathname::directory(&key), &self.destination_directory).is_none()
      {
        Some(Refusal::Outside)
      } else {
        self.issued.get(&key).cloned().map(Refusal::Taken)
      };
      if let Some(refusal) = refusal {
        refused.get_or_insert((pathname, by, refusal));
        (attr, name) = self.fallback_name(page, attr);
        continue;
      }
      self.issued.insert(key, Some(by));
      match refused {
        Some((refused, by, Refusal::Outside)) => Error!(
          "unexpected",
          &refused,
          "The page of {} ({}) must be within the destination directory ({}); it is written to {} instead",
          by,
          refused,
          self.destination_directory,
          pathname
        ),
        Some((refused, by, Refusal::Taken(owner))) => Warn!(
          "unexpected",
          &refused,
          "The page of {} would overwrite {} ({}); it is written to {} instead",
          by,
          owner.map_or_else(
            || "the document's own page".to_string(),
            |owner| format!("the page of {owner}")
          ),
          refused,
          pathname
        ),
        None => {},
      }
      return pathname;
    }
  }

  /// The pathname of a page named `name` by `attr` (`Split.pm` L176-191).
  fn pathname(
    &self,
    attr: &str,
    name: &str,
    parent: PageKeys,
    parent_path: &str,
    ext: &str,
    recursive: bool,
  ) -> String {
    let mut name = name.to_string();
    let as_dir = match self.naming {
      SplitNaming::IdRelative | SplitNaming::LabelRelative => {
        if let Some(rest) = strip_parent_name(&name, &page_name_segments(parent.key(attr))) {
          name = rest;
        }
        recursive
      },
      _ => false,
    };
    page_pathname(parent_path, &colons_to_underscore(&name), ext, as_dir)
  }
}

/// A label as a page name: its path segments as Perl's `pathname_canonical` leaves them (`Pathname.pm` L126-128) — no
/// empty or `.` segment, a `name/..` pair collapsed (`./x`, `a//b`, `a/../b` are `x`, `a/b`, `b`), a leading `..` kept —
/// but with no `~` expansion and no root (OD #1164). A name of separators alone is empty, so it takes the unnamed page's
/// fallback (`Split.pm` L167-175). `\` separates only on Windows.
pub(crate) fn page_name_segments(name: &str) -> String {
  let mut segments: Vec<&str> = Vec::new();
  for segment in name.split(std::path::is_separator) {
    match segment {
      "" | "." => {},
      ".." if segments.last().is_some_and(|last| *last != "..") => {
        segments.pop();
      },
      _ => segments.push(segment),
    }
  }
  segments.join("/")
}

/// A relative page name: the name less its parent's and at least one `.`, `_` or `:` after it, Perl's
/// `/^\Q$pname\E(\.|_|:)+(.*)$/` (`Split.pm` L180-181), as [`page_name_segments`]; `None` keeps the whole name.
pub(crate) fn strip_parent_name(name: &str, parent: &str) -> Option<String> {
  let rest = name
    .strip_prefix(parent)
    .filter(|rest| !parent.is_empty() && rest.starts_with(['.', '_', ':']))?;
  Some(page_name_segments(rest.trim_start_matches(['.', '_', ':']))).filter(|rest| !rest.is_empty())
}

/// Every run of `:` in a page name as one `_`, Perl's `s/:+/_/g` (`Split.pm` L183).
pub(crate) fn colons_to_underscore(name: &str) -> String {
  let mut out = String::with_capacity(name.len());
  for (i, c) in name.char_indices() {
    if c != ':' {
      out.push(c);
    } else if !name[..i].ends_with(':') {
      out.push('_');
    }
  }
  out
}

/// A page's pathname beside its parent's, or (`as_dir`) as `index` in a folder of its name: Perl `Split::getPageName`'s
/// `pathname_make` (`Split.pm` L185-191) over a name of [`page_name_segments`] (issue 1006, OD #1164).
pub(crate) fn page_pathname(parent_path: &str, name: &str, ext: &str, as_dir: bool) -> String {
  debug_assert!(
    !name.is_empty() && !name.starts_with(std::path::is_separator),
    "page name {name:?}"
  );
  let file = if as_dir {
    format!("{name}/index.{ext}")
  } else {
    format!("{name}.{ext}")
  };
  // The parent's directory as written: the parent is the canonical destination or a page named here, and
  // canonicalizing it again would expand a label's `~`.
  match std::path::Path::new(parent_path)
    .parent()
    .and_then(|dir| dir.to_str())
  {
    Some(dir) if !dir.is_empty() => format!("{dir}/{file}"),
    _ => file,
  }
}

impl Processor for Split {
  fn get_name(&self) -> &str { &self.name }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let root = match nodes.into_iter().next() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

    // Ensure root has an ID (Writer will remove TEMPORARY_DOCUMENT_ID)
    let mut root_mut = root;
    if get_xml_id(&root_mut).is_none() {
      root_mut
        .set_attribute("xml:id", "TEMPORARY_DOCUMENT_ID")
        .ok();
    }

    let pages = self.get_pages(&doc);
    // Filter out the root node itself (Perl: grep { $_->parentNode->parentNode })
    let pages: Vec<Node> = pages
      .into_iter()
      .filter(|p| p.get_parent().and_then(|pp| pp.get_parent()).is_some())
      .collect();

    if pages.is_empty() {
      Info!("split", "result", "[not split]");
      return Ok(vec![doc]);
    }

    // Save and remove navigation elements
    let nav_nodes: Vec<Node> = doc.findnodes("descendant::ltx:navigation");
    // The navs are copied back after the split, so their ancestors are never freed:
    // a namespace a nav uses may be declared on (or inside) a page.
    let nav_holders: Vec<Node> = nav_nodes
      .iter()
      .flat_map(|nav| std::iter::successors(nav.get_parent(), Node::get_parent))
      .collect();
    if !nav_nodes.is_empty() {
      doc.remove_nodes(&nav_nodes);
    }

    // Build the page tree
    let root_id = get_xml_id(&root_mut);
    let root_dest = doc.get_destination().unwrap_or("").to_string();
    let mut tree = PageEntry {
      node:     root_mut,
      id:       root_id,
      upid:     None,
      name:     root_dest,
      children: Vec::new(),
      document: Some(doc),
    };

    let mut haschildren = HashMap::default();
    Self::presort_pages(&mut tree, &mut haschildren, pages);
    self.namer.start(&tree.name);

    // Take doc out so we can pass &PostDocument and &mut tree without borrow conflict
    let doc_tmp = tree.document.take().unwrap();
    self.prename_pages(&doc_tmp, &mut tree, &haschildren);
    tree.document = Some(doc_tmp);

    // Process pages: extract and create sub-documents
    let mut doc = tree.document.take().unwrap();
    let mut docs = vec![];
    let mut child_docs = self.process_pages(&mut doc, &mut tree.children, &nav_holders);

    // Restore navigation to all documents
    if !nav_nodes.is_empty() && !self.no_navigation {
      // Put doc back into tree for navigation distribution
      tree.document = Some(doc);
      Self::add_navigation(&mut tree, &nav_nodes);
      doc = tree.document.take().unwrap();

      // Also add nav to child docs
      for child_doc in &mut child_docs {
        if let Some(mut root) = child_doc.get_document_element() {
          let nav_data: Vec<NodeData> = nav_nodes
            .iter()
            .map(|n| NodeData::XmlNode(n.clone()))
            .collect();
          child_doc.add_nodes(&mut root, &nav_data);
        }
      }
    }

    docs.insert(0, doc);
    docs.append(&mut child_docs);

    let n = docs.len();
    Info!(
      "split",
      "result",
      "{}",
      if n > 1 {
        format!(" [Split into {} pages]", n)
      } else {
        "[not split]".to_string()
      }
    );

    Ok(docs)
  }
}

/// Check if `child` is a descendant of `ancestor`.
fn is_child(child: &Node, ancestor: &Node) -> bool {
  let mut parent = child.get_parent();
  while let Some(ref p) = parent {
    if *p == *ancestor {
      return true;
    }
    parent = p.get_parent();
  }
  false
}

#[cfg(test)]
mod page_name_tests {
  use super::*;

  /// A label becomes a page name as Perl's `Split::getPageName` and `pathname_canonical` make it (`Split.pm` L180-183,
  /// `Pathname.pm` L126-128), less the `~` expansion and the root (OD #1164).
  #[test]
  fn page_names_take_perls_segments_strip_and_colons() {
    for (label, name) in [
      ("/x", "x"),
      ("./x", "x"),
      ("a//b/", "a/b"),
      ("a/./b", "a/b"),
      ("a/../b", "b"),
      ("a/../..", ".."),
      ("../esc", "../esc"),
      ("../../x", "../../x"),
      ("a/..", ""),
      ("/", ""),
      ("~one", "~one"),
    ] {
      assert_eq!(page_name_segments(label), name, "{label}");
    }
    assert_eq!(
      page_name_segments("a\\b"),
      if cfg!(windows) { "a/b" } else { "a\\b" }
    );
    assert_eq!(strip_parent_name("section", "sec"), None);
    assert_eq!(
      strip_parent_name("sec:./dot", "sec").as_deref(),
      Some("dot")
    );
    assert_eq!(strip_parent_name("sec._x", "sec").as_deref(), Some("x"));
    assert_eq!(strip_parent_name("sec:", "sec"), None);
    assert_eq!(strip_parent_name("x", ""), None);
    assert_eq!(colons_to_underscore("sec:::1:a"), "sec_1_a");
  }
}
