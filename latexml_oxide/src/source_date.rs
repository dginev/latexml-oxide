//! The date of a paper's source: the newest modification time among the TeX source files of the bundle it came in
//! (`Config::source_date_epoch`), the clock ar5iv's archival preload gives the job so `\today` prints the paper's own
//! date — the one arXiv's PDF shows — and never the conversion's (user ruling 2026-10-10).
//!
//! Only the files the paper is typeset from count: the `.tex` sources, the macro files shipped beside them and the
//! `.bbl`/`.bib` (a `.bbl` is written by the author's last compile), plus the detected main file whatever its
//! extension. Figures, compile by-products (`.aux`, `.log`, `.synctex`) and directories do not: a figure regenerated
//! after the paper, or a directory the packer touched, is not the paper's date. A time at or before the DOS epoch
//! (1980-01-01, a zip entry's "no time" — arXiv's single-file submissions arrive so) is no date.
//!
//! An archive's entries are read for their recorded times, not the extracted files' (extraction stamps its own
//! clock): a zip entry's Info-ZIP extended timestamp (`0x5455`, UTC) when it has one, else its DOS date and time read
//! as UTC; a tar entry's `mtime`. The result is deterministic per archive. A directory or single-file input reads the
//! filesystem's mtimes instead, which a checkout or a copy resets: deterministic only while the files are untouched.
//!
//! Known limits: an entry dated in the future still wins (no clock is consulted, so the result stays deterministic); a
//! single-file input counts only that file, not the files it `\input`s; a directory walk skips version-control
//! directories (`.git`, `.svn`, `.hg`, `CVS`); a `.tar.gz` is decompressed a second time for its dates.

use std::{fs::File, path::Path};

use chrono::NaiveDate;

/// The DOS epoch, 1980-01-01T00:00:00Z: a zip entry at (or a time before) it carries no date.
const DOS_EPOCH: i64 = 315_532_800;

/// Whether a source file named `name` is one the paper is typeset from (case-insensitive extension).
fn is_tex_source(name: &str) -> bool {
  const EXTENSIONS: &[&str] = &[
    "tex", "ltx", "latex", "sty", "cls", "clo", "def", "cfg", "fd", "bbl", "bib",
  ];
  Path::new(name)
    .extension()
    .and_then(|ext| ext.to_str())
    .is_some_and(|ext| EXTENSIONS.iter().any(|e| ext.eq_ignore_ascii_case(e)))
}

/// `epoch` when it is a date (after the DOS epoch).
fn dated(epoch: i64) -> Option<i64> { (epoch > DOS_EPOCH).then_some(epoch) }

/// A zip entry's recorded modification time: its extended timestamp (UTC) when present, else its DOS date and time
/// read as UTC.
fn zip_entry_epoch<R: std::io::Read>(entry: &zip::read::ZipFile<'_, R>) -> Option<i64> {
  let extended = entry.extra_data_fields().find_map(|field| match field {
    zip::ExtraField::ExtendedTimestamp(ts) => ts.mod_time(),
    _ => None,
  });
  if let Some(epoch) = extended {
    return dated(i64::from(epoch));
  }
  let dos = entry.last_modified()?;
  let utc = NaiveDate::from_ymd_opt(
    i32::from(dos.year()),
    u32::from(dos.month()),
    u32::from(dos.day()),
  )?
  .and_hms_opt(
    u32::from(dos.hour()),
    u32::from(dos.minute()),
    u32::from(dos.second()),
  )?
  .and_utc();
  dated(utc.timestamp())
}

/// The source date of an archive (`.zip`, `.tar.gz`/`.tgz`, `.tar`): the newest recorded time among its TeX source
/// entries and `main` (the detected main file's path inside the archive). `None` when no such entry carries a date, or
/// the archive cannot be read.
pub fn of_archive(archive: &Path, main: Option<&str>) -> Option<i64> {
  let counts =
    |name: &str| is_tex_source(name) || main.is_some_and(|m| name.trim_start_matches("./") == m);
  let name = archive.to_string_lossy();
  if name.ends_with(".zip") {
    let mut zip = zip::ZipArchive::new(File::open(archive).ok()?).ok()?;
    let mut newest = None;
    for i in 0..zip.len() {
      let Ok(entry) = zip.by_index_raw(i) else {
        continue;
      };
      if entry.is_file() && counts(entry.name()) {
        newest = newest.max(zip_entry_epoch(&entry));
      }
    }
    newest
  } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
    tar_newest(
      tar::Archive::new(flate2::read::GzDecoder::new(File::open(archive).ok()?)),
      &counts,
    )
  } else if name.ends_with(".tar") {
    tar_newest(tar::Archive::new(File::open(archive).ok()?), &counts)
  } else {
    None
  }
}

fn tar_newest<R: std::io::Read>(
  mut tar: tar::Archive<R>,
  counts: &dyn Fn(&str) -> bool,
) -> Option<i64> {
  let mut newest = None;
  for entry in tar.entries().ok()?.flatten() {
    let header = entry.header();
    let is_file = header.entry_type().is_file();
    let name = entry.path().ok().map(|p| p.to_string_lossy().into_owned());
    if is_file && name.as_deref().is_some_and(counts) {
      let epoch = header.mtime().ok().and_then(|t| i64::try_from(t).ok());
      newest = newest.max(epoch.and_then(dated));
    }
  }
  newest
}

/// A file's modification time as Unix seconds, when it is a date.
fn file_epoch(path: &Path) -> Option<i64> {
  let modified = path.metadata().ok()?.modified().ok()?;
  let secs = modified
    .duration_since(std::time::UNIX_EPOCH)
    .ok()?
    .as_secs();
  dated(i64::try_from(secs).ok()?)
}

/// The source date of a directory input: the newest modification time among the TeX source files beneath `dir` and
/// `main`.
pub fn of_directory(dir: &Path, main: Option<&Path>) -> Option<i64> {
  fn walk(dir: &Path, newest: &mut Option<i64>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
      return;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      let Ok(kind) = entry.file_type() else {
        continue;
      };
      if kind.is_dir() {
        // A version-control directory holds no source the paper is typeset from.
        if !matches!(
          entry.file_name().to_str(),
          Some(".git" | ".svn" | ".hg" | "CVS")
        ) {
          walk(&path, newest);
        }
      } else if kind.is_file() && is_tex_source(&entry.file_name().to_string_lossy()) {
        *newest = (*newest).max(file_epoch(&path));
      }
    }
  }
  let mut newest = main.and_then(file_epoch);
  walk(dir, &mut newest);
  newest
}

/// The source date of a single-file input: its modification time.
pub fn of_file(file: &Path) -> Option<i64> { file_epoch(file) }

#[cfg(test)]
mod tests {
  use std::io::Write;

  use super::*;

  /// A zip of `(name, DOS time)` entries, none with an extended timestamp.
  fn zip_with(entries: &[(&str, zip::DateTime)]) -> tempfile::NamedTempFile {
    let file = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
    for (name, time) in entries {
      let options = zip::write::SimpleFileOptions::default().last_modified_time(*time);
      zip.start_file(*name, options).unwrap();
      zip.write_all(b"x").unwrap();
    }
    zip.finish().unwrap();
    file
  }

  fn dos(y: u16, mo: u8, d: u8, h: u8, mi: u8) -> zip::DateTime {
    zip::DateTime::from_date_and_time(y, mo, d, h, mi, 0).unwrap()
  }

  #[test]
  fn newest_tex_source_entry_wins_over_figures() {
    let zip = zip_with(&[
      ("paper.tex", dos(2018, 1, 18, 10, 30)),
      ("ptapap.cls", dos(2017, 12, 1, 0, 0)),
      ("paper.bbl", dos(2018, 1, 18, 11, 5)),
      ("fig1.pdf", dos(2019, 6, 1, 0, 0)),
    ]);
    // 2018-01-18T11:05:00Z, the .bbl's time; the later figure does not count.
    assert_eq!(
      of_archive(zip.path(), Some("paper.tex")),
      Some(1_516_273_500)
    );
  }

  /// arxmliv's zips record each entry twice: an Info-ZIP extended timestamp (`0x5455`, UTC) and a DOS time in the
  /// packer's local zone (UTC+2: 2105.04321's `rlrd_rev1.tex`, DOS 17:46 against UT 15:46Z). The UT time wins, even
  /// across a day boundary.
  #[test]
  fn extended_timestamp_wins_over_the_dos_time() {
    let file = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    let mut zip = zip::ZipWriter::new(file.reopen().unwrap());
    // 2018-01-18T22:30:00Z, written by a packer at UTC+2 as a DOS 2018-01-19 00:30.
    let ut: u32 = 1_516_314_600;
    let mut field = vec![1u8]; // flags: modification time present
    field.extend_from_slice(&ut.to_le_bytes());
    let mut options =
      zip::write::FullFileOptions::default().last_modified_time(dos(2018, 1, 19, 0, 30));
    options.add_extra_data(0x5455, field, false).unwrap();
    zip.start_file("paper.tex", options).unwrap();
    zip.write_all(b"x").unwrap();
    zip.finish().unwrap();
    assert_eq!(
      of_archive(file.path(), Some("paper.tex")),
      Some(1_516_314_600)
    );
  }

  /// A tar entry's `mtime`, among its TeX sources only.
  #[test]
  fn tar_entries_date_by_their_mtime() {
    let file = tempfile::Builder::new().suffix(".tar").tempfile().unwrap();
    let mut tar = tar::Builder::new(file.reopen().unwrap());
    for (name, mtime) in [("paper.tex", 983_682_360u64), ("fig.eps", 1_516_314_600)] {
      let mut header = tar::Header::new_gnu();
      header.set_size(1);
      header.set_mtime(mtime);
      header.set_mode(0o644);
      header.set_entry_type(tar::EntryType::Regular);
      tar.append_data(&mut header, name, &b"x"[..]).unwrap();
    }
    tar.finish().unwrap();
    assert_eq!(
      of_archive(file.path(), Some("paper.tex")),
      Some(983_682_360)
    );
  }

  #[test]
  fn undated_entries_give_no_date() {
    let zip = zip_with(&[("paper.tex", zip::DateTime::default())]);
    assert_eq!(of_archive(zip.path(), Some("paper.tex")), None);
  }

  #[test]
  fn the_main_file_counts_whatever_its_extension() {
    let zip = zip_with(&[("paper.txt", dos(2001, 3, 4, 5, 6))]);
    assert_eq!(of_archive(zip.path(), Some("paper.txt")), Some(983_682_360));
    assert_eq!(of_archive(zip.path(), None), None);
  }
}
