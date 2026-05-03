use clap::Parser;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;

/// Per-process allocator: mimalloc avoids glibc's arena-mutex contention
/// which dominates multi-process workloads (seen as 3.4x slowdown at 16 workers).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

  /// For PDF graphics under N kilobytes, try `inkscape` first to preserve
  /// vector content; fall back to ImageMagick `convert` on failure/timeout.
  /// 0 disables (default). Suggested value: 200.
  /// See SYNC_STATUS.md for the file-size heuristic rationale
  /// (matplotlib/pgfplots vector PDFs are ~30 KB; raster-embedded PDFs
  /// are usually 500 KB+ and take >10s to vectorise).
  #[arg(
    long = "graphics-svg-threshold-kb",
    value_name = "N",
    default_value = "0"
  )]
  graphics_svg_threshold_kb: u32,

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
  /// Conversion timeout in seconds (default: 60). Use 0 to disable.
  #[arg(long, value_name = "SECONDS", default_value = "60")]
  timeout: u64,

  /// Maximum number of tokens to process before aborting (default: 100M).
  /// Protects against infinite loops in macro expansion.
  #[arg(long, value_name = "N")]
  token_limit: Option<usize>,

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

  /// Dump compiled schema model to stdout in `.model` plain-text format,
  /// then exit. Currently only the embedded `LaTeXML` schema is supported
  /// (matches Perl `LaTeXML::Common::Model::compileSchema` output).
  #[arg(long)]
  dump_model: bool,

  /// Write per-job telemetry as a single-line JSON record to this file.
  /// Falls back to env `LATEXML_TELEMETRY_OUT` if not set. Emitted only
  /// on the successful conversion path; codegen / dump-model exits skip it.
  /// See `docs/TELEMETRY.md`.
  #[arg(long, value_name = "PATH")]
  telemetry_out: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
  // Run all work on a worker thread with a 256 MB stack so deeply
  // nested math trees don't overflow the OS-default 8 MB main-thread
  // stack during finalize/post-processing. See cortex_worker.rs for
  // full rationale (sandbox 0711.4787 et al, #17).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(|| real_main().map_err(|e| e.to_string()))
    .expect("spawn worker thread")
    .join()
    .expect("worker thread panicked")
    .map_err(|s| s.into())
}

fn real_main() -> Result<(), Box<dyn Error>> {
  let wall_start = std::time::Instant::now();
  let cli = Cli::parse();

  // Initialize logger with verbosity level
  let verbosity: i32 = if cli.quiet {
    -1
  } else if cli.verbose {
    1
  } else {
    0
  };
  let log_level = match verbosity {
    v if v < 0 => log::LevelFilter::Warn,
    0 => log::LevelFilter::Info,
    _ => log::LevelFilter::Debug,
  };
  latexml_core::util::logger::init(log_level).ok();

  // Dump-model mode — load the embedded LaTeXML schema, serialise to
  // stdout in `.model` format, exit. Mirrors Perl
  // `LaTeXML::Common::Model::compileSchema` (Model.pm L121-136). Used
  // by tools/compileschema.sh stage 2 to regenerate `LaTeXML.model`
  // from the same source the runtime sees.
  if cli.dump_model {
    print!("{}", latexml::dump_compiled_latexml_model());
    process::exit(0);
  }

  // Codegen mode — handle early, no source file needed
  if let Some(dump_path) = cli.codegen {
    let output = cli.dest.unwrap_or_else(|| "latex_dump.rs".to_string());
    match latexml::ini_tex::codegen_from_dump(&dump_path, &output) {
      Ok(count) => {
        eprintln!("Codegen complete: {} entries → {}", count, output);
        process::exit(0);
      },
      Err(e) => {
        eprintln!("Codegen failed: {}", e);
        process::exit(1);
      },
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
      },
    }
  };
  let target = cli.dest.clone();

  // --whatsin=archive: extract archive to temp directory, find main .tex file
  let mut path_flags = cli.search_paths.clone();
  let _archive_tempdir; // hold tempdir alive for the duration of processing
  let is_archive_mode = cli.whatsin.as_deref() == Some("archive")
    || source.ends_with(".tar.gz")
    || source.ends_with(".tgz")
    || source.ends_with(".zip")
    || source.ends_with(".tar");
  let source = if is_archive_mode {
    let (tempdir, main_tex) = match unpack_archive(&source) {
      Ok(r) => r,
      Err(e) => {
        eprintln!("Failed to unpack archive '{}': {}", source, e);
        process::exit(1);
      },
    };
    let dir_str = tempdir.path().to_string_lossy().to_string();
    path_flags.push(dir_str);
    _archive_tempdir = Some(tempdir);
    main_tex
  } else {
    _archive_tempdir = None;
    source
  };

  // --whatsin=directory: auto-detect from trailing / or explicit flag
  let is_directory_mode = cli.whatsin.as_deref() == Some("directory") || source.ends_with('/');
  let source = if is_directory_mode {
    let dir_path = std::path::Path::new(&source);
    if let Ok(abs_source) = std::fs::canonicalize(dir_path) {
      path_flags.push(abs_source.to_string_lossy().to_string());
    } else {
      path_flags.push(source.clone());
    }
    // Find the main .tex file in the directory, matching Perl's behavior
    match find_main_tex(dir_path) {
      Ok(main_tex) => main_tex,
      Err(e) => {
        eprintln!("Failed to find main .tex file in '{}': {}", source, e);
        process::exit(1);
      },
    }
  } else {
    source
  };

  // Stash a copy of the resolved main-tex path for end-of-run telemetry,
  // since `source` itself is moved into `converter.convert(...)`.
  let telemetry_source = source.clone();

  // Prepare converter
  let preload = if cli.preload_files.is_empty() {
    None
  } else {
    Some(cli.preload_files.clone())
  };
  let search_paths = if path_flags.is_empty() {
    None
  } else {
    Some(path_flags)
  };

  let opts = Config {
    verbosity,
    format: OutputFormat::HTML5,
    whatsin: DataSize::Document,
    whatsout: DataSize::Document,
    preamble: cli.preamble.clone(),
    postamble: cli.postamble.clone(),
    mode: None,
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    preload,
    search_paths,
    include_comments: if cli.nocomments { Some(false) } else { None },
    nomathparse: if cli.nomathparse { Some(true) } else { None },
  };
  // CRITICAL: must be set BEFORE `prepare_session`. `tex.rs` /
  // `latex.rs`'s LoadFormat split (plain_bootstrap → plain_dump|base
  // → plain_constructs and the latex equivalent) reads
  // `LATEXML_INI_MODE` to decide whether to fully load the format
  // or stop after the bootstrap pool. If it's not set yet,
  // `prepare_session` pre-loads `plain_base` / `latex_base`, which
  // pollutes the snapshot taken later in `ini_tex::dump_format` and
  // silences the diff for everything raw plain.tex / latex.ltx defines
  // (the `\countdef\allocationnumber=21` → `Stored::Register{...}`
  // problem from 2026-04-26).
  if cli.init.is_some() {
    // SAFETY: setting the var before any thread is spawned. `prepare_session`
    // and `ini_tex::dump_format` both read it but neither mutates env.
    unsafe {
      std::env::set_var("LATEXML_INI_MODE", "1");
    }
  }

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
      latexml_core::common::store::Stored::Strings(std::rc::Rc::new([
        latexml_core::common::arena::pin("bbl"),
      ])),
      Some(latexml_core::state::Scope::Global),
    );
  }
  if cli.nonumbersections {
    latexml_core::state::assign_value(
      "no_number_sections",
      true,
      Some(latexml_core::state::Scope::Global),
    );
  }
  // Perl Core.pm L48: DOCUMENTID value
  if let Some(ref docid) = cli.documentid {
    latexml_core::state::assign_value(
      "DOCUMENTID",
      latexml_core::common::store::Stored::String(latexml_core::common::arena::pin(docid)),
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
      },
    }
  } else {
    // Normal mode: convert document
    //
    // Two-layer timeout: the cooperative stomach::check_timeout gives a graceful
    // Err(Fatal) when the digestion loop can poll it, and the Watchdog forcibly
    // aborts the process if the deadline is reached without cooperation (e.g. a
    // tight native loop in Marpa / libxml2 / libxslt). The Watchdog cancels
    // automatically on drop at end of main.
    let _watchdog = latexml_core::watchdog::Watchdog::new(cli.timeout);
    if cli.timeout > 0 {
      latexml_core::stomach::set_timeout(cli.timeout);
    }
    if let Some(limit) = cli.token_limit {
      latexml_core::gullet::set_token_limit(Some(limit));
    }

    let source_for_post = source.clone();
    let response = converter.convert(source);
    let _ = &source_for_post; // keep alive for post-processing
    if let Some(xml) = response.result {
      // Infer format from --dest extension if --format not specified (Perl Config.pm L408-441)
      let inferred_format: Option<String> = cli.format.clone().or_else(|| {
        target.as_ref().and_then(|dest| {
          Path::new(dest)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
              match ext.to_lowercase().as_str() {
                "html" | "htm" => "html5".to_string(), // Perl L435: html → html5
                "xhtml" => "xhtml".to_string(),
                "xml" => "xml".to_string(),
                "zip" => "html5".to_string(), // Perl L431: zip → html5
                "epub" | "mobi" => "epub".to_string(),
                other => other.to_string(),
              }
            })
        })
      });

      // Auto-select stylesheet from format (Perl Config.pm L543-551)
      let effective_stylesheet =
        cli
          .stylesheet
          .clone()
          .or_else(|| match inferred_format.as_deref() {
            Some("html5") => Some("resources/XSLT/LaTeXML-html5.xsl".to_string()),
            Some("html") | Some("xhtml") => {
              Some("resources/XSLT/LaTeXML-all-xhtml.xsl".to_string())
            },
            Some("epub") | Some("epub3") => Some("resources/XSLT/LaTeXML-epub3.xsl".to_string()),
            _ => None,
          });

      // Auto-enable post-processing when dest implies HTML (Perl Config.pm L448-455)
      let is_html_format = matches!(
        inferred_format.as_deref(),
        Some("html5") | Some("html") | Some("xhtml") | Some("epub") | Some("epub3")
      );
      let do_post = cli.post
        || cli.pmml
        || cli.cmml
        || effective_stylesheet.is_some()
        || is_html_format
        || cli.split
        || cli.splitat.is_some();

      // Build split XPath from --splitat
      let split_enabled =
        cli.split || cli.splitat.is_some() || cli.splitnaming.is_some() || cli.splitpath.is_some();
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
          pmml: cli.pmml || cli.post || is_html_format,
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
          graphics_svg_threshold_kb: cli.graphics_svg_threshold_kb,
        });
        let is_zip_output = target.as_ref().is_some_and(|t| t.ends_with(".zip"))
          || cli.whatsin.as_deref() == Some("archive");
        if is_zip_output {
          // whatsout=archive: pack output into ZIP
          if let Some(ref target_path) = target {
            let zip_dest = if target_path.ends_with(".zip") {
              target_path.clone()
            } else {
              format!(
                "{}.zip",
                target_path
                  .trim_end_matches(".html")
                  .trim_end_matches(".xml")
              )
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

  write_telemetry_record(cli.telemetry_out.as_deref(), &telemetry_source, wall_start, "ok", 0);
  process::exit(0);
}

/// Emit a single-line JSON telemetry record. No-op when neither
/// `--telemetry-out` nor `LATEXML_TELEMETRY_OUT` is set. Errors writing
/// the file are swallowed (the conversion already succeeded; logging
/// the failure on stderr would be noise for batch runs).
fn write_telemetry_record(
  cli_path: Option<&str>,
  source: &str,
  wall_start: std::time::Instant,
  category: &str,
  exit_code: i32,
) {
  use latexml_core::telemetry;
  let path = cli_path
    .map(|s| s.to_string())
    .or_else(|| std::env::var("LATEXML_TELEMETRY_OUT").ok());
  let Some(path) = path else { return };

  // paper_id ≈ source basename without extension; cortex_worker
  // overrides this when it knows the arxiv id. Keep the binary's
  // best-effort default for direct CLI users.
  let paper_id = std::path::Path::new(source)
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("")
    .to_string();
  telemetry::set_paper_id(&paper_id);
  telemetry::set_cmdline(&std::env::args().collect::<Vec<_>>().join(" "));
  if let Ok(host) = std::env::var("HOSTNAME").or_else(|_| {
    // Linux fallback: read /etc/hostname
    std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string())
  }) {
    telemetry::set_host(&host);
  }
  telemetry::set_wall_us(wall_start.elapsed().as_micros() as u64);
  telemetry::set_category(category);
  telemetry::set_exit_code(exit_code);
  telemetry::set_max_rss_kb(read_max_rss_kb());
  let (cu, cs) = read_child_rusage_us();
  telemetry::set_child_rusage_us(cu, cs);

  let record = telemetry::take();
  let line = record.to_json_line();
  if let Some(parent) = std::path::Path::new(&path).parent() {
    if !parent.as_os_str().is_empty() {
      let _ = std::fs::create_dir_all(parent);
    }
  }
  if let Ok(mut fh) = File::create(&path) {
    let _ = writeln!(fh, "{line}");
  }
}

/// Read peak resident-set size from `/proc/self/status` (`VmHWM`).
/// Returns 0 on non-Linux or read failure.
fn read_max_rss_kb() -> u64 {
  std::fs::read_to_string("/proc/self/status")
    .ok()
    .and_then(|content| {
      content
        .lines()
        .find(|l| l.starts_with("VmHWM:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|n| n.parse::<u64>().ok())
    })
    .unwrap_or(0)
}

/// Read accumulated child user/sys CPU time in microseconds via getrusage(2).
/// Returns (0, 0) on failure or non-unix.
#[cfg(unix)]
fn read_child_rusage_us() -> (u64, u64) {
  // SAFETY: getrusage(RUSAGE_CHILDREN, &ru) is async-signal-safe and
  // populates the struct unconditionally on success.
  unsafe {
    let mut ru: libc::rusage = std::mem::zeroed();
    if libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru) == 0 {
      let user = (ru.ru_utime.tv_sec as u64) * 1_000_000 + (ru.ru_utime.tv_usec as u64);
      let sys = (ru.ru_stime.tv_sec as u64) * 1_000_000 + (ru.ru_stime.tv_usec as u64);
      (user, sys)
    } else {
      (0, 0)
    }
  }
}

#[cfg(not(unix))]
fn read_child_rusage_us() -> (u64, u64) {
  (0, 0)
}

use latexml::post::PostOptions;

/// Delegate post-processing to the library API.
fn run_post_processing(xml: &str, opts: &PostOptions) -> String {
  // Per-phase telemetry now lives inside latexml::post::run_post_processing
  // (PostXmlParse, PostScan, Bibliography, Crossref, Graphics, Split, Xslt).
  latexml::post::run_post_processing(xml, opts)
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
  let all_units: Vec<&str> = std::iter::once(splitat)
    .chain(ancestors.iter().copied())
    .collect();
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
/// Faithful port of Perl's LaTeXML::Util::Pack::detect_source.
fn find_main_tex(dir: &Path) -> Result<String, Box<dyn Error>> {
  use once_cell::sync::Lazy;
  use regex::Regex;

  // Phase I.1: Check 00README.json (2025 arXiv format)
  // Format: { "sources": [{"filename": "main.tex", "usage": "toplevel"}, ...] }
  if let Some(filename) = parse_readme_json(dir) {
    let main_path = dir.join(&filename);
    if main_path.exists() {
      return Ok(main_path.to_string_lossy().to_string());
    }
  }

  // Phase I.1.2: Check 00README.XXX (legacy arXiv format)
  let readme_xxx = dir.join("00README.XXX");
  if readme_xxx.exists() {
    if let Ok(content) = std::fs::read_to_string(&readme_xxx) {
      for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "toplevelfile" {
          let main_path = dir.join(parts[0]);
          if main_path.exists() {
            return Ok(main_path.to_string_lossy().to_string());
          }
        }
      }
    }
  }

  // Phase I.2: Heuristic detection (ported from arXiv::FileGuess via Pack.pm)
  // Perl Pack.pm L25 TEX_EXT = qr/\.(?:[tT](:?[eE][xX]|[xX][tT])|ltx|LTX)$/
  // → .tex, .txt, .ltx (case-insensitive).
  fn collect_tex_files(dir: &Path, files: &mut Vec<PathBuf>, fallback: bool) {
    if let Ok(entries) = std::fs::read_dir(dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
          collect_tex_files(&path, files, fallback);
        } else if !fallback {
          if path.extension().is_some_and(|e| {
            let e = e.to_ascii_lowercase();
            e == "tex" || e == "txt" || e == "ltx"
          }) {
            files.push(path);
          }
        } else {
          // Perl Pack/Dir.pm L47 fallback: `!/\./ || /\.[^.]{4,}$/`
          //   → files with no extension, or with extension ≥4 chars.
          // Some arxiv sources ship bare names (e.g. "birkhoffproofrev1").
          let ext_opt = path.extension().and_then(|e| e.to_str());
          let keep = match ext_opt {
            None => true,
            Some(ext) => ext.len() >= 4,
          };
          if keep {
            files.push(path);
          }
        }
      }
    }
  }

  let mut tex_files: Vec<PathBuf> = Vec::new();
  collect_tex_files(dir, &mut tex_files, false);
  if tex_files.is_empty() {
    collect_tex_files(dir, &mut tex_files, true);
  }
  if tex_files.is_empty() {
    return Err("No .tex files found in directory".into());
  }

  // Regexes for content-based detection (Perl Pack.pm L116-166)
  static RE_AUTOIGNORE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%auto-ignore").unwrap());
  static RE_TEXINFO: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\input texinfo").unwrap());
  static RE_AUTOINCLUDE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%auto-include").unwrap());
  static RE_FORMAT_HINT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\r?%&(\S+)").unwrap());
  static RE_DOCCLASS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|\r)\s*\\document(?:style|class)").unwrap());
  static RE_MAYBE_TEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|\r)\s*\\(?:font|magnification|input|def|special|baselineskip|begin)").unwrap()
  });
  static RE_INPUT_INCLUDE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\(?:input|include)(?:\s+|\{)([^ \}]+)").unwrap());
  static RE_END_BYE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|\r)\s*\\(?:end|bye)(?:\s|$)").unwrap());
  static RE_END_BYE2: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\(?:end|bye)(?:\s|$)").unwrap());
  static RE_MAC_TEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\input *(?:harv|lanl)mac|\\input\s+phyzzx").unwrap());
  static RE_METAFONT: Lazy<Regex> = Lazy::new(|| Regex::new(r"beginchar\(").unwrap());
  static RE_BIBTEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(?:^|\r)@(?:book|article|inbook|unpublished)\{").unwrap());
  static RE_UUENCODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^begin \d{1,4}\s+\S+\r?$").unwrap());
  static RE_WITHDRAWN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"paper deliberately replaced by what little").unwrap());
  static RE_AMSTEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^amstex$").unwrap());

  // Score each file: likelihood 0-3 (Perl: Main_TeX_likelihood)
  let mut likelihood: std::collections::HashMap<PathBuf, f32> = std::collections::HashMap::new();
  let mut vetoed: Vec<PathBuf> = Vec::new();

  for tex_file in &tex_files {
    if !tex_file.exists() {
      continue;
    }
    // Use lossy read — TeX files may contain Latin-1 or other non-UTF8 bytes
    let Ok(raw) = std::fs::read(tex_file) else {
      continue;
    };
    let content = String::from_utf8_lossy(&raw);
    let mut maybe_tex = false;
    let mut maybe_tex_priority = false;
    let mut maybe_tex_priority2 = false;
    let mut determined = false;

    for (lineno, raw_line) in content.lines().enumerate() {
      let lineno1 = lineno + 1; // 1-based like Perl's $.
      // Perl L117-120: early-line checks (first 10-12 lines)
      if lineno1 <= 10
        && (RE_AUTOIGNORE.is_match(raw_line)
          || RE_TEXINFO.is_match(raw_line)
          || RE_AUTOINCLUDE.is_match(raw_line))
      {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if lineno1 <= 12 {
        if let Some(cap) = RE_FORMAT_HINT.captures(raw_line) {
          let fmt = &cap[1];
          if fmt == "latex209" || fmt == "biglatex" || fmt == "latex" || fmt == "LaTeX" {
            likelihood.insert(tex_file.clone(), 3.0);
          } else {
            likelihood.insert(tex_file.clone(), 1.0);
          }
          determined = true;
          break;
        }
      }
      // Perl L128: strip comments for subsequent checks
      let line = if let Some(pos) = raw_line.find('%') {
        &raw_line[..pos]
      } else {
        raw_line
      };

      if RE_DOCCLASS.is_match(line) {
        likelihood.insert(tex_file.clone(), 3.0);
        determined = true;
        break;
      }
      if RE_MAYBE_TEX.is_match(line) {
        maybe_tex = true;
      }
      // Perl L133-148: \input/\include → veto the included file
      if let Some(cap) = RE_INPUT_INCLUDE.captures(line) {
        maybe_tex = true;
        let mut vetoed_name = cap[1].to_string();
        if RE_AMSTEX.is_match(&vetoed_name) {
          likelihood.insert(tex_file.clone(), 2.0);
          determined = true;
          break;
        }
        if !vetoed_name.contains('.') {
          vetoed_name = vetoed_name.trim_end().to_string() + ".tex";
        }
        // Resolve relative to the file's directory
        let base_dir = tex_file.parent().unwrap_or(dir);
        vetoed.push(base_dir.join(&vetoed_name));
      }
      if RE_END_BYE.is_match(line) {
        maybe_tex_priority = true;
      }
      if RE_END_BYE2.is_match(line) {
        maybe_tex_priority2 = true;
      }
      if RE_MAC_TEX.is_match(line) {
        likelihood.insert(tex_file.clone(), 1.0);
        determined = true;
        break;
      }
      if RE_METAFONT.is_match(line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if RE_BIBTEX.is_match(raw_line) {
        // check raw_line (before comment stripping)
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if RE_UUENCODE.is_match(raw_line) {
        if maybe_tex_priority {
          likelihood.insert(tex_file.clone(), 2.0);
        } else if maybe_tex {
          likelihood.insert(tex_file.clone(), 1.0);
        } else {
          likelihood.insert(tex_file.clone(), 0.0);
        }
        determined = true;
        break;
      }
      if RE_WITHDRAWN.is_match(line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
    }
    // Perl L169-177: if not determined by any pattern
    if !determined {
      let score = if maybe_tex_priority {
        2.0
      } else if maybe_tex_priority2 {
        1.5
      } else if maybe_tex {
        1.0
      } else {
        0.0
      };
      likelihood.insert(tex_file.clone(), score);
    }
  }

  // Perl L181-182: remove vetoed files
  for v in &vetoed {
    likelihood.remove(v);
  }

  // Perl L184-185: filter to score > 0, sort by score descending
  let mut candidates: Vec<PathBuf> = likelihood
    .keys()
    .filter(|f| likelihood[*f] > 0.0)
    .cloned()
    .collect();
  candidates.sort_by(|a, b| likelihood[b].partial_cmp(&likelihood[a]).unwrap());

  if candidates.is_empty() {
    return Err("No viable .tex files found in directory".into());
  }

  // Perl L187-188: keep only max-scoring candidates
  let max_score = likelihood[&candidates[0]];
  candidates.retain(|f| (likelihood[f] - max_score).abs() < f32::EPSILON);

  // Perl L190-196: Heuristic 1 — prefer shallowest path (fewest '/' components)
  if candidates.len() > 1 {
    let min_depth = candidates
      .iter()
      .map(|f| f.strip_prefix(dir).unwrap_or(f).components().count())
      .min()
      .unwrap_or(0);
    candidates.retain(|f| f.strip_prefix(dir).unwrap_or(f).components().count() == min_depth);
  }

  // Perl L198-200: Heuristic 2 — prefer files with PDF-like \includegraphics
  if candidates.len() > 1 {
    let pdf_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        std::fs::read(f).ok().is_some_and(|raw| {
          let c = String::from_utf8_lossy(&raw);
          c.contains("\\includegraphics")
            && (c.contains(".pdf") || c.contains(".png") || c.contains(".jpg"))
        })
      })
      .cloned()
      .collect();
    if !pdf_candidates.is_empty() {
      candidates = pdf_candidates;
    }
  }

  // Perl L202-204: Heuristic 3 — prefer files with a matching .bbl file
  if candidates.len() > 1 {
    let bbl_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| f.with_extension("bbl").exists())
      .cloned()
      .collect();
    if !bbl_candidates.is_empty() {
      candidates = bbl_candidates;
    }
  }

  // Perl L208-210: Heuristic 4 — prefer common main file names
  if candidates.len() > 1 {
    let common: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        f.file_name().is_some_and(|n| {
          let n = n.to_str().unwrap_or("");
          n == "main.tex" || n == "ms.tex" || n == "paper.tex"
        })
      })
      .cloned()
      .collect();
    if !common.is_empty() {
      candidates = common;
    }
  }

  // Perl L212-213: Final tiebreaker — lexicographic order
  candidates.sort();

  Ok(candidates[0].to_string_lossy().to_string())
}

/// Parse 00README.json in `dir` and return the "filename" of the toplevel source.
/// Perl Pack.pm L68-80: looks for sources[] with usage=="toplevel".
/// Format: { "sources": [{"filename": "main.tex", "usage": "toplevel"}, ...] }
fn parse_readme_json(dir: &Path) -> Option<String> {
  let content = std::fs::read_to_string(dir.join("00README.json")).ok()?;
  // Minimal JSON parser: find objects in "sources" array with "usage":"toplevel"
  // and extract their "filename" value.
  // Scan for "usage" : "toplevel" and nearby "filename" : "value" pairs.
  let sources_start = content.find("\"sources\"")?;
  let rest = &content[sources_start..];
  let arr_start = rest.find('[')?;
  let arr_end = rest.find(']')?;
  let arr = &rest[arr_start + 1..arr_end];

  // Split by '}' to get individual objects, look for toplevel ones
  for obj_str in arr.split('}') {
    if !obj_str.contains("\"toplevel\"") {
      continue;
    }
    // Extract filename from this object
    if let Some(fn_pos) = obj_str.find("\"filename\"") {
      let after_key = &obj_str[fn_pos + 10..];
      let after_key = after_key.trim_start();
      let after_key = after_key.strip_prefix(':')?;
      let after_key = after_key.trim_start();
      let after_key = after_key.strip_prefix('"')?;
      let mut result = String::new();
      for ch in after_key.chars() {
        match ch {
          '"' => break,
          '\\' => continue, // skip escapes (simplified)
          c => result.push(c),
        }
      }
      if !result.is_empty() {
        return Some(result);
      }
    }
  }
  None
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
  let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

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
