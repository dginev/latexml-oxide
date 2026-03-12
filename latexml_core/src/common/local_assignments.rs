// We need RAII Guards!
// https://rust-unofficial.github.io/patterns/patterns/behavioural/RAII.html
// it would be massively superior to leverage Drop, compared to explicit `expire_*` calls

//! Perl-style local assignments (dynamic scope)
//!
//! This module can benefit from some thinking over and refactoring
//! in principle the "local" scope variables in Perl are a completely standalone feature
//! (it's global namespace value shadowing until scope expiration)
//! ... so the push/pop can be modeled with sentry values getting created and dropped (TODO)...

use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::rc::Rc;

use crate::Digested;
use crate::alignment::template::Template;
use crate::definition::conditional::IfFrame;
use crate::token::Token;

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
pub fn initialize_localized() {
  *locals_mut!() = Localized::default();
}

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
    None => {
      locals.align_group_count.push(1);
    },
  }
}
pub fn decrement_align_group_count() {
  let mut locals = locals_mut!();
  match locals.align_group_count.last_mut() {
    Some(v) => *v -= 1,
    None => {
      locals.align_group_count.push(-1);
    },
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
  if let Some(gc) = locals_mut!().align_group_count.last_mut() {
    *gc = v;
  } else {
    locals_mut!().align_group_count.push(v);
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
