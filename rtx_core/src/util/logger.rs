extern crate log;
extern crate ansi_term;

use ansi_term::Style;
use ansi_term::Colour::{Yellow, Red, Green, White};
use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};

struct Log;

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


impl log::Log for Log {
  fn enabled(&self, metadata: &LogMetadata) -> bool {
    metadata.level() <= LogLevel::Info
  }

  fn log(&self, record: &LogRecord) {
    if self.enabled(record.metadata()) {
      let record_target = record.target();
      let category_object = if record_target.is_empty() {
       "" // "unknown:unknown" ???
      } else {
        record_target
      };
      // Following the reporting syntax at: http://dlmf.nist.gov/LaTeXML/manual/errorcodes/
      let severity = if category_object.starts_with("Fatal:") {
          ""
        } else { match record.level() {
          LogLevel::Info => "Info",
          LogLevel::Warn => "Warn",
          LogLevel::Error => "Error",
          LogLevel::Debug => "Debug",
          _ => ""
        } };      
      let details = record.args();

      let message = if severity.is_empty() {
        format!("{} ", category_object)
      } else {
        format!("{}:{} ", severity, category_object)
      };
      let painted_message = match record.level() {
        LogLevel::Info => Style::default().paint(message),
        LogLevel::Warn => Yellow.paint(message),
        LogLevel::Error => Red.paint(message),
        LogLevel::Debug => Green.paint(message),
        _ => White.paint(message)
      }.to_string() + &details.to_string();

      println_stderr!("{}", painted_message);
    }
  }
}

pub fn init(level : LogLevelFilter) -> Result<(), SetLoggerError> {
  log::set_logger(|max_log_level| {
    max_log_level.set(level);
    Box::new(Log)
  })
}