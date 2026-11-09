//! Guards for KB2: the emitter's cut of an entry the reading took an address out of (P1b, `writer::address_cut`) keeps
//! `LATEXML_FM_EMIT=2`'s improvements where it harms none (the standing ruling, user 2026-11-08; OXIDIZED_DESIGN_DIVERGENCES
//! #980 KB2): an entry whose cut would strand the phrase its address completes is given back whole (`entry_cuts`,
//! `strands_a_phrase`) where its holders are exactly its address's owners (`held_by_its_owners`), and a cut e-mail printed
//! as a formula is written as its text (`cut_at`). The switch off's cut is unchanged. One test per rule.

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

/// `tex` clean with either switch, its switch-off reading `off`, and its `=2` creators' XML the switch off's.
fn emitted_as_off(tex: &str, off: &[Vec<&str>]) {
  let (_, off_xml) = run(tex, 0);
  assert_eq!(emitted(tex, off), creators_xml(&off_xml));
}

/// revtex4's `\affiliation{…, USA \\ $^{a}$Author to whom correspondence should be addressed: abrizard@smcvt.edu}`
/// (2306.04726; 2110.05460, 2602.09324): the address completes a phrase naming a contact that the take-out's label rule does
/// not take, and the entry's one holder owns it, so at `=2` the entry is written whole, as it prints (2004.11812's
/// "Correspondence to: a@x", another author's, keeps its cut: `a_whole_entry_gives_its_address_to_no_other_author`). Red
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

/// A block's last line `Correspondence to: \texttt{pascal.klink@tu-darmstadt.de}` read onto the Aalto entry, which only
/// Pajarinen holds (neurips_2020, 2004.11812): the address is Klink's, so the entry keeps its cut at `=2`, the e-mail his, as
/// off writes it (`held_by_its_owners`). Red (KB2 `=2`): Klink no e-mail, "…Finland Correspondence to:
/// pascal.klink@tu-darmstadt.de" in Pajarinen's affiliation. The stranded "Correspondence to" is the base's (open, OD #980).
#[test]
fn a_whole_entry_gives_its_address_to_no_other_author() {
  let darmstadt = "affiliation=Technische Universität Darmstadt, Germany";
  emitted_as_off(repro!("emit_a_whole_entry_is_its_owners"), &[
    vec![
      "Pascal Klink",
      darmstadt,
      "email=pascal.klink@tu-darmstadt.de",
    ],
    vec![
      "Joni Pajarinen",
      darmstadt,
      "affiliation=Aalto University, Finland Correspondence to",
    ],
  ]);
}

/// authblk's `\affil[1]{Univ. Alpha, USA. Correspondence to: ann@alpha.edu}`, held by Ann and Bob (review revJ2 k7, the
/// class of 2004.11812): Bob owns no address there, so the entry keeps its cut on both at `=2`, Ann's e-mail hers, as off
/// writes it. Red (KB2 `=2`): the entry whole on Ann and on Bob, Ann no e-mail apart.
#[test]
fn a_shared_entry_keeps_its_cut() {
  let alpha = "affiliation=Univ. Alpha, USA. Correspondence to";
  emitted_as_off(repro!("emit_a_shared_entry_keeps_its_cut"), &[
    vec!["Ann Able", alpha, "email=ann@alpha.edu"],
    vec!["Bob Bell", alpha],
    vec![
      "Cy Cole",
      "affiliation=Univ. Beta, Spain",
      "email=cy@beta.es",
    ],
  ]);
}
