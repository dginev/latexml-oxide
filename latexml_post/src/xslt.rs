//! XSLT transformation processor.
//!
//! Port of `LaTeXML::Post::XSLT`.
//! Applies an XSLT stylesheet to transform the document (e.g., LaTeXML XML → HTML5).
//! Handles CSS/JS/icon resource copying.

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::cell::RefCell;
use std::fs;
use std::path::Path;

use crate::document::{PostDocument, PostDocumentOptions};
use crate::processor::{PostError, ProcessResult, Processor};

/// Set libxslt's global template-recursion cap to Perl's value (1000) exactly
/// once per process. Mirrors `XML::LibXSLT->max_depth(1000)` in
/// `LaTeXML::Post::XSLT`. Prevents deeply-recursive stylesheet templates from
/// exhausting the C call stack (SIGSEGV) or RAM on pathological documents,
/// aborting the transform gracefully like Perl instead.
fn set_xslt_max_depth() {
  static SET_MAX_DEPTH: std::sync::Once = std::sync::Once::new();
  SET_MAX_DEPTH.call_once(|| {
    // SAFETY: `xsltMaxDepth` is libxslt's process-global recursion cap
    // (a plain C `int`). The libxslt crate exposes no safe setter. `Once`
    // guarantees a single writer; libxslt only ever READS this value (when
    // creating each transform context), so there is no data race with
    // concurrent transforms.
    //
    // PORTABILITY: resolved via `dlsym` rather than the crate's
    // `libxslt::bindings::xsltMaxDepth` extern static. Those pregenerated
    // (bindgen-on-Linux) bindings pin the raw ELF symbol name with
    // `#[link_name = "\u{1}xsltMaxDepth"]`, which fails to LINK on Mach-O
    // where the C symbol is `_xsltMaxDepth` (macOS probe 2026-06-07 — the
    // sole undefined symbol in the whole workspace link; see
    // docs/PORTABILITY_MACOS_PROBE_2026-06-07.md). `dlsym` applies the
    // platform's own C-symbol decoration, so it works on ELF and Mach-O
    // alike. If the symbol is ever absent (NULL), we skip the write:
    // libxslt's built-in default cap of 3000 still bounds recursion.
    unsafe {
      let sym = libc::dlsym(libc::RTLD_DEFAULT, c"xsltMaxDepth".as_ptr());
      if !sym.is_null() {
        *(sym as *mut std::os::raw::c_int) = 1000;
      }
    }
  });
}

#[cfg(test)]
mod max_depth_tests {
  /// The dlsym write must actually land: after `set_xslt_max_depth`,
  /// reading the global back through the same runtime resolution path
  /// must yield Perl's value (1000). Guards both the symbol lookup
  /// (platform decoration) and the write.
  #[test]
  fn dlsym_sets_perl_parity_cap() {
    super::set_xslt_max_depth();
    let val = unsafe {
      let sym = libc::dlsym(libc::RTLD_DEFAULT, c"xsltMaxDepth".as_ptr());
      assert!(!sym.is_null(), "xsltMaxDepth not resolvable via dlsym");
      *(sym as *const std::os::raw::c_int)
    };
    assert_eq!(val, 1000);
  }
}

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
      Error!(
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
        Error!(
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

    let basename = Path::new(src)
      .file_name()
      .and_then(|f| f.to_str())
      .unwrap_or(src);

    // Determine destination once — same logic regardless of whether
    // the resource ends up on disk or comes from the embedded table.
    let dest = if let Some(ref rd) = self.resource_directory {
      if let Some(site_dir) = doc.get_site_directory() {
        format!("{}/{}/{}", site_dir, rd, basename)
      } else {
        format!("{}/{}", rd, basename)
      }
    } else if let Some(dest_dir) = doc.get_destination_directory() {
      format!("{}/{}", dest_dir, basename)
    } else {
      basename.to_string()
    };
    let ensure_parent = |dest: &str| {
      if let Some(parent) = Path::new(dest).parent() {
        let _ = fs::create_dir_all(parent);
      }
    };

    match find_resource_file(src, info, &search_paths) {
      Some(path) => {
        // Found on disk via searchpath. Copy unless source == dest.
        if path != dest {
          ensure_parent(&dest);
          if let Err(e) = fs::copy(&path, &dest) {
            Warn!(
              "I/O", dest,
              "Couldn't copy {} to {}: {}", path, dest, e
            );
          }
        }
      },
      None => {
        // Not on disk — try the embedded table. CSS/JS assets are
        // baked into the binary at build time; we materialize them
        // straight to the destination, no temp dir round-trip.
        if let Some(bytes) = embedded_resources::lookup(basename) {
          ensure_parent(&dest);
          if let Err(e) = fs::write(&dest, bytes) {
            Warn!(
              "I/O", dest,
              "Couldn't write embedded resource {} to {}: {}", basename, dest, e
            );
          }
        } else {
          Warn!(
            "missing_file", src,
            "Couldn't find resource file {} in paths {:?}",
            src,
            search_paths
          );
          return src.to_string();
        }
      },
    }

    // Return path relative to destination directory.
    if let Some(dest_dir) = doc.get_destination_directory() {
      relative_path(&dest, dest_dir)
    } else {
      dest
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

    Info!("xslt", "stylesheet", "Applying XSLT stylesheet: {}", stylesheet_path);

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

    // Faithful port of Perl `XML::LibXSLT->max_depth(1000)`
    // (LaTeXML::Post::XSLT.pm L48): cap libxslt's template-recursion depth.
    // libxslt's compiled-in default is 3000; lowering to Perl's 1000 makes a
    // runaway / deeply-recursive stylesheet apply ABORT gracefully (matching
    // Perl) instead of growing the C call stack until SIGSEGV/OOM on
    // pathological input. `xsltMaxDepth` is a process-global libxslt static
    // read when each transform context is created, so setting it once is
    // sufficient. See docs/STABILITY_WITNESSES.md (Cluster A, hypothesis 3).
    set_xslt_max_depth();

    // Hand the source tree to libxslt WITHOUT a deep copy. `transform()`
    // takes its `Document` by value (the moved handle's Drop would free the
    // tree), so earlier code `dup()`'d (xmlCopyDoc — a full deep copy) to keep
    // this PostDocument's own tree alive. On a large-math document the DOM is
    // multi-GB, and that deep copy TRANSIENTLY DOUBLES peak RSS during the
    // transform — the dominant driver of post-processing OOM on the canvas
    // sweep (docs/STABILITY_WITNESSES.md, Cluster A / hypothesis 1).
    //
    // `libxml::Document` is `Rc<RefCell<_Document>>` and the underlying
    // `xmlDoc` is freed only when the LAST handle drops, so an Rc `clone()`
    // (a refcount bump, no copy) is the right tool: libxslt reads the shared
    // tree to build a SEPARATE result tree (`xsltApplyStylesheetUser` does not
    // free its source), the moved clone's Drop just decrements the count, and
    // `doc`'s own handle (dropped at function end) performs the single real
    // free. We never read `doc`'s tree again after this point — only its
    // string metadata below — so libxslt mutating the shared source while
    // applying is harmless. This mirrors Perl, which passes
    // `$doc->getDocument` straight to `transform` with no pre-copy
    // (LaTeXML::Post::XSLT.pm L79).
    let transform_doc = doc.get_document().clone();

    // Build parameters, relativizing path-valued ones (CSS, JAVASCRIPT,
    // ICON) for the current doc's destination. The crate-level params
    // hold basenames in site-relative form; split sub-pages live in a
    // subdirectory and need `../foo.css` etc. to resolve correctly.
    let per_doc_params = self.relativize_resource_params(&doc);
    let params: Vec<(&str, &str)> = per_doc_params
      .iter()
      .map(|(k, v)| (k.as_str(), v.as_str()))
      .collect();

    // Apply the transformation. The parsed `Stylesheet` lives in a
    // per-thread cache (`with_cached_stylesheet`) — `libxslt::parser::
    // parse_file` runs once per (thread, stylesheet path) instead of
    // once per conversion. The cache is thread-local, not shared, so
    // we don't lean on libxslt's undocumented thread-safety (mirroring
    // the caution that resolved KWARC/rust-libxslt issue #6).
    let result_doc = with_cached_stylesheet(&stylesheet_path, |stylesheet| {
      stylesheet
        .transform(transform_doc, params)
        .map_err(|e| PostError::Processing(format!("XSLT transformation failed: {}", e)))
    })?;

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
// Per-thread cache of parsed stylesheets.
//
// `libxslt::parser::parse_file` reads the .xsl from disk and compiles
// it. For LaTeXML-html5.xsl that's ~5–10 ms including its xsl:imports.
// On a single CLI run that's once per process — fine. On a daemon-mode
// `cortex_worker` chewing through 10 000 papers from a thread pool of
// 8 workers, naive code re-parses once per paper. With this cache,
// each worker thread parses each unique stylesheet path *once* and
// reuses the compiled artefact for the rest of its lifetime.
//
// ## Why thread-local (and not process-wide + Arc/Mutex)?
//
// libxslt is not documented as thread-safe. `xsltApplyStylesheetUser`
// is not audited to be read-only on the stylesheet — it may write
// back into namespace-internalisation caches, error context fields,
// or other internal state. This is the same kind of hidden mutation
// that issue KWARC/rust-libxslt#6 punctured for the input `Document`
// (libxslt silently mutates docs during whitespace stripping). A
// process-wide cache shared across worker threads via `Arc` would
// either need a `Mutex` (serialising transforms — defeats the
// throughput benefit) or rely on libxslt's undocumented thread-safety
// (the same bet that #6 retired).
//
// Thread-local keeps the safety story simple: each thread owns its
// own `Stylesheet` for its lifetime, no cross-thread sharing, and the
// `&mut Stylesheet` requirement is satisfied by `RefCell::borrow_mut`.
// Worst case: 8 worker threads × 1 parse per unique stylesheet =
// 8 parses per process, instead of N parses per N papers.

fn cache_key(path: &str) -> String {
  // Canonicalise so `./resources/XSLT/foo.xsl` and
  // `/abs/.../resources/XSLT/foo.xsl` hit the same entry. Falls back
  // to the raw path on canonicalisation failure (the file might not
  // exist yet — let parse_file emit its own error in that case).
  std::fs::canonicalize(path)
    .map(|p| p.to_string_lossy().into_owned())
    .unwrap_or_else(|_| path.to_string())
}

thread_local! {
  static STYLESHEET_CACHE: RefCell<HashMap<String, libxslt::stylesheet::Stylesheet>> =
    RefCell::new(HashMap::default());
}

/// Borrow a `&mut Stylesheet` from the per-thread cache, parsing on
/// miss. The closure runs while the cache is mutably borrowed, so
/// nested calls (which the LaTeXML pipeline never makes) would
/// `RefCell::borrow_mut`-panic — a deliberate single-borrow contract.
fn with_cached_stylesheet<F, R>(path: &str, f: F) -> Result<R, PostError>
where
  F: FnOnce(&mut libxslt::stylesheet::Stylesheet) -> Result<R, PostError>,
{
  let key = cache_key(path);
  STYLESHEET_CACHE.with(|cache| {
    let mut map = cache.borrow_mut();
    if !map.contains_key(&key) {
      let parsed = if let Some(name) = path.strip_prefix(embedded_xslt::URL_PREFIX) {
        // `embed:///<name>` sentinel from `find_stylesheet`. Parse the
        // root stylesheet from the embedded byte table; libxslt's
        // `xsl:import` machinery will then re-enter our libxml2 input
        // callback for every referenced URL.
        let bytes = embedded_xslt::lookup(name).ok_or_else(|| {
          PostError::Processing(format!("Embedded XSLT stylesheet {} not found", name))
        })?;
        libxslt::parser::parse_bytes(bytes.to_vec(), path)
          .map_err(|e| PostError::Processing(format!("Failed to parse embedded XSLT: {}", e)))?
      } else {
        libxslt::parser::parse_file(path)
          .map_err(|e| PostError::Processing(format!("Failed to parse XSLT stylesheet: {}", e)))?
      };
      map.insert(key.clone(), parsed);
    }
    let entry = map
      .get_mut(&key)
      .expect("cache entry just inserted is missing");
    f(entry)
  })
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

  use std::sync::OnceLock;

  /// URL scheme through which our embedded stylesheets are served to
  /// libxslt. Any URL starting with this prefix is intercepted by the
  /// input callback we install in [`install_callback_once`] and
  /// resolved against the [`FILES`] table.
  pub const URL_PREFIX: &str = "embed:///";

  /// Look up the embedded XSLT bytes by basename, or `None` if the
  /// stylesheet is not bundled.
  pub fn lookup(name: &str) -> Option<&'static [u8]> {
    FILES
      .iter()
      .find_map(|(n, c)| (*n == name).then_some(c.as_bytes()))
  }

  /// Install the libxml2 input callback that serves `embed:///`
  /// URLs from [`FILES`]. Called once per process; subsequent calls
  /// are no-ops. The callback fires whenever libxml2 itself opens a
  /// URL — including `xsl:import` / `xsl:include` resolution from
  /// inside `libxslt::parser::parse_bytes`. Result: every stylesheet
  /// (root + imports) is loaded from the binary's own `.rodata`
  /// section, no disk extraction required.
  pub fn install_callback_once() {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    INSTALLED.get_or_init(|| {
      libxml::io::register_input_callback(
        |url| url.starts_with(URL_PREFIX),
        |url| {
          let name = url.strip_prefix(URL_PREFIX)?;
          lookup(name).map(|s| s.to_vec())
        },
      );
    });
  }
}

// ======================================================================
// Embedded CSS / JavaScript resources — bundled at compile time so a
// single-binary distribution can serve them without an accompanying
// `resources/` tree on disk.
//
// Unlike XSLT (which libxslt needs as files on disk to resolve
// `xsl:import` chains), CSS and JS are pure leaf assets — the
// post-processor's job is to put a copy next to the output HTML so
// `<link rel="stylesheet">` resolves. We can write the embedded
// bytes straight to the destination directory, skipping the
// extract-to-temp-then-copy round-trip entirely.

mod embedded_resources {
  pub const CSS_FILES: &[(&str, &str)] = &[
    (
      "LaTeXML-blue.css",
      include_str!("../../resources/CSS/LaTeXML-blue.css"),
    ),
    (
      "LaTeXML-marginpar.css",
      include_str!("../../resources/CSS/LaTeXML-marginpar.css"),
    ),
    (
      "LaTeXML-navbar-left.css",
      include_str!("../../resources/CSS/LaTeXML-navbar-left.css"),
    ),
    (
      "LaTeXML-navbar-right.css",
      include_str!("../../resources/CSS/LaTeXML-navbar-right.css"),
    ),
    (
      "LaTeXML.css",
      include_str!("../../resources/CSS/LaTeXML.css"),
    ),
    (
      "ltx-amsart.css",
      include_str!("../../resources/CSS/ltx-amsart.css"),
    ),
    (
      "ltx-apj.css",
      include_str!("../../resources/CSS/ltx-apj.css"),
    ),
    (
      "ltx-article.css",
      include_str!("../../resources/CSS/ltx-article.css"),
    ),
    (
      "ltx-book.css",
      include_str!("../../resources/CSS/ltx-book.css"),
    ),
    (
      "ltx-listings.css",
      include_str!("../../resources/CSS/ltx-listings.css"),
    ),
    (
      "ltx-report.css",
      include_str!("../../resources/CSS/ltx-report.css"),
    ),
    (
      "ltx-svjour.css",
      include_str!("../../resources/CSS/ltx-svjour.css"),
    ),
    (
      "ltx-ulem.css",
      include_str!("../../resources/CSS/ltx-ulem.css"),
    ),
    (
      "relaxng-schema-rustdoc-theme.css",
      include_str!("../../resources/CSS/relaxng-schema-rustdoc-theme.css"),
    ),
  ];

  pub const JS_FILES: &[(&str, &str)] = &[
    (
      "LaTeXML-maybeMathjax.js",
      include_str!("../../resources/javascript/LaTeXML-maybeMathjax.js"),
    ),
    (
      "relaxng-schema-rustdoc-theme.js",
      include_str!("../../resources/javascript/relaxng-schema-rustdoc-theme.js"),
    ),
  ];

  /// Return the embedded bytes for `basename` if it's one of the
  /// bundled CSS/JS assets, or `None` otherwise. Callers write the
  /// returned slice straight to the destination directory — no temp
  /// dir, no intermediate copy.
  pub fn lookup(basename: &str) -> Option<&'static [u8]> {
    CSS_FILES
      .iter()
      .chain(JS_FILES.iter())
      .find_map(|(n, c)| (*n == basename).then_some(c.as_bytes()))
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
  // 3. Fallback: serve from the embedded table via the libxml2 input
  //    callback. We return an `embed:///<basename>` URL sentinel that
  //    `with_cached_stylesheet` routes through `libxslt::parser::
  //    parse_bytes`; subsequent `xsl:import` references inside that
  //    stylesheet compose against this base URI and re-enter our
  //    callback, so the whole chain stays in memory.
  let filename = Path::new(stylesheet)
    .file_name()
    .and_then(|f| f.to_str())
    .unwrap_or(stylesheet);
  if embedded_xslt::lookup(filename).is_some() {
    embedded_xslt::install_callback_once();
    return Ok(format!("{}{}", embedded_xslt::URL_PREFIX, filename));
  }
  Err(PostError::Processing(format!(
    "No stylesheet '{}' found!",
    stylesheet
  )))
}

/// Disk-only lookup for a CSS/JS/icon resource — searches the literal
/// path, then `info.subdir`-prefixed variants, then each `search_paths`
/// entry. Embedded (compile-time-bundled) assets are handled by the
/// caller via `embedded_resources::lookup`; this function deliberately
/// does NOT check the embed, so on-disk overrides always win and the
/// "couldn't find" branch can fall through to the embed cleanly.
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
