//! nlctuserguide.sty — Nicola Talbot's user-guide package (the glossaries,
//! glossaries-extra, datatool, mfirstuc, bib2gls … manuals).
//!
//! A manual documents every command, option and term in one
//! `\nlctuserguidegls{…}` block (nlctuserguide.sty:2928-3016): its writer
//! macros (`\gcmd`, `\gopt`, `\gidx`, `\gterm`, `\gabbr`, …) serialize each item
//! as a bib2gls record into `\jobname-gls.bib` through `\glsbibwriteentry` /
//! `\glsbibwritefield` (:2883-2892), and `\nlctuserguideloadgls` (:3062-3160)
//! asks the external bib2gls for the `.glstex` that defines the entries. No
//! `.glstex` ships in TeX Live, and neither engine runs bib2gls, so every entry
//! stayed undefined: the command summaries, term lists and `\gls` references of
//! the Talbot manuals were lost (glossaries-extra-manual recall 50.9 %,
//! datatool-user 67.4 %, glossaries-user 68.0 %, sweep #121; Perl has no binding).
//!
//! Here the two writers define the entries in the run instead, as bib2gls's
//! output would (its `\bibglsnew…` commands are `\newglossaryentry` /
//! `\newabbreviation` calls): the record type becomes the entry's category and
//! its glossary follows the resource's `entry-type-aliases` (`type=index`,
//! terms `dual-type=main`, icons `type=symbols`). Parents are defined before
//! their children, as bib2gls orders them. Their glossary definitions are
//! emitted at `\begin{document}`, once every entry exists (glossaries_sty.rs
//! `\iflx@glossaries@defer`), as the `.glstex` defines all entries before any
//! is typeset: a description may reference an entry defined after it. The icon
//! entries (`\symboldefinitions`, :792-817, written as raw `@icon` records) are
//! defined from the same macros. A real bib2gls run with the `.glstex` took
//! mfirstuc-manual from 76.6 % to 92.8 % recall
//! (OXIDIZED_DESIGN_DIVERGENCES #292). Guard: `class_census::nlctuserguide_entries_defined_in_run`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("nlctuserguide", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(r"\def\lx@nlct@push{\lx@queue@gpush{nlct}}%
\renewcommand{\glsbibwritefield}[2]{%
  \def\lx@nlct@key{#1}%
  \ifx\lx@nlct@key\lx@nlct@k@parent \g@addto@macro\lx@nlct@parent{#2}%
  \else\ifx\lx@nlct@key\lx@nlct@k@short \g@addto@macro\lx@nlct@short{#2}%
  \else\ifx\lx@nlct@key\lx@nlct@k@long \g@addto@macro\lx@nlct@long{#2}%
  \else \g@addto@macro\lx@nlct@fields{,#1={#2}}\fi\fi\fi}%
\def\lx@nlct@k@parent{parent}\def\lx@nlct@k@short{short}\def\lx@nlct@k@long{long}%
\renewcommand{\glsbibwriteentry}[3]{%
 {\edef\dhyphen{\string-}\let\-\empty\let\dsb\empty
  \protected@edef\entrylabel{#2}%
  \let\dhyphen\empty\let\dsb\dunderscore
  \gdef\lx@nlct@fields{}\gdef\lx@nlct@parent{}\gdef\lx@nlct@short{}\gdef\lx@nlct@long{}%
  #3%
  \edef\lx@nlct@record{\noexpand\lx@nlct@entry{\entrylabel}{#1}%
    {\unexpanded\expandafter{\lx@nlct@parent}}%
    {\unexpanded\expandafter{\lx@nlct@short}}{\unexpanded\expandafter{\lx@nlct@long}}%
    {\unexpanded\expandafter{\lx@nlct@fields}}}%
  \expandafter\lx@nlct@push\expandafter{\lx@nlct@record}}}%
\renewcommand{\nlctuserguideloadgls}[1]{%
  {\def\symbolentry##1{\lx@nlct@icon{##1}}\symboldefinitions}%
  \def\lx@nlct@pass{1}\lx@queue@use{nlct}%
  \def\lx@nlct@pass{2}\lx@queue@use{nlct}%
  \def\lx@nlct@pass{3}\lx@queue@use{nlct}%
  \lx@queue@clear{nlct}}%
\def\lx@nlct@entry#1#2#3#4#5#6{%
  \ifglsentryexists{#1}{}{%
    \if\relax\detokenize{#3}\relax
      \lx@nlct@define{#1}{#2}{}{#4}{#5}{#6}%
    \else
      \ifglsentryexists{#3}%
        {\lx@nlct@define{#1}{#2}{,parent={#3}}{#4}{#5}{#6}}%
        {\ifnum\lx@nlct@pass=3 \lx@nlct@define{#1}{#2}{}{#4}{#5}{#6}\fi}%
    \fi}}%
\def\lx@nlct@define#1#2#3#4#5#6{%
  \lx@nlct@typeof{#2}%
  \if\relax\detokenize{#5}\relax
    \newglossaryentry{#1}{name={#1},description={},category={#2},type={\lx@nlct@type}#6#3}%
  \else
    \newabbreviation[category={#2},type={\lx@nlct@type}#6#3]{#1}{#4}{#5}%
  \fi}%
\def\lx@nlct@icon#1{%
  \ifglsentryexists{sym.#1}{}{%
    \lx@nlct@typeof{icon}%
    \edef\lx@nlct@tmp{\noexpand\newglossaryentry{sym.#1}{%
      name={\expandafter\expandonce\csname #1text\endcsname},%
      symbol={\expandafter\noexpand\csname #1sym\endcsname},%
      description={\expandafter\expandonce\csname #1desc\endcsname},%
      category={icon},type={\lx@nlct@type}}}%
    \lx@nlct@tmp}}%
\def\lx@nlct@typeof#1{%
  \def\lx@nlct@type{index}%
  \ifcsname lx@nlct@main@#1\endcsname \def\lx@nlct@type{main}\fi
  \ifcsname lx@nlct@sym@#1\endcsname
    \ifglossaryexists*{symbols}{\def\lx@nlct@type{symbols}}{}\fi}%
\expandafter\let\csname lx@nlct@main@term\endcsname\relax
\expandafter\let\csname lx@nlct@main@termabbreviation\endcsname\relax
\expandafter\let\csname lx@nlct@main@termacronym\endcsname\relax
\expandafter\let\csname lx@nlct@main@dualindexabbreviation\endcsname\relax
\expandafter\let\csname lx@nlct@main@acronym\endcsname\relax
\expandafter\let\csname lx@nlct@sym@icon\endcsname\relax");
});
