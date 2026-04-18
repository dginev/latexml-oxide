//! english.sty — legacy english language support, advises babel
//! Perl: english.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // english.sty advises to do \usepackage[english]{babel} instead
  // PassOptions not yet supported; just load babel directly
  RequirePackage!("babel");

  // English captions. With the @currname fix (56b0c35d2), babel's own
  // \\captions<lang> usually defines these, but on the `\\usepackage
  // [english]{babel}` path some class-option orderings (e.g.
  // `[german]{article}` + `[french,english]{babel}` in page545_test) still
  // leave \\captionsenglish undefined at the time \\select@language fires.
  // \\providecommand supplies a safe fallback.
  RawTeX!(r"\providecommand\captionsenglish{%
    \def\prefacename{Preface}\def\refname{References}%
    \def\abstractname{Abstract}\def\bibname{Bibliography}%
    \def\chaptername{Chapter}\def\appendixname{Appendix}%
    \def\contentsname{Contents}%
    \def\listfigurename{List of Figures}%
    \def\listtablename{List of Tables}%
    \def\indexname{Index}\def\figurename{Figure}%
    \def\tablename{Table}\def\partname{Part}%
    \def\pagename{Page}\def\seename{see}%
    \def\alsoname{see also}\def\proofname{Proof}}");
  RawTeX!(r"\providecommand\dateenglish{}");
});
