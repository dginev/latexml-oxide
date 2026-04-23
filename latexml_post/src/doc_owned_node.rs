//! `DocOwnedNode` — a drop-safe wrapper for libxml `Node` handles whose
//! C storage is owned by the enclosing `Document`.
//!
//! # Why this exists
//!
//! The `libxml` crate's `_Node::drop` (v0.3.9 `src/tree/node.rs:71`)
//! unconditionally calls `xmlFreeNode(ptr)` when the wrapper's internal
//! `unlinked` flag is true. That flag is flipped by `Node::unlink_node()`
//! and has no public setter — once a Node has been unlinked, letting
//! Rust's drop glue run will always fire `xmlFreeNode`.
//!
//! For the unlinked *subtree* case we actually want the opposite: the
//! node was detached from the live document topology but its memory
//! (properties, namespace declarations, dict entries) is still shared
//! with the enclosing `Document`. When `xmlFreeDoc` eventually walks
//! the tree, it double-frees those shared allocations → SIGSEGV inside
//! `xmlFreeNodeList` / `xmlFreeProp` (see `docs/known_crashes/README.md`
//! and the 0705.0790 bisection).
//!
//! The prior fix for the idcache path was an in-place `std::mem::forget`
//! call inside `PostDocument::drop` (`document.rs:84-113`). A second
//! identical workaround landed for the `xmath` local in
//! `process_math_node` (cycle 236 commit `18c9640ee`). Scattered
//! `mem::forget`s are a code smell — they're easy to miss when a new
//! caller also holds a lingering `Node` clone for an unlinked subtree.
//!
//! `DocOwnedNode` hoists the pattern into a single RAII type. A
//! `DocOwnedNode` drops without invoking the inner `Node`'s Drop impl,
//! so the `xmlFreeNode` call is skipped and the Document remains the
//! sole owner of the C memory. The per-wrapper Rc control block
//! (~24 B) leaks until process exit — bounded by how many subtrees
//! the caller detaches.
//!
//! # Ergonomics note for upstream
//!
//! The proper fix is in `rust-libxml`: expose a public `relink()` or
//! `mark_as_doc_owned()` setter so the Drop impl can tell whether the
//! Doc or the caller owns the C allocation. Without that, every
//! detach-and-drop path in LaTeXML Post needs this wrapper.

use libxml::tree::Node;
use std::mem::ManuallyDrop;

/// A handle to a libxml `Node` that has been detached from its parent
/// but whose storage is still owned by the enclosing `Document`.
///
/// Dropping a `DocOwnedNode` does **not** call `xmlFreeNode` on the
/// inner C node; see the module-level docs for the rationale.
pub struct DocOwnedNode {
  inner: ManuallyDrop<Node>,
}

impl DocOwnedNode {
  /// Wrap a `Node` whose C memory is owned by the enclosing `Document`.
  ///
  /// Typical call: `DocOwnedNode::new(xmath.clone())` right after
  /// `doc.remove_nodes(&[xmath])`. The caller's original `Node` binding
  /// is kept alive only for the vector's lifetime; the `DocOwnedNode`
  /// preserves the Rc for the rest of program execution.
  pub fn new(node: Node) -> Self { Self { inner: ManuallyDrop::new(node) } }

  /// Borrow the underlying `Node`. Safe because `DocOwnedNode` keeps
  /// the Rc alive.
  pub fn as_node(&self) -> &Node { &self.inner }
}

impl Drop for DocOwnedNode {
  fn drop(&mut self) {
    // `ManuallyDrop` suppresses the inner Rc's Drop, which in turn
    // suppresses `_Node::drop` (and the offending `xmlFreeNode` call
    // on the unlinked subtree). The Rc control block itself leaks
    // (small, bounded), but the underlying libxml storage is reclaimed
    // by the enclosing `xmlFreeDoc` at Document drop time.
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use libxml::parser::Parser;

  #[test]
  fn detached_node_drop_is_noop() {
    let parser = Parser::default();
    let doc = parser
      .parse_string("<root><child/></root>")
      .expect("parse");
    let root = doc.get_root_element().expect("root");
    let child = root.get_first_child().expect("child");
    let mut child_mut = child.clone();
    child_mut.unlink_node();
    // Wrap and drop — must not call xmlFreeNode.
    let wrapper = DocOwnedNode::new(child);
    drop(wrapper);
    // If the wrapper's drop had fired xmlFreeNode, the subsequent
    // Document drop would double-free; the test would crash.
    drop(doc);
  }
}
