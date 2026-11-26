//! Build script for latexml_post: the release-only bundled `ar5iv.css`.
//!
//! When `resources/ar5iv-css/ar5iv.min.css` is present (placed by `tools/fetch_ar5iv_css.sh` from the release pinned
//! in `tools/ar5iv-css.pin`, gitignored), it is staged into `$OUT_DIR` behind an MIT banner naming its ar5iv-css
//! release, and `embedded_ar5iv_css.rs` makes `AR5IV_CSS` `Some` of it; otherwise `None`. The build never fetches
//! anything — the same contract as the kernel dumps in `latexml_engine/build.rs`.

use std::{env, fs, path::Path};

#[path = "src/build_rerun.rs"]
mod build_rerun;

fn main() {
  let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let out_dir = env::var("OUT_DIR").unwrap();
  let resources = Path::new(&manifest_dir).join("../resources");
  let dir = resources.join("ar5iv-css");
  let css = dir.join("ar5iv.min.css");
  let stamp = dir.join("ar5iv-css.version");
  let in_workspace = Path::new(&manifest_dir).join("../Cargo.toml").exists();

  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=src/build_rerun.rs");
  for p in build_rerun::ar5iv_css_rerun_paths(&resources, &[&dir, &css, &stamp], in_workspace) {
    println!("cargo:rerun-if-changed={}", p.display());
  }

  const DOC: &str = "/// The bundled `ar5iv.css` (release builds only).\n";
  let code = match fs::read_to_string(&css) {
    Ok(text) if in_workspace => {
      // The stamp is "<tag> <commit>" (tools/fetch_ar5iv_css.sh).
      let stamp = fs::read_to_string(&stamp).unwrap_or_default();
      let mut words = stamp.split_whitespace();
      let release = match (words.next(), words.next()) {
        (Some(tag), Some(commit)) => format!("{tag} ({})", commit.get(..12).unwrap_or(commit)),
        (Some(tag), None) => tag.to_string(),
        _ => "(unknown release)".to_string(),
      };
      let staged = Path::new(&out_dir).join("ar5iv.css");
      let banner = format!(
        "/*! ar5iv-css {release} | MIT License | Copyright (c) 2021-2027 Deyan Ginev | github.com/dginev/ar5iv-css */\n"
      );
      fs::write(&staged, banner + &text)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", staged.display()));
      format!(
        "{DOC}pub const AR5IV_CSS: Option<&str> = Some(include_str!({:?}));\n",
        staged.display().to_string()
      )
    },
    _ => format!("{DOC}pub const AR5IV_CSS: Option<&str> = None;\n"),
  };
  let generated = Path::new(&out_dir).join("embedded_ar5iv_css.rs");
  fs::write(&generated, code)
    .unwrap_or_else(|e| panic!("Failed to write {}: {e}", generated.display()));
}
