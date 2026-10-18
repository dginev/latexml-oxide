//! Red/green guards for batch 74a, abstract content: a raw class's refused `{abstract}` begin code runs at the kernel
//! abstract's head for its effects (sect05.rs `replay_class_begin_code`, OXIDIZED_DESIGN #482; the Frontiers/bioinfo
//! template family: 1508.06756, 1406.7114, 1404.0027), and a table of contents inside an abstract goes after it
//! (sect04.rs `insert_toc`, `Document::float_out_of_frontmatter`, OXIDIZED_DESIGN_DIVERGENCES #753; 1011.3976,
//! 1602.08755, 2007.03709, 2411.02384). Each repro's pdflatex reference text is in its header.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A conversion of a `class_abstract` repro beside the minimal copy of its class: its log and squashed XML.
macro_rules! convert_beside {
  ($path:literal, $cls:literal) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_abstract/",
        $path
      )),
      &[(
        $cls,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_abstract/",
          $cls
        )),
      )],
      Some("ar5iv.sty"),
    );
    (log, squash(&xml))
  }};
}

/// A conversion of a `sectioning-frontmatter` repro, which must be clean (0 errors, 0 warnings, no Fatal): its XML,
/// squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(!log.lines().any(|l| l.starts_with("Fatal:")), "{log}");
    squash(&xml)
  }};
}

/// The TOC element `\tableofcontents` makes (article's select list).
const TOC: &str = r#"<TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"> <title>Contents</title> </TOC>"#;

/// frontiersFPHY.cls:516-524 `\newenvironment{abstract}{\begingroup\let\section\absection…}{\endgroup…}`: the refused
/// begin code's `\let` holds in the abstract, so its `\section{…}`s are run-in bold labels (were sections nested in
/// the abstract, 2 Errors `malformed:ltx:section`), the font it selects is not kept, and after the abstract `\section`
/// is the kernel's (numbered 1).
#[test]
fn class_abstract_begin_code_makes_section_a_run_in_label() {
  let (log, xml) = convert_beside!(
    "abstract_class_begin_code_runs_frontiers.tex",
    "absfront.cls"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1"><text font="bold" xml:id="abstract1.1.1">Motivation:</text> First part.</p> <p xml:id="abstract1.2"><text font="bold" xml:id="abstract1.2.1">Results:</text> Second part.</p> <p xml:id="abstract1.3"><text font="bold" fontsize="80%" xml:id="abstract1.3.1">Keywords: alpha – beta</text></p> </abstract>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<section ").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<title><tag close=" ">1</tag>Introduction</title>"#),
    "{xml}"
  );
}

/// bioinfo.cls:495-500's `\newenvironment{abstract}` (a class not built on article defines it; refused by the kernel's
/// lock, its code now kept, sect08.rs): the same run-in labels (1406.7114, 0901.4589, 2108.03515).
#[test]
fn class_newenvironment_abstract_begin_code_runs() {
  let (log, xml) = convert_beside!(
    "abstract_class_begin_code_runs_bioinfo.tex",
    "absbioinfo.cls"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1"><text font="bold" xml:id="abstract1.1.1">Motivation:</text> First part.</p> <p xml:id="abstract1.2"><text font="bold" xml:id="abstract1.2.1">Results:</text> Second part.</p> </abstract>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<section ").count(), 1, "{xml}");
}

/// A `\tableofcontents{}` ending a balanced abstract (LyX; 1011.3976, 1310.0349): after the abstract, a child of the
/// document (was inside the abstract's `ltx:p`, 1 Error).
#[test]
fn toc_at_the_end_of_a_balanced_abstract_follows_it() {
  let xml = clean!("toc_inside_balanced_abstract_at_its_end.tex");
  assert!(
    xml.contains(&format!(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">First text.</p> </abstract> {TOC} <section inlist="toc" xml:id="S1">"#
    )),
    "{xml}"
  );
}

/// …in its middle (1602.08755): the TOC after it, the text after the TOC the abstract's second paragraph.
#[test]
fn toc_mid_abstract_follows_it_and_the_text_stays() {
  let xml = clean!("toc_inside_balanced_abstract_mid_text.tex");
  assert!(
    xml.contains(&format!(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">First text.</p> <p xml:id="abstract1.2">Second text.</p> </abstract> {TOC} <section inlist="toc" xml:id="S1">"#
    )),
    "{xml}"
  );
}

/// After a brace-less `\abstract` the TOC (`\section*` in article.cls) ends it, as a `\section` does: the text after
/// it is the document's (2007.03709, 2202.05904, hep-lat9910044).
#[test]
fn toc_ends_an_unclosed_abstract() {
  let xml = clean!("toc_ends_an_unclosed_abstract.tex");
  assert!(
    xml.contains(&format!(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">First text.</p> </abstract> {TOC} <para xml:id="p1"> <p xml:id="p1.1">After the contents.</p> </para> <section inlist="toc" xml:id="S1">"#
    )),
    "{xml}"
  );
}

/// …and inside an unclosed `\abstract{` group (2411.02384, 2506.07991), the runaway group reported as TeX does
/// ("\end occurred inside a group"): one Warning, no Error (was 1 Error, `malformed:ltx:TOC`).
#[test]
fn toc_ends_an_unclosed_braced_abstract() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/toc_ends_an_unclosed_braced_abstract.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Warning:unexpected:\\lx@end@abstract"),
    "{log}"
  );
  let xml = squash(&xml);
  assert!(
    xml.contains(&format!(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">First text.</p> </abstract> {TOC} <section inlist="toc" xml:id="S1">"#
    )),
    "{xml}"
  );
}

/// `\listoffigures` and `\listoftables` in an abstract take the TOC's place, after it.
#[test]
fn lists_of_figures_and_tables_follow_the_abstract() {
  let xml = clean!("lists_of_figures_and_tables_inside_abstract.tex");
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">First text.</p> <p xml:id="abstract1.2">Second text.</p> </abstract> <TOC lists="lof" scope="global"> <title>List of Figures</title> </TOC> <TOC lists="lot" scope="global"> <title>List of Tables</title> </TOC> <section inlist="toc" xml:id="S1">"#
    ),
    "{xml}"
  );
}

/// Outside a frontmatter container the TOC stays where it stands: in a section after a paragraph it closes, and in a
/// `{quote}` the inline logical block of OD #271's bridge (1412.2039, math0603642), unchanged.
#[test]
fn toc_outside_the_frontmatter_is_unchanged() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n\\section{A}\nText \\tableofcontents\n\\begin{quote}\\tableofcontents\\end{quote}\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(&format!(
      r#"<para xml:id="S1.p1"> <p xml:id="S1.p1.1">Text</p> </para> {TOC} <para xml:id="S1.p2"> <quote xml:id="S1.p2.1"> <inline-logical-block xml:id="S1.p2.1.1"> {TOC} </inline-logical-block> </quote> </para>"#
    )),
    "{xml}"
  );
}
