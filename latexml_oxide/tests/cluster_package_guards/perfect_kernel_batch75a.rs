//! Red/green guards for batch 75a, authored title-page text a raw class's title-page code holds (K11's off-table
//! stores): the `\maketitle` replay gate reads a body as TeX runs it — a `\def`'s parameter and replacement text are
//! not run where the definition stands, the delimiters of a called macro's parameter text are matched, not run, and a
//! macro a called one defines is defined (sect05.rs `body_vocabulary_is_defined`; 1505.04771, 1912.00312); a class's
//! append to the `\maketitle` the lock refused extends the class's own definition (state.rs
//! `extended_refused_definition`; wiley-article 1911.06213); and wiley-article's correspondence, e-mail, funding,
//! paper-type and journal-section stores and CSML's ACM CCS terms are K11 table setters (store_setters.rs; 1808.01384,
//! 1911.06213, 1608.04592); and the class's own record of a value its refused `\author` replay was given reads as empty in
//! the title-page replay (sect05.rs `note_replay_records`; vmsta 1506.02940). OXIDIZED_DESIGN_DIVERGENCES #756-#759. Each repro's pdflatex reference text is in its
//! header. Every guard asserts the three measures together: no Error/Warning, the authored text present once, and the
//! element carrying it.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion (0 errors, 0 warnings, no Fatal) of a `class_title_page` repro beside the minimal copy of its
/// class: its log and squashed XML.
macro_rules! convert_beside {
  ($path:literal, $cls:literal) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title_page/",
        $path
      )),
      &[(
        $cls,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title_page/",
          $cls
        )),
      )],
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(!log.lines().any(|l| l.starts_with("Fatal:")), "{log}");
    (log, squash(&xml))
  }};
}

/// The title and the one creator every repro's document sets, each once.
fn title_and_author_once(xml: &str) {
  assert!(
    xml.contains(
      r#"<title>On Widgets</title> <creator role="author"> <personname>A. Author</personname> </creator>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("On Widgets").count(), 1, "{xml}");
  assert_eq!(xml.matches("A. Author").count(), 1, "{xml}");
}

/// sig-alternate-05-2015.cls:726-737's `\maketitle` `\def`s `\@makefnmark` (reading `\@thefnmark`, which latex.ltx
/// defines only for a footnote) and peeks with `\futurelet`: neither stops its replay now, and its `\toappear` text is
/// typeset after the title as in the PDF (1505.04771; "not replayed: it uses \@thefnmark" in 204 of the 3,000 sample
/// papers).
#[test]
fn maketitle_replay_skips_stored_replacement_text() {
  let (log, xml) = convert_beside!(
    "maketitle_replay_skips_stored_replacement_text.tex",
    "tpfnmark.cls"
  );
  assert!(!log.contains("not replayed"), "{log}");
  title_and_author_once(&xml);
  assert!(
    xml.contains(
      r#"<para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><inline-block class="ltx_parbox" vattach="bottom" width="240.0pt" xml:id="p1.1.1"> <p xml:id="p1.1.1.1">This is a pre-print of an article appearing at KDD’16.</p> </inline-block></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Body text.</p> </para>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("pre-print").count(), 1, "{xml}");
}

/// nowfnt.cls:552-583: a helper delimited by `\@nil`, called by the class's own code, which defines the macro the code
/// reads next — the delimiter is matched, not run, and the helper's definitions count as defined (1912.00312).
#[test]
fn maketitle_replay_skips_called_delimiters() {
  let (log, xml) = convert_beside!(
    "maketitle_replay_skips_called_delimiters.tex",
    "tpdelim.cls"
  );
  assert!(!log.contains("not replayed"), "{log}");
  title_and_author_once(&xml);
  assert!(
    xml.contains(
      r#"<para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1">Series: Lecture Notes in Widgets</p> </para> <para xml:id="p2"> <p xml:id="p2.1">Body text.</p> </para>"#
    ),
    "{xml}"
  );
}

/// wiley-article.cls:420-466 renews `\maketitle`, :576 `\appto`s to it: the class's own title page and the append
/// both run, in order (1911.06213: "Ignoring redefinition of \maketitle" twice, the paper type lost).
#[test]
fn maketitle_class_append_extends_class_body() {
  let (_log, xml) = convert_beside!(
    "maketitle_class_append_extends_class_body.tex",
    "tpappend.cls"
  );
  title_and_author_once(&xml);
  assert!(
    xml.contains(
      r#"<para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Research Article</text></p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1">Funded by the Example Foundation.</p> </para> <para xml:id="p3"> <p xml:id="p3.1">Body text.</p> </para>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("Funded by").count(), 1, "{xml}");
}

/// wiley-article.cls:206-214 stores, printed by its `\maketitle` (type, section) and by its own abstract environment
/// (correspondence, e-mail, funding; :416-433), which the kernel's locked `{abstract}` refuses: each is a frontmatter
/// element, once, and the class's title-page replay, every value carried, adds nothing (1808.01384, 1911.06213).
#[test]
fn class_stores_printed_by_class_abstract() {
  let (_log, xml) = convert_beside!("class_stores_printed_by_class_abstract.tex", "tpwiley.cls");
  // (the name twice: the creator, and the correspondence block, which names the author as pdflatex prints it)
  assert!(
    xml.contains(
      r#"<title>On Widgets</title> <creator role="author"> <personname>A. Author</personname> </creator>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("On Widgets").count(), 1, "{xml}");
  assert_eq!(xml.matches("A. Author").count(), 2, "{xml}");
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">We study widgets.</p> </abstract> <pubnote name="Publication type: " role="type">Original Article</pubnote> <pubnote>Journal Section</pubnote> <note role="correspondence" xml:id="id1">A. Author, Department of Statistics, University of California, Davis, USA</note> <note role="email" xml:id="id2">author@example.edu</note> <note role="funding" xml:id="id3">This study was supported by the Example Foundation.</note> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"#
    ),
    "{xml}"
  );
  for text in [
    "Original Article",
    "Journal Section",
    "author@example.edu",
    "Example Foundation",
  ] {
    assert_eq!(
      xml.to_lowercase().matches(&text.to_lowercase()).count(),
      1,
      "{text}: {xml}"
    );
  }
}

/// CSML.cls:227-229 `\def\ACMCCS#1{\def\@ACMCCS{#1}}`, printed in a title-page footnote: the frontmatter's
/// classification (1608.04592, 1602.00476, whose `\@maketitle` deposit errs and was dropped with it).
#[test]
fn class_acmccs_store_is_a_classification() {
  let (_log, xml) = convert_beside!("class_acmccs_store_is_a_classification.tex", "tpccs.cls");
  title_and_author_once(&xml);
  assert!(
    xml.contains(
      r#"</creator> <classification name="Classification: " scheme="ACMCCS">[Theory of computation]: Models of computation</classification> <para xml:id="p1"> <p xml:id="p1.1">Body text.</p> </para>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("Models of computation").count(), 1, "{xml}");
}

/// vmsta.cls:296-315's refused `\author` replays for its side effects and appends the name to the class's own list,
/// which its `\maketitle` prints: the title-page replay reads that record as empty, so the author is the creator
/// once, and the received line the class alone prints is kept (1506.02940).
#[test]
fn maketitle_replay_reads_class_author_list_as_empty() {
  let (_log, xml) = convert_beside!(
    "maketitle_replay_reads_class_author_list_as_empty.tex",
    "tplist.cls"
  );
  title_and_author_once(&xml);
  assert!(
    xml.contains(
      r#"</creator> <para align="center" xml:id="p1"> <p xml:id="p1.1"><text fontsize="90%" xml:id="p1.1.1">Received: <text xml:id="p1.1.1.1">9 February 2015</text></text></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Body text.</p> </para>"#
    ),
    "{xml}"
  );
}
