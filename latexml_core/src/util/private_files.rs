//! Files trusted only when this user wrote them. A format dump is loaded as the format, and the
//! caches holding dumps live in shared places (`$TMPDIR`), where another local user could put a
//! file, or a directory, there first.

use std::path::Path;

/// The process's effective user id.
#[cfg(unix)]
pub fn user_id() -> u32 {
  // SAFETY: `geteuid` reads the process's effective user id; it cannot fail.
  unsafe { libc::geteuid() }
}
/// No user ids to compare off Unix.
#[cfg(not(unix))]
pub fn user_id() -> u32 { 0 }

/// Whether `path` is this user's own and no one else can write to it — not a symbolic link, whose
/// mode lets everyone write.
#[cfg(unix)]
pub fn private_to_user(path: &Path) -> bool {
  use std::os::unix::fs::MetadataExt;
  std::fs::symlink_metadata(path)
    .is_ok_and(|meta| meta.uid() == user_id() && meta.mode() & 0o022 == 0)
}
/// No ownership to check off Unix.
#[cfg(not(unix))]
pub fn private_to_user(_path: &Path) -> bool { true }

/// Whether `dir` is private to this user inside a directory this user or the superuser owns, so no
/// one else can have put it, or what it holds, there.
#[cfg(unix)]
pub fn trusted_dir(dir: &Path) -> bool {
  use std::os::unix::fs::MetadataExt;
  private_to_user(dir)
    && dir
      .parent()
      .and_then(|parent| std::fs::metadata(parent).ok())
      .is_some_and(|meta| meta.uid() == user_id() || meta.uid() == 0)
}
/// No ownership to check off Unix.
#[cfg(not(unix))]
pub fn trusted_dir(_dir: &Path) -> bool { true }

/// Creates `dir` for this user alone, or makes it private if this user owns it, and requires it to
/// be a [`trusted_dir`].
pub fn private_dir(dir: &Path) -> Result<(), String> {
  std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let meta =
      std::fs::symlink_metadata(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    if meta.uid() != user_id() || meta.file_type().is_symlink() {
      return Err(format!("{} belongs to another user", dir.display()));
    }
    if meta.mode() & 0o022 != 0 {
      std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("cannot make {} private: {e}", dir.display()))?;
    }
  }
  if trusted_dir(dir) {
    Ok(())
  } else {
    Err(format!(
      "{} is inside another user's directory",
      dir.display()
    ))
  }
}

/// Makes the file at `path` readable by all and writable by this user alone (0644), as a file
/// [`private_to_user`] accepts must be whatever the umask.
pub fn publish_read_only(path: &Path) -> std::io::Result<()> {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))?;
  }
  #[cfg(not(unix))]
  let _ = path;
  Ok(())
}

#[cfg(all(test, unix))]
mod tests {
  use std::os::unix::fs::PermissionsExt;

  use super::*;

  #[test]
  fn only_this_users_files_no_one_else_can_write_are_trusted() {
    let root = std::env::temp_dir().join(format!(
      "latexml-private-files-{}-{:?}",
      std::process::id(),
      std::thread::current().id()
    ));
    let dir = root.join("cache");
    private_dir(&dir).expect("create a private directory");
    assert_eq!(
      std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777 & 0o022,
      0
    );
    assert!(trusted_dir(&dir));

    let file = dir.join("dump.txt");
    std::fs::write(&file, "dump").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert!(
      !private_to_user(&file),
      "a file everyone can write is trusted"
    );
    publish_read_only(&file).unwrap();
    assert!(private_to_user(&file));

    let link = dir.join("link.txt");
    std::os::unix::fs::symlink(&file, &link).unwrap();
    assert!(!private_to_user(&link), "a symbolic link is trusted");

    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o777)).unwrap();
    assert!(
      !trusted_dir(&dir),
      "a directory everyone can write is trusted"
    );
    private_dir(&dir).expect("make the directory private again");
    assert!(trusted_dir(&dir));
    std::fs::remove_dir_all(&root).unwrap();
  }
}
