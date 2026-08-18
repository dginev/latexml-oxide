use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: overpic.sty.ltxml requires graphicx + epic. epic (only needed for the
  // `grid` option's `\grid`) has no Rust binding — a benign `missing_file`
  // warning; the grid option is off by default, so overlay rendering is
  // unaffected.
  RequirePackage!("graphicx");
  RequirePackage!("epic");

  // DIVERGENCE from Perl (OXIDIZED_DESIGN #135): Perl's overpic.sty.ltxml emits an
  // EMPTY `<ltx:picture tex='...'/>` and relies on the `PictureImages` post-
  // processor to render the whole thing (graphic + overlays) as one LaTeX image.
  // Two Rust realities make that produce nothing: `tex=` on `<ltx:picture>` is
  // suppressed unconditionally (intentional divergence), and the LaTeXImages
  // renderer is unwired — Rust renders `<ltx:picture>` as inline SVG from its
  // CHILD elements. So instead we reproduce overpic.sty's OWN construction: box
  // the graphic to size a `{picture}` to it, place the graphic at the origin, and
  // let the body's `\put` overlays draw on top. This routes through Rust's
  // working picture + `<ltx:graphics>` + `\put` machinery (the graphic resolves
  // via the picture-nested-graphics SVG path, PR #675), so all of the ~37 arXiv
  // papers that report a missing overpic figure now render.
  //
  // Faithful to overpic.sty's `\OVP@picture` + coordinate-mode keys. The default
  // is `percent` (`\ExecuteOptions{percent}` → `rel=100`): the LARGER of
  // width/total-height gets 100 coordinate units and `\unitlength = max(w,h)/100`
  // (so `\put(50,50)` is the centre of a square image). The OVP keys are ported
  // so a paper that overrides the mode positions its overlays correctly:
  //   percent / rel=N / permil : `\unitlength = max(w,h)/N`, larger dim spans N.
  //   abs                      : coordinates are `\unitlength`s (default 1pt or `unit=`).
  //   unit=<dim>               : set `\unitlength` (used by `abs`).
  //   grid / tics              : accepted; `grid` needs epic's `\grid` (unbound), so no-op.
  // Rust measures a boxed `\includegraphics` (`\wd`/`\ht`/`\dp`) as pdfTeX does,
  // which is what makes this port possible.
  //
  // Witnesses (arXiv/html_feedback): 2412.15262 (baseline, no `\put`),
  // 2510.17772 (percent labels + trim/clip), 2401.13599 (abs/unit), 2409.06474
  // (nested `\includegraphics` in `\put` + rotate/scale), 2405.00666 (128 envs).
  RawTeX!(concat!(
    r"\newsavebox\OVP@box", "\n",
    // percent/rel mode: unitlength = max(w,h)/scale; the larger dim spans `scale`
    // units. The `<\@ne` clamp keeps unitlength >= 1sp so a degenerate (missing /
    // zero-size) image never triggers `Illegal \divide by 0` — common on arXiv.
    r"\newcommand\OVP@calc@rel{%", "\n",
    r"  \ifnum\@tempcnta>\@tempcntb ", "\n",
    r"    \divide\@tempcnta by \OVP@scale \ifnum\@tempcnta<\@ne \@tempcnta=\@ne \fi ", "\n",
    r"    \unitlength=\@tempcnta sp\relax \@tempcnta=\OVP@scale \divide\@tempcntb by \unitlength ", "\n",
    r"  \else ", "\n",
    r"    \divide\@tempcntb by \OVP@scale \ifnum\@tempcntb<\@ne \@tempcntb=\@ne \fi ", "\n",
    r"    \unitlength=\@tempcntb sp\relax \@tempcntb=\OVP@scale \divide\@tempcnta by \unitlength ", "\n",
    r"  \fi}", "\n",
    // abs mode: coordinates are `\unitlength`s (set by `unit=`, else the 1pt
    // register default). Clamp unitlength >= 1sp for the same divide-by-0 guard.
    r"\newcommand\OVP@calc@abs{%", "\n",
    r"  \ifdim\unitlength<\@ne sp \unitlength=1pt\relax \fi ", "\n",
    r"  \divide\@tempcnta by \unitlength \divide\@tempcntb by \unitlength}", "\n",
    // OVP key family (overpic.sty L25-48). `\@m` = 1000 (permil).
    r"\define@key{OVP}{rel}{\def\OVP@scale{#1}\let\OVP@calc\OVP@calc@rel}", "\n",
    r"\define@key{OVP}{percent}[]{\def\OVP@scale{100}\let\OVP@calc\OVP@calc@rel}", "\n",
    r"\define@key{OVP}{permil}[]{\def\OVP@scale{\@m}\let\OVP@calc\OVP@calc@rel}", "\n",
    r"\define@key{OVP}{abs}[]{\let\OVP@calc\OVP@calc@abs}", "\n",
    r"\define@key{OVP}{unit}{\unitlength=\dimexpr#1\relax}", "\n",
    r"\define@key{OVP}{grid}[true]{}", "\n",
    r"\define@key{OVP}{tics}{}", "\n",
    r"\newenvironment{overpic}[2][]{%", "\n",
    r"  \sbox\OVP@box{\includegraphics[#1]{#2}}%", "\n",
    // per-env default is percent (overpic.sty `\ExecuteOptions{percent}`); then
    // read any OVP keys from the optarg. `\setkeys*` is relaxed — it ignores the
    // graphicx keys (width/trim/clip/...) that also live in `#1`.
    r"  \def\OVP@scale{100}\let\OVP@calc\OVP@calc@rel ", "\n",
    r"  \setkeys*{OVP}{#1}%", "\n",
    r"  \@tempcnta=\wd\OVP@box ", "\n",
    r"  \@tempcntb=\ht\OVP@box \advance\@tempcntb\dp\OVP@box ", "\n",
    r"  \OVP@calc ", "\n",
    r"  \ifnum\@tempcnta<\@ne \@tempcnta=\@ne \fi \ifnum\@tempcntb<\@ne \@tempcntb=\@ne \fi ", "\n",
    r"  \begin{picture}(\@tempcnta,\@tempcntb)%", "\n",
    r"    \put(0,0){\makebox(0,0)[bl]{\usebox\OVP@box}}%", "\n",
    r"}{%", "\n",
    r"  \end{picture}%", "\n",
    r"}",
  ));

  // Perl: {Overpic} (capital O) takes arbitrary TeX instead of an image — used in
  // ~3 arXiv papers, not ported (none of the 44 overpic reports use it).
});
