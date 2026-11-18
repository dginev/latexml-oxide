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

/// Unpacks a tar archive's files and directories into `dest`, skipping (with a warning on stderr) every symbolic or
/// hard link and every entry whose path would leave `dest`: a source tree is untrusted input, and a link in it could
/// lead a read (`\input`) or a Lua write outside the job (OXIDIZED_DESIGN_DIVERGENCES #1081). The zip path needs no
/// such pass: `zip`'s extraction creates a link only when its target resolves inside the destination.
pub fn unpack_tar<R: std::io::Read>(
  mut archive: tar::Archive<R>,
  dest: &std::path::Path,
) -> std::io::Result<()> {
  for entry in archive.entries()? {
    let mut entry = entry?;
    let name = entry
      .path()
      .map(|p| p.display().to_string())
      .unwrap_or_default();
    let kind = entry.header().entry_type();
    if kind.is_symlink() || kind.is_hard_link() {
      eprintln!("Warning:archive:link the archive entry '{name}' is a link; not unpacked");
      continue;
    }
    if !entry.unpack_in(dest)? {
      eprintln!(
        "Warning:archive:path the archive entry '{name}' would land outside the source directory; not unpacked"
      );
    }
  }
  Ok(())
}
