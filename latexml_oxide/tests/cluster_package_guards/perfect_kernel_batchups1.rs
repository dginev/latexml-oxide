//! Guards for UPS1, the port of the upstream LaTeXML master commits since #2833 that latexml-oxide lacked
//! (`docs/parity/UPSTREAM_SYNC_2833_to_0db6f47e.md`). Item 1 (#2820): `\fcolorbox`, amsmath's text `\boxed` and a
//! listing's frame are padded as TeX draws them — `\fboxsep` (the `\fbox` frame they are), a listing's `framesep` —
//! and a listing's `frame=` letters (`tb`, `trBL`) frame those sides.
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
