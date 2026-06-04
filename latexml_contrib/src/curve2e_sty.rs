//! curve2e.sty — extensions of the `picture` env (Bezier curves,
//! arrows, vector drawings) by Claudio Beccari.
//!
//! curve2e raw-loads into pictex-style territory: `\Dir@line`,
//! `\strokepath`, `\d@mX`, `\d@mY`, `\originalmoveto`,
//! `\pIIe@lineto` and a `\the\edef` pattern that our engine
//! rejects with "You can't use \edef after \the". Witness paper
//! arXiv:1408.2108 — amsart + curve2e (LOADED BUT UNUSED in the
//! body!) → 100+ errors + fatal. Perl converts the same input
//! with 26 warnings.
//!
//! Match Perl: stub the package as a no-op shell so the raw
//! .sty is never loaded. We provide the `picture` extensions as
//! pass-throughs (no fancy curves, but no crashes either). If a
//! paper uses `\Curve` / `\Arc` heavily the visual fidelity
//! suffers, but for the common case (loaded-but-unused) the
//! document converts cleanly.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "curve2e.sty",
    "curve2e.sty is minimally stubbed — Bezier curve / vector picture extensions are no-ops."
  );

  // curve2e.sty L16: `\RequirePackage{graphicx,color}` — these are the
  // package's *unconditional* hard dependencies (the curve/vector
  // machinery on lines 17-21 is what we stub out as no-ops below, but
  // graphicx+color are user-visible and papers rely on curve2e to pull
  // them in). Perl has no curve2e binding, so it raw-loads the real .sty
  // and executes this `\RequirePackage`, defining `\color`/`\definecolor`
  // etc. Our no-op stub previously omitted it, so a paper that uses
  // `\definecolor` *without* its own `\usepackage{color}` — relying on
  // curve2e to supply it — left `\definecolor` undefined where Perl is
  // clean. Witness 1810.10468 (ieeeconf + curve2e, `\definecolor{rouge}…`
  // with no explicit color load): RUST 1 (`undefined:\definecolor`) /
  // PERL 0 of that error. graphicx is idempotent if already loaded.
  RequirePackage!("graphicx");
  RequirePackage!("color");

  // curve2e exports — silently consume their arguments. None of
  // these have a faithful HTML rendering anyway.
  def_macro_noop("\\Curve")?;
  def_macro_noop("\\CbezierTo")?;
  def_macro_noop("\\Arc")?;
  def_macro_noop("\\VectorArc")?;
  def_macro_noop("\\Dashline")?;
  def_macro_noop("\\Dotline")?;
  def_macro_noop("\\Vector")?;
  def_macro_noop("\\VECTOR")?;
  def_macro_noop("\\polyvector")?;
  def_macro_noop("\\GraphLine")?;
  def_macro_noop("\\GraphGrid")?;
  // \Pbox and \Pnode-style — leave \put untouched (defined in TeX_Picture);
  // these are paper-local extensions that surface in <1% of curve2e papers.
});
