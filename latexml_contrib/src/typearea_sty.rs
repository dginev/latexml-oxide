//! No-op stub for typearea.sty (KOMA-Script page layout).
//!
//! typearea defines KOMA's typearea page-layout system; the user-facing
//! option is `DIV=<n>` (computed type-area divisions) and its variants.
//! All KOMA-Script page-layout settings are typesetting-only — XML/HTML
//! output never observes them — but our raw-load of typearea triggers
//! `\RequirePackage{scrkbase}` → `scrbase`, and scrbase's option
//! processing emits `Error:latex:\GenericError Package scrbase Error:
//! unknown option` when it can't parse `DIV=11` (the keyval setup
//! doesn't survive raw-load fully).
//!
//! Perl LaTeXML never observes this: its default TEXINPUTS excludes
//! `/usr/share/texlive`, so typearea.sty is reported as missing-file
//! and skipped, completing the conversion cleanly. Match that by
//! registering a no-op binding for typearea.sty.
//!
//! Witness clusters (CONVERR_5 .. CONVERR_8 with scrbase noise):
//! 1504.00554, 1502.06768, 1504.00666, and the ~6 R-stage papers in
//! the `Package scrbase Error: unknown option` cluster.

use latexml_package::prelude::*;

LoadDefinitions!({
  // `\areaset[BCOR]{width}{height}` — typearea's explicit page-area
  // setter. Pure typesetting, no semantic effect for XML output.
  // Witness: 1502.06768 uses `\areaset{...}{...}` to manually pin the
  // typearea instead of using `DIV=...`.
  def_macro_noop("\\areaset[]{}{}")?;
});
