//! Red/green guards for batch IMG1: an image leading a URL on a later line of `\author` is that URL contact's, kept in
//! place (`events::place_pictures` grows the contact's span over it); a picture no written item holds is counted
//! (`resolver::words::Written::unwritten_pictures`, `emit::warn_unwritten_pictures`): an image has no word for the word
//! check to miss. OXIDIZED_DESIGN_DIVERGENCES #989.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batchcr2::{convert_clean, elements},
  perfect_kernel_batchcr8::without_candidates,
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

/// The Warning of a block's pictures no written item holds.
const ONE_PICTURE: &str = "Warning:unexpected:frontmatter The frontmatter emitter's reading holds no place for 1 \
                           picture(s) its author block prints; they are lost";

/// Each creator of `xml`, whole, its graphics' TeX-tree paths aside.
fn creators(xml: &str) -> String {
  elements(xml, "creator")
    .iter()
    .map(|c| without_candidates(c))
    .collect::<Vec<_>>()
    .join("\n")
}

/// A repro's creators, its conversion free of errors, with exactly one Warning: the picture's.
fn counted(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains(ONE_PICTURE), "{log}");
  creators(&xml)
}

/// An image leading a URL on a later line (2503.08161's `\includegraphics{hf-logo.png} \texttt{\small\url{…}}`; reviews
/// revCR8b finding 1, revCR8c finding 2; int157a2: the image dropped, 0 Warnings): one URL contact holding the image, then
/// the link, 0 Warnings. The URL join of `place_pictures`. Repro sectioning-frontmatter/fm_image_before_url_is_in_the_url.
#[test]
fn image_before_a_url_is_in_the_url() {
  let xml = convert_clean(repro!("fm_image_before_url_is_in_the_url"));
  assert_eq!(
    creators(&xml),
    r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University</contact>
    <contact name="URL: " role="url"><graphics graphic="example-image" options="width=10.243pt,keepaspectratio=true" xml:id="g1a"/> <ref class="ltx_nolink ltx_url" font="typewriter" fontsize="90%" href="https://example.org/x">https://example.org/x</ref></contact>
  </creator>"##
  );
}

/// An image leading a mark-led affiliation line (`\includegraphics{…} $^{1}$Zebra University`; int157a2: dropped, 0
/// Warnings): the reading unchanged, the image counted by one Warning. Distinct: no placement, the coverage Warning alone.
/// Repro sectioning-frontmatter/fm_image_before_mark_is_counted.
#[test]
fn image_before_a_mark_is_counted() {
  assert_eq!(
    counted(repro!("fm_image_before_mark_is_counted")),
    r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University</contact>
  </creator>"##
  );
}

/// An image leading a run printing an address (`\includegraphics{…} \texttt{ann@x.org}`; int157a2: joined into the run,
/// written by the e-mail's text, dropped, 0 Warnings): the e-mail in its typewriter type, the image counted. Distinct: the
/// run join's address exclusion in `place_pictures`. Repro sectioning-frontmatter/fm_image_before_address_is_counted.
#[test]
fn image_before_an_address_is_counted() {
  assert_eq!(
    counted(repro!("fm_image_before_address_is_counted")),
    r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University</contact>
    <contact name="Email: " role="email"><text font="typewriter" xml:id="id1">ann@x.org</text></contact>
  </creator>"##
  );
}

/// An image between a label and its URL (`Web: \includegraphics{…} \url{…}`; int157a2: dropped, 0 Warnings): the URL
/// contact named "Web: ", the image counted. Distinct: the URL join grows the contact's span over the image, but the item
/// written is the label's entry, whose span does not hold it — the Warning reads the written items, not the join.
/// Repro sectioning-frontmatter/fm_image_before_labelled_url_is_counted.
#[test]
fn image_after_a_url_label_is_counted() {
  assert_eq!(
    counted(repro!("fm_image_before_labelled_url_is_counted")),
    r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University</contact>
    <contact name="Web: " role="url"><ref class="ltx_url" font="typewriter" href="https://example.org/x">https://example.org/x</ref></contact>
  </creator>"##
  );
}
