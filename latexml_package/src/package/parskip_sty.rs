use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl `parskip.sty.ltxml` is an EMPTY stub ("Nothing to do here, really") —
  // a simplification that drops the real package's entire effect, so both
  // engines leave the first-line indent that `\usepackage{parskip}` is meant to
  // remove (issue #558, reporter nasser1; same-host Perl 0.8.8 identical =>
  // SHARED-FAILURE). Ground truth is the real package, so load it raw —
  // surpass-Perl, OXIDIZED_DESIGN #106. parskip.sty v2.0h (2021-03-14):
  //  * requires kvoptions (L44) and processes its `indent`/`parfill`/`skip`/
  //    `tocskip` options (L45-50), then sets `\parskip` (L51-55, default
  //    `.5\baselineskip plus 2pt`), `\parfillskip` (L56-57) and `\parindent`
  //    (L58, default `indent=0pt`);
  //  * requires etoolbox (L59) for its `\patchcmd`s, which the document may use
  //    too: loading neither left `\AtEndPreamble` undefined wherever etoolbox
  //    did not arrive by another route (under pdfTeX it came only through babel;
  //    witnesses liftarm, polyomino under the `[luatex]` profile).
  //    Guard: `shipout_parskip::parskip_loads_kvoptions_and_etoolbox`.
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
  // Expected on every parskip document: parskip's `\patchcmd\@startsection`
  // fails (LaTeXML's `\@startsection` has no `\addvspace\@tempskipa`), giving
  // one `Info:unexpected:patchcmd` and the console line "Couldn't patch
  // \@startsection"; its `\@starttoc` and `\@xsect` patches apply. The spacing
  // it patches never reaches the XML, so neither message needs a fix.
  InputDefinitions!("parskip", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
