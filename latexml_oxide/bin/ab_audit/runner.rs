//! `ab_audit run`: convert a paper list with two binaries into `<out>/A/<id>/` and `<out>/B/<id>/`, then
//! compare them.
//!
//! It refuses to start unless each binary's stamped revision (`--version`'s `(revision <sha>)`, or the
//! conversion banner of a binary built before that) is the commit the caller expects: 64x r1 and 64ra
//! ran their A/Bs on stale binaries. A historical replay names the stamp it accepts with
//! `--accept-stamp <sha>`, and the TSV header prints it; there is no silent bypass. It also refuses:
//! * two binaries with one stamp but different expected commits, unless `--accept-same-stamp` (the stamp
//!   then tells them apart by nothing; a loud line is printed either way);
//! * a binary older than its expected commit (file mtime before the commit's committer time, when
//!   `git -C <repo>` resolves the commit);
//! * two byte-identical binaries, unless `--allow-same-binary`;
//! * an empty paper list.
//!
//! **What the stamp cannot prove.** `build.rs` re-runs only when `LATEXML_GIT_SHA` changes, not when HEAD
//! moves (#528), so a target directory keeps the stamp of its first build: five distinct binaries of
//! 2026-10-10/11 (64u, 64ufix, 64zb, 64zc, 64w) all say `9569ca27a7`. A build must set
//! `LATEXML_GIT_SHA=$(git rev-parse --short HEAD)` for the stamp to mean anything, and even then a dirty
//! tree is not reflected. The mtime check catches only a binary built before its commit, not one built
//! after it from other sources; the binary hash (recorded per conversion) tells two builds apart but not
//! which source each came from. The mtime is weak evidence: `cp` without `-p` or a `touch` gives any old
//! binary a fresh mtime, and an amend or rebase moves the commit's committer time past a binary that does
//! hold its code (refused until rebuilt, the safe side).
//!
//! Per paper and binary, the payload brief's standard command: a fresh directory, `ulimit -v 8912896`,
//! `timeout 300`, `taskset -c <cores>`, `--preload=ar5iv.sty --timeout=240 --dest=out.xml --log=out.log`.
//! A finished conversion leaves `meta.tsv` (stamp, binary hash, settings hash, exit code, seconds,
//! timeout); a rerun skips every paper whose `meta.tsv` names the same stamp, binary hash and settings
//! (dump dir, timeouts, memory, TeX Live bin, corpus).

use std::{
  fs,
  hash::Hasher,
  path::{Path, PathBuf},
  process::Command,
  time::Instant,
};

use rustc_hash::FxHasher;

use crate::{
  report::{Output, run_compare},
  tree::{RunMeta, par_map},
};

/// The `run` subcommand's settings.
pub struct RunArgs {
  pub bins:              [PathBuf; 2],
  pub expect:            [String; 2],
  pub accept:            Vec<String>,
  pub papers:            PathBuf,
  pub out:               PathBuf,
  pub jobs:              usize,
  pub cores:             String,
  pub timeout:           u32,
  pub wall:              u32,
  pub memory_kb:         u64,
  pub corpus:            PathBuf,
  pub texlive_bin:       PathBuf,
  pub dump_dir:          Option<PathBuf>,
  pub allow_same_binary: bool,
  pub accept_same_stamp: bool,
  /// The git checkout that resolves the expected commits (for the mtime check).
  pub repo:              PathBuf,
}

/// The inherited environment that shapes a conversion, by prefix: `LATEXML_*` (the dump directory, the
/// no-dump switch …), `LXML_*`, `TEX*` (TeX Live's search paths), plus `SOURCE_DATE_EPOCH` and `PATH`.
/// Sorted.
fn inherited_env() -> Vec<(String, String)> {
  let mut vars: Vec<(String, String)> = std::env::vars()
    .filter(|(k, _)| {
      ["LATEXML_", "LXML_", "TEX"]
        .iter()
        .any(|p| k.starts_with(p))
        || matches!(k.as_str(), "SOURCE_DATE_EPOCH" | "PATH" | "HOME")
    })
    .collect();
  vars.sort();
  vars
}

/// A hash of the settings that shape a conversion's output (the run's options and the inherited
/// environment), so a resume under other settings reruns.
fn settings_hash(args: &RunArgs, env: &[(String, String)]) -> String {
  let mut h = FxHasher::default();
  h.write(
    format!(
      "{:?}|{}|{}|{}|{:?}|{:?}|{env:?}",
      args.dump_dir, args.timeout, args.wall, args.memory_kb, args.texlive_bin, args.corpus
    )
    .as_bytes(),
  );
  format!("{:016x}", h.finish())
}

/// The committer time of `sha` in `repo`, when git resolves it.
fn commit_time(repo: &Path, sha: &str) -> Option<i64> {
  let out = Command::new("git")
    .arg("-C")
    .arg(repo)
    .args(["log", "-1", "--format=%ct", sha, "--"])
    .output()
    .ok()?;
  if !out.status.success() {
    return None;
  }
  String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// The binary-age check: `Err` when the binary's mtime is before the expected commit's committer time
/// (it cannot hold that commit's code), else the note for the report header.
fn age_verdict(committed: Option<i64>, built: Option<i64>) -> Result<String, String> {
  match (committed, built) {
    (Some(c), Some(b)) if b < c => Err(format!(
      "was built before its expected commit (mtime {b} < committer time {c}): it cannot hold that commit's code"
    )),
    (Some(_), Some(_)) => Ok("; built after the expected commit".to_string()),
    _ => Ok("; mtime not checked (git cannot resolve the expected commit)".to_string()),
  }
}

/// One stamp on both binaries: a loud line always; `Err` (refused) when their expected commits differ and
/// `--accept-same-stamp` was not given, since the stamp then tells them apart by nothing.
fn same_stamp_verdict(
  stamps: [&str; 2],
  expect: [&str; 2],
  accept: bool,
) -> Result<Option<String>, String> {
  if stamps[0] != stamps[1] {
    return Ok(None);
  }
  let different = expect[0] != expect[1] && !stamp_matches(expect[0], expect[1]);
  let line = format!(
    "WARNING: A and B carry ONE stamp {} (expected {} and {}): the stamp does not tell them apart, only the \
     binary hashes do",
    stamps[0], expect[0], expect[1]
  );
  match (different, accept) {
    (true, false) => Err(format!(
      "{line}\nREFUSED: different expected commits behind one stamp; pass --accept-same-stamp to replay such \
       binaries"
    )),
    (true, true) => Ok(Some(format!("{line} (ACCEPTED by --accept-same-stamp)"))),
    (false, _) => Ok(Some(line)),
  }
}

/// The paper lines of a list file, one per paper key (the first occurrence kept).
pub fn paper_lines(text: &str) -> Vec<String> {
  let mut seen = std::collections::HashSet::new();
  text
    .lines()
    .map(|l| l.split('\t').next().unwrap_or("").trim().to_string())
    .filter(|l| !l.is_empty() && !l.starts_with('#'))
    .filter(|l| seen.insert(paper_key(l)))
    .collect()
}

/// A paper's key: an archive path's file stem, an old-style `hep-th/9703192` without its slash, else the
/// line itself.
pub fn paper_key(line: &str) -> String {
  if line.ends_with(".zip") {
    return Path::new(line)
      .file_stem()
      .map(|s| s.to_string_lossy().into_owned())
      .unwrap_or_else(|| line.to_string());
  }
  match line.split_once('/') {
    Some((archive, number))
      if number.len() == 7
        && number.bytes().all(|b| b.is_ascii_digit())
        && !archive.contains('/') =>
    {
      format!("{archive}{number}")
    },
    _ => line.to_string(),
  }
}

/// The archive of a paper line: the path itself when it names a `.zip`, else
/// `<corpus>/<yymm>/<id>/<id>.zip` with `yymm` the first four digits of the id (`hep-th9703192` → 9703).
pub fn paper_zip(line: &str, corpus: &Path) -> PathBuf {
  if line.ends_with(".zip") {
    return PathBuf::from(line);
  }
  let id = paper_key(line);
  let digits: String = id
    .chars()
    .skip_while(|c| !c.is_ascii_digit())
    .take_while(char::is_ascii_digit)
    .collect();
  let yymm = digits.get(..4).unwrap_or(&digits);
  corpus.join(yymm).join(&id).join(format!("{id}.zip"))
}

/// Whether a stamp is the expected commit: one a prefix of the other, at least 7 hex digits each.
pub fn stamp_matches(expected: &str, stamp: &str) -> bool {
  let (e, s) = (
    expected.trim().to_ascii_lowercase(),
    stamp.trim().to_ascii_lowercase(),
  );
  e.len() >= 7
    && s.len() >= 7
    && s.bytes().all(|b| b.is_ascii_hexdigit())
    && (e.starts_with(&s) || s.starts_with(&e))
}

/// The revision in `… (revision <stamp>)` or `…; revision <stamp>) started …`.
pub fn parse_revision(text: &str) -> Option<String> {
  let at = text.find("revision ")? + "revision ".len();
  let stamp: String = text[at..]
    .chars()
    .take_while(|c| !matches!(c, ')' | ' ' | '\n'))
    .collect();
  (!stamp.is_empty()).then_some(stamp)
}

/// A binary's stamped revision: `--version`, else the banner of a one-line conversion (binaries built
/// before `--version` printed it).
/// `scratch` is the run's own `<root>/.stamp_<tag>` (checked by [`scratch_dir`]), never a shared
/// temporary directory.
fn read_stamp(
  bin: &Path,
  texlive_bin: &Path,
  scratch: &Path,
) -> Result<(String, &'static str), String> {
  let version = Command::new(bin)
    .arg("--version")
    .output()
    .map_err(|e| format!("{}: {e}", bin.display()))?;
  if let Some(stamp) = parse_revision(&String::from_utf8_lossy(&version.stdout)) {
    return Ok((stamp, "--version"));
  }
  let dir = scratch;
  let _ = fs::remove_dir_all(dir);
  fs::create_dir_all(dir).map_err(|e| e.to_string())?;
  fs::write(
    dir.join("stamp.tex"),
    "\\documentclass{article}\\begin{document}x\\end{document}\n",
  )
  .map_err(|e| e.to_string())?;
  let status = Command::new("timeout")
    .args(["120"])
    .arg(bin)
    .args(["--dest=stamp.xml", "--log=stamp.log", "stamp.tex"])
    .current_dir(dir)
    .env("PATH", tl_path(texlive_bin))
    .output();
  let log = fs::read_to_string(dir.join("stamp.log")).unwrap_or_default();
  let _ = fs::remove_dir_all(dir);
  status.map_err(|e| format!("{}: {e}", bin.display()))?;
  parse_revision(log.lines().next().unwrap_or(""))
    .map(|s| (s, "banner"))
    .ok_or_else(|| {
      format!(
        "{}: no `revision` in `--version` or in the conversion banner",
        bin.display()
      )
    })
}

/// A hash of the binary's bytes, so two copies of one stamp (a rebuild that did not move the stamp) are
/// still told apart.
fn hash_file(path: &Path) -> Result<String, String> {
  let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
  let mut h = FxHasher::default();
  for chunk in bytes.chunks(1 << 16) {
    h.write(chunk);
  }
  Ok(format!("{:016x}", h.finish()))
}

fn tl_path(texlive_bin: &Path) -> String {
  format!(
    "{}:{}",
    texlive_bin.display(),
    std::env::var("PATH").unwrap_or_default()
  )
}

const SCRIPT: &str = "ulimit -v \"$1\"; exec timeout -k 10 \"$2\" taskset -c \"$3\" \"$4\" --preload=ar5iv.sty \
                      --timeout=\"$5\" --dest=out.xml --log=out.log \"$6\" > stdout.txt 2> stderr.txt";

struct Side {
  tag:   &'static str,
  bin:   PathBuf,
  stamp: String,
  hash:  String,
}

/// A paper's directory, `<root>/<tag>/<key>`, provably inside the (canonical) output root: the key must be
/// one plain path component (no `..`, `.`, separator or absolute path), so the `remove_dir_all` that gives
/// each conversion a fresh directory can never reach outside the root.
pub fn paper_dir(root: &Path, tag: &str, key: &str) -> Result<PathBuf, String> {
  let mut parts = Path::new(key).components();
  let one_plain =
    matches!(parts.next(), Some(std::path::Component::Normal(_))) && parts.next().is_none();
  if !one_plain || key.contains('/') || key.contains('\\') {
    return Err(format!("paper key {key:?} is not one plain path component"));
  }
  let dir = root.join(tag).join(key);
  if !root.is_absolute() || !dir.starts_with(root) || dir.parent() != Some(&root.join(tag)) {
    return Err(format!(
      "{} is not inside the output root {}",
      dir.display(),
      root.display()
    ));
  }
  Ok(dir)
}

/// Create `<root>/<tag>` and prove it is a real directory directly inside the canonical `root`: a
/// symlinked `A`/`B` would carry every per-paper `remove_dir_all` outside it.
pub fn tag_dir(root: &Path, tag: &str) -> Result<PathBuf, String> {
  let dir = root.join(tag);
  fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
  let link = fs::symlink_metadata(&dir)
    .map(|m| m.file_type().is_symlink())
    .unwrap_or(true);
  let real = fs::canonicalize(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
  if link || real.parent() != Some(root) {
    return Err(format!(
      "{} is a symlink or not directly inside the output root {}",
      dir.display(),
      root.display()
    ));
  }
  Ok(dir)
}

/// The stamp-reading scratch `<root>/.stamp_<tag>`, inside the canonical root (not a shared temporary
/// directory), refused when it is a symlink.
pub fn scratch_dir(root: &Path, tag: &str) -> Result<PathBuf, String> {
  let dir = root.join(format!(".stamp_{tag}"));
  // `dir` is `root` joined with one plain name (the tag is `A` or `B`), so its parent is `root` by
  // construction; a symlink is what could carry the removal elsewhere.
  if fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) {
    return Err(format!("{} is a symlink", dir.display()));
  }
  Ok(dir)
}

/// Run one paper on one side unless done for the same stamp and binary.
fn run_one(
  args: &RunArgs,
  root: &Path,
  side: &Side,
  line: &str,
  settings: &str,
) -> std::io::Result<bool> {
  let key = paper_key(line);
  let dir = paper_dir(root, side.tag, &key).map_err(std::io::Error::other)?;
  let zip = paper_zip(line, &args.corpus);
  // The archive's size and mtime: a replaced archive reruns.
  let input = fs::metadata(&zip)
    .ok()
    .map(|m| {
      let mtime = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
      format!("{}:{}", m.len(), mtime.map_or(0, |d| d.as_nanos()))
    })
    .unwrap_or_default();
  if let Ok(text) = fs::read_to_string(dir.join("meta.tsv")) {
    let meta = RunMeta::parse(&text);
    // A missing archive is never a finished conversion: it reruns once the archive is there.
    if meta.stamp == side.stamp
      && meta.bin_hash == side.hash
      && meta.settings == settings
      && meta.input == input
      && meta.note != "input-missing"
    {
      return Ok(false);
    }
  }
  let _ = fs::remove_dir_all(&dir);
  fs::create_dir_all(&dir)?;
  let mut meta = RunMeta {
    stamp: side.stamp.clone(),
    bin_hash: side.hash.clone(),
    settings: settings.to_string(),
    input,
    ..RunMeta::default()
  };
  if !zip.is_file() {
    meta.note = "input-missing".to_string();
    fs::write(dir.join("meta.tsv"), meta.render())?;
    return Ok(true);
  }
  let start = Instant::now();
  let mut cmd = Command::new("bash");
  cmd
    .arg("-c")
    .arg(SCRIPT)
    .arg("ab_audit")
    .arg(args.memory_kb.to_string())
    .arg(args.wall.to_string())
    .arg(&args.cores)
    .arg(&side.bin)
    .arg(args.timeout.to_string())
    .arg(&zip)
    .current_dir(&dir)
    .env("PATH", tl_path(&args.texlive_bin));
  if let Some(dumps) = &args.dump_dir {
    cmd.env("LATEXML_DUMP_DIR", dumps);
  }
  let status = cmd.status()?;
  meta.secs = Some(start.elapsed().as_secs_f64());
  meta.rc = Some(status.code().unwrap_or(-1));
  meta.timed_out = matches!(meta.rc, Some(124 | 137));
  fs::write(dir.join("meta.tsv"), meta.render())?;
  Ok(true)
}

/// `ab_audit run`. Returns the process exit code: 2 refused, 1 a regression or review row, 0 otherwise.
pub fn run(args: &RunArgs) -> std::io::Result<i32> {
  let lines = paper_lines(&fs::read_to_string(&args.papers)?);
  // The output root, canonical, so every per-paper directory removed below is checked to lie inside it.
  fs::create_dir_all(&args.out)?;
  let root = fs::canonicalize(&args.out)?;
  let unsafe_keys: Vec<String> = lines
    .iter()
    .filter_map(|l| paper_dir(&root, "A", &paper_key(l)).err())
    .collect();
  if !unsafe_keys.is_empty() {
    eprintln!("ab_audit run: REFUSED: {}", unsafe_keys.join("; "));
    return Ok(2);
  }
  if lines.is_empty() {
    eprintln!(
      "ab_audit run: REFUSED: no papers in {}",
      args.papers.display()
    );
    return Ok(2);
  }
  let settings = settings_hash(args, &inherited_env());
  let mut sides = Vec::new();
  let mut verdicts = Vec::new();
  for (i, tag) in ["A", "B"].into_iter().enumerate() {
    let bin = fs::canonicalize(&args.bins[i]).unwrap_or_else(|_| args.bins[i].clone());
    let scratch = match scratch_dir(&root, tag) {
      Ok(dir) => dir,
      Err(e) => {
        eprintln!("ab_audit run: REFUSED: {e}");
        return Ok(2);
      },
    };
    let (stamp, source) = match read_stamp(&bin, &args.texlive_bin, &scratch) {
      Ok(s) => s,
      Err(e) => {
        eprintln!("ab_audit run: REFUSED: {tag}: cannot read the stamp of {e}");
        return Ok(2);
      },
    };
    let hash = hash_file(&bin).map_err(std::io::Error::other)?;
    let expected = &args.expect[i];
    let mut verdict = if stamp_matches(expected, &stamp) {
      "matches the expected commit".to_string()
    } else if args.accept.iter().any(|a| a == &stamp) {
      format!("ACCEPTED by --accept-stamp {stamp} (expected {expected})")
    } else {
      eprintln!(
        "ab_audit run: REFUSED: {tag} {} is stamped {stamp} ({source}), not the expected {expected}.\n  Rebuild \
         it from {expected} with LATEXML_GIT_SHA=$(git rev-parse --short HEAD) (build.rs keeps a stale stamp \
         otherwise), or replay a historical binary with --accept-stamp {stamp} (printed in the report header).",
        bin.display()
      );
      return Ok(2);
    };
    let mtime = fs::metadata(&bin)
      .and_then(|m| m.modified())
      .ok()
      .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
      .map(|d| d.as_secs() as i64);
    match age_verdict(commit_time(&args.repo, expected), mtime) {
      Ok(note) => verdict.push_str(&note),
      Err(why) => {
        eprintln!("ab_audit run: REFUSED: {tag} {} {why}", bin.display());
        return Ok(2);
      },
    }
    if verdict.ends_with("(git cannot resolve the expected commit)") {
      verdict.push_str(&format!(": {expected} in {}", args.repo.display()));
    }
    verdicts.push(format!("{source}: {verdict}"));
    sides.push(Side { tag, bin, stamp, hash });
  }
  let mut header = Vec::new();
  let stamps = [sides[0].stamp.as_str(), sides[1].stamp.as_str()];
  match same_stamp_verdict(
    stamps,
    [&args.expect[0], &args.expect[1]],
    args.accept_same_stamp,
  ) {
    Ok(Some(line)) => {
      eprintln!("ab_audit run: {line}");
      header.push(line);
    },
    Ok(None) => {},
    Err(line) => {
      eprintln!("ab_audit run: {line}");
      return Ok(2);
    },
  }
  if sides[0].hash == sides[1].hash && !args.allow_same_binary {
    eprintln!(
      "ab_audit run: REFUSED: A and B are the same binary (hash {}); pass --allow-same-binary for a noise baseline",
      sides[0].hash
    );
    return Ok(2);
  }
  header.push(format!(
    "command: ulimit -v {} timeout {} taskset -c {} <bin> --preload=ar5iv.sty --timeout={} -j {}{} (settings {settings})",
    args.memory_kb,
    args.wall,
    args.cores,
    args.timeout,
    args.jobs,
    args
      .dump_dir
      .as_ref()
      .map(|d| format!(" LATEXML_DUMP_DIR={}", d.display()))
      .unwrap_or_default()
  ));
  for (side, verdict) in sides.iter().zip(&verdicts) {
    if let Err(e) = tag_dir(&root, side.tag) {
      eprintln!("ab_audit run: REFUSED: {e}");
      return Ok(2);
    }
    fs::write(
      root.join(side.tag).join("STAMP"),
      format!(
        "binary\t{}\nstamp\t{}\nbin_hash\t{}\nverdict\t{verdict}\n",
        side.bin.display(),
        side.stamp,
        side.hash
      ),
    )?;
  }
  let jobs: Vec<(usize, &str)> = lines
    .iter()
    .flat_map(|l| [(0, l.as_str()), (1, l.as_str())])
    .collect();
  let total = jobs.len();
  let start = Instant::now();
  let ran = par_map(
    &jobs,
    args.jobs,
    |(side, line)| {
      run_one(args, &root, &sides[*side], line, &settings).unwrap_or_else(|e| {
        eprintln!("ab_audit run: {line} {}: {e}", sides[*side].tag);
        true
      })
    },
    |_| true,
    |done| {
      if done % 50 == 0 || done == total {
        eprintln!(
          "ab_audit run: {done}/{total} conversions ({:.0} s)",
          start.elapsed().as_secs_f64()
        );
      }
    },
  );
  header.push(format!(
    "{} conversions run, {} resumed as done",
    ran.iter().filter(|r| **r).count(),
    ran.iter().filter(|r| !**r).count()
  ));
  let keys_file = root.join("papers.txt");
  fs::write(
    &keys_file,
    lines
      .iter()
      .map(|l| paper_key(l) + "\n")
      .collect::<String>(),
  )?;
  let out = Output { prefix: root.join("compare") };
  let regression = run_compare(
    &root.join("A"),
    &root.join("B"),
    Some(&keys_file),
    &out,
    args.jobs,
    &header,
  )?;
  Ok(i32::from(regression))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn paper_lines_resolve_to_the_corpus_layout() {
    let corpus = Path::new("/data/arxmliv");
    assert_eq!(
      paper_zip("0810.4982", corpus),
      PathBuf::from("/data/arxmliv/0810/0810.4982/0810.4982.zip")
    );
    assert_eq!(
      paper_zip("hep-th/9703192", corpus),
      PathBuf::from("/data/arxmliv/9703/hep-th9703192/hep-th9703192.zip")
    );
    assert_eq!(paper_key("/x/y/astro-ph0102416.zip"), "astro-ph0102416");
    assert_eq!(paper_key("bundle/name"), "bundle/name");
  }

  #[test]
  fn the_paper_list_is_deduplicated_by_key() {
    assert_eq!(
      paper_lines("0810.4982\n# c\nhep-th/9703192\n0810.4982\nhep-th9703192\n\n/x/0810.4982.zip\n"),
      vec!["0810.4982".to_string(), "hep-th/9703192".to_string()]
    );
  }

  fn args(dump: Option<&str>, timeout: u32) -> RunArgs {
    RunArgs {
      bins: [PathBuf::from("a"), PathBuf::from("b")],
      expect: [String::new(), String::new()],
      accept: Vec::new(),
      papers: PathBuf::new(),
      out: PathBuf::new(),
      jobs: 1,
      cores: "0".into(),
      timeout,
      wall: 300,
      memory_kb: 1,
      corpus: PathBuf::from("/data/arxmliv"),
      texlive_bin: PathBuf::from("/tl"),
      dump_dir: dump.map(PathBuf::from),
      allow_same_binary: false,
      accept_same_stamp: false,
      repo: PathBuf::from("."),
    }
  }

  #[test]
  fn other_settings_invalidate_a_resume() {
    let base = settings_hash(&args(None, 240), &[]);
    assert_eq!(base, settings_hash(&args(None, 240), &[]));
    assert_ne!(base, settings_hash(&args(Some("/dumps"), 240), &[]));
    assert_ne!(base, settings_hash(&args(None, 60), &[]));
    // An inherited LATEXML_DUMP_DIR, LATEXML_NODUMP, TEXMF* or SOURCE_DATE_EPOCH counts too.
    let nodump = [("LATEXML_NODUMP".to_string(), "1".to_string())];
    assert_ne!(base, settings_hash(&args(None, 240), &nodump));
  }

  #[test]
  fn a_resume_reruns_under_other_settings_or_another_archive() {
    let base = std::env::temp_dir().join(format!("ab_audit_resume_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("corpus/0810/0810.4982")).unwrap();
    let root = fs::canonicalize(&base).unwrap();
    let zip = root.join("corpus/0810/0810.4982/0810.4982.zip");
    fs::write(&zip, "PK").unwrap();
    let mut a = args(None, 240);
    a.out = root.clone();
    a.corpus = root.join("corpus");
    // `true` exits at once: a finished (if empty) conversion.
    let side = Side {
      tag:   "A",
      bin:   PathBuf::from("/bin/true"),
      stamp: "s".into(),
      hash:  "h".into(),
    };
    let settings = settings_hash(&a, &[]);
    tag_dir(&root, "A").unwrap();
    assert!(run_one(&a, &root, &side, "0810.4982", &settings).unwrap());
    assert!(
      !run_one(&a, &root, &side, "0810.4982", &settings).unwrap(),
      "same settings: resumed"
    );
    assert!(
      run_one(&a, &root, &side, "0810.4982", "other").unwrap(),
      "other settings: rerun"
    );
    // A replaced archive (another size) reruns.
    fs::write(&zip, "PK and more").unwrap();
    assert!(
      run_one(&a, &root, &side, "0810.4982", "other").unwrap(),
      "another archive: rerun"
    );
    fs::remove_dir_all(&base).unwrap();
  }

  #[test]
  fn one_stamp_for_two_expected_commits_is_refused_unless_accepted() {
    let line = same_stamp_verdict(["9569ca27a7"; 2], ["bd6678a466", "734180d0bf"], false);
    assert!(
      line.as_ref().is_err_and(|l| l.contains("REFUSED")),
      "{line:?}"
    );
    let line = same_stamp_verdict(["9569ca27a7"; 2], ["bd6678a466", "734180d0bf"], true);
    assert!(
      line
        .as_ref()
        .is_ok_and(|l| l.as_deref().is_some_and(|l| l.contains("ACCEPTED"))),
      "{line:?}"
    );
    // One expected commit (a noise baseline) is only warned about; different stamps say nothing.
    assert!(
      same_stamp_verdict(["abc1234"; 2], ["abc1234", "abc1234"], false).is_ok_and(|l| l.is_some())
    );
    assert_eq!(
      same_stamp_verdict(["abc1234", "def5678"], ["x", "y"], false),
      Ok(None)
    );
  }

  #[test]
  fn a_binary_older_than_its_commit_is_refused() {
    assert!(age_verdict(Some(200), Some(100)).is_err());
    assert!(age_verdict(Some(100), Some(200)).is_ok_and(|n| n.contains("built after")));
    assert!(age_verdict(None, Some(200)).is_ok_and(|n| n.contains("not checked")));
  }

  #[test]
  fn commit_times_resolve_in_this_checkout_or_not_at_all() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(commit_time(here, "no-such-commit-label"), None);
    // HEAD resolves wherever the crate is built from a git checkout (not from a packaged crate).
    if here.join("../.git").exists() {
      assert!(commit_time(here, "HEAD").is_some_and(|t| t > 1_600_000_000));
    }
  }

  #[test]
  fn a_paper_directory_never_leaves_the_output_root() {
    let root = Path::new("/data/out");
    assert_eq!(
      paper_dir(root, "A", "0810.4982"),
      Ok(PathBuf::from("/data/out/A/0810.4982"))
    );
    assert!(paper_dir(root, "B", "hep-th9703192").is_ok());
    for key in ["..", ".", "", "../x", "a/../../b", "/etc", "a/b", "x/.."] {
      assert!(paper_dir(root, "A", key).is_err(), "{key:?}");
    }
    assert!(
      paper_dir(Path::new("relative/out"), "A", "x").is_err(),
      "a relative root"
    );
    // What a paper list can produce: an archive's stem is safe, a dotted path is refused.
    assert!(paper_dir(root, "A", &paper_key("../../x/2301.00001.zip")).is_ok());
    assert!(paper_dir(root, "A", &paper_key("../etc")).is_err());
  }

  #[test]
  fn a_symlinked_side_directory_is_refused() {
    let base = std::env::temp_dir().join(format!("ab_audit_tag_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("out")).unwrap();
    fs::create_dir_all(base.join("elsewhere")).unwrap();
    let root = fs::canonicalize(base.join("out")).unwrap();
    assert!(tag_dir(&root, "A").is_ok());
    std::os::unix::fs::symlink(base.join("elsewhere"), root.join("B")).unwrap();
    assert!(
      tag_dir(&root, "B").is_err(),
      "a symlinked B escapes the root"
    );
    assert_eq!(scratch_dir(&root, "A"), Ok(root.join(".stamp_A")));
    std::os::unix::fs::symlink(base.join("elsewhere"), root.join(".stamp_B")).unwrap();
    assert!(scratch_dir(&root, "B").is_err());
    fs::remove_dir_all(&base).unwrap();
  }

  #[test]
  fn a_missing_archive_is_never_resumed() {
    let base = std::env::temp_dir().join(format!("ab_audit_inputs_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let root = fs::canonicalize(&base).unwrap();
    let mut a = args(None, 240);
    a.out = root.clone();
    a.corpus = root.join("corpus");
    let side = Side {
      tag:   "A",
      bin:   PathBuf::from("/bin/true"),
      stamp: "s".into(),
      hash:  "h".into(),
    };
    tag_dir(&root, "A").unwrap();
    assert!(run_one(&a, &root, &side, "0810.4982", "x").unwrap());
    assert!(
      run_one(&a, &root, &side, "0810.4982", "x").unwrap(),
      "input-missing reruns"
    );
    fs::remove_dir_all(&base).unwrap();
  }

  #[test]
  fn stamps_match_by_prefix_and_never_unknown() {
    assert!(stamp_matches("44ff2ac956", "44ff2ac"));
    assert!(stamp_matches("44ff2ac", "44ff2ac956"));
    assert!(!stamp_matches("44ff2ac956", "9569ca27a7"));
    assert!(!stamp_matches("unknown", "unknown"));
    assert!(!stamp_matches("44f", "44f"));
  }

  #[test]
  fn the_revision_reads_from_version_and_banner() {
    assert_eq!(
      parse_revision("latexml_oxide 0.7.6 (revision 44ff2ac956)\n").as_deref(),
      Some("44ff2ac956")
    );
    assert_eq!(
      parse_revision(
        "latexml_oxide.64zc-rel (latexml-oxide 0.7.6; revision 9569ca27a7) started 2026-10-11"
      )
      .as_deref(),
      Some("9569ca27a7")
    );
    assert_eq!(parse_revision("latexml_oxide 0.7.6\n"), None);
  }
}
