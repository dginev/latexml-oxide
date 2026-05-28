//! Stub for spie.cls (SPIE conference proceedings).
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Pre-load xcolor with [dvipsnames, table] so a paper's later
  // `\usepackage[table]{xcolor}` doesn't silently option-clash and
  // leave colortbl unloaded → `\cellcolor` undefined. spie.cls itself
  // does NOT load xcolor (only `\LoadClassWithOptions{article}`), so
  // in Perl the user's `[table]{xcolor}` is the first load and colortbl
  // comes in via the `table` option; matching that outcome here. Same
  // anti-clash pattern as svproc_cls / mnras_cls / quantumarticle_cls.
  // Witness: 1807.04749 ("undefined:\\cellcolor").
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("hyperref");

  // spie.cls L107: \authorinfo{...} for author footnote — preserve.
  DefMacro!("\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}");
  def_macro_noop("\\skiplinehalf")?;
  DefMacro!("\\supit{}", "\\textsuperscript{#1}");
});
