//! Red/green guards for batch DATEICML: icml's `\printAffiliationsAndNotice` legends (`\icmlEqualContribution`'s `*`,
//! icml2026.sty:502, :519-536) are the notes of the names whose `\icmlsetsymbol` mark they open with (FIDELITY_CHARTER §4,
//! the mark rule; OXIDIZED_DESIGN_DIVERGENCES #1061). Code paths: the name's mark keyed by its label
//! (`\lx@frontmatter@namemark`) and the notice cut into labelled legend entries (icml_support_sty.rs `legend_entries`),
//! which the Kind step for a legend makes the names' notes, a declared legend whatever its words (insert.rs
//! `kind_legend_contacts`, `LEGEND_ENTRY`). A notice with a piece no name answers stays whole:
//! `perfect_kernel_batchinvdag::an_unmarked_document_level_note_is_a_block_line`. Witnesses 2602.04557, 1904.07793,
//! 2402.05011, 2310.02777.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $name,
      ".tex"
    ))
  };
}

fn folded(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Each starred name holds its legend as a note marked with its symbol, no mark-only note beside it and no block notice
/// left; the affiliations and Ann's e-mail as before (Cy Able shares Ann's surname, which `\icmlcorrespondingauthor`
/// spells: 1904.07793). Rows: the one legend (2602.04557); two legends cut at their marks, the comma between them
/// neither's, `$\dagger$` and `\dag` one mark, the second a declared legend whose words name an institution (2402.05011,
/// 2310.02777). Red on INT081RC6: mark-only notes and a top-level `affiliationnotice`.
#[test]
fn icml_notice_legends_are_the_marked_names_notes() {
  let equal = |id: &str| {
    format!(
      r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="{id}">Equal contribution</note>"#
    )
  };
  let affiliation = r#"<contact name="Affiliation: " role="affiliation">Univ A</contact>"#;
  let email = r#"<contact name="Correspondence to: " role="email">ann@a.edu</contact>"#;
  let intern = |id: &str| {
    format!(
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" mark="†" role="thanks" xml:id="{id}">Interns at Acme Corp</note>"#
    )
  };
  let rows: [(&str, &str, Vec<String>); 2] = [
    (
      "one legend",
      repro!("icml_equal_contribution_notice_is_the_starred_names_note"),
      vec![
        format!(
          r#"<creator role="author"> <personname>Ann Able</personname> {affiliation} {email} {} </creator>"#,
          equal("id1")
        ),
        format!(
          r#"<creator before=" " role="author"> <personname>Bob Baker</personname> {affiliation} {} </creator>"#,
          equal("id2")
        ),
        format!(
          r#"<creator before=" " role="author"> <personname>Cy Able</personname> {affiliation} </creator>"#
        ),
      ],
    ),
    (
      "two legends",
      repro!("icml_notice_two_legends_each_its_names_note"),
      vec![
        format!(
          r#"<creator role="author"> <personname>Ann Able</personname> {affiliation} {email} {} </creator>"#,
          equal("id1")
        ),
        format!(
          r#"<creator before=" " role="author"> <personname>Bob Baker</personname> {affiliation} {} {} </creator>"#,
          equal("id2"),
          intern("id3")
        ),
        format!(
          r#"<creator before=" " role="author"> <personname>Cat Cole</personname> {affiliation} {} </creator>"#,
          intern("id4")
        ),
      ],
    ),
  ];
  for (row, tex, expected) in rows {
    let xml = convert_clean(tex);
    let creators: Vec<String> = elements(&xml, "creator")
      .iter()
      .map(|c| folded(c))
      .collect();
    assert_eq!(creators, expected, "{row}");
    assert!(!xml.contains("affiliationnotice"), "{row}: {xml}");
    assert!(!xml.contains("footnotemark"), "{row}: {xml}");
  }
}
