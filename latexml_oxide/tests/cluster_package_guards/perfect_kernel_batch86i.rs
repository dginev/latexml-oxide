//! Red/green guards for batch 86i: a second abstract pdflatex prints where it stands is kept. OXIDIZED_DESIGN #154
//! forward-ported upstream's `%ReplaceableFrontmatterTags`, so a later `{abstract}` of the same name and language
//! replaced the earlier one — a silent content loss only Rust had: upstream removed the table (#2767), and the vendored
//! Perl, that HEAD, keeps both. Titles, dates and keywords (which bindings re-add as their list grows: OmniBus's
//! `\kwd`) still replace (2002.09766); an abstract replaces only a store's entry no title page printed
//! (`PRINTED_WHERE_THEY_STAND`, `frontmatter_clear_same_name`), an earlier one whose text the new one repeats is
//! dropped (`frontmatter_drop_repeats`), and the abstract's command form `\abstract{…}`, which Perl queues, is dequeued
//! by the next abstract until a flush (`frontmatter_clear_command_form`). Each later late abstract follows the cover
//! lines printed before it (`place_later_abstract`, cover_order.rs). Witnesses: 1006.5288 (amsart), 2406.04383 (ceurart),
//! 2404.04417 (elsarticle: an empty later `{abstract}` had cleared the only one), 2305.00493 (sn-jnl). Each guard
//! converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/` and asserts no Error or Warning and
//! the abstracts whole. One guard per path: the kernel `{abstract}` after `\maketitle` (amsart, the witness's shape),
//! the same path under other classes (article, IEEEtran with its accumulating keywords), the repeat dropped (`@until`
//! and `@now`), the command form given twice before `\maketitle` (sn-jnl), and the second late abstract's placement
//! (its streamed build: `114_streaming_structure::streaming_late_abstract_follows_its_cover`). The flipped pre-86i
//! guard is `06_cluster_regressions::cluster_frontmatter_replaceable_dedup` (two article abstracts, now both kept).
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  perfect_kernel_batch86g::root_order,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Converts `tex`, checks it converts with no Error or Warning, and returns the XML.
#[track_caller]
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// 1006.5288's shape: amsart's two `{abstract}`s after `\maketitle`, both printed where they stand (amsart.cls:854).
/// The second replaced the first, with its in-abstract keywords and MSC lines; both are kept, in source order, as the
/// vendored Perl keeps them.
#[test]
fn second_abstract_after_maketitle_is_kept() {
  let xml = convert_clean(repro!("second_abstract_after_maketitle_amsart.tex"));
  assert_eq!(root_order(&xml), [
    "title", "creator", "abstract", "abstract", "section"
  ]);
  assert_element(
    &xml,
    "abstract",
    &[r#"xml:id="abstract1""#],
    r#"<abstract inlist="toc" name="Abstract." xml:id="abstract1"><p xml:id="abstract1.1">English words. <text font="bold" xml:id="abstract1.1.1">Keywords:</text> alpha. <text font="bold" xml:id="abstract1.1.2">MSC:</text> 60G51.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "abstract",
    &[r#"xml:id="abstract2""#],
    r#"<abstract inlist="toc" name="Abstract." xml:id="abstract2"><p xml:id="abstract2.1">Mots français.</p></abstract>"#,
  );
}

/// The same path under article and IEEEtran (whose `{IEEEkeywords}` accumulate already): both abstracts, in source
/// order; IEEEtran's two keyword blocks after them, the kernel's frontmatter order, as in Perl.
#[test]
fn second_abstract_kept_in_other_classes() {
  let article = convert_clean(repro!("second_abstract_after_maketitle_article.tex"));
  assert_eq!(root_order(&article), [
    "title", "creator", "abstract", "abstract", "section"
  ]);
  let ieee = convert_clean(repro!("second_abstract_after_maketitle_ieeetran.tex"));
  assert_eq!(root_order(&ieee), [
    "title", "creator", "abstract", "abstract", "keywords", "keywords", "section"
  ]);
  for xml in [&article, &ieee] {
    assert_element(
      xml,
      "abstract",
      &[r#"xml:id="abstract1""#],
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">English words.</p></abstract>"#,
    );
    assert_element(
      xml,
      "abstract",
      &[r#"xml:id="abstract2""#],
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract2"><p xml:id="abstract2.1">Second words.</p></abstract>"#,
    );
  }
  let keywords: Vec<&str> = ieee
    .match_indices("<keywords ")
    .map(|(at, _)| &ieee[at..at + ieee[at..].find("</keywords>").unwrap() + "</keywords>".len()])
    .collect();
  assert_eq!(keywords, [
    "<keywords name=\"Index Terms:\u{a0}\">kw one</keywords>",
    "<keywords name=\"Index Terms:\u{a0}\">kw two</keywords>",
  ]);
}

/// An abstract whose name, language and text, whitespace aside, repeat an earlier one's — `{abstract}` twice
/// (`\lx@add@frontmatter@until`), then `\lx@add@frontmatter@now` — is one block, the safety net the clearing was for a
/// block a class or binding adds twice; an abstract with other text is a second one (the guards above). Keywords
/// re-added as their list grows still replace the earlier ones (the last, "alpha, beta").
#[test]
fn repeated_abstract_with_the_same_text_is_one() {
  let xml = convert_clean(repro!("repeated_abstract_with_the_same_text_is_one.tex"));
  assert_eq!(root_order(&xml), [
    "title", "creator", "abstract", "keywords", "section"
  ]);
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Same words.</p></abstract>"#,
  );
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">alpha, beta</keywords>",
  );
}

/// sn-jnl's template `\abstract{…}` and the paper's own, both before `\maketitle` (2305.00493, 2609.14034): one
/// abstract, the paper's, as the class prints the last and Perl's dequeue keeps it — the brace form, digested in place
/// here, is the command form Perl queues (`frontmatter_clear_command_form`). The dequeue's two departures from Perl's,
/// each a printed abstract the base kept and r1 lost (review probes): none after `\maketitle` (q01: `\abstract{…}`
/// then `{abstract}`; q07: then a supplement's sealed `{abstract}`, whose clear removed the only head abstract), and
/// none across a name or language (q15 before `\maketitle`, q16 after). The mark is its call's own keyval, so a
/// sealed command form marks nothing after it (q13: "Premier" was removed by "Second").
#[test]
fn abstract_command_form_twice_before_maketitle_keeps_the_last() {
  let xml = convert_clean(repro!(
    "abstract_command_form_twice_before_maketitle_keeps_the_last.tex"
  ));
  assert_eq!(root_order(&xml), [
    "title",
    "creator",
    "abstract",
    "classification",
    "section"
  ]);
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">The paper’s own abstract words.</p></abstract>"#,
  );
  assert!(!xml.contains("general introduction"), "{xml}");

  let english = |n: u8, words: &str| {
    format!(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract{n}"><p xml:id="abstract{n}.1">{words}</p></abstract>"#
    )
  };
  let french = |n: u8, words: &str| {
    format!(
      r#"<abstract inlist="toc" name="Résumé" xml:id="abstract{n}"><p xml:id="abstract{n}.1"><text xml:lang="fr" xml:id="abstract{n}.1.1">{words}</text></p></abstract>"#
    )
  };
  // (probe, repro, the head abstracts in order, the sections, a supplement's text kept in the body)
  type Variant<'a> = (&'a str, &'a str, Vec<String>, usize, Option<&'a str>);
  let variants: [Variant; 5] = [
    (
      "q01",
      repro!("abstract_command_form_after_maketitle_is_kept.tex"),
      vec![
        english(1, "Command abstract words."),
        english(2, "Environment abstract words."),
      ],
      1,
      None,
    ),
    (
      "q07",
      repro!("abstract_command_form_then_supplement_abstract.tex"),
      vec![english(1, "Command abstract words.")],
      2,
      Some("Supplement abstract words."),
    ),
    (
      "q15",
      repro!("abstract_command_form_then_french_abstract_before_maketitle.tex"),
      vec![
        english(1, "English abstract words."),
        french(2, "Mots du résumé."),
      ],
      1,
      None,
    ),
    (
      "q16",
      repro!("abstract_command_form_bilingual_after_maketitle.tex"),
      vec![
        english(1, "English abstract words."),
        french(2, "Mots du résumé."),
      ],
      1,
      None,
    ),
    (
      "q13",
      repro!("abstract_command_form_sealed_then_two_french_abstracts.tex"),
      vec![
        english(1, "English abstract words."),
        french(2, "Premier résumé mots."),
        french(3, "Second résumé mots."),
      ],
      1,
      Some("Supplement command words."),
    ),
  ];
  for (probe, tex, abstracts, sections, in_body) in variants {
    let xml = convert_clean(tex);
    let order = root_order(&xml);
    let heads: Vec<&str> = order
      .iter()
      .map(String::as_str)
      .filter(|n| *n != "section")
      .collect();
    let mut expected = vec!["title", "creator"];
    expected.extend(abstracts.iter().map(|_| "abstract"));
    assert_eq!(heads, expected, "{probe}:\n{xml}");
    assert_eq!(
      order.iter().filter(|n| *n == "section").count(),
      sections,
      "{probe}:\n{xml}"
    );
    assert_eq!(
      xml.matches("<abstract ").count(),
      abstracts.len(),
      "{probe}:\n{xml}"
    );
    for (at, element) in abstracts.iter().enumerate() {
      assert_element(
        &xml,
        "abstract",
        &[&format!(r#"xml:id="abstract{}""#, at + 1)],
        element,
      );
    }
    // (a supplement's, after a section, stays in place as body text: the seal)
    if let Some(words) = in_body {
      let section = xml
        .find("<section")
        .unwrap_or_else(|| panic!("{probe}:\n{xml}"));
      assert!(xml[section..].contains(words), "{probe}:\n{xml}");
    }
  }
}

/// Two late abstracts, a cover line before each: the first follows the first line (the `<titlepage>`,
/// OXIDIZED_DESIGN_DIVERGENCES #903), the second the second line (`place_later_abstract`), pdflatex's print order.
/// Both moved behind the first line once both were kept. (Two late abstracts with nothing between them stay together
/// behind the cover: `perfect_kernel_batch86g::late_abstract_follows_the_hand_typeset_cover`.) Each marker places the
/// abstract its own command added (`LATE_MARK_ORDINAL`), and one whose command added none places nothing: a repeat
/// dropped (review probe q10: "Other" sat ahead of the third cover line under counted markers) and an empty
/// `{abstract}` (q09, 2404.04417's shape: the real abstract follows both cover lines). Three and four late abstracts
/// each follow their own line (r2 review probe o2d: the middle ones stayed ahead of theirs), on the kernel path and on
/// smfart's (o6); an empty abstract's marker does not place the markerless abstract a later command adds (r3 probe
/// m6, `take_late_mark`). Late abstracts keep their source order: a move carries the later ones waiting right after
/// it, so one whose own move is refused stays right after the one before it — body prose (r3 review probe q1) or a
/// float (q2) after the second, prose after the second of four (q10: both carried) — and so does a markerless one
/// (q9).
#[test]
fn second_late_abstract_follows_its_own_cover_line() {
  let xml = convert_clean(repro!("two_late_abstracts_with_a_cover_line_between.tex"));
  assert_eq!(root_order(&xml), [
    "title",
    "creator",
    "titlepage[block: Cover line one]",
    "abstract",
    "para",
    "abstract",
    "section",
  ]);
  let at = |needle: &str| {
    xml
      .find(needle)
      .unwrap_or_else(|| panic!("{needle}:\n{xml}"))
  };
  assert!(at("First late abstract.") < at("Cover line two"), "{xml}");
  assert!(at("Cover line two") < at("Second late abstract."), "{xml}");
  assert!(
    !xml.contains("_Capture_"),
    "no marker reaches the output:\n{xml}"
  );

  let repeat = convert_clean(repro!("late_abstract_repeat_between_cover_lines.tex"));
  assert_eq!(root_order(&repeat), [
    "title",
    "creator",
    "titlepage[block: Cover line one|block: Cover line two]",
    "abstract",
    "para",
    "abstract",
    "section",
  ]);
  let at = |needle: &str| {
    repeat
      .find(needle)
      .unwrap_or_else(|| panic!("{needle}:\n{repeat}"))
  };
  assert_eq!(repeat.matches("Same late abstract.").count(), 1, "{repeat}");
  assert!(at("Cover line two") < at("Same late abstract."), "{repeat}");
  assert!(
    at("Same late abstract.") < at("Cover line three"),
    "{repeat}"
  );
  assert!(
    at("Cover line three") < at("Other late abstract."),
    "{repeat}"
  );

  let empty = convert_clean(repro!(
    "late_empty_abstract_then_real_after_cover_lines.tex"
  ));
  assert_eq!(root_order(&empty), [
    "title",
    "creator",
    "titlepage[block: Cover line one|block: Cover line two]",
    "abstract",
    "section",
  ]);
  assert_element(
    &empty,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Real late abstract.</p></abstract>"#,
  );
  for xml in [&repeat, &empty] {
    assert!(
      !xml.contains("_Capture_") && !xml.contains("_late"),
      "no marker reaches the output:\n{xml}"
    );
  }

  // (three and four late abstracts — each move carries the later ones waiting right after it, and each of those then
  // scans from right after the one before it, review probe o2d; smfart's `{altabstract}`s around its `{abstract}`, o6;
  // an empty abstract's marker, which places nothing, not the markerless abstract a later command adds:
  // `take_late_mark`; r4: a carried abstract whose own move is refused, and a markerless one, stay right after the one
  // before — r3 had printed them ahead of it, review probes q1, q2, q9, q10)
  let cases: [(&str, &[&str], &[&str]); 8] = [
    (
      repro!("three_late_abstracts_each_after_its_cover_line.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "para",
        "abstract",
        "para",
        "abstract",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Cover two",
        "Second late words.",
        "Cover three",
        "Third late words.",
        "Intro",
      ],
    ),
    (
      repro!("four_late_abstracts_each_after_its_cover_line.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "para",
        "abstract",
        "para",
        "abstract",
        "para",
        "abstract",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Cover two",
        "Second late words.",
        "Cover three",
        "Third late words.",
        "Cover four",
        "Fourth late words.",
        "Intro",
      ],
    ),
    (
      repro!("smfart_altabstract_abstract_altabstract_after_cover_lines.tex"),
      // (smfart's title page prints the "par" line too; its page breaks and empty paragraph are passed over)
      &[
        "title",
        "creator",
        "titlepage[block: par|block: Cover one]",
        "abstract",
        "pagination",
        "para",
        "pagination",
        "para",
        "abstract",
        "para",
        "abstract",
        "section",
      ],
      &[
        "Cover one",
        "Alt first words.",
        "Cover two",
        "Main second words.",
        "Cover three",
        "Alt third words.",
        "Intro text.",
      ],
    ),
    (
      repro!("late_empty_abstract_then_markerless_abstract.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "abstract",
        "para",
        "para",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Markerless words.",
        "Cover two",
        "Cover three",
        "Intro",
      ],
    ),
    (
      repro!("late_abstracts_prose_after_the_second_keep_source_order.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "para",
        "abstract",
        "abstract",
        "para",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Cover two",
        "Second late words.",
        "Third late words.",
        "Plain running prose",
        "Intro",
      ],
    ),
    (
      repro!("late_abstracts_float_after_the_second_keep_source_order.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "para",
        "abstract",
        "abstract",
        "figure",
        "para",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Cover two",
        "Second late words.",
        "Third late words.",
        "Fig box",
        "Cover three",
        "Intro",
      ],
    ),
    (
      repro!("late_abstracts_markerless_after_a_sealed_one_keep_source_order.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "para",
        "abstract",
        "abstract",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Cover two",
        "Second late words.",
        "Markerless words.",
        "Intro",
        "Supplement words.",
      ],
    ),
    (
      repro!("four_late_abstracts_prose_after_the_second_keep_source_order.tex"),
      &[
        "title",
        "creator",
        "titlepage[block: Cover one]",
        "abstract",
        "para",
        "abstract",
        "abstract",
        "abstract",
        "para",
        "para",
        "para",
        "section",
      ],
      &[
        "Cover one",
        "First late words.",
        "Cover two",
        "Second late words.",
        "Third late words.",
        "Fourth late words.",
        "Plain running prose",
        "Cover three",
        "Cover four",
        "Intro",
      ],
    ),
  ];
  for (tex, order, words) in cases {
    let xml = convert_clean(tex);
    assert_eq!(root_order(&xml), order, "{xml}");
    let at: Vec<usize> = words
      .iter()
      .map(|w| xml.find(w).unwrap_or_else(|| panic!("{w}:\n{xml}")))
      .collect();
    assert!(at.is_sorted(), "{words:?} out of print order:\n{xml}");
    assert_eq!(
      xml.matches("<abstract ").count(),
      order.iter().filter(|e| **e == "abstract").count(),
      "{xml}"
    );
    assert!(
      !xml.contains("_Capture_") && !xml.contains("_late"),
      "no marker reaches the output:\n{xml}"
    );
  }
}
