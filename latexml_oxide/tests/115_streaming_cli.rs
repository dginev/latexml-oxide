//! The `--streaming` CLI surface and the Fatal contract under streaming.
//!
//! Drives the real binary (the in-process `Converter` cannot see CLI
//! activation), pinning:
//!   * `--streaming` engages the fragmented pipeline and the output is
//!     byte-identical to a plain run of the same source;
//!   * an over-budget streaming run honors the Fatal contract (user policy
//!     2026-07-28): a `Fatal:` line names the breach, the run's verdict stays
//!     failed (exit 1, "Conversion failed"), and the partial document that
//!     was salvaged is WELL-FORMED XML — never a truncated fragment.

use std::{path::Path, process::Command};

fn run(args: &[&str], dir: &Path) -> (String, i32, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let output = Command::new(bin)
    .args(args)
    .current_dir(dir)
    .output()
    .expect("spawn latexml_oxide");
  (
    String::from_utf8_lossy(&output.stderr).into_owned(),
    output.status.code().expect("not killed by a signal"),
    output.status.to_string(),
  )
}

#[test]
fn streaming_flag_is_byte_identical_and_fatal_contract_holds() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let fixture = std::fs::canonicalize("tests/streaming/streaming_gate.tex").expect("fixture");
  let src = workdir.path().join("gate.tex");
  std::fs::copy(&fixture, &src).expect("copy fixture");

  // 1. Plain vs --streaming: byte-identical XML.
  let (log_a, code_a, _) = run(
    &[
      "gate.tex",
      "--dest",
      "eager.xml",
      "--nocomments",
      "--timeout",
      "0",
    ],
    workdir.path(),
  );
  assert_eq!(code_a, 0, "eager run failed:\n{log_a}");
  let (log_b, code_b, _) = run(
    &[
      "gate.tex",
      "--dest",
      "streamed.xml",
      "--nocomments",
      "--timeout",
      "0",
      "--streaming",
    ],
    workdir.path(),
  );
  assert_eq!(code_b, 0, "streaming run failed:\n{log_b}");
  let eager = std::fs::read_to_string(workdir.path().join("eager.xml")).unwrap();
  let streamed = std::fs::read_to_string(workdir.path().join("streamed.xml")).unwrap();
  assert_eq!(
    eager, streamed,
    "--streaming must be invisible in the output"
  );
  // The spill dir must not be left behind.
  assert!(
    !std::fs::read_dir(workdir.path()).unwrap().any(|e| e
      .unwrap()
      .file_name()
      .to_string_lossy()
      .starts_with(".latexml-spill")),
    "spill directory leaked into the destination"
  );

  // 2. Fatal contract: an over-budget streaming run ends gracefully with a
  // named Fatal, a failed verdict, and WELL-FORMED partial output. The corpus
  // is plain prose (as in 111_build_memory_guard) so digestion+build dominate
  // and the ceiling is hit mid-pipeline.
  let mut big = String::from("\\documentclass{article}\n\\begin{document}\n");
  for _ in 0..7_000 {
    for _ in 0..50 {
      big.push_str("alpha beta gamma delta epsilon ");
    }
    big.push_str("\n\n");
  }
  big.push_str("\\end{document}\n");
  std::fs::write(workdir.path().join("big.tex"), big).unwrap();
  let (log, code, status) = run(
    &[
      "big.tex",
      "--dest",
      "big.xml",
      "--nocomments",
      "--timeout",
      "0",
      "--streaming",
      "--max-memory",
      "2400",
    ],
    workdir.path(),
  );
  assert_ne!(
    code, 0,
    "an over-budget run must not report success ({status})"
  );
  assert!(
    log.contains("Fatal:Timeout:MemoryBudget"),
    "the breach must be named by a Fatal line:\n{log}"
  );
  assert!(
    !log.contains("Conversion complete"),
    "an over-budget run must not summarize as complete:\n{log}"
  );
  let xml = std::fs::read_to_string(workdir.path().join("big.xml")).unwrap_or_default();
  assert!(
    xml.len() > 500,
    "the salvaged partial document was not written"
  );
  // Well-formedness: the partial document must parse.
  libxml::parser::Parser::default()
    .parse_string(&xml)
    .expect("the salvaged partial document must be WELL-FORMED XML");
}
