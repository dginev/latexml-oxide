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
  assert_element(
    &xml,
    "td",
    &["rowspan=\"2\""],
    "<td align=\"left\" rowspan=\"2\" xml:id=\"p1.1.1.1\"><text xml:id=\"p1.1.1.1.1\">AB</text></td>",
  );
  assert!(
    log.contains("(Loading multirow_sty.rs... )"),
    "multirow was not loaded: {log}"
  );
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
      log.contains(&format!("Info:provide:{file} The paper's {file} defines "))
        && log.contains("not the file its binding stands in for, not read"),
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
  assert_element(
    &subsection,
    "title",
    &[],
    "<title font=\"bold\">X: Intro</title>",
  );
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
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">A4 CS</p>",
  );
}

/// A setter the copy calls stores a class default, which the document's own call replaces and which shows only where
/// the class's title page prints it (atlasdoc.cls:189-190 `\author{The ATLAS Collaboration}`, `\date{\today}`; the 2019
/// class's `\maketitle` prints `\@date` only in `\ATLAS@TitleHeader` under `datetop`, without `cernpreprint`, :567,
/// :716; 1805.01649, 1806.07355, 1807.02012, 1902.05892, 1904.03536). pdflatex: "DEFAULT DATE / T / Jane Doe";
/// `[nodatetop]` with no `\author`: "T / The ATLAS Collaboration"; `[cernpreprint]`: "T / Jane Doe". The document's
/// call replaces the default whatever its value (72a r4; the reviewer's probe fm_dateempty): `\date{}`, `\author{}`
/// and `\title{}` print none of the class's ("T / Jane Doe"; "DEFAULT DATE / T"; "DEFAULT DATE / Jane Doe"), and
/// `\date{}` then `\date{X}` prints X. A copy title page that runs the binding's (`\begingroup\@maketitle\endgroup`,
/// probe fm_kernelmt; or `\DeclareRobustCommand`'d) prints its defaults: "DEFAULT DATE / T / The ATLAS
/// Collaboration". A switch the copy's title page tests with `\unless` takes its other branch: `\unless\ifADOC@datetop`
/// prints no date.
#[test]
fn copy_setters_store_class_defaults() {
  let tex = repro!("frontmatter/copy_setters_store_class_defaults.tex");
  let cls = repro!("frontmatter/atlasdoc.cls");
  let kernel_title_page = cls.replace(
    "\\renewcommand{\\maketitle}{\\ifADOC@cernpreprint\\else\\ATLAS@TitleHeader\\fi\n  {\\centering\\@title\\par}{\\centering\\@author\\par}}",
    "\\renewcommand{\\maketitle}{\\begingroup\\@maketitle\\endgroup}",
  );
  assert_ne!(kernel_title_page, cls);
  let creator =
    |name: &str| format!("<creator role=\"author\"><personname>{name}</personname></creator>");
  let class = |options: &str| {
    tex.replace(
      "\\documentclass{atlasdoc}",
      &format!("\\documentclass[{options}]{{atlasdoc}}"),
    )
  };
  let date_set =
    |date: &str| tex.replace("\\author{Jane Doe}", &format!("\\author{{Jane Doe}}{date}"));
  // (tex, cls, title, author, date)
  type Case<'a> = (
    String,
    String,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
  );
  let cases: Vec<Case> = vec![
    (
      tex.to_string(),
      cls.to_string(),
      Some("T"),
      Some("Jane Doe"),
      Some("DEFAULT DATE"),
    ),
    (
      class("nodatetop").replace("\\author{Jane Doe}", ""),
      cls.to_string(),
      Some("T"),
      Some("The ATLAS Collaboration"),
      None,
    ),
    (
      class("cernpreprint"),
      cls.to_string(),
      Some("T"),
      Some("Jane Doe"),
      None,
    ),
    (
      date_set("\\date{}"),
      cls.to_string(),
      Some("T"),
      Some("Jane Doe"),
      None,
    ),
    (
      date_set("\\date{}\\date{X}"),
      cls.to_string(),
      Some("T"),
      Some("Jane Doe"),
      Some("X"),
    ),
    (
      tex.replace("\\author{Jane Doe}", "\\author{}"),
      cls.to_string(),
      Some("T"),
      None,
      Some("DEFAULT DATE"),
    ),
    (
      tex.replace("\\title{T}", "\\title{}"),
      cls.replace(
        "\\date{DEFAULT DATE}",
        "\\date{DEFAULT DATE}\\title{DEFAULT TITLE}",
      ),
      None,
      Some("Jane Doe"),
      Some("DEFAULT DATE"),
    ),
    (
      tex.replace("\\author{Jane Doe}", ""),
      kernel_title_page.clone(),
      Some("T"),
      Some("The ATLAS Collaboration"),
      Some("DEFAULT DATE"),
    ),
    (
      tex.replace("\\author{Jane Doe}", ""),
      kernel_title_page.replace(
        "\\renewcommand{\\maketitle}",
        "\\DeclareRobustCommand{\\maketitle}",
      ),
      Some("T"),
      Some("The ATLAS Collaboration"),
      Some("DEFAULT DATE"),
    ),
    (
      tex.to_string(),
      cls.replace(
        "{\\ifADOC@datetop\\@date\\par\\fi}",
        "{\\unless\\ifADOC@datetop\\@date\\par\\fi}",
      ),
      Some("T"),
      Some("Jane Doe"),
      None,
    ),
  ];
  for (tex, cls, title, author, date) in cases {
    let (log, xml) = convert_files_with(&tex, &[("atlasdoc.cls", &cls)], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{tex}\n{log}"
    );
    match title {
      Some(title) => assert_element(&xml, "title", &[], &format!("<title>{title}</title>")),
      None => assert!(!xml.contains("<title"), "no title: {tex}\n{xml}"),
    }
    match author {
      Some(author) => {
        assert_eq!(
          xml.matches("<creator ").count(),
          1,
          "one creator: {tex}\n{xml}"
        );
        assert_element(&xml, "creator", &[], &creator(author));
      },
      None => assert!(!xml.contains("<creator"), "no creator: {tex}\n{xml}"),
    }
    match date {
      Some(date) => assert_element(
        &xml,
        "date",
        &[],
        &format!("<date role=\"creation\">{date}</date>"),
      ),
      None => assert!(!xml.contains("<date"), "no date: {tex}\n{xml}"),
    }
  }
}

/// A name the copy `\let` to its own version of a name the binding owns takes the binding's back: emulateapj5.sty:386
/// `\def\keywords`, :394 `\let\subjectheadings=\keywords`, over the aas_support binding's own `\let` (astro-ph0011001).
/// pdflatex (an article stand-in for aastex.cls, absent from TL 2025): "Subject headings: stars"; the binding makes
/// the keywords.
#[test]
fn copy_let_alias_keeps_the_binding() {
  let (log, xml) = convert_files_with(
    repro!("alias/copy_let_alias_keeps_the_binding.tex"),
    &[("emulateapj5.sty", repro!("alias/emulateapj5.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords:\u{a0}\">stars</keywords>",
  );
  assert!(!xml.contains("Subject headings"), "{xml}");
}

/// color.sty:117-122 accepts a model whose `\color@<model>` exists (spotcolor.sty:71): ieeeaccess.cls:131's spot
/// `accessblue` keeps the earlier CMYK one, at Info (2007.03025, 2009.04554, 2011.07807, 2209.03883, 2209.12464,
/// 2609.20467; KNOWN_PERL_ERRORS #958). pdflatex: 0 errors.
#[test]
fn spot_color_model_keeps_the_earlier_color() {
  let (log, xml) = convert_files_with(
    repro!("spotcolor/spot_color_model_keeps_the_earlier_color.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "text",
    &["xml:id=\"p1.1.1\""],
    "<text color=\"#0080CC\" xml:id=\"p1.1.1\">X</text>",
  );
  assert!(
    log.contains(
      "Info:color:spotcolor Color accessblue of the driver's model 'spotcolor' is not rendered"
    ),
    "{log}"
  );
}

/// A spot colour whose name has no earlier definition takes the driver's alternate, the CMYK of its `\AddSpotColor`
/// (spotcolor.sty:31-41, the spotcolor binding), and a spot colour no `\AddSpotColor` gave black, at Info (the
/// reviewer's probe spot_new; ieeeaccess.cls:126-131's `PANTONE3015C`). pdflatex: 0 errors.
#[test]
fn spot_color_defined_first_takes_the_alternate() {
  let (log, xml) = convert_files_with(
    repro!("spotcolor/spot_color_defined_first_takes_the_alternate.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "text",
    &["xml:id=\"p1.1.1\""],
    "<text color=\"#0080CC\" xml:id=\"p1.1.1\">X</text>",
  );
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\"><text color=\"#0080CC\" xml:id=\"p1.1.1\">X</text> Y</p>",
  );
  assert!(
    log.contains("Info:color:spotcolor Color myblue of the driver's model 'spotcolor' is not rendered; defined as its alternate cmyk 1,0.3,0,0.2"),
    "{log}"
  );
  assert!(
    log.contains("Info:color:spotcolor Color myother of the driver's model 'spotcolor' is not rendered; defined black"),
    "{log}"
  );
}

/// A counter the stand-in binding did not number is the copy's: jheppub.sty:313-314 renumbers the kernel's equations
/// per section (2112.14648, 2607.20603). pdflatex: "(1.1)", "(2.1)".
#[test]
fn copy_numbers_the_kernel_counters() {
  let (log, xml) = convert_files_with(
    repro!("counters/copy_numbers_the_kernel_counters.tex"),
    &[("jheppub.sty", repro!("counters/jheppub.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, tag) in [("S1.E1", "1.1"), ("S2.E1", "2.1")] {
    let equation =
      latexml::util::test::xml_element(&xml, "equation", &[&format!("xml:id=\"{id}\"")])
        .unwrap_or_else(|| panic!("no equation {id}: {xml}"));
    assert_element(
      &equation,
      "tags",
      &[],
      &format!(
        "<tags><tag>({tag})</tag><tag role=\"autoref\">Equation\u{a0}{tag}<text xml:id=\"{id}.1\"/></tag><tag \
         role=\"refnum\">{tag}</tag></tags>"
      ),
    );
  }
}

/// The copy's identity (OXIDIZED_DESIGN_DIVERGENCES #731 rule 0): its `\ProvidesExplClass` declaration (2308.11432's
/// fcs.cls), a declaration of the class its binding is built on (2011.13564's IEEEtai.cls, `\ProvidesClass{IEEEtran}`)
/// read it; a binding with no command set of its own and no declaration read none (citesort.sty; aipcheck.tex,
/// mssymb.tex, revtex.sty in the census). pdflatex: "EXTRA", "EXTRA", "READ" (the last one the binding's: base
/// behaviour).
#[test]
fn copy_identity_by_declaration_and_command_set() {
  for (tex, file, copy, text) in [
    (
      repro!("identity/declared_expl_class_is_read.tex"),
      "fcs.cls",
      repro!("identity/fcs.cls"),
      "EXTRA",
    ),
    (
      repro!("identity/declared_base_class_is_read.tex"),
      "IEEEtai.cls",
      repro!("identity/IEEEtai.cls"),
      "EXTRA",
    ),
    (
      repro!("identity/empty_command_set_is_not_read.tex"),
      "citesort.sty",
      repro!("identity/citesort.sty"),
      "NOT READ",
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
      &format!("<p xml:id=\"p1.1\">{text}</p>"),
    );
  }
}

/// A package option list keeps every special whose catcode is LETTER (`_` under `\ExplSyntaxOn`): fcs.cls:150
/// `\LoadClass [ twoside, \g__fcs_article_fontsize_tl ] { article }` names its token list (2308.11432; the reviewer's
/// probe expl_opts). pdflatex: "EXTRA".
#[test]
fn package_options_keep_expl_letters() {
  let (log, xml) = convert_files_with(
    repro!("options/package_options_keep_expl_letters.tex"),
    &[("fcs.cls", repro!("options/fcs.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">EXTRA</p>",
  );
}

/// amsart's `\copyrightinfo{}{}` adds no copyright: amsart.cls:723-725 `\@setcopyright` prints none without a year
/// (jgcc.cls:397; 2309.03819, 2401.15504; Perl ams_support.sty.ltxml:139 adds ", ", KNOWN_PERL_ERRORS #959); a year
/// and holder add "2020, Me". pdflatex: 0 errors.
#[test]
fn empty_copyrightinfo_adds_no_copyright() {
  let tex = repro!("copyright/empty_copyrightinfo_adds_no_copyright.tex");
  for (tex, copyright) in [
    (tex.to_string(), None),
    (
      tex.replace("\\copyrightinfo{}{}", "\\copyrightinfo{2020}{Me}"),
      Some("2020, Me"),
    ),
  ] {
    let (log, xml) = convert_files_with(&tex, &[], Some("ar5iv.sty"));
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    match copyright {
      Some(text) => assert_element(
        &xml,
        "date",
        &["role=\"copyright\""],
        &format!("<date name=\"©\" role=\"copyright\">{text}</date>"),
      ),
      None => assert!(!xml.contains("role=\"copyright\""), "{xml}"),
    }
  }
}

/// A renamed copy of natbib is a stand-in for natbib (72a r4): the binding loads natbib's with the options the class
/// gave the copy, and the copy only provides — its extra stands, natbib's names stay the binding's. bmvc2k_natbib.sty
/// (bmvc2k.cls:710 `[sort,numbers]`; 2412.14509, 2212.00057, 2210.06270, 1501.07873, 2311.17607, 2406.08486) heads the
/// bibliography "References"; NJDnatbib.sty (WileyNJDv5.cls:6021 `[numbers,super,…]`; 2609.08425, 2101.06297,
/// 2211.16474, 2411.11493, 2206.08798, 2602.03829, 2408.07575, 2506.14995) "REFERENCES" in a chapterless class
/// (NJDnatbib.sty:1050-1053). Read raw, their `\@ifundefined{chapter}` took the `\chapter*{\bibname}` branch:
/// "Bibliography", `bib.bibx1`. pdflatex: "EXTRA [1]" / "References"; "EXTRA [1]" / "REFERENCES".
#[test]
fn renamed_natbib_copies_are_natbib() {
  for (tex, file, copy, cite, title) in [
    (
      repro!("natbib_forks/bmvc2k_natbib_is_natbib.tex"),
      "bmvc2k_natbib.sty",
      repro!("natbib_forks/bmvc2k_natbib.sty"),
      "<cite class=\"ltx_citemacro_cite\">[<bibref bibrefs=\"a\" separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>",
      "References",
    ),
    (
      repro!("natbib_forks/njdnatbib_is_natbib.tex"),
      "NJDnatbib.sty",
      repro!("natbib_forks/NJDnatbib.sty"),
      "<cite class=\"ltx_citemacro_cite\"><sup xml:id=\"p1.1.1\"><bibref bibrefs=\"a\" separator=\",\" show=\"Number\" yyseparator=\",\"/></sup></cite>",
      "REFERENCES",
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
      &format!("<p xml:id=\"p1.1\">EXTRA {cite}</p>"),
    );
    let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
      .unwrap_or_else(|| panic!("no bibliography: {xml}"));
    assert_element(
      &bibliography,
      "title",
      &[],
      &format!("<title>{title}</title>"),
    );
    assert!(
      bibliography.contains("<bibitem key=\"a\" xml:id=\"bib.bib1\">"),
      "{bibliography}"
    );
  }
}

/// NJDnatbib's bibliography heading by version (njdnatbib_sty.rs): a v5 copy's chapterless default "REFERENCES"
/// (NJDnatbib.sty:1052 `\bmsection*{…REFERENCES…}`: 2602.03829, 2408.07575, 2609.08425), a v2 or MRM one natbib's
/// `\refname` (`\section*{\refname…}`: 2101.06297, 2211.16474, 2411.11493, 2206.08798, 2506.14995) — under article
/// and under the version's own class binding (WileyNJDv5, WileyNJD-v2), whose OmniBus base defines `\chapter` today,
/// and again with `\chapter` undefined, as a class without chapters leaves it, with a `chapter` counter (the reviewer's
/// probe njd_v5_ctr: pdflatex "REFERENCES") and with both, as the real Wiley classes have them (WileyNJDv5.cls:5728,
/// MRM.cls:2160 `\newcounter{chapter}[part]`, no `\chapter`; `state::class_defines_chapter`). Every version's superscript carries the
/// postnote (:366-368). pdflatex on the real forks (the reviewer's probes njd_direct_v2, njd_direct_v5): "References",
/// "REFERENCES"; "1 and 2, p. 5" raised.
#[test]
fn njdnatbib_heading_by_version() {
  let chapter_undefined = "\\makeatletter\\let\\chapter\\@undefined\\makeatother\n";
  // (the real Wiley classes: a `chapter` counter and no `\chapter`, WileyNJDv5.cls:5728, MRM.cls:2160)
  let wiley = "\\newcounter{chapter}\\makeatletter\\let\\chapter\\@undefined\\makeatother\n";
  for (tex, copy, class, title) in [
    (
      repro!("natbib_forks/njdnatbib_is_natbib.tex"),
      repro!("natbib_forks/NJDnatbib.sty"),
      "WileyNJDv5",
      "REFERENCES",
    ),
    (
      repro!("natbib_forks/njdnatbib_v2_is_natbib.tex"),
      repro!("natbib_forks/v2/NJDnatbib.sty"),
      "WileyNJD-v2",
      "References",
    ),
  ] {
    for documentclass in ["article", class] {
      for chapter in ["", chapter_undefined, "\\newcounter{chapter}\n", wiley] {
        let source = tex.replace(
          "\\documentclass{article}\n",
          &format!("\\documentclass{{{documentclass}}}\n{chapter}"),
        );
        let (log, xml) = convert_files_with(&source, &[("NJDnatbib.sty", copy)], Some("ar5iv.sty"));
        let case = format!("{documentclass} {chapter:?} {title}");
        assert_eq!(
          (error_count(&log), warning_count(&log)),
          (0, 0),
          "{case}: {log}"
        );
        let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
          .unwrap_or_else(|| panic!("{case}: no bibliography: {xml}"));
        assert_element(
          &bibliography,
          "title",
          &[],
          &format!("<title>{title}</title>"),
        );
        if title == "References" {
          assert!(
            xml.contains("yyseparator=\",\"/>, p.\u{a0}5</sup></cite>"),
            "{case}: the postnote inside the superscript: {xml}"
          );
        }
      }
    }
  }
}

/// MRM.cls:2441-2442 loads NJDnatbib `[numbers,super]` under its AMA option and :3191 sets `\refname` "REFERENCES"; the
/// binding no longer loads natbib optionless first, so the class's copy sets the mode (2506.14995's PDF: superscripts,
/// "REFERENCES"); a natbib loaded already takes a copy's options as its own options, with an Info. ws-rv-har.sty is
/// natbib too (2301.03442: "(Mayor et al. 2011)", "References").
#[test]
fn renamed_natbib_copies_take_the_class_options() {
  let (log, xml) = convert_files_with(
    repro!("natbib_forks/mrm_loads_natbib_where_the_class_does.tex"),
    &[
      ("MRM.cls", repro!("natbib_forks/MRM.cls")),
      ("NJDnatbib.sty", repro!("natbib_forks/v2/NJDnatbib.sty")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_cite\"><sup xml:id=\"p1.1.1\"><bibref bibrefs=\"a\" separator=\",\" show=\"Number\" yyseparator=\",\"/></sup></cite> and <cite class=\"ltx_citemacro_cite\"><sup xml:id=\"p1.1.2\"><bibref bibrefs=\"a\" separator=\",\" show=\"Number\" yyseparator=\",\"/>, p.\u{a0}5</sup></cite>.</p>",
  );
  let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
    .unwrap_or_else(|| panic!("no bibliography: {xml}"));
  assert_element(&bibliography, "title", &[], "<title>REFERENCES</title>");

  // (a natbib loaded already takes the copy's options as a repeat load does, as natbib's options, with an Info)
  let (log, xml) = convert_files_with(
    repro!("natbib_forks/natbib_loaded_takes_the_copy_options.tex"),
    &[("NJDnatbib.sty", repro!("natbib_forks/v2/NJDnatbib.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("Applying options 'numbers,super' from a repeat load of natbib.sty"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_cite\"><sup xml:id=\"p1.1.1\"><bibref bibrefs=\"a\" separator=\",\" show=\"Number\" yyseparator=\",\"/></sup></cite>.</p>",
  );

  let (log, xml) = convert_files_with(
    repro!("natbib_forks/ws_rv_har_is_natbib.tex"),
    &[("ws-rv-har.sty", repro!("natbib_forks/ws-rv-har.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">EXTRA <cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"a\" separator=\";\" show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>)</cite></p>",
  );
  let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
    .unwrap_or_else(|| panic!("no bibliography: {xml}"));
  assert_element(&bibliography, "title", &[], "<title>References</title>");
  // (a class with `\chapter` heads it `\bibname`, ws-rv-har.sty:676-685: book's "Bibliography"; ws-rv9x6.cls:499, :91
  // "References" in 2301.03442)
  let (log, xml) = convert_files_with(
    &repro!("natbib_forks/ws_rv_har_is_natbib.tex").replace("{article}", "{book}"),
    &[("ws-rv-har.sty", repro!("natbib_forks/ws-rv-har.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
    .unwrap_or_else(|| panic!("no bibliography: {xml}"));
  assert_element(&bibliography, "title", &[], "<title>Bibliography</title>");
}

/// The spotcolor `[pantone]` preload (spotcolor.sty:81-82) keeps each colour's CMYK alternate: PANTONE 103 PC is
/// cmyk 0 0.03 1 0.18 (spotcolorpantone.tex:5; the reviewer's probe spot_pantone), not black.
#[test]
fn spot_color_preload_takes_the_alternate() {
  let (log, xml) = convert_files_with(
    repro!("spotcolor/spot_color_pantone_preload_takes_the_alternate.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\"><text color=\"#D1C900\" xml:id=\"p1.1.1\">X</text></p>",
  );
}

/// A stand-in's copy declares an environment whole: cimart.cls's `\newtheorem{theorem}[definition]` over OmniBus's
/// theorem stubs (2412.00032; 2408.07575's `{example}`; the reviewer's probe thm_copy). pdflatex: "Definition 1.1",
/// "Theorem 1.2", no group left open.
#[test]
fn copy_environments_decided_as_a_pair() {
  let (log, xml) = convert_files_with(
    repro!("environments/copy_environments_decided_as_a_pair.tex"),
    &[("cimart.cls", repro!("environments/cimart.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (the two share the `definition` counter, cimart.cls:295-315: `S1.Thmdefinition2` is the theorem)
  for (n, class, tag, autoref, body) in [
    (1, "definition", "Definition 1.1", "1.1", "A def."),
    (
      2,
      "theorem",
      "Theorem 1.2",
      "Theorem\u{a0}1.2",
      "<text font=\"italic\" xml:id=\"S1.Thmdefinition2.p1.1.1\">A thm.</text>",
    ),
  ] {
    let id = format!("S1.Thmdefinition{n}");
    assert_element(
      &xml,
      "theorem",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<theorem class=\"ltx_theorem_{class}\" inlist=\"thm theorem:{class}\" xml:id=\"{id}\">\n          <tags>\n            <tag>{tag}</tag>\n            <tag role=\"autoref\">{autoref}<text xml:id=\"{id}.1\"/></tag>\n            <tag role=\"refnum\">1.{n}</tag>\n            <tag role=\"typerefnum\">{tag}</tag>\n          </tags>\n          <title class=\"ltx_runin\"><tag><text font=\"sansserif bold\" xml:id=\"{id}.2\">{tag}</text></tag><text font=\"sansserif bold\" xml:id=\"{id}.3\">.</text></title>\n          <para xml:id=\"{id}.p1\">\n            <p xml:id=\"{id}.p1.1\">{body}</p>\n          </para>\n        </theorem>"
      ),
    );
  }
}

/// The copy's identity: a declaration naming a macro reads it (cimart.cls:13-14, 2412.00032), and a stand-in a file
/// the binding loads requests is not the binding's command set (OmniBus's aas_macros under asme2ej: 2309.12017;
/// nature-pre: 2102.01782, 2111.09589). pdflatex: "EXTRA" both.
#[test]
fn copy_identity_by_macro_declaration_and_own_command_set() {
  for (tex, file, copy) in [
    (
      repro!("identity/declared_by_macro_is_read.tex"),
      "cimart.cls",
      repro!("identity/cimart.cls"),
    ),
    (
      repro!("nested_load/nested_stand_in_joins_only_its_requester.tex"),
      "asme2ej.cls",
      repro!("nested_load/asme2ej.cls"),
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
      "<p xml:id=\"p1.1\">EXTRA</p>",
    );
  }
}

/// A text default the binding's base class or LaTeXML's kernel assigned is the copy's to set, as the real class sets
/// it over article.cls's: MRM.cls:3191 `\def\refname` (2506.14995), ws-rv9x6.cls:91 `\newcommand\bibname`
/// (2301.03442). pdflatex: "Summary", "Works", "REFERENCES".
#[test]
fn copy_text_defaults_hold() {
  let (log, xml) = convert_files_with(
    repro!("text_defaults/copy_text_defaults_hold.tex"),
    &[("cimart.cls", repro!("text_defaults/cimart.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<p xml:id=\"p1.1\">Works</p>"), "{xml}");
  assert!(
    xml.contains("<abstract inlist=\"toc\" name=\"Summary\""),
    "{xml}"
  );
  let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
    .unwrap_or_else(|| panic!("no bibliography: {xml}"));
  assert_element(&bibliography, "title", &[], "<title>REFERENCES</title>");
}

/// A repeat `\usepackage[…]{natbib}` after the document's world loaded natbib keeps the first options and is LaTeX's
/// "Option clash" Error (binding/content.rs `apply_new_options_on_reload`, `natbib.sty_repeat_clash`): the reviewer's
/// probes nat_twice and jmlr_numbers (jmlr.cls:150's own load), pdflatex 1 error and author-year. After a compiled
/// binding's own early load the later options apply (svjour3: sv_support's natbib, svjour3.cls:67-68 loads it only under
/// its `natbib` option; 2604.22515's `[numbers,sort&compress]`, as its PDF) — the 8 census papers 2112.14648,
/// 2607.20603, 2405.09762, 1012.4730, 2604.20688, 2107.02450, 2409.07188, 2604.22515.
#[test]
fn natbib_repeat_load_clashes_after_the_documents_load() {
  let tex = repro!("natbib_forks/natbib_repeat_after_the_documents_load_clashes.tex");
  let author_year = "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"a\" separator=\";\" show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>)</cite>.</p>";
  let numbers = "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a\" separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>.</p>";
  for (class, errors, paragraph) in [
    ("article", 1, author_year),
    ("jmlr", 1, author_year),
    ("svjour3", 0, numbers),
  ] {
    let source = tex.replace("{article}", &format!("{{{class}}}"));
    let (log, xml) = convert_files_with(&source, &[], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (errors, 0),
      "{class}: {log}"
    );
    assert_eq!(
      log.matches("Option clash for package natbib").count(),
      errors,
      "{class}: {log}"
    );
    assert_element(&xml, "p", &["xml:id=\"p1.1\""], paragraph);
  }
}

/// A stand-in's copy that `\LoadClass`es its base class has that class's names: its `\newcommand` of one is LaTeX's
/// "Command … already defined" Error and the first definition stays (`state::copy_provides_text_default_clashes`;
/// article.cls:616 `\refname`, its `\figurename`). The reviewer's probe newcmd_base: pdflatex 2 errors, "Figure 1",
/// "References". A name the base class lacks is the copy's (`copy_text_defaults_hold`: `\bibname` "Works").
#[test]
fn copy_newcommand_of_a_base_class_name_errs() {
  let (log, xml) = convert_files_with(
    repro!("text_defaults/loadclass/copy_newcommand_of_a_base_class_name_errs.tex"),
    &[("cimart.cls", repro!("text_defaults/loadclass/cimart.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  for name in ["\\refname", "\\figurename"] {
    assert!(
      log.contains(&format!("Command {name} already defined")),
      "{name}: {log}"
    );
  }
  assert_element(
    &xml,
    "figure",
    &[],
    "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><p xml:id=\"S0.F1.2\">x</p><toccaption><tag close=\" \">1</tag>C</toccaption><caption><tag close=\": \">Figure 1</tag>C</caption></figure>",
  );
  let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
    .unwrap_or_else(|| panic!("no bibliography: {xml}"));
  assert_element(&bibliography, "title", &[], "<title>References</title>");
}

/// A `chapter` counter with no `\chapter` numbers no appendix within chapters (latex_constructs `begin_appendices`,
/// `state::class_defines_chapter`): WileyNJD-v2.cls:2410's bookmark counter, declared for real now that a stand-in copy's
/// counter is (2211.16474: "Appendix A", not "Appendix 0.A"). pdflatex: "A".
#[test]
fn chapter_counter_without_chapter_numbers_appendix_alone() {
  let (log, xml) = convert_files_with(
    repro!("counters/chapter_counter_without_chapter_appendix.tex"),
    &[],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let appendix = latexml::util::test::xml_element(&xml, "appendix", &[])
    .unwrap_or_else(|| panic!("no appendix: {xml}"));
  assert_element(
    &appendix,
    "title",
    &[],
    "<title><tag close=\" \">Appendix A</tag>Extra</title>",
  );
}

/// A stand-in's copy that loads natbib itself makes the load TeX sees, which the binding's early load only anticipated
/// (binding/content.rs `apply_new_options_on_reload`): its options apply and join the first load's, and the document's
/// later `\usepackage[numbers]{natbib}` is LaTeX's "Option clash", the author-year citations kept (the reviewer's
/// probes neurips_rep, icml_rep: neurips_2024.sty:71, icml2024.sty:95; pdflatex 1 error). A copy that comments its load
/// out leaves the binding's early load, so the document's options apply (2506.00573, 2506.01987, 2506.08967,
/// 2506.17103, 2506.18071, 2506.22567: numbered, 0 errors); a copy's `[square,numbers]` covers the document's `numbers`
/// (pdflatex 0 errors, "[1, 2]"). Each numbered one also after a `\bibliographystyle{plainnat}`, which an explicit
/// style option makes inert (natbib.sty:238-262 `nobibstyle`; 2506.08967, pdflatex "[1, 2]").
#[test]
fn natbib_copy_repeat_ends_the_bindings_early_load() {
  let tex = repro!("natbib_forks/natbib_copy_repeat_ends_the_early_load.tex");
  let copy = repro!("natbib_forks/copy_load/neurips_2024.sty");
  let author_year = "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"a,b\" separator=\";\" show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>)</cite>.</p>";
  let numbers = "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a,b\" separator=\",\" show=\"Number\" yyseparator=\",\"/>]</cite>.</p>";
  for (case, load, errors, paragraph) in [
    (
      "copy loads natbib",
      "\\RequirePackage{natbib}",
      1,
      author_year,
    ),
    (
      "load commented out",
      "%\\RequirePackage{natbib}",
      0,
      numbers,
    ),
    (
      "copy's own options",
      "\\RequirePackage[square,numbers]{natbib}",
      0,
      numbers,
    ),
  ] {
    let copy = copy.replace("\\RequirePackage{natbib}", load);
    // (the numbered cases; under author-year a `\bibliographystyle` still sets the punctuation)
    let styles: &[&str] = if errors == 0 {
      &["", "\\bibliographystyle{plainnat}\n"]
    } else {
      &[""]
    };
    for style in styles {
      let tex = tex.replace("\\begin{document}", &format!("{style}\\begin{{document}}"));
      let (log, xml) = convert_files_with(&tex, &[("neurips_2024.sty", &copy)], Some("ar5iv.sty"));
      assert_eq!(
        (error_count(&log), warning_count(&log)),
        (errors, 0),
        "{case} {style}: {log}"
      );
      assert_eq!(
        log.matches("Option clash for package natbib").count(),
        errors,
        "{case} {style}: {log}"
      );
      assert_element(&xml, "p", &["xml:id=\"p1.1\""], paragraph);
    }
  }
}

/// "The base class defines this name" by the class load, not the class text (`state::ClassLoadFrame`,
/// OXIDIZED_DESIGN_DIVERGENCES #732): the article binding assigns article.cls:613-622's names in its own load, a raw
/// base class defines its own for real, and a class loaded while the copy's `\LoadClass` is open is the copy's base
/// class too. The reviewer's probes, each as pdflatex: nc_chain (`\LoadClass{mybase}` over article: 2 errors, "Figure
/// 1", "References"), nc_namedef (mybase's `\expandafter\def\csname bibname\endcsname`: 1 error), nc_cond (its
/// `\AtBeginDocument{\def\bibname…}` has not run: 0 errors), nc_letrelax (mybase's `\let\refname\relax`, definable as
/// for LaTeX's `\newcommand`: 0 errors, "Fig 1", "Works Cited"); and the copy's own `\let…\relax` before its
/// `\newcommand`.
#[test]
fn copy_newcommand_through_a_chained_base_class() {
  let tex = repro!("text_defaults/chain/copy_newcommand_through_a_chained_base_class_errs.tex");
  let cimart = repro!("text_defaults/chain/cimart.cls");
  let mybase = repro!("text_defaults/chain/mybase.cls");
  let tag = |tag: &str| format!("<caption><tag close=\": \">{tag}</tag>C</caption>");
  for (case, cimart, mybase, errors, figure, heading) in [
    (
      "nc_chain",
      cimart.to_string(),
      mybase.to_string(),
      vec!["\\refname", "\\figurename"],
      "Figure 1",
      "References",
    ),
    (
      "nc_namedef",
      cimart.replace(
        "\\newcommand\\refname{Works Cited}\n\\newcommand\\figurename{Fig}\n",
        "\\newcommand\\bibname{Works}\n",
      ),
      s_add(
        mybase,
        "\\expandafter\\def\\csname bibname\\endcsname{Literature}\n",
      ),
      vec!["\\bibname"],
      "Figure 1",
      "References",
    ),
    (
      "nc_cond",
      cimart.replace(
        "\\newcommand\\refname{Works Cited}\n\\newcommand\\figurename{Fig}\n",
        "\\newcommand\\bibname{Works}\n",
      ),
      s_add(mybase, "\\AtBeginDocument{\\def\\bibname{Literature}}\n"),
      vec![],
      "Figure 1",
      "References",
    ),
    (
      "nc_letrelax",
      cimart.to_string(),
      s_add(mybase, "\\let\\refname\\relax\n\\let\\figurename\\relax\n"),
      vec![],
      "Fig 1",
      "Works Cited",
    ),
    (
      "the copy's own \\let\\relax",
      cimart.replace(
        "\\LoadClass{mybase}\n",
        "\\LoadClass{article}\n\\let\\refname\\relax\n\\let\\figurename\\relax\n",
      ),
      mybase.to_string(),
      vec![],
      "Fig 1",
      "Works Cited",
    ),
  ] {
    let (log, xml) = convert_files_with(
      tex,
      &[("cimart.cls", &cimart), ("mybase.cls", &mybase)],
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (errors.len(), 0),
      "{case}: {log}"
    );
    for name in &errors {
      assert!(
        log.contains(&format!("Command {name} already defined")),
        "{case} {name}: {log}"
      );
    }
    assert!(xml.contains(&tag(figure)), "{case}: {xml}");
    let bibliography = latexml::util::test::xml_element(&xml, "bibliography", &[])
      .unwrap_or_else(|| panic!("{case}: no bibliography: {xml}"));
    assert_element(
      &bibliography,
      "title",
      &[],
      &format!("<title>{heading}</title>"),
    );
  }
}

fn s_add(base: &str, extra: &str) -> String { format!("{base}{extra}") }

/// The class loads' record is the conversion's own: a long-lived worker converting two papers on one thread
/// (cortex_worker: a `Converter` per paper, no engine reset between them) does not carry the first paper's base class
/// into the second, whose `mybase.cls` of the same name defines no `\bibname` (the r6 class-text cache, keyed by class
/// name, did): 1 error, then 0.
#[test]
fn class_load_record_does_not_reach_the_next_document() {
  let tex = repro!("text_defaults/chain/copy_newcommand_through_a_chained_base_class_errs.tex");
  let cimart = repro!("text_defaults/chain/cimart.cls").replace(
    "\\newcommand\\refname{Works Cited}\n\\newcommand\\figurename{Fig}\n",
    "\\newcommand\\bibname{Works}\n",
  );
  let mybase = repro!("text_defaults/chain/mybase.cls");
  let papers: Vec<tempfile::TempDir> = ["\\def\\bibname{Literature}\n", ""]
    .iter()
    .map(|extra| {
      let dir = tempfile::tempdir().expect("a paper directory");
      std::fs::write(dir.path().join("paper.tex"), tex).expect("the paper");
      std::fs::write(dir.path().join("cimart.cls"), &cimart).expect("the copy");
      std::fs::write(dir.path().join("mybase.cls"), format!("{mybase}{extra}"))
        .expect("the base class");
      dir
    })
    .collect();
  let sources: Vec<String> = papers
    .iter()
    .map(|dir| dir.path().join("paper.tex").to_string_lossy().into_owned())
    .collect();
  let logs = convert_on_one_thread(sources);
  for (log, errors) in logs.iter().zip([1, 0]) {
    assert_eq!((error_count(log), warning_count(log)), (errors, 0), "{log}");
    assert_eq!(
      log.matches("Command \\bibname already defined").count(),
      errors,
      "{log}"
    );
  }
  drop(papers);
}

/// One class slot (OXIDIZED_DESIGN_DIVERGENCES #736; latex.ltx:18800, :18938-18942 `\@twoloadclasserror`): a stand-in
/// class's copy whose own `\LoadClass` names a class the paper does not ship, after the binding loaded its base class,
/// does not load a second class over the first — ceurart.cls:62 `\LoadClass{scrartcl}` raw-read typearea over article
/// (Fatal "Conditional limit of 48000000 exceeded": 2509.11496, 2608.23214, 2310.02658, 2505.16276), atlasdoc.cls's
/// `\LoadClass{\ADOC@mainclass}` redefined `\@startsection` and `\@sect` (2509.01219). pdflatex: "A Title", "1 Intro",
/// "Text CEUR."; "1 Intro", "Text ATLAS."; 0 errors. A class the author ships is still read (the nc_chain case of
/// `copy_newcommand_through_a_chained_base_class`).
#[test]
fn copy_load_class_is_the_bindings_base_class() {
  let (log, xml) = convert_files_with(
    repro!("class_slot/copy_load_class_is_the_bindings_base_class.tex"),
    &[("ceurart.cls", repro!("class_slot/ceurart.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!(
    (
      error_count(&log),
      warning_count(&log),
      log.matches("Fatal:").count()
    ),
    (0, 0, 0),
    "{log}"
  );
  assert!(
    log.contains(
      "Info:provide:scrartcl.cls the binding's base class (OmniBus.cls, article.cls) stands for the copy's \
       \\LoadClass{scrartcl}: not loaded"
    ),
    "{log}"
  );
  assert_element(&xml, "title", &[], "<title>A Title</title>");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text \
     xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">§1</tag></tags><title><tag \
     close=\" \">1</tag>Intro</title><para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\">Text CEUR.</p></para></section>",
  );

  let (log, xml) = convert_files_with(
    repro!("class_slot/atlasdoc_main_class_is_the_bindings_base_class.tex"),
    &[("atlasdoc.cls", repro!("class_slot/atlasdoc.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!(
    (
      error_count(&log),
      warning_count(&log),
      log.matches("Fatal:").count()
    ),
    (0, 0, 0),
    "{log}"
  );
  assert!(!log.contains("KOMA"), "{log}");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag \
     role=\"typerefnum\">§1</tag></tags><title><tag close=\" \">1</tag>Intro</title><para xml:id=\"S1.p1\"><p \
     xml:id=\"S1.p1.1\">Text ATLAS.</p></para></section>",
  );
}

/// natbib's repeat options run in natbib's declaration order, as `\ProcessOptions` runs them (natbib.sty:238-263):
/// `[square,authoryear]` is authoryear's `bibstyle` then square's `nobibstyle`, so `\bibliographystyle{plainnat}` is
/// gobbled. pdflatex (the copy's own load commented out, so the document's load is natbib's first): "[Smith, 2001;
/// Jones et al., 2005]"; in the order written, plainnat's "," separated the entries.
#[test]
fn natbib_repeat_options_in_declaration_order() {
  let copy = repro!("natbib_forks/copy_load/neurips_2024.sty")
    .replace("\\RequirePackage{natbib}", "%\\RequirePackage{natbib}");
  let (log, xml) = convert_files_with(
    repro!("natbib_forks/natbib_repeat_options_in_declaration_order.tex"),
    &[("neurips_2024.sty", &copy)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a,b\" separator=\";\" \
     show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>]</cite>.</p>",
  );
}

/// Convert each of `sources` with one `Converter` each on one thread, no engine reset between them, as the CLI's
/// `latexml_oxide t.tex u.tex` and a long-lived worker do; the logs.
fn convert_on_one_thread(sources: Vec<String>) -> Vec<String> {
  use std::rc::Rc;

  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Info);
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        preload: Some(vec![String::from("ar5iv.sty")]),
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let logs: Vec<String> = sources
        .into_iter()
        .map(|source| {
          let mut converter = Converter::from_config(opts.clone());
          converter.note_main_source(&source);
          converter.prepare_session(&opts).expect("a session");
          converter.convert(source).log
        })
        .collect();
      latexml_core::reset_thread_engine();
      logs
    })
    .expect("a conversion thread")
    .join()
    .expect("the conversions")
}

/// The files read raw are the conversion's own (`reset_raw_read_paths` at every `prepare_session`): two documents of
/// one directory on one thread (the reviewer's probe two, `latexml_oxide t.tex u.tex`, both cimart) each read the
/// copy, 2 errors each as pdflatex; the second read nothing before.
#[test]
fn raw_read_record_does_not_reach_the_next_document() {
  let tex = repro!("text_defaults/loadclass/copy_newcommand_of_a_base_class_name_errs.tex");
  let dir = tempfile::tempdir().expect("a paper directory");
  for name in ["t.tex", "u.tex"] {
    std::fs::write(dir.path().join(name), tex).expect("the paper");
  }
  std::fs::write(
    dir.path().join("cimart.cls"),
    repro!("text_defaults/loadclass/cimart.cls"),
  )
  .expect("the copy");
  let sources: Vec<String> = ["t.tex", "u.tex"]
    .iter()
    .map(|name| dir.path().join(name).to_string_lossy().into_owned())
    .collect();
  for log in convert_on_one_thread(sources) {
    assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
    assert!(log.contains("Command \\refname already defined"), "{log}");
  }
  drop(dir);
}
