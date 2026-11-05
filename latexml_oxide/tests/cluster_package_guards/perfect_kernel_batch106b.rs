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

/// Among `$^{n}$` marks a reference mark is the symbol it prints (2312.10807's `Oier Mees$^3$\IEEEauthorrefmark{1}`,
/// "Mees3∗"): Ann's and Bob's `∗` take the `$^{*}$` contribution legend, Cat's `†` the correspondence one, and Cat no
/// longer takes Univ B by her refmark's number 2 (2409.20550, 2411.01611).
#[test]
fn a_refmark_among_digit_marks_is_the_symbol_it_prints() {
  assert_eq!(
    converted(repro!("ieee_refmark_mixed_with_digit_marks"), 0),
    r#"<title>Mixed marks</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Equal contribution</note> </creator> <creator before=" " role="author"> <personname>Bob Bell<sup xml:id="id3">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, City A</contact> <contact name="Affiliation: " role="affiliation">Univ B, City B</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id4"><sup xml:id="id4.1">*</sup>Equal contribution</note> </creator> <creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id5">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ C, City C</contact> <note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks" xml:id="id6"><sup xml:id="id6.1">†</sup>Corresponding author: Cat Cole <text font="typewriter" xml:id="id6.2">cat@c.edu</text></note> </creator> <creator before=" " role="author"> <personname>Dan Dorn</personname> <contact name="Affiliation: " role="affiliation">Univ B, City B</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
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

/// A URL alone on a block's line is the block's URL contact, once (2511.18438); a membership grade line is its carrier's
/// membership contact (2310.02678, 2505.15137); an affiliation set as a link printing no URL stays one (1812.03371). Two
/// shapes of one Kind step and its control.
#[test]
fn a_link_line_and_a_grade_line_are_no_affiliations() {
  assert_eq!(
    converted(repro!("legend_kind_link_line_is_the_blocks"), 0),
    r#"<title>Link line</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Bell<sup xml:id="id2">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator role="author"> <contact name="URL: " role="url"><break/><ref class="ltx_nolink ltx_url" font="typewriter" href="https://github.com/example/project">https://github.com/example/project</ref></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("legend_kind_membership_grade_line"), 0),
    r#"<title>Grade line</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation"><ref class="ltx_href" href="https://a.edu">Univ A, City A</ref></contact> </creator> <creator before=" " role="author"> <personname>Bob Bell</personname> <contact name="Affiliation: " role="affiliation">Univ B, City B</contact> <contact role="membership">Senior Member, IEEE</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  // (round 2, review rev106b S1: a URL line a mark labels is its owners' URL contact, each, not the block's)
  assert_eq!(
    converted(repro!("legend_kind_marked_link_line_stays_its_owners"), 0),
    r#"<title>Link lines</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="URL: " role="url"><ref class="ltx_nolink ltx_url" font="typewriter" href="https://a.edu">https://a.edu</ref></contact> </creator> <creator before=" " role="author"> <personname>Bob Bell<sup xml:id="id2">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole<sup xml:id="id3">*</sup></personname> <contact name="URL: " role="url"><ref class="ltx_nolink ltx_url" font="typewriter" href="https://a.edu">https://a.edu</ref></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  // (S3: a position at an institute is no grade; a URL line one author holds is that author's, beside an institution
  // it stays in the affiliation)
  assert_eq!(
    converted(repro!("legend_kind_position_is_no_grade"), 0),
    r#"<title>Link and grade</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, <ref class="ltx_url" font="typewriter" href="https://a.edu">https://a.edu</ref></contact> </creator> <creator before=" " role="author"> <personname>Bob Bell</personname> <contact name="Affiliation: " role="affiliation">Senior Fellow, IAS</contact> </creator> <creator before=" " role="author"> <personname>Cat Cole</personname> <contact name="URL: " role="url"><ref class="ltx_url" font="typewriter" href="https://www.c-lab.org">https://www.c-lab.org</ref></contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
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

/// Two printed lines that share their text are two entries, both kept: an article block's second "Groningen"
/// (0808.4092, 1106.1785), amsart's e-mails differing in case only and an address written again with a footnote of its
/// own (review rev106b B1, probes b3, b2). The drop reads provenance, never text alone.
#[test]
fn two_printed_lines_alike_both_stay() {
  let line =
    |text: &str| format!(r#"<contact name="Affiliation: " role="affiliation">{text}</contact>"#);
  let lines: String = [
    "Dept. of Physics,",
    "Univ A,",
    "Groningen",
    "and",
    "Dept. of Math,",
    "Univ B,",
    "Groningen",
  ]
  .iter()
  .map(|text| line(text))
  .collect::<Vec<_>>()
  .join(" ");
  assert_eq!(
    converted(repro!("contact_twice_two_lines_alike_both_stay"), 0),
    format!(
      r#"<title>Two addresses</title> <creator role="author"> <personname>Ann Able</personname> {lines} </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
    )
  );
  assert_eq!(
    converted(repro!("contact_twice_two_calls_alike_amsart"), 0),
    r#"<title>Two calls alike</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Univ A, City A</contact> <contact name="Email address: " role="email"><text font="typewriter" xml:id="id1">Ann.Able@x.org</text></contact> <contact name="Address: " role="address">Univ A, City A</contact> <contact name="Email address: " role="email"><text font="typewriter" xml:id="id2">ann.able@x.org</text></contact> </creator> <pubnote>On leave.</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
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
  let xml = &xmls[0];
  let start = xml.find("<title>").expect("a title");
  let end = xml.find("<para").expect("a paragraph");
  assert_eq!(
    &xml[start..end],
    r#"<title>Star marks</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A</contact> <note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id2"><sup xml:id="id2.1">*</sup>Equal contribution</note> </creator> <creator before=" " role="author"> <personname>Bob Bell<sup xml:id="id3">†</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B</contact> </creator> "#
  );
}
