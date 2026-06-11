//! XMath pseudo-generator for preserving LaTeXML's parsed math.
//!
//! Port of `LaTeXML::Post::XMath`.
//!
//! If XMath is the primary representation (or only one), it is left in place.
//! If secondary, it is cloned with modified IDs and moved.
//! Must be the last math formatter in the chain when used as secondary,
//! since XMath removal would break subsequent formatters.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  document::{NodeData, PostDocument, element_children_iter},
  math_processor::{MathConversion, MathProcessor},
  processor::{ProcessResult, Processor},
};

const XMATH_MIMETYPE: &str = "application/x-latexml";

/// XMath post-processor: preserves or clones XMath as a math representation.
///
/// Port of `LaTeXML::Post::XMath`.
pub struct XMath {
  name:         String,
  is_secondary: bool,
}

impl Default for XMath {
  fn default() -> Self { Self::new() }
}

impl XMath {
  pub fn new() -> Self {
    XMath {
      name:         "XMath".to_string(),
      is_secondary: false,
    }
  }
}

impl Processor for XMath {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
  }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult { Ok(vec![doc]) }
}

impl MathProcessor for XMath {
  fn convert_node(&self, _doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    let id_suffix = self.id_suffix();
    let xml = if !id_suffix.is_empty() {
      // Secondary: clone the XMath with modified IDs
      let children: Vec<NodeData> = element_children_iter(xmath)
        .map(NodeData::XmlNode)
        .collect();
      Some(NodeData::Element {
        tag: "ltx:XMath".to_string(),
        attributes: Some(HashMap::from_iter([(
          "_sourced".to_string(),
          "1".to_string(),
        )])),
        children,
      })
    } else {
      // Primary: just reference the existing XMath node
      Some(NodeData::XmlNode(xmath.clone()))
    };

    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype: Some(XMATH_MIMETYPE.to_string()),
      xml,
      string: None,
      src: None,
      width: None,
      height: None,
      depth: None,
    })
  }

  fn combine_parallel(
    &self,
    _doc: &PostDocument,
    _xmath: &Node,
    primary: MathConversion,
    secondaries: Vec<MathConversion>,
  ) -> MathConversion {
    let mut alt_children = Vec::new();

    // Primary XML goes first
    if let Some(ref xml) = primary.xml {
      alt_children.push(xml.clone());
    }

    // Add secondaries
    for secondary in &secondaries {
      let mimetype = secondary.mimetype.as_deref().unwrap_or("unknown");
      if mimetype == XMATH_MIMETYPE {
        if let Some(ref xml) = secondary.xml {
          alt_children.push(xml.clone());
        }
      } else if let Some(ref xml) = secondary.xml {
        // Other XML: needs wrapping (outerWrapper would be called by the processor)
        alt_children.push(xml.clone());
      }
    }

    MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(XMATH_MIMETYPE.to_string()),
      xml:            Some(NodeData::Element {
        tag:        "_Fragment_".to_string(),
        attributes: None,
        children:   alt_children,
      }),
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    }
  }

  fn raw_id_suffix(&self) -> &str { ".xm" }

  fn is_secondary(&self) -> bool { self.is_secondary }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn xmath_new_has_default_name() {
    let x = XMath::new();
    assert_eq!(x.get_name(), "XMath");
    assert!(!x.is_secondary);
  }

  #[test]
  fn xmath_default_matches_new() {
    let a = XMath::default();
    let b = XMath::new();
    assert_eq!(a.get_name(), b.get_name());
    assert_eq!(a.is_secondary, b.is_secondary);
  }

  #[test]
  fn xmath_raw_id_suffix() {
    let x = XMath::new();
    assert_eq!(x.raw_id_suffix(), ".xm");
  }

  #[test]
  fn xmath_is_secondary_false_by_default() {
    let x = XMath::new();
    assert!(!x.is_secondary());
  }

  #[test]
  fn xmath_mimetype_is_x_latexml() {
    assert_eq!(XMATH_MIMETYPE, "application/x-latexml");
  }
}
