//! natbib `\bibitem` label with a dotless-i (`\i`) must not infinite-loop.
//!
//! Root cause (2111.00584, revtex4-1 + aipnum `.bbl`): natbib's
//! `\lx@NAT@parselabel` fully-expands a "bare" bibitem label (to locate the
//! `(year)` paren). Under `[T1]{fontenc}` (here via mathptmx) the LaTeX kernel
//! redefines `\i` to the `\@changed@cmd` dispatcher `\T1-cmd \i \T1\i`, whose
//! typeset branch re-injects `\i` through
//! `\csname\cf@encoding\string\i\endcsname`. Under full `Expand!` that
//! re-expands forever → `Fatal:Timeout:PushbackLimit` + a box-list runaway,
//! and the aborted bibliography then emits dozens of
//! `malformed:ltx:bibitem in <ltx:bibblock>` errors. Perl's `Expand`
//! (natbib.sty.ltxml:564) happens to terminate on these; ours did not.
//!
//! Fix: extend `\lx@NAT@parselabel`'s "don't force-expand" guard (already
//! covering `\cite`/`\href`/`\bibinfo`) to text-encoding symbol commands
//! (`\i`, `\j`, `\ss`, `\oe`, …). The `(year)` is always a literal paren in
//! natbib/BibTeX output, so the raw label is sufficient.
//!
//! Fixture faithfulness: the label wraps its author in `\citenamefont`, which
//! is supplied by the revtex4-1 `.bbl` `\providecommand` preamble
//! (aipnum4-1.bst), NOT by natbib/revtex. The distilled reproducer originally
//! dropped that preamble, so the conversion logged a (parity, both-engine)
//! `undefined:\citenamefont` Error that the test silently tolerated. Restoring
//! the preamble mirrors a real `.bbl`, drops the run to 0 errors, AND
//! strengthens the guard test — `\citenamefont{…}` now expands to the dotless
//! `\i` inside `\lx@NAT@parselabel`, the exact path that must not loop.
//!
//! Conditional: needs the kernel dump (so expl3/pgf load cleanly) AND
//! revtex4-1 + mathptmx + pgfplots installed (the exact package set drives
//! the encoding state into the looping `\T1-cmd` form).
use latexml::util::test::{convert_fixture, dump_available, kpse_has};

#[test]
fn natbib_dotless_i_label_does_not_loop() {
  if !dump_available() {
    eprintln!(
      "SKIP natbib_dotless_i_label_does_not_loop: no latex kernel dump \
       in resources/dumps/ (run tools/make_formats.sh)"
    );
    return;
  }
  if !kpse_has("revtex4-1.cls") || !kpse_has("mathptmx.sty") || !kpse_has("pgfplots.sty") {
    eprintln!(
      "SKIP natbib_dotless_i_label_does_not_loop: revtex4-1/mathptmx/pgfplots \
       not installed in the host TeX tree"
    );
    return;
  }

  let r = convert_fixture("tests/cluster_regressions/natbib_label_dotless_i.tex");

  assert!(
    r.result.is_some(),
    "conversion produced no result — the `\\i`-in-natbib-label expansion loop \
     likely re-triggered (status_code={})",
    r.status_code
  );
  assert!(
    !r.log.contains("PushbackLimit") && !r.log.contains("Infinite digestion loop"),
    "detected an infinite-expansion / infinite-digestion fatal — \
     `\\lx@NAT@parselabel` is force-expanding a text-encoding symbol again"
  );
  assert!(
    r.status_code < 3,
    "conversion hit a fatal (status_code={}) — expected a clean run",
    r.status_code
  );
  // Strict: the faithful `.bbl` `\providecommand` preamble (aipnum4-1.bst)
  // supplies `\citenamefont` et al., so the conversion is now fully clean.
  // Previously the distilled fixture dropped that preamble and silently
  // tolerated an `undefined:\citenamefont` Error — a passing test that emitted
  // an error. Assert 0 so any future regression (a re-emerging loop-recovery
  // error, or a real binding gap) fails here rather than hiding in the log.
  let n_errors = latexml::util::test::error_count(&r.log);
  assert_eq!(
    n_errors, 0,
    "expected 0 errors but the conversion log carried {n_errors} Error:<class>: \
     markers (status_code={})",
    r.status_code
  );
}
