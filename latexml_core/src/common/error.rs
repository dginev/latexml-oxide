use std::{
  cell::RefCell,
  error::Error as ErrorTrait,
  fmt, io,
  num::{ParseFloatError, ParseIntError},
  result,
};

use once_cell::sync::Lazy;

use crate::common::arena::SymHashMap;

#[derive(Debug, Clone, Default)]
pub struct LogState {
  pub undefined:   SymHashMap<usize>,
  pub missing:     SymHashMap<usize>,
  pub debug:       usize,
  pub info:        usize,
  pub warning:     usize,
  pub error:       usize,
  pub fatal:       bool,
  pub status_code: usize,
}
pub enum LogStatus {
  Debug,
  Info,
  Warning,
  Error,
  Fatal,
  Undefined,
  Missing,
}

#[thread_local]
pub static REPORT: Lazy<RefCell<LogState>> = Lazy::new(|| RefCell::new(LogState::default()));

/// When true, Error!/Warn!/Info! macros still count in the report
/// but do **not** emit anything to stderr/log.
/// Used by tests that are known to produce errors in both Perl and Rust.
#[thread_local]
static SUPPRESS_LOG_OUTPUT: std::cell::Cell<bool> = std::cell::Cell::new(false);

/// Set or clear the log-output suppression flag. Returns the previous value.
pub fn set_suppress_log_output(suppress: bool) -> bool {
  let prev = SUPPRESS_LOG_OUTPUT.get();
  SUPPRESS_LOG_OUTPUT.set(suppress);
  prev
}

// Thread-local FATAL DEMOTION for bibliography post-processing (user
// policy 2026-07-04): with the live-state field interpretation, Warn!/
// Error! report at NATIVE severity and count normally (matching Perl's
// MergeStatus accounting, Common/Error.pm L669) — problems in bib fields
// are real conversion diagnostics. Only Fatal! is demoted: it notes and
// logs as an ERROR (`demoted_fatal:` target) instead of latching the
// document's sticky fatal — a broken bibliography must never lose the
// document. The Err return is unchanged, so the failing digestion still
// aborts (its caller degrades gracefully).
thread_local! {
  static DEMOTE_FATALS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Set or clear the fatal-demotion flag. Returns the previous value.
pub fn set_demote_fatals(demote: bool) -> bool {
  DEMOTE_FATALS.with(|c| {
    let prev = c.get();
    c.set(demote);
    prev
  })
}

/// Returns true if `Fatal!` is currently demoted to Error.
pub fn is_demote_fatals() -> bool { DEMOTE_FATALS.with(|c| c.get()) }

/// Returns true if log output is currently suppressed.
pub fn is_log_output_suppressed() -> bool { SUPPRESS_LOG_OUTPUT.get() }

/// Per-thread tracker for the most recently emitted error's
/// `category:object` signature, plus the count of how many
/// consecutive errors share the same signature. Used to detect
/// runaway loops where a single pathological control-sequence (like
/// plain-TeX `\tabalign` invoked in math mode → unbounded `\halign`
/// cell loop) keeps emitting the same error indefinitely. See
/// `wisdom_tabalign_math_runaway.md` for the canonical witness.
#[thread_local]
static LAST_ERROR_KEY: RefCell<Option<String>> = RefCell::new(None);
#[thread_local]
static CONSECUTIVE_ERROR_COUNT: std::cell::Cell<usize> = std::cell::Cell::new(0);

/// Threshold for "same error fired this many times in a row → bail."
/// Set well above any legitimate same-error pattern (a paper with
/// 500+ identical errors would already be near-useless output) but
/// well below the 10000 MAX_ERRORS cap so runaway papers don't
/// accumulate huge noise logs. Empirically, the pathological
/// `\tabalign`-in-math-mode runaway hits >9000 consecutive same
/// errors; this catches that at 500 instead. The threshold was
/// tightened from an initial 2000 after verifying no test in the
/// 1112-test suite exceeds it.
pub const MAX_CONSECUTIVE_ERRORS: usize = 500;

/// Record an error signature; returns the new consecutive count.
/// Call from the Error! macro after note_status. Resets count to 1
/// on a different signature, increments on a match.
pub fn note_consecutive_error(key: &str) -> usize {
  let mut last = LAST_ERROR_KEY.borrow_mut();
  if last.as_deref() == Some(key) {
    let c = CONSECUTIVE_ERROR_COUNT.get() + 1;
    CONSECUTIVE_ERROR_COUNT.set(c);
    c
  } else {
    *last = Some(key.to_string());
    CONSECUTIVE_ERROR_COUNT.set(1);
    1
  }
}

/// Reset the consecutive-error tracker (called from initialize_report).
fn reset_consecutive_error_tracker() {
  *LAST_ERROR_KEY.borrow_mut() = None;
  CONSECUTIVE_ERROR_COUNT.set(0);
}
#[macro_export]
macro_rules! report {
  () => {
    (*$crate::common::error::REPORT).borrow()
  };
}
#[macro_export]
macro_rules! report_mut {
  () => {
    (*$crate::common::error::REPORT).borrow_mut()
  };
}

/// Clear the sticky `report.fatal` flag. Used by best-effort
/// helpers (e.g. `\maketitle`'s deferred frontmatter digest) that
/// silently swallow a digest error and want to undo the
/// `note_status(Fatal)` side-effect so the overall conversion
/// status reflects the silently-handled fact.
pub fn clear_fatal_flag() {
  let mut report = REPORT.borrow_mut();
  report.fatal = false;
}

pub fn note_status(status: LogStatus, what: Option<&str>) {
  let mut report = REPORT.borrow_mut();
  use LogStatus::*;
  match status {
    Debug => report.debug += 1,
    Info => report.info += 1,
    Warning => report.warning += 1,
    Error => report.error += 1,
    Fatal => {
      // Diagnostic for "phantom fatals" (a fatal counted in the final summary
      // with no `Fatal:` line in the log — an `Err` raised via `Fatal!` that
      // some caller swallowed without `log_fatal`): dump a backtrace at the
      // moment the tally first flips. Witness math0402448.
      if !report.fatal && debug_fatal_enabled() {
        eprintln!("[debug-fatal] LogStatus::Fatal first noted here:");
        eprintln!("{}", std::backtrace::Backtrace::force_capture());
      }
      report.fatal = true;
    },
    Undefined => {
      // `what` may borrow the arena buffer; `entry` re-interns via `arena::pin`,
      // which can REALLOCATE that buffer and invalidate `what` mid-read, then
      // intern whatever bytes now occupy the slot (e.g. a freshly-interned
      // `\special_relax` family-token name → phantom undefined). Copy out first.
      let key = what.unwrap_or_default().to_string();
      let entry = report.undefined.entry(&key).or_insert(0);
      *entry += 1;
    },
    Missing => {
      let key = what.unwrap_or_default().to_string();
      let entry = report.missing.entry(&key).or_insert(0);
      *entry += 1;
    },
  }
}

pub fn get_status(status: LogStatus) -> usize {
  let report = REPORT.borrow();
  use LogStatus::*;
  match status {
    Debug => report.debug,
    Info => report.info,
    Warning => report.warning,
    Error => report.error,
    Fatal => {
      if report.fatal {
        1
      } else {
        0
      }
    },
    Undefined => report.undefined.0.values().sum(),
    Missing => report.missing.0.values().sum(),
  }
}

/// One shared probe for the `LATEXML_DEBUG_FATAL` diagnostics (first-fatal
/// backtrace, gullet pushback dump, recent-token ring). Lazy-cached so hot
/// paths pay a single bool test, and a single seam if the env contract grows
/// (PR #249 review P3-13).
pub fn debug_fatal_enabled() -> bool {
  use std::sync::OnceLock;
  static FLAG: OnceLock<bool> = OnceLock::new();
  *FLAG.get_or_init(|| std::env::var_os("LATEXML_DEBUG_FATAL").is_some())
}

pub fn initialize_report() {
  let mut report = REPORT.borrow_mut();
  *report = LogState::default();
  reset_consecutive_error_tracker();
  LAST_RESOURCE_FATAL.with(|c| *c.borrow_mut() = None);
}

/// Clear the arena-`SymStr`-keyed report maps (`undefined`, `missing`). MUST be
/// called whenever the arena is reset (see `crate::reset_thread_engine`): their
/// keys are arena interner ids, so after `arena::reset()` a stale key resolves to
/// whatever string now occupies that id — e.g. a `\special_relax` family-token
/// name — producing phantom "undefined macro" reports across conversions.
pub fn reset_arena_keyed_reports() {
  let mut report = REPORT.borrow_mut();
  report.undefined = Default::default();
  report.missing = Default::default();
}

thread_local! {
  /// Latch for the most recent RESOURCE-class fatal (`ErrorTarget::Timeout`
  /// with a unit category: token/pushback/if limits, cycle-guard recursion,
  /// memory budget, conversion deadline). Some layers flatten `Error` into a
  /// plain string on the way up (the marpa semantics boundary turns it into
  /// `marpa::error::Error`), destroying the structured identity — which made
  /// resource fatals indistinguishable from semantic parse rejections and
  /// produced "phantom fatals" (counted in the summary, never logged, parse
  /// grinding on). The `Fatal!` macro records here at raise time; consumers
  /// `take` it to re-classify a flattened error. PR #249 review P1-4.
  static LAST_RESOURCE_FATAL: RefCell<Option<Error>> = const { RefCell::new(None) };
}

/// Record a fatal into the resource-fatal latch — only Timeout-target fatals
/// with payload-free categories are kept (the latch exists for resource
/// fatals; payload-carrying `ErrorCategory` variants are not cloneable and
/// are never resource-class).
pub fn record_last_fatal(e: &Error) {
  use ErrorCategory as C;
  if !matches!(e.target, ErrorTarget::Timeout) {
    return;
  }
  let category = match &e.category {
    C::TokenLimit => C::TokenLimit,
    C::PushbackLimit => C::PushbackLimit,
    C::Recursion => C::Recursion,
    C::IfLimit => C::IfLimit,
    C::MemoryBudget => C::MemoryBudget,
    C::Convert => C::Convert,
    _ => return,
  };
  LAST_RESOURCE_FATAL.with(|c| {
    *c.borrow_mut() = Some(Error {
      target: ErrorTarget::Timeout,
      category,
      message: e.message.clone(),
    });
  });
}

/// Take (and clear) the latched resource fatal, if any. Returns the
/// structured `Error` so a boundary that received only a flattened string can
/// propagate the real thing.
pub fn take_last_resource_fatal() -> Option<Error> {
  LAST_RESOURCE_FATAL.with(|c| c.borrow_mut().take())
}

/// Build a status message matching Perl's `getStatusMessage()`.
/// Format: "N warnings; M errors; K fatal error; L undefined macros[\foo, \bar]; P missing
/// files[x.sty]" Returns "No obvious problems" when no issues detected.
pub fn get_status_message() -> String {
  let report = REPORT.borrow();
  let mut parts = Vec::new();
  if report.warning > 0 {
    parts.push(format!(
      "{} warning{}",
      report.warning,
      if report.warning > 1 { "s" } else { "" }
    ));
  }
  if report.error > 0 {
    parts.push(format!(
      "{} error{}",
      report.error,
      if report.error > 1 { "s" } else { "" }
    ));
  }
  if report.fatal {
    parts.push("1 fatal error".to_string());
  }
  let undef_keys: Vec<String> = report
    .undefined
    .keys()
    .map(|k| crate::common::arena::to_string(*k))
    .collect();
  if !undef_keys.is_empty() {
    parts.push(format!(
      "{} undefined macro{}[{}]",
      undef_keys.len(),
      if undef_keys.len() > 1 { "s" } else { "" },
      undef_keys.join(", ")
    ));
  }
  let miss_keys: Vec<String> = report
    .missing
    .keys()
    .map(|k| crate::common::arena::to_string(*k))
    .collect();
  if !miss_keys.is_empty() {
    parts.push(format!(
      "{} missing file{}[{}]",
      miss_keys.len(),
      if miss_keys.len() > 1 { "s" } else { "" },
      miss_keys.join(", ")
    ));
  }
  if parts.is_empty() {
    "No obvious problems".to_string()
  } else {
    parts.join("; ")
  }
}

/// Compute the status code from the report state (Perl getStatusCode).
/// 3 = fatal, 2 = errors, 1 = warnings, 0 = clean.
pub fn get_status_code() -> usize {
  let report = REPORT.borrow();
  if report.fatal {
    3
  } else if report.error > 0 {
    2
  } else if report.warning > 0 {
    1
  } else {
    0
  }
}

/// A thread-portable snapshot of the `REPORT`'s integer status counters
/// (everything EXCEPT the arena-`SymStr`-keyed `undefined`/`missing` maps,
/// whose keys are interner ids local to one thread's arena). Used to forward a
/// worker thread's diagnostic tally back to the main thread: `REPORT` is
/// `#[thread_local]`, so an `Error!`/`Warn!` raised on a spawned post-processing
/// worker increments only that worker's counters and is invisible to the
/// main-thread `status_code` unless merged here. See
/// [`crate::util::logger::capture`] / [`crate::util::logger::replay_captured`].
#[derive(Default, Clone, Copy)]
pub struct ReportCounts {
  pub debug:   usize,
  pub info:    usize,
  pub warning: usize,
  pub error:   usize,
  pub fatal:   bool,
}

/// Snapshot the current thread's `REPORT` integer counters.
pub fn snapshot_report_counts() -> ReportCounts {
  let r = REPORT.borrow();
  ReportCounts {
    debug:   r.debug,
    info:    r.info,
    warning: r.warning,
    error:   r.error,
    fatal:   r.fatal,
  }
}

/// Overwrite the current thread's `REPORT` counters with a prior snapshot.
/// The isolation primitive for RECURSIVE/auxiliary digestions whose
/// diagnostics must not count against the document (Perl analog: the
/// recursive MakeBibliography session keeps its tally out of the outer
/// document). Pair with [`set_suppress_log_output`] so neither the lines
/// nor the counts leak: snapshot -> suppress -> digest -> restore.
pub fn restore_report_counts(c: ReportCounts) {
  let mut r = REPORT.borrow_mut();
  r.debug = c.debug;
  r.info = c.info;
  r.warning = c.warning;
  r.error = c.error;
  r.fatal = c.fatal;
}

/// Add a worker thread's [`ReportCounts`] into the current (main) thread's
/// `REPORT`. Only the integer counts + the sticky `fatal` flag are merged; the
/// arena-keyed `undefined`/`missing` maps are NOT (a worker has its own
/// thread-local arena, so those keys are not portable).
pub fn merge_report_counts(c: ReportCounts) {
  let mut r = REPORT.borrow_mut();
  r.debug += c.debug;
  r.info += c.info;
  r.warning += c.warning;
  r.error += c.error;
  r.fatal |= c.fatal;
}

//======================================================================
// Debuggable features (Perl: `DebuggableFeature($name)` registration +
// `$LaTeXML::DEBUG{$name}` gating, enabled by the CLI's `--debug NAME`).
// Process-global (not thread-local): the CLI parses args on one thread
// and may convert on another (e.g. the big-stack worker in
// bin/latexml_oxide.rs); reads only occur on gated debug paths.
//======================================================================

static KNOWN_DEBUG_FEATURES: Lazy<std::sync::RwLock<std::collections::BTreeSet<String>>> =
  Lazy::new(|| std::sync::RwLock::new(std::collections::BTreeSet::new()));
static ENABLED_DEBUG_FEATURES: Lazy<std::sync::RwLock<rustc_hash::FxHashSet<String>>> =
  Lazy::new(|| std::sync::RwLock::new(rustc_hash::FxHashSet::default()));

/// Perl: `DebuggableFeature($name)` — register a feature name so it can
/// be listed/validated for `--debug`.
pub fn debuggable_feature(name: &str) {
  if let Ok(mut known) = KNOWN_DEBUG_FEATURES.write() {
    known.insert(name.to_string());
  }
}

/// All registered feature names (sorted), for `--debug` diagnostics.
pub fn known_debug_features() -> Vec<String> {
  KNOWN_DEBUG_FEATURES
    .read()
    .map(|k| k.iter().cloned().collect())
    .unwrap_or_default()
}

/// Perl: `$LaTeXML::DEBUG{$name} = 1` — called by the CLI per `--debug NAME`.
pub fn enable_debug_feature(name: &str) {
  if let Ok(mut enabled) = ENABLED_DEBUG_FEATURES.write() {
    enabled.insert(name.to_string());
  }
}

/// Perl: truthiness of `$LaTeXML::DEBUG{$name}`.
pub fn debug_enabled(name: &str) -> bool {
  ENABLED_DEBUG_FEATURES
    .read()
    .map(|enabled| enabled.contains(name))
    .unwrap_or(false)
}

/// Feature-gated debug logging — Perl's `Debug(...) if $LaTeXML::DEBUG{feature}`.
/// Usage: `DebugFeature!("frontmatter", "FRONT Add {}", entry)`.
/// Logs with the feature name as the `log` target (so output matches the
/// previous `log::debug!(target: "frontmatter", ...)` form) and counts a
/// Debug in the status report, like `Debug!`. NB deliberately does NOT
/// forward to `Debug!` — its 3-expr `(category, object, message)` arm
/// would mis-capture a format string with two arguments.
#[macro_export]
macro_rules! DebugFeature {
  ($feature:literal, $($arg:tt)*) => {{
    if $crate::common::error::debug_enabled($feature) {
      $crate::common::error::note_status(
        $crate::common::error::LogStatus::Debug, None);
      use log::debug;
      debug!(target: $feature, $($arg)*);
    }
  }};
}

#[macro_export]
macro_rules! Debug {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($message))
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($message, $($details),*))
  }};
  ($($simple:expr_2021),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Info {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($message))
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
  $crate::common::error::note_status(
    $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!(target: &format!("{}:{}", $category, $object), "{}",
    $crate::generate_message!($message, $($details),*))
  }};
  ($($simple:expr_2021),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Warn {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Warning, None);
    if !$crate::common::error::is_log_output_suppressed() {
      use log::warn;
      warn!(target: &format!("{}:{}", $category, $object), "{}",
        $crate::generate_message!($message))
    }
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Warning, None);
    if !$crate::common::error::is_log_output_suppressed() {
      use log::warn;
      warn!(target: &format!("{}:{}", $category, $object), "{}",
        $crate::generate_message!($message, $($details),*))
    }
  }}
}

#[macro_export]
macro_rules! Error {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::Error!($category,$object,$message,"")
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Error, None);
    if !$crate::common::error::is_log_output_suppressed() {
      use log::error;
      error!(target: &format!("{}:{}", $category, $object), "{}",
        $crate::generate_message!($message, $($details),*));
    }
    // In the fatal-demotion scope (bibliography post-processing) the
    // too-many/consecutive-error escalations are SKIPPED: their Fatal!
    // would demote back into an Error, turning the circuit-breaker into
    // an error multiplier (run-233 follow-up: 470 self-feeding
    // "Too many errors" lines on 2605.02213). The bib interpreter has its
    // own bounded failure latch instead.
    if !$crate::common::error::is_demote_fatals() {
    // Borrow-safe read: an Error! can be raised from inside a `state_mut()`
    // scope (e.g. push_value's BUG branch, a constructor's after_digest),
    // where a plain `lookup_int` would panic "RefCell already mutably
    // borrowed" and abort the conversion (tikz-cd 2001.08973).
    let max_from_state = $crate::state::try_lookup_int("MAX_ERRORS");
    // Match Perl LaTeXML default of 100 errors before Fatal('too_many_errors').
    // Past 100 errors a paper has already failed comprehension; continuing
    // produces noise without information. Override via state for tests
    // or specific bindings (e.g. tikz_sty raises to 1000, dump-build raises
    // to 1_000_000).
    let maxerrors = match max_from_state {
      // STATE contended: we cannot read the (possibly raised) cap, so skip the
      // too-many-errors check for *this* error rather than risk a spurious
      // Fatal from a stale default. The next uncontended error re-applies it.
      None => usize::MAX,
      Some(v) if v > 0 => v as usize,
      Some(_) => 100,
    };
    if $crate::common::error::get_status($crate::common::error::LogStatus::Error) > maxerrors {
      Fatal!(TooManyErrors, MaxLimit(maxerrors), format!("Too many errors (> {maxerrors})!"));
    }
    // Runaway-loop early-bail: if the same error signature has fired
    // MAX_CONSECUTIVE_ERRORS times in a row, we're stuck in a loop
    // (the canonical witness is plain-TeX `\tabalign` invoked in math
    // mode → unbounded `\halign` cell loop emitting `\hbox` end-mode
    // mismatches). Bail before MAX_ERRORS so logs stay short and
    // post-processing sees a clear cause. The threshold is well above
    // any legitimate same-error pattern (real papers max out at a few
    // hundred unique errors).
    let __consec_key = format!("{}:{}", $category, $object);
    let __consec = $crate::common::error::note_consecutive_error(&__consec_key);
    if __consec > $crate::common::error::MAX_CONSECUTIVE_ERRORS {
      Fatal!(
        TooManyErrors,
        MaxLimit($crate::common::error::MAX_CONSECUTIVE_ERRORS),
        format!(
          "Runaway: same error '{}' fired {} times in a row (cap = {})",
          __consec_key, __consec, $crate::common::error::MAX_CONSECUTIVE_ERRORS
        )
      );
    }
    }
  }}
}

// TODO: flesh out the messages
#[macro_export]
macro_rules! Fatal {
  ($target:expr_2021, $category:expr_2021, $message:expr_2021) => {{
    if $crate::common::error::is_demote_fatals() {
      // Demoted context (bibliography post-processing): count and log as
      // an ERROR — the problem is real and must be visible/accounted —
      // but never latch the document's sticky fatal. The Err return below
      // still aborts the failing digestion; its caller degrades
      // gracefully. A document must not be lost to a broken bibliography.
      $crate::common::error::note_status($crate::common::error::LogStatus::Error, None);
      if !$crate::common::error::is_log_output_suppressed() {
        use log::error;
        error!(target: "demoted_fatal", "{}", $message);
      }
    } else {
      $crate::common::error::note_status($crate::common::error::LogStatus::Fatal, None);
    }
    {
      use $crate::common::error::{Error as LatexmlError, ErrorCategory::*, ErrorTarget::*};
      let __fatal_err = LatexmlError {
        target:   $target,
        category: $category,
        message:  $message.to_string(),
      };
      // Latch resource-class fatals so layers that flatten errors to strings
      // (e.g. the marpa semantics boundary) can still recover the STRUCTURED
      // identity downstream. See `take_last_resource_fatal`.
      $crate::common::error::record_last_fatal(&__fatal_err);
      return Err(__fatal_err);
    }
  }};
}

#[macro_export]
macro_rules! fatal {
  ($target:expr_2021, $category:expr_2021, $message:expr_2021) => {{
    use $crate::common::error::{Error as LatexmlError, ErrorCategory::*, ErrorTarget::*};
    return Err(LatexmlError {
      target:   $target,
      category: $category,
      message:  $message.to_string(),
    });
  }};
}

#[macro_export]
macro_rules! generate_message {
  ($message:expr_2021) => {
    format!(
      "{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $crate::gullet::get_location(),
      file!(),
      line!(),
      column!()
    )
  };
  ($message:expr_2021, $detail:expr_2021) => {
    format!(
      "{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $crate::gullet::get_location(),
      $detail,
      file!(),
      line!(),
      column!()
    )
  };
  ($message:expr_2021, $detail:expr_2021, $detail2:expr_2021) => {
    format!(
      "{}\n\t{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $crate::gullet::get_location(),
      $detail,
      $detail2,
      file!(),
      line!(),
      column!()
    )
  };
  ($message:expr_2021, $detail:expr_2021, $detail2:expr_2021) => {
    format!(
      "{}\n\t{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $crate::gullet::get_location(),
      $detail,
      $detail2,
      file!(),
      line!(),
      column!()
    )
  };
  ($message:expr_2021, $detail:expr_2021, $detail2:expr_2021, $location:expr_2021) => {
    format!(
      "{}\n\t{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $location,
      $detail,
      $detail2,
      file!(),
      line!(),
      column!()
    )
  };
}

#[macro_export]
macro_rules! Note {
  ($input:expr_2021) => {
    if !$crate::common::error::is_log_output_suppressed()
      && log::max_level() >= log::LevelFilter::Info
    {
      let msg = $input;
      $crate::println_stderr!("{msg}");
    }
  };
}

#[macro_export]
macro_rules! NoteLog {
  ($input:expr_2021) => {
    if !$crate::common::error::is_log_output_suppressed()
      && log::max_level() >= log::LevelFilter::Debug
    {
      let msg = $input;
      $crate::println_stderr!("{msg}");
    }
  };
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
  pub target:   ErrorTarget,
  pub category: ErrorCategory,
  pub message:  String,
}
impl ErrorTrait for Error {}
// SAFETY: `Error` contains a `Locator` which embeds a Rc<RefCell<Mouth>> — !Send/!Sync
// by default. The invariant is the same as for `Stored`: errors propagate within a
// single thread's conversion pipeline; they never cross thread boundaries at runtime.
// These impls exist to satisfy `Box<dyn std::error::Error + Send + Sync>` bounds on
// error return types, which transitively require Send/Sync on all error variants.
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

#[derive(Debug)]
pub enum ErrorCategory {
  Init,
  Io(io::Error),
  NotFound,
  Unexpected,
  Expected,
  Misdefined,
  Unknown,
  MissingFile,
  Malformed,
  Libxml,
  Convert,
  Recursion,
  EoF,
  Endgroup,
  FailedParse,
  MaxLimit(usize),
  Generic(Box<dyn ErrorTrait>),
  Filename(String),
  ToDo,
  TokenLimit,
  PushbackLimit,
  IfLimit,
  MemoryBudget,
}

#[derive(Debug)]
pub enum ErrorTarget {
  Package,
  Parameter,
  ParamSpec,
  Prototype,
  Converter,
  Mouth,
  Core,
  State,
  Stomach,
  Codegen,
  Macro,
  XMath,
  MathParser,
  Document,
  Definition,
  TexPool,
  Internal,
  TargetUnexpected,
  TooManyErrors,
  Timeout,
}

impl fmt::Display for ErrorCategory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use ErrorCategory::*;
    match self {
      Init => write!(f, "Init"),
      Io(err) => err.fmt(f),
      NotFound => write!(f, "No matching cities with a population were found."),
      MissingFile => write!(f, "missing file"),
      Misdefined => write!(f, "misdefined"),
      Unknown => write!(f, "unknown"),
      Malformed => write!(f, "malformed"),
      Expected => write!(f, "expected"),
      Unexpected => write!(f, "unexpected"),
      Libxml => write!(f, "libxml error"),
      Recursion => write!(f, "<recursion>"),
      EoF => write!(f, "<EOF>"),
      ToDo => write!(f, "TODO"),
      Convert => write!(f, "conversion"),
      Endgroup => write!(f, "<endgroup>"),
      FailedParse => write!(f, "failed to parse"),
      MaxLimit(num) => write!(f, "{}", num),
      Generic(err) => err.fmt(f),
      Filename(name) => write!(f, "file:{name}"),
      TokenLimit => write!(f, "token_limit"),
      PushbackLimit => write!(f, "pushback_limit"),
      IfLimit => write!(f, "if_limit"),
      MemoryBudget => write!(f, "memory_budget"),
    }
  }
}
impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Error:{}:{:?} {}",
      self.category, self.target, self.message
    )
  }
}

impl Error {
  pub fn log_fatal(&self) {
    let target_str = s!("Fatal:{:?}:{:?} ", self.target, self.category);
    use log::error;
    error!(target: &target_str, "{}", self.message);
    // Mark the global report as fatal so cortex_worker's exit code is
    // 3 (conversion failure) instead of 0 (success). Without this,
    // `Fatal:Timeout:MemoryBudget` etc. printed but the runtime
    // status_code stayed at 0 — canvas would classify the worker as
    // OK with an empty HTML output. R35.A.
    note_status(LogStatus::Fatal, None);
  }
  pub fn todo() -> Self {
    Error {
      target:   ErrorTarget::Internal,
      category: ErrorCategory::ToDo,
      message:  String::from(
        "This section of the code is not yet implemented / ported over from Perl.",
      ),
    }
  }
}

#[macro_export]
macro_rules! unported {
  () => {{ ::latexml_core::common::error::Error::todo() }};
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Error {
    Error {
      target:   ErrorTarget::Mouth,
      category: ErrorCategory::Io(err),
      message:  s!("IO error"),
    }
  }
}

impl From<Box<dyn ErrorTrait>> for Error {
  fn from(err: Box<dyn ErrorTrait>) -> Error {
    Error {
      target:   ErrorTarget::Document,
      message:  err.to_string(),
      category: ErrorCategory::Generic(err),
    }
  }
}
impl From<Box<dyn ErrorTrait + Send + Sync>> for Error {
  fn from(err: Box<dyn ErrorTrait + Send + Sync>) -> Error {
    Error {
      target:   ErrorTarget::Document,
      message:  err.to_string(),
      category: ErrorCategory::Generic(err),
    }
  }
}

impl From<String> for Error {
  fn from(err: String) -> Error {
    Error {
      target:   ErrorTarget::Document,
      category: ErrorCategory::Generic(From::from(err.clone())),
      message:  err,
    }
  }
}

impl<'a> From<&'a str> for Error {
  fn from(err: &'a str) -> Error {
    Error {
      target:   ErrorTarget::Document,
      category: ErrorCategory::Generic(From::from(err.to_owned())),
      message:  err.to_owned(),
    }
  }
}

impl From<()> for Error {
  fn from(_e: ()) -> Error {
    Error {
      target:   ErrorTarget::Document,
      category: ErrorCategory::Libxml,
      message:  s!("LibXML error"),
    }
  }
}

impl From<ParseIntError> for Error {
  fn from(err: ParseIntError) -> Error {
    Error {
      target:   ErrorTarget::Document,
      message:  err.to_string(),
      category: ErrorCategory::Generic(Box::new(err)),
    }
  }
}

impl From<ParseFloatError> for Error {
  fn from(err: ParseFloatError) -> Error {
    Error {
      target:   ErrorTarget::Document,
      message:  err.to_string(),
      category: ErrorCategory::Generic(Box::new(err)),
    }
  }
}

impl From<marpa::error::Error> for Error {
  fn from(err: marpa::error::Error) -> Error {
    Error {
      target:   ErrorTarget::MathParser,
      category: ErrorCategory::FailedParse,
      message:  err.to_string(),
    }
  }
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Progress Reporting
//**********************************************************************
// Progress reporting.

pub fn progress_step(_note: &str) {
  // should we also do a spinner? It's often too fast to spin
  // _spinnerstep(note)
}

pub fn note_progress(stuff: &str) {
  use log::info;
  info!(target: "note", "{}", stuff);
}

// TODO: Rethink this reporting
pub fn note_progress_detailed(stuff: &str) {
  use log::debug;
  debug!(target: "note", "{}", stuff);
}

pub fn note_begin(stage: &str) {
  // $state->assignMapping('NOTE_TIMERS', $stage, [Time::HiRes::gettimeofday]);
  use log::info;
  info!(target: "note", "\n({}...", stage);
}

pub fn note_end(_stage: &str) {
  // if (my $start = $state && $state->lookupMapping('NOTE_TIMERS', $stage)) {
  //   $state->assignMapping('NOTE_TIMERS', $stage, undef);

  // my $elapsed = Time::HiRes::tv_interval($start, [Time::HiRes::gettimeofday]);
  // info!(target: "note", " %.2f sec)", elapsed);
  use log::info;
  info!(target: "note", " )");
}

#[cfg(test)]
mod tests {
  use super::*;

  // These tests share a thread-local `REPORT`, so each test must
  // `initialize_report()` first. They must NOT run truly in parallel
  // over the same thread, but cargo's default harness only runs tests
  // in parallel on separate threads (each with its own thread-local),
  // so this is safe.

  #[test]
  fn initialize_report_clears_state() {
    note_status(LogStatus::Warning, None);
    initialize_report();
    assert_eq!(get_status(LogStatus::Warning), 0);
  }

  #[test]
  fn note_status_increments_counters() {
    initialize_report();
    note_status(LogStatus::Warning, None);
    note_status(LogStatus::Warning, None);
    note_status(LogStatus::Error, None);
    assert_eq!(get_status(LogStatus::Warning), 2);
    assert_eq!(get_status(LogStatus::Error), 1);
    assert_eq!(get_status(LogStatus::Fatal), 0);
  }

  #[test]
  fn fatal_macro_latches_resource_fatals() {
    // The `Fatal!` macro must record Timeout-target fatals in the
    // resource-fatal latch so boundaries that flatten errors to strings
    // (marpa semantics) can recover the structured identity (P1-4).
    initialize_report();
    fn raise() -> Result<()> {
      Fatal!(
        Timeout,
        TokenLimit,
        "Token limit of 5 exceeded, infinite loop?"
      );
    }
    let err = raise().unwrap_err();
    assert!(matches!(err.target, ErrorTarget::Timeout));
    let latched = take_last_resource_fatal().expect("latch must hold the fatal");
    assert!(matches!(latched.target, ErrorTarget::Timeout));
    assert!(matches!(latched.category, ErrorCategory::TokenLimit));
    assert_eq!(latched.message, "Token limit of 5 exceeded, infinite loop?");
    // take() clears the latch.
    assert!(take_last_resource_fatal().is_none());
    // Non-Timeout fatals are NOT latched (the latch serves resource fatals).
    fn raise_other() -> Result<()> {
      Fatal!(Internal, EoF, "fell off the end");
    }
    let _ = raise_other().unwrap_err();
    assert!(take_last_resource_fatal().is_none());
    // initialize_report clears a stale latch.
    let _ = raise().unwrap_err();
    initialize_report();
    assert!(take_last_resource_fatal().is_none());
  }

  #[test]
  fn fatal_status_is_sticky_and_returns_1() {
    initialize_report();
    note_status(LogStatus::Fatal, None);
    note_status(LogStatus::Fatal, None);
    // get_status for Fatal returns 0 or 1, not a counter.
    assert_eq!(get_status(LogStatus::Fatal), 1);
  }

  #[test]
  fn get_status_code_priority_order() {
    initialize_report();
    assert_eq!(get_status_code(), 0, "clean state → 0");
    note_status(LogStatus::Warning, None);
    assert_eq!(get_status_code(), 1, "warning → 1");
    note_status(LogStatus::Error, None);
    assert_eq!(get_status_code(), 2, "error wins over warning → 2");
    note_status(LogStatus::Fatal, None);
    assert_eq!(get_status_code(), 3, "fatal wins over error → 3");
  }

  #[test]
  fn status_message_clean_is_no_obvious_problems() {
    initialize_report();
    assert_eq!(get_status_message(), "No obvious problems");
  }

  #[test]
  fn status_message_plural_warnings() {
    initialize_report();
    note_status(LogStatus::Warning, None);
    let m = get_status_message();
    assert_eq!(m, "1 warning", "singular form");

    note_status(LogStatus::Warning, None);
    let m = get_status_message();
    assert_eq!(m, "2 warnings", "plural form");
  }

  #[test]
  fn status_message_multiple_categories_joined() {
    initialize_report();
    note_status(LogStatus::Warning, None);
    note_status(LogStatus::Warning, None);
    note_status(LogStatus::Error, None);
    let m = get_status_message();
    assert!(
      m.contains("2 warnings") && m.contains("1 error") && m.contains("; "),
      "got {m:?}"
    );
  }

  #[test]
  fn suppress_log_output_returns_prior_value() {
    let prior = set_suppress_log_output(true);
    assert!(is_log_output_suppressed());
    let prior2 = set_suppress_log_output(false);
    assert!(prior2, "round-trip prior value");
    assert!(!is_log_output_suppressed());
    // Clean up to original state.
    set_suppress_log_output(prior);
  }
}
