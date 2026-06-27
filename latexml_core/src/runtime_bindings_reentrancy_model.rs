//! Miri-checkable soundness model for the `runtime-bindings` (Rhai) constructor
//! trampoline's re-entrant `&mut Document` round-trip (PR #248 review item **B1**).
//!
//! The real path lives in `latexml_contrib::script_bindings` and cannot be
//! Miri-checked directly: a `Document` is libxml2-backed (C FFI), and Miri does
//! not execute FFI. But the *borrow-aliasing* question is entirely independent of
//! libxml2 — it is purely about the Rust-side pattern:
//!
//!   1. the core calls a constructor body with `&mut Document` (the trampoline);
//!   2. the trampoline publishes that `&mut` as a `*mut Document` on a thread-local
//!      stack (RAII-guarded) and runs the script body;
//!   3. a `document.*` op in the body re-mints `&mut *ptr` from the **top** of that
//!      stack (`with_doc`);
//!   4. for a NESTED construct (`\wrap{\myemph{..}}`) the outer body's
//!      `Document::absorb` re-enters the bridge while its own `&mut self` is still
//!      live, so a *second* `&mut` is re-minted from a raw pointer.
//!
//! The B1 caveat worried that step 4 aliases the parked outer `&mut` (Stacked /
//! Tree Borrows UB). This module reproduces the EXACT pattern over a trivial
//! `Doc { n: i32 }` (no libxml2) so Miri can adjudicate it under both the Stacked
//! and Tree Borrows models. The decisive observation: the nested pointer is a
//! reborrow **descendant** of the outer one — the core threads a reborrow of
//! `absorb`'s `&mut self` down to the nested constructor (mirrored by
//! `Doc::absorb` calling `nested_trampoline(&mut *self)`), and `with_doc` always
//! re-mints from the **innermost** published pointer (top of stack). A descendant
//! reborrow does not invalidate its ancestors, so the chain
//! `outer → with_doc → absorb → nested_trampoline → with_doc` is a single linear
//! reborrow tree, which is sound.
//!
//! Run under Miri:
//!   cargo +nightly miri test -p latexml_core --lib reentrancy_model
//!   MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test -p latexml_core --lib reentrancy_model
//! (both are exercised by `tools/miri_check.sh reentrancy_model`).

#![cfg(test)]

use std::cell::RefCell;

/// Stand-in for the libxml2-backed `Document`: a trivial owner of mutable state.
struct Doc {
  n: i32,
}

thread_local! {
  /// Mirror of `script_bindings::CTOR_CTX`: a STACK of published `&mut Doc`s as
  /// raw pointers (a stack so nested construction works).
  static STACK: RefCell<Vec<*mut Doc>> = const { RefCell::new(Vec::new()) };
}

/// Mirror of `CtorCtxGuard`: RAII push/pop so the entry is removed on every exit
/// (normal return, `?`, or unwind) — review M1.
struct CtxGuard;
impl CtxGuard {
  fn new(ptr: *mut Doc) -> Self {
    STACK.with(|s| s.borrow_mut().push(ptr));
    CtxGuard
  }
}
impl Drop for CtxGuard {
  fn drop(&mut self) {
    STACK.with(|s| {
      s.borrow_mut().pop();
    });
  }
}

/// Mirror of `current_ctx`: copy the TOP pointer out (never hold the `RefCell`
/// borrow across the re-minted-`&mut` call that may re-enter the bridge).
fn current_ptr() -> *mut Doc {
  STACK.with(|s| {
    *s.borrow()
      .last()
      .expect("with_doc outside a constructor body")
  })
}

/// Mirror of `with_doc`: re-mint `&mut Doc` from the innermost published pointer.
fn with_doc<R>(f: impl FnOnce(&mut Doc) -> R) -> R {
  let ptr = current_ptr();
  // SAFETY (modeled): `ptr` is the innermost published `&mut Doc` for this body.
  // For the nested case it is a reborrow descendant of the outer one (see below),
  // so this is a child reborrow, not an alias.
  let doc = unsafe { &mut *ptr };
  f(doc)
}

/// Mirror of `closure_replacement`: publish the core's `&mut Doc` as `*mut` (RAII),
/// run the body, then return. Crucially, `document` is NOT touched after `body()`
/// — matching the real trampoline, where the body reaches the doc only via
/// `with_doc`.
fn trampoline(document: &mut Doc, body: impl FnOnce()) {
  let _g = CtxGuard::new(document as *mut Doc);
  body();
}

impl Doc {
  /// Mirror of `Document::absorb`: takes `&mut self` and re-enters the bridge for
  /// a nested construct, threading a **reborrow** of `&mut self` down to the
  /// nested constructor's trampoline (exactly as the core's `absorb` →
  /// `be_absorbed(self)` → constructor-closure(`&mut Document`) chain does).
  fn absorb_reentrant(&mut self) { nested_trampoline(&mut *self); }
  fn bump(&mut self) { self.n += 1; }
}

/// The nested constructor's trampoline body: re-mints the doc via `with_doc` and
/// mutates it (models the inner `\myemph` body calling `document.absorb(...)`).
fn nested_trampoline(document: &mut Doc) {
  trampoline(document, || {
    with_doc(|d| d.bump());
  });
}

/// `\myemph{X}` alone: a single (non-nested) constructor body. Baseline. This
/// also models the SIBLING `WHATSIT_CTX` re-mint (`engine.rs::setProperty`,
/// `&mut *ptr`): after-digest hooks run one-pass/sequentially on a fresh-local
/// whatsit and never re-enter on the SAME whatsit, so the whatsit case is ALWAYS
/// this single-body pattern (no nested same-object re-mint).
#[test]
fn reentrancy_model_single_body_sound() {
  let mut doc = Doc { n: 0 };
  trampoline(&mut doc, || {
    with_doc(|d| d.bump());
    with_doc(|d| d.bump()); // multiple sequential `with_doc`s re-mint from the same ptr
  });
  assert_eq!(doc.n, 2);
}

/// `\wrap{\myemph{..}}`: the B1 nested case — the outer body's `absorb` re-enters
/// the bridge and the inner body re-mints a second `&mut` while the outer is live.
#[test]
fn reentrancy_model_nested_construct_sound() {
  let mut doc = Doc { n: 0 };
  trampoline(&mut doc, || {
    with_doc(|d| d.absorb_reentrant());
  });
  assert_eq!(doc.n, 1);
}

/// Deeper nesting `\wrap{\wrap{\myemph{..}}}` — three live reborrow levels — plus
/// an outer `with_doc` AFTER the nested absorb returns (the parked outer pointer
/// must still be usable once the descendants are gone).
#[test]
fn reentrancy_model_deep_nesting_sound() {
  let mut doc = Doc { n: 0 };
  trampoline(&mut doc, || {
    with_doc(|d| {
      d.bump(); // before
      d.absorb_reentrant(); // level 2 → its own absorb re-enters again
    });
    with_doc(|d| d.bump()); // after the nested absorb returns: re-mint from the parked outer ptr
  });
  assert_eq!(doc.n, 3);
}

/// A nested body whose `absorb` ITSELF re-enters a further level — exercises a
/// 3-deep live reborrow tree (outer absorb parked, middle absorb parked, inner
/// mutates).
fn doubly_nested_trampoline(document: &mut Doc) {
  trampoline(document, || {
    with_doc(|d| d.absorb_reentrant());
  });
}

#[test]
fn reentrancy_model_three_levels_sound() {
  let mut doc = Doc { n: 0 };
  trampoline(&mut doc, || {
    with_doc(doubly_nested_trampoline);
  });
  assert_eq!(doc.n, 1);
}
