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

/// An unclosed `\begin{abstract}`: its `\section` closes it (user ruling 2026-10-18), so the section and its text follow
/// the abstract; the `\end{abstract}` it still owes is LaTeX's error at `\end{document}`, raised once on purpose and
/// pinned by count (pdflatex and Perl give 1).
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
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract only text.</p> </abstract>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title><tag close=" ">1</tag>Body</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body text.</p> </para>"#), "{xml}");
}

/// A `\section*` inside a balanced abstract closes it (user ruling 2026-10-18; was kept nested, schema-invalid; Perl
/// errs): the heading and its text follow the abstract, and the `\end{abstract}` after it is a no-op. Clean, as pdflatex.
#[test]
fn section_inside_balanced_abstract_closes_it() {
  let (log, xml) = convert_repro!("abstract_section_closes_the_abstract.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract text.</p> </abstract>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title>Keywords</title>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="Sx1.p1.1">alpha, beta</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title><tag close=" ">1</tag>Introduction</title>"#),
    "{xml}"
  );
}

/// Keywords left open inside an abstract left open: both bodies end at `\end{document}` and keep their text; LaTeX's
/// one error names the innermost open environment (`\@checkend` against `\@currenvir`, as pdflatex), raised on purpose
/// and pinned by count (int26 review p8; was an empty `<document/>`).
#[test]
fn keywords_unclosed_inside_unclosed_abstract_keep_their_text() {
  let (log, xml) = convert_repro!("keywords_unclosed_inside_unclosed_abstract.tex");
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.lines().any(|l| l.starts_with(
      "Error:unexpected:\\end{document} \\begin{IEEEkeywords} ended by \\end{document}"
    )),
    "{log}"
  );
  for text in ["Abstract text here.", "alpha, beta", "Body text here."] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<keywords name="Index Terms: ">"#), "{xml}");
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
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
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

/// A package's refused `\title` that typesets, inside `{titlepage}`: it runs once on the cover, not again as the kernel
/// part's replay (int26 review p10): its counter counts 1, as in pdflatex.
#[test]
fn titlepage_class_title_runs_once() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/title_in_titlepage_runs_once.tex"
    ),
    &[(
      "tp.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/tp.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(xml.contains("Count is 1."), "{xml}");
  assert_eq!(xml.matches("Cover Title").count(), 2, "{xml}"); // the cover print and the document's ltx:title
}

/// physics `\exp[…]` whose optional argument holds a matrix `&`, inside an `align`: read align-safe as ltcmd's `o`
/// argument, so both rows stand and the text after the display is kept (2606.03447; was 11 Errors, Perl the same).
#[test]
fn physics_optional_argument_amp_in_align_keeps_the_rows() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_optional_argument_amp_in_align.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(
    xml
      .matches(r#"<equationgroup class="ltx_eqn_align""#)
      .count(),
    1,
    "{xml}"
  );
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  assert!(xml.contains("<XMArray"), "{xml}");
  assert!(xml.contains("TAILTEXT survives."), "{xml}");
}

/// An abstract whose first line is a heading (scrartcl `\begin{abstract}\section*{Abstract:} …`, 1507.08575): the
/// heading closes it before any text, the text follows with the heading in the body, and the body that typesets
/// nothing adds no `ltx:abstract` (no empty element; was three sections nested in the abstract, 2 Errors).
#[test]
fn abstract_opening_with_a_section_leaves_no_empty_abstract() {
  let (log, xml) = convert_repro!("abstract_opening_with_a_section_leaves_no_empty_abstract.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<abstract"), "{xml}");
  assert!(xml.contains(r#"<title>Abstract:</title>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="Sx1.p1.1">Structured abstract text.</p>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title>Availability:</title>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="Sx2.p1.1">Code is online.</p>"#),
    "{xml}"
  );
}

/// A begin code called as a command has no extent: mnras's `\keywords{…}` (a `\list` no `\endkeywords` closes) ends
/// where the paper's first section begins, and the sections are the document's (was the whole document inside
/// `<ltx:keywords>`, 3 Errors; 1004.2791, 2201.07008, 2309.07195).
#[test]
fn keywords_command_form_ends_at_first_section() {
  let (log, xml) = convert_repro!("keywords_command_form_ends_at_first_section.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<keywords name="Keywords: ">planets and satellites: interiors </keywords>"#),
    "{xml}"
  );
  let after = xml.split("</keywords>").nth(1).unwrap_or_default();
  for text in ["<section", "Body text here.", "More text."] {
    assert!(after.contains(text), "{text}: {xml}");
  }
}

/// ieeeconf's `\keywords` inside `{abstract}` is a label: it ends with the abstract, and the section after it is the
/// document's (was the abstract kept open past its `\end{abstract}`, the whole document inside `<ltx:keywords>`, 3
/// Errors; 1401.5657).
#[test]
fn keywords_label_inside_abstract_ends_with_it() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/keywords_label_inside_abstract_ends_with_it.tex"
    ),
    &[(
      "ieeeconf.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/ieeeconf.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract text.</p> </abstract>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<keywords name="Index Terms: ">alpha, beta </keywords>"#),
    "{xml}"
  );
  let after = xml.split("</keywords>").nth(1).unwrap_or_default();
  for text in ["<section", "Body text here."] {
    assert!(after.contains(text), "{text}: {xml}");
  }
}
