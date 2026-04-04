//! Build script for latexml_package.
//!
//! 1. Checks for precompiled kernel dumps in `resources/dumps/`
//! 2. If found, generates a loader that embeds the dump via include_str!
//! 3. If missing, generates a no-op stub (engine falls back to runtime loading)
//! 4. Checks TeX Live version for staleness detection
//!
//! Real dumps are generated manually:
//!   cargo run --release --bin latexml_oxide -- --init=latex.ltx
//!   cargo run --release --bin latexml_oxide -- --init=plain.tex

use std::env;
use std::path::Path;
use std::process::Command;

const STUB: &str = r#"// No-op kernel dump (not yet generated).
// Generate with: cargo run --release --bin latexml_oxide -- --init=latex.ltx
// The engine will use classic runtime loading instead.

pub fn load_definitions() -> latexml_core::common::error::Result<()> {
  Ok(())
}
"#;

fn main() {
  let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let out_dir = env::var("OUT_DIR").unwrap();
  let engine_dir = Path::new(&manifest_dir).join("src/engine");
  let dumps_dir = Path::new(&manifest_dir).join("../resources/dumps");

  // Ensure both dump module files exist (Perl: plain_dump + latex_dump)
  for dump_name in &["latex_dump.rs", "plain_dump.rs"] {
    let dump_path = engine_dir.join(dump_name);
    if !dump_path.exists() {
      std::fs::write(&dump_path, STUB)
        .unwrap_or_else(|_| panic!("Failed to write {dump_name} stub"));
    }
    println!("cargo:rerun-if-changed=src/engine/{dump_name}");
  }

  // Generate latex_dump_loader.rs from text dump (or stub)
  let latex_dump_txt = dumps_dir.join("latex.dump.txt");
  let loader_path = Path::new(&out_dir).join("latex_dump_loader.rs");
  if latex_dump_txt.exists() {
    let abs_path = latex_dump_txt.canonicalize().unwrap();
    let loader = format!(
      r#"pub fn load_definitions() -> latexml_core::common::error::Result<()> {{
  let content = include_str!("{}");
  let count = latexml_core::dump_reader::load_from_str(content)
    .map_err(|e| latexml_core::common::error::Error::msg(e))?;
  log::info!("Loaded {{}} latex kernel definitions from precompiled dump", count);
  Ok(())
}}
"#,
      abs_path.display()
    );
    std::fs::write(&loader_path, loader)
      .unwrap_or_else(|_| panic!("Failed to write latex_dump_loader.rs"));
    println!(
      "cargo:rerun-if-changed={}",
      latex_dump_txt.display()
    );
  } else {
    std::fs::write(&loader_path, STUB)
      .unwrap_or_else(|_| panic!("Failed to write latex_dump_loader.rs stub"));
  }

  // Check TeX Live version for staleness detection
  check_texlive_version(&dumps_dir);

  println!("cargo:rerun-if-changed=build.rs");
}

fn check_texlive_version(dumps_dir: &Path) {
  let version = Command::new("kpsewhich")
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

  if let Some(version) = version {
    let version_file = dumps_dir.join("texlive.version");
    let cached = std::fs::read_to_string(&version_file).unwrap_or_default();
    if !cached.is_empty() && cached.trim() != version.trim() {
      println!(
        "cargo:warning=TeX Live version changed (cached: {}, current: {}). \
         Regenerate dumps: cargo run --release --bin latexml_oxide -- --init=latex.ltx",
        cached.trim(),
        version.trim()
      );
    }
  }
}
