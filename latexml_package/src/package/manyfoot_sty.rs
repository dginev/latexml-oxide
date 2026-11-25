//! manyfoot.sty (ncctools), which Perl LaTeXML does not bind: the raw package runs (its options, `\newfootnote`,
//! `\DeclareNewFootnote` with its counter and `\thefootnote<X>` style, `\SelectFootnoteRule`, `\SetFootnoteHook`), and
//! a declared series' notes are the kernel's footnotes marked as the series prints them, not insertions (the given
//! marks nccfoots', `nccfoots_sty.rs`). Its begin-document code keeps what it defines, not its patches placing the
//! series' insertions (the output routine, and LaTeX's minipage through `\g@addto@macro\endminipage`,
//! manyfoot.sty:388-390, which warned on every load). Witnesses: 2511.01971 (chet.sty's `\footnotetextEEE` in
//! `\author`, its text lost before), 2609.01974 (a copyright `\footnotetext<X>`), and loads that warned: 2205.09841
//! (sn-jnl), 2401.05860 (aamas), 2003.11645.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("manyfoot", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // The hook (with the `para` option's, :152-162, under the same label) is put back without its patches of the output
  // routine and minipage (:377-385, :387-401): what it defines, which other packages read (`\MFL@fudgefactor`,
  // dgruyter.sty:2379), is `\MFL@paraskip`, the series' skips (:371), `\defaultfootnoterule` (:372-376) and :386.
  // (`para` ran if `\MFL@xparaskip` is defined: bigfoot defines `\MFL@processpara` without it, 2506.12575)
  RawTeX!(r"\RemoveFromHook{begindocument}[manyfoot]
\AddToHook{begindocument}[manyfoot]{%
  \ifdefined\MFL@xparaskip\begingroup\footnotesize\@tempdima\footnotesep
    \advance\@tempdima-\ht\strutbox\ifdim\@tempdima<\z@\@tempdima\z@\fi
    \advance\@tempdima.5\normalbaselineskip\MFL@xparaskip\xdef\MFL@paraskip{\the\@tempdima\relax}\endgroup\fi
  {\let\@elt\MFL@start\MFL@list}%
  \@ifundefined{defaultfootnoterule}{\@ifundefined{pagefootnoterule}%
    {\let\defaultfootnoterule\footnoterule}{\let\defaultfootnoterule\pagefootnoterule}}{}%
  \MFL@floathook}");

  // manyfoot.sty:45, :68: `\Footnotetext<X>{<mark>}{<text>}`, which `\footnotetext<X>` calls, is nccfoots'
  // `\Footnotetext` set in the series' insertion #1 (`\lx@note@series`, so it is no main `\footnotemark`'s text);
  // `para` sets no mark unless one is given.
  RawTeX!(r"\long\def\lx@mf@fnoteplain#1{\edef\lx@note@series{\expandafter\@gobble\string#1}\Footnotetext}
\long\def\lx@mf@fnotepara#1{\let\@thefnmark\@empty\lx@mf@fnoteplain#1}
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
