//! TeX math preservation processor.
//!
//! Port of `LaTeXML::Post::TeXMath`.
//! Trivial math post-processor that supplies the TeX string
//! from the `tex` attribute of the `ltx:Math` element.

use libxml::tree::Node;

use crate::document::PostDocument;
use crate::math_processor::{MathConversion, MathProcessor};
use crate::processor::{ProcessResult, Processor};

const TEX_MIMETYPE: &str = "application/x-tex";

/// TeXMath post-processor: preserves the TeX source as a math representation.
///
/// Port of `LaTeXML::Post::TeXMath`.
pub struct TeXMath {
  name: String,
  is_secondary: bool,
}

impl Default for TeXMath {
    fn default() -> Self {
        Self::new()
    }
}

impl TeXMath {
  pub fn new() -> Self {
    TeXMath {
      name: "TeXMath".to_string(),
      is_secondary: false,
    }
  }
}

impl Processor for TeXMath {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
  }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    // Delegated to math_processor::process_math
    Ok(vec![doc])
  }
}

impl MathProcessor for TeXMath {
  fn convert_node(&self, _doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    let math = xmath.get_parent()?;
    let tex = math.get_attribute("tex")?;
    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype: Some(TEX_MIMETYPE.to_string()),
      xml: None,
      string: Some(tex),
      src: None,
      width: None,
      height: None,
      depth: None,
    })
  }

  fn raw_id_suffix(&self) -> &str {
    ".tm"
  }

  fn is_secondary(&self) -> bool {
    self.is_secondary
  }
}
