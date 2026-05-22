//! mathenv.sty — math-environment extensions by Mark Wooding (1996).
//!
//! mathenv.sty is shipped as part of the `mdwtools` bundle and
//! `\RequirePackage{mdwtab}` first (which we no-op-stub in
//! `mdwtab_sty.rs`). It then redefines `\eqnarray` and provides
//! display-math helpers (`\dsp@start...`, `\eqnumber{}`).
//!
//! Perl LaTeXML has no `mathenv.sty.ltxml`; with its default
//! `INCLUDE_STYLES=false` the raw `mathenv.sty` is NOT loaded —
//! Perl emits a "missing binding" warning and continues with the
//! binding-aware `\eqnarray`. The user's display-math then
//! renders cleanly, without mathenv's `\eqnumber` extension but
//! without any cascade either.
//!
//! Match Perl by stubbing as a no-op. Same pattern as
//! `mdwtab_sty.rs` / `delarray_sty.rs` / `trace_sty.rs`.
//!
//! Witness: canvas-3 stage-26 0910.3293.
use crate::prelude::*;

LoadDefinitions!({});
