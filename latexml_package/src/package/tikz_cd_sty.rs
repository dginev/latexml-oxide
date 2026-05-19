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
  // Also pull `decorations.markings` — used by tikz-cd's arrow-marking
  // styles; same vendor pattern as pathmorphing. Witness 2308.06778.
  TeX!(r"\usetikzlibrary{decorations.markings}");
  // Defensive: also set both `@loaded` flags GLOBALLY (pgf uses
  // `\global\let \csname ... \endcsname \pgfutil@empty` — without
  // \global the flag stays local to the binding's load group and
  // disappears after group exit, leaving the user's squiggly arrow
  // hitting the missing-flag PackageError). Witness 2306.03232.
  TeX!(r"\global\expandafter\let\csname tikz@library@decorations.pathmorphing@loaded\endcsname\@empty");
  TeX!(r"\global\expandafter\let\csname tikz@library@decorations.markings@loaded\endcsname\@empty");
});
