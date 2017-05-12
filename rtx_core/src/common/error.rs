use std::io;
use std::fmt;
use std::error::Error;
use std::result;

#[derive(Debug)]
pub enum RtxError {
  Io(io::Error),
  NotFound,
  // Unexpected,
  // Expected,
  MissingFile(String),
}

pub type Result<T> = result::Result<T, RtxError>;

impl fmt::Display for RtxError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      RtxError::Io(ref err) => err.fmt(f),
      RtxError::NotFound => write!(f, "No matching cities with a \
                                       population were found."),
      RtxError::MissingFile(ref name) => write!(f, "Missing file: {}", name)
    }
  }
}

impl Error for RtxError {
  fn description(&self) -> &str {
    match *self {
      RtxError::Io(ref err) => err.description(),
      RtxError::MissingFile(ref name) => "missing file",
      RtxError::NotFound => "not found",
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      RtxError::Io(ref err) => Some(err),
      // Our custom error doesn't have an underlying cause,
      // but we could modify it so that it does.
      RtxError::NotFound => None,
      RtxError::MissingFile(_) => None,
    }
  }
}

pub fn note_end(note: &str) {
  info!("--|End:  | {:?}", note);
}

pub fn note_begin(note: &str) {
  info!("--|Begin:| {:?}", note);
}
