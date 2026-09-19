//! Stub for hld.cls (inferred from the colt/midl pattern — no host `hld.cls`
//! was on hand to confirm the exact wrapper; harmless when `\documentclass{hld}`
//! never appears).
//!
//!  (Highlights in Learning Dynamics and similar jmlr-derived
//! conference proceedings). Like colt/midl it does `\LoadClass{jmlr}` and
//! defines `\hldauthor` in its (OmniBus-skipped) cls body as `\author{#1}`.
//! Without the wrapper the jmlr `\addr Until:\lx@jmlr@endaddr` scanner runs to
//! EOF (`Fatal:Mouth:EoF`); routing `\hldauthor` through `\author` lays the
//! sentinel and bounds it.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("jmlr");
  DefMacro!("\\hldauthor{}", "\\author{#1}");
  DefMacro!("\\jmlrpages{}", ""); // jmlr.cls:559 is 1-arg; swallow it (no body leak)
});
