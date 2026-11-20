//! Red/green guards for batch REGFIXA, three regressions of the v0.8.0 per-paper check (OK or Warning on 2026-11-05,
//! Error on the release candidate): amstex.sty's old-form matrices under `\documentstyle{amsart}`, fdsymbol's symbols,
//! mlbib's `\citeauthoryear` labels under the locked `\cite`. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// `\documentstyle{amsart}` is compatibility mode, where amsart.cls:353-362 loads amstex.sty, whose `\pmatrix`,
/// `\matrix`, `\cases` (amstex.sty:877-928) have no `\matrix@check`: no "Old form" error, the rows kept (math0101059,
/// hep-th9309006, math0003076). `\documentclass{amsart}` loads amsmath.sty, and the same `\pmatrix … \endpmatrix` is
/// its "Old form" error, as in pdflatex. Distinct from `perfect_kernel_batch77q`'s guards (Plain TeX's `\pmatrix{…}`
/// where amsmath is or is not loaded). Repro amstex/amstex_sty_old_form_matrices.
#[test]
fn amstex_sty_old_form_matrices_are_its_own() {
  let (log, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/amstex/amstex_sty_old_form_matrices.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  for text in [
    "text=\"matrix@(Array[[0, - 1], [1, 0]])\"",
    "text=\"matrix@(Array[[a, b], [c, d]])\"",
    "text=\"absolute-value@(x) = cases@(x, x &gt; 0, - x, x &lt;= 0)\"",
  ] {
    assert!(xml.contains(text), "{text}\n{xml}");
  }
  let (log, _) = convert_with(
    r"\documentclass{amsart}
\begin{document}
Before $\pmatrix 0 & -1 \\ 1 & 0 \endpmatrix$ after.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (3, 0), "{log}");
  assert!(
    log.contains("Package amsmath Error: Old form `\\pmatrix' should be \\begin{pmatrix}"),
    "{log}"
  );
}

/// fdsymbol.sty's own symbols, each at the code point of its font slot's glyph with its math class's role: the suits of
/// the author marks (2112.00443, 2010.03824, 2312.04649), a relation, an arrow, a big operator, an integral, the
/// semantic brackets, and `\dotcong` (fdsymbol.sty:93); `\dotplus` is amssymb's, which the binding loads. Repro
/// package-bindings/fdsymbol_symbols_are_unicode.
#[test]
fn fdsymbol_symbols_are_unicode() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/fdsymbol_symbols_are_unicode.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  for token in [
    r#"<XMTok fontsize="70%" name="vardiamondsuit" role="UNKNOWN">♦</XMTok>"#,
    r#"<XMTok fontsize="70%" name="varheartsuit" role="UNKNOWN">♥</XMTok>"#,
    r#"<XMTok name="coloneqq" role="RELOP">≔</XMTok>"#,
    r#"<XMTok name="dotplus" role="ADDOP">∔</XMTok>"#,
    r#"<XMTok name="mapsfrom" role="ARROW">↤</XMTok>"#,
    r#"<XMTok mathstyle="text" name="bigtimes" role="SUMOP" scriptpos="post">⨉</XMTok>"#,
    r#"<XMTok mathstyle="text" name="fint" role="INTOP">⨏</XMTok>"#,
    r#"<XMTok name="lsem" role="OPEN">⟦</XMTok>"#,
    r#"<XMTok name="rsem" role="CLOSE">⟧</XMTok>"#,
    r#"<XMTok name="dot" role="OVERACCENT">˙</XMTok>"#,
    r#"<XMTok meaning="approximately-equals" name="cong" role="RELOP">≅</XMTok>"#,
  ] {
    assert!(xml.contains(token), "{token}\n{xml}");
  }
}

/// mlbib.sty's `\cite` (cond-mat0011295) is `\@ifstar{\citeasnoun}{\new@cite}`, and its `\new@cite` is the only
/// definer of the `\citeauthoryear` its `.bbl` labels hold; the lock refuses that `\cite`, and the first kernel `\cite`
/// runs the copy's `\new@cite` (sect11.rs `label_macros_from_refused_cite`). Distinct from
/// `perfect_kernel_batchnext397::astroncite_label_under_the_lock`: the definer is not the branch's first `\def`, and the
/// first citation is starred. Repro index-bib/mlbib.
#[test]
fn mlbib_citeauthoryear_label_under_the_lock() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/mlbib/mlbib_citeauthoryear_label_under_the_lock.tex"
    ),
    &[(
      "mlbib.sty",
      include_str!("../../../tools/perfect_kernel/repros/index-bib/mlbib/mlbib.sty"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "bibitem",
    &[],
    "<bibitem key=\"Andersen96\" xml:id=\"bib.bibx1\"><tags><tag>[Andersen 1996]</tag>\
     <tag role=\"refnum\">Andersen 1996</tag></tags><bibblock> Andersen, T.: Return Volatility, 1996</bibblock></bibitem>",
  );
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">As shown by <cite class="ltx_citemacro_cite">[<bibref bibrefs="Andersen96" separator="," yyseparator=","/>]</cite> and <cite class="ltx_citemacro_cite">[<bibref bibrefs="Andersen96" separator="," yyseparator=","/>]</cite>.</p>"#,
  );
}
