//! Red/green guards for batch IMG1: an image leading a URL on a later line of `\author` is that URL contact's, kept in
//! place (`events::place_pictures` grows the contact's span over it); a picture no written item holds is counted
//! (`resolver::words::Written::unwritten_pictures`, `emit::warn_unwritten_pictures`): an image has no word for the word
//! check to miss; each item holds what it writes, its entry's span (`resolver::words::entry_span_in`, IMG1b).
//! OXIDIZED_DESIGN_DIVERGENCES #989.
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
/// contact named "Web: ", the image counted. Distinct: the emit writes the line as one affiliation holding the image (its
/// entry's span holds it, IMG1b), and the link label pass (`insert::kind_link_and_grade_contacts`) strips it with the
/// label's text, counted there. A LaTeX picture holding the image counts once, a `ltx:graphics` inside a `ltx:picture` its
/// picture's (review revIMG1b finding 3; IMG1b: "drops 2"). Repros
/// sectioning-frontmatter/fm_image_before_labelled_url_is_counted,
/// fm_picture_holding_image_before_labelled_url_is_counted_once.
#[test]
fn image_after_a_url_label_is_counted() {
  for tex in [
    repro!("fm_image_before_labelled_url_is_counted"),
    repro!("fm_picture_holding_image_before_labelled_url_is_counted_once"),
  ] {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
    assert!(
      log.contains(
        "Warning:unexpected:frontmatter The frontmatter's link label pass drops 1 picture(s) printed between the label \
         'Web:' and its link; they are lost"
      ),
      "{log}"
    );
    assert_eq!(
      creators(&xml),
      r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University</contact>
    <contact name="Web: " role="url"><ref class="ltx_url" font="typewriter" href="https://example.org/x">https://example.org/x</ref></contact>
  </creator>"##
    );
  }
}

/// An image after a run of text and before a URL on one later line, the line read as one affiliation holding both
/// (`Zebra University, \includegraphics{…} \url{…}`, with `\quad` and with no separator; review revIMG1 finding 1; IMG1:
/// the same XML as int157a2 but one false "they are lost" Warning, the check reading the affiliation's source span, narrower
/// than the entry it writes): the whole creator, 0 Warnings. Distinct: the check's holding span is the item's written one
/// (`resolver::words::entry_span_in`), not the join's. Repros
/// sectioning-frontmatter/fm_image{,_quad,_glued}_inside_affiliation_before_url_is_kept.
#[test]
fn image_inside_an_affiliation_before_a_url_is_kept() {
  let affiliation = |between: &str| {
    format!(
      r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University{between}<graphics graphic="example-image" options="width=8.5359pt,keepaspectratio=true" xml:id="g1a"/> <ref class="ltx_url" font="typewriter" href="https://example.org/x">https://example.org/x</ref></contact>
  </creator>"##
    )
  };
  for (tex, between) in [
    (
      repro!("fm_image_inside_affiliation_before_url_is_kept"),
      ", ",
    ),
    (
      repro!("fm_image_quad_inside_affiliation_before_url_is_kept"),
      " \u{2003}",
    ),
    (
      repro!("fm_image_glued_inside_affiliation_before_url_is_kept"),
      " ",
    ),
  ] {
    assert_eq!(creators(&convert_clean(tex)), affiliation(between));
  }
}

/// An image parted from a URL by fill glue (review revIMG1 finding 3, review revIMG1b finding 1; int157a2: dropped, 0
/// Warnings; IMG1b: `\hfill`'s dropped, 1 Warning, the other fill glue joined): pdflatex sets the image at the line's left
/// edge and the link at its right, a placement detail, so the image stays the line's — the url contact holding the image
/// then the link, 0 Warnings, for `\hfill` (the lexer's fill gap), `\hspace{\fill}` and `\hskip 0pt plus 1fil` (glue the
/// lexer reads as a plain gap). Distinct: the gap test of `events::place_pictures` passes any gap, fill glue included.
/// Repros sectioning-frontmatter/fm_image_{hfill,hspace_fill,hskip_fil}_before_url_is_kept.
#[test]
fn image_parted_from_a_url_by_fill_is_kept() {
  // (the contact as written: a space between the image and the link for `\hfill`, nothing for the other glue)
  for (tex, between) in [
    (repro!("fm_image_hfill_before_url_is_kept"), " "),
    (repro!("fm_image_hspace_fill_before_url_is_kept"), ""),
    (repro!("fm_image_hskip_fil_before_url_is_kept"), ""),
  ] {
    assert_eq!(
      creators(&convert_clean(tex)),
      format!(
        r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation">Zebra University, Department of Very Long Names and Places</contact>
    <contact name="URL: " role="url"><graphics graphic="example-image" options="width=28.45274pt,keepaspectratio=true" xml:id="g1a"/>{between}<ref class="ltx_url" font="typewriter" href="https://example.org/x">https://example.org/x</ref></contact>
  </creator>"##
      )
    );
  }
}
