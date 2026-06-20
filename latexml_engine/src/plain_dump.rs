//! Runtime loader for `plain.YYYY.dump.txt` — Perl-parity for
//! `LoadFormat('plain')`.
//!
//! Mirrors the generated `latex_dump_loader.rs` but is hand-written to
//! keep the dump pipeline surgical: build.rs emits exactly one loader
//! (for the LaTeX dump) and this file is the parallel one for Plain.
//!
//! Load order (mirrors Perl `TeX.pool.ltxml: LoadFormat('plain')`):
//!   plain_bootstrap → plain_base → **plain_dump (this)** → plain_constructs
//!
//! Resolution order (matches `latex_dump_loader.rs`):
//!   0. `$LATEXML_NODUMP` → skip (Perl `Package.pm` `LoadFormat` parity)
//!   1. `$LATEXML_PLAIN_DUMP_PATH` (explicit full path)
//!   2. `$LATEXML_DUMP_DIR/plain.YYYY.dump.txt` (best year-match in dir; else most-recent)
//!   3. `<exe_dir>/../resources/dumps/plain.YYYY.dump.txt` (installed layout)
//!   4. Sibling-of-exe `plain.YYYY.dump.txt`
//!   5. Dev-tree `$CARGO_MANIFEST_DIR/../resources/dumps/plain.YYYY.dump.txt`
//!   6. **Embedded dump** (compile-time-bundled snapshot)
//!
//! Year selection: prefer ambient TeXLive year (via
//! [`crate::dump_paths::detect_ambient_texlive_year`]); fall back to the
//! most-recent year present at each step.

use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;

use crate::prelude::*;

const DEV_DUMPS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/dumps");

// Process-once cached env vars. Hoisted out of the runtime path because
// `getenv` is unsafe under high-volume concurrent reads (test harness
// threads) — see WISDOM #56 in user memory and `gullet::TRACE_GROUP_END`
// for the underlying cause.
static NODUMP: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_NODUMP").is_some());
static PLAIN_DUMP_PATH: Lazy<Option<String>> =
  Lazy::new(|| std::env::var("LATEXML_PLAIN_DUMP_PATH").ok());
static DUMP_DIR: Lazy<Option<String>> = Lazy::new(|| std::env::var("LATEXML_DUMP_DIR").ok());

pub fn load_definitions() -> Result<()> {
  if *NODUMP {
    Info!(
      "plain_dump",
      "nodump",
      "LATEXML_NODUMP set — skipping dump, engine will reconstruct \
       plain.tex state from _base pool (slower, Perl-parity)"
    );
    return Ok(());
  }
  let prefer = crate::dump_paths::detect_ambient_texlive_year();
  let (content, source_label) = if let Some((path, _year)) = resolve_dump_path(prefer) {
    match std::fs::read_to_string(&path) {
      Ok(c) => (c, path.display().to_string()),
      Err(e) => {
        Warn!(
          "plain_dump",
          "read",
          s!("failed to read {}: {}", path.display(), e)
        );
        return Ok(());
      },
    }
  } else if let Some(embedded) = crate::embedded_dumps::embedded_plain_dump(prefer) {
    let year = crate::embedded_dumps::embedded_year(prefer).unwrap_or(0);
    Info!(
      "plain_dump",
      "embedded",
      s!("using embedded TL{} dump — no on-disk dump found", year)
    );
    (embedded.to_string(), format!("<embedded TL{}>", year))
  } else {
    Info!(
      "plain_dump",
      "missing",
      "no dump found (checked $LATEXML_PLAIN_DUMP_PATH, $LATEXML_DUMP_DIR, \
       exe-relative, dev-tree path, and embedded fallback); \
       run `latexml_oxide --init=plain.tex` to generate"
    );
    return Ok(());
  };
  // Single load message: `dump_reader:loaded` now names the real `source_label`
  // (disk path in dev, `<embedded TLyyyy>` in the shipped binary) and carries the
  // skipped/errors detail — so we no longer emit a second, redundant
  // `plain_dump:loaded` line for the same load.
  let _count = dump_reader::load_from_str_labeled(&content, &source_label)
    .map_err(|e: String| -> Error { e.into() })?;
  Ok(())
}

fn resolve_dump_path(prefer: Option<u32>) -> Option<(PathBuf, u32)> {
  // 1. Explicit full path.
  if let Some(p) = PLAIN_DUMP_PATH.as_deref() {
    let pb = PathBuf::from(p);
    if pb.is_file() {
      let year = pb
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| crate::dump_paths::parse_year_from_dump_filename(n, "plain"))
        .unwrap_or(0);
      return Some((pb, year));
    }
  }
  // 2. Directory override — pick versioned dump in that directory.
  if let Some(dir) = DUMP_DIR.as_deref()
    && let Some(found) =
      crate::dump_paths::resolve_versioned_in_dir(Path::new(dir), "plain", prefer)
  {
    return Some(found);
  }
  // 3. Installed layout: <exe_dir>/../resources/dumps/.
  if let Ok(exe) = std::env::current_exe()
    && let Some(exe_dir) = exe.parent()
  {
    let installed = exe_dir.join("../resources/dumps");
    if let Some(found) = crate::dump_paths::resolve_versioned_in_dir(&installed, "plain", prefer) {
      return Some(found);
    }
    // Sibling-of-exe (test binaries at target/<profile>/deps/).
    if let Some(found) = crate::dump_paths::resolve_versioned_in_dir(exe_dir, "plain", prefer) {
      return Some(found);
    }
  }
  // 4. Dev-tree path baked in at compile time.
  let dev = Path::new(DEV_DUMPS_DIR);
  if dev.is_dir()
    && let Some(found) = crate::dump_paths::resolve_versioned_in_dir(dev, "plain", prefer)
  {
    return Some(found);
  }
  None
}

/// True when any plain.YYYY.dump.txt is reachable through any of the
/// runtime-resolution paths or the embedded fallback. Used by `tex.rs`
/// to decide whether `LoadFormat('plain')` takes the dump branch or the
/// _base reconstruction branch.
pub fn plain_dump_available() -> bool {
  if *NODUMP {
    return false;
  }
  let prefer = crate::dump_paths::detect_ambient_texlive_year();
  resolve_dump_path(prefer).is_some()
    || crate::embedded_dumps::embedded_plain_dump(prefer).is_some()
}
