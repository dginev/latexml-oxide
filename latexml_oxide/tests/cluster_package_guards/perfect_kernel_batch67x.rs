//! Red/green guards for batch 67x: the IEEEtran family's binding gaps (G2) — the IEEEtran.cls internals documents
//! call, the IEEEeqnarray rule rows (IEEEtran and IEEEtrantools), the names of IEEEtran's V1.5-1.7a versions and of the
//! variant classes papers ship (IEEEtran4PSCC's `\thanksto`, the SIU template's `{ozet}`/`{IEEEanahtar}`), and
//! ieeeconf's (run 340 undefined-macro inventory: 692 papers).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// IEEEtran.cls internals papers' own title, page-style and list code calls: `\ps@headings`/`\ps@IEEEtitlepagestyle`
/// (a `\confheader` recipe; 1805.00151, 1805.02797), the compsoc-only executors and `\@IEEEnormalsizeunitybaselineskip`
/// (a `\newmaketitle`; 2202.10460, 2202.12377), `\sublargesize` (1208.3665), `\@ifmtarg` (1510.02032),
/// `\IEEEquantizevspace` (2601.10876), `\IEEEbibitemsep` (2007.01482), `\thesectiondis` (1605.01233),
/// `\IEEEnoauxwrite` (2006.13598), `{LaTeXenumerate}` (cs0701047): no error, the text where it stands.
#[test]
fn ieeetran_internals_documents_call() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_internals_documents_call.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let assert_element = latexml::util::test::assert_element;
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text fontsize="298%" xml:id="p1.1.1">Title again</text></p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1"><text fontsize="110%" xml:id="p2.1.1">Big</text></p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p xml:id="p3.1">full</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p4""#],
    r#"<para xml:id="p4"><p xml:id="p4.1">Quantized</p></para>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="S1.p1.1""#],
    r#"<p xml:id="S1.p1.1">I. x</p>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="S1.I1.i1.p1""#],
    r#"<para xml:id="S1.I1.i1.p1"><p xml:id="S1.I1.i1.p1.1">one</p></para>"#,
  );
}

/// The rule rows of an `{IEEEeqnarraybox}` table (cs0608041, 0911.1713, 1004.2757, 1112.1125): each spans all the
/// columns the specification defines (`\@IEEEeqnnumcols`) with a leader rule; `\IEEEeqnarraydblrulerow` is two of them
/// around a separator row, `\IEEEeqnarrayseprow` a blank row: 9 rows.
fn assert_ruled_table(xml: &str) {
  let rule_row = regex::Regex::new(
    r#"<XMCell align="left" colspan="5"[^>]*>\s*<XMText class="ltx_leader"[^>]*><rule height="0.4pt" width="100%"/></XMText>\s*</XMCell>"#,
  )
  .unwrap();
  assert_eq!(rule_row.find_iter(xml).count(), 4, "{xml}");
  assert_eq!(xml.matches("<XMRow").count(), 9, "{xml}");
  for text in [">Size</XMText>", ">Number</XMText>"] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
  for number in ["7", "1", "8", "2"] {
    let token = format!(r#"<XMTok meaning="{number}" role="NUMBER">{number}</XMTok>"#);
    assert!(xml.contains(&token), "{token}: {xml}");
  }
}

#[test]
fn ieeetran_eqnarray_rule_rows() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_eqnarray_rule_rows.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_ruled_table(&xml);
}

/// The same rows from IEEEtrantools.sty (:1566-1690) under another class: the IEEEtrantools binding shares the
/// IEEEeqnarray family.
#[test]
fn ieeetrantools_eqnarray_rule_rows() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_eqnarray_rule_rows.tex"
  )
  .replace(
    "\\documentclass{IEEEtran}",
    "\\documentclass{article}\n\\usepackage{IEEEtrantools}",
  );
  let (log, xml) = convert_counted(&tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_ruled_table(&xml);
}

/// IEEEtran V1.5-1.6c's names (`\centerfigcaptionstrue`, cs0501064; `\useRomanappendicesfalse`, cs0504030, 0805.0012;
/// `\nocalcleftmargintrue`, 2009.04715) and V1.7a's `\@IEEEtunefonts` (0705.1922): no error, the appendix "A".
#[test]
fn ieeetran_v16_names() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_v16_names.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "tags",
    &[],
    r#"<tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags>"#,
  );
  latexml::util::test::assert_element(&xml, "title", &[], r#"<title>A Title</title>"#);
  assert!(
    xml.contains(r#"<title><tag close=" ">Appendix A</tag>Proof</title>"#),
    "{xml}"
  );
}

/// IEEEtran4PSCC.cls's `\thanksto{<note>}` (:4793-4804; 185 papers, 1601.01675, 1710.09738): the frontmatter's thanks
/// note. The class has no binding of its own, so IEEEtran's serves it (the one Warning).
#[test]
fn ieeetran_pscc_thanksto() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_pscc_thanksto.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Can't find binding for class IEEEtran4PSCC (using IEEEtran)"),
    "{log}"
  );
  latexml::util::test::assert_element(
    &xml,
    "pubnote",
    &[],
    "<pubnote name=\"Thanks:\u{a0}\" role=\"thanks\">This work is funded by Z.</pubnote>",
  );
}

/// The SIU template's IEEEtran.cls (1504.02975, 1807.04577): `{ozet}` and `{IEEEanahtar}`, the Turkish abstract and
/// keywords, are set where they stand under their headings; the English `{abstract}` and `{IEEEkeywords}` stay the
/// frontmatter's. As the template sets them (:3746, :3765): the abstract bold, the keywords bold italic, each heading a
/// text of its own.
#[test]
fn ieeetran_siu_ozet() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_siu_ozet.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let assert_element = latexml::util::test::assert_element;
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para class=\"ltx_noindent\" xml:id=\"p1\"><p xml:id=\"p1.1\"><text font=\"bold italic\" xml:id=\"p1.1.1\">\
     Özetçe </text><text font=\"bold\" xml:id=\"p1.1.2\">—Türkçe özet.</text></p></para>",
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    "<para class=\"ltx_noindent\" xml:id=\"p2\"><p xml:id=\"p2.1\"><text font=\"bold italic\" xml:id=\"p2.1.1\">\
     Anahtar Kelimeler</text><text font=\"bold\" xml:id=\"p2.1.2\">—</text><text font=\"bold italic\" \
     xml:id=\"p2.1.3\">anahtar, kelime</text></p></para>",
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">English abstract.</p></abstract>"#,
  );
  assert_eq!(xml.matches("<abstract").count(), 1, "{xml}");
  assert_eq!(xml.matches("<keywords").count(), 1, "{xml}");
}

/// IEEEtran V1.7a's `\AtBeginDocument{\def\proof{…\IEEEproof}}` (:4690): a `{proof}` the document left undefined at
/// `\begin{document}` is IEEEproof there (2106.12111); V1.8b defines none, so only an undefined (or `\relax`) one is
/// replaced.
#[test]
fn ieeetran_v17_proof_at_begin_document() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeetran_v17_proof_at_begin_document.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r#"<title class="ltx_runin" font="bold italic">Proof:</title>"#,
  );
  assert!(xml.contains("It holds.\n∎"), "{xml}");
}

/// ieeeconf.cls (IEEEtran V1.6b): a title copying its `\@specialpapernotice` and `\@IEEEauthorblockconfadjspace`
/// (1811.05306, 1811.01547), `\thesubsubsectiondis` (1610.00529), `\centerfigcaptionsfalse` (1807.11264), and the
/// RBO copy's `\ieeefootline{<text>}` (1803.09650, 1807.07870): a publication note.
#[test]
fn ieeeconf_v16_title_internals_and_footline() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ieeeconf_v16_title_internals_and_footline.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "pubnote",
    &[],
    r#"<pubnote role="footline">Workshop on Inspection, 2018</pubnote>"#,
  );
  assert!(xml.contains(r#"<p xml:id="p2.2">Ann Able</p>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="S1.SS1.SSS1.p1.1">1) Text.</p>"#),
    "{xml}"
  );
}

/// The same with amsthm's proof let to `\relax` (2208.13860, 2310.09933, 2310.09935, 2509.09277): LaTeX's undefined
/// too, so IEEEproof at `\begin{document}` (pdflatex with V1.7a: "Proof: It holds.").
#[test]
fn ieeetran_v17_proof_after_amsthm_relaxed() {
  let tex = "\\documentclass{IEEEtran}\n\\usepackage{amsthm}\n\\let\\proof\\relax\n\\let\\endproof\\relax\n\
             \\begin{document}\n\\begin{proof}\nIt holds.\n\\end{proof}\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r#"<title class="ltx_runin" font="bold italic">Proof:</title>"#,
  );
}

/// `\appendix`/`\appendices` save the kernel's `\section` as `\@IEEEappendixsavesection` (IEEEtran.cls:5695-5696),
/// which a document's own appendix command copied from the class calls (1907.08915's `\suppl`; pdflatex: the
/// unnumbered heading "Supplementary materials", in the section's small caps since 78a).
#[test]
fn ieeetran_appendix_saves_section() {
  let tex = "\\documentclass{IEEEtran}\n\\begin{document}\n\\section{Intro}\nText.\n\\appendix\n\\makeatletter\n\
             \\@IEEEappendixsavesection*{Supplementary materials}\n\\makeatother\nBody.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<title font=\"smallcaps\">Supplementary materials</title>"),
    "{xml}"
  );
}

/// IEEEtrantools.sty's `{IEEEproof}` under any class (:2726-2747; 1710.09177, 1802.00899, 2002.01226, 2003.00712): the
/// IEEEtran binding's IEEEproof, shared (`define_ieeeproof`), its QED where `\IEEEQEDhere` stands and none at the end.
#[test]
fn ieeetrantools_ieeeproof_any_class() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/ieeetrantools_ieeeproof_any_class.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r#"<title class="ltx_runin" font="bold italic">Proof:</title>"#,
  );
  assert_eq!(xml.matches('∎').count(), 1, "{xml}");
  assert!(xml.contains("</Math>. ∎</text></p>"), "{xml}");
}

/// IEEEtrantools.sty's IED lists, `\IEEEauthorrefmark` and `\IEEEPARstart` under any class (:1000-1140, :2616,
/// :2721-2723; 1304.3949, 1609.02977, 1905.00616): `{IEEEdescription}` with its label width, the standard `{itemize}`
/// the IEEE one under `[redeflists]`, the mark 2 the footnote symbol †.
#[test]
fn ieeetrantools_lists_marks_any_class() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/block-model/ieeetrantools_lists_marks_any_class.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let assert_element = latexml::util::test::assert_element;
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">This text.</p>"#,
  );
  assert!(
    xml.contains(r#"<XMTok fontsize="80%" name="dagger" role="MULOP">†</XMTok>"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "description",
    &[],
    "<description xml:id=\"S0.I1\"><item xml:id=\"S0.I1.ix1\"><tags><tag><text font=\"bold\" \
     xml:id=\"S0.I1.ix1.1\">Term</text></tag><tag role=\"typerefnum\">item Term</tag></tags><para \
     xml:id=\"S0.I1.ix1.p1\"><p xml:id=\"S0.I1.ix1.p1.1\">One</p></para></item></description>",
  );
  assert_element(
    &xml,
    "itemize",
    &[],
    "<itemize xml:id=\"S0.I2\"><item xml:id=\"S0.I2.i1\"><tags><tag>•</tag><tag role=\"typerefnum\">1st \
     item</tag></tags><para xml:id=\"S0.I2.i1.p1\"><p xml:id=\"S0.I2.i1.p1.1\">alpha</p></para></item></itemize>",
  );
}

/// V1.7a's begin-document `{proof}` is a fallback a document's own begin-document `\newenvironment{proof}` replaces
/// (67x review probe g; the class's hook runs first): pdflatex with V1.8b "My proof. Mine.", no QED.
#[test]
fn ieeetran_v17_proof_yields_to_document_begin_document_proof() {
  let tex = "\\documentclass{IEEEtran}\n\\let\\proof\\relax\n\\let\\endproof\\relax\n\
             \\AtBeginDocument{\\newenvironment{proof}{\\par\\textit{My proof.} }{\\par}}\n\
             \\begin{document}\n\\begin{proof}Mine.\\end{proof}\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text font="italic" xml:id="p1.1.1">My proof.</text> Mine.</p></para>"#,
  );
  assert!(!xml.contains('∎'), "{xml}");
}

/// IEEEtrantools under the IEEEtran class only raises its error (IEEEtrantools.sty:73-78): the class binding's
/// `\IEEEauthorrefmark` stays, so each author keeps its own affiliation (67x review probe k; pdflatex "Ann Able∗, Bob
/// Baker†" over "∗ Uni One, City", "† Uni Two, Town") and a mark in `\author` its number (probe d).
#[test]
fn ieeetrantools_under_ieeetran_keeps_class_author_marks() {
  let tex = "\\documentclass[conference]{IEEEtran}\n\\usepackage{IEEEtrantools}\n\\begin{document}\n\\title{T}\n\
             \\author{\\IEEEauthorblockN{Ann Able\\IEEEauthorrefmark{1}, Bob Baker\\IEEEauthorrefmark{2}}\n\
             \\IEEEauthorblockA{\\IEEEauthorrefmark{1}Uni One, City}\n\
             \\IEEEauthorblockA{\\IEEEauthorrefmark{2}Uni Two, Town}}\n\\maketitle\nBody.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let assert_element = latexml::util::test::assert_element;
  assert_eq!(xml.matches("<creator").count(), 2, "{xml}");
  assert_element(
    &xml,
    "creator",
    &[r#"role="author""#],
    "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">1</sup></personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Uni One, City</contact></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id2\">2</sup></personname><contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Uni Two, Town</contact></creator>",
  );
  let tex = "\\documentclass{IEEEtran}\n\\usepackage{amsthm}\n\\usepackage{IEEEtrantools}\n\\begin{document}\n\
             \\title{T}\n\\author{A\\IEEEauthorrefmark{2}}\n\\maketitle\nBody.\n\\end{document}\n";
  let (log, xml) = convert_counted(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "personname",
    &[],
    r#"<personname>A<sup xml:id="id1">2</sup></personname>"#,
  );
}

/// `\appendix` (IEEEtran.cls:5723-5736) clears `\thesectiondis` and numbers the subsections `\Alph`: pdflatex "A. Sub",
/// "Dis [] sub [A]". ieeeconf's `\appendices` (ieeeconf.cls:4103-4117) does neither: "Appendix I", subsection "I-A".
/// Since 78d the `\appendix` prints its "APPENDIX" heading, an `ltx:appendix` the subsection is in (`A1.SS1`, was
/// `A0.SS1`; KNOWN_PERL_ERRORS #1022, OXIDIZED_DESIGN_DIVERGENCES #793).
#[test]
fn ieeetran_appendix_clears_sectiondis() {
  let body = "\\begin{document}\n\\section{Intro}\nText.\n\\appendix\n\\subsection{Sub}\n\
              Dis [\\thesectiondis] sub [\\thesubsection].\n\\end{document}\n";
  let (log, xml) = convert_counted(&format!("\\documentclass{{IEEEtran}}\n{body}"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  latexml::util::test::assert_element(
    &xml,
    "appendix",
    &[r#"xml:id="A1""#],
    r#"<appendix inlist="toc" xml:id="A1"><tags><tag>Appendix</tag><tag role="refnum">A</tag>
 <tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix</tag></title>
 <toctitle><tag close=" ">A</tag></toctitle><subsection inlist="toc" xml:id="A1.SS1"><tags><tag>A.</tag>
 <tag role="refnum">A</tag><tag role="typerefnum">§A</tag></tags><title><tag close=" ">A.</tag>
 <text font="italic" xml:id="A1.SS1.1">Sub</text></title><toctitle><tag close=" ">A</tag>Sub</toctitle>
 <para xml:id="A1.SS1.p1"><p xml:id="A1.SS1.p1.1">Dis [] sub [A].</p></para></subsection></appendix>"#,
  );
  let body = body.replace("\\appendix\n", "\\appendices\n\\section{First}\n");
  let (log, xml) = convert_counted(&format!("\\documentclass{{ieeeconf}}\n{body}"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(
      r#"<p xml:id="A1.SS1.p1.1">Dis [I] sub [<text xml:id="A1.SS1.p1.1.1">I-A</text>].</p>"#
    ),
    "{xml}"
  );
}
