//! Red/green guards for batch 65f, the kernel and mode clusters of run 338's Error→Fatal triage: a LaTeX kernel
//! command in a plain TeX document loaded the LaTeX format (`\verbatim`, graphics' begin-document hook under miniltx),
//! Perl's `\ProvidesFile`/`\ProvidesPackage` triggers answered generic-format probes with "LaTeX", the PDF/DVI choice
//! read a PDF the source never includes, and the gullet's cycle guard took a file's repeated text for an expansion
//! loop. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// (log, XML) of `tex` converted under ar5iv with `files` beside it.
fn convert_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"))
}

/// The XML of `tex` converted under ar5iv with `files` beside it, which must give no error and no warning.
fn convert_clean_files(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = convert_files(tex, files);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// An EPS file of a 10 bp square.
const SQUARE_EPS: &str = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\nnewpath 0 0 moveto 10 0 lineto 10 10 \
                          lineto 0 10 lineto closepath fill\n%%EOF\n";

/// 65f: in a plain TeX document a LaTeX kernel command is undefined, as TeX and Perl report it; the on-undefined
/// autoload loaded the format on `\verbatim`, whose LaTeX verbatim read to the end of the file (1504.02513, 1402.1807,
/// 1310.8121, 2405.04530: `Fatal:Mouth:EoF`). Repro loader/plain_kernel_command_stays_undefined.
#[test]
fn plain_kernel_command_stays_undefined() {
  let (log, xml) = convert_files(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/plain_kernel_command_stays_undefined.tex"
    ),
    &[],
  );
  assert_eq!(error_count(&log), 1, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(log.contains("Error:undefined:\\verbatim"), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert!(!log.contains("loading LaTeX.pool"), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"id1.1\">Some text.\n<ERROR class=\"undefined\" xml:id=\"id1.1.1\">\\verbatim</ERROR>x\n—endverbatim\nMore \
     text.</p>",
  );
}

/// 65f: graphics' epstopdf hook is registered as pdftex.def:681-700 registers it, only when `\AtEndOfPackage` exists;
/// under miniltx (plain TeX) it is not, and its `\@ifpackageloaded` no longer loads LaTeX mid-file (1710.04722,
/// 1802.06281: `Fatal:Timeout:Recursion`). Repro loader/plain_miniltx_graphicx_no_latex.
#[test]
fn plain_miniltx_graphicx_loads_no_latex() {
  let (log, xml) = convert_files(
    include_str!("../../../tools/perfect_kernel/repros/loader/plain_miniltx_graphicx_no_latex.tex"),
    &[],
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("loading LaTeX.pool"), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"id1.1\">Hello</p>");
}

/// 65f: in a plain TeX document `\ProvidesFile` and `\ProvidesPackage` are undefined to a probe, as in TeX
/// (OXIDIZED_DESIGN_DIVERGENCES #536; Perl's TeX.pool.ltxml:33-39 triggers answered "LaTeX": math0305287,
/// math0111222, the eplain papers on TL 2025); a use still loads the format, as Perl's trigger does. Repro
/// loader/plain_provides_probes_see_plain.
#[test]
fn plain_provides_probes_see_plain() {
  let (log, xml) = convert_files(
    include_str!("../../../tools/perfect_kernel/repros/loader/plain_provides_probes_see_plain.tex"),
    &[],
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(!log.contains("loading LaTeX.pool"), "{log}");
  assert_element(&xml, "p", &[], "<p xml:id=\"id1.1\">PLAIN PLAIN PLAIN</p>");
  // A use loads the format and runs, as Perl's trigger does.
  let (log, xml) = convert_files("\\ProvidesFile{x.tex}[2020/01/01 v1]\nHello.\n\\bye\n", &[]);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    log
      .matches("\\ProvidesFile is a LaTeX kernel command — loading LaTeX.pool")
      .count(),
    1,
    "{log}"
  );
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">Hello.</p>");
}

/// 65f: the figures a source includes choose its output mode, not a PDF in its tree that nothing includes (1012.0815:
/// EPS figures beside an IEEEtran manual PDF); a pstricks load is latex+dvips' cue (1002.1751); an included figure
/// latex+dvips cannot include is pdflatex's, whatever PostScript the tree holds (OXIDIZED_DESIGN_DIVERGENCES #537).
/// EPS figures stay DVI beside epstopdf conversions or an epstopdf load (1806.00093, 1804.02991; oracle: the Producer of
/// arXiv's PDF). Repros backend-persona/included_eps_beside_stray_pdf, backend-persona/pstricks_load_is_dvi,
/// backend-persona/epstopdf_conversion_beside_eps_is_dvi.
#[test]
fn included_figures_choose_the_output_mode() {
  let mode = |xml: &str, expected: &str| {
    assert_element(
      xml,
      "p",
      &["xml:id=\"p1.1\""],
      &format!("<p xml:id=\"p1.1\">Mode: {expected}.</p>"),
    );
  };
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/included_eps_beside_stray_pdf.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS), ("doc/manual.pdf", "")]);
  mode(&xml, "DVI");
  // The included figure as a PNG only: pdflatex, whatever EPS file the tree also holds.
  let xml = convert_clean_files(tex, &[("figs/a.png", ""), ("logo.eps", SQUARE_EPS)]);
  mode(&xml, "PDF");
  // A figure the tree holds in both forms decides nothing: the tree's files do (a PDF among them).
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS), ("figs/a.pdf", "")]);
  mode(&xml, "PDF");
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/backend-persona/pstricks_load_is_dvi.tex");
  let xml = convert_clean_files(tex, &[("figs/stale.pdf", "")]);
  mode(&xml, "DVI");
  // epstopdf's conversion beside an EPS figure, or an epstopdf load, is no PDF cue: arXiv compiled 1806.00093 and
  // 1806.00284 (conversions shipped), and 1804.02991 (an epstopdf-loading EPS-only paper), with latex+dvips — their
  // PDFs' Producer is "dvips + GPL Ghostscript" (1806.00284: "GPL Ghostscript", Creator dvips(k)).
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/epstopdf_conversion_beside_eps_is_dvi.tex"
  );
  let xml = convert_clean_files(tex, &[
    ("fig.eps", SQUARE_EPS),
    ("fig-eps-converted-to.pdf", ""),
  ]);
  mode(&xml, "DVI");
  let xml = convert_clean_files(
    &tex.replace("\\usepackage{graphicx}", "\\usepackage{graphicx,epstopdf}"),
    &[("fig.eps", SQUARE_EPS)],
  );
  mode(&xml, "DVI");
}

/// 65f: a file level opened past TeX's `max_in_open` (15, texmf.cnf:868) is pdftex's "TeX capacity exceeded, sorry
/// [text input levels=15]", a Fatal; a macro that `\input`s the file it is read from nested files without end, until
/// the process watchdog ended the run at `--timeout`. Repro
/// expansion-primitives/input_self_recursion_text_input_levels.
#[test]
fn self_input_stops_at_the_text_input_level_limit() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/input_self_recursion_text_input_levels.tex"
  );
  let (log, _xml) = convert_files(tex, &[(
    "loopf.tex",
    include_str!("../../../tools/perfect_kernel/repros/expansion-primitives/loopf.tex"),
  )]);
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Fatal:Mouth:Recursion  TeX capacity exceeded, sorry [text input levels=15]"),
    "{log}"
  );
  assert!(
    log.len() < 100_000,
    "the log must stay bounded: {} bytes",
    log.len()
  );
}

/// 65f: the gullet's cycle guard fingerprints a token read from the file with its source position, so a file's
/// repeated text, read as one argument past the guard's activation floor, is no loop; it was "a window of 8 token(s)
/// repeated 100+ times" (1009.1698's gnuplot label file, `Fatal:Timeout:Recursion`). A real expansion loop past the
/// floor still trips it; a `\scantokens` self-recursion stops at TeX's input-level limit. Repro
/// expansion-primitives/cycle_guard_repeated_file_input.
#[test]
fn repeated_file_text_is_no_expansion_loop() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/cycle_guard_repeated_file_input.tex"
  );
  let xml = convert_clean_files(tex, &[]);
  assert_element(&xml, "p", &[], "<p xml:id=\"p1.1\">done</p>");
  // The same warm-up, then an expansion loop: `\x` expands to `\y\x`, `\y` to nothing.
  let looping = tex.replace(
    "\\makeatletter\\def\\body{}%",
    "\\def\\y{}\\def\\x{\\y\\x}\\x",
  );
  let (log, _xml) = convert_files(&looping, &[]);
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains(
      "Fatal:Timeout:Recursion  Infinite expansion loop: a window of 2 token(s) repeated 100+ times"
    ),
    "{log}"
  );
  // A `\scantokens` self-recursion re-reads the same text from a new pseudo-file each period: TeX's input-level
  // limit stops it (etex: "TeX capacity exceeded, sorry [text input levels=15]").
  let looping = tex.replace(
    "\\makeatletter\\def\\body{}%",
    "\\def\\x{\\scantokens{\\x}}\\x",
  );
  let (log, _xml) = convert_files(&looping, &[]);
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.contains("Fatal:Mouth:Recursion  TeX capacity exceeded, sorry [text input levels=15]"),
    "{log}"
  );
}

/// 65f: `\endinput` reached through a macro on a line that ends in a comment ends the file after that line (tex.web
/// §362); with source comments off the Mouth read past the `%` into the next line, under the catcode the macro had
/// restored (atbegshi.sty:557 in plain TeX: `\AtBegShi` undefined). Repro
/// expansion-primitives/endinput_from_a_macro_before_a_comment.
#[test]
fn endinput_from_a_macro_ends_the_file_at_the_line() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/endinput_from_a_macro_before_a_comment.tex"
  );
  let (log, xml) = convert_files(tex, &[(
    "ei.sty",
    include_str!("../../../tools/perfect_kernel/repros/expansion-primitives/ei.sty"),
  )]);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &[],
    "<para xml:id=\"id1\"><p xml:id=\"id1.1\">BACK</p></para>",
  );
}
