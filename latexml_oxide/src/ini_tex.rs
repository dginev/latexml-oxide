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

use once_cell::sync::Lazy;
use std::path::Path;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static INIT_DEBUG: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_INIT_DEBUG").is_some());

use latexml_core::binding::content::InputDefinitionOptions;
use latexml_core::binding::content::input_definitions;
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

  // Strict Perl `iniTeX` + `DumpFile` order:
  //
  //   Core.pm L168-212 (iniTeX, default mode='Base'):
  //     initializeState('Base.pool');     # ← Step 1 below
  //     installDefinition('\jobname', ...);
  //     installDefinition('\dump', Tokens());  # no-op
  //     DumpFile($file, $dest);           # ← Step 2 below
  //
  //   TeX_Job.pool.ltxml L120-220 (DumpFile):
  //     LoadPool($name . '_bootstrap');   # ← Step 3 below
  //     $snap = ...                       # ← Step 4 below
  //     loadTeXDefinitions($name, ...)    # ← Step 5 below
  //     diff                              # ← Step 6 below
  //     write                             # ← Step 7 below
  //
  // CRITICAL: Perl iniTeX defaults to `mode='Base'`, so `Base.pool` is
  // loaded BEFORE any bootstrap. Without it, raw plain.tex / latex.ltx
  // can't expand any TeX primitive — every `\def`, `\catcode`,
  // `\let`, `\edef` etc. is undefined and we get an error cascade.
  // After Base.pool, only `<name>_bootstrap` is loaded — NEVER
  // `<name>_base`, `<name>_dump`, or `<name>_constructs`. Those
  // pollute the diff with `:locked` flags, base/constructs
  // definitions, etc. that the dump should NOT carry.

  // Step 1: Load Base.pool equivalent (Perl `initializeState('Base.pool')`).
  eprintln!("[ini_tex] Loading Base.pool (Perl `initializeState('Base.pool')`)");
  if let Err(e) = latexml_package::engine::base::load_definitions() {
    eprintln!("[ini_tex] base warning: {}", e);
  }

  // Mark this as init/dump mode so machinery elsewhere (notably
  // `tex_file_io::\\input`'s LaTeX-style brace-arg auto-load of
  // `LaTeX.pool`) skips behaviors that would corrupt the dump-build
  // — see `tex_file_io.rs` for the gate.
  state::assign_value("INI_TEX_MODE", true, Some(state::Scope::Global));

  // Clear LaTeX/expl3/AmSTeX autoload triggers and `\documentstyle`
  // installed by `tex.rs` during `prepare_session`. These triggers
  // pre-define `\makeatletter`, `\documentclass`, etc. — which then
  // poison the snapshot, causing raw `latex.ltx` at L1798
  // (`\DeclareRobustCommand\makeatletter`) to hit the "redefining"
  // branch in `\declare@robustcommand` (L1388), which calls
  // `\@latex@info{Redefining ...}` — but `\@latex@info` isn't
  // defined until L1799, triggering an undefined-CS cascade.
  //
  // Perl `Core.pm::iniTeX` defaults to `mode='Base'` for dump-build,
  // so `Base.pool` is loaded but `TeX.pool`'s autoload triggers are
  // NOT. Mirror that here by clearing them right before the snapshot.
  for trigger in &[
    // LaTeX autoload triggers (tex.rs L149-167)
    "\\documentclass",
    "\\newcommand",
    "\\renewcommand",
    "\\newenvironment",
    "\\renewenvironment",
    "\\NeedsTeXFormat",
    "\\ProvidesPackage",
    "\\RequirePackage",
    "\\ProvidesFile",
    "\\makeatletter",
    "\\makeatother",
    "\\begin",
    "\\listfiles",
    "\\nofiles",
    "\\typeout",
    "\\PassOptionsToPackage",
    // `\@load@latex@pool` itself
    "\\@load@latex@pool",
    // expl3 autoload triggers
    "\\ExplSyntaxOn",
    "\\ProvidesExplClass",
    "\\ProvidesExplPackage",
    // AmSTeX/amsmath autoload triggers
    "\\mathfrak",
    "\\mathbb",
    "\\Bbb",
    "\\theoremstyle",
    "\\numberwithin",
    "\\align",
    "\\subequations",
    "\\multline",
    "\\curraddr",
    "\\subjclass",
    // `\documentstyle` was also defined in tex.rs as a runtime macro
    "\\documentstyle",
  ] {
    state::assign_meaning(
      &latexml_core::T_CS!(*trigger),
      latexml_core::common::store::Stored::None,
      Some(state::Scope::Global),
    );
  }

  // Step 2 + 3: install \jobname / \dump (no-op), then load bootstrap.
  // (Perl Core.pm L204-207 + TeX_Job.pool.ltxml L127-129)
  let init_lower = init_file.to_ascii_lowercase();
  let is_plain_init = init_lower.contains("plain");

  if is_plain_init {
    eprintln!("[ini_tex] Loading plain_bootstrap (mirrors Perl `LoadPool('plain_bootstrap')`)");
    if let Err(e) = latexml_package::engine::plain_bootstrap::load_definitions() {
      eprintln!("[ini_tex] plain_bootstrap warning: {}", e);
    }
  } else {
    eprintln!("[ini_tex] Loading latex_bootstrap (mirrors Perl `LoadPool('latex_bootstrap')`)");
    // latex_bootstrap.rs L11 does `InnerPool!(plain_bootstrap)` itself
    // (mirrors Perl `LoadPool('plain_bootstrap')` at the top of
    // latex_bootstrap.pool.ltxml), so plain_bootstrap state is included.
    if let Err(e) = latexml_package::engine::latex_bootstrap::load_definitions() {
      eprintln!("[ini_tex] latex_bootstrap warning: {}", e);
    }
  }

  // Perl `DumpFile` L132-138: snapshot all tables AFTER the bootstrap pool.
  let snap = state::take_snapshot();
  // Re-stage as "bootstrap" so `dump_writer` finds it for let-alias
  // classification (early/late sections).
  state::stage_snapshot_value("bootstrap", snap.clone());
  let snap_size = snap.len();
  eprintln!(
    "[ini_tex] Snapshot taken at bootstrap ({} entries)",
    snap_size
  );

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
  // Set LATEXML_INIT_DEBUG=1 to keep errors visible (for debugging the
  // expl3 cascade — Perl parity target is zero errors during expl3 load).
  let init_debug = *INIT_DEBUG;
  let prev_suppress = latexml_core::common::error::set_suppress_log_output(!init_debug);

  // Suppress known expl3 loading errors at the state level too
  state::assign_value("SUPPRESS_UNDEFINED_ERRORS", !init_debug, None);
  state::assign_value("SUPPRESS_UNEXPECTED_ERRORS", !init_debug, None);

  // Lift the MAX_ERRORS cap during dump-build. Raw latex.ltx contains
  // many CSes our engine reports as errors (forward references in
  // expl3-code.tex, `\@onlypreamble` checks, autoload triggers, etc.).
  // The default 10000-error cap aborts dump-build before plain.tex's
  // `\outer\def\newread`, `\loop`, etc. land in the diff. Mirrors Perl
  // `DumpFile`'s behavior — Perl runs latex.ltx through to `\dump`
  // regardless of error count.
  state::assign_value("MAX_ERRORS", 1_000_000_i64, None);

  // Use the full filename with extension for proper file resolution
  let load_name = if ext.is_empty() {
    name.clone()
  } else {
    format!("{}.{}", name, ext)
  };
  let result = input_definitions(&load_name, InputDefinitionOptions {
    noltxml: true,
    ..InputDefinitionOptions::default()
  });
  if let Err(e) = result {
    eprintln!("[ini_tex] Warning during loading: {}", e);
  }

  // Step 2.5: For LaTeX init, also load raw `latex209.def` so its
  // LaTeX 2.09 compatibility wrappers (`\vpt`/`\ixpt`/`\xpt`/.../`\xxvpt`,
  // L351-362) land in the diff and are captured into the dump.
  //
  // Why: modern `latex.ltx` only defines the *internal* `\@vpt`/`\@xpt`/
  // etc. (digit values). The user-facing wrappers live in
  // `latex209.def` and are loaded on demand when `\documentstyle` is
  // invoked. Many arXiv-era papers — and several style files (`bbox.sty`,
  // `aaspp.sty`, ...) — assume those wrappers exist at top level (e.g.
  // `\expandafter\def\expandafter\xpt\expandafter{\xpt …}` in
  // bbox.sty:36) and silently fail in Rust under the dump-path if they
  // aren't.
  //
  // This is a deliberate Rust-side improvement over Perl LaTeXML's
  // `latex_dump.pool.ltxml`, which also omits these wrappers. Capturing
  // them at dump-build time means every dump-path run gets them
  // unconditionally, matching the behavior the bindings expect.
  //
  // Witness: hep-ph0109006 (stage 2 canvas RUST-REGRESSION, 5 ×
  // `Error:undefined:\xpt`).
  //
  // Skipped for plain init (`--init=plain.tex`): plain TeX has no
  // LaTeX 2.09 wrapper concept.
  if !is_plain_init {
    eprintln!("[ini_tex] Loading raw latex209.def to capture LaTeX 2.09 wrappers");
    let r2 = input_definitions("latex209", InputDefinitionOptions {
      extension: Some(std::borrow::Cow::Borrowed("def")),
      noltxml: true,
      noerror: true,
      ..InputDefinitionOptions::default()
    });
    if let Err(e) = r2 {
      eprintln!("[ini_tex] latex209.def load warning: {}", e);
    }
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

  // Step 4: Write the dump.
  // Default: write text dump to resources/dumps/ for build.rs embedding.
  // With --dest: write to the specified path.
  let (dest, is_text_dump) = match destination {
    Some(d) if d.ends_with(".rs") => (d.to_string(), false),
    Some(d) => (d.to_string(), true),
    None => {
      let dump_name = if name.contains("latex") {
        "latex.dump.txt"
      } else {
        "plain.dump.txt"
      };
      let dump_dir = "resources/dumps";
      std::fs::create_dir_all(dump_dir)
        .map_err(|e| format!("Failed to create {}: {}", dump_dir, e))?;
      (format!("{}/{}", dump_dir, dump_name), true)
    },
  };

  if is_text_dump {
    // Write text format (loaded at runtime via dump_reader::load_from_str)
    let write_count = latexml_core::dump_writer::write_dump(Path::new(&dest), &diff)?;
    // Save TeX Live version for staleness detection
    save_texlive_version();
    eprintln!("[ini_tex] Wrote {} text entries to {}", write_count, dest);
    eprintln!("Format dump complete: {} entries written", write_count);
    Ok(write_count)
  } else {
    // Write compiled Rust source (legacy format)
    let tmp = format!("{}.tmp", dest);
    let _write_count = latexml_core::dump_writer::write_dump(Path::new(&tmp), &diff)?;
    let rs_count = latexml_core::dump_codegen::generate_rs(Path::new(&tmp), Path::new(&dest))?;
    let _ = std::fs::remove_file(&tmp);
    eprintln!(
      "[ini_tex] Generated {} Rust definitions to {}",
      rs_count, dest
    );
    eprintln!("Format dump complete: {} entries written", rs_count);
    Ok(rs_count)
  }
}

/// Generate a compiled Rust module from a dump file.
/// Reads the text dump and produces a .rs file with direct state assignment calls.
pub fn codegen_from_dump(dump_path: &str, output_path: &str) -> Result<usize, String> {
  eprintln!("[ini_tex] Generating Rust module from {}", dump_path);
  let count =
    latexml_core::dump_codegen::generate_rs(Path::new(dump_path), Path::new(output_path))?;
  eprintln!("[ini_tex] Generated {} entries to {}", count, output_path);
  Ok(count)
}

fn save_texlive_version() {
  let version = std::process::Command::new("kpsewhich")
    .arg("--version")
    .output()
    .ok()
    .and_then(|o| {
      if o.status.success() {
        String::from_utf8(o.stdout).ok()
      } else {
        None
      }
    });
  if let Some(v) = version {
    let _ = std::fs::write("resources/dumps/texlive.version", v.trim());
  }
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
