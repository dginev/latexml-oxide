//! Guards for batch FB2a (`pk/FB2a`, 2026-11-08): at `LATEXML_FM_EMIT=2` the whole-block fallback is Perl's raw call
//! (`frontmatter/fallback.rs` `whole_block_author`, Base_Utility.pool.ltxml:693), and four of the old reader's rules that
//! its raw-token rewrites carried are ported onto the digested reading (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`,
//! the rows "Strict porting" and "The fallback"; OXIDIZED_DESIGN_DIVERGENCES #52, #980). Each guard is a distinct code
//! path: the queued raw call (`a_raw_queued_call_feeds_no_symbol_mark_to_the_reading`), the rehearsal's binding of a
//! tabular juggle (`a_tabular_juggle_is_a_row_break`), the reading's pre-pass of a trailing alignment environment
//! (`a_trailing_alignment_environment_is_lines_under_the_names`), the rehearsal's text-mode note and the emitter's note
//! call in a formula (`a_note_in_a_formula_keeps_its_words`), and the reading's block-shared URL decided at the block's
//! end (`a_url_line_under_every_name_is_the_blocks`). Batch FB2b1 (review revFB2a findings 1, 3): the juggle predicate
//! itself, a macro only closing a tabular (`a_macro_closing_a_tabular_is_no_juggle`), and the block links' institution
//! exception (`a_link_printing_an_institution_is_every_names`). Batch FB2b4: the juggle written inline
//! (`an_inline_tabular_juggle_is_a_column`).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{ENVIRONMENT_SHAPE, creators_xml, environment_row, run},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` converted off and at `=2`, both clean: their creators' XML.
fn both(tex: &str) -> (Vec<String>, Vec<String>) {
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, two) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (creators_xml(&off), creators_xml(&two))
}

/// The affiliation contact `text`, as both engines write a plain one (its label's space a no-break one).
fn affiliation(text: &str) -> String {
  format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{text}</contact>")
}

/// At `=2` the queued call is Perl's raw `\lx@add@author{…}`: the old preparation's `\lx@frontmatter@symbolmark` of
/// `\textsuperscript{*}` no longer feeds the reading, which glued "*Corresponding author" into Bob Baker's affiliation
/// (2608.19750); the note is his, as the PDF prints it under his address. (Off glues it too: the old reader's.)
#[test]
fn a_raw_queued_call_feeds_no_symbol_mark_to_the_reading() {
  let (log, xml) = run(repro!("emit_raw_fallback_symbol_mark_is_no_affiliation"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let two = creators_xml(&xml);
  assert_eq!(two.len(), 2, "{two:#?}");
  assert!(
    two[0].contains("<personname>Ann Able</personname>"),
    "{}",
    two[0]
  );
  assert!(two[0].contains(&affiliation("Univ A")), "{}", two[0]);
  let bob = &two[1];
  for part in [
    "<personname>Bob Baker<sup>*</sup></personname>",
    &affiliation("Univ B"),
    "<contact name=\"Email:\u{a0}\" role=\"email\">bob@b.org</contact>",
    "role=\"thanks\"><sup>*</sup>Corresponding author</note>",
  ] {
    assert!(bob.contains(part), "{part} in {bob}");
  }
  assert!(!bob.contains("Univ B *"), "{bob}");
}

/// A parameterless macro closing the title page's tabular and opening another (sig-alternate's
/// `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}`) is the row break it prints: at `=2` the rehearsal left the
/// juggle's tabular open and "Dept" in no box, lost with no Warning (0712.1016, 1704.03168, 1301.5612, 1401.6981).
#[test]
fn a_tabular_juggle_is_a_row_break() {
  let (off, two) = both(repro!("author_shared_affiliation_line"));
  assert_eq!(two.len(), 1, "{two:#?}");
  assert!(
    two[0].contains("<personname>A One</personname>"),
    "{}",
    two[0]
  );
  assert!(two[0].contains(&affiliation("Dept")), "{}", two[0]);
  assert_eq!(two, off);
  // (a juggle printing a break between its close and its reopen, 1301.5612's `\end{tabular}\newline\begin{tabular}{c}`)
  let tex = repro!("author_shared_affiliation_line");
  let newline = tex.replacen("\\end{tabular}\n", "\\end{tabular}\\newline\n", 1);
  assert_ne!(newline, tex);
  let (_, two_newline) = both(&newline);
  assert_eq!(two_newline, two);
}

/// An alignment environment opening after the names and ending the block is lines under them, each its own and every
/// name's of the row; its "and" is no separator (2609.17357, 2609.01563, 2609.33831). At `=2` the reading made a creator
/// "Astronomy"; with the Break alone, the environment's lines were one print unit, one affiliation.
#[test]
fn a_trailing_alignment_environment_is_lines_under_the_names() {
  let (off, two) = both(repro!("author_trailing_center_lines_are_lines"));
  assert_eq!(two.len(), 2, "{two:#?}");
  for (creator, name) in two.iter().zip(["Ann Able", "Bob Baker"]) {
    assert!(
      creator.contains(&format!("<personname>{name}</personname>")),
      "{creator}"
    );
    let lines = [
      "Department of Physics and Astronomy",
      "University of X",
      "City",
    ];
    let at: Vec<usize> = lines
      .iter()
      .map(|line| creator.find(&affiliation(line)).expect(line))
      .collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "{creator}");
  }
  assert_eq!(two, off);
  let (off, two) = both(repro!("author_block_environment_is_one_unit"));
  assert_eq!(two.len(), 2, "{two:#?}");
  assert_eq!(two, off);
}

/// A `\thanks` set in a formula (`Bob Baker$^{1}\thanks{…}$`) is read in text mode, as its content is digested in the
/// document, and written from its own call: at `=2` its words lost their spaces ("Thefirsttwoauthorscontributedequally",
/// Warning "note no argument"; 2407.15815, 2603.20851).
#[test]
fn a_note_in_a_formula_keeps_its_words() {
  let (off, two) = both(repro!("author_thanks_in_a_formula_keeps_its_spaces"));
  assert_eq!(two.len(), 2, "{two:#?}");
  assert!(
    two[1].contains("role=\"thanks\">The first two authors contributed equally</note>"),
    "{}",
    two[1]
  );
  assert_eq!(two, off);
}

/// A URL line under every name of the document's author list is the block's, its shared creator (OD #159; user ruling
/// 2026-10-20, FIDELITY_CHARTER §4 amended 2026-11-05: 2604.20841, 2601.17879, 2604.12896); under one `\and` column of
/// several, the column's names' — known only once the block's later columns are read.
#[test]
fn a_url_line_under_every_name_is_the_blocks() {
  let (off, two) = both(repro!("author_link_line_url_contact_unmarked"));
  assert_eq!(two.len(), 3, "{two:#?}");
  assert!(!two[2].contains("<personname"), "{}", two[2]);
  assert!(
    two[2].contains("role=\"url\"><ref") && two[2].contains(">https://project.example.org</ref>"),
    "{}",
    two[2]
  );
  assert!(
    two[..2].iter().all(|c| !c.contains("role=\"url\"")),
    "{two:#?}"
  );
  assert_eq!(two, off);
  let (off, two) = both(repro!("author_url_line_under_one_column_is_its_names"));
  assert_eq!(two.len(), 3, "{two:#?}");
  let urls: Vec<bool> = two.iter().map(|c| c.contains("role=\"url\"")).collect();
  assert_eq!(urls, [true, true, false], "{two:#?}");
  assert_eq!(two, off);
}

/// A juggle reopens the tabular it closes: a macro that only closes one (`\newcommand{\etab}{\end{tabular}}`, closing the
/// block's own `\begin{tabular}`) keeps its meaning at `=2` — bound to a row break, it left the block's tabulars open and
/// Bob Baker with both affiliations lost (review revFB2a finding 1). In any rehearsal, an API call's too: revtex's
/// `\affiliation{… \bx a&b\\ c&d\ex …}`, whose `\ex` closes the tabular `\bx` opened (z4's row, here at `=2`). Distinct
/// from `a_tabular_juggle_is_a_row_break`: the predicate's other side. (Off, the old driver's looser test, errs on the
/// first shape, E 4: kept until the flip.)
#[test]
fn a_macro_closing_a_tabular_is_no_juggle() {
  let (log, xml) = run(repro!("author_tabular_close_macro_is_no_juggle"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let expected = vec![
    vec!["Ann Able".to_string(), "affiliation=Univ A".to_string()],
    vec!["Bob Baker".to_string(), "affiliation=Univ B".to_string()],
  ];
  assert_eq!(creators(&xml), expected, "{xml}");
  let text = "1 Univ X a b c d";
  let (log, xml) = run(&environment_row(ENVIRONMENT_SHAPE, text), 2);
  assert_eq!(error_count(&log), 0, "{log}");
  let expected = vec![vec![
    "Ann Able".to_string(),
    format!("affiliation={text}"),
    "affiliation=2 Univ Y".to_string(),
  ]];
  assert_eq!(creators(&xml), expected, "{xml}");
}

/// A link line under every name that prints an institution (`\href{https://www.google.com}{Google}`, a bare domain) is
/// every name's affiliation, its link kept, as the old rule's exception to the block's URL gives it every author
/// (`author_block::link_prints_an_institution`, user ruling R12 2026-10-20): at `=2` it was a nameless block creator's
/// "url: Google" (review revFB2a finding 3). Distinct from `a_url_line_under_every_name_is_the_blocks`: the exception.
#[test]
fn a_link_printing_an_institution_is_every_names() {
  let (off, two) = both(repro!("author_link_printing_an_institution_is_every_names"));
  assert_eq!(two.len(), 2, "{two:#?}");
  let google = affiliation("<ref class=\"ltx_href\" href=\"https://www.google.com\">Google</ref>");
  for (creator, name) in two.iter().zip(["Ann Able", "Bob Baker"]) {
    assert!(
      creator.contains(&format!("<personname>{name}</personname>")),
      "{creator}"
    );
    let at: Vec<usize> = [affiliation("Univ A"), google.clone()]
      .iter()
      .map(|a| creator.find(a.as_str()).expect(a))
      .collect();
    assert!(at[0] < at[1], "{creator}");
    assert!(!creator.contains("role=\"url\""), "{creator}");
  }
  assert_eq!(two, off);
}

/// Batch FB2b4: the juggle written inline in `\author`, `\end{tabular}\begin{tabular}[t]{c}` between two authors, is the
/// column sig-alternate's `\alignauthor` is: at `=2` the rehearsal left the reopened tabular open and Bob Two with Univ
/// B in no box, lost with no Warning (1401.6981, authors 2 and 3). With a line break printed between
/// (`\end{tabular}\newline\begin{tabular}{c}`) that break stands, the line under it the author's. Distinct from
/// `a_tabular_juggle_is_a_row_break`: the written form, read off the tokens (`rehearsal::inline_juggles_as_columns`),
/// not a macro's meaning. (Off, the old driver's, errs E 4: kept until the flip.)
#[test]
fn an_inline_tabular_juggle_is_a_column() {
  let tex = repro!("author_raw_tabular_juggle_is_a_column");
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let expected = vec![
    vec!["A One".to_string(), "affiliation=Univ A".to_string()],
    vec!["Bob Two".to_string(), "affiliation=Univ B".to_string()],
  ];
  assert_eq!(creators(&xml), expected, "{xml}");
  let newline = tex.replacen(
    "\\end{tabular}\\begin{tabular}[t]{c} Bob Two\\\\ Univ B",
    "\\end{tabular}\\newline\\begin{tabular}{c} Dept",
    1,
  );
  assert_ne!(newline, tex);
  let (log, xml) = run(&newline, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let expected = vec![vec![
    "A One".to_string(),
    "affiliation=Univ A".to_string(),
    "affiliation=Dept".to_string(),
  ]];
  assert_eq!(creators(&xml), expected, "{xml}");
}
