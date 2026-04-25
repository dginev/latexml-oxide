//! babelbib.sty — multilingual bibliography support.
//!
//! No Perl binding upstream. ~9 sandbox papers (1812.01892 …
//! 1812.11376) hit `\selectbiblanguage` undefined when their
//! generated `.bbl` switches language per entry. babelbib's per-entry
//! commands need no XML-mode handling — emit nothing for language
//! switches; the surrounding text is already locale-neutral in our
//! pipeline.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Per-entry / per-document language switches.
  DefMacro!("\\selectbiblanguage{}", "");
  DefMacro!("\\setbtxfallbacklanguage{}", "");
  DefMacro!("\\setdefaultbiblanguage{}", "");
  DefMacro!("\\biblanguage{}", "");
  // Entry markers (mostly already passthrough in natbib).
  DefMacro!("\\bibsforlanguage{}{}", "#2");
  // Translation hooks.
  DefMacro!("\\captionsbib{}", "");
  DefMacro!("\\biblanguageoptions{}", "");
});
