use crate::prelude::*;

// xfor.sty — extended `\@for` loop with `break` support via `\@endfor`.
// Plain-TeX `\def`+`\newif` only; no expl3, no xparse. 85 lines.
//
// Perl LaTeXML has no `xfor.sty.ltxml` — its `glossaries.sty.ltxml`
// raw-loads `glossaries.sty`, whose `\RequirePackage{xfor}` then
// causes Perl to raw-load `xfor.sty` directly. Rust's default
// `INCLUDE_STYLES=false` blocks that path, so this shim forces
// raw-load via `noltxml=>true` (mirroring the
// `InputDefinitions(noltxml=>1)` pattern Perl's `glossaries.sty.ltxml`
// uses for its primary load).
//
// First step of the SYNC_STATUS "raw-load enablement" plan
// (commit d5a1334ea0): xfor is the smallest dependency in the
// glossaries dep-chain (mfirstuc, xfor, datatool-base, ...) and
// has no expl3 dependency itself, so it's the natural first
// proof-of-concept for the pattern.

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("xfor", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
