//! mhsetup.sty — internal helpers for mathtools (`\MH_let:NwN`,
//! `\MH_new_boolean:n`, etc.). Originally part of mathtools.
//!
//! The package's `\MHInternalSyntaxOn` flips `:` and `_` to letter
//! catcode so `\MH_*:` style names parse as single CSes, and queues
//! `\AtEndOfPackage{\MHInternalSyntaxOff}` to flip them back. Our
//! `\AtEndOfPackage` hook fires post-raw-load but its catcode-restore
//! assignments arrive after the next `\usepackage{tikz}` has already
//! started loading pgfutil-common.tex (where `\:` must be a control
//! SYMBOL — colon at catcode 12), so the redefined `\:` no longer
//! matches the `\expandafter\gdef\:` invocation and
//! `\pgfutil@xifnch` is left undefined.
//!
//! Fix: raw-load mhsetup.sty as usual via InputDefinitions, then
//! EXPLICITLY restore the `:` and `_` catcodes globally to their
//! LaTeX defaults. Defensive belt-and-suspenders on top of the
//! `\AtEndOfPackage` hook.
//!
//! Witness: arXiv:1207.2132 (mhsetup + tikz → `\pgfutil@xifnch`
//! cascade, fatal). Note: this fix only helps when the
//! `\usepackage{tikz}` is in a SEPARATE \usepackage line; if it
//! is part of a `\usepackage{mhsetup,…}` list its load is
//! interleaved with mhsetup's load via the digest auto-pop
//! upstream of this binding, which is a separate issue.

use crate::prelude::*;
use latexml_core::token::Catcode;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("mhsetup", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  latexml_core::state::assign_catcode(':', Catcode::OTHER, Some(latexml_core::state::Scope::Global));
  latexml_core::state::assign_catcode('_', Catcode::SUB, Some(latexml_core::state::Scope::Global));
});
