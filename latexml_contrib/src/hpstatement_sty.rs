//! hpstatement.sty — the GHS "hazard" (H) and "precautionary" (P) statements
//! for chemical labelling, part of the mhchem bundle. Provides
//! `\hpstatement[..][..]{Hxxx}` (the statement sentence), `\hpnumber[..][..]{Hxxx}`
//! (the formatted code), and `\hpsetup{keys}`.
//!
//! Perl LaTeXML ships no `hpstatement.sty.ltxml`; at the default `notex`
//! setting it does not raw-load the real file either, so a
//! `\usepackage{hpstatement}` document is parity (both error). This contrib
//! binding goes beyond Perl by raw-loading the genuine TL `hpstatement.sty`.
//! Note: hpstatement is an expl3 package (`\NewDocumentCommand`,
//! `\keys_set:nn`) that pulls its sentence database from per-language data
//! files (`hpstatement.inc/hpstatement-<lang>.inc.sty`) via `\file_input:n`.
//! Raw-loading therefore exercises the engine's expl3 + `\file_input:n` paths;
//! if a future TL layout hides those `.inc.sty` data files from kpathsea the
//! statements degrade to their codes rather than erroring.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("hpstatement", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
