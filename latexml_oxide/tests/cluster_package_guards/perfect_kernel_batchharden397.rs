//! Red/green guards for batch HARDEN397: an untrusted source cannot hang or crash a conversion without a log. A link
//! cycle in a source archive (`a -> .`) no longer walks forever in the search for the main file, and an expansion that
//! recurses through a balanced read (`\lx@ifundefined`'s argument expansion) ends in the recursion Fatal, not a stack
//! overflow. The cortex worker's own side is guarded in `bin/cortex_worker.rs` `archive_tests`.
use std::{path::Path, process::Command};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// Convert `source` (relative to `root`) with the CLI, the log written to `log`; its exit code and stderr.
fn convert_at(root: &Path, source: &str, dest: &str, log: &str) -> (Option<i32>, String) {
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      source,
      "--dest",
      dest,
      "--log",
      log,
      "--nocomments",
      "--timeout=110",
    ])
    .current_dir(root)
    .output()
    .expect("spawn latexml_oxide");
  (
    output.status.code(),
    String::from_utf8_lossy(&output.stderr).replace('\u{1b}', ""),
  )
}

/// Symbolic links to the source's own top (`a -> .`) stay inside it, so they are unpacked; the search for the main
/// file walks real directories only. Base: with four of them the walk branched 4^40 ways (each path ended only at the
/// kernel's 40-link limit), before the CLI's watchdog runs: the conversion hung past `--timeout`.
#[cfg(unix)]
#[test]
fn batchharden397_archive_link_cycle_converts() {
  let root = tempfile::tempdir().unwrap();
  let mut builder = tar::Builder::new(Vec::new());
  for (name, body) in [
    (
      "main.tex",
      "\\documentclass{article}\\begin{document}CYCLEBODY\\end{document}\n",
    ),
    ("other.tex", "% nothing\n"),
  ] {
    let mut header = tar::Header::new_gnu();
    header.set_size(body.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
      .append_data(&mut header, name, body.as_bytes())
      .unwrap();
  }
  for name in ["a", "b", "c", "d"] {
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    builder.append_link(&mut header, name, ".").unwrap();
  }
  std::fs::write(root.path().join("p.tar"), builder.into_inner().unwrap()).unwrap();
  let (code, log) = convert_at(root.path(), "p.tar", "p.xml", "p.log");
  let xml = std::fs::read_to_string(root.path().join("p.xml")).unwrap_or_default();
  assert_eq!(code, Some(0), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("<p>CYCLEBODY</p>"), "{xml}");
}

/// `\def\x{\@ifundefined{\x}{}{}}`: `\lx@ifundefined` expands its argument through a balanced read, which invoked
/// the next `\x` past the expansion-depth count. Base: "thread has overflowed its stack", SIGABRT, no log (CCJNL397,
/// `\@ifundefined{fmt@#1}` under mfirstuc's `\capitalisefmtwords`). Now one Fatal, logged, the CLI's Fatal exit; TeX
/// stops too ("TeX capacity exceeded, sorry [input stack size=10000]"). Repro
/// expansion-primitives/expansion_recursion_through_an_argument_expansion.
#[test]
fn batchharden397_expansion_recursion_through_an_argument_expansion_is_a_fatal() {
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/expansion_recursion_through_an_argument_expansion.tex"
    ),
  )
  .unwrap();
  let (code, stderr) = convert_at(root.path(), "t.tex", "t.xml", "t.log");
  let log = std::fs::read_to_string(root.path().join("t.log")).unwrap_or_default();
  let fatal = "Fatal:Timeout:Recursion Excessive expansion recursion (depth 12001 > 12000)";
  assert_eq!(code, Some(1), "{stderr}");
  assert_eq!(stderr.matches(fatal).count(), 1, "{stderr}");
  assert_eq!(log.matches(fatal).count(), 1, "{log}");
  // The Fatal is the log's only Error-or-Fatal line.
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
}
