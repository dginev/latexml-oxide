//! libertinust1math.sty — Libertine Unicode math font support.
//!
//! Same `\libus@undefine#1\let#1=\@undefined` pattern as fdsymbol.sty
//! (see [`crate::package::fdsymbol_sty`] for the full rationale): the
//! package re-declares every math symbol by first
//! `\let\X\@undefined`-ing it and then `\DeclareMathSymbol{\X}{...}`-ing
//! it back. Our kernel math chars (`\prime`, `\sum`, `\int`, ...) are
//! locked at `DefMath!` time, so the re-DeclareMathSymbol is silently
//! blocked and the previous `\let \@undefined` wins, cascading
//! "undefined:\prime" errors for every formula that uses primes.
//!
//! Perl LaTeXML never raw-loads libertinust1math.sty (Perl-default
//! `INCLUDE_STYLES=false`), so the kernel `\prime`/`\sum`/... stay
//! intact. Match that behaviour: no-op binding. Font choice is moot for
//! XML/HTML output. Witness 2406.04255, 2406.04389, 2407.16071
//! (~28 papers cumulative, up to 501 `\prime` errors per paper).
use crate::prelude::*;

LoadDefinitions!({
  // Intentionally empty — kernel math chars are authoritative.
});
