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
  let mut argv: Vec<String> = env::args().skip(1).collect();

  // Parse --init=<file> flag (Perl: latexml --init=latex.ltx --dest=dump.ltxml)
  let init_file = extract_flag(&mut argv, "--init");
  let dest_flag = extract_flag(&mut argv, "--dest");

  let source = if let Some(ref init) = init_file {
    init.clone()
  } else {
    match argv.first() {
      Some(s) => s.clone(),
      None => {
        eprintln!("Usage: latexml_oxide [--init=<file> --dest=<dump>] <source> [<destination>]");
        process::exit(1);
      }
    }
  };
  let target = dest_flag.or_else(|| argv.get(1).cloned());

  // Prepare converter
  let opts = Config {
    verbosity:               0,
    format:                  OutputFormat::HTML5,
    whatsin:                 DataSize::Document,
    whatsout:                DataSize::Document,
    preamble:                None,
    postamble:               None,
    mode:                    None,
    bindings_dispatch:       Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
  };
  let mut converter = Converter::from_config(opts.clone());
  if let Err(e) = converter.prepare_session(&opts) {
    eprintln!("Could not prepare converter session: {}", e);
    process::exit(1);
  }

  if init_file.is_some() {
    // Init mode: process file and dump state (Perl: iniTeX)
    match latexml::ini_tex::dump_format(&mut converter, &source, target.as_deref()) {
      Ok(count) => {
        eprintln!("Format dump complete: {} entries written", count);
      }
      Err(e) => {
        eprintln!("Format dump failed: {}", e);
        process::exit(1);
      }
    }
  } else {
    // Normal mode: convert document
    let response = converter.convert(source);
    if let Some(xml) = response.result {
      if let Some(target_path) = target {
        let mut out_fh = File::create(target_path)?;
        write!(out_fh, "{xml}")?;
      } else {
        print!("{xml}");
      }
    }
  }

  process::exit(0);
}

/// Extract a --flag=value from argv, removing it if found.
fn extract_flag(argv: &mut Vec<String>, prefix: &str) -> Option<String> {
  let eq_prefix = format!("{}=", prefix);
  if let Some(pos) = argv.iter().position(|a| a.starts_with(&eq_prefix)) {
    let val = argv[pos][eq_prefix.len()..].to_string();
    argv.remove(pos);
    Some(val)
  } else {
    None
  }
}
