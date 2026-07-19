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

fn main() {
  install_git_hooks();
  probe_texlive();
}

/// Emit `cfg(building_with_texlive)` when a TeX installation is usable **on the
/// machine running this build**, so tests that genuinely need one can be gated:
///
/// ```ignore
/// #[cfg_attr(not(building_with_texlive), ignore = "requires a TeX Live installation")]
/// ```
///
/// **For tests only — never for shipped behavior.** The flag describes the
/// BUILD host, not the machine that will eventually run the binary. Gating any
/// runtime path on it would bake the builder's TeX state into every user's
/// binary, so a release built on a TeX-equipped machine would misbehave on a
/// user's without one (and vice versa). Runtime must keep asking the actual
/// host, which is what the kpathsea backend selection does.
///
/// A host TeX tree is OPTIONAL for latexml-oxide — bindings and dumps are
/// embedded — so such tests must not fail where it is absent. They must not
/// silently *pass* either: an early `return` inside the test body reports green
/// while asserting nothing. `ignore` keeps the skip visible in the summary.
///
/// The probe resolves `cmr10.tfm`, present in every TeX distribution and the
/// same sentinel the runtime backend selection uses. Best-effort: any failure
/// simply leaves the cfg unset, and the gated tests are skipped rather than run
/// against a tree that isn't there.
fn probe_texlive() {
  println!("cargo:rustc-check-cfg=cfg(building_with_texlive)");
  // A TeX install/removal normally moves PATH; re-probe when it does.
  println!("cargo:rerun-if-env-changed=PATH");
  println!("cargo:rerun-if-env-changed=KPSEWHICH");

  let kpsewhich = std::env::var("KPSEWHICH").unwrap_or_else(|_| "kpsewhich".to_string());
  let found = Command::new(kpsewhich)
    .arg("cmr10.tfm")
    .output()
    .is_ok_and(|out| out.status.success() && !out.stdout.is_empty());
  if found {
    println!("cargo:rustc-cfg=building_with_texlive");
  }
}

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
