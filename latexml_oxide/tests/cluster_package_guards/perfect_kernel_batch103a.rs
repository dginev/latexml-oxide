//! Red/green guards for batch 103a: a list of local parts written without spaces before one domain
//! (`dzyang,ashishbshah@fb.com`) inside a `;`- or `,`-parted e-mail line is written out, each local part at the domain,
//! by the one reading both address readers share (latexml_engine/src/base_utilities/author_block/email_block_calls.rs
//! `write_out_unspaced_lists`: the author block's `email_addresses`, the frontmatter observer's `plain_addresses`), and
//! the flush's Link step writes an e-mail out only where the two readings of it agree, so no local part is ever lost
//! (frontmatter/insert.rs `link_entry_addresses`; OXIDIZED_DESIGN_DIVERGENCES #956). Code paths: the flush's
//! write-out of an e-mail an `\inst`-marked institute prints, and the address reader on llncs's shared institute row
//! (`shared_row::row_email_calls`) and on a hand-marked institute list (`affiliations::place_address_runs`). Each guard
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

/// The flush's write-out of an `\inst`-marked institute's e-mail (2112.00969): the unspaced list after a `;` or a `,`,
/// or alone, is each local part at the domain, the author it spells — "David Z. Yang" by initials and surname too. Red
/// (int130a): `dzyang` nowhere in the XML; alone, the list one address of a nameless creator. The control: a spaced
/// list not opening the line is no reader's, so the e-mail stays whole on its carriers, nothing lost.
#[test]
fn an_unspaced_list_of_local_parts_is_written_out() {
  for (tex, dan) in [
    (
      repro!("address_family_unspaced_list_semicolon_line"),
      "Dan Yang",
    ),
    (
      repro!("address_family_unspaced_list_comma_line"),
      "Dan Yang",
    ),
    (
      repro!("address_family_unspaced_list_initials_spelled"),
      "David Z. Yang",
    ),
  ] {
    assert_creators(tex, &[
      &["Ann Able", "affiliation=Univ A", "email=ann@a.edu"],
      &[dan, "affiliation=Meta AI", "email=dzyang@fb.com"],
      &[
        "Ash Shah",
        "affiliation=Meta AI",
        "email=ashishbshah@fb.com",
      ],
    ]);
  }
  assert_creators(repro!("address_family_unspaced_list_alone"), &[
    &["Ann Able", "affiliation=Univ A"],
    &["Dan Yang", "affiliation=Meta AI", "email=dzyang@fb.com"],
    &[
      "Ash Shah",
      "affiliation=Meta AI",
      "email=ashishbshah@fb.com",
    ],
  ]);
  // (the control after an address on the kernel block, review rev103a2 probe m2: each local part the author it
  // spells, by the shared-domain ruling — `jd@` Joe Doe's initials, `ann2@` Ann Able's)
  assert_creators(repro!("address_family_unspaced_list_after_an_address"), &[
    &["Ann Able", "email=ann@x.org", "email=ann2@x.org"],
    &["Joe Doe", "affiliation=Univ X", "email=jd@x.org"],
  ]);
  assert_creators(repro!("address_family_spaced_list_stays_whole"), &[
    &["Ann Able", "affiliation=Univ A"],
    &[
      "Dan Yang",
      "affiliation=Meta AI",
      "email=ann@a.edu; dzyang, ashishbshah@fb.com",
    ],
    &[
      "Ash Shah",
      "affiliation=Meta AI",
      "email=ann@a.edu; dzyang, ashishbshah@fb.com",
    ],
  ]);
}

/// The author block's address reader (`email_addresses`) on llncs's one institute: the shared row (no `\inst` marks,
/// `shared_row::row_email_calls`) and a list whose entries hand-printed marks label (`$^1$ … $^2$ …`, the witness's
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

/// Review rev103a B1: a word naming a contact glued to an address (`email,ann@x.org`, `contact,bob@a.edu`) is no local
/// part of a list (`author_block::is_listed_local_part` / `names_a_contact`), so no address is invented and the line
/// reads as on base (int130a) — whole on the kernel block's last author (article) or on llncs's nameless e-mail creator
/// of the shared institute. Red (103a r1): a nameless creator holding `email@x.org`, `contact@a.edu`.
#[test]
fn a_contact_word_glued_to_an_address_is_no_local_part() {
  assert_creators(repro!("address_family_contact_word_glued_stays_whole"), &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=email,ann@x.org, bob@x.org"],
  ]);
  assert_creators(
    repro!("address_family_contact_word_glued_in_email_stays_whole"),
    &[
      &["Ann Able", "affiliation=Univ A"],
      &["Bob Baker", "affiliation=Univ A"],
      &["email=ann@a.edu; contact,bob@a.edu"],
    ],
  );
}

/// Review rev103a S1: a one-letter word is an initial — it ends no list (`smith,j@x.org`,
/// `author_block::ends_a_listed_run`: the kernel block, as on base) and is no piece of one (`d,ashishbshah@fb.com`:
/// the take-out leaves the `d` in the e-mail's rest, so the flush keeps the e-mail whole on its carriers). The second
/// case pins a trade (review rev103a2 NIT): the `d` is no longer lost, but the whole e-mail, `ann@a.edu` with it, sits
/// on Dan Yang and Ash Shah, none on Ann Able. Red (103a r1): `smith@x.org` and a nameless `j@x.org`, `Univ X` moved;
/// a nameless `d@fb.com` (int130a: `d` lost).
#[test]
fn a_one_letter_word_is_an_initial_not_a_local_part() {
  assert_creators(
    repro!("address_family_one_letter_address_is_an_initial"),
    &[&["John Smith", "affiliation=Univ X"], &[
      "Jane Doe",
      "affiliation=smith,j@x.org, doe@x.org",
    ]],
  );
  assert_creators(
    repro!("address_family_one_letter_piece_is_no_local_part"),
    &[
      &["Ann Able", "affiliation=Univ A"],
      &[
        "Dan Yang",
        "affiliation=Meta AI",
        "email=ann@a.edu; d,ashishbshah@fb.com",
      ],
      &[
        "Ash Shah",
        "affiliation=Meta AI",
        "email=ann@a.edu; d,ashishbshah@fb.com",
      ],
    ],
  );
}

/// The flush's `taken == listed` gate (frontmatter/insert.rs `link_entry_addresses`, review rev103a NIT): a spaced list
/// opening the e-mail is the take-out's (`address_runs`) and not the printed-address reader's (`addresses_in`), so the
/// two readings disagree and the e-mail stays whole on its carriers. Red (int130a, no gate): `dzyang` lost, the two
/// other addresses written out.
#[test]
fn an_email_its_two_readings_read_apart_stays_whole() {
  assert_creators(
    repro!("address_family_spaced_list_opening_the_line_stays_whole"),
    &[
      &["Ann Able", "affiliation=Univ A"],
      &[
        "Dan Yang",
        "affiliation=Meta AI",
        "email=dzyang, ashishbshah@fb.com; ann@a.edu",
      ],
      &[
        "Ash Shah",
        "affiliation=Meta AI",
        "email=dzyang, ashishbshah@fb.com; ann@a.edu",
      ],
    ],
  );
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
