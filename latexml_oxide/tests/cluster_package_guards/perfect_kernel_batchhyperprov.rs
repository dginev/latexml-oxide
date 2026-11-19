//! Red/green guards for HYPERPROV: the ar5iv preload's arXiv-hyperref emulation (OXIDIZED_DESIGN #588) provides
//! `\texorpdfstring`, hyperref's user commands and atbegshi's shipout hooks on first use to a paper that never loaded
//! hyperref or atbegshi, only the names still undefined (DG rulings 2026-11-19).
use super::perfect_kernel_batch46::{error_count, warning_count};

const TEXORPDFSTRING: &str = include_str!(
  "../../../tools/perfect_kernel/repros/package-bindings/texorpdfstring_without_hyperref.tex"
);
const ATBEGSHI: &str = include_str!(
  "../../../tools/perfect_kernel/repros/package-bindings/atbegshi_hooks_without_package.tex"
);
const SHIPOUT_NOTICE: &str = include_str!(
  "../../../tools/perfect_kernel/repros/package-bindings/shipout_notice_and_preamble_hooks.tex"
);
const HYPERREF_COMMANDS: &str = include_str!(
  "../../../tools/perfect_kernel/repros/package-bindings/hyperref_commands_without_hyperref.tex"
);

fn title(xml: &str) -> &str {
  let start = xml.find("<title>").expect("a section title");
  &xml[start..start + xml[start..].find("</title>").unwrap() + "</title>".len()]
}

/// hyperref's `\texorpdfstring` keeps its TeX string in a section title and drops the PDF string, which was printed
/// after an `undefined` Error (hep-th0702068).
#[test]
fn texorpdfstring_without_hyperref() {
  let (log, xml) = latexml::util::test::convert_with(TEXORPDFSTRING, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let title = title(&xml);
  assert!(
    title.starts_with(r#"<title><tag close=" ">1</tag>Energy <Math mode="inline" tex="E=mc^{2}""#)
      && title.ends_with("</Math> law</title>")
      && title.matches("<Math ").count() == 1,
    "{title}"
  );
  assert!(!xml.contains("mc2") && !xml.contains("ERROR"), "{xml}");
}

/// Provide-only: a paper's own `\texorpdfstring` (here keeping the PDF string) stays its own.
#[test]
fn own_texorpdfstring_kept() {
  let tex = "\\documentclass{article}\n\\newcommand{\\texorpdfstring}[2]{#2}\n\\begin{document}\n\
             \\section{A \\texorpdfstring{$x$}{ex} b}\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    title(&xml),
    r#"<title><tag close=" ">1</tag>A ex b</title>"#
  );
}

/// atbegshi's hooks drop their material but for what pdflatex prints on a page, the upper-left pictures' and the page
/// boxes', each a publication note of its words with no picture; a rule or a `\special` makes no note (1612.04433's
/// tikz highlighter); `\AtBeginShipoutBox` is a box register (2309.06611, 2403.15239, 1607.05658, 2507.04353).
#[test]
fn atbegshi_hooks_without_package() {
  let (log, xml) = latexml::util::test::convert_with(ATBEGSHI, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let notes: Vec<&str> = xml
    .lines()
    .filter(|line| line.contains("<pubnote"))
    .map(str::trim)
    .collect();
  assert_eq!(
    notes,
    ["X", "Y", "Z", "W"].map(|text| format!(r#"<pubnote role="note">{text}</pubnote>"#)),
    "{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Body text.</p></para>"#,
  );
  assert!(!xml.contains("<picture") && !xml.contains("<rule"), "{xml}");
}

/// A preamble hook (as lpor2014.cls:2987, 1503.07687) and a body one in the same paper: the first use provides, so
/// neither errs; the first page's material (2302.00362's `\AtBeginShipoutFirst{\raisebox{0cm}{\parbox{…}{…}}}`), a
/// page box's `\put` and an upper-left notice registered twice (2309.06611's IEEE copyright) are a note each, of their
/// words and boxes, with no picture.
#[test]
fn shipout_notice_and_preamble_hooks() {
  let (log, xml) = latexml::util::test::convert_with(SHIPOUT_NOTICE, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let start = xml.find("<pubnote").expect("a note");
  let notes: String = xml[start..xml.find("<para").unwrap()]
    .split_whitespace()
    .collect();
  assert_eq!(
    notes,
    r#"<pubnoterole="note"><textyoffset="0.0pt"xml:id="id1"><inline-blockclass="ltx_parbox"vattach="middle"width="310.5pt"xml:id="id1.1"><palign="center"xml:id="id1.1.1">FirstNoticetext</p></inline-block></text></pubnote><pubnoterole="note">BoxNoticewords</pubnote><pubnoterole="note"><inline-blockclass="ltx_parbox"vattach="middle"width="142.3pt"xml:id="id2"><pxml:id="id2.1">Noticetext</p></inline-block></pubnote>"#
  );
  assert_eq!(xml.matches("<pubnote").count(), 3, "{xml}");
  assert!(!xml.contains(">P<") && !xml.contains("ERROR"), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">Body.</p></para>"#,
  );
}

/// hyperref's user commands on first use: the binding's references and anchor, `\hypersetup` in the preamble.
#[test]
fn hyperref_commands_without_hyperref() {
  let (log, xml) = latexml::util::test::convert_with(HYPERREF_COMMANDS, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"See <ref class="ltx_refmacro_autoref" labelref="LABEL:sec:b" show="autoref"/>, <ref class="ltx_refmacro_nameref" labelref="LABEL:sec:b" show="title"/>, <ref labelref="LABEL:sec:b">text</ref>, <anchor xml:id="tg">T</anchor>, <ref idref="tg">L</ref>."#
    ),
    "{xml}"
  );
  // the autoref tag of a target after the first use (hyperref's active `~`, a no-break space)
  assert!(
    xml.contains("<tag role=\"autoref\">section\u{a0}2<"),
    "{xml}"
  );
}

/// Nothing is predefined: before a first use `\ifdefined`/`\@ifundefined` see the names undefined, and a paper's own
/// `\newcommand` of one is no redefinition.
#[test]
fn nothing_predefined_before_first_use() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\edef\\probe{\\ifdefined\\hypersetup A\\else B\\fi\
             \\@ifundefined{AtBeginShipout}{C}{D}\\ifdefined\\texorpdfstring E\\else F\\fi}\n\\makeatother\n\
             \\newcommand\\hypersetup[1]{}\\hypersetup{colorlinks}\n\\begin{document}\n\\probe\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p xml:id="p1.1">BCF</p></para>"#,
  );
}

/// Without the ar5iv preload nothing is provided: both documents keep TeX's `undefined` Errors.
#[test]
fn no_provide_without_the_preload() {
  let (log, _xml) = latexml::util::test::convert_with(TEXORPDFSTRING, None);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:\\texorpdfstring"), "{log}");
  let (log, _xml) = latexml::util::test::convert_with(ATBEGSHI, None);
  assert_eq!((error_count(&log), warning_count(&log)), (9, 2), "{log}");
  assert!(log.contains("Error:undefined:\\AtBeginShipoutBox"), "{log}");
  let (log, _xml) = latexml::util::test::convert_with(HYPERREF_COMMANDS, None);
  assert!(
    log.contains("Error:undefined:\\hypersetup") && log.contains("Error:undefined:\\autoref"),
    "{log}"
  );
}
