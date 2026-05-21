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
  let zip_options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

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
/// directory structure relative to `base`. Skips `.html` files because
/// the post-processed HTML is added separately by [`pack_archive`].
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

    if path.is_dir() {
      add_dir_to_zip(zip, &path, base, options)?;
    } else if !name.ends_with(".html") {
      zip.start_file(&name, *options).map_err(io_err)?;
      let f = File::open(&path)?;
      let mut buf_reader = io::BufReader::with_capacity(ZIP_WRITE_BUF, f);
      std::io::copy(&mut buf_reader, zip)?;
    }
  }
  Ok(())
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
