//! spectralsequences.sty — Hovey/Barnes spectral-sequence diagrams.
//!
//! The package is built atop expl3 + TikZ with deep gullet trickery
//! (\sseq@DeclareDocumentCommand etc.) that our raw-load can't fully
//! interpret. Each unresolved internal CS triggers a cascade of
//! \tagclass / \structline / \sseq@xycoord / \sseq@thename undefined
//! errors per page that contains a spectral-sequence diagram.
//!
//! We can't realistically reproduce the diagram in XML/HTML output
//! (the result would be a TikZ picture anyway), so this binding skips
//! the raw load and provides minimal user-facing env stubs so the
//! diagram-bearing pages render without cascading errors.
//!
//! Witness: 2503.01123, 2503.08789, 2503.01690, 2503.08930.
use crate::prelude::*;

LoadDefinitions!({
  // Skip the expl3 raw-load entirely. Stub user-facing macros below.

  // Top-level spectralsequences environment — silently drop body
  // (it would render as TikZ; the math content is in the picture).
  DefEnvironment!("{sseqdata}", "");
  DefEnvironment!("{sseqpage}", "");
  DefEnvironment!("{sseq}", "");

  // Spectral-sequence drawing primitives — emit only their text
  // labels (#last arg typically) when reasonable; otherwise gobble.
  DefMacro!("\\class[]{}", "");
  DefMacro!("\\classoptions{}", "");
  DefMacro!("\\d{}{}", "");
  DefMacro!("\\structline[]", "");
  DefMacro!("\\structlineoptions{}", "");
  DefMacro!("\\tagclass{}{}", "");
  DefMacro!("\\replaceclass[]{}", "");
  DefMacro!("\\replacetagclass[]{}", "");
  DefMacro!("\\printpage[]", "");
  DefMacro!("\\differential{}{}{}{}", "");
  DefMacro!("\\quivermod{}{}{}", "");

  // Internal sseq@* helpers that surface in error logs — define as
  // gobble-anything so they don't trip our error stub installer.
  DefMacro!("\\sseq@DeclareDocumentCommand{}{}{}", "");
  DefMacro!("\\sseq@xycoord", "0,0");
  DefMacro!("\\sseq@thename", "");
  DefMacro!("\\sseq@classstyle", "");
});
