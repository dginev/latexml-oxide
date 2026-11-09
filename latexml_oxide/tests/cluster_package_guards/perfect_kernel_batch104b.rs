//! Red/green guards for batch 104b: a raw class's `\and`-separated institute store, numbered by its place in that list
//! where the class's own title page prints it, goes entry n to the names marked n (census129 cluster 3 and the long
//! tail's Coverage cluster; OXIDIZED_DESIGN_DIVERGENCES #955). Code path: whether the class numbers the entries is what
//! one speculative run of its title page printed of the store set to an `\and` list of sentinels (frontmatter_stores.rs
//! `note_and_numbering`), read by the shared labeller (affiliations.rs `class_numbered_institutes`) on K11's store route
//! (`\lx@add@store@affiliations`) and the displaced route (`displaced_entries`; its guard is batch 99b's
//! `an_and_list_in_a_displaced_store_goes_by_the_class_numbers_or_stays_the_classs`). Each guard converts its repro
//! under `tools/perfect_kernel/repros/sectioning-frontmatter/` eager and `--streaming` (batch 99b's `converted`) and
//! asserts the frontmatter whole.
use super::perfect_kernel_batch99b::converted;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Control: a class whose title page prints its `\and` entries unnumbered (`\def\and{\unskip; \ignorespaces}`) — the
/// speculative run shows no number before them — keeps the store every author's, the numeric marks in the names, as
/// before.
#[test]
fn an_unnumbered_and_list_stays_every_authors() {
  assert_eq!(
    converted(
      repro!("raw_class_unnumbered_and_institutes_stay_every_authors"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id2">2</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole<sup xml:id="id3">1</sup></personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A class numbering its `\and` list whose title page the speculative run does not print (here `\AtEndDocument
/// {\institutename}`; raa.cls's title page, 2003.13998, 2511.10805, astro-ph0701846): the run's text holds no sentinel
/// entry, so it decides nothing and the class's `\institutename` is read as before — entry n still goes to the names
/// marked n. Read as "not numbered", every raa author held every institute (batch 104b's first A/B: 14 papers).
#[test]
fn a_numbered_and_list_the_run_does_not_print_keeps_its_numbers() {
  assert_eq!(
    converted(
      repro!("raw_class_numbered_institutes_the_run_does_not_print_keep_their_numbers"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1"><sup xml:id="p2.1.1"><text font="italic" xml:id="p2.1.1.1">1</text></sup> Univ A, Town</p> </para> <para class="ltx_noindent" xml:id="p3"> <p xml:id="p3.1"><sup xml:id="p3.1.1"><text font="italic" xml:id="p3.1.1.1">2</text></sup> Univ B, City</p> </para>"#
  );
}
