//! Guards for NE2 (`agent_reports/2026-11-09_NE2_report.md`): Errors `LATEXML_FM_EMIT=2` raised that off doesn't, the
//! flip's no-harm gate. One guard per mechanism, each off and at `=2`: the footnote counter's steps of an author block's
//! notes run where they print (1507.05018, 2505.07115); an IJCAI sectioned block goes to its section split, its
//! `labelseq=author` affiliation the N-th author's (2202.05511); initials a gap joins to a name end no surname-first name
//! (cond-mat9905290); "representing" speaks for a collaboration (hep-ex9903035). A creator's labels written back as plain
//! characters (1910.05766) is the unit test `writer::emit::tests::a_creators_labels_are_plain_characters`. NE2b (review
//! revNE2): a counter move no name carries writes the block whole, with its Warning (1507.05018's Ombao, A1); a note
//! before every name steps before the first name (A2).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz3::in_a_directory};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// Convert `tex` under `emit`: the log and the XML.
fn run(tex: &str, emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(None);
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// A note's footnote step is its creator's, where it prints: `=2` wrote the notes apart, after every creator, so Bob
/// Baker's `\addtocounter{footnote}{-2}\footnotemark` ran at the counter's 0 (5 x "Counter too large", the mark empty);
/// now his mark is the "*" pdflatex prints, as off's.
#[test]
fn a_notes_footnote_step_runs_where_it_prints() {
  for emit in [0, 2] {
    let (log, xml) = run(repro!("ne2_thanks_addtocounter_order"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(
      creators(&xml),
      [row(&["Ann Able"]), row(&["Bob Baker * * footnote *"])],
      "emit={emit}"
    );
    let notes: Vec<&str> = xml
      .match_indices("<note ")
      .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
      .collect();
    assert_eq!(
      notes,
      [
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1""#,
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2""#,
        r#"<note mark="*" role="footnotemark" xml:id="footnote1""#,
      ],
      "emit={emit}: {xml}"
    );
  }
}

/// kr.sty's `\affiliations` and `\emails` are defined only in its `\@maketitle`: `=2` queued the block whole before the
/// IJCAI split, and its piece `Bob Baker \affiliations Univ X` was digested (undefined `\affiliations`). Now the split
/// runs first, as off. Its `labelseq=author` affiliation is Perl's `author:1`, Ann Able's, off; `=2` reads it as the
/// page prints it, under both names (2202.05511 alike).
#[test]
fn an_ijcai_block_is_split_at_its_sections() {
  let sty = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ne2_kr_affiliations.sty"
  );
  let univ = "affiliation=Univ X, City, Country";
  for (emit, bob) in [
    ("0", row(&["Bob Baker", "email=bob@x.edu"])),
    ("2", row(&["Bob Baker", univ, "email=bob@x.edu"])),
  ] {
    let expected = [row(&["Ann Able", univ, "email=ann@x.edu"]), bob];
    let (log, xml) = in_a_directory(
      repro!("ne2_kr_affiliations"),
      &[("ne2_kr_affiliations.sty", sty)],
      &[("LATEXML_FM_EMIT", emit)],
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creators(&xml), expected, "emit={emit}");
  }
}

/// `F. M.\, Munteanu` after a comma: the surname-first rule (`Santucho, V.`) joined the initials to "Yongmin Kim" and the
/// thin space's gap rule joined "Munteanu" after them, one creator (merged_creators under `=2`). Initials a gap joins to a
/// name end no piece: four creators, each with what its marks request — off too, whose † affiliation went to a nameless
/// creator.
#[test]
fn initials_a_thin_space_joins_are_a_name_of_their_own() {
  let lab = "affiliation=∗ Lab A, Los Alamos";
  let dept = "affiliation=† Dept B, Boston";
  let expected = [
    row(&["Yongmin Kim *", lab]),
    row(&["F. M. Munteanu ∗,†", lab, dept]),
    row(&["C. H. Perry ∗,†", lab, dept]),
    row(&["D. G. Rickel *", lab]),
  ];
  for emit in [0, 2] {
    let (log, xml) = run(repro!("ne2_merged_kim"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creators(&xml), expected, "emit={emit}");
  }
}

/// `\author{representing the ATLAS Pixel Collaboration}`, a call of its own: the names' collaboration, as off frames it
/// ("representing" in `SPEAKS_FOR`), not a creator (creator_not_a_name under `=2`).
#[test]
fn representing_a_collaboration_is_its_contact() {
  let expected = [row(&[
    "John Richardson",
    "affiliation=Lawrence Berkeley Laboratory",
    "collaboration=representing the ATLAS Pixel Collaboration",
  ])];
  for emit in [0, 2] {
    let (log, xml) = run(repro!("ne2_representing_collab"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creators(&xml), expected, "emit={emit}");
  }
}

/// An author block whose digestion moves the footnote counter where no name's run or mark carries it (Bob Baker's
/// `\addtocounter{footnote}{-2}` between his notes, 1507.05018's Ombao): `=2` lost the move and Ann Able's
/// `\footnotemark` after her note, so Carl Cole printed † where LaTeX prints *, with no Warning (review revNE2 A1). The
/// block is written whole, Perl's whole-block creator, its tokens digested in order: Ann †, Carl *, every note kept, the
/// counted Warning. Off reads it per name, as pdflatex prints the marks. (A move a mark's box covers,
/// `\addtocounter{footnote}{-2}\footnotemark`, is carried: `a_notes_footnote_step_runs_where_it_prints`.)
#[test]
fn a_counter_move_no_name_carries_writes_the_block_whole() {
  for (emit, expected, warnings) in [
    (
      0,
      vec![
        row(&["Ann Able † † footnote †"]),
        row(&["Bob Baker"]),
        row(&["Carl Cole * * footnote *"]),
      ],
      0,
    ),
    (
      2,
      vec![row(&[
        "Ann Able † † footnote † and Bob Baker and Carl Cole * * footnote *",
      ])],
      1,
    ),
  ] {
    let (log, xml) = run(repro!("ne2b_counter_moved_between_notes"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "emit={emit}: {log}"
    );
    assert_eq!(creators(&xml), expected, "emit={emit}");
    let notes: Vec<&str> = xml
      .match_indices("<note ")
      .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
      .collect();
    // (every note kept, each mark as pdflatex prints it: Ann's footnotemark †, Carl's *; the block written whole holds both
    // marks in its one creator, before the thanks notes)
    let (mark_ann, mark_carl) = (
      r#"<note mark="†" role="footnotemark" xml:id="footnote2""#,
      r#"<note mark="*" role="footnotemark" xml:id="footnote1""#,
    );
    let thanks = |n: u8| {
      format!(r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id{n}""#)
    };
    let mut expected = vec![mark_ann.to_string(), thanks(1), thanks(2), thanks(3)];
    expected.insert(if emit == 0 { 4 } else { 1 }, mark_carl.to_string());
    assert_eq!(notes, expected, "emit={emit}: {xml}");
  }
}

/// A note before every name steps the counter before the first creator's name, where LaTeX reads it: `=2` stepped it
/// after Ann Able's personname, so her `\footnotemark` printed * where pdflatex prints † (review revNE2 A2). Distinct
/// from `a_notes_footnote_step_runs_where_it_prints`: the step before a name, not after one.
#[test]
fn a_note_before_every_name_steps_before_it() {
  for emit in [0, 2] {
    let (log, xml) = run(repro!("ne2b_note_before_every_name"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(
      creators(&xml),
      [row(&["Ann Able † † footnote †"]), row(&["Bob Baker"])],
      "emit={emit}"
    );
  }
}
