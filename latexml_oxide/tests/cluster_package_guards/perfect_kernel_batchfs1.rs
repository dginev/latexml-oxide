//! Guards for FS1 (FS2: on juggles the reading takes as written): the frontmatter content check under
//! `LATEXML_FM_EMIT=2` does not trust a block's events when the block's rehearsal left a group open
//! (`rehearsal::Rehearsal::Done::left_open`): the words of its content's text (`ledger::content_text`) the events lack
//! are counted, a possible loss (`resolver::placed::content_check`).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators, perfect_kernel_batchn2::content_checks,
  perfect_kernel_batchz3::in_a_directory,
};

/// A block whose rehearsal leaves a group open at `=2`: a juggle the reading takes as written, not as a column
/// (`rehearsal::inline_juggles_as_columns` steps over spaces and commands with their arguments only), its reopened
/// tabular left open, the lines after it in no box and no event (the shape of 1401.6981's sig-alternate juggle). The check
/// counts the content's words the events lack, the environment names among them, a row per thing between that stops the
/// scan: a tie (`\end{tabular}~\begin{tabular}{c} Dept`: "Dept" lost, `c Dept tabular`) and a bare group (`{Extra Words}`,
/// review revH1 h3f) — Bob Two A One's affiliation and "Univ B" lost, silent on FB2b6's branch, which lacked this count
/// (FB2b6 report item 2); here `B c tabular Univ`, with the plain-text Warning of the affiliation the writer wrote whole.
/// A skip whose amount is written out (`\hskip 1em`, h3e) no longer stops the scan (FB2b8): a column,
/// `perfect_kernel_batchfb2a::a_juggle_judges_the_command_between_by_what_it_prints`. Off is unchanged (its own 4 Errors, no check). The plain
/// juggle (`\end{tabular}\begin{tabular}{c}`) is a column since FB2b4-FB2b6, its words kept (`perfect_kernel_batchfb2a`).
/// Distinct from RC1's unread-call guard: a block that was rehearsed, its events incomplete.
#[test]
fn an_open_group_rehearsals_words_are_counted() {
  macro_rules! repro {
    ($name:literal) => {
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $name,
        ".tex"
      ))
    };
  }
  let rows = [
    (
      "a tie between",
      repro!("author_raw_tabular_juggle_counted"),
      1,
      "c Dept tabular",
      vec![vec!["A One".to_string()]],
    ),
    (
      "a bare group between",
      repro!("author_raw_tabular_juggle_bare_group_between_counted"),
      2,
      "B c tabular Univ",
      vec![vec![
        "A One".to_string(),
        "affiliation=Univ A Extra Words Bob Two".to_string(),
      ]],
    ),
  ];
  for (case, tex, warnings, lost, names) in rows {
    for (emit, errors, warnings, lost) in [
      ("0", 4, 0, vec![]),
      ("2", 0, warnings, vec![lost.to_string()]),
    ] {
      let (log, xml) = in_a_directory(tex, &[], &[("LATEXML_FM_EMIT", emit)]);
      assert_eq!(
        log.matches("\nFatal:").count(),
        0,
        "{case} emit={emit} {log}"
      );
      assert_eq!(
        (error_count(&log), warning_count(&log)),
        (errors, warnings),
        "{case} emit={emit} {log}"
      );
      assert_eq!(
        content_checks(&log),
        (lost, vec![]),
        "{case} emit={emit} {log}"
      );
      if emit == "2" {
        assert_eq!(creators(&xml), names, "{case} emit={emit}");
      }
    }
  }
}

/// The check's word matching where a block's rehearsal left a group open (RC1e; review revG2 findings 1-3, the shape of
/// 1401.6981's juggle; each with a tie between, `\end{tabular}~\begin{tabular}{c}`, read as written: FS2), a row per
/// path: a content word read without the accent its block's events print (`Muller` for
/// `Müller`) and one read without a dotless `\i` (`Fsica` for `Física`) are the block's own print, matched by
/// `resolver::placed::reads_as` before the gap counts; a control symbol inside a word parts it (`Smith\,\\Univ`,
/// `ledger::content_text`), so the placed "Smith" holds and the lost "Univ" counts alone (the tie parts the two columns'
/// names: two creators). Only the really lost words are
/// counted. Only a word the content text read past an accent or a dotless letter in reads as a print by its base letters,
/// a dotless allowance an `i`/`j` where its `\i`/`\j` was (RC1f; review revH1 findings 1 and 4): a plain `Lu` is no
/// macro's `Liu`, a plain `Muller` no macro's `Müller`, both lost; exact matches come first, so the Warning names the lost
/// `Lan` beside a printed `Lian` and the lost `Fisica` beside `F\'{\i}sica`; an accent opening a word (`\'Ecole`, review
/// revH3 finding 2) is read past as one inside it, a line break opening one (`Dept,\\Muller`, review revCK1 2) never: the
/// plain `Muller` is matched as spelled, lost beside the macro's printed `Müller`. Distinct from
/// [`an_open_group_rehearsals_words_are_counted`]: there no word the check reads is the block's.
#[test]
fn an_open_group_rehearsals_word_matching() {
  macro_rules! repro {
    ($name:literal) => {
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $name,
        ".tex"
      ))
    };
  }
  let rows = [
    (
      "accented word",
      repro!("author_raw_tabular_juggle_accented_word"),
      1,
      "c Dept tabular",
      vec![vec!["A Müller".to_string()]],
    ),
    (
      "an accent opening a word",
      repro!("author_raw_tabular_juggle_accent_opens_word"),
      1,
      "c Dept tabular",
      vec![vec![
        "A One".to_string(),
        "affiliation=École Normale".to_string(),
      ]],
    ),
    (
      "a line break opening a word",
      repro!("author_raw_tabular_juggle_break_opens_word"),
      2,
      "c Muller tabular",
      vec![vec![
        "A Müller".to_string(),
        "affiliation=Univ A Dept,".to_string(),
      ]],
    ),
    (
      "dotless i",
      repro!("author_raw_tabular_juggle_dotless_i"),
      1,
      "c Dept tabular",
      vec![vec!["A One".to_string(), "affiliation=Física".to_string()]],
    ),
    (
      "control symbol parts",
      repro!("author_raw_tabular_juggle_control_symbol_parts"),
      1,
      "c tabular Univ",
      vec![vec!["A One".to_string()], vec![
        "Brasília Naïve Pîle Smith".to_string(),
      ]],
    ),
    (
      "plain word, no dotless allowance",
      repro!("author_raw_tabular_juggle_plain_word_exact"),
      1,
      "c Dept Lu tabular",
      vec![vec![
        "Wei Liu".to_string(),
        "affiliation=Univ A".to_string(),
      ]],
    ),
    (
      "plain word, no base letters",
      repro!("author_raw_tabular_juggle_plain_word_unaccented"),
      1,
      "c Dept Muller tabular",
      vec![vec![
        "A Müller".to_string(),
        "affiliation=Univ A".to_string(),
      ]],
    ),
    (
      "the lost word named, not the printed one",
      repro!("author_raw_tabular_juggle_lost_beside_printed"),
      1,
      "c Dept Lan tabular",
      vec![vec![
        "Lian Wu".to_string(),
        "affiliation=Univ A".to_string(),
      ]],
    ),
    (
      "the plain word lost beside the dotless one",
      repro!("author_raw_tabular_juggle_lost_beside_dotless"),
      1,
      "c Dept Fisica tabular",
      vec![vec!["A One".to_string(), "affiliation=Física".to_string()]],
    ),
  ];
  for (case, tex, warnings, lost, names) in rows {
    let (log, _) = in_a_directory(tex, &[], &[("LATEXML_FM_EMIT", "0")]);
    assert_eq!(
      (
        error_count(&log),
        log.matches("\nFatal:").count(),
        warning_count(&log)
      ),
      (4, 0, 0),
      "{case} off {log}"
    );
    assert_eq!(content_checks(&log), (vec![], vec![]), "{case} off {log}");
    let (log, xml) = in_a_directory(tex, &[], &[("LATEXML_FM_EMIT", "2")]);
    assert_eq!(
      (
        error_count(&log),
        log.matches("\nFatal:").count(),
        warning_count(&log)
      ),
      (0, 0, warnings),
      "{case} =2 {log}"
    );
    assert_eq!(
      content_checks(&log),
      (vec![lost.to_string()], vec![]),
      "{case} =2 {log}"
    );
    assert_eq!(creators(&xml), names, "{case} =2");
  }
}
