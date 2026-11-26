//! Red/green guards for batch DATEICML: icml's `\printAffiliationsAndNotice` legends (`\icmlEqualContribution`'s `*`,
//! icml2026.sty:502, :519-536) are the notes of the names whose `\icmlsetsymbol` mark they open with (FIDELITY_CHARTER §4,
//! the mark rule; OXIDIZED_DESIGN_DIVERGENCES #1061). Code paths: the name's mark keyed by its label
//! (`\lx@frontmatter@namemark`) and the notice cut into labelled legend entries (icml_support_sty.rs `legend_entries`),
//! which the Kind step for a legend makes the names' notes, a declared legend whatever its words (insert.rs
//! `kind_legend_contacts`, `LEGEND_ENTRY`). A notice with a piece no name answers stays whole:
//! `perfect_kernel_batchinvdag::an_unmarked_document_level_note_is_a_block_line`; so does one holding a conditional, an
//! assignment or a note, and a notice printed twice gives each name its legend once (`icml_notice_read_by_meaning`,
//! review revDATEICML). Witnesses 2602.04557, 1904.07793, 2402.05011, 2310.02777, 2306.03504.
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

/// The note opening with `open` (`<note role="affiliationnotice"`), whole with the notes it holds, its whitespace
/// folded; `None` if absent.
fn note_at(xml: &str, open: &str) -> Option<String> {
  let start = xml.find(open)?;
  let mut depth = 0usize;
  let mut at = start;
  loop {
    let next_open = xml[at..].find("<note").map(|i| at + i);
    let next_close = at + xml[at..].find("</note>")?;
    match next_open {
      Some(i) if i < next_close => {
        let end = i + xml[i..].find('>')?;
        depth += usize::from(!xml[..end].ends_with('/'));
        at = end + 1;
      },
      _ => {
        depth -= 1;
        at = next_close + "</note>".len();
        if depth == 0 {
          return Some(folded(&xml[start..at]));
        }
      },
    }
  }
}

/// Review revDATEICML: a notice is cut into legends only where reading it as written gives each name its mark's
/// legend; else it stays whole, the block's note, every word as INT081RC6 keeps it. Rows (red on DATEICML
/// 4e31feeb990): an assignment (`\let\thefootnote\relax\footnotetext`: "Code at X." lost), a conditional
/// (`\ifx\icmlEqualContribution`: `Error: \undefined`), a definition (`\def\x{Z}\x`: an Error, the wrong text), a
/// footnote (copied to each name, 2 Warnings), the notice printed twice (two notes per name), a `\#` symbol's legend
/// (2306.03504: left whole by the TeX-special filter, now the name's note). Re-review rerevDATEICML (red on DATEICML
/// c759b427346, the second legend inside the first, each `*` name's note): a marker after `\quad`, `\\` or `\protect`
/// (not expanded after a control sequence), a mark in a group and a math mark stay whole; two markers back to back
/// stay two legends (2202.03884). Ann's and Bob's elements whole, in order.
#[test]
fn icml_notice_read_by_meaning() {
  let ann_mark = r#"<creator role="author"> <personname>Ann Able</personname> <note mark="*" role="footnotemark" xml:id="id1"/> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Correspondence to: " role="email">ann@a.edu</contact> </creator>"#;
  // (Bob's two symbols one joined mark `*,†`, where icml prints two superscripts: RED
  // icml_notice_whole_two_symbols_one_joined_mark_red, post-merge)
  let bob_marks = r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <note mark="*,†" role="footnotemark" xml:id="id2"/> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator>"#;
  let ann_note = r#"<creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <contact name="Correspondence to: " role="email">ann@a.edu</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id1">Equal contribution</note> </creator>"#;
  let whole = |text: &str| {
    Some(format!(
      r#"<note role="affiliationnotice" xml:id="id3"><sup xml:id="id3.1">*</sup>{text}</note>"#
    ))
  };
  let rows: [(&str, &str, [&str; 2], Option<String>); 12] = [
    (
      "assignment",
      repro!("icml_notice_with_an_assignment_stays_whole"),
      [ann_mark, bob_marks],
      whole(
        r#"Equal contribution <note role="footnotetext" xml:id="footnotex1">Code at X.</note>"#,
      ),
    ),
    (
      "conditional",
      repro!("icml_notice_with_a_conditional_stays_whole"),
      [ann_mark, bob_marks],
      whole("Equal contribution DEF"),
    ),
    (
      "definition",
      repro!("icml_notice_with_a_definition_stays_whole"),
      [ann_mark, bob_marks],
      whole(r#"Equal contribution Z <sup xml:id="id3.2">†</sup>Lead"#),
    ),
    (
      "footnote",
      repro!("icml_notice_with_a_footnote_stays_whole"),
      [ann_mark, bob_marks],
      whole("Equal contribution "),
    ),
    (
      "printed twice",
      repro!("icml_notice_printed_twice_one_note_per_name"),
      [
        ann_note,
        r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <note mark="†" role="footnotemark" xml:id="id2"/> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id3">Equal contribution</note> </creator>"#,
      ],
      None,
    ),
    (
      "a TeX special symbol",
      repro!("icml_notice_legend_of_a_tex_special_symbol"),
      [
        ann_note,
        r##"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id2">Equal contribution</note> <note class="ltx_note_frontmatter ltx_thanks_note" mark="#" role="thanks" xml:id="id3">Interns at Acme</note> </creator>"##,
      ],
      None,
    ),
    (
      "a marker after \\quad",
      repro!("icml_notice_marker_after_quad_stays_whole"),
      [ann_mark, bob_marks],
      whole(r#"Equal contribution <sup xml:id="id3.2">†</sup>Project lead"#),
    ),
    (
      "a marker after \\\\",
      repro!("icml_notice_marker_after_a_line_break_stays_whole"),
      [ann_mark, bob_marks],
      whole(r#"Equal contribution <break/><sup xml:id="id3.2">†</sup>Project lead"#),
    ),
    (
      "a marker after \\protect",
      repro!("icml_notice_marker_after_protect_stays_whole"),
      [ann_mark, bob_marks],
      whole(r#"Equal contribution <sup xml:id="id3.2">†</sup>Project lead"#),
    ),
    (
      "a mark in a group",
      repro!("icml_notice_mark_in_a_group_stays_whole"),
      [ann_mark, bob_marks],
      whole(
        r#"Equal contribution <sup xml:id="id3.2"><text fontsize="90%" xml:id="id3.2.1">†</text></sup><text fontsize="90%" xml:id="id3.3">Project lead</text>"#,
      ),
    ),
    (
      "a math mark inside a legend",
      repro!("icml_notice_math_mark_inside_a_legend_stays_whole"),
      [ann_mark, bob_marks],
      whole(r#"Equal contribution <sup xml:id="id3.2">†</sup>Project lead"#),
    ),
    (
      "markers back to back",
      repro!("icml_notice_back_to_back_markers_each_a_legend"),
      [
        ann_note,
        r#"<creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id2">Equal contribution</note> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks" xml:id="id3">Project lead</note> </creator>"#,
      ],
      None,
    ),
  ];
  for (row, tex, expected, notice) in rows {
    let xml = convert_clean(tex);
    let creators: Vec<String> = elements(&xml, "creator")
      .iter()
      .map(|c| folded(c))
      .collect();
    assert_eq!(creators, expected, "{row}");
    assert_eq!(
      note_at(&xml, r#"<note role="affiliationnotice""#),
      notice,
      "{row}"
    );
    // (the footnote's words once, where INT081RC6 keeps them)
    if row == "footnote" {
      assert_eq!(xml.matches("inner fn").count(), 1, "{xml}");
      assert!(xml.contains("<pubnote>inner fn</pubnote>"), "{xml}");
    }
  }
}
