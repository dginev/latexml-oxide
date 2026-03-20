use libxml::tree::{Node, NodeType};

/// Resolve an XMRef node to its target by following the idref via DOM traversal.
/// Perl: realizeXMNode (MathParser.pm) — dereferences XMRef to the target element.
fn resolve_xmref(node: &Node) -> Option<Node> {
  if node.get_name() == "XMRef" {
    if let Some(idref) = node.get_attribute("idref") {
      // Walk up to document root
      let mut ancestor = node.clone();
      while let Some(parent) = ancestor.get_parent() {
        ancestor = parent;
      }
      // Search for element with matching xml:id
      return find_by_xml_id(&ancestor, &idref);
    }
  }
  None
}

/// Find an element by xml:id attribute in the subtree (depth-first search).
fn find_by_xml_id(root: &Node, id: &str) -> Option<Node> {
  for child in root.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) {
      // Check both xml:id and id attributes
      if child.get_attribute("xml:id").as_deref() == Some(id) {
        return Some(child);
      }
      // Also check via namespace-qualified access
      if child
        .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
        .as_deref()
        == Some(id)
      {
        return Some(child);
      }
      if let Some(found) = find_by_xml_id(&child, id) {
        return Some(found);
      }
    }
  }
  None
}

pub fn get_grammatical_role(node: &Node) -> String {
  // Resolve XMRef to target node
  if let Some(target) = resolve_xmref(node) {
    return get_grammatical_role(&target);
  }
  match p_get_attribute(node, "role") {
    Some(role) => role,
    None => {
      let tag = node.get_name();
      if tag == "XMTok" {
        "UNKNOWN".to_string()
      } else if tag == "XMDual" {
        match node.get_first_element_child() {
          Some(child) => child
            .get_attribute("role")
            .unwrap_or_else(|| "UNKNOWN".to_string()),
          None => "UNKNOWN".to_string(),
        }
      } else {
        "ATOM".to_string()
      }
    },
  }
}

pub fn get_token_meaning(node: &Node) -> String {
  // Resolve XMRef to target node
  if let Some(target) = resolve_xmref(node) {
    return get_token_meaning(&target);
  }
  match p_get_attribute(node, "meaning") {
    Some(meaning) => meaning,
    None => match p_get_attribute(node, "name") {
      Some(name) => name,
      None => {
        let content = node.get_content();
        if !content.is_empty() {
          content
        } else {
          p_get_attribute(node, "role").unwrap_or_default()
        }
      },
    },
  }
}

fn p_get_attribute(item: &Node, key: &str) -> Option<String> {
  if item.get_type() == Some(NodeType::ElementNode) {
    item.get_attribute(key)
  } else {
    None
  }
}
