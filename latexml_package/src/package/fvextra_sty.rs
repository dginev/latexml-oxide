use crate::prelude::*;

// fvextra.sty — extends fancyvrb (breaklines, breakanywhere, improved line
// numbering, math-mode verbatim, …). Perl LaTeXML ships no fvextra.sty.ltxml
// and raw-loads the real file under `rawstyles`; we do the same so every
// environment/key a document declares (`\DefineVerbatimEnvironment{Prompt}
// {Verbatim}{breaklines,breakanywhere,…}`) is defined.
//
// AFTER loading, we neutralise fvextra's automatic line-BREAKING by routing
// its breaking line-processor back to the non-breaking one. Rationale:
//   * `breakanywhere=true` installs `\FancyVerbBreakStart=\FV@Break`, a
//     recursive char-by-char scanner that measures every character by boxing
//     line-prefixes (`\sbox{\FV@LineBox}{\FV@BProcessLine{#1}}`). In our
//     engine that recurses through `predigest_box_contents_in_mode` and grows
//     the gullet pushback unboundedly until the 650000 `PushbackLimit` Fatal
//     fires — where Perl converts the document cleanly.
//   * Line-breaking is a PDF-visual concern with no HTML semantics: Perl's
//     HTML for a `breakanywhere` verbatim line is byte-identical to a plain
//     verbatim line (the browser wraps `<pre>`), so disabling the scan is
//     output-faithful while turning a Fatal into a clean conversion.
// Forcing `\FV@ListProcessLine@Break` to the `@NoBreak` processor (the same
// path `breaklines=false` uses) keeps the `font="typewriter"` verbatim
// styling and all non-breaking fvextra features intact; only the soft
// break-points are dropped.
//
// Drove 121/185 fatal `Timeout/PushbackLimit` papers in the sandbox-arxiv-2605
// corpus (witness: 2605.01024 EmoMM — `\DefineVerbatimEnvironment{Prompt}
// {Verbatim}{breaklines=true,breakanywhere=true}` over multi-line prompts).
#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("fvextra", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Route the breaking line-processor to the non-breaking one. The `breaklines`
  // key (`\let\FV@ListProcessLine\FV@ListProcessLine@Break`) then resolves to
  // `@NoBreak` for every later `\fvset`/`\DefineVerbatimEnvironment`.
  RawTeX!(r"\let\FV@ListProcessLine@Break\FV@ListProcessLine@NoBreak");
});
