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
            "the archive entry '{name}' links to a file that does not resolve inside the source directory; not unpacked"
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
          "the archive entry '{name}' is a link to '{}' {LINK_NOT_MADE}; not unpacked",
          target.display()
        ),
      );
    }
  }
  normalize_unpacked_permissions(dest);
  Ok(())
}

/// Maximum bytes read from a zip symlink entry's target payload (mirrors
/// `zip`'s `MAX_SYMLINK_TARGET_LEN` / Linux `PATH_MAX`).
const MAX_ZIP_SYMLINK_TARGET_LEN: u64 = 4096;

/// Unpacks a `.zip` archive into `dest` with sandbox-safe permissions and
/// symlink containment, used by both `latexml_oxide` and `cortex_worker`:
/// - Every entry path is validated via `ZipFile::enclosed_name()` so `..` /
///   absolute paths never land outside `dest` (skipped with a counted Warning).
/// - Regular files and directories are written without inheriting archive exec
///   or read-only mode bits, and permissions are normalized at the end
///   (`a-x,u+rwX` on Unix; clearing `readonly` on non-Unix) so read-only
///   archive directories (`0555`) never leave temp entries behind on drop and
///   executable files (`0755`) lose their exec bits.
/// - Symlinks are handled in a second pass after all files/dirs exist:
///   in-archive symlinks stay symlinks (resolving to their target's text, via
///   `make_enclosed_link`), while symlinks whose target would escape `dest`
///   (e.g. `-> /etc/hostname`) are written as plain text files with a counted
///   Warning instead of failing the whole paper.
pub fn unpack_zip<R: std::io::Read + std::io::Seek>(
  mut archive: zip::ZipArchive<R>,
  dest: &std::path::Path,
) -> Result<(), zip::result::ZipError> {
  use std::io::Read as _;

  use latexml_core::common::error::warn_at_session_start;

  let mut links: Vec<(std::path::PathBuf, Vec<u8>, String)> = Vec::new();
  for i in 0..archive.len() {
    let mut entry = archive.by_index(i)?;
    let raw_name = entry
      .name()
      .map(|s| s.into_owned())
      .unwrap_or_else(|_| String::from_utf8_lossy(entry.name_raw()).into_owned());
    let Some(rel_path) = entry.enclosed_name() else {
      warn_at_session_start(
        "archive",
        "path",
        format!(
          "the archive entry '{raw_name}' would land outside the source directory; not unpacked"
        ),
      );
      continue;
    };
    if entry.is_symlink() {
      let mut target_bytes = Vec::new();
      entry
        .by_ref()
        .take(MAX_ZIP_SYMLINK_TARGET_LEN + 1)
        .read_to_end(&mut target_bytes)?;
      if target_bytes.len() as u64 > MAX_ZIP_SYMLINK_TARGET_LEN {
        warn_at_session_start(
          "archive",
          "link",
          format!("the archive entry '{raw_name}' has an oversized symlink target; not unpacked"),
        );
        continue;
      }
      links.push((rel_path, target_bytes, raw_name));
      continue;
    }
    let out_path = dest.join(&rel_path);
    if entry.is_dir() {
      std::fs::create_dir_all(&out_path)?;
    } else {
      if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
      }
      let mut out = std::fs::File::create(&out_path)?;
      std::io::copy(&mut entry, &mut out)?;
    }
  }

  if !links.is_empty() {
    let root = dest.canonicalize()?;
    for (rel_path, target_bytes, raw_name) in links {
      let target_path: Option<std::path::PathBuf> = {
        #[cfg(unix)]
        {
          use std::os::unix::ffi::OsStrExt as _;
          Some(std::path::PathBuf::from(std::ffi::OsStr::from_bytes(
            &target_bytes,
          )))
        }
        #[cfg(not(unix))]
        {
          std::str::from_utf8(&target_bytes)
            .ok()
            .map(std::path::PathBuf::from)
        }
      };

      let linked = target_path
        .as_deref()
        .is_some_and(|target| make_enclosed_link(&root, &rel_path, target));

      if !linked {
        warn_at_session_start(
          "archive",
          "link",
          format!(
            "the archive entry '{raw_name}' is a link to '{}' {LINK_NOT_MADE}; unpacked as a plain file",
            String::from_utf8_lossy(&target_bytes)
          ),
        );
        // Write the escaping symlink as a plain file inside `root` (verifying
        // its parent still canonicalizes inside `root`) so the paper still
        // converts rather than failing on a stray symlink.
        if let (Some(parent), Some(file_name)) = (rel_path.parent(), rel_path.file_name()) {
          let parent_dir = root.join(parent);
          std::fs::create_dir_all(&parent_dir)?;
          if let Ok(real_parent) = parent_dir.canonicalize()
            && real_parent.starts_with(&root)
          {
            let plain_path = real_parent.join(file_name);
            std::fs::write(&plain_path, &target_bytes)?;
          }
        }
      }
    }
  }

  normalize_unpacked_permissions(dest);
  Ok(())
}

/// Normalize permissions across an unpacked archive tree (`chmod -R a-x,u+rwX`
/// on Unix, clearing `readonly` on non-Unix): strips execute bits from regular
/// files and ensures owner read/write/traverse on directories so read-only
/// archive directories (`0555`) never leave temporary files behind on cleanup.
/// Walks real directories only (`file_type()`, never following symlinks) so
/// in-archive directory link cycles (`a -> .`, #924) never loop.
fn normalize_unpacked_permissions(dir: &std::path::Path) {
  fn fix_dir(path: &std::path::Path) {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt as _;
      if let Ok(meta) = std::fs::symlink_metadata(path) {
        let mode = (meta.permissions().mode() & !0o111) | 0o700;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
      }
    }
    #[cfg(not(unix))]
    {
      if let Ok(meta) = std::fs::symlink_metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
          perms.set_readonly(false);
          let _ = std::fs::set_permissions(path, perms);
        }
      }
    }
  }

  fn fix_file(path: &std::path::Path) {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt as _;
      if let Ok(meta) = std::fs::symlink_metadata(path) {
        let mode = (meta.permissions().mode() & !0o111) | 0o600;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
      }
    }
    #[cfg(not(unix))]
    {
      if let Ok(meta) = std::fs::symlink_metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
          perms.set_readonly(false);
          let _ = std::fs::set_permissions(path, perms);
        }
      }
    }
  }

  // Ensure the directory itself is owner-rwx BEFORE reading its entries (a
  // read-only tar/zip dir like `0555` or `0444` must be traversable first).
  fix_dir(dir);
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let Ok(ft) = entry.file_type() else {
      continue;
    };
    let path = entry.path();
    if ft.is_symlink() {
      continue;
    }
    if ft.is_dir() {
      normalize_unpacked_permissions(&path);
    } else if ft.is_file() {
      fix_file(&path);
    }
  }
}

/// Why an archive link was not made, for its counted Warning. On Unix a link is made when it resolves inside the
/// source directory, so a refused one is absolute, escapes it, climbs (`..`) past a component that does not exist yet,
/// or runs through a cycle; elsewhere [`make_enclosed_link`] makes none, and every link is reported.
#[cfg(unix)]
const LINK_NOT_MADE: &str = "that does not resolve inside the source directory";
#[cfg(not(unix))]
const LINK_NOT_MADE: &str = "and links are not made on this platform";

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

#[cfg(test)]
mod archive_unpack_tests {
  use std::io::{Cursor, Write as _};

  use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

  use super::*;

  fn build_zip(build: impl FnOnce(&mut ZipWriter<Cursor<Vec<u8>>>)) -> ZipArchive<Cursor<Vec<u8>>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    build(&mut writer);
    let cursor = writer.finish().expect("finish zip");
    ZipArchive::new(cursor).expect("open zip")
  }

  /// Synthetic zip covering read-only dirs (`0555`), executable files (`0755`),
  /// in-archive symlinks, escaping symlinks, path traversal (`..`), and a
  /// self-referential directory symlink cycle (`a -> .`, #924).
  #[test]
  fn unpack_zip_normalizes_permissions_and_contains_symlinks() {
    let archive = build_zip(|zw| {
      zw.add_directory("ro/", SimpleFileOptions::default().unix_permissions(0o555))
        .unwrap();
      zw.start_file(
        "ro/part.tex",
        SimpleFileOptions::default().unix_permissions(0o755),
      )
      .unwrap();
      zw.write_all(b"INCLUDED_PART").unwrap();

      zw.start_file(
        "main.tex",
        SimpleFileOptions::default().unix_permissions(0o444),
      )
      .unwrap();
      zw.write_all(b"\\documentclass{article}\\begin{document}\\input{alias}\\end{document}\n")
        .unwrap();

      // In-archive symlink: must resolve to target's text (on Unix).
      zw.add_symlink("alias.tex", "ro/part.tex", SimpleFileOptions::default())
        .unwrap();
      // Escaping symlink: must be unpacked as a plain file, not fail extraction.
      zw.add_symlink("escape.tex", "/etc/hostname", SimpleFileOptions::default())
        .unwrap();
      // Self-referential directory link (#924): must not cause recursion cycles.
      zw.add_symlink("loopdir", ".", SimpleFileOptions::default())
        .unwrap();
      // Path traversal entry: skipped via `enclosed_name`.
      zw.start_file("../outside.tex", SimpleFileOptions::default())
        .unwrap();
      zw.write_all(b"ESCAPED").unwrap();
    });

    let parent = tempfile::tempdir().expect("outer tempdir");
    let extract_dir = parent.path().join("work");
    std::fs::create_dir(&extract_dir).unwrap();

    unpack_zip(archive, &extract_dir).expect("unpack_zip must succeed");

    // Path traversal never escapes `extract_dir`.
    assert!(!parent.path().join("outside.tex").exists());

    // Read-only dir and exec-bit file permissions normalized (`a-x,u+rwX`).
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt as _;
      let ro_mode = std::fs::symlink_metadata(extract_dir.join("ro"))
        .unwrap()
        .permissions()
        .mode();
      assert_eq!(
        ro_mode & 0o700,
        0o700,
        "ro/ must be owner rwx: {ro_mode:#o}"
      );
      let part_mode = std::fs::symlink_metadata(extract_dir.join("ro/part.tex"))
        .unwrap()
        .permissions()
        .mode();
      assert_eq!(
        part_mode & 0o111,
        0,
        "ro/part.tex must have no exec bits: {part_mode:#o}"
      );
      assert_eq!(
        part_mode & 0o600,
        0o600,
        "ro/part.tex must be owner rw: {part_mode:#o}"
      );

      // In-archive symlink stays a symlink and resolves to target text.
      let alias_meta = std::fs::symlink_metadata(extract_dir.join("alias.tex")).unwrap();
      assert!(alias_meta.file_type().is_symlink());
      assert_eq!(
        std::fs::read_to_string(extract_dir.join("alias.tex")).unwrap(),
        "INCLUDED_PART"
      );
    }

    // Escaping symlink unpacked as a plain file containing target bytes, never a symlink.
    let esc_meta = std::fs::symlink_metadata(extract_dir.join("escape.tex")).unwrap();
    assert!(
      esc_meta.file_type().is_file() && !esc_meta.file_type().is_symlink(),
      "escaping symlink must be a plain file"
    );
    assert_eq!(
      std::fs::read_to_string(extract_dir.join("escape.tex")).unwrap(),
      "/etc/hostname"
    );

    // `find_main_tex` tolerates `loopdir -> .` without cycling and locates `main.tex`.
    let main = crate::main_tex::find_main_tex(&extract_dir).expect("find_main_tex");
    assert_eq!(main.file_name().and_then(|s| s.to_str()), Some("main.tex"));

    // Cleanup of read-only directory succeeds cleanly with zero leftovers.
    std::fs::remove_dir_all(&extract_dir).expect("remove_dir_all must succeed on normalized ro/");
    assert!(!extract_dir.exists());
  }
}
