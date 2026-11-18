//! gtpart.cls — Geometry & Topology Publications' class (Colin Rourke, 2006; not in TeX Live). A paper that ships it has
//! it read raw; one whose source lacks it gets OmniBus with the loader's missing-file Warning, and the class's
//! publication metadata setters, which print nothing (gtpart.cls:573-600, `\def\corresponding#1{\relax}` …): the
//! template asks for them left empty (`\corresponding{}`, the corresponding editor's, beside `\proposed` and
//! `\seconded`; no author's correspondence). OmniBus's `\doi`, `\received`, `\revised` and `\accepted` stay its own.
//! 1108.0474, 1201.1579, 1301.0616, 1301.5693, 1410.6974, 1512.00353 (no copy); 0901.0208, 2404.01499 (a copy).
use latexml_package::prelude::*;

const GTPART: crate::shipped_class::RawRead = crate::shipped_class::RawRead {
  shipped: true,
  ..crate::shipped_class::RawRead::new("gtpart", "cls")
};

LoadDefinitions!({
  if crate::shipped_class::read_raw(&GTPART)? {
    return Ok(());
  }
  // gtpart.cls:573-600 (`\MR`'s link at :212 is redefined so at :576)
  RawTeX!(
    r"\def\givenname#1{\relax}\def\surname#1{\relax}\def\MR#1{\relax}\def\Zbl#1{\relax}
\def\issuenumber#1{\relax}\def\publishedonline#1{\relax}\def\proposed#1{\relax}\def\seconded#1{\relax}
\def\corresponding#1{\relax}\def\editor#1{\relax}\def\version#1{\relax}\def\arxivreference#1{\relax}
\def\arxivpassword#1{\relax}\def\volumenumber#1{\relax}\def\volumename#1{\relax}\def\publicationyear#1{\relax}
\def\papernumber#1{\relax}\def\startpage#1{\relax}\def\endpage#1{\relax}\def\published#1{\relax}"
  );
});
