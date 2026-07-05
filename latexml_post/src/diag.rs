//! Diagnostic emission for `latexml_post`.
//!
//! Five capitalized macros — `Note!`, `Info!`, `Warn!`, `Error!`,
//! `Fatal!` — mirror the LaTeXML Perl `Note()`/`Info()`/`Warn()`/
//! `Error()`/`Fatal()` reporting conventions. Local to this crate so
//! `latexml_post` does not need to import `latexml_core::common::error`
//! just to emit diagnostics. The macro names deliberately shadow the
//! identically-named macros from `latexml_core` — same shape (`(category,
//! object, …)`), simpler implementation (no error-cap → Fatal unwinding,
//! no location trace appended). They DO bump the shared `latexml_core`
//! `REPORT` status counters via `note_status`, so a post-processing
//! `Warn!`/`Error!`/`Fatal!` raises the conversion's `status_code` exactly
//! like a core-phase one — the run's severity is the combined worst of the
//! core and post phases (`cortex_worker` folds them as `max(core, post)`).
//! Without this a post-only failure (e.g. an image that fails every
//! converter) logged its line but left `status_code` at 0.
//!
//! For diagnostics raised on a post-processing WORKER THREAD (the graphics
//! conversion pool), both the log text AND these counter bumps are
//! `#[thread_local]`, so they are captured per-worker and replayed on the
//! main thread via `latexml_core::util::logger::capture`/`replay_captured`.
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
  ($input:expr_2021) => {{
    if log::max_level() >= log::LevelFilter::Info {
      eprintln!("{}", $input);
    }
  }};
}

// The single-message arms delegate to the format arms (`"{}", $msg`) so the
// `note_status` count + the `log::*!` emission live in exactly ONE place per
// macro — mirroring `latexml_core`'s own `Error!` self-delegation.
#[macro_export]
macro_rules! Info {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    $crate::Info!($category, $object, "{}", $msg)
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {{
    latexml_core::common::error::note_status(latexml_core::common::error::LogStatus::Info, None);
    log::info!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  }};
}

#[macro_export]
macro_rules! Warn {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    $crate::Warn!($category, $object, "{}", $msg)
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {{
    latexml_core::common::error::note_status(latexml_core::common::error::LogStatus::Warning, None);
    log::warn!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  }};
}

#[macro_export]
macro_rules! Error {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    $crate::Error!($category, $object, "{}", $msg)
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {{
    latexml_core::common::error::note_status(latexml_core::common::error::LogStatus::Error, None);
    log::error!(target: &format!("{}:{}", $category, $object), $fmt, $($arg)+)
  }};
}

#[macro_export]
macro_rules! Fatal {
  ($category:expr_2021, $object:expr_2021, $msg:expr_2021) => {
    $crate::Fatal!($category, $object, "{}", $msg)
  };
  ($category:expr_2021, $object:expr_2021, $fmt:expr_2021, $($arg:tt)+) => {{
    latexml_core::common::error::note_status(latexml_core::common::error::LogStatus::Fatal, None);
    let __m = format!($fmt, $($arg)+);
    log::error!(target: &format!("Fatal:{}:{}", $category, $object), "{}", __m);
    return Err($crate::processor::PostError::Processing(
      format!("{}:{}: {}", $category, $object, __m)
    ));
  }};
}
