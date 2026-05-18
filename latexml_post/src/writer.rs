//! XML/HTML output sink — port of `LaTeXML::Post::Writer`.
//!
//! Two related concerns live here:
//!
//! 1. The [`Writer`] post-processor (last in the chain) that
//!    serializes a `PostDocument` to its `destination`, handling
//!    DOCTYPE removal, TEMPORARY_DOCUMENT_ID cleanup, and HTML vs
//!    XML serialization.
//! 2. Free-standing helpers ([`write_output`], [`ensure_parent_dir`])
//!    used by binary main()s that already have the serialized string
//!    in hand (post-processing returns a `String`) and need to route
//!    it to a destination path or stdout. Replaces the duplicated
//!    `File::create + write! + ensure_parent_dir` boilerplate that
//!    used to live in `latexml_oxide.rs` and `latexmlpost_oxide.rs`.
//!
//! Companion module: [`crate::pack`] (the `LaTeXML::Post::Pack` analog)
//! handles archive bundling when the destination is a zip.

use libxml::tree::{Node, SaveOptions};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::document::PostDocument;
use crate::processor::{PostError, ProcessResult, Processor};

/// Write the serialized output `content` to `dest` if `Some`, else to
/// stdout. Creates parent directories as needed.
///
/// Used by `latexml_oxide.rs` and `latexmlpost_oxide.rs` main()s for
/// the "write a single HTML/XML file" exit path. For the zip-archive
/// exit path, use [`crate::pack::pack_archive`].
pub fn write_output(content: &str, dest: Option<&str>) -> io::Result<()> {
  match dest {
    Some(path) => {
      ensure_parent_dir(path)?;
      fs::write(path, content)?;
      log::info!("Wrote '{}' ({} bytes)", path, content.len());
      Ok(())
    },
    None => io::stdout().write_all(content.as_bytes()),
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

  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let mut root = match nodes.into_iter().next() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

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
          PostError::Io(std::io::Error::new(
            e.kind(),
            format!("Couldn't create directory '{}': {}", destdir, e),
          ))
        })?;
      }
      fs::write(destination, &serialized).map_err(|e| {
        PostError::Io(std::io::Error::new(
          e.kind(),
          format!("Couldn't write '{}': {}", destination, e),
        ))
      })?;
      log::info!("Wrote '{}' ({})", destination, serialized.len());
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
}
