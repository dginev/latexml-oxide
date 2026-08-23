use std::{
  cell::RefCell,
  sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError, max_level};

// ANSI SGR escape sequences (drop-in replacement for the
// unmaintained `ansi_term` crate; bytes match `ansi_term::Colour::*.paint(...)`).
const ANSI_RESET: &str = "\x1b[0m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_WHITE: &str = "\x1b[37m";

fn paint(color: &str, text: &str) -> String { format!("{color}{text}{ANSI_RESET}") }

/// Whether to emit ANSI color escapes on stderr. Colors are a convenience for
/// an interactive terminal ONLY; when stderr is redirected to a file or pipe
/// (the canvas/auto-upgrade path: `cortex_worker ... > log.txt 2>&1`) they are
/// noise that breaks line-anchored error parsing — a naive `grep '^Error:'`
/// matches `\x1b[31mError:` ZERO times and silently reports "0 errors" on a
/// failed paper (the false-negative that masked real Rust-only regressions; see
/// CLAUDE.md "canvas signal integrity"). So: colorize iff stderr is a TTY and
/// `NO_COLOR` is unset. Cached once — stderr's terminal-ness can't change
/// mid-process. Note the captured LOG_BUFFER (`.latexml.log`) is already
/// ANSI-stripped independently; this makes the *redirected stderr* match it.
fn stderr_use_color() -> bool {
  use std::{io::IsTerminal, sync::OnceLock};
  static USE_COLOR: OnceLock<bool> = OnceLock::new();
  *USE_COLOR
    .get_or_init(|| std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none())
}

/// The STDERR verbosity gate, decoupled from the log-file floor.
///
/// Perl LaTeXML separates the two: the log file (`$LOG`) receives every message
/// while it is open, but STDERR prints only when `$USE_STDERR && $VERBOSITY >= 0`
/// (`Common/Error.pm` `_printline`). `--quiet` lowers THIS (to `Warn`) without
/// lowering what the log keeps — so `(Loading …)` progress notes and `Info:`
/// records still reach `.latexml.log`, which is what BookML's makefile dependency
/// tracking reads (issue #763). `--verbose`/`--debug` raise it (to `Debug`).
///
/// Stored as the `LevelFilter` discriminant (`Off`=0 … `Trace`=5). Process-global
/// like `max_level`: one shared stderr descriptor across `cortex_worker`'s pool.
static STDERR_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// The current STDERR verbosity (see [`STDERR_LEVEL`]).
fn stderr_level() -> LevelFilter {
  match STDERR_LEVEL.load(Ordering::Relaxed) {
    0 => LevelFilter::Off,
    1 => LevelFilter::Error,
    2 => LevelFilter::Warn,
    3 => LevelFilter::Info,
    4 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
  }
}

/// Does the current STDERR verbosity admit a record of `level`? `Error`/`Fatal`
/// are handled unconditionally by the caller (the project's deliberate
/// always-emit-errors divergence); this governs the verbosity-gated levels.
pub fn stderr_admits(level: Level) -> bool { level.to_level_filter() <= stderr_level() }

/// Whether STDERR currently shows `Info`-level progress notes — Perl
/// `$VERBOSITY >= 0`. The `Note!`/`NoteSTDERR!` macros gate their STDERR echo on
/// this, decoupled from the log-file floor so `--quiet` silences the console note
/// while the log keeps it (issue #763).
pub fn stderr_shows_info() -> bool { stderr_admits(Level::Info) }

struct LatexmlLogger;
static LOGGER: LatexmlLogger = LatexmlLogger;

/// Thread-local log capture buffer. When enabled, log messages are
/// appended here (without ANSI colors) in addition to stderr.
#[thread_local]
static LOG_BUFFER: RefCell<Option<String>> = RefCell::new(None);

/// Does stderr currently sit at the start of a line?
///
/// A diagnostic record must begin on a fresh line so CorTeX's line-anchored
/// parser (`^Error:`/`^Warning:`/`^Info:`) matches and the record never glues
/// onto an in-flight progress note like `(Loading "foo.sty"… )`, which is
/// written WITHOUT a trailing newline. That guarantee used to be bought with an
/// unconditional leading `\n` on every record — which emits a BLANK line
/// whenever stderr was already at line start, i.e. almost always.
///
/// Measured on the 131 MB witness: **1,440,571 of 3,142,509 captured stderr
/// lines (45.8%) were blank**, matching the record count almost exactly. The
/// `LOG_BUFFER` path never had this bug — it tests `ends_with('\n')` — so the
/// on-disk `.latexml.log` and the captured stderr disagreed on line count.
///
/// Tracking line-start state reproduces the guarantee at half the bytes.
///
/// **Process-global, not `#[thread_local]`, and mutated under the stderr lock.**
/// stderr is one shared descriptor: `cortex_worker --pool-size N` runs N
/// conversion threads in one process, and `logger::capture` exists precisely to
/// run conversions on worker threads. A per-thread flag would let thread A skip
/// its leading newline believing the cursor is at line start while thread B had
/// left it mid-line — gluing a record onto B's output, where CorTeX's
/// `^Error:`/`^Warning:` anchor then misses it entirely. That is a silent
/// error-count loss, the exact failure this project's signal-integrity rule
/// forbids (false negatives hide regressions). The unconditional leading `\n`
/// was robust to that by construction, so the replacement has to be too:
/// check-write-set happens as one critical section holding the stderr lock.
static STDERR_AT_LINE_START: AtomicBool = AtomicBool::new(true);

/// Tell the logger that stderr is back at line start. For the `Note!` /
/// `NoteLog!` macros, which write to stderr directly via `println_stderr!`
/// rather than through `log::Log` — without this the next diagnostic record
/// could emit a spurious leading newline.
pub fn mark_stderr_at_line_start() { STDERR_AT_LINE_START.store(true, Ordering::Release); }

/// Start capturing log output into the buffer (Perl: bind_log).
pub fn bind_log() { *LOG_BUFFER.borrow_mut() = Some(String::new()); }

/// Flush and return the captured log output, stopping capture (Perl: flush_log).
pub fn flush_log() -> String { LOG_BUFFER.borrow_mut().take().unwrap_or_default() }

/// Append a progress note to the captured log (`.latexml.log`) only — the LOG
/// half of Perl `Note`/`NoteLog` (`Common/Error.pm`: `print $LOG _freshline($LOG),
/// strip_ansi($message), "\n" if $LOG`). ANSI is stripped and the note lands on
/// its own fresh line with a single trailing newline, so CorTeX's line-anchored
/// parser and the `.latexml.log` stay clean (mirrors the diagnostic-record
/// freshline path below). No-op when no buffer is bound or output is suppressed —
/// unlike a `log::info!` record it is NOT gated on the stderr verbosity, because
/// the log is the verbose record (Perl writes it regardless of `$VERBOSITY`).
pub fn note_to_log(msg: &str) {
  if crate::common::error::is_log_output_suppressed() {
    return;
  }
  if let Ok(mut buf) = LOG_BUFFER.try_borrow_mut()
    && let Some(ref mut log) = *buf
  {
    let clean = strip_ansi(msg);
    if !log.is_empty() && !log.ends_with('\n') {
      log.push('\n');
    }
    log.push_str(&clean);
    if !log.ends_with('\n') {
      log.push('\n');
    }
  }
}

/// Diagnostics captured from a worker thread by [`capture`], for the main thread
/// to fold back in via [`replay_captured`]. Carries both the already-formatted
/// log text AND the `REPORT` count deltas — `LOG_BUFFER` and `REPORT` are BOTH
/// `#[thread_local]`, so forwarding only the text would still leave
/// `status_code` blind to a worker's failures.
pub struct CapturedDiagnostics {
  pub log:    String,
  pub counts: crate::common::error::ReportCounts,
}

/// Run `f` on the CURRENT (worker) thread with diagnostic capture. Binds a fresh
/// thread-local log buffer for the duration so any `Error!`/`Warn!`/`Info!` `f`
/// emits (directly or deep inside a conversion helper) is recorded instead of
/// lost, and snapshots the worker's `REPORT` counters afterward. The returned
/// [`CapturedDiagnostics`] is replayed on the main thread by [`replay_captured`]
/// after the worker is joined, so the messages reach the bound `cortex.log` and
/// the failures register in `status_code`.
///
/// Assumes the worker thread has no pre-bound buffer (the spawned post-processing
/// pool threads start clean); it does not save/restore a prior binding.
///
/// INVARIANTS (unenforced by types; guard the fleet's canonical signal):
/// - one `capture` per thread lifetime — `bind_log()` CLOBBERS any pre-bound
///   buffer, and `snapshot_report_counts` does not reset, so reusing capture
///   on a pooled thread would drop earlier text and double-merge counts;
/// - callers must `replay_captured` the result on the MAIN thread exactly
///   once (a panicking worker never returns, losing its pre-panic capture —
///   the real-time stderr echo retains it, and the caller's worker_panicked
///   Error keeps status from reading clean).
pub fn capture<R>(f: impl FnOnce() -> R) -> (R, CapturedDiagnostics) {
  debug_assert!(
    LOG_BUFFER
      .try_borrow()
      .map(|b| b.is_none())
      .unwrap_or(false),
    "logger::capture on a thread with a pre-bound buffer — pooled-thread reuse?"
  );
  bind_log();
  let result = f();
  let log = flush_log();
  let counts = crate::common::error::snapshot_report_counts();
  (result, CapturedDiagnostics { log, counts })
}

/// Fold worker-thread diagnostics (from [`capture`]) into the main thread: append
/// the captured log text to the bound `LOG_BUFFER` and merge the count deltas
/// into the main `REPORT`. Call on the MAIN thread, in a deterministic order
/// (e.g. worker/job order), after the workers join. The worker already echoed
/// each line to the shared stderr fd in real time, so this does NOT re-print to
/// stderr — it only repairs the captured log + status tally.
pub fn replay_captured(d: CapturedDiagnostics) {
  debug_assert!(
    LOG_BUFFER.try_borrow().is_ok(),
    "replay_captured: LOG_BUFFER contended — captured text would be dropped"
  );
  if !d.log.is_empty()
    && let Ok(mut buf) = LOG_BUFFER.try_borrow_mut()
    && let Some(ref mut log) = *buf
  {
    // The captured text is already per-record newline-terminated; just make
    // sure it starts on a fresh line so it can't glue onto an in-flight note.
    if !log.is_empty() && !log.ends_with('\n') {
      log.push('\n');
    }
    log.push_str(&d.log);
  }
  crate::common::error::merge_report_counts(d.counts);
}

/// Strip ANSI escape sequences from a string for log file output.
fn strip_ansi(s: &str) -> String {
  // Match ESC[ ... m sequences
  let mut result = String::with_capacity(s.len());
  let mut in_escape = false;
  for c in s.chars() {
    if in_escape {
      if c == 'm' {
        in_escape = false;
      }
    } else if c == '\x1b' {
      in_escape = true;
    } else {
      result.push(c);
    }
  }
  result
}

/// Append a progress note to the capture buffer as INLINE flowing text — the
/// faithful Perl LaTeXML / tex.web terminal-progress format. `note_begin`
/// carries a leading '\n' so each stage opens on a fresh line; `note_end`
/// (` )`) and `note_progress` (`[1][2]…`, `N formulae …`) append inline so a
/// load's closing paren and the page markers stay on the SAME line as their
/// opener, and nested closes chain (`… 0.00 sec) 0.05 sec)`). A leading '\n'
/// is collapsed against an existing trailing '\n' (or buffer start) so we never
/// emit a blank line. Replaces the old unconditional `push('\n')` per note,
/// which put every `)` on its own line and doubled newlines into blank lines
/// (the reported `.latexml.log` noise on corpora.latexml.rs).
fn append_note(buf: &mut String, note: &str) {
  if note.starts_with('\n') && (buf.is_empty() || buf.ends_with('\n')) {
    buf.push_str(&note[1..]);
  } else {
    buf.push_str(note);
  }
}

/// prints a single line to STDERR
#[macro_export]
macro_rules! println_stderr(
    ($($arg:tt)*) => ({
      use std::io::Write;
      match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
        Ok(_) => {},
        Err(x) => panic!("Unable to write to stderr: {}", x),
      }
    })
);

/// prints a to STDERR without a line break
#[macro_export]
macro_rules! print_stderr(
    ($($arg:tt)*) => ({
      use std::io::Write;
      match write!(&mut ::std::io::stderr(), $($arg)* ) {
        Ok(_) => {},
        Err(x) => panic!("Unable to write to stderr: {}", x),
      }
    })
);

impl log::Log for LatexmlLogger {
  fn enabled(&self, metadata: &Metadata) -> bool { metadata.level() <= max_level() }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      let record_target = record.target();
      let details = record.args();
      if record_target == "note" {
        let note = details.to_string();
        // A note (e.g. `(Loading foo.sty… )`) is a live progress indicator — but when a capture
        // buffer is active it must ALSO land there so it reaches the flushed `cortex.log` and
        // CorTeX's `loaded_file` parser, which anchors on `^(Loading …`. `append_note` keeps it
        // INLINE (Perl-faithful: `(Loading X… )` on one line, `[1][2]…` chained) while preserving
        // the `^(Loading` anchor — every `note_begin` carries a leading '\n', so each load still
        // opens at line start. The following `Info:/Warning:` record re-asserts its own line break
        // (see the diagnostic-record path below), so the note can't glue onto its anchor.
        if let Ok(mut buf) = LOG_BUFFER.try_borrow_mut()
          && let Some(ref mut log) = *buf
        {
          append_note(log, &strip_ansi(&note));
        }
        // The LOG append above is the log-file record (Perl `$LOG` write, always).
        // The STDERR echo is the live console indicator and is gated on the
        // decoupled verbosity: `--quiet` silences the note here while the log
        // still keeps it (issue #763; Perl `ProgressSpinup` gates STDERR on
        // `$VERBOSITY >= 0`). Write and publish the cursor state under ONE stderr
        // lock, so a concurrent diagnostic record cannot observe a stale flag.
        if stderr_admits(record.level()) {
          use std::io::Write;
          let mut err = std::io::stderr().lock();
          let _ = err.write_all(note.as_bytes());
          let _ = err.flush();
          // A note carries no trailing newline of its own unless its text ends
          // in one (`note_begin` opens with a leading '\n'), so record where it
          // left the cursor for the next diagnostic record.
          STDERR_AT_LINE_START.store(note.ends_with('\n'), Ordering::Release);
        }
        return;
      }
      let category_object = if record_target.is_empty() {
        "" // "unknown:unknown" ???
      } else {
        record_target
      };
      // Following the reporting syntax at: https://math.nist.gov/~BMiller/LaTeXML/manual/errorcodes/
      // The severity word is the FULL Perl LaTeXML token (Info/Warning/Error/Fatal) — consumers
      // (CorTeX's log parser, the --server LSP) key on it, so it must match Perl exactly. In
      // particular WARN must serialize as `Warning` (not the abbreviated `Warn`): CorTeX maps an
      // unrecognized severity to Info, so `Warn:` silently misfiled every warning (see
      // LaTeXML/lib/LaTeXML/Common/Error.pm: `"Warning:" . $category . …`).
      let severity = if category_object.starts_with("Fatal:") {
        ""
      } else {
        match record.level() {
          Level::Info => "Info",
          Level::Warn => "Warning",
          Level::Error => "Error",
          Level::Debug => "Debug",
          Level::Trace => "Trace",
        }
      };

      // Tally every printed diagnostic record that was NOT emitted by a
      // counting macro (those count at raise time, even under output
      // suppression — the MAX_ERRORS cap depends on that). This is the
      // lossless half of the tally contract: any `Warning:`/`Error:` line
      // this backend prints from a raw `log::warn!`/`log::error!` call now
      // registers in `REPORT`, so the final "Conversion complete: N
      // warnings" agrees with what a reader greps from the log. Witness:
      // the 131 MB witness logged 12,105 `Warning:` lines (12,103 of them
      // the math parser's raw `log_math_warn!`) and reported "2 warnings".
      {
        use crate::common::error::{LogStatus, note_status_from_logger};
        let status = if category_object.starts_with("Fatal:") {
          Some(LogStatus::Fatal)
        } else {
          match record.level() {
            Level::Warn => Some(LogStatus::Warning),
            Level::Error => Some(LogStatus::Error),
            Level::Info => Some(LogStatus::Info),
            Level::Debug => Some(LogStatus::Debug),
            // No Trace counter exists; Trace records stay uncounted.
            Level::Trace => None,
          }
        };
        if let Some(status) = status {
          note_status_from_logger(status);
        }
      }
      let message = if severity.is_empty() {
        s!("{} ", category_object)
      } else {
        s!("{}:{} ", severity, category_object)
      };
      let painted_message = match record.level() {
        Level::Info => message,
        Level::Warn => paint(ANSI_YELLOW, &message),
        Level::Error => paint(ANSI_RED, &message),
        Level::Debug => paint(ANSI_GREEN, &message),
        _ => paint(ANSI_WHITE, &message),
      } + &details.to_string();

      // Capture to log buffer if active (strip ANSI for clean log text).
      // A diagnostic record (Info/Warning/Error/Fatal) must start on a fresh
      // line so CorTeX's line-anchored parser (^Error:/^Warning:/^Info:)
      // matches and the record never glues onto an in-flight progress note
      // (notes no longer force a trailing newline — see append_note).
      if let Ok(mut buf) = LOG_BUFFER.try_borrow_mut()
        && let Some(ref mut log) = *buf
      {
        if !log.is_empty() && !log.ends_with('\n') {
          log.push('\n');
        }
        log.push_str(&strip_ansi(&painted_message));
        // Exactly one trailing newline — a multi-detail message (e.g.
        // `Info:…loaded …\n\tat …\n\tIn …`) already ends with '\n', so an
        // unconditional push would double it into a blank line before the next
        // `(Loading …` note.
        if !log.ends_with('\n') {
          log.push('\n');
        }
      }

      // Use `\n` (not `\r`) to guarantee each log line starts on a fresh
      // line in both TTY and file output. The previous `\r` prefix made
      // log lines visually overlay any in-flight progress indicator like
      // `(Loading "foo.sty" definitions... )` — convenient in a terminal
      // but produced `(...)<CR>Error:...` byte sequences in log files,
      // breaking line-anchored counts in canvas harnesses
      // (`grep -cE '^...Error:'` silently returned 0 even when errors
      // were present). Trade-off: progress indicators in a TTY no longer
      // get overwritten, but they were not really self-erasing anyway
      // (they always emitted ` )` to close their parens), so the visual
      // change is small.
      // Colorize for an interactive terminal only; when stderr is redirected
      // to a file/pipe, emit the ANSI-stripped text so on-disk logs stay
      // grep-clean (matches the captured `.latexml.log` buffer above).
      // Break onto a fresh line ONLY when a note left the cursor mid-line —
      // an unconditional leading '\n' was 45.8% of the witness's log (see
      // STDERR_AT_LINE_START). One critical section: read the flag, write, and
      // republish it while holding the stderr lock, so the "record starts at
      // line start" guarantee survives concurrent loggers.
      //
      // The LOG_BUFFER append above is the log-file record (Perl `$LOG`, always,
      // subject only to the Info floor). The STDERR echo is gated on the decoupled
      // console verbosity so `--quiet` reduces the console without touching the log
      // (issue #763) — EXCEPT `Error`/`Fatal`, which always reach STDERR (the
      // project's deliberate always-emit-errors divergence, since cortex aggregates
      // success rates from `Error:`/`Fatal:` lines). `Error` is the lowest `Level`
      // value and Fatal records are emitted at `Level::Error`, so `<= Error`
      // captures both.
      let to_stderr = record.level() <= Level::Error || stderr_admits(record.level());
      if to_stderr {
        use std::io::Write;
        let text = if stderr_use_color() {
          painted_message
        } else {
          strip_ansi(&painted_message)
        };
        let mut err = std::io::stderr().lock();
        if !STDERR_AT_LINE_START.load(Ordering::Acquire) {
          let _ = err.write_all(b"\n");
        }
        let _ = err.write_all(text.as_bytes());
        let _ = err.write_all(b"\n");
        let _ = err.flush();
        STDERR_AT_LINE_START.store(true, Ordering::Release);
      }
    }
  }

  fn flush(&self) {}
}

/// initialize the logger at a given STDERR verbosity `level`
///
/// `level` is the **console** verbosity (`--quiet` ⇒ `Warn`, default ⇒ `Info`,
/// `--verbose`/`--debug` ⇒ `Debug`). Perl decouples this from the log-file floor:
/// the `.latexml.log` keeps a minimum of `Info` regardless of `--quiet`, while
/// STDERR follows `level` (`Common/Error.pm` `_printline`; issue #763). So the
/// global `log` `max_level` is set to the MORE verbose of `level` and `Info` (the
/// floor), admitting to `LatexmlLogger::log` everything the log needs; the STDERR
/// echo is then gated separately on the `STDERR_LEVEL` atomic.
///
/// Returns the underlying `SetLoggerError` if another `log` global logger
/// is already installed (e.g. an embedder set up `tracing-log` first).
/// Callers can decide whether to ignore that — the in-process `bind_log` /
/// `flush_log` buffers are independent of the `log` crate sink and keep
/// working either way.
pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
  // Global config (STDERR gate + log-file floor) is set unconditionally — it must
  // reflect the requested verbosity even if the logger was already installed (a
  // second `init` in-process returns `Err` from `set_logger`, which callers `.ok()`).
  STDERR_LEVEL.store(level as usize, Ordering::Relaxed);
  log::set_max_level(level.max(LevelFilter::Info));
  log::set_logger(&LOGGER)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn append_note_inline_and_no_blank_lines() {
    // note_begin opens a fresh line; note_end / note_progress stay inline.
    let mut b = String::new();
    append_note(&mut b, "\n(Loading keyval.sty..."); // note_begin (buffer empty: no leading blank)
    append_note(&mut b, " )"); // note_end inline
    assert_eq!(b, "(Loading keyval.sty... )");
    append_note(&mut b, "\n(Loading graphics.sty..."); // mid-line: keep the break
    append_note(&mut b, " )");
    assert_eq!(b, "(Loading keyval.sty... )\n(Loading graphics.sty... )");
    // page markers chain inline
    append_note(&mut b, "\n410 formulae ...");
    append_note(&mut b, "[1]");
    append_note(&mut b, "[2]");
    assert!(b.ends_with("410 formulae ...[1][2]"));
    // a leading '\n' note after a buffer already at line-start collapses (no blank line)
    let mut c = String::from("Info:foo\n");
    append_note(&mut c, "\n(Building...");
    assert_eq!(c, "Info:foo\n(Building...");
  }

  /// The lossless-tally contract (user directive 2026-08-02): every printed
  /// diagnostic record counts exactly once, whether it came from a counting
  /// macro (raise-time count, logger skips) or a raw `log::warn!`-family call
  /// (logger counts). Defect this pins: the 131 MB witness logged 12,105
  /// `Warning:` lines — 12,103 from the math parser's raw `log_math_warn!` —
  /// and the final verdict said "2 warnings".
  ///
  /// One test, not several: the global logger and the thread-local `REPORT`
  /// are both process/thread state, and cargo runs sibling `#[test]`s on
  /// separate threads — but a SINGLE test body sees one thread and one
  /// deterministic sequence.
  #[test]
  fn raw_log_records_count_and_macro_records_do_not_double_count() {
    use crate::common::error::{
      LogStatus, get_status, initialize_report, macro_diag_guard, note_status,
      note_status_from_logger,
    };
    // Another test may have installed the logger already; counting rides
    // note_status_from_logger either way, so drive it exactly as
    // `LatexmlLogger::log` does rather than through the global sink.
    initialize_report();

    // Raw record: the logger's tally path counts it.
    note_status_from_logger(LogStatus::Warning);
    assert_eq!(get_status(LogStatus::Warning), 1, "raw warning must count");

    // Macro-emitted record: raise-time count happens under the guard, and
    // the logger's observation of the same record must be a no-op.
    {
      let _g = macro_diag_guard();
      note_status(LogStatus::Warning, None); // what the macro does
      note_status_from_logger(LogStatus::Warning); // what the logger then sees
    }
    assert_eq!(
      get_status(LogStatus::Warning),
      2,
      "a macro warning counts once, not twice"
    );

    // Nested guards (Error! escalating into Fatal!) keep the marker set.
    {
      let _outer = macro_diag_guard();
      {
        let _inner = macro_diag_guard();
      }
      note_status_from_logger(LogStatus::Error);
    }
    assert_eq!(
      get_status(LogStatus::Error),
      0,
      "the logger stays muted for the whole macro emission, even after a \
       nested guard drops"
    );

    // Raw error and a raw Fatal-target record (log_fatal's shape when some
    // future caller bypasses it): the error counts, the fatal latches.
    note_status_from_logger(LogStatus::Error);
    note_status_from_logger(LogStatus::Fatal);
    assert_eq!(get_status(LogStatus::Error), 1);
    assert_eq!(get_status(LogStatus::Fatal), 1, "raw fatal latches sticky");
    initialize_report();
  }

  /// Suppression semantics (user decision 2026-08-03): Debug/Info/Warning
  /// records are muted under suppression, but Error and Fatal EMIT
  /// UNCONDITIONALLY — cortex-class frameworks aggregate success rates from
  /// `Error:`/`Fatal:` lines, and a suppressed error would vanish from that
  /// measurement while still counting locally. Counts accrue either way.
  #[test]
  fn suppression_never_mutes_error_or_fatal() {
    use crate::common::error::{
      LogStatus, emit_record, get_status, initialize_report, set_suppress_log_output,
    };
    initialize_report();
    let _ = log::set_logger(&LOGGER); // ok if another test installed it
    log::set_max_level(LevelFilter::Warn);
    let prev = set_suppress_log_output(true);

    bind_log();
    emit_record(LogStatus::Warning, "quiet:warn", "muted warning");
    emit_record(LogStatus::Error, "loud:error", "unmutable error");
    emit_record(LogStatus::Fatal, "Fatal:loud:fatal ", "unmutable fatal");
    let captured = flush_log();

    set_suppress_log_output(prev);
    assert!(
      !captured.contains("muted warning"),
      "suppression must mute Warning records, captured:\n{captured}"
    );
    assert!(
      captured.contains("Error:loud:error unmutable error"),
      "Error must emit under suppression, captured:\n{captured}"
    );
    // (Fatal targets carry their own trailing space, so the formatter's
    // separator makes it two — match the pieces, not the join.)
    assert!(
      captured.contains("Fatal:loud:fatal") && captured.contains("unmutable fatal"),
      "Fatal must emit under suppression, captured:\n{captured}"
    );
    // Counts accrue regardless of emission.
    assert_eq!(get_status(LogStatus::Warning), 1);
    assert_eq!(get_status(LogStatus::Error), 1);
    assert_eq!(get_status(LogStatus::Fatal), 1);
    initialize_report();
  }

  #[test]
  fn strip_ansi_removes_color_codes() {
    // ESC [ ... m should vanish; other content preserved.
    let red = "\x1b[31mhello\x1b[0m";
    assert_eq!(strip_ansi(red), "hello");
  }

  #[test]
  fn strip_ansi_noop_on_plain() {
    assert_eq!(strip_ansi("plain text"), "plain text");
    assert_eq!(strip_ansi(""), "");
  }

  #[test]
  fn strip_ansi_multiple_sequences() {
    let s = "\x1b[31merror:\x1b[0m \x1b[33mwarning\x1b[0m";
    assert_eq!(strip_ansi(s), "error: warning");
  }

  #[test]
  fn strip_ansi_preserves_unicode() {
    let s = "\x1b[31mαβγ\x1b[0m";
    assert_eq!(strip_ansi(s), "αβγ");
  }

  #[test]
  fn strip_ansi_handles_incomplete_escape() {
    // Unterminated ESC[ sequence — we should not hang.
    // Current impl: scan until 'm' is found. If never found, consumes
    // the rest of the input. Document that behavior.
    let s = "\x1b[1;31m hello";
    // The scan consumes characters until 'm' is found → the 'm' in the
    // escape closes, then " hello" remains.
    assert_eq!(strip_ansi(s), " hello");
  }

  #[test]
  fn bind_log_and_flush_log_roundtrip() {
    // Before bind_log, flush_log returns empty.
    // After bind_log, the buffer is active but empty until a log
    // message arrives. Since we can't easily exercise the Log impl
    // without initializing a global logger, just verify the
    // capture-buffer lifecycle primitives.
    let before = flush_log();
    assert!(before.is_empty(), "no active buffer → empty flush");

    bind_log();
    let after = flush_log();
    assert!(
      after.is_empty(),
      "empty buffer is still empty after bind/flush with no log traffic"
    );
  }
}
