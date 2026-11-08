//! Guards for NL1, the names line of the one engine's reading ported from the old reader's procedure
//! (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; the rule table RT3, rows N1-N3, C2-C5, S1, A2): each fixture
//! converted with the emitter's switch off and at `LATEXML_FM_EMIT=2`, the creators (name and every contact, in order)
//! the same both ways. One guard per rule: a marked name (N1-N3), glued initials (S1), a collaboration remark (C2-C5),
//! a comma a command draws inside a word (A2).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` with `files` beside it converted under `emit`: the log and the XML.
fn run_with(tex: &str, files: &[(&str, &str)], emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    files,
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(Some(true));
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// `tex`'s creators off and at `=2`, each run clean (`warnings` each way).
fn both(tex: &str, warnings: (usize, usize)) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  both_with(tex, &[], warnings)
}

/// [`both`] with `files` beside `tex`.
fn both_with(
  tex: &str,
  files: &[(&str, &str)],
  warnings: (usize, usize),
) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
  let (off_log, off) = if files.is_empty() {
    run(tex, 0)
  } else {
    run_with(tex, files, 0)
  };
  let (log, xml) = if files.is_empty() {
    run(tex, 2)
  } else {
    run_with(tex, files, 2)
  };
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, warnings.0),
    "{off_log}"
  );
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings.1),
    "{log}"
  );
  (creators(&off), creators(&xml))
}

/// The `collaboration` contacts of `xml`, each element whole, its `xml:id`s dropped: their text and markup as written.
fn collaborations(xml: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find("<contact role=\"collaboration\"") {
    let end = rest[at..].find("</contact>").expect("a closed contact") + at + "</contact>".len();
    let mut element = rest[at..end].to_string();
    while let Some(id) = element.find(" xml:id=\"") {
      let close = element[id + 9..].find('"').expect("a closed id") + id + 10;
      element.replace_range(id..close, "");
    }
    out.push(element);
    rest = &rest[end..];
  }
  out
}

/// Rows N1-N3: a names line's piece carrying its own mark is a name whatever its words — the generic split never reads
/// an institution word (`R. New$^2$`, 0904.4122; `S. Michelle Consiglio,$^{1}$`, 2103.02641; `Alexander New$^2$`,
/// 2311.16860), and a per-author class's line a surname after an initial (`Gary P. Centers,$^{1}$`, revtex,
/// 1901.10843). Red (MV1 `=2`): each the name before's affiliation. An unmarked piece keeps the reading's institution
/// (`John Smith, University of Toronto`, the main session's decision 2026-11-08), off a creator of its own.
#[test]
fn a_marked_name_with_an_institution_word_is_a_name() {
  let (off, two) = both(
    repro!("emit_a_marked_name_with_an_institution_word_is_a_name"),
    (0, 0),
  );
  let want = [
    vec!["A. M. Broomhall", "affiliation=Univ A"],
    vec!["S. Michelle Consiglio", "affiliation=Univ A"],
    vec!["R. New", "affiliation=Univ B"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let (off, two) = both(
    repro!("revtex_marked_list_surname_after_initial_is_a_name"),
    (0, 0),
  );
  let alpha = "affiliation=University of Alpha";
  let beta = "affiliation=University of Beta";
  assert_eq!(two, [
    vec!["Ann Able", alpha],
    vec!["Bob Baker", alpha],
    vec!["Gary P. Centers", alpha],
    vec!["Dan Dee", beta],
    vec!["Eve Eden 1,2", alpha, beta],
  ]);
  assert_eq!(off.len(), 5);
  let (off, two) = both(
    "\\documentclass{article}\n\\begin{document}\n\\title{T}\n\\author{Ann Able$^1$, University of \
     Toronto\\\\ $^1$Univ A}\n\\maketitle\nx\n\\end{document}\n",
    (0, 0),
  );
  assert_eq!(off, [vec!["Ann Able", "affiliation=Univ A"], vec![
    "University of Toronto"
  ],]);
  assert_eq!(two, [vec![
    "Ann Able",
    "affiliation=University of Toronto",
    "affiliation=Univ A"
  ]]);
}

/// Row S1 (a rule of the reading alone): initials after a comma join the name before only after a bare surname; a word
/// of glued initials and a surname is read through its initials (`unglued_initials`, as `name_shaped` reads it):
/// `P.A.Vshivtseva$^{a}$, E.T.\, Einiev` is two names (2409.13569). Red (MV1 `=2`): one creator, `merged_creators`.
#[test]
fn glued_initials_are_no_surname_before_initials() {
  let (off, two) = both(
    repro!("emit_glued_initials_are_no_surname_before_initials"),
    (0, 0),
  );
  let want = [
    vec!["M.O. Astashenkov", "affiliation=Univ A"],
    vec!["P.A.Vshivtseva", "affiliation=Univ A"],
    vec!["E.T. Einiev", "affiliation=Univ B"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
}

/// Rows C2-C5: a name's collaboration remark is the name's `collaboration` contact, no part of the name
/// (FIDELITY_CHARTER §4, OXIDIZED_DESIGN_DIVERGENCES #946), read whole where a note, a note reference, a box or a
/// command's own spacing parts it into pieces (C3, C4: `Kalanand Mishra\footnote{…} (for the \babar\ collaboration)`,
/// 0711.1544; elsarticle's `Ann Able (for the XYZ\fnref{c1} Collaboration)`, 1212.4278, 1408.0414), after the name's
/// notes when it names whom the author speaks for (C2: webofc's `for CAPP/IBS`, 1702.03664), and as a line of its own
/// under one name (C5: `Carolyn Brown$^1$\\ on behalf of the Bcool project team`, 1310.6507, sourced at the line's
/// ink: its `\footnotesize`, 1502.03246). Red (MV1 `=2`): each in the personname, or "Collaboration)" the contact; red
/// (NL1 `=2`, review revNL1 B1, S3): `\babar`'s remark "(for the B A B A R collaboration)" in plain text, the C5 line's
/// size lost. The remark after the name's note is a run of its own, the text its pieces print joined as the source joins
/// them (`lines::printed_join`; `\babar`'s kerns back up, no space), so it is written from its tokens in its markup, as
/// off writes it. A name whose own event opens the remark (`Ann Able (for the XYZ`) joins with it: the contact is
/// written from its text at `=2`, a Warning each for it and the name (the joined event's span holds both).
#[test]
fn a_collaboration_remark_is_the_names_contact() {
  let tex = repro!("emit_a_collaboration_remark_parted_by_a_note_and_a_box_is_a_contact");
  let (off, two) = both(tex, (0, 0));
  let want = [vec![
    "Kalanand Mishra",
    "note=Email: k@x.edu",
    "collaboration=(for the B A B AR collaboration)",
    "affiliation=Univ A",
  ]];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let babar = "<contact role=\"collaboration\">(for the <text font=\"slanted\">B<text fontsize=\"90%\">A</text>B<text \
               fontsize=\"90%\">AR</text></text> collaboration)</contact>";
  assert_eq!(collaborations(&run(tex, 0).1), [babar]);
  assert_eq!(collaborations(&run(tex, 2).1), [babar]);
  let (off, two) = both(
    repro!("elsarticle_collaboration_remark_note_reference_is_the_names"),
    (0, 2),
  );
  let want = [vec![
    "Ann Able",
    "collaboration=(for the XYZ Collaboration)",
    "address=Univ X",
    "note=A list of members is at the end.",
  ]];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let (off, two) = both_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc_remark_after_the_notes_is_a_contact.tex"
    ),
    &[(
      "webofc.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/webofc/webofc.cls"),
    )],
    (0, 0),
  );
  assert_eq!(off, [vec![
    "Ann Able",
    "affiliation=Univ A, Korea",
    "email=ann@a.edu",
    "collaboration=for CAPP/IBS"
  ]]);
  assert_eq!(two, [vec![
    "Ann Able",
    "email=ann@a.edu",
    "collaboration=for CAPP/IBS",
    "affiliation=Univ A, Korea"
  ]]);
  let (off, two) = both(
    repro!("emit_a_line_speaking_for_a_collaboration_is_a_contact"),
    (0, 0),
  );
  let want = [
    vec!["Ann Able", "affiliation=Univ A"],
    vec!["Bob Baker", "affiliation=Univ A"],
    vec![
      "Carolyn Brown",
      "collaboration=on behalf of the Bcool project team",
      "affiliation=Univ A",
    ],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let bcool = "<contact role=\"collaboration\"><text fontsize=\"80%\">on behalf of the Bcool project \
               team</text></contact>";
  let tex = repro!("emit_a_line_speaking_for_a_collaboration_is_a_contact");
  assert_eq!(collaborations(&run(tex, 0).1), [bcool]);
  assert_eq!(collaborations(&run(tex, 2).1), [bcool]);
}

/// Rows C2-C3 (review revNL1 S1): a remark after the name's note opens a run of its own (`lines::remark_start`), so the
/// name keeps its own event and the remark a span of its own: `Ann Able\thanks{…} (for the \textsc{Atlas}
/// Collaboration)` is written as off writes it, the note before the collaboration, its small capitals kept. Red (NL1
/// `=2`): the name, the note and the remark joined, the contact's text plain before the note, a Warning (probes c2e-c2g;
/// 1201.5130's `(on behalf of the D\O\ and CDF collaborations)` a false Warning).
#[test]
fn a_remark_after_a_note_keeps_its_own_span() {
  let tex = repro!("emit_a_remark_after_a_note_keeps_its_own_span");
  let (off, two) = both(tex, (0, 0));
  let want = [vec![
    "Ann Able",
    "collaboration=(for the Atlas Collaboration)",
  ]];
  assert_eq!(off, want);
  assert_eq!(two, want);
  let atlas = "<contact role=\"collaboration\">(for the <text font=\"smallcaps\">Atlas</text> Collaboration)</contact>";
  for emit in [0, 2] {
    let xml = run(tex, emit).1;
    assert_eq!(collaborations(&xml), [atlas]);
    // (her thanks note before the collaboration, as the source prints them)
    let note = xml.find("<note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\"");
    assert!(
      note.is_some_and(|note| xml
        .find("<contact role=\"collaboration\"")
        .is_some_and(|c| note < c)),
      "{xml}"
    );
  }
}

/// Row N5 of the old reader (`add_authors_calls_sectioned`: an unmarked line under a labelled entry is appended to it),
/// behind the line above holding one labelled entry (the main session's decision K, 2026-11-08): `$^1$The Graduate
/// Center, CUNY\\ Computer Science\\ New York, USA` is one affiliation (2501.04046; hep-ph0511130). Red (MV1, NL1
/// `=2`): "Computer Science" a creator. Under two entries a line of names stays a names row (SYNC row 46,
/// `perfect_kernel_batchfms2a2::sync_row_shapes_46_53_66`'s `Cat Cole`; cs0011034 a residual).
#[test]
fn a_line_under_one_labelled_entry_continues_it() {
  let (off, two) = both(
    repro!("emit_a_line_under_one_labelled_entry_continues_it"),
    (0, 0),
  );
  let cuny = "affiliation=The Graduate Center, CUNY Computer Science New York, USA";
  let want = [vec!["Ann Able", cuny], vec!["Bob Baker", cuny]];
  assert_eq!(off, want);
  assert_eq!(two, want);
}

/// Row A2: the ink one command draws inside a word is the word's letters — combelow's `\cb{t}` stacks a `t` over a
/// comma in an alignment (combelow.sty:16), so its comma and row break part no names: `Bogdan Rai\cb{t}\u{a}`
/// (2507.13598) is one name, written from its tokens as off writes it. Red (MV1 `=2`): the line every author's
/// affiliation, two creators lost.
#[test]
fn a_comma_a_command_draws_inside_a_name_parts_no_names() {
  let (off, two) = both(
    repro!("emit_a_comma_a_command_draws_inside_a_name_parts_no_names"),
    (0, 0),
  );
  let want = [
    vec!["Ann Able", "affiliation=Univ A"],
    vec!["Bob Baker", "affiliation=Univ A"],
    vec!["Bogdan Rai t , ă", "affiliation=Univ B"],
    vec!["Sarah Adel Bargal", "affiliation=Univ A"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  // (a comma or a row break the command prints after its box, at its own level, parts names as any does — review
  // revNL1 S2: `\newcommand\nlx{\mbox{Jr.},}`; red (NL1): "Ann Able" and "Jr.,Bob Baker", off a `merged_creators`
  // Error)
  let (off, two) = both(
    repro!("emit_a_comma_a_command_prints_after_its_box_parts_names"),
    (0, 0),
  );
  let want = [vec!["Ann Able Jr."], vec!["Bob Baker"]];
  assert_eq!(off, want);
  assert_eq!(two, want);
  // (its row break, `\mbox{Jr.}\\`: "Jr." stays Ann Able's, the line after it the reading's to read — red (NL1): the
  // names "Ann Able" and "Jr. Bob Baker"; off holds both lines in one name, a `merged_creators` Error)
  let (log, xml) = run(
    "\\documentclass{article}\n\\newcommand\\nlx{\\mbox{Jr.}\\\\}\n\\begin{document}\n\\title{T}\n\\author{Ann \
     Able~\\nlx Bob Baker}\n\\maketitle\nx\n\\end{document}\n",
    2,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [vec![
    "Ann Able Jr.",
    "affiliation=Bob Baker"
  ]]);
}
