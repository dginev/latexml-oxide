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
  let cmml_flag = argv.iter().any(|a| a == "--cmml");
  let keep_xmath_flag = argv.iter().any(|a| a == "--keepXMath" || a == "--xmath");
  let stylesheet_flag = extract_flag(&mut argv, "--stylesheet");
  let format_flag = extract_flag(&mut argv, "--format");
  let nodefaultresources_flag = argv.iter().any(|a| a == "--nodefaultresources");
  let nocomments_flag = argv.iter().any(|a| a == "--nocomments");
  let nomathparse_flag = argv.iter().any(|a| a == "--nomathparse");
  let noinvisibletimes_flag = argv.iter().any(|a| a == "--noinvisibletimes");
  // Repeatable flags
  let css_flags = extract_flags(&mut argv, "--css");
  let preload_flags = extract_flags(&mut argv, "--preload");
  let path_flags = extract_flags(&mut argv, "--path");
  // Timeout
  let _timeout_flag = extract_flag(&mut argv, "--timeout"); // TODO: implement timeout (A10)
  // Remove boolean flags
  argv.retain(|a| !["--post", "--pmml", "--cmml", "--keepXMath", "--xmath",
    "--noscan", "--nocrossref", "--nodefaultresources", "--nocomments",
    "--nomathparse", "--noinvisibletimes"].contains(&a.as_str()));

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
    preload:                 if preload_flags.is_empty() { None } else { Some(preload_flags) },
    search_paths:            if path_flags.is_empty() { None } else { Some(path_flags) },
    include_comments:        if nocomments_flag { Some(false) } else { None },
    nomathparse:             if nomathparse_flag { Some(true) } else { None },
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
          Some("html5") =>
            Some("resources/XSLT/LaTeXML-html5.xsl".to_string()),
          Some("html") | Some("xhtml") =>
            Some("resources/XSLT/LaTeXML-all-xhtml.xsl".to_string()),
          Some("epub") | Some("epub3") =>
            Some("resources/XSLT/LaTeXML-epub3.xsl".to_string()),
          _ => None,
        }
      });
      let do_post = post_flag || pmml_flag || cmml_flag || effective_stylesheet.is_some() || format_flag.is_some();

      if do_post {
        // Post-process the XML
        let output = run_post_processing(
          &xml,
          pmml_flag || post_flag || format_flag.is_some(),
          cmml_flag,
          keep_xmath_flag,
          effective_stylesheet.as_deref(),
          target.as_deref(),
          nodefaultresources_flag,
          &css_flags,
          noinvisibletimes_flag,
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
  cmml: bool,
  keep_xmath: bool,
  stylesheet: Option<&str>,
  destination: Option<&str>,
  nodefaultresources: bool,
  css_files: &[String],
  noinvisibletimes: bool,
) -> String {
  use latexml_post::document::{PostDocument, PostDocumentOptions};
  use latexml_post::object_db::ObjectDB;
  use latexml_post::processor::Processor;

  let mut opts = PostDocumentOptions::default();
  if let Some(dest) = destination {
    opts.destination = Some(dest.to_string());
  }
  let doc = match PostDocument::new_from_string(xml, opts) {
    Ok(d) => d,
    Err(e) => {
      eprintln!("Post-processing: failed to parse XML: {}", e);
      return xml.to_string();
    }
  };

  // Phase 1: Scan — collect structural info into ObjectDB
  let db = ObjectDB::new();
  let mut scanner = latexml_post::scan::Scan::new(db);
  let scan_nodes = scanner.to_process(&doc);
  let doc = match scanner.process(doc, scan_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: Scan failed: {}", e);
      return xml.to_string();
    }
  };

  // Phase 2: CrossRef — resolve references using the scanned DB
  let db = scanner.db; // Transfer ownership of populated DB
  let mut crossref = latexml_post::crossref::CrossRef::new(
    db,
    latexml_post::crossref::UrlStyle::File,
    true, // number_sections
  );
  let xref_nodes = crossref.to_process(&doc);
  let doc = match crossref.process(doc, xref_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: CrossRef failed: {}", e);
      return xml.to_string();
    }
  };

  // Phase 3: MathML + XSLT chain
  let mut post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  if pmml {
    let mathml = latexml_post::mathml::MathML::new_presentation()
      .with_keep_xmath(keep_xmath)
      .with_invisible_times(!noinvisibletimes);
    processors.push(Box::new(mathml));
  }

  if cmml {
    let cmathml = latexml_post::mathml::MathML::new_content()
      .with_keep_xmath(keep_xmath)
      .with_invisible_times(!noinvisibletimes);
    processors.push(Box::new(cmathml));
  }

  if let Some(xsl_path) = stylesheet {
    let searchpaths = vec![
      "resources/XSLT".to_string(),
      ".".to_string(),
    ];
    // Pass --css files as XSLT parameter (Perl: $params{CSS} = '"file1|file2"')
    let mut xslt_params = std::collections::HashMap::new();
    if !css_files.is_empty() {
      xslt_params.insert("CSS".to_string(), format!("\"{}\"", css_files.join("|")));
    }
    match latexml_post::xslt::XSLT::new(
      xsl_path, xslt_params, nodefaultresources, None, searchpaths
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

/// Extract ALL occurrences of a `--flag=value` argument (for repeatable flags like --css, --path)
fn extract_flags(argv: &mut Vec<String>, prefix: &str) -> Vec<String> {
  let eq_prefix = format!("{}=", prefix);
  let mut values = Vec::new();
  while let Some(pos) = argv.iter().position(|a| a.starts_with(&eq_prefix)) {
    let val = argv[pos][eq_prefix.len()..].to_string();
    argv.remove(pos);
    values.push(val);
  }
  values
}
