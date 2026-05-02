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
#[ignore = "math0610119: \\sb in amsppt \\Refs/\\paper context — see project_math0610119_sb_amsppt_refs.md"]
fn sb_in_amsppt_refs() {
  // math0610119 — `\sb` (Let to T_SUB) is not invokable inside
  // amsppt's `\@bibfield XUntil:\@end@bibfield` field-capture path.
  // Direct `$\sb{0}$` works (Let in plain_constructs is reached);
  // the bug is at invocation time in build_invocation when
  // `lookup_definition` for a Let-to-token alias returns None
  // during the bibfield XUntil expansion.
  let (code, status) = run_trip("sb_in_amsppt_refs");
  assert_eq!(
    code, 0,
    "sb_in_amsppt_refs expected clean conversion, got status={status:?}"
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
