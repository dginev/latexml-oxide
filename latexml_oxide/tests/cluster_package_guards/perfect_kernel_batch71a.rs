//! Red/green guards for batch 71a: binding shapes from the rc_struct5 study (agent_reports/2026-10-17_rc_struct5.md).
//! B1 amsrefs `{biblist}` without `{bibdiv}`; T1 mn2e defines no `\ion`; P1 appolb.cls; U1 tkz-euclide's old
//! `\usetkzobj`; N1 siunitx decimal uncertainty; L1 txfonts `\lambdabar`; the small ones: forest's and todonotes' TikZ
//! libraries, pgfmath `Mod`, amsmath/amsfonts macro accents, breqn's `\sb`/`\sp`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

fn fatal_count(log: &str) -> usize { log.lines().filter(|l| l.starts_with("Fatal:")).count() }

/// amsrefs.sty:532-538: `{biblist}` stands alone, so an author's `\section*{References}\begin{biblist}` is the
/// document's bibliography, headed as the PDF heads it (1002.1686, 1112.3058, 2111.14165).
#[test]
fn amsrefs_biblist_without_bibdiv() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/amsrefs_biblist_without_bibdiv.tex"
  ));
  assert_eq!(
    (error_count(&log), warning_count(&log), fatal_count(&log)),
    (0, 0, 0),
    "{log}"
  );
  assert!(!xml.contains("<section"), "{xml}");
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography citestyle="numbers" inlist="toc" xml:id="bib"><title>References</title><biblist><bibentry key="A" type="article" xml:id="bib.bib1"><bib-name role="author"><surname>Doe</surname><givenname>J.</givenname></bib-name><bib-title>T</bib-title><bib-related role="host" type="journal"><bib-title>J</bib-title></bib-related><bib-date role="publication">2000</bib-date><bib-data role="self" type="BibTeX">@article{A, author = {Doe, J.}, title = {T}, journal = {J}, year = {2000}}</bib-data></bibentry></biblist></bibliography>"#,
  );
}

/// mn2e.cls defines no `\ion`, so the author's `\newcommand{\ion}` is the document's (0908.0351, astro-ph0501147).
#[test]
fn mn2e_author_ion_wins() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/mn2e_author_ion_wins.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">the prominent He\u{2009}<text font=\"smallcaps\" xml:id=\"p1.1.1\">I</text> lines.</p>",
  );
}

/// mnras.cls:488 keeps its `\ion`, on amsmath's `\text`.
#[test]
fn mnras_keeps_its_ion() {
  let (log, xml) = convert_counted(
    "\\documentclass{mnras}\n\\usepackage{amsmath}\n\\begin{document}\nthe \\ion{He}{II} line\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">the <text xml:id=\"p1.1.1\">He\u{2009}<text font=\"smallcaps\" xml:id=\"p1.1.1.1\">ii</text></text> line</p>",
  );
}

/// appolb.cls:322 `\PACS` is the classification after the abstract (0712.3606, 1007.2277).
#[test]
fn appolb_pacs() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/appolb_pacs.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "classification",
    &[],
    r#"<classification name="PACS numbers:" scheme="pacs">27.75.-q</classification>"#,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\"><personname>A</personname><contact name=\"Address:\u{a0}\" role=\"address\">B</contact></creator>",
  );
}

/// appolb.cls:299, :304 running heads and :1019-1033 abbreviations (hep-ph0403117).
#[test]
fn appolb_running_heads_and_abbreviations() {
  let (log, xml) = convert_counted(
    "\\documentclass{appolb}\n\\begin{document}\n\\title{T}\\author{A}\\maketitle\n\\headtitle{Short T}\\headauthor{A. B.}\nSee \\ie, this and \\etal\\ here.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<note role="runningtitle""#) && xml.contains(">Short T</note>"),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<note role="runningauthor""#) && xml.contains(">A. B.</note>"),
    "{xml}"
  );
  assert!(
    xml.contains(r#"font="italic""#) && xml.contains(">i.e.</text>, this"),
    "{xml}"
  );
}

/// tkz-euclide's pre-v3 `\usetkzobj{all}` loads nothing more: v3+ loads every object (1506.03598, 2005.14485).
#[test]
fn tkz_euclide_old_usetkzobj() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/tkz_euclide_old_usetkzobj.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<picture"), "{xml}");
}

/// siunitx.sty:700-848: a decimal marker in the bracketed uncertainty sets the uncertainty in the number's last places
/// (2311.18489, 2504.14399).
#[test]
fn siunitx_uncertainty_with_decimal_marker() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/siunitx_uncertainty_with_decimal_marker.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for tex in [
    r#"tex="4.02(52)\text{\,}\mathrm{m}""#,
    r#"tex="1.08(3)""#,
    r#"tex="11.0(17)\text{\,}\mathrm{V}""#,
  ] {
    assert!(xml.contains(tex), "{tex}: {xml}");
  }
  assert_element(
    &xml,
    "XMDual",
    &[r#"xml:id="p1.m1.2.4""#],
    r#"<XMDual xml:id="p1.m1.2.4"><XMRef idref="p1.m1.3" xml:id="p1.m1.2.4.1"/><XMWrap xml:id="p1.m1.2.4.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="52" role="NUMBER" xml:id="p1.m1.3">52</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual>"#,
  );
  // the number's places padded to the uncertainty's; more integer digits than the number's is no match (siunitx.sty:758)
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{siunitx}\n\\begin{document}\n\\num{4.0(0.52)} \\num{4(0.5)}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(
    xml.contains(r#"tex="4.00(52)""#) && xml.contains(r#"tex="4.0(5)""#),
    "{xml}"
  );
  let (log, _) = convert_counted(
    "\\documentclass{article}\n\\usepackage{siunitx}\n\\begin{document}\n\\num{1.5(12.3)}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 1, "{log}");
}

/// txfonts.sty:727-728 `\lambdaslash`, `\lambdabar` (0801.4523, 2104.03192).
#[test]
fn txfonts_lambdabar() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/txfonts_lambdabar.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<XMTok font="italic" name="lambdabar" role="UNKNOWN">ƛ</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="italic" name="lambdaslash" role="UNKNOWN">ƛ</XMTok>"#),
    "{xml}"
  );
}

/// forest.sty:50-53, :129: its TikZ libraries and its key family (2012.09112, 1910.06571, 2003.06880).
#[test]
fn forest_loads_its_tikz_libraries() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/forest_loads_its_tikz_libraries.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<picture"), "{xml}");
}

/// todonotes.sty:32 loads positioning (1807.00512, 2107.03711).
#[test]
fn todonotes_loads_positioning() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/stub-dependencies/todonotes_loads_positioning.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<picture"), "{xml}");
}

/// pgfmathfunctions.basic.code.tex:263-297: `mod` truncates, `Mod` adds the divisor to a negative `mod` only, so a
/// multiple of the divisor is 0 (1811.01404).
#[test]
fn pgfmath_mod_of_a_multiple_is_zero() {
  let (log, _) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/pgfmath_mod_of_a_multiple_is_zero.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let values: Vec<&str> = log
    .split("MOD(")
    .skip(1)
    .map(|s| s.split_whitespace().next().unwrap_or(""))
    .collect();
  assert_eq!(
    values,
    ["-10)=0", "-5)=0", "-4)=1", "-1)=4", "0)=0", "4)=4", "5)=0"],
    "{log}"
  );
  let (log, _) = convert_counted(
    "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\pgfmathparse{mod(-7,5)}\\typeout{TRUNC=\\pgfmathresult}\\pgfmathmod{-7}{5}\\typeout{MACRO=\\pgfmathresult}\\pgfmathMod{-7}{5}\\typeout{FLOOR=\\pgfmathresult}\n\\end{document}\n",
  );
  assert!(
    log.contains("TRUNC=-2.0") && log.contains("MACRO=-2.0") && log.contains("FLOOR=3.0"),
    "{log}"
  );
}

/// amsmath.sty:754-797: an accent's accentee is an undelimited argument, so a parameterless macro after it is accented
/// whole (0801.1527); a command taking arguments keeps the item reading.
#[test]
fn amsmath_accent_macro_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/amsmath_accent_macro_argument.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"tex="\tilde{m_{\gamma^{\prime}}}_{a}^{2}""#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"tex="\hat{m_{\gamma^{\prime}}}_{b}""#),
    "{xml}"
  );
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n$\\hat x$ $\\vec\\nabla$ $\\dot\\gamma_1$ $\\hat\\mathbf{x}$\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  for tex in [
    r#"tex="\hat{x}""#,
    r#"tex="\vec{\nabla}""#,
    r#"tex="\dot{\gamma}_{1}""#,
    r#"tex="\hat{\mathbf{x}}""#,
  ] {
    assert!(xml.contains(tex), "{tex}: {xml}");
  }
}

/// amsfonts.sty:78-87 `\widehat#1` (2609.01909).
#[test]
fn amsfonts_widehat_macro_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/ams-internals/amsfonts_widehat_macro_argument.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"tex="\widehat{C_{\mathrm{BA}}}_{\rm CF}=1""#),
    "{xml}"
  );
}

/// mathstyle.sty:124-127 (breqn): `\sb`/`\sp` read their script as an argument (2509.22335).
#[test]
fn breqn_script_macro_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/breqn_script_macro_argument.tex"
  ));
  // the breqn stub's Warning
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(xml.contains(r#"tex="\Delta_{\tau}\doteq 1""#), "{xml}");
  // a script that is a command reading an argument is read as before (pdflatex: "Argument of \text@ has an extra }";
  // OXIDIZED_DESIGN_DIVERGENCES #721)
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{breqn}\n\\begin{document}\n$x_\\text{abc}$ and $y^\\mathrm{T}$\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    xml.contains(r#"tex="x_{\text{abc}}""#) && xml.contains(r#"tex="y^{\mathrm{T}}""#),
    "{xml}"
  );
}
