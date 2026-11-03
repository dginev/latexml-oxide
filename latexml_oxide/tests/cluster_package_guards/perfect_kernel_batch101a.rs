//! Red/green guards for batch 101a: hyperref's `\Hy@driver` follows the output mode (hyperref.sty:4246-4407), not a
//! constant `hpdftex`, and `\Hy@PutCatalog` is defined (hpdftex.def:40). A shipped siamart copy under the DVI persona
//! loads hypdvips.sty, which errs unless the driver is `hdvips` (cortex 2605/2606 cluster `latex:(hypdvips)`: 2605.04427,
//! 2606.08053, 1703.08345). Repro `tools/perfect_kernel/repros/stand-in-copies/siamart_hypdvips/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The witnesses' shape: the siamart copy's `\ifpdf\else\usepackage{hypdvips}\fi` under `\pdfoutput=0`. pdflatex:
/// "Driver (default): hdvips", 0 errors; both links kept. Its `\AtEndDocument` calls `\Hy@PutCatalog` only for an
/// open action, so the hook's definition is asserted directly.
#[test]
fn siamart_copy_loads_hypdvips_in_dvi_mode() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/stand-in-copies/siamart_hypdvips/hypdvips_follows_the_dvi_output_mode.tex"
  );
  let cls = include_str!(
    "../../../tools/perfect_kernel/repros/stand-in-copies/siamart_hypdvips/siamart.cls"
  );
  let (log, xml) = convert_files_with(tex, &[("siamart.cls", cls)], Some("ar5iv.sty"));
  // hypdvips.sty:2002 `\g@addto@macro{\endfigure}{\pp@subfigurefalse}`: `\endfigure` is a constructor here, so the
  // addition (a flag reset, no content) is dropped with Perl's Warning (Package.pm:2542 `AddToMacro`).
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log
      .contains("Warning:unexpected:\\endfigure \\endfigure is not an expandable control sequence"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">See <ref class=\"ltx_href\" href=\"https://arxiv.org\">arXiv</ref> and <ref \
     class=\"ltx_url\" font=\"typewriter\" href=\"https://example.org\">https://example.org</ref>.</p>",
  );
  // hypdvips.sty:3091-3108's catalog writer: hpdftex.def:40's `\@gobble`, nothing written.
  let tex = tex.replace(
    "See ",
    "\\makeatletter\\Hy@PutCatalog{/OpenAction <<x>>}\\makeatother See ",
  );
  let (log, xml) = convert_files_with(&tex, &[("siamart.cls", cls)], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(!xml.contains("OpenAction"), "{xml}");
}

/// `\Hy@driver` as hyperref.sty's `\Hy@AtEndOfPackage` leaves it, each shape checked with pdflatex: PDF mode is
/// `hpdftex` whatever option is given (`[dvips]` forced, :4251-4265; the default persona), the `luatex` profile
/// `hluatex`; DVI mode takes the last driver option, the class's global ones read before the package's own (:4066),
/// and defaults to `hdvips` (:4339). hrefhide.sty:153-166 then raises "Producing not a pdf file" under DVI, as
/// latex+dvips does; `perfect_kernel_batch54::singleton_internal_surface` keeps the PDF default. Two shapes pin a
/// RESIDUAL, not TeX: `[dvips]` in PDF mode and `[pdftex]` in DVI mode get the driver pdflatex/latex picks, but real
/// TeX also raises hyperref.sty:2528-2563's "Wrong DVI mode driver option" / "Wrong driver option" Error, which we do
/// not (base did not either; SYNC_STATUS row 101a, review rev101a finding 1).
#[test]
fn hyperref_driver_follows_the_output_mode() {
  let doc = |mode: &str, class: &str, package: &str| {
    format!(
      "{mode}\\documentclass{class}{{article}}\n\\usepackage{package}{{hyperref}}\n\\makeatletter\n\\begin{{document}}\nDriver: \\Hy@driver.\n\\end{{document}}\n"
    )
  };
  for (mode, class, package, preload, driver) in [
    ("", "", "", Some("ar5iv.sty"), "hpdftex"),
    ("", "", "[dvips]", Some("ar5iv.sty"), "hpdftex"),
    ("", "", "", Some("[luatex]latexml.sty"), "hluatex"),
    ("\\pdfoutput=0\n", "", "", Some("ar5iv.sty"), "hdvips"),
    (
      "\\pdfoutput=0\n",
      "",
      "[dvipdfmx]",
      Some("ar5iv.sty"),
      "hdvipdfm",
    ),
    (
      "\\pdfoutput=0\n",
      "[dvipdfmx]",
      "",
      Some("ar5iv.sty"),
      "hdvipdfm",
    ),
    (
      "\\pdfoutput=0\n",
      "[dvipdfmx]",
      "[dvips]",
      Some("ar5iv.sty"),
      "hdvips",
    ),
    (
      "\\pdfoutput=0\n",
      "",
      "[pdftex]",
      Some("ar5iv.sty"),
      "hdvips",
    ),
  ] {
    let (log, xml) = convert_with(&doc(mode, class, package), preload);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{mode}{class}{package}: {log}"
    );
    assert!(
      xml.contains(&format!("Driver: {driver}.")),
      "{mode}{class}{package} {preload:?} wants {driver}: {xml}"
    );
  }
}
