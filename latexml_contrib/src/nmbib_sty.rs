use latexml_package::prelude::*;

// nmbib.sty raw-loads over the natbib binding, then reimplements natbib's
// LOW-LEVEL citation engine (`\NAT@citexnum`, nmbib.sty:141) and adds
// `\citeall` (:343 → `\@citeall` → `\@@@citeall`, :347), which opens with
// natbib.sty:780's `\NAT@reset@parser` and the full parser/sort/state surface
// that the natbib binding (a high-level `<ltx:cite>` emulation, ~20 `\NAT@*`)
// does not carry — 57 undefined internals, malformed text-mode citations
// (nmbib-sample 22 errors; Perl's natbib.sty.ltxml is the same emulation and
// has no nmbib binding). `\citealn` is already `[\citenum{#1}]` (:338).
// `\citeall` cites the entry with all authors and the year — `\citet*`'s
// content — so it is emulated as that constructor, exactly how natbib's
// public commands are. Guard: `perfect_kernel_batch54::nmbib_citeall_is_a_cite`.
//
// The bibliography end: `\multibibliography{bibs}` (nmbib.sty:72) only
// writes `\bibdata` to one `.aux` per view type, and each
// `\printbibliography{type}` (:383) inputs `\jobname-<type>.bbl` — bibtex
// products LaTeXML never has, so the three views (timeline, sequence,
// authors) vanished at 0 bibitems (Perl raw-loads the same package, 9
// errors, no list). The views are one cited-entry list sorted three ways,
// so the saved `.bib` list runs the kernel's `\lx@bibliography`
// (`latex_constructs.pool.ltxml:3942`) once, at the first
// `\printbibliography`; natbib's `\citep`/`\citet` select the entries.
// The list is captured in the preamble, so the override must be in place
// at package load, not `\AtBeginDocument`. nmbib.sty:92 also
// `\let\@biblabel\NAT@biblabel`, a natbib internal the natbib emulation
// does not define; the kernel default (latex.ltx `\@biblabel`, sect11.rs)
// is restored. Guard: `06_cluster_bibliography::nmbib_prints_the_saved_bib_list_once`.
#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("nmbib", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\citeall OptionalMatch:* [][]{}", "\\citet*[#2][#3]{#4}");
  DefMacro!("\\@biblabel{}", "[#1]");
  RawTeX!(r"\newif\ifNMBIB@rendered");
  DefMacro!("\\multibibliography{}", "\\gdef\\NMBIB@saved@bib@list{#1}");
  DefMacro!("\\printbibliography{}",
    "\\ifNMBIB@rendered\\else\\expandafter\\lx@bibliography\\expandafter{\\NMBIB@saved@bib@list}\\global\\NMBIB@renderedtrue\\fi");
});
