//! Red/green guards for batch 106b (census132 LEGEND, the KIND-misc duplicate pocket; OXIDIZED_DESIGN_DIVERGENCES #968):
//! IEEEtran's `\IEEEauthorrefmark{n}` prints the n-th footnote symbol and pairs as it prints (ieeetran_cls.rs
//! `\lx@author@refmark@symbol` and `printed_refmarks`, author_macros.rs `\lx@author@refmark@printed`, relocate.rs
//! `sup_marks`), and the flush-time Kind steps for a link line and a membership grade and the drop of a contact held twice
//! (frontmatter/insert.rs `kind_link_and_grade_contacts`, `drop_repeated_contacts`). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, and asserts the
//! frontmatter it makes whole. Round 2 (review rev106b): the drop of a contact held twice reads its entry's
//! provenance — a call placed twice, a label's line, a late restatement — never its text alone (B1); a URL line a mark
//! labels stays its owners' (S1); a refmark printing a legend's glyph under another number keeps it (S2); a grade is
//! one in a society (S3).
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
  perfect_kernel_batch99b::converted,
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

/// A block marked with reference marks alone pairs as before, the names printing the symbols (`∗‡`), and its marked
/// legend line is its carriers' note marked `‡`, the symbol the names print (2311.03600, 2410.12184, 2205.00125).
#[test]
fn refmark_blocks_pair_and_print_their_symbols() {
  assert_eq!(
    converted(repro!("ieee_refmark_blocks_pair_by_symbol"), 0),
    r#"<title>Block marks</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup><sup xml:id="id2">‡</sup></personname> <note class="ltx_note_frontmatter ltx_thanks_funding" role="thanks" xml:id="id3">Supported by Grant 1.</note> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="‡" role="thanks" xml:id="id4">These authors contributed equally</note> </creator> <creator before=" " role="author"> <personname>Bob Bell<sup xml:id="id5">†</sup><sup xml:id="id6">‡</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B, City B</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="‡" role="thanks" xml:id="id7">These authors contributed equally</note> </creator> <creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id8">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A creator holding one entry twice keeps the first: a marked list's line repeated under a later name, both lines of
/// the label its mark requests (1204.6725; the other routes: a call placed twice, 1207.1783, a late restatement,
/// 2311.16348).
#[test]
fn a_contact_held_twice_keeps_the_first() {
  assert_eq!(
    converted(repro!("contact_twice_keeps_the_first"), 0),
    r#"<title>Contact twice</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> <contact name="Affiliation: " role="affiliation">Univ B, City B</contact> </creator> <creator before=" " role="author"> <personname>Bob Bell</personname> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A call made after `\maketitle` placed the frontmatter, restating an entry its author holds, is that entry: a
/// supplement's `\author{…}\affiliation{…}` written again, which no title prints (2311.16348, 1407.0849).
#[test]
fn a_late_restatement_is_held_once() {
  // (the frontmatter only: the late `\author{Ann Able}` prints its name in the body, "Text. Ann Able Supplement.", on
  // int134b alike, where pdflatex prints nothing — OD #969's residual)
  let xml = converted(repro!("contact_twice_late_restatement"), 0);
  assert_eq!(
    &xml[..xml.find(" <para").expect("a paragraph")],
    r#"<title>Late restatement</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> </creator>"#
  );
}

/// `\IEEEauthorrefmark{1}\IEEEauthorrefmark{*}`: the `{1}` prints the `∗` an `\IEEEauthorrefmark{*}` legend prints,
/// and stays on the name, the mark of its own line, Univ A; the `{*}` pairs with the legend's note. Its two
/// "Missing number" Warnings are TeX's (IEEEtran.cls's `\ifcase` on `*`; pdflatex errs and prints the `*`). Review
/// rev106b S2, probe c4.
#[test]
fn a_refmark_printing_a_legends_glyph_keeps_it() {
  let tex = repro!("ieee_refmark_beside_a_starred_legend");
  let mut xmls = Vec::new();
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
    assert_eq!(
      log.matches("Missing number, treated as zero").count(),
      2,
      "{log}"
    );
    xmls.push(xml.split_whitespace().collect::<Vec<_>>().join(" "));
  }
  assert_eq!(xmls[0], xmls[1]);
}
