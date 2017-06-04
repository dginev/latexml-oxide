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
  Converter,
  Mouth,
  Codegen,
  Macro,
  XMath,
  Document,
  Definition,
}

#[derive(Debug)]
pub enum ErrorCategory {
  Init,
  Io(io::Error),
  NotFound,
  Unexpected,
  Expected,
  Unknown,
  MissingFile,
  Malformed,
  Libxml,
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
      Unknown => write!(f, "unknown"),
      Malformed => write!(f, "malformed"),
      Expected => write!(f, "expected"),
      Unexpected => write!(f, "unexpected"),
      Libxml => write!(f, "libxml error")
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
      Malformed => "malformed",
      Expected => "expected",
      Unexpected => "unexpected",
      Libxml => "libxml error"
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
    let target_str = format!("Fatal:{:?}:{:?} ",self.target, self.category);
    error!(target: &target_str, "{}", self.message);
  }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error{target: ErrorTarget::Mouth, category: ErrorCategory::Io(err), message: "IO error".to_owned()}
    }
}

impl From<()> for Error {
    fn from(_e: ()) -> Error {
        Error{target: ErrorTarget::Document, category: ErrorCategory::Libxml, message: "LibXML error".to_owned()}
    }
}


//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Progress Reporting
//**********************************************************************
// Progress reporting.

pub fn note_progress(stuff: &str) {
  // my $state = $STATE;
  // my $verbosity = $state && $state->lookupValue('VERBOSITY') || 0;
  // print STDERR @stuff if $verbosity >= 0;
  info!(target: "note", "{}", stuff);
}

pub fn note_progress_detailed(stuff: &str) {
  // my $state = $STATE;
  // my $verbosity = $state && $state->lookupValue('VERBOSITY') || 0;
  // print STDERR @stuff if $verbosity >= 1;
  info!(target: "note", "{}", stuff);
}

pub fn note_begin(stage: &str) {
  // my ($stage) = @_;
  // my $state = $STATE;
  // my $verbosity = $state && $state->lookupValue('VERBOSITY') || 0;
  // if ($state && ($verbosity >= 0)) {
  // $state->assignMapping('NOTE_TIMERS', $stage, [Time::HiRes::gettimeofday]);
  info!(target: "note", "\n({}...", stage);
}


pub fn note_end(_stage: &str) {
  // my ($stage) = @_;
  // my $state = $STATE;
  // my $verbosity = $state && $state->lookupValue('VERBOSITY') || 0;
  // if (my $start = $state && $state->lookupMapping('NOTE_TIMERS', $stage)) {
  //   $state->assignMapping('NOTE_TIMERS', $stage, undef);
    // if ($verbosity >= 0) {
      // my $elapsed = Time::HiRes::tv_interval($start, [Time::HiRes::gettimeofday]);
  // info!(target: "note", " %.2f sec)", elapsed);
  info!(target: "note", " )");
}
