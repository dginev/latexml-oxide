//! Red/green guards for batch 78w (determinism, det354): post's report of the references it could not resolve in
//! Perl's order with its ids sorted, and an extensionless graphic's candidates in TeX's order, not the directory's. The
//! dump's record order, a processing instruction's keys, the held-tag roles and the fragment index's records have unit
//! tests beside their code (`dump_writer`, `document::pi_data_order`, `sect11::held_tag_role_order`,
//! `fragment_index`). Each guard converts its repro under `tools/perfect_kernel/repros/` (pdflatex-checked) and asserts
//! the diagnostics by count and whole elements or lines.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{convert_html, error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The HTML run reports its unresolved references as Perl's CrossRef.pm:64-76 does — `error, warn, info`, the types of
/// each sorted — with the ids of each type sorted (Perl's are in hash order): "Entry for citation" first. Red (int64a):
/// "Target for Label: LABEL:mid,LABEL:zeta,LABEL:alpha" first, then "Entry for citation: bk1,ak2". det354 C1;
/// OXIDIZED_DESIGN_DIVERGENCES #883. Repro cross-references/missing_targets_are_reported_in_order.
#[test]
fn missing_targets_are_reported_in_order() {
  let (log, html) = convert_html(repro!(
    "cross-references/missing_targets_are_reported_in_order.tex"
  ));
  assert!(html.contains("ltx_missing_citation"), "{html}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  let missing: Vec<&str> = log
    .lines()
    .filter(|line| line.contains("Missing "))
    .collect();
  assert_eq!(
    missing,
    [
      "Warning:expected:ids Missing Entry for citation: ak2,bk1",
      "Warning:expected:ids Missing Target for Label: LABEL:alpha,LABEL:mid,LABEL:zeta",
    ],
    "{log}"
  );
}

/// `\includegraphics{xl_fig}` beside `xl_fig.ps`, `xl_fig.pdf` and `xl_fig.eps` (written in that order) lists them by
/// type, `\Gin@extensions` order (pdftex.def:597-600), whatever order the directory gives, and the box takes the
/// `.pdf`'s natural size (200bp, 20.075em), as pdflatex reads `xl_fig.pdf`: the whole-output check. Red on int64a only
/// where `/tmp` is a tmpfs, which lists the files newest first (candidates="xl_fig.eps,xl_fig.pdf,xl_fig.ps", 4.015em
/// wide); on ext4 (the CI runner) int64a's `read_dir` order already passes, so the order itself is guarded by the unit
/// test `util::image::candidate_order_tests::extensionless_candidates_in_one_order_whatever_the_directory_lists`
/// (review rev78w finding 1). det354 4b; witness 2011.14197; OXIDIZED_DESIGN_DIVERGENCES #883.
/// Repro package-bindings/includegraphics_extensionless_candidates_in_type_order.
#[test]
fn includegraphics_extensionless_candidates_in_type_order() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/includegraphics_extensionless_candidates_in_type_order.tex"),
    &[
      ("xl_fig.ps", repro!("package-bindings/eps_as_pdf_two.eps")),
      ("xl_fig.pdf", repro!("package-bindings/eps_as_pdf_one.pdf")),
      ("xl_fig.eps", repro!("package-bindings/eps_as_pdf_two.eps")),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "graphics",
    &[],
    r#"<graphics candidates="xl_fig.pdf,xl_fig.eps,xl_fig.ps" cssstyle="width:20.075em; height:10.037em" graphic="xl_fig" xml:id="p1.g1"/>"#,
  );
}
