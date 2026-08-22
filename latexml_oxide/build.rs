//! Build script for the top-level `latexml` crate.
//!
//! Its only job is a developer convenience: point this checkout's git at the
//! repo's tracked hooks (`.githooks/`) so every contributor gets the pre-push
//! lint gate (`tools/lint.sh`, the same script CI runs) automatically on their
//! first `cargo build`/`test` — no manual `git config core.hooksPath` step
//! (which is easy to forget, and its absence is exactly how unformatted /
//! clippy-dirty branches reach CI).
//!
//! It is a strict no-op outside a source git checkout — packaged crates,
//! `cargo install`, and release tarballs have no `.git`, so distribution builds
//! are unaffected — and it never fails the build (every git call is best-effort).

use std::{
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  install_git_hooks();
  probe_texlive();
  emit_git_revision();
}

/// Embed the source git revision as `env!("LATEXML_GIT_SHA")`, so every
/// conversion log can name the exact binary that produced it (Perl's
/// `$LaTeXML::Version::REVISION`, filled by `make` — see `identity.rs`).
///
/// Resolution order, mirroring Perl's make-filled revision:
///   1. An explicit `LATEXML_GIT_SHA` in the build environment wins — the
///      release workflow injects the tag's sha for tarball / crates.io builds
///      that have no `.git`.
///   2. Otherwise, best-effort `git rev-parse --short HEAD` in the source tree.
///   3. Failing both (no git, no `.git`), `"unknown"`.
///
/// **Freshness model — deliberately no VCS `rerun-if-changed`.** This script is
/// NOT told to re-run when `.git/HEAD` or a ref moves. Watching those would
/// re-run it (and recompile the crate + its reverse-deps) on every branch
/// switch / commit / pull — a routine-git-activity recompile cascade the
/// existing hook-install logic goes out of its way to avoid (#528). So a *dev*
/// binary reports the revision as of the last time the script ran (a clean
/// build, a `build.rs` edit, or a `LATEXML_GIT_SHA` change); it can lag HEAD
/// after a pull. Force an exact value with
/// `LATEXML_GIT_SHA=$(git rev-parse --short HEAD) cargo build`. Release builds
/// are always accurate: `release-dumps.yml` builds fresh and/or sets the env.
///
/// Best-effort throughout: a distribution build with neither the env override
/// nor a `.git` simply embeds `"unknown"` and never fails the build.
fn emit_git_revision() {
  println!("cargo:rerun-if-env-changed=LATEXML_GIT_SHA");

  // 1. Explicit override (release workflow, packaged builds without `.git`).
  if let Ok(sha) = std::env::var("LATEXML_GIT_SHA") {
    let sha = sha.trim();
    if !sha.is_empty() {
      println!("cargo:rustc-env=LATEXML_GIT_SHA={sha}");
      return;
    }
  }

  // 2. Best-effort probe of the source checkout (repo root = manifest's parent).
  let value = std::env::var("CARGO_MANIFEST_DIR")
    .ok()
    .and_then(|d| Path::new(&d).parent().map(Path::to_path_buf))
    .as_deref()
    .and_then(git_short_sha)
    .unwrap_or_else(|| "unknown".to_string());
  println!("cargo:rustc-env=LATEXML_GIT_SHA={value}");
}

/// `git rev-parse --short HEAD` in `root`; `None` outside a git checkout.
fn git_short_sha(root: &Path) -> Option<String> {
  let head = Command::new("git")
    .current_dir(root)
    .args(["rev-parse", "--short", "HEAD"])
    .output()
    .ok()?;
  if !head.status.success() {
    return None;
  }
  let sha = String::from_utf8(head.stdout).ok()?.trim().to_string();
  if sha.is_empty() { None } else { Some(sha) }
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
  // hot incremental-rebuild path once core.hooksPath is set). Guard the hook path
  // on existence: `../.githooks/pre-push` is absent from a crates.io tarball, and
  // a `rerun-if-changed` on a non-existent path makes Cargo re-run this script —
  // rebuilding the top crate — on every invocation, regardless of the no-op body
  // below (#528).
  println!("cargo:rerun-if-changed=build.rs");
  if Path::new("../.githooks/pre-push").exists() {
    println!("cargo:rerun-if-changed=../.githooks/pre-push");
  }

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

  let current = Command::new("git")
    .current_dir(repo_root)
    .args(["config", "--local", "--get", "core.hooksPath"])
    .output();
  let current_raw = current
    .as_ref()
    .ok()
    .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
    .unwrap_or_default();

  // Resolve a configured hooksPath the way git does — relative to the repo root
  // — so we compare DIRECTORIES, not strings.
  let resolve = |value: &str| -> Option<PathBuf> {
    let trimmed = Path::new(value.trim_end_matches('/'));
    let abs = if trimmed.is_absolute() {
      trimmed.to_path_buf()
    } else {
      repo_root.join(trimmed)
    };
    std::fs::canonicalize(abs).ok()
  };
  let same_dir = |a: &str, b: &Path| -> bool {
    matches!((resolve(a), std::fs::canonicalize(b).ok()), (Some(x), Some(y)) if x == y)
  };

  // Idempotent: leave it alone if it already points at our hooks. This compares
  // resolved paths because a string compare against ".githooks" missed both
  // ".githooks/" — the trailing-slash form CLAUDE.md tells contributors to use —
  // and any absolute path to the same directory. Those fell through to the
  // "custom hooksPath" branch below, so the gate stayed off while this script
  // believed the user had deliberately chosen that.
  if !current_raw.is_empty() && same_dir(&current_raw, &repo_root.join(".githooks")) {
    return;
  }

  // A hooksPath pointing at git's OWN default (<repo>/.git/hooks) is not a
  // deliberate choice — it is where git looks anyway — so treat it as unset and
  // wire up the gate. Without this, a checkout whose hooksPath had been set to
  // the default kept the gate silently disabled forever: the notice below goes
  // to stderr, which cargo shows only under `-vv` or on failure. That is how
  // three unformatted/doc-broken pushes reached CI on 2026-07-24.
  //
  // Unless the directory actually carries a hook, that is — everything git ships
  // there is a `.sample`. If a real one is present, it is someone's, so respect it.
  let default_hooks = repo_root.join(".git/hooks");
  let default_carries_hooks = std::fs::read_dir(&default_hooks)
    .map(|entries| {
      entries
        .flatten()
        .any(|e| e.path().is_file() && !e.file_name().to_string_lossy().ends_with(".sample"))
    })
    .unwrap_or(false);
  let redundant_default = same_dir(&current_raw, &default_hooks) && !default_carries_hooks;

  // Respect a deliberately-set custom hooksPath rather than clobbering it; just
  // nudge. Otherwise (the common unset case) wire up the gate.
  let already_custom = !current_raw.is_empty() && !redundant_default;
  if already_custom {
    // Print to stderr, NOT `cargo:warning=` — cargo replays build-script
    // warnings on every build until the script re-runs, which would turn a
    // one-time notice into perpetual noise. Build-script stderr is surfaced
    // only under `cargo build -vv` or on failure.
    eprintln!(
      "latexml: git core.hooksPath is set to a custom value; the pre-push lint \
       gate lives in .githooks/ — point core.hooksPath there to enable it."
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
      "latexml: enabled the pre-push lint gate \
       (set git core.hooksPath=.githooks). Bypass once with `git push --no-verify`."
    );
  }
}
