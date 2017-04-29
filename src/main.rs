#[macro_use]
extern crate log;
extern crate rtx_core;
extern crate rtx;

use std::env;
use std::process;
// use std::io::Write;
use rtx_core::common::{Config, OutputFormat, DataSize};
use rtx::converter::Converter;

fn main() {
  rtx_core::util::logger::init(log::LogLevelFilter::Info).unwrap_or(error!("Failed to load logger, aborting early. Please check rtx_core::util::logger installed correctly."));
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
  converter.prepare_session(&opts);
  // Perform the conversion:
  let response = converter.convert(source);
  match response {
    Ok(r) => {
      info!("{:?}\n\n", r.log);
      if let Some(xml) = r.result {
        info!("{}", xml);
      }
    }
    Err(e) => error!("Conversion error: {:?}", e),
  };


  // Normal exit
  process::exit(0);
}
