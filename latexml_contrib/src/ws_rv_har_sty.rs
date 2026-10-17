//! ws-rv-har.sty — World Scientific's renamed copy of natbib 7.x (`\ProvidesPackage{ws-rv-har}`, ws-rv-har.sty:16,
//! "derived from natbib.sty"), the author-year citations of its review-volume class ws-rv9x6 (the document's
//! `\usepackage{ws-rv-har}`, 2301.03442; 1602.07982 ships it and loads ws-rv-van). A stand-in for the author's file
//! (author_files.rs `AUTHOR_FILE_STAND_INS`): it is natbib, so the natbib binding loads with its options
//! (`njdnatbib_sty::natbib_fork_options`), and the paper's copy, read after it, only provides
//! (OXIDIZED_DESIGN_DIVERGENCES #731). Read raw, its `\citep` found no `\bibitem` its `\bibitem` had read: 2301.03442's
//! 28 citations "(?)", where pdflatex prints "(Mayor et al. 2011)" (`\bibliographystyle{aa}`, natbib's
//! `\bibstyle@aa`).
//!
//! The default it changes that the PDF shows: a class with chapters heads the bibliography `\section*{\bibname}`
//! (:681-685; natbib `\chapter*{\bibname}`), one without `\refname` (:677) — a class with chapters has a `chapter`
//! counter, as ws-rv9x6.cls:399 declares (its `\bibname` "References", :91, 2301.03442's PDF). Its punctuation
//! defaults (:244-249: `(`, `)`, `;`, `,`) are natbib's; its superscript postnote in parentheses (:256-259) shows only
//! under `super`.
use latexml_package::prelude::*;

LoadDefinitions!({
  crate::njdnatbib_sty::natbib_fork_options()?;
  RawTeX!(r"\@ifundefined{c@chapter}{}{\def\lx@bibliography@title{\bibname}}");
});
