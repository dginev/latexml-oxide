//! Forced-streaming sweep over `tests/grouping` — the last fixture directory
//! the `114_streaming_*` family did not cover.
//!
//! It needs its own dispatcher: `scopemacro.latexml` is a TEST helper, not a
//! contrib entry, so the plain `sweep_dir` would convert that fixture without
//! its bindings and prove nothing.
//!
//! Scope note, so this file is not mistaken for more than it is: it does NOT
//! guard `Document::lookup_rewrite_label`'s shared-index fallback
//! (`rewrite_labels_shared`). `scopemacro.tex` scopes a *macro*
//! (`DefMacro!(… scope => "label:labelled")`), which is State scoping — a
//! different mechanism from the rewrite-rule `label:` scope that
//! `rewrite.rs:274` resolves. Verified by red-test: disabling the shared
//! fallback outright leaves this sweep green.
//!
//! A rewrite-rule `label:` scope needs a declaration made while a `\label`'s
//! scope is active (`latex_constructs.rs:8237` turns `LABEL:x` into
//! `label:x`), and NO fixture in the tree pairs `\lxDeclare` with `\label` —
//! so that path is untested in eager mode too. Pre-existing gap, worth its own
//! fixture.

// `helpers` carries `LoadDefinitions!`-based binding sources, so it needs the
// same macro preamble the eager grouping suite (`12_grouping.rs`) has.
#[macro_use]
extern crate latexml_engine;
#[macro_use]
extern crate latexml_codegen;
extern crate latexml_contrib;
extern crate latexml_package;

mod helpers;
mod streaming_sweep;

use latexml_core::common::error::Result;
use streaming_sweep::sweep_dir_with;

fn grouping_tests_dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    "scopemacro.latexml" => Some(helpers::scopemacro_src::load_definitions()),
    _ => latexml_contrib::dispatch(filename),
  }
}

#[test]
fn streaming_matches_eager_on_grouping() {
  sweep_dir_with("tests/grouping", grouping_tests_dispatch);
}
