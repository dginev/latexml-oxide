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

pub struct Config {
  pub verbosity : usize,
  pub format : OutputFormat,
  pub whatsin : DataSize,
  pub whatsout : DataSize,
  pub preamble : Option<String>,
  pub postamble : Option<String>
}
impl Clone for Config {
  fn clone(&self) -> Self {
    Config {
      verbosity : self.verbosity.clone(),
      format : self.format.clone(),
      whatsin : self.whatsin.clone(),
      whatsout : self.whatsout.clone(),
      preamble : self.preamble.clone(),
      postamble : self.postamble.clone()
    }
  }
}