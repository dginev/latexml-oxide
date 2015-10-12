#[macro_use(println_stderr)]
extern crate rustexml;

use std::env;
use std::process;
use std::io::Write;
use rustexml::converter::{Converter};
use rustexml::common::{Config, OutputFormat, InputFormat, DataSize};

fn main() {
  let mut argv = env::args();
  println!("Welcome to {:?} -- a Rust implementation for LaTeXML", argv.next().unwrap());
  
  let source = match argv.next() {
    Some(s) => s,
    None => {
     println!("Please provide a source document! Exiting...");
     process::exit(1);
   }
  };
  // Prepare to convert:
  let opts = Config {
    verbosity : 0,
    format: OutputFormat::HTML5,
    whatsin: DataSize::Document,
    whatsout: DataSize::Document,
    preamble: None,
    postamble: None,
    mode: None
  };
  let mut converter = Converter::from_config(opts.clone());
  converter.prepare_session(&opts);
  // Perform the conversion:
  let response = converter.convert(source);
  match response {
    Ok(r) => {
      println_stderr!("{:?}", r.log);
      println!("{:?}
        ", r.result);
    },
    Err(e) => println_stderr!("Conversion error: {:?}", e)
  };


  // Normal exit
  process::exit(0);
}