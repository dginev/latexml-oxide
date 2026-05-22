//! Abstract manifest creation processor.
//!
//! Port of `LaTeXML::Post::Manifest`.
//! Abstract class for creating manifests (e.g., EPUB).
//! Concrete implementations live in submodules.

pub mod epub;

use libxml::tree::Node;

use crate::document::PostDocument;
use crate::processor::{ProcessResult, Processor};

/// Manifest format specifier.
#[derive(Debug, Clone)]
pub enum ManifestFormat {
  Epub,
}

/// Abstract manifest post-processor.
///
/// Port of `LaTeXML::Post::Manifest`.
pub struct Manifest {
  name:           String,
  format:         Option<ManifestFormat>,
  site_directory: Option<String>,
}

impl Manifest {
  pub fn new(format: Option<ManifestFormat>, site_directory: Option<String>) -> Self {
    let name = match &format {
      Some(ManifestFormat::Epub) => "Manifest[Epub]".to_string(),
      None => "Manifest".to_string(),
    };
    Manifest { name, format, site_directory }
  }
}

impl Processor for Manifest {
  fn get_name(&self) -> &str { &self.name }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    match &self.format {
      Some(ManifestFormat::Epub) => {
        Info!("manifest", "epub", "EPUB manifest generation delegated to epub submodule");
        Ok(vec![doc])
      },
      None => {
        Warn!("manifest", "format", "No manifest format specified; skipping");
        Ok(vec![doc])
      },
    }
  }
}
