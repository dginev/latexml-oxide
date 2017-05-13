use std::io;
use std::fmt;
use std::error::Error as ErrorTrait;
use std::result;

#[derive(Debug)]
pub struct Error {
  pub target: ErrorTarget,
  pub category: ErrorCategory,
  pub message: String
}

#[derive(Debug)]
pub enum ErrorTarget {
  Package,
  Parameter,
  Converster,
}

#[derive(Debug)]
pub enum ErrorCategory {
  Init,
  Io(io::Error),
  NotFound,
  // Unexpected,
  // Expected,
  Unknown,
  MissingFile,
}

#[macro_export]
macro_rules! fatal {
  ($target:tt, $category:tt, $message:expr) => ({
    use $crate::common::error::Error as RtxError;
    use $crate::common::error::ErrorTarget::*;
    use $crate::common::error::ErrorCategory::*;
    return Err(RtxError{
      target: $target, category: $category, message: $message
    })
  })
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::ErrorCategory::*;
    match self.category {
      Init => write!(f, "Init"),
      Io(ref err) => err.fmt(f),
      NotFound => write!(f, "No matching cities with a \
                                       population were found."),
      MissingFile => write!(f, "missing file"),
      Unknown => write!(f, "unknown")
    }
  }
}

impl ErrorTrait for Error {
  fn description(&self) -> &str {
    use self::ErrorCategory::*;
    match self.category {
      Init => "initialization failed",
      Io(ref err) => err.description(),
      MissingFile => "missing file",
      NotFound => "not found",
      Unknown => "unknown",
    }
  }

  fn cause(&self) -> Option<&ErrorTrait> {
    use self::ErrorCategory::*;
    match self.category {
      Io(ref err) => Some(err),
      // Our custom error doesn't have an underlying cause,
      // but we could modify it so that it does.
      _ => None,
    }
  }
}

pub fn note_end(note: &str) {
  info!("--|End:  | {:?}", note);
}

pub fn note_begin(note: &str) {
  info!("--|Begin:| {:?}", note);
}
