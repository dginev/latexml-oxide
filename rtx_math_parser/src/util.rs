use crate::data::{get_grammatical_role, get_token_meaning};
use libxml::tree::Node;

/// Generate a textual token for each node; The parser operates on this encoded
/// string.
pub fn node_to_grammar_lexemes(mathnode: &Node, idx:&mut usize) -> (Vec<String>, Vec<Node>) {
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  let top_role_opt = mathnode.get_attribute("role");
  if let Some(ref top_role) = top_role_opt {
    *idx+=1;
    lexemes.push(format!("start_{}:start:{}",top_role,idx));
    nodes.push(mathnode.clone());
  }
  let child_nodes = filter_hints(mathnode.get_child_nodes());
  for node in child_nodes.into_iter() {
    if node.get_name() == "XMApp" {
      let (mut inner_lexes,mut inner_nodes) = node_to_grammar_lexemes(&node, idx);
      for (inner_lex, inner_node) in inner_lexes.drain(..).zip(inner_nodes.drain(..)) {
        lexemes.push(inner_lex);
        nodes.push(inner_node);
      }
    } else {
      let role = get_grammatical_role(&node);
      let mut text = get_token_meaning(&node);
      if text.is_empty() {
        text = "UNKNOWN".to_string();
      }
      *idx+=1;
      let lexeme = format!("{}:{}:{}", role, text, idx).replace(' ', "");
      lexemes.push(lexeme);
      nodes.push(node);
    }
  }
  if let Some(top_role) = top_role_opt {
    *idx+=1;
    lexemes.push(format!("end_{}:end:{}",top_role, idx));
    nodes.push(mathnode.clone());
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

pub fn filter_hints(nodes: Vec<Node>) -> Vec<Node> { nodes }
