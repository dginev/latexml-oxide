//! CorTeX Worker for latexml-oxide
//!
//! Implements the pericortex Worker trait to integrate latexml_oxide
//! with the CorTeX distributed processing framework.
//!
//! Two modes:
//! - Worker mode (default): connects to CorTeX dispatcher via ZMQ
//! - Standalone mode (--standalone): single ZIP-to-ZIP conversion

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;

use clap::Parser;
use pericortex::worker::Worker;
use tempfile::TempDir;

use latexml::converter::Converter;
use latexml_core::common::{Config, DataSize, OutputFormat};

/// CorTeX worker for LaTeXML-Oxide: distributed TeX-to-HTML conversion
#[derive(Parser, Debug)]
#[command(name = "cortex-worker", about = "CorTeX worker for latexml-oxide")]
struct Cli {
  /// Dispatcher ventilator address
  #[arg(long, default_value = "tcp://localhost:51695")]
  source_address: String,

  /// Dispatcher sink address
  #[arg(long, default_value = "tcp://localhost:51696")]
  sink_address: String,

  /// Service name as registered in CorTeX
  #[arg(long, default_value = "oxidized_tex_to_html")]
  service: String,

  /// Number of parallel worker threads
  #[arg(long, default_value = "1")]
  pool_size: usize,

  /// ZMQ message frame size in bytes
  #[arg(long, default_value = "100000")]
  message_size: usize,

  /// Maximum number of tasks to process before exiting
  #[arg(long)]
  limit: Option<usize>,

  /// Run in standalone mode (single conversion, no dispatcher)
  #[arg(long)]
  standalone: bool,

  /// Input ZIP file (standalone mode only)
  #[arg(long)]
  input: Option<String>,

  /// Output ZIP file (standalone mode only, default: stdout)
  #[arg(long)]
  output: Option<String>,

  /// Conversion profile: ar5iv, generic
  #[arg(long, default_value = "ar5iv")]
  profile: String,

  /// Additional packages to preload
  #[arg(long)]
  preload: Vec<String>,

  /// Per-document timeout in seconds
  #[arg(long, default_value = "600")]
  timeout: u64,

  /// Disable Presentation MathML
  #[arg(long)]
  no_pmml: bool,

  /// Disable TeX annotations in MathML
  #[arg(long)]
  no_mathtex: bool,

  /// Verbose output
  #[arg(short, long)]
  verbose: bool,

  /// Quiet output
  #[arg(short, long)]
  quiet: bool,
}

/// Conversion profile presets
#[allow(dead_code)] // Fields used when post-processing is enabled
#[derive(Clone, Debug)]
struct ConversionProfile {
  preloads: Vec<String>,
  pmml: bool,
  mathtex: bool,
  noinvisibletimes: bool,
  nodefaultresources: bool,
  timeout: u64,
}

impl ConversionProfile {
  fn ar5iv(extra_preloads: &[String], timeout: u64, no_pmml: bool, no_mathtex: bool) -> Self {
    let mut preloads = vec!["ar5iv.sty".to_string()];
    preloads.extend(extra_preloads.iter().cloned());
    ConversionProfile {
      preloads,
      pmml: !no_pmml,
      mathtex: !no_mathtex,
      noinvisibletimes: true,
      nodefaultresources: true,
      timeout,
    }
  }

  fn generic(extra_preloads: &[String], timeout: u64, no_pmml: bool, no_mathtex: bool) -> Self {
    ConversionProfile {
      preloads: extra_preloads.to_vec(),
      pmml: !no_pmml,
      mathtex: !no_mathtex,
      noinvisibletimes: false,
      nodefaultresources: false,
      timeout,
    }
  }
}

/// The CorTeX worker implementation for latexml-oxide
#[derive(Clone)]
struct LatexmlWorker {
  service: String,
  source_address: String,
  sink_address: String,
  identity: String,
  msg_size: usize,
  threads: usize,
  profile: ConversionProfile,
  verbosity: i32,
}

impl LatexmlWorker {
  /// Run the conversion pipeline on an input ZIP archive.
  /// Returns the path to the output ZIP file.
  fn convert_archive(&self, input_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    // 1. Unpack the input archive
    let (tempdir, main_tex) = unpack_archive(input_path)?;
    let source_dir = tempdir.path().to_string_lossy().to_string();

    // 2. Set up the converter with the profile
    let mut preloads = vec!["TeX.pool".to_string()];
    preloads.extend(self.profile.preloads.iter().cloned());

    let opts = Config {
      verbosity: self.verbosity,
      format: OutputFormat::HTML5,
      whatsin: DataSize::Document,
      whatsout: DataSize::Document,
      preamble: None,
      postamble: None,
      mode: None,
      bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
      extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
      preload: Some(self.profile.preloads.clone()),
      search_paths: Some(vec![source_dir.clone()]),
      include_comments: Some(false),
      nomathparse: None,
    };

    let mut converter = Converter::from_config(opts.clone());
    if let Err(e) = converter.prepare_session(&opts) {
      return Err(format!("Failed to prepare converter: {}", e).into());
    }

    // 3. Convert
    let response = converter.convert(main_tex.clone());
    let xml = response
      .result
      .ok_or_else(|| format!("Conversion failed for {}", main_tex))?;

    // 4. Get log and status
    // TODO: add post-processing (MathML + XSLT) once latexml_post APIs are public
    let log = response.log;
    let status_str = format!("Status:conversion:{}", response.status_code);

    // 6. Pack into output ZIP in /tmp (outside tempdir so it survives cleanup)
    let output_path = std::env::temp_dir().join(format!("cortex_output_{}.zip", std::process::id()));
    pack_output_zip(&output_path, &xml, &log, &status_str)?;

    Ok(output_path)
  }
}

impl Worker for LatexmlWorker {
  fn convert(&self, path: &Path) -> Result<File, Box<dyn Error>> {
    let output_path = self.convert_archive(path)?;
    let file = File::open(&output_path)?;
    // Clean up temp file after opening
    let _ = fs::remove_file(&output_path);
    Ok(file)
  }

  fn message_size(&self) -> usize {
    self.msg_size
  }

  fn get_service(&self) -> &str {
    &self.service
  }

  fn get_source_address(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.source_address)
  }

  fn get_sink_address(&self) -> Cow<'_, str> {
    Cow::Borrowed(&self.sink_address)
  }

  fn pool_size(&self) -> usize {
    self.threads
  }

  fn set_identity(&mut self, identity: String) {
    self.identity = identity;
  }

  fn get_identity(&self) -> &str {
    &self.identity
  }
}


// --- Helper functions (shared with latexml_oxide.rs) ---

fn unpack_archive(archive_path: &Path) -> Result<(TempDir, String), Box<dyn Error>> {
  let tempdir = TempDir::new()?;
  let dest = tempdir.path();

  let path_str = archive_path.to_string_lossy();
  if path_str.ends_with(".zip") {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(dest)?;
  } else if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
    let file = File::open(archive_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    archive.unpack(dest)?;
  } else if path_str.ends_with(".tar") {
    let file = File::open(archive_path)?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(dest)?;
  } else {
    return Err(format!("Unsupported archive format: {}", path_str).into());
  }

  let main_tex = find_main_tex(dest)?;
  Ok((tempdir, main_tex))
}

fn find_main_tex(dir: &Path) -> Result<String, Box<dyn Error>> {
  // Simple heuristic: find the .tex file with \documentclass
  let mut candidates: Vec<PathBuf> = Vec::new();
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.extension().map_or(false, |e| e == "tex") {
      candidates.push(path);
    }
  }

  if candidates.is_empty() {
    return Err("No .tex files found in archive".into());
  }
  if candidates.len() == 1 {
    return Ok(candidates[0].to_string_lossy().to_string());
  }

  // Check for \documentclass
  for c in &candidates {
    if let Ok(content) = fs::read_to_string(c) {
      if content.contains("\\documentclass") {
        return Ok(c.to_string_lossy().to_string());
      }
    }
  }

  // Fallback: largest file
  candidates.sort_by(|a, b| {
    let sa = fs::metadata(a).map(|m| m.len()).unwrap_or(0);
    let sb = fs::metadata(b).map(|m| m.len()).unwrap_or(0);
    sb.cmp(&sa)
  });
  Ok(candidates[0].to_string_lossy().to_string())
}

fn pack_output_zip(
  output_path: &Path,
  html: &str,
  log: &str,
  status: &str,
) -> Result<(), Box<dyn Error>> {
  let file = File::create(output_path)?;
  let mut zip = zip::ZipWriter::new(file);
  let options = zip::write::SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated);

  zip.start_file("output.html", options)?;
  zip.write_all(html.as_bytes())?;

  zip.start_file("cortex.log", options)?;
  zip.write_all(log.as_bytes())?;

  zip.start_file("status", options)?;
  zip.write_all(status.as_bytes())?;

  zip.finish()?;
  Ok(())
}

// --- Main ---

fn main() -> Result<(), Box<dyn Error>> {
  let cli = Cli::parse();

  let verbosity = if cli.quiet {
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

  let profile = match cli.profile.as_str() {
    "ar5iv" => ConversionProfile::ar5iv(&cli.preload, cli.timeout, cli.no_pmml, cli.no_mathtex),
    "generic" => ConversionProfile::generic(&cli.preload, cli.timeout, cli.no_pmml, cli.no_mathtex),
    other => {
      eprintln!("Unknown profile '{}', using ar5iv", other);
      ConversionProfile::ar5iv(&cli.preload, cli.timeout, cli.no_pmml, cli.no_mathtex)
    }
  };

  let hostname = hostname::get()
    .unwrap_or_else(|_| OsString::from("localhost"))
    .into_string()
    .unwrap_or_else(|_| "localhost".to_string());

  let mut worker = LatexmlWorker {
    service: cli.service.clone(),
    source_address: cli.source_address.clone(),
    sink_address: cli.sink_address.clone(),
    identity: format!("{}:{}:01", hostname, cli.service),
    msg_size: cli.message_size,
    threads: cli.pool_size,
    profile,
    verbosity,
  };

  if cli.standalone {
    // Standalone mode: single conversion
    let input = cli.input.unwrap_or_else(|| {
      eprintln!("Error: --input required in standalone mode");
      process::exit(1);
    });

    eprintln!("Converting {} ...", input);
    let result_path = worker.convert_archive(Path::new(&input))?;

    // Read result and write to output
    let mut result_data = Vec::new();
    File::open(&result_path)?.read_to_end(&mut result_data)?;

    if let Some(output) = cli.output {
      fs::write(&output, &result_data)?;
      eprintln!("Output written to {}", output);
    } else {
      std::io::stdout().write_all(&result_data)?;
    }
  } else {
    // Worker mode: connect to CorTeX dispatcher
    eprintln!(
      "Starting CorTeX worker '{}' (pool_size={}, profile={})",
      cli.service, cli.pool_size, cli.profile
    );
    eprintln!("  source: {}", cli.source_address);
    eprintln!("  sink:   {}", cli.sink_address);

    worker.start(cli.limit)?;
  }

  Ok(())
}
