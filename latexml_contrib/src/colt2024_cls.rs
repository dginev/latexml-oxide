//! Stub for colt2024.cls (COLT 2024 — Conference on Learning Theory); also
//! serves the sibling PMLR proceedings classes via the registry — colt2025,
//! colt2026, l4dc2026 (Learning for Dynamics & Control), neus2025 (NeuS) — all
//! identical in the load-shape (`\LoadClass[pmlr]{jmlr}` +
//! `\coltauthor`→`\author`). This is the concrete instance of the general
//! "unknown author-bundled PMLR class" pattern (PLANS 13(f): raw-loader native
//! fallbacks); until that generalization lands, new PMLR proceedings classes are
//! routed here by name.
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
