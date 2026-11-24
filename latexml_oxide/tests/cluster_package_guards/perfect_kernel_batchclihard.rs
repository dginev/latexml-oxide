//! CLIHARD: the `latexml_oxide` front end's own messages. A warning the command line raises before the session is
//! logged as well as shown, in a `.tex` run and an XML input's alike; `-q` hides Info and Warnings on stderr but keeps
//! the Errors and the verdict (OD #1158), while the log keeps everything. These spawn the binary.

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
/// report reset in a `.tex` run, by post-processing in an XML input's, which has no session, and by the front end
/// into `--log` for an XML input not post-processed. Red (c53f1ad424a): 0 in the `.latexml.log` and in each zip's log;
/// red (3533a021fa5): 0 in the `--nopost` run's `out.log`.
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

  // An XML input not post-processed: no session and no post-processing to log it, the front end does into `--log`.
  let core = run_in(staged("sections.tex"), &["sections.tex", "--dest=core.xml"]);
  assert_eq!(core.code, Some(0), "{}", core.stderr);
  let r = run_in(core.dir, &[
    "--max-memory=0",
    "core.xml",
    "--whatsin=xml",
    "--format=xml",
    "--nopost",
    "--log=out.log",
    "--dest=o.xml",
  ]);
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  for (text, what) in [(r.stderr.clone(), "stderr"), (r.file("out.log"), "out.log")] {
    assert_eq!(
      (count(&text, MEMORY), count(&text, "Warning:")),
      (1, 1),
      "{what}:\n{text}"
    );
  }
  assert_eq!(
    r.file("out.log").lines().last(),
    Some("Status:conversion:1")
  );
}

/// `-q` and `-qq` hide Info and Warnings on stderr but keep the Error record (with its continuation lines) and the
/// final verdict (OD #1158; DG 2026-08-02, reaffirmed 2026-11-24: the success-rate signal is never silent; Perl's
/// `--quiet` prints nothing, Common/Error.pm `_printline`). The log keeps the Info, the Warning and the Error, and the
/// exit status and a zip's `status` are the default run's. Red (3533a021fa5): empty stderr. A run without
/// post-processing (to XML) prints the verdict too, once, where the core's own is a muted Info note (red 8739436de83:
/// the Error alone); an EPUB whose log name is taken still says "Log not written" (red 8739436de83: nothing). Repro
/// cli-options/undefined_macro.tex (an undefined macro, a missing `\ref` target).
#[test]
fn quiet_keeps_errors_and_the_verdict_on_stderr() {
  const ERROR: &str = "Error:undefined:\\undefinedmacro";
  const WARNING: &str = "Warning:expected:ids Missing Target for Label: LABEL:sec:nowhere";
  const VERDICT: &str =
    "Conversion complete: 1 warning; 1 error; 1 undefined macro[\\undefinedmacro]";
  // The lines that open something: a record or a note, not a record's indented continuation.
  let heads = |text: &str| -> Vec<String> {
    text
      .lines()
      .filter(|l| !l.is_empty() && !l.starts_with(char::is_whitespace))
      .map(String::from)
      .collect()
  };
  let quiet_stderr_with = |r: &Run, verdict: &str| {
    let heads = heads(&r.stderr);
    assert_eq!(
      heads.len(),
      2,
      "the Error and the verdict alone:\n{}",
      r.stderr
    );
    assert!(heads[0].starts_with(ERROR), "{}", r.stderr);
    assert_eq!(heads[1], verdict, "{}", r.stderr);
    assert!(
      r.stderr.contains("\tat undefined_macro.tex; line 12"),
      "the Error keeps its location line:\n{}",
      r.stderr
    );
  };
  let quiet_stderr = |r: &Run| quiet_stderr_with(r, VERDICT);

  let loud = run_in(staged("undefined_macro.tex"), &[
    "undefined_macro.tex",
    "--dest=u.html",
  ]);
  assert_eq!(loud.code, Some(0), "{}", loud.stderr);
  assert_eq!(
    (
      count(&loud.stderr, ERROR),
      count(&loud.stderr, WARNING),
      count(&loud.stderr, "Log written to u")
    ),
    (1, 1, 1),
    "{}",
    loud.stderr
  );
  assert!(count(&loud.stderr, "Info:") > 0, "{}", loud.stderr);
  assert_eq!(
    heads(&loud.stderr).last().map(String::as_str),
    Some(VERDICT)
  );

  let r = run_in(staged("undefined_macro.tex"), &[
    "-q",
    "undefined_macro.tex",
    "--dest=u.html",
  ]);
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  quiet_stderr(&r);
  let log = r.file("undefined_macro.latexml.log");
  assert_eq!(
    (
      count(&log, "Error:"),
      count(&log, ERROR),
      count(&log, "Warning:"),
      count(&log, WARNING)
    ),
    (1, 1, 1, 1),
    "{log}"
  );
  assert!(
    count(&log, "Info:") > 0,
    "the log keeps its Info floor:\n{log}"
  );
  assert_eq!(log.lines().last(), Some("Status:conversion:2"), "{log}");

  let r = run_in(staged("undefined_macro.tex"), &[
    "-qq",
    "undefined_macro.tex",
    "--dest=u.zip",
  ]);
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  quiet_stderr(&r);
  assert_eq!(r.member("u.zip", "status"), "Status:conversion:2");
  let log = r.member("u.zip", "undefined_macro.latexml.log");
  assert_eq!((count(&log, ERROR), count(&log, WARNING)), (1, 1), "{log}");

  // No post-processing: the missing `\ref` target is post's Warning, so the verdict counts the Error alone.
  const CORE_VERDICT: &str = "Conversion complete: 1 error; 1 undefined macro[\\undefinedmacro]";
  let loud = run_in(staged("undefined_macro.tex"), &[
    "undefined_macro.tex",
    "--dest=u.xml",
  ]);
  assert_eq!(
    (
      count(&loud.stderr, ERROR),
      count(&loud.stderr, CORE_VERDICT)
    ),
    (1, 1),
    "the core's verdict, once:\n{}",
    loud.stderr
  );
  for dest in [None, Some("--dest=u.xml")] {
    let mut args = vec!["-q", "undefined_macro.tex"];
    args.extend(dest);
    let r = run_in(staged("undefined_macro.tex"), &args);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    quiet_stderr_with(&r, CORE_VERDICT);
  }

  // A note about output that was not produced prints under `-q` too.
  let dir = staged("undefined_macro.tex");
  std::fs::write(dir.path().join("undefined_macro.latexml.log"), "kept").expect("write the log");
  let r = run_in(dir, &["-q", "undefined_macro.tex", "--dest=u.epub"]);
  assert_eq!(r.code, Some(0), "{}", r.stderr);
  let heads = heads(&r.stderr);
  assert_eq!(heads.len(), 3, "{}", r.stderr);
  assert!(heads[0].starts_with(ERROR), "{}", r.stderr);
  assert_eq!(heads[1..], [
    "Log not written: 'undefined_macro.latexml.log' exists and is kept",
    VERDICT
  ]);
  assert_eq!(r.file("undefined_macro.latexml.log"), "kept");
}

/// `-q` hides a format build's progress (`[ini_tex] …`, `[dump_writer] Wrote …`, "Format dump complete"), Info that
/// went to stderr unconditionally (red 8739436de83: nine lines). `--init` runs the build a first conversion with a
/// fresh `LATEXML_FORMAT_CACHE` runs (`format_dumps::build` calls `ini_tex::dump_format`); without `-q` it still prints.
#[test]
fn quiet_hides_the_format_build_progress() {
  let dir = tempfile::tempdir().expect("tempdir");
  let init = |quiet: bool, dest: &str| {
    let mut args = vec!["--init=plain.tex", dest];
    if quiet {
      args.insert(0, "-q");
    }
    let r = run_in(tempfile::tempdir().expect("tempdir"), &args);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    r.stderr
  };
  let loud_dest = dir.path().join("loud.dump.txt");
  let loud = init(false, &format!("--dest={}", loud_dest.display()));
  assert_eq!(
    count(&loud, "[ini_tex] Dumping format from plain.tex"),
    1,
    "{loud}"
  );
  let quiet_dest = dir.path().join("quiet.dump.txt");
  let quiet = init(true, &format!("--dest={}", quiet_dest.display()));
  assert_eq!(quiet, "", "-q prints nothing");
  let entries = |path: &Path| {
    std::fs::read_to_string(path)
      .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
      .lines()
      .count()
  };
  assert!(entries(&quiet_dest) > 100, "the quiet build wrote its dump");
  assert_eq!(entries(&quiet_dest), entries(&loud_dest));
}
