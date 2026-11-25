//! manyfoot.sty (ncctools), which Perl LaTeXML does not bind: the raw package runs (its options, `\newfootnote`,
//! `\DeclareNewFootnote` with its counter and `\thefootnote<X>` style, `\SelectFootnoteRule`, `\SetFootnoteHook`), and
//! a declared series' notes are the kernel's footnotes marked as the series prints them, not insertions (the given
//! marks nccfoots', `nccfoots_sty.rs`). Its begin-document code only places the series' insertions (the output routine,
//! and LaTeX's minipage through `\g@addto@macro\endminipage`, manyfoot.sty:370-402, which warned on every load), so it
//! is removed. Witnesses: 2511.01971 (chet.sty's `\footnotetextEEE` in `\author`, its text lost before), 2609.01974 (a
//! copyright `\footnotetext<X>`), and loads that warned: 2205.09841 (sn-jnl), 2401.05860 (aamas), 2003.11645.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("manyfoot", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // (the `para` option's begin-document code is under the same label)
  RawTeX!(r"\RemoveFromHook{begindocument}[manyfoot]");

  // manyfoot.sty:45, :68: `\Footnotetext<X>{<mark>}{<text>}`, which `\footnotetext<X>` calls, is nccfoots'
  // `\Footnotetext` set as an insertion (`para` sets no mark unless one is given).
  RawTeX!(r"\long\def\lx@mf@fnoteplain#1{\Footnotetext}
\long\def\lx@mf@fnotepara#1{\let\@thefnmark\@empty\Footnotetext}
\let\MFL@fnoteplain\lx@mf@fnoteplain
\let\MFL@fnotepara\lx@mf@fnotepara");

  // manyfoot.sty:242-257: `\footnote<X>` is LaTeX's `\footnote` (latex.ltx:17649) stepping the series' counter, and
  // `\footnote<X>[<n>]` sets it, so `\@mpfn` and `\thempfn` name them for that note (`\lx@mf@series`, read as a
  // minipage's are); the text's own notes keep the outer ones (`\lx@mf@outer`); each sets `\@thefnmark`.
  RawTeX!(r"\def\lx@mf@footnote#1{\@ifnextchar[{\lx@mf@footnote@opt{#1}}{\lx@mf@footnote@noopt{#1}}}
\long\def\lx@mf@footnote@opt#1[#2]#3{\begingroup\lx@mf@series{#1}\footnote[#2]{{\lx@mf@outer#3}}%
  \csname c@#1\endcsname#2\relax\protected@xdef\@thefnmark{\thempfn}\endgroup}
\long\def\lx@mf@footnote@noopt#1#2{\begingroup\lx@mf@series{#1}\footnote{{\lx@mf@outer#2}}%
  \protected@xdef\@thefnmark{\thempfn}\endgroup}
\def\lx@mf@series#1{\let\lx@mf@mpfn\@mpfn\let\lx@mf@thempfn\thempfn\def\@mpfn{#1}\def\thempfn{\csname the#1\endcsname}}
\def\lx@mf@outer{\let\@mpfn\lx@mf@mpfn\let\thempfn\lx@mf@thempfn}
\let\lx@mf@declare\MFL@declare
\def\MFL@declare#1#2[#3]{\lx@mf@declare{#1}{#2}[#3]%
  \expandafter\def\csname footnote#2\endcsname{\lx@mf@footnote{footnote#2}}}");
});
