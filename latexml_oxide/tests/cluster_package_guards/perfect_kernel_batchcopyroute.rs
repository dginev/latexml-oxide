//! Red/green guards for batch COPYROUTE: a paper's file that declares itself a bound file and reads as it is read where
//! the binding reads that file, and its request for the file reads the real one (`content::SubstitutedRead`,
//! OXIDIZED_DESIGN_DIVERGENCES #1118); a file that does not read as it stays its own; `\ProvidesPackage`/`\ProvidesClass`
//! record the version under the requested name and warn as LaTeX does (latex.ltx:18471-18486; KNOWN_PERL_ERRORS #1307).
//! Known red, no guard (rows R COPYTHENREAL, COPYSMALL): repros copy_then_real_load, small_file_declares_tikzcd,
//! small_file_declares_tcolorbox.
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

/// The Info of a paper's file read after the binding it declares itself, as the binding's stand-in's copy.
#[track_caller]
fn read_after(log: &str, file: &str, binding: &str) {
  let info = format!(
    "Info:provide:{file} The paper's {file} is read after the {binding} binding, as its stand-in's copy"
  );
  assert!(log.contains(&info), "{info}\n{log}");
}

/// copy_route/class_wrapper_with_options: `\ProvidesClass{smfart}\LoadClassWithOptions{smfart}`, defining none of
/// smfart.cls's names, is read after smfart_cls.rs's TeX Live class (`content::reads_as_the_file`), its load a repeat.
/// Distinct: a class wrapper whose load no text test of `\LoadClass` sees (the advisor's counter-example), kept out by
/// the content test alone.
#[test]
fn class_wrapper_with_options_reads_the_class() {
  let (log, xml) = convert(
    repro!("class_wrapper_with_options.tex"),
    &[("mysmfw.cls", repro!("mysmfw.cls"))],
    (0, 1),
  );
  provides_warning(&log, "document class", "mysmfw", "smfart");
  read_after(&log, "mysmfw.cls", "smfart.cls");
  assert!(
    xml.contains("<title><tag close=\" \"> 1</tag>Intro</title>"),
    "{xml}"
  );
  assert!(xml.contains("class=\"ltx_leqno\""), "{xml}");
}

/// copy_route/conditional_wrapper: `\IfFileExists{tikz-cd.sty}{\RequirePackage{tikz-cd}}{}` in a file declaring
/// tikz-cd that defines only `\mytcd` is read after the binding's TeX Live file. Distinct: the request made through a
/// conditional, a package.
#[test]
fn conditional_wrapper_reads_the_file() {
  let (log, xml) = convert(
    repro!("conditional_wrapper.tex"),
    &[("mycondtcd.sty", repro!("mycondtcd.sty"))],
    (0, 1),
  );
  provides_warning(&log, "package", "mycondtcd", "tikz-cd");
  read_after(&log, "mycondtcd.sty", "tikz-cd.sty");
  diagram_after(&xml, "MYTCD");
}

/// copy_route/wrapper_renews_a_command: an author's `\ProvidesPackage{csquotes}` wrapper whose
/// `\RequirePackageWithOptions{csquotes}` reads TeX Live's csquotes.sty and whose `\renewcommand{\mktextquote}` then
/// stands. Red on INT7b: 25 Errors, the wrapper read after the binding. Distinct: rule W on its own for a package the
/// wrapper renews a command of (the advisor's F3).
#[test]
fn wrapper_renewing_a_command_reads_the_file() {
  let (log, xml) = convert(
    repro!("wrapper_renews_a_command.tex"),
    &[("mycsq.sty", repro!("mycsq.sty"))],
    (0, 1),
  );
  provides_warning(&log, "package", "mycsq", "csquotes");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">CSQ-Y MY-Y Say <text class=\"ltx_inline-quote ltx_outerquote ltx_textquote\" \
     xml:id=\"p1.1.1\">\u{201c}[hello]\u{201d}</text> twice.</p>",
  );
}

/// copy_route/ims/class_declaring_imsart_keeps_its_package: a class declaring imsart that loads imsart, beside the
/// paper's imsart.sty and an author's myims.sty: imsart_cls.rs's read of imsart.sty reads the paper's imsart.sty, not
/// myims.sty, which the class never loads. Red on the COPYROUTE r0 branch: "PKG-N". Distinct: the binding's read of
/// its other type, where the copy's own file of that type is routed only when the copy loads it (`loads_file`).
#[test]
fn class_declaring_imsart_reads_the_papers_imsart() {
  let (log, xml) = convert(
    repro!("ims/class_declaring_imsart_keeps_its_package.tex"),
    &[
      ("myims.cls", repro!("ims/myims.cls")),
      ("myims.sty", repro!("ims/myims.sty")),
      ("imsart.sty", repro!("ims/imsart.sty")),
    ],
    (0, 1),
  );
  provides_warning(&log, "document class", "myims", "imsart");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">IMS-Y MY-Y PKG-Y MYMAC</p>",
  );
}

/// copy_route/misdeclared_csquotes, misdeclared_cleveref_then_real: an author's file declaring a raw-reading binding's
/// name that defines none of its names stays its own, the binding's code after its read never run over it, and a later
/// real request loads the binding once. Red on the COPYROUTE r0 branch: 24 Errors (csquotes), a Fatal timeout
/// (cleveref). Distinct: the content test that keeps a misdeclared file out of the binding's place.
#[test]
fn misdeclared_files_stay_their_own() {
  for (tex, file, copy, declared) in [
    (
      repro!("misdeclared_csquotes.tex"),
      "mycsquotes.sty",
      repro!("mycsquotes.sty"),
      "csquotes",
    ),
    (
      repro!("misdeclared_cleveref_then_real.tex"),
      "mycleveref.sty",
      repro!("mycleveref.sty"),
      "cleveref",
    ),
  ] {
    let (log, xml) = convert(tex, &[(file, copy)], (0, 1));
    assert!(!log.contains("Fatal:"), "{file}: {log}");
    provides_warning(&log, "package", file.trim_end_matches(".sty"), declared);
    assert_element(
      &xml,
      "p",
      &[r#"xml:id="p1.1""#],
      "<p xml:id=\"p1.1\">Text MYFOO.</p>",
    );
  }
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
