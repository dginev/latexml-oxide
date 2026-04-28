//! Runtime loader for `resources/dumps/plain.dump.txt` — Perl-parity for
//! `LoadFormat('plain')`. Mirrors `latex_dump.rs` (which pulls a generated
//! loader from `OUT_DIR/latex_dump_loader.rs`) but is hand-written here to
//! keep the dump pipeline surgical: build.rs emits exactly one loader for
//! the LaTeX dump, and this file is the parallel one for the Plain dump.
//!
//! Load order (mirrors Perl `TeX.pool.ltxml: LoadFormat('plain')`):
//!   plain_bootstrap → plain_base → **plain_dump (this)** → plain_constructs
//!
//! Resolution order matches `latex_dump_loader.rs`:
//!   0. `$LATEXML_NODUMP` → skip (Perl `Package.pm` `LoadFormat` parity)
//!   1. `$LATEXML_PLAIN_DUMP_PATH` (explicit override, full path)
//!   2. `$LATEXML_DUMP_DIR/plain.dump.txt` (directory override)
//!   3. `<exe_dir>/../resources/dumps/plain.dump.txt` (installed layout)
//!   4. dev-tree path (compile-time `CARGO_MANIFEST_DIR/../resources/dumps/plain.dump.txt`)
//!
//! If no dump is found we log at info and return `Ok(())` — the engine
//! falls back to whatever stubs `plain_base.rs` already installed. This
//! matches Perl's `LoadFormat` graceful-fallback behavior.

use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

const DEV_PATH: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../resources/dumps/plain.dump.txt"
);

// Process-once cached env vars. Hoisted out of the runtime path because
// `getenv` is unsafe under high-volume concurrent reads (test harness
// threads) — see WISDOM #56 in user memory and `gullet::TRACE_GROUP_END`
// for the underlying cause.
static NODUMP: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_NODUMP").is_some());
static PLAIN_DUMP_PATH: Lazy<Option<String>> =
  Lazy::new(|| std::env::var("LATEXML_PLAIN_DUMP_PATH").ok());
static DUMP_DIR: Lazy<Option<String>> = Lazy::new(|| std::env::var("LATEXML_DUMP_DIR").ok());

pub fn load_definitions() -> latexml_core::common::error::Result<()> {
  if *NODUMP {
    log::info!(
      "[plain_dump] LATEXML_NODUMP set — skipping dump, engine will reconstruct \
       plain.tex state from _base pool (slower, Perl-parity)"
    );
    return Ok(());
  }
  let Some(path) = resolve_dump_path() else {
    log::info!(
      "[plain_dump] no dump found (checked $LATEXML_PLAIN_DUMP_PATH, $LATEXML_DUMP_DIR, \
       exe-relative, and dev-tree path); run `latexml_oxide --init=plain.tex` to generate"
    );
    return Ok(());
  };
  let content = match std::fs::read_to_string(&path) {
    Ok(c) => c,
    Err(e) => {
      log::warn!("[plain_dump] failed to read {}: {}", path.display(), e);
      return Ok(());
    },
  };
  let count = latexml_core::dump_reader::load_from_str_plain(&content)
    .map_err(|e: String| -> latexml_core::common::error::Error { e.into() })?;
  log::info!(
    "[plain_dump] loaded {} entries from {}",
    count,
    path.display()
  );
  Ok(())
}

fn resolve_dump_path() -> Option<PathBuf> {
  if let Some(p) = PLAIN_DUMP_PATH.as_deref() {
    let pb = PathBuf::from(p);
    if pb.is_file() {
      return Some(pb);
    }
  }
  if let Some(dir) = DUMP_DIR.as_deref() {
    let pb = Path::new(dir).join("plain.dump.txt");
    if pb.is_file() {
      return Some(pb);
    }
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(exe_dir) = exe.parent() {
      let installed = exe_dir.join("../resources/dumps/plain.dump.txt");
      if installed.is_file() {
        return Some(installed);
      }
      let sibling = exe_dir.join("plain.dump.txt");
      if sibling.is_file() {
        return Some(sibling);
      }
    }
  }
  let dev = PathBuf::from(DEV_PATH);
  if dev.is_file() {
    return Some(dev);
  }
  None
}
