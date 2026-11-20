//! The on-undefined LaTeX-kernel autoload, and the plain TeX document it stays out of.
//!
//! `latexml_engine/src/latex_kernel.rs` loads `LaTeX.pool` when an *undefined*
//! control sequence turns out to be one the ambient kernel dump defines, so a
//! document may use a kernel command before `\documentclass` — as real LaTeX
//! allows, since `latex.ltx` IS the format. The happy path is guarded by the
//! `tests/structure/preclass_*.tex+.xml` pairs (`preclass_iffileexists_test`,
//! `preclass_kernel_cs_test`).
//!
//! This file guards a LaTeX-route document with a dump and the place the mechanism must deliberately stay out of, a
//! plain TeX document. Without a dump there is no catalogue, and the format loads on the first undefined control
//! sequence: that half shares the one NODUMP conversion of
//! `dump_gate_init::nodump_latex_branch_converts_healthily`, whose document opens with the same idiom.

use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The witness idiom (arXiv 2605.25877, 2606.06905): `\IfFileExists` is not on
/// Perl's `TeX.pool.ltxml` L33-56 trigger list, so without the autoload the
/// conditional collapses and the *rejected* branch's class is what gets picked.
const PRECLASS_TEX: &str = concat!(
  "\\IfFileExists{ltxo-no-such-class.cls}",
  "{\\documentclass{ltxo-no-such-class}}{\\documentclass{article}}\n",
  "\\begin{document}\n",
  "Selected the fallback class.\n",
  "\\end{document}\n"
);

/// The witness idiom with no `\documentclass` executed: nothing here is on the trigger list, and the skipped
/// `\documentclass` makes it a LaTeX document by arXiv AutoTeX's test (`converter::compile_route`), where the
/// autoload may act.
const LATEX_ROUTE_PRECLASS_TEX: &str = "\\iffalse\\documentclass{article}\\fi\n\\IfFileExists{ltxo-no-such-class.cls}\
                                        {found-branch}{fallback-branch}\n\\bye\n";

/// The same body as a plain TeX document: no class named anywhere.
const PLAIN_PRECLASS_TEX: &str =
  "\\IfFileExists{ltxo-no-such-class.cls}{found-branch}{fallback-branch}\n\\bye\n";

/// Convert `tex` in-process (`latexml::util::test::convert_with_status`). Returns (finished: XML written and a
/// non-fatal verdict, log, XML).
fn convert(tex: &str) -> (bool, String, String) {
  let (log, xml, _, status_code) = latexml::util::test::convert_with_status(tex, None);
  (!xml.is_empty() && status_code < 3, log, xml)
}

/// With a dump present the autoload fires, the FALSE branch's class wins, and nothing is reported undefined.
#[test]
fn pre_documentclass_kernel_cs_selects_the_right_class() {
  let (_, stderr, xml) = convert(PRECLASS_TEX);
  assert!(
    !stderr.contains("Error:undefined:\\IfFileExists"),
    "\\IfFileExists before \\documentclass must autoload the LaTeX kernel:\n{stderr}"
  );
  assert!(
    xml.contains("<?latexml class=\"article\"?>"),
    "the \\IfFileExists FALSE branch must select `article`:\n{stderr}\n{xml}"
  );
}

/// In a LaTeX document an `\IfFileExists` met before the format is loaded loads it, from the dump's catalogue: the
/// FALSE branch alone is typeset, and nothing is reported. The no-dump half (no catalogue: the format loads on the first
/// undefined control sequence) is the opening line of `dump_gate_init::nodump_latex_branch_converts_healthily`.
#[test]
fn latex_route_pre_documentclass_kernel_cs_autoloads() {
  let (ok, stderr, xml) = convert(LATEX_ROUTE_PRECLASS_TEX);
  assert!(
    ok,
    "the LaTeX-route twin with a dump exited non-zero:\n{stderr}"
  );
  assert!(
    !stderr.contains("Error:undefined:\\IfFileExists"),
    "with a dump, \\IfFileExists in a LaTeX document must autoload the LaTeX kernel:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The FALSE branch alone is typeset.
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>fallback-branch</p></para>"#,
  );
}

/// A plain TeX document (arXiv AutoTeX's test: no class named) is TeX's: a LaTeX
/// kernel command there is undefined, with or without a dump, as TeX and Perl
/// report it — the autoload stays out (batch 65f; loading the format on a plain
/// paper's `\verbatim` read to the end of its file, 1504.02513).
#[test]
fn plain_document_leaves_kernel_cs_undefined() {
  let (ok, stderr, xml) = convert(PLAIN_PRECLASS_TEX);
  assert!(ok, "the plain-TeX conversion exited non-zero:\n{stderr}");
  assert!(
    stderr.contains("Error:undefined:\\IfFileExists"),
    "in a plain TeX document \\IfFileExists must stay undefined:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para><ERROR class="undefined">\IfFileExists</ERROR>
      <p>ltxo-no-such-class.clsfound-branchfallback-branch</p></para>"#,
  );
}
