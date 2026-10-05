//! A format dump for the TeX tree this process reads.
//!
//! A dump is the format: the state `latex.ltx` (and the L3 kernel it loads) or `plain.tex` leaves,
//! built from one TeX tree's files. Packages come raw from the tree the process reads, so a dump
//! built from another tree is the wrong format for them — `expl3.sty` refuses a format whose L3
//! kernel date differs from its own ("Mismatched LaTeX support files", expl3.sty:64-78). The 2609
//! release ran a dump built from `/usr/local/texlive/2025` with `/usr/share/texlive`'s packages, and
//! 12,144 papers ended as Error (witnesses 2607.28725, 2609.40175).
//!
//! Every dump records the files it was built from, and the loaders pass over one this tree does not
//! have (`latexml_engine::dump_paths::dump_matches_tree`). When no dump found matches, the format
//! is built here once, as `--init` and fmtutil build it, into a per-user cache keyed by the engine
//! build and the tree's files, and that dump is used. When it cannot be built, the format loads
//! from the engine's own definitions, `LoadFormat`'s branch for a missing dump (Perl
//! Package.pm:2755-2766).

use std::{
  path::{Path, PathBuf},
  sync::OnceLock,
  time::{Duration, Instant},
};

use latexml_core::util::private_files::{
  private_dir, private_to_user, publish_read_only, trusted_dir, user_id,
};
use latexml_engine::dump_paths;

/// Marks, per format (`.failed.<kind>`), a cache directory where it could not be built, holding the
/// reason: later processes take the base branch at once instead of building again. The mark lapses
/// after [`FAILED_LAPSE`]; removing it retries at once.
const FAILED: &str = ".failed";

/// Counts, per format (`.attempts.<kind>`), the builds into a cache directory that have not
/// succeeded in a row, with the last one's reason: written before a build, as one that dies with
/// its process (killed, or a panic in a binary built to abort) records nothing after; removed by a
/// success, or replaced by the [`FAILED`] mark.
const ATTEMPTS: &str = ".attempts";

/// Builds of a format that may fail in a row before it is marked [`FAILED`]: a format that does not
/// build is not tried by every process, and one passing failure (a full disk, no thread) is not
/// taken for it.
const MAX_ATTEMPTS: u32 = 2;

/// How long a [`FAILED`] mark holds: a failure that passes (the disk is cleared, the tree repaired)
/// is not kept for good.
const FAILED_LAPSE: Duration = Duration::from_secs(24 * 60 * 60);

/// How long a process waits for another one building into the same directory — far longer than a
/// build takes, even on a loaded host.
const LOCK_WAIT: Duration = Duration::from_secs(300);

/// Ensures each format has a dump built from this process's TeX tree, building any that has none
/// into the cache. Once per process, before the first session loads a format; nothing to do without
/// a TeX tree, under `LATEXML_NODUMP`, or while a format is being built (`--init`, or the thread
/// building one here). A format that cannot be built is reported once, on stderr (and as a warning
/// in each conversion's log, `converter::note_format_build_failure`), and loads from the engine's
/// definitions. A build changes the process's working directory while it runs (see `build`), so it
/// belongs before the process converts anything.
pub fn ensure_format_dumps() {
  if dump_paths::building_format() {
    return;
  }
  static ENSURED: OnceLock<()> = OnceLock::new();
  ENSURED.get_or_init(|| {
    if let Err(reason) = ensure() {
      eprintln!(
        "[format_dumps] {reason}: the format loads from the engine's own definitions instead of a dump"
      );
      dump_paths::set_format_build_failure(reason);
    }
  });
}

fn ensure() -> Result<(), String> {
  if std::env::var_os("LATEXML_NODUMP").is_some() || std::env::var_os("LATEXML_INI_MODE").is_some()
  {
    return Ok(());
  }
  let kinds: Vec<&'static str> = ["plain", "latex"]
    .into_iter()
    .filter(|kind| dump_paths::runtime_source_stamps(kind).is_some())
    .collect();
  if kinds.is_empty() {
    return Ok(());
  }
  // A dump the configuration names, installs or keeps beside the sources is used before the cache's;
  // the embedded one only after it, as reading it back costs every process a 4 MB dump.
  let wanted: Vec<&'static str> = kinds
    .into_iter()
    .filter(|kind| !dump_paths::format_dump_on_disk(kind))
    .collect();
  if wanted.is_empty() {
    return Ok(());
  }
  // Only the dump's file name carries the year.
  let year = dump_paths::detect_ambient_texlive_year().unwrap_or(2000);
  let dir = match cache_dir() {
    Ok(dir) => dir,
    Err(_)
      if wanted
        .iter()
        .all(|kind| dump_paths::format_dump_available(kind)) =>
    {
      return Ok(());
    },
    Err(reason) => return Err(reason),
  };
  // Each kind uses the cache's dump on its own merits.
  let use_cached = |kind: &str| match cached_dump(&dir, kind, year) {
    Some((path, found_year)) => {
      dump_paths::set_rebuilt_dump(kind, path, found_year);
      true
    },
    None => false,
  };
  let missing: Vec<&'static str> = wanted
    .into_iter()
    .filter(|kind| !use_cached(kind) && !dump_paths::format_dump_available(kind))
    .collect();
  if missing.is_empty() {
    return Ok(());
  }
  private_dir(&dir)?;
  let mut reasons = Vec::new();
  let missing: Vec<&'static str> = missing
    .into_iter()
    .filter(|kind| match failed_before(&dir, kind) {
      Some(reason) => {
        reasons.push(reason);
        false
      },
      None => true,
    })
    .collect();
  if !missing.is_empty() {
    // What the builds so far have left, to tell whether one failed while this process waited. (A
    // build that fails between this read and its release of the lock goes unseen: the window is
    // the few instructions between its last write and its unlock.)
    let seen: Vec<Option<String>> = missing
      .iter()
      .map(|kind| attempts_text(&dir, kind))
      .collect();
    // One process builds; the others wait for it and find the dump, or its failure.
    let _lock = match lock_dir(&dir) {
      Ok(lock) => lock,
      Err(reason) => {
        reasons.push(reason);
        return Err(reasons.join("; "));
      },
    };
    // No other process builds here while this one holds the lock: a partial dump is a killed
    // build's.
    remove_partial_dumps(&dir);
    for (kind, seen) in missing.into_iter().zip(seen) {
      // Built by the process this one waited for.
      if use_cached(kind) {
        continue;
      }
      if let Some(reason) = failed_before(&dir, kind) {
        reasons.push(reason);
        continue;
      }
      // A build that failed while this process waited is not tried again at once: a passing
      // failure would likely recur, and a later process retries it.
      let now = attempts_text(&dir, kind);
      if now.is_some() && now != seen {
        reasons.push(previous_attempts(&dir, kind).1);
        continue;
      }
      if let Err(reason) = attempt(kind, year, &dir, &use_cached) {
        reasons.push(reason);
      }
    }
  }
  if reasons.is_empty() {
    Ok(())
  } else {
    Err(reasons.join("; "))
  }
}

/// Builds the `kind` dump into `dir`, counting it until it succeeds and marking the format
/// [`FAILED`] after [`MAX_ATTEMPTS`] builds in a row that did not.
fn attempt(
  kind: &'static str,
  year: u32,
  dir: &Path,
  use_cached: &dyn Fn(&str) -> bool,
) -> Result<(), String> {
  let (failed, last_reason) = previous_attempts(dir, kind);
  if failed >= MAX_ATTEMPTS {
    return Err(mark_failed(dir, kind, &last_reason));
  }
  note_attempt(
    dir,
    kind,
    failed + 1,
    &format!("the {kind} format build died with its process"),
  )?;
  eprintln!(
    "[format_dumps] no {kind} dump matches this TeX tree; building one into {}",
    dir.display()
  );
  let built = build(kind, year, dir).and_then(|()| {
    if use_cached(kind) {
      Ok(())
    } else {
      Err(format!(
        "the {kind} dump built into {} does not match this TeX tree",
        dir.display()
      ))
    }
  });
  match built {
    Ok(()) => {
      let _ = std::fs::remove_file(kind_file(dir, ATTEMPTS, kind));
      Ok(())
    },
    Err(reason) => {
      let _ = note_attempt(dir, kind, failed + 1, &reason);
      if failed + 1 >= MAX_ATTEMPTS {
        Err(mark_failed(dir, kind, &reason))
      } else {
        Err(reason)
      }
    },
  }
}

/// `dir`'s `marker` file (`.attempts`, `.failed`) for the `kind` format.
fn kind_file(dir: &Path, marker: &str, kind: &str) -> PathBuf {
  dir.join(format!("{marker}.{kind}"))
}

/// What the builds of the `kind` format into `dir` that have not succeeded have recorded.
fn attempts_text(dir: &Path, kind: &str) -> Option<String> {
  std::fs::read_to_string(kind_file(dir, ATTEMPTS, kind)).ok()
}

/// The builds of the `kind` format into `dir` that have not succeeded in a row so far, and the last
/// one's reason.
fn previous_attempts(dir: &Path, kind: &str) -> (u32, String) {
  let Some(text) = attempts_text(dir, kind) else {
    return (0, String::new());
  };
  let (count, reason) = text.split_once('\t').unwrap_or((&text, ""));
  let reason = if reason.is_empty() {
    format!("the {kind} format build did not succeed")
  } else {
    reason.to_string()
  };
  (count.trim().parse().unwrap_or(1), reason)
}

/// Records that the `count`th build of the `kind` format into `dir` in a row has not succeeded
/// (yet), and why.
fn note_attempt(dir: &Path, kind: &str, count: u32, reason: &str) -> Result<(), String> {
  let path = kind_file(dir, ATTEMPTS, kind);
  std::fs::write(&path, format!("{count}\t{reason}"))
    .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Marks the `kind` format as one that could not be built into `dir`, and says so. The mark holds
/// the reason in place of the count, so removing it gives the format fresh attempts.
fn mark_failed(dir: &Path, kind: &str, reason: &str) -> String {
  let path = kind_file(dir, FAILED, kind);
  if std::fs::write(&path, reason).is_ok() {
    let _ = std::fs::remove_file(kind_file(dir, ATTEMPTS, kind));
  }
  failure_message(&path, reason)
}

/// The failure a [`FAILED`] mark at `path` records, with how to retry.
fn failure_message(path: &Path, reason: &str) -> String {
  format!(
    "no dump matches this TeX tree and none could be built ({reason}; remove {} to retry)",
    path.display()
  )
}

/// Where dumps built here go: `$LATEXML_FORMAT_CACHE`, else the user's cache directory
/// (`$XDG_CACHE_HOME`, `~/.cache`, when absolute), else the temp dir under the user's id — one
/// directory per engine build and TeX tree, so a process reuses what an earlier one built. The build
/// is the executable's size and modification time beside the version: the embedded git revision is
/// the one `build.rs` last saw, and a dump is the engine's as much as the tree's. Absolute, as the
/// build runs in it.
fn cache_dir() -> Result<PathBuf, String> {
  let absolute_var = |name: &str| {
    std::env::var_os(name)
      .map(PathBuf::from)
      .filter(|dir| dir.is_absolute())
  };
  let base = std::env::var_os("LATEXML_FORMAT_CACHE")
    .map(PathBuf::from)
    .or_else(|| absolute_var("XDG_CACHE_HOME").map(|dir| dir.join("latexml-oxide/formats")))
    .or_else(|| absolute_var("HOME").map(|dir| dir.join(".cache/latexml-oxide/formats")))
    .unwrap_or_else(|| std::env::temp_dir().join(format!("latexml-oxide-formats-{}", user_id())));
  let base = std::path::absolute(&base)
    .map_err(|e| format!("cannot place the format cache {}: {e}", base.display()))?;
  let tree: String = ["plain", "latex"]
    .into_iter()
    .filter_map(dump_paths::runtime_source_stamps)
    .flatten()
    .map(|stamp| format!("{:08x}", stamp.checksum))
    .collect::<Vec<_>>()
    .join("-");
  let build = std::env::current_exe()
    .and_then(std::fs::metadata)
    .map(|meta| {
      let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_nanos());
      format!("{:x}-{modified:x}", meta.len())
    })
    .unwrap_or_default();
  Ok(base.join(format!("{}-{build}-{tree}", crate::identity::VERSION)))
}

/// The cache's `kind` dump for this tree and its year, if this user built it: a dump is loaded as
/// the format, so one in a directory or file another user could have written is not used.
fn cached_dump(dir: &Path, kind: &str, year: u32) -> Option<(PathBuf, u32)> {
  if !trusted_dir(dir) {
    return None;
  }
  dump_paths::resolve_versioned_in_dir(dir, kind, Some(year))
    .filter(|(path, _)| private_to_user(path))
}

/// The failure an earlier build of the `kind` format into `dir` recorded, if its mark has not lapsed
/// ([`FAILED_LAPSE`]); a lapsed mark is removed (no count is kept beside a mark, see `mark_failed`).
fn failed_before(dir: &Path, kind: &str) -> Option<String> {
  let path = kind_file(dir, FAILED, kind);
  let reason = std::fs::read_to_string(&path).ok()?;
  let lapsed = std::fs::metadata(&path)
    .and_then(|meta| meta.modified())
    .ok()
    .and_then(|marked| marked.elapsed().ok())
    .is_some_and(|age| age > FAILED_LAPSE);
  if lapsed {
    let _ = std::fs::remove_file(&path);
    return None;
  }
  Some(failure_message(&path, &reason))
}

/// Takes `dir`'s build lock, waiting up to [`LOCK_WAIT`] for a process building there.
fn lock_dir(dir: &Path) -> Result<std::fs::File, String> {
  let file = std::fs::File::create(dir.join(".lock"))
    .map_err(|e| format!("cannot lock {}: {e}", dir.display()))?;
  let deadline = Instant::now() + LOCK_WAIT;
  loop {
    match file.try_lock() {
      Ok(()) => return Ok(file),
      Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
        std::thread::sleep(Duration::from_millis(100));
      },
      Err(std::fs::TryLockError::WouldBlock) => {
        return Err(format!(
          "another process has been building into {} for {} s",
          dir.display(),
          LOCK_WAIT.as_secs()
        ));
      },
      Err(std::fs::TryLockError::Error(e)) => {
        return Err(format!("cannot lock {}: {e}", dir.display()));
      },
    }
  }
}

/// Removes the partial dumps killed builds left in `dir`.
fn remove_partial_dumps(dir: &Path) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    if entry.file_name().to_string_lossy().ends_with(".partial") {
      let _ = std::fs::remove_file(entry.path());
    }
  }
}

/// Builds the `kind` dump into `dir` as `--init` does, on a thread of its own: the engine's state is
/// per thread, so the build starts from a fresh one, leaves the caller's untouched, and is released
/// when it ends. Like fmtutil, it runs in a directory holding no TeX files — `dir` — as kpathsea
/// searches `.` first and a document's own copy of a format file (`./expl3-code.tex`) must not go
/// into the format; the working directory is the process's, so it is `dir` for every thread until
/// the build ends, whatever the outcome. As `tools/make_formats.sh` does, a build that logs an
/// error or a fatal one is not kept. Written beside its final name and renamed, so a concurrent
/// reader never sees half a dump.
fn build(kind: &'static str, year: u32, dir: &Path) -> Result<(), String> {
  let dest = dir.join(dump_paths::dump_filename(kind, year));
  let partial = dir.join(format!(
    ".{}.{}.partial",
    dump_paths::dump_filename(kind, year),
    std::process::id()
  ));
  let partial_name = partial.to_string_lossy().into_owned();
  let init_file = if kind == "latex" {
    "latex.ltx"
  } else {
    "plain.tex"
  };
  let previous = std::env::current_dir().map_err(|e| format!("no working directory: {e}"))?;
  let workdir = dir.to_path_buf();
  let joined = std::thread::Builder::new()
    .name(format!("{kind}-format"))
    .stack_size(256 * 1024 * 1024)
    .spawn(move || -> Result<(), String> {
      dump_paths::set_building_format(true);
      std::env::set_current_dir(&workdir)
        .map_err(|e| format!("cannot enter {}: {e}", workdir.display()))?;
      let opts = latexml_core::common::Config {
        bindings_dispatch: Some(std::rc::Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(std::rc::Rc::new(latexml_contrib::dispatch)),
        ..latexml_core::common::Config::default()
      };
      let mut converter = crate::converter::Converter::from_config(opts.clone());
      let result = converter
        .prepare_session(&opts)
        .map_err(|e| format!("cannot prepare the {kind} format build: {e}"))
        .and_then(|()| crate::ini_tex::dump_format(&mut converter, init_file, Some(&partial_name)))
        .and_then(|_| {
          let counts = latexml_core::common::error::snapshot_report_counts();
          if counts.fatal || counts.error > 0 {
            Err(format!(
              "building the {kind} format logged {} error(s){}",
              counts.error,
              if counts.fatal { " and a fatal one" } else { "" }
            ))
          } else {
            Ok(())
          }
        });
      drop(converter);
      latexml_core::reset_thread_engine();
      result
    })
    .map_err(|e| format!("cannot start the {kind} format build: {e}"))?
    .join();
  let returned = std::env::set_current_dir(&previous);
  let built = joined
    .map_err(|_| format!("the {kind} format build panicked"))
    .and_then(|built| built)
    .and_then(|()| returned.map_err(|e| format!("cannot return to {}: {e}", previous.display())))
    .and_then(|()| {
      publish_read_only(&partial)
        .map_err(|e| format!("cannot set the {kind} dump's permissions: {e}"))?;
      std::fs::rename(&partial, &dest)
        .map_err(|e| format!("cannot move the {kind} dump to {}: {e}", dest.display()))
    });
  if built.is_err() {
    let _ = std::fs::remove_file(&partial);
  }
  built
}
