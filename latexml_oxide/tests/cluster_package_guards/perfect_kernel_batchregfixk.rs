//! Red/green guards for batch REGFIXK, two regressions of the v0.8.0 per-paper check (0 errors on 2026-10-08): a
//! `\bibitem` label a later `\cite` prints, kept raw past the group its `.bbl` was read in (alpha.bst's `\etalchar`),
//! and amstext's `\text` raising the alignment count before it reads its argument.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// index-bib/etalchar_grouped: `{\small\bibliography{…}}` reads an alpha.bst `.bbl` that defines `\etalchar`, cited
/// after the group (1101.1169, 2402.10127, 2203.12786; 30 papers of the check). `\@lbibitem` writes the label to the
/// `.aux` expanded under `\let\protect\noexpand` (latex.ltx:17746-17749), so the `\cite` prints "ADH+19" (pdflatex 0
/// errors); kept raw, the refnum met an undefined `\etalchar` (INTREGFIX4 1 Error). Distinct from the OD #750 mn2e
/// guards (`perfect_kernel_batch70c`): there the `\protect`ed label waits for the `\cite`; here the label's macro
/// expands at the `\bibitem`.
#[test]
fn etalchar_label_cited_after_its_bbl_group() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/etalchar_grouped/etalchar_grouped.tex"
  );
  let files = [(
    // the jobname's .bbl, as `\bibliography` reads it (the harness names the document t.tex)
    "t.bbl",
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/etalchar_grouped/etalchar_grouped.bbl"
    ),
  )];
  let (log, xml) = convert_files_with(tex, &files, Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_element(
    &xml,
    "bibitem",
    &[r#"key="arora""#],
    "<bibitem key=\"arora\" xml:id=\"bib.bibx1\"><tags><tag>[ADH<sup xml:id=\"bib.bibx1.1\"><text font=\"italic\" xml:id=\"bib.bibx1.1.1\">+</text></sup>19]</tag>\
     <tag role=\"refnum\">ADH<sup xml:id=\"bib.bibx1.2\"><text font=\"italic\" xml:id=\"bib.bibx1.2.1\">+</text></sup>19</tag></tags>\
     <bibblock><text fontsize=\"90%\" xml:id=\"bib.bibx1.3\">\nS.\u{a0}Arora et al.\n</text></bibblock>\
     <bibblock><text fontsize=\"90%\" xml:id=\"bib.bibx1.4\">Fine-grained analysis.\n</text></bibblock></bibitem>",
  );
}

/// kernel-alignment/text_without_braces_before_a_cell_end: `$x$\text & 0.6` in a tabular row (2212.00883's
/// biophys-new table). `\text` reads `#1` as a macro does, at the cell's count, so the `&` ends the cell (tex.web §342,
/// pdflatex 0 errors, "x | 0.6"); the count raised before the read (int19, KNOWN_PERL_ERRORS #931) took the `&` as
/// the argument (INTREGFIX4 3 errors). Distinct from `perfect_kernel_batchint19::text_argument_is_braced_in_an_alignment_cell`,
/// the braced argument's own column ends.
#[test]
fn text_without_braces_before_a_cell_end() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/kernel-alignment/text_without_braces_before_a_cell_end.tex"
  );
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tr",
    &[r#"xml:id="p1.1.2""#],
    "<tr xml:id=\"p1.1.2\"><td align=\"left\" xml:id=\"p1.1.2.1\"><Math mode=\"inline\" tex=\"x\" text=\"x\" \
     xml:id=\"p1.m1\"><XMath xml:id=\"p1.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math>\
     <text xml:id=\"p1.1.2.1.1\"/></td><td align=\"left\" xml:id=\"p1.1.2.2\">0.6</td></tr>",
  );
}

/// macro-state/text_survives_an_edef: `\text` is a protected macro (amstext.sty:28 `\DeclareRobustCommand`; Perl's
/// constructor is unexpandable), so a plain `\edef` and a `\protected@edef` keep `\text` (OXIDIZED_DESIGN #1104); round
/// 1's unprotected macro showed `\lx@text {z}`. Distinct from `text_without_braces_before_a_cell_end`: the
/// expansion-only contexts, not the digestion.
#[test]
fn text_survives_an_edef() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/macro-state/text_survives_an_edef.tex");
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\"><text font=\"typewriter\" xml:id=\"p1.1.1\">macro:-&gt;\\text {z}</text> \
     <text font=\"typewriter\" xml:id=\"p1.1.2\">\\protected macro:#1-&gt;\\lx@text {#1}</text>\n\
     <text font=\"typewriter\" xml:id=\"p1.1.3\">macro:-&gt;\\text {q}</text>\n\
     <Math mode=\"inline\" tex=\"x_{\\text{max}}\\text{z}\\text{q}\" text=\"x _ [max] * [zq]\" xml:id=\"p1.m1\">\
     <XMath xml:id=\"p1.m1.1\"><XMApp xml:id=\"p1.m1.1.1\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\
     <XMApp xml:id=\"p1.m1.1.1.2\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\
     <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMText xml:id=\"p1.m1.1.1.2.3\">\
     <text fontsize=\"70%\" xml:id=\"p1.m1.1.1.2.3.1\">max</text></XMText></XMApp>\
     <XMText xml:id=\"p1.m1.1.1.3\"><text xml:id=\"p1.m1.1.1.3.1\">z</text><text xml:id=\"p1.m1.1.1.3.2\">q</text>\
     </XMText></XMApp></XMath></Math></p>",
  );
}
