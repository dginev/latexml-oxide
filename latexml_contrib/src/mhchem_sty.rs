//! mhchem.sty — chemical formula typesetting (`\ce`, `\cee`, `\cf`, `\cesplit`,
//! `\bond`, …).
//!
//! **2026-06-27: RAW-LOAD the genuine TL `mhchem.sty`.** Perl LaTeXML ships no
//! `mhchem.sty.ltxml` and raw-loads the real package; the engine's expl3 /
//! xparse / chemgreek support is now mature enough that we can do the same, so
//! this binding simply `InputDefinitions(noltxml)` the real file. Chemistry
//! therefore renders with proper digit subscripts (`\ce{H2O}` → H₂O), charge
//! superscripts (`\ce{SO4^2-}` → SO₄²⁻), reaction arrows (`->`/`<=>`/`->[..]`),
//! bonds, states, `\cesplit`, etc. — full fidelity, matching Perl.
//!
//! This SUPERSEDES the former minimal "`\ce`-as-roman-text" stub (retired here;
//! see git history at `latexml_contrib/src/mhchem_sty.rs` before this commit).
//! The stub hand-shimmed a handful of behaviours (digit→subscript was NOT among
//! them — formulae rendered flat), plus `amsmath`/`graphicx` `RequirePackage`s
//! to paper over the auto-dep-scan gap; the real `mhchem.sty` runs its own
//! `\RequirePackage{expl3,amsmath,calc,…}` chain, so none of that is needed.
//!
//! Known edge-case residuals under raw-load (tracked in docs/SYNC_STATUS.md):
//! `\ce` used inside an amsmath `align*` alignment, and a few `$`-toggle /
//! `\cesplit`-derived example patterns, can still emit `\lx@begin@alignment` /
//! `\lx@end@inline@math` errors. Simple `\ce` (the overwhelmingly common case)
//! is clean.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("mhchem", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
