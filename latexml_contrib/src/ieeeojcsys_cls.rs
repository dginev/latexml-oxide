//! Stub for IEEEojcsys.cls (IEEE Open Journal of Control Systems).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");

  // Frontmatter — gobble.
  DefMacro!("\\paper{}", "");
  DefMacro!("\\jvol{}", "");
  DefMacro!("\\jnum{}", "");
  DefMacro!("\\jmonth{}", "");
  DefMacro!("\\corresp{}", "");
  DefMacro!("\\authornote{}", "");
  DefMacro!("\\receiveddate{}", "");
  DefMacro!("\\affilmark[]{}", "");

  // \appendices is an IEEE-style appendix env.
  DefMacro!("\\appendices", "\\appendix");

  // {IEEEkeywords} env.
  DefEnvironment!(
    "{IEEEkeywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );
});
