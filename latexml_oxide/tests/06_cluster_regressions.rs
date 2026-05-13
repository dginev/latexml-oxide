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
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert(source.to_string());
  assert!(
    r.result.is_some(),
    "{source}: conversion produced no result"
  );
  // Count inline `Error:<class>:` markers (parity_check.sh's lax pattern,
  // see feedback_strict_vs_lax_error_grep.md). Errors are emitted INLINE
  // within `(Building...Error:..)` envelopes, not at line starts.
  let n_errors = r
    .log
    .match_indices("Error:")
    .filter(|(i, _)| {
      let tail = &r.log.as_bytes()[*i + 6..];
      let n_class = tail.iter().take_while(|b| b.is_ascii_lowercase()).count();
      n_class > 0 && tail.get(n_class) == Some(&b':')
    })
    .count();
  assert_eq!(
    n_errors, 0,
    "{source}: expected 0 errors but log contained {n_errors} Error:<class>: markers (status_code={})",
    r.status_code
  );
  assert!(
    r.status_code <= 1,
    "{source}: status_code {} (expected 0/1), status={:?}",
    r.status_code,
    r.status
  );
}

#[test]
fn cluster_nbsp_csname() { convert_clean("tests/cluster_regressions/nbsp_csname.tex"); }

#[test]
fn cluster_at_ifundefined() { convert_clean("tests/cluster_regressions/at_ifundefined.tex"); }

#[test]
fn cluster_setdec_dec() { convert_clean("tests/cluster_regressions/setdec_dec.tex"); }

#[test]
fn cluster_cite_uppercase() { convert_clean("tests/cluster_regressions/cite_uppercase.tex"); }

/// Twemoji-style csname construction with accent macros (`\'`, `\^`, `\~`)
/// and `\textquoteright` apostrophe — must produce 0 errors after the
/// csname-stream soft-substitute fixes for `\lx@applyaccent`, the canonical
/// `\text…` primitives, and the NFSS `\<encoding>\i`/`\j` glyphs.
/// Pinned by stage-1..3 of the 100k warning corpus (arXiv:2603.22193,
/// 2603.23433, 2604.20621 — twemoji St. Barthélemy / Côte d'Ivoire / São Tomé).
#[test]
fn cluster_csname_accent() { convert_clean("tests/cluster_regressions/csname_accent.tex"); }
