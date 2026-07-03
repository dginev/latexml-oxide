use std::cell::RefCell;

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

struct LatexmlLogger;
static LOGGER: LatexmlLogger = LatexmlLogger;

/// Thread-local log capture buffer. When enabled, log messages are
/// appended here (without ANSI colors) in addition to stderr.
#[thread_local]
static LOG_BUFFER: RefCell<Option<String>> = RefCell::new(None);

/// Start capturing log output into the buffer (Perl: bind_log).
pub fn bind_log() { *LOG_BUFFER.borrow_mut() = Some(String::new()); }

/// Flush and return the captured log output, stopping capture (Perl: flush_log).
pub fn flush_log() -> String { LOG_BUFFER.borrow_mut().take().unwrap_or_default() }

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
        print_stderr!("{}", note);
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
      if stderr_use_color() {
        println_stderr!("\n{}", painted_message);
      } else {
        println_stderr!("\n{}", strip_ansi(&painted_message));
      }
    }
  }

  fn flush(&self) {}
}

/// initialize the logger at a given verbosity `level`
///
/// Returns the underlying `SetLoggerError` if another `log` global logger
/// is already installed (e.g. an embedder set up `tracing-log` first).
/// Callers can decide whether to ignore that — the in-process `bind_log` /
/// `flush_log` buffers are independent of the `log` crate sink and keep
/// working either way.
pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
  log::set_logger(&LOGGER)?;
  log::set_max_level(level);
  Ok(())
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
