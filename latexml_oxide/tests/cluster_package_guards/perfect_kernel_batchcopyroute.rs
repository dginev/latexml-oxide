//! Red/green guards for batch COPYROUTE: a paper's file that declares itself a bound file is read where the binding
//! reads that file, its request for the file reads the real one, and it leaves the file unloaded unless it loaded it, so
//! a later request loads the binding afresh (`content::SubstitutedRead`, `content::RouteCopyRead`; OXIDIZED_DESIGN_
//! DIVERGENCES #1118); `\ProvidesPackage`/`\ProvidesClass` record the version under the requested name and warn as
//! LaTeX does (latex.ltx:18471-18486; KNOWN_PERL_ERRORS #1307).
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/copy_route/",
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

/// LaTeX's warning for a file that declares a name other than its request.
#[track_caller]
fn provides_warning(log: &str, kind: &str, request: &str, declared: &str) {
  let warning = format!(
    "Warning:latex:provides You have requested {kind} `{request}', but the {kind} provides `{declared}'"
  );
  assert!(log.contains(&warning), "{warning}\n{log}");
}

/// A tikzcd diagram drawn after the paragraph's text `text`.
#[track_caller]
fn diagram_after(xml: &str, text: &str) {
  assert!(xml.contains(&format!("<p xml:id=\"p1.1\">{text}")), "{xml}");
  assert!(xml.contains("<picture "), "{xml}");
  assert!(xml.contains("class=\"ltx_tikzmatrix\""), "{xml}");
}

/// copy_route/class_wrapper_with_options: `\ProvidesClass{smfart}\LoadClassWithOptions{smfart}` is read where
/// smfart_cls.rs reads its class, and its `\LoadClassWithOptions` reads TeX Live's smfart.cls raw. Distinct: a class
/// wrapper whose load no text test of `\LoadClass` sees (the advisor's counter-example); green on the base by its other
/// test, so a guard of the executed rule W for a class.
#[test]
fn class_wrapper_with_options_reads_the_class() {
  let (log, xml) = convert(
    repro!("class_wrapper_with_options.tex"),
    &[("mysmfw.cls", repro!("mysmfw.cls"))],
    (0, 1),
  );
  provides_warning(&log, "document class", "mysmfw", "smfart");
  assert!(
    log.contains(
      "smfart is requested while a copy is read in its place: its own file is read, not its binding"
    ),
    "{log}"
  );
  assert!(
    xml.contains("<title><tag close=\" \"> 1</tag>Intro</title>"),
    "{xml}"
  );
  assert!(xml.contains("class=\"ltx_leqno\""), "{xml}");
}

/// copy_route/conditional_wrapper: `\IfFileExists{tikz-cd.sty}{\RequirePackage{tikz-cd}}{}` in a file declaring
/// tikz-cd reads the real tikz-cd.sty. Distinct: the request made through a conditional, a package.
#[test]
fn conditional_wrapper_reads_the_file() {
  let (log, xml) = convert(
    repro!("conditional_wrapper.tex"),
    &[("mycondtcd.sty", repro!("mycondtcd.sty"))],
    (0, 1),
  );
  provides_warning(&log, "package", "mycondtcd", "tikz-cd");
  diagram_after(&xml, "MYTCD");
}

/// copy_route/copy_then_real_load: a renamed copy of tikz-cd.sty leaves tikz-cd unloaded, so `\@ifpackageloaded`
/// is false after it and the document's `\usepackage{tikz-cd}` loads it. Red: "BEFORE-YES", the binding marked loaded
/// by the copy's read. Distinct: a true copy, then the real load (rule M).
#[test]
fn copy_then_real_load_reads_the_file() {
  let (log, xml) = convert(
    repro!("copy_then_real_load.tex"),
    &[("mytcdcopy.sty", repro!("mytcdcopy.sty"))],
    (0, 1),
  );
  provides_warning(&log, "package", "mytcdcopy", "tikz-cd");
  diagram_after(&xml, "COPY BEFORE-NO AFTER-YES");
}

/// copy_route/small_file_declares_tikzcd: an author's one-name file declaring tikz-cd, then `\usepackage{tikz-cd}`.
/// Red: 4 Errors, no picture (row R COPYSMALL). Distinct: a file sharing one name with tikz-cd's library, read in the
/// binding's file's place, and the binding's own code after the read (tikz_cd_sty.rs's libraries) with no tikz loaded.
#[test]
fn small_file_declaring_tikzcd_keeps_the_diagram() {
  let (log, xml) = convert(
    repro!("small_file_declares_tikzcd.tex"),
    &[("myar.sty", repro!("myar.sty"))],
    (0, 1),
  );
  provides_warning(&log, "package", "myar", "tikz-cd");
  assert!(xml.contains(r#"tex="a\rightarrow b""#), "{xml}");
  diagram_after(&xml, "Arrow ");
}

/// copy_route/small_file_declares_tcolorbox: an author's one-name file declaring tcolorbox, then
/// `\usepackage{tcolorbox}`. Red: 1 Error, the box lost (row R COPYSMALL). Distinct: a binding with no code after its
/// read.
#[test]
fn small_file_declaring_tcolorbox_keeps_the_box() {
  let (log, xml) = convert(
    repro!("small_file_declares_tcolorbox.tex"),
    &[("mybox.sty", repro!("mybox.sty"))],
    (0, 1),
  );
  provides_warning(&log, "package", "mybox", "tcolorbox");
  assert!(
    xml.contains(
      r##"<p xml:id="p1.pic1.1.1"><text color="#000000" xml:id="p1.pic1.1.1.1">BOXED</text></p>"##
    ),
    "{xml}"
  );
}

/// copy_route/requested_name: `\ProvidesPackage{foonobinding}` in mywrap.sty and `\ProvidesClass{fooclass}` in
/// myart.cls record their versions under mywrap.sty and myart.cls, the declared names stay unloaded, and LaTeX's two
/// warnings are logged. Red: "FOO-YES", no warning, `\ver@mywrap.sty` the format's date. Distinct: the kernel event
/// alone, with no binding.
#[test]
fn provides_records_the_requested_name() {
  let (log, xml) = convert(
    repro!("requested_name.tex"),
    &[
      ("mywrap.sty", repro!("mywrap.sty")),
      ("myart.cls", repro!("myart.cls")),
    ],
    (0, 2),
  );
  provides_warning(&log, "document class", "myart", "fooclass");
  provides_warning(&log, "package", "mywrap", "foonobinding");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">WRAP FOO-NO WRAP-YES\nFC-NO MA-YES\n[2020/01/01 v1 wrap] [2020/01/01 v2 \
     art]</p></para>",
  );
}
