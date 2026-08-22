//! Per-conversion identity banner — executable name, version, git revision and
//! exact start time, logged once at the head of every conversion so any log
//! names the precise binary and moment that produced it.
//!
//! Faithful to Perl LaTeXML, which logs `Note("$LaTeXML::IDENTITY processing
//! $source")` at each conversion start (`bin/latexml` L83). `$IDENTITY` is
//! `"$FindBin::Script ($LaTeXML::FULLVERSION)"` (`LaTeXML.pm` L40) — the invoked
//! script's basename plus `"LaTeXML version <v>; revision <sha>"`, the revision
//! filled into `Version.pm` by `make`. We mirror that (revision embedded by
//! `build.rs` instead of `make`) and additionally stamp the exact wall-clock
//! start time, which Perl only emits under `--verbose` (`processing started …`).
//!
//! Generic across every front-end: [`Converter::convert`](crate::converter::Converter::convert)
//! emits it for `latexml_oxide` and `cortex_worker`, and `latexmlmath_oxide` (a
//! separate digest path) emits it directly. The executable name is read from
//! `argv[0]` at runtime, so each binary self-identifies without per-binary wiring.

use std::path::Path;

use chrono::{DateTime, Local};

/// This crate's version (`latexml_oxide`) — the emulated engine's own version,
/// NOT the Perl LaTeXML version it targets. Full `CARGO_PKG_VERSION`, keeping any
/// `-rc` pre-release suffix (a log should reveal an rc); the bare-`X.Y.Z` form
/// for BookML's version gate is [`crate::core_interface::LATEXML_VERSION`].
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Short git revision of the source that built this binary, embedded by
/// `build.rs` (`"unknown"` off a checkout with no `.git` and no
/// `LATEXML_GIT_SHA` override). Perl's `$LaTeXML::Version::REVISION`.
pub const GIT_REVISION: &str = env!("LATEXML_GIT_SHA");

/// Basename of the invoked executable — Perl's `$FindBin::Script`
/// (`latexml_oxide`, `cortex_worker`, `latexmlmath_oxide`, …). Read from
/// `argv[0]` so each binary self-identifies; `"latexml-oxide"` when argv is
/// empty or unreadable (e.g. an embedder driving `Converter` directly).
pub fn executable_name() -> String {
  std::env::args_os()
    .next()
    .as_deref()
    .map(Path::new)
    .and_then(Path::file_name)
    .map(|s| s.to_string_lossy().into_owned())
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "latexml-oxide".to_string())
}

/// Conversion start instant. Honours `SOURCE_DATE_EPOCH` (reproducible builds),
/// exactly as the engine's `\today`/date registers do (`tex_job.rs`), so a
/// pinned epoch yields a deterministic banner; otherwise the local wall clock.
fn start_time() -> DateTime<Local> {
  if let Some(epoch) = std::env::var("SOURCE_DATE_EPOCH")
    .ok()
    .and_then(|e| e.trim().parse::<i64>().ok())
    && let Some(utc) = DateTime::from_timestamp(epoch, 0)
  {
    return utc.with_timezone(&Local);
  }
  Local::now()
}

/// The one-line identity banner, e.g.
/// `latexml_oxide (latexml-oxide 0.9.0; revision a1b2c3d) started 2026-08-21 14:32:05 -0400`.
///
/// Emit it through [`Note!`](latexml_core::Note) so it reaches both stderr and
/// the captured `.latexml.log`, and inherits the verbosity gate (`--quiet`
/// suppresses it).
pub fn identity_banner() -> String {
  format!(
    "{exe} (latexml-oxide {VERSION}; revision {GIT_REVISION}) started {when}",
    exe = executable_name(),
    when = start_time().format("%Y-%m-%d %H:%M:%S %z"),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The banner carries all four requested fields: an executable name, the
  /// crate version, the embedded revision, and a `started <timestamp>` stamp.
  #[test]
  fn banner_has_exe_version_revision_and_time() {
    let banner = identity_banner();
    assert!(
      banner.contains(VERSION),
      "banner {banner:?} missing version {VERSION:?}"
    );
    assert!(
      banner.contains("revision "),
      "banner {banner:?} missing revision field"
    );
    assert!(
      banner.contains(GIT_REVISION),
      "banner {banner:?} missing revision {GIT_REVISION:?}"
    );
    assert!(
      banner.contains(" started "),
      "banner {banner:?} missing start-time stamp"
    );
    assert!(
      !executable_name().is_empty(),
      "executable name must be non-empty"
    );
  }

  /// `SOURCE_DATE_EPOCH` pins the timestamp for a deterministic (reproducible)
  /// banner. Epoch 0 = 1970-01-01 UTC; the local-time render must land on the
  /// 1969-12-31/1970-01-01 boundary depending on the tester's zone.
  #[test]
  fn source_date_epoch_pins_the_timestamp() {
    // SAFETY: single-threaded test; no other thread reads the environment here.
    unsafe { std::env::set_var("SOURCE_DATE_EPOCH", "0") };
    let when = start_time().format("%Y-%m-%d").to_string();
    unsafe { std::env::remove_var("SOURCE_DATE_EPOCH") };
    assert!(
      when == "1970-01-01" || when == "1969-12-31",
      "SOURCE_DATE_EPOCH=0 should render the epoch date, got {when:?}"
    );
  }
}
