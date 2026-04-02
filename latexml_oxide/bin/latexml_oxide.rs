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
  let mathtex_flag = argv.iter().any(|a| a == "--mathtex");
  // Repeatable flags
  let css_flags = extract_flags(&mut argv, "--css");
  let js_flags = extract_flags(&mut argv, "--javascript");
  let preload_flags = extract_flags(&mut argv, "--preload");
  let mut path_flags = extract_flags(&mut argv, "--path");
  // Value flags
  let timeout_flag = extract_flag(&mut argv, "--timeout");
  let navigationtoc_flag = extract_flag(&mut argv, "--navigationtoc");
  let source_flag = extract_flag(&mut argv, "--source");
  let log_flag = extract_flag(&mut argv, "--log");
  let whatsin_flag = extract_flag(&mut argv, "--whatsin");
  // Split options
  let split_flag = argv.iter().any(|a| a == "--split");
  let splitat_flag = extract_flag(&mut argv, "--splitat");
  let splitnaming_flag = extract_flag(&mut argv, "--splitnaming");
  let splitpath_flag = extract_flag(&mut argv, "--splitpath");
  // Verbosity: --verbose / --quiet (Perl: -v increments, --quiet sets to -1)
  let verbose_flag = argv.iter().any(|a| a == "--verbose" || a == "-v");
  let quiet_flag = argv.iter().any(|a| a == "--quiet" || a == "-q");
  // Preamble/postamble
  let preamble_flag = extract_flag(&mut argv, "--preamble");
  let postamble_flag = extract_flag(&mut argv, "--postamble");
  // Input encoding
  let inputencoding_flag = extract_flag(&mut argv, "--inputencoding");
  // Number sections control
  let nonumbersections_flag = argv.iter().any(|a| a == "--nonumbersections");
  // Remove boolean flags
  argv.retain(|a| !["--post", "--pmml", "--cmml", "--keepXMath", "--xmath",
    "--noscan", "--nocrossref", "--nodefaultresources", "--nocomments",
    "--nomathparse", "--noinvisibletimes", "--mathtex", "--split",
    "--verbose", "-v", "--quiet", "-q", "--nonumbersections"].contains(&a.as_str()));

  // Initialize logger with verbosity level (Perl: -v increments, --quiet sets -1)
  let verbosity: i32 = if quiet_flag { -1 } else if verbose_flag { 1 } else { 0 };
  let log_level = match verbosity {
    v if v < 0 => log::LevelFilter::Warn,
    0 => log::LevelFilter::Info,
    _ => log::LevelFilter::Debug,
  };
  latexml_core::util::logger::init(log_level).ok();

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

  // Determine source: --source flag overrides positional arg (Perl Config.pm L212)
  let source = if let Some(ref init) = init_file {
    init.clone()
  } else if let Some(ref src) = source_flag {
    src.clone()
  } else {
    match argv.first() {
      Some(s) => s.clone(),
      None => {
        eprintln!("Usage: latexml_oxide [options] <source>");
        process::exit(1);
      }
    }
  };
  let target = dest_flag.or_else(|| argv.get(1).cloned());

  // --whatsin=directory: auto-detect from trailing / or explicit flag (Perl Config.pm L220-225)
  let is_directory_mode = whatsin_flag.as_deref() == Some("directory")
    || source.ends_with('/');
  if is_directory_mode {
    if let Ok(abs_source) = std::fs::canonicalize(&source) {
      path_flags.push(abs_source.to_string_lossy().to_string());
    } else {
      path_flags.push(source.clone());
    }
  }

  // Prepare converter
  let opts = Config {
    verbosity,
    format:                  OutputFormat::HTML5,
    whatsin:                 DataSize::Document,
    whatsout:                DataSize::Document,
    preamble:                preamble_flag,
    postamble:               postamble_flag,
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

  // Wire --nonumbersections into state (Perl Config.pm L478)
  if nonumbersections_flag {
    latexml_core::state::assign_value("no_number_sections", true, Some(latexml_core::state::Scope::Global));
  }
  // Wire --inputencoding (Perl Config.pm: sets inputencoding to load fontenc)
  if let Some(ref _enc) = inputencoding_flag {
    // TODO: trigger \usepackage[enc]{inputenc} — for now, UTF-8 is assumed
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
    // Set conversion timeout if specified
    if let Some(ref timeout_str) = timeout_flag {
      if let Ok(secs) = timeout_str.parse::<u64>() {
        latexml_core::stomach::set_timeout(secs);
      }
    }
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
      let do_post = post_flag || pmml_flag || cmml_flag || effective_stylesheet.is_some()
        || format_flag.is_some() || split_flag || splitat_flag.is_some();

      // Build split XPath from --splitat (Perl Config.pm make_splitpaths)
      let split_enabled = split_flag || splitat_flag.is_some()
        || splitnaming_flag.is_some() || splitpath_flag.is_some();
      let split_xpath = if split_enabled {
        splitpath_flag.or_else(|| {
          let splitat = splitat_flag.as_deref().unwrap_or("section");
          Some(make_splitpaths(splitat))
        })
      } else {
        None
      };

      if do_post {
        // Post-process the XML
        let output = run_post_processing(&xml, &PostOptions {
          pmml: pmml_flag || post_flag || format_flag.is_some(),
          cmml: cmml_flag,
          keep_xmath: keep_xmath_flag,
          stylesheet: effective_stylesheet.as_deref(),
          destination: target.as_deref(),
          nodefaultresources: nodefaultresources_flag,
          css_files: &css_flags,
          js_files: &js_flags,
          noinvisibletimes: noinvisibletimes_flag,
          mathtex: mathtex_flag,
          navigationtoc: navigationtoc_flag.as_deref(),
          split: split_enabled,
          split_xpath,
          split_naming: splitnaming_flag.as_deref(),
        });
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
    // --log: write conversion log to file (Perl: UseLog/NoteLog)
    if let Some(ref log_path) = log_flag {
      if let Ok(mut log_fh) = File::create(log_path) {
        let _ = write!(log_fh, "{}", response.log);
        eprintln!("Log written to {}", log_path);
      }
    }
  }

  process::exit(0);
}

struct PostOptions<'a> {
  pmml: bool,
  cmml: bool,
  keep_xmath: bool,
  stylesheet: Option<&'a str>,
  destination: Option<&'a str>,
  nodefaultresources: bool,
  css_files: &'a [String],
  js_files: &'a [String],
  noinvisibletimes: bool,
  mathtex: bool,
  navigationtoc: Option<&'a str>,
  split: bool,
  split_xpath: Option<String>,
  split_naming: Option<&'a str>,
}

/// Run the post-processing pipeline on XML output.
fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  let PostOptions { pmml, cmml, keep_xmath, stylesheet, destination,
    nodefaultresources, css_files, js_files, noinvisibletimes, mathtex,
    navigationtoc, split, ref split_xpath, split_naming } = *opts;
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

  // Phase 2.5: Split — split document into multiple pages if requested
  let doc = if split {
    if let Some(ref xpath) = split_xpath {
      let naming = match split_naming {
        Some("id") | None => latexml_post::split::SplitNaming::Id,
        Some("idrelative") => latexml_post::split::SplitNaming::IdRelative,
        Some("label") => latexml_post::split::SplitNaming::Label,
        Some("labelrelative") => latexml_post::split::SplitNaming::LabelRelative,
        Some(other) => {
          eprintln!("Unknown splitnaming '{}', using 'id'", other);
          latexml_post::split::SplitNaming::Id
        }
      };
      let mut splitter = latexml_post::split::Split::new(xpath, naming, false);
      let split_nodes = splitter.to_process(&doc);
      match splitter.process(doc, split_nodes) {
        Ok(mut docs) => {
          // For now, we only output the first (root) document
          // Multi-file output would need the Writer processor
          if docs.len() > 1 {
            eprintln!("Split into {} documents", docs.len());
          }
          docs.remove(0)
        }
        Err(e) => {
          eprintln!("Post-processing: Split failed: {}", e);
          return xml.to_string();
        }
      }
    } else { doc }
  } else { doc };

  // Phase 3: MathML + XSLT chain
  let mut post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  if pmml {
    let mathml = latexml_post::mathml::MathML::new_presentation()
      .with_keep_xmath(keep_xmath)
      .with_invisible_times(!noinvisibletimes)
      .with_mathtex(mathtex);
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
    // Pass --css/--javascript/--navigationtoc as XSLT parameters
    let mut xslt_params = std::collections::HashMap::new();
    if !css_files.is_empty() {
      xslt_params.insert("CSS".to_string(), format!("\"{}\"", css_files.join("|")));
    }
    if !js_files.is_empty() {
      xslt_params.insert("JAVASCRIPT".to_string(), format!("\"{}\"", js_files.join("|")));
    }
    if let Some(navtoc) = navigationtoc {
      xslt_params.insert("NAVIGATIONTOC".to_string(), format!("\"{}\"", navtoc));
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

/// Build the XPath expression for splitting at a given level.
///
/// Port of Perl `Config.pm::make_splitpaths`.
fn make_splitpaths(splitat: &str) -> String {
  let ancestors: &[&str] = match splitat {
    "part" => &[],
    "chapter" => &["part"],
    "section" => &["part", "chapter"],
    "subsection" => &["part", "chapter", "section"],
    "subsubsection" => &["part", "chapter", "section", "subsection"],
    _ => &["part", "chapter"],
  };
  let back = ["bibliography", "appendix", "index"];
  let mut paths = Vec::new();
  // Add the target level and all ancestor levels
  let all_units: Vec<&str> = std::iter::once(splitat).chain(ancestors.iter().copied()).collect();
  for unit in &all_units {
    paths.push(format!("//ltx:{}", unit));
    for b in &back {
      let mut conditions = vec![format!("preceding-sibling::ltx:{}", unit)];
      // Add parent conditions from this unit's ancestors
      let unit_ancestors: &[&str] = match *unit {
        "part" => &[],
        "chapter" => &["part"],
        "section" => &["part", "chapter"],
        "subsection" => &["part", "chapter", "section"],
        "subsubsection" => &["part", "chapter", "section", "subsection"],
        _ => &[],
      };
      for anc in unit_ancestors {
        conditions.push(format!("parent::ltx:{}", anc));
      }
      paths.push(format!("//ltx:{}[{}]", b, conditions.join(" or ")));
    }
  }
  paths.join(" | ")
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
