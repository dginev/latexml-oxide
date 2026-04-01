//! XML file output processor.
//!
//! Port of `LaTeXML::Post::Writer`.
//! Serializes the XML document to a file or stdout.

use libxml::tree::Node;
use std::fs;

use crate::document::PostDocument;
use crate::processor::{PostError, ProcessResult, Processor};

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
  name: String,
  format: OutputFormat,
  omit_doctype: bool,
  is_html: bool,
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
  fn get_name(&self) -> &str {
    &self.name
  }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    // Writer processes the document element (anything can be printed)
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

    // Remove TEMPORARY_DOCUMENT_ID if present
    if let Some(id) = root.get_attribute("xml:id") {
      if id == "TEMPORARY_DOCUMENT_ID" {
        let _ = root.remove_attribute("xml:id");
      }
    }

    // Serialize
    let serialized = doc.to_xml_string();

    if let Some(destination) = doc.get_destination() {
      // Write to file
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
    } else {
      // Write to stdout
      print!("{}", serialized);
    }

    Ok(vec![doc])
  }
}
