//! mhsetup.sty — internal helpers for mathtools (`\MH_let:NwN`,
//! `\MH_new_boolean:n`, etc.). Originally part of mathtools.
//!
//! The package's `\MHInternalSyntaxOn` flips `:` and `_` to
//! letter catcode so `\MH_*:` style names parse as single CSes,
//! and queues `\AtEndOfPackage{\MHInternalSyntaxOff}` to flip
//! them back. Our `\AtEndOfPackage` hook fires post-raw-load
//! but its catcode-restore assignments don't reach the parent
//! frame in time for the next `\usepackage{tikz}` invocation
//! to see `:` at catcode 12 — pgfutil-common.tex line 174
//! `\def\:{\pgfutil@xifnch} \expandafter\gdef\: {\futurelet...}`
//! requires `\:` to be parsed as a control symbol (catcode 12
//! colon), and parses it as a control word when colon is still
//! a letter. The redefined `\:` then no longer matches the
//! `\expandafter\gdef\:` invocation, leaving `\pgfutil@xifnch`
//! undefined.
//!
//! Fix: raw-load mhsetup.sty as usual, then EXPLICITLY restore
//! the `:` and `_` catcodes globally to their LaTeX defaults.
//! This is a defensive belt-and-suspenders on top of the
//! `\AtEndOfPackage` hook — if the hook does fire, the
//! re-assignment is a no-op; if it doesn't, we're safe.
//!
//! Witness: arXiv:1207.2132 (mhsetup + tikz → \pgfutil@xifnch
//! cascade, fatal).

use crate::prelude::*;
use latexml_core::token::Catcode;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("mhsetup", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // Defensive catcode restore — see file docstring above for why.
  // Defaults: `:` is OTHER (12), `_` is SUB (8).
  // NOTE: this only fully helps when the next `\usepackage{tikz}` is
  // processed AFTER our binding returns. In some preambles tikz is
  // pulled in during InputDefinitions's input-loop processing (mhsetup
  // chained with mathtools or similar), in which case the catcode reset
  // arrives too late — that case still needs an upstream fix to
  // `\AtEndOfPackage` hook timing or to the InputDefinitions
  // re-entry guarantees.
  latexml_core::state::assign_catcode(':', Catcode::OTHER, Some(latexml_core::state::Scope::Global));
  latexml_core::state::assign_catcode('_', Catcode::SUB, Some(latexml_core::state::Scope::Global));
});
