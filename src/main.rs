#[macro_use(println_stderr)]
extern crate rtx_core;
extern crate rtx;

use std::env;
use std::process;
// use std::io::Write;
use rtx_core::common::{Config, OutputFormat, DataSize};
use rtx::converter::Converter;

fn main() {
  let mut argv = env::args();
  argv.next();
  println!("Welcome to rtx -- a Rust implementation for LaTeXML");

  let source = match argv.next() {
    Some(s) => s,
    None => {
      println!("Please provide a source document! Exiting...");
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
      println_stderr!("{:?}\n\n", r.log);
      if let Some(xml) = r.result {
        println!("{}", xml);
      }
    }
    Err(e) => println_stderr!("Conversion error: {:?}", e),
  };


  // Normal exit
  process::exit(0);
}
