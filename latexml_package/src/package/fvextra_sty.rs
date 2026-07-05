use crate::prelude::*;

// fvextra.sty — extends fancyvrb (breaklines, breakanywhere, improved line
// numbering, math-mode verbatim, …). Perl LaTeXML ships no fvextra.sty.ltxml
// (it raw-loads the real file only under `--includestyles`; by default the
// package is simply missing there). We raw-load unconditionally — house
// idiom — so every environment/key a document declares
// (`\DefineVerbatimEnvironment{Prompt}{Verbatim}{breaklines,…}`) is defined.
//
// AFTER loading, we neutralise ONLY fvextra's char-by-char break scanner:
// `breakanywhere`/`breakbefore`/`breakafter` do `\let\FancyVerbBreakStart
// \FV@Break` at key-SET time — a recursive scanner that measures every
// character by boxing line-prefixes (`\sbox{\FV@LineBox}{\FV@BProcessLine
// {#1}}`). In our engine that recurses through
// `predigest_box_contents_in_mode` and grows the gullet pushback unboundedly
// until the 650000 `PushbackLimit` Fatal fires (display path; 121/185 fatal
// papers in sandbox-arxiv-2605, witness 2605.01024) or hangs the inline
// `\Verb` path to Fatal:Timeout:TokenLimit — where Perl converts cleanly.
// Aliasing the TARGET `\FV@Break` to `\relax` makes every later key-set
// propagate `\relax`, and BOTH consumers gate on
// `\ifx\FancyVerbBreakStart\relax`, taking their plain paths.
//
// The `breaklines` line-processor `\FV@ListProcessLine@Break` itself is
// left INTACT: with the scanner relaxed it typesets an over-wide line as a
// `\parbox[t]{\FV@LineWidth}` with ragged-right and breakable spaces —
// plain TeX paragraph machinery our engine wraps natively, so the measured
// height budget counts the same wrapped lines pdflatex produces and the
// content stays inside the drawn frame (2605.00468 prompt boxes poked
// 4-60px past the right border when this was over-neutralised to the
// `@NoBreak` processor, which hboxes each SOURCE line unbroken).
#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("fvextra", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  RawTeX!(r"\let\FV@Break\relax");
});
