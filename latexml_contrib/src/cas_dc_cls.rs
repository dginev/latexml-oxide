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
  // Title-note text — author prose. Preserve as ltx:note frontmatter.
  DefMacro!("\\tnotetext[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#2}");
  DefMacro!("\\tnotemark[]", "");  // mark only, no body
  DefMacro!("\\tnoteref[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote-ref]{#2}");
  DefMacro!("\\fnmark[]", "");
  DefMacro!("\\fnref[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote-ref]{#2}");
  // Footnote / author-note text — preserve as ltx:note rather than
  // gobble (content-preserving). `\fntext` / `\nonumnote` carry the
  // actual note prose, `\cortext` is the corresponding-author byline.
  DefMacro!("\\fntext[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote]{#2}");
  DefMacro!("\\nonumnote{}",
    "\\@add@frontmatter{ltx:note}[role=note]{#1}");
  DefMacro!("\\nonumtnotetext{}",
    "\\@add@frontmatter{ltx:note}[role=note]{#1}");
  DefMacro!("\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}");
  DefMacro!("\\cormark[]", "");  // mark only, no body
  DefMacro!("\\corref[]", "");
  // \affiliation[id]{text} — affiliation string author typed.
  DefMacro!("\\affiliation[]{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#2}}");
  // \ead[type]{address} — author email/url, preserve as contact.
  DefMacro!("\\ead[]{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{#2}}");
  // ltx:contact stubs (mirror elsart_support_core@@@affiliation form)
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#1'>#2</ltx:contact>");

  // \sep — author/affil separator that cas-common defines.
  DefMacro!("\\sep", ",");

  // cas-common credit-tagging macros (CRediT taxonomy). \credit{role}
  // attaches an author contribution (Conceptualization, Investigation,
  // etc.); \printcredits emits the credit list. Round-34 surpass-Perl:
  // preserve the role text as a frontmatter note so the CRediT
  // taxonomy is retained for downstream JATS conversion. Witness
  // 2405.20972.
  DefMacro!("\\credit{}",
    "\\@add@frontmatter{ltx:note}[role=credit]{#1}");
  DefMacro!("\\printcredits", "");

  // Elsevier highlights / biography environments (cas-common.sty).
  // `{highlights}` collects bullet points for the dedicated highlights
  // section; `{bio}` / `\bio` defines an author biography. Render as
  // ltx:note (frontmatter) so author content is preserved.
  // Witness 2503.16816, 2502.18516 (cas-dc).
  // Use internal_vertical mode so the body can contain paragraphs,
  // lists, and other block-level content. Without the explicit mode,
  // the env infers restricted_horizontal and trips
  // `Error:unexpected:\endbio Attempt to end mode restricted_horizontal
  // in internal_vertical` when the bio contains \par-separated prose.
  // Witness 2503.16816.
  DefEnvironment!("{highlights}",
    "<ltx:note role='highlights'>#body</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{bio}{}",
    "<ltx:note role='biography' name='#1'>#body</ltx:note>",
    mode => "internal_vertical");
  // \newproof{env}{display-name} — cas-common's environment factory
  // for proof-like environments. Define a basic environment that
  // wraps content in ltx:proof.
  DefMacro!("\\newproof{}{}",
    "\\newenvironment{#1}{\\par\\noindent\\textbf{#2.}\\hspace{0.5em}}{\\par}");
});
