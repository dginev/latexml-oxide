//! Shared helper: discard an environment body by reading tokens up to
//! matching `\end{kind}`. Mirrors the Perl `discard_env_body` closures
//! used in ar5iv-bindings/nicematrix / forest / diagrams / pb-diagram
//! stub packages.
//!
//! Perl reference (identical across those stubs):
//! ```perl
//! sub discard_env_body {
//!   my ($stomach, $kind) = @_;
//!   my $gullet = $stomach->getGullet;
//!   $stomach->bgroup;
//!   if (!$reported{$kind}) {
//!     $reported{$kind} = 1;
//!     Error('undefined', "{$kind}", $gullet,
//!       "$kind has no support, this is a stub binding."); }
//!   while (my $_ = $gullet->readUntil(T_CS('\end'))) {
//!     my $_drop_open = $gullet->readToken;
//!     my $env        = $gullet->readBalanced;
//!     last if ToString($env) eq $kind; }
//!   $stomach->egroup;
//!   return; }
//! ```

use latexml_package::prelude::*;
use std::cell::RefCell;
use rustc_hash::FxHashSet as HashSet;

thread_local! {
  /// One error per kind per conversion run, matching Perl's
  /// `our $reported{$kind}` cache inside each stub package.
  static REPORTED: RefCell<HashSet<String>> = RefCell::new(HashSet::default());
}

/// Read and discard tokens up to and including a matching `\end{kind}`.
/// Emits a one-time `Error("undefined", "{kind}", ...)` on the first
/// invocation per `kind`.
pub fn discard_env_body(kind: &str, source: &str) -> latexml_core::common::error::Result<()> {
  bgroup();
  let first_time = REPORTED.with(|cell| {
    let mut set = cell.borrow_mut();
    if set.contains(kind) {
      false
    } else {
      set.insert(kind.to_string());
      true
    }
  });
  if first_time {
    let obj = format!("{{{}}}", kind);
    let msg = format!(
      "{} has no support in {}, this is a stub binding.",
      kind, source
    );
    Error!("undefined", &obj, msg);
  }
  let end_delim = Tokens!(T_CS!("\\end"));
  loop {
    let _upto_end = gullet::read_until(&end_delim)?;
    let _drop_open = gullet::read_token()?;
    // require_open=false because `_drop_open` just consumed the `{` —
    // read_balanced should read the inside, not a second `{`. Mirrors
    // Perl's argless `$gullet->readBalanced` which assumes the `{` is
    // already open. Driver: 2402.09676 + nicematrix stub cascaded
    // "Expected opening '{'" because of the spurious require_open.
    let env = gullet::read_balanced(latexml_core::gullet::ExpansionLevel::Off, false, false)?;
    if env.to_string() == kind {
      break;
    }
  }
  egroup()?;
  Ok(())
}
