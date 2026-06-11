//! Build script for the top-level `latexml` crate.
//!
//! Its only job is a developer convenience: point this checkout's git at the
//! repo's tracked hooks (`.githooks/`) so every contributor gets the
//! fmt + clippy pre-push gate automatically on their first `cargo build`/`test`
//! — no manual `git config core.hooksPath` step (which is easy to forget, and
//! its absence is exactly how unformatted / clippy-dirty branches reach CI).
//!
//! It is a strict no-op outside a source git checkout — packaged crates,
//! `cargo install`, and release tarballs have no `.git`, so distribution builds
//! are unaffected — and it never fails the build (every git call is best-effort).

use std::{path::Path, process::Command};

fn main() { install_git_hooks(); }

fn install_git_hooks() {
  // Re-run only when this script or the hook itself changes (keeps it off the
  // hot incremental-rebuild path once core.hooksPath is set).
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=../.githooks/pre-push");

  // CARGO_MANIFEST_DIR is <repo>/latexml_oxide; the repo root is its parent.
  let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
    return;
  };
  let Some(repo_root) = Path::new(&manifest_dir).parent() else {
    return;
  };

  // Only act inside the source repo: require both the git work tree and the
  // tracked hook. Distribution / packaged / crates.io builds have neither.
  if !repo_root.join(".git").exists() || !repo_root.join(".githooks/pre-push").exists() {
    return;
  }

  // Idempotent: leave it alone if it already points at our hooks.
  let current = Command::new("git")
    .current_dir(repo_root)
    .args(["config", "--local", "--get", "core.hooksPath"])
    .output();
  if let Ok(out) = &current
    && String::from_utf8_lossy(&out.stdout).trim() == ".githooks"
  {
    return;
  }

  // Respect a deliberately-set custom hooksPath rather than clobbering it; just
  // nudge. Otherwise (the common unset case) wire up the gate.
  let already_custom = matches!(&current, Ok(out) if !out.stdout.is_empty());
  if already_custom {
    // Print to stderr, NOT `cargo:warning=` — cargo replays build-script
    // warnings on every build until the script re-runs, which would turn a
    // one-time notice into perpetual noise. Build-script stderr is surfaced
    // only under `cargo build -vv` or on failure.
    eprintln!(
      "latexml: git core.hooksPath is set to a custom value; the fmt+clippy \
       pre-push gate lives in .githooks/ — point core.hooksPath there to enable it."
    );
    return;
  }

  let set = Command::new("git")
    .current_dir(repo_root)
    .args(["config", "--local", "core.hooksPath", ".githooks"])
    .status();
  if matches!(set, Ok(s) if s.success()) {
    // stderr, not `cargo:warning=` (see the note above re: per-build replay).
    eprintln!(
      "latexml: enabled the fmt+clippy pre-push gate \
       (set git core.hooksPath=.githooks). Bypass once with `git push --no-verify`."
    );
  }
}
