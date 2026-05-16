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
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");

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
});
