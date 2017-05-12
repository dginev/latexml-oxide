extern crate log;
extern crate ansi_term;

use ansi_term::ANSIByteString;
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
      // Following the reporting syntax at: http://dlmf.nist.gov/LaTeXML/manual/errorcodes/
      let severity = match record.level() {
        LogLevel::Info => "Info",
        LogLevel::Warn => "Warn",
        LogLevel::Error => "Error",
        LogLevel::Debug => "Debug",
        _ => ""
      };
      let record_target = record.target();
      let category_object = if record_target.is_empty() {
       "" // "unknown:unknown" ???
      } else {
        record_target
      };
      let details = record.args();

      let message = format!("{}:{} {}\n", severity, category_object, details);
      let painted_message : ANSIByteString = match record.level() {
        LogLevel::Info => Style::default().paint(message.as_bytes()),
        LogLevel::Warn => Yellow.paint(message.as_bytes()),
        LogLevel::Error => Red.paint(message.as_bytes()),
        LogLevel::Debug => Green.paint(message.as_bytes()),
        _ => White.paint(message.as_bytes())
      };

      match painted_message.write_to((&mut ::std::io::stderr())) {
        Ok(_) => {},
        Err(x) => panic!("Unable to write to stderr: {}", x),
      };
    }
  }
}

pub fn init(level : LogLevelFilter) -> Result<(), SetLoggerError> {
  log::set_logger(|max_log_level| {
    max_log_level.set(level);
    Box::new(Log)
  })
}