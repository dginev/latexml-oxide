pub mod arena;
#[macro_use]
pub mod error;
pub mod cleaners;
pub mod def_parser;
pub mod dimension;
pub mod float;
pub mod font;
pub mod glue;
pub mod ligature;
pub mod locator;
pub mod model;
pub mod mudimension;
pub mod muglue;
pub mod number;
pub mod numeric_ops;
pub mod object;
pub mod relaxng;
pub mod store;
pub mod xml;

use crate::common::error::*;
use crate::fmt;
use crate::state::State;
use crate::stomach::Stomach;
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

pub type BindingDispatcher = Rc<dyn Fn(&str, &mut Stomach, &mut State) -> Option<Result<()>>>;

#[derive(Clone)]
pub struct Config {
  pub verbosity: i32,
  pub format: OutputFormat,
  pub whatsin: DataSize,
  pub whatsout: DataSize,
  pub preamble: Option<String>,
  pub postamble: Option<String>,
  pub mode: Option<DigestionMode>,
  pub bindings_dispatch: Option<BindingDispatcher>,
  pub extra_bindings_dispatch: Option<BindingDispatcher>,
}
impl Default for Config {
  fn default() -> Self {
    Config {
      verbosity: 1,
      format: OutputFormat::XML,
      whatsin: DataSize::Document,
      whatsout: DataSize::Document,
      preamble: None,
      postamble: None,
      mode: Some(DigestionMode::LaTeX),
      bindings_dispatch: None,
      extra_bindings_dispatch: None,
    }
  }
}
