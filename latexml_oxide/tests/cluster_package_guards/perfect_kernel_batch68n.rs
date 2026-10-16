//! Red/green guards for batch 68n: a raw class's keywords reach the frontmatter API (K11, frontmatter_stores.rs) in
//! the two shapes the store reroute did not serve — a one-argument `\keywords` the class defines only for its
//! abstract's body (wiley-article/nbdt-article's `\NewEnviron`, astlb/pazh*'s `{pazhabstract}`, mtt's own
//! `{abstract}`; 1706.00599, 1801.08881, 1202.2889, astro-ph0306137, 1507.06302, 2103.05801, 1310.0585), and a
//! `{keywords}` environment whose begin code only typesets a heading with the class's label (paper.cls:621-627). Each
//! repro's pdflatex reference text is in its header.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A conversion of a `class_keywords` repro beside the minimal copy of its class, with its log and squashed XML.
macro_rules! convert_beside {
  ($path:literal, $cls:literal) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/",
        $path
      )),
      &[(
        $cls,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/",
          $cls
        )),
      )],
      Some("ar5iv.sty"),
    );
    (log, squash(&xml))
  }};
}

/// The abstract every `abstract_keywords_*` repro keeps, its keywords gone from it.
const ABSTRACT: &str = r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract text.</p> </abstract>"#;

/// wiley-article.cls:446-489 `\NewEnviron{wiley@abstract}{\newcommand{\keywords}[1]{…}…}` with `\renewcommand
/// {\abstract}{\wiley@abstract}`, which the kernel's locked `{abstract}` refuses: `\keywords{…}` in the abstract was
/// undefined (1 Error), its text a paragraph of the abstract (1706.00599, 1712.05593, 1801.08881's nbdt-article.cls).
/// The keywords are named by the class's label ahead of the argument, its text only (kwwiley.cls's `\noindent{\bfseries
/// KEYWORDS\par}`; the real class's `\allcaps[…]{keywords}` is not that shape, and keeps the default name).
#[test]
fn wiley_abstract_keywords_are_the_frontmatter_keywords() {
  let (log, xml) = convert_beside!("abstract_keywords_wiley.tex", "kwwiley.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(ABSTRACT), "{xml}");
  assert!(
    xml.contains(r#"<keywords name="KEYWORDS">first, second</keywords>"#),
    "{xml}"
  );
}

/// astlb.cls:91-205: the abstract's body goes to a token register, `\endabstract` calls `\maketitle`, and the class's
/// `\@maketitle` typesets it inside `{pazhabstract}`, which defines `\def\keywords##1` (1202.2889, astro-ph0306137's
/// pazhastl.cls, 1507.06302's pazhb_eng.cls, 2103.05801's pazhb.cls), named by astlb.cls:200's label `\medskip Key
/// words:~~\emph{##1}`, its text only.
#[test]
fn pazh_abstract_keywords_are_the_frontmatter_keywords() {
  let (log, xml) = convert_beside!("abstract_keywords_pazh.tex", "kwpazh.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(ABSTRACT), "{xml}");
  assert!(
    xml.contains(r#"<keywords name="Key words:">first, second</keywords>"#),
    "{xml}"
  );
}

/// mtt.cls:176-181's own `\renewenvironment{abstract}[1][ru]{…\def\keywords##1{…}…}`, refused by the lock: the
/// refused begin code is kept (`\lx@dropped@abstract`, sect08.rs) and its `\keywords` sets the keywords (1310.0585,
/// 1212.3890's mttr.cls).
#[test]
fn class_abstract_environment_keywords_are_the_frontmatter_keywords() {
  let (log, xml) = convert_beside!("abstract_keywords_mtt.tex", "kwmtt.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(ABSTRACT), "{xml}");
  assert!(
    xml.contains(r#"<keywords name="Keywords:">first, second</keywords>"#),
    "{xml}"
  );
}

/// A supplement's abstract after the document body began stays in place, and so do the keywords its class's abstract
/// defines for it, typeset by the class's own definition (kwmtt.cls's `\par\noindent{\bfseries Keywords:}
/// \itshape##1`; pdflatex "English abstract. Keywords: english words"); the paper's keywords stay the frontmatter's
/// (1310.0585's English abstract at its end, whose keywords had replaced the paper's Russian ones and were then dropped
/// as a supplement's).
#[test]
fn supplement_abstract_keywords_stay_in_place() {
  let (log, xml) = convert_beside!(
    "abstract_keywords_supplement_abstract_in_place.tex",
    "kwmtt.cls"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Main abstract.</p> </abstract> <keywords name="Keywords:">main words</keywords>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<para xml:id="S1.p2"> <p xml:id="S1.p2.1">English abstract.</p> </para> <para class="ltx_noindent" xml:id="S1.p3"> <p xml:id="S1.p3.1"><text font="bold" xml:id="S1.p3.1.1">Keywords:</text> <text font="italic" xml:id="S1.p3.1.2">english words </text></p> </para>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<keywords").count(), 1, "{xml}");
}

/// Made as the class makes it: inside the abstract the class's `\def\keywords##1` replaces the document's own
/// `\keywords` (pdflatex prints the class's "Keywords: first"), outside it the document's stays ("KW: outside").
#[test]
fn class_def_of_abstract_keywords_wins_only_in_the_abstract() {
  let (log, xml) = convert_beside!(
    "abstract_keywords_class_def_wins_in_the_abstract.tex",
    "kwmtt.cls"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(ABSTRACT), "{xml}");
  assert!(
    xml.contains(r#"<keywords name="Keywords:">first</keywords>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Body. KW: outside</p>"#),
    "{xml}"
  );
}

/// paper.cls:621-622 `\def\keywords{\small\paragraph*{\keywordname:}}` `\def\endkeywords{\par\bigskip}`: the
/// environment is the frontmatter's keywords, named by the class's label, not a run-in `<paragraph>` (R9 of the
/// 2026-10-15 rawclasses regressions: 150 paper-class papers under the lever).
#[test]
fn heading_keywords_environment_is_the_frontmatter_keywords() {
  let (log, xml) = convert_beside!("keywords_environment_paper.tex", "kwpaper.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(ABSTRACT), "{xml}");
  assert!(
    xml.contains(r#"<keywords name="Keywords:"> first, second </keywords>"#),
    "{xml}"
  );
  assert!(!xml.contains("<paragraph"), "{xml}");
}

/// The label-only `{keywords}` called as a command: braced, the keywords of its argument; bare, the class's own run-in
/// heading, whose extent the frontmatter cannot know, with the rest of the document where it was.
#[test]
fn heading_keywords_command_forms() {
  let (log, xml) = convert_beside!("keywords_environment_command_forms.tex", "kwpaper.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<keywords").count(), 1, "{xml}");
  assert!(xml.contains(r#"<keywords name="Keywords:"><text fontsize="90%" xml:id="id1">first, second</text></keywords>"#), "{xml}");
  assert!(xml.contains(r#"<p xml:id="p1.1">Text.</p>"#), "{xml}");
  assert!(
    xml.contains(r#"<title fontsize="90%">Keywords:</title>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text fontsize="90%" xml:id="S0.SS0.SSS0.Px1.p1.1.1">bare words</text>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<section inlist="toc" xml:id="S1">"#),
    "{xml}"
  );
}

/// Label-only begin codes of shipped raw classes: pasa.cls:101-103's class font macro (`\keywordfont`, :56) and
/// `\textbf{Keywords: }`, biometrika.cls:525-527's `{\itshape Some key words}:\ ` (1801.03506, 2412.19242).
#[test]
fn shipped_label_only_keywords_environments() {
  let (log, xml) = convert_beside!("keywords_environment_label_shapes.tex", "kwlabels.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<keywords name="Keywords:">alpha, beta </keywords> <keywords name="Some key words:"> gamma </keywords>"#),
    "{xml}"
  );
}

/// A begin code that also sets a flag the title page reads (tADR2e.cls:450 `\global\keywordstrue`) is not
/// label-only: the environment stays the class's.
#[test]
fn keywords_environment_with_a_flag_stays_the_classs() {
  let (log, xml) = convert_beside!("keywords_environment_with_a_flag_stays.tex", "kwflag.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<keywords"), "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Keywords:</text> alpha, beta</p>"#
    ),
    "{xml}"
  );
}

/// The document's own `{keywords}`, defined after the class's, stays the document's.
#[test]
fn documents_own_keywords_environment_stays() {
  let (log, xml) = convert_beside!(
    "keywords_environment_documents_own_stays.tex",
    "kwpaper.cls"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<keywords"), "{xml}");
  assert!(
    xml.contains(r#"<para xml:id="p1"> <p xml:id="p1.1">KW: first</p> </para>"#),
    "{xml}"
  );
}

/// `\keywords` that nothing defines stays an undefined-macro Error, as in pdflatex.
#[test]
fn undefined_keywords_stay_an_error() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/keywords_undefined_stays_an_error.tex"
    ),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(log.contains(r"Error:undefined:\keywords"), "{log}");
  assert!(!xml.contains("<keywords"), "{xml}");
}

/// A label-only `{keywords}` whose label is a brace group (kwgroups.cls): the group's font and layout commands are the
/// label's — the keywords' `name` is the label's text only, and the keywords text is not in the label's font (pdflatex:
/// the label in CMBX, the keywords not bold). One conversion per class option, each asserting the whole element.
macro_rules! group_label_guard {
  ($test:ident, $tex:literal, $name:literal) => {
    #[test]
    fn $test() {
      let (log, xml) = convert_beside!($tex, "kwgroups.cls");
      assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
      assert!(
        xml.contains(concat!(
          r#"<keywords name=""#,
          $name,
          r#"">alpha, beta </keywords> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
        )),
        "{xml}"
      );
      assert!(!xml.contains("font="), "{xml}");
      assert_eq!(xml.matches("<keywords").count(), 1, "{xml}");
    }
  };
}

// The review's minimal shapes: `{\noindent\bfseries Keywords.}` (va), `{\bfseries Keywords.\vspace{\z@}}` (vb),
// `{\bfseries Keywords:}` (ve).
group_label_guard!(
  group_label_with_layout_inside,
  "keywords_group_label_va.tex",
  "Keywords."
);
group_label_guard!(
  group_label_with_trailing_space,
  "keywords_group_label_vb.tex",
  "Keywords."
);
group_label_guard!(
  group_label_alone,
  "keywords_group_label_ve.tex",
  "Keywords:"
);
// emsprocart.cls:668-670 `\vspace{2pt}\small{\noindent\bfseries{\keywordname.}\vspace{\z@}}` (1202.4161, 1006.0632,
// 1610.08545).
group_label_guard!(
  group_label_emsprocart,
  "keywords_group_label_emsprocart.tex",
  "Keywords."
);
// ieice.cls:952-954 `\noindent\footnotesize\itshape{\bfseries key words:\ \ignorespaces}` (2002.00666, 1604.03276,
// 2509.21881).
group_label_guard!(
  group_label_ieice,
  "keywords_group_label_ieice.tex",
  "key words:"
);
// 1604.03313's lni.cls:157-158 `\fontsize{9}{10}\selectfont\noindent{\bfseries Keywords:}`.
group_label_guard!(group_label_lni, "keywords_group_label_lni.tex", "Keywords:");

/// `\keywords {alpha}, beta, gamma`: the first group does not end the keywords (no `\par`, blank line or `\end…` after
/// it), so the class's own code runs and they stay together, a run-in paragraph as in pdflatex.
#[test]
fn braced_first_group_that_does_not_end_the_keywords_runs_the_classs_code() {
  let (log, xml) = convert_beside!("keywords_environment_braced_first_group.tex", "kwpaper.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<keywords"), "{xml}");
  assert!(
    xml.contains(r#"<paragraph xml:id="S0.SS0.SSS0.Px1"> <title fontsize="90%">Keywords:</title> <para xml:id="S0.SS0.SSS0.Px1.p1"> <p xml:id="S0.SS0.SSS0.Px1.p1.1"><text fontsize="90%" xml:id="S0.SS0.SSS0.Px1.p1.1.1">alpha, beta, gamma</text></p> </para> </paragraph>"#),
    "{xml}"
  );
}

/// A bare `\keywords alpha, beta \endkeywords`: outside the environment `\endkeywords` is the class's own end code
/// (paper.cls:622 `\par\bigskip`), so "After text." is a paragraph of its own, as in pdflatex.
#[test]
fn bare_endkeywords_is_the_classs_end_code() {
  let (log, xml) = convert_beside!("keywords_environment_bare_endkeywords.tex", "kwpaper.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<keywords"), "{xml}");
  assert!(
    xml.contains(r#"<paragraph xml:id="S0.SS0.SSS0.Px1"> <title fontsize="90%">Keywords:</title> <para xml:id="S0.SS0.SSS0.Px1.p1"> <p xml:id="S0.SS0.SSS0.Px1.p1.1"><text fontsize="90%" xml:id="S0.SS0.SSS0.Px1.p1.1.1">alpha, beta</text></p> </para> <para xml:id="S0.SS0.SSS0.Px1.p2"> <p xml:id="S0.SS0.SSS0.Px1.p2.1"><text fontsize="90%" xml:id="S0.SS0.SSS0.Px1.p2.1.1">After text.</text></p> </para> </paragraph>"#),
    "{xml}"
  );
}

/// A one-argument `\keywords` the class's abstract defines that does not typeset its argument (it only warns:
/// beilstein.cls:854's, fbithesis.cls:272's shape) is not the keywords' setter: it is the class's own definition, whose
/// warning is the one diagnostic, as in pdflatex.
#[test]
fn abstract_setter_that_only_warns_is_the_classs_own() {
  let (log, xml) = convert_beside!("abstract_keywords_setter_that_only_warns.tex", "kwwarn.cls");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Class kwwarn Warning: Keywords are not part of this document type"),
    "{log}"
  );
  assert!(!xml.contains("<keywords"), "{xml}");
  assert!(xml.contains(ABSTRACT), "{xml}");
}
