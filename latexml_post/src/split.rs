//! Document splitting processor.
//!
//! Port of `LaTeXML::Post::Split`.
//! Splits a document into multiple pages based on an XPath expression
//! that identifies section-level elements to extract as separate documents.

use libxml::tree::Node;
use std::path::Path;

use crate::document::PostDocument;
use crate::processor::{ProcessResult, Processor};

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

/// Split post-processor: splits a document into multiple pages.
///
/// Port of `LaTeXML::Post::Split`.
pub struct Split {
  name: String,
  /// XPath expression to find elements that become pages.
  split_xpath: String,
  /// Naming strategy for page files.
  split_naming: SplitNaming,
  /// Whether to suppress navigation links.
  no_navigation: bool,
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
  fn get_pages(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes(&self.split_xpath)
  }

  /// Generate a name for an unnamed page.
  fn generate_unnamed_page_name(&mut self) -> String {
    self.unnamed_page_counter += 1;
    format!("FOO{}", self.unnamed_page_counter)
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

    let mut name = page.get_attribute(attr).unwrap_or_default();

    // Truncate to first label, strip LABEL: prefix
    if let Some(first) = name.split_whitespace().next() {
      name = first.to_string();
    }
    if let Some(stripped) = name.strip_prefix("LABEL:") {
      name = stripped.to_string();
    }

    if name.is_empty() {
      if attr == "labels" {
        if let Some(id) = page.get_attribute("xml:id") {
          name = id;
        } else {
          name = self.generate_unnamed_page_name();
        }
      } else {
        name = self.generate_unnamed_page_name();
      }
    }

    // Relative naming: strip parent prefix
    let as_dir = match self.split_naming {
      SplitNaming::IdRelative | SplitNaming::LabelRelative => {
        if let Some(pname) = parent.get_attribute(attr) {
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
      }
      _ => false,
    };

    // Sanitize colons
    name = name.replace(':', "_");

    let ext = doc.get_destination_extension().unwrap_or_else(|| "xml".to_string());
    let parent_dir = Path::new(parent_path)
      .parent()
      .and_then(|p| p.to_str())
      .unwrap_or(".");

    if as_dir {
      format!("{}/{}/index.{}", parent_dir, name, ext)
    } else {
      format!("{}/{}.{}", parent_dir, name, ext)
    }
  }
}

impl Processor for Split {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let root = match nodes.into_iter().next() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

    // Ensure root has an ID (Writer will remove TEMPORARY_DOCUMENT_ID)
    let mut root_mut = root.clone();
    if root_mut.get_attribute("xml:id").is_none() {
      root_mut.set_attribute("xml:id", "TEMPORARY_DOCUMENT_ID").ok();
    }

    let pages = self.get_pages(&doc);
    // Filter out the root node itself
    let pages: Vec<Node> = pages
      .into_iter()
      .filter(|p| {
        p.get_parent()
          .and_then(|pp| pp.get_parent())
          .is_some()
      })
      .collect();

    if pages.is_empty() {
      log::info!("[not split]");
      return Ok(vec![doc]);
    }

    let n = pages.len();
    log::info!(" [Split into {} pages]", n + 1);

    // Full splitting requires PostDocument::newDocument infrastructure
    // For now, return the unsplit document with a warning
    log::warn!("Document splitting not yet fully implemented; returning unsplit");
    Ok(vec![doc])
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
