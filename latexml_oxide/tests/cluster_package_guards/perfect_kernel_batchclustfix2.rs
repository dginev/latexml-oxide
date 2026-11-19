//! Red/green guards for batch CLUSTFIX2, four more Error roots of run 397 where TeX (or the class TeX would read) is
//! clean: ar5iv's provided `\href` without hyperref's anchor layer, a pasj00 paper's `\timeform`/`\emissiontype` without
//! its class, psfig's `\psfigurepath`, and an elsart-style `\sep` in revtex4's `\keywords`. Repros under
//! `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// The text of the first `<p …>…</p>` of `xml`, markup aside.
fn first_p(xml: &str) -> String {
  let start = xml.find("<p ").expect("a paragraph");
  let end = start + xml[start..].find("</p>").expect("its end");
  text(&xml[start..end])
}

/// ar5iv's provided `\href` (OXIDIZED_DESIGN #588) tells a package hyperref is loaded, so the anchor layer it then calls
/// is provided with it (hyperref.sty's no-op anchors, hdvips.def's `\Hy@raisedlink`): a shipped algorithm2e.sty 3.6's
/// `\nl` (cs0602002, its :740-748) had three undefined-macro Errors and an ERROR box per line number. Repro
/// package-bindings/ar5iv_href_hyperref_anchor_layer.
#[test]
fn ar5iv_href_brings_hyperref_anchors() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/ar5iv_href_hyperref_anchor_layer.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">B text</p>"#);
}

/// A pasj00 paper whose source lacks the class (OmniBus): the class's `\timeform`/`\TF` parser, `\emissiontype`, its
/// abbreviations (`\fd`, `\arcsec`, `\micron`) and `\draft`, read verbatim (pasj00.cls:3716-3980, 3578-3598, 3684-3713,
/// 284-300; astro-ph0201169, astro-ph0204505, 0706.0253: an Error each and the time or angle lost). Each unit is a superscript with the decimal point under the last, as the class prints
/// "2'.5"; the 4 Warnings are the math parser's on the class's `\mathchoice` boxes, as under the shipped class. Distinct
/// from `perfect_kernel_batchclustfix::pasj00_without_its_class_keeps_its_figures` (the same branch, its other
/// commands). Repro class-bindings/pasj00_unshipped_timeform.
#[test]
fn pasj00_without_its_class_sets_timeform() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/pasj00_unshipped_timeform.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 4), "{log}");
  assert!(!log.contains("undefined"), "{log}");
  assert_eq!(
    first_p(&xml),
    "A shell of 2 . ′ 5 diameter, seeing from 0 . ′′ 4 at 12 h 34 m 56 . s 7 . We found the Fe II emission after 3 . d \
     5, at 5 ′′ and 3 \u{2062} μ m ."
  );
}

/// psfig.sty's `\psfigurepath{a:b}` (1.10:122-123, shipped by 0712.1995) adds each directory to the graphics search
/// path, so `\psfig{file=…}` resolves in the listed subdirectory (cond-mat0010435, cond-mat0012297: an Error and the
/// figure unresolved). Repro package-bindings/psfigurepath.
#[test]
fn psfigurepath_is_the_graphics_path() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/psfigurepath/psfigurepath.tex"
    ),
    &[(
      "figs/plot.eps",
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/psfigurepath/figs/plot.eps"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &[],
    r#"<graphics candidates="figs/plot.eps" graphic="plot.eps" options="width=56.9055pt" xml:id="p1.g1"/>"#,
  );
}

/// revtex4 never prints `\keywords` (only under `showkeys`), so an elsart-style `\sep` in it is never expanded by TeX;
/// digested into `ltx:keywords` it is elsart's separator where the paper leaves `\sep` undefined (cond-mat0309530: an
/// Error). A paper's own `\sep` stays its own. Repro sectioning-frontmatter/revtex4_keywords_elsart_sep.
#[test]
fn revtex4_keywords_read_elsart_sep() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex4_keywords_elsart_sep.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">vortex matter, columnar defects</keywords>",
  );
  let own = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex4_keywords_elsart_sep.tex"
  )
  .replace(
    "\\begin{document}",
    "\\newcommand\\sep{;}\n\\begin{document}",
  );
  let (log, xml) = convert_with(&own, Some("ar5iv.sty"));
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">vortex matter ;columnar defects</keywords>",
  );
}
