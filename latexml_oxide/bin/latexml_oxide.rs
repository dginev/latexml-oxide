use clap::Parser;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::rc::Rc;

/// LaTeXML-oxide: convert TeX/LaTeX documents to XML/HTML/MathML
#[derive(Parser, Debug)]
#[command(name = "latexml_oxide", version, about)]
struct Cli {
  /// Source TeX file (overridden by --source)
  #[arg(value_name = "SOURCE")]
  source_positional: Option<String>,

  /// Destination output file
  #[arg(long)]
  dest: Option<String>,

  /// Source file (overrides positional argument)
  #[arg(long)]
  source: Option<String>,

  /// Output format: html5, html, xhtml, xml, epub
  #[arg(long)]
  format: Option<String>,

  /// XSLT stylesheet path
  #[arg(long)]
  stylesheet: Option<String>,

  // === Post-processing flags ===
  /// Enable post-processing
  #[arg(long)]
  post: bool,

  /// Generate Presentation MathML
  #[arg(long)]
  pmml: bool,

  /// Generate Content MathML
  #[arg(long)]
  cmml: bool,

  /// Keep XMath in output alongside MathML
  #[arg(long, alias = "xmath")]
  #[arg(name = "keepXMath")]
  keep_xmath: bool,

  /// Wrap MathML in semantics with TeX annotation
  #[arg(long)]
  mathtex: bool,

  /// Replace invisible times with zero-width space
  #[arg(long)]
  noinvisibletimes: bool,

  /// Suppress default CSS/JS resources
  #[arg(long)]
  nodefaultresources: bool,

  /// Omit XML comments from output
  #[arg(long)]
  nocomments: bool,

  /// Disable math parsing
  #[arg(long)]
  nomathparse: bool,

  /// Disable section numbering
  #[arg(long)]
  nonumbersections: bool,

  // === Repeatable flags ===
  /// CSS files to inject (repeatable)
  #[arg(long = "css", value_name = "URL")]
  css_files: Vec<String>,

  /// JavaScript files to inject (repeatable)
  #[arg(long = "javascript", value_name = "URL")]
  js_files: Vec<String>,

  /// Packages to preload (repeatable)
  #[arg(long = "preload", value_name = "FILE")]
  preload_files: Vec<String>,

  /// Additional search paths (repeatable)
  #[arg(long = "path", value_name = "DIR")]
  search_paths: Vec<String>,

  // === Value flags ===
  /// Conversion timeout in seconds
  #[arg(long, value_name = "SECONDS")]
  timeout: Option<u64>,

  /// Navigation TOC style (e.g. "context")
  #[arg(long, value_name = "STYLE")]
  navigationtoc: Option<String>,

  /// Write conversion log to file
  #[arg(long, value_name = "PATH")]
  log: Option<String>,

  /// Input type: "document" or "directory"
  #[arg(long, value_name = "TYPE")]
  whatsin: Option<String>,

  /// Preamble file
  #[arg(long, value_name = "FILE")]
  preamble: Option<String>,

  /// Postamble file
  #[arg(long, value_name = "FILE")]
  postamble: Option<String>,

  /// Input encoding (default: UTF-8)
  #[arg(long, value_name = "ENC")]
  inputencoding: Option<String>,

  // === Split options ===
  /// Enable document splitting
  #[arg(long)]
  split: bool,

  /// Section level to split at (default: section)
  #[arg(long, value_name = "LEVEL")]
  splitat: Option<String>,

  /// Naming strategy for split files: id, idrelative, label, labelrelative
  #[arg(long, value_name = "STRATEGY")]
  splitnaming: Option<String>,

  /// XPath expression for split points (overrides --splitat)
  #[arg(long, value_name = "XPATH")]
  splitpath: Option<String>,

  // === Verbosity ===
  /// Increase output verbosity
  #[arg(short = 'v', long)]
  verbose: bool,

  /// Suppress most output
  #[arg(short = 'q', long)]
  quiet: bool,

  /// Assign an ID to the document root element
  #[arg(long, value_name = "ID")]
  documentid: Option<String>,

  /// Site directory for relative path resolution
  #[arg(long, value_name = "DIR")]
  sitedirectory: Option<String>,

  /// Source directory for relative path resolution
  #[arg(long, value_name = "DIR")]
  sourcedirectory: Option<String>,

  /// Additional XSLT parameters (repeatable, key=value)
  #[arg(long = "xsltparameter", value_name = "KEY=VALUE")]
  xslt_parameters: Vec<String>,

  // === Dev/internal flags ===
  /// Init mode: process and dump format state
  #[arg(long, value_name = "FILE")]
  init: Option<String>,

  /// Codegen mode: generate Rust from dump file
  #[arg(long, value_name = "DUMP")]
  codegen: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::parse();

  // Initialize logger with verbosity level
  let verbosity: i32 = if cli.quiet { -1 } else if cli.verbose { 1 } else { 0 };
  let log_level = match verbosity {
    v if v < 0 => log::LevelFilter::Warn,
    0 => log::LevelFilter::Info,
    _ => log::LevelFilter::Debug,
  };
  latexml_core::util::logger::init(log_level).ok();

  // Codegen mode — handle early, no source file needed
  if let Some(dump_path) = cli.codegen {
    let output = cli.dest.unwrap_or_else(|| "latex_dump.rs".to_string());
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

  // Determine source: --source > --init > positional
  let source = if let Some(ref init) = cli.init {
    init.clone()
  } else if let Some(ref src) = cli.source {
    src.clone()
  } else {
    match cli.source_positional {
      Some(ref s) => s.clone(),
      None => {
        eprintln!("Error: no source file specified. Use: latexml_oxide [OPTIONS] <SOURCE>");
        process::exit(1);
      }
    }
  };
  let target = cli.dest.clone();

  // --whatsin=directory: auto-detect from trailing /
  let mut path_flags = cli.search_paths.clone();
  let is_directory_mode =
    cli.whatsin.as_deref() == Some("directory") || source.ends_with('/');
  if is_directory_mode {
    if let Ok(abs_source) = std::fs::canonicalize(&source) {
      path_flags.push(abs_source.to_string_lossy().to_string());
    } else {
      path_flags.push(source.clone());
    }
  }

  // Prepare converter
  let preload = if cli.preload_files.is_empty() { None } else { Some(cli.preload_files.clone()) };
  let search_paths = if path_flags.is_empty() { None } else { Some(path_flags) };

  let opts = Config {
    verbosity,
    format:                  OutputFormat::HTML5,
    whatsin:                 DataSize::Document,
    whatsout:                DataSize::Document,
    preamble:                cli.preamble.clone(),
    postamble:               cli.postamble.clone(),
    mode:                    None,
    bindings_dispatch:       Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    preload,
    search_paths,
    include_comments:        if cli.nocomments { Some(false) } else { None },
    nomathparse:             if cli.nomathparse { Some(true) } else { None },
  };
  let mut converter = Converter::from_config(opts.clone());
  if let Err(e) = converter.prepare_session(&opts) {
    eprintln!("Could not prepare converter session: {}", e);
    process::exit(1);
  }

  // Wire state-level options
  if cli.nonumbersections {
    latexml_core::state::assign_value(
      "no_number_sections", true,
      Some(latexml_core::state::Scope::Global),
    );
  }
  // Perl Core.pm L48: DOCUMENTID value
  if let Some(ref docid) = cli.documentid {
    latexml_core::state::assign_value(
      "DOCUMENTID", latexml_core::common::store::Stored::String(latexml_core::common::arena::pin(docid)),
      Some(latexml_core::state::Scope::Global),
    );
  }

  if cli.init.is_some() {
    // Init mode: process file and dump state
    match latexml::ini_tex::dump_format(&mut converter, &source, target.as_deref()) {
      Ok(count) => eprintln!("Format dump complete: {} entries written", count),
      Err(e) => {
        eprintln!("Format dump failed: {}", e);
        process::exit(1);
      }
    }
  } else {
    // Normal mode: convert document
    if let Some(secs) = cli.timeout {
      latexml_core::stomach::set_timeout(secs);
    }

    let response = converter.convert(source);
    if let Some(xml) = response.result {
      // Auto-select stylesheet from --format
      let effective_stylesheet = cli.stylesheet.clone().or_else(|| {
        match cli.format.as_deref() {
          Some("html5") => Some("resources/XSLT/LaTeXML-html5.xsl".to_string()),
          Some("html") | Some("xhtml") => Some("resources/XSLT/LaTeXML-all-xhtml.xsl".to_string()),
          Some("epub") | Some("epub3") => Some("resources/XSLT/LaTeXML-epub3.xsl".to_string()),
          _ => None,
        }
      });

      let do_post = cli.post || cli.pmml || cli.cmml
        || effective_stylesheet.is_some() || cli.format.is_some()
        || cli.split || cli.splitat.is_some();

      // Build split XPath from --splitat
      let split_enabled = cli.split || cli.splitat.is_some()
        || cli.splitnaming.is_some() || cli.splitpath.is_some();
      let split_xpath = if split_enabled {
        cli.splitpath.clone().or_else(|| {
          let splitat = cli.splitat.as_deref().unwrap_or("section");
          Some(make_splitpaths(splitat))
        })
      } else {
        None
      };

      if do_post {
        let output = run_post_processing(&xml, &PostOptions {
          pmml: cli.pmml || cli.post || cli.format.is_some(),
          cmml: cli.cmml,
          keep_xmath: cli.keep_xmath,
          stylesheet: effective_stylesheet.as_deref(),
          destination: target.as_deref(),
          nodefaultresources: cli.nodefaultresources,
          css_files: &cli.css_files,
          js_files: &cli.js_files,
          noinvisibletimes: cli.noinvisibletimes,
          mathtex: cli.mathtex,
          navigationtoc: cli.navigationtoc.as_deref(),
          split: split_enabled,
          split_xpath,
          split_naming: cli.splitnaming.as_deref(),
          xslt_parameters: &cli.xslt_parameters,
        });
        if let Some(target_path) = target {
          let mut out_fh = File::create(target_path)?;
          write!(out_fh, "{output}")?;
        } else {
          print!("{output}");
        }
      } else {
        if let Some(target_path) = target {
          let mut out_fh = File::create(target_path)?;
          write!(out_fh, "{xml}")?;
        } else {
          print!("{xml}");
        }
      }
    }

    // --log: write conversion log to file
    if let Some(ref log_path) = cli.log {
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
  xslt_parameters: &'a [String],
}

/// Run the post-processing pipeline on XML output.
fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  let PostOptions { pmml, cmml, keep_xmath, stylesheet, destination,
    nodefaultresources, css_files, js_files, noinvisibletimes, mathtex,
    navigationtoc, split, ref split_xpath, split_naming, xslt_parameters } = *opts;
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

  // Phase 1: Scan
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

  // Phase 2: CrossRef
  let db = scanner.db;
  let mut crossref = latexml_post::crossref::CrossRef::new(
    db,
    latexml_post::crossref::UrlStyle::File,
    true,
  );
  let xref_nodes = crossref.to_process(&doc);
  let doc = match crossref.process(doc, xref_nodes) {
    Ok(mut docs) => docs.remove(0),
    Err(e) => {
      eprintln!("Post-processing: CrossRef failed: {}", e);
      return xml.to_string();
    }
  };

  // Phase 2.5: Split
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

  // Phase 3: MathML + XSLT
  let mut post = latexml_post::Post::new();
  let mut processors: Vec<Box<dyn Processor>> = Vec::new();

  if pmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_presentation()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes)
        .with_mathtex(mathtex),
    ));
  }
  if cmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_content()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes),
    ));
  }
  if let Some(xsl_path) = stylesheet {
    let searchpaths = vec!["resources/XSLT".to_string(), ".".to_string()];
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
    // --xsltparameter key=value pairs
    for param in xslt_parameters {
      if let Some((key, value)) = param.split_once('=') {
        xslt_params.insert(key.to_string(), format!("\"{}\"", value));
      }
    }
    match latexml_post::xslt::XSLT::new(
      xsl_path, xslt_params, nodefaultresources, None, searchpaths,
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

/// Build the XPath expression for splitting at a given level.
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
  let all_units: Vec<&str> = std::iter::once(splitat).chain(ancestors.iter().copied()).collect();
  for unit in &all_units {
    paths.push(format!("//ltx:{}", unit));
    for b in &back {
      let mut conditions = vec![format!("preceding-sibling::ltx:{}", unit)];
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
