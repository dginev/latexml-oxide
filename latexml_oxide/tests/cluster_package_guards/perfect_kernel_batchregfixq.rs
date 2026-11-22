//! Red/green guards for batch REGFIXQ, v0.8.0 regressions of a paper's copy read beside the binding that stands for
//! its file: a renamed copy is read once, where its binding reads its own file raw; a copy whose binding is loaded
//! already is still read; and what a binding sets up beyond its file (an option's effect, a borrowed package, fields its
//! file initialises) is what the copy's own code finds, as in TeX.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Converts `tex` beside `files` with ar5iv.sty preloaded; checks the Error and Warning counts; returns (log, XML).
#[track_caller]
fn convert(tex: &str, files: &[(&str, &str)], counts: (usize, usize)) -> (String, String) {
  let (log, xml) = convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), counts, "{log}");
  (log, xml)
}

/// The paragraph `text`, the document's first.
#[track_caller]
fn first_paragraph(xml: &str, text: &str) {
  assert_element(
    xml,
    "para",
    &[r#"xml:id="p1""#],
    &format!(r#"<para xml:id="p1"><p xml:id="p1.1">{text}</p></para>"#),
  );
}

/// provides_route/smfart_renamed_copy_read_once: a shipped class declaring `\ProvidesClass{smfart}` under another name
/// (1401.0915's fsmfart.cls, smfartVS.cls of 1110.1467 and 1404.0804) is read once, where smfart_cls.rs reads its own
/// file raw (`content::routed_copy_request`). Red: the binding read TeX Live's smfart.cls, then the copy, whose
/// first-load test ran with amsmath loaded (`\fi` unexpected, 2 `\mathfrak` warnings). The same read serves a renamed
/// package (tikzcd.sty under tikz_cd_sty.rs, 1201.4414, 1512.05388; pdfplots.sty under pgfplots_sty.rs, 2105.12769).
#[test]
fn renamed_copy_is_read_once_as_its_bindings_file() {
  let (_, xml) = convert(
    repro!("provides_route", "smfart_renamed_copy_read_once.tex"),
    &[("smfcopy.cls", repro!("provides_route", "smfcopy.cls"))],
    (0, 1),
  );
  first_paragraph(&xml, "SMF-COPY-MACRO Text ok.");
}

/// provides_route/copy_of_a_loaded_binding_is_read: a shipped package declaring a bound file whose binding is loaded
/// already (2304.14243's packages/pictures.sty `\ProvidesPackage{packages/graphics}`, after graphicx) is still read.
/// Red: 1 Error (`\mypicmacro` undefined), the copy dropped silently. Distinct from the case above: no binding runs.
#[test]
fn copy_routed_to_a_loaded_binding_is_read() {
  let (_, xml) = convert(
    repro!("provides_route", "copy_of_a_loaded_binding_is_read.tex"),
    &[("picscopy.sty", repro!("provides_route", "picscopy.sty"))],
    (0, 1),
  );
  first_paragraph(&xml, "PICTURE-MACRO");
}

/// provides_route/renamed_class_reads_its_own_package: under a renamed class copy declaring `\ProvidesClass{imsart}`,
/// imsart_cls.rs's raw read of imsart.sty reads the copy's own package, which the copy loads (2405.11676's
/// imsart-altered.cls:149 `\RequirePackage{imsart-altered}`), not the unaltered imsart.sty the paper ships beside it.
/// Red: 1 Error (`\imscopymacro` undefined): imsart.sty read, the copy's package dropped. Distinct: a file of the
/// binding's other type (`routed_copy_request`'s sibling branch). The paragraph has no "UNALTERED-READ": the repro
/// tests `\imsunalteredmacro`, which only the unaltered imsart.sty defines.
#[test]
fn renamed_class_reads_its_own_package() {
  let (_, xml) = convert(
    repro!("provides_route", "renamed_class_reads_its_own_package.tex"),
    &[
      ("imscopy.cls", repro!("provides_route", "imscopy.cls")),
      ("imscopy.sty", repro!("provides_route", "imscopy.sty")),
      ("imsart.sty", repro!("provides_route", "imsart.sty")),
    ],
    (0, 1),
  );
  first_paragraph(&xml, "IMS-COPY-PACKAGE");
}

/// provides_route/titlesec_copy_passes_the_title: a shipped titlesec copy's format passes the title to
/// `\ttl@passexplicit` (titlesecMod.sty:1464, 1501.02633, 1505.02642; mytitlesec.sty, 1801.07174, 2007.08085;
/// titlesecfix.sty, 2002.06448), which titlesec_sty.rs defines as titlesec's `\ttl@select` lets it (:301). Red: 1
/// Error (`\ttl@passexplicit` undefined) in the heading.
#[test]
fn titlesec_copy_format_passes_the_title() {
  let (_, xml) = convert(
    repro!("provides_route", "titlesec_copy_passes_the_title.tex"),
    &[(
      "titlesec_copy.sty",
      repro!("provides_route", "titlesec_copy.sty"),
    )],
    (0, 1),
  );
  assert_element(
    &xml,
    "title",
    &[r#"font="bold""#],
    // (the 1em gap: an em space and a hair space)
    "<title font=\"bold\">1\u{2003}\u{200A}Introduction</title>",
  );
}

/// options/icml_option_after_the_copys_test: the icml binding's `nohyperref` takes effect at the package's end, after
/// the paper's copy, which tests `\nohyperref` before its own `\ProcessOptions` (1603.00748's icml2016.sty:88 vs :130).
/// Red: 2 Errors (the copy's rest skipped to its end, "conditional fell off end"; `\icmlcopymacro` undefined). The one
/// Warning is TeX's too: the copy leaves its first `\ifdefined` open (one `\fi` short), "(\end occurred when … was
/// incomplete)".
#[test]
fn icml_option_takes_effect_after_the_copys_test() {
  let (log, xml) = convert(
    repro!("options", "icml_option_after_the_copys_test.tex"),
    &[("icml2016.sty", repro!("options", "icml2016.sty"))],
    (0, 1),
  );
  assert!(
    log.contains("Attempt to end document with open groups, environments or conditionals"),
    "{log}"
  );
  first_paragraph(&xml, "ICML-COPY-MACRO");
}

/// beyond_original/sn_jnl_copy_without_amsthm_borrows_it: a sn-jnl.cls copy that does not load amsthm tests
/// `\@ifpackageloaded{amsthm}` at class time (:1547), false in pdflatex, so its theorem styles stay undefined; the
/// binding borrows amsthm there (2305.14168, 2408.00130, 2409.14383, 2503.20196, 2506.14776). Red: 1 Error
/// (`\proofthmtrue` undefined, thmstylefour's head). The Warning is amsthm's "Unknown theoremstyle", as in pdflatex.
#[test]
fn sn_jnl_copy_without_amsthm_borrows_it() {
  let (log, xml) = convert(
    repro!(
      "beyond_original",
      "sn_jnl_copy_without_amsthm_borrows_it.tex"
    ),
    &[("sn-jnl.cls", repro!("beyond_original", "sn-jnl.cls"))],
    (0, 1),
  );
  assert!(
    log.contains(
      "Warning:undefined:theoremstyle Unknown theorem style 'thmstylefour', reverting to 'plain'."
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "title",
    &[r#"class="ltx_runin""#],
    r#"<title class="ltx_runin"><tag><text font="bold" xml:id="Thmlemma1.2">Lemma 1</text></tag><text font="bold" xml:id="Thmlemma1.3">.</text></title>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="Thmlemma1.p1.1""#],
    r#"<p xml:id="Thmlemma1.p1.1"><text font="italic" xml:id="Thmlemma1.p1.1.1">A claim.</text></p>"#,
  );
}

/// beyond_original/fancyhdr_copy_reads_the_fields: a paper's fancyhdr.sty copy read raw over the fancyhdr binding (an
/// ICML style's `\RequirePackage{icml/fancyhdr}`, 2606.30940, 2602.04163, 2602.02841, 2602.00576, 2603.09787) finds the
/// head and foot fields fancyhdr's load leaves empty. Red: 12 Errors (`\f@nch@olh` … `\f@nch@erf` undefined).
#[test]
fn fancyhdr_copy_finds_the_fields() {
  let (_, xml) = convert(
    repro!("beyond_original", "fancyhdr_copy_reads_the_fields.tex"),
    &[
      ("icml.sty", repro!("beyond_original", "icml.sty")),
      (
        "sub/fancyhdr.sty",
        repro!("beyond_original", "sub/fancyhdr.sty"),
      ),
    ],
    (0, 1),
  );
  first_paragraph(&xml, "FANCY-COPY-MACRO");
}

/// The paragraph whose text starts with `text`, a `<picture>` (a tikzcd diagram or a pgfplots axis) drawn after it.
#[track_caller]
fn picture_after(xml: &str, text: &str) {
  assert!(
    xml.contains(&format!("<p xml:id=\"p1.1\">{text}<picture ")),
    "{xml}"
  );
}

/// provides_route/wrapper_loads_the_file_it_declares: a shipped wrapper that declares itself a file a raw-reading
/// binding stands for and loads it (`\ProvidesPackage{tikz-cd}\RequirePackage{tikz-cd}`, the pgfplots one that
/// `\pgfplotsset`s) is not that file's copy (`content::loads_file`): the binding reads TeX Live's file, the wrapper after
/// it. Red: 4 Errors each (`\usetikzlibrary`, `{tikzcd}`, `\arrow`; `{axis}`), the diagram and the plot lost: the wrapper
/// read as the file, its `\RequirePackage` hit the binding still loading. Two Warnings, LaTeX's "You have requested
/// package …" (KNOWN_PERL_ERRORS #1307). Reviewer's REGFIXQ probes A and C.
#[test]
fn wrapper_is_read_after_the_file_it_loads() {
  let (log, xml) = convert(
    repro!("provides_route", "wrapper_loads_the_file_it_declares.tex"),
    &[
      ("mytikzcd.sty", repro!("provides_route", "mytikzcd.sty")),
      ("mypgfplots.sty", repro!("provides_route", "mypgfplots.sty")),
    ],
    (0, 2),
  );
  for (copy, binding) in [("mytikzcd", "tikz-cd"), ("mypgfplots", "pgfplots")] {
    assert!(
      log.contains(&format!(
        "Info:provide:{copy}.sty The paper's {copy}.sty is read after the {binding}.sty binding, as its stand-in's copy"
      )),
      "{log}"
    );
  }
  picture_after(&xml, "MYTCD");
  assert!(xml.contains("class=\"ltx_tikzmatrix\""), "{xml}");
  for tick in ["0", "0.2", "0.4", "0.6", "0.8", "1"] {
    assert!(xml.contains(&format!("text=\"{tick}\"")), "{tick}: {xml}");
  }
}

/// provides_route/misdeclared_file_is_not_the_file: an author's file declaring `\ProvidesPackage{tikz-cd}` that defines
/// none of tikz-cd.sty's names (`content::reads_as_the_file`) is not read as it; the document's later
/// `\usepackage{tikz-cd}` finds tikz-cd loaded, as TeX reads it. Red: 4 Errors, the diagram lost: the file read as
/// tikz-cd.sty and the binding marked loaded with it. Reviewer's REGFIXQ probe B. Distinct: a file that loads nothing.
#[test]
fn misdeclared_file_is_not_read_as_the_file() {
  let (log, xml) = convert(
    repro!("provides_route", "misdeclared_file_is_not_the_file.tex"),
    &[(
      "mytcdmacros.sty",
      repro!("provides_route", "mytcdmacros.sty"),
    )],
    (0, 1),
  );
  assert!(!log.contains("is read as tikz-cd.sty"), "{log}");
  picture_after(&xml, "MYTCD");
  assert!(xml.contains("class=\"ltx_tikzmatrix\""), "{xml}");
}

/// provides_route/misdeclared_file_shares_a_load: an author's `\ProvidesPackage{tikz-cd}` file that loads tikz, as
/// tikz-cd.sty does, and defines `\mytcd` is not read as tikz-cd.sty: a shared load is no evidence of a copy, and
/// tikz-cd.sty's names are its library's (`\usetikzlibrary{cd}`), none of them the file's (`content::reads_as_the_file`).
/// Red: 3 Errors, no picture. Reviewer's REGFIXQ2 probe p1. Distinct: the real file defines nothing itself.
#[test]
fn misdeclared_file_sharing_a_load_is_not_the_file() {
  let (log, xml) = convert(
    repro!("provides_route", "misdeclared_file_shares_a_load.tex"),
    &[("mytkmacros.sty", repro!("provides_route", "mytkmacros.sty"))],
    (0, 1),
  );
  assert!(!log.contains("is read as tikz-cd.sty"), "{log}");
  picture_after(&xml, "MYTCD");
  assert!(xml.contains("class=\"ltx_tikzmatrix\""), "{xml}");
}

/// provides_route/misdeclared_pgfplots_shares_a_load: an author's `\ProvidesPackage{pgfplots}` file that loads
/// graphicx, as pgfplots.sty does, defines none of pgfplots.sty's names and is not read as it. Red: 3 Errors, the plot
/// lost. Reviewer's REGFIXQ2 probe p3. Distinct: the real file defines names of its own (`\pgfplotsversion`).
#[test]
fn misdeclared_pgfplots_sharing_a_load_is_not_the_file() {
  let (log, xml) = convert(
    repro!("provides_route", "misdeclared_pgfplots_shares_a_load.tex"),
    &[("mygxplots.sty", repro!("provides_route", "mygxplots.sty"))],
    (0, 1),
  );
  assert!(!log.contains("is read as pgfplots.sty"), "{log}");
  picture_after(&xml, "MYPGF");
  for tick in ["0", "0.2", "0.4", "0.6", "0.8", "1"] {
    assert!(xml.contains(&format!("text=\"{tick}\"")), "{tick}: {xml}");
  }
}

/// provides_route/class_wrapper_loads_the_class: a class wrapper `\ProvidesClass{smfart}\LoadClass{smfart}` is not
/// smfart.cls's copy (`content::loads_file`'s `\LoadClass`): the binding reads TeX Live's class. Red: 0 Errors, but
/// the section unnumbered and the equation's `ltx_leqno` gone (pdflatex "1. Intro", "(1)" on the left). Reviewer's
/// REGFIXQ probe F. Distinct: the class load, whose repeat is silent.
#[test]
fn class_wrapper_is_read_after_the_class() {
  let (_, xml) = convert(
    repro!("provides_route", "class_wrapper_loads_the_class.tex"),
    &[("mysmf.cls", repro!("provides_route", "mysmf.cls"))],
    (0, 1),
  );
  assert!(
    xml.contains("<title><tag close=\" \"> 1</tag>Intro</title>"),
    "{xml}"
  );
  assert!(xml.contains("class=\"ltx_leqno\""), "{xml}");
}

/// provides_route/exact_tikzcd_copy_is_read_as_the_file: an exact renamed tikz-cd.sty (`\RequirePackage{tikz}`,
/// `\usetikzlibrary{cd}`, no name of its own) is read as tikz-cd.sty: neither file defines a name, so both are compared
/// by the names of the library they read (`content::names_read_as`'s one-level extension), as for 1201.4414's and
/// 1512.05388's copies. Distinct: the only guard on that extension (REGFIXQ3 review S2).
#[test]
fn exact_tikzcd_copy_is_read_as_the_file() {
  let (log, xml) = convert(
    repro!(
      "provides_route",
      "exact_tikzcd_copy_is_read_as_the_file.tex"
    ),
    &[(
      "tikzcd_exact.sty",
      repro!("provides_route", "tikzcd_exact.sty"),
    )],
    (0, 1),
  );
  assert!(
    log.contains("The paper's tikzcd_exact.sty is read as tikz-cd.sty"),
    "{log}"
  );
  picture_after(&xml, "COPY");
  assert!(xml.contains("class=\"ltx_tikzmatrix\""), "{xml}");
}
