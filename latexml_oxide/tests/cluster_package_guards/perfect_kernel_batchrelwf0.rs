//! RELWF0: a conversion on a machine without a TeX tree, as a release artifact runs there (tools/artifact_check.sh
//! `notex`; docs/parity/DUMP_DESIGN.md "Embedded dumps"). These spawn the binary with a scrubbed environment.

use std::process::Command;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// RED row NOTEX-L3BACKEND (the target: the no-TeX hello converts Warning-free). With no TeX tree, a
/// conversion on the format dump (the embedded one in an artifact, the on-disk one here: the same records) warns
/// `missing_file:l3backend-pdftex`: `\begin{document}` fires `\lx@sys@load@backend` (latex_constructs/sect02.rs,
/// RUST-ONLY), which inputs `l3backend-pdftex.def` from a tree there is none of. With a tree, and on the no-dump
/// branch, it converts with no Warning. This pins today's output; the fix flips the count to 0 and drops the line.
/// Skipped, visibly, where no latex dump is reachable or the binary still resolves TeX files.
/// Repro `tools/perfect_kernel/repros/loader/notex_dump_backend_load_warns.tex`.
#[test]
fn notex_dump_backend_load_warns() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let work = tempfile::tempdir().expect("create tempdir");
  std::fs::write(
    work.path().join("t.tex"),
    include_str!("../../../tools/perfect_kernel/repros/loader/notex_dump_backend_load_warns.tex"),
  )
  .expect("write t.tex");
  for dir in ["home", "tmp"] {
    std::fs::create_dir(work.path().join(dir)).expect("create a scratch dir");
  }
  let (home, tmp) = (work.path().join("home"), work.path().join("tmp"));
  let mut command = Command::new(bin);
  // No kpsewhich on PATH, and no texmf.cnf for a linked libkpathsea: tools/artifact_check.sh's view of the machine.
  command
    .env_clear()
    .env("PATH", "/nonexistent")
    .env("TEXMFCNF", "/nonexistent")
    .env("NO_COLOR", "1")
    .env("HOME", &home)
    .env("USERPROFILE", &home)
    .env("TMPDIR", &tmp)
    .env("TMP", &tmp)
    .env("TEMP", &tmp);
  for kept in ["SYSTEMROOT", "WINDIR", "LATEXML_DUMP_DIR"] {
    if let Some(value) = std::env::var_os(kept) {
      command.env(kept, value);
    }
  }
  let output = command
    .args(["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=180"])
    .current_dir(work.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(work.path().join("t.xml")).unwrap_or_default();
  if !stderr.lines().any(|l| {
    l.contains("Info:kpathsea:backend unavailable") || l.contains("resolves no host files")
  }) {
    eprintln!(
      "SKIP notex_dump_backend_load_warns: the binary still resolves TeX files here\n{stderr}"
    );
    return;
  }
  // The plain dump's load, then the latex one's.
  if stderr.matches("Info:dump_reader:loaded Loaded").count() < 2 {
    eprintln!(
      "SKIP notex_dump_backend_load_warns: no latex format dump reachable (the no-dump branch)\n{stderr}"
    );
    return;
  }
  assert!(
    output.status.success(),
    "exited {:?}:\n{stderr}",
    output.status
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // RED (NOTEX-L3BACKEND): the fix makes this 0, with no l3backend line.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains(
      "Warning:missing_file:l3backend-pdftex Can't find binding or file for 'l3backend-pdftex.def'."
    ),
    "{stderr}"
  );
  assert!(xml.contains("<p>Hello World!</p>"), "{xml}");
}
