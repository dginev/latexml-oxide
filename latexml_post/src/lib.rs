#![allow(dead_code)] // Library in progress — many APIs not yet consumed externally
#![allow(clippy::collapsible_match, clippy::collapsible_if)]
#![allow(clippy::if_same_then_else)] // Intentional per-type branches for FMT_SPEC clarity
#![allow(clippy::cloned_ref_to_slice_refs)] // Node::clone() is Rc clone (cheap), needed for &[Node] APIs
//! Post-processing pipeline for latexml_oxide.
//!
//! Rust port of `LaTeXML::Post` — the driver that orchestrates
//! post-processors (Scan, CrossRef, MathML conversion, XSLT, Writer, etc.)
//! on a converted LaTeXML XML document.
//!
//! # Architecture
//!
//! The processing pipeline follows the Perl original:
//!
//! 1. **Input**: An XML document produced by the core LaTeXML conversion
//! 2. **ProcessChain**: Each `Processor` in sequence gets:
//!    - `to_process(doc)` → nodes relevant to this processor
//!    - `process(doc, nodes)` → the (possibly split) result document(s)
//! 3. **Output**: One or more processed XML documents
//!
//! # Modules
//!
//! - [`document`] — `PostDocument`: XML wrapper with ID management, XPath, caching
//! - [`processor`] — `Processor` trait: abstract base for all post-processors
//! - [`math_processor`] — `MathProcessor` trait: abstract base for math converters
//! - [`radix`] — Radix utilities for ID generation (a,b,...,z,aa,ab,...)

// Core infrastructure
pub mod doc_owned_node;
pub mod document;
pub mod math_processor;
pub mod object_db;
pub mod processor;
pub mod radix;

// Concrete post-processors (alphabetical)
pub mod collector;
pub mod crossref;
pub mod graphics;
pub mod latex_images;
pub mod lex_math;
pub mod make_bibliography;
pub mod make_index;
pub mod manifest;
pub mod math_images;
pub mod mathml;
pub mod open_math;
pub mod picture_images;
pub mod scan;
pub mod split;
pub mod svg;
pub mod tex_math;
pub mod unicode;
pub mod unicode_math;
pub mod writer;
pub mod xmath;
pub mod xslt;

use document::PostDocument;
use processor::{PostError, Processor};
use std::sync::LazyLock;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static POST_AUDIT: LazyLock<bool> = LazyLock::new(|| std::env::var("LATEXML_POST_AUDIT").is_ok());

/// The post-processing pipeline driver.
///
/// Port of `LaTeXML::Post`.
/// Manages a chain of processors and orchestrates their execution.
pub struct Post {
  /// Status tracking.
  pub status: PostStatus,
}

/// Status of post-processing.
#[derive(Debug, Default)]
pub struct PostStatus {
  pub warning_count: u32,
  pub error_count:   u32,
  pub fatal_count:   u32,
  pub info_count:    u32,
}

impl Post {
  /// Create a new post-processing driver.
  pub fn new() -> Self { Post { status: PostStatus::default() } }

  /// Run the processing chain on a document.
  ///
  /// Each processor in order gets the current document(s),
  /// finds nodes to process via `to_process()`, and transforms them
  /// via `process()`. Documents may be split (producing multiple outputs).
  ///
  /// Port of `Post::ProcessChain` + `ProcessChain_internal`.
  pub fn process_chain(
    &mut self,
    doc: PostDocument,
    processors: &mut [Box<dyn Processor>],
  ) -> Result<Vec<PostDocument>, PostError> {
    let mut docs = vec![doc];

    log::info!("post-processing");
    let audit = *POST_AUDIT;

    for processor in processors.iter_mut() {
      // Map processor names to telemetry phases. See docs/TELEMETRY.md.
      // Names come from each Processor's get_name() — XSLT prefixes
      // with "XSLT[", MathML uses "MathML::Presentation"/"::Content".
      // Anything unrecognised attributes to Xslt as a coarse fallback.
      let pname = processor.get_name();
      let phase = if pname.starts_with("MathML::Presentation") {
        latexml_core::telemetry::Phase::MathmlPres
      } else if pname.starts_with("MathML::Content") {
        latexml_core::telemetry::Phase::MathmlCont
      } else if pname.starts_with("XSLT") {
        latexml_core::telemetry::Phase::Xslt
      } else {
        latexml_core::telemetry::Phase::Xslt
      };
      let _gp = latexml_core::telemetry::phase(phase);
      let mut new_docs = Vec::new();
      for doc in docs {
        let nodes = processor.to_process(&doc);
        if !nodes.is_empty() {
          let n = nodes.len();
          let msg = format!(
            "{} {} {}",
            processor.get_name(),
            doc.site_relative_destination().unwrap_or_default(),
            if n > 1 {
              format!("{} to process", n)
            } else {
              "processing".to_string()
            }
          );
          log::info!("{}", msg);
          let t0 = if audit {
            Some(std::time::Instant::now())
          } else {
            None
          };
          let result_docs = processor.process(doc, nodes)?;
          if let Some(t0) = t0 {
            let ms = t0.elapsed().as_millis();
            log::info!(
              "POST_AUDIT stage {} took {}ms ({} nodes)",
              processor.get_name(),
              ms,
              n
            );
          }
          new_docs.extend(result_docs);
        } else {
          new_docs.push(doc);
        }
      }
      docs = new_docs;
    }

    log::info!("post-processing complete");
    Ok(docs)
  }
}

impl Default for Post {
  fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::document::PostDocumentOptions;
  use crate::writer::{OutputFormat, Writer};

  #[test]
  fn test_empty_pipeline() {
    let mut post = Post::new();
    let doc = document::PostDocument::new_from_string(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'/>",
      PostDocumentOptions::default(),
    )
    .unwrap();

    let mut processors: Vec<Box<dyn Processor>> = vec![];
    let result = post.process_chain(doc, &mut processors);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
  }

  #[test]
  fn test_writer_pipeline() {
    let mut post = Post::new();
    let doc = document::PostDocument::new_from_string(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'><title>Test</title></document>",
      PostDocumentOptions::default(),
    )
    .unwrap();

    // Writer without destination prints to stdout (we just test it doesn't crash)
    let writer = Writer::new(Some(OutputFormat::Xml), false, false);
    let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(writer)];
    let result = post.process_chain(doc, &mut processors);
    assert!(result.is_ok());
  }

  #[test]
  fn test_pmml_pipeline() {
    let mut post = Post::new();
    let doc = document::PostDocument::new_from_string(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML'>\
         <para xml:id='p1'><p>Inline <Math mode='inline' tex='a+b' text='a + b' xml:id='p1.m1'>\
           <XMath><XMApp>\
             <XMTok meaning='plus' role='ADDOP'>+</XMTok>\
             <XMTok font='italic' role='ID'>a</XMTok>\
             <XMTok font='italic' role='ID'>b</XMTok>\
           </XMApp></XMath></Math></p></para>\
       </document>",
      PostDocumentOptions::default(),
    )
    .unwrap();

    let pmml = crate::mathml::MathML::new_presentation().with_keep_xmath(true);
    let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(pmml)];
    let result = post.process_chain(doc, &mut processors);
    assert!(result.is_ok());
    let docs = result.unwrap();
    let output = docs[0].to_xml_string();
    eprintln!("PMML output:\n{}", output);
    // Should contain both XMath and m:math
    assert!(
      output.contains("<XMath>") || output.contains("<XMath "),
      "XMath should be preserved"
    );
    assert!(
      output.contains("m:math"),
      "m:math element should be present"
    );
    assert!(output.contains("m:mi"), "m:mi element should be present");
    assert!(output.contains("m:mo"), "m:mo element should be present");
  }

  #[test]
  fn test_scan_pipeline() {
    let mut post = Post::new();
    let doc = document::PostDocument::new_from_string(
      "<document xmlns='http://dlmf.nist.gov/LaTeXML' xml:id='doc'>\
         <section xml:id='s1'><title>First</title></section>\
         <section xml:id='s2'><title>Second</title></section>\
       </document>",
      PostDocumentOptions::default(),
    )
    .unwrap();

    let db = object_db::ObjectDB::new();
    let scanner = scan::Scan::new(db);
    let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(scanner)];
    let result = post.process_chain(doc, &mut processors);
    assert!(result.is_ok());
  }
}
