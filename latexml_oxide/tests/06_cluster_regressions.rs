//! Cluster-regression integration tests.
//!
//! Pins the surpass-Perl wins from the post-100k cluster work
//! (NBSP, @ifundefined, setdec/dec, \CITE) as 0-error.
//! If a future change re-introduces the cluster errors, CI fails
//! before the PR can land.
use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

fn convert_clean(source: &str) {
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config { format: OutputFormat::HTML5, ..Config::default() };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(r.result.is_some(), "{source}: conversion produced no result");
  assert!(
    r.status_code <= 1,
    "{source}: status_code {} (expected 0/1), status={:?}",
    r.status_code, r.status
  );
}

#[test]
fn cluster_nbsp_csname() {
  convert_clean("tests/cluster_regressions/nbsp_csname.tex");
}

#[test]
fn cluster_at_ifundefined() {
  convert_clean("tests/cluster_regressions/at_ifundefined.tex");
}

#[test]
fn cluster_setdec_dec() {
  convert_clean("tests/cluster_regressions/setdec_dec.tex");
}

#[test]
fn cluster_cite_uppercase() {
  convert_clean("tests/cluster_regressions/cite_uppercase.tex");
}

/// `\emph{$$math$$}` triggers `Error:unexpected:_/^` because of
/// shared `\lx@dollar@default` logic in BOTH Perl and Rust: the
/// `$$`-as-display check requires `BOUND_MODE` to end with
/// "vertical", and inside `\emph{...}` BOUND_MODE is
/// "restricted_horizontal". This is SHARED-FAILURE with Perl,
/// NOT a Rust regression — verified 2026-05-03 via parity check
/// on 0705.0102 (R=Perl=36) and four other witnesses. Pinned as
/// `#[ignore]`d sentinel so future changes that "fix" this in Rust
/// (which would be Rust-beats-Perl divergence, not parity work)
/// can flip the flag to mark the win. See SYNC_STATUS.md Gate 2.A.
#[test]
#[ignore = "Phase B Gate 2.A — emph+$$ shared-failure with Perl, not a regression"]
fn cluster_emph_dollar_math() {
  convert_clean("tests/cluster_regressions/emph_dollar_math.tex");
}
