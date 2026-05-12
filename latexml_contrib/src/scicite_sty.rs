//! scicite.sty — Science (the journal) citation style
//!
//! `scicite.sty` is a slightly-modified `cite.sty` (D. Arseneau, 1989-2003)
//! tailored for the journal Science. The raw file is 513 lines of
//! `\edef`/`\catcode` manipulation; without a Rust binding we attempt to
//! tokenize the whole thing and hang on its catcode dance.
//!
//! Like `cite.sty.ltxml` / `cite_sty.rs` (Perl + Rust), we short-circuit
//! the raw load by defining only the public-API macros that downstream
//! documents actually invoke (`\citeleft`, `\citeright`, `\citepunct`,
//! …). Citation list compression / sorting / styling has no equivalent
//! in XML output anyway — formatting is handled at the post-XSLT layer.
//!
//! Recovers the 7-paper hang cluster in the 10k_errors v4 sandbox
//! (1010.2781, 1011.5494, 1102.0562, 1210.1294, 1303.2601, 1704.07345,
//! 1706.03851 + similarly-named files) seen in v4 partial logs.

use latexml_package::prelude::*;

LoadDefinitions!({
  // Mirror `cite_sty.rs` — defaults for citation formatting macros.
  // scicite.sty has slightly different defaults (Science-journal
  // bracketing/punctuation) but the XML-side semantics are the same:
  // emit a parenthesized comma-separated list. Override at the post
  // layer if a per-journal style is desired.
  DefMacro!("\\citeleft",  "[");
  DefMacro!("\\citeright", "]");
  DefMacro!("\\citedash",  "--");
  DefMacro!("\\citemid",   ", ");
  DefMacro!("\\citepunct", ", ");
  DefMacro!("\\citeform{}", "#1");

  // scicite-specific options that papers often set in the preamble.
  // No-op them so `\usepackage[option]{scicite}` doesn't error.
  DefMacro!("\\nocitepunct", "");
  DefMacro!("\\citen", "\\cite");
});
