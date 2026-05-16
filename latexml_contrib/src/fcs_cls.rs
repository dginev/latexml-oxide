//! Stub for fcs.cls (Frontiers of Computer Science).
//!
//! fcs.cls is an expl3/xparse-heavy Springer-style class. The raw load
//! trips on the `\NewDocumentCommand \fcssetup { m }` definition body.
//! Provide a minimal stub: route most user-facing macros through
//! \\@add@frontmatter so author content (title, authors, abstract,
//! keywords, etc.) reaches the XML output.
//!
//! Witness: 2503.12978 (\\fcssetup, {acknowledgement} env).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");
  RequirePackage!("booktabs");
  RequirePackage!("array");
  RequirePackage!("multirow");
  RequirePackage!("caption");

  // \fcssetup{key=value, ...} — main metadata block. The keys
  // (title, author, address, abstract, keywords) are user-content;
  // routing the whole arg as a ltx:note is the simplest preservation.
  DefMacro!("\\fcssetup{}",
    "\\@add@frontmatter{ltx:note}[role=fcssetup]{#1}");

  // {acknowledgement} env — render as acknowledgements.
  DefEnvironment!("{acknowledgement}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>");

  // {compactenum} env — like enumerate but compact.
  DefEnvironment!("{compactenum}", "<ltx:enumerate>#body</ltx:enumerate>");
  DefEnvironment!("{compactitem}", "<ltx:itemize>#body</ltx:itemize>");

  // Chinese-typesetting helpers — gobble (visual only).
  DefMacro!("\\zihang[]{}", "");
});
