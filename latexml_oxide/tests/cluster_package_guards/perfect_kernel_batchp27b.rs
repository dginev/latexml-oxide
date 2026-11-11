//! Guards for P27b, Option C step P2-7's S8 half (`agent_reports/2026-11-11_P27_report.md` "The flush (S8)";
//! OXIDIZED_DESIGN_DIVERGENCES #901): a flush with no author block whose queue holds a class's row contact (`_share`,
//! llncs's sole `\institute`) is written by the emitter (`emit.rs` `stays_out`), not shaped by the old row split for
//! Perl's digestion (`share_row_contacts`, deleted). One guard per path: the shared institute with each author's own
//! e-mail, and the institutes the authors' `\inst{n}` marks pick. Each asserts no Error, no Warning, and every creator
//! whole, in order. Green on the base (int159a1) and on P27b.

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::{creators_xml, run};

/// `tex` converted clean (0 errors, 0 warnings), its creators whole, in order.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&str]) {
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators_xml(&xml), expected, "{xml}");
}

/// llncs + authblk, two `\author` calls (no block) and the sole institute: the institute every author's, each `\email`
/// its speller's (the Z18n probe's shape).
#[test]
fn llncs_authblk_calls_share_the_sole_institute() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_authblk_calls_share_the_sole_institute.tex"
    ),
    &[
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">University of Somewhere, Sometown</contact>\n    <contact name=\"E-mail\u{a0}\" \
       role=\"email\"><text font=\"typewriter\">ann.able@x.edu</text></contact>\n  </creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
       name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of Somewhere, Sometown</contact>\n    <contact \
       name=\"E-mail\u{a0}\" role=\"email\"><text font=\"typewriter\">bob.baker@x.edu</text></contact>\n  </creator>",
    ],
  );
}

/// llncs + authblk, two `\author` calls (no block) whose `\inst{n}` marks pick the numbered institutes: each its own,
/// the e-mail with its institute, nothing shared.
#[test]
fn llncs_authblk_calls_inst_marks_pick_the_institutes() {
  assert_creators(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_authblk_calls_inst_marks_pick_the_institutes.tex"
    ),
    &[
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">Univ A </contact>\n    <contact name=\"E-mail\u{a0}\" role=\"email\"><text \
       font=\"typewriter\">ann.able@x.edu</text></contact>\n  </creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
       name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
    ],
  );
}
