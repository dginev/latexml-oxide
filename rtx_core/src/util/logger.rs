extern crate ansi_term;
extern crate log;

use ansi_term::Colour::{Green, Red, White, Yellow};
use ansi_term::Style;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct RtxLogger;
static LOGGER: RtxLogger = RtxLogger;

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

impl log::Log for RtxLogger {
  fn enabled(&self, metadata: &Metadata) -> bool { metadata.level() <= Level::Info }

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
          _ => "",
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
      }.to_string() + &details.to_string();

      println_stderr!("\r{}", painted_message);
    }
  }

  fn flush(&self) {}
}

pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
  log::set_logger(&LOGGER).unwrap();
  log::set_max_level(level);
  Ok(())
}
