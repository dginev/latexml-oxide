//! Red/green guard for batch STRMWARN: the math parser's "warn once per distinct formula" set is scoped per conversion
//! (a thread-local reset by `latexml_math_parser::reset_conversion_notices`), not per `MathParser`, so a streamed
//! conversion, which builds a parser per segment, warns exactly as the eager one (OXIDIZED_DESIGN_DIVERGENCES #1132).
use std::process::Command;

use super::perfect_kernel_batch46::error_count;

/// `tex` converted from the binary, eager or streamed with every section its own spilled segment: (log, XML).
fn conversion(tex: &str, streaming: bool) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let mut command = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"));
  command.args([
    "t.tex",
    "--dest",
    "t.xml",
    "--nocomments",
    "--timeout=110",
    "--preload=ar5iv.sty",
  ]);
  if streaming {
    command
      .arg("--streaming=true")
      .env("LATEXML_SPILL_AT_MIB", "0")
      .env("LATEXML_SOFT_YIELD_MIN_BOXES", "1");
  }
  let output = command
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("read t.xml");
  (log, xml)
}

/// The same unparsable formula in four sections: one `Warning:unparsed_math` eager, and streamed across at least two
/// spilled segments one too (base: one per segment, 4), with byte-equal XML. Witnesses: the Nasser FAQ slices
/// k600/k1200 (101 Warnings eager, 171 streamed, same XML). Distinct path: the dedup across `MathParser` instances.
#[test]
fn a_streamed_formula_warns_once_per_conversion() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/streaming/streamed_unparsed_math_warns_once.tex"
  );
  let (eager_log, eager_xml) = conversion(tex, false);
  let (streamed_log, streamed_xml) = conversion(tex, true);
  let segments = streamed_log.lines().find_map(|line| {
    line
      .split("PASS 1 ")
      .nth(1)?
      .split(" yield(s), ")
      .nth(1)?
      .split(' ')
      .next()?
      .parse::<usize>()
      .ok()
  });
  assert!(
    segments.is_some_and(|segments| segments >= 2),
    "pass 1 spilled fewer than 2 segments: {streamed_log}"
  );
  let unparsed = |log: &str| log.matches("Warning:unparsed_math:").count();
  assert_eq!(error_count(&eager_log), 0, "{eager_log}");
  assert_eq!(error_count(&streamed_log), 0, "{streamed_log}");
  assert_eq!(unparsed(&eager_log), 1, "{eager_log}");
  assert_eq!(unparsed(&streamed_log), 1, "{streamed_log}");
  assert_eq!(streamed_xml, eager_xml);
}
