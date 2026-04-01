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
  // Parse --codegen=<dump_path> flag: generate Rust module from a dump file
  let codegen_flag = extract_flag(&mut argv, "--codegen");
  // Parse --post flag: enable post-processing (MathML, XSLT)
  let post_flag = argv.iter().any(|a| a == "--post");
  let pmml_flag = argv.iter().any(|a| a == "--pmml");
  let keep_xmath_flag = argv.iter().any(|a| a == "--keepXMath" || a == "--xmath");
  let stylesheet_flag = extract_flag(&mut argv, "--stylesheet");
  let format_flag = extract_flag(&mut argv, "--format");
  // Remove boolean flags
  argv.retain(|a| !["--post", "--pmml", "--keepXMath", "--xmath",
    "--noscan", "--nocrossref"].contains(&a.as_str()));

  // Codegen mode doesn't need a source file — handle it early.
  if let Some(dump_path) = codegen_flag {
    let output = dest_flag.unwrap_or_else(|| "latex_dump.rs".to_string());
    match latexml::ini_tex::codegen_from_dump(&dump_path, &output) {
      Ok(count) => {
        eprintln!("Codegen complete: {} entries → {}", count, output);
        process::exit(0);
      }
      Err(e) => {
        eprintln!("Codegen failed: {}", e);
        process::exit(1);
      }
    }
  }

  let source = if let Some(ref init) = init_file {
    init.clone()
  } else {
    match argv.first() {
      Some(s) => s.clone(),
      None => {
        eprintln!("Usage: latexml_oxide [--post] [--pmml] [--keepXMath] [--stylesheet=path.xsl] [--dest=output] <source>");
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
      // Auto-select stylesheet from --format
      let effective_stylesheet = stylesheet_flag.clone().or_else(|| {
        match format_flag.as_deref() {
          Some("html5") | Some("html") | Some("xhtml") =>
            Some("resources/XSLT/LaTeXML-all-xhtml.xsl".to_string()),
          Some("epub") | Some("epub3") =>
            Some("resources/XSLT/LaTeXML-epub3.xsl".to_string()),
          _ => None,
        }
      });
      let do_post = post_flag || pmml_flag || effective_stylesheet.is_some() || format_flag.is_some();

      if do_post {
        // Post-process the XML
        let output = run_post_processing(
          &xml, pmml_flag || post_flag || format_flag.is_some(),
          keep_xmath_flag,
          effective_stylesheet.as_deref(),
        );
        if let Some(target_path) = target {
          let mut out_fh = File::create(target_path)?;
          write!(out_fh, "{output}")?;
        } else {
          print!("{output}");
        }
      } else {
        // Output raw XML
        if let Some(target_path) = target {
          let mut out_fh = File::create(target_path)?;
          write!(out_fh, "{xml}")?;
        } else {
          print!("{xml}");
        }
      }
    }
  }

  process::exit(0);
}

/// Run the post-processing pipeline on XML output.
fn run_post_processing(
  xml: &str,
  pmml: bool,
  keep_xmath: bool,
  stylesheet: Option<&str>,
) -> String {
  use latexml_post::document::{PostDocument, PostDocumentOptions};
  use latexml_post::processor::Processor;

  let doc = match PostDocument::new_from_string(xml, PostDocumentOptions::default()) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Post-processing: failed to parse XML: {}", e);
      return xml.to_string();
    }
  };

  let mut post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  if pmml {
    let mathml = latexml_post::mathml::MathML::new_presentation()
      .with_keep_xmath(keep_xmath);
    processors.push(Box::new(mathml));
  }

  if let Some(xsl_path) = stylesheet {
    let searchpaths = vec![
      "resources/XSLT".to_string(),
      ".".to_string(),
    ];
    match latexml_post::xslt::XSLT::new(
      xsl_path, std::collections::HashMap::new(), false, None, searchpaths
    ) {
      Ok(xslt) => processors.push(Box::new(xslt)),
      Err(e) => eprintln!("Post-processing: XSLT error: {}", e),
    }
  }

  match post.process_chain(doc, &mut processors) {
    Ok(results) => results[0].to_xml_string(),
    Err(e) => {
      eprintln!("Post-processing failed: {}", e);
      xml.to_string()
    }
  }
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
