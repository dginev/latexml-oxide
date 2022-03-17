use crate::semantics::Tree;
use libxml::tree::{Node, NodeType};

pub fn get_grammatical_role(node: &Node) -> String {
  let role = match p_get_attribute(node, "role") {
    Some(role) => role,
    None => {
      let tag = node.get_name();
      if tag == "XMTok" {
        "UNKNOWN".to_string() }
      else if tag == "XMDual" {
        match node.get_first_element_child() {
          Some(child) => child.get_attribute("role").unwrap_or_else(|| {"UNKNOWN".to_string()}),
          None => "UNKNOWN".to_string()
        }
      } else {
        "ATOM".to_string()
      }
    }
  };
  role
}

pub fn get_token_meaning(node: &Node) -> String {
  match p_get_attribute(node, "meaning") {
    Some(meaning) => meaning,
    None => match p_get_attribute(node, "name") {
      Some(name) => name,
      None => { let content = node.get_content();
        if !content.is_empty() {
          content
        } else {
          p_get_attribute(node, "role").unwrap_or_default()
        }
      }
    }
  }
}

fn p_get_attribute(item: &Node, key: &str) -> Option<String> {
  if item.get_type() == Some(NodeType::ElementNode) {
    item.get_attribute(key)
  } else {
    None
  }
}
