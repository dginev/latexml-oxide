//! clear2025.cls (CLeaR 2025 proceedings, PMLR; author-bundled): `\LoadClass[pmlr]{jmlr}` after its own `anon` and
//! `final` options, and `\clearauthor`, the review form `\newcommand{\clearauthor}[1]{}` or the final form
//! `\newcommand{\clearauthor}[1]{\author{#1}}` (2412.17080, 2503.16206). The base class is jmlr, as the class loads it
//! (OXIDIZED_DESIGN_DIVERGENCES #736): the paper's copy, read after the binding, finds it loaded. `\clearauthor` is the
//! final form, as colt2024_cls.rs's `\coltauthor`.
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!("anon", None);
  DeclareOption!("final", None);
  ProcessOptions!();
  LoadClass!("jmlr", vec![s!("pmlr")], Tokens!());
  DefMacro!("\\clearauthor{}", "\\author{#1}");
});
