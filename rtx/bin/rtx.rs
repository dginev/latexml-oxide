#[macro_use]
extern crate rtx_core;
use rtx::converter::Converter;
use rtx_core::common::{Config, DataSize, OutputFormat};
use rtx_package::package;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::rc::Rc;
use std::result::Result;

fn main() -> Result<(), Box<dyn Error>> {
  if rtx_core::util::logger::init(log::LevelFilter::Info).is_err() {
    Error!(
      "rtx",
      "logger",
      None,
      None,
      "Failed to load logger. Please check rtx_core::util::logger installed correctly."
    );
  }
  let mut argv = env::args();
  argv.next();

  let source = match argv.next() {
    Some(s) => s,
    None => {
      Error!(
        "rtx",
        "",
        None,
        None,
        "Please provide a source document! Exiting..."
      );
      process::exit(1);
    },
  };
  let target = argv.next();
  // Prepare to convert:
  let opts = Config {
    verbosity: 0,
    format: OutputFormat::HTML5,
    whatsin: DataSize::Document,
    whatsout: DataSize::Document,
    preamble: None,
    postamble: None,
    mode: None,
    bindings_dispatch: Some(Rc::new(package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(rtx_contrib::dispatch)),
  };
  let mut converter = Converter::from_config(opts.clone());
  if let Err(e) = converter.prepare_session(&opts) {
    let message = s!("Could not prepare converter session! : {}", e);
    Error!("rtx", "session", None, None, message);
    process::exit(1);
  }
  // Perform the conversion:
  let response = converter.convert(source);

  // TODO: Should never have to handle the response log for print?
  // the right arguments can be passed in so that the response is either captured - and
  // passed, or printed internally by the logger info!("{:?}\n\n", r.log);
  if let Some(xml) = response.result {
    if let Some(target_path) = target {
      let mut out_fh = File::create(target_path)?;
      writeln!(out_fh, "{xml}")?;
    } else {
      println!("{xml}");
    }
  }

  // Normal exit
  process::exit(0);
}
