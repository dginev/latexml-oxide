//! Schema validity (the perfect-kernel S2 bar): constructs that produced core
//! XML the LaTeXML RelaxNG schema rejects. Every guard pins the WHOLE offending
//! element in its corrected form AND validates the document with `jing`
//! (`latexml::util::test::rng_error_count`; `None` when jing is absent).
//! Sweep 78 clusters (`docs/perfect_kernel/LEDGER.md`, batch 56cf).

use std::{path::Path, process::Command};

use latexml::util::test::{assert_element, error_count, rng_error_count};

/// The raw-interpretation profile of the corpus sweeps, through the binary.
fn convert(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let out = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .env("NO_COLOR", "1")
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&out.stderr).to_string();
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (log, xml)
}

fn assert_valid(xml: &str) {
  if let Some(n) = rng_error_count(xml) {
    assert_eq!(n, 0, "schema-invalid core XML:\n{xml}");
  }
}

/// A `\lower`ed `\hbox` inside a tikz node label opens an `svg:g` while the
/// label's ltx content sits in an `svg:foreignObject`. The compiled model's
/// `*`/`*:*` child wildcards on `svg:foreignObject` (batch 54j) let the `svg:g`
/// NEST there, which the schema forbids (`SVG.foreignObject.content` is the
/// ltx flow + `svg:svg`); Perl's model has no wildcard, so `openElement`
/// auto-closes the foreignObject and the `svg:g` becomes its SIBLING
/// (15,650 jing lines, 25 sweep-78 manuals; envelope-letter.sty:144).
#[test]
fn lowered_hbox_in_a_tikz_node_is_a_sibling_of_the_foreign_object() {
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{tikz}\\begin{document}\n\\begin{tikzpicture}\\node at (0,0) {$\\otimes$\\lower2pt\\hbox{x}};\\end{tikzpicture}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "svg:svg",
    &[],
    r##"<svg:svg height="20.06" overflow="visible" version="1.1" viewBox="0 0 27.29 20.06" width="27.29"><svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,20.06) matrix(1 0 0 -1 0 0) translate(13.64,0) translate(0,10.03) matrix(1.0 0.0 0.0 1.0 -9.03 -2.65)"><svg:foreignObject height="9.22" overflow="visible" style="--ltx-fo-width:0.78em;--ltx-fo-height:0.58em;--ltx-fo-depth:0.08em;font-size:10pt;" transform="matrix(1 0 0 -1 0 8.07)" width="10.76"><Math mode="inline" tex="\otimes" text="tensor-product" xml:id="p1.pic1.m1"><XMath><XMTok meaning="tensor-product" name="otimes" role="MULOP">⊗</XMTok></XMath></Math></svg:foreignObject><svg:g transform="translate(0,-2.77)"><svg:foreignObject height="5.96" overflow="visible" style="--ltx-fo-width:0.53em;--ltx-fo-height:0.43em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 5.96)" width="7.3">x</svg:foreignObject></svg:g></svg:g></svg:svg>"##,
  );
  assert_valid(&xml);
}

/// pgf's `line cap=rect` is SVG's `square`; Perl's pgfsys-latexml.def.ltxml:411
/// emitted `stroke-linecap="rect"` (PERL-ORIGIN, 6,733 lines / 21 manuals).
#[test]
fn rect_line_cap_is_square() {
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{tikz}\\begin{document}\n\\begin{tikzpicture}\\draw[line cap=rect,line width=3pt] (0,0)--(2,0);\\end{tikzpicture}\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "svg:g",
    &["stroke-linecap"],
    r##"<svg:g fill="#000000" stroke="#000000" stroke-linecap="square" stroke-width="3.0pt" transform="translate(0,4.15) matrix(1 0 0 -1 0 0) translate(2.08,0) translate(0,2.08)"><svg:path d="M 0 0 L 78.74 0" style="fill:none"/></svg:g>"##,
  );
  assert_valid(&xml);
}

/// A zero-size box in an SVG context (quantikz's empty cells) still gets a
/// sized `svg:foreignObject`: Perl TeX_Box.pool.ltxml:409-424 sets width and
/// height whenever the whatsit exists; a nonzero guard left 3,010 of them
/// sizeless (32 manuals) against the schema's required attributes.
#[test]
fn empty_node_foreign_object_is_sized() {
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{tikz}\\usetikzlibrary{quantikz}\\begin{document}\n\\begin{quantikz}\\lstick{$\\ket{0}$} & \\gate{H} & \\qw\\end{quantikz}\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "svg:foreignObject",
    &[],
    r##"<svg:foreignObject height="0" overflow="visible" style="--ltx-fo-width:0.91em;--ltx-fo-height:0em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 0)" width="12.63"><text font="nullfont">  </text></svg:foreignObject>"##,
  );
  assert_valid(&xml);
}

/// `\iint`/`\iiint`/`\iiiint`/`\idotsint` carry the dynamic `mathstyle` of
/// `\int` (Perl `\&doVariablesizeOp`: `display` in display style, else `text`);
/// the literal `\displaystyle` string was schema-invalid (16 manuals).
#[test]
fn multiple_integrals_have_a_variablesize_op() {
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{amsmath}\n\\begin{document}\nInline $\\iint_D f$\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "XMTok",
    &["name=\"iint\""],
    r##"<XMTok mathstyle="text" meaning="double-integral" name="iint" role="INTOP">∬</XMTok>"##,
  );
  assert_valid(&xml);
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{amsmath}\n\\begin{document}\nDisplay \\[ \\iint_D f \\]\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "XMTok",
    &["name=\"iint\""],
    r##"<XMTok mathstyle="display" meaning="double-integral" name="iint" role="INTOP">∬</XMTok>"##,
  );
  assert_valid(&xml);
}

/// A pdfcomment annotation inside math floats its `<note>` out to the nearest
/// element that admits one (`^` prefix, as the kernel footnote), instead of
/// staying inside the `<XMText>` its marked text opened (5 manuals).
#[test]
fn pdfcomment_note_floats_out_of_math() {
  let (log, xml) = convert(
    "\\documentclass{article}\\usepackage{pdfcomment}\n\\begin{document}\n\\( P(E)\\pdfmarkupcomment{ = 1}{a note} = 1 \\)\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r##"<p><Math mode="inline" tex="P(E)=1\lx@pdfcomment@note{pdfmarkupcomment}{a note}=1" text="P@(E) = 1 = 1" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="multirelation"/><XMApp><XMTok font="italic" role="UNKNOWN">P</XMTok><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">E</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math><note role="pdfmarkupcomment">a note</note></p>"##,
  );
  assert_valid(&xml);
}
