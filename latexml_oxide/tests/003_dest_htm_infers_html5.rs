//! Regression pin: a `.htm` destination extension infers `--format=html5`,
//! exactly like `.html`.
//!
//! `bin/latexml_oxide.rs` maps `"html" | "htm" => "html5"` (Perl Config.pm
//! L435), lowercased so `.HTM` works too — so `--dest=index.htm` with no
//! explicit `--format` must produce a post-processed HTML5 page (DOCTYPE +
//! `<link>`ed, copied `LaTeXML.css`), NOT the raw-XML default. This pins the
//! `.htm` half, which every other CLI test exercises only via `.html`; it is
//! the exact invocation shape from GitHub #312 (`--dest=index.htm`).

use std::{path::Path, process::Command};

const HELLO_TEX: &str = "\\documentclass{article}\n\
                         \\begin{document}\n\
                         Hello World!\n\
                         \\end{document}\n";

#[test]
fn dest_htm_extension_infers_html5_and_copies_css() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("hello.tex"), HELLO_TEX).expect("write hello.tex");

  // Nasser's invocation shape: `.htm` destination, NO explicit --format.
  let output = Command::new(bin)
    .arg("hello.tex")
    .arg("--dest")
    .arg("hello.htm")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr),
  );

  // The `.htm` extension must have inferred html5 → a post-processed HTML page,
  // not the raw-XML fallback.
  let html = std::fs::read_to_string(workdir.path().join("hello.htm")).expect("read hello.htm");
  assert!(
    html.contains("<!DOCTYPE html>"),
    "a .htm destination should infer html5 (DOCTYPE html), got:\n{html}",
  );
  assert!(
    html.contains("LaTeXML.css"),
    "html5 output should link LaTeXML.css, got:\n{html}",
  );

  // ...and the html5 pipeline copies the bundled default stylesheet next to it.
  assert!(
    workdir.path().join("LaTeXML.css").is_file(),
    "expected LaTeXML.css copied next to hello.htm (html5 resource copy)",
  );
}
