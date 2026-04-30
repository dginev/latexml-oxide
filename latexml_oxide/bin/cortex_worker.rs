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

/// Per-process allocator: mimalloc avoids glibc's arena-mutex contention
/// which dominates multi-process workloads (seen as 3.4x slowdown at 16 workers).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
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
  #[arg(long, default_value = "60")]
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
  preloads:           Vec<String>,
  pmml:               bool,
  mathtex:            bool,
  noinvisibletimes:   bool,
  nodefaultresources: bool,
  timeout:            u64,
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
  service:        String,
  source_address: String,
  sink_address:   String,
  identity:       String,
  msg_size:       usize,
  threads:        usize,
  profile:        ConversionProfile,
  verbosity:      i32,
}

impl LatexmlWorker {
  /// Run the conversion pipeline on an input ZIP archive.
  /// Returns the path to the output ZIP file.
  fn convert_archive(&self, input_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    // Per-document timeout: two-layer guard.
    //   1. Watchdog thread forcibly aborts after profile.timeout seconds. Catches tight native
    //      loops (Marpa, libxml2, libxslt) that never return to the Rust digestion loop.
    //   2. Cooperative stomach::set_timeout gives a graceful Err(Fatal) for the common case where
    //      digestion polls check_timeout.
    // Watchdog cancels automatically on drop at end of this function.
    let _watchdog = latexml_core::watchdog::Watchdog::new(self.profile.timeout);
    if self.profile.timeout > 0 {
      latexml_core::stomach::set_timeout(self.profile.timeout);
    }

    // 1. Unpack the input archive
    let (tempdir, main_tex) = unpack_archive(input_path)?;
    let source_dir = tempdir.path().to_string_lossy().to_string();

    // 2. Set up the converter with the profile
    let mut preloads = vec!["TeX.pool".to_string()];
    preloads.extend(self.profile.preloads.iter().cloned());

    let opts = Config {
      verbosity:               self.verbosity,
      format:                  OutputFormat::HTML5,
      whatsin:                 DataSize::Document,
      whatsout:                DataSize::Document,
      preamble:                None,
      postamble:               None,
      mode:                    None,
      bindings_dispatch:       Some(Rc::new(latexml_package::dispatch)),
      extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
      preload:                 Some(self.profile.preloads.clone()),
      search_paths:            Some(vec![source_dir.clone()]),
      include_comments:        Some(false),
      nomathparse:             None,
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

    // 4. Create destination directory for images/resources Perl LaTeXML.pm L200-205: derive HTML
    //    name from source TeX name e.g., 9256.tex → 9256.html (format "html5" → extension "html")
    let source_name = Path::new(&main_tex)
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("document");
    let html_filename = format!("{}.html", source_name);
    let dest_dir = TempDir::new()?;
    let dest_html = dest_dir.path().join(&html_filename);
    let dest_html_str = dest_html.to_string_lossy().to_string();

    // 5. Post-process: MathML + XSLT (matching CorTeX tex_to_html settings)
    let html = latexml::post::run_post_processing(&xml, &latexml::post::PostOptions {
      pmml:                      self.profile.pmml,
      cmml:                      true, // CorTeX produces both pmml and cmml
      keep_xmath:                false,
      stylesheet:                Some("resources/XSLT/LaTeXML-html5.xsl"),
      destination:               Some(&dest_html_str),
      source_directory:          Some(&source_dir),
      nodefaultresources:        self.profile.nodefaultresources,
      css_files:                 &[],
      js_files:                  &[],
      noinvisibletimes:          self.profile.noinvisibletimes,
      mathtex:                   self.profile.mathtex,
      navigationtoc:             None,
      split:                     false,
      split_xpath:               None,
      split_naming:              None,
      xslt_parameters:           &[],
      graphics_svg_threshold_kb: 0,
    });

    // 6. Get log and status (Perl: status line is last line of log)
    let status_str = format!("Status:conversion:{}", response.status_code);
    let log = format!("{}\n{}", response.log, status_str);

    // 7. Pack output ZIP: HTML (named after source) + images + log + status
    let output_path =
      std::env::temp_dir().join(format!("cortex_output_{}.zip", std::process::id()));
    pack_output_zip_with_resources(
      &output_path,
      &html_filename,
      &html,
      &log,
      &status_str,
      dest_dir.path(),
    )?;

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

  fn message_size(&self) -> usize { self.msg_size }

  fn get_service(&self) -> &str { &self.service }

  fn get_source_address(&self) -> Cow<'_, str> { Cow::Borrowed(&self.source_address) }

  fn get_sink_address(&self) -> Cow<'_, str> { Cow::Borrowed(&self.sink_address) }

  fn pool_size(&self) -> usize { self.threads }

  fn set_identity(&mut self, identity: String) { self.identity = identity; }

  fn get_identity(&self) -> &str { &self.identity }
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

/// Faithful port of Perl Pack.pm detect_source / arXiv::FileGuess.
/// Identical to the find_main_tex in latexml_oxide.rs.
fn find_main_tex(dir: &Path) -> Result<String, Box<dyn Error>> {
  use once_cell::sync::Lazy;
  use regex::Regex;

  // Phase I.1: Check 00README.json (2025 arXiv format)
  if let Some(filename) = parse_readme_json(dir) {
    let main_path = dir.join(&filename);
    if main_path.exists() {
      return Ok(main_path.to_string_lossy().to_string());
    }
  }

  // Phase I.1.2: Check 00README.XXX (legacy arXiv format)
  let readme_xxx = dir.join("00README.XXX");
  if readme_xxx.exists() {
    if let Ok(content) = fs::read_to_string(&readme_xxx) {
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
    if let Ok(entries) = fs::read_dir(dir) {
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
          // arxiv 0908.4110 ships a bare "birkhoffproofrev1" LaTeX source.
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
    return Err("No .tex files found in archive".into());
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
    let Ok(raw) = fs::read(tex_file) else {
      continue;
    };
    let content = String::from_utf8_lossy(&raw);
    let mut maybe_tex = false;
    let mut maybe_tex_priority = false;
    let mut maybe_tex_priority2 = false;
    let mut determined = false;

    for (lineno, raw_line) in content.lines().enumerate() {
      let lineno1 = lineno + 1;
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

  // Remove vetoed files
  for v in &vetoed {
    likelihood.remove(v);
  }

  // Filter to score > 0, sort by score descending
  let mut candidates: Vec<PathBuf> = likelihood
    .keys()
    .filter(|f| likelihood[*f] > 0.0)
    .cloned()
    .collect();
  candidates.sort_by(|a, b| likelihood[b].partial_cmp(&likelihood[a]).unwrap());

  if candidates.is_empty() {
    return Err("No viable .tex files found in archive".into());
  }

  // Keep only max-scoring candidates
  let max_score = likelihood[&candidates[0]];
  candidates.retain(|f| (likelihood[f] - max_score).abs() < f32::EPSILON);

  // Heuristic 1: prefer shallowest path
  if candidates.len() > 1 {
    let min_depth = candidates
      .iter()
      .map(|f| f.strip_prefix(dir).unwrap_or(f).components().count())
      .min()
      .unwrap_or(0);
    candidates.retain(|f| f.strip_prefix(dir).unwrap_or(f).components().count() == min_depth);
  }

  // Heuristic 2: prefer files with PDF-like \includegraphics
  if candidates.len() > 1 {
    let pdf_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        fs::read(f).ok().is_some_and(|raw| {
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

  // Heuristic 3: prefer files with a matching .bbl file
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

  // Heuristic 4: prefer common main file names
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

  // Final tiebreaker: lexicographic order
  candidates.sort();
  Ok(candidates[0].to_string_lossy().to_string())
}

/// Parse 00README.json for toplevel source filename.
fn parse_readme_json(dir: &Path) -> Option<String> {
  let content = fs::read_to_string(dir.join("00README.json")).ok()?;
  let sources_start = content.find("\"sources\"")?;
  let rest = &content[sources_start..];
  let arr_start = rest.find('[')?;
  let arr_end = rest.find(']')?;
  let arr = &rest[arr_start + 1..arr_end];
  for obj_str in arr.split('}') {
    if !obj_str.contains("\"toplevel\"") {
      continue;
    }
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
          '\\' => continue,
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

fn pack_output_zip_with_resources(
  output_path: &Path,
  html_filename: &str,
  html: &str,
  log: &str,
  status: &str,
  resource_dir: &Path,
) -> Result<(), Box<dyn Error>> {
  let file = File::create(output_path)?;
  let mut zip = zip::ZipWriter::new(file);
  let options =
    zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

  // HTML file named after the source TeX (Perl LaTeXML.pm L200-205)
  zip.start_file(html_filename, options)?;
  zip.write_all(html.as_bytes())?;

  // Add all resource files (images, etc.) from the destination directory
  if resource_dir.exists() {
    add_dir_to_zip(&mut zip, resource_dir, resource_dir, &options)?;
  }

  zip.start_file("cortex.log", options)?;
  zip.write_all(log.as_bytes())?;

  zip.start_file("status", options)?;
  zip.write_all(status.as_bytes())?;

  zip.finish()?;
  Ok(())
}

/// Recursively add files from a directory to a ZIP archive.
/// Skips the output.html (already added separately).
fn add_dir_to_zip(
  zip: &mut zip::ZipWriter<File>,
  dir: &Path,
  base: &Path,
  options: &zip::write::SimpleFileOptions,
) -> Result<(), Box<dyn Error>> {
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    let rel = path.strip_prefix(base).unwrap_or(&path);
    let name = rel.to_string_lossy().to_string();

    if path.is_dir() {
      add_dir_to_zip(zip, &path, base, options)?;
    } else if !name.ends_with(".html") {
      // Skip the HTML file — it's already added separately
      zip.start_file(&name, *options)?;
      let mut f = File::open(&path)?;
      std::io::copy(&mut f, zip)?;
    }
  }
  Ok(())
}

// --- Main ---

fn main() -> Result<(), Box<dyn Error>> {
  // Run all work on a worker thread with a 256 MB stack so deeply
  // nested math trees (XMApp(op, [XMApp(...)]) chains in grammar-
  // ambiguous papers — sandbox 0711.4787 et al, #17) don't overflow
  // the OS-default 8 MB main-thread stack during finalize/post-
  // processing. Validated: 0711.4787 converts cleanly under
  // `ulimit -s unlimited` (959 maths, Status:conversion:1).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(|| real_main().map_err(|e| e.to_string()))
    .expect("spawn worker thread")
    .join()
    .expect("worker thread panicked")
    .map_err(|s| s.into())
}

fn real_main() -> Result<(), Box<dyn Error>> {
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
    },
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
