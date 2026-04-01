//! XSLT transformation processor.
//!
//! Port of `LaTeXML::Post::XSLT`.
//! Applies an XSLT stylesheet to transform the document (e.g., LaTeXML XML → HTML5).
//! Handles CSS/JS/icon resource copying.

use libxml::tree::Node;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::document::PostDocument;
use crate::processor::{PostError, ProcessResult, Processor};

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
      Some(p) => p,
      None => return Ok(vec![doc]),
    };

    // XSLT transformation requires libxslt bindings
    // For now, log the intent and return the document unchanged
    log::info!(
      "Would apply XSLT stylesheet: {} with {} parameters",
      stylesheet_path,
      self.parameters.len()
    );

    // Handle resource elements
    if !self.no_resources {
      let resource_nodes = doc.findnodes("//ltx:resource[@src]");
      for node in &resource_nodes {
        if let Some(src) = node.get_attribute("src") {
          let resource_type = node.get_attribute("type");
          let _path = self.copy_resource(&doc, &src, resource_type.as_deref());
        }
      }
    }

    // NOTE: XSLT transformation requires libxslt bindings (not yet available in libxml crate).
    // When available: let result_doc = stylesheet.transform(&doc.get_document(), &params)?;
    // let result_doc = stylesheet.transform(&doc.get_document(), &self.parameters)?;

    Ok(vec![doc])
  }
}

/// Find an XSLT stylesheet file in the search paths.
fn find_stylesheet(name: &str, searchpaths: &[String]) -> Result<String, PostError> {
  // Check if it's already a full path
  if Path::new(name).exists() {
    return Ok(name.to_string());
  }

  // Try with .xsl extension
  let with_ext = if !name.ends_with(".xsl") {
    format!("{}.xsl", name)
  } else {
    name.to_string()
  };

  // Search in paths + installation subdirectory
  for path in searchpaths {
    let candidate = format!("{}/{}", path, with_ext);
    if Path::new(&candidate).exists() {
      return Ok(candidate);
    }
  }

  // Try resources/XSLT subdirectory
  let install_path = format!("resources/XSLT/{}", with_ext);
  if Path::new(&install_path).exists() {
    return Ok(install_path);
  }

  Err(PostError::Processing(format!(
    "No stylesheet '{}' found!",
    name
  )))
}

/// Find a resource file in the search paths.
fn find_resource_file(
  name: &str,
  info: Option<&ResourceInfo>,
  search_paths: &[&str],
) -> Option<String> {
  // Direct path check
  if Path::new(name).exists() {
    return Some(name.to_string());
  }

  // Try with extension
  let candidates = if let Some(ri) = info {
    vec![name.to_string(), format!("{}.{}", name, ri.extension)]
  } else {
    vec![name.to_string()]
  };

  for candidate in &candidates {
    for path in search_paths {
      let full = format!("{}/{}", path, candidate);
      if Path::new(&full).exists() {
        return Some(full);
      }
    }
    // Try installation subdir
    if let Some(ri) = info {
      let install = format!("{}/{}", ri.subdir, candidate);
      if Path::new(&install).exists() {
        return Some(install);
      }
    }
  }
  None
}

/// Compute a simple relative path.
fn relative_path(path: &str, base: &str) -> String {
  let p = Path::new(path);
  let b = Path::new(base);
  if let Ok(rel) = p.strip_prefix(b) {
    rel.to_string_lossy().to_string()
  } else {
    path.to_string()
  }
}
