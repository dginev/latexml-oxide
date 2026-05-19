//! Stub for colm2025_conference.sty (COLM 2025 conference template).
use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

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

  // Author-list separators (colm L107-153).
  DefMacro!("\\And", " ");
  DefMacro!("\\AND", " ");
  DefMacro!("\\Ands", " ");
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
