//! `\LaTeXMLversion` (Perl `latexml.sty.ltxml` L136-139) expands to
//! `$LaTeXML::VERSION`. The Rust analog is the latexml_oxide *binary's* own
//! crate version (the `LATEXML_VERSION` state value seeded at session init),
//! three-part `MAJOR.MINOR.PATCH`. The binding used to hard-code a stale
//! `"0.4.0"` — latexml_contrib's version, not the tool's (issue #541,
//! reporter xworld21).

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
  let (log, xml) = latexml::util::test::convert_with(tex, None);
  assert!(!xml.is_empty(), "no output\n{log}");
  xml
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
