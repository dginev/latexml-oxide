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
  // Real KOMA classes always load typearea (scrartcl.cls L2593
  // \RequirePackage{typearea}[\KOMAScriptVersion]) — its binding carries
  // \typearea/\recalctypearea/\areaset (sweep-11 `\recalctypearea`
  // cluster, 26 docs, witness bohr/bohr_en via cnltx-doc.cls L190).
  RequirePackage!("typearea");
  RequirePackage!("scrlfile");
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

  // KOMA user-level structure commands (TL doc corpus: \minisec 17 bundles,
  // {labeling} 11). These carry CONTENT, so they get semantic mappings, not
  // stubs (policy 2026-08-31: stubs only for clearly out-of-scope features):
  // \minisec{title} — an unnumbered freestanding mini-heading → the starred
  // paragraph heading, preserving the title as <ltx:paragraph><ltx:title>.
  // {labeling}[delim]{widest} — a description list with fixed label width →
  // {description}; only the label-width/delimiter PRESENTATION args are
  // dropped (print-layout, out of scope). The \begin-in-body alias idiom
  // keeps the environment stack balanced.
  DefMacro!("\\minisec{}", "\\paragraph*{#1}");
  // KOMA \ifpdfoutput{then}{else} (scrkbase; deprecated KOMA compat) tests
  // \pdfoutput>0 — always TRUE in our pdftex model. Witness l2tabu/l2tabuen
  // L43 (perfect-kernel).
  DefMacro!("\\ifpdfoutput{}{}", "#1");
  // KOMA logo macros (scrkbase).
  DefMacro!("\\KOMAScript", "KOMA-Script");
  DefMacro!("\\KOMA", "KOMA");
  def_macro_noop("\\newkomafont{}{}")?;
  def_macro_noop("\\addtokomafont{}{}")?;
  DefMacro!("\\labeling[]{}", "\\begin{description}");
  DefMacro!("\\endlabeling", "\\end{description}");
});
