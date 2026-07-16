//! Stable, high-level conversion API for using `latexml` as a **library**.
//!
//! Downstream Rust crates depend on `latexml` and call one function — no
//! binary, no manual [`Config`](latexml_core::common::Config) or
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
pub fn convert_to_html(tex: &str) -> Result<String, String> {
  let tex = tex.to_string();
  on_worker(move || {
    let opts = library_config(OutputFormat::HTML5);
    let mut converter = Converter::from_config(opts.clone());
    converter
      .prepare_session(&opts)
      .map_err(|e| format!("could not prepare session: {e}"))?;
    let xml = converter
      .convert(format!("literal:{tex}"))
      .result
      .ok_or_else(|| "conversion failed before post-processing".to_string())?;

    let post_opts = post::PostOptions {
      pmml: true,
      cmml: false,
      keep_xmath: false,
      // Same per-format sheet the CLI uses (shared source of truth).
      stylesheet: post::default_stylesheet(Some("html5")),
      destination: None,
      source_directory: None,
      site_directory: None,
      search_paths: &[],
      nodefaultresources: false,
      css_files: &[],
      js_files: &[],
      noinvisibletimes: false,
      mathtex: false,
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
    };
    Ok(post::run_post_processing(&xml, &post_opts))
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  const DOC: &str = r"\documentclass{article}\begin{document}Hello \(x^2\)\end{document}";

  #[test]
  fn xml_conversion_returns_content() {
    let xml = convert_to_xml(DOC).expect("convert_to_xml");
    assert!(xml.contains("Hello"), "XML should contain the body text: {xml}");
    assert!(xml.contains("<?xml") || xml.contains("<document"), "looks like XML");
  }

  #[test]
  fn html_conversion_returns_page() {
    let html = convert_to_html(DOC).expect("convert_to_html");
    assert!(html.contains("Hello"), "HTML should contain the body text");
    assert!(html.contains("<math") || html.contains("ltx_Math"), "math rendered");
  }
}
