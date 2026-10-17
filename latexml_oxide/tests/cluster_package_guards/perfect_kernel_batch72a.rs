//! Red/green guards for batch 72a: "a stand-in's copy provides" (OXIDIZED_DESIGN_DIVERGENCES #731). After a binding
//! that stands in for an author's file has loaded, the copy of that file the paper ships is read raw, every definition
//! of a name LaTeXML's semantic layer owns holding only inside the copy's own code, every other one standing as TeX
//! makes it; the document-time code the copy defers is not registered; a file the copy loads is a real load.
//! Repros `tools/perfect_kernel/repros/stand-in-copies/`.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/",
      $path
    ))
  };
}

/// The census repro: the copy's `\kms` is an extra, its `\aj` the binding's (aas_macros.sty:94-97; 1309.5950,
/// 1805.06442, 2001.05519, 1406.5372). pdflatex: "Speed 5 km s−1 in AJ-COPY."; the binding keeps "AJ".
#[test]
fn aas_macros_copy_adds_its_extras() {
  let (log, xml) = convert_files_with(
    repro!("aas_extras/copy_adds_its_extras.tex"),
    &[("aas_macros.sty", repro!("aas_extras/aas_macros.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Speed 5\u{2009}km\u{a0}s<sup xml:id=\"p1.1.1\"><text font=\"italic\" xml:id=\"p1.1.1.1\">-1</text></sup> in \
     AJ.</p>",
  );
  assert!(
    log.contains(
      "Info:provide:\\aj \\aj keeps LaTeXML's definition: the paper's aas_macros.sty redefines it"
    ),
    "{log}"
  );
}

/// The copy's load-time logic runs as TeX runs it, whatever the bindings loaded before made of its scratch names
/// (natbib_sty.rs's and graphics_sty.rs's `RawTeX` scratch; the census hazard 4): "Y ON" with the copy as jmlr2e.sty
/// and as aas_macros.sty.
#[test]
fn copy_scratch_logic_runs_as_tex() {
  let tex = repro!("scratch_logic/copy_scratch_logic_runs_as_tex.tex");
  for style in ["jmlr2e", "aas_macros"] {
    let tex = tex.replace("\\usepackage{jmlr2e}", &format!("\\usepackage{{{style}}}"));
    let (log, xml) = convert_files_with(
      &tex,
      &[(
        &format!("{style}.sty"),
        &repro!("scratch_logic/jmlr2e.sty").replace("jmlr2e", style),
      )],
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{style}: {log}"
    );
    assert_element(
      &xml,
      "p",
      &["xml:id=\"p1.1\""],
      "<p xml:id=\"p1.1\">Y ON</p>",
    );
  }
}

/// A package the copy requires and the binding did not load is a real load: bmvc2k.cls's `\RequirePackage{multirow}`
/// makes `\multirow` work, and the copy's `\bmvcreviewcopy` is an extra (bmvc2k.cls:139, 2406.08486, 2210.06270).
#[test]
fn copy_loads_a_package_whole() {
  let (log, xml) = convert_files_with(
    repro!("nested_load/copy_loads_a_package_whole.tex"),
    &[("bmvc2k.cls", repro!("nested_load/bmvc2k.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("rowspan=\"2\""), "{xml}");
  assert!(log.contains("multirow"), "multirow was not loaded: {log}");
}

/// A copy its binding reads raw itself (arxiv_sty.rs `InputDefinitions!`; the 17 such stand-ins, 2010.06291) is not
/// read again: the copy's "read twice" check stays quiet, and the copy is processed once.
#[test]
fn copy_read_raw_by_its_binding_is_not_read_again() {
  let (log, xml) = convert_files_with(
    repro!("raw_once/copy_read_raw_by_its_binding_is_not_read_again.tex"),
    &[("arxiv.sty", repro!("raw_once/arxiv.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    log.matches("arxiv.sty...").count(),
    1,
    "the copy was not read exactly once: {log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">EXTRA</p>",
  );
}

/// Extras of every declaration kind work (`\newif`, `\newcounter`, `\newtheorem`, `\newenvironment`, `\let`, an option
/// of the copy's own `\DeclareOption`), and what the binding owns stays the binding's: the title and creator, the
/// abstract, the section, the equation numbers — the copy's `\theequation` and its `\numberwithin{equation}{section}`
/// (jheppub.sty:313-314, 2112.14648) hold for its own code only; the copy's `\LoadClass{article}` and `\ProcessOptions`
/// are quiet. The
/// copy's `\maketitle` is refused by the lock without being kept for a replay (no "COPY TITLE PAGE").
#[test]
fn copy_declarations_provide_extras_only() {
  let (log, xml) = convert_files_with(
    repro!("declarations/copy_declarations_provide_extras_only.tex"),
    &[("svjour3.cls", repro!("declarations/svjour3.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    !xml.contains("COPY"),
    "the copy's own definitions reached the document: {xml}"
  );
  assert_element(&xml, "title", &[], "<title>The Title</title>");
  assert_element(
    &xml,
    "creator",
    &[],
    "<creator role=\"author\">\n    <personname>An Author</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">The Institute</contact>\n  </creator>",
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    "<abstract inlist=\"toc\" name=\"Abstract\" xml:id=\"abstract1\"><p xml:id=\"abstract1.1\">The abstract.</p></abstract>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p1.1\""],
    "<p xml:id=\"S1.p1.1\">See <ref labelref=\"LABEL:e\"/>. YES 7 yes</p>",
  );
  for (id, number) in [("S1.E1", "1"), ("S2.E2", "2")] {
    let equation =
      latexml::util::test::xml_element(&xml, "equation", &[&format!("xml:id=\"{id}\"")])
        .unwrap_or_else(|| panic!("no equation {id}: {xml}"));
    assert!(
      equation.contains(&format!(
        "<tags><tag>({number})</tag><tag role=\"refnum\">{number}</tag></tags>"
      )),
      "{id}: {equation}"
    );
  }
  assert!(xml.contains("<tag>ExtraThm 1</tag>"), "{xml}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S1.p2.1\""],
    "<p xml:id=\"S1.p2.1\">[E] <text font=\"bold\" xml:id=\"S1.p2.1.1\">B</text> AEOC</p>",
  );
}

/// The code a copy defers to the document is not registered, and what it would redefine stays the binding's — each
/// shape against the same document converted with the copy's hook taken out: sn-jnl.cls:1409-1440's {appendices} and
/// `\thetable` (2604.20688, 1012.4730), svmult.cls:1677-1688's `\bibsection` (cond-mat0303533), jfm.cls:84-89's
/// {subequations} (cond-mat0701539), aipproc.cls:549-557's `\eqref` (astro-ph0105128). The copy's extra works.
#[test]
fn copy_document_hooks_keep_the_bindings() {
  for (tex, class, file, extra, element, attrs) in [
    (
      repro!("hooks/sn_jnl_begin_document_keeps_the_bindings.tex"),
      repro!("hooks/sn-jnl.cls"),
      "sn-jnl.cls",
      "SN-EXTRA",
      "table",
      &[] as &[&str],
    ),
    (
      repro!("hooks/svmult_bibsection_keeps_the_bindings.tex"),
      repro!("hooks/svmult.cls"),
      "svmult.cls",
      "SV-EXTRA",
      "bibliography",
      &[],
    ),
    (
      repro!("hooks/jfm_subequations_keeps_the_bindings.tex"),
      repro!("hooks/jfm.cls"),
      "jfm.cls",
      "JFM-EXTRA",
      "equationgroup",
      &[],
    ),
    (
      repro!("hooks/aipproc_eqref_keeps_the_bindings.tex"),
      repro!("hooks/aipproc.cls"),
      "aipproc.cls",
      "AIP-EXTRA",
      "p",
      &["xml:id=\"p1.2\""],
    ),
  ] {
    let (log, xml) = convert_files_with(tex, &[(file, class)], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{file}: {log}"
    );
    assert!(xml.contains(extra), "{file}: the extra is missing: {xml}");
    assert!(
      log.contains(&format!(
        "The paper's {file} defers code to the document (\\AtBeginDocument); not registered"
      )),
      "{file}: {log}"
    );
    let unhooked = class
      .find("\\AtBeginDocument")
      .map(|at| &class[..at])
      .unwrap_or(class);
    let (_, plain) = convert_files_with(tex, &[(file, unhooked)], Some("ar5iv.sty"));
    let expected = latexml::util::test::xml_element(&plain, element, attrs)
      .unwrap_or_else(|| panic!("{file}: no <{element}> without the hook: {plain}"));
    assert_element(&xml, element, attrs, &expected);
  }
}

/// A file of the stood-in name that is another file is not read: hep-ph9609379's diagrams.sty (`\special` PostScript
/// macros, no declaration, none of the diagrams binding's command set) and 2411.02780's equations.sty
/// (`\ProvidesPackage{annotate-equations}`). pdflatex reads them ("READ"); the binding stands in for another package,
/// so the base behaviour stands ("NOT READ"; reading them raw is an open row).
#[test]
fn copy_of_another_file_is_not_read() {
  for (tex, file, copy) in [
    (
      repro!("identity/same_name_other_file_is_not_read.tex"),
      "diagrams.sty",
      repro!("identity/diagrams.sty"),
    ),
    (
      repro!("identity/renamed_package_is_not_read.tex"),
      "equations.sty",
      repro!("identity/equations.sty"),
    ),
  ] {
    let (log, xml) = convert_files_with(tex, &[(file, copy)], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{file}: {log}"
    );
    assert_element(
      &xml,
      "p",
      &["xml:id=\"p1.1\""],
      "<p xml:id=\"p1.1\">NOT READ</p>",
    );
    assert!(
      log.contains(&format!(
        "Info:provide:{file} The paper's {file} defines 0 of the"
      )) && log.contains("not the file its binding stands in for, not read"),
      "{file}: {log}"
    );
  }
}

/// WileyNJDv5.cls:377 loads lettersp before the algorithms (:6820), so LETTERSP.STY:3-12 defines its own `\Comment`
/// block; the binding loads it in the class's order where the paper's copy loads it (2408.07575, 2501.12471,
/// 2602.03829, 2609.08425, 2609.19994). pdflatex: "FIRST LS". The copy's `\received` (WileyNJDv5.cls:5071) keeps the
/// binding's OmniBus base's dated frontmatter, a guess (`Fallback`) to the document but the binding's own to the copy
/// (2609.19994).
#[test]
fn wiley_loads_lettersp_before_the_algorithms() {
  let (log, xml) = convert_files_with(
    repro!("wiley_order/lettersp_before_the_algorithms.tex"),
    &[
      ("WileyNJDv5.cls", repro!("wiley_order/WileyNJDv5.cls")),
      ("lettersp.sty", repro!("wiley_order/lettersp.sty")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">FIRST LS</p>",
  );
  assert_element(
    &xml,
    "date",
    &["role=\"received\""],
    "<date name=\"Received\u{a0}\" role=\"received\">Date X</date>",
  );
}

/// crckapb.sty:665-760's raw citation commands (`\@citex`, `\citeyear` over `\@citedata`, which print an aux file's
/// `b@<key>`: "?" in astro-ph9601169) leave the citations LaTeXML's: the kernel's `\cite`, natbib's `\citeyear` (the
/// binding's autoload, as Perl's OmniBus), the copy's `\shortcite` over the binding's `\cite`. pdflatex prints the
/// copy's "RAWCITEX"/"RAWYEAR".
#[test]
fn copy_citations_keep_the_semantic_cite() {
  let (log, xml) = convert_files_with(
    repro!("citations/copy_citations_keep_the_semantic_cite.tex"),
    &[("crckapb.sty", repro!("citations/crckapb.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("RAW"), "the copy's citation code ran: {xml}");
  assert_eq!(
    xml.matches("<bibref bibrefs=\"k1\"").count(),
    3,
    "every citation is a <cite>: {xml}"
  );
}

/// pnas-new.cls:229 `\RequirePackage[explicit]{titlesec}` (1907.12289; :358 in 2509.24970): the binding loads titlesec
/// with the option the paper's copy gives it, so the copy's and the document's title formats take the title as `#1`
/// (pdflatex: "X: Intro").
#[test]
fn pnas_loads_titlesec_explicit() {
  let (log, xml) = convert_files_with(
    repro!("pnas_explicit/pnas_titlesec_explicit.tex"),
    &[("pnas-new.cls", repro!("pnas_explicit/pnas-new.cls"))],
    Some("ar5iv.sty"),
  );
  // (the binding's mdframed load, pnas-new.cls:53-55, is the stub's Warning)
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Warning:missing_file:mdframed.sty mdframed.sty is only minimally stubbed"),
    "{log}"
  );
  let subsection = latexml::util::test::xml_element(&xml, "subsection", &[])
    .unwrap_or_else(|| panic!("no subsection: {xml}"));
  assert!(subsection.contains("X: Intro</title>"), "{subsection}");
}

/// IEEEtran.cls:268-270's paper-size flags, which ieeeaccess.cls:21-23's `a4paper` handler sets (2007.03025,
/// 2009.04554, 2011.07807, 2209.03883, 2209.12464, 2609.20467). pdflatex: "A4 CS".
#[test]
fn ieeetran_paper_size_flags() {
  let (log, xml) = convert_files_with(
    repro!("ieee_a4/ieeetran_paper_size_flags.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(">A4 CS</p>"), "{xml}");
}
