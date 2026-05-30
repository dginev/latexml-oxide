use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "scrbook.cls",
    "scrbook.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  // Real scrbook.cls loads the KOMA dependency chain (scrkbase, tocbasic, …),
  // which transitively loads `iftex`, defining `\ifpdf`/`\ifpdftex`/… for
  // author engine/driver detection. Perl raw-loads the .cls and gets iftex
  // that way; our OmniBus stub intercepts it. Mirror the dependency (same
  // rationale as scrartcl_cls.rs, witness 1802.07175).
  RequirePackage!("iftex");
  // KOMA section-font hooks (`\sectfont` + empty `\size@<unit>` family) — see
  // scrartcl_cls.rs for the full rationale. tocloft expands these in
  // `\cfttoctitlefont` when a KOMA class is detected; as a chapter class
  // scrbook genuinely uses the `\size@chapter` form (tocloft.sty L169). Without
  // them, scrbook + tocloft + `\tableofcontents` hits undefined `\sectfont` /
  // `\size@chapter` where Perl (raw scrbook) is clean.
  def_macro_noop("\\maybesffamily")?;
  DefMacro!("\\sectfont", "\\normalcolor\\maybesffamily\\bfseries");
  def_macro_noop("\\size@part")?;
  def_macro_noop("\\size@partnumber")?;
  def_macro_noop("\\size@chapter")?;
  def_macro_noop("\\size@section")?;
  def_macro_noop("\\size@subsection")?;
  def_macro_noop("\\size@subsubsection")?;
  def_macro_noop("\\size@paragraph")?;
  def_macro_noop("\\size@subparagraph")?;
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
  // KOMA page-style marks — see scrartcl_cls.rs for rationale.
  def_macro_noop("\\headmark")?;
  def_macro_noop("\\pagemark")?;
});
