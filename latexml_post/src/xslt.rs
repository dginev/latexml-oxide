//! XSLT transformation processor.
//!
//! Port of `LaTeXML::Post::XSLT`.
//! Applies an XSLT stylesheet to transform the document (e.g., LaTeXML XML → HTML5).
//! Handles CSS/JS/icon resource copying.

use libxml::tree::Node;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::os::raw::c_char;
use std::path::Path;

use crate::document::{PostDocument, PostDocumentOptions};
use crate::processor::{PostError, ProcessResult, Processor};

// libxslt + libexslt FFI bindings
extern "C" {
  fn exsltRegisterAll();
  fn xsltParseStylesheetFile(filename: *const u8) -> *mut XsltStylesheet;
  fn xsltApplyStylesheet(
    style: *mut XsltStylesheet,
    doc: *mut std::ffi::c_void,
    params: *mut *const c_char,
  ) -> *mut std::ffi::c_void;
  fn xsltSaveResultToString(
    doc_txt_ptr: *mut *mut u8,
    doc_txt_len: *mut i32,
    result: *mut std::ffi::c_void,
    style: *mut XsltStylesheet,
  ) -> i32;
  fn xsltFreeStylesheet(style: *mut XsltStylesheet);
  fn xmlFreeDoc(doc: *mut std::ffi::c_void);
}

#[repr(C)]
struct XsltStylesheet {
  _data: [u8; 0],
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
    if !self.no_resources {
      let resource_nodes = doc.findnodes("//ltx:resource[@src]");
      for node in &resource_nodes {
        if let Some(src) = node.get_attribute("src") {
          let resource_type = node.get_attribute("type");
          let _path = self.copy_resource(&doc, &src, resource_type.as_deref());
        }
      }
    }

    // Apply XSLT transformation via libxslt FFI
    let c_stylesheet_path = CString::new(stylesheet_path.as_str())
      .map_err(|e| PostError::Processing(format!("Invalid stylesheet path: {}", e)))?;

    unsafe {
      // Register EXSLT extension functions (if(), etc.)
      exsltRegisterAll();

      // Parse the stylesheet
      let style = xsltParseStylesheetFile(c_stylesheet_path.as_ptr() as *const u8);
      if style.is_null() {
        return Err(PostError::Processing(format!(
          "Failed to parse XSLT stylesheet: {}", stylesheet_path
        )));
      }

      // Build parameters array (null-terminated array of key/value pairs)
      let mut param_c_strings: Vec<CString> = Vec::new();
      let mut param_ptrs: Vec<*const c_char> = Vec::new();
      for (key, value) in &self.parameters {
        let c_key = CString::new(key.as_str()).unwrap();
        let c_value = CString::new(format!("'{}'", value)).unwrap();
        param_ptrs.push(c_key.as_ptr());
        param_ptrs.push(c_value.as_ptr());
        param_c_strings.push(c_key);
        param_c_strings.push(c_value);
      }
      param_ptrs.push(std::ptr::null());

      // Serialize and re-parse for transformation (xsltApplyStylesheet consumes the doc)
      let doc_xml = doc.to_xml_string();

      // Parse a fresh copy for transformation (xsltApplyStylesheet takes ownership)
      let parser = libxml::parser::Parser::default();
      let transform_doc = parser.parse_string(&doc_xml)
        .map_err(|e| PostError::Processing(format!("Failed to re-parse document: {:?}", e)))?;

      // Get the raw pointer (this is platform-specific but works with standard libxml)
      let raw_doc_ptr = transform_doc.doc_ptr() as *mut std::ffi::c_void;

      // Apply the transformation
      let result = xsltApplyStylesheet(style, raw_doc_ptr, param_ptrs.as_mut_ptr());
      if result.is_null() {
        xsltFreeStylesheet(style);
        return Err(PostError::Processing(
          "XSLT transformation failed".to_string()
        ));
      }

      // Serialize the result to string
      let mut txt_ptr: *mut u8 = std::ptr::null_mut();
      let mut txt_len: i32 = 0;
      let ret = xsltSaveResultToString(&mut txt_ptr, &mut txt_len, result, style);

      let result_string = if ret == 0 && !txt_ptr.is_null() && txt_len > 0 {
        let s = String::from_utf8_lossy(
          std::slice::from_raw_parts(txt_ptr, txt_len as usize)
        ).to_string();
        libc::free(txt_ptr as *mut std::ffi::c_void);
        s
      } else {
        // Fallback: try to serialize using xmlDocDumpMemoryEnc
        String::new()
      };

      // Cleanup
      xmlFreeDoc(result);
      xsltFreeStylesheet(style);
      // Don't free transform_doc — it's owned by the libxml Document

      if result_string.is_empty() {
        return Err(PostError::Processing(
          "XSLT produced empty output".to_string()
        ));
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
