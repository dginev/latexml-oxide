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
