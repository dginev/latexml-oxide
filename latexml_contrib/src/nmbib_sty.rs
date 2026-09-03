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
LoadDefinitions!({
  InputDefinitions!("nmbib", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\citeall OptionalMatch:* [][]{}", "\\citet*[#2][#3]{#4}");
});
