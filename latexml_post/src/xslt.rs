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

// EXSLT registration — needed for str:tokenize, math:*, etc. used in LaTeXML stylesheets.
// The rust-libxslt crate doesn't yet expose this.
// TODO: Add exsltRegisterAll() to the rust-libxslt crate.
extern "C" {
  fn exsltRegisterAll();
}

/// Resource type information.
struct ResourceInfo {
  extension: &'static str,
  subdir: &'static str,
}

const RESOURCE_CSS: ResourceInfo = ResourceInfo {
  extension: "css",
  subdir: "resources/CSS",
};
const RESOURCE_JS: ResourceInfo = ResourceInfo {
  extension: "js",
  subdir: "resources/javascript",
};

/// XSLT post-processor: applies a stylesheet transformation.
///
/// Port of `LaTeXML::Post::XSLT`.
pub struct XSLT {
  name: String,
  /// Path to the XSLT stylesheet.
  stylesheet_path: Option<String>,
  /// Parameters to pass to the XSLT stylesheet.
  parameters: HashMap<String, String>,
  /// Whether to remove resource requests (CSS/JS not copied).
  no_resources: bool,
  /// Resource directory for copied resources.
  resource_directory: Option<String>,
  /// Search paths for finding resources.
  searchpaths: Vec<String>,
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
  fn copy_resource(
    &self,
    doc: &PostDocument,
    src: &str,
    resource_type: Option<&str>,
  ) -> String {
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
      }
      None => {
        log::warn!(
          "Couldn't find resource file {} in paths {:?}",
          src,
          search_paths
        );
        src.to_string()
      }
    }
  }
}

impl Processor for XSLT {
  fn get_name(&self) -> &str {
    &self.name
  }

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

    // Register EXSLT extension functions (str:tokenize, etc.)
    // TODO: Move to rust-libxslt crate (L3)
    unsafe { exsltRegisterAll(); }

    // Parse the stylesheet using the libxslt crate
    let mut stylesheet = libxslt::parser::parse_file(&stylesheet_path)
      .map_err(|e| PostError::Processing(format!("Failed to parse XSLT stylesheet: {}", e)))?;

    // Serialize and re-parse for transformation (transform consumes the doc)
    let doc_xml = doc.to_xml_string();
    let parser = libxml::parser::Parser::default();
    let transform_doc = parser.parse_string(&doc_xml)
      .map_err(|e| PostError::Processing(format!("Failed to re-parse document: {:?}", e)))?;

    // Build parameters
    let params: Vec<(&str, &str)> = self.parameters.iter()
      .map(|(k, v)| (k.as_str(), v.as_str()))
      .collect();

    // Apply the transformation
    let result_doc = stylesheet.transform(&transform_doc, params)
      .map_err(|e| PostError::Processing(format!("XSLT transformation failed: {}", e)))?;

    // Serialize as XML (not as_html). The as_html serializer in libxml2 drops
    // closing tags after void elements like <br>, corrupting span nesting.
    // We fix HTML5-specific issues (self-closing non-void tags, closing void tags)
    // via regex post-processing in the binary's run_post_processing.
    let result_string = result_doc.to_string_with_options(libxml::tree::SaveOptions {
      format: false,
      no_declaration: false,
      no_empty_tags: false,
      no_xhtml: false,
      xhtml: false,
      as_xml: true,
      as_html: false,
      non_significant_whitespace: false,
    });

    if result_string.is_empty() {
      return Err(PostError::Processing("XSLT produced empty output".to_string()));
    }

    // Create a new PostDocument from the result
    let result_doc = PostDocument::new_from_string(
      &result_string,
      PostDocumentOptions {
        destination: doc.destination.clone(),
        destination_directory: doc.destination_directory.clone(),
        site_directory: doc.site_directory.clone(),
        source: doc.source.clone(),
        source_directory: doc.source_directory.clone(),
        searchpaths: Some(doc.searchpaths.clone()),
        ..PostDocumentOptions::default()
      },
    ).map_err(|e| PostError::Processing(format!("Failed to parse XSLT result: {}", e)))?;

    Ok(vec![result_doc])
  }
}

// ======================================================================
// File search helpers

fn find_stylesheet(stylesheet: &str, searchpaths: &[String]) -> Result<String, PostError> {
  if Path::new(stylesheet).is_file() {
    return Ok(stylesheet.to_string());
  }
  for sp in searchpaths {
    let p = format!("{}/{}", sp, stylesheet);
    if Path::new(&p).is_file() {
      return Ok(p);
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
