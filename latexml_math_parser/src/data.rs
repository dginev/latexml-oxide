use libxml::tree::{Node, NodeType};
use rustc_hash::FxHashMap;
use std::cell::RefCell;

// Thread-local idstore for XMRef resolution during math parsing.
// Set by the parser before parsing, cleared after.
// Perl uses $doc->lookupID($idref) which accesses the document's idstore.
thread_local! {
  static MATH_IDSTORE: RefCell<Option<FxHashMap<String, Node>>> = const { RefCell::new(None) };
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
        cell
          .borrow()
          .as_ref()
          .and_then(|store| store.get(&idref).cloned())
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
        content_role
          .or(pres_role)
          .unwrap_or_else(|| "UNKNOWN".to_string())
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

#[cfg(test)]
mod tests {
  use super::*;
  use libxml::parser::Parser as XmlParser;
  use libxml::tree::Document;

  fn parse(xml: &str) -> Document { XmlParser::default().parse_string(xml).expect("parse xml") }

  fn root(doc: &Document) -> Node { doc.get_root_element().expect("root element") }

  #[test]
  fn role_from_attribute_wins() {
    let doc = parse(r#"<XMTok role="ADDOP" meaning="plus">+</XMTok>"#);
    assert_eq!(get_grammatical_role(&root(&doc)), "ADDOP");
  }

  #[test]
  fn role_xmtok_without_role_is_unknown() {
    let doc = parse(r#"<XMTok>x</XMTok>"#);
    assert_eq!(get_grammatical_role(&root(&doc)), "UNKNOWN");
  }

  #[test]
  fn role_non_xmtok_without_role_is_atom() {
    let doc = parse(r#"<XMApp><XMTok>f</XMTok></XMApp>"#);
    assert_eq!(get_grammatical_role(&root(&doc)), "ATOM");
  }

  #[test]
  fn role_xmdual_prefers_content_branch() {
    let doc = parse(
      r#"<XMDual><XMTok role="CONTENTROLE">c</XMTok><XMTok role="PRESROLE">p</XMTok></XMDual>"#,
    );
    assert_eq!(get_grammatical_role(&root(&doc)), "CONTENTROLE");
  }

  #[test]
  fn role_xmdual_falls_back_to_presentation_branch() {
    let doc = parse(r#"<XMDual><XMTok>c</XMTok><XMTok role="PRESROLE">p</XMTok></XMDual>"#);
    assert_eq!(get_grammatical_role(&root(&doc)), "PRESROLE");
  }

  #[test]
  fn role_xmdual_unknown_when_both_missing() {
    let doc = parse(r#"<XMDual><XMTok>c</XMTok><XMTok>p</XMTok></XMDual>"#);
    assert_eq!(get_grammatical_role(&root(&doc)), "UNKNOWN");
  }

  #[test]
  fn meaning_from_meaning_attribute() {
    let doc = parse(r#"<XMTok meaning="plus" name="+">+</XMTok>"#);
    assert_eq!(get_token_meaning(&root(&doc)), "plus");
  }

  #[test]
  fn meaning_falls_back_to_name() {
    let doc = parse(r#"<XMTok name="+">+</XMTok>"#);
    assert_eq!(get_token_meaning(&root(&doc)), "+");
  }

  #[test]
  fn meaning_falls_back_to_content() {
    let doc = parse(r#"<XMTok>x</XMTok>"#);
    assert_eq!(get_token_meaning(&root(&doc)), "x");
  }

  #[test]
  fn meaning_falls_back_to_role_when_no_content() {
    let doc = parse(r#"<XMTok role="ADDOP"/>"#);
    assert_eq!(get_token_meaning(&root(&doc)), "ADDOP");
  }

  #[test]
  fn meaning_empty_when_nothing_present() {
    let doc = parse(r#"<XMTok/>"#);
    assert_eq!(get_token_meaning(&root(&doc)), "");
  }

  #[test]
  fn idstore_set_and_clear_is_balanced() {
    // Setting then clearing leaves no state.
    let store: FxHashMap<String, Node> = FxHashMap::default();
    set_math_idstore(store);
    clear_math_idstore();
    MATH_IDSTORE.with(|cell| assert!(cell.borrow().is_none()));
  }

  #[test]
  fn idstore_clear_is_idempotent() {
    clear_math_idstore();
    clear_math_idstore();
    MATH_IDSTORE.with(|cell| assert!(cell.borrow().is_none()));
  }
}
