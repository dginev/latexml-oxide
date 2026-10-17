//! Binding for tkz-euclide.sty: the real package, read raw, serving its old API too.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("tkz-euclide", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\usetkzobj[…]{all|<objects>}` and `\usetkztool[…]{…}`, the module loaders of tkz-euclide before v3
  // (tkz-tools-eu-modules.tex:18-58, still in TeX Live, no longer read): each `\input`s the `tkz-obj-<object>.tex` files
  // not yet loaded. Since v3 the package loads every object itself (tkz-euclide.sty:173-199), and those files are gone,
  // so with it there is nothing left to load: the loaders take their arguments and load nothing. A paper's own older
  // copy of the package defines its own. Witnesses 1506.03598, 1612.05374, 2005.14485, 1906.04134, 1608.04592 (in
  // the paper's own sung-tikz.sty); TL 2025 pdflatex errs ("Undefined control sequence"), the papers were written for
  // the old API (user ruling 2026-10-17; OXIDIZED_DESIGN_DIVERGENCES #720). Guard
  // `perfect_kernel_batch71a::tkz_euclide_old_usetkzobj`.
  RawTeX!(
    r"\@ifundefined{usetkzobj}{\def\usetkzobj{\pgfutil@ifnextchar[{\use@usetkzobj}{\use@@usetkzobj}}\def\use@usetkzobj[#1]{\use@@usetkzobj{#1}}\def\use@@usetkzobj#1{}}{}
\@ifundefined{usetkztool}{\def\usetkztool{\pgfutil@ifnextchar[{\use@usetkztool}{\use@@usetkztool}}\def\use@usetkztool[#1]{\use@@usetkztool{#1}}\def\use@@usetkztool#1{}}{}"
  );
});
