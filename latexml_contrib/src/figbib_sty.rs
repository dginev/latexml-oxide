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
//! interception (OXIDIZED_DESIGN_DIVERGENCES #87).
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  let opts: Vec<String> = lookup_vecdeque("opt@figbib.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("figbib", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
  DefMacro!("\\fbList{}", "\\nocite{*}\\lx@bibliography{#1}");
});
