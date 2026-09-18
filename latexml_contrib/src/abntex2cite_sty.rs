//! abntex2cite.sty — ABNT citations (Brazilian standard), the bibliography end.
//!
//! The raw package is loaded with the document's options (`num`/`alf`,
//! `foot`, …; abntex2cite.sty:158-174); its `\bibliography` redefinition is
//! refused by the kernel's lock and its `\cite` handed back to the kernel:
//! abntex2cite.sty:375 redefines `\bibliography` to write `\bibdata` to the `.aux` and
//! `\@input@{\jobname.bbl}` — a file only a bibtex run produces, which LaTeXML
//! never has, so the reference list vanished at 0 errors (abntex2cite's manual
//! is half bibliography; abntex2cite-alf and unbtex share it). The native
//! `\bibliography` (latex_constructs.pool.ltxml:3895; sect11.rs) prefers a
//! shipped `.bbl` and otherwise runs the `.bib` session at post
//! (`\lx@ifusebbl`), and the `.bib` list is live only here, in the
//! command's argument — the same interception Perl makes for bibunits'
//! `\bibliography` wrapper (bibunits.sty.ltxml; OXIDIZED_DESIGN_DIVERGENCES
//! #87); since batch 56cx the kernel macro is locked against that
//! redefinition (OXIDIZED_DESIGN_DIVERGENCES #236). Perl has no abntex2cite
//! binding and loses the list too. The
//! package's own `\cite` (abntex2cite.sty:748/:862, a `\DeclareRobustCommand`
//! resolving numbers from the `.aux`) typeset `(??)` and registered no
//! citation, so no entry was ever selected; `\cite` and `\citeonline` (the
//! textual form) keep the kernel's citation constructor, which the
//! bibliography stage resolves.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!(r"\let\lx@abnt@orig@cite\cite");
  let opts: Vec<String> = lookup_vecdeque("opt@abntex2cite.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("abntex2cite", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
  // `\bibliography` itself is the kernel's, locked against the package's
  // `\@input{\jobname.bbl}` redefinition since batch 56cx (sect11.rs).
  RawTeX!(r"\let\cite\lx@abnt@orig@cite\let\citeonline\lx@abnt@orig@cite");
});
