//! Guards for batch P3 (Option C P3, OD #993): a name printed inside a macro call's expansion — ptephy's `\name{…}{marks}`,
//! a document's `\nm{…}` — is written as the call whole, which digests to the name and its marks as the block did, never
//! as its plain text ("a name its events don't spell") nor from inside the call with the marks argument as the name's
//! text ("Bob Baker2"). Each repro compiles clean with pdflatex (TL 2025; ptephy with 1304.2086's ptephy.cls, OmniBus's
//! `nosuchclass` as `article`); each row asserts every creator's personname element and contacts, in order, and the
//! exact Error and Warning counts. One test, one conversion per row: the rows are the shapes of one code path
//! (`emit::block_names`'s run of the call, `emit::made_by_its_call`), each in a context of its own (the list's first
//! call, its last call glued to the block's end, a cite inside the call, a font macro) and the wrapper's mark the rule
//! must leave apart (2606.08894's shape, which a draft of P3 wrote into 20 papers' names).

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/author_one_block/",
      $name,
      ".tex"
    ))
  };
}

/// Each `<personname>` element of `xml`, in order, its `xml:id`s dropped.
fn personnames(xml: &str) -> Vec<String> {
  xml
    .match_indices("<personname>")
    .map(|(start, _)| {
      let end = start
        + xml[start..]
          .find("</personname>")
          .expect("closed personname");
      let mut element = xml[start..end + "</personname>".len()].to_string();
      while let Some(at) = element.find(" xml:id=\"") {
        let close =
          at + " xml:id=\"".len() + element[at + " xml:id=\"".len()..].find('"').expect("id");
        element.replace_range(at..=close, "");
      }
      element
    })
    .collect()
}

/// A name a macro call prints is written as the call whole: its personname element as the call digests it, no plain
/// text Warning, no marks argument in the name; a wrapper's mark after the name stays apart. Red on int159a: the first
/// call's name by text (W 1, "Yoshiharu Kawamura", its ∗ lost), the last call's marks argument as text ("Bob Baker2",
/// "Ann Able…1").
#[test]
fn a_name_inside_a_macro_call_is_written_from_the_call() {
  let cite = concat!(
    r#"<personname>Ann Able<cite class="ltx_citemacro_cite">[<bibref bibrefs="y" separator="," "#,
    r#"yyseparator=","/>]</cite></personname>"#
  );
  for (name, tex, counts, names, contacts) in [
    (
      "first and last call",
      repro!("ptephy_name_inside_its_call_is_written_from_the_call"),
      (0, 0),
      vec![
        "<personname>Yoshiharu Kawamura<sup>∗</sup></personname>",
        "<personname>Bob Baker</personname>",
      ],
      vec![
        vec!["Yoshiharu Kawamura ∗", "affiliation=1 Univ A, Town"],
        vec!["Bob Baker", "affiliation=2 Univ B, City"],
      ],
    ),
    (
      "last call glued",
      repro!("ptephy_author_list_glued_trailing_mark_leaves_the_name"),
      (0, 0),
      vec![
        "<personname>Ann Able</personname>",
        "<personname>Bob Baker</personname>",
      ],
      vec![vec!["Ann Able", "affiliation=Univ One"], vec![
        "Bob Baker",
        "affiliation=Univ Two",
      ]],
    ),
    (
      "cite inside the call",
      repro!("ptephy_name_and_cite_inside_its_call_is_written_from_the_call"),
      (0, 0),
      vec![cite],
      vec![vec!["Ann Able [ ]", "affiliation=Univ X"]],
    ),
    (
      "wrapper mark stays apart",
      repro!("article_author_wrapper_mark_after_the_name_stays_apart"),
      (0, 0),
      vec![
        "<personname>Ann Able</personname>",
        "<personname>Bob Baker</personname>",
      ],
      vec![vec!["Ann Able", "affiliation=Univ X"], vec![
        "Bob Baker",
        "affiliation=Univ Y",
      ]],
    ),
    (
      "font macro",
      repro!("article_author_name_macro_with_a_font_keeps_it"),
      (0, 0),
      vec![r#"<personname><text font="smallcaps">Ann Able</text></personname>"#],
      vec![vec!["Ann Able"]],
    ),
  ] {
    let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      counts,
      "{name}: {log}"
    );
    assert_eq!(personnames(&xml), names, "{name}");
    let contacts: Vec<Vec<String>> = contacts
      .iter()
      .map(|row| row.iter().map(ToString::to_string).collect())
      .collect();
    assert_eq!(creators(&xml), contacts, "{name}");
  }
}
