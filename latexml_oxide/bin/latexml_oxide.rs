use clap::Parser;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
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
  #[arg(long, alias = "destination")]
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
  #[arg(long, alias = "presentationmathml")]
  pmml: bool,

  /// Generate Content MathML
  #[arg(long, alias = "contentmathml")]
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

  /// Use .bbl file instead of running BibTeX (for arXiv-like builds)
  #[arg(long)]
  nobibtex: bool,

  /// Disable math parsing
  #[arg(long, alias = "noparse")]
  nomathparse: bool,

  /// Disable section numbering
  #[arg(long, alias = "nosectionnumbers")]
  nonumbersections: bool,

  /// Output type (currently only "document" supported; "archive" auto-detected from --dest)
  #[arg(long, value_name = "TYPE")]
  whatsout: Option<String>,

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

  // --whatsin=archive: extract archive to temp directory, find main .tex file
  let mut path_flags = cli.search_paths.clone();
  let _archive_tempdir; // hold tempdir alive for the duration of processing
  let is_archive_mode = cli.whatsin.as_deref() == Some("archive")
    || source.ends_with(".tar.gz") || source.ends_with(".tgz")
    || source.ends_with(".zip") || source.ends_with(".tar");
  let source = if is_archive_mode {
    let (tempdir, main_tex) = match unpack_archive(&source) {
      Ok(r) => r,
      Err(e) => {
        eprintln!("Failed to unpack archive '{}': {}", source, e);
        process::exit(1);
      }
    };
    let dir_str = tempdir.path().to_string_lossy().to_string();
    path_flags.push(dir_str.clone());
    _archive_tempdir = Some(tempdir);
    main_tex
  } else {
    _archive_tempdir = None;
    source
  };

  // --whatsin=directory: auto-detect from trailing /
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
  if cli.nobibtex {
    // Set BIB_CONFIG to ['bbl'] — skip BibTeX, use pre-existing .bbl file
    latexml_core::state::assign_value(
      "BIB_CONFIG",
      latexml_core::common::store::Stored::Strings(std::rc::Rc::new([latexml_core::common::arena::pin("bbl")])),
      Some(latexml_core::state::Scope::Global),
    );
  }
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

    let source_for_post = source.clone();
    let response = converter.convert(source);
    let _ = &source_for_post; // keep alive for post-processing
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
        let source_dir = Path::new(&source_for_post)
          .parent()
          .map(|p| p.to_string_lossy().to_string())
          .unwrap_or_else(|| ".".to_string());
        let output = run_post_processing(&xml, &PostOptions {
          pmml: cli.pmml || cli.post || cli.format.is_some(),
          cmml: cli.cmml,
          keep_xmath: cli.keep_xmath,
          stylesheet: effective_stylesheet.as_deref(),
          destination: target.as_deref(),
          source_directory: Some(&source_dir),
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
        let is_zip_output = target.as_ref().is_some_and(|t| t.ends_with(".zip"))
          || cli.whatsin.as_deref() == Some("archive");
        if is_zip_output {
          // whatsout=archive: pack output into ZIP
          if let Some(ref target_path) = target {
            let zip_dest = if target_path.ends_with(".zip") {
              target_path.clone()
            } else {
              format!("{}.zip", target_path.trim_end_matches(".html").trim_end_matches(".xml"))
            };
            ensure_parent_dir(&zip_dest);
            pack_output_zip(&zip_dest, &output, &response.log, &response.status)?;
          } else {
            print!("{output}");
          }
        } else if let Some(ref target_path) = target {
          ensure_parent_dir(target_path);
          let mut out_fh = File::create(target_path)?;
          write!(out_fh, "{output}")?;
        } else {
          print!("{output}");
        }
      } else {
        if let Some(ref target_path) = target {
          ensure_parent_dir(target_path);
          let mut out_fh = File::create(target_path)?;
          write!(out_fh, "{xml}")?;
        } else {
          print!("{xml}");
        }
      }
    }

    // --log: write conversion log to file (skip if already packed into ZIP)
    let is_zip_output = target.as_ref().is_some_and(|t| t.ends_with(".zip"))
      || cli.whatsin.as_deref() == Some("archive");
    if let Some(ref log_path) = cli.log {
      if !is_zip_output {
        ensure_parent_dir(log_path);
        if let Ok(mut log_fh) = File::create(log_path) {
          let _ = write!(log_fh, "{}", response.log);
          eprintln!("Log written to {}", log_path);
        }
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
  source_directory: Option<&'a str>,
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
    source_directory, nodefaultresources, css_files, js_files, noinvisibletimes,
    mathtex, navigationtoc, split, ref split_xpath, split_naming, xslt_parameters } = *opts;
  use latexml_post::document::{PostDocument, PostDocumentOptions};
  use latexml_post::object_db::ObjectDB;
  use latexml_post::processor::Processor;

  let mut opts = PostDocumentOptions::default();
  if let Some(dest) = destination {
    opts.destination = Some(dest.to_string());
  }
  if let Some(src_dir) = source_directory {
    opts.source_directory = Some(src_dir.to_string());
    // Also add source dir to search paths for file resolution
    let mut sp = opts.searchpaths.take().unwrap_or_default();
    sp.push(src_dir.to_string());
    opts.searchpaths = Some(sp);
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

  // Phase 2.5: Graphics
  let mut graphics_proc = latexml_post::graphics::Graphics::new(None, true);
  let graphics_nodes = graphics_proc.to_process(&doc);
  let doc = if !graphics_nodes.is_empty() {
    match graphics_proc.process(doc, graphics_nodes) {
      Ok(mut docs) => docs.remove(0),
      Err(e) => {
        eprintln!("Post-processing: Graphics failed: {}", e);
        return xml.to_string();
      }
    }
  } else { doc };

  // Phase 2.75: Split
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

  // ar5iv.sty.ltxml: adds intent=":literal" on all <math> elements
  let intent_literal = xml.contains("package=\"ar5iv");

  if pmml {
    processors.push(Box::new(
      latexml_post::mathml::MathML::new_presentation()
        .with_keep_xmath(keep_xmath)
        .with_invisible_times(!noinvisibletimes)
        .with_mathtex(mathtex)
        .with_intent_literal(intent_literal),
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
    // Resolve XSLT searchpaths: stylesheet path is "resources/XSLT/LaTeXML-html5.xsl"
    // so searchpaths should be directories where that relative path can be found.
    // Binary is at target/release/latexml_oxide, project root is 3 levels up.
    let mut searchpaths = vec![".".to_string()];
    if let Ok(exe) = std::env::current_exe() {
      if let Some(project_root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
        searchpaths.insert(0, project_root.display().to_string());
      }
    }
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
    Ok(results) => {
      let output = results[0].to_xml_string();
      // Fix self-closing non-void HTML elements for HTML5 output.
      // libxml2's XML serializer produces <span/> for empty spans, but HTML5
      // parsers treat <span/> as an opening tag, causing unclosed elements.
      if stylesheet.map_or(false, |s| s.contains("html")) {
        // Fix 1: Expand self-closing non-void elements: <span/> → <span></span>
        let re = regex::Regex::new(
          r"<(span|div|p|a|td|th|tr|section|article|figure|figcaption|pre|code|em|strong|b|i|u|sub|sup|small|cite)(\s[^>]*)?/>"
        ).unwrap();
        let output = re.replace_all(&output, "<$1$2></$1>").to_string();
        // Fix 2: Remove closing tags for void elements: </br>, </img>, </hr> etc.
        // These are invalid in HTML5 and break nesting when present.
        let void_close_re = regex::Regex::new(
          r"</(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)>"
        ).unwrap();
        let output = void_close_re.replace_all(&output, "").to_string();
        // Fix 3: Normalize self-closing void elements: <br ... /> → <br ...>
        let void_selfclose_re = regex::Regex::new(
          r"<(br|img|hr|input|meta|link|col|area|base|source|track|wbr|embed|param)(\s[^>]*?)\s*/>"
        ).unwrap();
        void_selfclose_re.replace_all(&output, "<$1$2>").to_string()
      } else {
        output
      }
    },
    Err(e) => {
      eprintln!("Post-processing failed: {}", e);
      xml.to_string()
    }
  }
}

/// Build the XPath expression for splitting at a given level.
/// Ensure the parent directory of a file path exists, creating it recursively if needed.
fn ensure_parent_dir(path: &str) {
  if let Some(parent) = Path::new(path).parent() {
    if !parent.as_os_str().is_empty() {
      let _ = std::fs::create_dir_all(parent);
    }
  }
}

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

/// Unpack a ZIP (primary) or tar.gz archive into a temp directory.
/// Returns (TempDir, main_tex_path).
///
/// Port of Perl LaTeXML::Util::Pack::unpack_source.
fn unpack_archive(archive_path: &str) -> Result<(tempfile::TempDir, String), Box<dyn Error>> {
  let tempdir = tempfile::tempdir()?;
  let dest = tempdir.path();

  if archive_path.ends_with(".zip") {
    // Primary format: ZIP
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
      let mut entry = archive.by_index(i)?;
      let outpath = dest.join(entry.mangled_name());
      if entry.is_dir() {
        std::fs::create_dir_all(&outpath)?;
      } else {
        if let Some(parent) = outpath.parent() {
          std::fs::create_dir_all(parent)?;
        }
        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut entry, &mut outfile)?;
      }
    }
  } else if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
    let file = File::open(archive_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(dest)?;
  } else if archive_path.ends_with(".tar") {
    let file = File::open(archive_path)?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(dest)?;
  } else {
    return Err(format!("Unsupported archive format: {}", archive_path).into());
  }

  // Find main .tex file (Perl: LaTeXML::Util::Pack looks for the largest .tex file
  // or one containing \documentclass)
  let main_tex = find_main_tex(dest)?;
  Ok((tempdir, main_tex))
}

/// Find the main .tex file in an unpacked directory.
/// Strategy: look for files containing \documentclass, prefer the largest one.
fn find_main_tex(dir: &Path) -> Result<String, Box<dyn Error>> {
  let mut candidates: Vec<(PathBuf, u64)> = Vec::new();

  fn collect_tex_files(dir: &Path, candidates: &mut Vec<(PathBuf, u64)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          collect_tex_files(&path, candidates);
        } else if path.extension().is_some_and(|e| e == "tex") {
          let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
          candidates.push((path, size));
        }
      }
    }
  }

  collect_tex_files(dir, &mut candidates);

  if candidates.is_empty() {
    return Err("No .tex files found in archive".into());
  }

  // First pass: find files with \documentclass
  let mut doc_class_files: Vec<(PathBuf, u64)> = Vec::new();
  for (path, size) in &candidates {
    if let Ok(content) = std::fs::read_to_string(path) {
      if content.contains("\\documentclass") || content.contains("\\documentstyle") {
        doc_class_files.push((path.clone(), *size));
      }
    }
  }

  // Prefer file with \documentclass; if multiple, pick the largest
  let main = if !doc_class_files.is_empty() {
    doc_class_files.sort_by(|a, b| b.1.cmp(&a.1));
    doc_class_files[0].0.clone()
  } else {
    // No \documentclass found — pick the largest .tex file
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates[0].0.clone()
  };

  Ok(main.to_string_lossy().to_string())
}

/// Pack the conversion output into a ZIP archive (whatsout=archive).
/// Includes the HTML/XML output, log, and status.
fn pack_output_zip(
  zip_path: &str,
  output: &str,
  log: &str,
  status: &str,
) -> Result<(), Box<dyn Error>> {
  use zip::write::SimpleFileOptions;
  let file = File::create(zip_path)?;
  let mut zip = zip::ZipWriter::new(file);
  let options = SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated);

  // Derive output filename from zip name: paper.zip → paper.html
  let stem = Path::new(zip_path)
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("document");

  // Write the main output file
  let output_name = format!("{}.html", stem);
  zip.start_file(&output_name, options)?;
  zip.write_all(output.as_bytes())?;

  // Write the log
  if !log.is_empty() {
    let log_name = format!("{}.log", stem);
    zip.start_file(&log_name, options)?;
    zip.write_all(log.as_bytes())?;
  }

  // Write status
  zip.start_file("status", options)?;
  zip.write_all(status.as_bytes())?;

  zip.finish()?;
  eprintln!("Output written to {}", zip_path);
  Ok(())
}
