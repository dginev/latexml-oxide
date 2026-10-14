//! Red/green guards for batch 66h: run 345's Error → Fatal regressions (root-causer report
//! `agent_reports/2026-10-14_ef345_regressions.md`) and three sandbox regressions from runs 341/342.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

fn fatal_count(log: &str) -> usize { log.lines().filter(|l| l.starts_with("Fatal:")).count() }

/// A one-token box argument `\caption` carries the box's `}` into the caption wrapper, which must not close a braced
/// branch with it (astro-ph0107461 and 15 more: Fatal Mouth:EoF since 66d). As pdflatex prints it ("Figure 1:", then
/// the text): the caption holds its tag, the text follows in the figure.
#[test]
fn caption_one_token_box_argument() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_one_token_box_argument.tex"
  ));
  assert_eq!((error_count(&log), fatal_count(&log)), (2, 0), "{log}");
  assert!(
    xml.contains(r#"<caption><tag close=": ">Figure 1</tag></caption>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="S0.F1.1">The variation in the diffuse emission.</p>"#),
    "{xml}"
  );
  assert!(xml.contains("Text after."), "{xml}");
}

/// amstex.tex defines no `\title`: a document's own one made before `\input amstex` stays (math0103177 and 6 AGT
/// papers).
#[test]
fn amstex_title_before_input_stays_the_documents() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/loader/amstex_title_before_input_stays_the_documents.tex"
  ));
  assert_eq!((error_count(&log), fatal_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("The title is Generalized Orbifold Euler Characteristic."),
    "{xml}"
  );
}

/// amstex.tex:72's `\RIfM@` (Michor's diag.tex; math9611223 and 6 more, 6 of the 7 out of their Fatal).
#[test]
fn amstex_rifm_mode_test() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/macro-state/amstex_rifm_mode_test.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(">text and <"), "{xml}");
  assert!(xml.contains(">math</text>"), "{xml}");
}

/// revtex's `\@email` closes the `\sanitize@url` group a class's re-dressed `\email` opens (cej.cls; 0710.1022).
#[test]
fn revtex_email_redressed_over_sanitize_url() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/revtex_email_redressed_over_sanitize_url.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<contact name="E-mail: " role="email"><text font="typewriter" xml:id="id1">a@b.c</text></contact>"#
    ),
    "{xml}"
  );
  assert!(!xml.contains("SHOULD-NOT-APPEAR"), "{xml}");
}

/// svjour3's `\institute` sentinel `\at` is not the affiliation's text, so a document's own `\at` macro does not run
/// there (1404.4148 and 8 more: Fatal Stomach:Recursion).
#[test]
fn svjour3_institute_with_document_at_macro() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_institute_with_document_at_macro.tex"
  ));
  assert_eq!((error_count(&log), fatal_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches(r#"role="affiliation""#).count(), 1, "{xml}");
  assert!(
    xml.contains(r#"role="affiliation">Center for Risk</contact>"#),
    "{xml}"
  );
}

/// A `\left` read in text (the document's `\at` running in an affiliation) gets its `\right.` once, not at every later
/// end of input (Fatal Stomach:Recursion; the 1404.4148 shape with an `\at` the sentinel strip leaves).
#[test]
fn svjour3_doubled_at_left_in_text_closes_once() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_doubled_at_left_in_text_closes_once.tex"
  ));
  assert_eq!(
    (error_count(&log), warning_count(&log), fatal_count(&log)),
    (3, 2, 0),
    "{log}"
  );
  assert_eq!(
    log
      .lines()
      .filter(|l| l.contains(r"Missing \right. inserted"))
      .count(),
    1,
    "{log}"
  );
  assert!(
    xml.contains(r#"role="affiliation">Second Place</contact>"#),
    "{xml}"
  );
  assert!(xml.contains("Text body."), "{xml}");
}

/// amsrefs' `\strip@MRprefix`/`\MRhref` for a paper's own `\MR` (0806.1269, 1612.01009, 1706.05540 and 2 more);
/// `msc-links` loads hyperref (amsrefs.sty:198-205), so `\MRhref` links the review.
#[test]
fn amsrefs_document_mr_redress() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/amsrefs_document_mr_redress.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The number and its review, each linked as amsrefs.sty:2939-2948's `\MRhref` sets them.
  let link = r#"<ref class="ltx_href" href="http://www.ams.org/mathscinet-getitem?mr=MR0498596">"#;
  assert!(xml.contains(&format!("{link}MR0498596")), "{xml}");
  assert!(xml.contains(&format!("{link}(58 #16687)")), "{xml}");
}

/// amsrefs' `{bibchapter}` (amsrefs.sty:1242): a bibliography titled Bibliography holding the entry.
#[test]
fn amsrefs_bibchapter() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/amsrefs_bibchapter.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    flat.contains(
      r#"<title>Bibliography</title> <biblist> <bibentry key="GiesekerGlobal" type="article""#
    ),
    "{xml}"
  );
}

/// lipics-v2021 loads thm-restate (and thmtools) only on its `thm-restate` option, so keytheorems loads beside it
/// (2604.22493).
#[test]
fn lipics_thm_restate_only_on_its_option() {
  let (log, _) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lipics_thm_restate_only_on_its_option.tex"
  ));
  assert_eq!((error_count(&log), fatal_count(&log)), (0, 0), "{log}");
}

/// With the `thm-restate` option the class loads thm-restate: `{restatable}` is a theorem, and its restatement a
/// reference to it as thm-restate.sty.ltxml:28's `\lx@thm@restate` sets it (2211.04601).
#[test]
fn lipics_thm_restate_option_loads_restatable() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lipics_thm_restate_option_loads_restatable.tex"
  ));
  assert_eq!((error_count(&log), fatal_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_theorem""#),
    "{xml}"
  );
  assert_eq!(xml.matches("Restated body.").count(), 1, "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="p1.1">See <ref labelref="LABEL:restatable:mythm"/></p>"#),
    "{xml}"
  );
}

/// lipics-v2021 under `autoref` without `thm-restate` gives each theorem sibling an aliascnt alias of `theorem`'s
/// counter, so a document can number a theorem by `corollary` (2604.22493: keytheorems' `sibling=corollary`, six
/// `No counter 'corollary'` Errors). As pdflatex numbers them: one sequence within the section.
#[test]
fn lipics_autoref_alias_sibling_counters() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/lipics_autoref_alias_sibling_counters.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let tags: Vec<&str> = xml
    .split(r#"<tag role="typerefnum">"#)
    .skip(1)
    .filter_map(|rest| rest.split("</tag>").next())
    .filter(|tag| !tag.starts_with('§'))
    .collect();
  assert_eq!(
    tags,
    [
      "Theorem 1.1",
      "Corollary 1.2",
      "Theorem 1.3",
      "Corollary 1.4"
    ],
    "{xml}"
  );
}

/// babel's ini caption template captures `\fnum@table`'s body (2605.29738: Fatal, the capture was the token itself).
#[test]
fn babel_ini_caption_template_needs_fnum() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/babel_ini_caption_template_needs_fnum.tex"
  ));
  assert_eq!((error_count(&log), fatal_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<caption><tag close=": ">Table 1</tag>A caption.</caption>"#),
    "{xml}"
  );
}

/// revtex4-2's `\frontmatter@footnote` as a document calls it for a note on several authors (2605.04155).
#[test]
fn revtex_frontmatter_footnote_on_two_authors() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_frontmatter_footnote_on_two_authors.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml
      .matches(r#"role="thanks">These two authors contributed equally to this work.</contact>"#)
      .count(),
    2,
    "{xml}"
  );
}

/// float.sty's `\newfloat` is a macro scrhack's floatbytocbasic can patch (2606.04906, 2606.11358).
#[test]
fn scrhack_patches_newfloat() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/scrhack_patches_newfloat.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<float class="ltx_float_program" inlist="lop" xml:id="program1">"#),
    "{xml}"
  );
  assert!(xml.contains("A program</caption>"), "{xml}");
}
