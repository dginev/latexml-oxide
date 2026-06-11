//! Document splitting processor.
//!
//! Port of `LaTeXML::Post::Split`.
//! Splits a document into multiple pages based on an XPath expression
//! that identifies section-level elements to extract as separate documents.

use std::path::Path;

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
  name:                 String,
  /// XPath expression to find elements that become pages.
  split_xpath:          String,
  /// Naming strategy for page files.
  split_naming:         SplitNaming,
  /// Whether to suppress navigation links.
  no_navigation:        bool,
  /// Counter for unnamed pages.
  unnamed_page_counter: u32,
}

impl Split {
  pub fn new(split_xpath: &str, split_naming: SplitNaming, no_navigation: bool) -> Self {
    Split {
      name: "Split".to_string(),
      split_xpath: split_xpath.to_string(),
      split_naming,
      no_navigation,
      unnamed_page_counter: 0,
    }
  }

  /// Get the nodes that will become separate pages.
  fn get_pages(&self, doc: &PostDocument) -> Vec<Node> { doc.findnodes(&self.split_xpath) }

  /// Generate a name for an unnamed page.
  fn generate_unnamed_page_name(&mut self) -> String {
    self.unnamed_page_counter += 1;
    format!("FOO{}", self.unnamed_page_counter)
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
      // Copy xml:lang and backgroundcolor from ancestors
      for attr in &["xml:lang", "backgroundcolor"] {
        let xpath = format!("ancestor-or-self::*[@{}][1]", attr);
        if let Some(anc) = doc.findnode_at(&xpath, node) {
          if let Some(val) = anc.get_attribute(attr) {
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
        if intoc && page.get_attribute("inlist").is_none() {
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
        let mut child_docs = self.process_pages(doc, &mut entry.children);

        // Create sub-document from the extracted page
        let subdoc = doc.new_document(page, &entry.name);
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
  /// Port of `Split::getPageName`.
  fn get_page_name(
    &mut self,
    doc: &PostDocument,
    page: &Node,
    parent: &Node,
    parent_path: &str,
    recursive: bool,
  ) -> String {
    let attr = match self.split_naming {
      SplitNaming::Id | SplitNaming::IdRelative => "xml:id",
      SplitNaming::Label | SplitNaming::LabelRelative => "labels",
    };

    let mut name = if attr == "xml:id" {
      get_xml_id(page).unwrap_or_default()
    } else {
      page.get_attribute(attr).unwrap_or_default()
    };

    // Truncate to first label, strip LABEL: prefix
    if let Some(first) = name.split_whitespace().next() {
      name = first.to_string();
    }
    if let Some(stripped) = name.strip_prefix("LABEL:") {
      name = stripped.to_string();
    }

    if name.is_empty() {
      if attr == "labels" {
        if let Some(id) = get_xml_id(page) {
          Info!(
            "split",
            "pathname",
            "Using '{}' to create page pathname, instead of missing '{}'",
            id,
            attr
          );
          name = id;
        } else {
          name = self.generate_unnamed_page_name();
          Info!(
            "split",
            "pathname",
            "Using '{}' to create page pathname, instead of missing '{}'",
            name,
            attr
          );
        }
      } else {
        name = self.generate_unnamed_page_name();
        Info!(
          "split",
          "pathname",
          "Using '{}' to create page pathname, instead of missing '{}'",
          name,
          attr
        );
      }
    }

    // Relative naming: strip parent prefix
    let as_dir = match self.split_naming {
      SplitNaming::IdRelative | SplitNaming::LabelRelative => {
        let parent_attr = if attr == "xml:id" {
          get_xml_id(parent)
        } else {
          parent.get_attribute(attr)
        };
        if let Some(pname) = parent_attr {
          let pname = pname.split_whitespace().next().unwrap_or("");
          let pname = pname.strip_prefix("LABEL:").unwrap_or(pname);
          if let Some(rest) = name.strip_prefix(pname) {
            let rest = rest.trim_start_matches(['.', '_', ':']);
            if !rest.is_empty() {
              name = rest.to_string();
            }
          }
        }
        recursive
      },
      _ => false,
    };

    // Sanitize colons
    name = name.replace(':', "_");

    let ext = doc
      .get_destination_extension()
      .unwrap_or_else(|| "xml".to_string());
    let parent_dir = Path::new(parent_path)
      .parent()
      .and_then(|p| p.to_str())
      .unwrap_or(".");

    // Normalize empty parent_dir to "."
    let parent_dir = if parent_dir.is_empty() {
      "."
    } else {
      parent_dir
    };

    if as_dir {
      format!("{}/{}/index.{}", parent_dir, name, ext)
    } else {
      format!("{}/{}.{}", parent_dir, name, ext)
    }
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

    // Take doc out so we can pass &PostDocument and &mut tree without borrow conflict
    let doc_tmp = tree.document.take().unwrap();
    self.prename_pages(&doc_tmp, &mut tree, &haschildren);
    tree.document = Some(doc_tmp);

    // Process pages: extract and create sub-documents
    let mut doc = tree.document.take().unwrap();
    let mut docs = vec![];
    let mut child_docs = self.process_pages(&mut doc, &mut tree.children);

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
