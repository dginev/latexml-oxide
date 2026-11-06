//! multibib: the raw package, with each list's bibliography style read at that list.
//!
//! multibib.sty:234 defines `\bibliographystyle<suffix>` to write `\bibstyle` to the side aux file bibtex reads
//! (`<suffix>.aux`); it never calls `\bibliographystyle`, so `BIBSTYLE`, which `\bibliographystyle` records for the
//! `.bib` route (`\lx@record@bibstyle`, sect11.rs), stayed empty. A document with only such lists then read no `.bst`
//! (`bst_declared_fields`, OXIDIZED_DESIGN_DIVERGENCES #823): the fields its style declares stayed raw data
//! (`urldate`, quantum.bst:44, printed "(accessed:~12.07.2019)" at :719; the quantum-bibliographystyle-demo manual).
//!
//! The record is keyed by the suffix `\newcites` declares, and `\bibliography<suffix>` reads its own in a group around
//! the list: `\bibliographystyle{alpha}\bibliographystyleweb{plain}` gives `alpha` on the main list and `plain` on
//! `web`, as pdflatex's `[Zet01]` and `[1]`, in either order of the lists; opticajnl.cls:288's and osajnl's
//! `\bibliographystylefullrefs{opticajnl}` beside the document's `opticajnlnt` (1908.11477, 2006.07118, 2008.09542,
//! 2205.11481, 2308.15957, 2607.03975). Where `\bibstyle` is still the
//! kernel's the list takes the style's citation style too; a package that took `\bibstyle` owns that (natbib's
//! `\@gobble`, natbib_sty.rs). `BIBSTYLE` after the lists stays the main list's, or with none the first declared.
//! Limit: a style declared after its own list reaches `BIBSTYLE` only, not that list's attribute.
use latexml_engine::latex_constructs::set_bibstyle;
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("multibib", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\bibliographystyle<suffix>`: the suffix's record; `BIBSTYLE` too while nothing holds it (a `\bibliographystyle` always may).
  DefPrimitive!("\\lx@mb@declare{}{}", sub[(suffix, style)] {
    let suffix = Expand!(suffix).to_string();
    let style = pin(Expand!(style).to_string());
    assign_value(&s!("lx@mb@style@{suffix}"), style, Some(Scope::Global));
    if lookup_string("BIBSTYLE").is_empty() {
      assign_value("BIBSTYLE", style, Some(Scope::Global));
    }
  });
  // `\bibliography<suffix>`, in the list's group: its style while the list opens.
  DefPrimitive!("\\lx@mb@open{}", sub[(suffix)] {
    let style = lookup_string(&s!("lx@mb@style@{}", Expand!(suffix).to_string()));
    if style.is_empty() {
      // (none declared: the standing one, as before)
    } else if matches!(lookup_meaning(&T_CS!("\\bibstyle")), Some(Stored::Constructor(_))) {
      set_bibstyle(&style);
    } else {
      assign_value("BIBSTYLE", pin(style), None);
    }
  });
  RawTeX!(
    r"\def\lx@mb@record#1{\@ifundefined{bibliographystyle#1}{}{%
    \expandafter\let\csname lx@mb@style@#1\expandafter\endcsname\csname bibliographystyle#1\endcsname
    \expandafter\let\csname lx@mb@list@#1\expandafter\endcsname\csname bibliography#1\endcsname
    \@namedef{bibliographystyle#1}##1{\lx@mb@declare{#1}{##1}\@nameuse{lx@mb@style@#1}{##1}}%
    \@namedef{bibliography#1}##1{{\lx@mb@open{#1}\@nameuse{lx@mb@list@#1}{##1}}}}}
\let\lx@mb@newcites\newcites
\def\newcites#1#2{\lx@mb@newcites{#1}{#2}%
  \@for\@tempa:=#1\do{\expandafter\lx@mb@record\expandafter{\@tempa}}}"
  );
});
