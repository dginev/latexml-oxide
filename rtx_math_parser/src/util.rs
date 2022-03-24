use crate::data::{get_grammatical_role, get_token_meaning};
use libxml::tree::Node;

/// Generate a textual token for each node; The parser operates on this encoded
/// string.
pub fn node_to_grammar_lexemes(mathnode: &Node) -> (Vec<String>, Vec<Node>) {
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  for (idx, node) in mathnode.get_child_nodes().into_iter().enumerate() {
    let role = get_grammatical_role(&node);
    let mut text = get_token_meaning(&node);
    if text.is_empty() {
      text = "UNKNOWN".to_string();
    }
    let lexeme = format!("{}:{}:{}", role, text, 1 + idx).replace(' ', "");
    lexemes.push(lexeme);
    nodes.push(node);
  }
  (lexemes, nodes)
}

/// Auxiliary separator for ROLE:style-lexeme into ("ROLE:style", '-', lexeme)
pub fn distill_lexeme(name: &str) -> (&str, &str, &str) {
  // dash separates styles, colons separate grammatical roles, and we are
  // only trying to distill the last pure lexeme
  // note that we are only trying to do this reasonably for letter-based names (UNKNOWN:italic-x),
  // since some of the content symbols contain dashes themselves (e.g.
  // OPERATOR:partial-differential)
  if let Some(position) = name.rfind('-') {
    let (base, trailer) = name.split_at(position);
    let (sep, lexeme) = trailer.split_at(1);
    (base, sep, lexeme)
  } else if let Some(position) = name.rfind(':') {
    let (base, trailer) = name.split_at(position);
    let (sep, lexeme) = trailer.split_at(1);
    (base, sep, lexeme)
  } else {
    ("", "", name)
  }
}
