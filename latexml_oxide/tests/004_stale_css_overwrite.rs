//! Regression for GitHub #312: a stale/empty `LaTeXML.css` / `ltx-article.css`
//! already sitting in the destination must be OVERWRITTEN with the bundled
//! stylesheet, not left as-is.
//!
//! The reporter had empty `.css` files left by an earlier failed run. Because
//! the destination directory (== the source directory here) is on the resource
//! search path, `copy_resource` *found the stale destination file itself* and
//! `fs::copy`'d it onto itself — truncating it to empty — instead of writing the
//! embedded canonical CSS. The browser then loaded empty CSS and the math
//! rendered flush-left. The `path != dest` guard was a string compare that can't
//! detect the same file reached via a different path string.

use std::{path::Path, process::Command};

const HELLO_TEX: &str = "\\documentclass{article}\n\
                         \\begin{document}\n\
                         Hello World! $E=mc^2$\n\
                         \\end{document}\n";

#[test]
fn stale_css_in_dest_is_overwritten_with_bundled() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  let p = workdir.path();
  std::fs::write(p.join("hello.tex"), HELLO_TEX).expect("write hello.tex");
  // Reporter's exact setup: stale stylesheets already in the destination (==
  // source) directory, which is on the resource search path — one empty, one
  // with junk content, to catch both the truncate-to-empty and skip variants.
  std::fs::write(p.join("LaTeXML.css"), b"").expect("write empty LaTeXML.css");
  std::fs::write(p.join("ltx-article.css"), b"/* STALE LEFTOVER */").expect("write stale css");

  let output = Command::new(bin)
    .arg("hello.tex")
    .arg("--dest")
    .arg("hello.html")
    .current_dir(p)
    .output()
    .expect("spawn latexml_oxide");
  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr),
  );

  let latexml_css = std::fs::read_to_string(p.join("LaTeXML.css")).expect("read LaTeXML.css");
  let article_css =
    std::fs::read_to_string(p.join("ltx-article.css")).expect("read ltx-article.css");

  assert!(
    latexml_css.len() > 1000 && latexml_css.contains("ltx_"),
    "a stale/empty LaTeXML.css in the dest must be overwritten with the bundled \
     stylesheet, got {} bytes:\n{latexml_css}",
    latexml_css.len(),
  );
  assert!(
    !article_css.contains("STALE") && article_css.contains("ltx_"),
    "a stale ltx-article.css in the dest must be overwritten with the bundled \
     stylesheet, got:\n{article_css}",
  );
}
