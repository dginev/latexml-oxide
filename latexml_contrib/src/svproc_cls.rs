//! Stub for svproc.cls (Springer Proceedings template, sister of svjour).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Pre-load xcolor with [dvipsnames, table] so a paper's later
  // `\usepackage[table]{xcolor}` doesn't silently option-clash and
  // leave colortbl unloaded → `\cellcolor` undefined. svproc.cls
  // itself does NOT load xcolor (only uses the kernel `\normalcolor`),
  // so in Perl the user's `[table]{xcolor}` is the first load and
  // colortbl comes in via the `table` option; matching that outcome
  // here. Same anti-clash pattern as mnras_cls / quantumarticle_cls.
  // Witness: 1706.04315, 1804.09301, 1807.04749 ("undefined:\\cellcolor").
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("hyperref");

  // svproc.cls L864: \newtoks\tocauthor / \toctitle for TOC entries.
  // Preserve author content as ltx:note.
  DefMacro!("\\tocauthor{}",
    "\\@add@frontmatter{ltx:note}[role=tocauthor]{#1}");
  DefMacro!("\\toctitle{}",
    "\\@add@frontmatter{ltx:note}[role=toctitle]{#1}");
  DefMacro!("\\institute{}",
    "\\@add@frontmatter{ltx:note}[role=institute]{#1}");
  // \inst{N} is a superscript marker keyed to numbered affiliations.
  DefMacro!("\\inst{}", "\\textsuperscript{#1}");
  def_macro_noop("\\mainmatter")?;
});
