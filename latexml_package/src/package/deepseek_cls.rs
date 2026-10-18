use crate::prelude::*;

// deepseek.cls is not in TeX Live: each paper ships its own copy (2401.14196), which loads raw as written.
// Its title page's footer (`\fancypagestyle{firststyle}`, deepseek.cls:166-171, which `\maketitle` selects with
// `\thispagestyle{firststyle}`, :260/:279) prints `*\the\correspondingauthor` (a `\newtoks`, :112): the note naming
// the authors the `*` marks. The footer is page furniture, which we do not set (`\fancypagestyle` is a no-op), but
// that note is frontmatter (ruling 2026-10-01: semantic notes a class prints in a footer are kept), and it was lost
// with 0 errors. The binding adds it where the class's `\maketitle` selects the footer, under the footer's own test
// (:168-170). Repro sectioning-frontmatter/class_notes/deepseek_corresponding_author_footer_note; guard
// `perfect_kernel_batch74b::deepseek_corresponding_author_note_is_kept`.
#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("deepseek", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RawTeX!(r"\let\lx@deepseek@thispagestyle\thispagestyle
\def\thispagestyle#1{\lx@deepseek@thispagestyle{#1}\def\reserved@a{#1}\def\reserved@b{firststyle}%
  \ifx\reserved@a\reserved@b\lx@deepseek@firststyle@notes\fi}
\def\lx@deepseek@firststyle@notes{\ifdefined\correspondingauthor\if\relax\the\correspondingauthor\relax\else
  \@add@frontmatter{ltx:note}[role=corresponding]{*\the\correspondingauthor}\fi\fi}");
});
