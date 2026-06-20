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
        // A note (e.g. `(Loading foo.sty… )`) is a live stderr progress indicator — but when a
        // capture buffer is active it must ALSO land there (on its own line), so it reaches the
        // flushed `cortex.log` and CorTeX's `loaded_file` parser, which anchors on `^(Loading …`.
        // Its own line both lets that `^`-anchored regex match and keeps the note from gluing onto a
        // following `Info:/Warning:` line (which would break that line's own anchor).
        if let Ok(mut buf) = LOG_BUFFER.try_borrow_mut()
          && let Some(ref mut log) = *buf
        {
          log.push_str(&strip_ansi(&note));
          log.push('\n');
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

      // Capture to log buffer if active (strip ANSI for clean log text)
      if let Ok(mut buf) = LOG_BUFFER.try_borrow_mut()
        && let Some(ref mut log) = *buf
      {
        log.push_str(&strip_ansi(&painted_message));
        log.push('\n');
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
