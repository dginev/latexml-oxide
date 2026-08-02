use std::cell::RefCell;

use libxml::tree::{Node, NodeType};
use rustc_hash::FxHashMap;

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

/// Drop every id under `node` from the snapshot, before that subtree is released.
///
/// The snapshot holds live `Node` handles, so a released subtree must not stay
/// reachable through it — `resolve_xmref` would hand back freed memory. A
/// purged id simply fails to resolve, which is what "the node is gone" means,
/// and `resolve_xmref` already falls back to a DOM walk for a genuine miss.
pub fn purge_math_idstore_subtree(node: &Node) {
  fn collect(node: &Node, ids: &mut Vec<String>) {
    // `get_attribute("xml:id")` is the footgun the lint ratchet guards: libxml2
    // stores xml:id as local name `id` in the XML namespace, so the
    // string-keyed form can match NOTHING and the purge would silently collect
    // no ids at all. Same accessor `find_by_xml_id` below already uses.
    if let Some(id) = node.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace") {
      ids.push(id);
    }
    for child in node.get_child_nodes() {
      collect(&child, ids);
    }
  }
  MATH_IDSTORE.with(|cell| {
    let mut borrow = cell.borrow_mut();
    let Some(store) = borrow.as_mut() else { return };
    let mut ids = Vec::new();
    collect(node, &mut ids);
    for id in ids {
      store.remove(&id);
    }
  });
}

// Thread-local LOSTNODES map: lost_id → kept_id. Perl
// `MathParser::ReplacedBy` (MathParser.pm L1562-1588) records that a node
// was structurally absorbed by another node during semantics rules — e.g.
// the second `+` in `a + b + c` is absorbed into the first when forming
// the n-ary `Apply(+, a, b, c)`. Without this tracking, any pre-existing
// `XMRef[idref=lost_id]` becomes dangling (the canonical
// `Error:expected:id Cannot find a node with xml:id='...'` cluster from
// stage_51, ~63% of CONVERR papers).
//
// At end-of-parse the top-level parser sweeps remaining XMRefs and
// rewrites their idref through this map (with cycle-safe transitive
// lookup matching Perl L287-297).
thread_local! {
  static LOST_NODES: RefCell<FxHashMap<String, String>> =
    RefCell::new(FxHashMap::default());
}

/// Record that `lost_id` was absorbed by `keep_id` during a semantics
/// rule. Mirrors Perl `MathParser::ReplacedBy` body. Idempotent; first
/// recorded mapping wins (a node can only be replaced once at any
/// point in the parse).
pub fn record_replacement(lost_id: &str, keep_id: &str) {
  if lost_id == keep_id || lost_id.is_empty() || keep_id.is_empty() {
    return;
  }
  LOST_NODES.with(|cell| {
    cell
      .borrow_mut()
      .entry(lost_id.to_string())
      .or_insert_with(|| keep_id.to_string());
  });
}

/// Take ownership of the LOSTNODES map and clear the thread-local
/// (caller is responsible for performing the rewrite walk and then
/// dropping the map).
pub fn take_lost_nodes() -> FxHashMap<String, String> {
  LOST_NODES.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// Reset LOSTNODES (called at the start of each new math parse to ensure
/// per-document isolation when the same thread processes multiple docs).
pub fn clear_lost_nodes() { LOST_NODES.with(|cell| cell.borrow_mut().clear()); }

/// Resolve an XMRef node to its target using the idstore (matching Perl's lookupID).
/// Falls back to DOM traversal if idstore is not set.
fn resolve_xmref(node: &Node) -> Option<Node> {
  if node.get_name() == "XMRef"
    && let Some(idref) = node.get_attribute("idref")
  {
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
  use libxml::{parser::Parser as XmlParser, tree::Document};

  use super::*;

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

  #[test]
  fn stale_handles_from_a_dead_document_are_swept_without_panic() {
    // A pooled-worker thread's PRIOR conversion aborted math parsing (resource
    // fatal or caught panic), leaving deferred-discard and idstore handles
    // whose document is gone. Pre-sweep, the next paper's first drain walked
    // them and panicked in libxml's ptr_as_option on the dead docref — the
    // three fleet-only panic:caught fatals of the 2026-08-02 rc4 sweep
    // (2605.08935, 2606.01083, 2606.22705). The sweep must neutralize
    // wrapper-only: no traversal, no FFI free against the reclaimed C tree.
    let doc = parse(r#"<XMApp><XMTok role="ADDOP"/></XMApp>"#);
    let r = root(&doc);
    let child = r.get_first_element_child().expect("child exists");
    let mut store: FxHashMap<String, Node> = FxHashMap::default();
    store.insert("t1".to_string(), child.clone());
    set_math_idstore(store);
    defer_discard(child);
    drop(r);
    drop(doc);
    let swept = sweep_stale_math_state();
    assert_eq!(swept, 2, "one pending discard + one idstore entry");
    PENDING_DISCARDS.with(|cell| assert!(cell.borrow().is_empty()));
    MATH_IDSTORE.with(|cell| assert!(cell.borrow().is_none()));
    // Idempotent on a clean thread.
    assert_eq!(sweep_stale_math_state(), 0);
  }
}

thread_local! {
  /// Subtrees discarded during math parsing, retained until the parse finishes.
  static PENDING_DISCARDS: RefCell<Vec<Node>> = const { RefCell::new(Vec::new()) };
}

/// Detach `node` now, free it AFTER math parsing.
///
/// Freeing mid-parse is a use-after-free: the parse holds live `Node` handles
/// in several places at once — the formula list `parse_math` collects up front,
/// the `MATH_IDSTORE` snapshot, `LOSTNODES`, XMDual bookkeeping — and a freed
/// node's memory is promptly recycled. The observed crash read a node name
/// whose bytes had already become `_pvis`, an attribute name from a later
/// allocation (witness 2605.00812, 344 formulae, dies on the 305th; 14 such
/// fatals in the 2026-07-30 sandbox-arxiv-2605 rerun).
///
/// Chasing those references one at a time does not converge — purging the
/// idstore snapshot alone still segfaulted. Deferring closes every window at
/// once, and it is bounded: under streaming, math parsing runs per FRAGMENT,
/// so the queue is fragment-sized, not document-sized.
///
/// The node is UNLINKED immediately, so the tree — and therefore the output —
/// is exactly as if it had been freed here.
pub fn defer_discard(mut node: Node) {
  node.unlink();
  PENDING_DISCARDS.with(|cell| cell.borrow_mut().push(node));
}

/// Hand back everything queued by [`defer_discard`], for the caller to free once
/// no parse state references it.
pub fn take_pending_discards() -> Vec<Node> {
  PENDING_DISCARDS.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// Put back a subtree that cannot be released yet — it still contains a formula
/// the parse has not reached.
pub fn requeue_pending_discard(node: Node) {
  PENDING_DISCARDS.with(|cell| cell.borrow_mut().push(node));
}

/// Sweep residue a PRIOR conversion left in this thread's math-parser state.
///
/// The pooled `cortex_worker` reuses threads across papers with no thread-state
/// reset, and an aborted math parse — a propagated resource fatal, or a panic
/// caught by the worker's per-task isolation — can exit without draining
/// [`PENDING_DISCARDS`] or the `MATH_IDSTORE` snapshot. Those `Node` handles
/// then outlive their document: the next paper's first traversal panics in
/// libxml's `ptr_as_option` on the dead docref (witnesses 2605.08935,
/// 2606.01083, 2606.22705 — the three fleet-only `panic:caught` fatals of the
/// 2026-08-02 rc4 sweep), and even dropping a stale `Unlinked` wrapper reads
/// the freed C node. `set_linked()` is wrapper-only bookkeeping (public since
/// libxml 0.3.20 for exactly this teardown shape), so marking each handle
/// linked first makes the drop a no-op at the FFI layer — the dead document
/// already reclaimed the C memory.
///
/// Returns the number of stale handles swept, for the caller's `Info` line —
/// the paper being converted is innocent, so this must never raise an error
/// against it.
pub fn sweep_stale_math_state() -> usize {
  let mut swept = 0;
  PENDING_DISCARDS.with(|cell| {
    let stale = std::mem::take(&mut *cell.borrow_mut());
    swept += stale.len();
    for node in &stale {
      node.set_linked();
    }
  });
  MATH_IDSTORE.with(|cell| {
    if let Some(stale) = cell.borrow_mut().take() {
      swept += stale.len();
      for node in stale.values() {
        node.set_linked();
      }
    }
  });
  swept
}
