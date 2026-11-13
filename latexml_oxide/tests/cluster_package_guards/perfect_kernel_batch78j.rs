//! Red/green guards for batch 78j (quality batch 13): numeric citations sorted and compressed as natbib's `sort`/
//! `compress` options, cite.sty, revtex4* and acmart print them ("[1–3]"); cleveref's label format, an equation's
//! "(1)"; a negative `\multirow{-n}` spanning the rows above it; pdfpages' `\includepdf` placing the file's pages. Round
//! 2 (the review rev78j): the document's `\crefformat`/`\Crefformat`/`\crefmultiformat`/`\labelcrefformat` replacing
//! the whole reference, the label type's forms with the object's number, a label format expanded once, a page past a
//! PDF's last an error, natbib's numbers mode printing a given label, cite.sty's unsorted labels first. Round 3 (the
//! review rev78j2): a document format that leaves a group open, its errors held until a reference prints it, a format
//! of symbols alone, the tag's math a formula, a `\labelcref` list under a `\crefmultiformat`, cite.sty's
//! `[prefix]number[suffix]` labels, natbib's textual citation of a label given. Round 4 (the review rev78j3): a form
//! that leaves a group open keeps the document's alignment brace count, the empty held tags of minipage panels tied
//! and removed once the document is built, a format closing a group it did not open.
//! Each guard converts its repro under `tools/perfect_kernel/repros/` and asserts no Error or Warning (unless the shape
//! warns on purpose) and the authored content in whole elements, against pdflatex. Witnesses 2112.15328 (acmart),
//! 1509.00049 (negative `\multirow`), 2004.14422, 2312.00252, 2406.15983, 2609.28438, 1905.03839, 2510.06152,
//! 2008.12750, 2604.24399 (cleveref formats), 2608.05327 (natbib).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `tex` converted to HTML by the binary in a directory of its own, as a paper is (the citation and `\ref` text are the
/// post-processor's): its HTML, the conversion free of errors and warnings.
fn html_of(tex: &str) -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html")
}

/// natbib's `sort&compress` (natbib.sty:267-271): `\citep{c,a,b}` prints "[1–3]" (`\NAT@sort@cites`, :377; the run
/// compressed in `\NAT@citexnum`, :401-421), a textual `\citet` and a run of two keep their form ("Able [1]", "[1, 3]").
/// Red (int50a): "[3, 1, 2]", the options no-ops (Perl natbib.sty.ltxml:69-71 alike, KNOWN_PERL_ERRORS #1072). Repro
/// index-bib/natbib_sort_compress_numbers.
#[test]
fn natbib_sort_compress_numbers() {
  let html = html_of(repro!("index-bib/natbib_sort_compress_numbers.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">X <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>, <cite class="ltx_cite ltx_citemacro_citet">Able [<a href="#bib.bib1" title="" class="ltx_ref">1</a>]</cite>, <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib2" title="" class="ltx_ref">2</a>, p. 2]</cite>, <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>.</p>"##,
  );
}

/// The superscript form compresses inside its superscript: "1–3", a run of two "1; 2", "1–3; 5" (natbib.sty:401-421,
/// the form achemso.cls:1396 asks for). Red (int50a): "3; 1; 2". Repro index-bib/natbib_super_sort_compress.
#[test]
fn natbib_super_sort_compress() {
  let html = html_of(repro!("index-bib/natbib_super_sort_compress.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">X<cite class="ltx_cite ltx_citemacro_cite"><sup id="p1.1.1" class="ltx_sup"><a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a></sup></cite>, Y<cite class="ltx_cite ltx_citemacro_cite"><sup id="p1.1.2" class="ltx_sup"><a href="#bib.bib1" title="" class="ltx_ref">1</a>; <a href="#bib.bib2" title="" class="ltx_ref">2</a></sup></cite>, Z<cite class="ltx_cite ltx_citemacro_cite"><sup id="p1.1.3" class="ltx_sup"><a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>; <a href="#bib.bib5" title="" class="ltx_ref">5</a></sup></cite>.</p>"##,
  );
}

/// cite.sty sorts and compresses by default (cite.sty:95, :272-297): "[1–4]", a run of two "[1, 2]". Red (int50a):
/// "[3, 1, 2, 4]" (cite_sty.rs ignored the order, as Perl's cite.sty.ltxml:17-19). Repro
/// index-bib/cite_sty_sorts_and_compresses.
#[test]
fn cite_sty_sorts_and_compresses() {
  let html = html_of(repro!("index-bib/cite_sty_sorts_and_compresses.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">X <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib4" title="" class="ltx_ref">4</a>]</cite>, <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bib2" title="" class="ltx_ref">2</a>]</cite>, <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>]</cite>.</p>"##,
  );
}

/// Under `nosort` (cite.sty:412) the order stays as written and a run in that order still compresses ("[2–4]",
/// cite.sty:514-520). Repro index-bib/cite_sty_nosort_keeps_the_order.
#[test]
fn cite_sty_nosort_keeps_the_order() {
  let html = html_of(repro!("index-bib/cite_sty_nosort_keeps_the_order.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">X <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib3" title="" class="ltx_ref">3</a>, <a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bib2" title="" class="ltx_ref">2</a>, <a href="#bib.bib4" title="" class="ltx_ref">4</a>]</cite>, <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib2" title="" class="ltx_ref">2</a>–<a href="#bib.bib4" title="" class="ltx_ref">4</a>]</cite>.</p>"##,
  );
}

/// revtex4-2.cls:7042 (revtex4-1.cls:7021, revtex4.cls:5480) loads natbib `[sort&compress]`: "[1–3]". Red (int50a):
/// "[3, 1, 2]". Repro index-bib/revtex_citations_sort_and_compress.
#[test]
fn revtex_citations_sort_and_compress() {
  let html = html_of(repro!("index-bib/revtex_citations_sort_and_compress.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">X <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>, <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>.</p>"##,
  );
}

/// acmart's default `\citestyle{acmnumeric}` sets `sort&compress` through the class's own `\setcitestyle`
/// (acmart.cls:306-345, :385-389): "[1–3]", "[1, 2]". Red (int50a): "[3, 1, 2]", "[2, 1]". Witness 2112.15328. Repro
/// index-bib/acmart_numeric_citations_sort_and_compress.
#[test]
fn acmart_numeric_citations_sort_and_compress() {
  let html = html_of(repro!(
    "index-bib/acmart_numeric_citations_sort_and_compress.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">See <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>, <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bib2" title="" class="ltx_ref">2</a>]</cite>.</p>"##,
  );
}

/// cleveref prints an equation's number in its label format "(1)" (cleveref.sty:7838-7842
/// `\creflabelformat{equation}{\textup{(#2#1#3)}}`): `\cref` "eq. (1)", the pair "eqs. (1) and (2)", `\crefrange`
/// "eqs. (1) to (3)", `\labelcref` "(1)"; an item keeps the default `#2#1#3`, "item 1". Red (int50a): "eq. 1", …, "1"
/// (Perl cleveref.sty.ltxml:46-76 alike, KNOWN_PERL_ERRORS #1073). Repro
/// cross-references/cleveref_equation_label_format.
#[test]
fn cleveref_equation_label_format() {
  let html = html_of(repro!(
    "cross-references/cleveref_equation_label_format.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p"><a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq.</span> <span class="ltx_text ltx_ref_tag">(1)</span></a>; <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eqs.</span> <span class="ltx_text ltx_ref_tag">(1)</span></a> and <a href="#S0.E2" title="Equation 2" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(2)</span></a>; <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eqs.</span> <span class="ltx_text ltx_ref_tag">(1)</span></a> to <a href="#S0.E3" title="Equation 3" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(3)</span></a>; <a href="#S0.E1" title="Equation 1" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(1)</span></a>; <a href="#S0.I1.i1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">item</span> <span class="ltx_text ltx_ref_tag">1</span></a>.</p>"##,
  );
}

/// A document's `\creflabelformat{equation}{#2#1#3}` and `\crefdefaultlabelformat{[i] #2#1#3}` (cleveref.sty:1264-1272)
/// replace the formats: "eq. 1", "item [i] 1". Repro cross-references/cleveref_creflabelformat_overrides.
#[test]
fn cleveref_creflabelformat_overrides() {
  let html = html_of(repro!(
    "cross-references/cleveref_creflabelformat_overrides.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p"><a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq.</span> <span class="ltx_text ltx_ref_tag">1</span></a>; <a href="#S0.I1.i1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">item</span> <span class="ltx_text ltx_ref_tag">[i] 1</span></a>.</p>"##,
  );
}

/// The label format is typeset only where a `\cref` prints it (cleveref.sty:1329-1334): a document's malformed
/// `\creflabelformat{enumi}{label*=\arabic*}` (an enumitem key list) for items it never references raises nothing,
/// as in pdflatex; the tag's diagnostics are held (`type_tag_deferred`, ruling 7e). Red before the hold: 8 Warnings
/// ("Counter '*' was not defined"). Witness 2506.11592 (96 Warnings). Repro
/// cross-references/cleveref_label_format_waits_for_a_reference.
#[test]
fn cleveref_label_format_waits_for_a_reference() {
  let html = html_of(repro!(
    "cross-references/cleveref_label_format_waits_for_a_reference.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">See <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq.</span> <span class="ltx_text ltx_ref_tag">(1)</span></a>.</p>"##,
  );
}

/// A negative `\multirow{-n}` spans its row and the n-1 above it (multirow.sty:172 raises the box over them, the
/// colortbl idiom): its content moves to the span's top row, `rowspan="2"` / `rowspan="3"`, and the table is the one
/// the positive spans make, each row's `\rowcolor` kept. Red (int50a): 2 `Warning:unsupported:multirow`, "Up"/"Low" in the last row, spanning one
/// (Perl multirow.sty.ltxml:28 alike, KNOWN_PERL_ERRORS #1074). Witness 1509.00049 (42 such warnings). Repro
/// alignment-bindings/multirow_negative_spans_the_rows_above.
#[test]
fn multirow_negative_spans_the_rows_above() {
  let tex = repro!("alignment-bindings/multirow_negative_spans_the_rows_above.tex");
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // The same table written with positive spans in the top rows.
  let positive = tex
    .replace(
      " & a & b \\\\\n\\multirow{-2}{*}{Up}",
      "\\multirow{2}{*}{Up} & a & b \\\\\n",
    )
    .replace(
      "\\rowcolor{gray!20} & e",
      "\\rowcolor{gray!20} \\multirow{3}{*}{Low} & e",
    )
    .replace("\\multirow{-3}{*}{Low}", "");
  let body = &positive[positive.find("\\begin{document}").unwrap()..];
  assert_eq!(body.matches("\\multirow{").count(), 2, "{positive}");
  assert!(!body.contains("{-"), "{positive}");
  let (positive_log, positive_xml) = convert_with(&positive, Some(PRELOAD));
  assert_eq!(
    (error_count(&positive_log), warning_count(&positive_log)),
    (0, 0),
    "{positive_log}"
  );
  let table =
    |xml: &str| xml[xml.find("<tabular").unwrap()..xml.find("</tabular>").unwrap()].to_string();
  // The rows' `\rowcolor`s stay theirs: the last row of the span keeps its colour (its covered cell carried it), which
  // a positive span's covered rows lose.
  let last_row = r##"<tr backgroundcolor="#E6E6E6" xml:id="p1.1.5">"##;
  assert!(xml.contains(last_row), "{xml}");
  assert_eq!(
    table(&xml).replace(last_row, r#"<tr xml:id="p1.1.5">"#),
    table(&positive_xml)
  );
  assert_element(
    &xml,
    "td",
    &["xml:id=\"p1.1.1.1\""],
    r#"<td align="left" border="t" rowspan="2" thead="column row" xml:id="p1.1.1.1"><text xml:id="p1.1.1.1.1">Up</text></td>"#,
  );
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.2\""],
    r#"<tr xml:id="p1.1.2"><td align="center" thead="column" xml:id="p1.1.2.1">c</td><td align="center" thead="column" xml:id="p1.1.2.2">d</td></tr>"#,
  );
  assert_element(
    &xml,
    "td",
    &["xml:id=\"p1.1.3.1\""],
    r##"<td align="left" border="b t" rowspan="3" thead="row" xml:id="p1.1.3.1"><text backgroundcolor="#E6E6E6" xml:id="p1.1.3.1.1">Low</text></td>"##,
  );
}

/// The control: over a cell that holds text (TeX overprints it, HTML cannot) the content stays in its row, spanning it
/// alone, with one Warning naming the cell. Repro alignment-bindings/multirow_negative_over_content_stays.
#[test]
fn multirow_negative_over_content_stays() {
  let (log, xml) = convert_with(
    repro!("alignment-bindings/multirow_negative_over_content_stays.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("\\multirow{-2} in cell(1,0) covers non-empty cells above it"),
    "{log}"
  );
  assert_element(
    &xml,
    "tr",
    &["xml:id=\"p1.1.2\""],
    r#"<tr xml:id="p1.1.2"><td align="left" thead="row" xml:id="p1.1.2.1"><text xml:id="p1.1.2.1.1">Up</text></td><td align="center" xml:id="p1.1.2.2">c</td></tr>"#,
  );
}

/// pdfpages' `\includepdf[pages=<list>]{file}` places the file's pages (pdfpages.sty: `pages` defaults to 1, `last` is
/// the page count read from the file): one `ltx:graphics` a page with `page=N`, in its own paragraph, as pdflatex
/// ships each as a page. Red (int50a): "See pages 2-4 of ⟨link⟩" with the PDF as a resource (Perl
/// pdfpages.sty.ltxml:30-36 alike; OXIDIZED_DESIGN_DIVERGENCES #846). Repro singletons/includepdf_places_its_pages.
#[test]
fn includepdf_places_its_pages() {
  let (log, xml) = convert_with(
    repro!("singletons/includepdf_places_its_pages.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // (the candidates are the file's path relative to the conversion's directory)
  let xml = regex::Regex::new(r#" candidates="[^"]*""#)
    .unwrap()
    .replace_all(&xml, "")
    .to_string();
  for (para, page) in [(2, 2), (3, 3), (4, 4), (5, 12), (6, 1)] {
    assert_element(
      &xml,
      "para",
      &[&format!("xml:id=\"p{para}\"")],
      &format!(
        r#"<para xml:id="p{para}"><graphics cssstyle="width:59.751em; height:84.505em" graphic="example-image-a4-numbered.pdf" options="page={page}" xml:id="p{para}.g1"/></para>"#
      ),
    );
  }
  assert!(!xml.contains("<resource"), "{xml}");
}

/// The page list and the file name are expanded, as pdfpages reads them: a `\foreach` loop's `pages={\x}`
/// (2605.05847), a name kept in a macro (2406.13026's `{\supplementfilename}`); the blank page `{}` places nothing.
/// Red (78j r1): "See pages 3 of ⟨link⟩". Repro singletons/includepdf_expands_its_arguments.
#[test]
fn includepdf_expands_its_arguments() {
  let (log, xml) = convert_with(
    repro!("singletons/includepdf_expands_its_arguments.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = regex::Regex::new(r#" candidates="[^"]*""#)
    .unwrap()
    .replace_all(&xml, "")
    .to_string();
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><graphics cssstyle="width:59.751em; height:84.505em" graphic="example-image-a4-numbered.pdf" options="page=3" xml:id="p2.g1"/></para>"#,
  );
  assert!(!xml.contains("<resource"), "{xml}");
}

/// A document's `\crefformat`/`\Crefformat` replace the whole reference (cleveref.sty:1851, :1886-1906; `#1` the number,
/// `#2`/`#3` the link's ends): `\crefformat{equation}{#2#1#3}` prints "1", `\Crefformat{equation}{Equation~#2#1#3}`
/// "Equation 1", and `\labelcref` of that type the bare number (cleveref builds it no `\labelcref` format,
/// :1438-1440); a type with no format of the document's keeps cleveref's ("section [1]"). Red (78j r1): "eq. (1)",
/// "Equation (1)", "(1)". Witnesses 2004.14422, 2312.00252, 2406.15983. Repro
/// cross-references/cleveref_document_formats_replace_the_reference.
#[test]
fn cleveref_document_formats_replace_the_reference() {
  let html = html_of(repro!(
    "cross-references/cleveref_document_formats_replace_the_reference.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"S1.p1.1\""],
    r##"<p id="S1.p1.1" class="ltx_p">P <a href="#S1.E1" title="In 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>. Q <a href="#S1.E1" title="In 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Equation 1</span></a>. R <a href="#S1.E1" title="Equation 1 ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>. U <a href="#S1" title="1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">section</span> <span class="ltx_text ltx_ref_tag">[1]</span></a>. V <a href="#S1" title="1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">[1]</span></a>.</p>"##,
  );
}

/// `\crefmultiformat`'s position formats hold their conjunctions (cleveref.sty:1000-1018, :1939-1975): `\cref{e1,e3}`
/// "(1, 3)", cleveref's own pair conjunction dropped; the `\Cref` variant derived by case change (:1888-1904) "(1)";
/// a subsection takes the section's `\crefformat{section}{\S#2#1#3}` (`\@crefcopyformats`, :1688-1736) "§1.1"; figures,
/// with no format of the document's, keep "figs. 1 and 2". Red (78j r1): "eq. (1)", "eqs. (1) and (3)", "section 1.1".
/// Witnesses 1409.7964, 1603.08012. Repro cross-references/cleveref_document_multi_formats.
#[test]
fn cleveref_document_multi_formats() {
  let html = html_of(repro!(
    "cross-references/cleveref_document_multi_formats.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"S1.SS1.p1.1\""],
    r##"<p id="S1.SS1.p1.1" class="ltx_p">A <a href="#S1.E1" title="In 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(1)</span></a>. B <a href="#S1.E1" title="In 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(1)</span></a>. C <a href="#S1.E1" title="In 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(1</span></a><a href="#S1.E3" title="Equation 3 ‣ 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">, 3)</span></a>. E <a href="#S1.SS1" title="1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">§1.1</span></a>. F <a href="#S1.F1" title="In 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">figs.</span> <span class="ltx_text ltx_ref_tag">1</span></a> and <a href="#S1.F2" title="Figure 2 ‣ 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">2</span></a>. G <a href="#S1.E1" title="Equation 1 ‣ 1.1 Sub ‣ 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>. H <a href="#S1" title="1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>.</p>"##,
  );
}

/// Under `\label[<type>]` the label type's formats print the object's own number (cleveref.sty:82-96 changes the
/// label's type only): "Lem. 1" (the document's `\crefformat{lemma}`), "app. eq. [2]" (its label format), and
/// `\labelcref` "1", "[2]". Red (78j r1): "lemma 1", "app. eq. (2)", "(2)". Repro
/// cross-references/cleveref_label_type_formats.
#[test]
fn cleveref_label_type_formats() {
  let html = html_of(repro!("cross-references/cleveref_label_type_formats.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Lem. 1</span></a>. B <a href="#S0.E2" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">app. eq.</span> <span class="ltx_text ltx_ref_tag">[2]</span></a>. C <a href="#S0.E3" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq.</span> <span class="ltx_text ltx_ref_tag">(3)</span></a>. D <a href="#S0.E1" title="Lemma 1" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a>. E <a href="#S0.E2" title="App. eq. 2" class="ltx_ref"><span class="ltx_text ltx_ref_tag">[2]</span></a>.</p>"##,
  );
}

/// In natbib's numbers mode a label with no `(year)` and no `, ` is the number (natbib.sty:811-817 `\NAT@bare`,
/// :1005-1012 `\NAT@apalk`): `\citep{k1,k2,k3}` prints "[Knu84, Lam86, Mil90]", uncompressed, as the list does. Red
/// (78j r1): "[4–6]", the entry counter. Repro index-bib/natbib_numbers_prints_a_label_given.
#[test]
fn natbib_numbers_prints_a_label_given() {
  let html = html_of(repro!("index-bib/natbib_numbers_prints_a_label_given.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>. G <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib4" title="" class="ltx_ref">Knu84</a>, <a href="#bib.bib5" title="" class="ltx_ref">Lam86</a>, <a href="#bib.bib6" title="" class="ltx_ref">Mil90</a>]</cite>.</p>"##,
  );
}

/// cite.sty prints a citation it cannot sort (a label not a number, an undefined key) at once, before the sorted ones
/// (cite.sty:205-230 `\@cite@dump@now`): "[Knu84, Lam86, 1]"; natbib puts them after. Red (78j r1): "[1, Knu84,
/// Lam86]". Repro index-bib/cite_sty_prints_unsorted_labels_first.
#[test]
fn cite_sty_prints_unsorted_labels_first() {
  let html = html_of(repro!(
    "index-bib/cite_sty_prints_unsorted_labels_first.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>. C <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bibx1" title="" class="ltx_ref">Knu84</a>, <a href="#bib.bibx2" title="" class="ltx_ref">Lam86</a>, <a href="#bib.bib1" title="" class="ltx_ref">1</a>]</cite>. E <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bib1" title="" class="ltx_ref">1</a>–<a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>.</p>"##,
  );
}

/// A label format is expanded once, where its tag is built, and only for the reference that prints it: the document's
/// malformed `\creflabelformat{enumi}{label*=\arabic*}` raises one expansion's 2 Warnings for the one `\cref` (pdflatex
/// one "Missing number" error). Red (78j r1): 4, the format expanded again to test for a plain number. Witness
/// 2506.11592. Repro cross-references/cleveref_label_format_expanded_once.
#[test]
fn cleveref_label_format_expanded_once() {
  let (log, xml) = convert_with(
    repro!("cross-references/cleveref_label_format_expanded_once.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(log.contains("Counter '*' was not defined"), "{log}");
  assert!(
    xml.contains(r#"<tag role="creflabel">label*=0</tag>"#),
    "{xml}"
  );
}

/// A page past the file's last is an error, placing nothing (pdfpages.sty:909-920 `\AM@checkpagenumber`, "Page 13 of
/// `example-image-a4-numbered.pdf' does not exist"): `pages=11-14` of the 12-page file places 11 and 12. Red (78j r1):
/// pages 13 and 14 placed (the last page again), no error. Repro singletons/includepdf_page_past_the_last.
#[test]
fn includepdf_page_past_the_last() {
  let (log, xml) = convert_with(
    repro!("singletons/includepdf_page_past_the_last.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  for page in [13, 14] {
    assert!(
      log.contains(&format!(
        "Package pdfpages Error: Page {page} of `example-image-a4-numbered.pdf' does not exist."
      )),
      "{log}"
    );
  }
  let xml = regex::Regex::new(r#" candidates="[^"]*""#)
    .unwrap()
    .replace_all(&xml, "")
    .to_string();
  for (para, page) in [(2, 11), (3, 12)] {
    assert_element(
      &xml,
      "para",
      &[&format!("xml:id=\"p{para}\"")],
      &format!(
        r#"<para xml:id="p{para}"><graphics cssstyle="width:59.751em; height:84.505em" graphic="example-image-a4-numbered.pdf" options="page={page}" xml:id="p{para}.g1"/></para>"#
      ),
    );
  }
  assert_eq!(xml.matches("<graphics").count(), 2, "{xml}");
}

/// A document format that typesets a numbered object of its own, or computes, gives no form: `\crefformat{footnote}
/// {#2\footnotemark[#1]#3}` (pdflatex typesets the mark at the `\cref`) made a footnote mark whose forms made a mark
/// again (Fatal `Stomach:Recursion` in the r2 A/B: 1601.06262, 1711.01177, 2005.13962, 2008.04624), a format with a
/// `\cref` (or 2608.12136's xstring calls on the label being referenced, a Fatal PushbackLimit) reads what the target
/// cannot know; those references print cleveref's defaults "footnote 1", "lemma 1", the other formats theirs ("Fact 1"
/// through `\ensuremath`, 1905.03839), their math a formula: "(1)" one `<math>`, where r2 flattened it to "Fact 1 "
/// with the formula's whitespace (the tag stored as text; Scan now stores a tag with markup as its copy, Perl
/// Scan.pm:269). Repro cross-references/cleveref_document_format_with_a_note_mark.
#[test]
fn cleveref_document_format_with_a_note_mark() {
  let html = html_of(repro!(
    "cross-references/cleveref_document_format_with_a_note_mark.tex"
  ));
  let text = regex::Regex::new(r"\s+")
    .unwrap()
    .replace_all(&html, " ")
    .to_string();
  for reference in [
    r##"A <a href="#Thmfact1" title="Fact 1." class="ltx_ref"><span class="ltx_text ltx_ref_tag">Fact <math class="ltx_Math" alttext="{{1}}" display="inline" intent=":literal"><mn>1</mn></math></span></a>."##,
    r##"B <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><math class="ltx_Math" alttext="({{1}})" display="inline" intent=":literal"><mrow><mo stretchy="false">(</mo><mn>1</mn><mo stretchy="false">)</mo></mrow></math></span></a>."##,
    r##"C <a href="#footnote1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">footnote</span> <span class="ltx_text ltx_ref_tag">1</span></a>."##,
    r##"D <a href="#Thmlemma1" title="Lemma 1." class="ltx_ref"><span class="ltx_text ltx_ref_tag">lemma</span> <span class="ltx_text ltx_ref_tag">1</span></a>."##,
  ] {
    assert!(text.contains(reference), "{text}");
  }
  assert_eq!(text.matches("ltx_note_mark").count(), 2, "{text}");
}

/// A cleveref read after `\begin{document}` (a class's `\AtBeginDocument{\RequirePackage{cleveref}\crefformat…}`)
/// runs its hooks at once: the formats set after it are the document's, "(1)", "Equation (1)". Red (an r2 build): 1
/// Error, `\lx@cref@building` undefined (the bracket's hook ran before the binding defined it). Witnesses 2512.04983,
/// 2608.14769. Repro cross-references/cleveref_loaded_at_begin_document.
#[test]
fn cleveref_loaded_at_begin_document() {
  let html = html_of(repro!(
    "cross-references/cleveref_loaded_at_begin_document.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(1)</span></a>. B <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Equation (1)</span></a>.</p>"##,
  );
}

/// A document format whose derived variant does not close what it opens gives no form, and the others print as set:
/// `\crefformat{figure}{#2\color{blue}Fig.~#1#3}` derives the `\Cref` variant `\MakeUppercase#2\color{blue}…`
/// (cleveref.sty:1886-1906), whose `\MakeUppercase` takes `\color` alone; built at the target, its open group was left
/// to the tag's `\endgroup`. pdflatex "A Fig. 1. B theorem 1. C Eq. 1.", each blue. Red (78j r2): "Attempt to close
/// non-boxing group", "Attempt to end mode internal_vertical", the figure unended and the reference "id1". Witnesses
/// 2008.12750, 2604.24399. Repro cross-references/cleveref_document_format_opening_a_group.
#[test]
fn cleveref_document_format_opening_a_group() {
  let html = html_of(repro!(
    "cross-references/cleveref_document_format_opening_a_group.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.F1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Fig. 1</span></span></a>. B <a href="#Thmtheorem1" title="Theorem 1" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text ltx_font_sansserif ltx_font_bold" style="--ltx-fg-color:#0000FF;">theorem </span><span class="ltx_text" style="--ltx-fg-color:#0000FF;">1</span></span></a>. C <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Eq. 1</span></span></a>.</p>"##,
  );
}

/// A document format the target has is the whole reference, with no context sought for one with no letter or digit:
/// siamart's `\crefformat{equation}{\textup{#2(#1)#3}}` with `\tag{$\star$}` prints "(⋆)" (pdflatex). Red (78j r2):
/// "( ⋆ ) ‣ section 1", "(*) ‣ section 1" (crossref.rs `generate_ref`'s walk to the parent). Witness 1905.03839. Repro
/// cross-references/cleveref_symbol_format_stands_alone.
#[test]
fn cleveref_symbol_format_stands_alone() {
  let html = html_of(repro!(
    "cross-references/cleveref_symbol_format_stands_alone.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"S1.p1.1\""],
    r##"<p id="S1.p1.1" class="ltx_p">A <a href="#S1.Ex1" title="In 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(<math class="ltx_Math" alttext="\star" display="inline" intent=":literal"><mo>⋆</mo></math>)</span></a>. B <a href="#S1.Ex2" title="In 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">(*)</span></a>.</p>"##,
  );
}

/// A tag with math is stored as its copy (Perl Scan.pm:269 `cleanNode`), so a reference prints its formula: `\tag{$
/// \mathrm{P_1}$}` gives "P1", "(P1)", "eq. (P1)" (pdflatex), each a `<math>`. Red (78j r2): " P 1 ", "( P 1 )", "eq. (
/// P 1 )", the tag's flattened text with the formula's whitespace. Witnesses 2609.28438, 1905.03839, 2510.06152. Repro
/// cross-references/cleveref_document_format_with_math_in_the_tag.
#[test]
fn cleveref_document_format_with_math_in_the_tag() {
  let html = html_of(repro!(
    "cross-references/cleveref_document_format_with_math_in_the_tag.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.Ex1" title="Equation P_1" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><math class="ltx_Math" alttext="\mathrm{P_{1}}" display="inline" intent=":literal"><msub><mi mathvariant="normal">P</mi><mn>1</mn></msub></math></span></a>. B (<a href="#S0.Ex1" title="Equation P_1" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><math class="ltx_Math" alttext="\mathrm{P_{1}}" display="inline" intent=":literal"><msub><mi mathvariant="normal">P</mi><mn>1</mn></msub></math></span></a>). C <a href="#S0.Ex1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq. (<math class="ltx_Math" alttext="\mathrm{P_{1}}" display="inline" intent=":literal"><msub><mi mathvariant="normal">P</mi><mn>1</mn></msub></math>)</span></a>.</p>"##,
  );
}

/// A `\labelcref` list of a type whose `\crefmultiformat` the document set prints the default `\labelcref` format, the
/// number as it is: cleveref builds a type's `\labelcref` multi formats only where it has no `\cref` multi format yet
/// (cleveref.sty:8028-8030, :1543-1545; :1620-1669 the defaults). pdflatex "A 1 and 3. B Eqs. (1) and (3). C 1 and 2."
/// Red (78j r2): "A (1) and (3)" (the equation's label format). Witness 1905.03839 (siamart's `\crefmultiformat
/// {equation}`). Repro cross-references/cleveref_labelcref_list_under_a_multi_format.
#[test]
fn cleveref_labelcref_list_under_a_multi_format() {
  let html = html_of(repro!(
    "cross-references/cleveref_labelcref_list_under_a_multi_format.tex"
  ));
  assert_element(
    &html,
    "p",
    &["id=\"p2.1\""],
    r##"<p id="p2.1" class="ltx_p">A <a href="#S0.E1" title="Equation 1" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a> and <a href="#S0.E3" title="Equation 3" class="ltx_ref"><span class="ltx_text ltx_ref_tag">3</span></a>. B <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Eqs. (1)</span></a><a href="#S0.E3" title="Equation 3" class="ltx_ref"><span class="ltx_text ltx_ref_tag"> and (3)</span></a>. C <a href="#S0.T1" title="Table 1" class="ltx_ref"><span class="ltx_text ltx_ref_tag">1</span></a> and <a href="#S0.T2" title="Table 2" class="ltx_ref"><span class="ltx_text ltx_ref_tag">2</span></a>.</p>"##,
  );
}

/// cite.sty sorts `[prefix]number[suffix]` labels by its sort numbers (cite.sty:127-222): "2a" after 2, before 3; a run
/// of consecutive suffixes compresses (:266-297), "[4a–4c]"; a label it cannot read prints first ("Knu84"). pdflatex
/// "A [1, 2a, 7]. B [Knu84, 2, 3]. C [4a–4c]." Red (78j r2): "[2a, 1, 7]", "[4b, 4a, 4c]". Repro
/// index-bib/cite_sty_sorts_numbers_with_a_suffix.
#[test]
fn cite_sty_sorts_numbers_with_a_suffix() {
  let html = html_of(repro!("index-bib/cite_sty_sorts_numbers_with_a_suffix.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="" class="ltx_ref">1</a>, <a href="#bib.bibx2" title="" class="ltx_ref">2a</a>, <a href="#bib.bibx3" title="" class="ltx_ref">7</a>]</cite>. B <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bibx1" title="" class="ltx_ref">Knu84</a>, <a href="#bib.bib2" title="" class="ltx_ref">2</a>, <a href="#bib.bib3" title="" class="ltx_ref">3</a>]</cite>. C <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bibx4" title="" class="ltx_ref">4a</a>–<a href="#bib.bibx6" title="" class="ltx_ref">4c</a>]</cite>.</p>"##,
  );
}

/// In natbib's numbers mode a label given (`\bibitem[Knu84]`) is the number, not the full authors as well: `\citet{k}`
/// prints "Knu84", as an entry without authors does ("2"; pdflatex "(author?) [Knu84]", "(author?) [2]": the "(author?)"
/// is not printed, as in Perl). Red (78j r2): "Knu84 [Knu84]". Witness 2608.05327 ("[21]" → "[FNW23]"). Repro
/// index-bib/natbib_numbers_textual_label.
#[test]
fn natbib_numbers_textual_label() {
  let html = html_of(repro!("index-bib/natbib_numbers_textual_label.tex"));
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <cite class="ltx_cite ltx_citemacro_citet"><a href="#bib.bib1" title="" class="ltx_ref">Knu84</a></cite>. B <cite class="ltx_cite ltx_citemacro_citet"><a href="#bib.bib2" title="" class="ltx_ref">2</a></cite>. C <cite class="ltx_cite ltx_citemacro_citep">[<a href="#bib.bib1" title="" class="ltx_ref">Knu84</a>]</cite>.</p>"##,
  );
}

/// A form's errors are raised where a reference prints it, as TeX raises them at the reference: `\crefformat{figure}
/// {F & #2#1#3}`'s stray `&` once, for the one `\cref` (the tab dropped, "F 1"), not for every figure's two forms; the
/// `\Cref` variant of `\crefformat{table}{#2\color{blue}Tab.~#1#3}`, which leaves a group open, once for its `\Cref`
/// (which prints cleveref's default "Table 2"). pdflatex 3 errors at the references, "A F 1. B Tab. 1.": the `&`'s
/// "Misplaced alignment tab character", then 2 for the `\Cref`'s one runaway argument ("Argument of \@declaredcolor has
/// an extra }", "Paragraph ended before \@declaredcolor was complete"), which TeX recovers from by typesetting on and
/// we by printing cleveref's default — one Error for the form, 2 in all. Red (78j r2): 6 "Stray alignment" Errors and 2
/// group Errors, the table unended. Repro cross-references/cleveref_document_format_errors_wait_for_a_reference.
#[test]
fn cleveref_document_format_errors_wait_for_a_reference() {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(
    workdir.path().join("t.tex"),
    repro!("cross-references/cleveref_document_format_errors_wait_for_a_reference.tex"),
  )
  .expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert_eq!(log.matches("Stray alignment \"&\"").count(), 1, "{log}");
  assert_eq!(
    log
      .matches("The cleveref format of the table (at t; line 18 col 36 - line 18 col 36) leaves 1 group(s) open")
      .count(),
    1,
    "{log}"
  );
  let html = std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html");
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.F1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">F  1</span></a>. B <a href="#S0.T1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Tab. 1</span></span></a>. C <a href="#S0.T2" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Table</span> <span class="ltx_text ltx_ref_tag">2</span></a>.</p>"##,
  );
}

/// `tex` converted to HTML by the binary as [`html_of`] converts it: its log and HTML, the conversion raising `errors`
/// Errors and no Warning.
fn html_with_errors(tex: &str, errors: usize) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (errors, 0),
    "{log}"
  );
  let html = std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html");
  (log, html)
}

/// Whether `xml` holds an empty `ltx:tag` (`<tag …/>`).
fn has_empty_tag(xml: &str) -> bool {
  xml.match_indices("<tag").any(|(at, _)| {
    let rest = &xml[at..];
    matches!(rest.as_bytes().get(4), Some(b' ' | b'/' | b'>'))
      && rest[..rest.find('>').unwrap_or(rest.len())].ends_with('/')
  })
}

/// A form is not the document's input: the `{` its text scans and leaves open is undone with its group
/// (`stomach::digest_in_own_group`). Under `\crefformat{figure}{#2\color{blue}Fig.~#1#3}` (2008.12750, 2604.24399) the
/// derived `\Cref` form of each captioned minipage in a figure's tabular leaves one; pdflatex 0 errors, the rows "x | y"
/// and "p | q", each row asserted whole. Red (78j r3): 2 "Missing } inserted" + "Extra alignment tab '&'", "p" moved
/// into the row above. Repro
/// cross-references/cleveref_format_form_keeps_the_alignment_brace_count.
#[test]
fn cleveref_format_form_keeps_the_alignment_brace_count() {
  let html = html_of(repro!(
    "cross-references/cleveref_format_form_keeps_the_alignment_brace_count.tex"
  ));
  assert_element(
    &html,
    "tr",
    &["id=\"fig1.1.1\""],
    r##"<tr id="fig1.1.1" class="ltx_tr">
<td id="fig1.1.1.1" class="ltx_td ltx_align_center"><span id="fig1.1.1.1.1" class="ltx_inline-logical-block ltx_minipage ltx_align_middle" style="width:113.8pt;">
<span id="S0.F1" class="ltx_figure ltx_figure_panel">
<span id="S0.F1.2" class="ltx_p">x</span>
<span class="ltx_caption"><span class="ltx_tag ltx_tag_figure">Figure 1: </span>left</span>
</span></span></td>
<td id="fig1.1.1.2" class="ltx_td ltx_align_center"><span id="fig1.1.1.2.1" class="ltx_inline-logical-block ltx_minipage ltx_align_middle" style="width:113.8pt;">
<span id="S0.F2" class="ltx_figure ltx_figure_panel">
<span id="S0.F2.2" class="ltx_p">y</span>
<span class="ltx_caption"><span class="ltx_tag ltx_tag_figure">Figure 2: </span>right</span>
</span></span></td></tr>"##,
  );
  assert_element(
    &html,
    "tr",
    &["id=\"fig1.1.2\""],
    r##"<tr id="fig1.1.2" class="ltx_tr">
<td id="fig1.1.2.1" class="ltx_td ltx_align_center">p</td>
<td id="fig1.1.2.2" class="ltx_td ltx_align_center">q</td></tr>"##,
  );
  assert_eq!(html.matches("<tr ").count(), 2, "{html}");
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.F1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Fig. 1</span></span></a>. B <a href="#S0.F2" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Fig. 2</span></span></a>.</p>"##,
  );
}

/// The same for an item's form built in a tabular cell's minipage: the row's `\\` ends the row. pdflatex 2 errors at
/// `\Cref{i1}`, one runaway argument ("Argument of \@declaredcolor has an extra }", "Paragraph ended before
/// \@declaredcolor was complete"), which TeX typesets on from and we answer with cleveref's default ("Item 1"): one Error.
/// Both rows asserted whole ("x | <the list>", "z | w"). Red (78j r3): "Missing } inserted" + "Extra alignment tab
/// '&'", the row's `\\` swallowed. Repro cross-references/cleveref_format_form_keeps_an_alignment_row_end.
#[test]
fn cleveref_format_form_keeps_an_alignment_row_end() {
  let (log, html) = html_with_errors(
    repro!("cross-references/cleveref_format_form_keeps_an_alignment_row_end.tex"),
    1,
  );
  assert_eq!(
    log
      .matches("The cleveref format of the enumi (at t; line 18 col 54 - line 18 col 54) leaves 1 group(s) open")
      .count(),
    1,
    "{log}"
  );
  assert_element(
    &html,
    "tr",
    &["id=\"p1.1.1\""],
    r##"<tr id="p1.1.1" class="ltx_tr">
<th id="p1.1.1.1" class="ltx_td ltx_align_left ltx_th ltx_th_row">x</th>
<td id="p1.1.1.2" class="ltx_td ltx_align_left">
<span id="p1.1.1.2.1" class="ltx_inline-block ltx_minipage ltx_align_middle" style="width:85.4pt;">
<span id="S0.I1" class="ltx_enumerate">
<span id="S0.I1.i1" class="ltx_item" style="list-style-type:none;"><span class="ltx_tag ltx_tag_item">1.</span> 
<span id="S0.I1.i1.p1" class="ltx_para">
<span id="S0.I1.i1.p1.1" class="ltx_p">y</span>
</span></span>
</span>
</span></td></tr>"##,
  );
  assert_eq!(html.matches("<tr id=").count(), 2, "{html}");
  assert_element(
    &html,
    "tr",
    &["id=\"p1.1.2\""],
    r##"<tr id="p1.1.2" class="ltx_tr">
<th id="p1.1.2.1" class="ltx_td ltx_align_left ltx_th ltx_th_row">z</th>
<td id="p1.1.2.2" class="ltx_td ltx_align_left">w</td></tr>"##,
  );
  assert_element(
    &html,
    "p",
    &["id=\"p1.2\""],
    r##"<p id="p1.2" class="ltx_p">A <a href="#S0.E1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq.</span> <span class="ltx_text ltx_ref_tag">(1)</span></a>. B <a href="#S0.E2" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">eq.</span> <span class="ltx_text ltx_ref_tag">(2)</span></a>. C <a href="#S0.I1.i1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">It. 1</span></span></a>. D <a href="#S0.I1.i1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Item</span> <span class="ltx_text ltx_ref_tag">1</span></a>.</p>"##,
  );
}

/// The empty held tags of two captioned minipages in a figure, which become its panels (`settle_float_paragraph`) and
/// never close over their tags, are tied to the panels' labels and removed once the document is built
/// (`settle_empty_held_tags`): `\Cref{fa}`'s form error is raised (pdflatex 2 errors there, one runaway argument; we 1,
/// and cleveref's default "Figure 1"), and no `<tag/>` is left. A plain label format's `creflabel` tag, whose
/// typesetting raised only Debug messages, holds nothing and is removed where it is built. Red (78j r3): 0 errors, two
/// empty `crefformatcap` and two empty `creflabel` tags. Repro cross-references/cleveref_format_errors_in_minipage_panels.
#[test]
fn cleveref_format_errors_in_minipage_panels() {
  let tex = repro!("cross-references/cleveref_format_errors_in_minipage_panels.tex");
  let (_log, html) = html_with_errors(tex, 1);
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    r##"<p id="p1.1" class="ltx_p">A <a href="#S0.F1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Fig. 1</span></span></a>. B <a href="#S0.F2" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag"><span class="ltx_text" style="--ltx-fg-color:#0000FF;">Fig. 2</span></span></a>. C <a href="#S0.F1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Figure</span> <span class="ltx_text ltx_ref_tag">1</span></a>.</p>"##,
  );
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(!has_empty_tag(&xml), "{xml}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    r##"<figure class="ltx_figure_panel ltx_minipage" inlist="lof" labels="LABEL:fa" panelwidth="0.33" vattach="middle" width="113.8pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="crefformat"><text color="#0000FF" xml:id="S0.F1.1">Fig. 1</text></tag><tag role="creftype">fig.</tag><tag role="creftypecap">Figure</tag><tag role="creftypeplural">figs.</tag><tag role="creftypepluralcap">Figures</tag><tag role="labelcrefformat">1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.2">x</p><toccaption><tag close=" ">1</tag>left</toccaption><caption><tag close=": ">Figure 1</tag>left</caption></figure>"##,
  );
}

/// A format closing a group it did not open (`\crefformat{figure}{#2Fig.~#1#3\egroup}`; `\egroup` is a command let to
/// `}`, text) raises one Error where a reference prints it, as pdflatex's "Extra }, or forgotten \endgroup"; the
/// reference prints cleveref's default and the author's italic group closes where the author closed it. Red (78j r3): 0
/// errors (the format read as "not text", no form built). Repro
/// cross-references/cleveref_format_closing_a_group_it_did_not_open.
#[test]
fn cleveref_format_closing_a_group_it_did_not_open() {
  let (log, html) = html_with_errors(
    repro!("cross-references/cleveref_format_closing_a_group_it_did_not_open.tex"),
    1,
  );
  assert_eq!(
    log
      .matches("The cleveref format of the figure (at t; line 12 col 54 - line 12 col 54) closes a group it did not open")
      .count(),
    1,
    "{log}"
  );
  assert_element(
    &html,
    "p",
    &["id=\"p2.1\""],
    r##"<p id="p2.1" class="ltx_p"><span id="p2.1.1" class="ltx_text ltx_font_italic">still italic.</span>
Upright here.
A <a href="#S0.F1" title="" class="ltx_ref"><span class="ltx_text ltx_ref_tag">fig.</span> <span class="ltx_text ltx_ref_tag">1</span></a>.</p>"##,
  );
}

/// The count of empty held tags waiting to be settled is the job's (`logger::note_empty_held_tag`): a conversion starts
/// at none (each `initialize_report` resets it with the deferred diagnostics), whatever a paper before it on the same
/// thread left (a long-lived cortex_worker's: a paper that died before its end), and ends at none (the document's end
/// settles them all).
#[test]
fn empty_held_tags_start_at_none_on_a_reused_thread() {
  use std::rc::Rc;

  use latexml::converter::Converter;
  use latexml_core::{
    common::{Config, OutputFormat},
    util::logger::{empty_held_tags_pending, note_empty_held_tag},
  };
  let tex = repro!("cross-references/cleveref_format_errors_in_minipage_panels.tex");
  let runs = std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(move || {
      let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
      let opts = Config {
        format: OutputFormat::XML,
        include_comments: Some(false),
        preload: Some(vec![PRELOAD.to_string()]),
        bindings_dispatch: Some(Rc::new(latexml_package::dispatch)),
        extra_bindings_dispatch: Some(Rc::new(latexml_contrib::dispatch)),
        ..Config::default()
      };
      let mut runs = Vec::new();
      for _ in 0..2 {
        // (as a paper that died with empty held tags pending leaves the count)
        note_empty_held_tag();
        latexml_core::common::error::initialize_report();
        let reset = !empty_held_tags_pending();
        note_empty_held_tag();
        let mut converter = Converter::from_config(opts.clone());
        converter.prepare_session(&opts).expect("a session");
        let response = converter.convert_content_with_provenance("t.tex", tex.to_string());
        runs.push((
          reset,
          empty_held_tags_pending(),
          response.result.unwrap_or_default(),
        ));
      }
      latexml_core::reset_thread_engine();
      runs
    })
    .expect("a conversion thread")
    .join()
    .expect("the conversions");
  for (reset, pending, xml) in runs {
    assert!(reset, "the count survived initialize_report");
    assert!(!pending, "the document ended with empty held tags pending");
    assert!(xml.contains("labels=\"LABEL:fb\""), "{xml}");
    assert!(!has_empty_tag(&xml), "{xml}");
  }
}

/// An empty held tag no target settled is tied to its owner's labels only — the element holding its `ltx:tags`
/// (`settle_empty_held_tags`): an unlabelled panel is an object no reference prints, so its form's error is dropped with
/// the tag, as pdflatex never typesets that format. The section's labels (`sec`, `other`) and the outer figure's (`fo`)
/// are not the panels'; `\Cref{fo}` errs on the outer figure's own form (pdflatex 2 errors there, one runaway argument;
/// we 1, and cleveref's default "Figure 5"). Red (78j r4): 5 errors, each panel's form error replayed at a reference
/// to the section or the outer figure. Repro cross-references/cleveref_format_of_an_unlabelled_panel_raises_nothing
/// (rev78j4 m1c, m1, m2, m5).
#[test]
fn cleveref_format_of_an_unlabelled_panel_raises_nothing() {
  let tex = repro!("cross-references/cleveref_format_of_an_unlabelled_panel_raises_nothing.tex");
  let (log, html) = html_with_errors(tex, 1);
  assert_eq!(
    log
      .matches("The cleveref format of the figure (at t; line 22 col 1 - line 22 col 26) leaves 1 group(s) open")
      .count(),
    1,
    "{log}"
  );
  assert_element(
    &html,
    "p",
    &["id=\"S1.p1.1\""],
    r##"<p id="S1.p1.1" class="ltx_p">See <a href="#S1" title="1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Section</span> <span class="ltx_text ltx_ref_tag">1</span></a> and <a href="#S1" title="1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">section</span> <span class="ltx_text ltx_ref_tag">1</span></a>.</p>"##,
  );
  assert_element(
    &html,
    "p",
    &["id=\"S1.p2.1\""],
    r##"<p id="S1.p2.1" class="ltx_p">See <a href="#S1.F5" title="In 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">Figure</span> <span class="ltx_text ltx_ref_tag">5</span></a>.</p>"##,
  );
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(!has_empty_tag(&xml), "{xml}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S1.F1\""],
    r##"<figure class="ltx_figure_panel ltx_minipage" inlist="lof" panelwidth="0.33" vattach="middle" width="113.8pt" xml:id="S1.F1"><tags><tag>Figure 1</tag><tag role="crefformat"><text color="#0000FF" xml:id="S1.F1.1">Fig. 1</text></tag><tag role="creftype">fig.</tag><tag role="creftypecap">Figure</tag><tag role="creftypeplural">figs.</tag><tag role="creftypepluralcap">Figures</tag><tag role="labelcrefformat">1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S1.F1.2">x</p><toccaption><tag close=" ">1</tag>left</toccaption><caption><tag close=": ">Figure 1</tag>left</caption></figure>"##,
  );
}

/// A held tag's text that errs on its own ("Attempt to end mode", the `}` TeX inserts) raises no second, summary Error
/// for the same fault (`\lx@tag@intags@held`, as `\lx@cref@docformat`): `\renewcommand\figureautorefname{\bgroup Fig}`
/// for a `\captionof` in a tabular cell's minipage gives pdflatex's one Error ("Missing } inserted") at `\autoref{f1}`,
/// "A Fig 1.", both rows whole. Red (78j r4): 2 errors, the summary "The text of a autoref tag does not close the
/// groups it opens" beside the name's own. Repro cross-references/autoref_name_opening_a_group_errs_once (rev78j4 n9).
#[test]
fn autoref_name_opening_a_group_errs_once() {
  let (log, html) = html_with_errors(
    repro!("cross-references/autoref_name_opening_a_group_errs_once.tex"),
    1,
  );
  assert!(!log.contains("does not close the groups it opens"), "{log}");
  assert_element(
    &html,
    "tr",
    &["id=\"p1.1.1\""],
    r##"<tr id="p1.1.1" class="ltx_tr">
<td id="p1.1.1.1" class="ltx_td ltx_align_center"><span id="p1.1.1.1.1" class="ltx_inline-logical-block ltx_minipage ltx_align_middle" style="width:113.8pt;">
<span id="p1.p1" class="ltx_para">
<span id="p1.p1.1" class="ltx_p">x</span>
</span>
<span id="S0.F1" class="ltx_figure">
<span class="ltx_caption"><span class="ltx_tag ltx_tag_figure">Figure 1: </span>c</span>
</span></span></td>
<td id="p1.1.1.2" class="ltx_td ltx_align_center">m</td></tr>"##,
  );
  assert_element(
    &html,
    "tr",
    &["id=\"p1.1.2\""],
    r##"<tr id="p1.1.2" class="ltx_tr">
<td id="p1.1.2.1" class="ltx_td ltx_align_center">p</td>
<td id="p1.1.2.2" class="ltx_td ltx_align_center">q</td></tr>"##,
  );
  assert_eq!(html.matches("<tr ").count(), 2, "{html}");
  assert_element(
    &html,
    "p",
    &["id=\"p1.2\""],
    r##"<p id="p1.2" class="ltx_p">A <a href="#S0.F1" title="Figure 1" class="ltx_ref ltx_refmacro_autoref"><span class="ltx_text ltx_ref_tag">Fig 1</span></a>.</p>"##,
  );
}

/// cleveref's `\label[type]` gives its type to the object the label names, found as the kernel's `\lx@label` finds it
/// (`Document::float_to_label_scoped`, 78p): after a closed table the section, after an inner equation still in its
/// theorem the theorem, after a `\footnote` the section — their steps in scope (latex.ltx:14963-14967
/// `\@currentlabel`). pdflatex 0 errors, "Refs: appendix section 1, appendix theorem 1 and appendix section 2."; the
/// table and the equation keep their own types. Red (78j r4): the type's tags on the table and the equation just closed,
/// the references "section 1", "Theorem 1". Repro
/// cross-references/cleveref_label_type_after_a_closed_float_names_the_object_in_scope (rev78p).
#[test]
fn cleveref_label_type_after_a_closed_float_names_the_object_in_scope() {
  let tex = repro!(
    "cross-references/cleveref_label_type_after_a_closed_float_names_the_object_in_scope.tex"
  );
  let (_log, html) = html_with_errors(tex, 0);
  assert_element(
    &html,
    "p",
    &["id=\"S2.p2.1\""],
    r##"<p id="S2.p2.1" class="ltx_p">Refs: <a href="#S1" title="1 One" class="ltx_ref"><span class="ltx_text ltx_ref_tag">appendix section</span> <span class="ltx_text ltx_ref_tag">1</span></a>, <a href="#Thmtheorem1" title="Theorem 1. ‣ 1 One" class="ltx_ref"><span class="ltx_text ltx_ref_tag">appendix theorem</span> <span class="ltx_text ltx_ref_tag">1</span></a> and <a href="#S2" title="2 Two" class="ltx_ref"><span class="ltx_text ltx_ref_tag">appendix section</span> <span class="ltx_text ltx_ref_tag">2</span></a>.</p>"##,
  );
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "table",
    &["xml:id=\"S1.T1\""],
    r##"<table inlist="lot" xml:id="S1.T1"><tags><tag>Table 1</tag><tag role="creftype">table</tag><tag role="creftypecap">Table</tag><tag role="creftypeplural">tables</tag><tag role="creftypepluralcap">Tables</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>T</toccaption><caption><tag close=": ">Table 1</tag>T</caption></table>"##,
  );
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S1.E1\""],
    r##"<equation xml:id="S1.E1"><tags><tag>(1)</tag><tag role="creflabel">(1)</tag><tag role="creftype">eq.</tag><tag role="creftypecap">Equation</tag><tag role="creftypeplural">eqs.</tag><tag role="creftypepluralcap">Equations</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="c=d" text="c = d" xml:id="S1.E1.m1"><XMath xml:id="S1.E1.m1.1"><XMApp xml:id="S1.E1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok><XMTok font="italic" role="UNKNOWN">d</XMTok></XMApp></XMath></Math></equation>"##,
  );
  assert_element(
    &xml,
    "note",
    &["xml:id=\"footnote1\""],
    r##"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="creftype">footnote</tag><tag role="creftypecap">Footnote</tag><tag role="creftypeplural">footnotes</tag><tag role="creftypepluralcap">Footnotes</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>note</note>"##,
  );
}
