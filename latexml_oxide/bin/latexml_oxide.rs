#[macro_use]
extern crate latexml_core;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn Error>> {
  if latexml_core::util::logger::init(log::LevelFilter::Info).is_err() {
    let err = || {
      Error!(
        "latexml",
        "logger",
        "Failed to load logger. Please check latexml_core::util::logger installed correctly."
      );
      Ok(())
    };
    err().ok();
  }
  let mut argv = env::args();
  argv.next();

  let source = match argv.next() {
    Some(s) => s,
    None => {
      let err = || {
        Error!(
          "latexml_oxide",
          "",
          "Please provide a source document! Exiting..."
        );
        Ok(())
      };
      err().ok();
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
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
  };
  let mut converter = Converter::from_config(opts.clone());
  if let Err(e) = converter.prepare_session(&opts) {
    let message = s!("Could not prepare converter session! : {}", e);
    let err = || {
      Error!("latexml_oxide", "session", message);
      Ok(())
    };
    err().ok();
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
