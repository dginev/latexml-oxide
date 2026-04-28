//! Abstract math processor base.
//!
//! Port of `LaTeXML::Post::MathProcessor`.
//! Extends [`Processor`] with math-specific conversion infrastructure:
//! - Parallel markup support (primary + secondary formats)
//! - Cross-referencing between math formats
//! - XMath node visibility and realization
//! - XMText content conversion

use libxml::tree::Node;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::document::{NodeData, PostDocument};
use crate::processor::{PostError, Processor};

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static POST_AUDIT: LazyLock<bool> = LazyLock::new(|| std::env::var("LATEXML_POST_AUDIT").is_ok());

/// Result of converting a math node.
#[derive(Debug, Clone)]
pub struct MathConversion {
  /// The processor that produced this conversion.
  pub processor_name: String,
  /// MIME type of the conversion (e.g., "application/mathml+xml").
  pub mimetype:       Option<String>,
  /// The converted XML (as a NodeData tree).
  pub xml:            Option<NodeData>,
  /// String representation (for non-XML formats).
  pub string:         Option<String>,
  /// Image source path (for image-based conversions).
  pub src:            Option<String>,
  /// Image width.
  pub width:          Option<String>,
  /// Image height.
  pub height:         Option<String>,
  /// Image depth (baseline offset).
  pub depth:          Option<String>,
}

/// Abstract base trait for math-processing post-processors.
///
/// Port of `LaTeXML::Post::MathProcessor`.
///
/// Implementors must define:
/// - [`convert_node`] — convert an XMath node to the target format
/// - [`raw_id_suffix`] — suffix for generated IDs (e.g., ".pmml")
///
/// For parallel markup, also implement:
/// - [`combine_parallel`] — merge primary + secondary conversions
pub trait MathProcessor: Processor {
  /// Convert an XMath node to this processor's format.
  ///
  /// Port of `MathProcessor::convertNode` (abstract in Perl).
  fn convert_node(&self, doc: &PostDocument, xmath: &Node) -> Option<MathConversion>;

  /// Combine parallel markup from primary conversion + secondaries.
  ///
  /// Port of `MathProcessor::combineParallel` (abstract in Perl).
  /// Default implementation just returns the primary, dropping secondaries.
  fn combine_parallel(
    &self,
    _doc: &PostDocument,
    _xmath: &Node,
    primary: MathConversion,
    secondaries: Vec<MathConversion>,
  ) -> MathConversion {
    if !secondaries.is_empty() {
      log::error!(
        "Abstract combineParallel: dropping extra markup from: {}",
        secondaries
          .iter()
          .map(|s| s.processor_name.as_str())
          .collect::<Vec<_>>()
          .join(", ")
      );
    }
    primary
  }

  /// Raw ID suffix for this format (e.g., ".pmml", ".cmml", ".om").
  /// Primary format returns empty string; secondaries return their suffix.
  ///
  /// Port of `MathProcessor::rawIDSuffix`.
  fn raw_id_suffix(&self) -> &str { "" }

  /// Whether this processor is a secondary (parallel) processor.
  fn is_secondary(&self) -> bool { false }

  /// ID suffix: empty for primary, raw_id_suffix for secondary.
  ///
  /// Port of `MathProcessor::IDSuffix`.
  fn id_suffix(&self) -> &str {
    if self.is_secondary() {
      self.raw_id_suffix()
    } else {
      ""
    }
  }

  /// Whether this processor can convert the given math node.
  /// Default: always true.
  ///
  /// Port of `MathProcessor::canConvert`.
  fn can_convert(&self, _doc: &PostDocument, _math: &Node) -> bool { true }

  /// Optional preprocessing before conversion begins.
  ///
  /// Port of `MathProcessor::preprocess`.
  fn preprocess(&self, _doc: &PostDocument, _nodes: &[Node]) {}

  /// Wrap the converted XML in the appropriate outer element (e.g., `m:math`).
  ///
  /// Port of `MathProcessor::outerWrapper`.
  fn outer_wrapper(&self, _doc: &PostDocument, _xmath: &Node, conversion: NodeData) -> NodeData {
    conversion
  }
}

/// Check if a Math element was successfully parsed (not marked as unparsed).
///
/// Port of `MathProcessor::mathIsParsed`.
pub fn math_is_parsed(math: &Node) -> bool {
  math
    .get_attribute("class")
    .map(|c| !c.contains("ltx_math_unparsed"))
    .unwrap_or(true)
}

/// Process all top-level Math nodes in the document using a math processor.
///
/// This is the main orchestration function that handles:
/// - Finding top-level Math nodes (not nested inside other Math)
/// - Parallel markup coordination
/// - Cross-referencing between formats
///
/// When `keep_xmath` is true, the XMath elements are preserved in the output
/// alongside the generated MathML.
///
/// Port of `MathProcessor::process`.
pub fn process_math(
  processor: &dyn MathProcessor,
  doc: &mut PostDocument,
  maths: Vec<Node>,
  keep_xmath: bool,
) -> Result<(), PostError> {
  doc.mark_xm_node_visibility();
  processor.preprocess(doc, &maths);

  // Re-fetch once after preprocess in case it restructured things (matches
  // Perl Post.pm L307-308 "# Re-Fetch the math nodes, in case preprocessing
  // has messed them up!!!"). Then iterate in reverse so nested math is
  // converted first and carried along with the enclosing math.
  let maths = doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]");
  let n = maths.len();
  // LATEXML_POST_AUDIT=1 records per-node wall-clock for the math
  // post-processing loop — diagnosis aid for MathML::Presentation perf.
  let audit = *POST_AUDIT;
  let mut total_ns: u128 = 0;
  let mut max_ns: u128 = 0;
  let mut max_idx: usize = 0;
  for (i, math) in maths.into_iter().rev().enumerate() {
    let t0 = if audit {
      Some(std::time::Instant::now())
    } else {
      None
    };
    process_math_node(processor, doc, &math, keep_xmath)?;
    if let Some(t0) = t0 {
      let ns = t0.elapsed().as_nanos();
      total_ns += ns;
      if ns > max_ns {
        max_ns = ns;
        max_idx = i;
      }
    }
  }
  if audit {
    log::info!(
      "POST_AUDIT: {} math nodes in {}ms (max {}µs at index {})",
      n,
      total_ns / 1_000_000,
      max_ns / 1_000,
      max_idx
    );
  }

  // Clean up _cvis/_pvis internal visibility markers from XMath nodes
  for mut node in doc.findnodes("//*[@_cvis or @_pvis]") {
    let _ = node.remove_attribute("_cvis");
    let _ = node.remove_attribute("_pvis");
  }

  log::info!("converted {} Maths", n);
  Ok(())
}

/// Process a single Math node: convert XMath and add result to the Math element.
///
/// Port of `MathProcessor::processNode`.
fn process_math_node(
  processor: &dyn MathProcessor,
  doc: &mut PostDocument,
  math: &Node,
  keep_xmath: bool,
) -> Result<(), PostError> {
  let xmath = match doc.findnode_at("ltx:XMath", math) {
    Some(x) => x,
    None => return Ok(()), // Nothing to convert
  };

  // Convert
  let mut conversion = processor
    .convert_node(doc, &xmath)
    .unwrap_or(MathConversion {
      processor_name: processor.get_name().to_string(),
      mimetype:       None,
      xml:            None,
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    });

  // Apply outer wrapper if we got XML
  if let Some(xml) = conversion.xml.take() {
    conversion.xml = Some(processor.outer_wrapper(doc, &xmath, xml));
  } else if let Some(ref string) = conversion.string {
    // Wrap string in ltx:text
    let mimetype = conversion.mimetype.as_deref().unwrap_or("unknown");
    conversion.xml = Some(NodeData::Element {
      tag:        "ltx:text".to_string(),
      attributes: Some(HashMap::from([(
        "class".to_string(),
        format!("ltx_math_{}", mimetype),
      )])),
      children:   vec![NodeData::Text(string.clone())],
    });
  }

  if !keep_xmath {
    // Mark XMath IDs as reusable (it will be removed)
    doc.preremove_nodes(&[xmath.clone()]);
    // Remove XMath from the Math element
    doc.remove_nodes(&[xmath.clone()]);
    // After unlink, the `xmath` Rc must not fire `_Node::drop` — that
    // would call `xmlFreeNode` on a subtree whose props/ns are still
    // shared with the enclosing Document, leading to a UAF at
    // `xmlFreeDoc` time. Wrap in `DocOwnedNode` which suppresses the
    // Drop; `xmlFreeDoc` remains the sole owner.
    //
    // Reproducer (cycle 236): `$X$` with ar5iv preload → SIGSEGV in
    // PMML pass without this wrapper. See docs/known_crashes/README.md.
    let _kept = crate::doc_owned_node::DocOwnedNode::new(xmath);
  }

  // Remove blank text nodes from Math
  doc.remove_blank_nodes(math);

  // Add the converted content
  if let Some(xml) = &conversion.xml {
    let mut math_mut = math.clone();
    doc.add_nodes(&mut math_mut, &[xml.clone()]);
  }

  // Copy image attributes if applicable
  maybe_set_math_image(math, &conversion);

  Ok(())
}

/// Set image attributes on the Math element if the conversion produced an image.
///
/// Port of `MathProcessor::maybeSetMathImage`.
fn maybe_set_math_image(math: &Node, conversion: &MathConversion) {
  if let Some(ref mimetype) = conversion.mimetype {
    if mimetype.starts_with("image/") && math.get_attribute("imagesrc").is_none() {
      if let Some(ref src) = conversion.src {
        let mut math_mut = math.clone();
        math_mut.set_attribute("imagesrc", src).ok();
        if let Some(ref w) = conversion.width {
          math_mut.set_attribute("imagewidth", w).ok();
        }
        if let Some(ref h) = conversion.height {
          math_mut.set_attribute("imageheight", h).ok();
        }
        if let Some(ref d) = conversion.depth {
          math_mut.set_attribute("imagedepth", d).ok();
        }
      }
    }
  }
}

/// Find top-level Math nodes (not nested within other Math nodes).
///
/// Port of `MathProcessor::toProcess`.
pub fn find_top_level_math(doc: &PostDocument) -> Vec<Node> {
  doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn math_conversion_clone_preserves() {
    let c = MathConversion {
      processor_name: "test".to_string(),
      mimetype:       Some("application/mathml+xml".to_string()),
      xml:            None,
      string:         Some("x+y".to_string()),
      src:            None,
      width:          Some("5em".to_string()),
      height:         None,
      depth:          None,
    };
    let d = c.clone();
    assert_eq!(d.processor_name, "test");
    assert_eq!(d.mimetype.as_deref(), Some("application/mathml+xml"));
    assert_eq!(d.string.as_deref(), Some("x+y"));
    assert_eq!(d.width.as_deref(), Some("5em"));
  }

  #[test]
  fn math_conversion_all_none_is_valid() {
    // A MathConversion with nothing set is well-formed; it's just not
    // useful output.
    let c = MathConversion {
      processor_name: "noop".to_string(),
      mimetype:       None,
      xml:            None,
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    };
    assert!(c.mimetype.is_none());
    assert!(c.xml.is_none());
    assert!(c.string.is_none());
    assert!(c.src.is_none());
    assert!(c.width.is_none());
    assert!(c.height.is_none());
    assert!(c.depth.is_none());
  }

  #[test]
  fn math_conversion_debug_is_non_empty() {
    let c = MathConversion {
      processor_name: "pmml".to_string(),
      mimetype:       None,
      xml:            None,
      string:         None,
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    };
    let s = format!("{c:?}");
    assert!(s.contains("pmml"));
  }
}
