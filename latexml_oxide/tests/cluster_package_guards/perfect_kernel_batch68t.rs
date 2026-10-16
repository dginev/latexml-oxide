//! Red/green guards for batch 68t: a refused printing `\title` called inside the document's `{titlepage}` prints there
//! (title study rule R2, 2026-10-16; OXIDIZED_DESIGN #629). Witnesses 0904.2708 (cernart.cls:217), gr-qc9404034 (the
//! document's own `\def\title`, gr-qc9404034.tex:96). Expected text from pdflatex. Round 2: the print, between
//! cover-title marks, goes with a cover that stops being one (unwound, demoted), text included; its argument's side
//! effects are the kernel's; a `\title` that stores and prints prints; the HTML shows the title once.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch64::post_with,
};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion (0 errors, 0 warnings), squashed.
fn clean(source: &str) -> String { squash(&clean_raw(source)) }

/// A clean conversion (0 errors, 0 warnings), as built.
fn clean_raw(source: &str) -> String {
  let (log, xml) = convert_with(source, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// How often the HTML page (`LaTeXML-html5.xsl`, no POST error) shows `title` in its body: the visible titles.
fn visible_in_html(source: &str, title: &str) -> usize {
  let html = post_with(&clean_raw(source), "resources/XSLT/LaTeXML-html5.xsl");
  let body = html.split_once("<body").map_or("", |(_, body)| body);
  assert!(!body.is_empty(), "{html}");
  assert!(!html.contains("Capture"), "{html}");
  body.matches(title).count()
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
  assert!(
    !xml.contains("_lx_cover") && !xml.contains("Capture"),
    "{xml}"
  );
  // the HTML shows the cover in place of the hidden document title: one visible title
  assert_eq!(
    visible_in_html(
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_refused_title_prints_on_the_cover.tex"
      ),
      "Static axisymmetric spacetimes"
    ),
    1
  );
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

/// Round 2 (review probe p13): `\maketitle` inside the cover unwinds it (`\unwind@titlepage`), so the document's
/// `ltx:title` is displayed: the class's print, between the cover-title marks, goes with the cover. pdflatex shows the
/// title once (its `\maketitle` errs "No \title given": this `\title` stores nothing).
#[test]
fn titlepage_unwound_cover_drops_the_class_title_print() {
  let source = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_unwound_cover_drops_the_class_title_print.tex"
  );
  let xml = clean(source);
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"> <title>Static axisymmetric spacetimes</title> <section inlist="toc" xml:id="S1">"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("Static axisymmetric spacetimes").count(),
    1,
    "{xml}"
  );
  assert!(
    !xml.contains("titlepage") && !xml.contains("Capture"),
    "{xml}"
  );
  assert_eq!(visible_in_html(source, "Static axisymmetric spacetimes"), 1);
}

/// Round 2 (review probe p9): a print that is bare text in the cover's open paragraph goes too when the stranded cover
/// is demoted (`demote_stranded_titlepage`): everything between the marks, text included. pdflatex prints the title
/// once, in "Cover Static axisymmetric spacetimes text."
#[test]
fn titlepage_demoted_cover_drops_the_class_title_text() {
  let source = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_demoted_cover_drops_the_class_title_text.tex"
  );
  let xml = clean(source);
  assert!(
    xml.contains(r#"<title>Static axisymmetric spacetimes</title> <para xml:id="p1"> <p xml:id="p1.1">Preprint 253.</p> </para> <para class="ltx_titlepage" xml:id="p2"> <p xml:id="p2.1">Cover text.</p> </para> <section"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("Static axisymmetric spacetimes").count(),
    1,
    "{xml}"
  );
  assert!(!xml.contains("Capture"), "{xml}");
  assert_eq!(visible_in_html(source, "Static axisymmetric spacetimes"), 1);
}

/// Round 2 (review probe p5): the print runs with the argument's `\label`, `\thanks`, `\footnote`, `\footnotemark` and
/// `\index` gobbled — the kernel's copy owns them: one `ltx:pubnote`, no label on `ltx:document`. pdflatex: the cover
/// "Static spacetimes" with its footnote, and "See ." for the `\ref`.
#[test]
fn titlepage_class_title_print_gobbles_label_and_thanks() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_class_title_print_gobbles_label_and_thanks.tex"
  ));
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"> <titlepage xml:id="id1"> <p align="center" xml:id="id1.1"><text fontsize="120%" xml:id="id1.1.1">Static spacetimes</text></p> <p xml:id="id1.2">Cover text.</p> </titlepage> <title>Static spacetimes</title> <pubnote name="Thanks: " role="thanks">Supported by X.</pubnote> <section"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<pubnote").count(), 1, "{xml}");
  assert_eq!(xml.matches("Supported by X.").count(), 1, "{xml}");
  assert!(!xml.contains("labels="), "{xml}");
}

/// Round 2 (review probe p2): a refused `\title` that both stores and prints (`\gdef\@title{#1}\centerline{#1}`) prints
/// on the cover, as pdflatex does; one frontmatter title.
#[test]
fn titlepage_storing_class_title_prints_on_the_cover() {
  let source = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlepage_storing_class_title_prints_on_the_cover.tex"
  );
  let xml = clean(source);
  assert!(
    xml.contains(r#"<titlepage xml:id="id1"> <p class="ltx_align_center" xml:id="id1.1"><text xml:id="id1.1.1">Static axisymmetric spacetimes</text></p> <p xml:id="id1.2">Cover text.</p> </titlepage> <title>Static axisymmetric spacetimes</title> <section"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<title>Static").count(), 1, "{xml}");
  assert_eq!(visible_in_html(source, "Static axisymmetric spacetimes"), 1);
}

/// Round 2: a printing `\title` that calls the saved kernel `\title` (`\let\oldtitle\title`) prints on the cover and
/// hands the title to the frontmatter once (`\lx@titlepage@kernel` skips the kernel's part). pdflatex: the centered title
/// on the cover.
#[test]
fn titlepage_class_title_calling_the_kernel_title_adds_it_once() {
  let source = r"\documentclass{article}
\let\oldtitle\title
\def\title#1{\oldtitle{#1}\centerline{#1}}
\begin{document}
\begin{titlepage}
\title{Static axisymmetric spacetimes}
Cover text.
\end{titlepage}
\section{Intro}
Text.
\end{document}
";
  let xml = clean(source);
  assert!(
    xml.contains(r#"<titlepage xml:id="id1"> <p class="ltx_align_center" xml:id="id1.1"><text xml:id="id1.1.1">Static axisymmetric spacetimes</text></p> <p xml:id="id1.2">Cover text.</p> </titlepage> <title>Static axisymmetric spacetimes</title> <section"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<title>Static").count(), 1, "{xml}");
  assert_eq!(visible_in_html(source, "Static axisymmetric spacetimes"), 1);
}
