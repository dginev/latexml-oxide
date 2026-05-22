//! Diagnostic emission for `latexml_post`.
//!
//! Five capitalized macros — `Note!`, `Info!`, `Warn!`, `Error!`,
//! `Fatal!` — mirror the LaTeXML Perl `Note()`/`Info()`/`Warn()`/
//! `Error()`/`Fatal()` reporting conventions. Local to this crate so
//! `latexml_post` does not need to import `latexml_core::common::error`
//! just to emit diagnostics. The macro names deliberately shadow the
//! identically-named macros from `latexml_core` — same shape (`(category,
//! object, …)`), simpler implementation (no `note_status` counter,
//! no error-cap unwinding, no location trace appended).
//!
//! Output format goes through the shared `latexml_core::util::logger`
//! formatter via `log::info!`/`warn!`/`error!` with an explicit
//! `target = "<category>:<object>"`, yielding the canonical
//! `{Severity}:{category}:{object} {message}` line the harness
//! aggregates from every other stage (engine, package, contrib).
//!
//! `Note!` is the lone exception: it bypasses the logger formatter
//! entirely and writes the bare message to stderr, matching the
//! prefix-less `Note(…)` output style from Perl LaTeXML.
//!
//! `Fatal!` additionally `return`s `Err(PostError::Processing(…))`
//! so the calling function can early-exit via `?`, mirroring the way
//! Perl `Fatal()` early-exits via `die`. The crate's `PostError` type
//! differs from `latexml_core::common::error::Error`, which is why we
//! cannot reuse the upstream `Fatal!`.
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
macro_rules! Note {
  ($input:expr) => {{
    if log::max_level() >= log::LevelFilter::Info {
      eprintln!("{}", $input);
    }
  }};
}

#[macro_export]
macro_rules! Info {
  ($category:expr, $object:expr, $msg:expr) => {
    log::info!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::info!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! Warn {
  ($category:expr, $object:expr, $msg:expr) => {
    log::warn!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::warn!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! Error {
  ($category:expr, $object:expr, $msg:expr) => {
    log::error!(target: &format!("{}:{}", $category, $object), "{}", $msg)
  };
  ($category:expr, $object:expr, $fmt:expr, $($arg:tt)+) => {
    log::error!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  };
}

#[macro_export]
macro_rules! Fatal {
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
