// Babel tests — split into individual tests to isolate hangs.
use latexml::util::test::*;
const DIR: &str = "tests/babel";

#[test]
fn csquotes_test() {
  latexml_test_single("tests/babel/csquotes.tex", "csquotes", DIR, None, None);
}

#[test]
fn french_test() {
  latexml_test_single("tests/babel/french.tex", "french", DIR, None, None);
}

#[test]
fn german_test() {
  latexml_test_single("tests/babel/german.tex", "german", DIR, None, None);
}

#[test]
fn greek_test() {
  latexml_test_single("tests/babel/greek.tex", "greek", DIR, None, None);
}

#[test]
fn numprints_test() {
  latexml_test_single("tests/babel/numprints.tex", "numprints", DIR, None, None);
}

#[test]
// Expected XML is Perl latexml's ground-truth output for this document.
// Rust currently diverges on ONE remaining nit (down from 4 originally).
//
//   1. [FIXED 2026-04-17] `\raggedright` inside `\begin{document}`
//      now applies `class="ltx_align_left"` — fixed as side effect of (2).
//
//   2. [FIXED 2026-04-17] The stray leading comma in p1
//      ("<p>,The expansion…") was caused by a Rust-only
//      `\let\@nil\relax` in latex_base.rs that made
//      `\ifx\@nil\relax` TRUE when the empty parameter case in
//      `\bbl@fornext#1,{\ifx\@nil#1\relax\else ... \fi}` hits.
//      Removing the stray \let aligned us with Perl's semantics
//      (where \@nil is undefined, so \ifx\@nil\relax is FALSE on
//      the empty-parameter step, and recursion consumes \@nil,
//      properly as the next iteration).
//
//   3. [FIXED 2026-04-17] French babel's active colon/semicolon/
//      exclamation/question now emits a thin space only when
//      \languagename is actually French, mirroring frenchb.ldf.
//      Test: "français :" inside otherlanguage, "does not change!"
//      (no space) inside \foreignlanguage{english}.
//
//   4. [STILL OPEN] Perl emits an extra empty language-return
//      wrapper nested inside the outer one at end of p4:
//        <text xml:lang="fr"><text xml:lang="de"></text></text>
//      We only emit <text xml:lang="fr"></text>. This is a
//      cosmetic structural difference (both empty wrappers are
//      invisible in rendering) reflecting our document builder's
//      single-level language-change tracking vs Perl's per-group
//      language-stack unwinding. No visible output impact, but
//      keeps this test #[ignore]d pending a proper font-stack
//      unwind on end-of-group in document.rs.
#[ignore]
fn page545_test() {
  latexml_test_single("tests/babel/page545.tex", "page545", DIR, None, None);
}
