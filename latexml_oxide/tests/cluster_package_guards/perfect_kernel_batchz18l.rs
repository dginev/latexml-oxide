//! Guards for Z18l: AASTeX's `\affil` e-mail lines go to the frontmatter API as the affiliation entries they are written
//! as, and the API places their addresses (OXIDIZED_DESIGN_DIVERGENCES #992): the Kind step takes a line of addresses
//! out of the entry (`kind_entry_addresses`), the Link step gives each address its owner (`entry_address_owners`), off
//! `author_block/`'s binding-side readers (`placed_email_line_calls`, `take_placed_line_emails`, `labelled_email_line`,
//! deleted). The unmarked line and the `\email` set in a line stay `perfect_kernel_batch97f`'s guards (astro-ph0408475,
//! 1503.02933), green through the reroute; this one guards what the reroute changed: an e-mail line a requested mark
//! leads is the e-mail of the one name requesting it even where its address spells no one (marks win), in a block of
//! `\\` lines (the binding's entries loop) and as an `\affil` of its own (the marked rule) — two paths of the binding,
//! one placement. It converts each repro and asserts the exact Error and Warning counts, every creator's name and
//! contacts in order, and the e-mail contact whole.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// `\affil{… \\ $^*$ Email: zz@uni.edu}` and `\affil{$^*$ Email: zz@uni.edu}` under `Bob Baker\altaffilmark{2,*}`
/// (1808.01089's shape, its address spelling no one): Bob Baker's e-mail, named by its label. The base
/// (`placed_email_line_calls`) gave it to the block's nameless creator in both.
#[test]
fn aastex_affil_marked_email_line_no_one_spells_is_the_marked_authors() {
  for tex in [
    repro!("aastex_affil_marked_email_line_no_one_spells_is_the_marked_authors.tex"),
    repro!("aastex_affil_marked_email_line_of_its_own_no_one_spells.tex"),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(
      creators(&xml),
      [
        vec!["Ann Able", "affiliation=Uni One"],
        vec!["Bob Baker", "affiliation=Uni Two", "email=zz@uni.edu"],
        vec!["Cat Cole", "affiliation=Uni One", "affiliation=Uni Two"],
      ],
      "{xml}"
    );
    assert_eq!(
      xml
        .matches("<contact name=\"Email:\u{a0}\" role=\"email\">zz@uni.edu</contact>")
        .count(),
      1,
      "{xml}"
    );
  }
}
