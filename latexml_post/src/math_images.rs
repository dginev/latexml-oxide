//! Math image generation processor.
//!
//! Port of `LaTeXML::Post::MathImages`.
//! Extends both MathProcessor and LaTeXImages to generate images for math.

use libxml::tree::Node;

use crate::{
  document::PostDocument,
  math_processor::{MathConversion, MathProcessor},
  processor::{ProcessResult, Processor},
};

const MIME_TYPES: &[(&str, &str)] = &[
  ("gif", "image/gif"),
  ("jpeg", "image/jpeg"),
  ("png", "image/png"),
  ("svg", "image/svg+xml"),
];

/// MathImages post-processor: generates images for math.
///
/// Port of `LaTeXML::Post::MathImages`.
pub struct MathImages {
  name:               String,
  is_secondary:       bool,
  resource_directory: String,
  resource_prefix:    String,
  image_type:         String,
}

impl MathImages {
  pub fn new(image_type: &str) -> Self {
    MathImages {
      name:               "MathImages".to_string(),
      is_secondary:       false,
      resource_directory: "mi".to_string(),
      resource_prefix:    "mi".to_string(),
      image_type:         image_type.to_string(),
    }
  }

  /// Extract the TeX string for a Math node.
  ///
  /// Port of `MathImages::extractTeX`.
  fn extract_tex(&self, node: &Node) -> Option<String> {
    let mode = node
      .get_attribute("mode")
      .map(|m| m.to_uppercase())
      .unwrap_or_else(|| "INLINE".to_string());
    let mut tex = node.get_attribute("tex")?;
    let display = if tex.trim_start().starts_with("\\displaystyle") {
      tex = tex
        .trim_start()
        .strip_prefix("\\displaystyle")?
        .trim_start()
        .to_string();
      "DISPLAY"
    } else {
      &mode
    };
    if tex.trim().is_empty() {
      return None;
    }
    Some(format!("\\begin{} {}\\end{}", display, tex, display))
  }
}

impl Processor for MathImages {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> { doc.findnodes("//ltx:Math") }

  fn resource_directory(&self) -> Option<&str> { Some(&self.resource_directory) }

  fn resource_prefix(&self) -> Option<&str> { Some(&self.resource_prefix) }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult { Ok(vec![doc]) }
}

impl MathProcessor for MathImages {
  fn convert_node(&self, doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    let math = xmath.get_parent()?;
    let tex = self.extract_tex(&math)?;

    let key = format!("MathImages:{}:{}", self.image_type, tex);
    if let Some(cached) = doc.cache_lookup(&key) {
      // Parse cached value: "path;width;height;depth"
      let parts: Vec<&str> = cached.split(';').collect();
      if parts.len() == 4 {
        let mimetype = MIME_TYPES
          .iter()
          .find(|(ext, _)| *ext == self.image_type)
          .map(|(_, mime)| mime.to_string());
        return Some(MathConversion {
          processor_name: self.name.clone(),
          mimetype,
          xml: None,
          string: None,
          src: Some(parts[0].to_string()),
          width: Some(parts[1].to_string()),
          height: Some(parts[2].to_string()),
          depth: Some(parts[3].to_string()),
        });
      }
    }

    Warn!(
      "missing_file",
      "math_images",
      "MathImages: no cached image for '{}'",
      key
    );
    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype:       None,
      xml:            None,
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    })
  }

  fn raw_id_suffix(&self) -> &str { ".mi" }

  fn is_secondary(&self) -> bool { self.is_secondary }

  fn preprocess(&self, _doc: &PostDocument, nodes: &[Node]) {
    Info!(
      "math_images",
      "generate",
      "MathImages: would generate {} images",
      nodes.len()
    );
  }
}
