//! Stub for Wiley NJD family of classes (WileyNJD-v1, WileyASNA-v1, ...).
//!
//! These Wiley journal classes share a common set of frontmatter macros
//! (\corres, \authormark, \jnlcitation, \cname, \cyear, \vol, \DOI,
//! \papertype, ...). Route to OmniBus and gobble the frontmatter so
//! downstream content renders cleanly.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // amssymb pulls in \gtrsim/\lesssim and other relation symbols Wiley
  // journal papers commonly use without an explicit \usepackage{amssymb}.
  // Witness 2406.06228 (WileyASNA-v1 paper).
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  // Wiley journals frequently load hyperref; mirror so cross-refs work.
  RequirePackage!("hyperref");

  // Wiley frontmatter — gobbled.
  DefMacro!("\\authormark{}", "");
  DefMacro!("\\corres{}", "");
  DefMacro!("\\jnlcitation OptionalMatch:* []{}{}", "");
  DefMacro!("\\presentadd[]{}", "");
  DefMacro!("\\fundingInfo{}", "");
  DefMacro!("\\papertype{}", "");
  DefMacro!("\\paperfield{}", "");
  DefMacro!("\\jname{}", "");
  DefMacro!("\\jvol{}", "");
  DefMacro!("\\jnum{}", "");
  DefMacro!("\\cname{}{}", "");
  DefMacro!("\\cyear{}", "");
  DefMacro!("\\Copyrightline{}", "");
  DefMacro!("\\artmonth{}", "");
  DefMacro!("\\DOI{}", "");
  DefMacro!("\\doiline{}", "");
  DefMacro!("\\runningheads{}{}", "");
  DefMacro!("\\receiveddate{}", "");
  DefMacro!("\\reviseddate{}", "");
  DefMacro!("\\accepteddate{}", "");
  // wileyNJDv5.cls (newer template) adds these. Witness 2407.00139.
  DefMacro!("\\copyyear{}", "");
  DefMacro!("\\titlemark{}", "");
  DefMacro!("\\startpage{}", "");
  DefMacro!("\\bmsection{}", "\\section{#1}");
  DefMacro!("\\bmsubsection{}", "\\subsection{#1}");
  // 'corres' (without trailing 's' from real wileynjd) — alternative
  // \corres macro signature in v5 templates.
});
