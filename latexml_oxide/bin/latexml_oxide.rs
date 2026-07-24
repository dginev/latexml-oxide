#![feature(alloc_error_hook)]

use std::{
  alloc::{Layout, set_alloc_error_hook},
  error::Error,
  fs::File,
  io::prelude::*,
  path::Path,
  process,
  rc::Rc,
};

use clap::Parser;
use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, DigestionMode, OutputFormat};

/// Per-process allocator: mimalloc avoids glibc's arena-mutex contention
/// which dominates multi-process workloads (seen as 3.4x slowdown at 16 workers).
#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Heap-profiling allocator (`--features dhat-heap`): replaces mimalloc so dhat
/// can attribute every allocation to its call site. Diagnostic only.
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static GLOBAL: dhat::Alloc = dhat::Alloc;

/// LaTeXML-oxide: convert TeX/LaTeX documents to XML/HTML/MathML
#[derive(Parser, Debug)]
#[command(name = "latexml_oxide", version, about)]
struct Cli {
  /// The TeX/LaTeX source file to convert (overridden by --source). A `.bib`,
  /// a `.zip`/`.tar.gz` archive, or a directory is auto-detected.
  #[arg(value_name = "SOURCE")]
  source_positional: Option<String>,

  /// Output file (default: stdout). The extension can imply --format (e.g.
  /// .html → html5, .xml → xml, .zip → archive).
  #[arg(long, alias = "destination")]
  dest: Option<String>,

  /// Source file, overriding the positional SOURCE argument.
  #[arg(long)]
  source: Option<String>,

  /// Output format: html5, html, xhtml, xml, epub. Inferred from the --dest
  /// extension when omitted; falls back to xml.
  #[arg(long)]
  format: Option<String>,

  /// Shortcut for --format=xml: emit the raw LaTeXML XML with no HTML
  /// post-processing.
  #[arg(long)]
  xml: bool,

  /// Custom XSLT stylesheet path (overrides the format's built-in default).
  #[arg(long)]
  stylesheet: Option<String>,

  // === Post-processing flags ===
  /// Enable HTML/MathML post-processing (auto-enabled for HTML/ePub formats).
  #[arg(long)]
  post: bool,

  /// Skip post-processing, emitting the raw LaTeXML XML even for an
  /// HTML-implying destination.
  #[arg(long)]
  nopost: bool,

  /// Generate Presentation MathML (on by default for HTML formats).
  #[arg(long, alias = "presentationmathml")]
  pmml: bool,

  /// Suppress Presentation MathML even when the format would enable it.
  #[arg(long, alias = "nopresentationmathml")]
  nopmml: bool,

  /// Generate Content MathML.
  #[arg(long, alias = "contentmathml")]
  cmml: bool,

  /// Suppress Content MathML.
  #[arg(long, alias = "nocontentmathml")]
  nocmml: bool,

  /// Keep the intermediate XMath in the output alongside MathML.
  #[arg(long, alias = "xmath")]
  #[arg(name = "keepXMath")]
  keep_xmath: bool,

  /// Drop the XMath representation from the output.
  #[arg(long, alias = "nokeepXMath")]
  noxmath: bool,

  /// Wrap MathML in a `<semantics>` element with the TeX source as annotation.
  #[arg(long)]
  mathtex: bool,

  /// Suppress the TeX-source annotation on MathML.
  #[arg(long)]
  nomathtex: bool,

  /// Replace invisible-times operators (U+2062) with a zero-width space.
  #[arg(long)]
  noinvisibletimes: bool,

  /// Keep invisible-times operators (the default). Overrides a package/profile
  /// that turned them off; --noinvisibletimes wins if both are given.
  #[arg(long)]
  invisibletimes: bool,

  /// Suppress the built-in CSS/JS resources.
  #[arg(long)]
  nodefaultresources: bool,

  /// Include the built-in CSS/JS resources (the default);
  /// --nodefaultresources wins if both are given.
  #[arg(long)]
  defaultresources: bool,

  /// Omit source comments from the output.
  #[arg(long)]
  nocomments: bool,

  /// Preserve source `%` comments in the output. This Rust port omits them by
  /// default (Perl keeps them); --nocomments wins if both are given.
  // Divergence from Perl's default-on: see OXIDIZED_DESIGN #2.
  #[arg(long)]
  comments: bool,

  /// Strict mode: treat selected recoverable conditions as hard errors.
  // Perl Core.pm L43: State STRICT.
  #[arg(long)]
  strict: bool,

  /// Raw-load `.sty`/`.cls` sources from the search path instead of relying on
  /// LaTeXML's own bindings.
  ///
  /// WARNING: this enables raw TeX loading for BOTH packages (.sty) AND
  /// document classes (.cls) at once — a common source of errors, since raw
  /// class code is unlikely to convert cleanly.
  // Perl --includestyles / Core.pm L55-57: sets INCLUDE_STYLES + INCLUDE_CLASSES.
  #[arg(long)]
  includestyles: bool,

  /// Reuse an existing `.bbl` file instead of running BibTeX (for arXiv-style
  /// builds that ship their bibliography pre-compiled).
  #[arg(long)]
  nobibtex: bool,

  /// Process the input as a BibTeX `.bib` bibliography. Auto-detected when
  /// SOURCE ends in `.bib` or starts with `literal:@`.
  #[arg(long)]
  bibtex: bool,

  /// Disable math parsing (leave formulae as unparsed token lists).
  #[arg(long, alias = "noparse")]
  nomathparse: bool,

  /// Enable math parsing (the default). Restores it if a profile/package
  /// disabled it; --nomathparse wins if both are given.
  #[arg(long)]
  mathparse: bool,

  /// Emit source locators: record each construct's source range as a
  /// `data-sourcepos` attribute, plus a document-level tag→file table. Off by
  /// default (a normal conversion pays nothing for it). Powers editor/preview
  /// sync and precise linting. Also enabled via `LATEXML_SOURCE_MAP=1`.
  // Issues #47/#92; see docs/performance/SOURCE_PROVENANCE.md.
  #[arg(long = "source-map")]
  source_map: bool,

  /// Disable section numbering.
  #[arg(long, alias = "nosectionnumbers")]
  nonumbersections: bool,

  /// Enable section numbering (the default). Restores it if a profile/package
  /// turned it off; --nonumbersections wins if both are given.
  #[arg(long)]
  numbersections: bool,

  /// Vector-SVG fast path for PDF graphics. `0` (default) auto-detects: vector
  /// PDFs (no raster image, at most 500 KB) go through the SVG converters
  /// (mutool → pdftocairo); raster-bearing PDFs stay on the gs/convert path.
  /// `N > 0` forces the SVG path for any PDF at most N KB. Env
  /// `LATEXML_GRAPHICS_VECTOR_AUTO_OFF=1` disables auto-detect.
  #[arg(
    long = "graphics-svg-threshold-kb",
    value_name = "N",
    default_value = "0"
  )]
  graphics_svg_threshold_kb: u32,

  /// Convert `\includegraphics` figures to web images (the default);
  /// --nographicimages overrides.
  #[arg(long)]
  graphicimages: bool,

  /// Skip figure conversion: leave the raw `<ltx:graphics>` references in the
  /// output. Faster, and works on hosts without the image tools installed.
  #[arg(long)]
  nographicimages: bool,

  /// What to emit: `document` (default; the full page), `fragment` (an
  /// embeddable inline snippet), `math` (just the math subtree), or `archive`
  /// (the page + resources zipped — also implied by a `.zip` --dest, and writes
  /// `<source-name>.zip` when --dest is omitted).
  #[arg(long, value_name = "TYPE")]
  whatsout: Option<String>,

  /// Shortcut for --whatsout=fragment: emit an embeddable inline snippet.
  #[arg(long)]
  embed: bool,

  // === Repeatable flags ===
  /// Add a CSS stylesheet link to the HTML output (repeatable).
  #[arg(long = "css", value_name = "URL")]
  css_files: Vec<String>,

  /// Add a JavaScript link to the HTML output (repeatable).
  #[arg(long = "javascript", value_name = "URL")]
  js_files: Vec<String>,

  /// Preload a package/module before processing, e.g. --preload=amsmath
  /// (repeatable).
  #[arg(long = "preload", value_name = "FILE")]
  preload_files: Vec<String>,

  /// Add a directory to the file/package search path, like TEXINPUTS
  /// (repeatable).
  #[arg(long = "path", value_name = "DIR")]
  search_paths: Vec<String>,

  // === Value flags ===
  /// Conversion timeout in seconds (default: 60). Use 0 to disable.
  #[arg(long, value_name = "SECONDS", default_value = "60")]
  timeout: u64,

  /// Per-conversion memory ceiling in MiB (default: 6144 = 6 GiB). The single
  /// memory knob: a cooperative guard raises a graceful Fatal as digestion
  /// nears it, and a hard watchdog aborts the process (exit 137) at the
  /// ceiling. Use 0 to disable memory limiting entirely. Also settable via the
  /// `LATEXML_MAX_MEMORY` env var; this flag wins when both are given.
  #[arg(long, value_name = "MIB", env = "LATEXML_MAX_MEMORY", default_value = "6144")]
  max_memory: u64,

  /// Abort after processing this many tokens — guards against runaway macro
  /// expansion (default: 400M; env `LATEXML_TOKEN_LIMIT`, 0 disables).
  #[arg(long, value_name = "N")]
  token_limit: Option<usize>,

  /// Navigation table-of-contents style: context or none.
  #[arg(long, alias = "navtoc", value_name = "STYLE")]
  navigationtoc: Option<String>,

  /// Favicon for the generated site: emitted as `<link rel="icon">` and copied
  /// to the destination.
  #[arg(long, value_name = "FILE")]
  icon: Option<String>,

  /// Timestamp string embedded in the page (e.g. a build date in the footer);
  /// --timestamp=0 omits it. Omitted by default, for reproducible output.
  #[arg(long, value_name = "STRING")]
  timestamp: Option<String>,

  /// Apply scholarly-schema post-processing: kind chips on definitions,
  /// pretty-printed content models, and a per-module item index. Intended for
  /// the `tools/generate-scholarly-schema-docs` pipeline; harmless (no effect)
  /// on a generic document.
  #[arg(long)]
  schemadocs: bool,

  /// Write the conversion log to this file (default: stderr).
  #[arg(long, value_name = "PATH")]
  log: Option<String>,

  /// What the input is: `document` (default; a standalone file), `fragment` (a
  /// snippet wrapped with --preamble/--postamble or a standard pre/postamble,
  /// implied if either is given), `math` (a bare formula), `archive` (a `.zip`
  /// bundle, also implied by a `.zip` source), or `directory` (a source dir,
  /// also implied by a trailing `/`).
  #[arg(long, value_name = "TYPE")]
  whatsin: Option<String>,

  /// TeX file effectively prepended to the document (implies --whatsin=fragment).
  #[arg(long, value_name = "FILE")]
  preamble: Option<String>,

  /// TeX file effectively appended to the document (implies --whatsin=fragment).
  #[arg(long, value_name = "FILE")]
  postamble: Option<String>,

  /// Input encoding for decoding source bytes to UTF-8 (default: utf-8), e.g.
  /// iso-8859-1. Translates bytes only, not catcodes — use the inputenc
  /// package for those.
  #[arg(long, value_name = "ENC")]
  inputencoding: Option<String>,

  // === Split options ===
  /// Split the output into multiple linked pages (by section, by default).
  #[arg(long)]
  split: bool,

  /// Force splitting off even when --splitat/--splitpath would enable it.
  #[arg(long)]
  nosplit: bool,

  /// Level to split at: part, chapter, section, subsection, ... (default:
  /// section).
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

  /// Enable a named debug feature, e.g. --debug frontmatter (repeatable).
  /// Implies debug-level logging.
  #[arg(long = "debug", value_name = "FEATURE")]
  debug: Vec<String>,

  /// Assign an ID to the document's root element.
  #[arg(long, value_name = "ID")]
  documentid: Option<String>,

  /// Root directory of the generated site; resource URLs are made relative to
  /// it (default: the destination's directory).
  #[arg(long, value_name = "DIR")]
  sitedirectory: Option<String>,

  /// Directory of the original source, searched for graphics and resources
  /// during post-processing (default: the source file's directory).
  #[arg(long, value_name = "DIR")]
  sourcedirectory: Option<String>,

  /// Additional XSLT parameters (repeatable, key=value)
  #[arg(long = "xsltparameter", value_name = "KEY=VALUE")]
  xslt_parameters: Vec<String>,

  // === Dev/internal flags ===
  /// Developer tool: process a format file and dump its compiled engine state.
  #[arg(long, value_name = "FILE")]
  init: Option<String>,

  /// Developer tool: generate Rust source from a dump file.
  #[arg(long, value_name = "DUMP")]
  codegen: Option<String>,

  /// Developer tool: dump the compiled schema model (.model text) to stdout
  /// and exit. Only the embedded LaTeXML schema is supported.
  #[arg(long)]
  dump_model: bool,

  /// Write a one-line JSON telemetry record for this job to this file (or set
  /// env `LATEXML_TELEMETRY_OUT`). Written only on a successful conversion.
  #[arg(long, value_name = "PATH")]
  telemetry_out: Option<String>,

  /// Run as a persistent JSON-RPC-over-stdio (LSP) server for editor/preview
  /// integration.
  #[arg(long)]
  server: bool,
}

/// Allocation-failure hook — emits a `Fatal:` line in the project's
/// logging convention so aggregation tooling records the failure, then
/// exits with code 137. See `cortex_worker.rs::custom_alloc_error_hook`
/// for full rationale + witness paper.
fn custom_alloc_error_hook(layout: Layout) {
  eprintln!(
    "Fatal:oom:alloc_failed allocation of {} bytes (align {}) failed; \
     likely runaway macro expansion (gullet pushback Vec growth past \
     worker memory budget). Exiting with code 137.",
    layout.size(),
    layout.align()
  );
  process::exit(137);
}

fn main() -> Result<(), Box<dyn Error>> {
  set_alloc_error_hook(custom_alloc_error_hook);

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
  // Heap profiler (`--features dhat-heap`). Held for the whole conversion, which
  // runs on this (worker) thread. The success/fatal exits below go through
  // `process::exit`, which skips destructors, so the profile is flushed
  // explicitly via `_dhat.take()` before those exits (writing `dhat-heap.json`);
  // a normal `return` still drops it as a fallback.
  #[cfg(feature = "dhat-heap")]
  let mut _dhat = Some(dhat::Profiler::new_heap());

  let wall_start = std::time::Instant::now();
  let cli = Cli::parse();

  // Kick off kpathsea pre-init in a background thread. Force-runs
  // `kpathsea_init_db` + per-format `kpathsea_init_format` so the
  // first real `find_file` from digest sees the fast post-init path
  // instead of paying ~30-40 ms of setup on its first lookup. The
  // worker briefly holds the `KPSE` Mutex while it probes — a main-
  // thread `kpsewhich(...)` racing in early would block briefly, but
  // dump load + arg parsing run for >50 ms before any digest-time
  // package resolution, so the warm-up usually completes before its
  // first real consumer arrives. Disable with
  // `LATEXML_NO_KPATHSEA_PREWARM=1` for A/B benchmarking.
  //
  // This is now purely a *latency* pre-warm (overlap the init cost with dump
  // load). The CORRECTNESS invariant — tables warm before the first `find_file`
  // — is enforced by the shared `Converter::initialize_session`, the single path
  // both this binary and the library (`latexml::api`, tests) funnel through, so
  // the library can no longer drift from the binaries the way it did (the flaky
  // spurious "1 warning" root-caused 2026-07-16).
  let _kpse_warmup_handle = if std::env::var("LATEXML_NO_KPATHSEA_PREWARM").is_err() {
    Some(std::thread::spawn(
      latexml_core::util::pathname::prewarm_kpathsea,
    ))
  } else {
    None
  };

  // Initialize logger with verbosity level
  let verbosity: i32 = if cli.quiet {
    -1
  } else if cli.verbose {
    1
  } else {
    0
  };
  let log_level = if !cli.debug.is_empty() {
    // --debug NAME implies debug-level logging (Perl: Debug() output is
    // emitted whenever the feature flag is set).
    log::LevelFilter::Debug
  } else {
    match verbosity {
      v if v < 0 => log::LevelFilter::Warn,
      0 => log::LevelFilter::Info,
      _ => log::LevelFilter::Debug,
    }
  };
  latexml_core::util::logger::init(log_level).ok();
  // Perl: --debug=NAME sets $LaTeXML::DEBUG{NAME}; gates DebugFeature! sites.
  for feature in &cli.debug {
    latexml_core::common::error::enable_debug_feature(feature);
  }

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

  // Persistent LSP Server mode — handle early before source file checks
  if cli.server {
    log::info!("Starting persistent LSP server...");
    latexml::lsp_server::run_lsp_server(cli.timeout, cli.max_memory)?;
    process::exit(0);
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

  // Resolve `--whatsout <mode>` (Perl Pack.pm `whatsout` option +
  // Config.pm L421-439). Explicit `--whatsout` wins; otherwise a `.zip`
  // destination extension implies `archive` (Config.pm L421-426).
  // Unknown explicit values fall back to `document`, like Perl
  // `pack_collection`. Hoisted here (rather than inside the post block)
  // so both the post stage and the post-run `--log` guard can see it.
  let dest_ext_is_zip = target
    .as_deref()
    .is_some_and(|t| t.to_ascii_lowercase().ends_with(".zip"));
  let whatsout_mode = match cli.whatsout.as_deref() {
    Some(s) => latexml_post::extract::Whatsout::from_cli(s).unwrap_or_default(),
    None if dest_ext_is_zip => latexml_post::extract::Whatsout::Archive,
    // `--embed` is Perl's shortcut for `--whatsout=fragment` (Config.pm L72).
    None if cli.embed => latexml_post::extract::Whatsout::Fragment,
    None => latexml_post::extract::Whatsout::Document,
  };
  let is_archive_out = whatsout_mode.is_archive();

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
    let dir_path = Path::new(&source);
    if let Ok(abs_source) = std::fs::canonicalize(dir_path) {
      path_flags.push(abs_source.to_string_lossy().to_string());
    } else {
      path_flags.push(source.clone());
    }
    // Find the main .tex file in the directory, matching Perl's behavior
    match latexml::main_tex::find_main_tex(dir_path) {
      Ok(main_tex) => main_tex.to_string_lossy().to_string(),
      Err(e) => {
        eprintln!("Failed to find main .tex file in '{}': {}", source, e);
        process::exit(1);
      },
    }
  } else {
    source
  };

  // Some arXiv source archives ship a PDF mis-named with a `.tex` extension
  // (e.g. 2301.04210.tex). Perl LaTeXML detects the `%PDF-` magic and bails
  // with a single Fatal; without this guard the binary catcode-tokenizes
  // the PDF stream and emits ~100 Error:undefined / Error:unexpected lines.
  if Path::new(&source).is_file() && latexml::main_tex::is_pdf_magic(Path::new(&source)) {
    eprintln!(
      "Fatal:invalid:not_tex_source PDF magic detected in source file '{}'",
      source
    );
    process::exit(1);
  }

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

  // Perl `Common/Config.pm:24,216`: `$is_bibtex = qr/(^literal:\s*@)|(\.bib$)/`.
  // `--bibtex` forces the type; otherwise auto-detect when the
  // source ends in `.bib` or begins with `literal:@`.
  let is_literal_bib = {
    let trimmed = source.trim_start_matches("literal:");
    trimmed.trim_start().starts_with('@') && trimmed.len() < source.len()
  };
  let mode = if cli.bibtex || source.ends_with(".bib") || is_literal_bib {
    Some(DigestionMode::BibTeX)
  } else {
    None
  };

  // Map `--whatsin` to the core input-chunk size (Perl Config.pm
  // L399-404 + LaTeXML.pm:165-194). `archive`/`directory` have already
  // been resolved to a concrete main `.tex` above, so the core digests
  // them as a plain document; only `math`/`fragment` change the core's
  // preamble/postamble wrapping. When `--whatsin` is unset, a supplied
  // `--preamble`/`--postamble` implies a `fragment` input.
  let whatsin_size = match cli.whatsin.as_deref() {
    Some("math") => DataSize::Math,
    Some("fragment") => DataSize::Fragment,
    Some("document") | Some("archive") | Some("directory") => DataSize::Document,
    None if cli.preamble.is_some() || cli.postamble.is_some() => DataSize::Fragment,
    _ => DataSize::Document,
  };

  let opts = Config {
    verbosity,
    format: OutputFormat::HTML5,
    whatsin: whatsin_size,
    whatsout: DataSize::Document,
    preamble: cli.preamble.clone(),
    postamble: cli.postamble.clone(),
    mode,
    bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
    extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
    preload,
    search_paths,
    // Perl Core.pm L45: INCLUDE_COMMENTS. `--nocomments` wins over `--comments`;
    // otherwise leave unset so the Rust default (OFF — OXIDIZED_DESIGN #2) holds.
    include_comments: if cli.nocomments {
      Some(false)
    } else if cli.comments {
      Some(true)
    } else {
      None
    },
    // Perl Core.pm L43: STRICT; L55-57: INCLUDE_STYLES/INCLUDE_CLASSES.
    strict: if cli.strict { Some(true) } else { None },
    include_styles: if cli.includestyles { Some(true) } else { None },
    // `--nomathparse` disables; `--mathparse` explicitly enables (the default).
    nomathparse: if cli.nomathparse {
      Some(true)
    } else if cli.mathparse {
      Some(false)
    } else {
      None
    },
    // `--source-map` flag OR `LATEXML_SOURCE_MAP` env enables locator
    // tracking + emission; otherwise leave unset (off). The env reads
    // once here, off the hot path. See `docs/performance/SOURCE_PROVENANCE.md`.
    source_map: if cli.source_map || std::env::var_os("LATEXML_SOURCE_MAP").is_some() {
      Some(true)
    } else {
      None
    },
    // Perl Config.pm L57 / Core.pm L60-61: --inputencoding seeds State
    // PERL_INPUT_ENCODING, which the Mouth reads to decode source bytes
    // (default utf-8 when unset).
    inputencoding: cli.inputencoding.clone(),
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
  // Skip engine init for already-converted XML input: post-processing is pure
  // libxml2 and never touches the TeX engine or its dump, so loading
  // TeX.pool/latex + the format dump (~85–160 ms and a chunk of RSS) is wasted
  // work. Init mode (`--init`) and every real TeX conversion still need it.
  if (cli.init.is_some() || !is_xml_input(&source))
    && let Err(e) = converter.prepare_session(&opts)
  {
    eprintln!("Could not prepare converter session: {}", e);
    process::exit(1);
  }

  // Wire state-level options
  if cli.nobibtex {
    // Set BIB_CONFIG to ['bbl'] — skip BibTeX, use pre-existing .bbl file
    latexml_core::state::assign_value(
      "BIB_CONFIG",
      latexml_core::common::store::Stored::Strings(Rc::new([latexml_core::common::arena::pin(
        "bbl",
      )])),
      Some(latexml_core::state::Scope::Global),
    );
  }
  if cli.nonumbersections {
    latexml_core::state::assign_value(
      "no_number_sections",
      true,
      Some(latexml_core::state::Scope::Global),
    );
  } else if cli.numbersections {
    // Positive complement (Perl `numbersections!`, default on): explicitly
    // restore section numbering if a profile/package turned it off.
    latexml_core::state::assign_value(
      "no_number_sections",
      false,
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
    // automatically on drop at end of main. `--max-memory` rides the same
    // Watchdog (it was previously a silent no-op outside `--server`).
    let _watchdog = latexml_core::watchdog::Watchdog::with_limits(
      cli.timeout,
      cli.max_memory.saturating_mul(1024),
    );
    // `--max-memory` (or its `LATEXML_MAX_MEMORY` env) is the SINGLE memory
    // knob. The hard Watchdog ceiling above rides it directly; the cooperative
    // stomach RSS fuse is DERIVED from it (a fixed fraction below, leaving
    // post-processing headroom) rather than being an independent number — so
    // one flag governs one limit and `--max-memory=0` disables both. An
    // explicit `LATEXML_RSS_CAP_BYTES` still overrides just the soft fuse (the
    // fleet/test decoupling escape hatch), so honor it when present.
    let rss_cap_env_set = std::env::var_os("LATEXML_RSS_CAP_BYTES").is_some();
    if !rss_cap_env_set {
      latexml_core::stomach::set_memory_cap(Some(
        latexml_core::stomach::soft_cap_from_ceiling(cli.max_memory),
      ));
    }
    if cli.max_memory == 0 && !rss_cap_env_set {
      // Removing the surprise of one flag silently disabling two guards: say
      // so, out loud, when the whole memory limit is off.
      latexml_core::Warn!(
        "memory",
        "unlimited",
        "--max-memory=0: memory limiting disabled entirely (cooperative guard + hard watchdog); a runaway conversion may exhaust host RAM"
      );
    }
    if cli.timeout > 0 {
      latexml_core::stomach::set_timeout(cli.timeout);
    }
    if let Some(limit) = cli.token_limit {
      // 0 disables (as documented), matching the LATEXML_TOKEN_LIMIT env
      // convention (`Some(0) => None` at the gullet initializer). Passing a
      // literal `Some(0)` would instead fatal on the first token.
      latexml_core::gullet::set_token_limit((limit != 0).then_some(limit));
    }

    let source_for_post = source.clone();
    // XML-input mode: when the source is already-converted LaTeXML XML
    // (file extension `.xml`/`.xhtml` or content starts with `<?xml`
    // / `<document xmlns="…">`), skip the TeX → XML converter and feed
    // the file straight to post-processing. Mirrors what
    // `latexmlpost_oxide` did as a separate binary (per the
    // retirement plan in `docs/SYNC_STATUS.md`).
    let response = if is_xml_input(&source) {
      // Do NOT slurp the file into a String — a large already-converted
      // document (the reporter's index.xml is 614 MB) would sit resident on
      // top of the ~11× libxml2 DOM. Post-processing streams it from disk via
      // `PostDocument::new_from_file` (see the `run_post_processing_from_file*`
      // call below). We only need a non-empty `result` sentinel so the post
      // gate fires; the real input is the source path.
      latexml::converter::ConversionResponse {
        result:      Some(String::new()),
        log:         String::new(),
        status:      String::from("Status:conversion:0"),
        status_code: 0,
      }
    } else {
      converter.convert(source)
    };
    let _ = &source_for_post; // keep alive for post-processing
    // Post-phase log (Graphics/MathML/XSLT) captured by
    // `run_post_processing_logged`; written after the core log into --log / the
    // archive log so BOTH conversion phases reach the persisted log (SYNC_STATUS
    // task 5; Perl LaTeXML.pm flushes once after convert_post). Declared out here
    // (not in the `Some(xml)` arm) so the post-if-let --log write can still see
    // it; stays empty when post-processing is skipped, keeping that --log
    // byte-identical to before.
    let mut post_log = String::new();
    if let Some(xml) = response.result {
      // Infer format from --dest extension if --format not specified (Perl Config.pm L408-441)
      let inferred_format: Option<String> = cli
        .format
        .clone()
        // `--xml` is Perl's shortcut for `--format=xml` (Config.pm L59).
        .or(if cli.xml {
          Some("xml".to_string())
        } else {
          None
        })
        .or_else(|| {
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
        })
        // `--whatsout=archive` with no `--dest`/`--format` still wants a
        // web bundle — default it to html5, matching the `--dest *.zip`
        // inference above (a `.zip` dest already maps to html5).
        .or_else(|| {
          if is_archive_out {
            Some("html5".to_string())
          } else {
            None
          }
        });

      // Auto-select stylesheet from format (Perl Config.pm L543-551)
      // Shared with the library entrypoint via `post::default_stylesheet` so
      // the CLI and `latexml::api` never disagree on the per-format sheet.
      let effective_stylesheet = cli.stylesheet.clone().or_else(|| {
        latexml::post::default_stylesheet(inferred_format.as_deref()).map(String::from)
      });

      // Auto-enable post-processing when dest implies HTML (Perl Config.pm L448-455)
      let is_html_format = matches!(
        inferred_format.as_deref(),
        Some("html5") | Some("html") | Some("xhtml") | Some("epub") | Some("epub3")
      );
      // XML-input mode implies post-processing — there's nothing to
      // convert (the file is already converted XML), so the only
      // meaningful action is to run the post-pipeline on it.
      // Matches the always-on post-processing behaviour of the now-
      // retired `latexmlpost_oxide` binary.
      let xml_input_mode = is_xml_input(&source_for_post);
      // `--nopost` (Perl `post!` negated) force-skips post-processing so the
      // raw LaTeXML XML is emitted even for an HTML-implying destination.
      let do_post = !cli.nopost
        && (cli.post
          || cli.pmml
          || cli.cmml
          || effective_stylesheet.is_some()
          || is_html_format
          || cli.split
          || cli.splitat.is_some()
          || xml_input_mode
          // Perl Config.pm L454: any non-`document` whatsout forces post.
          || whatsout_mode.requires_post());

      // Build split XPath from --splitat
      // `--nosplit` (Perl `split!` negated) forces splitting off even when a
      // `--splitat`/`--splitpath`/`--splitnaming` would otherwise enable it.
      let split_enabled = !cli.nosplit
        && (cli.split
          || cli.splitat.is_some()
          || cli.splitnaming.is_some()
          || cli.splitpath.is_some());
      let split_xpath = if split_enabled {
        cli.splitpath.clone().or_else(|| {
          let splitat = cli.splitat.as_deref().unwrap_or("section");
          Some(make_splitpaths(splitat))
        })
      } else {
        None
      };

      if do_post {
        // Perl LaTeXML.pm:429 passes opts{sourcedirectory} to Post as
        // `sourceDirectory`; when omitted, Post derives it from the source
        // filename (Post.pm:727-729). Mirror that: honour `--sourcedirectory`
        // if given, else fall back to the source file's own directory.
        let source_dir = cli.sourcedirectory.clone().unwrap_or_else(|| {
          Path::new(&source_for_post)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
        });

        // `--whatsout=archive` (or a `.zip` destination) bundles into a
        // zip. When `--dest` is omitted, Perl LaTeXML.pm:185-187 invents
        // a placeholder `<source-name>.zip`; mirror that so an archive
        // job always lands a file rather than dumping HTML to stdout.
        let zip_dest: Option<String> = if is_archive_out {
          Some(match &target {
            Some(t) if t.to_ascii_lowercase().ends_with(".zip") => t.clone(),
            Some(t) => format!(
              "{}.zip",
              t.trim_end_matches(".html").trim_end_matches(".xml")
            ),
            None => {
              let stem = Path::new(&source_for_post)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
              format!("{stem}.zip")
            },
          })
        } else {
          None
        };

        // For zip output, route graphics conversions through a TempDir so
        // the converted PNG/SVG files can be collected and bundled into
        // the output zip (mirroring `cortex_worker::pack_output_zip_with_resources`).
        // Without this, the Graphics post-processor wrote PNGs next to
        // `target` on the filesystem but the zip only carried HTML+log+status —
        // confirmed-bug 2026-05-18 on 1910.01256.
        let resource_tempdir: Option<tempfile::TempDir> = if is_archive_out {
          Some(tempfile::tempdir()?)
        } else {
          None
        };
        let dest_for_post: Option<String> = if let Some(tmp) = resource_tempdir.as_ref() {
          // Use a stable HTML filename derived from the zip stem so the
          // Graphics processor's relative paths resolve naturally.
          let stem = zip_dest
            .as_deref()
            .and_then(|z| Path::new(z).file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("document");
          Some(
            tmp
              .path()
              .join(format!("{stem}.html"))
              .to_string_lossy()
              .to_string(),
          )
        } else {
          target.clone()
        };

        // latexmlpost_oxide's default was "if no --pmml AND no
        // --stylesheet, default pmml = true". Apply the same rule for
        // XML-input mode so `latexml_oxide foo.xml --dest out.html`
        // does something useful out of the box.
        let default_pmml_for_xml_input =
          xml_input_mode && !cli.pmml && effective_stylesheet.is_none();
        let post_opts = PostOptions {
          // The `no*` forms (Perl `removeMathFormat`) suppress a rep that a
          // format would otherwise default on.
          pmml: (cli.pmml || cli.post || is_html_format || default_pmml_for_xml_input)
            && !cli.nopmml,
          cmml: cli.cmml && !cli.nocmml,
          keep_xmath: cli.keep_xmath && !cli.noxmath,
          stylesheet: effective_stylesheet.as_deref(),
          destination: dest_for_post.as_deref(),
          source_directory: Some(&source_dir),
          // Perl LaTeXML.pm:430 `siteDirectory`; None ⇒ Post defaults it to the
          // destination's directory (document.rs / Perl Config.pm L466-469).
          site_directory: cli.sitedirectory.as_deref(),
          search_paths: &cli.search_paths,
          nodefaultresources: cli.nodefaultresources && !cli.defaultresources,
          css_files: &cli.css_files,
          js_files: &cli.js_files,
          noinvisibletimes: cli.noinvisibletimes && !cli.invisibletimes,
          mathtex: cli.mathtex && !cli.nomathtex,
          navigationtoc: cli.navigationtoc.as_deref(),
          schemadocs: cli.schemadocs,
          split: split_enabled,
          split_xpath,
          split_naming: cli.splitnaming.as_deref(),
          xslt_parameters: &cli.xslt_parameters,
          graphics_svg_threshold_kb: cli.graphics_svg_threshold_kb,
          graphicimages: cli.graphicimages || !cli.nographicimages,
          // Perl `if ($timestamp)`: "0" (and empty) means "omit the timestamp".
          timestamp: cli
            .timestamp
            .as_deref()
            .filter(|t| !t.is_empty() && *t != "0"),
          icon: cli.icon.as_deref(),
          whatsout: whatsout_mode,
        };
        // XML-input mode parses the (possibly huge) source straight from disk
        // via the streaming file reader; TeX-conversion output is already in
        // memory as `xml`.
        let post = if xml_input_mode {
          latexml::post::run_post_processing_from_file_logged(&source_for_post, &post_opts)
        } else {
          latexml::post::run_post_processing_logged(&xml, &post_opts)
        };
        let output = post.html;
        post_log = post.log;
        if let Some(zip_dest) = zip_dest {
          // whatsout=archive: pack the full document + resources into a ZIP.
          latexml_post::writer::ensure_parent_dir(&zip_dest)?;
          let resource_dir = resource_tempdir.as_ref().map(|t| t.path());
          let stem = Path::new(&zip_dest)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
          let html_name = format!("{stem}.html");
          let log_name = format!("{stem}.log");
          // Reproducible-build support: honour SOURCE_DATE_EPOCH for the
          // zip member timestamps (Perl Pack/Zip.pm L113-115).
          let source_date_epoch = std::env::var("SOURCE_DATE_EPOCH")
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok());
          latexml_post::pack::pack_archive(&latexml_post::pack::PackOptions {
            zip_path: &zip_dest,
            html_filename: &html_name,
            html: &output,
            log_filename: Some(&log_name),
            log: &assemble_conversion_log(&response.log, &post_log),
            status: &response.status,
            resource_dir,
            telemetry_json: None,
            source_date_epoch,
          })?;
          eprintln!("Output written to {}", zip_dest);
        } else {
          latexml_post::writer::write_output(&output, target.as_deref())?;
        }
        // resource_tempdir is dropped here (after pack_archive has copied
        // every file in), cleaning up the converted-PNG staging directory.
      } else {
        latexml_post::writer::write_output(&xml, target.as_deref())?;
      }
    }

    // --log: write conversion log to file (skip if already packed into
    // the ZIP by the archive output stage).
    if let Some(ref log_path) = cli.log
      && !is_archive_out
    {
      // Write the core log and the post-phase log sequentially rather than
      // concatenating — both are already-allocated and large for real
      // articles, so a merged `format!` would allocate a third copy of their
      // combined size on the conversion path.
      if post_log.is_empty() {
        latexml_post::writer::write_output_segments(&[response.log.as_str()], Some(log_path))?;
      } else {
        latexml_post::writer::write_output_segments(
          &[response.log.as_str(), "\n", post_log.as_str()],
          Some(log_path),
        )?;
      }
      eprintln!("Log written to {}", log_path);
    }
  }

  // Perl bin/latexml:151 — `if ($exit_message) { exit(1); }`: a Fatal
  // (status_code 3) conversion exits non-zero. cortex_worker already carries the
  // identical guard (`if final_status >= 3 { process::exit(...) }`); the standalone
  // CLI was missing it, so a 0-byte "complete" run (e.g. the plain-TeX
  // `$\displaylines{...}$` runaway that trips the memory-budget Fatal — shared with
  // Perl, which terminates at the same line) exited 0 and masqueraded as success.
  // Read the global status (thread-local REPORT, as cortex_worker does) — `response`
  // is scoped to the conversion branch. Match bin/latexml's exit(1) exactly;
  // status_code 2 ("errors but recoverable") stays a 0 exit, as in Perl.
  let final_status_code = latexml_core::common::error::get_status_code();
  if final_status_code >= 3 {
    write_telemetry_record(
      cli.telemetry_out.as_deref(),
      &telemetry_source,
      wall_start,
      "fatal",
      final_status_code as i32,
    );
    #[cfg(feature = "dhat-heap")]
    drop(_dhat.take());
    process::exit(1);
  }
  write_telemetry_record(
    cli.telemetry_out.as_deref(),
    &telemetry_source,
    wall_start,
    "ok",
    0,
  );
  #[cfg(feature = "dhat-heap")]
  drop(_dhat.take());
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
  let paper_id = Path::new(source)
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
  if let Some(parent) = Path::new(&path).parent()
    && !parent.as_os_str().is_empty()
  {
    let _ = std::fs::create_dir_all(parent);
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
fn read_child_rusage_us() -> (u64, u64) { (0, 0) }

use latexml::post::PostOptions;

/// Assemble the persisted conversion log: core conversion log followed by the
/// captured post-phase log (Graphics/MathML/XSLT). Mirrors Perl `LaTeXML.pm`,
/// whose single `flush_log()` after `convert_post` returns both phases in one
/// buffer. `post_log` is empty when post-processing was skipped, in which case
/// the core log is returned unchanged (no behavioral drift for non-post runs).
fn assemble_conversion_log(core_log: &str, post_log: &str) -> String {
  if post_log.trim().is_empty() {
    core_log.to_string()
  } else {
    format!("{}\n{}", core_log.trim_end(), post_log.trim_end())
  }
}

// `ensure_parent_dir` now lives in `latexml_post::writer` so all
// post-processing binaries share one implementation. Perl analog:
// `LaTeXML::Post::Writer`.

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
/// Detect whether `source` is already-converted LaTeXML XML — i.e. a
/// `.xml` file — so the TeX → XML converter front-end can be skipped
/// and the file fed straight to post-processing. Matches what Perl
/// `latexmlpost` accepts and replaces the separate (now retired)
/// `latexmlpost_oxide` binary.
fn is_xml_input(source: &str) -> bool {
  Path::new(source)
    .extension()
    .and_then(|e| e.to_str())
    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
}

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
  let main_tex =
    latexml::main_tex::find_main_tex(dest).map_err(|e| -> Box<dyn Error> { e.into() })?;
  Ok((tempdir, main_tex.to_string_lossy().to_string()))
}

// Output-zip packing moved to `latexml_post::pack::pack_archive`
// (2026-05-18, audit follow-up for the latexml_oxide --post image-
// bundling fix). The previous inline `pack_output_zip` +
// `add_dir_to_zip` here and the parallel pair in `cortex_worker.rs`
// have been replaced by a single shared implementation — mirrors
// Perl `LaTeXML::Post::Pack`.
