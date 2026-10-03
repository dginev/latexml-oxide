//! figbib.sty — figure-source lists (the bibliography end).
//!
//! `figbib.sty:330` defines `\fbList{bibs}` to write its private
//! `\figbib@aux` stream and `\InputIfFileExists` the `\jobname.figbib.bbl`
//! (`:122`, `:335`) that only a bibtex run produces; the per-figure `\fbEpsfig` citations (`:271`)
//! go to that private stream, never the kernel's citation set, so from
//! LaTeXML's view nothing was cited and the list vanished at 0 errors (Perl
//! raw-loads the same package, 11 errors, no list). Each figure command now also
//! cites its key in the kernel's set where it stands (`\lx@mark@nocite`, the mark
//! `\nocite` defers to the document's end — the entry's "Cited by" is the
//! figure, as pdflatex's "Fig. n (p. n)"), and the command is the kernel's
//! `\lx@bibliography` (`latex_constructs.pool.ltxml:3942`), the bibunits
//! interception (OXIDIZED_DESIGN_DIVERGENCES #87): the figures used, in figure
//! order, as bibtex lists them, under the list's own header (`\figbibListHeader`,
//! `:65`, the `\section*` of `thefigbiblist`, `:348`). The figures are cited in a
//! list of their own (`figbib`, the bibunits mechanism, #87), which the list reads,
//! so the document's citations stay out of it. Residuals: every citation also
//! reaches the main list (Scan.pm:379-380), so a regular `\bibliography` beside the
//! list sees the figure keys ("Missing bibkeys", or a figure entry listed when the
//! two share a `.bib`; RED index-bib/figbib_beside_a_bibliography); a later
//! `\bibliographystyle` restyles the first bibliography, the list (sect11.rs
//! `\bibstyle`).
//!
//! An `@fig` entry's fields are what figbib.bst writes into `\figbibitem`
//! (figbib.bst:19-35; figbib.sty:365-381 prints "`main`, `add`. \figbibFrom
//! `source`"), which no BibTeX handler knew, so each entry printed as its number
//! alone (`[1] [2] [3]`; figbib_sample recall 54 %; Perl has no list at all): the
//! entry is a `misc`, `main` its title, `add` and `source` its notes, the source
//! only `\if@figbibsource` (`nosource`, `:104-105`), an empty one not at all, the
//! list in citation order (figbib.bst has no SORT), as Perl's handler dispatch
//! (BibTeX.pool.ltxml:135-157) finds `\bib@field@fig@<field>`. Repro
//! index-bib/figbib_fig_entries_print_their_fields.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  let opts: Vec<String> = lookup_vecdeque("opt@figbib.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("figbib", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
  // The list's own style, local to it (figbib.bst has no SORT: citation order).
  DefPrimitive!("\\lx@figbib@bibstyle", { latexml_engine::latex_constructs::set_bibstyle("figbib"); });
  // The figures' citation list (`inlist`), as a bibunit's.
  DefPrimitive!("\\lx@figbib@unit", { assign_value("CITE_UNIT", "figbib", None); });
  DefMacro!("\\fbList{}", "\\begingroup\\let\\refname\\figbibListHeader\\lx@figbib@bibstyle\
\\lx@bibliography[figbib]{#1}\\endgroup");
  // Each figure command writes `\citation{key}` to the private aux (figbib.sty:271, :284, :299, :310), so bibtex lists
  // exactly the figures used, in figure order: here the same citation is marked at the figure, in the figures' list.
  RawTeX!(r"\def\lx@figbib@cite#1{{\lx@figbib@unit\lx@mark@nocite{#1}}}
\let\lx@figbib@Epsfig\fbEpsfig \def\fbEpsfig#1{\lx@figbib@cite{#1}\lx@figbib@Epsfig{#1}}
\let\lx@figbib@EpsfigM\fbEpsfigM \def\fbEpsfigM#1{\lx@figbib@cite{#1}\lx@figbib@EpsfigM{#1}}
\let\lx@figbib@Eps\fbEps \def\fbEps#1{\lx@figbib@cite{#1}\lx@figbib@Eps{#1}}
\let\lx@figbib@EpsM\fbEpsM \def\fbEpsM#1{\lx@figbib@cite{#1}\lx@figbib@EpsM{#1}}");
  RawTeX!(r"\def\bib@entry@fig@alias{misc}");
  RawTeX!(r"\def\bib@field@fig@main{\bib@@field{ltx:bib-title}}");
  // An empty `add` or `source` prints nothing (figbib.sty:378, :380 `\ifx #5\@empty`).
  RawTeX!(r"\def\bib@field@fig@add#1{\if\relax\detokenize{#1}\relax\else\bib@field@default@note{#1}\fi}");
  RawTeX!(r"\def\bib@field@fig@source#1{\if@figbibsource\if\relax\detokenize{#1}\relax\else
    \bib@field@default@note{\figbibFrom\ #1}\fi\fi}");
});
