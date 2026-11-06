//! multibib: the raw package, with each list's bibliography style recorded.
//!
//! multibib.sty:234 defines `\bibliographystyle<suffix>` to write `\bibstyle` to the side aux file bibtex reads
//! (`<suffix>.aux`); it never calls `\bibliographystyle`, so `BIBSTYLE`, which `\bibliographystyle` records for the
//! `.bib` route (`\lx@record@bibstyle`, sect11.rs), stayed empty. A document with only such lists then read no `.bst`
//! (`bst_declared_fields`, OXIDIZED_DESIGN_DIVERGENCES #823): the fields its style declares stayed raw data
//! (`urldate`, quantum.bst:44, printed "(accessed:~12.07.2019)" at :719; the quantum-bibliographystyle-demo manual).
//! One `BIBSTYLE` holds per document: of mixed styles, a list reads the one in force at its `\bibliography<suffix>`,
//! and the `.bst` fields are the last's.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("multibib", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  RawTeX!(
    r"\def\lx@mb@record#1{\@ifundefined{bibliographystyle#1}{}{%
    \expandafter\let\csname lx@mb@style@#1\expandafter\endcsname\csname bibliographystyle#1\endcsname
    \@namedef{bibliographystyle#1}##1{\lx@record@bibstyle{##1}\@nameuse{lx@mb@style@#1}{##1}}}}
\let\lx@mb@newcites\newcites
\def\newcites#1#2{\lx@mb@newcites{#1}{#2}%
  \@for\@tempa:=#1\do{\expandafter\lx@mb@record\expandafter{\@tempa}}}"
  );
});
