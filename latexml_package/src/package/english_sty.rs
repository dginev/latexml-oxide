//! english.sty — legacy english language support, advises babel
//! Perl: english.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // english.sty advises to do \usepackage[english]{babel} instead
  // PassOptions not yet supported; just load babel directly
  RequirePackage!("babel");

  // English captions — must reset all names when switching from other languages.
  // Equivalent to what babel's english.ldf defines.
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
