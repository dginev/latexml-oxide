//! Red/green guards for batch 66f: regressions found by the cortex sandbox reruns of 2026-10-13 (runs 341/342).
//! Repro `tools/perfect_kernel/repros/loader/jobname_aux_exists_after_begin_document.tex`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// latex.ltx's `\document` (:9491-9494) opens the `.aux` under the job's name and writes `\relax` to it, so a class
/// `\enddocument` that `\input`s `\jobname.aux` (jpsj3.cls:90-96; witnesses 2605.24927, 2605.04825, 2605.28673,
/// 2605.20125 with bio.cls, cond-mat9607109 with lamuphys.sty, astro-ph9805185 with caosp.sty) finds it. The kernel's
/// format-load `\openout` ran before the job's name was known (a dump's build, a preload): `Error:missing_file`.
#[test]
fn jobname_aux_exists_after_begin_document() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/loader/jobname_aux_exists_after_begin_document.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("aux present."), "{xml}");
  assert!(xml.contains("Text after."), "{xml}");
}

/// `xml` from the first `open` to the end of its `close`, each line trimmed: a whole element, indentation-free.
fn element(xml: &str, open: &str, close: &str) -> String {
  let start = xml.find(open).unwrap_or_else(|| panic!("no {open}\n{xml}"));
  let end = xml[start..].find(close).map_or_else(
    || panic!("no {close} after {open}\n{xml}"),
    |e| start + e + close.len(),
  );
  xml[start..end]
    .lines()
    .map(str::trim)
    .collect::<Vec<_>>()
    .join("\n")
}

/// latex.ltx:10545-10554 expands `\mathversion`'s name: figureversions.sty:169-179 (XCharter, via fontaxes) passes
/// `\l_tmpa_str` from its `\boldmath` (witnesses 2605.11518, 2605.12374, 2605.22763).
#[test]
fn mathversion_expands_its_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/mathversion_expands_its_argument.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<XMTok font="bold italic" role="UNKNOWN">x</XMTok>"#),
    "{xml}"
  );
}

/// After `\appendix`, a `\section*` in a float body is a section of class `ltx_appendix`, as a numbered one is
/// (OD #189; witnesses 2605.02378, 2605.25534).
#[test]
fn unnumbered_appendix_section_in_float() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/unnumbered_appendix_section_in_float.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let figure = element(&xml, "<figure", "</figure>");
  assert!(
    figure.contains(r#"<section class="ltx_appendix""#),
    "{figure}"
  );
  assert!(figure.contains("Strategy A</title>"), "{figure}");
  assert!(!figure.contains("<appendix"), "{figure}");
}

/// authblk's `\affil` is robust (authblk.sty:148), so it survives titling's `\protected@xdef\theauthor` inside
/// `\author` (witness 2605.03796, whose body was lost).
#[test]
fn authblk_affil_in_author_under_titling() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/authblk_affil_in_author_under_titling.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    element(&xml, "<creator", "</creator>"),
    "<creator role=\"author\">\n<personname>Ann Author<sup xml:id=\"id1\">1</sup></personname>\n<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\">1</sup>Some University</contact>\n</creator>",
    "{xml}"
  );
  assert!(xml.contains("<p xml:id=\"p1.1\">Body.</p>"), "{xml}");
}

/// SNmult.cls (Springer Nature, v6.00) is svmult.cls renamed, so it gets the svmult binding: `\institute{… \at …}`
/// is an affiliation (witness 2605.13070).
#[test]
fn snmult_is_svmult() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/snmult_is_svmult.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Some University</contact>"
    ),
    "{xml}"
  );
}

/// url.sty:204-205 defines `\path` only if undefined: a document's own `\path` stays (witness 2605.13185).
#[test]
fn url_path_keeps_user_definition() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/url_path_keeps_user_definition.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"tex="\rho v\in W""#), "{xml}");
}

/// jmlr.cls:551-556 `\jmlrissue` and `\jmlrpages`, printed with the volume (witness 2605.20066).
#[test]
fn jmlr_issue_and_pages() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/jmlr_issue_and_pages.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for note in [
    r#"<note role="volume" xml:id="id1">1</note>"#,
    r#"<note role="issue" xml:id="id2">2</note>"#,
    "<note role=\"pages\" xml:id=\"id3\">1\u{2013}10</note>",
  ] {
    assert!(xml.contains(note), "{note}\n{xml}");
  }
}

/// IEEEtran's `\appendices` reaches the kernel's `\appendix` through a copy, so `\renewcommand{\appendix}{\appendices}`
/// runs once (witness 2605.23316: a PushbackLimit Fatal).
#[test]
fn ieeetran_appendix_let_to_appendices() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_appendix_let_to_appendices.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert!(
    xml.contains(r#"<title><tag close=" ">Appendix A</tag>Proofs</title>"#),
    "{xml}"
  );
}

/// nicematrix.sty:5605-5616 reads a `\CodeBefore` color's optional argument as keys: `opacity=` is the fill opacity
/// (here a mix with the white page), the rest the model (witness 2605.25123). Text tabulars paint no fills.
#[test]
fn nicematrix_color_opacity_key() {
  let (log, _) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/nicematrix_color_opacity_key.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{nicematrix}\n\\begin{document}\n$\\begin{pNiceMatrix}\n\\CodeBefore\n\
     \\columncolor[opacity=0.2]{blue}{2}\n\\columncolor{red}{1}\n\\Body\na & b \\\\\nc & d\n\\end{pNiceMatrix}$\n\
     \\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml.matches(r##"backgroundcolor="#CCCCFF""##).count(),
    2,
    "{xml}"
  );
  assert_eq!(
    xml.matches(r##"backgroundcolor="#FF0000""##).count(),
    2,
    "{xml}"
  );
}

/// braket's lowercase `\set` splits only at `|` (braket.sty:61-64): a `\|` is the operand's, even inside a
/// `\left(…\right)` hidden in macros (witness 2605.31379, "Extra \right").
#[test]
fn braket_set_double_bar_is_operand() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/braket_set_double_bar_is_operand.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"tex="\set{D\left(a\|b\right)}{M\in N}""#),
    "{xml}"
  );
  assert_eq!(xml.matches(r#"meaning="set""#).count(), 1, "{xml}");
}

/// natbib.sty:702-730 builds `\citealp`/`\citealt` and the capitalised forms over `\NAT@citetp`, never calling
/// `\citep`/`\citet`, so a document's `\renewcommand{\citep}{\oldcitep*}` does not reach them (witnesses 2606.16139,
/// 2606.30583: keys lost).
#[test]
fn natbib_citealp_after_citep_redefined() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/natbib_citealp_after_citep_redefined.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<cite class="ltx_citemacro_citep"><bibref bibrefs="a" separator=";" show="AuthorsPhrase1Year" yyseparator=",">"#
    ),
    "{xml}"
  );
}

/// calc reads a numeric term through a register assignment, whose `<optional equals>` takes a leading `=`
/// (calc.sty:110, tex.web §1224; witness 2606.05909).
#[test]
fn calc_optional_equals() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/calc_optional_equals.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("172.5pt"), "{xml}");
}

/// appendix.sty's internals that classes re-dressing `appendices` call (sn-jnl, wlscirep; witness 2606.09094): one
/// appendix, lettered.
#[test]
fn appendix_internals_resets_pp() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/appendix_internals_resets_pp.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<appendix ").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<title><tag close=" ">Appendix A</tag>Proofs</title>"#),
    "{xml}"
  );
}

/// amsart.cls's `\@classname` and `\subjclassname@<year>`, which classes built on amsart read (witness 2606.20943).
#[test]
fn amsart_classname_and_subjclass_editions() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/amsart_classname_and_subjclass_editions.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Class: amsart."), "{xml}");
}

/// amsart.cls:58-60 names the file that loaded amsart, the top of `\@currnamestack`: under a class that
/// `\LoadClass`es amsart, that class (quantumdust.cls, witness 2606.20943). Read in a file amsart loads, it named
/// `ams_core`.
#[test]
fn amsart_classname_is_the_loading_class() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/amsart_classname_is_the_loading_class.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("Class: amsloader."), "{xml}");
}

/// An `\if` opened inside an `\IEEEauthorblockA` argument and closed after it keeps its `\else`/`\fi` with it
/// (witness 2606.17718).
#[test]
fn ieeetran_author_conditional_from_block_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_author_conditional_from_block_argument.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The second, empty affiliation is a residual: the block's `\vspace{-5mm}` and the conditional's tail stay in
  // the `\author` argument after the block and are read as a further (empty) block line; pdflatex prints nothing.
  assert_eq!(
    element(&xml, "<creator", "</creator>"),
    "<creator role=\"author\">\n<personname>A. Author</personname>\n\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ <break/>\nE-mail: a@b.c</contact>\n\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"/>\n</creator>",
    "{xml}"
  );
  assert!(!xml.contains("x@y.z"), "{xml}");
}

/// `\hypertarget`'s text follows the anchor in the current list (hyperref.sty:4805-4811); in math the anchor is
/// empty and a brace-less call's one-token text is read in place (witness 2606.15438), after the mode test, so a
/// `\frac` there takes its own arguments, not the `\fi` (hyperref's own `#2` precedes its `\fi`). In text the
/// anchor holds its text.
#[test]
fn hypertarget_braceless_in_math() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/hypertarget_braceless_in_math.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"tex="\hypertarget{e1}{}\left(\frac{n}{p}\right)=1""#),
    "{xml}"
  );
  // The `[] *` is the anchor parsed as an operand, the defect RED `hypertarget_math_anchor_keeps_its_id` tracks: fixing
  // that RED changes this expectation.
  assert!(
    xml.contains(r#"tex="a\hypertarget{e2}{}\frac{n}{p}b" text="a * [] * (n / p) * b""#),
    "{xml}"
  );
  // At a math alignment row's start (`align`, `aligned`, `array`) the call takes the math branch too.
  assert!(
    xml.contains(r#"tex="\displaystyle\hypertarget{e3}{}\left(\frac{n}{p}\right)""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"{c}\hypertarget{e5}{}\frac{n}{p}\end{array}""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"See <anchor xml:id="t1">Target</anchor> here."#),
    "{xml}"
  );
}

/// braket's split never cuts inside a `\left…\right` pair (witness 2606.06185).
#[test]
fn braket_bar_inside_left_right() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/braket_bar_inside_left_right.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"text="D = quantum-operator-product@(phi, delimited-[]@(m cdot r - m * (absolute-value@(R)) ^ 2), psi)""#
    ),
    "{xml}"
  );
  // A `\right` whose `\left` is hidden in a macro does not take the count below zero.
  assert!(
    xml.contains(r#"text="inner-product@(a, delimited-[]@(conditional@(b, c)))""#),
    "{xml}"
  );
}
