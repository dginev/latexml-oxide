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
LoadDefinitions!({
  InputDefinitions!("apxproof", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
