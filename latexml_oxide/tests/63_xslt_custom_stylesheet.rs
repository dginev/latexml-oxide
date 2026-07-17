//! Issue #292: a user `--stylesheet` that `<xsl:import>`s the built-in engine
//! via the LaTeXML-canonical `urn:x-LaTeXML:XSLT:` scheme must resolve against
//! the embedded XSLT (as Perl's XML catalog does), not fail with
//! "unable to load urn:x-LaTeXML:XSLT:LaTeXML-html5.xsl".
//!
//! Like the other `*_xslt_*` guards this can only be exercised end-to-end via
//! the binary — the in-process `Converter` stops at Core XML and never runs
//! post-processing/XSLT.

use std::{path::Path, process::Command};

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
  Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .current_dir(cwd)
    .output()
    .expect("spawn latexml_oxide")
}

/// A custom header stylesheet that imports the native HTML5 engine by URN and
/// overrides `head-resources` to inject an extra CSS link — the exact shape from
/// the issue. The import must resolve from the embedded XSLT table.
const CUSTOM_XSL: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:ltx="http://dlmf.nist.gov/LaTeXML"
  exclude-result-prefixes="ltx">
  <xsl:import href="urn:x-LaTeXML:XSLT:LaTeXML-html5.xsl"/>
  <xsl:template match="/" mode="head-resources">
    <xsl:apply-imports/>
    <link href="/styles/css/nma_latexml.css" rel="stylesheet" type="text/css"/>
  </xsl:template>
</xsl:stylesheet>
"#;

#[test]
fn custom_stylesheet_imports_engine_by_urn() {
  let work = tempfile::tempdir().expect("tempdir");
  std::fs::write(
    work.path().join("B.tex"),
    "\\documentclass{article}\n\\begin{document}\nHello stylesheet world.\n\\end{document}\n",
  )
  .unwrap();
  std::fs::write(work.path().join("my_header.xsl"), CUSTOM_XSL).unwrap();

  let out = run(work.path(), &[
    "--stylesheet=my_header.xsl",
    "B.tex",
    "--dest",
    "B.html",
  ]);
  let stderr = String::from_utf8_lossy(&out.stderr);

  // Canary: the urn import must not fail to load.
  assert!(
    !stderr.contains("Failed to parse XSLT stylesheet")
      && !stderr.contains("unable to load urn:x-LaTeXML:XSLT"),
    "#292: `urn:x-LaTeXML:XSLT:` import in a user stylesheet failed to resolve:\n{stderr}"
  );
  assert!(
    out.status.success(),
    "#292: conversion failed (status {:?}):\n{stderr}",
    out.status.code()
  );

  let html = std::fs::read_to_string(work.path().join("B.html")).expect("read B.html");
  // The engine imported (real HTML5 body rendered) AND the custom override fired.
  assert!(
    html.contains("Hello stylesheet world"),
    "#292: document body missing — engine import didn't apply:\n{html}"
  );
  assert!(
    html.contains("nma_latexml.css"),
    "#292: the custom head-resources override was not applied:\n{html}"
  );
}
