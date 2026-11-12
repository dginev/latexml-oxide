//! Guards for P26 (Option C P2-6): the stores' `\and` gates read an argument's separators by meaning, through the
//! binding-callable query (`observer/separators.rs` `argument_separators`, OXIDIZED_DESIGN_DIVERGENCES #995), not by a
//! literal `\and`'s spelling (`latexml_engine/src/frontmatter_stores.rs` `and_separators`, `holds_and_separators`,
//! `unnumbered_and_list`, `kernel_and_list`; the numbering gates before `note_and_numbering`). Code paths, one guard each:
//! a rerouted in-place setter's line (`class_contact`'s gate `kernel_and_list`: the kernel separator's meaning, trap T8),
//! a displaced store's `\and` list (`displaced_argument_in_doubt`'s `unnumbered_and_list`), and K11's numbered store
//! route (`\lx@add@store@affiliations`, the numbering gate). Each converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, and asserts the
//! frontmatter whole, with exact Error and Warning counts.
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

/// `tex` converted eager and `--streaming`, each with no Error and `warnings` Warnings of any kind (batch 99b's
/// `converted` counts only the unclaimed-text net's), the two XMLs the same; the XML from its `<title>` to its end, each
/// whitespace run one space.
#[track_caller]
pub(super) fn converted_any(tex: &str, warnings: usize) -> String {
  let mut xmls = Vec::new();
  for streaming in [false, true] {
    let (log, xml) = cli_conversion(tex, streaming);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{log}"
    );
    xmls.push(xml);
  }
  assert_eq!(xmls[0], xmls[1]);
  let xml = xmls[0].split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end().to_string()
}

/// A rerouted in-place setter's line (`class_contact`) is the class's where a separator of its own level means the
/// kernel's `\and` (" and ", a word the API's contact would invent where pdflatex sets columns): written (a pin, the
/// same on int158a), a `\let` copy, one a document macro holds, a copy saved before the document renewed `\and` (RED on
/// int158a: an API contact "… and …", the content check's Warning). A class's own `\and` (smfart's, 2508.07946; a pin)
/// and a renewed one are no kernel separator: the API's contact prints them. An `\and` inside a `\parbox` or a note is
/// no separator of the line (the query's nest 0): both lines are Ann Able's addresses (int158a kept them the class's
/// by spelling), the content check counting the kernel `\and`'s word in the box's table.
#[test]
fn an_in_place_line_is_the_classs_where_a_separator_means_the_kernels() {
  assert_eq!(
    converted(
      repro!("inplace_contact_kernel_and_line_stays_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <tabular vattach="top" xml:id="p1.1"> <tbody> <tr xml:id="p1.1.1"> <td align="center" xml:id="p1.1.1.1"><text fontsize="90%" xml:id="p1.1.1.1.1">Univ A, Town and Univ B, City</text></td> </tr> </tbody> </tabular> </para> <para xml:id="p2"> <p xml:id="p2.1">Body.</p> </para>"#
  );
  assert_eq!(
    converted(repro!("inplace_contact_class_and_line_is_the_apis"), 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Univ A, Town &amp; Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("inplace_contact_copied_or_held_kernel_and_stays_the_classs"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <tabular vattach="top" xml:id="p1.1"> <tbody> <tr xml:id="p1.1.1"> <td align="center" xml:id="p1.1.1.1"><text fontsize="90%" xml:id="p1.1.1.1.1">Univ A, Town and Univ B, City</text></td> </tr> </tbody> </tabular> </para> <para xml:id="p2"> <tabular vattach="top" xml:id="p2.1"> <tbody> <tr xml:id="p2.1.1"> <td align="center" xml:id="p2.1.1.1"><text fontsize="90%" xml:id="p2.1.1.1.1">Univ A, Town and Univ B, City</text></td> </tr> </tbody> </tabular> </para> <para xml:id="p3"> <p xml:id="p3.1">Body.</p> </para>"#
  );
  assert_eq!(
    converted(
      repro!("inplace_contact_kernel_and_copy_under_a_renewed_and"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Univ A, Town, Univ B, City</contact> </creator> <para xml:id="p1"> <tabular vattach="top" xml:id="p1.1"> <tbody> <tr xml:id="p1.1.1"> <td align="center" xml:id="p1.1.1.1"><text fontsize="90%" xml:id="p1.1.1.1.1">Univ C, Place and Univ D, Dale</text></td> </tr> </tbody> </tabular> </para> <para xml:id="p2"> <p xml:id="p2.1">Body.</p> </para>"#
  );
  let boxed = converted_any(
    repro!("inplace_contact_and_inside_a_box_or_note_parts_no_line"),
    1,
  );
  assert_eq!(
    boxed,
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Address: " role="address">Univ A, Town <tabular class="ltx_parbox" vattach="middle" width="85.4pt" xml:id="id1"> <tbody> <tr xml:id="id1.1"> <td align="center" xml:id="id1.1.1">Lab X and Lab Y</td> </tr> </tbody> </tabular></contact> <contact name="Address: " role="address">Univ B, City</contact> </creator> <pubnote role="thanks">Visiting <tabular vattach="middle" xml:id="id2"> <tbody> <tr xml:id="id2.1"> <td align="center" xml:id="id2.1.1">Lab Z and Lab W</td> </tr> </tbody> </tabular></pubnote> <para xml:id="p1"> <p xml:id="p1.1">Body.</p> </para>"#
  );
}

/// A displaced store's `\and` list the class's title page does not number is in doubt and stays the class's text where
/// it prints it (`displaced_argument_in_doubt`, astro-ph9805069, 1704.03333; batch 99b's guard pins the written shape),
/// its separators read by meaning: a `\csname and\endcsname` the title page prints as its own `\and` too (RED on
/// int158a: one contact "Univ A, Town and Univ B, City" under both authors, the word invented).
#[test]
fn a_displaced_and_list_is_read_by_meaning() {
  assert_eq!(
    converted(repro!("displaced_csname_and_institutes_stay_the_classs"), 0),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> </creator> <creator before=" " role="author"> <personname>Bob Baker</personname> </creator> <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="italic" fontsize="90%" xml:id="p1.1.1">Univ A, Town</text></p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1"><text font="italic" fontsize="90%" xml:id="p2.1.1">Univ B, City</text></p> </para> <para xml:id="p3"> <p xml:id="p3.1">Text.</p> </para>"#
  );
}

/// K11's store route (`\lx@add@store@affiliations`): a raw class's `\institute{A \and B \and C}` store whose title page
/// numbers its `\and` entries (sab.cls's `\refstepcounter{inst}`, 2308.05831; edpsci, baaa 2206.02830, comsis2
/// 2207.09249) is tried by the numbering run (`note_and_numbering`, its gate by meaning since P26), entry n going to the
/// names marked n. A pin, the same on int158a and P26: it pins the route only — its repro writes the `\and`, so no
/// input here shows the gate deciding by meaning differently from the spelling (revP26 N2).
#[test]
fn a_numbered_store_route_tries_the_numbering_by_meaning() {
  assert_eq!(
    converted(
      repro!("raw_class_refstepcounter_numbered_institutes_go_by_their_numbers"),
      0
    ),
    r#"<title>A Title</title> <creator role="author"> <personname>Ann Able</personname> <contact name="Affiliation: " role="affiliation">Univ A, Town</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id1">∗</sup></personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <creator before=" " role="author"> <personname>Cy Cole</personname> <contact name="Affiliation: " role="affiliation">Univ C, Place</contact> </creator> <creator before=" " role="author"> <personname>Dan Dole</personname> <contact name="Affiliation: " role="affiliation">Univ B, City</contact> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}
