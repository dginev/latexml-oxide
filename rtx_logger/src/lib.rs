extern crate log;

use log::{LogRecord, LogLevel, LogMetadata, SetLoggerError, LogLevelFilter};

struct RtxLogger;

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


impl log::Log for RtxLogger {
  fn enabled(&self, metadata: &LogMetadata) -> bool {
    metadata.level() <= LogLevel::Info
  }

  fn log(&self, record: &LogRecord) {
    if self.enabled(record.metadata()) {
      println_stderr!("{} - {}", record.level(), record.args());
    }
  }
}

pub fn init() -> Result<(), SetLoggerError> {
  log::set_logger(|max_log_level| {
    max_log_level.set(LogLevelFilter::Info);
    Box::new(RtxLogger)
  })
}