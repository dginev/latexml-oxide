//! tabls.sty — table layout tweaks (variable line height/depth).
//!
//! tabls redefines `\@array` to wrap its expansion with a
//! `\@unrecurse` cleanup that restores global table-layout
//! parameters. The redefinition lives inside `\@array`'s body, so
//! `\@unrecurse` only gets `\edef`'d at `\@array` invocation. Our
//! engine's tabular machinery bypasses the kernel `\@array` path,
//! so `\@unrecurse` is never defined — when the (also-redefined)
//! `\@xtabularcr` later calls `\@unrecurse`, it fires undefined.
//!
//! Witness arXiv:2003.12942 — `\usepackage{tabls}` + tabular env
//! emits `\@unrecurse undefined` once per table row. Perl LaTeXML
//! has no tabls binding and INCLUDE_STYLES=false skips raw load.
//!
//! Stub: pre-define `\@unrecurse` as `\relax` so the cleanup is
//! a no-op. The visual fidelity loss is minor: our tabular
//! rendering doesn't honour tabls's per-cell strut adjustments
//! anyway.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "tabls.sty",
    "tabls.sty is minimally stubbed — line-height tweaks are not honored; \\@unrecurse pre-defined as \\relax."
  );
  // Pre-define \@unrecurse before raw-load (defensive: in case
  // our array path never instantiates the inner \edef).
  Let!("\\@unrecurse", "\\relax");
  // tabls's tabular customizations don't translate to HTML layout;
  // leave the kernel tabular machinery intact. Don't raw-load.
});
