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

  // fvextra L2249 `\def\FancyVerbFormatLine#1{#1}` OVERWRITES the ltx_verbatim
  // css-class hook that fancyvrb_sty.rs installs on `\FancyVerbFormatLine` (via
  // `\lx@add@cssclass{ltx_verbatim}`). fvextra runs its `\def` AFTER
  // `\RequirePackage{fancyvrb}`, so a Verbatim loaded once fvextra is present
  // lost `class="ltx_verbatim"` — and with it `white-space:pre` — collapsing to
  // ordinary typewriter text (issue #502). Re-install the hook over fvextra's
  // redefinition (with the scanner relaxed above, `\FancyVerbFormatLine` is
  // just the per-line formatter, so wrapping it is safe).
  Let!("\\lx@save@FancyVerbFormatLine", "\\FancyVerbFormatLine");
  DefMacro!("\\FancyVerbFormatLine{}",
    "\\lx@add@cssclass{ltx_verbatim}\\lx@save@FancyVerbFormatLine{#1}");

  // Issue #525: define fvextra's `backgroundcolor`/`bgcolor` keys if the host
  // fvextra predates them (the TL fvextra loaded above may be older — it then
  // errors `keyval: backgroundcolor undefined` on the reporter's document).
  // Faithful port of fvextra.sty L2435-2444: store the colour name in
  // \FancyVerbBackgroundColor, which fancyvrb_sty.rs's `frame` box reads as its
  // background. Guarded on the key's absence so a newer host fvextra's own
  // definition (with its per-line \colorbox machinery) is left untouched.
  RawTeX!(concat!(
    r"\@ifundefined{KV@FV@backgroundcolor}{",
    r"\define@key{FV}{backgroundcolor}{\def\FancyVerbBackgroundColor{#1}}",
    r"\fvset{backgroundcolor=none}",
    r"\define@key{FV}{bgcolor}{\fvset{backgroundcolor=#1}}",
    r"}{}",
  ));

  // A newer host fvextra paints `backgroundcolor` per line via `\FV@BGColor@List`
  // (`fvextra.sty` L2547: a `\colorbox` `\rlap`+`\hspace{\linewidth}` strip
  // behind each line, plus over-deep struts to close the vertical seams). LaTeXML
  // captures that as nested `backgroundcolor` `<ltx:text>` boxes with runs of
  // padding spaces — version-dependent noise. `fancyvrb_sty.rs`'s `frame` box
  // already carries the background (reading `\FancyVerbBackgroundColor`), so
  // neutralize the per-line strip to a pass-through: the background then comes
  // from the single wrapper on BOTH fvextra versions, keeping the output clean
  // and stable. Guarded on the macro's presence (the older fvextra has none).
  RawTeX!(r"\@ifundefined{FV@BGColor@List}{}{\long\def\FV@BGColor@List#1{#1}}");

  // Issue #702: surface fvextra's `breaklines` wrapping directive as a stable
  // `ltx_break` css class on the framed verbatim box, so a stylesheet can style a
  // wrapping verbatim apart from a non-wrapping one WITHOUT the fragile
  // `:has(.ltx_parbox)` selector (the only distinguisher otherwise, an accident of
  // fvextra's `\parbox`-based break rendering). `fancyvrb_sty.rs`'s frame box reads
  // `\lx@fv@breakclass` (default EMPTY — plain fancyvrb never wraps); redefine it to
  // expand to `1` off `\ifFV@breaklines`, the boolean fvextra's `breaklines` key
  // sets (fvextra.sty L2757-2762, `\newbool{FV@breaklines}`), which is live by the
  // time the frame box opens (set at option-parse, frame fires later in \FV@List).
  // The constructor maps `1` → `ltx_break`. Beyond-Perl (both engines emit no
  // breaklines hook natively), same spirit as the frame remap (#525 / OXIDIZED_DESIGN
  // #111). Only the framed case is marked — a frameless breaklines verbatim has no
  // single wrapper to hang the class on; that is a follow-up.
  RawTeX!(r"\def\lx@fv@breakclass{\ifFV@breaklines 1\fi}");
});
