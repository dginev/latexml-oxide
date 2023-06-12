use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::result;
use std::cell::RefCell;
use once_cell::sync::Lazy;
use crate::common::arena::{self,EMPTY_SYM};

use rustc_hash::{FxHashMap as HashMap};
use string_interner::symbol::SymbolU32;

#[derive(Debug,Clone,Default)]
pub struct LogState {
  pub undefined: HashMap<SymbolU32,usize>,
  pub missing: HashMap<SymbolU32,usize>,
  pub debug: usize,
  pub info: usize,
  pub warning: usize,
  pub error: usize,
  pub fatal: bool,
}
pub enum LogStatus {
  Debug,
  Info,
  Warning,
  Error,
  Fatal,
  Undefined,
  Missing,
}

#[thread_local]
pub static REPORT : Lazy<RefCell<LogState>> = Lazy::new(|| RefCell::new(LogState::default()));

pub fn note_status(status: LogStatus, what:Option<&str>) {
  let mut state = REPORT.borrow_mut();
  use LogStatus::*;
  match status {
    Debug => {state.debug += 1},
    Info => {state.info += 1},
    Warning => {state.warning += 1},
    Error => {state.error += 1},
    Fatal => {state.fatal = true},
    Undefined => {
      let entry = state.undefined.entry(
        what.map(arena::pin).unwrap_or(*EMPTY_SYM)).or_insert(0);
      *entry +=1;
    },
    Missing => {
      let entry = state.missing.entry(
        what.map(arena::pin).unwrap_or(*EMPTY_SYM)).or_insert(0);
      *entry +=1;
    },
  }
}

pub fn get_status(status: LogStatus) -> usize {
  let state = REPORT.borrow();
  use LogStatus::*;
  match status {
    Debug => state.debug,
    Info => state.info,
    Warning => state.warning,
    Error => state.error,
    Fatal => if state.fatal {1} else {0},
    _ => unimplemented!()
  }
}

pub fn init_report() {
  let mut report = REPORT.borrow_mut();
  *report = LogState::default();
}

#[macro_export]
macro_rules! Debug {
  ($category:expr, $object:expr, $where:ident, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!(target: &s!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $message:expr, $($details:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!(target: &s!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($where, $message, -1, $($details),*))
  }};
  ($($simple:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Debug, None);
    use log::debug;
    debug!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Info {
  ($category:expr, $object:expr, $where:ident, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $message:expr, $($details:expr),*) => {{
  $crate::common::error::note_status(
    $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!(target: &format!("{}:{}", $category, $object), "{}",
    $crate::generate_message!($where, $message, -1, $($details),*))
  }};
  ($($simple:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Info, None);
    use log::info;
    info!($($simple),*);
  }};

}

#[macro_export]
macro_rules! Warn {
  ($category:expr, $object:expr, $where:ident, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Warning, None);
    use log::warn;
    warn!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($where, $message, -1))
  }};
 ($category:expr, $object:expr, $where:ident, $message:expr, $($details:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Warning, None);
    use log::warn;
    warn!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($where, $message, -1, $($details),*))
  }}
}

#[macro_export]
macro_rules! Error {
  ($category:expr, $object:expr, $where:ident, $message:expr) => {{
    $crate::Error!($category,$object,$where,$message,"")
  }};
 ($category:expr, $object:expr, $where:ident, $message:expr, $($details:expr),*) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Error, None);
    use log::error;
    error!(target: &format!("{}:{}", $category, $object), "{}",
      $crate::generate_message!($where, $message, -1, $($details),*));
    let maxerrors = 100; //TODO: ($state ? $state->lookupValue('MAX_ERRORS') : 100);
    if $crate::common::error::get_status($crate::common::error::LogStatus::Error) > maxerrors {
      Fatal!(TooManyErrors, MaxLimit(maxerrors), $where, format!("Too many errors (> {maxerrors})!"));
    }
  }}
}

// TODO: flesh out the messages
#[macro_export]
macro_rules! Fatal {
  ($target:expr, $category:expr, $where:expr, $message:expr) => {{
    $crate::common::error::note_status(
      $crate::common::error::LogStatus::Fatal, None);
    fatal!($target, $category, $message);
  }};
}

#[macro_export]
macro_rules! fatal {
  ($target:expr, $category:expr, $message:expr) => {{
    use $crate::common::error::Error as RtxError;
    use $crate::common::error::ErrorCategory::*;
    use $crate::common::error::ErrorTarget::*;
    return Err(RtxError {
      target: $target,
      category: $category,
      message: $message.to_string(),
    });
  }};
  ($target:tt, $category:expr, $where: ident, $message:expr) => {{
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
    format!("{}\n\tIn {}:{}:{}\n", $message, file!(), line!(), column!())
  };
  (None, $message:expr, $level:literal, $detail:expr) => {
    format!(
      "{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $detail,
      file!(),
      line!(),
      column!()
    )
  };
  ($where:ident, $message:expr, $level:literal) => {
    format!(
      "{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $where.get_location(),
      file!(),
      line!(),
      column!()
    )
  };
  ($where:ident, $message:expr, $level:literal, $detail:expr) => {
    format!(
      "{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $where.get_location(),
      $detail,
      file!(),
      line!(),
      column!()
    )
  };
  ($where:ident, $message:expr, $level:literal, $detail:expr, $detail2:expr) => {
    format!(
      "{}\n\t{}\n\t{}\n\t{}\n\tIn {}:{}:{}\n",
      $message,
      $where.get_location(),
      $detail,
      $detail2,
      file!(),
      line!(),
      column!()
    )
  };
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
  pub target: ErrorTarget,
  pub category: ErrorCategory,
  pub message: String,
}
impl ErrorTrait for Error {}
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

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
  FailedParse,
  MaxLimit(usize),
  Generic(Box<dyn ErrorTrait>),
  Filename(String),
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
  MathParser,
  Document,
  Definition,
  TexPool,
  Internal,
  TargetUnexpected,
  SmuggledCatcode,
  TooManyErrors,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::ErrorCategory::*;
    match self.category {
      Init => write!(f, "Init"),
      Io(ref err) => err.fmt(f),
      NotFound => write!(f, "No matching cities with a population were found."),
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
      FailedParse => write!(f, "failed to parse"),
      MaxLimit(num) => write!(f, "{}", num),
      Generic(ref err) => err.fmt(f),
      Filename(ref name) => write!(f, "file:{name}"),
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

impl From<Box<dyn ErrorTrait>> for Error {
  fn from(err: Box<dyn ErrorTrait>) -> Error {
    Error {
      target: ErrorTarget::Document,
      message: err.to_string(),
      category: ErrorCategory::Generic(err),
    }
  }
}
impl From<Box<dyn ErrorTrait + Send + Sync>> for Error {
  fn from(err: Box<dyn ErrorTrait + Send + Sync>) -> Error {
    Error {
      target: ErrorTarget::Document,
      message: err.to_string(),
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
      message: err.to_string(),
      category: ErrorCategory::Generic(Box::new(err)),
    }
  }
}

impl From<ParseFloatError> for Error {
  fn from(err: ParseFloatError) -> Error {
    Error {
      target: ErrorTarget::Document,
      message: err.to_string(),
      category: ErrorCategory::Generic(Box::new(err)),
    }
  }
}

impl From<marpa::error::Error> for Error {
  fn from(err: marpa::error::Error) -> Error {
    Error {
      target: ErrorTarget::MathParser,
      category: ErrorCategory::FailedParse,
      message: err.to_string(),
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
