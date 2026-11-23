//! Red/green guards for batch REGFIXH, a regression of the v0.8.0 per-paper check (0 errors on 2026-10-08, an Error on
//! the release candidate): a class binding loaded another binding as a file its real class never loads (Perl's
//! code-sharing `LoadClass`/`RequirePackage`, KNOWN_PERL_ERRORS #1300), so the paper's copy of that file was read
//! after it (OXIDIZED_DESIGN_DIVERGENCES #731). The shared definitions are now code, with no load. Repros under
//! `tools/perfect_kernel/repros/stand-in-copies/binding_internal_load/`.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/binding_internal_load/",
      $path
    ))
  };
}

/// Convert `tex` with the paper's `files`: 0 errors, `warnings` warnings, and its first paragraph is `p`. Returns the
/// log.
fn convert_to_paragraph(tex: &str, files: &[(&str, &str)], warnings: usize, p: &str) -> String {
  let (log, xml) = convert_files_with(tex, files, Some(PRELOAD));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    &format!("<p xml:id=\"p1.1\">{p}</p>"),
  );
  log
}

/// svjour3.cls loads no svjour.cls (Perl svjour3.cls.ltxml `LoadClass('svjour')`): `\@ifclassloaded{svjour}` is false
/// and the paper's older svjour.cls is not read, so its :93 "No valid journal specified in option list" over svjour3's
/// options is not raised; the paper's svjour3.cls is (pdflatex "Body: no svjour."). Red (INTREGFIX4): 1 error, svjour.cls
/// read before svjour3.cls, "Body: svjour.". Witnesses 2101.09497, 2110.05134, 2408.12532.
#[test]
fn svjour3_reads_no_older_svjour_copy() {
  let log = convert_to_paragraph(
    repro!("svjour3_reads_no_older_svjour_copy.tex"),
    &[
      ("svjour.cls", repro!("svjour.cls")),
      ("svjour3.cls", repro!("svjour3.cls")),
    ],
    0,
    "Body: no svjour.",
  );
  assert!(
    log.contains("svjour3.cls declares itself svjour3")
      && !log.contains("svjour.cls declares itself"),
    "{log}"
  );
}

/// emulateapj.cls loads no emulateapj.sty (Perl emulateapj.cls.ltxml:26 `RequirePackage('emulateapj')`):
/// `\@ifpackageloaded{emulateapj}` is false and the paper's truncated emulateapj.sty is not read (pdflatex "Text: no
/// emulateapj.sty."). Red (INTREGFIX4): 1 error, "Gullet->readBalanced ran out of input" at the copy's end. Distinct
/// from the svjour3 guard: a package a class binding requires, not a base class. Witness 1305.6912.
#[test]
fn emulateapj_reads_no_shipped_sty_copy() {
  let log = convert_to_paragraph(
    repro!("emulateapj_reads_no_shipped_sty_copy.tex"),
    &[("emulateapj.sty", repro!("emulateapj.sty"))],
    0,
    "Text: no emulateapj.sty.",
  );
  assert!(!log.contains("emulateapj.sty declares itself"), "{log}");
}

/// A svjour1/svjour2 paper's own class file, Springer's svjour code declaring `\ProvidesClass{svjour}`, is read: svjour
/// is a file the svjour3 binding is built on (its definitions made as code, `svjour_cls::svjour_definitions`), and the
/// copy is of the class the paper requested — unlike the svjour3 guard's svjour.cls, which nothing requests (OD #731
/// rule 0). pdflatex "Fig. 1: Cap.". Red (REGFIXH): "Figure 1", the copy "declared svjour: … not read". Witnesses
/// hep-ph0608235 ("Fig. 1", svepj.clo's 180mm), 0712.4224, 1301.5646, 1710.10527, hep-ph0310058, physics0101008.
#[test]
fn svjour2_copy_declaring_svjour_is_read() {
  let (log, xml) = convert_files_with(
    repro!("svjour2_copy_declaring_svjour_is_read.tex"),
    &[("svjour2.cls", repro!("svjour2.cls"))],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("svjour2.cls declares itself svjour"), "{log}");
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\": \">Fig. 1</tag>Cap.</caption>",
  );
}

/// The shipped-class `\vec` test reads the class the paper requested (`\@currname`), here a svjour2.cls copy whose `\vec`
/// is `\mathbf`, so ω stays italic as in pdflatex; it read svjour.cls / svjour3.cls, neither shipped (REGFIXH2 review).
#[test]
fn svjour2_vec_reads_the_requested_class() {
  let (log, xml) = convert_files_with(
    repro!("svjour2_vec_reads_the_requested_class.tex"),
    &[("svjour2.cls", repro!("svjour2_vec.cls"))],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("svjour2.cls declares itself svjour"), "{log}");
  assert!(
    xml.contains("<XMTok font=\"bold\" role=\"UNKNOWN\">v</XMTok>"),
    "{xml}"
  );
  assert!(
    xml.contains("<XMTok font=\"italic\" name=\"omega\" role=\"UNKNOWN\">ω</XMTok>"),
    "{xml}"
  );
}

/// A JHEP3.cls copy declaring `\ProvidesClass{JHEP}` is the paper's requested class file: read, as before the code
/// reuse (the JHEP3 binding records JHEP as a file it is built on, never loaded by it). Its declaration records
/// `\ver@JHEP3.cls`, so `\@ifclassloaded{JHEP}` is false, as in pdflatex ("JLOADED" until COPYROUTE: KNOWN_PERL_ERRORS
/// #1307).
#[test]
fn jhep3_copy_declaring_jhep_is_read() {
  convert_to_paragraph(
    repro!("jhep3_copy_declaring_jhep_is_read.tex"),
    &[("JHEP3.cls", repro!("JHEP3_declares_jhep.cls"))],
    // (LaTeX's "You have requested document class `JHEP3', but the document class provides `JHEP'")
    1,
    "Body JNOT COPYREAD.",
  );
}
