//! Stub for IEEEtaes.cls (IEEE Trans on Aerospace and Electronic Systems).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");

  // IEEEtaes-specific frontmatter.
  DefMacro!("\\doiinfo{}", "");
  DefMacro!("\\receiveddate{}", "");
  DefMacro!("\\authoraddress{}", "");
  DefMacro!("\\jvol{}", "");
  DefMacro!("\\jnum{}", "");
  DefMacro!("\\jmonth{}", "");
  DefMacro!("\\jyear{}", "");
  DefMacro!("\\pubyear{}", "");
  DefMacro!("\\paper{}", "");
  DefMacro!("\\corresp{}", "");

  // IEEE-style keywords env.
  DefEnvironment!(
    "{IEEEkeywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );
  // IEEE biography envs — wrap in an appendix-style section so they
  // pass schema validation.
  DefMacro!(T_CS!("\\begin{IEEEbiography}"), None,
    "\\section*{Biography}\\@ifnextchar[{\\@IEEEbio@opt}{\\@IEEEbio@noopt}");
  DefMacro!("\\@IEEEbio@opt[]{}", "\\textbf{#2}\\par");
  DefMacro!("\\@IEEEbio@noopt{}", "\\textbf{#1}\\par");
  DefMacro!(T_CS!("\\end{IEEEbiography}"), None, "");
  DefMacro!(T_CS!("\\begin{IEEEbiographynophoto}"), None,
    "\\section*{Biography}\\textbf");
  DefMacro!(T_CS!("\\end{IEEEbiographynophoto}"), None, "");
});
