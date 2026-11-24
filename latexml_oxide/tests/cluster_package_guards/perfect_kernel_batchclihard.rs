//! CLIHARD: the `latexml_oxide` front end's own messages. A warning the command line raises before the session is
//! logged as well as shown, in a `.tex` run and an XML input's alike; `-q` prints nothing on stderr, as Perl's
//! latexmlc (Common/Error.pm `_printline`: `$VERBOSITY >= 0`), while the log keeps everything. These spawn the binary.

use std::{
  io::Read,
  path::{Path, PathBuf},
  process::Command,
};

const DIR: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tools/perfect_kernel/repros/cli-options"
);

const MEMORY: &str = "Warning:memory:unlimited --max-memory=0: memory limiting disabled entirely";
const SPLITAT: &str = "Warning:unexpected:splitat Value for --splitat, paragraph, doesn't match";

struct Run {
  dir:    tempfile::TempDir,
  code:   Option<i32>,
  stderr: String,
}

impl Run {
  fn path(&self, name: &str) -> PathBuf { self.dir.path().join(name) }
  fn file(&self, name: &str) -> String {
    std::fs::read_to_string(self.path(name)).unwrap_or_else(|e| panic!("read {name}: {e}"))
  }
  fn member(&self, zip: &str, name: &str) -> String {
    let file = std::fs::File::open(self.path(zip)).unwrap_or_else(|e| panic!("open {zip}: {e}"));
    let mut archive = zip::ZipArchive::new(file).expect("a zip");
    let mut text = String::new();
    archive
      .by_name(name)
      .unwrap_or_else(|e| panic!("{zip} has no {name}: {e}"))
      .read_to_string(&mut text)
      .expect("read the member");
    text
  }
}

/// `args` in `dir`, holding a copy of the repro `tex` (and whatever an earlier run left there).
fn run_in(dir: tempfile::TempDir, args: &[&str]) -> Run {
  let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(dir.path())
    .output()
    .expect("spawn latexml_oxide");
  Run {
    dir,
    code: out.status.code(),
    stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
  }
}

fn staged(tex: &str) -> tempfile::TempDir {
  let dir = tempfile::tempdir().expect("tempdir");
  std::fs::copy(Path::new(DIR).join(tex), dir.path().join(tex)).expect("stage the repro");
  dir
}

fn count(text: &str, head: &str) -> usize { text.lines().filter(|l| l.starts_with(head)).count() }

/// Each early warning once in `text`, and no other.
fn assert_early_warnings(text: &str, what: &str) {
  assert_eq!(
    (
      count(text, MEMORY),
      count(text, SPLITAT),
      count(text, "Warning:")
    ),
    (1, 1, 2),
    "{what}:\n{text}"
  );
}

/// `--max-memory=0` and a `--splitat` outside Perl's list warn before the session starts. Their warnings reached
/// stderr and the status (`Status:conversion:1`) but no log: the report had been reset for the session and the log
/// was not yet bound. Held for the session now (`warn_at_session_start`), each is logged once: by the session's
/// report reset in a `.tex` run, by post-processing in an XML input's, which has no session. Red (c53f1ad424a): 0 in
/// the `.latexml.log` and in each zip's log.
#[test]
fn early_option_warnings_reach_the_log() {
  let early = ["--max-memory=0", "--split", "--splitat=paragraph"];
  let with =
    |args: &[&str]| -> Vec<String> { early.iter().chain(args).map(|a| a.to_string()).collect() };
  let args = with(&["sections.tex", "--dest=out/index.html"]);
  let r = run_in(
    staged("sections.tex"),
    &args.iter().map(String::as_str).collect::<Vec<_>>(),
  );
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  assert_early_warnings(&r.stderr, "stderr");
  let log = r.file("sections.latexml.log");
  assert_early_warnings(&log, "the .latexml.log");
  assert_eq!(log.lines().last(), Some("Status:conversion:1"), "{log}");

  let args = with(&["sections.tex", "--dest=out.zip"]);
  let r = run_in(
    staged("sections.tex"),
    &args.iter().map(String::as_str).collect::<Vec<_>>(),
  );
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  assert_early_warnings(&r.stderr, "stderr");
  assert_early_warnings(
    &r.member("out.zip", "sections.latexml.log"),
    "the zip's log",
  );
  assert_eq!(r.member("out.zip", "status"), "Status:conversion:1");

  // An XML input: the core document of a run without the options, post-processed with them.
  let core = run_in(staged("sections.tex"), &["sections.tex", "--dest=core.xml"]);
  assert_eq!(core.code, Some(0), "{}", core.stderr);
  let args = with(&["core.xml", "--whatsin=xml", "--dest=x.zip"]);
  let r = run_in(
    core.dir,
    &args.iter().map(String::as_str).collect::<Vec<_>>(),
  );
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  assert_early_warnings(&r.stderr, "stderr");
  assert_early_warnings(&r.member("x.zip", "core.latexml.log"), "the zip's log");
  assert_eq!(r.member("x.zip", "status"), "Status:conversion:1");
}

/// `-q` prints nothing on stderr, Errors included, as Perl's latexmlc `--quiet` (same host, 0.8.8: no byte);
/// `-qq` the same. The log keeps the Error and the status, and the exit status is the default run's. Red
/// (c53f1ad424a): the Error, "Log written to" and the verdict on stderr. Repro cli-options/undefined_macro.tex.
#[test]
fn quiet_prints_nothing_on_stderr() {
  const ERROR: &str = "Error:undefined:\\undefinedmacro";
  let loud = run_in(staged("undefined_macro.tex"), &[
    "undefined_macro.tex",
    "--dest=u.html",
  ]);
  assert_eq!(loud.code, Some(0), "{}", loud.stderr);
  assert_eq!(count(&loud.stderr, ERROR), 1, "{}", loud.stderr);

  let r = run_in(staged("undefined_macro.tex"), &[
    "-q",
    "undefined_macro.tex",
    "--dest=u.html",
  ]);
  assert_eq!((r.code, r.stderr.as_str()), (Some(0), ""));
  let log = r.file("undefined_macro.latexml.log");
  assert_eq!((count(&log, "Error:"), count(&log, ERROR)), (1, 1), "{log}");
  assert_eq!(log.lines().last(), Some("Status:conversion:2"), "{log}");

  let r = run_in(staged("undefined_macro.tex"), &[
    "-qq",
    "undefined_macro.tex",
    "--dest=u.zip",
  ]);
  assert_eq!((r.code, r.stderr.as_str()), (Some(0), ""));
  assert_eq!(r.member("u.zip", "status"), "Status:conversion:2");
  assert_eq!(
    count(&r.member("u.zip", "undefined_macro.latexml.log"), ERROR),
    1
  );
}
