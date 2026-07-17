//! Reusable engine presets that don't depend on the test harness.
//!
//! Extracted from `util/test.rs` (audit DEP-02, 2026-05-18) so the
//! `latexmlmath_oxide` standalone binary can build without the test
//! harness's `glob`/`phf` dependencies. The two helpers here have no
//! ties to the harness machinery — they just construct a minimal
//! `Core` and tokenize a single inline formula.

use std::rc::Rc;

use latexml_core::{Core, CoreOptions, document::Document, state};
use latexml_math_parser::node_to_grammar_lexemes;
use libxml::tree::Node;

use crate::core_interface::DigestionAPI;

/// Provide a default `Core` engine preloaded with `article.cls`
/// and `amsmath.sty` — the minimum needed to digest most formulae.
pub fn new_test_engine() -> Core {
  let core_engine = Core::new(CoreOptions {
    // `LaTeX.pool` FIRST — a class preload must not be the thing that drags the
    // pool in. `\@pushfilename` changes meaning when the pool (and the kernel dump
    // behind it) loads: preloading `article.cls` alone pushes the class's filename
    // with the pre-pool `\@pushfilename` (which never touches
    // `\g__hook_name_stack_seq`), then pops it with the real expl3 `\@popfilename`
    // that the pool installed — popping a seq that only ever saw the *inner*
    // packages' pushes. Hence "LaTeX hooks Error: Extra \PopDefaultHookLabel" on
    // every latexmlmath_oxide run. Same order ar5iv's preload list uses.
    // See SYNC_STATUS "`--preload=<cls>` hook-stack imbalance" — the underlying
    // ordering bug is still open; this preset just stops provoking it.
    preload: Some(
      ["LaTeX.pool", "article.cls", "amsmath.sty"]
        .map(|x| x.to_string())
        .to_vec(),
    ),
    verbosity: Some(-2),
    search_paths: None,
    nomathparse: Some(true),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  // Shared model loader — see crate::load_latexml_default_model.
  crate::load_latexml_default_model();
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  state::add_binding_names(latexml_package::binding_names());
  core_engine
}

/// Simple tokenization of a single formula, without any custom preloads
/// beyond latex and amsmath.
pub fn lex_single_tex_formula(
  tex: &str,
  latexml: &mut Core,
) -> (Vec<String>, Vec<Node>, Option<Node>, Document) {
  let xml_result = latexml.convert_file(format!("literal:\\[ {tex} \\]"));
  assert!(xml_result.is_ok(), "{:?}", xml_result.err());
  let mut doc = xml_result.unwrap();

  match doc.findnode("//*[local-name()='XMath']", None) {
    Some(math) => {
      let mut idx = 0;
      let (lexemes, nodes) = node_to_grammar_lexemes(&math, &mut idx);
      (lexemes, nodes, Some(math), doc)
    },
    None => (Vec::new(), Vec::new(), None, doc),
  }
}
