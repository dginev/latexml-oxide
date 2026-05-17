//! Trip tests — minimal repros from sandbox-canvas regressions.
//!
//! Each test runs `latexml_oxide` against a single `tests/trip/<name>.tex`
//! min-repro and asserts an expected error count. Currently-failing
//! repros are marked `#[ignore]` so they're red/green TDD targets:
//! `cargo test --tests --ignored` runs them; flipping a fix flips
//! the test from `ignored` to `passing` (remove the `#[ignore]`).
//!
//! The repros are derived from canvas papers in
//! `~/data/100k_noproblem_sandbox/`; each test file's leading comment
//! cites the witness arXiv-id.
//!
//! Repros that depend on external `.sty`/`.cls` files (paper-local
//! tarball contents, e.g. `mn1.sty`, `aprim.sty`, `PASJ95.STY`) are
//! intentionally excluded — those are validated end-to-end via the
//! 100k canvas sweep, not the in-tree test suite.

use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};
use std::rc::Rc;
use std::sync::Once;

static LOGGER_INIT: Once = Once::new();

fn run_trip(stem: &str) -> (usize, String) {
  // `log::set_logger` panics on re-init; cargo runs all tests in the
  // same process, so we guard initialization with std::sync::Once.
  LOGGER_INIT.call_once(|| {
    let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  });
  let source = format!("tests/trip/{stem}.tex");
  let config = Config {
    format: OutputFormat::HTML5,
    // Mirror the CLI binaries: register latexml_contrib's dispatch
    // so contrib bindings (ascmac, jmlr, etc.) resolve in tests.
    // Otherwise tests using a contrib package would error with
    // "Can't find binding or file for 'X.sty'".
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  };
  let mut converter = Converter::from_config(config);
  converter
    .initialize_session()
    .expect("can initialize session");
  let response = converter.convert(source);
  (response.status_code, response.status)
}

#[test]
fn iopart_la_user_override() {
  // hep-ph0404036 — fixed by commit bf3397078 (drop speculative
  // \la→\lesssim binding from iopart_support_sty.rs). Without the
  // fix, Rust pre-binds `\la` so user `\newcommand\la{\langle}`
  // is silently ignored, and `\la` later expands to undefined
  // `\lesssim`. With the fix, the user override sticks.
  let (code, status) = run_trip("iopart_la_user_override");
  assert_eq!(
    code, 0,
    "iopart_la_user_override expected clean conversion, got status={status:?}"
  );
}

#[test]
fn sb_in_amsppt_refs() {
  // math0610119 — `\sb` (Let to T_SUB) inside amsppt's
  // `\@bibfield XUntil:\@end@bibfield` field-capture path. Fixed by
  // gating XUntil's "re-Invocation-emit" arm on `lookup_meaning`
  // returning a genuine Stored::Expandable (not a Token-alias
  // synthetically wrapped as Expandable by lookup_definition_stored).
  // Token aliases now pass through unchanged for the digester to
  // resolve via the meaning lookup. See base_parameter_types.rs XUntil.
  let (code, status) = run_trip("sb_in_amsppt_refs");
  assert_eq!(
    code, 0,
    "sb_in_amsppt_refs expected clean conversion, got status={status:?}"
  );
}

#[test]
fn halign_body_implicit_cr() {
  // `\let\rowEnd=\cr` followed by `\halign{...\rowEnd ... \rowEnd}`.
  // Before the fix in `is_implicit_cr` (tex_tables.rs), the preamble
  // parser only recognised implicit \cr when its meaning was
  // Stored::Token(\cr). But `\let \rowEnd \cr` against a Constructor
  // `\cr` produces Stored::Constructor — the parser missed it, ate
  // the entire body as template, and emitted no tabular silently
  // (code == 0, empty <document/>). So a plain "no errors" assertion
  // is insufficient; we also assert the output XML contains tabular
  // rows by converting through the lower-level API and inspecting
  // the produced document.
  LOGGER_INIT.call_once(|| {
    let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  });
  let config = Config {
    format: OutputFormat::XML,
    ..Config::default()
  };
  let mut converter = Converter::from_config(config);
  converter
    .initialize_session()
    .expect("can initialize session");
  let response = converter.convert("tests/trip/halign_body_implicit_cr.tex".to_string());
  assert_eq!(
    response.status_code, 0,
    "halign_body_implicit_cr expected clean conversion, got status={:?}",
    response.status
  );
  let xml = response.result.as_deref().unwrap_or("");
  // Two rows: one for `a&b\rowEnd`, one for `c&d\rowEnd`.
  // (The trailing `\rowEnd` ends the alignment; no third row.)
  // `<tr>` may carry attributes (`<tr xml:id=...>`) or none (`<tr>`),
  // so accept both forms.
  let tr_count = xml.matches("<tr>").count() + xml.matches("<tr ").count();
  assert_eq!(
    tr_count, 2,
    "expected 2 <tr> rows from `\\let\\rowEnd=\\cr` halign body, got {tr_count}; xml = {xml}"
  );
}

#[test]
fn psfig_via_compat_loadpackages() {
  // Baseline regression test for the `\compat@loadpackages` option
  // forwarding path: `\documentstyle[epsfig]{article}` → article.cls
  // populates `@unusedoptionlist` with `epsfig` → `\compat@loadpackages`
  // calls `\RequirePackage{epsfig}` → `\psfig` defined.
  //
  // The astro-ph0002213 RESIDUAL (Perl=0, Rust=1 `\psfig`) is
  // observable only with the paper-local `mn1.sty` fallback chain
  // (`\documentstyle[epsfig]{mn1}` → mn.cls.ltxml → article.cls
  // where mn.cls's PassOptions plumbing fails to populate
  // @unusedoptionlist). The min repro requires the real arxiv
  // tarball, so this test only protects the option-passthrough
  // baseline. See project_astro_ph0002213_psfig_residual.md for
  // the deeper paper-local-fallback bug.
  let (code, status) = run_trip("psfig_via_compat_loadpackages");
  assert_eq!(
    code, 0,
    "psfig_via_compat_loadpackages expected clean conversion, got status={status:?}"
  );
}

#[test]
fn ieeetran_newlineauthors() {
  // Round-33 fix (commit 6be8b2e01e). User \newcommand{\newlineauthors}
  // installing the IEEE halign unbalanced-pair recipe must be silently
  // dropped on the floor — same as \linebreakand — so the surrounding
  // \author body doesn't break the frontmatter digest.
  let (code, status) = run_trip("ieeetran_newlineauthors");
  assert_eq!(
    code, 0,
    "ieeetran_newlineauthors expected clean conversion, got status={status:?}"
  );
}

#[test]
fn ascmac_itembox() {
  // Round-33 fix (commit f5fa292f89). ascmac.sty {itembox}/{screen}/
  // {shadebox} stub bindings — papers using these Japanese boxed envs
  // must convert cleanly (driver 2601.09339).
  let (code, status) = run_trip("ascmac_itembox");
  assert_eq!(
    code, 0,
    "ascmac_itembox expected clean conversion, got status={status:?}"
  );
}

#[test]
fn quantumarticle_bare_acknowledgments() {
  // Round-33 fix (commit d3e220f40c). REVTeX-style bare \acknowledgments
  // (no \begin/\end, just on its own line followed by body + \bibliography)
  // must open <ltx:acknowledgements> with auto_close handling the implicit
  // close — no phantom-close errors at \end{document}.
  let (code, status) = run_trip("quantumarticle_bare_acknowledgments");
  assert_eq!(
    code, 0,
    "quantumarticle_bare_acknowledgments expected clean conversion, got status={status:?}"
  );
}
