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
/// its "Old form" error, as in pdflatex, and so does `\usepackage{amstex}` (amstex.sty:23-30 inputs amsmath.sty outside
/// compatibility mode), where `\documentstyle[amstex]{article}` does not. Distinct from `perfect_kernel_batch77q`'s guards
/// (Plain TeX's `\pmatrix{…}` where amsmath is or is not loaded). Repros amstex/amstex_sty_old_form_matrices,
/// amstex/amstex_package_in_latex2e_is_amsmath.
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
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/amstex/amstex_package_in_latex2e_is_amsmath.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (3, 0), "{log}");
  assert!(
    log.contains("Package amsmath Error: Old form `\\pmatrix' should be \\begin{pmatrix}"),
    "{log}"
  );
  assert!(xml.contains("Second paragraph survives."), "{xml}");
  let (log, xml) = convert_with(
    r"\documentstyle[amstex]{article}
\begin{document}
Before $\pmatrix 0 & -1 \\ 1 & 0 \endpmatrix$ after.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert!(
    xml.contains("text=\"matrix@(Array[[0, - 1], [1, 0]])\""),
    "{xml}"
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

/// fdsymbol.sty declares the AMS symbols and its `\mathbb`/`\mathfrak` itself and loads neither amssymb nor amsfonts
/// (fdsymbol.sty:194-195): `\@ifpackageloaded{amssymb}` is false, a user's `\newcommand{\blacksquare}` (an amssymb name
/// fdsymbol never declares) is kept, as is `\Bbb`; `\nsubset` is fdsymbol's own (fdsymbol.sty:772). The amssymb names
/// fdsymbol lacks stay undefined, an Error each as in pdflatex (`\mho` is the LaTeXML kernel's, math_common.pool:76).
/// Distinct from `fdsymbol_symbols_are_unicode` (what the binding defines; this: what it must not). Repro
/// package-bindings/fdsymbol_declares_no_amssymb.
#[test]
fn fdsymbol_declares_no_amssymb() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/fdsymbol_declares_no_amssymb.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert!(
    xml.contains("AMSSYMB-NOT-LOADED. Proof done <rule "),
    "{xml}"
  );
  assert!(!xml.contains('\u{25A0}'), "{xml}");
  for token in [
    r#"<XMTok font="blackboard" role="UNKNOWN">R</XMTok>"#,
    r#"<XMTok name="nsubset" role="RELOP">⊄</XMTok>"#,
  ] {
    assert!(xml.contains(token), "{token}\n{xml}");
  }
  let (log, _) = convert_with(
    r"\documentclass{article}
\usepackage{fdsymbol}
\begin{document}
$\varkappa \digamma \backepsilon \diagup \eth$ and $\lsem x \rsem$
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (5, 0), "{log}");
  for cs in ["varkappa", "digamma", "backepsilon", "diagup", "eth"] {
    assert!(
      log.contains(&format!("Error:undefined:\\{cs} ")),
      "{cs}\n{log}"
    );
  }
}

/// mlbib.sty's `\cite` (cond-mat0011295) is `\@ifstar{\citeasnoun}{\new@cite}`, and its `\new@cite` is the only
/// definer of the `\citeauthoryear` its `.bbl` labels hold; the lock refuses that `\cite`, and each kernel `\cite`
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

/// A refused `\cite` whose unstarred branch steps a counter and typesets before `\@internalcite` is not run: TeX runs it
/// at every citation (pdflatex "Count 3", three "see"), and the kernel `\cite` the lock keeps runs it at none, its label
/// macro undefined as in MAIN935, never once (sect11.rs `only_definitions`). Distinct from the two label guards above,
/// where the branch only `\def`s. Repro index-bib/sidecite.
#[test]
fn cite_branch_with_side_effects_is_not_run() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/sidecite/cite_branch_with_side_effects_is_not_run.tex"
    ),
    &[(
      "sidecite.sty",
      include_str!("../../../tools/perfect_kernel/repros/index-bib/sidecite/sidecite.sty"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:\\citelabel "), "{log}");
  let cite = r#"<cite class="ltx_citemacro_cite">[<bibref bibrefs="k" separator="," yyseparator=","/>]</cite>"#;
  assert_element(
    &xml,
    "p",
    &[],
    &format!(r#"<p xml:id="p1.1">A {cite} B {cite} C {cite}. Count 0.</p>"#),
  );
}
