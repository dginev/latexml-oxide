#[derive(Clone)]
pub enum InputFormat {
  TeX,
  Bib,
}
#[derive(Clone)]
pub enum OutputFormat {
  TeX,
  Box,
  XML,
  HTML5,
  XHTML
}
#[derive(Clone)]
pub enum DataSize {
  Math,
  Fragment,
  Document,
  Archive
}
#[derive(Clone, Debug)]
pub enum Error {
  Unexpected,
  Expected
}
#[derive(Clone)]
pub enum DigestionMode {
  TeX,
  LaTeX,
  AmSTeX,
  BibTeX
}
impl DigestionMode {
  pub fn extension(&self) -> String {
    match *self {
      DigestionMode::TeX => "tex",
      DigestionMode::LaTeX => "tex",
      DigestionMode::AmSTeX => "tex",
      DigestionMode::BibTeX => "bib"
    }.to_string()
  }
}
#[derive(Clone)]
pub struct Config {
  pub verbosity : i32,
  pub format : OutputFormat,
  pub whatsin : DataSize,
  pub whatsout : DataSize,
  pub preamble : Option<String>,
  pub postamble : Option<String>,
  pub mode : Option<DigestionMode>,
}
impl Config {
  pub fn new() -> Config {
  Config {
    verbosity : 1,
    format : OutputFormat::XML,
    whatsin : DataSize::Document,
    whatsout : DataSize::Document,
    preamble : None,
    postamble : None,
    mode : Some(DigestionMode::LaTeX)
  }
  }
}