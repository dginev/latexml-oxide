//! Perl-style local assignments (dynamic scope)
//!
//! This module provides stack-based dynamic scoping for global state,
//! matching Perl's `local` mechanism. Each "localized" field uses a `Vec<T>`
//! as a stack: push to shadow, pop to restore.
//!
//! ## RAII Guards
//!
//! Each localized variable has a corresponding `Guard` type that pushes on
//! creation and pops on `Drop`. This prevents leaked state on early returns
//! or panics:
//!
//! ```rust,no_run
//! # use latexml_core::common::local_assignments::*;
//! # use latexml_core::token::Token;
//! # fn example(token: Token) {
//! let _guard = local_current_token_guard(token);
//! // ... do work ...
//! // guard auto-pops when _guard goes out of scope
//! # }
//! ```
//!
//! The explicit `set_*/expire_*` pairs still exist for cases where
//! RAII doesn't fit (e.g., cross-function push/pop with different lifetimes).

use std::{cell::RefCell, rc::Rc};

use once_cell::sync::Lazy;

use crate::{
  Digested, alignment::template::Template, definition::conditional::IfFrame, token::Token,
};

/// These are fields realized via Perl's "local" mechanism in LaTeXML,
/// but (for now) require explicit "expire" calls in Rust.
/// Ideally their ergonomics gets improved, or we gradually phase them out.
#[derive(Debug, Default)]
pub struct Localized {
  dual_branch:       Vec<&'static str>,
  if_frames:         Vec<Option<Rc<RefCell<IfFrame>>>>,
  current_token:     Vec<Token>,
  align_group_count: Vec<i32>, // was $LaTeXML::ALIGN_STATE
  reading_alignment: Vec<Digested>,
  build_template:    Vec<Template>,
  unlocked:          Vec<bool>,
}

macro_rules! locals {
  () => {
    (*LOCALIZED_VARS).borrow()
  };
}
macro_rules! locals_mut {
  () => {
    (*LOCALIZED_VARS).borrow_mut()
  };
}

#[thread_local]
static LOCALIZED_VARS: Lazy<RefCell<Localized>> = Lazy::new(|| RefCell::new(Localized::default()));

/// Reset all localized variables to their default state.
/// Must be called between conversion runs to prevent state pollution.
pub fn initialize_localized() { *locals_mut!() = Localized::default(); }

/// sets a (originally Perl-local) `IfFrame` that needs to be manually expired.
pub fn set_ifframe(if_frame: Option<Rc<RefCell<IfFrame>>>) {
  locals_mut!().if_frames.push(if_frame);
}

/// retrieves the most recent (originally Perl-local) `IfFrame`
pub fn get_ifframe() -> Option<Rc<RefCell<IfFrame>>> {
  match locals!().if_frames.last() {
    Some(Some(frame)) => Some(Rc::clone(frame)),
    _ => None,
  }
}
/// expires the most recent (originally Perl-local) `IfFrame`
pub fn expire_ifframe() { locals_mut!().if_frames.pop(); }
/// localizes a new current token. see `Stomach::invoke_token`
pub fn local_current_token(token: Token) { locals_mut!().current_token.push(token); }
/// expires the most recent (localized) current token.
pub fn expire_current_token() { locals_mut!().current_token.pop(); }
/// gets the (localized) current token
pub fn get_current_token() -> Option<Token> { locals!().current_token.last().cloned() }

/// sets the (localized) flag for "dual branch"
pub fn set_dual_branch(mode: &'static str) { locals_mut!().dual_branch.push(mode); }
/// expire (localized) flag for "dual branch"
pub fn expire_dual_branch() { locals_mut!().dual_branch.pop(); }
/// get the current value for "dual branch"
pub fn get_dual_branch() -> Option<&'static str> { locals!().dual_branch.last().cloned() }

pub fn increment_align_group_count() {
  let mut locals = locals_mut!();
  match locals.align_group_count.last_mut() {
    Some(v) => *v += 1,
    None => locals.align_group_count.push(1),
  }
}
pub fn decrement_align_group_count() {
  let mut locals = locals_mut!();
  match locals.align_group_count.last_mut() {
    Some(v) => *v -= 1,
    None => locals.align_group_count.push(-1),
  }
}

pub fn state_is_unlocked() -> bool { locals!().unlocked.last().copied().unwrap_or(false) }
pub fn local_state_unlocked(v: bool) { locals_mut!().unlocked.push(v); }
pub fn expire_state_unlocked() { locals_mut!().unlocked.pop(); }

pub fn align_group_count() -> i32 {
  locals!()
    .align_group_count
    .last()
    .copied()
    .unwrap_or_default()
}
pub fn set_align_group_count(v: i32) {
  match locals_mut!().align_group_count.last_mut() {
    Some(gc) => {
      *gc = v;
    },
    _ => {
      locals_mut!().align_group_count.push(v);
    },
  }
}
pub fn local_align_group_count(v: i32) { locals_mut!().align_group_count.push(v); }
pub fn expire_align_group_count() -> Option<i32> { locals_mut!().align_group_count.pop() }

pub fn get_reading_alignment() -> Option<Digested> { locals!().reading_alignment.last().cloned() }
pub fn has_reading_alignment() -> bool { !locals!().reading_alignment.is_empty() }
pub fn local_reading_alignment(alignment: &Digested) {
  locals_mut!().reading_alignment.push(alignment.clone());
}
pub fn expire_reading_alignment() -> Option<Digested> { locals_mut!().reading_alignment.pop() }

pub fn local_build_template(template: Template) { locals_mut!().build_template.push(template); }
pub fn set_build_template(template: Template) {
  *locals_mut!()
    .build_template
    .last_mut()
    .expect("set_build_template should not be called before the first local_build_template") =
    template;
}
pub fn take_build_template() -> Option<Template> { locals_mut!().build_template.pop() }

pub fn with_current_build_template<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(Option<&mut Template>) -> R {
  caller(locals_mut!().build_template.last_mut())
}

// ============================================================
// RAII Guards — auto-restore on Drop
// ============================================================

/// Generic RAII guard that calls an expire function on drop.
/// Zero-cost when the expire function is a simple fn pointer.
pub struct LocalGuard {
  expire: fn(),
}
impl Drop for LocalGuard {
  fn drop(&mut self) { (self.expire)(); }
}

/// Push a localized current token; returns guard that pops on drop.
pub fn local_current_token_guard(token: Token) -> LocalGuard {
  local_current_token(token);
  LocalGuard { expire: expire_current_token }
}

/// Push a localized if-frame; returns guard that pops on drop.
pub fn local_ifframe_guard(frame: Option<Rc<RefCell<IfFrame>>>) -> LocalGuard {
  set_ifframe(frame);
  LocalGuard { expire: expire_ifframe }
}

/// Push a localized dual branch; returns guard that pops on drop.
pub fn local_dual_branch_guard(mode: &'static str) -> LocalGuard {
  set_dual_branch(mode);
  LocalGuard { expire: expire_dual_branch }
}

/// Push a localized align group count; returns guard that pops on drop.
pub fn local_align_group_count_guard(v: i32) -> LocalGuard {
  local_align_group_count(v);
  LocalGuard {
    expire: || {
      expire_align_group_count();
    },
  }
}

/// Push a localized state unlocked flag; returns guard that pops on drop.
pub fn local_state_unlocked_guard(v: bool) -> LocalGuard {
  local_state_unlocked(v);
  LocalGuard { expire: expire_state_unlocked }
}

/// Push a localized reading alignment; returns guard that pops on drop.
pub fn local_reading_alignment_guard(alignment: &Digested) -> LocalGuard {
  local_reading_alignment(alignment);
  LocalGuard {
    expire: || {
      expire_reading_alignment();
    },
  }
}

// ============================================================
// Unit tests
// ============================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_dual_branch_push_pop() {
    initialize_localized();
    assert_eq!(get_dual_branch(), None);
    set_dual_branch("true");
    assert_eq!(get_dual_branch(), Some("true"));
    set_dual_branch("false");
    assert_eq!(get_dual_branch(), Some("false"));
    expire_dual_branch();
    assert_eq!(get_dual_branch(), Some("true"));
    expire_dual_branch();
    assert_eq!(get_dual_branch(), None);
  }

  #[test]
  fn test_dual_branch_guard_auto_restore() {
    initialize_localized();
    assert_eq!(get_dual_branch(), None);
    {
      let _guard = local_dual_branch_guard("guarded");
      assert_eq!(get_dual_branch(), Some("guarded"));
    }
    // Guard dropped — value restored
    assert_eq!(get_dual_branch(), None);
  }

  #[test]
  fn test_align_group_count() {
    initialize_localized();
    assert_eq!(align_group_count(), 0);
    local_align_group_count(10);
    assert_eq!(align_group_count(), 10);
    increment_align_group_count();
    assert_eq!(align_group_count(), 11);
    decrement_align_group_count();
    assert_eq!(align_group_count(), 10);
    expire_align_group_count();
    assert_eq!(align_group_count(), 0);
  }

  #[test]
  fn test_align_group_count_guard() {
    initialize_localized();
    assert_eq!(align_group_count(), 0);
    {
      let _guard = local_align_group_count_guard(42);
      assert_eq!(align_group_count(), 42);
      increment_align_group_count();
      assert_eq!(align_group_count(), 43);
    }
    // Guard dropped — outer value restored
    assert_eq!(align_group_count(), 0);
  }

  #[test]
  fn test_state_unlocked() {
    initialize_localized();
    assert!(!state_is_unlocked());
    local_state_unlocked(true);
    assert!(state_is_unlocked());
    {
      let _guard = local_state_unlocked_guard(false);
      assert!(!state_is_unlocked());
    }
    assert!(state_is_unlocked());
    expire_state_unlocked();
    assert!(!state_is_unlocked());
  }
}
