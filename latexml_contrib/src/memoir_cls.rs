use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "memoir.cls",
    "memoir.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  RequirePackage!("iftex");
  RequirePackage!("array");
  RequirePackage!("dcolumn");
  RequirePackage!("tabularx");
  RequirePackage!("textcase");
  // These are originally \EmulatedPackage directives
  RequirePackage!("appendix");
  RequirePackage!("booktabs");
  RequirePackage!("changepage");
  RequirePackage!("chngcntr");
  RequirePackage!("chngpage");
  RequirePackage!("crop");
  RequirePackage!("enumerate");
  RequirePackage!("epigraph");
  RequirePackage!("makeidx");
  RequirePackage!("needspace");
  RequirePackage!("parskip");
  RequirePackage!("setspace");
  RequirePackage!("titling");
  RequirePackage!("tocbibind");
  RequirePackage!("verbatim");

  // memoir page-geometry preamble idiom (memman.pdf §2: every memoir doc
  // sets its type block with these before \\checkandfixthelayout). STUB
  // JUSTIFICATION (policy 2026-08-31 — stubs only for clearly out-of-scope
  // features): these compute the printed page's margins/type block, a
  // paper-geometry concern with no analogue in reflowable XML/HTML output;
  // they carry no document content whatsoever. Undefined they errored across
  // the biblatex-oxref doc family (perfect-kernel sweep 2026-08-31).
  def_macro_noop("\\setlrmarginsandblock{}{}{}")?;
  def_macro_noop("\\setulmarginsandblock{}{}{}")?;
  def_macro_noop("\\checkandfixthelayout []")?;
  def_macro_noop("\\setheadfoot{}{}")?;
  def_macro_noop("\\setheaderspaces{}{}{}")?;
});
