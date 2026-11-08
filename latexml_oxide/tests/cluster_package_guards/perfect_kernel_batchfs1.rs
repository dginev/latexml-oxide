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
