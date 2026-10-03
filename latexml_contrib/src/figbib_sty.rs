//! figbib.sty — figure-source lists (the bibliography end).
//!
//! `figbib.sty:330` defines `\fbList{bibs}` to write its private
//! `\figbib@aux` stream and `\InputIfFileExists` the `\jobname.figbib.bbl`
//! (`:122`, `:335`) that only a bibtex run produces; the per-figure `\fbEpsfig` citations (`:271`)
//! go to that private stream, never the kernel's citation set, so from
//! LaTeXML's view nothing is cited and the `.bib` list lives only in this
//! argument — the list vanished at 0 errors (Perl raw-loads the same package,
//! 11 errors, no list). A figure-source list is by construction every source,
//! so the command is the kernel's `\lx@bibliography` over all entries
//! (`latex_constructs.pool.ltxml:3942`, `:4214` for `\nocite`), the bibunits
//! interception (OXIDIZED_DESIGN_DIVERGENCES #87), under the list's own header
//! (`\figbibListHeader`, `:65`, the `\section*` of `thefigbiblist`, `:348`).
//!
//! An `@fig` entry's fields are what figbib.bst writes into `\figbibitem`
//! (figbib.bst:19-35; figbib.sty:365-381 prints "`main`, `add`. \figbibFrom
//! `source`"), which no BibTeX handler knew, so each entry printed as its number
//! alone (`[1] [2] [3]`; figbib_sample recall 54 %; Perl has no list at all): the
//! entry is a `misc`, `main` its title, `add` and `source` its notes, the source
//! only `\if@figbibsource` (`nosource`, `:104-105`), as Perl's handler dispatch
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
  DefMacro!("\\fbList{}",
    "\\nocite{*}\\begingroup\\let\\refname\\figbibListHeader\\lx@bibliography{#1}\\endgroup");
  RawTeX!(r"\def\bib@entry@fig@alias{misc}");
  RawTeX!(r"\def\bib@field@fig@main{\bib@@field{ltx:bib-title}}");
  RawTeX!(r"\def\bib@field@fig@add{\bib@field@default@note}");
  RawTeX!(r"\def\bib@field@fig@source#1{\if@figbibsource\bib@field@default@note{\figbibFrom\ #1}\fi}");
});
