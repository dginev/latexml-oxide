//! NJDnatbib.sty — Wiley's renamed copy of natbib 8.31b (`\ProvidesPackage{NJDnatbib}`, NJDnatbib.sty:29), which
//! WileyNJD-v2/v5/MRM.cls load in natbib's place (WileyNJDv5.cls:6021-6085 `\usepackage[numbers,super,…]{NJDnatbib}`).
//! A stand-in for the author's file (author_files.rs `AUTHOR_FILE_STAND_INS`): it is natbib, so the natbib binding
//! loads with the options the class gave the copy, and the paper's copy, read after it, only provides
//! (OXIDIZED_DESIGN_DIVERGENCES #731): read raw, its `\bibsection`'s `\@ifundefined{chapter}` took the `\chapter*`
//! branch, heading the references "Bibliography" with `bib.bibxN` ids (2609.08425, 2101.06297, 2211.16474,
//! 2411.11493, 2206.08798, 2602.03829, 2408.07575, 2506.14995). Its one changed default the PDF shows: a class without
//! chapters heads the bibliography "REFERENCES" (:1050-1053, under the copy's own condition; xelatex 2609.08425), an
//! amsart one `\refname` (:1065). Its superscript citations' postnote placement (:366-368) is presentational.
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{natbib}")?;
  });
  ProcessOptions!();
  RequirePackage!("natbib");
  RawTeX!(
    r"\@ifundefined{chapter}{\def\lx@bibliography@title{REFERENCES}}{}\@ifclassloaded{amsart}{\let\lx@bibliography@title\@empty}{}"
  );
});
