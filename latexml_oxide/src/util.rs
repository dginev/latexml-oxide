/// Engine preset helpers used by both the production
/// `latexmlmath_oxide` binary and by the test harness. Always
/// compiled — no test-only dependencies.
pub mod preset;

/// Integration-test harness. Gated behind the `test-utils` feature
/// (default on) so the distribution build (`cargo build
/// --no-default-features --profile maxperf --bin latexml_oxide`)
/// drops `glob` from the dependency graph (audit DEP-02).
#[cfg(feature = "test-utils")]
#[macro_use]
pub mod test;

/// Unpacks a tar archive into `dest`. A source tree is untrusted input: a link in it could lead a read (`\input`) or a
/// Lua write outside the job (OXIDIZED_DESIGN_DIVERGENCES #1081). So a symbolic link is made, after every other entry,
/// only when its target, resolved component by component through the links already there, stays inside `dest` (zip
/// 9.7's `extract` rule, `zip/src/path.rs` `resolve_enclosed`); a hard link only to a file inside `dest` (`tar`'s
/// own check). Every other link, and every entry whose path would leave `dest`, is skipped with a counted Warning.
pub fn unpack_tar<R: std::io::Read>(
  mut archive: tar::Archive<R>,
  dest: &std::path::Path,
) -> std::io::Result<()> {
  use latexml_core::common::error::warn_at_session_start;
  let mut links = Vec::new();
  for entry in archive.entries()? {
    let mut entry = entry?;
    let name = entry
      .path()
      .map(|p| p.display().to_string())
      .unwrap_or_default();
    let kind = entry.header().entry_type();
    if kind.is_symlink() {
      match (entry.path(), entry.link_name()) {
        (Ok(path), Ok(Some(target))) => links.push((path.into_owned(), target.into_owned(), name)),
        _ => warn_at_session_start(
          "archive",
          "link",
          format!("the archive entry '{name}' is a link without a target; not unpacked"),
        ),
      }
      continue;
    }
    if kind.is_hard_link() {
      if entry.unpack_in(dest).ok() != Some(true) {
        warn_at_session_start(
          "archive",
          "link",
          format!(
            "the archive entry '{name}' links to a file outside the source directory; not unpacked"
          ),
        );
      }
      continue;
    }
    if !entry.unpack_in(dest)? {
      warn_at_session_start(
        "archive",
        "path",
        format!("the archive entry '{name}' would land outside the source directory; not unpacked"),
      );
    }
  }
  let root = dest.canonicalize()?;
  for (path, target, name) in links {
    if !make_enclosed_link(&root, &path, &target) {
      warn_at_session_start(
        "archive",
        "link",
        format!(
          "the archive entry '{name}' is a link to '{}', outside the source directory; not unpacked",
          target.display()
        ),
      );
    }
  }
  Ok(())
}

/// Makes the symbolic link `path` (relative to `root`) to `target` when both the link's directory and the target,
/// resolved physically one component at a time, stay inside `root`; false otherwise.
fn make_enclosed_link(
  root: &std::path::Path,
  path: &std::path::Path,
  target: &std::path::Path,
) -> bool {
  use std::path::Component;
  if !path
    .components()
    .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
  {
    return false;
  }
  let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) else {
    return false;
  };
  let parent = root.join(parent);
  if std::fs::create_dir_all(&parent).is_err() {
    return false;
  }
  let Ok(mut current) = parent.canonicalize() else {
    return false;
  };
  if !current.starts_with(root) {
    return false;
  }
  let link = current.join(file_name);
  // Once a component does not exist, a later `..` could mean another place once something is made there.
  let mut missing = false;
  for component in target.components() {
    match component {
      Component::CurDir => {},
      Component::ParentDir => {
        if missing || !current.pop() || !current.starts_with(root) {
          return false;
        }
      },
      Component::Normal(part) => {
        current.push(part);
        if missing {
          continue;
        }
        match std::fs::symlink_metadata(&current) {
          Ok(meta) if meta.file_type().is_symlink() => match current.canonicalize() {
            Ok(real) if real.starts_with(root) => current = real,
            _ => return false,
          },
          Ok(_) => {},
          Err(_) => missing = true,
        }
      },
      Component::RootDir | Component::Prefix(_) => return false,
    }
  }
  if !current.starts_with(root) {
    return false;
  }
  #[cfg(unix)]
  {
    std::os::unix::fs::symlink(target, &link).is_ok()
  }
  #[cfg(not(unix))]
  {
    let _ = (link, target);
    false
  }
}
