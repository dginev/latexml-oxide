//! Guards for KB2: the emitter's cut of an entry the reading took an address out of (P1b, `writer::address_cut`) keeps
//! `LATEXML_FM_EMIT=2`'s improvements where it harms none (the standing ruling, user 2026-11-08; OXIDIZED_DESIGN_DIVERGENCES
//! #980 KB2): an entry whose cut would strand the phrase its address completes is given back whole (`entry_cuts`,
//! `strands_a_phrase`), and a cut e-mail printed as a formula is written as its text (`cut_at`). The switch off's cut is
//! unchanged. One test per rule.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
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

/// `tex` clean with either switch: its `=2` creators' XML (ids aside), and its switch-off reading `off`.
fn emitted(tex: &str, off: &[Vec<&str>]) -> Vec<String> {
  let (off_log, off_xml) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  assert_eq!(creators(&off_xml), off);
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  creators_xml(&xml)
}

/// revtex4's `\affiliation{…, USA \\ $^{a}$Author to whom correspondence should be addressed: abrizard@smcvt.edu}`
/// (2306.04726; "Correspondence to: a@x", 2004.11812; 2110.05460, 2602.09324): the address completes a phrase naming a
/// contact that the take-out's label rule does not take, so at `=2` the entry is written whole, as it prints. Red
/// (P1b–int151a `=2`): "… USA a Author to whom correspondence should be addressed" left in the affiliation, the ": " lost,
/// the e-mail apart. Off keeps its cut.
#[test]
fn a_cut_never_strands_the_phrase_its_address_completes() {
  let off = [vec![
    "Alain J. Brizard a",
    "affiliation=Department of Physics, Saint Michael’s College, VT 05439, USA a Author to whom correspondence should be \
     addressed",
    "email=abrizard@smcvt.edu",
  ]];
  assert_eq!(
    emitted(repro!("emit_a_cut_never_strands_its_phrase"), &off),
    [
      "<creator role=\"author\">\n    <personname>Alain J.\u{a0}Brizard<sup><text \
      font=\"italic\">a</text></sup></personname>\n    <contact name=\"Affiliation:\u{a0}\" \
      role=\"affiliation\">Department of Physics, Saint Michael’s College, VT 05439, USA <break/><sup><text \
      font=\"italic\">a</text></sup>Author to whom correspondence should be addressed: \
      abrizard@smcvt.edu</contact>\n  </creator>"
    ]
  );
}

/// mn2e's `$^1$…, USA; dcook12$@$uwyo.edu` (1409.0847) and `$^2$…, France; bjohnson$@$iap.fr`: each cut address its owner's
/// e-mail written as its text, as off writes it. Red (P1b–int151a `=2`): each e-mail holding `<Math>@</Math>`. Dale's copy
/// of the shared line is not asserted: it loses the address (open, OD #980 KB2).
#[test]
fn a_cut_email_printed_as_math_is_its_text() {
  let wyoming = "affiliation=Department of Physics, University of Wyoming, WY 82071, USA";
  let off = [
    vec!["David O. Cook", wyoming, "email=dcook12@uwyo.edu"],
    vec!["Daniel A. Dale", wyoming],
    vec![
      "Benjamin D. Johnson",
      "affiliation=UPMC-CNRS, Institut d’Astrophysique de Paris, France",
      "email=bjohnson@iap.fr",
    ],
  ];
  let xml = emitted(repro!("emit_a_cut_email_printed_as_math_is_its_text"), &off);
  assert_eq!(xml.len(), 3, "{xml:?}");
  assert_eq!([&xml[0], &xml[2]], [
    "<creator role=\"author\">\n    <personname>David O. Cook</personname>\n    <contact \
       name=\"Affiliation:\u{a0}\" role=\"affiliation\">Department of Physics, University of Wyoming, WY 82071, \
       USA</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">dcook12@uwyo.edu</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Benjamin D. Johnson</personname>\n    \
       <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">UPMC-CNRS, Institut d’Astrophysique de Paris, \
       France</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">bjohnson@iap.fr</contact>\n  </creator>",
  ]);
}
