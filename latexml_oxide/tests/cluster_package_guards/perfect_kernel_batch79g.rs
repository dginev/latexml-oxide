//! Red/green guards for batch 79g (complete breakage, user 2026-10-24: a missing image, a broken `\ref` or `\cite` is a
//! readability failure held to 100 %; the fixes are coverage gaps). Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// A graphic named without its directory is found under an `\input@path` entry, as pdflatex finds it (graphics.sty's
/// `\Gin@getbase` tests each name with `\IfFileExists`, which looks under every `\input@path` entry, latex.ltx:9698
/// `\@iffileonpath`). Red (int79a): "No graphic source found", an `ltx_missing_image` in the HTML — the image search
/// took `\graphicspath`, the search paths and the source directory only. Witness 1112.0840 (8 figures in `graphics/`
/// under its class's `\csdef{input@path}{{style/}{graphics/}{}}`); the box is sized from the file found. Repro
/// package-bindings/includegraphics_on_input_path.
#[test]
fn includegraphics_looks_under_the_input_path() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/includegraphics_on_input_path.tex"),
    &[(
      "figs/ip_plot.pdf",
      repro!("package-bindings/eps_as_pdf_one.pdf"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"ip_plot\""],
    r#"<graphics candidates="figs/ip_plot.pdf" cssstyle="width:20.075em; height:10.037em" graphic="ip_plot" xml:id="p1.g1"/>"#,
  );
}

/// A graphic named with quotes inside it is found unquoted, as pdflatex finds it (`\IfFileExists` drops every `"` from a
/// name, `\set@curr@file`'s `\unquote@name`, latex.ltx:9654-9666). Red (int79a): no candidates, an `ltx_missing_image`
/// — the image search stripped only quotes around the whole name. Witnesses 1907.03916 (`{"Figure_2".jpg}`, 5 figures),
/// 2005.11727 (6). Repro package-bindings/includegraphics_quoted_name.
#[test]
fn includegraphics_finds_a_quoted_name_unquoted() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/includegraphics_quoted_name.tex"),
    &[("ip_plot.pdf", repro!("package-bindings/eps_as_pdf_one.pdf"))],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &["xml:id=\"p1.g1\""],
    r#"<graphics candidates="ip_plot.pdf" cssstyle="width:20.075em; height:10.037em" graphic="&quot;ip_plot&quot;.pdf" xml:id="p1.g1"/>"#,
  );
}

/// The subfile `subfiles`' `\subfile` loads (subfiles.sty:120-131, by import.sty's `\subimport`) finds the figures it
/// names in its own directory, as pdflatex does. Red (int79a): no candidates — `\subfile` was `\input` (Perl's binding).
/// Witness 2209.05589 (`\subfile{./sub/OPA_v2.tex}`, its 11 figures in `sub/`). Repro
/// package-bindings/subfile_figure_in_its_directory.
#[test]
fn a_subfile_finds_the_figures_in_its_directory() {
  const PART: &str = "\\documentclass[../main.tex]{subfiles}\n\\begin{document}\nPart text. \\includegraphics{sf_plot}\n\\end{document}\n";
  let (log, xml) = convert_files_with(
    repro!("package-bindings/subfile_figure_in_its_directory.tex"),
    &[
      ("sub/part.tex", PART),
      (
        "sub/sf_plot.pdf",
        repro!("package-bindings/eps_as_pdf_one.pdf"),
      ),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"sf_plot\""],
    r#"<graphics candidates="sub/sf_plot.pdf" cssstyle="width:20.075em; height:10.037em" graphic="sf_plot" xml:id="p1.g1"/>"#,
  );
  assert!(xml.contains("Part text."), "{xml}");
}

/// The other document's `.aux`, as xr's `\externaldocument` reads it.
const SUPP_AUX: &str = "\\relax\n\\newlabel{fig:s1}{{S1}{3}}\n\\newlabel{eq:s2}{{S2}{4}}\n";

/// xr's `\externaldocument{supp}` imports the labels of `supp.aux`: `\ref` and `\eqref` print the numbers pdflatex
/// prints ("See Fig. S1, Eq. (S2) and Section 1."), and a label of this document stays a reference post resolves. Red
/// (int79a): xr was a stub, so both references were missing labels. Witness 2412.14256 (its appendix labels in the
/// shipped `supplemental_aux.aux`). Repro package-bindings/xr_external_document_labels.
#[test]
fn xr_prints_the_other_documents_numbers() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/xr_external_document_labels.tex"),
    &[("supp.aux", SUPP_AUX)],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:fig:s1\""],
    r#"<ref class="ltx_ref_external" labelref="LABEL:fig:s1">S1</ref>"#,
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:eq:s2\""],
    r#"<ref class="ltx_ref_external" labelref="LABEL:eq:s2">S2</ref>"#,
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:sec:main\""],
    r#"<ref labelref="LABEL:sec:main"/>"#,
  );
}

/// A label the other document and this one both hold is this document's, as in pdflatex, whose own `.aux`, read at
/// `\begin{document}`, redefines the `\r@sec:main` xr made: post drops the imported text and links the reference to the
/// section here, while the imported-only label stays its text with no missing-label mark.
#[test]
fn xr_yields_to_a_label_of_this_document() {
  let aux = format!("{SUPP_AUX}\\newlabel{{sec:main}}{{{{X}}{{1}}}}\n");
  let html = html_of(
    repro!("package-bindings/xr_external_document_labels.tex"),
    &[("supp.aux", &aux)],
  );
  assert!(
    html.contains(r##"<a href="#S1" title="1 Main" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>"##),
    "{html}"
  );
  assert!(
    html.contains(r#"<span class="ltx_ref ltx_ref_external ltx_ref_self">S1</span>"#),
    "{html}"
  );
  assert!(!html.contains("ltx_missing_label"), "{html}");
}

/// `tex` (and `files`) converted to HTML by the binary in a directory of their own, as a paper is (the `\ref` text is
/// the post-processor's): its HTML, the conversion free of errors and warnings.
fn html_of(tex: &str, files: &[(&str, &str)]) -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write a file");
  }
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html")
}
