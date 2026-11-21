//! Red/green guards for batch REGFIXF, a regression of the v0.8.0 per-paper check (Warning on 2026-10-08, Error on
//! the release candidate): the ACM classes' `\citeauthoryear` labels under the locked `\cite`. Repros under
//! `tools/perfect_kernel/repros/index-bib/acmcite/`.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// sig-alternate-05-2015.cls's `\DeclareRobustCommand\cite{\start@cite23}` (1703.08365; sig-alternate.cls 1806.05694,
/// acm_proc_article-sp.cls 1312.2098) and the older classes' plain `\def\cite{\start@cite23}` (0705.2819, cs0404029):
/// `\start@cite#1#2` `\edef`s the only `\citeauthoryear` the `.bbl` labels hold, then branches; the lock refuses that
/// `\cite` (and the robust one's inner `\cite␣`), and the first kernel `\cite` runs that leading `\edef` with the
/// call's arguments `2`, `3` (sect11.rs `label_macros_from_refused_cite`). Distinct from
/// `perfect_kernel_batchregfixa::mlbib_citeauthoryear_label_under_the_lock`: a robust wrapper, a call with arguments,
/// an `\edef` before a branch that is not run.
#[test]
fn acm_robust_cite_label_under_the_lock() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/acmcite/acm_robust_cite_label_under_the_lock.tex"
  );
  let files = [
    (
      "acmcite.cls",
      include_str!("../../../tools/perfect_kernel/repros/index-bib/acmcite/acmcite.cls"),
    ),
    (
      "acmcite_def.cls",
      include_str!("../../../tools/perfect_kernel/repros/index-bib/acmcite/acmcite_def.cls"),
    ),
  ];
  let plain = tex.replace(r"\documentclass{acmcite}", r"\documentclass{acmcite_def}");
  for source in [tex, plain.as_str()] {
    let (log, xml) = convert_files_with(source, &files, Some("ar5iv.sty"));
    assert!(!log.contains("Fatal:"), "{log}");
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_element(
      &xml,
      "biblist",
      &[],
      "<biblist><bibitem key=\"a\" xml:id=\"bib.bibx1\"><tags><tag>[Smith et\u{a0}al. 2012]</tag>\
       <tag role=\"refnum\">Smith et\u{a0}al. 2012</tag></tags><bibblock> A. Smith. A Title. 2012.</bibblock></bibitem>\
       <bibitem key=\"b\" xml:id=\"bib.bibx2\"><tags><tag>[Doe 1999]</tag><tag role=\"refnum\">Doe 1999</tag></tags>\
       <bibblock> J. Doe. Another Title. 1999.</bibblock></bibitem></biblist>",
    );
    assert_element(
      &xml,
      "p",
      &[],
      r#"<p xml:id="p1.1">See <cite class="ltx_citemacro_cite">[<bibref bibrefs="a,b" separator="," yyseparator=","/>]</cite> and <cite class="ltx_citemacro_cite">[<bibref bibrefs="b" separator="," yyseparator=","/>]</cite>.</p>"#,
    );
  }
}
