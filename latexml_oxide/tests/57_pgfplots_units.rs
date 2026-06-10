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
use latexml::util::test::{convert_fixture, dump_available, error_count, kpse_has};

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

  let r = convert_fixture("tests/cluster_regressions/pgfplots_symbolic_bar_units.tex");
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
