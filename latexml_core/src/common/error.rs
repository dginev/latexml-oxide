use crate::common::arena::SymHashMap;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::result;

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
    Fatal => report.fatal = true,
    Undefined => {
      let entry = report
        .undefined
        .entry(what.unwrap_or_default())
        .or_insert(0);
      *entry += 1;
    },
    Missing => {
      let entry = report.missing.entry(what.unwrap_or_default()).or_insert(0);
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

pub fn initialize_report() {
  let mut report = REPORT.borrow_mut();
  *report = LogState::default();
  reset_consecutive_error_tracker();
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

#[macro_export]
macro_rules! Debug {
  ($category:expr, $object:expr, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!(target: &s!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($message))
  }};
 ($category:expr, $object:expr, $message:expr, $($details:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!(target: &s!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($message, $($details),*))
  }};
  ($($simple:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Info {
  ($category:expr, $object:expr, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($message))
  }};
 ($category:expr, $object:expr, $message:expr, $($details:expr),*) => {{
  $crate::common::error::note_status(
    $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!(target: &format!("{}:{}", $category, $object), "{}",
    $crate::generate_message!($message, $($details),*))
  }};
  ($($simple:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Warn {
  ($category:expr, $object:expr, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Warning, None);
    if !$crate::common::error::is_log_output_suppressed() {
      use log::warn;
      warn!(target: &format!("{}:{}", $category, $object), "{}",
        $crate::generate_message!($message))
    }
  }};
 ($category:expr, $object:expr, $message:expr, $($details:expr),*) => {{
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
  ($category:expr, $object:expr, $message:expr) => {{
    $crate::Error!($category,$object,$message,"")
  }};
 ($category:expr, $object:expr, $message:expr, $($details:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Error, None);
    if !$crate::common::error::is_log_output_suppressed() {
      use log::error;
      error!(target: &format!("{}:{}", $category, $object), "{}",
        $crate::generate_message!($message, $($details),*));
    }
    let max_from_state = $crate::state::lookup_int("MAX_ERRORS");
    // Match Perl LaTeXML default of 100 errors before Fatal('too_many_errors').
    // Past 100 errors a paper has already failed comprehension; continuing
    // produces noise without information. Override via state for tests
    // or specific bindings (e.g. tikz_sty raises to 1000, dump-build raises
    // to 1_000_000).
    let maxerrors = if max_from_state > 0 {
      max_from_state as usize
    } else {
      100
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
  }}
}

// TODO: flesh out the messages
#[macro_export]
macro_rules! Fatal {
  ($target:expr, $category:expr, $message:expr) => {{
    $crate::common::error::note_status($crate::common::error::LogStatus::Fatal, None);
    fatal!($target, $category, $message);
  }};
}

#[macro_export]
macro_rules! fatal {
  ($target:expr, $category:expr, $message:expr) => {{
    use $crate::common::error::Error as LatexmlError;
    use $crate::common::error::ErrorCategory::*;
    use $crate::common::error::ErrorTarget::*;
    return Err(LatexmlError {
      target:   $target,
      category: $category,
      message:  $message.to_string(),
    });
  }};
}

#[macro_export]
macro_rules! generate_message {
  ($message:expr) => {
    format!(
      "{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $crate::gullet::get_location(),
      file!(),
      line!(),
      column!()
    )
  };
  ($message:expr, $detail:expr) => {
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
  ($message:expr, $detail:expr, $detail2:expr) => {
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
  ($message:expr, $detail:expr, $detail2:expr) => {
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
  ($message:expr, $detail:expr, $detail2:expr, $location:expr) => {
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
  ($input:expr) => {
    if !$crate::common::error::is_log_output_suppressed()
      && log::max_level() >= log::LevelFilter::Info
    {
      let msg = $input;
      println_stderr!("{msg}");
    }
  };
}

#[macro_export]
macro_rules! NoteLog {
  ($input:expr) => {
    if !$crate::common::error::is_log_output_suppressed()
      && log::max_level() >= log::LevelFilter::Debug
    {
      let msg = $input;
      println_stderr!("{msg}");
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
      Io(ref err) => err.fmt(f),
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
      Generic(ref err) => err.fmt(f),
      Filename(ref name) => write!(f, "file:{name}"),
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
