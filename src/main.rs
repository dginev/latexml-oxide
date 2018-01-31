#[macro_use]
extern crate log;
extern crate rtx_core;
extern crate rtx;

use std::env;
use std::process;
use rtx_core::common::{Config, OutputFormat, DataSize};
use rtx::converter::Converter;

fn main() {
  if let Err(_) = rtx_core::util::logger::init(log::LevelFilter::Info) {
    error!("Failed to load logger, aborting early. Please check rtx_core::util::logger installed correctly.")
  }
  let mut argv = env::args();
  argv.next();
  info!("Welcome to rtx -- a Rust implementation for LaTeXML");

  let source = match argv.next() {
    Some(s) => s,
    None => {
      error!("Please provide a source document! Exiting...");
      process::exit(1);
    }
  };
  // Prepare to convert:
  let opts = Config {
    verbosity: 0,
    format: OutputFormat::HTML5,
    whatsin: DataSize::Document,
    whatsout: DataSize::Document,
    preamble: None,
    postamble: None,
    mode: None,
  };
  let mut converter = Converter::from_config(opts.clone());
  if let Err(e) = converter.prepare_session(&opts) {
    panic!(format!("Could not prepare converter session! : {}", e));
  }
  // Perform the conversion:
  let response = converter.convert(source);
  
  // TODO: Should never have to handle the response log for print?
  //       the right arguments can be passed in so that the response is either captured - and passed, or printed internally by the logger
  // info!("{:?}\n\n", r.log);
  if let Some(xml) = response.result {
    info!("{}", xml);
  }

  // Normal exit
  process::exit(0);
}
