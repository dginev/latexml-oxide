use latexml_package::prelude::*;

// apxproof.sty — "Automatic proofs in appendix" (Senellart, CTAN).
//
// No Perl LaTeXML binding exists (neither upstream nor ar5iv-bindings),
// so Perl relies on raw-loading apxproof.sty under --includestyles and
// *fails*: apxproof L58 `\ProcessLocalKeyvalOptions*` trips Perl's
// kvoptions handling ("intended for packages only") → the whole
// bibliography is dropped (0 rendered entries). See
// docs/parity/KNOWN_PERL_ERRORS.md.
//
// This binding's single action is to force the raw .sty to load in EVERY
// config (bare, --includestyles, ar5iv), not only under raw-style loading.
// That is required because apxproof, when merely skipped (missing_file in
// bare mode), still leaves a `<?latexml package="apxproof"?>` footprint
// while its biblatex citation wiring never runs — so `\cite`s render
// unlinked and every entry warns `expected:ids Missing Entry`. Forcing the
// raw load runs apxproof's real setup (it internally `\RequirePackage`s
// biblatex, kvoptions, etc.), so proofs keep LaTeXML's usual amsthm
// `ltx_proof` markup and citations resolve against the bibliography.
//
// Rust's kvoptions raw-load handles `\ProcessLocalKeyvalOptions*`, so the
// raw apxproof.sty loads cleanly where Perl aborts — a faithful
// surpass-Perl improvement.
//
// apxproof builds on amsthm's proof rather than replacing it: it keeps the binding as `\axp@oldproof`
// (apxproof.sty:136-137) and its `\def\proof` (:393-417) sends the proof of a `theoremrep` to the appendix file, which
// reads it back as `\begin{axp@oldproof}`, and every other proof to `\axp@oldproof`. So amsthm's lock against the
// class and package files (amsthm_sty.rs, OXIDIZED_DESIGN_DIVERGENCES #596) yields while the raw file loads — also to
// its `\let\proof\apx@oldamsthmproof` (:46), which puts back a proof defined before amsthm — and holds again after
// it: without the yield, 2502.18805's 53 proofs stayed in the body instead of the appendix, after its restated
// theorems, where pdflatex prints them. Guard `perfect_kernel_batch67s::apxproof_wraps_the_locked_proof`.
LoadDefinitions!({
  AssignValue!("\\proof:locked@files" => false, Some(Scope::Global));
  AssignValue!("\\endproof:locked@files" => false, Some(Scope::Global));
  InputDefinitions!("apxproof", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  AssignValue!("\\proof:locked@files" => true, Some(Scope::Global));
  AssignValue!("\\endproof:locked@files" => true, Some(Scope::Global));
});
