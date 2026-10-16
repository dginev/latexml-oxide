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
  // tabls.sty:16-18 its parameters, the minimum gaps between table lines, array lines and a rule and the text: a
  // document sets them (`\tablinesep=2pt`; 22 papers, 1011.3468, 1101.2746, 1205.3429, 1305.1117, hep-th0309263).
  // The rows' spacing is print layout, so nothing reads them here. Guard:
  // `perfect_kernel_batch68j::tabls_line_separations_are_dimens`.
  RawTeX!(
    r"\newdimen\tablinesep \tablinesep=1pt \let\tablineskip=\tablinesep
\newdimen\arraylinesep \arraylinesep=1pt \let\arraylineskip=\arraylinesep
\newdimen\extrarulesep \extrarulesep=3pt"
  );
  // tabls's tabular customizations don't translate to HTML layout;
  // leave the kernel tabular machinery intact. Don't raw-load.
});
