//! `\LaTeXMLversion` (Perl `latexml.sty.ltxml` L136-139) expands to
//! `$LaTeXML::VERSION`. The Rust analog is the latexml_oxide *binary's* own
//! crate version (the `LATEXML_VERSION` state value seeded at session init),
//! three-part `MAJOR.MINOR.PATCH`. The binding used to hard-code a stale
//! `"0.4.0"` — latexml_contrib's version, not the tool's (issue #541,
//! reporter xworld21).

use std::{path::Path, process::Command};

/// The three-part version of the binary crate — exactly what `LATEXML_VERSION`
/// resolves to (`core_interface.rs`), computed the same way so the test tracks
/// the version automatically instead of pinning a literal.
const EXPECTED: &str = concat!(
  env!("CARGO_PKG_VERSION_MAJOR"),
  ".",
  env!("CARGO_PKG_VERSION_MINOR"),
  ".",
  env!("CARGO_PKG_VERSION_PATCH"),
);

fn convert_body(tex: &str) -> String {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("p.tex"), tex).expect("write p.tex");
  let output = Command::new(bin)
    .args(["p.tex", "--dest", "p.xml", "--nocomments"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_else(|e| {
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    panic!("no output: {e}\n{stderr}");
  })
}

/// `\LaTeXMLversion` prints the running binary's version, not the stale 0.4.0.
#[test]
fn latexmlversion_is_the_binary_version() {
  let xml = convert_body(
    "\\documentclass{article}\n\
     \\usepackage{latexml}\n\
     \\begin{document}v=\\LaTeXMLversion\\end{document}\n",
  );
  assert!(
    xml.contains(&format!("v={EXPECTED}")),
    "expected \\LaTeXMLversion to be {EXPECTED}:\n{xml}"
  );
  assert!(
    !xml.contains("0.4.0"),
    "\\LaTeXMLversion still reports the stale latexml_contrib 0.4.0:\n{xml}"
  );
}
