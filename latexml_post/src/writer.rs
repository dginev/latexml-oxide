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
  path::Path,
};

use libxml::tree::{Node, SaveOptions};

use crate::{
  document::PostDocument,
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

    // Remove TEMPORARY_DOCUMENT_ID if present (Perl Writer.pm L41-42)
    if let Some(id) = root.get_attribute("xml:id") {
      if id == "TEMPORARY_DOCUMENT_ID" {
        let _ = root.remove_attribute("xml:id");
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
}
