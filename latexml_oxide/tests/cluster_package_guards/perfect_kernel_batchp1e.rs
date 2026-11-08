//! Guards for P1b (E1, the strict port of the switch-off writer's cut, RT1's rows 54-58): an address inside an entry's
//! text is its owner's e-mail at `LATEXML_FM_EMIT=2` as with the switch off (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980 P1b): the reading's take-out (`Resolver::take_out_entry_addresses`) is written from the
//! entry's own tokens without the runs its cut covers, at the positions the lexer recorded for each character of its inks
//! (`events::InkTable`, `writer::address_cut`), each address the e-mail of the owner the reading gives it, as printed. One test
//! per way the emitter writes an entry: a note's argument (`\thanks`, a font group and a bracketed address), an API call's
//! content (llncs's `\institute`, authblk's `\affil`, a local-part list), a block line read on with an address line (the
//! entry's own span), and a label word with brackets (the e-mail's name; RP1's draft regression, 2311.02475). Tests and
//! repros from RP1's draft `pk/P1` (645e93227c), re-pointed. P1br2 (review revP1b): the label the Kind step for an address
//! line reads before the address goes with it (B1), the address keeps its link or font (B2, RT1 row 56), a mark after the
//! separator the cut takes stays (S1).

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

/// `tex` clean with either switch, its `=2` creators' XML (ids aside) the switch-off one's, and their reading `want`.
fn as_off(tex: &str, want: &[Vec<&str>]) {
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&off), want);
  assert_eq!(creators(&xml), want);
  assert_eq!(creators_xml(&xml), creators_xml(&off), "{tex}");
}

/// A note's entry (`\thanks{$^{1}$Ann Able and Bob Baker are with … {\tt\small a@x; b@x}}`, ieeeconf's, 2011.02560,
/// 2104.05078, 2204.02371, 2410.06013, 1912.07315; `({\tt\small cy.cole@b.org}).`, 2211.06034): its text without the
/// addresses — the font group, the brackets and the stop round them — and each address its spelled author's e-mail in
/// the group's typewriter font. Red (LO4 `=2`): the whole text, addresses and brackets in it, on each carrier.
#[test]
fn an_address_in_a_note_entry_is_its_owners_email() {
  let entry =
    "affiliation=Ann Able and Bob Baker are with the Department of X, Univ A, City, Country";
  as_off(repro!("emit_an_address_in_a_note_entry_is_its_owners"), &[
    vec!["Ann Able", entry, "email=ann.able@a.org"],
    vec!["Bob Baker", entry, "email=bob.baker@a.org"],
    vec![
      "Cy Cole",
      "affiliation=Cy Cole is with Univ B, Town",
      "email=cy.cole@b.org",
    ],
  ]);
}

/// An API call's content: llncs's `\institute{Univ A, … \texttt{ann.able@a.org} \and …}` (svmult's, astro-ph0610446,
/// cond-mat0610708, physics0603032) and authblk's `\affil[1]{\texttt{\{ann.able,bbaker\}@a.org}, Univ A}` (2406.02913,
/// astro-ph0412523, 2508.09777): each entry without its addresses, each address the e-mail of the author it spells —
/// one of a local-part list each its own. Red (LO4 `=2`): every carrier of an entry held its addresses as text.
#[test]
fn an_address_in_an_api_entry_is_its_owners_email() {
  as_off(repro!("emit_an_address_in_an_institute_is_its_owners"), &[
    vec![
      "Ann Able",
      "affiliation=Univ A, Street 1, City, Country",
      "email=ann.able@a.org",
    ],
    vec!["Bob Baker", "affiliation=Univ A, Street 1, City, Country"],
    vec![
      "Cy Cole",
      "affiliation=Univ B, Road 2, Town, Country",
      "email=cy.cole@b.org",
    ],
  ]);
  as_off(repro!("emit_an_address_list_in_an_affil_is_its_owners"), &[
    vec!["Ann Able", "affiliation=Univ A", "email=ann.able@a.org"],
    vec!["Bob Baker", "affiliation=Univ A", "email=bbaker@a.org"],
    vec!["Cy Cole", "affiliation=Univ B", "email=cy.cole@b.org"],
  ]);
}

/// A labelled line the reading read on with an address line (`$^2$Univ B, Town, Country\\ Correspondence to:
/// bob.baker@b.org`; 2004.11812, 1203.0253, 2405.03617): the entry's tokens over both lines cut, its label word kept as
/// with the switch off, the address its spelled author's e-mail. Red (LO4 `=2`): the address in Bob's affiliation.
#[test]
fn an_address_line_read_on_is_its_owners_email() {
  as_off(repro!("emit_an_address_line_read_on_is_its_owners"), &[
    vec!["Ann Able", "affiliation=Univ A, City, Country"],
    vec![
      "Bob Baker",
      "affiliation=Univ B, Town, Country Correspondence to",
      "email=bob.baker@b.org",
    ],
  ]);
}

/// A label word and brackets the reading's run takes (`… Country (E-mail: \texttt{ann.able@a.org})`, `…, (E-mail: …)`;
/// 2311.02475, RT1 rows 47, 49, 52, 57): the entry without them or the comma before them, the e-mail named by the label
/// (`E-mail: `), in its typewriter font. RP1's draft left the `)` and named the e-mail `Email: `; LO4 `=2` (the Kind step
/// after the flush) and the switch off agree.
#[test]
fn an_address_s_label_and_brackets_go_with_it() {
  as_off(repro!("emit_an_address_label_and_brackets_go_with_it"), &[
    vec![
      "Ann Able",
      "affiliation=Department of X, Univ A, City, Country",
      "email=ann.able@a.org",
    ],
    vec![
      "Bob Baker",
      "affiliation=Lab Y, Univ B, Town, Country",
      "email=bob.baker@b.org",
    ],
  ]);
  let (_, xml) = run(repro!("emit_an_address_label_and_brackets_go_with_it"), 2);
  assert!(
    xml.contains(r#"<contact name="E-mail: " role="email"><text font="typewriter""#),
    "{xml}"
  );
}

/// `tex` clean at `=2`, its creators' XML (ids aside) `want`.
fn emitted(tex: &str, want: &[&str]) {
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators_xml(&xml), want, "{xml}");
}

/// A label printed before an entry's address goes with its run (RT1 rows 64-67, 72: the Kind step for an address line's
/// labelled-line rule, `insert::address_line_cut`, read in the cut — `writer::address_cut::labelled`): authblk's unmarked
/// `\affil[ ]{Corresponding author: \textit{a@x}}` whole — the e-mail named `Corresponding author: `, the role-note step's
/// correspondence note of its owner (2305.02090, 2404.05566, whose words went to another author; 2107.10505, 1510.03749) —,
/// a second line `Univ Z\\ Correspondence: c@z` with its break (1807.09187, 2008.08137's `Contact e-mail:`); a marked
/// `\affil[*]` stays the legend its carriers' note is made of, its mark shown (1812.03816, 2004.06340, 2012.13817). Red (P1b):
/// "Corresponding author" the last author's affiliation, "Univ Z Correspondence" Cy's, the e-mail `Email: `.
#[test]
fn a_label_before_an_address_goes_with_its_run() {
  emitted(
    repro!("emit_an_address_label_before_it_goes_with_its_run"),
    &[
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ X</contact>\n    <note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" \
     role=\"thanks\">Corresponding author</note>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text \
     font=\"italic\">ann@x.org</text></contact>\n  </creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact>\n    <note class=\"ltx_note_frontmatter \
     ltx_thanks_correspondence\" mark=\"*\" role=\"thanks\">Corresponding author</note>\n    <contact \
     name=\"Email:\u{a0}\" role=\"email\">bob@x.org</contact>\n  </creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cy Cole</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n    <contact name=\"Correspondence: \" \
     role=\"email\">cy@z.org</contact>\n  </creator>",
    ],
  );
}

/// A cut address is its owner's e-mail as printed (RT1 row 56, the switch-off writer's wrapper): the call wrapping it alone
/// whole — `\href{mailto:a@x}{a@x}` in an `\affil` and in a `\thanks` (2608.15371) —, else its own tokens in the font it
/// prints in: of a wrapper holding more (`\textit{Univ Z, b@z, Dept W}`, 2008.08137), of its `@` where a note's ink switches
/// font inside it (`\thanks{\IEEEauthorrefmark{1} … - \texttt{E-Mail: a@x}}`, 2405.06709; `events::InkTable::fonts`). Red
/// (P1b): each e-mail plain text, the link and the fonts lost.
#[test]
fn a_cut_address_keeps_its_link_and_font() {
  emitted(repro!("emit_a_cut_address_keeps_its_link_and_font"), &[
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ X, Dept Y</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><ref \
     class=\"ltx_href\" href=\"mailto:ann.able@x.org\">ann.able@x.org</ref></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text font=\"italic\">Univ Z, Dept W</text></contact>\n    \
     <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"italic\">bob.baker@z.org</text></contact>\n  \
     </creator>",
  ]);
  emitted(repro!("emit_a_cut_address_in_a_note_keeps_its_font"), &[
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"> Ann Able is with Univ A, City</contact>\n    <contact name=\"E-Mail: \" \
     role=\"email\"><text font=\"typewriter\">ann.able@a.org</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Bob Baker is with Univ B, Town</contact>\n    <contact \
     name=\"Email:\u{a0}\" role=\"email\"><ref class=\"ltx_href\" \
     href=\"mailto:bob.baker@b.org\">bob.baker@b.org</ref></contact>\n  </creator>",
  ]);
}

/// A cut that takes an address and the separator after it takes what prints nothing up to the kept text after it, never a
/// mark leading that text: `\affil[1]{Univ X, {\tt a@x}, $^1$Dept of Physics}` keeps its `¹` (review revP1b S1; a mark
/// leading the address side goes with the cut, `\\$^*\,$a@x`, 2506.02272). Red (P1b, off): "Univ X, Dept of Physics", the
/// mark lost, uncounted.
#[test]
fn a_mark_after_a_cut_address_stays() {
  emitted(repro!("emit_a_mark_after_a_cut_address_stays"), &[
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ X, <sup>1</sup>Dept of Physics</contact>\n    <contact name=\"Email:\u{a0}\" \
     role=\"email\"><text font=\"typewriter\">ann.able@x.org</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Z</contact>\n  </creator>",
  ]);
}

/// An entry whose lines no balanced run holds (`\\begin{small} $^{b}$Univ B, Lab C.\\end{small} \\\\ \\begin{small} Road 2,
/// Town. bob@b.org \\end{small}`, 1312.5468) is written line by line without the address run the cut takes inside its last
/// line (`writer::emit::lines_run`), its small type and its break kept, each e-mail in the small type its `@` prints in. Red
/// (P1b): the entry its plain text, counted (`Warning:unexpected:frontmatter`), the e-mails plain.
#[test]
fn a_cut_entry_over_lines_keeps_its_markup() {
  emitted(repro!("emit_a_cut_entry_over_lines_keeps_its_markup"), &[
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\"><text fontsize=\"90%\"> Univ A. City.</text></contact>\n    <contact \
     name=\"Email:\u{a0}\" role=\"email\"><text fontsize=\"90%\">ann@a.org</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\"><text fontsize=\"90%\">Univ B, Lab C.<break/>Road 2, \
     Town.</text></contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text \
     fontsize=\"90%\">bob@b.org</text></contact>\n  </creator>",
  ]);
}
