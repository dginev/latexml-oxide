//! Picture image generation processor.
//!
//! Port of `LaTeXML::Post::PictureImages`.
//! Extends LaTeXImages to generate images for ltx:picture elements.

use libxml::tree::Node;

use crate::document::PostDocument;
use crate::processor::{ProcessResult, Processor};

/// PictureImages post-processor.
///
/// Port of `LaTeXML::Post::PictureImages`.
pub struct PictureImages {
  name:               String,
  resource_directory: String,
  resource_prefix:    String,
  use_dvipng:         bool,
  empty_only:         bool,
}

impl PictureImages {
  pub fn new(empty_only: bool) -> Self {
    PictureImages {
      name: "PictureImages".to_string(),
      resource_directory: "pic".to_string(),
      resource_prefix: "pic".to_string(),
      use_dvipng: false,
      empty_only,
    }
  }

  /// Extract the TeX string for a picture node.
  ///
  /// Port of `PictureImages::extractTeX`.
  fn extract_tex(&self, node: &Node) -> Option<String> {
    let mut tex = node.get_attribute("tex").unwrap_or_default();
    tex = tex.replace('\n', "");

    if let Some(u) = node.get_attribute("unitlength") {
      tex = format!("\\setlength{{\\unitlength}}{{{}}}{}", u, tex);
    }
    if let Some(s) = node.get_attribute("scale") {
      tex = format!("\\scalebox{{{}}}{{{}}}", s, tex);
    }

    Some(format!("\\beginPICTURE {}\\endPICTURE", tex))
  }
}

impl Processor for PictureImages {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    let nodes = doc.findnodes("//ltx:picture");
    if self.empty_only {
      nodes
        .into_iter()
        .filter(|n| n.get_first_child().is_none())
        .collect()
    } else {
      nodes
    }
  }

  fn resource_directory(&self) -> Option<&str> { Some(&self.resource_directory) }

  fn resource_prefix(&self) -> Option<&str> { Some(&self.resource_prefix) }

  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    log::info!(
      "PictureImages: would generate {} picture images",
      nodes.len()
    );
    // NOTE: delegates to LaTeXImages::generateImages (requires latex + dvipng)
    Ok(vec![doc])
  }
}
