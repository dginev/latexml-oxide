//! Stable, high-level conversion API for using `latexml` as a **library**.
//!
//! Downstream Rust crates depend on `latexml` and call one function — no
//! binary, no manual [`Config`] or
//! binding-dispatch wiring:
//!
//! ```no_run
//! let xml = latexml::api::convert_to_xml(r"\documentclass{article}\begin{document}Hi\end{document}")?;
//! let html = latexml::api::convert_to_html(r"\documentclass{article}\begin{document}Hi\end{document}")?;
//! # Ok::<(), String>(())
//! ```
//!
//! ## What these encapsulate
//! The engine is a **thread-local singleton**, so each call runs on its own
//! worker thread with a large (256 MiB) stack — matching the `latexml_oxide`
//! binary — so deeply nested math can't overflow the 8 MiB default stack, and
//! the thread's `#[thread_local]` engine roots (~110 MiB) are released via
//! [`reset_thread_engine`](latexml_core::reset_thread_engine) before the thread
//! exits (those roots do **not** run destructors on a bare thread exit). The
//! standard package + contrib binding-dispatch chain is wired for you.
//!
//! ## Requirements at runtime
//! Same host dependencies as the binary: a **TeX distribution** on `PATH` for
//! packages/classes/fonts, and (only for figure-bearing HTML) the graphics
//! tools. XML/XSLT/RelaxNG assets are embedded.
//!
//! For finer control (preloads, search paths, encoding, whatsin/out, split,
//! …), drive [`crate::converter::Converter`] and [`crate::post`] directly; this
//! module is the batteries-included entrypoint.

use std::rc::Rc;

use latexml_core::common::{Config, OutputFormat};

use crate::{converter::Converter, post};

/// Build the standard library `Config`: quiet, LaTeX mode, with the package +
/// contrib binding-dispatch chain a normal conversion uses.
fn library_config(format: OutputFormat) -> Config {
  Config {
    // Quiet by default: a library caller doesn't want progress notes on stderr.
    // The per-conversion log is still captured internally and returned in the
    // error path. Callers wanting logs can use `Converter` directly.
    verbosity: -1,
    format,
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    ..Config::default()
  }
}

/// Run `job` on a fresh 256 MiB-stack worker thread and free the thread-local
/// engine before the thread exits. Mirrors `latexml_oxide::main`'s worker
/// thread and `util::test`'s per-conversion `reset_thread_engine()`.
fn on_worker<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static) -> T {
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let out = job();
      // `#[thread_local]` engine roots don't Drop on thread exit; free the
      // ~110 MiB explicitly so repeated calls don't accumulate.
      latexml_core::reset_thread_engine();
      out
    })
    .expect("spawn latexml worker thread")
    .join()
    .expect("latexml worker thread panicked")
}

/// Convert a TeX/LaTeX source string to LaTeXML **XML** (the intermediate
/// representation, before any HTML post-processing).
///
/// Returns the serialized XML on success, or the captured conversion log on
/// failure (a fatal error, or an engine that could not initialize).
pub fn convert_to_xml(tex: &str) -> Result<String, String> {
  let tex = tex.to_string();
  on_worker(move || {
    let opts = library_config(OutputFormat::XML);
    let mut converter = Converter::from_config(opts.clone());
    converter.note_main_source(&format!("literal:{tex}"));
    converter
      .prepare_session(&opts)
      .map_err(|e| format!("could not prepare session: {e}"))?;
    let response = converter.convert(format!("literal:{tex}"));
    response
      .result
      .ok_or_else(|| format!("conversion failed:\n{}", response.log))
  })
}

/// Convert a TeX/LaTeX source string all the way to a standalone **HTML5**
/// document (LaTeXML XML + the HTML post-processing pipeline, with
/// Presentation MathML).
///
/// Figures are left as raw `<ltx:graphics>` references — image conversion
/// writes files next to a destination, and this string-returning API has none;
/// use [`crate::post`] with a `destination` when you need converted images.
///
/// The page links the bundled CSS/JS, and with no destination those files are
/// copied into the working directory; [`convert_to_html_with`] and
/// [`HtmlOptions`] choose otherwise.
pub fn convert_to_html(tex: &str) -> Result<String, String> {
  convert_to_html_with(tex, &HtmlOptions::default())
}

/// The settings [`convert_to_html_with`] takes. [`Default`] is what
/// [`convert_to_html`] does. Construct with `..Default::default()`, so that a
/// setting added later leaves existing callers compiling:
///
/// ```no_run
/// use latexml::api::{HtmlOptions, convert_to_html_with};
/// let options = HtmlOptions { default_resources: false, ..Default::default() };
/// let html = convert_to_html_with(r"\documentclass{article}\begin{document}Hi\end{document}", &options)?;
/// # Ok::<(), String>(())
/// ```
#[derive(Clone, Debug)]
pub struct HtmlOptions {
  /// Link and copy the bundled CSS/JS (`true`, Perl `defaultresources`). With no
  /// destination the files land in the working directory; `false` is the
  /// `--nodefaultresources` equivalent: no links to them, nothing written.
  pub default_resources: bool,
}

impl Default for HtmlOptions {
  fn default() -> Self { HtmlOptions { default_resources: true } }
}

/// [`convert_to_html`] with the given [`HtmlOptions`].
pub fn convert_to_html_with(tex: &str, options: &HtmlOptions) -> Result<String, String> {
  let tex = tex.to_string();
  let options = options.clone();
  on_worker(move || {
    let opts = library_config(OutputFormat::HTML5);
    let mut converter = Converter::from_config(opts.clone());
    converter.note_main_source(&format!("literal:{tex}"));
    converter
      .prepare_session(&opts)
      .map_err(|e| format!("could not prepare session: {e}"))?;
    let xml = converter
      .convert(format!("literal:{tex}"))
      .result
      .ok_or_else(|| "conversion failed before post-processing".to_string())?;

    let post_opts = html_post_options(options.default_resources);
    post_result(post::run_post_processing_logged(&xml, &post_opts))
  })
}

/// The post-processing [`convert_to_html_with`] runs: the CLI's `html5` defaults, no destination.
fn html_post_options(default_resources: bool) -> post::PostOptions<'static> {
  post::PostOptions {
    pmml: true,
    cmml: false,
    keep_xmath: false,
    // Same per-format sheet the CLI uses (shared source of truth).
    stylesheet: post::default_stylesheet(Some("html5")),
    destination: None,
    source_directory: None,
    site_directory: None,
    search_paths: &[],
    default_resources,
    css_files: &[],
    js_files: &[],
    noinvisibletimes: false,
    plane1: true,
    hackplane1: false,
    mathtex: false,
    url_style: latexml_post::crossref::UrlStyle::File,
    navigationtoc: None,
    schemadocs: false,
    split: false,
    split_xpath: None,
    split_naming: None,
    xslt_parameters: &[],
    graphics_svg_threshold_kb: 0,
    // No destination to write images to; keep the raw graphics references.
    graphicimages: false,
    timestamp: None,
    icon: None,
    whatsout: latexml_post::extract::Whatsout::Document,
  }
}

/// The HTML, or post's log when post-processing died (`Fatal:post:convert`, #782) and left no
/// document, the CLI's test. The status also counts core's: a core Fatal's rescued document is
/// post-processed and returned, as in Perl's convert (LaTeXML.pm L277-337).
fn post_result(post: post::PostOutcome) -> Result<String, String> {
  if post.html.is_empty() && post.status_code >= 3 {
    Err(format!("post-processing failed:\n{}", post.log))
  } else {
    Ok(post.html)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const DOC: &str = r"\documentclass{article}\begin{document}Hello \(x^2\)\end{document}";

  #[test]
  fn xml_conversion_returns_content() {
    let xml = convert_to_xml(DOC).expect("convert_to_xml");
    assert!(
      xml.contains("Hello"),
      "XML should contain the body text: {xml}"
    );
    assert!(
      xml.contains("<?xml") || xml.contains("<document"),
      "looks like XML"
    );
  }

  /// #782: a post Fatal (here a sheet that terminates) is an `Err` carrying the log, not `Ok("")`.
  #[test]
  fn a_post_fatal_is_an_err() {
    let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
    let dir = tempfile::tempdir().expect("tempdir");
    let xsl = dir.path().join("terminate.xsl");
    std::fs::write(
      &xsl,
      "<xsl:stylesheet version=\"1.0\" xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\">\
       <xsl:template match=\"/\"><xsl:message terminate=\"yes\">STOP</xsl:message></xsl:template>\
       </xsl:stylesheet>",
    )
    .unwrap();
    let xsl = xsl.to_string_lossy().into_owned();
    let xml = "<?xml version=\"1.0\"?><document xmlns=\"http://dlmf.nist.gov/LaTeXML\"><para><p>Hi</p></para></document>";
    let result = on_worker(move || {
      post_result(post::run_post_processing_logged(xml, &post::PostOptions {
        stylesheet: Some(&xsl),
        ..html_post_options(false)
      }))
    });
    let log = result.expect_err("a post Fatal is an Err");
    assert_eq!(log.matches("Fatal:post:convert").count(), 1, "{log}");
  }

  /// A core Fatal that keeps its partial document (too many errors) is still post-processed and
  /// returned, as Perl's convert and the CLI do (LaTeXML.pm L277-337); the Fatal is in the core log.
  #[test]
  fn a_core_fatal_keeps_its_rescued_document() {
    let tex = format!(
      "\\documentclass{{article}}\\begin{{document}}Kept text.\n\n{}\\end{{document}}",
      (0..101u8)
        .map(|i| format!(
          "\\nosuch{}{} ",
          (b'a' + i / 26) as char,
          (b'a' + i % 26) as char
        ))
        .collect::<String>()
    );
    let (core_log, result) = on_worker(move || {
      let opts = library_config(OutputFormat::HTML5);
      let mut converter = Converter::from_config(opts.clone());
      converter.prepare_session(&opts).expect("session");
      let response = converter.convert(format!("literal:{tex}"));
      let xml = response.result.expect("the rescued document");
      let post = post_result(post::run_post_processing_logged(
        &xml,
        &html_post_options(false),
      ));
      ((response.status_code, response.log), post)
    });
    assert_eq!(core_log.0, 3, "{}", core_log.1);
    assert!(
      core_log.1.contains("101 errors; 1 fatal error;"),
      "{}",
      core_log.1
    );
    let html = result.expect("a core Fatal's rescued document is Ok");
    assert!(html.contains("Kept text."), "{html}");
  }

  #[test]
  fn html_conversion_returns_page() {
    let html = convert_to_html_with(DOC, &HtmlOptions { default_resources: false })
      .expect("convert_to_html_with");
    assert!(html.contains("Hello"), "HTML should contain the body text");
    assert!(
      html.contains("<math") || html.contains("ltx_Math"),
      "math rendered"
    );
  }
}
