use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tikz-cd.sty.ltxml
  InputDefinitions!("tikz-cd", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // tikz-cd's `rightsquigarrow` / `leadsto` arrow styles look for the
  // flag `\tikz@library@decorations.pathmorphing@loaded` (set by
  // `\usetikzlibrary{decorations.pathmorphing}`) and emit a
  // `\PackageError` when it's missing. arXiv's compile cluster
  // appears to load the library by default; papers that rely on
  // squiggly arrows in tikzcd diagrams thus compile cleanly there
  // but fire 1 error per squiggly use on our pipeline. Pull in the
  // library after raw-load so the flag is set before any \begin{tikzcd}
  // expansion. Witness: arXiv:2508.13059 — 4 errors -> 0.
  TeX!(r"\usetikzlibrary{decorations.pathmorphing}");
  // Defensive: also set the @loaded flag directly in case the
  // \usetikzlibrary call above gets short-circuited by an earlier
  // partial load (Rust's raw-load timing differs from pdflatex).
  TeX!(r"\expandafter\let\csname tikz@library@decorations.pathmorphing@loaded\endcsname\@empty");
});
