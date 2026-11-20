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
//! Like the `latexml_oxide` binary, these calls interpret a `.sty`/`.cls` that has no binding raw
//! (Perl LaTeXML's `--includestyles`) and use the arXiv profile's runaway limits; a caller wanting
//! Perl LaTeXML's binding-only default drives a [`Converter`] with its
//! own [`Config`] (`include_styles: Some(false)`; OXIDIZED_DESIGN_DIVERGENCES #1080).
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

use crate::{
  converter::{ConversionResponse, Converter},
  post,
};

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
    // The CLI's defaults: raw `.sty`/`.cls` loading and ar5iv.sty's runaway limits (OD #1080).
    include_styles: Some(true),
    if_limit: Some(latexml_core::common::RUNAWAY_IF_LIMIT),
    absorb_limit: Some(latexml_core::common::RUNAWAY_ABSORB_LIMIT),
    ..Config::default()
  }
}

/// Run `job` on a fresh 256 MiB-stack worker thread and free the thread-local
/// engine before the thread exits (even if `job` panics). Mirrors
/// `latexml_oxide::main`'s worker thread and `util::test`'s per-conversion
/// `reset_thread_engine()`. Returns `Err` on spawn failure or worker panic
/// rather than crashing the library caller.
fn on_worker<T: Send + 'static>(
  job: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
  let handle = std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      // The CLI's pushback cap (ar5iv.sty's `pushbacklimit`), on the thread whose gullet converts.
      if std::env::var_os("LATEXML_NO_DEFAULT_PUSHBACK_LIMIT").is_none() {
        latexml_core::gullet::set_pushback_limit(Some(5_000_000));
      }
      // `#[thread_local]` engine roots don't Drop on thread exit; free the
      // ~110 MiB explicitly (including on unwind) so repeated calls don't accumulate.
      struct EngineResetGuard;
      impl Drop for EngineResetGuard {
        fn drop(&mut self) { latexml_core::reset_thread_engine(); }
      }
      let _guard = EngineResetGuard;
      job()
    })
    .map_err(|e| format!("failed to spawn latexml worker thread: {e}"))?;
  match handle.join() {
    Ok(result) => result,
    Err(payload) => {
      let msg = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic");
      Err(format!("latexml worker thread panicked: {msg}"))
    },
  }
}

/// Extract the converted XML from a [`ConversionResponse`], preserving the
/// captured conversion log in the `Err` message when conversion failed.
fn core_result_xml(response: ConversionResponse, prefix: &str) -> Result<String, String> {
  response.result.ok_or_else(|| {
    if response.log.is_empty() {
      prefix.to_string()
    } else {
      format!("{prefix}:\n{}", response.log)
    }
  })
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
    core_result_xml(response, "conversion failed")
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
/// [`convert_to_html`] does. Start from [`HtmlOptions::default()`] and use its
/// builder methods so future options remain backwards-compatible:
///
/// ```no_run
/// use latexml::api::{HtmlOptions, convert_to_html_with};
/// let options = HtmlOptions::default().with_default_resources(false);
/// let html = convert_to_html_with(r"\documentclass{article}\begin{document}Hi\end{document}", &options)?;
/// # Ok::<(), String>(())
/// ```
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct HtmlOptions {
  /// Link and copy the bundled CSS/JS (`true`, Perl `defaultresources`). With no
  /// destination the files land in the working directory; `false` is the
  /// `--nodefaultresources` equivalent: no links to them, nothing written.
  pub default_resources: bool,
}

impl Default for HtmlOptions {
  fn default() -> Self { HtmlOptions { default_resources: true } }
}

impl HtmlOptions {
  /// Set whether to link and copy the bundled CSS/JS resources (`--defaultresources` /
  /// `--nodefaultresources`).
  pub fn with_default_resources(mut self, default_resources: bool) -> Self {
    self.default_resources = default_resources;
    self
  }
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
    let xml = core_result_xml(
      converter.convert(format!("literal:{tex}")),
      "conversion failed before post-processing",
    )?;

    let post_opts = html_post_options(options.default_resources);
    post_result(post::run_post_processing_logged(&xml, &post_opts))
  })
}

/// The post-processing [`convert_to_html_with`] runs: the CLI's `html5` defaults, no destination.
fn html_post_options(default_resources: bool) -> post::PostOptions<'static> {
  post::PostOptions {
    pmml: true,
    // Same per-format sheet the CLI uses (shared source of truth).
    stylesheet: post::default_stylesheet(Some("html5")),
    default_resources,
    // No destination to write images to; keep the raw graphics references.
    graphicimages: false,
    ..post::PostOptions::default()
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
      Ok(((response.status_code, response.log), post))
    })
    .expect("worker thread");
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
    let html = convert_to_html_with(DOC, &HtmlOptions::default().with_default_resources(false))
      .expect("convert_to_html_with");
    assert!(html.contains("Hello"), "HTML should contain the body text");
    assert!(
      html.contains("<math") || html.contains("ltx_Math"),
      "math rendered"
    );
  }

  #[test]
  fn worker_panic_returns_err_instead_of_crashing_caller() {
    let err =
      on_worker::<()>(|| panic!("boom in worker")).expect_err("worker panic should surface as Err");
    assert!(
      err.contains("latexml worker thread panicked: boom in worker"),
      "unexpected panic error: {err}"
    );
  }

  #[test]
  fn core_failure_preserves_conversion_log() {
    let response = ConversionResponse {
      result:      None,
      log:         "Fatal:foo:bar detailed diagnostic".to_string(),
      status:      "Fatal".to_string(),
      status_code: 3,
    };
    let err = core_result_xml(response, "conversion failed before post-processing")
      .expect_err("missing XML is an Err");
    assert!(
      err.contains("conversion failed before post-processing"),
      "{err}"
    );
    assert!(err.contains("Fatal:foo:bar detailed diagnostic"), "{err}");
  }
}
