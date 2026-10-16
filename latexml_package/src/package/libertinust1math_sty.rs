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
  // libertinust1math.sty:59 reads its options with `\ProcessOptionsX`, which marks them processed: `\@curroptions` stays as it was.
  key_options_processed()?;
  // Kernel math chars are authoritative: the binding re-declares none of them.
  //
  // libertinust1math.sty:7 `\RequirePackage{xkeyval,etoolbox}` and :75 `\RequirePackage{amsmath}` (stubs load real
  // dependencies): a document's `\DeclareMathOperator`/`\operatorname`/`\text` came from these (2110.09723,
  // 2201.10869, 2208.02596).
  RequirePackage!("xkeyval");
  RequirePackage!("etoolbox");
  RequirePackage!("amsmath");
  // The package's own names papers reach for, which no kernel binding defines (a name a package defined first keeps
  // its definition): :166 `\mathbcal` (`\DeclareMathAlphabet{\mathbcal}{U}{stixcal}{b}{n}`, the bold calligraphic
  // letters) and :129 `\mathbfit` (the `bold-letters` alphabet, bold italic) — 2603.08217, 2605.03875; :1603
  // `\widebar` (9 papers: 2311.14681, 2405.05597, 2411.03279) and :912 `\widecheck` (2602.14897), the wide accents;
  // :1188-1189 `\lesssim` (2408.05538) and `\gtrsim` (2208.02596), :1443 `\blacksquare` (2412.05468), :643
  // `\varkappa`, :1377 `\eqdef` (2506.09249), :1215 `\cupdot`, :872 `\bigcupdot` (2512.09624), :835 `\oiint`
  // (2605.03888). Guard:
  // `perfect_kernel_batch68j::libertinust1math_loads_amsmath_and_its_names`.
  if !is_defined("\\mathbcal") {
    DefConstructor!("\\mathbcal{}", "#1", bounded => true, require_math => true,
      font => { family => "caligraphic", series => "bold", shape => "upright" });
  }
  if !is_defined("\\mathbfit") {
    DefConstructor!("\\mathbfit{}", "#1", bounded => true, require_math => true,
      font => { family => "serif", series => "bold", shape => "italic" });
  }
  if !is_defined("\\widebar") {
    DefMath!("\\widebar Digested", "\u{00AF}", operator_role => "OVERACCENT");
  }
  if !is_defined("\\widecheck") {
    DefMath!("\\widecheck Digested", "\u{02C7}", operator_role => "OVERACCENT");
  }
  if !is_defined("\\lesssim") {
    DefMath!("\\lesssim", None, "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  }
  if !is_defined("\\gtrsim") {
    DefMath!("\\gtrsim", None, "\u{2273}", role => "RELOP", meaning => "greater-than-or-similar-to");
  }
  if !is_defined("\\blacksquare") {
    DefMath!("\\blacksquare", None, "\u{25A0}");
  }
  if !is_defined("\\varkappa") {
    DefMath!("\\varkappa", None, "\u{03F0}");
  }
  if !is_defined("\\eqdef") {
    DefMath!("\\eqdef", None, "\u{225D}", role => "RELOP");
  }
  if !is_defined("\\cupdot") {
    DefMath!("\\cupdot", None, "\u{228D}", role => "ADDOP");
  }
  if !is_defined("\\bigcupdot") {
    DefMath!("\\bigcupdot", "\u{2A03}", role => "BIGOP", scriptpos => "mid", variablesize_op => true);
  }
  if !is_defined("\\oiint") {
    DefMath!("\\oiint", None, "\u{222F}", role => "INTOP", meaning => "surface-integral", variablesize_op => true);
  }
});
