use libxml::tree::{Node, NodeType};
use std::cell::RefCell;
use rustc_hash::FxHashMap;

// Thread-local idstore for XMRef resolution during math parsing.
// Set by the parser before parsing, cleared after.
// Perl uses $doc->lookupID($idref) which accesses the document's idstore.
thread_local! {
  static MATH_IDSTORE: RefCell<Option<FxHashMap<String, Node>>> = RefCell::new(None);
}

/// Set the idstore for XMRef resolution. Called before math parsing starts.
pub fn set_math_idstore(idstore: FxHashMap<String, Node>) {
  MATH_IDSTORE.with(|cell| {
    *cell.borrow_mut() = Some(idstore);
  });
}

/// Clear the idstore after math parsing.
pub fn clear_math_idstore() {
  MATH_IDSTORE.with(|cell| {
    *cell.borrow_mut() = None;
  });
}

/// Resolve an XMRef node to its target using the idstore (matching Perl's lookupID).
/// Falls back to DOM traversal if idstore is not set.
fn resolve_xmref(node: &Node) -> Option<Node> {
  if node.get_name() == "XMRef" {
    if let Some(idref) = node.get_attribute("idref") {
      // Use idstore first (fast and reliable, matching Perl's $doc->lookupID)
      let store_result = MATH_IDSTORE.with(|cell| {
        cell.borrow().as_ref().and_then(|store| store.get(&idref).cloned())
      });
      if store_result.is_some() {
        return store_result;
      }
      // Fallback: walk DOM to document root, then search by xml:id
      let mut ancestor = node.clone();
      while let Some(parent) = ancestor.get_parent() {
        ancestor = parent;
      }
      return find_by_xml_id(&ancestor, &idref);
    }
  }
  None
}

/// Find an element by xml:id attribute in the subtree (depth-first search).
fn find_by_xml_id(root: &Node, id: &str) -> Option<Node> {
  for child in root.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode) {
      if child.get_attribute("xml:id").as_deref() == Some(id) {
        return Some(child);
      }
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
        // Perl: check content branch first, then presentation branch
        let children: Vec<_> = node.get_child_elements();
        let content_role = children.first().and_then(|c| c.get_attribute("role"));
        let pres_role = children.get(1).and_then(|p| p.get_attribute("role"));
        content_role.or(pres_role).unwrap_or_else(|| "UNKNOWN".to_string())
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
