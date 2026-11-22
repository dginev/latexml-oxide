//! XML/HTML output sink — port of `LaTeXML::Post::Writer`.
//!
//! Two related concerns live here:
//!
//! 1. The [`Writer`] post-processor (last in the chain) that serializes a `PostDocument` to its
//!    `destination`, handling DOCTYPE removal, TEMPORARY_DOCUMENT_ID cleanup, and HTML vs XML
//!    serialization.
//! 2. Free-standing helpers ([`write_output`], [`ensure_parent_dir`]) used by binary main()s that
//!    already have the serialized string in hand (post-processing returns a `String`) and need to
//!    route it to a destination path or stdout. Replaces the duplicated `File::create + write! +
//!    ensure_parent_dir` boilerplate that used to live in `latexml_oxide.rs` (and the now-retired
//!    `latexmlpost_oxide.rs`).
//!
//! Companion module: [`crate::pack`] (the `LaTeXML::Post::Pack` analog)
//! handles archive bundling when the destination is a zip.

use std::{
  fs,
  io::{self, Write},
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

use libxml::tree::{Node, SaveOptions};

use crate::{
  document::{PostDocument, get_xml_id},
  processor::{PostError, ProcessResult, Processor},
};

/// Write the serialized output `content` to `dest` if `Some`, else to
/// stdout. Creates parent directories as needed.
///
/// Used by `latexml_oxide.rs`'s main() (XML-input mode included) for
/// the "write a single HTML/XML file" exit path. For the zip-archive
/// exit path, use [`crate::pack::pack_archive`].
pub fn write_output(content: &str, dest: Option<&str>) -> io::Result<()> {
  match dest {
    Some(path) => {
      ensure_parent_dir(path)?;
      fs::write(path, content)?;
      Info!(
        "writer",
        "wrote",
        "Wrote '{}' ({} bytes)",
        path,
        content.len()
      );
      Ok(())
    },
    None => io::stdout().write_all(content.as_bytes()),
  }
}

/// Like [`write_output`] but writes several segments back-to-back without
/// concatenating them into one buffer first. Used for the conversion log,
/// where the core and post-phase segments are each already-allocated and
/// large for real articles — a `format!("{core}{post}")` would allocate a
/// third copy of their combined size on the conversion hot path. Segments are
/// written verbatim and in order through a single `BufWriter` (one file
/// open/truncate); insert any separators (e.g. `"\n"`) as their own segments.
pub fn write_output_segments(segments: &[&str], dest: Option<&str>) -> io::Result<()> {
  match dest {
    Some(path) => {
      ensure_parent_dir(path)?;
      let mut writer = io::BufWriter::new(fs::File::create(path)?);
      let mut total = 0usize;
      for seg in segments {
        writer.write_all(seg.as_bytes())?;
        total += seg.len();
      }
      writer.flush()?;
      Info!("writer", "wrote", "Wrote '{}' ({} bytes)", path, total);
      Ok(())
    },
    None => {
      let stdout = io::stdout();
      let mut handle = stdout.lock();
      for seg in segments {
        handle.write_all(seg.as_bytes())?;
      }
      Ok(())
    },
  }
}

/// Ensure the parent directory of `path` exists, creating it (and any
/// missing ancestors) as needed. No-op when `path` has no parent or
/// the parent is the current directory.
pub fn ensure_parent_dir(path: &str) -> io::Result<()> {
  if let Some(parent) = Path::new(path).parent() {
    if !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)?;
    }
  }
  Ok(())
}

/// The directories a page render already made: a split run writes thousands of pages into a handful of
/// directories, and each `create_dir_all` is a `mkdir` and a `statx` (Nasser's 71,476-page document,
/// #777 FAQ).
#[derive(Debug, Default)]
pub struct MadeDirs(std::collections::HashSet<PathBuf>);

std::thread_local! {
  static DIRS_MADE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many directories [`MadeDirs::ensure_parent`] asked the OS for on this thread (a test seam).
pub fn made_dir_count() -> usize { DIRS_MADE.get() }

impl MadeDirs {
  /// [`ensure_parent_dir`], once per directory; an error leaves the directory to be retried.
  pub fn ensure_parent(&mut self, path: &str) -> io::Result<()> {
    let Some(parent) = Path::new(path).parent() else {
      return Ok(());
    };
    if parent.as_os_str().is_empty() || self.0.contains(parent) {
      return Ok(());
    }
    DIRS_MADE.set(DIRS_MADE.get() + 1);
    fs::create_dir_all(parent)?;
    self.0.insert(parent.to_path_buf());
    Ok(())
  }
}

/// Monotonic per-process counter for unique sibling temp files in [`write_atomic`] / [`copy_atomic`].
static ATOMIC_TMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn atomic_tmp_path(dest: &Path) -> PathBuf {
  let seq = ATOMIC_TMP_SEQ.fetch_add(1, Ordering::Relaxed);
  let pid = std::process::id();
  let name = dest
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("resource");
  let tmp_name = format!(".{name}.tmp.{pid}.{seq}");
  match dest.parent() {
    Some(p) if !p.as_os_str().is_empty() => p.join(tmp_name),
    _ => PathBuf::from(tmp_name),
  }
}

fn install_temp(tmp: &Path, dest: &Path, expected_len: u64) -> io::Result<()> {
  let _ = expected_len;
  match fs::rename(tmp, dest) {
    Ok(()) => Ok(()),
    Err(e) => {
      let _ = fs::remove_file(tmp);
      // On Windows, `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` can fail with
      // sharing/access errors when parallel render workers race to publish the
      // same shared resource (CSS/JS/graphics). If a peer worker already
      // installed a regular file of the expected length at `dest`, treat the
      // race as satisfied; keep real errors (e.g. `dest` is a directory).
      #[cfg(windows)]
      if dest.is_file() && fs::metadata(dest).is_ok_and(|m| m.len() == expected_len) {
        return Ok(());
      }
      Err(e)
    },
  }
}

/// Atomically write `bytes` to `dest` via a sibling temp file + rename.
///
/// Safe when multiple parallel render workers publish the same shared resource
/// (`LaTeXML.css`, `LaTeXML.js`, etc.) concurrently: Unix readers never observe
/// a truncated/interleaved file, and Windows avoids `ERROR_SHARING_VIOLATION`
/// on concurrent writers.
pub fn write_atomic(dest: &Path, bytes: &[u8]) -> io::Result<()> {
  let tmp = atomic_tmp_path(dest);
  if let Err(e) = fs::write(&tmp, bytes) {
    let _ = fs::remove_file(&tmp);
    return Err(e);
  }
  install_temp(&tmp, dest, bytes.len() as u64)
}

/// Atomically copy `src` to `dest` via a sibling temp file + rename.
pub fn copy_atomic(src: &Path, dest: &Path) -> io::Result<()> {
  let tmp = atomic_tmp_path(dest);
  let len = match fs::copy(src, &tmp) {
    Ok(n) => n,
    Err(e) => {
      let _ = fs::remove_file(&tmp);
      return Err(e);
    },
  };
  install_temp(&tmp, dest, len)
}

/// Output format for the writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  Xml,
  Html,
}

/// Writer post-processor: serializes document to file.
///
/// Port of `LaTeXML::Post::Writer`.
pub struct Writer {
  name:         String,
  format:       OutputFormat,
  omit_doctype: bool,
  is_html:      bool,
}

impl Writer {
  pub fn new(format: Option<OutputFormat>, omit_doctype: bool, is_html: bool) -> Self {
    Writer {
      name: "Writer".to_string(),
      format: format.unwrap_or(OutputFormat::Xml),
      omit_doctype,
      is_html,
    }
  }
}

impl Processor for Writer {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    match doc.get_document_element() {
      Some(el) => vec![el],
      None => vec![],
    }
  }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let mut root = match nodes.into_iter().next() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

    // Remove the internal DTD subset if requested (Perl Writer.pm L38:
    // `$doc->getDocument->removeInternalSubset if $$self{omit_doctype}`).
    // Backed by `libxml::tree::Document::remove_internal_subset`, added
    // in rust-libxml 0.3.11 specifically to close this gap.
    if self.omit_doctype {
      doc.get_document_mut().remove_internal_subset();
    }

    // Remove TEMPORARY_DOCUMENT_ID if present (Perl Writer.pm L41-42).
    // NS-aware forms: the bare get/remove_attribute("xml:id") pair silently
    // no-opped, so this last-chance strip never fired (the split path
    // removes the temp id earlier via core_interface, but this writer-side
    // guard is the one Perl relies on).
    if let Some(id) = get_xml_id(&root) {
      if id == "TEMPORARY_DOCUMENT_ID" {
        let _ = root.remove_attribute_ns("id", latexml_core::common::xml::XML_NS);
      }
    }

    // Serialize: HTML uses toStringHTML, XML uses toString(1)  (Perl Writer.pm L44-47)
    let serialized = if self.is_html {
      doc.get_document().to_string_with_options(SaveOptions {
        as_html: true,
        format: true,
        ..SaveOptions::default()
      })
    } else {
      doc.get_document().to_string_with_options(SaveOptions {
        format: true,
        ..SaveOptions::default()
      })
    };

    if let Some(destination) = doc.get_destination() {
      // Create destination directory if needed
      if let Some(destdir) = doc.get_destination_directory() {
        fs::create_dir_all(destdir).map_err(|e| {
          PostError::Io(io::Error::new(
            e.kind(),
            format!("Couldn't create directory '{}': {}", destdir, e),
          ))
        })?;
      }
      fs::write(destination, &serialized).map_err(|e| {
        PostError::Io(io::Error::new(
          e.kind(),
          format!("Couldn't write '{}': {}", destination, e),
        ))
      })?;
      Info!(
        "writer",
        "wrote",
        "Wrote '{}' ({})",
        destination,
        serialized.len()
      );
    } else {
      print!("{}", serialized);
    }

    Ok(vec![doc])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn output_format_variants() {
    let _ = OutputFormat::Xml;
    let _ = OutputFormat::Html;
  }

  #[test]
  fn output_format_partial_eq() {
    assert_eq!(OutputFormat::Xml, OutputFormat::Xml);
    assert_ne!(OutputFormat::Xml, OutputFormat::Html);
    assert_eq!(OutputFormat::Html, OutputFormat::Html);
  }

  #[test]
  fn output_format_copy_clone() {
    // Copy trait: move-after-use still works.
    let a = OutputFormat::Xml;
    let b = a;
    let _ = a; // still usable due to Copy
    assert_eq!(a, b);
  }

  #[test]
  fn writer_new_default_format_xml() {
    let w = Writer::new(None, false, false);
    assert_eq!(w.get_name(), "Writer");
    assert_eq!(w.format, OutputFormat::Xml);
    assert!(!w.omit_doctype);
    assert!(!w.is_html);
  }

  #[test]
  fn writer_new_explicit_format() {
    let w = Writer::new(Some(OutputFormat::Html), true, true);
    assert_eq!(w.format, OutputFormat::Html);
    assert!(w.omit_doctype);
    assert!(w.is_html);
  }

  #[test]
  fn writer_get_name_is_writer() {
    let w = Writer::new(None, false, false);
    assert_eq!(w.get_name(), "Writer");
  }

  #[test]
  /// `Writer::process` with `omit_doctype = true` drops the
  /// `<!DOCTYPE …>` preamble from the output, mirroring Perl
  /// `Post::Writer` L38 behaviour.
  fn writer_omit_doctype_strips_doctype_preamble() {
    use crate::document::{PostDocument, PostDocumentOptions};

    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "example.dtd">
<root><child>hi</child></root>"#;
    let doc = PostDocument::new_from_string(xml, PostDocumentOptions::default())
      .expect("parse test fixture");

    // omit_doctype=true → DOCTYPE stripped after Writer::process.
    let mut writer = Writer::new(
      None, /* omit_doctype= */ true, /* is_html= */ false,
    );
    let to_process = writer.to_process(&doc);
    let result = writer.process(doc, to_process).expect("process");
    let after = result
      .into_iter()
      .next()
      .expect("at least one doc")
      .get_document()
      .to_string();
    assert!(
      !after.contains("<!DOCTYPE"),
      "expected DOCTYPE stripped, got: {after}"
    );
    assert!(after.contains("<root>"));
  }

  #[test]
  /// `Writer::process` with `omit_doctype = false` (the default)
  /// preserves the `<!DOCTYPE …>` preamble — opt-in behaviour.
  fn writer_default_preserves_doctype() {
    use crate::document::{PostDocument, PostDocumentOptions};

    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "example.dtd">
<root><child>hi</child></root>"#;
    let doc = PostDocument::new_from_string(xml, PostDocumentOptions::default())
      .expect("parse test fixture");

    let mut writer = Writer::new(
      None, /* omit_doctype= */ false, /* is_html= */ false,
    );
    let to_process = writer.to_process(&doc);
    let result = writer.process(doc, to_process).expect("process");
    let after = result
      .into_iter()
      .next()
      .expect("at least one doc")
      .get_document()
      .to_string();
    assert!(
      after.contains("<!DOCTYPE"),
      "expected DOCTYPE preserved, got: {after}"
    );
  }

  #[test]
  fn write_and_copy_atomic_succeed_and_clean_up_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let dst = dir.path().join("LaTeXML.css");
    write_atomic(&dst, b"body { color: black; }\n").unwrap();
    assert_eq!(fs::read(&dst).unwrap(), b"body { color: black; }\n");

    let src = dir.path().join("custom.css");
    fs::write(&src, b".a { color: red; }\n").unwrap();
    copy_atomic(&src, &dst).unwrap();
    assert_eq!(fs::read(&dst).unwrap(), b".a { color: red; }\n");

    // Failing install onto a directory must return Err and leave no stray .tmp file.
    let bad = dir.path().join("blocked.css");
    fs::create_dir(&bad).unwrap();
    assert!(write_atomic(&bad, b"x").is_err());
    assert!(copy_atomic(&src, &bad).is_err());
    let strays: Vec<_> = fs::read_dir(dir.path())
      .unwrap()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
      .collect();
    assert!(strays.is_empty(), "leftover atomic temp files: {strays:?}");
  }

  #[test]
  fn concurrent_write_atomic_produces_intact_file_without_errors() {
    let dir = tempfile::tempdir().unwrap();
    let dst = dir.path().join("shared.css");
    let payload: Vec<u8> = b"/* css */\n".repeat(4096);
    std::thread::scope(|s| {
      for _ in 0..8 {
        let dst = &dst;
        let payload = &payload;
        s.spawn(move || {
          for _ in 0..16 {
            write_atomic(dst, payload).expect("concurrent atomic write");
          }
        });
      }
    });
    assert_eq!(fs::read(&dst).unwrap(), payload);
  }
}
