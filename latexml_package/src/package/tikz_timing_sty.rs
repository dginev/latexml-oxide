//! No-op stub for tikz-timing.sty.
//!
//! Why a stub instead of raw-load: tikz-timing.sty (TL) defines
//! macros that use `\xdef body{...\value{tikztimingtrans}...}` *before*
//! its `\newcounter{tikztimingtrans}` runs. During the `\xdef`'s eager
//! expansion, `\c@tikztimingtrans` is undefined and our `readBalanced`
//! triggers `generate_error_stub` — exactly mirroring Perl
//! `Gullet.pm:518`'s `$STATE->generateErrorStub($self, $token)` (L517-519).
//!
//! Perl LaTeXML never observes this because its default TEXINPUTS
//! doesn't include `/usr/share/texlive`, so `tikz-timing.sty` is
//! reported missing and skipped. Our `FindFile` does walk the TL tree,
//! so we attempt the raw-load and hit the same readBalanced error path
//! Perl would have hit had it found the file.
//!
//! Matching Perl's *effective* behavior (= no-op, file missing) by
//! providing a no-op stub at the binding layer. Timing-diagram authors
//! who want actual diagrams need a future real binding; the cluster of
//! 8 R-stage failures observed are papers that load `tikz-timing` in
//! the preamble but never use `\begin{tikztimingtable}` or
//! `\tikztiming…`. Witness: arXiv:1912.11312.

use crate::prelude::*;

LoadDefinitions!({});
