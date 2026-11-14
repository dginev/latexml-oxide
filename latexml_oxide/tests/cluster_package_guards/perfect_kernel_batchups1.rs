//! Guards for UPS1, the port of the upstream LaTeXML master commits since #2833 that latexml-oxide lacked
//! (`docs/parity/UPSTREAM_SYNC_2833_to_0db6f47e.md`). Item 1 (#2820): `\fcolorbox`, amsmath's text `\boxed` and a
//! listing's frame are padded as TeX draws them — `\fboxsep` (the `\fbox` frame they are), a listing's `framesep` —
//! and a listing's `frame=` letters (`tb`, `trBL`) frame those sides. Item 3 (#2875): `<mtable align>` of a top or
//! bottom array is its first or last row's baseline. Item 4 (#2845): the regenerated metrics size the lozenges.
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

/// boxes-groups/fboxsep_pads_fcolorbox_boxed_and_listing_frames: xcolor's `\fcolorbox` (0pt under `\fboxsep=0pt`),
/// text `\boxed`, and the listing frames `single`, `tb`, `trBL` (its own `framesep`), `leftline`, `none`, `\lstinline`
/// under `frame=single`. Before: no padding, `tb`/`trBL` unframed (with #2829's CSS padding gone).
#[test]
fn fboxsep_pads_fcolorbox_boxed_and_listing_frames() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/fboxsep_pads_fcolorbox_boxed_and_listing_frames.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "text",
    &[r##"backgroundcolor="#FF0000""##],
    r##"<text backgroundcolor="#FF0000" cssstyle="padding:3.0pt" framecolor="#0000FF" framed="rectangle">fc</text>"##,
  );
  assert!(
    xml.contains(
      r##"<text backgroundcolor="#FF0000" cssstyle="padding:0.0pt" framecolor="#0000FF" framed="rectangle">fz</text>"##
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<Math cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle" mode="display" text="t = 1""##),
    "{xml}"
  );
  for (data, frame) in [
    (
      "YT0x",
      r##"cssstyle="padding:5.0pt" data="YT0x" dataencoding="base64" datamimetype="text/plain" framecolor="#000000" framed="rectangle""##,
    ),
    (
      "Yj0y",
      r##"cssstyle="padding:5.0pt" data="Yj0y" dataencoding="base64" datamimetype="text/plain" framecolor="#000000" framed="top bottom""##,
    ),
    (
      "Yz0z",
      r##"cssstyle="padding:2.0pt" data="Yz0z" dataencoding="base64" datamimetype="text/plain" framecolor="#000000" framed="top bottom left right""##,
    ),
    (
      "ZD00",
      r##"cssstyle="padding:5.0pt" data="ZD00" dataencoding="base64" datamimetype="text/plain" framecolor="#000000" framed="left""##,
    ),
    (
      "ZT01",
      r##"data="ZT01" dataencoding="base64" datamimetype="text/plain""##,
    ),
  ] {
    let tag = format!(r#"<listing class="ltx_lstlisting" {frame}>"#);
    assert_eq!(xml.matches(&tag).count(), 1, "{data}: {tag}\n{xml}");
  }
  assert!(
    xml.contains(r#"<p>x <text class="ltx_lstlisting"><text class="ltx_lst_identifier">f</text>=6</text> y</p>"#),
    "{xml}"
  );
}

/// alignment/mtable_align_is_the_first_or_last_baseline (#2875): `array[t]` sits on its first row's baseline,
/// `array[b]` on its last's, `[c]` on the axis (MathML Core keeps no `align` on `<mtable>`: attribute residue).
#[test]
fn mtable_align_is_the_first_or_last_baseline() {
  let html = latexml::util::test::convert_to_html_page(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/mtable_align_is_the_first_or_last_baseline.tex"
  ))
  .expect("HTML");
  let tables: Vec<&str> = html
    .match_indices("<mtable")
    .map(|(i, _)| &html[i..i + html[i..].find("</mtable>").unwrap() + 9])
    .collect();
  assert_eq!(
    tables,
    [
      r#"<mtable align="baseline1" displaystyle="true" rowspacing="0pt"><mtr><mtd><mn>1</mn></mtd></mtr><mtr><mtd><mn>2</mn></mtd></mtr></mtable>"#,
      r#"<mtable align="baseline-1" displaystyle="true" rowspacing="0pt"><mtr><mtd><mn>3</mn></mtd></mtr><mtr><mtd><mn>4</mn></mtd></mtr></mtable>"#,
      r#"<mtable displaystyle="true" rowspacing="0pt"><mtr><mtd><mn>5</mn></mtd></mtr><mtr><mtd><mn>6</mn></mtd></mtr></mtable>"#,
    ],
    "{html}"
  );
}

/// fonts-nfss/lozenge_widths_are_msam_glyph_widths (#2845): amssymb's `\lozenge` (U+25CA) and `\blacklozenge`
/// (U+29EB) are msam10's glyphs, sized by its metrics (pdflatex 6.66669pt); before, no metric held those code points
/// and the cmr fallback gave 7.5pt.
#[test]
fn lozenge_widths_are_msam_glyph_widths() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/lozenge_widths_are_msam_glyph_widths.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>6.66669pt 6.66669pt</p>");
}
