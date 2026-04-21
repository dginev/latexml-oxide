//! Lexeme-based math processor.
//!
//! Port of `LaTeXML::Post::LexMath`.
//! Trivial math post-processor that supplies the lexemes string
//! from the `lexemes` attribute of the `ltx:Math` element.
//! Requires `--preload=llamapun.sty` to populate that attribute.

use libxml::tree::Node;

use crate::document::PostDocument;
use crate::math_processor::{MathConversion, MathProcessor};
use crate::processor::{ProcessResult, Processor};

const LEX_MIMETYPE: &str = "application/x-llamapun";

/// LexMath post-processor: supplies lexeme strings as a math representation.
///
/// Port of `LaTeXML::Post::LexMath`.
pub struct LexMath {
  name:         String,
  is_secondary: bool,
}

impl Default for LexMath {
  fn default() -> Self { Self::new() }
}

impl LexMath {
  pub fn new() -> Self {
    LexMath {
      name:         "LexMath".to_string(),
      is_secondary: false,
    }
  }
}

impl Processor for LexMath {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:Math[not(ancestor::ltx:Math)]")
  }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult { Ok(vec![doc]) }
}

impl MathProcessor for LexMath {
  fn convert_node(&self, _doc: &PostDocument, xmath: &Node) -> Option<MathConversion> {
    let math = xmath.get_parent()?;
    let lexemes = math.get_attribute("lexemes")?;
    Some(MathConversion {
      processor_name: self.name.clone(),
      mimetype:       Some(LEX_MIMETYPE.to_string()),
      xml:            None,
      string:         Some(lexemes),
      src:            None,
      width:          None,
      height:         None,
      depth:          None,
    })
  }

  fn raw_id_suffix(&self) -> &str { ".lm" }

  fn is_secondary(&self) -> bool { self.is_secondary }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lex_math_new_has_default_name() {
    let lm = LexMath::new();
    assert_eq!(lm.get_name(), "LexMath");
    assert!(!lm.is_secondary);
  }

  #[test]
  fn lex_math_default_matches_new() {
    let a = LexMath::default();
    let b = LexMath::new();
    assert_eq!(a.get_name(), b.get_name());
    assert_eq!(a.is_secondary, b.is_secondary);
  }

  #[test]
  fn lex_math_raw_id_suffix() {
    let lm = LexMath::new();
    assert_eq!(lm.raw_id_suffix(), ".lm");
  }

  #[test]
  fn lex_math_is_secondary_false_by_default() {
    let lm = LexMath::new();
    assert!(!lm.is_secondary());
  }

  #[test]
  fn lex_mimetype_is_llamapun() {
    assert_eq!(LEX_MIMETYPE, "application/x-llamapun");
  }
}
