use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::result;

lazy_static! {
  static ref _NOTE_TIMERS: HashMap<String, String> = HashMap::new();
}


#[macro_export]
macro_rules! Debug {
  ($category:expr, $object:expr, $where:ident, None, $message:expr) => {{
    // $state.note_status("debug"); // TODO: We're losing the info count this way...
    use log::debug;
    debug!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, None, $message:expr, $($details:expr),*) => {{
    // $state.note_status("debug"); // TODO: We're losing the debug count this way...
    use log::debug;
    debug!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }};
  ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr) => {{
    use log::debug;
    $state.note_status("debug");
    debug!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr, $($details:expr),*) => {{
    use log::debug;
    $state.note_status("debug");
    debug!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }};
  ($($simple:expr),*) => {{
    // $state.note_status("debug"); // TODO: We're losing the info count this way...
    use log::debug;
    debug!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Info {
  ($category:expr, $object:expr, $where:ident, None, $message:expr) => {{
    use log::info;
    // $state.note_status("info"); // TODO: We're losing the info count this way...
    info!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, None, $message:expr, $($details:expr),*) => {{
    // $state.note_status("info"); // TODO: We're losing the info count this way...
    use log::info;
    info!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }};
  ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr) => {{
    $state.note_status("info");
    use log::info;
    info!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr, $($details:expr),*) => {{
    $state.note_status("info");
    use log::info;
    info!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }};
  ($($simple:expr),*) => {{
    // $state.note_status("debug"); // TODO: We're losing the info count this way...
    use log::info;
    info!($($simple),*);
  }};
}

#[macro_export]
macro_rules! Warn {
  ($category:expr, $object:expr, $where:ident, None, $message:expr) => {{
    // $state.note_status("warn"); // TODO: We're losing the warn count this way...
    use log::warn;
    warn!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, None, $message:expr, $($details:expr),*) => {{
    // $state.note_status("warn"); // TODO: We're losing the warn count this way...
    use log::warn;
    warn!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }};
  ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr) => {{
    $state.note_status("warn");
    use log::warn;
    warn!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr, $($details:expr),*) => {{
    $state.note_status("warn");
    use log::warn;
    warn!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }}
}

#[macro_export]
macro_rules! Error {
  ($category:expr, $object:expr, $where:ident, None, $message:expr) => {{
    // $state.note_status("error"); // TODO: We're losing the error count this way...
    use log::error;
    error!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, None, $message:expr, $($details:expr),*) => {{
    // $state.note_status("error"); // TODO: We're losing the error count this way...
    use log::error;
    error!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }};
  ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr) => {{
    $state.note_status("error");
    use log::error;
    error!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $state:expr, $message:expr, $($details:expr),*) => {{
    $state.note_status("error");
    use log::error;
    error!(target: &s!("{}:{}", $category, $object), "{}", generate_message!($where, $message, -1, $($details),*))
  }}
}


#[macro_export]
macro_rules! fatal {
  ($target:tt, $category:tt, $message:expr) => {{
    use $crate::common::error::Error as RtxError;
    use $crate::common::error::ErrorCategory::*;
    use $crate::common::error::ErrorTarget::*;
    return Err(RtxError {
      target: $target,
      category: $category,
      message: $message.to_string(),
    });
  }};
  ($target:tt, $category:expr, $where: ident, $state: ident, $message:expr) => {{
    use $crate::common::error::Error as RtxError;
    use $crate::common::error::ErrorCategory::*;
    use $crate::common::error::ErrorTarget::*;
    return Err(RtxError {
      target: $target,
      category: $category,
      message: $message.to_string(),
    });
  }};

}

#[macro_export]
macro_rules! generate_message {
  (None, $message:expr, $level:literal) => {
    s!("{}\n\tIn {}:{}:{}\n", $message, file!(), line!(), column!())
  };
  (None, $message:expr, $level:literal, $detail:expr) => {
    s!("{}\n\t{}\n\tIn {}:{}:{}\n", $message, $detail, file!(), line!(), column!())
  };
  ($where:ident, $message:expr, $level:literal) => {
    s!("{}\n\t{}\n\tIn {}:{}:{}\n", $message, $where.get_location(), file!(), line!(), column!())
  };
  ($where:ident, $message:expr, $level:literal, $detail:expr) => {
    s!("{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n", $message, $where.get_location(),$detail, file!(), line!(), column!());
  }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
  pub target: ErrorTarget,
  pub category: ErrorCategory,
  pub message: String,
}

#[derive(Debug)]
pub enum ErrorCategory {
  Init,
  Io(io::Error),
  NotFound,
  Unexpected,
  Expected,
  Misdefined,
  Unknown,
  MissingFile,
  Malformed,
  Libxml,
  Convert,
  Recursion,
  EoF,
  Endgroup,
  Generic(Box<ErrorTrait>),
  Filename(String)
}

#[derive(Debug)]
pub enum ErrorTarget {
  Package,
  Parameter,
  ParamSpec,
  Prototype,
  Converter,
  Mouth,
  Core,
  State,
  Stomach,
  Codegen,
  Macro,
  XMath,
  Document,
  Definition,
  TexPool,
  Internal,
  TargetUnexpected,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::ErrorCategory::*;
    match self.category {
      Init => write!(f, "Init"),
      Io(ref err) => err.fmt(f),
      NotFound => write!(
        f,
        "No matching cities with a \
         population were found."
      ),
      MissingFile => write!(f, "missing file"),
      Misdefined => write!(f, "misdefined"),
      Unknown => write!(f, "unknown"),
      Malformed => write!(f, "malformed"),
      Expected => write!(f, "expected"),
      Unexpected => write!(f, "unexpected"),
      Libxml => write!(f, "libxml error"),
      Recursion => write!(f, "<recursion>"),
      EoF => write!(f, "<EOF>"),
      Convert => write!(f, "conversion"),
      Endgroup => write!(f, "<endgroup>"),
      Generic(ref err) => err.fmt(f),
      Filename(ref name) => write!(f, "file:{}", name)
    }
  }
}

impl ErrorTrait for Error {
  fn description(&self) -> &str {
    use self::ErrorCategory::*;
    match self.category {
      Init => "initialization failed",
      Io(ref err) => err.description(),
      Convert => "conversion",
      MissingFile => "missing file",
      NotFound => "not found",
      Misdefined => "misdefined",
      Unknown => "unknown",
      Malformed => "malformed",
      Expected => "expected",
      Unexpected => "unexpected",
      Libxml => "libxml error",
      Recursion => "<recursion>",
      EoF => "<EOF>",
      Endgroup => "<endgroup>",
      Generic(ref err) => err.description(),
      Filename(ref name) => name
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

impl Error {
  pub fn log_fatal(&self) {
    let target_str = s!("Fatal:{:?}:{:?} ", self.target, self.category);
    use log::error;
    error!(target: &target_str, "{}", self.message);
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Error {
    Error {
      target: ErrorTarget::Mouth,
      category: ErrorCategory::Io(err),
      message: s!("IO error"),
    }
  }
}

impl From<Box<ErrorTrait>> for Error {
  fn from(err: Box<ErrorTrait>) -> Error {
    Error {
      target: ErrorTarget::Document,
      message: err.description().to_string(),
      category: ErrorCategory::Generic(err),
    }
  }
}

impl From<String> for Error {
  fn from(err: String) -> Error {
    Error {
      target: ErrorTarget::Document,
      category: ErrorCategory::Generic(From::from(err.clone())),
      message: err,
    }
  }
}

impl<'a> From<&'a str> for Error {
  fn from(err: &'a str) -> Error {
    Error {
      target: ErrorTarget::Document,
      category: ErrorCategory::Generic(From::from(err.to_owned())),
      message: err.to_owned(),
    }
  }
}

impl From<()> for Error {
  fn from(_e: ()) -> Error {
    Error {
      target: ErrorTarget::Document,
      category: ErrorCategory::Libxml,
      message: s!("LibXML error"),
    }
  }
}

impl From<ParseIntError> for Error {
  fn from(err: ParseIntError) -> Error {
    Error {
      target: ErrorTarget::Document,
      message: err.description().to_string(),
      category: ErrorCategory::Generic(Box::new(err)),
    }
  }
}

impl From<ParseFloatError> for Error {
  fn from(err: ParseFloatError) -> Error {
    Error {
      target: ErrorTarget::Document,
      message: err.description().to_string(),
      category: ErrorCategory::Generic(Box::new(err)),
    }
  }
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Progress Reporting
//**********************************************************************
// Progress reporting.

pub fn note_progress(stuff: &str) {
  use log::info;
  info!(target: "note", "{}", stuff);
}

// TODO: Rethink this reporting
pub fn note_progress_detailed(stuff: &str) {
  use log::debug;
  debug!(target: "note", "{}", stuff);
}

pub fn note_begin(stage: &str) {
  // $state->assignMapping('NOTE_TIMERS', $stage, [Time::HiRes::gettimeofday]);
  use log::info;
  info!(target: "note", "\n({}...", stage);
}

pub fn note_end(_stage: &str) {
  // if (my $start = $state && $state->lookupMapping('NOTE_TIMERS', $stage)) {
  //   $state->assignMapping('NOTE_TIMERS', $stage, undef);

  // my $elapsed = Time::HiRes::tv_interval($start, [Time::HiRes::gettimeofday]);
  // info!(target: "note", " %.2f sec)", elapsed);
  use log::info;
  info!(target: "note", " )");
}
