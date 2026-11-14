//! Build-time probe for the optional PDF→SVG converters.
//!
//! The vector-SVG tests need `mutool` or `pdftocairo`, which are optional at runtime (the
//! raster path covers their absence). Where neither spawns, the tests are `ignore`d through the
//! `svg_converter_on_path` cfg, so the skip shows in the test summary instead of a silent green.

use std::process::Command;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-env-changed=PATH");
  println!("cargo:rustc-check-cfg=cfg(svg_converter_on_path)");
  // Spawning is the signal, not the exit status: both tools exit non-zero on `--version`.
  let found = ["mutool", "pdftocairo"]
    .iter()
    .any(|tool| Command::new(tool).arg("--version").output().is_ok());
  if found {
    println!("cargo:rustc-cfg=svg_converter_on_path");
  }
}
