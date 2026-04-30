pub mod arena;
pub mod color;
#[macro_use]
pub mod error;
pub mod cleaners;
pub mod def_parser;
pub mod dimension;
pub mod float;
pub mod font;
pub mod glue;
pub mod ligature;
pub mod local_assignments;
pub mod locator;
pub mod mathchar;
pub mod model;
pub mod mudimension;
pub mod muglue;
pub mod number;
pub mod numeric_ops;
pub mod object;
pub mod pair;
pub mod relaxng;
pub mod store;
pub mod xml;

use crate::common::error::*;
use crate::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum InputFormat {
  TeX,
  Bib,
}
#[derive(Clone, Debug)]
pub enum OutputFormat {
  TeX,
  Box,
  XML,
  HTML5,
  XHTML,
}
#[derive(Clone, Debug)]
pub enum DataSize {
  Math,
  Fragment,
  Document,
  Archive,
}

#[derive(Clone, Debug)]
pub enum DigestionMode {
  TeX,
  LaTeX,
  AmSTeX,
  BibTeX,
}
impl fmt::Display for DigestionMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::DigestionMode::*;
    let formatted = match *self {
      TeX => "TeX",
      LaTeX => "LaTeX",
      AmSTeX => "AmSTeX",
      BibTeX => "BibTeX",
    };
    write!(f, "{formatted}")
  }
}

impl DigestionMode {
  pub fn extension(&self) -> String {
    match *self {
      DigestionMode::TeX | DigestionMode::LaTeX | DigestionMode::AmSTeX => "tex",
      DigestionMode::BibTeX => "bib",
    }
    .to_string()
  }
}

pub type BindingDispatcher = Rc<dyn Fn(&str) -> Option<Result<()>>>;

/// Perl: LABEL_MAPPING_HOOK => sub { ($label, $ctr, $norefnum) => ($refnum, $id) }
/// Returns (optional refnum string, optional id string)
pub type LabelMappingHook = Rc<dyn Fn(&str, &str, bool) -> (Option<String>, Option<String>)>;

#[derive(Clone)]
pub struct Config {
  pub verbosity:               i32,
  pub format:                  OutputFormat,
  pub whatsin:                 DataSize,
  pub whatsout:                DataSize,
  pub preamble:                Option<String>,
  pub postamble:               Option<String>,
  pub mode:                    Option<DigestionMode>,
  pub bindings_dispatch:       Option<BindingDispatcher>,
  pub extra_bindings_dispatch: Option<BindingDispatcher>,
  /// Packages to preload before processing (e.g. --preload=ar5iv.sty)
  pub preload:                 Option<Vec<String>>,
  /// Additional search paths for finding packages/inputs (e.g. --path=dir)
  pub search_paths:            Option<Vec<String>>,
  /// Whether to include XML comments in output (--nocomments sets false)
  pub include_comments:        Option<bool>,
  /// Whether to skip math parsing (--nomathparse)
  pub nomathparse:             Option<bool>,
}
impl Default for Config {
  fn default() -> Self {
    Config {
      verbosity:               1,
      format:                  OutputFormat::XML,
      whatsin:                 DataSize::Document,
      whatsout:                DataSize::Document,
      preamble:                None,
      postamble:               None,
      mode:                    Some(DigestionMode::LaTeX),
      bindings_dispatch:       None,
      extra_bindings_dispatch: None,
      preload:                 None,
      search_paths:            None,
      include_comments:        None,
      nomathparse:             None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn digestion_mode_display() {
    assert_eq!(format!("{}", DigestionMode::TeX), "TeX");
    assert_eq!(format!("{}", DigestionMode::LaTeX), "LaTeX");
    assert_eq!(format!("{}", DigestionMode::AmSTeX), "AmSTeX");
    assert_eq!(format!("{}", DigestionMode::BibTeX), "BibTeX");
  }

  #[test]
  fn digestion_mode_extension() {
    assert_eq!(DigestionMode::TeX.extension(), "tex");
    assert_eq!(DigestionMode::LaTeX.extension(), "tex");
    assert_eq!(DigestionMode::AmSTeX.extension(), "tex");
    assert_eq!(DigestionMode::BibTeX.extension(), "bib");
  }

  #[test]
  fn config_default_fields() {
    let c = Config::default();
    assert_eq!(c.verbosity, 1);
    assert!(matches!(c.format, OutputFormat::XML));
    assert!(matches!(c.whatsin, DataSize::Document));
    assert!(matches!(c.whatsout, DataSize::Document));
    assert!(c.preamble.is_none());
    assert!(c.postamble.is_none());
    assert!(matches!(c.mode, Some(DigestionMode::LaTeX)));
    assert!(c.bindings_dispatch.is_none());
    assert!(c.extra_bindings_dispatch.is_none());
    assert!(c.preload.is_none());
    assert!(c.search_paths.is_none());
    assert!(c.include_comments.is_none());
    assert!(c.nomathparse.is_none());
  }

  #[test]
  fn config_clone_preserves_fields() {
    let mut c = Config::default();
    c.verbosity = 5;
    c.preload = Some(vec!["ar5iv.sty".to_string()]);
    let c2 = c.clone();
    assert_eq!(c2.verbosity, 5);
    assert_eq!(c2.preload.as_ref().unwrap(), &vec!["ar5iv.sty".to_string()]);
  }

  #[test]
  fn input_format_variants() {
    // Debug trait at minimum is derived; Clone too.
    let _ = InputFormat::TeX;
    let _ = InputFormat::Bib;
    let cloned = InputFormat::TeX.clone();
    assert!(matches!(cloned, InputFormat::TeX));
  }

  #[test]
  fn output_format_variants() {
    let _ = OutputFormat::TeX;
    let _ = OutputFormat::Box;
    let _ = OutputFormat::XML;
    let _ = OutputFormat::HTML5;
    let _ = OutputFormat::XHTML;
    let cloned = OutputFormat::XML.clone();
    assert!(matches!(cloned, OutputFormat::XML));
  }

  #[test]
  fn data_size_variants() {
    let _ = DataSize::Math;
    let _ = DataSize::Fragment;
    let _ = DataSize::Document;
    let _ = DataSize::Archive;
    let cloned = DataSize::Document.clone();
    assert!(matches!(cloned, DataSize::Document));
  }
}
