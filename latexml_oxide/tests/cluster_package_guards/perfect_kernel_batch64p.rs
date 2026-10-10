//! Red/green guards for perfect-kernel batch 64p: run 336's loops (PushbackLimit) — a class's bibliography standing in
//! the kernel's, a tag reaching `\theequation` through another macro, a `\let` of a robust font switch, and a
//! document's own `\mathaccentV` accent.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// The XML of `tex` converted under ar5iv with `files` beside it, which must give no error and no warning.
fn convert_clean_files(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// 64p: a package's (a class's) `\renewenvironment{thebibliography}` in its end-of-package hook, which a binding's
/// load runs unlocked, is refused as Perl refuses it (the lock read as a value, latex_constructs.pool.ltxml:2796-2803):
/// wiley2sp's w2sp-pss.clo:444 and 131 more of run 336's papers reopened the class's list per `\bibitem`. Repro
/// index-bib/class_renews_bibliography_in_a_hook.
#[test]
fn class_renews_bibliography_in_a_hook() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/class_renews_bibliography_in_a_hook.tex"
  );
  let xml = convert_clean_files(tex, &[(
    "bibhook.sty",
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bibhook.sty"),
  )]);
  // (the class's `\section*{\refname}` is the bibliography's title, no section of its own)
  assert_eq!(xml.matches("<section").count(), 0, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, Title A.</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> B. Author, Title B.</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64p: a class's own list `\let` over the kernel's `{thebibliography}` (ajour, cjour, IEEE-Con-Sys-mag): the first
/// `\bibitem` ends that list and opens the kernel's bibliography, rather than reopening the class's per item. Repro
/// index-bib/class_lets_its_own_bibliography.
#[test]
fn class_lets_its_own_bibliography() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/class_lets_its_own_bibliography.tex"
  );
  let xml = convert_clean_files(tex, &[(
    "biblet.cls",
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblet.cls"),
  )]);
  assert_eq!(xml.matches("<section").count(), 0, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, Title A.</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> B. Author, Title B.</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64p: `\tag{\thesubeqn}` with `\thesubeqn` = `\theequation\alph{subeqn}` (1208.5957, 1209.0051): the tag text is
/// typeset with `\theequation` the counter's. Repro macro-state/amsmath_tag_reaches_theequation_indirectly.
#[test]
fn amsmath_tag_reaches_theequation_indirectly() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/amsmath_tag_reaches_theequation_indirectly.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "equationgroup",
    &[],
    r#"<equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.Ex1"><tags><tag>(0a)</tag><tag role="refnum">0a</tag></tags><MathFork><Math tex="\displaystyle a=b" text="a = b" xml:id="S0.Ex1.m3"><XMath xml:id="S0.Ex1.m3.1"><XMApp xml:id="S0.Ex1.m3.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math><MathBranch><td align="right" xml:id="S0.Ex1.1"><Math mode="inline" tex="\displaystyle a" text="a" xml:id="S0.Ex1.m1"><XMath xml:id="S0.Ex1.m1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex1.2"><Math mode="inline" tex="\displaystyle=b" text="absent = b" xml:id="S0.Ex1.m2"><XMath xml:id="S0.Ex1.m2.1"><XMApp xml:id="S0.Ex1.m2.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation><equation xml:id="S0.Ex2"><tags><tag>(0b)</tag><tag role="refnum">0b</tag></tags><MathFork><Math tex="\displaystyle c=d" text="c = d" xml:id="S0.Ex2.m3"><XMath xml:id="S0.Ex2.m3.1"><XMApp xml:id="S0.Ex2.m3.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math><MathBranch><td align="right" xml:id="S0.Ex2.1"><Math mode="inline" tex="\displaystyle c" text="c" xml:id="S0.Ex2.m1"><XMath xml:id="S0.Ex2.m1.1"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math></td><td align="left" xml:id="S0.Ex2.2"><Math mode="inline" tex="\displaystyle=d" text="absent = d" xml:id="S0.Ex2.m2"><XMath xml:id="S0.Ex2.m2.1"><XMApp xml:id="S0.Ex2.m2.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup>"#,
  );
}

/// 64p: mathfixs' autobold `\let\bfseries=\mafx@bfseries` (2205.10090, 2211.07279) leaves `\bfseries<space>` alone.
/// Repro fonts-nfss/mathfixs_autobold_let_of_robust_switch.
#[test]
fn mathfixs_autobold_let_of_robust_switch() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/mathfixs_autobold_let_of_robust_switch.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p xml:id="p1.1">Plain <text font="bold" xml:id="p1.1.1">Bold <Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></text> text.</p>"#,
  );
}

/// 64p: a document's `\bar` made of `\mathaccentV{bar}<family><slot>` (1503.00176, 1710.11113) is amsmath's
/// `\mathaccent` of that slot, not a call of `\bar` again. Repro math-parse/amsmath_mathaccentV_document_accent.
#[test]
fn amsmath_mathaccent_v_document_accent() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/amsmath_mathaccentV_document_accent.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="less-than" role="RELOP">&lt;</XMTok><XMApp xml:id="p1.m1.1.1.2"><XMTok name="bar" role="OVERACCENT" stretchy="false">¯</XMTok><XMTok font="italic" role="UNKNOWN">p</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.3"><XMTok meaning="divide" role="MULOP">/</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMath>"#,
  );
}

/// 64p: a class's own `{references}` list with `\bibitem`s (ajour.cls:2395-2427, cjour.cls; astro-ph0002202,
/// gr-qc0207046): the first `\bibitem` opens the kernel's bibliography, not the class's `\let` one, inside the list,
/// which its own end closes. Repro index-bib/class_references_list_with_bibitems.
#[test]
fn class_references_list_with_bibitems() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/class_references_list_with_bibitems.tex"
  );
  let xml = convert_clean_files(tex, &[(
    "bibrefs.cls",
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bibrefs.cls"),
  )]);
  assert_eq!(xml.matches("<section").count(), 0, "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, Title A.</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> B. Author, Title B.</bibblock></bibitem></biblist></bibliography>"#,
  );
}

/// 64p: `\DeclareCommandCopy` onto an existing robust command copies the body still (a `\let` deep copy skipped only
/// when the body calls the copy itself): `\DeclareRobustCommand\bfseries{\mytt\itshape}` after
/// `\DeclareCommandCopy\mytt\bfseries` is the old bold, italic. Repro macro-state/declarecommandcopy_onto_robust_command.
#[test]
fn declarecommandcopy_onto_robust_command() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/declarecommandcopy_onto_robust_command.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"S1.p1\""],
    r#"<para xml:id="S1.p1"><p xml:id="S1.p1.1">See <ref labelref="LABEL:s"/> and <ref labelref="LABEL:s"/>. <text font="bold italic" xml:id="S1.p1.1.1">bold</text></p></para>"#,
  );
}

/// 64p: the class's `{references}` end closes the kernel's bibliography before its own `\endlist`, so what follows
/// `\end{references}` follows the bibliography, as pdflatex prints it (not in the last entry's block). Repro
/// index-bib/class_references_list_followed_by_text.
#[test]
fn class_references_list_followed_by_text() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/class_references_list_followed_by_text.tex"
  );
  let xml = convert_clean_files(tex, &[(
    "bibrefs.cls",
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bibrefs.cls"),
  )]);
  assert_eq!(xml.matches("<bibliography ").count(), 1, "{xml}");
  assert_eq!(xml.matches("<bibitem ").count(), 2, "{xml}");
  let end = xml.find("</bibliography>").expect("a bibliography");
  assert_element(
    &xml[end..],
    "figure",
    &[],
    r#"<figure inlist="lof" placement="h" xml:id="Sx1.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p align="center" xml:id="Sx1.F1.1">A figure.</p><toccaption class="ltx_centering"><tag close=" ">1</tag>After.</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>After.</caption></figure>"#,
  );
  assert_element(
    &xml[end..],
    "para",
    &[],
    r#"<para xml:id="p2"><p xml:id="p2.1">After the references, more text.</p></para>"#,
  );
}

/// 64p: a bare `\bibitem` in an environment with no `\end<env>` (`{small}`, nested in `{center}`; valid LaTeX): the
/// saved end is `\relax`, as `\csname endsmall\endcsname` is in TeX, the environment's end closes the bibliography, and
/// what follows the environments follows it (64p review rounds 3-4).
#[test]
fn bibitem_in_an_environment_without_an_end_macro() {
  let xml = convert_clean(
    r"\documentclass{article}
\begin{document}
Body text.
\begin{center}
\begin{small}
\bibitem[a]{a} Alpha, A. 2001.
\end{small}
\end{center}
After nested.
\end{document}",
  );
  assert_eq!(xml.matches("<bibitem ").count(), 1, "{xml}");
  let end = xml.find("</bibliography>").expect("a bibliography");
  assert_element(
    &xml[end..],
    "para",
    &[],
    r#"<para xml:id="p2"><p xml:id="p2.1">After nested.</p></para>"#,
  );
}
