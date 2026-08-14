//! `\lx@save@parameter{key}{value}` → a `<?latexml key="value"?>` processing
//! instruction (Perl `latexml.sty.ltxml` L86-96): the constructor inserts the
//! PI, and the `dpi`/`magnify`/`upsample`/`zoomout` package options schedule it
//! at `\begin{document}`. The Rust `latexml_sty` binding never defined it — so a
//! direct call errored `undefined:\lx@save@parameter`, and the image-scaling
//! options silently dropped their PIs (they assigned a dead `PI@latexml@…` state
//! value that nothing ever emitted). Issue #536 (reporter xworld21).
//!
//! Expectations ground-truthed against Perl LaTeXML 0.8.8 on the same input.

use std::{path::Path, process::Command};

/// Convert `tex` through the binary; return `(core-xml, ansi-stripped stderr)`.
fn convert(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("p.tex"), tex).expect("write p.tex");
  let output = Command::new(bin)
    .args(["p.tex", "--dest", "p.xml", "--nocomments"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let xml = std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_default();
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  (xml, stderr)
}

/// The four image-scaling options each save their value as a `<?latexml …?>` PI
/// (Perl emits `DPI` uppercase; the other three keep the keyval name).
#[test]
fn latexml_sty_image_scaling_options_emit_pis() {
  let (xml, stderr) = convert(
    "\\documentclass{article}\n\
     \\usepackage[dpi=300,magnify=1.5,upsample=2,zoomout=3]{latexml}\n\
     \\begin{document}Hello.\\end{document}\n",
  );
  assert!(
    xml.contains("<?latexml DPI=\"300\"?>"),
    "DPI PI missing:\n{xml}"
  );
  assert!(
    xml.contains("<?latexml magnify=\"1.5\"?>"),
    "magnify PI missing:\n{xml}"
  );
  assert!(
    xml.contains("<?latexml upsample=\"2\"?>"),
    "upsample PI missing:\n{xml}"
  );
  assert!(
    xml.contains("<?latexml zoomout=\"3\"?>"),
    "zoomout PI missing:\n{xml}"
  );
  assert!(
    !stderr.contains("undefined"),
    "unexpected undefined:\n{stderr}"
  );
}

/// A direct `\lx@save@parameter{key}{value}` emits its PI and does not error.
#[test]
fn latexml_sty_save_parameter_direct_call() {
  let (xml, stderr) = convert(
    "\\documentclass{article}\n\
     \\usepackage{latexml}\n\
     \\makeatletter\\lx@save@parameter{foo}{bar}\\makeatother\n\
     \\begin{document}Hello.\\end{document}\n",
  );
  assert!(
    xml.contains("<?latexml foo=\"bar\"?>"),
    "direct-call PI missing:\n{xml}"
  );
  assert!(
    !stderr.contains("is not defined") && !stderr.contains("undefined:\\lx@save@parameter"),
    "\\lx@save@parameter still undefined:\n{stderr}"
  );
}
