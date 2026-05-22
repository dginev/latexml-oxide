//! Resolve a writeable cache directory for runtime resource extraction.
//!
//! The binary embeds XSLT stylesheets, CSS/JS, and the RelaxNG tree at
//! compile time. On first use each subsystem materialises its embedded
//! files to disk so that libxslt / libxml2 / the post-processor can
//! treat them as ordinary file paths. The default landing zone is the
//! user's XDG cache directory; that's per-user (no collision when two
//! users share a host), persistent across reboots (no re-extraction
//! cost on subsequent invocations), and matches the convention every
//! other well-behaved CLI follows.
//!
//! Resolution order (first hit wins):
//!
//! 1. `$LATEXML_CACHE_DIR` — explicit override, useful for sandboxed
//!    runners and one-off testing. Used verbatim, joined with `subdir`.
//! 2. `$XDG_CACHE_HOME/latexml_oxide/<subdir>` — POSIX standard.
//! 3. `$HOME/.cache/latexml_oxide/<subdir>` — fallback when XDG_CACHE_HOME
//!    is unset, which is the common case on a fresh user account.
//! 4. `$TMPDIR/latexml_oxide_<uid>/<subdir>` — no-`$HOME` fallback for
//!    daemons / sandbox runners / service accounts that lack a homedir.
//!    The `_<uid>` suffix preserves multi-user safety on shared /tmp.
//! 5. `/tmp/latexml_oxide_<uid>/<subdir>` — last resort.
//!
//! The directory is created if missing. Returns `None` only when every
//! tier above failed to produce a writable path — at which point the
//! caller should log a warning and fall back to in-memory operation
//! (or, for embedded-XSLT, refuse the stylesheet resolution).

use std::path::PathBuf;

/// Resolve and create the cache subdirectory for `subdir`. Returns the
/// canonical path, or `None` if no writable location could be found.
///
/// `subdir` is the per-subsystem leaf (e.g. `"xslt"`, `"resources"`,
/// `"relaxng"`). Callers may treat the returned path as stable across
/// invocations within the same process and across reboots — extracted
/// files survive system restarts and idempotently overwrite stale
/// content.
pub fn for_subdir(subdir: &str) -> Option<PathBuf> {
  for candidate in candidate_paths(subdir) {
    if ensure_writable(&candidate) {
      return Some(candidate);
    }
  }
  None
}

/// Yield candidates in priority order without performing IO. Public
/// for `#[cfg(test)]` introspection; production code goes through
/// [`for_subdir`].
fn candidate_paths(subdir: &str) -> Vec<PathBuf> {
  let mut out = Vec::with_capacity(5);

  if let Ok(override_root) = std::env::var("LATEXML_CACHE_DIR") {
    if !override_root.is_empty() {
      out.push(PathBuf::from(override_root).join(subdir));
    }
  }
  if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
    if !xdg.is_empty() {
      out.push(PathBuf::from(xdg).join("latexml_oxide").join(subdir));
    }
  }
  if let Ok(home) = std::env::var("HOME") {
    if !home.is_empty() {
      out.push(
        PathBuf::from(home)
          .join(".cache")
          .join("latexml_oxide")
          .join(subdir),
      );
    }
  }

  // Per-uid tmp fallback. `std::env::temp_dir()` honours `$TMPDIR`
  // first, then falls back to `/tmp` — so this covers both bullets 4
  // and 5 of the doc comment unless `TMPDIR=` is explicitly empty,
  // which we treat as unset.
  let uid_suffix = current_uid_suffix();
  out.push(
    std::env::temp_dir()
      .join(format!("latexml_oxide_{}", uid_suffix))
      .join(subdir),
  );
  out
}

/// Return the effective uid as a string. On unix this is `geteuid()`;
/// on non-unix targets we use the literal `"shared"` so the path is
/// still well-formed (only relevant for the Windows port that does not
/// exist yet).
#[cfg(unix)]
fn current_uid_suffix() -> String {
  // SAFETY: `geteuid()` is documented as always succeeding — it
  // returns the current effective uid without setting errno or
  // requiring any prerequisite state.
  let uid = unsafe { libc::geteuid() };
  uid.to_string()
}

#[cfg(not(unix))]
fn current_uid_suffix() -> String { "shared".to_string() }

fn ensure_writable(dir: &PathBuf) -> bool {
  if std::fs::create_dir_all(dir).is_err() {
    return false;
  }
  // Probe writability — `create_dir_all` succeeds against a read-only
  // existing path on some filesystems, so we explicitly stat + write
  // a sentinel. Use a uniquely-named sentinel so concurrent processes
  // don't race on each other's probes.
  let sentinel = dir.join(format!(".latexml_oxide_probe_{}", std::process::id()));
  if std::fs::write(&sentinel, b"").is_err() {
    return false;
  }
  let _ = std::fs::remove_file(&sentinel);
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  // The candidate-paths logic reads env vars; tests must serialise
  // because Rust runs them in parallel by default and changing
  // env vars is process-global.
  use std::sync::Mutex;
  static ENV_LOCK: Mutex<()> = Mutex::new(());

  fn with_env<F: FnOnce()>(vars: &[(&str, Option<&str>)], f: F) {
    let _guard = ENV_LOCK.lock().unwrap();
    let saved: Vec<_> = vars
      .iter()
      .map(|(k, _)| (k.to_string(), std::env::var(k).ok()))
      .collect();
    for (k, v) in vars {
      // SAFETY: tests are serialised via ENV_LOCK; no other thread
      // is reading or writing the process env block concurrently.
      unsafe {
        match v {
          Some(val) => std::env::set_var(k, val),
          None => std::env::remove_var(k),
        }
      }
    }
    f();
    for (k, prev) in saved {
      // SAFETY: same serialisation guarantee as above.
      unsafe {
        match prev {
          Some(val) => std::env::set_var(&k, val),
          None => std::env::remove_var(&k),
        }
      }
    }
  }

  #[test]
  fn override_wins_over_xdg_and_home() {
    with_env(
      &[
        ("LATEXML_CACHE_DIR", Some("/explicit/override")),
        ("XDG_CACHE_HOME", Some("/xdg/cache")),
        ("HOME", Some("/home/user")),
      ],
      || {
        let paths = candidate_paths("xslt");
        assert_eq!(paths[0], PathBuf::from("/explicit/override/xslt"));
      },
    );
  }

  #[test]
  fn xdg_wins_over_home_when_no_override() {
    with_env(
      &[
        ("LATEXML_CACHE_DIR", None),
        ("XDG_CACHE_HOME", Some("/xdg/cache")),
        ("HOME", Some("/home/user")),
      ],
      || {
        let paths = candidate_paths("resources");
        assert_eq!(paths[0], PathBuf::from("/xdg/cache/latexml_oxide/resources"));
      },
    );
  }

  #[test]
  fn home_used_when_xdg_unset() {
    with_env(
      &[
        ("LATEXML_CACHE_DIR", None),
        ("XDG_CACHE_HOME", None),
        ("HOME", Some("/home/user")),
      ],
      || {
        let paths = candidate_paths("relaxng");
        assert_eq!(
          paths[0],
          PathBuf::from("/home/user/.cache/latexml_oxide/relaxng"),
        );
      },
    );
  }

  #[test]
  fn tmp_fallback_when_no_home() {
    with_env(
      &[
        ("LATEXML_CACHE_DIR", None),
        ("XDG_CACHE_HOME", None),
        ("HOME", None),
      ],
      || {
        let paths = candidate_paths("xslt");
        assert_eq!(paths.len(), 1);
        let last = &paths[0];
        assert!(
          last
            .to_string_lossy()
            .contains(&format!("latexml_oxide_{}", current_uid_suffix())),
          "expected uid-suffixed tmp path, got {:?}",
          last,
        );
        assert!(last.ends_with("xslt"), "missing subdir suffix in {:?}", last);
      },
    );
  }

  #[test]
  fn empty_env_vars_treated_as_unset() {
    with_env(
      &[
        ("LATEXML_CACHE_DIR", Some("")),
        ("XDG_CACHE_HOME", Some("")),
        ("HOME", Some("/home/user")),
      ],
      || {
        let paths = candidate_paths("xslt");
        // Empty string should NOT produce a candidate; HOME wins.
        assert_eq!(
          paths[0],
          PathBuf::from("/home/user/.cache/latexml_oxide/xslt"),
        );
      },
    );
  }

  #[test]
  fn for_subdir_creates_the_directory() {
    let tmpdir = tempfile::tempdir().expect("tempdir");
    with_env(
      &[(
        "LATEXML_CACHE_DIR",
        Some(tmpdir.path().to_str().unwrap()),
      )],
      || {
        let resolved = for_subdir("xslt").expect("resolve");
        assert!(resolved.is_dir(), "expected created dir at {:?}", resolved);
        assert!(resolved.ends_with("xslt"));
      },
    );
  }
}
