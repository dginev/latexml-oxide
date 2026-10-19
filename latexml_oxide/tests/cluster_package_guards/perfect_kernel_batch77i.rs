//! Red/green guards for batch 77i, the last silent-loss rows in the frontmatter stores (SYNC_STATUS "Silent-loss
//! inventory" #5 (a), #6, #7, #8; 23b): a raw class's store set again after `\maketitle` (`\lx@store@hand`,
//! `settle_placed_stores`), a class's abstract store beside the document's `{abstract}` (`labelled_api`), a replayed
//! setter's own stores (`replay_dropped_class_code`, `note_replay_records`), a late class `\author` that prints
//! (`sealed_storing_run`: in place), the off-table stores no title-page run typesets (`hand_untypeset_stores`), and the
//! value of an unset rerouted store in the deposit (`decide_unset_nulls`). What the class's title page prints is decided
//! by TeX: a speculative run of it with each candidate store set to a sentinel (`before_title_page_runs`). Each guard
//! asserts the three measures: no Error or Warning, the authored text present once (or absent when pdflatex prints
//! none), its semantic element. Repros under `tools/perfect_kernel/repros/sectioning-frontmatter/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning, with its whitespace between tags
/// removed for counting.
fn convert_clean(tex: &str) -> (String, String) {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  if let Some(lines) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(lines, 0, "jing:\n{xml}");
  }
  let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  (xml, flat)
}

/// #6b: a class store set before `\maketitle` and again after it keeps the value the title page printed ("KW: alpha,
/// beta"), named by the class's label, which leaves the body; the late value only re-sets the store.
#[test]
fn store_set_again_after_maketitle_keeps_the_title_page_value() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_set_again_after_maketitle_keeps_the_title_page_value.tex"
  ));
  assert_element(
    &xml,
    "keywords",
    &[],
    r#"<keywords name="KW:">alpha, beta</keywords>"#,
  );
  assert_eq!(flat.matches("alpha, beta").count(), 1, "{xml}");
  assert!(!flat.contains("gamma"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// #6a: the class's abstract store, printed by its title page as "Resumen: …", and the document's `{abstract}` are
/// both kept, the class's named by its label and read at `\maketitle`, after the document's abstract set before it
/// (pdflatex's order); no orphan "Resumen:" before the body.
#[test]
fn abstract_store_keeps_the_document_abstract() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_store_keeps_the_document_abstract.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[r#"name="Resumen:""#],
    r#"<abstract inlist="toc" name="Resumen:" xml:id="abstract2"><p xml:id="abstract2.1">Spanish abstract.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "abstract",
    &[r#"name="Abstract""#],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">English abstract.</p></abstract>"#,
  );
  assert_eq!(flat.matches("Spanish abstract.").count(), 1, "{xml}");
  assert_eq!(flat.matches("English abstract.").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// #7: a document's `\renewcommand\author[1]{\gdef\myauth{#1}\gdef\@author{#1}}` keeps `\myauth` for the body; the
/// author is one creator.
#[test]
fn replayed_document_setter_keeps_its_store() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/replayed_document_setter_keeps_its_store.tex"
  ));
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Alice</personname></creator>"#,
  );
  assert_eq!(flat.matches("Alice").count(), 2, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Written by Alice.</p></para>"#,
  );
}

/// #8: a late class `\author` that stores and prints, after the document body began, runs in place: each author line
/// is an italic paragraph at the head of its section, as pdflatex prints it, and the document's creator stays the one
/// before the body (user ruling 2026-10-09; OXIDIZED_DESIGN #526).
#[test]
fn late_class_author_that_prints_is_kept_in_place() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/late_class_author_that_prints_is_kept_in_place.tex"
  ));
  assert_eq!(xml.matches("<creator ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>The Editor</personname></creator>"#,
  );
  for (unit, name) in [("S1", "Guest Writer"), ("S2", "Second Writer")] {
    assert_eq!(flat.matches(name).count(), 1, "{xml}");
    assert_element(
      &xml,
      "para",
      &[&format!(r#"xml:id="{unit}.p1""#)],
      &format!(
        r#"<para class="ltx_noindent" xml:id="{unit}.p1"><p xml:id="{unit}.p1.1"><text font="italic" xml:id="{unit}.p1.1.1">{name}</text></p></para>"#
      ),
    );
  }
}

/// #5 (a), #6 of the r2 review: the class's off-table stores that only its own `\@author` reads (jfpc.cls:124-132,
/// :188-191; 1503.05530): the first, its names, makes the creator when the document has none, and the one beside it is
/// a row of that creator's author block (`role=authorblock`), not a hover-only `ltx:pubnote`; once each.
#[test]
fn offtable_stores_read_by_class_author_are_the_creators() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_stores_read_by_class_author_are_the_creators.tex"
  ));
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Mohammed Bekkouche</personname><contact role="authorblock">bekkouche@unice.fr</contact></creator>"#,
  );
  assert_eq!(flat.matches("Mohammed Bekkouche").count(), 1, "{xml}");
  assert_eq!(flat.matches("bekkouche@unice.fr").count(), 1, "{xml}");
  for absent in [
    "donner les auteurs",
    "preciser les mails",
    "<note",
    "<pubnote",
  ] {
    assert!(!flat.contains(absent), "{xml}");
  }
}

/// #5 (a), the duplicate direction (CONTROL): a store the kept `\@maketitle` deposit typesets is not a note too.
#[test]
fn offtable_store_typeset_by_the_deposit_stays_there() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_store_typeset_by_the_deposit_stays_there.tex"
  ));
  assert_eq!(flat.matches("Widget Lab").count(), 1, "{xml}");
  assert!(!flat.contains("note"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Laboratory: Widget Lab</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">Text.</p></para>"#,
  );
}

/// #5 (a), jfpc's shape: the store the class's own `\title` hands its argument to is the document's title, the one its
/// `\author` hands on its creator, and the year its title page prints after "Actes JFPC" a note named so, with the
/// space printed before the year ("Actes JFPC 2015") — no visible label made of a setter's name (1503.05530).
#[test]
fn offtable_store_the_class_title_hands_on_is_the_title() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_store_the_class_title_hands_on_is_the_title.tex"
  ));
  assert!(
    flat.contains(r#"<title>Exploration</title> <creator role="author"> <personname>Mohammed Bekkouche</personname> </creator> <pubnote name="#),
    "{xml}"
  );
  // (the tie, U+00A0, which the whitespace split above takes as a space)
  assert!(
    xml.contains("<pubnote name=\"Actes JFPC\u{a0}\">2015</pubnote>"),
    "{xml}"
  );
  for text in ["Exploration", "Mohammed Bekkouche", "2015"] {
    assert_eq!(flat.matches(text).count(), 1, "{xml}");
  }
  for placeholder in ["definir le titre", "donner les auteurs", "<note"] {
    assert!(!flat.contains(placeholder), "{xml}");
  }
}

/// #5 (a), the negative direction: an off-table store the title page prints only under a switch that is off — the
/// class's own boolean (sigplanconf.cls:87-90, :1118; 1603.03011's placeholder DOI) or a `\newif` (aiaa-tc.cls:199-200
/// `[handcarry]`) — or only in page furniture (a `\markboth` running head, a `\setbox` it never uses there:
/// 1704.03582, 1508.03599) is not handed on. With the `\newif` on, the kept deposit typesets the value, once.
#[test]
fn offtable_store_the_title_page_does_not_print_is_not_handed_on() {
  for (repro, value) in [
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_store_under_a_class_boolean_that_is_off_is_not_handed_on.tex"
      ),
      "nnnnnnn.nnnnnnn",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_store_under_a_newif_that_is_off_is_not_handed_on.tex"
      ),
      "AIAA Conference 2004",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_store_printed_only_in_running_heads_is_not_handed_on.tex"
      ),
      "Short Widgets",
    ),
  ] {
    let (xml, flat) = convert_clean(repro);
    assert!(!flat.contains(value), "{xml}");
    assert!(!flat.contains("Vol. 12"), "{xml}");
    assert!(!flat.contains("note"), "{xml}");
    assert_element(
      &xml,
      "para",
      &[r#"xml:id="p1""#],
      r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
    );
  }
  let (xml, flat) = convert_clean(
    &include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/offtable_store_under_a_newif_that_is_off_is_not_handed_on.tex"
    )
    .replace(r"\documentclass{rcnewif}", r"\documentclass[handcarry]{rcnewif}"),
  );
  assert_eq!(flat.matches("AIAA Conference 2004").count(), 1, "{xml}");
  assert!(!flat.contains("note"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text fontsize="70%" xml:id="p1.1.1">AIAA Conference 2004</text></p></para>"#,
  );
}

/// #6b, the negative direction: a store set again after `\maketitle` that the title page does not print (the class's
/// abstract prints it at its end) takes the later value, which pdflatex prints there; the preamble placeholder goes.
#[test]
fn store_printed_outside_the_title_page_takes_the_later_value() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_set_again_after_maketitle_printed_outside_the_title_page_is_replaced.tex"
  ));
  // (named by the kernel's default, `\lx@keywords@name` "Keywords:~", as before 77i)
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">alpha, beta</keywords>",
  );
  assert_eq!(flat.matches("alpha, beta").count(), 1, "{xml}");
  assert!(!flat.contains("TO BE FILLED"), "{xml}");
}

/// 77h follow-up: a rerouted store the paper never set reads in the title-page deposit as the value under which the
/// title page prints less — `\@empty` where the class tests `\ifx\@institute\@empty` (no orphan "Institute:"),
/// `\relax` where it tests `\@ifundefined` (ptptex.cls:88-104: no journal line, no "(Received )"; hep-th9803240,
/// 1309.6905, 0903.1491). The an.cls shape is `perfect_kernel_batch77h::an_without_institute_keeps_the_title_block`.
#[test]
fn unset_store_reads_as_the_title_page_tests_it() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/unset_store_tested_for_emptiness_reads_empty_in_the_deposit.tex"
  ));
  assert!(!flat.contains("Institute"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Key words: lensing</p></para>"#,
  );
  assert_eq!(flat.matches("lensing").count(), 1, "{xml}");
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/unset_store_tested_for_undefinedness_reads_relax_in_the_deposit.tex"
  ));
  assert!(!flat.contains("Journal"), "{xml}");
  assert!(!flat.contains("Received"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Prog. Theor. Phys.</p></para>"#,
  );
}

/// #6a, r2 review finding 1: a named abstract or keywords store is read at `\maketitle`, as LaTeX expands it there, so
/// a value using a macro and a package defined later in the preamble reads as pdflatex prints it, with no error.
#[test]
fn named_store_using_later_definitions_reads_them_at_maketitle() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/named_store_using_later_definitions_reads_them_at_maketitle.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[],
    r##"<abstract inlist="toc" name="Resumen:" xml:id="abstract1"><p xml:id="abstract1.1">Uses bar and <text color="#FF0000" xml:id="abstract1.1.1">red</text>.</p></abstract>"##,
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    r#"<keywords name="KW:">kw bar</keywords>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
  assert!(!flat.contains("ERROR"), "{xml}");
}

/// r2 review finding 2 (a): a store the title page reads only as a number (`\setcounter{page}{\@fp}`, `\ifnum`) is
/// not printed there — both sentinels must reach the text — and is not handed on as a note.
#[test]
fn store_read_as_a_number_is_not_printed() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_read_as_a_number_is_not_printed.tex"
  ));
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Anton</personname></creator>"#,
  );
  assert!(!flat.contains("pubnote"), "{xml}");
  assert!(!flat.contains("17"), "{xml}");
  assert!(!flat.contains("Big volume"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// r2 review finding 2 (b): a store used only as an image's file or a link's target is not text on the page and is
/// not handed on; one typeset by `\url` is.
#[test]
fn store_in_an_image_path_or_link_target_is_not_printed() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_in_an_image_path_or_link_target_is_not_printed.tex"
  ));
  assert_element(
    &xml,
    "pubnote",
    &[],
    r#"<pubnote>https://site.org</pubnote>"#,
  );
  assert_eq!(flat.matches("<pubnote").count(), 1, "{xml}");
  assert!(!flat.contains("example-image"), "{xml}");
  assert!(!flat.contains("example.org"), "{xml}");
}

/// r2 review finding 3 (#6b): a store the title page prints in a footnote keeps the value it printed when set again
/// after `\maketitle`, and the footnote its label and store leave empty goes with them.
#[test]
fn store_printed_in_a_title_page_footnote_keeps_its_value() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_printed_in_a_title_page_footnote_keeps_its_value.tex"
  ));
  assert_element(
    &xml,
    "keywords",
    &[],
    r#"<keywords name="KW:">alpha, beta</keywords>"#,
  );
  assert!(!flat.contains("gamma"), "{xml}");
  assert!(!flat.contains("<note"), "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// Main-session decision on r2 review finding 5 (nowfnt, the page-furniture ruling): a cover author list that only
/// repeats the creators' names is not handed on (2308.02698); one that carries more is a note whose `\and` between
/// the authors' blocks is a line break (2108.12618).
#[test]
fn cover_author_list_is_dropped_only_when_it_repeats_the_names() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/cover_author_list_repeating_the_creators_is_dropped.tex"
  ));
  assert_two_creators(&xml, &flat);
  assert_eq!(flat.matches("Ann Smith").count(), 1, "{xml}");
  assert!(!flat.contains("pubnote"), "{xml}");
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/cover_author_list_with_affiliations_is_a_note_with_line_breaks.tex"
  ));
  assert_two_creators(&xml, &flat);
  assert_element(
    &xml,
    "pubnote",
    &[],
    r#"<pubnote>Ann Smith <break/>Univ A <break/>Bob Jones <break/>Univ B</pubnote>"#,
  );
}

/// The two creators of the cover-list and role repros, Ann Smith and Bob Jones, as whole elements.
fn assert_two_creators(xml: &str, flat: &str) {
  assert_eq!(flat.matches("<creator ").count(), 2, "{xml}");
  assert_element(
    xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Ann Smith</personname></creator>"#,
  );
  assert_element(
    xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Jones</personname></creator>",
  );
}

/// r3 review finding 1 (a), (b): a store used as a cross-reference key (`\ref{\@vb}`, pdflatex "see 1") is not
/// printed and not handed on; a store printed as `\href{mailto:\@vg}{\@vg}`'s text is, without the URL argument's
/// "mailto:" as its name (TeX's run of the title page does not print it before the store).
#[test]
fn store_label_is_printed_text_not_a_link_target_or_key() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_label_is_printed_text_not_a_link_target_or_key.tex"
  ));
  assert_element(&xml, "pubnote", &[], r#"<pubnote>me@x.org</pubnote>"#);
  assert_eq!(flat.matches("<pubnote").count(), 1, "{xml}");
  assert!(!flat.contains("sec:intro<"), "{xml}");
  assert!(!flat.contains("mailto:"), "{xml}");
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Anton</personname></creator>"#,
  );
}

/// r3 review finding 1 (c): the label of a store printed in a footnote after a sentence is the phrase right before
/// the store ("KW:"); the footnote keeps the sentence ("Funded.").
#[test]
fn store_label_is_the_phrase_printed_before_it() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_label_is_the_phrase_printed_before_it.tex"
  ));
  assert_element(
    &xml,
    "keywords",
    &[],
    r#"<keywords name="KW:">alpha, beta</keywords>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"role="footnote""#],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Funded. </note>"#,
  );
  assert!(!flat.contains("gamma"), "{xml}");
  assert_eq!(flat.matches("alpha, beta").count(), 1, "{xml}");
}

/// r3 review finding 2 (#6a): a class abstract store its title page prints without a label is read at `\maketitle`
/// and kept beside the document's `{abstract}` after it, first, as pdflatex prints them.
#[test]
fn unlabelled_abstract_store_keeps_the_document_abstract() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/unlabelled_abstract_store_keeps_the_document_abstract.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[r#"xml:id="abstract1""#],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Spanish abstract.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "abstract",
    &[r#"xml:id="abstract2""#],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract2"><p xml:id="abstract2.1">English abstract.</p></abstract>"#,
  );
  assert_eq!(flat.matches("Spanish abstract.").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// r3 review finding 2 (#6a): the class's own `\maketitle` prints `\end{center}\noindent Resumen: \@resumen`; the
/// abstract store is named "Resumen:", after the document's abstract, and no orphan "Resumen:" paragraph is left.
#[test]
fn class_maketitle_labelled_abstract_after_an_environment() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_maketitle_labelled_abstract_after_an_environment.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[r#"name="Abstract""#],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">English abstract.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "abstract",
    &[r#"name="Resumen:""#],
    r#"<abstract inlist="toc" name="Resumen:" xml:id="abstract2"><p xml:id="abstract2.1">Spanish abstract.</p></abstract>"#,
  );
  assert_eq!(flat.matches("Resumen:").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}

/// r3 review finding 5: a store printed under a label whose value is one author's name is a role, kept as a note
/// named by the label — not dropped as repeating the creators' names.
#[test]
fn labelled_role_store_naming_one_author_is_a_note() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/labelled_role_store_naming_one_author_is_a_note.tex"
  ));
  assert_two_creators(&xml, &flat);
  assert_element(
    &xml,
    "pubnote",
    &[],
    "<pubnote name=\"Corresponding author:\u{a0}\">Bob Jones</pubnote>",
  );
}

/// r3 review finding 6 (#6b): a store printed in the title page's `\thanks` keeps the value printed there when set
/// again after `\maketitle`; the `\footnotemark\footnotetext{…}` form leaves no empty footnote mark.
#[test]
fn store_printed_in_a_title_page_thanks_keeps_its_value() {
  for tex in [
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_printed_in_a_title_page_thanks_keeps_its_value.tex"
    ),
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_printed_in_a_footnotetext_drops_its_mark.tex"
    ),
  ] {
    let (xml, flat) = convert_clean(tex);
    assert_element(
      &xml,
      "keywords",
      &[],
      r#"<keywords name="KW:">alpha, beta</keywords>"#,
    );
    assert!(!flat.contains("gamma"), "{xml}");
    assert!(!flat.contains("<note"), "{xml}");
    assert_element(
      &xml,
      "para",
      &[r#"xml:id="p1""#],
      r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
    );
  }
}

/// r3 review finding 7: a store the title page tests against a word, printing the class's own word for it
/// (`\ifx\@vc\@open OPEN\else\@vc\fi`), is not handed on — the run with the document's own values does not print its
/// value; a store printed plainly is.
#[test]
fn store_tested_against_a_word_is_not_printed() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_tested_against_a_word_is_not_printed.tex"
  ));
  assert_element(&xml, "pubnote", &[], r#"<pubnote>dee</pubnote>"#);
  assert_eq!(flat.matches("<pubnote").count(), 1, "{xml}");
  assert!(!flat.contains(">open<"), "{xml}");
}

/// The deferred abstract store of a class whose own `[preprint]` `\maketitle` runs, prints `\@abst` and clears it
/// (ptptex, ptp-prep.clo:27-34) is handed on with the value its setter stored (0906.3938, 0907.2968).
#[test]
fn abstract_store_cleared_by_the_class_title_page_is_kept() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_store_cleared_by_the_class_title_page_is_kept.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">An abstract text.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "creator",
    &[],
    r#"<creator role="author"><personname>Anton</personname></creator>"#,
  );
  // KNOWN RESIDUAL, pinned (SYNC_STATUS "Silent-loss inventory", row "ptptex `[preprint]` double abstract"): the
  // class's own `\maketitle` runs and its printed block keeps the abstract too — twice where pdflatex prints it once,
  // as on base int29a (0906.3938). A fix makes this 1.
  assert_eq!(flat.matches("An abstract text.").count(), 2, "{xml}");
}

/// r4 review finding 1 (a): two setters of one store (`\let\keyword\keywords`, `\let\resumen\abst`) make one entry
/// each — the entry is the store's, handed on once whichever setter the document called.
#[test]
fn two_setters_of_one_store_make_one_entry() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/two_setters_of_one_store_make_one_entry.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Class abstract.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">alpha, beta</keywords>",
  );
  assert_eq!(flat.matches("Class abstract.").count(), 1, "{xml}");
  assert_eq!(flat.matches("alpha, beta").count(), 1, "{xml}");
}

/// r4 review finding 1 (b): an abstract store the title page reads but does not print (`\hypersetup{pdfsubject=
/// {\@abst}}`) is an ordinary entry, which the document's own `{abstract}` replaces: one abstract, as pdflatex prints.
#[test]
fn abstract_store_read_but_not_printed_yields_to_the_document_abstract() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_store_read_but_not_printed_yields_to_the_document_abstract.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Document abstract.</p></abstract>"#,
  );
  assert_eq!(flat.matches("<abstract ").count(), 1, "{xml}");
  assert_eq!(flat.matches("Metadata abstract.").count(), 0, "{xml}");
}

/// 77i r5 re-check: the same store with the document's abstracts set before `\maketitle` — an English one and an
/// `otherlanguage` German one — is dropped because the document already has its own abstract
/// (`frontmatter_release_store`); both document abstracts are kept, as pdflatex prints them.
#[test]
fn abstract_store_read_but_not_printed_yields_to_document_abstracts_before_maketitle() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/abstract_store_read_but_not_printed_yields_to_document_abstracts_before_maketitle.tex"
  ));
  assert_eq!(flat.matches("<abstract ").count(), 2, "{xml}");
  assert!(flat.contains("English document abstract."), "{xml}");
  assert!(flat.contains(r#"name="Zusammenfassung""#), "{xml}");
  assert!(flat.contains("Deutsche Zusammenfassung."), "{xml}");
  assert_eq!(flat.matches("Metadata abstract.").count(), 0, "{xml}");
}

/// r4 review finding 2: store values TeX typesets with other glyphs (`O'Brien` as O’Brien, ``` ``x'' ``` as “x”) or in
/// another case (`\MakeLowercase`) are confirmed printed, their letters and digits compared; the label with an
/// apostrophe names its note.
#[test]
fn store_values_typeset_with_quotes_are_printed() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_values_typeset_with_quotes_are_printed.tex"
  ));
  assert_element(
    &xml,
    "pubnote",
    &["name=\"Organiser\u{2019}s series:\u{a0}\""],
    "<pubnote name=\"Organiser\u{2019}s series:\u{a0}\">O\u{2019}Brien Lectures</pubnote>",
  );
  assert_element(
    &xml,
    "pubnote",
    &["name=\"Note:\u{a0}\""],
    "<pubnote name=\"Note:\u{a0}\">\u{201c}x\u{201d} words</pubnote>",
  );
  assert_element(
    &xml,
    "pubnote",
    &["name=\"Pages:\u{a0}\""],
    "<pubnote name=\"Pages:\u{a0}\">1\u{2013}10</pubnote>",
  );
  assert!(flat.contains("<pubnote>Mixed Case</pubnote>"), "{xml}");
  assert_eq!(flat.matches("<pubnote").count(), 4, "{xml}");
}

/// r4 review finding 2: a label with an apostrophe (`Author's keywords:`, typeset with ’) names the keywords and leaves
/// the title page with the store — no orphan label paragraph.
#[test]
fn store_label_with_an_apostrophe_names_the_entry() {
  let (xml, flat) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/store_label_with_an_apostrophe_names_the_entry.tex"
  ));
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Author\u{2019}s keywords:\">alpha, beta</keywords>",
  );
  assert_eq!(flat.matches("keywords:").count(), 1, "{xml}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text.</p></para>"#,
  );
}
