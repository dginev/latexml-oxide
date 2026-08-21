use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // STATUS — the whole scalerel family is NEUTRALIZED, not yet fully supported.
  // This binding is deliberately STEP 1: *preserve the content* (the object survives,
  // sized to text height) and stop the `Error:undefined` + broken layout — at the cost
  // of the ACTUAL scale. The target height (`\scalerel`/`\scaleto`/`\stretchto`) and the
  // numeric factor (`\scaleobj`/`\hstretch`/`\vstretch`) are DROPPED, so every scaled
  // object renders at ~1em (text height) regardless of the requested size. Do NOT mistake
  // this approximation for correct sizing.
  // FUTURE (complete support): honour the real scale — measure the object box and the
  // reference box (or read the factor), compute the ratio, and emit a CSS
  // `transform: scale()` / SVG viewBox — which needs box measurement the engine does not
  // yet expose. Tracked in docs/SYNC_STATUS.md (scalerel full box-measurement scaling).
  //
  // arXiv/html_feedback#6895: the `scalerel` package has no `.ltxml` binding in
  // Perl OR Rust, and the raw `.sty` load leaves `\scalerel` undefined — so an
  // inline icon built with `\scalerel*` (the `\orcidicon` of arXiv:2608.12272)
  // raised `Error:undefined:\scalerel` and rendered its picture unscaled ("too
  // big / multi-line"). Beyond Perl 0.8.8, which errors identically.
  //
  // `\scalerel*[maxwidth]{obj}{ref}` scales `obj` to the height of `ref`, aspect
  // preserved (scalerel.sty L68-84). The dominant use is an inline object scaled
  // to the surrounding *text* height, so — box-measurement scaling being
  // unavailable in this engine — we wrap `obj` in an inline-block that CSS sizes
  // to text height (`.ltx_scalerel`, `LaTeXML.css`). The `[maxwidth]` cap
  // (default `99in`, i.e. unbounded) is accepted and dropped. The starred form
  // yields just the scaled object; the plain `\scalerel{obj}{ref}` appends `ref`
  // afterwards (scalerel.sty L84, `\scalerelplus`).
  DefMacro!("\\scalerel", "\\@ifstar\\lx@scalerel@star\\lx@scalerel@plus");
  DefMacro!("\\lx@scalerel@star []{}{}", "\\lx@scalerel@obj{#2}");
  DefMacro!("\\lx@scalerel@plus []{}{}", "\\lx@scalerel@obj{#2}#3");
  DefConstructor!("\\lx@scalerel@obj{}",
    "<ltx:inline-block class='ltx_scalerel'>#1</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true, bounded => true);
  // `\stretchrel` stretches ignoring the aspect ratio; aspect-preserving is the
  // safe default for an inline icon, so alias it to `\scalerel` (scalerel.sty L86).
  Let!("\\stretchrel", "\\scalerel");
  // Binding scalerel short-circuits the raw `.sty`, so EVERY command it defines is
  // lost unless re-added here. Before this binding the raw load defined the whole
  // family (as `\newcommand`s), so papers using them converted clean; covering only
  // `\scalerel`/`\stretchrel` regressed `\scaleto`/`\scaleobj`/`\stretchto` &c. to
  // `Error:undefined` (sandbox-arxiv-2605: 20+ papers, e.g. 2605.02053, 2605.03024,
  // 2605.03521). Each scales/stretches its OBJECT to a height or by a factor we
  // cannot measure box-wise, so — as with `\scalerel` — we wrap just the object in
  // the text-height inline-block and drop the target (scalerel.sty L104/117/145-159).
  //   \scaleto  [max]{obj}{ht}     obj=#2  (scalerel.sty L104)
  //   \stretchto[min]{obj}{ht}     obj=#2  (scalerel.sty L117)
  //   \scaleobj      {factor}{obj} obj=#2  (scalerel.sty L145)
  //   \hstretch      {factor}{obj} obj=#2  (scalerel.sty L138)
  //   \vstretch      {factor}{obj} obj=#2  (scalerel.sty L141)
  DefMacro!("\\scaleto []{}{}",   "\\lx@scalerel@obj{#2}");
  DefMacro!("\\stretchto []{}{}", "\\lx@scalerel@obj{#2}");
  DefMacro!("\\scaleobj {}{}",    "\\lx@scalerel@obj{#2}");
  DefMacro!("\\hstretch {}{}",    "\\lx@scalerel@obj{#2}");
  DefMacro!("\\vstretch {}{}",    "\\lx@scalerel@obj{#2}");
  // `\scaleleftright`/`\stretchleftright` place a scaled left and right object around
  // a reference; both are built on `\scalerel`/`\stretchrel` (scalerel.sty L129-137),
  // and the `.`-sentinel `\ifx` skips an empty side.
  DefMacro!("\\scaleleftright []{}{}{}",
    "\\ifx.#2#3\\else\\scalerel[#1]{#2}{#3}\\fi\\ifx.#4\\else\\scalerel*[#1]{#4}{#3}\\fi");
  DefMacro!("\\stretchleftright []{}{}{}",
    "\\ifx.#2#3\\else\\stretchrel[#1]{#2}{#3}\\fi\\ifx.#4\\else\\stretchrel*[#1]{#4}{#3}\\fi");
});
