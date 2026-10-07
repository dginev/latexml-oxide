use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: etex.sty.ltxml
  LoadPool!("eTeX");
  // etex.sty:172 `\def\eTeX{$\m@th\varepsilon$-\TeX}` — the engine logo the
  // package defines beside its register allocation; absent from Perl's
  // etex.sty.ltxml. (bibleref-parse's manual :138 uses `\eTeX` WITHOUT
  // loading etex — etoolbox.sty:29-34 skips it once latex.ltx has
  // `\extrafloats` — so pdflatex reports the same undefined control
  // sequence there; that one stays.)
  DefMacro!("\\eTeX", "$\\m@th\\varepsilon$-\\TeX");
  // etex.sty register-allocator macros (etex.sty L332-348). Real defs
  // use `\et@xglob`/`\et@xloc` to allocate from extended register
  // pools (Numbers 256+ for count/dimen/etc.). For our purposes the
  // semantic is "allocate a new register"; forward to LaTeX's
  // `\newcount`/`\newdimen`/etc. which already exist.
  //
  // Glob* variants allocate globally; loc* variants locally.
  // In LaTeXML's flat-state model these are effectively equivalent.
  //
  // Witness: arXiv:2506.16610 / .16657 / .20642 (papers via etex.sty
  // raw-load + linegoal.sty / etextools / similar). Rust 2 → 0
  // expected, beating Perl=3.
  DefMacro!("\\globcount", "\\newcount");
  DefMacro!("\\loccount", "\\newcount");
  DefMacro!("\\globdimen", "\\newdimen");
  DefMacro!("\\locdimen", "\\newdimen");
  DefMacro!("\\globskip", "\\newskip");
  DefMacro!("\\locskip", "\\newskip");
  DefMacro!("\\globmuskip", "\\newmuskip");
  DefMacro!("\\locmuskip", "\\newmuskip");
  DefMacro!("\\globbox", "\\newbox");
  DefMacro!("\\locbox", "\\newbox");
  DefMacro!("\\globtoks", "\\newtoks");
  DefMacro!("\\loctoks", "\\newtoks");
  DefMacro!("\\globmarks", "\\newmarks");
  DefMacro!("\\locmarks", "\\newmarks");
  // etex.sty:382-425 block allocators: `\globtoksblk\foo{17}` `\mathchardef`s `\foo` to the first of 17 consecutive
  // registers (`\et@xgblk`). They exist only in etex.sty's pre-2020 branch: on a kernel with extended allocation
  // (`\count10`-`\count15` past 255, LaTeX 2020-02 on) etex.sty:103-139 aliases the single forms, as this binding does,
  // and stops, leaving the block forms undefined (TL2025 pdflatex: "Undefined control sequence"). Papers of that era use
  // them (OXIDIZED_DESIGN_DIVERGENCES #457). A block is taken downward from the top of each register range, as etex's
  // local blocks were (`\count27`), clear of LaTeX's upward allocation and of `\count@`/`\count255`. Undefined, the
  // "proofs at the end" idiom's `\toks\numexpr\prooftoks+\count@` read "Missing number" and left `+\count@\relax`, which
  // reset its loop counter: an endless `\loop` (1610.01929, 1801.07292; Perl loops too).
  RawTeX!(r"\def\lx@etex@allocblk#1#2#3{\global\allocationnumber\numexpr\csname lx@etex@blktop@#1\endcsname-(#3)\relax
\expandafter\xdef\csname lx@etex@blktop@#1\endcsname{\the\allocationnumber}\global\mathchardef#2\allocationnumber}
\def\lx@etex@blktop@count{32768}\def\lx@etex@blktop@dimen{32768}\def\lx@etex@blktop@skip{32768}
\def\lx@etex@blktop@muskip{32768}\def\lx@etex@blktop@box{32768}\def\lx@etex@blktop@toks{32768}
\def\lx@etex@blktop@marks{32768}
\def\globcountblk{\lx@etex@allocblk{count}}\let\loccountblk\globcountblk
\def\globdimenblk{\lx@etex@allocblk{dimen}}\let\locdimenblk\globdimenblk
\def\globskipblk{\lx@etex@allocblk{skip}}\let\locskipblk\globskipblk
\def\globmuskipblk{\lx@etex@allocblk{muskip}}\let\locmuskipblk\globmuskipblk
\def\globboxblk{\lx@etex@allocblk{box}}\let\locboxblk\globboxblk
\def\globtoksblk{\lx@etex@allocblk{toks}}\let\loctoksblk\globtoksblk
\def\globmarksblk{\lx@etex@allocblk{marks}}\let\locmarksblk\globmarksblk");
});
