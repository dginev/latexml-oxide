//! Guards for LO3, content lost at `=2` only (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): a contact command of an author block whose kind is a note — jheppub's `\note{…}` —
//! written as the contact command it is (`a_note_command_in_an_author_is_its_contact`).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

/// jheppub's `\note{…}` inside an `\author`, `\lx@add@contact[role=note]{…}` (Perl's jheppub.sty.ltxml:39): each author's
/// notes are that author's `<contact role="note">` with the switch on as off — after a comma (1403.1099, 2001.06318), glued
/// to the name, two of them, math and an `\emph` in one (2308.04217's "Speaker at …" notes), and a marked `\note[*]{…}`.
/// The reading's Contact event of kind note was written as an `ltx:note` around the call, where the call means nothing, so
/// every note was lost and the content check counted it (int147a `=2`: 1 Warning). The creators' XML (ids aside) is the
/// switch-off one.
#[test]
fn a_note_command_in_an_author_is_its_contact() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_jheppub_note_in_an_author_is_its_contact.tex"
  );
  let (log, xml) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want = [
    vec![
      "Ann Able",
      "note=Corresponding author.",
      "affiliation=Univ A",
      "affiliation=Univ B",
      "email=ann@a.org",
    ],
    vec![
      "Bob Baker",
      "note=Also at Univ C.",
      "note=Visiting α -Lab, Univ D .",
      "affiliation=Univ A",
    ],
    vec!["Cy Cole", "note=Equal contribution.", "affiliation=Univ B"],
  ];
  assert_eq!(creators(&xml), want);
  assert_eq!(creators_xml(&xml), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Corresponding author.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\">ann@a.org</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Also at Univ C.</contact>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Visiting <Math mode=\"inline\" tex=\"\\alpha\" text=\"alpha\">\n        <XMath>\n          <XMTok font=\"italic\" name=\"alpha\" role=\"UNKNOWN\">α</XMTok>\n        </XMath>\n      </Math>-Lab, <emph font=\"italic\">Univ D</emph>.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Cole</personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\">Equal contribution.</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>"
  ]);
}
