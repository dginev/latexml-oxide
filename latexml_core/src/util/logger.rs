use ansi_term::Colour::{Green, Red, White, Yellow};
use ansi_term::Style;
use log::max_level;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::cell::RefCell;

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
        // simple print here
        print_stderr!("{}", details.to_string());
        return;
      }
      let category_object = if record_target.is_empty() {
        "" // "unknown:unknown" ???
      } else {
        record_target
      };
      // Following the reporting syntax at: http://dlmf.nist.gov/LaTeXML/manual/errorcodes/
      let severity = if category_object.starts_with("Fatal:") {
        ""
      } else {
        match record.level() {
          Level::Info => "Info",
          Level::Warn => "Warn",
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
        Level::Info => Style::default().paint(message),
        Level::Warn => Yellow.paint(message),
        Level::Error => Red.paint(message),
        Level::Debug => Green.paint(message),
        _ => White.paint(message),
      }
      .to_string()
        + &details.to_string();

      // Capture to log buffer if active (strip ANSI for clean log text)
      if let Ok(mut buf) = LOG_BUFFER.try_borrow_mut() {
        if let Some(ref mut log) = *buf {
          log.push_str(&strip_ansi(&painted_message));
          log.push('\n');
        }
      }

      println_stderr!("\r{}", painted_message);
    }
  }

  fn flush(&self) {}
}

/// initialize the logger at a given verbosity `level`
pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
  log::set_logger(&LOGGER).unwrap();
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
