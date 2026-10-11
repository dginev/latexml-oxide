//! Output trees: finding the document directories under a root, loading one document's files, and the
//! parallel map the subcommands share.
//!
//! A document directory holds a conversion's XML and log under one of the names the harnesses use:
//! `out.xml` + `out.log` (`ab_audit run`, the payload A/B runners), or `<name>.xml` + `<name>.latexml.log`
//! (the manual sweeps). `meta.tsv` (`ab_audit run`), `verdict.tsv` (the manual sweeps) or `exit.txt` (a
//! runner's exit code) give the exit code, the wall time and the timeout; the key is the directory's path
//! under the root. A directory holding only a run record or a stderr/stdout capture is a document too.

use std::{
  fs,
  path::{Path, PathBuf},
  sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
  },
};

/// One conversion's run record (`meta.tsv`, or `exit.txt`).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RunMeta {
  pub rc:        Option<i32>,
  pub secs:      Option<f64>,
  pub timed_out: bool,
  /// `input-missing` when the paper's archive was not found.
  pub note:      String,
  pub stamp:     String,
  pub bin_hash:  String,
  /// A hash of the run settings (`runner.rs` `settings_hash`): a resume needs the same.
  pub settings:  String,
  /// The archive's `size:mtime` when it was converted: a replaced archive reruns.
  pub input:     String,
}

impl RunMeta {
  /// Parse `meta.tsv`: `key<TAB>value` lines.
  pub fn parse(text: &str) -> RunMeta {
    let mut meta = RunMeta::default();
    for line in text.lines() {
      let (k, v) = line.split_once('\t').unwrap_or((line, ""));
      match k {
        "rc" => meta.rc = v.parse().ok(),
        "secs" => meta.secs = v.parse().ok(),
        "timed_out" => meta.timed_out = v == "1",
        "note" => meta.note = v.to_string(),
        "stamp" => meta.stamp = v.to_string(),
        "bin_hash" => meta.bin_hash = v.to_string(),
        "settings" => meta.settings = v.to_string(),
        "input" => meta.input = v.to_string(),
        _ => {},
      }
    }
    // `timeout` exits 124 on its TERM and 137 when it had to KILL.
    meta.timed_out |= matches!(meta.rc, Some(124 | 137));
    meta
  }
  pub fn render(&self) -> String {
    format!(
      "stamp\t{}\nbin_hash\t{}\nsettings\t{}\ninput\t{}\nrc\t{}\nsecs\t{}\ntimed_out\t{}\nnote\t{}\n",
      self.stamp,
      self.bin_hash,
      self.settings,
      self.input,
      self.rc.map(|r| r.to_string()).unwrap_or_default(),
      self.secs.map(|s| format!("{s:.2}")).unwrap_or_default(),
      u8::from(self.timed_out),
      self.note
    )
  }
}

/// One document's files as read from disk.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DocInput {
  /// The directory exists in this tree.
  pub present: bool,
  pub xml:     Option<Vec<u8>>,
  pub log:     Option<String>,
  pub meta:    Option<RunMeta>,
  /// The captured output — stderr (`stderr.txt`, a sweep's `<name>.log`) then stdout (`stdout.txt`,
  /// `<name>.stdout`), every capture present — read when the conversion log is missing, has no
  /// `Status:conversion:` line, or the run timed out or was killed (exit 124/137): its `Fatal:` lines say
  /// why. A runner that sends stderr into its stdout capture leaves the watchdog's `Fatal:timeout` only in
  /// `stdout.txt` (diagnostic-honesty audit, 2026-10-11).
  pub stderr:  Option<String>,
}

/// The files a document directory may hold. A directory with any of them is a document, even with no
/// XML and no log: a manual-sweep timeout leaves only `<name>.log` (stderr), `<name>.stdout` and
/// `verdict.tsv` (lie-hasse, pgf-PeriodicTableManual, tcolorbox in s148), and such a document must be a
/// flagged row, not a missing one.
const MARKERS: &[&str] = &[
  "out.xml",
  "out.log",
  "meta.tsv",
  "verdict.tsv",
  "exit.txt",
  "stderr.txt",
  "stdout.txt",
];

/// A document directory's XML and conversion log, if any.
struct DocFiles {
  xml: Option<PathBuf>,
  log: Option<PathBuf>,
}

/// Whether `dir` is a document directory, and its XML and log.
fn doc_files(dir: &Path) -> Option<DocFiles> {
  let name = dir.file_name()?.to_string_lossy().into_owned();
  let first = |candidates: [String; 2]| {
    candidates
      .into_iter()
      .map(|c| dir.join(c))
      .find(|p| p.is_file())
  };
  let xml = first(["out.xml".into(), format!("{name}.xml")]);
  let log = first(["out.log".into(), format!("{name}.latexml.log")]);
  let named = [format!("{name}.log"), format!("{name}.stdout")];
  let marked =
    MARKERS.iter().any(|m| dir.join(m).is_file()) || named.iter().any(|m| dir.join(m).is_file());
  (xml.is_some() || log.is_some() || marked).then_some(DocFiles { xml, log })
}

/// The document directories under a root.
#[derive(Debug, Default, PartialEq)]
pub struct Discovery {
  /// Relative paths, sorted.
  pub keys:         Vec<String>,
  /// Directories that hold none of the document files and are leaves (no subdirectory) or at the depth
  /// limit: rows of their own (`unrecognised`), since a layout this tool does not know would otherwise
  /// vanish from the comparison.
  pub unrecognised: Vec<String>,
}

/// The document directories under `root`, searching at most `depth` levels down.
pub fn discover(root: &Path, depth: usize) -> Discovery {
  let mut found = Discovery::default();
  let mut queue = vec![(root.to_path_buf(), 0)];
  while let Some((dir, level)) = queue.pop() {
    let rel = || {
      dir
        .strip_prefix(root)
        .map(|r| r.to_string_lossy().into_owned())
        .unwrap_or_default()
    };
    if level > 0 && doc_files(&dir).is_some() {
      found.keys.push(rel());
      continue;
    }
    let subdirs: Vec<PathBuf> = fs::read_dir(&dir)
      .map(|entries| {
        entries
          .flatten()
          .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
          .map(|e| e.path())
          .collect()
      })
      .unwrap_or_default();
    if level > 0 && (subdirs.is_empty() || level >= depth) {
      // A leaf without document files, or a directory the depth limit cuts: either would vanish from the
      // comparison, so both are listed.
      found.unrecognised.push(rel());
      continue;
    }
    queue.extend(subdirs.into_iter().map(|d| (d, level + 1)));
  }
  found.keys.sort();
  found.unrecognised.sort();
  found
}

/// The run record of `verdict.tsv` (`tools/perfect_kernel/run_doc.sh`: bundle, name, status, exit, errors,
/// fatals, warnings, seconds) or `exit.txt` (an exit code).
fn sweep_meta(dir: &Path) -> Option<RunMeta> {
  let (rc, secs) = if let Ok(text) = fs::read_to_string(dir.join("verdict.tsv")) {
    let cells: Vec<&str> = text.lines().next().unwrap_or("").split('\t').collect();
    (
      cells.get(3).and_then(|v| v.parse().ok()),
      cells.last().and_then(|v| v.parse().ok()),
    )
  } else {
    let text = fs::read_to_string(dir.join("exit.txt")).ok()?;
    (
      text.split_whitespace().next().and_then(|v| v.parse().ok()),
      None,
    )
  };
  Some(RunMeta {
    rc,
    secs,
    timed_out: matches!(rc, Some(124 | 137)),
    ..RunMeta::default()
  })
}

/// Load one document's files; an unreadable file counts as missing.
pub fn load(root: &Path, key: &str) -> DocInput {
  let dir = root.join(key);
  if !dir.is_dir() {
    return DocInput::default();
  }
  // A directory that exists but holds none of the files is still present: a failed run.
  let files = doc_files(&dir).unwrap_or(DocFiles { xml: None, log: None });
  let meta = fs::read_to_string(dir.join("meta.tsv"))
    .ok()
    .map(|t| RunMeta::parse(&t))
    .or_else(|| sweep_meta(&dir));
  let read_text = |p: PathBuf| {
    fs::read(p)
      .ok()
      .map(|b| String::from_utf8_lossy(&b).into_owned())
  };
  let log = files.log.and_then(read_text);
  let cut_short = matches!(meta.as_ref().and_then(|m| m.rc), Some(124 | 137));
  let unfinished = log
    .as_deref()
    .is_none_or(|l| !l.lines().any(|line| line.contains("Status:conversion:")));
  let stderr = if unfinished || cut_short {
    let name = dir
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    let captures: Vec<String> = [
      "stderr.txt".to_string(),
      format!("{name}.log"),
      "stdout.txt".to_string(),
      format!("{name}.stdout"),
    ]
    .into_iter()
    .map(|f| dir.join(f))
    .filter(|p| p.is_file())
    .filter_map(read_text)
    .collect();
    (!captures.is_empty()).then(|| captures.concat())
  } else {
    None
  };
  DocInput {
    present: true,
    xml: files.xml.and_then(|p| fs::read(p).ok()),
    log,
    meta,
    stderr,
  }
}

/// A tree's `STAMP` file (written by `ab_audit run`): `key<TAB>value` lines, shown in the report header.
pub fn stamp_lines(root: &Path) -> Vec<String> {
  match fs::read_to_string(root.join("STAMP")) {
    Ok(text) => text.lines().map(|l| l.replace('\t', "=")).collect(),
    Err(_) => vec!["stamp=unknown (no STAMP file: not produced by `ab_audit run`)".to_string()],
  }
}

/// Map `f` over `items` on `jobs` threads, results in input order. A panic in `f` becomes `on_panic(item)`.
pub fn par_map<T: Sync, R: Send>(
  items: &[T],
  jobs: usize,
  f: impl Fn(&T) -> R + Sync,
  on_panic: impl Fn(&T) -> R + Sync,
  progress: impl Fn(usize) + Sync,
) -> Vec<R> {
  let next = AtomicUsize::new(0);
  let done = AtomicUsize::new(0);
  let results: Mutex<Vec<Option<R>>> = Mutex::new((0..items.len()).map(|_| None).collect());
  std::thread::scope(|scope| {
    for _ in 0..jobs.max(1) {
      scope.spawn(|| {
        loop {
          let i = next.fetch_add(1, Ordering::Relaxed);
          let Some(item) = items.get(i) else { break };
          let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(item)))
            .unwrap_or_else(|_| on_panic(item));
          results.lock().unwrap_or_else(|e| e.into_inner())[i] = Some(r);
          progress(done.fetch_add(1, Ordering::Relaxed) + 1);
        }
      });
    }
  });
  results
    .into_inner()
    .unwrap_or_else(|e| e.into_inner())
    .into_iter()
    .map(|r| r.expect("every item mapped"))
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn discovery_finds_both_layouts_and_load_reads_them() {
    let root = std::env::temp_dir().join(format!("ab_audit_tree_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("1001.0001")).unwrap();
    fs::write(root.join("1001.0001/out.xml"), "<a/>").unwrap();
    fs::write(root.join("1001.0001/out.log"), "Status:conversion:0\n").unwrap();
    fs::write(root.join("1001.0001/meta.tsv"), "rc\t124\nsecs\t300.00\n").unwrap();
    fs::create_dir_all(root.join("bundle/name")).unwrap();
    fs::write(
      root.join("bundle/name/name.latexml.log"),
      "Status:conversion:3\n",
    )
    .unwrap();
    assert_eq!(discover(&root, 3).keys, vec![
      "1001.0001".to_string(),
      "bundle/name".to_string()
    ]);
    let one = load(&root, "1001.0001");
    assert!(one.present && one.xml.is_some() && one.log.is_some());
    assert!(one.meta.as_ref().is_some_and(|m| m.timed_out), "{one:?}");
    let two = load(&root, "bundle/name");
    assert!(two.present && two.xml.is_none() && two.log.is_some());
    assert!(!load(&root, "nope").present);
    fs::remove_dir_all(&root).unwrap();
  }

  #[test]
  fn a_sweep_timeout_with_no_xml_and_no_log_is_a_document() {
    // lie-hasse in s148: `timeout 180` killed the run before any log or XML was written.
    let root = std::env::temp_dir().join(format!("ab_audit_timeout_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let doc = root.join("lie-hasse/lie-hasse");
    fs::create_dir_all(&doc).unwrap();
    fs::write(doc.join("lie-hasse.log"), "contour: Using real outline\n").unwrap();
    fs::write(doc.join("lie-hasse.stdout"), "").unwrap();
    fs::write(
      doc.join("verdict.tsv"),
      "lie-hasse\tlie-hasse\t124\t124\t1\t0\t5\t180.1\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("odd/leaf")).unwrap();
    fs::write(root.join("odd/leaf/something.dat"), "").unwrap();
    let found = discover(&root, 3);
    assert_eq!(found.keys, vec!["lie-hasse/lie-hasse".to_string()]);
    assert_eq!(found.unrecognised, vec!["odd/leaf".to_string()]);
    // A directory the depth limit cuts is listed, not dropped.
    assert_eq!(discover(&root, 1).unrecognised, vec![
      "lie-hasse".to_string(),
      "odd".to_string()
    ]);
    let input = load(&root, "lie-hasse/lie-hasse");
    assert!(
      input
        .stderr
        .as_deref()
        .is_some_and(|e| e.contains("contour")),
      "{input:?}"
    );
    assert!(
      input.present && input.xml.is_none() && input.log.is_none(),
      "{input:?}"
    );
    let meta = input.meta.expect("verdict.tsv read");
    assert_eq!(
      (meta.rc, meta.secs, meta.timed_out),
      (Some(124), Some(180.1), true)
    );
    fs::remove_dir_all(&root).unwrap();
  }

  #[test]
  fn a_fatal_only_in_the_stdout_capture_is_read() {
    // A runner that sends stderr into stdout.txt: the watchdog's `Fatal:timeout` is there and nowhere else,
    // and the log stops before its `Status:conversion:` line.
    let root = std::env::temp_dir().join(format!("ab_audit_stdout_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let doc = root.join("2101.00001");
    fs::create_dir_all(&doc).unwrap();
    fs::write(doc.join("out.log"), "(Loading article.cls)\n").unwrap();
    fs::write(
      doc.join("stdout.txt"),
      "Fatal:timeout:wallclock latexml-oxide: main-level wall-clock timeout after 240s\n",
    )
    .unwrap();
    let input = load(&root, "2101.00001");
    assert!(
      input
        .stderr
        .as_deref()
        .is_some_and(|e| e.contains("Fatal:timeout:wallclock")),
      "{input:?}"
    );
    // A finished log needs no capture.
    fs::write(doc.join("out.log"), "Status:conversion:0\n").unwrap();
    assert!(load(&root, "2101.00001").stderr.is_none());
    fs::remove_dir_all(&root).unwrap();
  }

  #[test]
  fn par_map_keeps_order_and_catches_panics() {
    let items: Vec<u32> = (0..50).collect();
    let out = par_map(
      &items,
      4,
      |x| if *x == 7 { panic!("boom") } else { x * 2 },
      |_| 999,
      |_| {},
    );
    assert_eq!(out[3], 6);
    assert_eq!(out[7], 999);
  }
}
