//! pdfcomment.sty — PDF annotations (sticky notes, tooltips, markup
//! comments) as `ltx:note`s.
//!
//! The real package (pdfcomment.sty:1353-1370) picks a hyperref driver by
//! `\ifpdf`; with `\pdfoutput=0` it takes the dvips branch and emits raw
//! `\pdfmark` dictionaries, so raw-loading it left every comment's text
//! buried in `pdfmark=/ANN,Subtype=/Text,…/Contents (…)` body text plus
//! `undefined:\pdfmark`, `\pc@hyenc@color`, `\textCR`, `\pc@pdfenc@*` (the
//! pdfcomment manuals example/example_math_markup/example_latex_dvips_ps2pdf,
//! 5 errors each; Perl has no binding and dies earlier in the dependency
//! stack). An annotation is metadata attached to the text: an `ltx:note`
//! with `role` = the command's kind — the shape acmart's `\Description` and
//! endnotes already use. Public API per pdfcomment.sty:1418-2703.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("xkeyval");
  RequirePackage!("hyperref");
  RequirePackage!("marginnote"); // pdfcomment.sty:1347
  // pd1enc.def:82 `\DeclareTextCommand{\textCR}{PD1}{\015}` — a line break
  // inside an annotation's text (example.tex:144).
  DefMacro!("\\textCR", "\\newline");
  // Setup and declarations carry only PDF presentation keys.
  def_macro_noop("\\pdfcommentsetup{}")?;
  def_macro_noop("\\defineavatar{}{}")?;
  def_macro_noop("\\definestyle{}{}")?;
  def_macro_noop("\\defineliststyle{}{}")?;
  def_macro_noop("\\listofpdfcomments[]")?;
  // The note body is text even when the annotation sits in math
  // (`$x\pdftooltip{y}{tip}$`, example_math_markup).
  DefConstructor!("\\lx@pdfcomment@note{}{}", "<ltx:note role='#1'>#2</ltx:note>",
    mode => "text");
  // Single-body annotations: the text IS the note.
  DefMacro!("\\pdfcomment[]{}", "\\lx@pdfcomment@note{pdfcomment}{#2}");
  DefMacro!(
    "\\pdfmargincomment[]{}",
    "\\lx@pdfcomment@note{pdfmargincomment}{#2}"
  );
  DefMacro!(
    "\\pdffreetextcomment[]{}",
    "\\lx@pdfcomment@note{pdffreetextcomment}{#2}"
  );
  DefMacro!(
    "\\pdfsquarecomment[]{}",
    "\\lx@pdfcomment@note{pdfsquarecomment}{#2}"
  );
  DefMacro!(
    "\\pdfcirclecomment[]{}",
    "\\lx@pdfcomment@note{pdfcirclecomment}{#2}"
  );
  DefMacro!(
    "\\pdflinecomment[]{}",
    "\\lx@pdfcomment@note{pdflinecomment}{#2}"
  );
  DefMacro!("\\pdfreply[]{}", "\\lx@pdfcomment@note{pdfreply}{#2}");
  // Two-body forms: the marked/visible text stays in the flow, the comment
  // follows it as the note.
  DefMacro!(
    "\\pdfmarkupcomment[]{}{}",
    "#2\\lx@pdfcomment@note{pdfmarkupcomment}{#3}"
  );
  DefMacro!("\\pdftooltip[]{}{}", "#2\\lx@pdfcomment@note{tooltip}{#3}");
  // pdfcomment.sty:2255 `{pdfsidelinecomment}[opts]{comment}`: the body is
  // typeset with a side line, the comment annotates it.
  DefEnvironment!(
    "{pdfsidelinecomment}[]{}",
    "<ltx:note role='pdfsidelinecomment'>#2</ltx:note>#body"
  );
});
