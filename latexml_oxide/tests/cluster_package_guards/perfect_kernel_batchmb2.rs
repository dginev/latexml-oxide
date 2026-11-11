//! Guards for MB2, the entry-name join's glued-label path (OXIDIZED_DESIGN_DIVERGENCES #992; review revMB1 S1): a line
//! whose e-mail label is glued to its address (`Email:zz@x.org`) has that label split off as a spaced one is (GL1), and
//! the class's name of the entry joins before it. The join's two other shapes, aa's single space and its colon (#944, revMB1
//! N1, P2), are pinned by `perfect_kernel_batchmb1::labelled_line_is_named_after_the_class` and
//! `perfect_kernel_batch97b::aa_mail_is_labelled_as_its_copy_prints_it`.
//!
//! Each converts its repro and asserts the exact Error and Warning counts, every creator's name and contacts in order,
//! and the e-mail contact (and the note it gives) whole.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batchz4::run,
  perfect_kernel_batchz18o::{assert_creators, email_contacts},
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

/// An entry the class names holding a line whose label is glued to its address names its contact as the spaced line
/// does: the label split off (`email_line_label`, the whole line reading the same addresses as the rest) and joined
/// after the class's name. aa's `\mail{Email:zz@x.org}` is "Correspondence to: Email: " (a plain space, the spaced
/// `\mail{Email: zz@x.org}`'s bytes) holding `zz@x.org` (the PDF's
/// "Correspondence to: Email:zz@x.org"); REVTeX's `\altaffiliation[Corresponding author: ]{Email:zz@x.org}` gives Ann's
/// correspondence note "Corresponding author" before the e-mail `zz@x.org` named "Email: ". Red on int157a: the label
/// kept in the content (`Email:zz@x.org`, a `mailto:Email:zz@x.org`), named "Correspondence to: " and "Email: ";
/// before MB1 the class's name printed nowhere. Synthetic, review revMB1 S1 (probes `a1`, `r2`); the shape's witnesses
/// hep-ph9606235, astro-ph0001190, hep-ph0411269, 2206.00543.
#[test]
fn glued_label_line_names_its_contacts() {
  let (log, xml) = run(repro!("aa_mail_glued_email_label_takes_the_class_name"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[&[
    "A. Smith",
    "affiliation=Univ. X, Y",
    "email=zz@x.org",
  ]]);
  assert_eq!(
    email_contacts(&xml),
    ["<contact name=\"Correspondence to: Email: \" role=\"email\">zz@x.org</contact>"],
    "{xml}"
  );

  let (log, xml) = run(repro!(
    "revtex_altaffiliation_glued_email_label_takes_the_class_name"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able", "affiliation=Univ A", "email=zz@x.org"],
    &["Bob Baker", "affiliation=Univ A"],
  ]);
  assert_eq!(
    email_contacts(&xml),
    ["<contact name=\"Email:\u{a0}\" role=\"email\">zz@x.org</contact>"],
    "{xml}"
  );
  let note = "<note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" role=\"thanks\" \
              xml:id=\"id1\">Corresponding author</note>";
  assert_eq!(xml.matches("<note ").count(), 1, "{xml}");
  assert!(
    xml.contains(&format!("{note}\n    <contact name=\"Email:\u{a0}\"")),
    "{xml}"
  );
}
