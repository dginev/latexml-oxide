//! overpic: render the `\includegraphics` background + `\put` overlays as a
//! populated `<ltx:picture>`. The prior binding faithfully mirrored Perl
//! (empty `<ltx:picture tex=.../>` for the unwired LaTeX-image renderer), so
//! it emitted NO graphic and dropped `#body` — 37 arXiv papers reported the
//! missing overpic figure. See `overpic_sty.rs` (the surpass-perl divergence).
use crate::cluster::convert_to_xml;

#[test]
fn overpic_emits_populated_sized_picture_with_graphic_and_overlays() {
  let xml = convert_to_xml("tests/cluster_regressions/overpic_render.tex");
  // Emitted and SIZED (the empty binding produced a `<ltx:picture/>` with no
  // `unitlength`).
  assert!(xml.contains("<picture"), "no picture element:\n{xml}");
  assert!(
    xml.contains("unitlength="),
    "picture not sized (no unitlength):\n{xml}"
  );
  // THE FIX: the `\includegraphics` background renders as a nested graphic
  // (the empty binding emitted no graphic at all). Holds whether or not
  // `example-image`'s file resolves — the element is a core-stage product.
  assert!(
    xml.contains(r#"graphic="example-image""#),
    "overpic dropped the background graphic:\n{xml}"
  );
  // The body `\put` overlays land as picture content (the empty binding
  // dropped `#body`).
  assert!(xml.contains("OVLABELA"), "overlay A was dropped:\n{xml}");
  assert!(xml.contains("OVLABELB"), "overlay B was dropped:\n{xml}");
}

/// A missing/unmeasurable image with no size option must NOT raise
/// `Illegal \divide by 0` (common on arXiv, where submissions omit referenced
/// images). `convert_to_xml` asserts zero `Error:` markers; the pre-guard
/// binding raised `Error:misdefined:0`.
#[test]
fn overpic_missing_natural_size_image_does_not_divide_by_zero() {
  let xml = convert_to_xml("tests/cluster_regressions/overpic_missing_image.tex");
  assert!(
    xml.contains("<picture"),
    "no picture for missing image:\n{xml}"
  );
  assert!(
    xml.contains("OVLABELC"),
    "overlay dropped for missing image:\n{xml}"
  );
}
