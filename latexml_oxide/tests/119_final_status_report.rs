//! The end-of-run verdict contract: a conversion's LAST report line names the
//! COMBINED core+post outcome, and a core Fatal stays a Fatal to the end.
//!
//! Driver defect this pins (131 MB witness UAT, 2026-08-01): a core
//! memory-budget Fatal produced a truncated "cheap partial", post processed it
//! Perl-faithfully — and then the run *ended* on `Info:writer:wrote …` with a
//! silent `exit(1)`. The core's own `Conversion failed:` line sat hundreds of
//! per-page post lines up-scroll, so a fatally-truncated site masqueraded as
//! a successful conversion unless the user checked `$?`.
//!
//! Drives the real binary: the verdict line and the exit code are CLI
//! behavior, invisible to the in-process `Converter` API.

use std::{path::Path, process::Command};

fn run(args: &[&str], dir: &Path) -> (String, i32) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let output = Command::new(bin)
    .args(args)
    .current_dir(dir)
    .output()
    .expect("spawn latexml_oxide");
  (
    String::from_utf8_lossy(&output.stderr).into_owned(),
    output.status.code().expect("not killed by a signal"),
  )
}

/// 101+ DISTINCT undefined macros trip the Perl-parity MAX_ERRORS=100 cap
/// into a deterministic `Fatal:TooManyErrors` (duplicates dedupe, so each
/// name must differ).
fn fatal_fixture(dir: &Path) -> String {
  let body: String = (0..150)
    .map(|i| {
      format!(
        "\\zzundefined{}{} ",
        char::from(b'a' + (i / 26) as u8),
        char::from(b'a' + (i % 26) as u8)
      )
    })
    .collect();
  let tex = format!("\\documentclass{{article}}\\begin{{document}}{body}\\end{{document}}");
  let path = dir.join("toomany.tex");
  std::fs::write(&path, tex).expect("fixture written");
  path.to_string_lossy().into_owned()
}

/// The last line that carries a conversion verdict, and the last report-ish
/// line of the whole run (ignoring blank lines).
fn last_lines(stderr: &str) -> (Option<String>, String) {
  let lines: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
  let last_verdict = lines
    .iter()
    .rev()
    .find(|l| l.contains("Conversion complete") || l.contains("Conversion failed"))
    .map(|l| l.to_string());
  (last_verdict, lines.last().unwrap_or(&"").to_string())
}

#[test]
fn core_fatal_is_the_final_verdict_of_a_post_run() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = fatal_fixture(workdir.path());
  let (stderr, code) = run(
    &[&tex, "--dest=toomany.html", "--format=html5"],
    workdir.path(),
  );

  // The Fatal is raised…
  assert!(
    stderr.contains("Fatal:TooManyErrors"),
    "expected the deterministic TooManyErrors fatal — stderr tail:\n{}",
    stderr.lines().rev().take(8).collect::<Vec<_>>().join("\n")
  );
  // …the process exit is non-zero…
  assert_eq!(code, 1, "a fatal conversion must exit 1");
  // …and the FINAL report of the run is the failure verdict: the last
  // verdict-bearing line says failed, and it is the LAST line of the run —
  // nothing (page writes, log notes) is allowed to bury it.
  let (last_verdict, very_last) = last_lines(&stderr);
  let verdict = last_verdict.expect("a conversion verdict line must be present");
  assert!(
    verdict.contains("Conversion failed"),
    "the final verdict must be a failure, got: {verdict}"
  );
  assert!(
    very_last.contains("Conversion failed"),
    "the failure verdict must be the LAST line of the run, got: {very_last}"
  );
}

/// The `--log` file's LAST line is the canonical combined
/// `Status:conversion:N` (Perl parity; ~/git/cortex derives final severity
/// from exactly this line and defaults to Fatal when it is absent), and the
/// archive `status` member carries the same canonical string — not the
/// human-readable core-only message it used to.
#[test]
fn log_and_archive_end_with_the_canonical_combined_status() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = fatal_fixture(workdir.path());

  // --log: last line is the combined status, here a fatal (3).
  let (_, code) = run(
    &[
      &tex,
      "--dest=toomany.html",
      "--format=html5",
      "--log=toomany.log",
    ],
    workdir.path(),
  );
  assert_eq!(code, 1);
  let log = std::fs::read_to_string(workdir.path().join("toomany.log")).expect("log written");
  let last = log
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert_eq!(
    last, "Status:conversion:3",
    "the --log file must END with the combined status line"
  );

  // whatsout=archive: the zip's `status` member is the canonical line, and
  // its packed log is status-terminated too.
  let (_, code) = run(
    &[&tex, "--dest=toomany.zip", "--format=html5"],
    workdir.path(),
  );
  assert_eq!(code, 1);
  let zip_file = std::fs::File::open(workdir.path().join("toomany.zip")).expect("zip written");
  let mut zip = zip::ZipArchive::new(zip_file).expect("readable zip");
  let mut status = String::new();
  std::io::Read::read_to_string(
    &mut zip.by_name("status").expect("status member"),
    &mut status,
  )
  .expect("status readable");
  assert_eq!(
    status, "Status:conversion:3",
    "the archive status member must be the canonical combined line"
  );
  let mut packed_log = String::new();
  std::io::Read::read_to_string(
    &mut zip.by_name("toomany.log").expect("log member"),
    &mut packed_log,
  )
  .expect("log readable");
  let last = packed_log
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert_eq!(
    last, "Status:conversion:3",
    "the packed log must END with the combined status line"
  );
}

#[test]
fn clean_post_run_ends_with_a_complete_verdict() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = workdir.path().join("ok.tex");
  std::fs::write(
    &tex,
    "\\documentclass{article}\\begin{document}Fine text $x^2$.\\end{document}",
  )
  .expect("fixture written");
  let (stderr, code) = run(
    &[&tex.to_string_lossy(), "--dest=ok.html", "--format=html5"],
    workdir.path(),
  );
  assert_eq!(code, 0, "clean run exits 0 — stderr:\n{stderr}");
  let (_, very_last) = last_lines(&stderr);
  assert!(
    very_last.contains("Conversion complete"),
    "a post run must END on its combined verdict, got: {very_last}"
  );
}

/// The lossless-tally contract at the binary level (user directive
/// 2026-08-02): `mouth.rs`'s deterministic "HTTP input not supported"
/// warning — a raw `log::warn!` before the single-vehicle migration, an
/// `emit_warn` after it — must register in the final verdict's count, not
/// just print. Defect this pins: the 131 MB
/// witness logged 12,105 `Warning:` lines (12,103 of them the math parser's
/// raw `log_math_warn!`) while the verdict said "2 warnings".
#[test]
fn raw_log_warnings_register_in_the_final_verdict() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = workdir.path().join("rawwarn.tex");
  std::fs::write(
    &tex,
    "\\documentclass{article}\\begin{document}\n\
     \\input{http://example.com/nonexistent.tex}\nText.\n\\end{document}\n",
  )
  .expect("fixture written");
  let (stderr, code) = run(
    &[
      &tex.to_string_lossy(),
      "--dest=rawwarn.html",
      "--format=html5",
    ],
    workdir.path(),
  );
  assert_eq!(
    code, 0,
    "warnings alone do not fail a run — stderr:\n{stderr}"
  );
  assert!(
    stderr.contains("Warning:unsupported:http_input HTTP input not supported"),
    "the raw-log warning line must print"
  );
  let (last_verdict, _) = last_lines(&stderr);
  let verdict = last_verdict.expect("verdict line present");
  assert!(
    verdict.contains("Conversion complete: 1 warning"),
    "the raw-log warning must be COUNTED in the verdict, got: {verdict}"
  );
}

/// Error and Fatal messages are the success-rate signal — they must reach the
/// log AND stderr at ANY verbosity (user directive 2026-08-02). The quietest CLI
/// mapping sets the STDERR level to `Warn`, and the logger always echoes
/// `Error`/`Fatal` records to stderr regardless of the console level
/// (`logger.rs`: `level <= Error || stderr_admits(level)`); this pins that
/// contract — if someone dropped the always-emit-errors clause, the Fatal line or
/// the verdict would vanish and this test names it. (`-q` is a plain bool flag —
/// quietest mode.)
#[test]
fn quiet_mode_still_reports_error_and_fatal() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = fatal_fixture(workdir.path());
  let (stderr, code) = run(
    &[&tex, "--dest=toomany.html", "--format=html5", "-q"],
    workdir.path(),
  );
  assert_eq!(code, 1, "a fatal conversion must exit 1 even at -q");
  assert!(
    stderr.contains("Fatal:TooManyErrors"),
    "the Fatal line must print at any verbosity — stderr tail:\n{}",
    stderr.lines().rev().take(6).collect::<Vec<_>>().join("\n")
  );
  assert!(
    stderr.contains("Error:undefined:"),
    "Error lines must print at any verbosity"
  );
  let (_, very_last) = last_lines(&stderr);
  assert!(
    very_last.contains("Conversion failed"),
    "the failure verdict stays the last line at -q, got: {very_last}"
  );
}

/// Perl latexmlc parity (bin/latexmlc L103-120, reported on the rc5 witness
/// UAT): with `--log` unset a conversion ALWAYS writes
/// `<jobname>.latexml.log` in the working directory, ending with the
/// canonical combined status line — so a fatal that scrolled off stderr can
/// always be consulted on disk.
#[test]
fn default_latexml_log_is_written_like_perl() {
  let workdir = tempfile::tempdir().expect("tempdir");
  let tex = fatal_fixture(workdir.path());
  let (_, code) = run(
    &[&tex, "--dest=toomany.html", "--format=html5"],
    workdir.path(),
  );
  assert_eq!(code, 1);
  let log_path = workdir.path().join("toomany.latexml.log");
  let log = std::fs::read_to_string(&log_path)
    .unwrap_or_else(|e| panic!("default log {} must exist: {e}", log_path.display()));
  assert!(
    log.contains("Fatal:TooManyErrors"),
    "the fatal must be consultable in the default log"
  );
  let last = log
    .lines()
    .rev()
    .find(|l| !l.trim().is_empty())
    .unwrap_or("");
  assert_eq!(
    last, "Status:conversion:3",
    "default log ends with the status line"
  );
}
