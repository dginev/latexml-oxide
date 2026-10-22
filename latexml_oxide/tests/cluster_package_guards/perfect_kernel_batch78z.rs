//! Red/green guards for batch 78z (user ruling 2026-10-22 "fix to TeX's bp"): an EPS/PS graphic's box at digestion is
//! its `%%BoundingBox` in bp, as latex and pdflatex read it, not pixels at `DPI` 100 (Perl Util/Image.pm:271); the
//! first readable candidate sizes the box whatever its type; a raster keeps Perl's DPI rule (OXIDIZED_DESIGN_DIVERGENCES
//! #886, #337). Side item B (review rev78y2): ar5iv.sty's `classcaptionfonts` option, and float.sty's `\restylefloat`
//! beside IEEEtran's caption format. Each guard converts its pdflatex-checked repro under `tools/perfect_kernel/repros/`
//! and asserts the elements whole, after 0 errors and 0 warnings.
use latexml::util::test::{assert_element, convert_bytes_files_then, convert_files_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch78t::float_rows,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\sbox0{\includegraphics[…]{e.eps}}` (`%%BoundingBox: 0 0 40 20`) is 40bp = 40.15pt wide, under `scale=`, `width=`
/// alone, `height=` alone, `viewport=` and `angle=` too; a `%%HiResBoundingBox` beside the BoundingBox does not decide
/// (graphics.sty `\Gread@eps`, epstopdf); an extensionless `fig` beside `fig.pdf` (200bp) and `fig.ps` (40bp) takes the
/// `.pdf`'s box, as pdflatex reads `fig.pdf`; `\psfig` alike. latex + dvips: "natural 40.15pt 20.075pt." "scale
/// 22.68483pt 11.3424pt." "width 100.0pt 50.00186pt." "height 144.5733pt 72.26999pt." "viewport 20.075pt 10.03749pt."
/// "angle 20.075pt 40.15pt." "hires 200.75pt 100.375pt."; pdflatex 7sp under them and "pdf beside ps 200.7495pt
/// 100.37474pt." The PNG control (200 x 100 px) keeps Perl's 100 DPI (OD #337; pdflatex 200.7495pt at 72 DPI). Red
/// (int67a): "natural 28.908pt 14.454pt." … "pdf beside ps 28.908pt 14.454pt." "hires 143.8173pt 71.5473pt.". Witness
/// 1106.2745. Repro package-bindings/includegraphics_eps_box_is_its_bounding_box.
#[test]
fn includegraphics_eps_box_is_its_bounding_box() {
  let eps = repro!("package-bindings/eps_as_pdf_two.eps").as_bytes();
  let (log, xml, ()) = convert_bytes_files_then(
    repro!("package-bindings/includegraphics_eps_box_is_its_bounding_box.tex").as_bytes(),
    &[
      ("e.eps", eps),
      (
        "hr.eps",
        repro!("package-bindings/eps_bounding_box_hires.eps").as_bytes(),
      ),
      // (the `.ps` written first: a tmpfs lists a directory newest first)
      ("fig.ps", eps),
      (
        "fig.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf").as_bytes(),
      ),
      (
        "r.png",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/trimquad.png"),
      ),
    ],
    Some(PRELOAD),
    |_, _| (),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">natural 40.15pt 20.075pt.\nscale 22.68475pt 11.34238pt.\nwidth 100.0pt 50.0pt.\n\
     height 144.53998pt 72.26999pt.\nviewport 20.075pt 10.03749pt.\nangle 20.075pt 40.15pt.\nhires 200.75pt 100.375pt.\n\
     pdf beside ps 200.75pt 100.375pt.\npng 144.54pt 72.27pt.\npng scale 72.27pt 36.135pt.\npsfig 100.0pt 50.0pt.</p>",
  );
}

/// Three `\subfigure[…]{\psfig{figure=wide.eps}}` (150bp = 150.5625pt each, no size keys) are two and one in the 345pt
/// text width, as latex and pdflatex set them. Red (int67a): one row of three, each panel 108.405pt (the BoundingBox as
/// pixels at 100 DPI). OD #876, #886; witness 1106.2745 Figs 1-2, 7, 8, 10. Repro
/// captions-floats/psfig_natural_panels_are_as_wide_as_their_bounding_box.
#[test]
fn psfig_natural_panels_are_as_wide_as_their_bounding_box() {
  let (log, xml) = convert_files_with(
    repro!("captions-floats/psfig_natural_panels_are_as_wide_as_their_bounding_box.tex"),
    &[(
      "wide.eps",
      repro!("captions-floats/eps_bounding_box_150bp.eps"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(float_rows(&xml, "S0.F1"), "A B | C", "{xml}");
}

/// Every `<caption>` element of `xml` in document order.
fn captions(xml: &str) -> Vec<String> {
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(element) = latexml::util::test::xml_element(rest, "caption", &[]) {
    found.push(element);
    let at = rest.find("<caption").unwrap();
    rest = &rest[at + 1..];
  }
  found
}

/// `--preload=[classcaptionfonts]ar5iv.sty` sets the class caption fonts knob (caption_sty.rs `CLASS_CAPTION_FONTS`):
/// acmart sigconf's captions carry their bold label and text (acmart.cls:960-961; pdflatex LinLibertineTB), with ar5iv's
/// `ids` (`xml:id` on the texts); `noclasscaptionfonts`, or no option, leaves them unstyled. Red (int67a): the option
/// ignored, no font. Review rev78y2 finding 1; OXIDIZED_DESIGN_DIVERGENCES #884. Repro
/// class-bindings/ar5iv_classcaptionfonts_option_sets_the_knob.
#[test]
fn ar5iv_classcaptionfonts_option_sets_the_knob() {
  let tex = repro!("class-bindings/ar5iv_classcaptionfonts_option_sets_the_knob.tex");
  let plain = [
    r#"<caption><tag close=": ">Figure 1</tag>Fig one.</caption>"#,
    r#"<caption><tag close=": ">Table 1</tag>Tab one.</caption>"#,
  ];
  for (preload, expected) in [
    ("[classcaptionfonts]ar5iv.sty", [
      r#"<caption><tag close=": "><text font="bold" xml:id="S0.F1.3">Figure 1</text></tag><text font="bold" xml:id="S0.F1.4">Fig one.</text></caption>"#,
      r#"<caption><tag close=": "><text font="bold" xml:id="S0.T1.2">Table 1</text></tag><text font="bold" xml:id="S0.T1.3">Tab one.</text></caption>"#,
    ]),
    ("[noclasscaptionfonts]ar5iv.sty", plain),
    (PRELOAD, plain),
  ] {
    let (log, xml) = convert_files_with(tex, &[], Some(preload));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{preload}: {log}"
    );
    assert_eq!(captions(&xml), expected, "{preload}: {xml}");
  }
}

/// With the knob given to ar5iv.sty, ar5iv's own options to latexml.sty hold: `rawclasses` reads TeX Live's paper.cls
/// raw ("Fig. 1: Fig one.", as pdflatex), `ids` and `magnify`/`zoomout` stay. Red (int67a, the replaced recipe
/// `--preload=[classcaptionfonts]latexml.sty --preload=ar5iv.sty`): "Warning:missing_file:paper", OmniBus. Review
/// rev78y2 finding 1. Repro class-bindings/ar5iv_classcaptionfonts_option_keeps_ar5iv_options.
#[test]
fn ar5iv_classcaptionfonts_option_keeps_ar5iv_options() {
  let (log, xml) = convert_files_with(
    repro!("class-bindings/ar5iv_classcaptionfonts_option_keeps_ar5iv_options.tex"),
    &[],
    Some("[classcaptionfonts]ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for pi in [
    r#"<?latexml class="paper"?>"#,
    r#"<?latexml magnify="1.2"?>"#,
    r#"<?latexml zoomout="1.2"?>"#,
  ] {
    assert!(xml.contains(pi), "{pi}: {xml}");
  }
  assert_element(
    &xml,
    "p",
    &["xml:id=\"section1.p1.1\""],
    r#"<p xml:id="section1.p1.1">Hello.</p>"#,
  );
  assert_eq!(
    captions(&xml),
    // (paper.cls's `Fig.~\thefigure`, a no-break space)
    ["<caption><tag close=\": \">Fig.\u{a0}1</tag>Fig one.</caption>"],
    "{xml}"
  );
}

/// Under `\usepackage[ruled,labelfont=it]{caption}` a float.sty ruled float's label is in caption's `labelfont` alone
/// (caption.sty:904-909 declares no `ruled` style then; caption.sty:860-861 resets float.sty's `\@fs@cfont`), a boxed
/// one's too: pdflatex "Program 1: Prog one." and "Snip 1: Snip one.", both labels CMTI10 only. The 78v residual
/// (bold italic), resolved by 78y r2's `float_sty_caption` reset. Review rev78y2 finding 4. Repro
/// captions-floats/caption_ruled_option_ruled_float_takes_caption_label_font.
#[test]
fn caption_ruled_option_ruled_float_takes_caption_label_font() {
  let (log, xml) = convert_files_with(
    repro!("captions-floats/caption_ruled_option_ruled_float_takes_caption_label_font.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    captions(&xml),
    [
      r#"<caption><tag close=": "><text font="italic" xml:id="program1.3">Program 1</text></tag>Prog one.</caption>"#,
      r#"<caption><tag close=": "><text font="italic" xml:id="snip1.3">Snip 1</text></tag>Snip one.</caption>"#,
    ],
    "{xml}"
  );
}
