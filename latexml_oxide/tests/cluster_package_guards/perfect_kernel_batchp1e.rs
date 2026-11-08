//! Guards for P1b (E1, the strict port of the switch-off writer's cut, RT1's rows 54-58): an address inside an entry's
//! text is its owner's e-mail at `LATEXML_FM_EMIT=2` as with the switch off (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980 P1b): the reading's take-out (`Resolver::take_out_entry_addresses`) is written from the
//! entry's own tokens without the runs its cut covers, at the positions the lexer recorded for each character of its inks
//! (`Event::ink_at`, `writer::address_cut`), each address the e-mail of the owner the reading gives it, as printed. One test
//! per way the emitter writes an entry: a note's argument (`\thanks`, a font group and a bracketed address), an API call's
//! content (llncs's `\institute`, authblk's `\affil`, a local-part list), a block line read on with an address line (the
//! entry's own span), and a label word with brackets (the e-mail's name; RP1's draft regression, 2311.02475). Tests and
//! repros from RP1's draft `pk/P1` (645e93227c), re-pointed.

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
