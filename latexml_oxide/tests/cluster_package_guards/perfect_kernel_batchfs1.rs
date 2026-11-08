//! Guards for FS1: the frontmatter content check under `LATEXML_FM_EMIT=2` does not trust a block's events when the
//! block's rehearsal left a group open (`rehearsal::Rehearsal::Done::left_open`): the words of its content's text
//! (`ledger::content_text`) the events lack are counted, a possible loss (`resolver::placed::content_check`).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators, perfect_kernel_batchn2::content_checks,
  perfect_kernel_batchz3::in_a_directory,
};

/// `\author{A One \end{tabular}\begin{tabular}{c} Dept}`: the rehearsal leaves the tabular open, "Dept" in no box and no
/// event, and `=2` lost it with no Warning (FB2a; the shape of 1401.6981's sig-alternate juggle). The check counts the
/// content's words the events lack, the environment names among them: one Warning, `c Dept tabular`. Off is unchanged (its
/// own 4 Errors, no check). Distinct from RC1's unread-call guard: a block that was rehearsed, its events incomplete.
#[test]
fn an_open_group_rehearsals_words_are_counted() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_raw_tabular_juggle_counted.tex"
  );
  for (emit, errors, warnings) in [("0", 4, 0), ("2", 0, 1)] {
    let (log, xml) = in_a_directory(tex, &[], &[("LATEXML_FM_EMIT", emit)]);
    assert_eq!(log.matches("\nFatal:").count(), 0, "{log}");
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (errors, warnings),
      "emit={emit} {log}"
    );
    let lost = if emit == "2" {
      vec!["c Dept tabular".to_string()]
    } else {
      vec![]
    };
    assert_eq!(content_checks(&log), (lost, vec![]), "emit={emit} {log}");
    assert_eq!(creators(&xml), [vec!["A One".to_string()]], "emit={emit}");
  }
}

/// The check's word matching where a block's rehearsal left a group open (RC1e; review revG2 findings 1-3, the shape of
/// 1401.6981's juggle), a row per path: a content word read without the accent its block's events print (`Muller` for
/// `Müller`) and one read without a dotless `\i` (`Fsica` for `Física`) are the block's own print, matched by
/// `resolver::placed::reads_as` before the gap counts; a control symbol inside a word parts it (`Smith\,\\Univ`,
/// `ledger::content_text`), so the placed "Smith" holds and the lost "Univ" counts alone. Only the really lost words are
/// counted. Only a word the content text read past an accent or a dotless letter in reads as a print by its base letters,
/// a dotless allowance an `i`/`j` where its `\i`/`\j` was (RC1f; review revH1 findings 1 and 4): a plain `Lu` is no
/// macro's `Liu`, a plain `Muller` no macro's `Müller`, both lost; exact matches come first, so the Warning names the lost
/// `Lan` beside a printed `Lian` and the lost `Fisica` beside `F\'{\i}sica`. Distinct from
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
      "dotless i",
      repro!("author_raw_tabular_juggle_dotless_i"),
      1,
      "c Dept tabular",
      vec![vec!["A One".to_string(), "affiliation=Física".to_string()]],
    ),
    (
      "control symbol parts",
      repro!("author_raw_tabular_juggle_control_symbol_parts"),
      2,
      "c tabular Univ",
      vec![vec!["A One Brasília Naïve Pîle Smith".to_string()]],
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
