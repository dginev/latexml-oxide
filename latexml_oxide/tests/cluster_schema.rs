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

/// As `convert`, through the html5 post-processor.
fn convert_html(tex: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let out = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--format=html5",
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .env("NO_COLOR", "1")
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&out.stderr).to_string();
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (log, html)
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
    r##"<p><Math mode="inline" tex="P(E)=1=1" text="P@(E) = 1 = 1" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="multirelation"/><XMApp><XMTok font="italic" role="UNKNOWN">P</XMTok><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">E</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp></XMath></Math><note role="pdfmarkupcomment">a note</note></p>"##,
  );
  assert_valid(&xml);
}

/// `\parbox` runs in Perl's `inline_internal_vertical` mode: in running text it
/// keeps the enclosing `<p>` and a multi-paragraph body becomes an
/// `inline-logical-block`; as plain `internal_vertical` it ended the paragraph
/// and `insert_block` emitted a `logical-block` inside `<para>` — the KOMA
/// letter demos' 381 schema errors (batch 56cy). Whole `<para>`, schema-valid.
#[test]
fn parbox_in_running_text_is_an_inline_logical_block() {
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\begin{document}\nX\\parbox{3cm}{\\noindent A\\par\\noindent B}Y\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_valid(&xml);
  assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>X<inline-logical-block class="ltx_parbox" vattach="middle" width="85.4pt"><para class="ltx_noindent" xml:id="p1.p1"><p>A</p></para><para class="ltx_noindent" xml:id="p1.p2"><p>B</p></para></inline-logical-block>Y</p></para>"##,
  );
}

/// `class` values are NMTOKENs (batch 56cz, user ruling): listings' language
/// literal `C++` produced `ltx_lst_language_C++`, invalid in both engines; the
/// emitter now keeps only NameChars. Whole `<listing>`, schema-valid.
#[test]
fn listing_language_class_is_an_nmtoken() {
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\usepackage{listings}\n\\begin{document}\n\\begin{lstlisting}[language=C++]\nint x;\n\\end{lstlisting}\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_valid(&xml);
  assert_element(
    &xml,
    "listing",
    &[],
    r##"<listing class="ltx_lst_language_C ltx_lstlisting" data="aW50IHg7" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_keyword" font="bold">int</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">x</text>;</listingline></listing>"##,
  );
}

/// A quote admits the paragraph and box containers a quotation can hold (batch
/// 56gw, user ruling 2026-09-23, OXIDIZED_DESIGN #271). Perl's `quote_model` is
/// `Block.model` (LaTeXML-block.rnc): a minipage holding `\tableofcontents`
/// (`logical-block`, webquiz) or `\section*` (`sectional-block`, aguplus) made
/// the document invalid, and a `\noindent` paragraph lost its `ltx:para` (and its
/// `ltx_noindent` class) to the auto-close (tagpair/sample). Whole `<para>`,
/// schema-valid.
#[test]
fn quote_holds_paragraph_and_box_blocks() {
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\begin{document}\n\\begin{quote}\n\\noindent Quoted.\n\\end{quote}\n\
     \\begin{quote}\n\\begin{minipage}{0.8\\linewidth}\n\\tableofcontents\n\\end{minipage}\n\\end{quote}\n\
     \\begin{quote}\n\\begin{minipage}{\\linewidth}\n\\section*{Notation}\nBody.\n\\end{minipage}\n\\end{quote}\n\
     \\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_valid(&xml);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r##"<para xml:id="p1"><quote><para class="ltx_noindent" xml:id="p1.p1"><p>Quoted.</p></para></quote><quote><logical-block class="ltx_minipage" vattach="middle" width="276.0pt"><TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Contents</title></TOC></logical-block></quote><quote><sectional-block class="ltx_minipage"><section xml:id="Sx1"><title>Notation</title><para xml:id="Sx1.p1"><p>Body.</p></para></section></sectional-block></quote></para>"##,
  );
}

/// `\footnote`/`\index`/`\nomenclature` inside `\text{}` in math float out of the Math (batch 56gx,
/// user ruling 2026-09-23, OXIDIZED_DESIGN #272). They constructed in the
/// `ltx:text` that `\text` opens; `cleanup_xmtext` unwrapped it and left the marker a
/// direct `XMText` child: schema-invalid, and the footnote read into the formula's
/// `text=` (`[b11footnote 1fn]`). Both engines (ribbonproofs, sidenotesplus,
/// ryethesis). Whole `<p>`: each marker right after its Math, schema-valid.
#[test]
fn meta_in_math_text_floats_out_of_the_math() {
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{makeidx}\n\\makeindex\n\
     \\usepackage{nomencl}\n\\makenomenclature\n\\begin{document}\n\
     A $a = \\text{b\\footnote{fn}} + c$ B.\nC $x \\text{y\\index{idx}} z$ D.\n\
     E $u \\text{v\\nomenclature{$u$}{speed}} w$ F.\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_valid(&xml);
  assert_element(
    &xml,
    "p",
    &[],
    r##"<p>A <Math mode="inline" tex="a=\text{b}+c" text="a = [b] + c" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMText>b</XMText><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMath></Math><note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>fn</note> B.
C <Math mode="inline" tex="x\text{y{\@index{\@indexphrase{idx}}}}z" text="x * [y] * z" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMText>y</XMText><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp></XMath></Math><indexmark><indexphrase key="idx">idx</indexphrase></indexmark> D.
E <Math mode="inline" tex="u\text{v\lx@nomencl@definition{a}{$u$}{{speed}}{}{}}w" text="u * [v] * w" xml:id="p1.m3"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">u</XMTok><XMText>v</XMText><XMTok font="italic" role="UNKNOWN">w</XMTok></XMApp></XMath></Math><glossarydefinition inlist="nomenclature" key="nomencl.1"><glossaryphrase key="nomencl.1" role="sort">a<Math mode="inline" tex="u" text="u" xml:id="p1.m3.m1"><XMath><XMTok font="italic" role="UNKNOWN">u</XMTok></XMath></Math></glossaryphrase><glossaryphrase key="nomencl.1" role="name"><Math mode="inline" tex="u" text="u" xml:id="p1.m3.m2"><XMath><XMTok font="italic" role="UNKNOWN">u</XMTok></XMath></Math></glossaryphrase><glossaryphrase key="nomencl.1" role="description">speed</glossaryphrase></glossarydefinition> F.</p>"##,
  );
}

const DISPLAY_NOTES: &str = "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n\
  \\[ d = \\text{e\\footnote{disp}} \\]\n\\begin{align}\nf &= \\text{g\\footnote{al}}\\\\\n\
  h &= k\\footnote{bare}\n\\end{align}\n\\end{document}\n";

/// In display math the floated footnote lands where a bare `\footnote` does: in the
/// `equation` after its Math. In an `align` cell it stays in the presentation `td`;
/// the MathFork no longer clones it into the main branch's math as a second `.mf`
/// note (batch 56gx, #272). Whole display `<equation>`, one note per footnote.
#[test]
fn footnote_in_display_math_text_floats_to_the_equation() {
  let (stderr, xml) = convert(DISPLAY_NOTES);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_valid(&xml);
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S0.Ex1\""],
    r##"<equation xml:id="S0.Ex1"><Math mode="display" tex="d=\text{e}" text="d = [e]" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok><XMText>e</XMText></XMApp></XMath></Math><note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>disp</note></equation>"##,
  );
  assert_eq!(
    xml.matches("<note ").count(),
    3,
    "one note per footnote:\n{xml}"
  );
  assert_element(
    &xml,
    "td",
    &["align=\"left\""],
    r##"<td align="left"><Math mode="inline" tex="\displaystyle=\text{g}" text="absent = [g]" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="absent"/><XMText>g</XMText></XMApp></XMath></Math><note mark="2" role="footnote" xml:id="footnote2"><tags><tag>2</tag><tag role="refnum">2</tag><tag role="typerefnum">footnote 2</tag></tags>al</note></td>"##,
  );
}

/// The equation-level notes render in HTML (batch 56gx, #272). Perl's XSLT renders
/// "all of equation_model EXCEPT Meta", so a footnote in display math, a bare
/// `\footnote` included, was dropped from the page. Now it renders after the math in
/// the equation's cell, and in an aligned row in the row's right padding cell.
/// Whole note spans, each inside its own equation's table.
#[test]
fn footnote_in_display_math_renders_in_html() {
  let (stderr, html) = convert_html(DISPLAY_NOTES);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for (n, text, table) in [
    (1, "disp", "S0.Ex1"),
    (2, "al", "S0.EGx1"),
    (3, "bare", "S0.EGx1"),
  ] {
    assert_element(
      &html,
      "span",
      &[&format!("id=\"footnote{n}\"")],
      &format!(
        r##"<span id="footnote{n}" class="ltx_note ltx_role_footnote"><sup class="ltx_note_mark">{n}</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">{n}</sup><span class="ltx_tag ltx_tag_note">{n}</span>{text}</span></span></span>"##
      ),
    );
    let t = latexml::util::test::xml_element(&html, "table", &[&format!("id=\"{table}\"")])
      .expect("equation table");
    assert!(
      t.contains(&format!("id=\"footnote{n}\"")),
      "footnote {n} outside {table}:\n{t}"
    );
  }
}
