//! Test requirements: Perl's `check_requirements` (`LaTeXML/lib/LaTeXML/Util/Test.pm:82-117`), shared by the
//! `tex_tests!` fixture harness and the guard tests. A test whose requirement the host TeX tree does not meet is
//! skipped VISIBLY: Perl's `Skip:` message on stderr and one `<test>\t<reason>` line in the skip log.

use std::{io::Write, path::PathBuf};

use latexml_core::util::pathname;

/// What a test needs from the host: Perl's `requires` value, in any of its three forms (a file, a list of files,
/// or `{packages => …, texlive_min => N, env => VAR}`), plus programs on `PATH` (a Rust addition: `jing`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Requirements<'a> {
  /// Files found through the engine's own kpathsea lookup, else a find from the working directory.
  pub packages:    &'a [&'a str],
  /// Programs found on `PATH`.
  pub programs:    &'a [&'a str],
  /// The minimal TeX Live year (0: any); the host year is `LATEXML_TEST_TEXLIVE` or the engine's detected year.
  pub texlive_min: u32,
  /// An environment variable that must be set (Perl's CI-only tests, `env => 'CI'`).
  pub env:         Option<&'a str>,
}

impl<'a> Requirements<'a> {
  /// The files `packages` (Perl's scalar and list forms).
  pub const fn packages(packages: &'a [&'a str]) -> Self {
    Requirements {
      packages,
      programs: &[],
      texlive_min: 0,
      env: None,
    }
  }
  /// Also require the programs `programs` on `PATH`.
  pub const fn programs(self, programs: &'a [&'a str]) -> Self { Requirements { programs, ..self } }
  /// Also require TeX Live `year` or later.
  pub const fn texlive_min(self, year: u32) -> Self { Requirements { texlive_min: year, ..self } }
  /// Also require the environment variable `var` to be set.
  pub const fn env(self, var: &'a str) -> Self { Requirements { env: Some(var), ..self } }

  /// Why `test` cannot run here, in Perl's words; `None` when every requirement is met. Perl's order: the packages,
  /// (the programs,) then `texlive_min`, then `env`.
  pub fn unmet(&self, test: &str) -> Option<String> {
    if let Some(file) = self.packages.iter().find(|file| !file_available(file)) {
      return Some(format!("Missing requirement {file} for {test}"));
    }
    if let Some(program) = self
      .programs
      .iter()
      .find(|program| !program_available(program))
    {
      return Some(format!("Missing program {program} for {test}"));
    }
    if self.texlive_min > 0 && texlive_version() < self.texlive_min {
      return Some(format!(
        "Minimal texlive {} requirement not met for {test}",
        self.texlive_min
      ));
    }
    if let Some(var) = self.env
      && std::env::var_os(var).is_none_or(|value| value.is_empty())
    {
      return Some(format!(
        "{test} is only checked in continuous integration. (set {var}=true)"
      ));
    }
    None
  }

  /// True when `test` may run; otherwise records the skip ([`skip`]) and returns false.
  pub fn check(&self, test: &str) -> bool {
    match self.unmet(test) {
      None => true,
      Some(reason) => {
        skip(test, &reason);
        false
      },
    }
  }

  /// [`Requirements::check`] for the running `#[test]`, named by its thread.
  pub fn met(&self) -> bool { self.check(&current_test_name()) }
}

/// The guard form: `if !requires(&["tcolorbox.sty"]) { return; }` skips the running test, visibly, when a file
/// is missing from the host TeX tree.
pub fn requires(files: &[&str]) -> bool { Requirements::packages(files).met() }

/// Perl's `pathname_kpsewhich($reqmt) || pathname_find($reqmt)`, through the lookup the conversion itself uses.
pub fn file_available(file: &str) -> bool {
  pathname::kpsewhich(&[file]).is_some()
    || pathname::find(file, pathname::PathnameFindOptions::default()).is_some()
}

/// An executable `program` (or `program.exe`) in a `PATH` directory.
pub fn program_available(program: &str) -> bool {
  let Some(path) = std::env::var_os("PATH") else {
    return false;
  };
  std::env::split_paths(&path).any(|dir| {
    [program.to_string(), format!("{program}.exe")]
      .iter()
      .any(|name| dir.join(name).is_file())
  })
}

/// Perl's `texlive_version`: `LATEXML_TEST_TEXLIVE`, else the host TeX Live year the engine resolves its dumps by,
/// else 0.
pub fn texlive_version() -> u32 {
  std::env::var("LATEXML_TEST_TEXLIVE")
    .ok()
    .and_then(|year| year.trim().parse().ok())
    .or_else(latexml_engine::dump_paths::detect_ambient_texlive_year)
    .unwrap_or(0)
}

/// Record a skipped test: Perl's `diag("Skip: …")` on stderr and one line in [`skip_log_path`]. The only writer of
/// the skip log.
pub fn skip(test: &str, reason: &str) {
  eprintln!("Skip: {reason}");
  let line = format!("{test}\t{reason}\n");
  let path = skip_log_path();
  // One `write_all` of one line on an O_APPEND file: nextest's concurrent test processes do not interleave.
  let written = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&path)
    .and_then(|mut log| log.write_all(line.as_bytes()));
  if let Err(err) = written {
    eprintln!("Skip log {}: {err}", path.display());
  }
}

/// `LATEXML_TEST_SKIP_LOG`, else `test-skips.log` in the cargo target directory of the running test binary
/// (`<target>/<profile>/deps/<binary>`), else in the temporary directory.
pub fn skip_log_path() -> PathBuf {
  #[cfg(test)]
  if let Some(path) = tests::LOG_OVERRIDE.with(|cell| cell.borrow().clone()) {
    return path;
  }
  if let Some(path) = std::env::var_os("LATEXML_TEST_SKIP_LOG") {
    return PathBuf::from(path);
  }
  std::env::current_exe()
    .ok()
    .and_then(|exe| {
      let deps = exe.parent()?;
      (deps.file_name()? == "deps").then(|| {
        deps
          .parent()?
          .parent()
          .map(|target| target.join("test-skips.log"))
      })?
    })
    .unwrap_or_else(|| std::env::temp_dir().join("latexml-test-skips.log"))
}

/// The running `#[test]`'s path (`module::test`): libtest names each test's thread after it.
pub fn current_test_name() -> String {
  std::thread::current()
    .name()
    .unwrap_or("<unnamed test>")
    .to_string()
}

/// The fixture harness's table: Perl's `requires => {name => …, '*' => …}`, a test's own key and `'*'` (every
/// test of the directory).
pub type RequirementTable = [(&'static str, Requirements<'static>)];

/// Perl's `check_requirements($test, 1, $$requires{'*'}, $$requires{$name})` for fixture `name` of `dirpath`.
pub fn check_table(dirpath: &str, name: &str, table: Option<&RequirementTable>) -> bool {
  let test = format!("{dirpath}/{name}");
  table
    .into_iter()
    .flatten()
    .filter(|(key, _)| *key == "*" || *key == name)
    .all(|(_, reqs)| reqs.check(&test))
}

#[cfg(test)]
mod tests {
  use std::cell::RefCell;

  use super::*;

  // Each row writes its skips to its own log, so the rows run in parallel.
  fn with_log(tag: &str, run: impl FnOnce() -> bool) -> (bool, String) {
    let path =
      std::env::temp_dir().join(format!("latexml-req-test-{}-{tag}.log", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let ran = LOG_OVERRIDE.with(|cell| {
      *cell.borrow_mut() = Some(path.clone());
      let ran = run();
      *cell.borrow_mut() = None;
      ran
    });
    let log = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);
    (ran, log)
  }

  thread_local! {
    pub(super) static LOG_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
  }

  #[test]
  fn requirement_forms() {
    const PRESENT: &str = "article.cls";
    const ABSENT: &str = "no-such-package-req1.sty";
    let (present, absent) = (PRESENT, ABSENT);
    if !file_available(present) {
      skip(
        "requirement_forms",
        "Missing requirement article.cls (no TeX tree)",
      );
      return;
    }
    let year = texlive_version();
    let rows: &[(&str, Requirements, Option<String>)] = &[
      ("file present", Requirements::packages(&[PRESENT]), None),
      (
        "file absent",
        Requirements::packages(&[ABSENT]),
        Some(format!("Missing requirement {absent} for t")),
      ),
      (
        "list, first absent wins",
        Requirements::packages(&[PRESENT, ABSENT, "also-absent.sty"]),
        Some(format!("Missing requirement {absent} for t")),
      ),
      (
        "texlive_min met",
        Requirements::packages(&[PRESENT]).texlive_min(year),
        None,
      ),
      (
        "texlive_min unmet",
        Requirements::packages(&[PRESENT]).texlive_min(year + 1),
        Some(format!(
          "Minimal texlive {} requirement not met for t",
          year + 1
        )),
      ),
      ("env set", Requirements::default().env("PATH"), None),
      (
        "program present",
        Requirements::default().programs(&["cargo"]),
        None,
      ),
      (
        "program absent",
        Requirements::packages(&[PRESENT]).programs(&["no-such-program-req1"]),
        Some("Missing program no-such-program-req1 for t".into()),
      ),
      (
        "env unset",
        Requirements::default().env("LATEXML_REQ1_UNSET_VAR"),
        Some(
          "t is only checked in continuous integration. (set LATEXML_REQ1_UNSET_VAR=true)".into(),
        ),
      ),
      (
        "packages before texlive_min",
        Requirements::packages(&[ABSENT]).texlive_min(year + 1),
        Some(format!("Missing requirement {absent} for t")),
      ),
    ];
    for (label, reqs, expected) in rows {
      assert_eq!(&reqs.unmet("t"), expected, "{label}");
    }
  }

  #[test]
  fn a_skip_is_logged_and_a_met_requirement_runs() {
    const ABSENT: &str = "no-such-package-req1.sty";
    let absent = ABSENT;
    let (ran, log) = with_log("skip", || {
      Requirements::packages(&[ABSENT]).check("dir/name")
    });
    assert!(!ran);
    assert_eq!(
      log,
      format!("dir/name\tMissing requirement {absent} for dir/name\n")
    );

    let (ran, log) = with_log("met", || Requirements::default().check("dir/name"));
    assert!(ran);
    assert_eq!(log, "");

    // The table: `'*'` and the test's own key both apply; another test's key does not.
    let table: &RequirementTable = &[
      ("*", Requirements::packages(&[])),
      (
        "other",
        Requirements::packages(&["no-such-package-req1.sty"]),
      ),
      (
        "name",
        Requirements::default().env("LATEXML_REQ1_UNSET_VAR"),
      ),
    ];
    let (ran, log) = with_log("table", || check_table("dir", "name", Some(table)));
    assert!(!ran);
    assert_eq!(
      log,
      "dir/name\tdir/name is only checked in continuous integration. (set LATEXML_REQ1_UNSET_VAR=true)\n"
    );
    let (ran, _) = with_log("table-other", || check_table("dir", "third", Some(table)));
    assert!(ran);
  }
}
