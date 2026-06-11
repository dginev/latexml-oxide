//! scicite.sty — Science (the journal) citation style
//!
//! `scicite.sty` is a slightly-modified `cite.sty` (D. Arseneau, 1989-2003)
//! tailored for the journal Science. The raw file is 513 lines of
//! `\edef`/`\catcode` manipulation that Rust's tokenizer chokes on (the
//! 7-paper hang cluster in the 10k_errors v4 sandbox: 1010.2781, 1011.5494,
//! 1102.0562, 1210.1294, 1303.2601, 1704.07345, 1706.03851 + similarly-named
//! files).
//!
//! Perl LaTeXML has no `scicite.sty.ltxml` and instead inherits cite.sty's
//! Perl binding via the raw scicite.sty load picking up cite.sty's
//! `\citen` / `\citenum` / `\citeonline` closures. Our stub bypasses
//! the raw load (to avoid the tokenizer hang) but we restore the
//! citation-CS chain by requiring the `cite` package's binding, which
//! defines `\citen` as a full closure and Lets `\citenum` / `\citeonline`
//! to it. Then we layer the Science-specific punctuation overrides.
//!
//! TODO (root cause): make Rust's tokenizer survive scicite.sty's catcode
//! dance so this stub can be removed entirely (a `feedback_prefer_raw_load`
//! pattern — Perl raw-loads scicite.sty and inherits cite.sty's closures
//! through the chain).

use latexml_package::prelude::*;

LoadDefinitions!({
  // Inherit cite.sty's full citation-CS chain: `\citen` is a closure
  // (natbib-style multi-args), `\citenum` and `\citeonline` are Let to
  // `\citen`. Loading the binding pre-empts the raw scicite.sty load
  // that would otherwise hang our tokenizer.
  RequirePackage!("cite");

  // Science-journal punctuation overrides. cite.sty's defaults are
  // `[`/`]` (matching scicite) and `, ` (matching scicite) — keep these
  // explicit to make scicite-specific tuning at the post-XSLT layer
  // (per-journal style) discoverable here.
  DefMacro!("\\citeleft", "[");
  DefMacro!("\\citeright", "]");
  DefMacro!("\\citedash", "--");
  DefMacro!("\\citemid", ", ");
  DefMacro!("\\citepunct", ", ");
  DefMacro!("\\citeform{}", "#1");

  // scicite-specific option no-op (papers often write
  // `\usepackage[<opt>]{scicite}` — option processing without us).
  def_macro_noop("\\nocitepunct")?;
});
