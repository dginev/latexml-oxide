//! Guards for IO3b: a creator call a document note makes (amsart's `\thanks{… \author{…}\address{…}\email{…}}`,
//! 0807.4720) is an event of the note's reading (`rehearsal`'s `\lx@fm@obs@add`, `EventKind::ApiCall` "ltx:creator"),
//! and such a note is digested as Perl's at `=2`: each contact in it the newest creator's
//! (`\lx@annotate@frontmatter`, Base_Utility.pool.ltxml:520-531), no flush's reading answering one.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// The late author's address and e-mail are his, the first author holds none of them, on both switches (the only
/// path: the note's digestion; the A&A `\offprints` e-mail notes, which make no creator, stay the reading's).
#[test]
fn a_note_making_a_creator_is_digested_as_perls() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/late_author_in_amsart_thanks_once.tex"
  );
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [
    vec!["Ann Able", "address=Univ A", "email=ann@a.org"],
    vec!["Bob Baker", "address=Univ B", "email=bob@b.org"]
  ]);
}
