//! XSLT transformation processor.
//!
//! Port of `LaTeXML::Post::XSLT`.
//! Applies an XSLT stylesheet to transform the document (e.g., LaTeXML XML → HTML5).
//! Handles CSS/JS/icon resource copying.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
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
      // Perl XSLT.pm:36 — Error('expected', 'stylesheet', undef,
      //   "No stylesheet specified!")
      log_post_error!(
        "expected", "stylesheet",
        "No stylesheet specified!"
      );
      return Err(PostError::Processing(
        "No stylesheet specified!".to_string(),
      ));
    }

    // Find the stylesheet file
    let stylesheet_path = match find_stylesheet(stylesheet, &searchpaths) {
      Ok(p) => p,
      Err(e) => {
        // Perl XSLT.pm:42 — Error('missing-file', $stylesheet, undef,
        //   "No stylesheet '$stylesheet' found!")
        log_post_error!(
          "missing-file", stylesheet,
          "No stylesheet '{}' found!", stylesheet
        );
        return Err(e);
      }
    };

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
            log_post_warn!(
              "I/O", dest,
              "Couldn't copy {} to {}: {}", path, dest, e
            );
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
        log_post_warn!(
          "missing_file", src,
          "Couldn't find resource file {} in paths {:?}",
          src,
          search_paths
        );
        src.to_string()
      },
    }
  }

  /// Build a per-doc parameter map with `CSS`, `JAVASCRIPT`, and `ICON`
  /// relativized so each split sub-page references the resource at the
  /// correct relative path.
  ///
  /// The raw values are constructed as `"foo.css|bar.css"` (quoted,
  /// pipe-separated basenames) by the binary's `run_post_processing`.
  /// They are interpreted as paths relative to the site root, so
  /// sub-pages need `../foo.css` etc.
  fn relativize_resource_params(&self, doc: &PostDocument) -> HashMap<String, String> {
    let mut out = self.parameters.clone();
    let (Some(site), Some(dest)) = (doc.get_site_directory(), doc.get_destination_directory())
    else {
      return out;
    };
    let prefix = match relative_dir_prefix(site, dest) {
      Some(p) => p,
      None => return out,
    };
    if prefix.is_empty() {
      return out;
    }
    for key in ["CSS", "JAVASCRIPT", "ICON"] {
      if let Some(value) = out.get(key).cloned() {
        out.insert(key.to_string(), relativize_quoted_pipe_list(&value, &prefix));
      }
    }
    out
  }
}

/// Walk-up prefix from `dest_dir` to `site_dir`. Returns `Some("")` when
/// they're identical, `Some("../")` when `dest_dir` is one level deeper,
/// `Some("../../")` two levels, etc. Returns `None` if `dest_dir` is not
/// inside `site_dir`.
fn relative_dir_prefix(site_dir: &str, dest_dir: &str) -> Option<String> {
  let site = Path::new(site_dir);
  let dest = Path::new(dest_dir);
  let rel = dest.strip_prefix(site).ok()?;
  let depth = rel.components().count();
  Some("../".repeat(depth))
}

/// Apply `prefix` to every basename in a `"a|b|c"` quoted pipe-list, but
/// only when the entry doesn't already look absolute or scheme-prefixed.
fn relativize_quoted_pipe_list(value: &str, prefix: &str) -> String {
  let inner = value.trim_matches('"');
  let parts: Vec<String> = inner
    .split('|')
    .map(|p| {
      let p = p.trim();
      if p.is_empty()
        || p.starts_with('/')
        || p.starts_with("./")
        || p.starts_with("../")
        || p.contains("://")
      {
        p.to_string()
      } else {
        format!("{}{}", prefix, p)
      }
    })
    .collect();
  format!("\"{}\"", parts.join("|"))
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

    // Duplicate the libxml Document directly (xmlCopyDoc, C-level memcpy
    // of the tree) instead of serialize-then-reparse. Saves ~5-15 ms on
    // a typical mid-size paper vs the string roundtrip the earlier
    // code used. Required because `stylesheet.transform(...)` consumes
    // its source Document by value.
    let transform_doc = doc.get_document().dup().map_err(|_| {
      PostError::Processing("Failed to duplicate document for XSLT transform".to_string())
    })?;

    // Build parameters, relativizing path-valued ones (CSS, JAVASCRIPT,
    // ICON) for the current doc's destination. The crate-level params
    // hold basenames in site-relative form; split sub-pages live in a
    // subdirectory and need `../foo.css` etc. to resolve correctly.
    let per_doc_params = self.relativize_resource_params(&doc);
    let params: Vec<(&str, &str)> = per_doc_params
      .iter()
      .map(|(k, v)| (k.as_str(), v.as_str()))
      .collect();

    // Apply the transformation — libxslt 0.1.3 (post-KWARC upstream bump)
    // takes the source Document by value rather than by reference.
    let result_doc = stylesheet
      .transform(transform_doc, params)
      .map_err(|e| PostError::Processing(format!("XSLT transformation failed: {}", e)))?;

    // XSLT returns a libxml `Document` directly — wrap it into a
    // PostDocument without the serialize → reparse roundtrip the
    // earlier code did. Saves ~10-30 ms on a typical mid-size paper
    // (XML serialize + libxml2 reparse of ~100-500 KB markup).
    if result_doc.get_root_element().is_none() {
      return Err(PostError::Processing(
        "XSLT produced empty output".to_string(),
      ));
    }

    let result_doc = PostDocument::new(result_doc, PostDocumentOptions {
      destination: doc.destination.clone(),
      destination_directory: doc.destination_directory.clone(),
      site_directory: doc.site_directory.clone(),
      source: doc.source.clone(),
      source_directory: doc.source_directory.clone(),
      searchpaths: Some(doc.searchpaths.clone()),
      ..PostDocumentOptions::default()
    });

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
          log_post_warn!(
            "I/O", "xslt_tempdir",
            "Failed to create XSLT temp dir: {}", e
          );
          return None;
        }
        for (name, content) in FILES {
          if let Err(e) = std::fs::write(dir.join(name), content) {
            log_post_warn!(
              "I/O", name,
              "Failed to write embedded XSLT {}: {}", name, e
            );
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
