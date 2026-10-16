//! Red/green guards for batch 68t: a refused printing `\title` called inside the document's `{titlepage}` prints there
//! (title study rule R2, 2026-10-16; OXIDIZED_DESIGN #629). Witnesses 0904.2708 (cernart.cls:217), gr-qc9404034 (the
//! document's own `\def\title`, gr-qc9404034.tex:96). Expected text from pdflatex.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion (0 errors, 0 warnings), squashed.
fn clean(source: &str) -> String {
  let (log, xml) = convert_with(source, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// The document's own printing `\title` called inside `{titlepage}` prints the title on the cover, as pdflatex does;
/// the document's `ltx:title` stays beside the titlepage (metadata: the XSLT shows the cover in its place), and the
/// title-ink dedup leaves the cover alone. The refused printing `\author` is not run: the author is the frontmatter's
/// `ltx:creator`, once.
#[test]
fn titlepage_refused_title_prints_on_the_cover() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_refused_title_prints_on_the_cover.tex"
  ));
  assert!(
    xml.contains(r#"<titlepage xml:id="id1"> <p xml:id="id1.1">TIT/HEP-253</p> <p align="center" xml:id="id1.2"><text fontsize="120%" xml:id="id1.2.1">Static axisymmetric spacetimes</text></p> <abstract inlist="toc" xml:id="abstract1"> <p xml:id="abstract1.1">The conditions are presented.</p> </abstract> </titlepage> <title>Static axisymmetric spacetimes</title> <creator role="author"> <personname>M. Tanimoto</personname> </creator> <section"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("Static axisymmetric spacetimes").count(),
    2,
    "{xml}"
  );
  assert_eq!(xml.matches("M. Tanimoto").count(), 1, "{xml}");
  assert!(!xml.contains("_lx_cover"), "{xml}");
}

/// Outside a titlepage the refused printing `\title` is not run: the frontmatter's title, placed first, is what the PDF
/// prints there (rule R1), once.
#[test]
fn refused_title_outside_titlepage_is_the_frontmatter_title() {
  let xml = clean(
    r"\documentclass{article}
\def\title#1{\begin{center}{\large #1}\end{center}}
\begin{document}
\title{Static axisymmetric spacetimes}
Text.
\end{document}
",
  );
  assert!(
    xml.contains(r#"<title>Static axisymmetric spacetimes</title> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("Static axisymmetric spacetimes").count(),
    1,
    "{xml}"
  );
}

/// A refused class `\title` that keeps its value (a frontmatter setter) prints nothing at its call, in a titlepage
/// too: pdflatex shows only what the titlepage typesets.
#[test]
fn refused_storing_title_in_titlepage_prints_nothing() {
  let xml = clean(
    r"\documentclass{article}
\makeatletter
\def\title#1{\gdef\my@title{#1}}
\makeatother
\begin{document}
\begin{titlepage}
Cover text.
\title{Static axisymmetric spacetimes}
\end{titlepage}
Text.
\end{document}
",
  );
  assert!(
    xml.contains(r#"<titlepage xml:id="id1"> <p xml:id="id1.1">Cover text.</p> </titlepage> <title>Static axisymmetric spacetimes</title>"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("Static axisymmetric spacetimes").count(),
    1,
    "{xml}"
  );
}

/// The same rule for a binding's own `\title` (revtex4_support under `\documentstyle{revtex}`, gr-qc9404034): the
/// document's printing `\title` inside `{titlepage}` prints on the cover.
#[test]
fn binding_title_in_titlepage_runs_the_refused_printing_title() {
  let xml = clean(
    r"\documentstyle[aps]{revtex}
\def\title#1{\begin{center}{\large #1}\end{center}}
\begin{document}
\begin{titlepage}
\title{Static axisymmetric spacetimes}
\end{titlepage}
Text.
\end{document}
",
  );
  assert!(
    xml.contains(r#"<titlepage xml:id="id1"> <p align="center" xml:id="id1.1"><text fontsize="120%" xml:id="id1.1.1">Static axisymmetric spacetimes</text></p> </titlepage> <title>Static axisymmetric spacetimes</title>"#),
    "{xml}"
  );
}

/// A titlepage that stands after body content is demoted to a paragraph (`demote_stranded_titlepage`), beside which
/// the document's `ltx:title` is displayed: the class's print on it would be a second copy, and goes. pdflatex prints
/// the title once (on the cover page, after "Preprint 253.").
#[test]
fn stranded_titlepage_drops_the_class_title_print() {
  let xml = clean(
    r"\documentclass{article}
\def\title#1{\begin{center}\bf #1\end{center}}
\begin{document}
Preprint 253.
\begin{titlepage}
\title{Static axisymmetric spacetimes}
Cover text.
\end{titlepage}
\section{Intro}
Text.
\end{document}
",
  );
  assert!(
    xml.contains(r#"<title>Static axisymmetric spacetimes</title> <para xml:id="p1"> <p xml:id="p1.1">Preprint 253.</p> </para> <para class="ltx_titlepage" xml:id="p2"> <p xml:id="p2.1">Cover text.</p> </para>"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("Static axisymmetric spacetimes").count(),
    1,
    "{xml}"
  );
  assert!(!xml.contains("_lx_cover"), "{xml}");
}
