//! rsphrase.sty — R-/S-phrases (the legacy EU "risk and safety" sentences for
//! chemical hazards), part of the mhchem bundle. Provides `\rsphrase[lang]{N}`,
//! which renders the localized phrase text for risk/safety number `N`.
//!
//! Perl LaTeXML ships no `rsphrase.sty.ltxml`; at the default `notex` setting
//! it does not raw-load the real file either, so a `\usepackage{rsphrase}`
//! document is parity (both error on `\rsphrase`). This contrib binding goes
//! beyond Perl by raw-loading the genuine TL `rsphrase.sty` — a classic
//! `\newcommand`/`\ifthenelse`-based package (its phrase tables are plain
//! token data, no expl3), so the real localized text renders unmodified.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("rsphrase", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
