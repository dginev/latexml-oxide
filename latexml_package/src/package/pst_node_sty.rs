//! pst-node.sty — PSTricks node connections (DVI-only)
//! Perl: pst-node.sty.ltxml — 557 lines
//! Node definition and connection macros for PSTricks diagrams.
//! DVI-only: all node/connection commands produce no visible output.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("pstricks");

  // Node definition macros — Perl L30-120
  DefMacro!("\\rnode[]{}{}", "#3");
  DefMacro!("\\Rnode[]{}{}", "#3");
  DefMacro!("\\pnode[]", "");
  DefMacro!("\\cnode[]{}", "");
  DefMacro!("\\Cnode[]{}", "");
  DefMacro!("\\circlenode[]{}{}", "#3");
  DefMacro!("\\ovalnode[]{}{}", "#3");
  DefMacro!("\\fnode OptionalMatch:* []{}", "");
  DefMacro!("\\dotnode OptionalMatch:* []{}", "");
  DefMacro!("\\trinode[]{}{}", "#3");
  DefMacro!("\\dianode[]{}{}", "#3");

  // Connection macros — Perl L130-350
  DefMacro!("\\ncline OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcline OptionalMatch:* []{}{}", "");
  DefMacro!("\\nccurve OptionalMatch:* []{}{}", "");
  DefMacro!("\\pccurve OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncbar OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncdiag OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncangle OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncangles OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncloop OptionalMatch:* []{}{}", "");
  DefMacro!("\\ncarc OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcbar OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcdiag OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcangle OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcangles OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcloop OptionalMatch:* []{}{}", "");
  DefMacro!("\\pcarc OptionalMatch:* []{}{}", "");

  // Label macros — Perl L360-450
  DefMacro!("\\naput OptionalMatch:* []{}", "#2");
  DefMacro!("\\nbput OptionalMatch:* []{}", "#2");
  DefMacro!("\\ncput OptionalMatch:* []{}", "#2");
  DefMacro!("\\nput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\lput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\aput OptionalMatch:* []{}", "#2");
  DefMacro!("\\bput OptionalMatch:* []{}", "#2");
  DefMacro!("\\mput OptionalMatch:* {}", "#1");
  DefMacro!("\\Lput OptionalMatch:* []{}{}", "#3");
  DefMacro!("\\Mput OptionalMatch:* {}", "#1");
  DefMacro!("\\Rput OptionalMatch:* []{}{}", "#3");

  // Box macros — Perl L460-520
  DefMacro!("\\psmatrix", "");
  DefMacro!("\\endpsmatrix", "");
  DefMacro!("\\psrowalign{}", "");
  DefMacro!("\\pscolalign{}", "");
  DefMacro!("\\ncdot", "");
});
