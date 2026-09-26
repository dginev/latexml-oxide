//! The on-undefined LaTeX-kernel autoload and its two hard boundaries.
//!
//! `latexml_engine/src/latex_kernel.rs` loads `LaTeX.pool` when an *undefined*
//! control sequence turns out to be one the ambient kernel dump defines, so a
//! document may use a kernel command before `\documentclass` — as real LaTeX
//! allows, since `latex.ltx` IS the format. The happy path is guarded by the
//! `tests/structure/preclass_*.tex+.xml` pairs (`preclass_iffileexists_test`,
//! `preclass_kernel_cs_test`).
//!
//! This file guards the two places the mechanism must deliberately stay out of.
//! Both are binary-driven (fresh process) because they are process-level modes.

use std::{path::Path, process::Command};

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

/// The NODUMP twin of the witness idiom, as a plain-TeX body. Without a dump the
/// degraded `LoadFormat` branch raw-loads `latex.ltx` + expl3 on the first
/// trigger-list control sequence (`\documentclass`), which runs past the CLI's
/// 60 s default timeout at opt-level 0 — killed after the `\IfFileExists` error
/// was already printed, so the assertion passed without the conversion finishing.
/// Nothing here is on the trigger list, so the run stays plain TeX and finishes.
const PLAIN_PRECLASS_TEX: &str =
  "\\IfFileExists{ltxo-no-such-class.cls}{found-branch}{fallback-branch}\n\\bye\n";

/// Convert `tex` in a fresh process with the extra child environment `env`.
/// Returns (exit success, ANSI-stripped stderr, XML — empty if none was written).
fn convert(tex: &str, env: &[(&str, &str)]) -> (bool, String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("p.tex"), tex).expect("write p.tex");
  let mut cmd = Command::new(bin);
  cmd
    .args(["p.tex", "--dest", "p.xml", "--nocomments"])
    .current_dir(workdir.path());
  for (k, v) in env {
    cmd.env(k, v);
  }
  let output = cmd.output().expect("spawn latexml_oxide");
  // The logger TTY-gates colours, so a piped stderr is ANSI-free; strip anyway
  // (project signal-integrity rule — never let a parse miss hide a diagnostic).
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_default();
  (output.status.success(), stderr, xml)
}

/// Baseline for the negative test below: with a dump present the autoload
/// fires, the FALSE branch's class wins, and nothing is reported undefined.
#[test]
fn pre_documentclass_kernel_cs_selects_the_right_class() {
  let (_, stderr, xml) = convert(PRECLASS_TEX, &[]);
  assert!(
    !stderr.contains("Error:undefined:\\IfFileExists"),
    "\\IfFileExists before \\documentclass must autoload the LaTeX kernel:\n{stderr}"
  );
  assert!(
    xml.contains("<?latexml class=\"article\"?>"),
    "the \\IfFileExists FALSE branch must select `article`:\n{stderr}\n{xml}"
  );
}

/// `LoadFormat('latex')` has two mutually exclusive branches (CLAUDE.md durable
/// parity rule 1). On the degraded one there is no dump to test membership
/// against, so the autoload must not fire at all and behaviour must stay
/// exactly as it was before the mechanism existed: the Perl `TeX.pool` L33-56
/// trigger list is the only thing that loads the format, and `\IfFileExists` —
/// which is not on it — is reported undefined.
///
/// This asserts a *limitation on purpose*. If the no-dump branch ever gains a
/// membership oracle of its own, change this test deliberately; do not delete
/// it to make a run green.
///
/// The same plain-TeX body is converted with and without the dump, so the only
/// variable is the dump: with it the autoload fires and the FALSE branch is
/// typeset; without it `\IfFileExists` is undefined and all three arguments are
/// typeset as text. Both runs must finish (exit 0, XML written) — a conversion
/// killed mid-way must not pass for a checked limitation.
#[test]
fn nodump_leaves_pre_documentclass_kernel_cs_undefined() {
  let (ok, stderr, xml) = convert(PLAIN_PRECLASS_TEX, &[]);
  assert!(
    ok,
    "the plain-TeX twin with a dump exited non-zero:\n{stderr}"
  );
  assert!(
    !stderr.contains("Error:undefined:\\IfFileExists"),
    "with a dump, \\IfFileExists in plain TeX must autoload the LaTeX kernel:\n{stderr}"
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

  let (ok, stderr, xml) = convert(PLAIN_PRECLASS_TEX, &[("LATEXML_NODUMP", "1")]);
  assert!(
    ok,
    "the NODUMP plain-TeX conversion exited non-zero:\n{stderr}"
  );
  assert!(
    stderr.contains("Error:undefined:\\IfFileExists"),
    "with LATEXML_NODUMP the kernel autoload has no oracle and must stay inert:\n{stderr}"
  );
  // That one error, and nothing else.
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The undefined `\IfFileExists` leaves its three arguments as text.
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para><ERROR class="undefined">\IfFileExists</ERROR>
      <p>ltxo-no-such-class.clsfound-branchfallback-branch</p></para>"#,
  );
}
