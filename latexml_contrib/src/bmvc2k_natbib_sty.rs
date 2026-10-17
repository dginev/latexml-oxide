//! bmvc2k_natbib.sty — the BMVC class's renamed copy of natbib 8.1 (`\ProvidesPackage{bmvc2k_natbib}`,
//! bmvc2k_natbib.sty:29), which bmvc2k.cls:710 loads as `\usepackage[sort,numbers]{bmvc2k_natbib}`. A stand-in for the
//! author's file (author_files.rs `AUTHOR_FILE_STAND_INS`): it is natbib, so the natbib binding loads with the options
//! the class gave the copy, and the paper's copy, read after it, only provides (OXIDIZED_DESIGN_DIVERGENCES #731):
//! read raw, its `\bibsection`'s `\@ifundefined{chapter}` (:779) took the `\chapter*` branch, heading the references
//! "Bibliography" with `bib.bibxN` ids (2412.14509, 2212.00057, 2210.06270, 1501.07873, 2311.17607, 2406.08486;
//! pdflatex 2412.14509: "References"). Against TL's natbib 8.31b it changes no default the PDF shows (its `\bibstyle`
//! tests for the style, its spaces are `\ `).
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{natbib}")?;
  });
  ProcessOptions!();
  RequirePackage!("natbib");
});
