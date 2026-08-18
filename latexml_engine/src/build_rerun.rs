//! Which paths the versioned-dump embed step tracks with
//! `cargo:rerun-if-changed`. Shared by `build.rs` (pulled in with `#[path]
//! mod`) and the test suite (`#[cfg(test)] mod build_rerun` in `lib.rs`); pure
//! and std-only so the build-script crate compiles it with no dependency.
//!
//! **Invariant (#528, #676): NEVER return a path that does not exist.** Cargo
//! treats a `rerun-if-changed` on a missing path as perpetually dirty and
//! rebuilds the crate on every invocation — the #676 symptom on a fresh
//! checkout, where the committed `resources/` tree is present but the
//! gitignored, `tools/make_formats.sh`-generated `resources/dumps/` is not.

use std::path::{Path, PathBuf};

/// `rerun-if-changed` paths for the dump embed, given whether we build inside
/// the source workspace and whether the dumps dir currently exists on disk.
///
/// * `resources_parent` — the committed `resources/` dir (present in every
///   checkout). Tracking it lets the *first-ever* creation of `dumps/`
///   re-trigger the embed: adding that subdir bumps `resources/`'s mtime, which
///   Cargo notices (#528's "a first-ever generation is seen" property, but
///   without pointing at the not-yet-existing subdir).
/// * `dumps_dir` — tracked **only when `dumps_exists`**, so new dump files (a
///   new TL year written into an already-present dir — the deploy Docker
///   persistent-`target/` cache case #528 called out) are seen, while an absent
///   dir is never tracked (#676).
///
/// Outside a workspace (a crates.io tarball unpack) there is no `resources/`
/// tree to embed from, so nothing is tracked — a missing-path `rerun-if-changed`
/// there would rebuild the crate on every registry build (the original #528).
pub fn dump_rerun_paths(
  resources_parent: &Path,
  dumps_dir: &Path,
  in_workspace: bool,
  dumps_exists: bool,
) -> Vec<PathBuf> {
  let mut paths = Vec::new();
  if in_workspace {
    // Committed → always exists in a checkout; safe to track unconditionally.
    paths.push(resources_parent.to_path_buf());
    if dumps_exists {
      paths.push(dumps_dir.to_path_buf());
    }
  }
  paths
}

#[cfg(test)]
mod tests {
  use super::*;

  const RES: &str = "/ws/resources";
  const DUMPS: &str = "/ws/resources/dumps";

  #[test]
  fn fresh_checkout_tracks_parent_never_the_absent_dumps_dir() {
    // Workspace present, dumps/ not yet generated: the #676 case. Tracking the
    // absent dumps dir here is exactly what made Cargo rebuild on every build.
    let paths = dump_rerun_paths(Path::new(RES), Path::new(DUMPS), true, false);
    assert_eq!(paths, vec![PathBuf::from(RES)]);
    assert!(
      !paths.contains(&PathBuf::from(DUMPS)),
      "must not emit rerun-if-changed on the absent dumps dir (#676 always-dirty rebuild)"
    );
  }

  #[test]
  fn generated_dumps_dir_is_also_tracked() {
    // After tools/make_formats.sh: dumps/ now exists, so track it directly to
    // catch new dump files landing in it (#528).
    let paths = dump_rerun_paths(Path::new(RES), Path::new(DUMPS), true, true);
    assert_eq!(paths, vec![PathBuf::from(RES), PathBuf::from(DUMPS)]);
  }

  #[test]
  fn crates_io_unpack_tracks_nothing() {
    // No workspace one level up → no resources tree to embed from (original #528).
    assert!(dump_rerun_paths(Path::new(RES), Path::new(DUMPS), false, false).is_empty());
    assert!(dump_rerun_paths(Path::new(RES), Path::new(DUMPS), false, true).is_empty());
  }
}
