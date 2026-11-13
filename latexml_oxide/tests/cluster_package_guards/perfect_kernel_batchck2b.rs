//! Guards for CK2b, the reading's one record of its rule-drops (`Resolved::drops`): every word an entry's take-out cuts
//! that is no address's — its label and the separators it strands — is written for the checks, as the entry given back
//! whole wrote it before CK2 (review revI1 1).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// A `\thanks` ending `E-mails: ann.able@x.org and bob.baker@x.org.` (1610.05419's shape): the take-out cuts the label
/// and the "and" between the two addresses with them; both are the reading's drops, so no word is counted lost and no
/// nameless creator repeats the entry. Distinct from the label-only take-outs of batchz4b (`E-mails: a, b`): a word
/// between two addresses of one run.
#[test]
fn a_take_out_writes_every_word_it_cuts() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ck2b_take_out_and_between_addresses.tex"
  );
  let (log, two) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let univ_x = "affiliation=The authors are with the Dept of Physics, Univ of X, Town, USA";
  assert_eq!(creators(&two), [
    vec!["Ann Able", univ_x, "email=ann.able@x.org"],
    vec!["Bob Baker", univ_x, "email=bob.baker@x.org"],
    vec![
      "Cy Dee",
      "affiliation=The author is with Univ of Y",
      "email=cy.dee@y.org"
    ],
  ]);
}
