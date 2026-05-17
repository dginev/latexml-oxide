//! Stub for Elsevier cas-dc.cls / cas-sc.cls (CAS journals double-column).
//!
//! The cas-* classes load cas-common.sty which uses xparse/expl3
//! NewDocumentCommand to define many frontmatter helpers. Our raw load
//! may not invoke them; provide gobble stubs for the most common
//! frontmatter macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  // cas-dc.cls L63: \RequirePackage{booktabs,makecell,multirow,array,colortbl,dcolumn,stfloats}.
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("array");
  RequirePackage!("colortbl");
  RequirePackage!("makecell");

  // cas-common frontmatter — gobble cleanly.
  DefMacro!("\\tnotetext[]{}", "");
  DefMacro!("\\tnotemark[]", "");
  DefMacro!("\\tnoteref[]{}", "");
  DefMacro!("\\fnmark[]", "");
  DefMacro!("\\fnref[]{}", "");
  DefMacro!("\\fntext[]{}", "");
  DefMacro!("\\nonumnote{}", "");
  DefMacro!("\\nonumtnotetext{}", "");
  DefMacro!("\\cortext[]{}", "");
  DefMacro!("\\cormark[]", "");
  DefMacro!("\\corref[]", "");
  DefMacro!("\\affiliation[]{}", "");
  DefMacro!("\\ead[]{}", "");

  // \sep — author/affil separator that cas-common defines.
  DefMacro!("\\sep", ",");

  // cas-common credit-tagging macros (CRediT taxonomy). \credit{role}
  // attaches an author contribution; \printcredits emits the credit list.
  // Both are pure metadata; gobble cleanly. Witness 2405.20972.
  DefMacro!("\\credit{}", "");
  DefMacro!("\\printcredits", "");

  // Elsevier highlights / biography environments (cas-common.sty).
  // `{highlights}` collects bullet points for the dedicated highlights
  // section; `{bio}` / `\bio` defines an author biography. Render as
  // ltx:note (frontmatter) so author content is preserved.
  // Witness 2503.16816, 2502.18516 (cas-dc).
  DefEnvironment!("{highlights}",
    "<ltx:note role='highlights'>#body</ltx:note>");
  DefEnvironment!("{bio}{}",
    "<ltx:note role='biography' name='#1'>#body</ltx:note>");
  // \newproof{env}{display-name} — cas-common's environment factory
  // for proof-like environments. Define a basic environment that
  // wraps content in ltx:proof.
  DefMacro!("\\newproof{}{}",
    "\\newenvironment{#1}{\\par\\noindent\\textbf{#2.}\\hspace{0.5em}}{\\par}");
});
