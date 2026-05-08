//! Abstract collector base for content generation processors.
//!
//! Port of `LaTeXML::Post::Collector` (113 lines of Perl).
//! Base class for processors that collect information from multiple documents
//! and build derived content (indexes, bibliographies, etc.).
//! Supports splitting collected content into sub-documents by initial letter.

use libxml::tree::Node;
use std::collections::HashMap;
use std::path::Path;

use crate::document::{NodeData, PostDocument};
use crate::processor::{ProcessResult, Processor};

/// Abstract collector post-processor.
///
/// Port of `LaTeXML::Post::Collector`.
/// Subclasses (MakeIndex, MakeBibliography) implement the actual `process` method.
pub struct Collector {
  name:               String,
  resource_directory: Option<String>,
  resource_prefix:    Option<String>,
  /// Optional scanner to rescan generated content.
  /// In Perl: `$$self{scanner}->process($doc, $$self{scanner}->toProcess($doc))`
  has_scanner:        bool,
}

impl Collector {
  pub fn new(name: &str) -> Self {
    Collector {
      name:               name.to_string(),
      resource_directory: None,
      resource_prefix:    None,
      has_scanner:        false,
    }
  }

  /// Set whether this collector has an attached scanner for rescanning.
  pub fn with_scanner(mut self) -> Self {
    self.has_scanner = true;
    self
  }
}

/// Given collected content broken into portions by initial letter,
/// fill in the main document with the first sub-collection,
/// and create new documents for the rest.
///
/// Port of `Collector::makeSubCollectionDocuments`.
///
/// The `collections` map has: initial → XML data for that section.
/// The first sub-collection fills the existing `root` element;
/// each subsequent one gets a new sub-document.
pub fn make_sub_collection_documents(
  doc: &mut PostDocument,
  root: &Node,
  collections: &HashMap<String, Vec<NodeData>>,
) -> Vec<PostDocument> {
  let mut initials: Vec<&String> = collections.keys().collect();
  initials.sort();

  if initials.is_empty() {
    return vec![];
  }

  let _root_tag = doc
    .get_qname(root)
    .unwrap_or_else(|| "ltx:index".to_string());
  let root_id = root.get_attribute("xml:id").unwrap_or_default();

  // Build (id, initial) pairs for each sub-collection
  let ids: Vec<(String, &str)> = initials
    .iter()
    .enumerate()
    .map(|(i, init)| {
      if i == 0 {
        (root_id.clone(), init.as_str())
      } else {
        (format!("{}.{}", root_id, init), init.as_str())
      }
    })
    .collect();

  // For the first sub-collection, fill the main document's root element
  if let Some(first_init) = initials.first() {
    if let Some(data) = collections.get(*first_init) {
      // Build TOC linking all sub-collections
      let toc_entries: Vec<NodeData> = ids
        .iter()
        .enumerate()
        .map(|(i, (id, init))| {
          if i == 0 {
            NodeData::Element {
              tag:        "ltx:tocentry".to_string(),
              attributes: None,
              children:   vec![NodeData::Text(init.to_string())],
            }
          } else {
            NodeData::Element {
              tag:        "ltx:tocentry".to_string(),
              attributes: None,
              children:   vec![NodeData::Element {
                tag:        "ltx:ref".to_string(),
                attributes: Some(HashMap::from([
                  ("idref".to_string(), id.clone()),
                  ("show".to_string(), "refnum".to_string()),
                ])),
                children:   vec![NodeData::Text(init.to_string())],
              }],
            }
          }
        })
        .collect();

      let toc = NodeData::Element {
        tag:        "ltx:TOC".to_string(),
        attributes: Some(HashMap::from([(
          "format".to_string(),
          "veryshort".to_string(),
        )])),
        children:   vec![NodeData::Element {
          tag:        "ltx:toclist".to_string(),
          attributes: None,
          children:   toc_entries,
        }],
      };

      let mut root_mut = root.clone();
      let mut content = vec![toc];
      content.extend(data.clone());
      doc.add_nodes(&mut root_mut, &content);
    }
  }

  // For subsequent sub-collections, we'd create new documents
  // This requires PostDocument::newDocument which needs more infrastructure
  log::info!(
    "Collector: {} sub-collections by initial: {:?}",
    initials.len(),
    initials
  );

  // NOTE: Sub-documents for initials[1..] require PostDocument::newDocument
  //   which creates XML documents from element roots with proper ID remapping.
  vec![]
}

/// Compute a page name for a sub-collection document.
///
/// If the main document is "index.html", use just the initial as the name.
/// Otherwise, append the initial to the document name.
///
/// Port of `Collector::getPageName`.
pub fn get_page_name(doc: &PostDocument, initial: &str) -> String {
  if let Some(dest) = doc.get_destination() {
    let path = Path::new(dest);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("doc");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("xml");
    let dir = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
    let name = if stem == "index" {
      initial.to_string()
    } else {
      format!("{}.{}", stem, initial)
    };
    format!("{}/{}.{}", dir, name, ext)
  } else {
    format!("{}.xml", initial)
  }
}

impl Processor for Collector {
  fn get_name(&self) -> &str { &self.name }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    // Mirrors Perl Post.pm:177 `Fatal("misdefined", $self, $doc, "abstract; ...")`
    // but at Warn severity (Rust trait can't fatal here without changing the
    // signature). A concrete subtype reaching this branch is a misconfig.
    log_post_warn!(
      "misdefined", "Collector",
      "Abstract Collector::process called — concrete subclass should override"
    );
    Ok(vec![doc])
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::document::PostDocumentOptions;

  fn doc_with_dest(dest: Option<&str>) -> PostDocument {
    let opts = PostDocumentOptions {
      destination: dest.map(|s| s.to_string()),
      ..PostDocumentOptions::default()
    };
    PostDocument::new_from_string("<root/>", opts).expect("parse")
  }

  #[test]
  fn new_sets_name_defaults() {
    let c = Collector::new("MyCollector");
    assert_eq!(c.get_name(), "MyCollector");
    assert!(c.resource_directory.is_none());
    assert!(c.resource_prefix.is_none());
    assert!(!c.has_scanner);
  }

  #[test]
  fn with_scanner_flips_flag() {
    let c = Collector::new("X").with_scanner();
    assert!(c.has_scanner);
  }

  #[test]
  fn get_page_name_no_destination_is_bare_initial_xml() {
    let doc = doc_with_dest(None);
    assert_eq!(get_page_name(&doc, "A"), "A.xml");
  }

  #[test]
  fn get_page_name_index_uses_just_initial() {
    let doc = doc_with_dest(Some("/tmp/index.html"));
    // stem == "index" → name becomes just the initial.
    assert_eq!(get_page_name(&doc, "A"), "/tmp/A.html");
  }

  #[test]
  fn get_page_name_non_index_appends_initial() {
    let doc = doc_with_dest(Some("/tmp/doc.html"));
    assert_eq!(get_page_name(&doc, "A"), "/tmp/doc.A.html");
  }

  #[test]
  fn get_page_name_preserves_extension() {
    let doc = doc_with_dest(Some("/tmp/foo.xml"));
    assert_eq!(get_page_name(&doc, "B"), "/tmp/foo.B.xml");
  }

  #[test]
  fn make_sub_collection_documents_empty_map_returns_empty() {
    let mut doc = doc_with_dest(None);
    let root = doc.get_document_element().expect("root");
    let collections: HashMap<String, Vec<NodeData>> = HashMap::new();
    let result = make_sub_collection_documents(&mut doc, &root, &collections);
    assert!(result.is_empty());
  }

  #[test]
  fn make_sub_collection_documents_populated_map_currently_returns_empty() {
    // The implementation notes it doesn't yet create sub-documents for
    // initials[1..] (needs PostDocument::newDocument infra). Lock that in.
    let mut doc = doc_with_dest(None);
    let root = doc.get_document_element().expect("root");
    let mut collections: HashMap<String, Vec<NodeData>> = HashMap::new();
    collections.insert("A".to_string(), vec![NodeData::Text("entry-a".to_string())]);
    collections.insert("B".to_string(), vec![NodeData::Text("entry-b".to_string())]);
    let result = make_sub_collection_documents(&mut doc, &root, &collections);
    assert!(result.is_empty());
  }
}
