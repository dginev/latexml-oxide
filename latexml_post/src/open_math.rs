//! OpenMath conversion processor.
//!
//! Port of `LaTeXML::Post::OpenMath`.
//! Converts XMath nodes into OpenMath XML representation.
//! Uses a converter table (DefOpenMath) for dispatching Token/Apply conversion.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  document::{NodeData, PostDocument, element_children},
  math_processor::{MathConversion, MathProcessor, math_is_parsed},
  processor::{ProcessResult, Processor},
};

const OM_URI: &str = "http://www.openmath.org/OpenMath";
const OM_MIMETYPE: &str = "application/openmath+xml";

/// OpenMath converter table entry.
type OmConverter = fn(&PostDocument, &Node) -> NodeData;

/// OpenMath post-processor.
///
/// Port of `LaTeXML::Post::OpenMath`.
pub struct OpenMath {
  name:         String,
  is_secondary: bool,
  hack_plane1:  bool,
  plane1:       bool,
}

impl Default for OpenMath {
  fn default() -> Self { Self::new() }
}

impl OpenMath {
  pub fn new() -> Self {
    OpenMath {
      name:         "OpenMath".to_string(),
      is_secondary: false,
      hack_plane1:  false,
      plane1:       true,
    }
  }
}

impl Processor for OpenMath {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
  }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult { Ok(vec![doc]) }
}

impl MathProcessor for OpenMath {
  fn convert_node(&self, doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    let children = element_children(xmath);
    let xml = if children.len() == 1 {
      om_expr(doc, &children[0])
    } else {
      om_unparsed(doc, &children)
    };

    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(OM_MIMETYPE.to_string()),
      xml:            Some(xml),
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    })
  }

  fn combine_parallel(
    &self,
    _doc: &PostDocument,
    _xmath: &Node,
    primary: MathConversion,
    secondaries: Vec<MathConversion>,
  ) -> MathConversion {
    let mut attr_children = Vec::new();

    for secondary in &secondaries {
      let mimetype = secondary.mimetype.as_deref().unwrap_or("unknown");
      attr_children.push(NodeData::Element {
        tag:        "om:OMS".to_string(),
        attributes: Some(HashMap::from_iter([
          ("cd".to_string(), "Alternate".to_string()),
          ("name".to_string(), mimetype.to_string()),
        ])),
        children:   vec![],
      });

      if mimetype == OM_MIMETYPE {
        if let Some(ref xml) = secondary.xml {
          attr_children.push(xml.clone());
        }
      } else if let Some(ref xml) = secondary.xml {
        attr_children.push(NodeData::Element {
          tag:        "om:OMFOREIGN".to_string(),
          attributes: None,
          children:   vec![xml.clone()],
        });
      } else if let Some(ref string) = secondary.string {
        attr_children.push(NodeData::Element {
          tag:        "om:OMSTR".to_string(),
          attributes: None,
          children:   vec![NodeData::Text(string.clone())],
        });
      }
    }

    if let Some(ref xml) = primary.xml {
      attr_children.push(xml.clone());
    }

    MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(OM_MIMETYPE.to_string()),
      xml:            Some(NodeData::Element {
        tag:        "om:OMATTR".to_string(),
        attributes: None,
        children:   attr_children,
      }),
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    }
  }

  fn outer_wrapper(&self, _doc: &PostDocument, _xmath: &Node, conversion: NodeData) -> NodeData {
    NodeData::Element {
      tag:        "om:OMOBJ".to_string(),
      attributes: None,
      children:   vec![conversion],
    }
  }

  fn raw_id_suffix(&self) -> &str { ".om" }

  fn is_secondary(&self) -> bool { self.is_secondary }

  fn can_convert(&self, _doc: &PostDocument, math: &Node) -> bool { math_is_parsed(math) }

  fn preprocess(&self, _doc: &PostDocument, _nodes: &[Node]) {
    // Register om namespace (would need &mut doc)
    log::trace!("OpenMath: would register om namespace");
  }
}

// ======================================================================
// OpenMath expression conversion

/// Convert an XMath element node to OpenMath.
///
/// Port of `om_expr`.
fn om_expr(doc: &PostDocument, node: &Node) -> NodeData {
  // Realize XMRef nodes
  let real_node = if doc.is_qname(node, "ltx:XMRef") {
    if let Some(idref) = node.get_attribute("idref") {
      doc.find_node_by_id(&idref).cloned()
    } else {
      None
    }
  } else {
    Some(node.clone())
  };

  match real_node {
    Some(ref n) => om_expr_aux(doc, n),
    None => om_error("Missing Subexpression"),
  }
}

/// Core OpenMath expression conversion.
///
/// Port of `om_expr_aux`.
fn om_expr_aux(doc: &PostDocument, node: &Node) -> NodeData {
  let tag = match doc.get_qname(node) {
    Some(t) => t,
    None => return om_error("Missing Subexpression"),
  };

  match tag.as_str() {
    "ltx:XMWrap" | "ltx:XMArg" => {
      let children = element_children(node);
      if children.len() == 1 {
        om_expr(doc, &children[0])
      } else {
        om_unparsed(doc, &children)
      }
    },
    "ltx:XMDual" => {
      let children = element_children(node);
      if !children.is_empty() {
        om_expr(doc, &children[0]) // Content branch
      } else {
        om_error("Empty XMDual")
      }
    },
    "ltx:XMApp" => {
      let children = element_children(node);
      if children.is_empty() {
        return om_error("Missing Operator");
      }
      // Generic application
      let mut oma_children = Vec::new();
      for child in &children {
        oma_children.push(om_expr(doc, child));
      }
      NodeData::Element {
        tag:        "om:OMA".to_string(),
        attributes: None,
        children:   oma_children,
      }
    },
    "ltx:XMTok" => {
      if let Some(meaning) = node.get_attribute("meaning") {
        let cd = node
          .get_attribute("omcd")
          .unwrap_or_else(|| "latexml".to_string());
        NodeData::Element {
          tag:        "om:OMS".to_string(),
          attributes: Some(HashMap::from_iter([
            ("name".to_string(), meaning),
            ("cd".to_string(), cd),
          ])),
          children:   vec![],
        }
      } else {
        // Variable
        let name = node.get_content();
        let name = if name.trim().is_empty() {
          node
            .get_attribute("name")
            .unwrap_or_else(|| "?".to_string())
        } else {
          name
        };
        NodeData::Element {
          tag:        "om:OMV".to_string(),
          attributes: Some(HashMap::from_iter([("name".to_string(), name)])),
          children:   vec![],
        }
      }
    },
    "ltx:XMHint" => {
      // Hints are ignored in OpenMath
      NodeData::Text(String::new())
    },
    "ltx:XMText" => {
      let text = node.get_content();
      NodeData::Element {
        tag:        "om:OMSTR".to_string(),
        attributes: None,
        children:   vec![NodeData::Text(text)],
      }
    },
    _ => {
      let text = node.get_content();
      NodeData::Element {
        tag:        "om:OMSTR".to_string(),
        attributes: None,
        children:   vec![NodeData::Text(text)],
      }
    },
  }
}

/// Convert unparsed (multiple) nodes to OpenMath error expression.
fn om_unparsed(doc: &PostDocument, nodes: &[Node]) -> NodeData {
  if nodes.is_empty() {
    return om_error("Missing Subexpression");
  }

  let mut children = vec![NodeData::Element {
    tag:        "om:OMS".to_string(),
    attributes: Some(HashMap::from_iter([
      ("cd".to_string(), "ambiguous".to_string()),
      ("name".to_string(), "fragments".to_string()),
    ])),
    children:   vec![],
  }];

  for node in nodes {
    let tag = doc.get_qname(node).unwrap_or_default();
    if tag == "ltx:XMHint" {
      continue;
    }
    children.push(om_expr_aux(doc, node));
  }

  NodeData::Element {
    tag: "om:OME".to_string(),
    attributes: None,
    children,
  }
}

/// Create an OpenMath error element.
fn om_error(msg: &str) -> NodeData {
  NodeData::Element {
    tag:        "om:OME".to_string(),
    attributes: None,
    children:   vec![
      NodeData::Element {
        tag:        "om:OMS".to_string(),
        attributes: Some(HashMap::from_iter([
          ("name".to_string(), "unexpected".to_string()),
          ("cd".to_string(), "moreerrors".to_string()),
        ])),
        children:   vec![],
      },
      NodeData::Element {
        tag:        "om:OMSTR".to_string(),
        attributes: None,
        children:   vec![NodeData::Text(msg.to_string())],
      },
    ],
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn openmath_new_has_default_name() {
    let o = OpenMath::new();
    assert_eq!(o.get_name(), "OpenMath");
    assert!(!o.is_secondary);
    assert!(!o.hack_plane1);
    assert!(o.plane1, "plane1 defaults to true");
  }

  #[test]
  fn openmath_default_matches_new() {
    let a = OpenMath::default();
    let b = OpenMath::new();
    assert_eq!(a.get_name(), b.get_name());
    assert_eq!(a.is_secondary, b.is_secondary);
    assert_eq!(a.plane1, b.plane1);
    assert_eq!(a.hack_plane1, b.hack_plane1);
  }

  #[test]
  fn openmath_raw_id_suffix() {
    let o = OpenMath::new();
    assert_eq!(o.raw_id_suffix(), ".om");
  }

  #[test]
  fn openmath_is_secondary_false_by_default() {
    let o = OpenMath::new();
    assert!(!o.is_secondary());
  }

  #[test]
  fn om_constants() {
    assert_eq!(OM_URI, "http://www.openmath.org/OpenMath");
    assert_eq!(OM_MIMETYPE, "application/openmath+xml");
  }
}
