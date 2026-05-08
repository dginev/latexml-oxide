//! Diagnostic emission for `latexml_post`.
//!
//! Post-processing functions return `PostError`, not the engine's
//! `latexml_core::common::error::Error`, so we cannot directly reuse
//! the full `Error!`/`Warn!`/`Fatal!` macros from
//! `latexml_core::common::error` — those early-return
//! `Err(LatexmlError)` on max-errors / runaway-loop and would type-mismatch
//! against `PostError` returns.
//!
//! Instead we emit through `log::{error,warn,info}!` with a `target:`
//! attribute shaped exactly the way `latexml_core::common::error` shapes it.
//! The shared logger formatter at `latexml_core::util::logger::log` reads
//! `record.target()` and emits:
//!
//!   `{Severity}:{target} {message}`
//!
//! With `target = "<class>:<object>"` that yields the canonical
//! `Error:<class>:<object> <message>` line the harness aggregates from
//! every other stage (engine, package, contrib).
//!
//! Convention notes carried over from Perl `LaTeXML::Post::*`:
//!   * `Error('expected', 'source', …)`        — Graphics.pm:216 (missing source)
//!   * `Error('imageprocessing', $source, …)`  — Graphics.pm:274 (conversion fail)
//!   * `Error('expected', 'stylesheet', …)`    — XSLT.pm:36/47 (XSLT setup)
//!   * `Error('missing-file', $stylesheet, …)` — XSLT.pm:42 (missing XSLT)
//!   * `Error('expected', 'Image::Magick', …)` — LaTeXImages.pm:128 (env)
//!   * `Error('I/O', $path, …)`                — LaTeXImages.pm:259 (I/O)
//!   * `Error('shell', $cmd, …)`               — LaTeXImages.pm:293/328 (subprocess)
//!   * `Fatal('misdefined', (ref $self), …)`   — Post.pm:177/434 (no-process / abstract)
//!   * `Fatal('unexpected', $dir, …)`          — Post.pm:701 (bad destdir)

#[macro_export]
macro_rules! log_post_error {
  ($category:expr, $object:expr, $msg:expr) => {
    log::error!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::error!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! log_post_warn {
  ($category:expr, $object:expr, $msg:expr) => {
    log::warn!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::warn!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! log_post_info {
  ($category:expr, $object:expr, $msg:expr) => {
    log::info!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::info!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

/// `log_post_fatal!` emits the same target-prefixed `Fatal:<class>:<object>`
/// line a Perl `Fatal('class', 'object', …)` would, and then converts to
/// a `PostError::Processing` so the calling function can early-return via
/// `?` like Perl's `die`.
///
/// The logger's "Fatal:" prefix is matched by
/// `latexml_core::util::logger::log` line 82 (`starts_with("Fatal:")` →
/// no severity prefix added, the class-prefix `Fatal:` is the severity).
#[macro_export]
macro_rules! log_post_fatal {
  ($category:expr, $object:expr, $msg:expr) => {{
    log::error!(target: &format!("Fatal:{}:{}", $category, $object), "{}", $msg);
    return Err($crate::processor::PostError::Processing(
      format!("{}:{}: {}", $category, $object, $msg)
    ));
  }};
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {{
    let __m = format!($fmt, $($arg)+);
    log::error!(target: &format!("Fatal:{}:{}", $category, $object), "{}", __m);
    return Err($crate::processor::PostError::Processing(
      format!("{}:{}: {}", $category, $object, __m)
    ));
  }};
}
