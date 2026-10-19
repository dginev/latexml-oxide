//! Stub for colm2025_conference.sty (COLM 2025 conference template).
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");
  // Some COLM 2025 templates author-edit the .sty to add `\definecolor`
  // calls before users `\usepackage{color}`. Eager-load color/xcolor so
  // the templates' early color definitions don't trip "\\definecolor
  // undefined". Witness 2503.21480 (definecolor at colm2025 L11).
  RequirePackage!("color");
  // Pre-load xcolor with [dvipsnames, table] so user xcolor calls
  // don't silently option-clash and miss dvipsnam.def/colortbl.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);

  // Author-list separators (colm L107-153): a space each (a `" "` string body would tokenize as an empty line, `\par`;
  // KNOWN_PERL_ERRORS #984).
  DefMacro!(T_CS!("\\And"), None, Tokens!(T_SPACE!()));
  DefMacro!(T_CS!("\\AND"), None, Tokens!(T_SPACE!()));
  DefMacro!(T_CS!("\\Ands"), None, Tokens!(T_SPACE!()));
  // ICLR/NeurIPS-style author email-aside.
  // \affilmark{N,M,...} — affiliation superscript markers on the
  // author line. Author content; emit as superscript inline.
  DefMacro!("\\affilmark{}", "\\textsuperscript{#1}");
  def_macro_noop("\\thanksauthor")?;
  DefConditional!("\\ifcolmsubmission");
  DefConditional!("\\ifcolmfinal");
  // colm2025_conference.sty L16-17 also declares \ifcolmpreprint.
  // Witnesses: 2504.03048, 2504.05625, 2504.09394 (papers passing
  // [preprint] class option which calls \colmpreprinttrue).
  DefConditional!("\\ifcolmpreprint");
});
