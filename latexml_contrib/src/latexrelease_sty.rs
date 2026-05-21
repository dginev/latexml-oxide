//! Stub for latexrelease.sty (LaTeX kernel rollback machinery).
//!
//! latexrelease.sty is a 17k-line LaTeX2e rollback package: papers
//! that need a specific historical kernel date
//! (`\RequirePackage[2020-02-02]{latexrelease}`) use it to re-bind
//! kernel macros to older variants. The raw sty defines
//! `\IncludeInRelease`/`\EndIncludeInRelease` block-pairs gated by
//! kernel-date comparisons; inside those blocks it does
//! `\let \@expl@str@if@eq@@nnTF \@undefined` and similar undef-then-
//! rebind dance. Our system does NOT track kernel dates, so it executes
//! every `\IncludeInRelease` block linearly — leading to undefined-CS
//! cascades during the rollback machinery itself.
//!
//! In our HTML conversion pipeline, kernel-date rollback is meaningless
//! (it controls pagination/font/parser internals that don't affect
//! semantic content). Stub the whole package as a no-op. Witness:
//! 2305.09466, 2305.12592, 2305.14116, 2305.16020, 2305.17060 (all
//! 5-error conversion_fatal from \@expl@str@if@eq@@nnTF cascade after
//! latexrelease.sty raw-load).
use latexml_package::prelude::*;

LoadDefinitions!({
  // No-op: skip rollback machinery entirely. If a future paper actually
  // depends on `\IncludeInRelease` / `\EndIncludeInRelease` macro-pairs
  // in user code (rather than just inside latexrelease.sty's own bundle),
  // we'd stub those here too.
});
