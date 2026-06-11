//! Regression test: CLI `--css` resources are searched on `--path` and
//! COPIED into the destination directory.
//!
//! Before the fix, `--css=foo.css --path=DIR` emitted a `<link>` to `foo.css`
//! in the HTML but never searched `--path` for the file nor copied it, so the
//! page rendered unstyled (the file simply wasn't there next to the HTML).
//!
//! `--nodefaultresources` is orthogonal and must NOT suppress CLI-specified
//! resources — it only drops the bundled defaults (`LaTeXML.css` /
//! `ltx-article.css`). This test pins both halves: with
//! `--nodefaultresources` set, the custom `--css` file is still copied, while
//! the bundled defaults are not.
//!
//! Faithful to Perl `LaTeXML::Post::XSLT::process` L71-78 (the CSS/JAVASCRIPT
//! param copy, which sits OUTSIDE the `noresources` guard).

use std::{path::Path, process::Command};

const HELLO_TEX: &str = "\\documentclass{article}\n\
                         \\begin{document}\n\
                         Hello World!\n\
                         \\end{document}\n";

const CUSTOM_CSS: &str = "/* oxide-test-marker */\nbody { color: rebeccapurple; }\n";

#[test]
fn cli_css_is_searched_on_path_and_copied_even_with_nodefaultresources() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  // The custom CSS lives in a subdirectory reachable ONLY via `--path`.
  let cssdir = workdir.path().join("styles");
  std::fs::create_dir(&cssdir).expect("mkdir styles");
  std::fs::write(cssdir.join("mystyle.css"), CUSTOM_CSS).expect("write mystyle.css");

  let tex_path = workdir.path().join("hello.tex");
  let html_path = workdir.path().join("hello.html");
  std::fs::write(&tex_path, HELLO_TEX).expect("write hello.tex");

  let output = Command::new(bin)
    .arg(tex_path.file_name().unwrap())
    .arg("--dest")
    .arg(html_path.file_name().unwrap())
    .arg("--format")
    .arg("html5")
    .arg("--css")
    .arg("mystyle.css")
    .arg("--path")
    .arg(&cssdir)
    .arg("--nodefaultresources")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr),
  );

  // The HTML links the custom CSS.
  let html = std::fs::read_to_string(&html_path).expect("read hello.html");
  assert!(
    html.contains("mystyle.css"),
    "expected mystyle.css link in HTML, got:\n{html}",
  );

  // THE FIX: the file is searched on `--path` and copied into the destination
  // directory (next to the HTML), even though `--nodefaultresources` is set.
  let copied = workdir.path().join("mystyle.css");
  assert!(
    copied.is_file(),
    "expected mystyle.css copied next to hello.html (the --css/--path copy), \
     missing at {}\nstderr:\n{}",
    copied.display(),
    String::from_utf8_lossy(&output.stderr),
  );
  assert_eq!(
    std::fs::read_to_string(&copied).expect("read copied css"),
    CUSTOM_CSS,
    "copied CSS content should match the source on --path",
  );

  // The custom file came from `--path`, NOT the embedded table, so there must
  // be no `missing_file` warning for it.
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    !stderr.contains("missing_file") || !stderr.contains("mystyle.css"),
    "unexpected missing_file warning for mystyle.css:\n{stderr}",
  );

  // `--nodefaultresources` must still suppress the bundled defaults.
  assert!(
    !workdir.path().join("LaTeXML.css").exists(),
    "--nodefaultresources should suppress bundled LaTeXML.css",
  );
}

/// The copied CSS's LOCAL `@import` targets are followed recursively, with
/// their subdirectory structure recreated under the destination so the
/// cascade still resolves (the ar5iv "glowup" pattern: `ar5iv.css` →
/// `@import "./ar5iv/*.css"`). Remote (`https://…`) imports are left alone.
#[test]
fn cli_css_local_imports_are_recursively_copied_with_subdirs() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

  let workdir = tempfile::tempdir().expect("create tempdir");
  let styles = workdir.path().join("styles");
  std::fs::create_dir_all(styles.join("layer")).expect("mkdir styles/layer");
  // main.css imports a LOCAL sub-file (in a subdir) and a REMOTE sheet.
  std::fs::write(
    styles.join("main.css"),
    "@import url(\"./layer/part.css\") layer(base);\n\
     @import url('https://example.invalid/remote.css');\n\
     body { color: red; }\n",
  )
  .expect("write main.css");
  let part_css = "/* part marker */\np { margin: 0; }\n";
  std::fs::write(styles.join("layer").join("part.css"), part_css).expect("write part.css");

  let tex_path = workdir.path().join("hello.tex");
  std::fs::write(&tex_path, HELLO_TEX).expect("write hello.tex");

  let output = Command::new(bin)
    .arg("hello.tex")
    .arg("--dest")
    .arg("hello.html")
    .arg("--format")
    .arg("html5")
    .arg("--css")
    .arg("main.css")
    .arg("--path")
    .arg(&styles)
    .arg("--nodefaultresources")
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");

  assert!(
    output.status.success(),
    "binary exited {:?}\nstderr:\n{}",
    output.status.code(),
    String::from_utf8_lossy(&output.stderr),
  );

  // Top-level CSS copied next to the HTML (flattened to its basename)...
  assert!(
    workdir.path().join("main.css").is_file(),
    "main.css not copied next to hello.html",
  );

  // ...and the LOCAL @import target was followed AND its subdirectory was
  // recreated under the destination, so `./layer/part.css` resolves.
  let imported = workdir.path().join("layer").join("part.css");
  assert!(
    imported.is_file(),
    "expected @import target recreated at {} (subdir structure preserved)\nstderr:\n{}",
    imported.display(),
    String::from_utf8_lossy(&output.stderr),
  );
  assert_eq!(
    std::fs::read_to_string(&imported).expect("read imported part.css"),
    part_css,
    "recursively-copied @import content should match the source",
  );

  // The remote @import must NOT have been fetched/written locally.
  assert!(
    !workdir.path().join("remote.css").exists(),
    "remote @import should be left untouched, not copied locally",
  );
}
