//! Red/green guards for batch 103a: a list of local parts written without spaces before one domain
//! (`dzyang,ashishbshah@fb.com`) inside a `;`- or `,`-parted e-mail line is written out, each local part at the domain,
//! by the one reading both address readers share (latexml_engine/src/base_utilities/frontmatter/classify/email_block_calls.rs
//! `write_out_unspaced_lists`: the author block's `email_addresses`, the frontmatter observer's `plain_addresses`), and
//! the flush's Link step writes an e-mail out only where the two readings of it agree, so no local part is ever lost
//! (frontmatter/insert.rs `link_entry_addresses`; OXIDIZED_DESIGN_DIVERGENCES #956). Code paths: the flush's
//! write-out of an e-mail an `\inst`-marked institute prints, and the address reader on llncs's shared institute row
//! (the old `shared_row::row_email_calls`, deleted in P27b; the emitter's entry cut since) and on a hand-marked institute list (`affiliations::place_address_runs`). Each guard
//! converts its repros under `tools/perfect_kernel/repros/sectioning-frontmatter/` and asserts no Error or Warning and
//! every creator's name and contacts (role and text) in order. Witness 2112.00969 (batch 100b's regression: `dzyang`
//! lost).
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

/// Converts `tex`, checks it has no Error and no Warning and that its creators are `expected` (each its name, then each
/// contact as `role=text`).
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) {
  let (log, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
}

/// The author block's address reader (`email_addresses`) on llncs's one institute: the shared row (no `\inst` marks,
/// the old `shared_row::row_email_calls`, deleted in P27b) and a list whose entries hand-printed marks label (`$^1$ … $^2$ …`, the witness's
/// shape, 2112.00969: `affiliations::place_address_runs`). The line's unspaced list is written out, each address the
/// author it spells. Red (int130a): the shared row's line one address of a nameless creator; the marked list's
/// `dzyang` lost.
#[test]
fn an_unspaced_list_in_a_shared_institute_is_written_out() {
  assert_creators(repro!("address_family_unspaced_list_shared_institute"), &[
    &["Ann Able", "affiliation=Meta AI", "email=ann@a.edu"],
    &["Dan Yang", "affiliation=Meta AI", "email=dzyang@fb.com"],
    &[
      "Ash Shah",
      "affiliation=Meta AI",
      "email=ashishbshah@fb.com",
    ],
  ]);
  assert_creators(repro!("address_family_unspaced_list_marked_institute"), &[
    &["Ann Able", "affiliation=Univ A", "email=ann@a.edu"],
    &["Dan Yang", "affiliation=Meta AI", "email=dzyang@fb.com"],
    &[
      "Ash Shah",
      "affiliation=Meta AI",
      "email=ashishbshah@fb.com",
    ],
  ]);
}

/// Review rev103a2 B1: the unspaced list's checks (no one-letter word, no contact word: `is_listed_local_part`,
/// `ends_a_listed_run`) are that list's alone; a part with a space after its comma takes `address_runs`' spaced-list
/// rule as on base (int130a), so a labelled spaced list keeps its one-letter mailbox (`E-mail: ann, b@x.org`) and a
/// one-letter first part does not split the line (`E-mail: a, bob@x.org`: no affiliation "E-mail: a" beside the
/// e-mail). Red (103a r2): Bob Baker's affiliation "E-mail: ann", `ann@x.org` lost; "E-mail: a" split off. Since 106a
/// the whole line is addresses, as `address_runs` reads it (the Kind step for an entry's address line, OD #966):
/// `bob@x.org` Bob Baker's, `a@x.org`, spelling no one, the block's (int133a: the whole line Bob Baker's affiliation,
/// the imperfection this pinned).
#[test]
fn a_spaced_list_takes_the_spaced_list_rule() {
  assert_creators(
    repro!("address_family_spaced_list_one_letter_last_kept"),
    &[&["Ann Able", "affiliation=Univ X", "email=ann@x.org"], &[
      "Bob Baker",
      "email=b@x.org",
    ]],
  );
  assert_creators(
    repro!("address_family_spaced_list_one_letter_piece_kept"),
    &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "email=bob@x.org"],
      &["email=a@x.org"],
    ],
  );
}
