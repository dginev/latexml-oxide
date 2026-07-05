//! scrreprt.cls — the KOMA-Script report class (chapter-based, like `report` /
//! `scrbook`).
//!
//! Raw-loading a real `.cls` is not yet reliable in latexml-oxide (the class
//! bootstrap infrastructure is a long-term goal), so — exactly like
//! `scrbook_cls` / `scrartcl_cls` — this binding maps scrreprt onto the
//! `OmniBus` fallback class and stubs the KOMA-specific commands a typical
//! document touches. Perl LaTeXML has no `scrreprt.cls.ltxml` and (lacking the
//! same raw-`.cls` support) behaves equivalently.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "scrreprt.cls",
    "scrreprt.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  // Mirror scrbook_cls.rs: the real KOMA chain transitively loads iftex, which
  // OmniBus does not, so pull it in for `\ifpdf` / engine-detection authors.
  RequirePackage!("iftex");

  // KOMA section-font hooks — see scrbook_cls.rs for the full rationale (tocloft
  // expands `\sectfont` / `\size@chapter` when a KOMA class is detected; as a
  // chapter class scrreprt uses the `\size@chapter` form).
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
  def_macro_noop("\\addtokomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
  // KOMA page-style marks — see scrartcl_cls.rs.
  def_macro_noop("\\headmark")?;
  def_macro_noop("\\pagemark")?;

  // KOMA `\minisec{title}`: an unnumbered, un-TOC'd run-in-ish heading below
  // `\subparagraph`. Map to the closest standard structural heading so it lands
  // as `ltx:paragraph` rather than erroring.
  DefMacro!("\\minisec{}", "\\paragraph*{#1}");

  // KOMA `addmargin` environment: `\begin{addmargin}[innermargin]{outermargin}`
  // (or `{both}`) indents a block. The margin is a visual-layout concern with no
  // structural meaning in the XML tree, so render the body transparently.
  RawTeX!(r"\newenvironment{addmargin}[2][]{}{}");
  RawTeX!(r"\newenvironment{addmargin*}[2][]{}{}");
});
