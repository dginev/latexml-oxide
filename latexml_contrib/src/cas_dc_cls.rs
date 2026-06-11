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
  // amssymb supplies \lesssim, \gtrsim, \nleq, \ngeq, \square, \blacksquare
  // and other binary-relation/blackboard-bold symbols that CAS authors
  // routinely use without explicit \usepackage{amssymb}. Witness
  // 2312.12523 (\lesssim undefined).
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  // cas-dc.cls L63: \RequirePackage{booktabs,makecell,multirow,array,colortbl,dcolumn,stfloats}.
  RequirePackage!("booktabs");
  RequirePackage!("multirow");
  RequirePackage!("array");
  RequirePackage!("colortbl");
  RequirePackage!("makecell");
  // cas-dc.cls L: `\RequirePackage{etoolbox,balance}` — pulls in the
  // balance package which defines `\balance` (column-balancing in
  // 2-col docs). Without it, papers using `\balance` near the end
  // trip Error:undefined. Witness 2303.04712.
  RequirePackage!("balance");
  // cas-dc.cls (Elsevier CAS) aliases \newdefinition and \newproof to \newtheorem
  // (elsarticle convention), used to declare definition/proof-like theorem envs.
  // Were undefined (Perl defines them). Witness 2306.04212.
  Let!("\\newdefinition", "\\newtheorem");
  Let!("\\newproof", "\\newtheorem");

  // cas-common.sty dynamically defines `\tblwidth` via
  // `\csgdef{tblwidth}{\dim_use:N \l_tbl_width_dim}` only inside its
  // own table-float wrapper. Authors use `\tblwidth` directly as the
  // width argument of `\begin{tabular*}{\tblwidth}{...}` even outside
  // that wrapper. Provide a \linewidth fallback so the tabular still
  // renders with reasonable width. Witness 2209.06932 (cas-dc table
  // cascade — 136 errors).
  DefMacro!("\\tblwidth", "\\linewidth");
  // cas-common.sty L2070-2072: \newcolumntype{L|R|C}{...}. Authors use
  // L/R/C column types in `\begin{tabular*}{...}{@{} LLLLLL @{}}`.
  // Without these, the unrecognized column letters trigger one
  // "Extra alignment tab '&'" per & cell. Define matching short-hand
  // raggedright/etc. column types.
  RawTeX!(
    r"\newcolumntype{L}{@{\extracolsep{\fill}}l}\newcolumntype{R}{@{\extracolsep{\fill}}r}\newcolumntype{C}{@{\extracolsep{\fill}}c}"
  );

  // cas-common frontmatter — gobble cleanly.
  // Title-note text — author prose. Preserve as ltx:note frontmatter.
  DefMacro!(
    "\\tnotetext[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#2}"
  );
  def_macro_noop("\\tnotemark[]")?; // mark only, no body
  DefMacro!(
    "\\tnoteref[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote-ref]{#2}"
  );
  def_macro_noop("\\fnmark[]")?;
  DefMacro!(
    "\\fnref[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote-ref]{#2}"
  );
  // Footnote / author-note text — preserve as ltx:note rather than
  // gobble (content-preserving). `\fntext` / `\nonumnote` carry the
  // actual note prose, `\cortext` is the corresponding-author byline.
  DefMacro!(
    "\\fntext[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote]{#2}"
  );
  DefMacro!(
    "\\nonumnote{}",
    "\\@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  DefMacro!(
    "\\nonumtnotetext{}",
    "\\@add@frontmatter{ltx:note}[role=note]{#1}"
  );
  DefMacro!(
    "\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#2}"
  );
  def_macro_noop("\\cormark[]")?; // mark only, no body
  def_macro_noop("\\corref[]")?;
  // \affiliation[id]{text} — affiliation string author typed.
  DefMacro!(
    "\\affiliation[]{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#2}}"
  );
  // \ead[type]{address} — author email/url, preserve as contact.
  DefMacro!(
    "\\ead[]{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}{#2}}"
  );
  // ltx:contact stubs (mirror elsart_support_core@@@affiliation form)
  DefConstructor!(
    "\\@@@affiliation{}",
    "^ <ltx:contact role='affiliation'>#1</ltx:contact>"
  );
  DefConstructor!(
    "\\@@@email{}{}",
    "^ <ltx:contact role='#1'>#2</ltx:contact>"
  );

  // \sep — author/affil separator that cas-common defines.
  DefMacro!("\\sep", ",");

  // cas-common credit-tagging macros (CRediT taxonomy). \credit{role}
  // attaches an author contribution (Conceptualization, Investigation,
  // etc.); \printcredits emits the credit list. Round-34 surpass-Perl:
  // preserve the role text as a frontmatter note so the CRediT
  // taxonomy is retained for downstream JATS conversion. Witness
  // 2405.20972.
  DefMacro!(
    "\\credit{}",
    "\\@add@frontmatter{ltx:note}[role=credit]{#1}"
  );
  def_macro_noop("\\printcredits")?;

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
  DefMacro!(
    "\\newproof{}{}",
    "\\newenvironment{#1}{\\par\\noindent\\textbf{#2.}\\hspace{0.5em}}{\\par}"
  );
});
