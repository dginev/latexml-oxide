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

/// Depth of diagnostic-macro emission on this thread (see [`macro_diag_guard`]).
/// A depth counter, not a bool: `Error!` can raise `Fatal!` inside its own
/// scope (the too-many-errors escalation), nesting two guards.
#[thread_local]
static MACRO_DIAG_DEPTH: std::cell::Cell<u32> = std::cell::Cell::new(0);

/// RAII marker: "the log record currently being emitted comes from a
/// diagnostic macro that already counted itself via [`note_status`]".
///
/// The tally has TWO producers which must never overlap: the macros
/// (`Info!`/`Warn!`/`Error!`/`Fatal!`/`Debug!`) count at RAISE time — even when
/// output is suppressed, which the `MAX_ERRORS` cap depends on — and the
/// logger backend counts every OTHER record it prints (raw `log::warn!` and
/// friends, which previously printed `Warning:` lines that no counter ever
/// saw: the 131 MB witness logged 12,105 `Warning:` lines and reported
/// "2 warnings"). This guard is how the logger tells the two apart.
pub struct MacroDiagGuard(());
impl Drop for MacroDiagGuard {
  fn drop(&mut self) { MACRO_DIAG_DEPTH.set(MACRO_DIAG_DEPTH.get().saturating_sub(1)); }
}
#[must_use = "the guard must live across the log emission it marks"]
pub fn macro_diag_guard() -> MacroDiagGuard {
  MACRO_DIAG_DEPTH.set(MACRO_DIAG_DEPTH.get() + 1);
  MacroDiagGuard(())
}

/// Count a diagnostic record observed by the logger backend, unless it was
/// emitted by a macro (already counted at raise time) or the report is
/// mid-borrow (a raw log call from inside a `report_mut!` scope must not
/// panic the conversion over a tally increment — matching the logger's own
/// `try_borrow` discipline for `LOG_BUFFER`).
pub fn note_status_from_logger(status: LogStatus) {
  if MACRO_DIAG_DEPTH.get() > 0 || REPORT.try_borrow_mut().is_err() {
    return;
  }
  note_status(status, None);
}

/// When true, Debug!/Info!/Warn! (and their emit_* forms) still count in
/// the report but do **not** emit anything to stderr/log. `Error!`/`Fatal!`
/// are NOT suppressible — they emit unconditionally (user decision
/// 2026-08-03), so success-rate aggregation (cortex) never loses them.
/// Used by tests/dump-builds that deliberately exercise noisy paths.
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

/// The ONE emission primitive every diagnostic flows through (DRY pass,
/// user directive 2026-08-02): count on the emitting thread's `REPORT`,
/// then log with the pre-formatted `target`, respecting output suppression —
/// EXCEPT for `Error` and `Fatal` records, which are emitted UNCONDITIONALLY
/// (user decision 2026-08-03): frameworks such as cortex aggregate success
/// rates from `Error:`/`Fatal:` lines, so muting either would hide exactly
/// the signal they measure. Suppression mutes Debug/Info/Warning only.
/// The `MacroDiagGuard` marks the emission so the logger backend does not
/// count it a second time.
pub fn emit_record(status: LogStatus, target: &str, message: &str) {
  let _diag_guard = macro_diag_guard();
  // After a `TooManyErrors` Fatal has latched, drop further Error-level
  // records entirely (don't log, don't count). Perl dies at the Fatal so
  // nothing ever logs past it; our recovery machinery keeps converting, and
  // paths that swallow the `Error!` macro's Err (e.g. tex_logic::compare
  // inside a bool-returning conditional) otherwise churn tens of thousands
  // of post-cap records — gckanbun 12.8k, panda-doc 3.6k, past the tikz
  // 1000-cap (perfect-kernel sweep 13). Only the too-many-errors latch
  // gates this: a Timeout/other Fatal still reports the trailing errors
  // that explain it.
  if matches!(status, LogStatus::Error) && too_many_errors_latched() {
    return;
  }
  let level = match status {
    LogStatus::Debug => log::Level::Debug,
    LogStatus::Info => log::Level::Info,
    LogStatus::Warning => log::Level::Warn,
    _ => log::Level::Error,
  };
  let unconditional = matches!(status, LogStatus::Error | LogStatus::Fatal);
  note_status(status, None);
  if unconditional || !is_log_output_suppressed() {
    log::log!(target: target, level, "{message}");
  }
}

/// The single diagnostic vehicle, function form — for contexts that cannot
/// use the `Error!`/`Warn!`/`Info!` macros because those are return-based
/// (`Error!` escalates to `Fatal!`, which `return Err(...)`s, so it only
/// typechecks in `Result<_, error::Error>` functions).
///
/// Perl LaTeXML has exactly one emission vehicle per severity (`Error.pm`),
/// which is what lets its tally and cortex's aggregation be lossless by
/// construction. These functions restore that property for Rust's
/// non-`Result` contexts (post-processing drivers, workers, the LSP server):
/// they count, emit with a proper `category:object` target, respect output
/// suppression, and participate in the runaway circuit-breakers — everything
/// the raw `log::warn!`-family calls they replace silently skipped. Raw
/// `log::*!` diagnostics are BANNED in workspace crates (see
/// `tools/lint_raw_log_diag.sh`); the logger backend's own tally
/// (`note_status_from_logger`) remains only as the net for FOREIGN crates
/// logging through the `log` facade, which can never use this vehicle.
pub fn emit_info(category: &str, object: &str, message: &str) {
  emit_record(LogStatus::Info, &format!("{category}:{object}"), message);
}

/// See [`emit_info`].
pub fn emit_warn(category: &str, object: &str, message: &str) {
  emit_record(LogStatus::Warning, &format!("{category}:{object}"), message);
}

/// See [`emit_info`]. Unlike the `Error!` macro this cannot escalate by
/// `return`ing — there is no `Err` channel in the contexts it serves — so
/// when the error count or the consecutive-error count crosses its cap it
/// emits the `Fatal:TooManyErrors` record and latches the sticky fatal
/// instead: the run continues (its caller has no unwind path) but the
/// conversion's verdict and status code report the fatal honestly.
pub fn emit_error(category: &str, object: &str, message: &str) {
  emit_record(LogStatus::Error, &format!("{category}:{object}"), message);
  if is_demote_fatals() {
    return;
  }
  let maxerrors = match crate::state::try_lookup_int("MAX_ERRORS") {
    None => usize::MAX, // STATE contended: skip the check for this error
    Some(v) if v > 0 => v as usize,
    Some(_) => 100,
  };
  let consec = note_consecutive_error(&format!("{category}:{object}"));
  let over_total = get_status(LogStatus::Error) > maxerrors;
  let over_consec = consec > MAX_CONSECUTIVE_ERRORS;
  // Latch exactly at the crossing, not on every error past it.
  if (over_total && get_status(LogStatus::Error) == maxerrors + 1)
    || (over_consec && consec == MAX_CONSECUTIVE_ERRORS + 1)
  {
    latch_too_many_errors();
    emit_fatal(
      "TooManyErrors",
      "MaxLimit",
      &format!(
        "Too many errors (> {})!",
        if over_total {
          maxerrors
        } else {
          MAX_CONSECUTIVE_ERRORS
        }
      ),
    );
  }
}

/// See [`emit_info`]. Latches the sticky fatal and emits the canonical
/// `Fatal:<category>:<object>` line — NEVER suppressed (see [`emit_record`]).
/// The CALLER owns any early-exit control flow (e.g. `latexml_post`'s
/// `Fatal!` returns its own `PostError` after this) — a fatal, unlike an
/// error, needs no cap bookkeeping.
pub fn emit_fatal(category: &str, object: &str, message: &str) {
  emit_record(
    LogStatus::Fatal,
    &format!("Fatal:{category}:{object} "),
    message,
  );
}

/// Reset the consecutive-error tracker (called from initialize_report).
fn reset_consecutive_error_tracker() {
  *LAST_ERROR_KEY.borrow_mut() = None;
  CONSECUTIVE_ERROR_COUNT.set(0);
  TOO_MANY_ERRORS_LATCHED.set(false);
}

#[thread_local]
static TOO_MANY_ERRORS_LATCHED: std::cell::Cell<bool> = std::cell::Cell::new(false);

/// Latch the too-many-errors state: `emit_record` drops further Error-level
/// records until the next conversion resets the report. Set by the
/// `TooManyErrors` Fatal paths (the `Error!` macro cap and `emit_error`'s
/// latch); cleared in `reset_consecutive_error_tracker`.
pub fn latch_too_many_errors() { TOO_MANY_ERRORS_LATCHED.set(true); }
/// Whether the too-many-errors latch is set (see `latch_too_many_errors`).
pub fn too_many_errors_latched() -> bool { TOO_MANY_ERRORS_LATCHED.get() }
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
/// The canonical machine-readable conversion-status line, `Status:conversion:N`.
///
/// The contract every multi-phase executable ends its log with: the cortex
/// framework derives a task's final severity from the LAST such line in the
/// log (and defaults to Fatal when it is absent), so `code` must be the
/// combined `max(core, post)` verdict. Shared here so the CLI, the worker and
/// the archive `status` member can never drift in format.
pub fn conversion_status_line(code: usize) -> String { format!("Status:conversion:{code}") }

/// The human-readable end-of-run verdict: `Conversion complete|failed: <counts>`.
///
/// `failed` iff `code` is fatal (>= 3), mirroring Perl LaTeXML's summary
/// line. Callers pass their combined `max(core, post)` code; the counts come
/// from the shared REPORT counter via [`get_status_message`].
pub fn conversion_verdict(code: usize) -> String {
  format!(
    "Conversion {}: {}",
    if code >= 3 { "failed" } else { "complete" },
    get_status_message()
  )
}

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

/// Would a `Debug`-status record actually reach the log right now? True only
/// when the global `log` level admits `Debug` (default is `Info`; `--verbose`/
/// `--debug` raise it — `util/logger.rs::init`) AND output is not suppressed.
/// The `Debug!` macro gates *message construction* on this: the 2026-08-23
/// audit measured up to ~26% of a build-bound conversion spent serializing
/// `node_to_string` subtrees into `Debug!` messages that `emit_record` then
/// discarded (PERFORMANCE.md, Open levers P0). An atomic load + thread-local
/// read — cheap enough for token-frequency call sites.
#[inline]
pub fn debug_record_enabled() -> bool {
  log::max_level() >= log::LevelFilter::Debug && !is_log_output_suppressed()
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
      let __diag_guard = $crate::common::error::macro_diag_guard();
      $crate::common::error::note_status(
        $crate::common::error::LogStatus::Debug, None);
      use log::debug;
      debug!(target: $feature, $($arg)*);
    }
  }};
}

/// Debug-status diagnostics. **Lazy**: the argument expressions — which at
/// several sites build whole `node_to_string` subtree serializations — are
/// evaluated only when [`debug_record_enabled`] says the record would actually
/// be logged (2026-08-23 audit, PERFORMANCE.md Open levers P0). The Debug
/// status tally (`note_status`) is preserved unconditionally, so status counts
/// are identical to the eager form at every verbosity.
#[macro_export]
macro_rules! Debug {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    if $crate::common::error::debug_record_enabled() {
      $crate::common::error::emit_record(
        $crate::common::error::LogStatus::Debug,
        &format!("{}:{}", $category, $object),
        &$crate::generate_message!($message))
    } else {
      $crate::common::error::note_status(
        $crate::common::error::LogStatus::Debug, None);
    }
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    if $crate::common::error::debug_record_enabled() {
      $crate::common::error::emit_record(
        $crate::common::error::LogStatus::Debug,
        &format!("{}:{}", $category, $object),
        &$crate::generate_message!($message, $($details),*))
    } else {
      $crate::common::error::note_status(
        $crate::common::error::LogStatus::Debug, None);
    }
  }};
  ($($simple:expr_2021),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    if $crate::common::error::debug_record_enabled() {
      let __diag_guard = $crate::common::error::macro_diag_guard();
      use log::debug;
      debug!($($simple),*);
    }
  }};

}

#[macro_export]
macro_rules! Info {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::common::error::emit_info(
      &format!("{}", $category), &format!("{}", $object),
      &$crate::generate_message!($message))
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    $crate::common::error::emit_info(
      &format!("{}", $category), &format!("{}", $object),
      &$crate::generate_message!($message, $($details),*))
  }};
  ($($simple:expr_2021),*) => {{
    let __diag_guard = $crate::common::error::macro_diag_guard();
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Warn {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::common::error::emit_warn(
      &format!("{}", $category), &format!("{}", $object),
      &$crate::generate_message!($message))
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    $crate::common::error::emit_warn(
      &format!("{}", $category), &format!("{}", $object),
      &$crate::generate_message!($message, $($details),*))
  }}
}

#[macro_export]
macro_rules! Error {
  ($category:expr_2021, $object:expr_2021, $message:expr_2021) => {{
    $crate::Error!($category,$object,$message,"")
  }};
 ($category:expr_2021, $object:expr_2021, $message:expr_2021, $($details:expr_2021),*) => {{
    $crate::common::error::emit_record(
      $crate::common::error::LogStatus::Error,
      &format!("{}:{}", $category, $object),
      &$crate::generate_message!($message, $($details),*),
    );
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
      $crate::common::error::latch_too_many_errors();
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
      $crate::common::error::emit_record(
        $crate::common::error::LogStatus::Error,
        "demoted_fatal",
        &format!("{}", $message),
      );
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

/// Progress note to BOTH the log and stderr — Perl `Note` (`_printline`): the LOG
/// always (if a buffer is bound, ANSI-stripped), STDERR only when the verbosity
/// admits it (`$USE_STDERR && $VERBOSITY>=0`). The STDERR gate is the decoupled
/// console verbosity ([`crate::util::logger::stderr_shows_info`]), NOT `max_level`
/// — under `--quiet` the log-file floor keeps `max_level` at `Info`, but the
/// console note must still be silenced (issue #763).
#[macro_export]
macro_rules! Note {
  ($input:expr_2021) => {{
    let msg = $input;
    $crate::util::logger::note_to_log(&msg.to_string());
    if !$crate::common::error::is_log_output_suppressed()
      && $crate::util::logger::stderr_shows_info()
    {
      $crate::println_stderr!("{msg}");
      $crate::util::logger::mark_stderr_at_line_start();
    }
  }};
}

/// Progress note to the LOG only — Perl `NoteLog` (`print $LOG … if $LOG`). Always
/// written to the bound log buffer (the log is the verbose record), never stderr.
#[macro_export]
macro_rules! NoteLog {
  ($input:expr_2021) => {
    $crate::util::logger::note_to_log(&($input).to_string());
  };
}

/// Progress note to STDERR only — Perl `NoteSTDERR` (`if $USE_STDERR &&
/// $VERBOSITY>=0`). Never touches the log. Gated on the decoupled console
/// verbosity ([`crate::util::logger::stderr_shows_info`]), not `max_level`
/// (issue #763).
#[macro_export]
macro_rules! NoteSTDERR {
  ($input:expr_2021) => {
    if !$crate::common::error::is_log_output_suppressed()
      && $crate::util::logger::stderr_shows_info()
    {
      let msg = $input;
      $crate::println_stderr!("{msg}");
      $crate::util::logger::mark_stderr_at_line_start();
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
    // One primitive does both halves: the `Fatal:<target>:<category>` line
    // AND the sticky `LogStatus::Fatal` latch. Without the latch,
    // `Fatal:Timeout:MemoryBudget` etc. printed but the runtime status_code
    // stayed at 0 — canvas would classify the worker as OK with an empty
    // HTML output. R35.A.
    emit_record(
      LogStatus::Fatal,
      &s!("Fatal:{:?}:{:?} ", self.target, self.category),
      &self.message,
    );
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

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Progress Reporting
//**********************************************************************
// Progress reporting.

/// Advance the progress indicator by one step.
///
/// Perl `Common/Error.pm:ProgressStep` L430-433 ticks a terminal spinner. This
/// port draws no spinner — conversion steps go by faster than a spinner can
/// usefully render — so the call is a deliberate no-op, kept as the seam Perl
/// bindings call through. The reporting that does reach the log is
/// [`note_progress`] and the [`note_begin`]/[`note_end`] pair.
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

/// Open a named progress stage, logging `(stage...`.
///
/// Perl `Common/Error.pm:ProgressSpinup` L435ff. Pair every call with
/// [`note_end`], which closes the parenthesis — the nesting of those
/// parentheses is what makes a conversion log readable as a phase tree.
/// Perl also stamps a `NOTE_TIMERS` entry here so the close can report elapsed
/// time; this port logs the structure without the timing (per-phase wall times
/// are the telemetry module's job).
pub fn note_begin(stage: &str) {
  // $state->assignMapping('NOTE_TIMERS', $stage, [Time::HiRes::gettimeofday]);
  use log::info;
  info!(target: "note", "\n({}...", stage);
}

/// Close the progress stage opened by [`note_begin`], logging the matching `)`.
///
/// Perl `Common/Error.pm:ProgressSpindown` additionally prints the stage's
/// elapsed time from its `NOTE_TIMERS` entry; see [`note_begin`] for why this
/// port does not.
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
