//! Red/green guards for int26, from the silent-loss inventory (agent_reports/2026-10-18_silent_loss_inventory.md) and
//! the `undefined:\thetitle` cluster: a frontmatter body still open at `\end{document}` ends there and keeps its text
//! (sect02.rs `\lx@end@open@frontmatter`; was an empty `<document/>` with 0 errors), and a refused `\title` replays for
//! its side effects as `\author`'s does (sect05.rs `\title`, the support bindings' `\title`; OXIDIZED_DESIGN #482). Each
//! green asserts the three targets: the log's tally, the authored text, and the element that carries it.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A `sectioning-frontmatter` repro's log and squashed XML.
macro_rules! convert_repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert!(!log.lines().any(|l| l.starts_with("Fatal:")), "{log}");
    (log, squash(&xml))
  }};
}

/// The abstract both repros below keep, whole.
const TWO_PARAGRAPH_ABSTRACT: &str = r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract only text.</p> <p xml:id="abstract1.2">More abstract text.</p> </abstract>"#;

/// A brace-less `\abstract` no sectioning command ended: both paragraphs in the abstract, clean, as pdflatex prints them
/// (was `<document/>`, 2 warnings).
#[test]
fn braceless_abstract_unclosed_at_end_document_keeps_its_text() {
  let (log, xml) =
    convert_repro!("braceless_abstract_unclosed_at_end_document_keeps_the_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(TWO_PARAGRAPH_ABSTRACT), "{xml}");
}

/// An unbalanced `\abstract{`: the same abstract; its open group is the one warning, as pdflatex's "\end occurred inside
/// a group".
#[test]
fn braced_abstract_unclosed_at_end_document_keeps_its_text() {
  let (log, xml) =
    convert_repro!("braced_abstract_unclosed_at_end_document_keeps_the_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log
      .lines()
      .any(|l| l.starts_with("Warning:unexpected:\\lx@end@abstract")),
    "{log}"
  );
  assert!(xml.contains(TWO_PARAGRAPH_ABSTRACT), "{xml}");
}

/// An unclosed `\begin{abstract}`: LaTeX's error, raised once on purpose and pinned by count (pdflatex and Perl give 1),
/// and the text kept — the abstract's paragraph and the following section with its body (nested in the abstract as in
/// the PDF, until the ruling on a heading inside an abstract, SYNC 74a row (b)).
#[test]
fn environment_abstract_unclosed_at_end_document_errs_and_keeps_its_text() {
  let (log, xml) = convert_repro!("unclosed_abstract_at_end_document_keeps_the_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.lines().any(|l| l
      .starts_with("Error:unexpected:\\end{document} \\begin{abstract} ended by \\end{document}")),
    "{log}"
  );
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract only text.</p>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title><tag close=" ">1</tag>Body</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body text.</p> </para>"#), "{xml}");
}

/// The cvpr template's `\renewcommand{\title}[1]{\titleold{#1}\newcommand{\thetitle}{#1}}` in a package: the refused
/// `\title` replays, so `\maketitlesupplementary` prints the title (was `undefined:\thetitle`, 2308.03900 and 15 more).
#[test]
fn refused_package_title_defines_thetitle() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/title_replay_defines_thetitle.tex"
    ),
    &[(
      "ttl.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/ttl.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(
    xml.matches("<title>Universal Widgets</title>").count(),
    1,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text font="bold" fontsize="144%" xml:id="p2.1.1">Universal Widgets<break/><text font="medium" xml:id="p2.1.1.1">Supplementary Material</text></text>"#),
    "{xml}"
  );
}

/// The same idiom under acmart, whose `\title` is ams_support's: it replays there too (2506.01799).
#[test]
fn refused_document_title_defines_thetitle_under_acmart() {
  let (log, xml) = convert_repro!("class_title/title_replay_defines_thetitle_acmart.tex");
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(
    xml.matches("<title>Universal Widgets</title>").count(),
    1,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Body Universal Widgets.</p>"#),
    "{xml}"
  );
}
