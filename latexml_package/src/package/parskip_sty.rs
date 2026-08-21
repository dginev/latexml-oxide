use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl `parskip.sty.ltxml` is an EMPTY stub ("Nothing to do here, really") —
  // a simplification that drops the real package's entire effect, so both
  // engines leave the first-line indent that `\usepackage{parskip}` is meant to
  // remove (issue #558, reporter nasser1; same-host Perl 0.8.8 identical =>
  // SHARED-FAILURE). Ground truth is the real package: parskip.sty v2.0h sets
  // `\setlength\parindent{0pt}` (L58, default `indent=0pt`) and
  // `\parskip=.5\baselineskip plus 2pt` (L51-54). Port that here — surpass-Perl,
  // OXIDIZED_DESIGN #106.
  //
  // `\parindent=0` is the whole fix for the reported symptom: the paragraph
  // machinery flips every paragraph to the `ltx_noindent` class when the
  // `\parindent` register is zero (`tex_paragraph.rs`, the boolean no-indent
  // toggle), exactly as a manual `\setlength{\parindent}{0pt}` already does.
  // The FIRST paragraph joined this only with issue #719 (same reporter): the
  // deferred `\par` mechanism records the class for the NEXT paragraph, so
  // before #719 the first paragraph kept the stylesheet's default indent.
  // Guarded by `parskip_test` (all three paragraphs now `ltx_noindent`).
  // `\parskip` is set for faithfulness to the real package (its glue is not
  // typeset into HTML by LaTeXML — neither here nor for a manual `\setlength`).
  //
  // Package options (`skip`/`indent`/`tocskip`, kvoptions) are not yet handled;
  // the no-option `\usepackage{parskip}` default is the common case (and the one
  // reported).
  RawTeX!(r"\setlength{\parindent}{0pt}\setlength{\parskip}{.5\baselineskip plus 2pt}");
});
