use latexml_package::prelude::*;

LoadDefinitions!({
  // xltabular.sty — tabularx's `X` columns inside a page-breaking longtable.
  //
  // The real environment (xltabular.sty L118-135 `\newenvironment{xltabular}[1][x]`)
  // collects the body through tabularx's trial loop and then runs
  // `\expandafter\longtable\the\toks@\endlongtable` (L94-96), restoring
  // `\caption`/`\endhead`/`\endfirsthead`/`\endfoot`/`\endlastfoot` to
  // longtable's versions first (L86-91). So semantically it IS a longtable
  // whose column spec may contain `X`; `X` is a global column type once
  // tabularx is loaded, so longtable's alignment machinery accepts it as is.
  //
  // The former alias `\xltabular → \tabularx` left `\caption`/`\endhead`
  // bound to the class versions: under a binding class that gave `Use of
  // \caption outside any known float`, and under raw KOMA (tocbasic's expl3
  // `\caption` reads `\@captype`, xltabular-doc, hvfloat: 36→101 errors) a
  // fatal cascade. Guard: `perfect_kernel_batch53::xltabular_is_a_longtable`.
  //
  // Signature: `\begin{xltabular}[l|r|c|x]{width}[vpos]{cols}` — the leading
  // optional is longtable's horizontal alignment (L120-128 sets
  // `\LTleft`/`\LTright`), the width is presentational (the X columns fill
  // it), the second optional is tabularx's vertical position.
  RequirePackage!("tabularx");
  RequirePackage!("longtable");
  DefMacro!(
    "\\xltabular[]{}[]{}",
    r"\lx@longtable@bindings{#4}\@@longtable[#1]{#4}\lx@begin@alignment"
  );
  DefMacro!("\\endxltabular", r"\lx@end@alignment\@end@tabular");
  // xltabular.sty:19-21 — `\newif\ifXLT@normalPB` plus the two user toggles
  // that flip it; the boolean only steers longtable's page-break patches
  // (L103-104), which have no XML counterpart. The binding replaces the raw
  // .sty, so it must carry the user surface (witness: xltabular-doc, 2
  // `undefined` errors). Guard: `perfect_kernel_batch54::xltabular_pagebreak_toggles`.
  RawTeX!(r"\newif\ifXLT@normalPB \XLT@normalPBtrue");
  DefMacro!("\\normalLTpagebreak", r"\global\XLT@normalPBtrue");
  DefMacro!("\\specialLTpagebreak", r"\global\XLT@normalPBfalse");
});
