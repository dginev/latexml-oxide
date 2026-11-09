//! Guards for P1c (E1's last fix round, review revP1br2): the label before an address is the reading's take-out's
//! (`resolver::labels::take_labels`; the emitter writes what it says — its unit test `take_labels_takes_the_label_before_a_run`,
//! its output `perfect_kernel_batchp1e::a_label_before_an_address_goes_with_its_run`), and the two ways the emitter's cut of
//! an entry's tokens went wrong (OXIDIZED_DESIGN_DIVERGENCES #980 P1c): a whole label line cut from an entry written line
//! by line lost every break of it (S1, 2604.06805). The icon beside an address (S3, RT1 row 56) stays RED, a catalog repro
//! (`emit_a_cut_never_takes_an_icon`), no guard.

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::{creators_xml, run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// An entry whose lines no balanced run holds (`\textsuperscript{2}Univ Z\\ 2 Side Road, City\\ \small{\textbf{Correspondence:}
/// \href{mailto:bob@z.org}{bob@z.org},}`, 2604.06805) whose label line the cut takes whole (`writer::emit::lines_run`): the
/// line goes with one break beside it, the entry's other lines and their break kept, the e-mail its owner's link named by
/// the label. Red (P1br2): "Univ Z 2 Side Road, City" plain, counted by 2 Warnings.
#[test]
fn a_whole_label_line_cut_keeps_the_breaks() {
  let (log, xml) = run(repro!("emit_a_whole_label_line_cut_keeps_the_breaks"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    creators_xml(&xml),
    [
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">Univ X<break/>1 Main Road, Town</contact>\n  </creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
       name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z<break/>2 Side Road, City</contact>\n    <contact \
       name=\"Correspondence: \" role=\"email\"><ref class=\"ltx_href\" \
       href=\"mailto:bob@z.org\">bob@z.org</ref></contact>\n  </creator>",
    ],
    "{xml}"
  );
}
