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
  // Faithful to overpic.sty's `\OVP@picture` + `\OVP@calc@rel` (the default
  // `percent` mode set by `\ExecuteOptions{percent}` → `rel=100`): the LARGER of
  // width/total-height gets 100 coordinate units and `\unitlength = max(w,h)/100`
  // (so `\put(50,50)` is the centre of a square image; on a landscape image x
  // runs 0..100). Rust measures a boxed `\includegraphics` (`\wd`/`\ht`/`\dp`)
  // exactly as pdfTeX does, which is what makes this port possible.
  //
  // Witnesses (arXiv/html_feedback): 2412.15262 (baseline, no `\put`),
  // 2510.17772 (one label + trim/clip), 2401.13599, 2409.12952, 2405.00666.
  RawTeX!(concat!(
    r"\newsavebox\OVP@box ",
    r"\newenvironment{overpic}[2][]{%", "\n",
    r"  \sbox\OVP@box{\includegraphics[#1]{#2}}%", "\n",
    r"  \@tempcnta=\wd\OVP@box ", "\n",
    r"  \@tempcntb=\ht\OVP@box \advance\@tempcntb\dp\OVP@box ", "\n",
    r"  \ifnum\@tempcnta>\@tempcntb ", "\n",
    r"    \divide\@tempcnta by 100 \ifnum\@tempcnta<\@ne \@tempcnta=\@ne \fi ", "\n",
    r"    \unitlength=\@tempcnta sp\relax ", "\n",
    r"    \@tempcnta=100 \divide\@tempcntb by \unitlength ", "\n",
    r"  \else ", "\n",
    r"    \divide\@tempcntb by 100 \ifnum\@tempcntb<\@ne \@tempcntb=\@ne \fi ", "\n",
    r"    \unitlength=\@tempcntb sp\relax ", "\n",
    r"    \@tempcntb=100 \divide\@tempcnta by \unitlength ", "\n",
    r"  \fi ", "\n",
    // Guard: a missing/unmeasurable image with no size option leaves both
    // dimensions 0, so `max/100` truncates to 0 and `\divide by \unitlength`
    // would raise `Illegal \divide by 0`; the `<\@ne` clamps keep unitlength at
    // >=1sp (a degenerate tiny picture, but no error). Common on arXiv, where
    // submissions routinely omit referenced images.
    r"  \begin{picture}(\@tempcnta,\@tempcntb)%", "\n",
    r"    \put(0,0){\makebox(0,0)[bl]{\usebox\OVP@box}}%", "\n",
    r"}{%", "\n",
    r"  \end{picture}%", "\n",
    r"}",
  ));

  // Perl: {Overpic} (capital O) takes arbitrary TeX instead of an image — used in
  // ~3 arXiv papers, not ported (none of the 44 overpic reports use it).
});
