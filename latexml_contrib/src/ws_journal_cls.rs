//! Stub for World Scientific journal classes (ws-ijgmmp, ws-ijmpd, etc.).
//!
//! These classes share frontmatter macros like `\catchline{vol}{no}{year}{pgs}{pgsE}`,
//! `\title{...}`, `\Author{name}`. Raw load of the cls may fail before
//! defining these (cls uses complex font/page-layout machinery that
//! doesn't transfer to HTML conversion). Stubs preserve substantive
//! content as frontmatter notes.
//!
//! Witness: 2306.12455 (ws-ijgmmp.cls), 2306.15982 (ws-ijmpd.cls).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("graphicx");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");

  // \catchline{vol}{issue}{year}{pageStart}{pageEnd} — bibliographic
  // running-header metadata. Preserve as frontmatter note.
  DefMacro!("\\catchline{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=catchline]{Vol. #1, No. #2 (#3) #4--#5}");

  // \Journal{...} {...}{...}{...} — citation format helper (4-arg per WS conv).
  // Render the args inline as a generic citation string.
  DefMacro!("\\Journal{}{}{}{}", "{#1} {\\bf #2}, #3 (#4)");

  // No-op layout helpers.
  def_macro_noop("\\copyrightheading{}")?;
  def_macro_noop("\\paperBack")?;
  DefMacro!("\\catchlinefont", "\\footnotesize");

  // \ccode{...} — WS classification-codes block (PACS-style codes in
  // ws-ijmpe and friends). Real def is styled paragraph; preserve as
  // a classification note. Witness 2307.12748, 2307.16467.
  DefMacro!("\\ccode{}",
    "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");
  // \category{code}{name} — ws classification 2-arg variant.
  DefMacro!("\\category{}{}",
    "\\@add@frontmatter{ltx:classification}[scheme=#1]{#2}");

  // {history} env — publication-history wrapper (received/revised/
  // accepted dates). Preserve as a frontmatter note. Witness
  // 2307.12748, 2307.16467 + 2 stage-3/4 ws papers.
  DefEnvironment!("{history}",
    "<ltx:note role='history'>#body</ltx:note>",
    mode => "internal_vertical");
  // \received{date}, \revised{date}, \accepted{date}, \comby{name}
  // — used inside {history}. Preserve as inline notes.
  DefMacro!("\\received{}", "Received #1\\par");
  DefMacro!("\\revised{}", "Revised #1\\par");
  DefMacro!("\\accepted{}", "Accepted #1\\par");
  DefMacro!("\\comby{}", "Communicated by #1\\par");
  // \email / \http / \uurl — render contact info inline.
  DefMacro!("\\email{}", "\\textit{#1}\\par");
  DefMacro!("\\http{}", "\\textit{http://#1}\\par");
  DefMacro!("\\uurl{}", "\\textit{#1}\\par");

  // ws-mpla.cls and other ws classes: \tbl{caption}{body} — table
  // caption + tabular body wrapper. Real def is essentially
  // \caption{#1}<body #2 in tabular>. Bridge to \caption{caption}
  // followed by the body content. Witness 2404.18954 (ws-mpla).
  DefMacro!("\\tbl{}{}", "\\caption{#1}#2");
  // \ttbl{label}{caption}{body} — 3-arg variant with explicit label.
  DefMacro!("\\ttbl{}{}{}", "\\caption{#2}\\label{#1}#3");
  // \tcaption{caption} — single-arg variant in some ws families.
  DefMacro!("\\tcaption{}", "\\caption{#1}");
});
