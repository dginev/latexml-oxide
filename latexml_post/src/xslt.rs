//! XSLT transformation processor.
//!
//! Port of `LaTeXML::Post::XSLT`.
//! Applies an XSLT stylesheet to transform the document (e.g., LaTeXML XML → HTML5).
//! Handles CSS/JS/icon resource copying.

use libxml::tree::Node;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::document::{PostDocument, PostDocumentOptions};
use crate::processor::{PostError, ProcessResult, Processor};

/// Resource type information.
struct ResourceInfo {
  extension: &'static str,
  subdir:    &'static str,
}

const RESOURCE_CSS: ResourceInfo = ResourceInfo {
  extension: "css",
  subdir:    "resources/CSS",
};
const RESOURCE_JS: ResourceInfo = ResourceInfo {
  extension: "js",
  subdir:    "resources/javascript",
};

/// XSLT post-processor: applies a stylesheet transformation.
///
/// Port of `LaTeXML::Post::XSLT`.
pub struct XSLT {
  name:               String,
  /// Path to the XSLT stylesheet.
  stylesheet_path:    Option<String>,
  /// Parameters to pass to the XSLT stylesheet.
  parameters:         HashMap<String, String>,
  /// Whether to remove resource requests (CSS/JS not copied).
  no_resources:       bool,
  /// Resource directory for copied resources.
  resource_directory: Option<String>,
  /// Search paths for finding resources.
  searchpaths:        Vec<String>,
}

impl XSLT {
  pub fn new(
    stylesheet: &str,
    parameters: HashMap<String, String>,
    no_resources: bool,
    resource_directory: Option<String>,
    searchpaths: Vec<String>,
  ) -> Result<Self, PostError> {
    if stylesheet.is_empty() {
      return Err(PostError::Processing(
        "No stylesheet specified!".to_string(),
      ));
    }

    // Find the stylesheet file
    let stylesheet_path = find_stylesheet(stylesheet, &searchpaths)?;

    Ok(XSLT {
      name: format!("XSLT[using {}]", stylesheet),
      stylesheet_path: Some(stylesheet_path),
      parameters,
      no_resources,
      resource_directory,
      searchpaths,
    })
  }

  /// Copy a resource file and return the path relative to the destination.
  ///
  /// Port of `XSLT::copyResource`.
  fn copy_resource(&self, doc: &PostDocument, src: &str, resource_type: Option<&str>) -> String {
    // If it's a URL, return as-is
    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("//") {
      return src.to_string();
    }

    let info = match resource_type {
      Some("text/css") => Some(&RESOURCE_CSS),
      Some("text/javascript") => Some(&RESOURCE_JS),
      _ => None,
    };

    // Try to find the file
    let search_paths: Vec<&str> = doc
      .get_search_paths()
      .iter()
      .chain(self.searchpaths.iter())
      .map(String::as_str)
      .collect();

    let found_path = find_resource_file(src, info, &search_paths);

    match found_path {
      Some(path) => {
        let file_name = Path::new(&path)
          .file_name()
          .and_then(|f| f.to_str())
          .unwrap_or(src);

        // Determine destination
        let dest = if let Some(ref rd) = self.resource_directory {
          if let Some(site_dir) = doc.get_site_directory() {
            format!("{}/{}/{}", site_dir, rd, file_name)
          } else {
            format!("{}/{}", rd, file_name)
          }
        } else {
          // Preserve relative path
          if let Some(dest_dir) = doc.get_destination_directory() {
            format!("{}/{}", dest_dir, file_name)
          } else {
            file_name.to_string()
          }
        };

        // Copy if source != destination
        if path != dest {
          if let Some(parent) = Path::new(&dest).parent() {
            let _ = fs::create_dir_all(parent);
          }
          if let Err(e) = fs::copy(&path, &dest) {
            log::warn!("Couldn't copy {} to {}: {}", path, dest, e);
          }
        }

        // Return relative to destination directory
        if let Some(dest_dir) = doc.get_destination_directory() {
          relative_path(&dest, dest_dir)
        } else {
          dest
        }
      },
      None => {
        log::warn!(
          "Couldn't find resource file {} in paths {:?}",
          src,
          search_paths
        );
        src.to_string()
      },
    }
  }
}

impl Processor for XSLT {
  fn get_name(&self) -> &str { &self.name }

  fn process(&mut self, doc: PostDocument, _nodes: Vec<Node>) -> ProcessResult {
    let stylesheet_path = match &self.stylesheet_path {
      Some(p) => p.clone(),
      None => return Ok(vec![doc]),
    };

    log::info!("Applying XSLT stylesheet: {}", stylesheet_path);

    // Handle resource elements first (before transformation removes them)
    let resource_nodes = doc.findnodes("//ltx:resource[@src]");
    if self.no_resources {
      // Perl L64-65: remove resource nodes so XSLT won't generate CSS/JS links
      for mut node in resource_nodes {
        node.unlink_node();
      }
    } else {
      for node in &resource_nodes {
        if let Some(src) = node.get_attribute("src") {
          let resource_type = node.get_attribute("type");
          let _path = self.copy_resource(&doc, &src, resource_type.as_deref());
        }
      }
    }

    // Register EXSLT extension functions (str:tokenize, math:*, etc.)
    // used by LaTeXML stylesheets. Safe-wrapped upstream in
    // rust-libxslt — `register_exslt()` is Once-guarded.
    libxslt::register_exslt();

    // Parse the stylesheet using the libxslt crate
    let mut stylesheet = libxslt::parser::parse_file(&stylesheet_path)
      .map_err(|e| PostError::Processing(format!("Failed to parse XSLT stylesheet: {}", e)))?;

    // Serialize and re-parse for transformation (transform consumes the doc)
    let doc_xml = doc.to_xml_string();
    let parser = libxml::parser::Parser::default();
    let transform_doc = parser
      .parse_string(&doc_xml)
      .map_err(|e| PostError::Processing(format!("Failed to re-parse document: {:?}", e)))?;

    // Build parameters
    let params: Vec<(&str, &str)> = self
      .parameters
      .iter()
      .map(|(k, v)| (k.as_str(), v.as_str()))
      .collect();

    // Apply the transformation — libxslt 0.1.3 (post-KWARC upstream bump)
    // takes the source Document by value rather than by reference.
    let result_doc = stylesheet
      .transform(transform_doc, params)
      .map_err(|e| PostError::Processing(format!("XSLT transformation failed: {}", e)))?;

    // Serialize as XML (not as_html). The as_html serializer in libxml2 drops
    // closing tags after void elements like <br>, corrupting span nesting.
    // We fix HTML5-specific issues (self-closing non-void tags, closing void tags)
    // via regex post-processing in the binary's run_post_processing.
    let result_string = result_doc.to_string_with_options(libxml::tree::SaveOptions {
      format:                     false,
      no_declaration:             true, // HTML5: no <?xml version...?> prolog
      no_empty_tags:              false,
      no_xhtml:                   false,
      xhtml:                      false,
      as_xml:                     true,
      as_html:                    false,
      non_significant_whitespace: false,
    });

    if result_string.is_empty() {
      return Err(PostError::Processing(
        "XSLT produced empty output".to_string(),
      ));
    }

    // Create a new PostDocument from the result
    let result_doc = PostDocument::new_from_string(&result_string, PostDocumentOptions {
      destination: doc.destination.clone(),
      destination_directory: doc.destination_directory.clone(),
      site_directory: doc.site_directory.clone(),
      source: doc.source.clone(),
      source_directory: doc.source_directory.clone(),
      searchpaths: Some(doc.searchpaths.clone()),
      ..PostDocumentOptions::default()
    })
    .map_err(|e| PostError::Processing(format!("Failed to parse XSLT result: {}", e)))?;

    Ok(vec![result_doc])
  }
}

// ======================================================================
// Embedded XSLT stylesheets — bundled at compile time for portable binary.
// When the resources/XSLT/ directory is not available on disk, these are
// extracted to a temp directory so libxslt can resolve xsl:import chains.

mod embedded_xslt {
  pub const FILES: &[(&str, &str)] = &[
    (
      "LaTeXML-html5.xsl",
      include_str!("../../resources/XSLT/LaTeXML-html5.xsl"),
    ),
    (
      "LaTeXML-all-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-all-xhtml.xsl"),
    ),
    (
      "LaTeXML-bib-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-bib-xhtml.xsl"),
    ),
    (
      "LaTeXML-block-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-block-xhtml.xsl"),
    ),
    (
      "LaTeXML-common.xsl",
      include_str!("../../resources/XSLT/LaTeXML-common.xsl"),
    ),
    (
      "LaTeXML-epub3.xsl",
      include_str!("../../resources/XSLT/LaTeXML-epub3.xsl"),
    ),
    (
      "LaTeXML-html4.xsl",
      include_str!("../../resources/XSLT/LaTeXML-html4.xsl"),
    ),
    (
      "LaTeXML-inline-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-inline-xhtml.xsl"),
    ),
    (
      "LaTeXML-jats.xsl",
      include_str!("../../resources/XSLT/LaTeXML-jats.xsl"),
    ),
    (
      "LaTeXML-math-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-math-xhtml.xsl"),
    ),
    (
      "LaTeXML-meta-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-meta-xhtml.xsl"),
    ),
    (
      "LaTeXML-misc-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-misc-xhtml.xsl"),
    ),
    (
      "LaTeXML-para-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-para-xhtml.xsl"),
    ),
    (
      "LaTeXML-picture-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-picture-xhtml.xsl"),
    ),
    (
      "LaTeXML-structure-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-structure-xhtml.xsl"),
    ),
    (
      "LaTeXML-tabular-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-tabular-xhtml.xsl"),
    ),
    (
      "LaTeXML-tei.xsl",
      include_str!("../../resources/XSLT/LaTeXML-tei.xsl"),
    ),
    (
      "LaTeXML-webpage-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-webpage-xhtml.xsl"),
    ),
    (
      "LaTeXML-xhtml5.xsl",
      include_str!("../../resources/XSLT/LaTeXML-xhtml5.xsl"),
    ),
    (
      "LaTeXML-xhtml.xsl",
      include_str!("../../resources/XSLT/LaTeXML-xhtml.xsl"),
    ),
  ];

  use std::path::PathBuf;
  use std::sync::OnceLock;

  static EXTRACTED_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

  /// Extract embedded XSLT files to a temp directory.
  /// Returns the directory path, or None if extraction fails.
  pub fn ensure_extracted() -> Option<PathBuf> {
    EXTRACTED_DIR
      .get_or_init(|| {
        let dir = std::env::temp_dir().join("latexml_oxide_xslt");
        if let Err(e) = std::fs::create_dir_all(&dir) {
          log::warn!("Failed to create XSLT temp dir: {e}");
          return None;
        }
        for (name, content) in FILES {
          if let Err(e) = std::fs::write(dir.join(name), content) {
            log::warn!("Failed to write embedded XSLT {name}: {e}");
            return None;
          }
        }
        Some(dir)
      })
      .clone()
  }
}

// ======================================================================
// File search helpers

fn find_stylesheet(stylesheet: &str, searchpaths: &[String]) -> Result<String, PostError> {
  // 1. Check if the stylesheet exists as an absolute/relative path
  if Path::new(stylesheet).is_file() {
    return Ok(stylesheet.to_string());
  }
  // 2. Check each search path
  for sp in searchpaths {
    let p = format!("{}/{}", sp, stylesheet);
    if Path::new(&p).is_file() {
      return Ok(p);
    }
  }
  // 3. Fallback: extract embedded XSLT to temp dir and use that
  if let Some(embedded_dir) = embedded_xslt::ensure_extracted() {
    let filename = Path::new(stylesheet)
      .file_name()
      .and_then(|f| f.to_str())
      .unwrap_or(stylesheet);
    let embedded_path = embedded_dir.join(filename);
    if embedded_path.is_file() {
      return Ok(embedded_path.to_string_lossy().to_string());
    }
    // Also check the full relative path inside the embedded dir
    let full_path = embedded_dir.join(stylesheet);
    if full_path.is_file() {
      return Ok(full_path.to_string_lossy().to_string());
    }
  }
  Err(PostError::Processing(format!(
    "No stylesheet '{}' found!",
    stylesheet
  )))
}

fn find_resource_file(
  src: &str,
  info: Option<&ResourceInfo>,
  search_paths: &[&str],
) -> Option<String> {
  let name = Path::new(src).file_name()?.to_str()?;
  let mut candidates = vec![src.to_string()];
  if let Some(info) = info {
    candidates.push(format!("{}/{}", info.subdir, name));
    candidates.push(format!("{}/{}", info.subdir, src));
  }
  for candidate in &candidates {
    if Path::new(candidate).is_file() {
      return Some(candidate.clone());
    }
    for sp in search_paths {
      let p = format!("{}/{}", sp, candidate);
      if Path::new(&p).is_file() {
        return Some(p);
      }
    }
  }
  None
}

fn relative_path(target: &str, base: &str) -> String {
  let target_path = Path::new(target);
  let base_path = Path::new(base);
  if let Ok(rel) = target_path.strip_prefix(base_path) {
    rel.to_string_lossy().to_string()
  } else {
    target.to_string()
  }
}
