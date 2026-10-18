//! Red/green guards for batch 73a: the binding shapes of the rc_struct6 study (agent_reports/2026-10-17_rc_struct6.md)
//! and the two user rulings of 2026-10-18 — an `\index`/`\nomenclature` entry is typeset only where its list is printed
//! (OXIDIZED_DESIGN_DIVERGENCES #738), and aastex701's `\tablecaption` before `\begin{deluxetable}` is kept (#739).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The conversion of a repro, which must be clean (0 errors, 0 warnings, no Fatal); returns the XML.
fn clean(tex: &str) -> String {
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.lines().any(|l| l.starts_with("Fatal:")), "{log}");
  xml
}

/// aastex701.cls:12656-12661 `\dataset[text]{url}` is `\href{url}{text}` (2502.04486, 2608.18221, 1601.05101).
#[test]
fn aastex_dataset_links_its_url() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aastex_dataset_links_its_url.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">The code is on <ref class="ltx_href" href="https://github.com/zfried/astro_amase">GitHub</ref> and<ref class="ltx_href" href="http://sha.ipac.caltech.edu/x?id=1&amp;RequestClass=ServerRequest">SHA</ref>.</p>"#,
  );
}

/// A `\tablecaption` ahead of `\begin{deluxetable*}` is kept on the table (user ruling 2026-10-18, OD #739; 1010.4329,
/// 1903.10014), unnumbered: the PDF prints no caption there, so the table takes no number (main-session call, review
/// 73a r2 finding 5). The `\label` inside it, which pdflatex never runs, names the table: the caption takes an id step
/// (review 73a r3 finding 2, main-session call 2026-10-18), and no label lands on the document.
#[test]
fn aastex_tablecaption_before_deluxetable() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aastex_tablecaption_before_deluxetable.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[],
    "<table labels=\"LABEL:tab1\" xml:id=\"S0.Tx1\"><caption>Frequencies for KIC\u{a0}11026764.</caption><tabular xml:id=\"S0.Tx1.1\"><thead><tr xml:id=\"S0.Tx1.1.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.Tx1.1.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.Tx1.1.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.Tx1.1.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.Tx1.1.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S0.Tx1.1.2.2\">2</td></tr></tbody></tabular></table>",
  );
  assert_eq!(xml.matches("<table ").count(), 1, "{xml}");
  assert!(xml.contains("labels=\"LABEL:tab1\""), "{xml}");
  assert!(
    !xml.contains("<document xmlns=\"http://dlmf.nist.gov/LaTeXML\" labels"),
    "{xml}"
  );
}

/// aastex701.cls:6726-6732: without `longbib` a reference's `\bibinfo{title}` is dropped (2508.20177, 2510.05388).
#[test]
fn aastex7_bibinfo_needs_longbib() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aastex7_bibinfo_needs_longbib.tex"
  ));
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> Labbe, I. 2024, ApJ, 978, 92\n</bibblock>",
  );
  let (log, xml) = convert_counted(
    "\\documentclass[longbib]{aastex701}\n\\begin{document}\nSee \\citet{L24}.\n\\begin{thebibliography}{}\n\
     \\bibitem[{Labbe}(2024)]{L24} Labbe, I. 2024, \\bibinfo{title}{UNCOVER: Red AGN,} ApJ, 978, 92\n\
     \\end{thebibliography}\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "bibblock",
    &[],
    "<bibblock> Labbe, I. 2024, UNCOVER: Red AGN, ApJ, 978, 92\n</bibblock>",
  );
}

/// subcaption.sty:262-276 provides `\subfloat` at `\begin{document}`: a preamble's own stands (2511.09162).
#[test]
fn subcaption_subfloat_is_provided_at_begin_document() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/subcaption_subfloat_provided_at_begin_document.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &[],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%" xml:id="S0.F1.1">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p xml:id="S0.F1.2"><text cssstyle="padding:3.0pt" framecolor="#000000" framed="rectangle" xml:id="S0.F1.2.1">A</text></p><toccaption><tag close=" ">1</tag>Two panels.</toccaption><caption><tag close=": "><text fontsize="90%" xml:id="S0.F1.3">Figure 1</text></tag><text fontsize="90%" xml:id="S0.F1.4">Two panels.</text></caption></figure>"##,
  );
}

/// revtex4.cls:6030-6032 lets `\bm` be its own only at `\begin{document}`, if undefined (nucl-th0307033).
#[test]
fn revtex_bm_is_provided_at_begin_document() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/revtex_bm_provided_at_begin_document.tex"
  ));
  assert_element(
    &xml,
    "bibitem",
    &[],
    r#"<bibitem key="kelly_5" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="refnum">[1]</tag><tag role="key">kelly_5</tag></tags><bibblock> J. Kelly, Phys. Rev. (1995).</bibblock></bibitem>"#,
  );
  // Without a preamble definition, REVTeX's `\bm` is there in the document.
  let (log, xml) = convert_counted(
    "\\documentclass[aps]{revtex4}\n\\begin{document}\n$\\bm{x}$\n\\end{document}\n",
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert!(xml.contains(r#"font="bold"#), "{xml}");
}

/// elsarticle.cls has no `{thm}`: the author's `\newcommand{\thm}` stands (2212.01438); elsart keeps it.
#[test]
fn elsarticle_has_no_thm_environment() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/elsarticle_has_no_thm_environment.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    "<p xml:id=\"p1.1\">See Theorem\u{a0}<ref labelref=\"LABEL:theorem:town_th\"/>.</p>",
  );
  let xml = clean(
    "\\documentclass{elsart}\n\\begin{document}\n\\begin{thm}A.\\end{thm}\n\\end{document}\n",
  );
  assert!(
    xml.contains(r#"<theorem class="ltx_theorem_thm" inlist="thm theorem:thm" xml:id="Thmthm1">"#),
    "{xml}"
  );
}

/// pst-node.tex:1163-1192 `{psmatrix}` is an alignment of boxed nodes (0705.2689, 1712.01407).
#[test]
fn psmatrix_is_an_alignment() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/psmatrix_is_an_alignment.tex"
  ));
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_markedasmath" vattach="bottom" xml:id="S0.Ex1.1"><tr xml:id="S0.Ex1.1.1"><td class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.1.1.1"/><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.1.1.2"><text xml:id="S0.Ex1.1.1.2.1"><Math mode="inline" tex="\displaystyle b" text="b" xml:id="S0.Ex1.m1.m1"><XMath xml:id="S0.Ex1.m1.m1.1"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></text></td><td class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.1.1.3"/></tr><tr xml:id="S0.Ex1.1.2"><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.1.2.1"><text xml:id="S0.Ex1.1.2.1.1"><Math mode="inline" tex="\displaystyle x" text="x" xml:id="S0.Ex1.m1.m2"><XMath xml:id="S0.Ex1.m1.m2.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></text></td><td class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.1.2.2"/><td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.1.2.3"><text xml:id="S0.Ex1.1.2.3.1"><Math mode="inline" tex="\displaystyle z" text="z" xml:id="S0.Ex1.m1.m3"><XMath xml:id="S0.Ex1.m1.m3.1"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math></text></td></tr></tabular>"#,
  );
}

/// pst-node.tex:206-220 `\cnodeput[par](x,y){name}{stuff}` reads its coordinate (cs0702114).
#[test]
fn cnodeput_reads_its_coordinate() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/cnodeput_reads_its_coordinate.tex"
  ));
  assert_element(
    &xml,
    "picture",
    &[],
    r#"<picture fill="none" height="113.81pt" origin-x="-85.36pt" origin-y="-56.91pt" stroke="none" unitlength="28.45pt" width="170.72pt" xml:id="p1.pic1"><g transform="translate(118.11,78.74)"><g transform="translate(-78.74,-39.37)"><text xml:id="p1.pic1.1"/></g><g transform="translate(78.74,-39.37)"><text xml:id="p1.pic1.2"/></g></g></picture>"#,
  );
}

/// ieeeaccess.cls:270-300 `\Figure` is a figure with its image and caption (1803.04935, 1910.07402).
#[test]
fn ieeeaccess_figure_is_a_figure() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeeaccess_figure_is_a_figure.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &[],
    r#"<figure inlist="lof" labels="LABEL:block_diagram" placement="t!" xml:id="S0.F1"><tags><tag>Fig. 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Fig. 1</tag></tags><graphics candidates="../../usr/local/texlive/2025/texmf-dist/tex/latex/mwe/example-image-a.png" class="ltx_centering" graphic="example-image-a.png" options="scale=0.1" xml:id="S0.F1.g1"/><toccaption class="ltx_centering"><tag close=" ">1</tag>Block diagram of the passband system.</toccaption><caption class="ltx_centering"><tag close=": ">Fig. 1</tag>Block diagram of the passband system.</caption></figure>"#,
  );
}

/// sigma.cls:20-21 passes `fleqn` to article and amsmath: a `$` in `\[…\]` is text between two formulas (2111.06645).
#[test]
fn sigma_passes_fleqn() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/sigma_passes_fleqn.tex"
  ));
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_fleqn">"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "XMText",
    &[],
    "<XMText xml:id=\"S0.Ex1.m1.1.4.3.3\"> and\u{a0}</XMText>",
  );
}

/// graphics.sty:116-117 `\DeclareGraphicsExtensions` records its list (1810.02483).
#[test]
fn declare_graphics_extensions_is_not_typeset() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/declare_graphics_extensions_not_typeset.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text.</p>"#,
  );
  // epsfig.sty defines none: graphics.sty's macro stands under it too (review 73a finding 4, epsfig_sty.rs).
  let xml = clean(
    "\\documentclass{article}\n\\usepackage{epsfig}\n\\DeclareGraphicsExtensions{.pdf,.pdf_tex}\n     \\begin{document}\nText.\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text.</p>"#,
  );
}

/// `\bibstyle`'s name is not typeset (2412.10240).
#[test]
fn bibstyle_name_is_not_typeset() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/bibstyle_name_not_typeset.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text.</p>"#,
  );
}

/// elsart.cls:600 `{subbibitems}` takes the group's citation key (0707.3045).
#[test]
fn elsart_subbibitems_reads_its_key() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/elsart_subbibitems_reads_its_key.tex"
  ));
  assert_element(
    &xml,
    "biblist",
    &[],
    r#"<biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A. Author, J. 1 (2000).</bibblock></bibitem><bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> B. Author, J. 2 (2001).</bibblock></bibitem></biblist>"#,
  );
}

/// svjour2.cls:1032 `\tableheadseprule` is a `\noalign` rule (astro-ph0609697).
#[test]
fn sv_tableheadseprule_is_a_noalign_rule() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/sv_tableheadseprule_is_a_noalign_rule.tex"
  ));
  assert_element(
    &xml,
    "tabular",
    &[],
    r#"<tabular class="ltx_guessed_headers" vattach="middle" xml:id="S0.T1.1"><thead><tr xml:id="S0.T1.1.1"><td align="left" border="t" cssstyle="padding-bottom: 2.0pt" thead="column row" xml:id="S0.T1.1.1.1">A</td><td align="left" border="t" cssstyle="padding-bottom: 2.0pt" thead="column" xml:id="S0.T1.1.1.2">B</td><td align="left" border="t" cssstyle="padding-bottom: 2.0pt" thead="column" xml:id="S0.T1.1.1.3">C</td></tr></thead><tbody><tr xml:id="S0.T1.1.2"><td align="left" border="b t" thead="row" xml:id="S0.T1.1.2.1">0.0</td><td align="left" border="b t" xml:id="S0.T1.1.2.2">1865</td><td align="left" border="b t" xml:id="S0.T1.1.2.3">1965</td></tr></tbody></tabular>"#,
  );
}

/// mdframed's options are keys, not text (2408.09945); the one Warning is the binding's standing notice.
#[test]
fn mdframed_options_are_not_typeset() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/mdframed_options_not_typeset.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Warning:missing_file:mdframed.sty"), "{log}");
  assert!(!xml.contains("chinese"), "{xml}");
  assert!(xml.contains("Text."), "{xml}");
}

/// An `\index` entry with no printed index is never typeset (user ruling 2026-10-18, OD #738; 1112.3972).
#[test]
fn index_entry_is_typeset_only_when_printed() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/index/index_entry_typeset_only_when_printed.tex"
  ));
  assert!(!xml.contains("<indexmark"), "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Use <text font="typewriter" xml:id="p1.1.1">SELF_HEALING</text> policy.</p>"#,
  );
}

/// A printed index keeps its entries, each mark where its `\index` stands — the ones after `\printindex` too.
#[test]
fn printed_index_keeps_its_entries() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/index/printed_index_keeps_its_entries.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Alpha<indexmark><indexphrase key="alpha">alpha</indexphrase></indexmark> and beta<indexmark><indexphrase key="beta">beta</indexphrase><indexphrase key="sub">sub</indexphrase></indexmark>.</p>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">Gamma<indexmark><indexphrase key="gamma">gamma</indexphrase></indexmark>.</p>"#,
  );
  assert_eq!(xml.matches("<index ").count(), 1, "{xml}");
}

/// A nomenclature entry is typeset only where `\printnomenclature` prints the list, before the entries too; one that
/// makeindex rejects is left out with a Warning (2506.02168).
#[test]
fn nomenclature_entry_is_typeset_only_when_printed() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/index/nomenclature_entry_typeset_only_when_printed.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("makeindex rejects the nomenclature entry \\|\\cdot\\|_p (Extra `|')"),
    "{log}"
  );
  assert_eq!(xml.matches("<glossarydefinition").count(), 1, "{xml}");
  assert_element(
    &xml,
    "glossaryphrase",
    &[r#"role="description""#],
    r#"<glossaryphrase key="nomencl.1" role="description">The area</glossaryphrase>"#,
  );
  // Without `\printnomenclature` nothing is typeset.
  let xml = clean(
    "\\documentclass{article}\n\\usepackage{nomencl}\n\\makenomenclature\n\\begin{document}\n\
     Norm. \\nomenclature{\\|\\cdot\\|_p}{$L^p$ norm}\n\\end{document}\n",
  );
  assert!(!xml.contains("<glossarydefinition"), "{xml}");
}

/// Review 73a finding 1 (repro tablecaption_inside_the_table_wins): a caption inside the deluxetable wins over a stored one (aastex701.cls:11303-11304
/// empties the store at the table's start), and the table float after it is Table 2.
#[test]
fn tablecaption_inside_the_table_wins() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/tablecaption_inside_the_table_wins.tex"
  ));
  assert_eq!(xml.matches("<caption>").count(), 2, "{xml}");
  assert!(!xml.contains("Outer stray caption"), "{xml}");
  assert_element(
    &xml,
    "caption",
    &[],
    r#"<caption><tag close=": ">Table 1</tag>Inner real caption.</caption>"#,
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T2""#],
    "<table inlist=\"lot\" xml:id=\"S0.T2\"><tags><tag>Table 2</tag><tag role=\"autoref\">Table\u{a0}2<text xml:id=\"S0.T2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><toccaption><tag close=\" \">2</tag>In a table float.</toccaption><caption><tag close=\": \">Table 2</tag>In a table float.</caption><tabular vattach=\"middle\" xml:id=\"S0.T2.2\"><tbody><tr xml:id=\"S0.T2.2.1\"><td align=\"center\" xml:id=\"S0.T2.2.1.1\">3</td><td align=\"center\" xml:id=\"S0.T2.2.1.2\">4</td></tr></tbody></tabular></table>",
  );
}

/// Review 73a finding 1 (repro stored_tablecaption_scoped_to_its_group): AASTeX 6+ scope a stored `\tablecaption` to its group, so a caption a group closed on
/// is not set on the next, unrelated, deluxetable (OD #739).
#[test]
fn stored_tablecaption_scoped_to_its_group() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/stored_tablecaption_scoped_to_its_group.tex"
  ));
  assert!(!xml.contains("Stray caption"), "{xml}");
  assert_eq!(xml.matches("<table ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "table",
    &[],
    "<table xml:id=\"tab1\"><tabular xml:id=\"tab1.1\"><thead><tr xml:id=\"tab1.1.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"tab1.1.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"tab1.1.1.2\">B</td></tr></thead><tbody><tr xml:id=\"tab1.1.2\"><td align=\"center\" border=\"b t\" xml:id=\"tab1.1.2.1\">5</td><td align=\"center\" border=\"b t\" xml:id=\"tab1.1.2.2\">6</td></tr></tbody></tabular></table>",
  );
  // emulateapj's store is global, as the class's (emulateapj.cls:1217-1218): the next deluxetable prints it (1010.4329).
  let xml = clean(
    "\\documentclass{emulateapj}\n\\begin{document}\n\\begin{center}\\tablecaption{Kept.}\\end{center}\n\
     \\begin{deluxetable}{cc}\n\\tablehead{\\colhead{A} & \\colhead{B}}\n\\startdata\n1 & 2 \\\\\n\\enddata\n\
     \\end{deluxetable}\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "table",
    &[],
    "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Kept.</toccaption><caption><tag>Table 1</tag>Kept.</caption><tabular xml:id=\"S0.T1.2\"><thead><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.2\">2</td></tr></tbody></tabular></table>",
  );
}

/// 2204.03327: an aastex `{longrotatetable}` around a plain `tabular` is that table's float, its stored `\tablecaption`
/// set at the `\begin{tabular}`, unnumbered (the PDF never prints it; review 73a r2 finding 5), its `\label` the last
/// numbered unit's, as pdflatex's aux has it (2204.03327: `tab:allesFitted` = 4.1.5). The longrotatetable leaves the
/// next deluxetable a long one (aastex631.cls `\global\startlongtabletrue`, reset only by a long table's end), whose
/// caption the class always prints: "Table 1", empty, as pdflatex (TL 2025 aastex701.cls); around a deluxetable it is the page's
/// setting (main-session call 2026-10-18).
#[test]
fn longrotatetable_around_a_tabular_is_its_table() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/longrotatetable_around_a_tabular_is_its_table.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="tab1""#],
    "<table xml:id=\"tab1\"><caption>Caption of the plain tabular.</caption><tabular vattach=\"middle\" xml:id=\"tab1.1\"><tbody><tr xml:id=\"tab1.1.1\"><td align=\"center\" xml:id=\"tab1.1.1.1\">1</td><td align=\"center\" xml:id=\"tab1.1.1.2\">2</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag></toccaption><caption><tag close=\": \">Table 1</tag></caption><tabular xml:id=\"S0.T1.2\"><thead><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.1\">5</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.2\">6</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T2""#],
    "<table inlist=\"lot\" xml:id=\"S0.T2\"><tags><tag>Table 2</tag><tag role=\"autoref\">Table\u{a0}2<text xml:id=\"S0.T2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><toccaption><tag close=\" \">2</tag>A long deluxetable.</toccaption><caption><tag close=\": \">Table 2</tag>A long deluxetable.</caption><tabular xml:id=\"S0.T2.2\"><thead><tr xml:id=\"S0.T2.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T2.2.1.1\">C</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T2.2.1.2\">D</td></tr></thead><tbody><tr xml:id=\"S0.T2.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T2.2.2.1\">7</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T2.2.2.2\">8</td></tr></tbody></tabular></table>",
  );
  assert_eq!(xml.matches("<table ").count(), 3, "{xml}");
}

/// revtex4-1.cls:7565 (revtex4-2.cls:7586) defines `\bm` at load (review 73a finding 2, repro revtex41_bm_defined_at_load): Perl's bold `\bm`
/// (revtex4_support.sty.ltxml:167) stays, the author's `\newcommand` ignored with LaTeXML's Info (pdflatex errs and its
/// `\REV@bm` gobbles the argument, "Text .").
#[test]
fn revtex41_bm_is_defined_at_load() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/revtex41_bm_defined_at_load.tex"
  ));
  assert!(!xml.contains("AUTHOR"), "{xml}");
  assert_element(
    &xml,
    "XMTok",
    &[],
    r#"<XMTok font="bold italic" role="UNKNOWN">x</XMTok>"#,
  );
}

/// pst-node.tex:1163/:1191-1192: `\psmatrix … \endpsmatrix` is a group, its `\\` does not outlive it (review 73a
/// finding 3).
#[test]
fn psmatrix_is_a_group() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/psmatrix_is_a_group.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p3.1""#],
    r#"<p xml:id="p3.1">Line one<break/>line two.</p>"#,
  );
}

/// latex.ltx:17737-17741: a `\glossary` entry is typeset only where the glossary is printed (main-session call
/// 2026-10-18 under user ruling 1, OD #738).
#[test]
fn glossary_entry_is_typeset_only_when_printed() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/index/glossary_entry_typeset_only_when_printed.tex"
  ));
  assert!(!xml.contains("<indexmark"), "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text one. Text two.</p>"#,
  );
  // Printed (the `.gls` stand-in), the entry is a `glo` mark where it was made.
  let xml = clean(
    "\\documentclass{article}\n\\makeglossary\n\\newenvironment{theglossary}{}{}\n\\begin{document}\n\
     Text one\\glossary{alpha}. Text two.\n\\makeatletter\\@input@{\\jobname.gls}\\makeatother\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Text one<indexmark inlist="glo"><indexphrase key="alpha">alpha</indexphrase></indexmark>. Text two.</p>"#,
  );
}

/// Review 73a finding 8: the "list already printed" path — the index and glossary printed before their entries, each
/// entry typeset where it is made, the marks those of `index_entry_ampersand_is_literal` (batch59).
#[test]
fn index_entry_ampersand_is_literal_list_printed_first() {
  fn marks(xml: &str) -> Vec<String> {
    let flat: String = xml.lines().map(str::trim).collect();
    let mut out = Vec::new();
    let mut rest = flat.as_str();
    while let Some(start) = rest.find("<indexmark") {
      let tail = &rest[start..];
      let end = tail.find("</indexmark>").expect("closed indexmark") + "</indexmark>".len();
      let mark = &tail[..end];
      // ids aside
      let mut plain = String::new();
      let mut segment = mark;
      while let Some(at) = segment.find(" xml:id=\"") {
        plain.push_str(&segment[..at]);
        let after = &segment[at + " xml:id=\"".len()..];
        segment = &after[after.find('"').expect("closed id") + 1..];
      }
      plain.push_str(segment);
      out.push(plain);
      rest = &tail[end..];
    }
    out
  }
  let first = clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/index_entry_ampersand_is_literal_list_printed_first.tex"
  ));
  let last = clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/index_entry_ampersand_is_literal.tex"
  ));
  assert_eq!(marks(&first).len(), 7, "{first}");
  assert_eq!(marks(&first), marks(&last));
  assert_element(
    &first,
    "p",
    &[r#"xml:id="p1.1""#],
    r##"<p xml:id="p1.1">G: Text.<indexmark inlist="glo"><indexphrase key="B&amp;Borel subgroup"><Math mode="inline" tex="B" text="B" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="italic" role="UNKNOWN">B</XMTok></XMath></Math>&amp;Borel subgroup</indexphrase></indexmark></p>"##,
  );
  assert_element(
    &first,
    "tabular",
    &[r#"xml:id="p5.1""#],
    r##"<tabular vattach="middle" xml:id="p5.1"><tbody><tr xml:id="p5.1.1"><td align="center" xml:id="p5.1.1.1"><text font="bold" xml:id="p5.1.1.1.1">e<indexmark><indexphrase key="y&amp;z">y&amp;z</indexphrase></indexmark>X</text></td><td align="center" xml:id="p5.1.1.2">f</td></tr></tbody></tabular>"##,
  );
}

/// nomencl.sty:228-232: in table mode the unit and the note are written too, so a `|` there makes makeindex reject the
/// entry, left out with a Warning (review 73a finding 9).
#[test]
fn nomentbl_unit_with_a_bar_is_rejected() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{nomentbl}\n\\makenomenclature\n\\begin{document}\n\
     Area\\nomenclature{$A$}{The area}{m$^2$}{}. Speed\\nomenclature{$v$}{The speed}{m|s}{}.\n\
     \\printnomenclature\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("makeindex rejects the nomenclature entry $v$ (Extra `|')"),
    "{log}"
  );
  assert_eq!(xml.matches("<glossarydefinition").count(), 1, "{xml}");
}

/// 1903.10014 (review 73a r2 finding 1, the blocker): in a longtable a `\tablecaption` is the caption only while the
/// longtable has none — after a `\caption` it is stored, never printed there (aastex.cls:1280/:1507; aastex701.cls
/// :8418), so the printed caption stays; alone it is the longtable's numbered caption, tag and list entry, as the PDF
/// counts the longtable (review 73a r3 finding 4; OD #739/#740).
#[test]
fn longtable_keeps_its_printed_caption() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/longtable_keeps_its_printed_caption.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" labels=\"LABEL:tab:main\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><caption><tag close=\": \">Table 1</tag>Printed caption.</caption><toccaption><tag close=\" \">1</tag>Printed caption.</toccaption><tabular xml:id=\"S0.T1.2\"><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" xml:id=\"S0.T1.2.1.2\">B</td></tr><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" xml:id=\"S0.T1.2.2.1\">1</td><td align=\"center\" xml:id=\"S0.T1.2.2.2\">2</td></tr></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T2""#],
    "<table inlist=\"lot\" xml:id=\"S0.T2\"><tags><tag>Table 2</tag><tag role=\"autoref\">Table\u{a0}2<text xml:id=\"S0.T2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><caption><tag close=\": \">Table 2</tag>Only a table caption.</caption><toccaption><tag close=\" \">2</tag>Only a table caption.</toccaption><tabular xml:id=\"S0.T2.2\"><tr xml:id=\"S0.T2.2.1\"><td align=\"center\" xml:id=\"S0.T2.2.1.1\">C</td><td align=\"center\" xml:id=\"S0.T2.2.1.2\">D</td></tr></tabular></table>",
  );
  assert!(!xml.contains("Stored never printed"), "{xml}");
}

/// emulateapj.cls:1293-1294 stores `\tablecaption` globally, never emptied, and every table's head prints the store
/// (:1325/:1333): a table without a caption of its own repeats the last one, the third table is Table 3, and each caption
/// is above its tabular, as pdflatex (review 73a r2 findings 3 and 4; 1010.4329; OD #740).
#[test]
fn emulateapj_stored_caption_repeats() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/emulateapj_stored_caption_repeats.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Stored One.</toccaption><caption><tag>Table 1</tag>Stored One.</caption><tabular xml:id=\"S0.T1.2\"><thead><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.2\">2</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T2""#],
    "<table inlist=\"lot\" placement=\"ht\" xml:id=\"S0.T2\"><tags><tag>Table 2</tag><tag role=\"autoref\">Table\u{a0}2<text xml:id=\"S0.T2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><toccaption><tag close=\" \">2</tag>Stored One.</toccaption><caption><tag>Table 2</tag>Stored One.</caption><tabular xml:id=\"S0.T2.2\"><thead><tr xml:id=\"S0.T2.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T2.2.1.1\">C</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T2.2.1.2\">D</td></tr></thead><tbody><tr xml:id=\"S0.T2.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T2.2.2.1\">3</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T2.2.2.2\">4</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T3""#],
    "<table inlist=\"lot\" xml:id=\"S0.T3\"><tags><tag>Table 3</tag><tag role=\"autoref\">Table\u{a0}3<text xml:id=\"S0.T3.1\"/></tag><tag role=\"refnum\">3</tag><tag role=\"typerefnum\">Table 3</tag></tags><toccaption><tag close=\" \">3</tag>Inner Three.</toccaption><caption><tag>Table 3</tag>Inner Three.</caption><tabular xml:id=\"S0.T3.2\"><thead><tr xml:id=\"S0.T3.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T3.2.1.1\">E</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T3.2.1.2\">F</td></tr></thead><tbody><tr xml:id=\"S0.T3.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T3.2.2.1\">5</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T3.2.2.2\">6</td></tr></tbody></tabular></table>",
  );
  // An empty store prints the number alone (pdflatex "TABLE 1", no text); a `\tablecaption` given after the head, in
  // the data, is then the table's caption, after the tabular (OD #664; 1104.2619).
  let xml = clean(
    "\\documentclass{emulateapj}\n\\begin{document}\n\\begin{deluxetable}{cc}\n\\tablehead{\\colhead{A} & \\colhead{B}}\n\
     \\tablecaption{Given after the head.}\n\\startdata\n1 & 2 \\\\\n\\enddata\n\\end{deluxetable}\n\\end{document}\n",
  );
  assert_element(
    &xml,
    "table",
    &[],
    "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><tabular xml:id=\"S0.T1.2\"><thead><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.2\">2</td></tr></tbody></tabular><toccaption><tag close=\" \">1</tag>Given after the head.</toccaption><caption><tag>Table 1</tag>Given after the head.</caption></table>",
  );
}

/// A `\tablecaption` directly in an aastex `{longrotatetable}` is stored and the deluxetable in it takes it — numbered,
/// as the long table prints its caption in the PDF ("Table 1.", empty) — with its `\label`; no caption paragraph, no
/// label on the document (review 73a r2 finding 2; 2204.03327; OD #739).
#[test]
fn longrotatetable_tablecaption_goes_to_its_table() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/longrotatetable_tablecaption_goes_to_its_table.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" labels=\"LABEL:tx\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Caption outside the deluxetable.</toccaption><caption><tag close=\": \">Table 1</tag>Caption outside the deluxetable.</caption><tabular xml:id=\"S0.T1.2\"><thead><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T1.2.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T1.2.2.2\">2</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T2""#],
    "<table inlist=\"lot\" labels=\"LABEL:ty\" xml:id=\"S0.T2\"><tags><tag>Table 2</tag><tag role=\"autoref\">Table\u{a0}2<text xml:id=\"S0.T2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Table 2</tag></tags><toccaption><tag close=\" \">2</tag>Caption inside.</toccaption><caption><tag close=\": \">Table 2</tag>Caption inside.</caption><tabular xml:id=\"S0.T2.2\"><thead><tr xml:id=\"S0.T2.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T2.2.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T2.2.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S0.T2.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T2.2.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T2.2.2.2\">2</td></tr></tbody></tabular></table>",
  );
  assert!(!xml.contains("ltx_caption"), "{xml}");
  assert!(
    !xml.contains("<document xmlns=\"http://dlmf.nist.gov/LaTeXML\" labels"),
    "{xml}"
  );
}

/// A caption only the surpass keeps (OD #739) is unnumbered, so the later numbers and `\ref`s are the PDF's: the table
/// float is Table 1 and `\ref` prints 1, as pdflatex (review 73a r2 finding 5; 2204.03327's `tab:wasp43_res` = 3).
#[test]
fn kept_tablecaption_is_unnumbered() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/kept_tablecaption_is_unnumbered.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="tab1""#],
    "<table placement=\"h\" xml:id=\"tab1\"><caption>Second stray.</caption><tabular xml:id=\"tab1.1\"><thead><tr xml:id=\"tab1.1.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"tab1.1.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"tab1.1.1.2\">B</td></tr></thead><tbody><tr xml:id=\"tab1.1.2\"><td align=\"center\" border=\"b t\" xml:id=\"tab1.1.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"tab1.1.2.2\">2</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" labels=\"LABEL:r\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Real.</toccaption><caption><tag close=\": \">Table 1</tag>Real.</caption><tabular vattach=\"middle\" xml:id=\"S0.T1.2\"><tbody><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" xml:id=\"S0.T1.2.1.1\">3</td><td align=\"center\" xml:id=\"S0.T1.2.1.2\">4</td></tr></tbody></tabular></table>",
  );
  assert!(!xml.contains("First stray"), "{xml}");
  // the kept caption is taken once: the last deluxetable has none
  assert_eq!(xml.matches("<caption").count(), 2, "{xml}");
}

/// Review 73a r3 finding 1: a `\tablecaption` in an aastex `{longrotatetable}` that no `\begin{tabular}` takes — a
/// `tabular*`, a `longtable`, or a caption after the tabular — is set at the longrotatetable's end, in its group,
/// unnumbered; the `\label` inside the first names its table; the longtable keeps its own number and id; the
/// deluxetable after it is Table 2, as pdflatex (which prints none of the three captions; OD #739).
#[test]
fn longrotatetable_caption_without_a_tabular() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/longrotatetable_caption_without_a_tabular.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.Tx1""#],
    "<table labels=\"LABEL:ts\" xml:id=\"S0.Tx1\"><tabular vattach=\"middle\" xml:id=\"S0.Tx1.1\"><tbody><tr xml:id=\"S0.Tx1.1.1\"><td align=\"center\" xml:id=\"S0.Tx1.1.1.1\">A</td><td align=\"center\" xml:id=\"S0.Tx1.1.1.2\">B</td></tr></tbody></tabular><caption>Star caption.</caption></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><tabular xml:id=\"S0.T1.2\"><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" xml:id=\"S0.T1.2.1.1\">C</td><td align=\"center\" xml:id=\"S0.T1.2.1.2\">D</td></tr></tabular><caption>Longtable caption.</caption></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="tab1""#],
    "<table xml:id=\"tab1\"><tabular vattach=\"middle\" xml:id=\"tab1.1\"><tbody><tr xml:id=\"tab1.1.1\"><td align=\"center\" xml:id=\"tab1.1.1.1\">E</td><td align=\"center\" xml:id=\"tab1.1.1.2\">F</td></tr></tbody></tabular><caption>After the tabular.</caption></table>",
  );
  assert!(
    xml.contains(r#"<caption><tag close=": ">Table 2</tag>Real one.</caption>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<table ").count(), 4, "{xml}");
}

/// Review 73a r3 finding 2 (main-session call 2026-10-18): a caption kept unnumbered takes an id step, so the `\label`
/// inside it names its table (pdflatex never runs it, "See ?? and 1."); no label lands on the section, and the next
/// table is Table 1 with its own label.
#[test]
fn kept_caption_label_names_the_table() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/kept_caption_label_names_the_table.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S1.Tx1""#],
    "<table labels=\"LABEL:tab:k\" xml:id=\"S1.Tx1\"><caption>Kept caption.</caption><tabular xml:id=\"S1.Tx1.1\"><thead><tr xml:id=\"S1.Tx1.1.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S1.Tx1.1.1.1\">A</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S1.Tx1.1.1.2\">B</td></tr></thead><tbody><tr xml:id=\"S1.Tx1.1.2\"><td align=\"center\" border=\"b t\" xml:id=\"S1.Tx1.1.2.1\">1</td><td align=\"center\" border=\"b t\" xml:id=\"S1.Tx1.1.2.2\">2</td></tr></tbody></tabular></table>",
  );
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S1.T1""#],
    "<table inlist=\"lot\" labels=\"LABEL:tab:n\" xml:id=\"S1.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S1.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>Numbered.</toccaption><caption><tag close=\": \">Table 1</tag>Numbered.</caption><tabular xml:id=\"S1.T1.2\"><thead><tr xml:id=\"S1.T1.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S1.T1.2.1.1\">C</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S1.T1.2.1.2\">D</td></tr></thead><tbody><tr xml:id=\"S1.T1.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S1.T1.2.2.1\">3</td><td align=\"center\" border=\"b t\" xml:id=\"S1.T1.2.2.2\">4</td></tr></tbody></tabular></table>",
  );
  assert!(!xml.contains("<section inlist=\"toc\" labels"), "{xml}");
}

/// Review 73a r3 finding 4: an AASTeX 6+ longtable whose only caption is a `\tablecaption` has the longtable's numbered
/// caption, tag and list entry; the deluxetable after it is Table 2 ("See 1 and 2.", as pdflatex).
#[test]
fn longtable_tablecaption_is_numbered() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/longtable_tablecaption_is_numbered.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    "<table inlist=\"lot\" labels=\"LABEL:tab:lo\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"autoref\">Table\u{a0}1<text xml:id=\"S0.T1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><caption><tag close=\": \">Table 1</tag>Only tablecaption.</caption><toccaption><tag close=\" \">1</tag>Only tablecaption.</toccaption><tabular xml:id=\"S0.T1.2\"><tr xml:id=\"S0.T1.2.1\"><td align=\"center\" xml:id=\"S0.T1.2.1.1\">A</td><td align=\"center\" xml:id=\"S0.T1.2.1.2\">B</td></tr><tr xml:id=\"S0.T1.2.2\"><td align=\"center\" xml:id=\"S0.T1.2.2.1\">1</td><td align=\"center\" xml:id=\"S0.T1.2.2.2\">2</td></tr></tabular></table>",
  );
  assert_element(
    &xml,
    "caption",
    &[],
    r#"<caption><tag close=": ">Table 1</tag>Only tablecaption.</caption>"#,
  );
  assert!(
    xml.contains("<toccaption><tag close=\" \">2</tag>Next table.</toccaption>"),
    "{xml}"
  );
}

/// Review 73a r3 finding 3: an emulateapj in-data `\tablecaption` is kept on its table (OD #664) and the store repeats it
/// on the next caption-less table, as pdflatex, without its `\label`: one label, on the table that shows it first.
#[test]
fn emulateapj_repeat_drops_the_label() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/emulateapj_repeat_drops_the_label.tex"
  ));
  assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T3""#],
    "<table inlist=\"lot\" xml:id=\"S0.T3\"><tags><tag>Table 3</tag><tag role=\"autoref\">Table\u{a0}3<text xml:id=\"S0.T3.1\"/></tag><tag role=\"refnum\">3</tag><tag role=\"typerefnum\">Table 3</tag></tags><toccaption><tag close=\" \">3</tag>In data two.</toccaption><caption><tag>Table 3</tag>In data two.</caption><tabular xml:id=\"S0.T3.2\"><thead><tr xml:id=\"S0.T3.2.1\"><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T3.2.1.1\">E</td><td align=\"center\" border=\"tt\" thead=\"column\" xml:id=\"S0.T3.2.1.2\">F</td></tr></thead><tbody><tr xml:id=\"S0.T3.2.2\"><td align=\"center\" border=\"b t\" xml:id=\"S0.T3.2.2.1\">5</td><td align=\"center\" border=\"b t\" xml:id=\"S0.T3.2.2.2\">6</td></tr></tbody></tabular></table>",
  );
  assert_eq!(xml.matches("LABEL:t2\"").count(), 2, "{xml}");
  assert!(
    xml.contains("<table inlist=\"lot\" labels=\"LABEL:t2\" xml:id=\"S0.T2\">"),
    "{xml}"
  );
  assert!(xml.contains("<caption>In data two.</caption>"), "{xml}");
}

/// 1907.05060 (elife.cls loads caption with `labelsep=period`): caption's title takes the caption text `\long`, as
/// caption3's `\caption@@make` (caption3.sty:1488), so a caption with a paragraph break is set whole, without
/// "Paragraph ended before \format@title@figure" and its two boxing-group Errors.
#[test]
fn caption_package_title_takes_a_paragraph_break() {
  let xml = clean(include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_package_title_takes_a_paragraph_break.tex"
  ));
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\". \"><text font=\"bold\" fontsize=\"90%\" xml:id=\"S0.F1.1\">Figure 1</text></tag><text fontsize=\"90%\" xml:id=\"S0.F1.2\">Long A. B.</text></caption>",
  );
}

/// neurips, user 2026-10-18 option C: the binding defines no `\projectpage`/`\emailaddress` (2605.09681's copy defines
/// them, `perfect_kernel_batchint23::neurips_copy_setters_are_contacts`); without such a copy `\projectpage` is
/// undefined, one Error, as in TeX, and no contact is made.
#[test]
fn neurips_projectpage_undefined_without_a_copy() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/stand-in-copies/neurips_contact/projectpage_undefined_without_a_copy.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("Error:undefined:\\projectpage"), "{log}");
  assert!(!xml.contains("<contact"), "{xml}");
}
