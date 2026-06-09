//! pgfplots `bar shift` + `symbolic x coords` units-flag regression.
//!
//! Root cause (2110.14597, +12 Rust-only errors → 0): the native pgfmath
//! parser (`pgfmath_code_tex.rs`) dropped the `\ifpgfmathunitsdeclared` flag
//! when evaluating a user-declared 0-arg pgfmath function whose body itself
//! parses a unit'd value. pgfplots' `\pgfplotbarwidth` resolves through the
//! `pgfplotsbarwidthgeneric` pseudo-constant, whose body does
//! `\pgfmathparse{<bar width>pt}` — that nested parse sets the global units
//! flag, but the outer parse clobbered it back to false on exit, so
//! `\pgfplots@bar@mathparse@` mis-classified the bar shift as a unitless
//! coordinate and fed it through the symbolic x-coord trafo:
//!   `Package pgfplots Error: ... \pgfplots@loc@TMPa has not been defined`.
//!
//! Conditional: needs the kernel dump (so expl3/pgf load cleanly, not the
//! degraded raw path) AND pgf/pgfplots installed in the host TeX tree.
use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};
use std::process::Command;

/// True iff a year-versioned latex kernel dump is present in the dev tree.
/// Without it the engine raw-loads `expl3-code.tex` (degraded mode) and the
/// error landscape is dominated by unrelated raw-load cascades — this test
/// would be measuring the wrong thing, so we skip.
fn dump_available() -> bool {
  let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/dumps");
  std::fs::read_dir(dir)
    .map(|rd| {
      rd.filter_map(|e| e.ok()).any(|e| {
        let n = e.file_name();
        let n = n.to_string_lossy();
        n.starts_with("latex.") && n.ends_with(".dump.txt")
      })
    })
    .unwrap_or(false)
}

/// True iff `kpsewhich` resolves the named file in the host TeX tree.
fn kpse_has(file: &str) -> bool {
  Command::new("kpsewhich")
    .arg(file)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// Count inline `Error:<class>:` markers (same lax pattern as
/// `06_cluster_regressions.rs::convert_clean`).
fn error_count(log: &str) -> usize {
  log
    .match_indices("Error:")
    .filter(|(i, _)| {
      let tail = &log.as_bytes()[*i + 6..];
      let n_class = tail.iter().take_while(|b| b.is_ascii_lowercase()).count();
      n_class > 0 && tail.get(n_class) == Some(&b':')
    })
    .count()
}

#[test]
fn pgfplots_bar_shift_symbolic_coords_units_flag() {
  if !dump_available() {
    eprintln!(
      "SKIP pgfplots_bar_shift_symbolic_coords_units_flag: no latex kernel dump \
       in resources/dumps/ (run tools/make_formats.sh)"
    );
    return;
  }
  if !kpse_has("pgfplots.sty") || !kpse_has("pgf.sty") {
    eprintln!(
      "SKIP pgfplots_bar_shift_symbolic_coords_units_flag: pgf/pgfplots not \
       installed in the host TeX tree"
    );
    return;
  }

  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let cfg = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut c = Converter::from_config(cfg);
  c.initialize_session().expect("initialize");
  let r = c.convert("tests/cluster_regressions/pgfplots_symbolic_bar_units.tex".to_string());
  assert!(r.result.is_some(), "conversion produced no result");

  let n = error_count(&r.log);
  assert_eq!(
    n, 0,
    "expected 0 errors (Perl converts cleanly) but got {n} — the pgfmath \
     units-declared flag is being dropped through the pgfplotsbarwidthgeneric \
     pseudo-constant eval, mis-routing `bar shift` through the symbolic-coord \
     trafo (status_code={})",
    r.status_code
  );
}
