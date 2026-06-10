//! Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory).
//!
//! colt2024.cls does `\LoadClass[pmlr]{jmlr}` and defines `\coltauthor` in its
//! body: the review form `\newcommand{\coltauthor}[1]{}` (anonymizes), the final
//! form `\newcommand{\coltauthor}[1]{\author{#1}}`. As an unbound class Rust uses
//! the OmniBus fallback, which dep-scans the cls (so jmlr loads) but does NOT
//! execute the cls body — so `\coltauthor` was undefined where Perl raw-executes
//! the cls and defines it. Route to jmlr (Rust's jmlr binding handles `\editor`/
//! `\jmlrworkshop`/`\acks`, the rest of the cls body) and define `\coltauthor` as
//! the final-submission form `\author{#1}` — content-preserving (real author
//! names, correct for published arXiv versions). Witness 2308.08218.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("jmlr");
  DefMacro!("\\coltauthor{}", "\\author{#1}");
});
