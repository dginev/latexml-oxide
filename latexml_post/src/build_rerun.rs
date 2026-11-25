//! Which paths the bundled-`ar5iv.css` embed step tracks with `cargo:rerun-if-changed`. Shared by `build.rs` (pulled
//! in with `#[path] mod`) and the unit tests (`#[cfg(test)] mod build_rerun` in `lib.rs`); pure and std-only.
//!
//! The rule of `latexml_engine/src/build_rerun.rs` (#528, #676): NEVER return a path that does not exist (Cargo
//! rebuilds on every build for a missing one), yet see the file appear and disappear, so a warm cache never keeps a
//! stale embed. The committed `resources/` sees the directory's creation; the directory, its files coming and going.
//! It is that file's twin, not a shared copy: a file shared across crates is not packaged for crates.io.

use std::path::{Path, PathBuf};

/// `rerun-if-changed` paths, given whether we build inside the workspace and which of `existing` are on disk.
/// Outside a workspace (a crates.io unpack, docs.rs) there is nothing to embed from, so nothing is tracked.
pub fn ar5iv_css_rerun_paths(
  resources_parent: &Path,
  existing: &[&Path],
  in_workspace: bool,
) -> Vec<PathBuf> {
  if !in_workspace {
    return Vec::new();
  }
  std::iter::once(resources_parent)
    .chain(existing.iter().copied().filter(|p| p.exists()))
    .map(Path::to_path_buf)
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tracks_only_what_exists() {
    let root = tempfile::tempdir().unwrap();
    let res = root.path().join("resources");
    let dir = res.join("ar5iv-css");
    let css = dir.join("ar5iv.min.css");
    std::fs::create_dir(&res).unwrap();
    let paths = || ar5iv_css_rerun_paths(&res, &[&dir, &css], true);
    // Absent (dev, CI): the committed parent only, never the missing directory (#676).
    assert_eq!(paths(), vec![res.clone()]);
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(&css, "x").unwrap();
    // Present (a release): the directory catches the file's removal, the file its replacement.
    assert_eq!(paths(), vec![res.clone(), dir.clone(), css.clone()]);
    std::fs::remove_file(&css).unwrap();
    assert_eq!(paths(), vec![res.clone(), dir.clone()]);
    // A crates.io unpack tracks nothing (#528).
    assert!(ar5iv_css_rerun_paths(&res, &[&dir, &css], false).is_empty());
  }
}
