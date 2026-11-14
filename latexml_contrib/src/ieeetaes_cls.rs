//! Stub for IEEEtaes.cls (IEEE Trans on Aerospace and Electronic Systems).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  RequirePackage!("hyperref");
  RequirePackage!("authblk");
  // IEEEtaes.cls:5158 `\usepackage[capitalize]{cleveref}`; the IEEEtran stub replaces
  // the real `.cls`, so \cref/\Cref come out Error:undefined — Perl raw-loads the `.cls`
  // and gets cleveref. Must load AFTER hyperref. Witness 2606.01169.
  RequirePackage!("cleveref", options => vec!["capitalize".to_string()]);

  // IEEEtaes-specific frontmatter — preserve author content.
  DefMacro!("\\doiinfo{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!(
    "\\receiveddate{}",
    "\\@add@frontmatter{ltx:note}[role=received]{#1}"
  );
  DefMacro!(
    "\\authoraddress{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}"
  );
  DefMacro!("\\jvol{}", "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\jnum{}", "\\@add@frontmatter{ltx:note}[role=issue]{#1}");
  DefMacro!("\\jmonth{}", "\\@add@frontmatter{ltx:note}[role=month]{#1}");
  DefMacro!("\\jyear{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\pubyear{}", "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!(
    "\\paper{}",
    "\\@add@frontmatter{ltx:note}[role=paper-type]{#1}"
  );
  DefMacro!(
    "\\corresp{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  // IEEEtaes.cls:4882 `\member{grade}` after an `\author{…}`: that author's membership, printed with the name; an
  // empty one is nothing (2403.15966). :3455-3456 `\editor`, `\supplementary`: printed in the title footnote
  // (:3427-3428).
  DefMacro!(
    "\\member{}",
    "\\if!#1!\\else\\lx@add@contact[role=membership]{#1}\\fi"
  );
  DefMacro!(
    "\\editor{}",
    "\\if!#1!\\else\\@add@frontmatter{ltx:note}[role=editor]{#1}\\fi"
  );
  DefMacro!(
    "\\supplementary{}",
    "\\if!#1!\\else\\@add@frontmatter{ltx:note}[role=supplementary]{#1}\\fi"
  );

  // IEEE-style keywords env.
  DefEnvironment!(
    "{IEEEkeywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );
  // IEEE biography envs — wrap in an appendix-style section so they
  // pass schema validation.
  DefMacro!(
    T_CS!("\\begin{IEEEbiography}"),
    None,
    "\\section*{Biography}\\@ifnextchar[{\\@IEEEbio@opt}{\\@IEEEbio@noopt}"
  );
  DefMacro!("\\@IEEEbio@opt[]{}", "\\textbf{#2}\\par");
  DefMacro!("\\@IEEEbio@noopt{}", "\\textbf{#1}\\par");
  DefMacro!(T_CS!("\\end{IEEEbiography}"), None, "");
  DefMacro!(
    T_CS!("\\begin{IEEEbiographynophoto}"),
    None,
    "\\section*{Biography}\\textbf"
  );
  DefMacro!(T_CS!("\\end{IEEEbiographynophoto}"), None, "");
});
