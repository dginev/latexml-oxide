//! Output bundling — port of `LaTeXML::Post::Pack`.
//!
//! Separates final-bundle concerns from the post-processing pipeline.
//! Both `latexml_oxide` and `cortex_worker` previously inlined their own
//! `pack_output_zip` / `pack_output_zip_with_resources` + `add_dir_to_zip`
//! copies; this module is the single source of truth so the two binaries
//! produce byte-identical bundle layouts.
//!
//! Bundle layout (zip):
//! ```text
//! <html_filename>            — post-processed HTML
//! <resource_dir>/...         — every non-`.html` file under the staging
//!                               dir (Graphics-converted PNG/SVG, CSS, …),
//!                               preserving subdirectories.
//! <log_filename>             — log text, if `log_filename` is set and
//!                               `log` is non-empty.
//! status                     — single-line status string.
//! telemetry.json             — single-line JSON per-job telemetry, only
//!                               written when `telemetry_json` is set
//!                               (cortex_worker canvas runs).
//! ```

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Write-buffer size for the zip output. 64 KiB matches the typical
/// compressed-block size from miniz_oxide/flate2 on HTML+image
/// content; smaller buffers (8 KiB) cause one syscall per block,
/// larger (1 MiB) waste RSS without improving throughput. Measured on
/// 1910.01256 (3 PNG + 2 SVG + 2 CSS + HTML = 1.2 MB zip): unbuffered
/// → ~70 write() syscalls; 64 KiB buffer → ~12.
const ZIP_WRITE_BUF: usize = 64 * 1024;

/// Options for [`pack_archive`].
pub struct PackOptions<'a> {
  /// Destination zip path.
  pub zip_path:       &'a str,
  /// Name to use for the HTML entry inside the zip (e.g. `paper.html`).
  /// Conventionally `<stem>.html` where `stem` is the source TeX name.
  pub html_filename:  &'a str,
  /// Post-processed HTML content.
  pub html:           &'a str,
  /// Name for the log entry; pass `None` to skip writing a log entry.
  pub log_filename:   Option<&'a str>,
  /// Log content. Skipped if empty even when `log_filename` is set.
  pub log:            &'a str,
  /// Single-line status string. Always written as `status`.
  pub status:         &'a str,
  /// Resource staging directory (typically a `TempDir`). Every
  /// non-`.html` file under it is bundled, preserving subdirectories.
  /// Pass `None` to skip resource bundling.
  pub resource_dir:   Option<&'a Path>,
  /// Optional per-job telemetry JSON line. When `Some`, written as
  /// `telemetry.json` inside the zip. Used by `cortex_worker` canvas
  /// runs; `benchmark_canvas.sh` extracts this member and appends to
  /// `<output_dir>/telemetry.jsonl`. See `docs/TELEMETRY.md`.
  pub telemetry_json: Option<&'a str>,
  /// `SOURCE_DATE_EPOCH` (Unix seconds, UTC). When `Some`, every zip
  /// member's last-modified time is pinned to it for reproducible
  /// archives — Perl `Pack/Zip.pm` L113-115
  /// (`setLastModFileDateTimeFromUnix`). `None` lets the zip crate use
  /// its default write timestamp.
  pub source_date_epoch: Option<u64>,
}

/// Pack the post-processing outputs into a zip archive.
///
/// Returns an `io::Result` rather than `crate::processor::PostError`
/// because callers (binary mains) are already `Box<dyn Error>`-typed.
///
/// **IO performance:** the underlying file is wrapped in a 64 KiB
/// `BufWriter` before handing it to `ZipWriter`. The zip crate's
/// internal deflate output is small chunks (per-block from miniz);
/// without buffering each chunk would be its own `write()` syscall.
/// Measured ~6× fewer syscalls on 1910.01256 (7 resource files + HTML).
pub fn pack_archive(opts: &PackOptions) -> io::Result<()> {
  let file = File::create(opts.zip_path)?;
  let buf_file = BufWriter::with_capacity(ZIP_WRITE_BUF, file);
  let mut zip = ZipWriter::new(buf_file);
  let mut zip_options =
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
  // Reproducible archives: pin every member's mod-time to SOURCE_DATE_EPOCH
  // when provided (Perl Pack/Zip.pm L113-115). DOS/zip timestamps only
  // span 1980-2107, so out-of-range epochs are silently left at the
  // crate default — matching the spirit of `setLastModFileDateTimeFromUnix`
  // (which would clamp) without failing the whole archive.
  if let Some(epoch) = opts.source_date_epoch {
    if let Some(dt) = epoch_to_zip_datetime(epoch) {
      zip_options = zip_options.last_modified_time(dt);
    }
  }

  // HTML first, so `unzip -l` shows the main artifact at the top.
  zip
    .start_file(opts.html_filename, zip_options)
    .map_err(io_err)?;
  zip.write_all(opts.html.as_bytes())?;

  // Resource files (Graphics-converted PNG/SVG, injected CSS, etc.).
  if let Some(dir) = opts.resource_dir {
    if dir.exists() {
      add_dir_to_zip(&mut zip, dir, dir, &zip_options)?;
    }
  }

  // Log entry.
  if let Some(log_name) = opts.log_filename {
    if !opts.log.is_empty() {
      zip.start_file(log_name, zip_options).map_err(io_err)?;
      zip.write_all(opts.log.as_bytes())?;
    }
  }

  // Status.
  zip.start_file("status", zip_options).map_err(io_err)?;
  zip.write_all(opts.status.as_bytes())?;

  // Telemetry (cortex_worker only).
  if let Some(tjson) = opts.telemetry_json {
    zip
      .start_file("telemetry.json", zip_options)
      .map_err(io_err)?;
    zip.write_all(tjson.as_bytes())?;
  }

  zip.finish().map_err(io_err)?;
  Ok(())
}

/// Recursively add files from `dir` to a ZIP archive, preserving the
/// directory structure relative to `base`.
///
/// Two skip rules apply:
///  * `.html` files — the post-processed HTML is added separately by
///    [`pack_archive`] (and the staging dir may hold a stray copy
///    written there for the Graphics processor's relative paths).
///  * [`is_excluded_archive_entry`] — Perl `Pack/Zip.pm`'s
///    `ARCHIVE_EXT_EXCLUDE` (source `.tex`/`.bib`, nested archives,
///    dotfiles, editor backups). Applied per-basename, matching Perl's
///    `addTree` filter `sub { !/$ext_exclude/ }`.
///
/// Each source file is wrapped in a 64 KiB `BufReader` to amortise
/// `read()` syscalls on the input side (the `io::copy` 8 KiB default
/// chunk would otherwise issue ~ceil(filesize/8K) reads per resource).
fn add_dir_to_zip<W: Write + io::Seek>(
  zip: &mut ZipWriter<W>,
  dir: &Path,
  base: &Path,
  options: &SimpleFileOptions,
) -> io::Result<()> {
  for entry in std::fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    let rel = path.strip_prefix(base).unwrap_or(&path);
    let name = rel.to_string_lossy().to_string();
    let basename = entry.file_name().to_string_lossy().to_string();

    if path.is_dir() {
      // Perl's `addTree` filter excludes whole subtrees whose *directory*
      // name matches (e.g. a nested `.git`); honour the same per-basename
      // rule before recursing.
      if !is_excluded_archive_entry(&basename) {
        add_dir_to_zip(zip, &path, base, options)?;
      }
    } else if !name.ends_with(".html") && !is_excluded_archive_entry(&basename) {
      zip.start_file(&name, *options).map_err(io_err)?;
      let f = File::open(&path)?;
      let mut buf_reader = io::BufReader::with_capacity(ZIP_WRITE_BUF, f);
      std::io::copy(&mut buf_reader, zip)?;
    }
  }
  Ok(())
}

/// Whether a bundle entry should be excluded from the archive — port of
/// Perl `Pack/Zip.pm` `$ARCHIVE_EXT_EXCLUDE`
/// (`qr/(?:^\.)|(?:\.(?:zip|gz|epub|tex|bib|mobi|cache)$)|(?:~$)/`),
/// applied to the file's basename:
///  * hidden dotfiles (`^\.`),
///  * editor backups (`~$`),
///  * nested archives / source / cache files
///    (`.zip`, `.gz`, `.epub`, `.tex`, `.bib`, `.mobi`, `.cache`).
fn is_excluded_archive_entry(basename: &str) -> bool {
  if basename.starts_with('.') || basename.ends_with('~') {
    return true;
  }
  // Suffix test on the lowercase extension (Perl anchors `$`, i.e. the
  // final extension). `rsplit('.')` yields the extension before any dot.
  match basename.rsplit_once('.') {
    Some((_, ext)) => matches!(
      ext.to_ascii_lowercase().as_str(),
      "zip" | "gz" | "epub" | "tex" | "bib" | "mobi" | "cache"
    ),
    None => false,
  }
}

/// Convert a Unix epoch (seconds, UTC) into a zip [`zip::DateTime`].
///
/// DOS/zip timestamps only represent 1980-01-01..=2107; epochs outside
/// that window return `None` (caller falls back to the crate default).
/// Pure civil-date arithmetic (Howard Hinnant's `civil_from_days`) so we
/// don't pull in a date-time crate just for `SOURCE_DATE_EPOCH`.
fn epoch_to_zip_datetime(epoch: u64) -> Option<zip::DateTime> {
  let days = (epoch / 86_400) as i64;
  let secs_of_day = (epoch % 86_400) as u32;
  let (hour, minute, second) = (
    (secs_of_day / 3600) as u8,
    ((secs_of_day % 3600) / 60) as u8,
    (secs_of_day % 60) as u8,
  );
  // civil_from_days: days since 1970-01-01 → (year, month, day).
  let z = days + 719_468;
  let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
  let doe = z - era * 146_097; // [0, 146096]
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
  let year = yoe + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
  let mp = (5 * doy + 2) / 153; // [0, 11]
  let day = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
  let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8; // [1, 12]
  let year = year + i64::from(month <= 2);
  if !(1980..=2107).contains(&year) {
    return None;
  }
  zip::DateTime::from_date_and_time(year as u16, month, day, hour, minute, second).ok()
}

/// Convert a `zip::result::ZipError` into an `io::Error` so the caller
/// signature can stay `io::Result`. The zip crate wraps `io::Error`
/// already; we just re-wrap unrecognized kinds as `Other`.
fn io_err(e: zip::result::ZipError) -> io::Error {
  match e {
    zip::result::ZipError::Io(inner) => inner,
    other => io::Error::other(other.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Read;

  /// Read back the set of entry names from a zip on disk.
  fn zip_entry_names(zip_path: &Path) -> Vec<String> {
    let f = File::open(zip_path).expect("open zip");
    let mut archive = zip::ZipArchive::new(f).expect("parse zip");
    (0..archive.len())
      .map(|i| archive.by_index(i).expect("entry").name().to_string())
      .collect()
  }

  #[test]
  fn excludes_perl_archive_ext_set() {
    // Perl Zip.pm ARCHIVE_EXT_EXCLUDE = qr/(?:^\.)|(?:\.(?:zip|gz|epub|
    // tex|bib|mobi|cache)$)|(?:~$)/ — applied to the basename.
    assert!(is_excluded_archive_entry("paper.tex"));
    assert!(is_excluded_archive_entry("refs.bib"));
    assert!(is_excluded_archive_entry("bundle.zip"));
    assert!(is_excluded_archive_entry("page.gz"));
    assert!(is_excluded_archive_entry("book.epub"));
    assert!(is_excluded_archive_entry("book.mobi"));
    assert!(is_excluded_archive_entry("LaTeXML.cache"));
    assert!(is_excluded_archive_entry(".hidden"));
    assert!(is_excluded_archive_entry("backup~"));
    // Kept: real bundle resources.
    assert!(!is_excluded_archive_entry("fig1.png"));
    assert!(!is_excluded_archive_entry("diagram.svg"));
    assert!(!is_excluded_archive_entry("LaTeXML.css"));
    assert!(!is_excluded_archive_entry("logo.jpg"));
  }

  #[test]
  fn pack_archive_bundles_resources_minus_excluded() {
    let staging = tempfile::tempdir().expect("tempdir");
    let p = staging.path();
    // Resources that SHOULD be bundled.
    std::fs::write(p.join("fig1.png"), b"PNGDATA").unwrap();
    std::fs::write(p.join("LaTeXML.css"), b"body{}").unwrap();
    std::fs::create_dir(p.join("sub")).unwrap();
    std::fs::write(p.join("sub").join("img.svg"), b"<svg/>").unwrap();
    // Resources that must be EXCLUDED.
    std::fs::write(p.join("paper.tex"), b"\\documentclass{article}").unwrap();
    std::fs::write(p.join("refs.bib"), b"@book{x}").unwrap();
    std::fs::write(p.join("LaTeXML.cache"), b"cache").unwrap();
    std::fs::write(p.join(".hidden"), b"secret").unwrap();
    std::fs::write(p.join("backup~"), b"old").unwrap();
    // The HTML is added separately by pack_archive; a stray copy in
    // the staging dir must not be double-added.
    std::fs::write(p.join("doc.html"), b"<html>staging copy</html>").unwrap();

    let out = tempfile::tempdir().expect("out dir");
    let zip_path = out.path().join("bundle.zip");
    let zip_path_str = zip_path.to_string_lossy().to_string();

    pack_archive(&PackOptions {
      zip_path:         &zip_path_str,
      html_filename:    "doc.html",
      html:             "<html>real document</html>",
      log_filename:     Some("doc.log"),
      log:              "log line",
      status:           "Status:conversion:0",
      resource_dir:     Some(p),
      telemetry_json:   None,
      source_date_epoch: None,
    })
    .expect("pack archive");

    let names = zip_entry_names(&zip_path);
    // Bundled resources present.
    assert!(names.iter().any(|n| n == "fig1.png"), "names: {names:?}");
    assert!(names.iter().any(|n| n == "LaTeXML.css"), "names: {names:?}");
    assert!(
      names.iter().any(|n| n == "sub/img.svg"),
      "subdir resource missing; names: {names:?}"
    );
    // Core entries present.
    assert!(names.iter().any(|n| n == "doc.html"));
    assert!(names.iter().any(|n| n == "doc.log"));
    assert!(names.iter().any(|n| n == "status"));
    // Excluded resources absent.
    for forbidden in ["paper.tex", "refs.bib", "LaTeXML.cache", ".hidden", "backup~"] {
      assert!(
        !names.iter().any(|n| n == forbidden),
        "{forbidden} must be excluded; names: {names:?}"
      );
    }
    // Exactly one doc.html (the real one), not the staging copy too.
    assert_eq!(
      names.iter().filter(|n| n.as_str() == "doc.html").count(),
      1,
      "doc.html must not be double-added; names: {names:?}"
    );
    // And the real HTML, not the staging copy, is what got stored.
    let f = File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(f).unwrap();
    let mut html_entry = archive.by_name("doc.html").unwrap();
    let mut body = String::new();
    html_entry.read_to_string(&mut body).unwrap();
    assert_eq!(body, "<html>real document</html>");
  }

  #[test]
  fn source_date_epoch_sets_member_timestamp() {
    // Perl Zip.pm L113-115: when SOURCE_DATE_EPOCH is set, every member
    // gets that fixed mod-time for reproducible archives. 2021-01-01
    // 00:00:00 UTC = 1609459200.
    let staging = tempfile::tempdir().expect("tempdir");
    std::fs::write(staging.path().join("fig.png"), b"x").unwrap();
    let out = tempfile::tempdir().expect("out");
    let zip_path = out.path().join("ts.zip");
    let zip_path_str = zip_path.to_string_lossy().to_string();
    pack_archive(&PackOptions {
      zip_path:          &zip_path_str,
      html_filename:     "d.html",
      html:              "<html/>",
      log_filename:      None,
      log:               "",
      status:            "ok",
      resource_dir:      Some(staging.path()),
      telemetry_json:    None,
      source_date_epoch: Some(1_609_459_200),
    })
    .expect("pack");

    let f = File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(f).unwrap();
    let entry = archive.by_name("fig.png").unwrap();
    let dt = entry.last_modified().expect("has mod time");
    assert_eq!(dt.year(), 2021, "year");
    assert_eq!(dt.month(), 1, "month");
    assert_eq!(dt.day(), 1, "day");
  }
}
