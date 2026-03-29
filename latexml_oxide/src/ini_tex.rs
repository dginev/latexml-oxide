//! Format dump mode — Rust equivalent of Perl's iniTeX + DumpFile.
//!
//! Usage: `latexml_oxide --init=latex.ltx --dest=latex_dump.oxide`
//!
//! Follows Perl's make formats scaffold (Makefile.PL + Core.pm::iniTeX):
//! 1. Initialize the engine (load pools)
//! 2. Take a snapshot of the state
//! 3. Process the init file (e.g., latex.ltx) as raw TeX
//! 4. Compute the diff (what changed)
//! 5. Write the dump file with changed entries
//!
//! The resulting dump can be loaded at runtime to skip re-processing
//! the LaTeX kernel on every test run.

use std::path::Path;

use latexml_core::binding::content::input_definitions;
use latexml_core::binding::content::InputDefinitionOptions;
use latexml_core::state;

use crate::converter::Converter;

/// Process an init file and write a format dump.
/// Perl equivalent: Core.pm::iniTeX → TeX_Job.pool.ltxml::DumpFile
pub fn dump_format(
  _converter: &mut Converter,
  init_file: &str,
  destination: Option<&str>,
) -> Result<usize, String> {
  eprintln!("[ini_tex] Dumping format from {}", init_file);

  // Step 1: Take a snapshot of the state BEFORE processing.
  // Perl: DumpFile takes snapshot, then loads file, then diffs.
  let snap = state::take_snapshot();
  let snap_size = snap.len();
  eprintln!("[ini_tex] Pre-dump snapshot: {} entries", snap_size);

  // Step 2: Process the init file as raw TeX definitions.
  // Perl: loadTeXDefinitions($name, $path, type => $type)
  // This digests the file through the engine, creating definitions.
  let (_, name, ext) = split_path(init_file);
  eprintln!("[ini_tex] Loading {} (ext: {})", name, ext);

  // Lift the token limit for format dumps — expl3-code.tex alone uses ~5M tokens.
  let saved_limit = latexml_core::gullet::set_token_limit(None);

  // In init mode, suppress error/warning output during format loading.
  // Raw latex.ltx redefines commands already in the compiled engine ("already defined"),
  // and expl3-code.tex has forward references that produce transient errors.
  // All these errors are benign — the dump captures the final correct state.
  let prev_suppress = latexml_core::common::error::set_suppress_log_output(true);

  // Suppress known expl3 loading errors at the state level too
  state::assign_value("SUPPRESS_UNDEFINED_ERRORS", true, None);
  state::assign_value("SUPPRESS_UNEXPECTED_ERRORS", true, None);

  // Use the full filename with extension for proper file resolution
  let load_name = if ext.is_empty() { name.clone() } else { format!("{}.{}", name, ext) };
  let result = input_definitions(
    &load_name,
    InputDefinitionOptions {
      noltxml: true,
      ..InputDefinitionOptions::default()
    },
  );
  if let Err(e) = result {
    eprintln!("[ini_tex] Warning during loading: {}", e);
  }

  // Restore limits and suppression
  latexml_core::gullet::restore_token_limit(saved_limit);
  latexml_core::common::error::set_suppress_log_output(prev_suppress);
  state::assign_value("SUPPRESS_UNDEFINED_ERRORS", false, None);
  state::assign_value("SUPPRESS_UNEXPECTED_ERRORS", false, None);

  // Step 3: Compute the diff — only entries that changed.
  let diff = state::diff_snapshot(&snap);
  eprintln!(
    "[ini_tex] Post-load diff: {} changed entries (from {} pre-dump)",
    diff.len(),
    snap_size
  );

  // Step 4: Write the dump as native Rust source.
  let default_dest = "latexml_package/src/engine/latex_dump.rs";
  let dest = destination.unwrap_or(default_dest);

  // Write to temp intermediate format, then codegen to .rs
  let tmp = format!("{}.tmp", dest);
  let write_count = latexml_core::dump_writer::write_dump(Path::new(&tmp), &diff)?;
  let rs_count = latexml_core::dump_codegen::generate_rs(Path::new(&tmp), Path::new(dest))?;
  let _ = std::fs::remove_file(&tmp);

  eprintln!("[ini_tex] Generated {} Rust definitions to {}", rs_count, dest);
  eprintln!("Format dump complete: {} entries written", rs_count);

  Ok(rs_count)
}

/// Generate a compiled Rust module from a dump file.
/// Reads the text dump and produces a .rs file with direct state assignment calls.
pub fn codegen_from_dump(dump_path: &str, output_path: &str) -> Result<usize, String> {
  eprintln!("[ini_tex] Generating Rust module from {}", dump_path);
  let count = latexml_core::dump_codegen::generate_rs(
    Path::new(dump_path),
    Path::new(output_path),
  )?;
  eprintln!("[ini_tex] Generated {} entries to {}", count, output_path);
  Ok(count)
}

fn split_path(path: &str) -> (String, String, String) {
  let p = Path::new(path);
  let dir = p
    .parent()
    .map(|d| d.to_string_lossy().to_string())
    .unwrap_or_default();
  let stem = p
    .file_stem()
    .map(|s| s.to_string_lossy().to_string())
    .unwrap_or_default();
  let ext = p
    .extension()
    .map(|e| e.to_string_lossy().to_string())
    .unwrap_or_default();
  (dir, stem, ext)
}
