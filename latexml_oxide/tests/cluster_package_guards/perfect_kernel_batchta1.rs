//! Guards for TA1: an author block digested while a flush digests its queue (an `\author` in the `\title`, 1502.01001,
//! 1510.00875, hep-ph0207056, hep-th0111116) has its own flush at `=2` (`\lx@frontmatter@late`), which
//! reads it as an early block is read: its split creators, each with its own contacts, no merged creator beside them.
//! The amsart `\thanks` shape (0807.4720, `late_author_in_amsart_thanks_once`) is IO3b's: `perfect_kernel_batchio3b`.

use latexml::util::test::{error_count, warning_count};

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

/// One guard per path the late block's flush meets: the `\maketitle` flush (`split_at_its_flush`), the seal of a later
/// flush after the body began (`survives_the_seal`), the fallback flush at `\section` (`fallback_flush`), and a
/// `\thanks` inside the late block (`thanks_each_its_own`, hep-th0111116). Each clean on both switches, `=2` writing
/// the creators off writes.
#[test]
fn a_late_author_block_is_read_at_its_own_flush() {
  let affiliations = [vec!["Ann Able", "affiliation=Univ A"], vec![
    "Bob Baker",
    "affiliation=Univ B",
  ]];
  for tex in [
    repro!("late_author_in_title_split_at_its_flush"),
    repro!("late_author_in_title_survives_the_seal"),
    repro!("late_author_in_title_fallback_flush"),
  ] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(creators(&xml), affiliations, "{tex}");
  }
  let tex = repro!("late_author_in_title_thanks_each_its_own");
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let names: Vec<Vec<String>> = creators(&xml);
  assert_eq!(names.len(), 2, "{xml}");
  for (creator, (name, thanks)) in xml
    .split("<creator ")
    .skip(1)
    .zip([("Ann Able", "Univ A"), ("Bob Baker", "Univ B")])
  {
    let creator = &creator[..creator.find("</creator>").expect("closed creator")];
    assert!(creator.contains(name), "{creator}");
    assert!(
      creator.contains("role=\"thanks\"") && creator.contains(thanks),
      "{creator}"
    );
  }
}
