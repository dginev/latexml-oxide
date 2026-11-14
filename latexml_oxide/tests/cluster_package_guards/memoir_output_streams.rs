//! memoir.cls output streams (L10965-11063) are CONTENT-BEARING: docs
//! write body fragments to \jobname.<ext> and \input them back
//! (dlfltxbmarkup-showkeys routes its whole body that way). Our memoir
//! binding delegates to REAL TeX write streams so the round-trip works.

const TEX: &str = "\\documentclass{memoir}\n\
    \\begin{document}\n\
    \\newoutputstream{keys}\n\
    \\openoutputfile{\\jobname.keys}{keys}\n\
    \\addtostream{keys}{ROUNDTRIP}\n\
    \\closeoutputstream{keys}\n\
    K[\\input{\\jobname.keys}]\n\
    \\end{document}\n";

#[test]
fn stream_write_and_readback() {
  let (stderr, xml) = super::convert(TEX, false);
  assert!(
    xml.contains("K[ROUNDTRIP"),
    "stream content must round-trip through the aux file:\n{xml}\n{stderr}",
  );
}
