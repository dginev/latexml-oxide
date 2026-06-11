//! Abstract post-processor base.
//!
//! Port of `LaTeXML::Post::Processor`.
//! All post-processors implement the [`Processor`] trait.

use std::path::PathBuf;

use libxml::tree::Node;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;

use crate::document::PostDocument;

/// Options for constructing a processor.
#[derive(Debug, Default, Clone)]
pub struct ProcessorOptions {
  pub resource_directory: Option<String>,
  pub resource_prefix:    Option<String>,
}

/// Result of processing: the document (possibly split into multiple).
pub type ProcessResult = Result<Vec<PostDocument>, PostError>;

/// Errors from post-processing.
#[derive(Debug)]
pub enum PostError {
  /// A processing error with context message.
  Processing(String),
  /// An I/O error.
  Io(std::io::Error),
  /// An XML error.
  Xml(String),
}

impl std::fmt::Display for PostError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PostError::Processing(msg) => write!(f, "Post-processing error: {}", msg),
      PostError::Io(err) => write!(f, "I/O error: {}", err),
      PostError::Xml(msg) => write!(f, "XML error: {}", msg),
    }
  }
}

impl std::error::Error for PostError {}

impl From<std::io::Error> for PostError {
  fn from(err: std::io::Error) -> Self { PostError::Io(err) }
}

/// Abstract base trait for all post-processors.
///
/// Corresponds to `LaTeXML::Post::Processor`.
/// Processors operate on a [`PostDocument`] and return one or more documents
/// (splitting may produce multiple outputs).
pub trait Processor {
  /// Human-readable name for this processor.
  fn get_name(&self) -> &str;

  /// Resource directory for generated resources (images, etc.).
  fn resource_directory(&self) -> Option<&str> { None }

  /// Resource prefix for generated resource filenames.
  fn resource_prefix(&self) -> Option<&str> { None }

  /// Return the nodes to be processed; by default the document element.
  /// This allows processors to focus on specific kinds of nodes,
  /// or to skip processing if there are none to process.
  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    match doc.get_document_element() {
      Some(el) => vec![el],
      None => vec![],
    }
  }

  /// Process the document given the nodes returned by `to_process`.
  /// Returns the resulting document(s) — splitting may produce multiple.
  fn process(&mut self, doc: PostDocument, nodes: Vec<Node>) -> ProcessResult;

  /// Hint for a desired resource pathname.
  fn desired_resource_pathname(
    &self,
    _doc: &PostDocument,
    _node: &Node,
    _source: Option<&str>,
    _type_ext: Option<&str>,
  ) -> Option<PathBuf> {
    None
  }

  /// Auto-generate a unique resource pathname using a counter from the doc cache.
  fn generate_resource_pathname(
    &self,
    doc: &mut PostDocument,
    _node: &Node,
    _source: Option<&str>,
    type_ext: Option<&str>,
  ) -> PathBuf {
    let subdir = self.resource_directory().unwrap_or("");
    let prefix = self.resource_prefix().unwrap_or("x");
    let counter_key = format!("_max_{}_{}_counter_", subdir, prefix);
    let n = doc
      .cache_lookup(&counter_key)
      .and_then(|v| v.parse::<u32>().ok())
      .unwrap_or(0)
      + 1;
    doc.cache_store(&counter_key, &n.to_string());
    let name = format!("{}{}", prefix, n);
    let mut path = PathBuf::from(subdir);
    let filename = if let Some(ext) = type_ext {
      format!("{}.{}", name, ext)
    } else {
      name
    };
    path.push(filename);
    path
  }
}

/// Information about a document class or package extracted from processing instructions.
#[derive(Debug, Clone)]
pub struct ClassInfo {
  pub name:     String,
  pub options:  String,
  pub oldstyle: Option<String>,
}

/// Information about a loaded package.
#[derive(Debug, Clone)]
pub struct PackageInfo {
  pub name:    String,
  pub options: String,
}

/// Extract the document class and packages from `<?latexml ...?>` processing instructions.
///
/// Returns `(class_info, packages)` where `class_info` defaults to "article" if none found.
///
/// Port of `Processor::find_documentclass_and_packages`.
pub fn find_documentclass_and_packages(doc: &PostDocument) -> (ClassInfo, Vec<PackageInfo>) {
  let pi_re = Regex::new(r#"\s*([\w\-_]*)=[\"'](.*?)[\"']"#).unwrap();
  let mut class: Option<String> = None;
  let mut classoptions = String::from("onecolumn");
  let mut oldstyle: Option<String> = None;
  let mut packages = Vec::new();

  for pi in doc.findnodes(".//processing-instruction('latexml')") {
    let data = pi.get_content();
    let mut entry = HashMap::default();
    for cap in pi_re.captures_iter(&data) {
      entry.insert(cap[1].to_string(), cap[2].to_string());
    }
    if let Some(cls) = entry.get("class") {
      class = Some(cls.clone());
      classoptions = entry
        .get("options")
        .cloned()
        .unwrap_or_else(|| "onecolumn".to_string());
      oldstyle = entry.get("oldstyle").cloned();
    } else if let Some(pkg) = entry.get("package") {
      let opts = entry.get("options").cloned().unwrap_or_default();
      for p in pkg.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        packages.push(PackageInfo {
          name:    p.to_string(),
          options: opts.clone(),
        });
      }
    }
  }

  if class.is_none() {
    // Perl Post.pm:226 — Warn('expected', 'class', undef,
    //   "No document class found; using article")
    Warn!(
      "expected",
      "class",
      "No document class found; using article"
    );
  }

  let class_info = ClassInfo {
    name: class.unwrap_or_else(|| "article".to_string()),
    options: classoptions,
    oldstyle,
  };
  (class_info, packages)
}

/// Extract preamble data from `<?latexml ...?>` processing instructions.
///
/// Port of `Processor::find_preambles`.
pub fn find_preambles(doc: &PostDocument) -> String {
  let pi_re = Regex::new(r#"\s*([\w\-_]*)=[\"'](.*?)[\"']"#).unwrap();
  let mut preambles = Vec::new();

  for pi in doc.findnodes(".//processing-instruction('latexml')") {
    let data = pi.get_content();
    for cap in pi_re.captures_iter(&data) {
      if &cap[1] == "preamble" {
        preambles.push(cap[2].to_string());
      }
    }
  }

  preambles.join("\n")
}

/// Copy foreign-namespace attributes from `source` to `target`.
///
/// "Foreign" means attributes with a namespace prefix (contains ':')
/// but NOT `xml:*` attributes.
///
/// Port of `Processor::copy_foreign_attributes`.
pub fn copy_foreign_attributes(target: &mut Node, source: &Node) {
  let props = source.get_properties();
  for (key, value) in &props {
    if key.starts_with("xml:") {
      continue;
    }
    if !key.contains(':') {
      continue;
    }
    // Only set if target doesn't already have this attribute
    if target.get_attribute(key).is_none() {
      target.set_attribute(key, value).ok();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn processor_options_default_is_empty() {
    let o = ProcessorOptions::default();
    assert!(o.resource_directory.is_none());
    assert!(o.resource_prefix.is_none());
  }

  #[test]
  fn processor_options_clone_preserves() {
    let o = ProcessorOptions {
      resource_directory: Some("/tmp".to_string()),
      resource_prefix:    Some("pre".to_string()),
    };
    let c = o.clone();
    assert_eq!(c.resource_directory, Some("/tmp".to_string()));
    assert_eq!(c.resource_prefix, Some("pre".to_string()));
  }

  #[test]
  fn post_error_display_processing() {
    let e = PostError::Processing("boom".to_string());
    let s = format!("{e}");
    assert!(s.contains("Post-processing error"));
    assert!(s.contains("boom"));
  }

  #[test]
  fn post_error_display_xml() {
    let e = PostError::Xml("malformed".to_string());
    let s = format!("{e}");
    assert!(s.contains("XML error"));
    assert!(s.contains("malformed"));
  }

  #[test]
  fn post_error_from_io_error() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
    let pe: PostError = io.into();
    match pe {
      PostError::Io(_) => {},
      other => panic!("expected Io, got {other:?}"),
    }
  }

  #[test]
  fn post_error_display_io() {
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let e = PostError::Io(io);
    let s = format!("{e}");
    assert!(s.contains("I/O error"));
  }

  #[test]
  fn post_error_impls_std_error() {
    // The blanket impl lets us box it as dyn Error.
    fn take_err<E: std::error::Error>(_: &E) {}
    let e = PostError::Processing("x".to_string());
    take_err(&e);
  }
}
