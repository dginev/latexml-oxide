//! Stub for IEEEojcsys.cls (IEEE Open Journal of Control Systems).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");

  // Frontmatter — preserve author content.
  DefMacro!("\\paper{}",
    "\\@add@frontmatter{ltx:note}[role=paper-type]{#1}");
  DefMacro!("\\jvol{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\jnum{}",
    "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\jmonth{}",
    "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!("\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}");
  DefMacro!("\\authornote{}",
    "\\@add@frontmatter{ltx:note}[role=authornote]{#1}");
  DefMacro!("\\receiveddate{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}");
  DefMacro!("\\affilmark[]{}", "\\textsuperscript{#2}");

  // \appendices is an IEEE-style appendix env.
  DefMacro!("\\appendices", "\\appendix");

  // {IEEEkeywords} env.
  DefEnvironment!(
    "{IEEEkeywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );
});
