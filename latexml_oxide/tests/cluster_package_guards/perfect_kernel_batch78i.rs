//! Red/green guards for batch 78i (quality batch 10): biblatex's own `.bib` fields kept raw on the BibTeX route
//! (natbib + plainnat prints none of them) and printed under biblatex; thm-restate's restated theorems printing their
//! statements; a minipage's footnotes lettered on their own counter; hyperref's PDF information strings recorded
//! expanded; an AMS-TeX paper's own `\documentstyle` read raw. Each guard converts its repro under
//! `tools/perfect_kernel/repros/` and asserts no Error or Warning and the authored content in whole elements, against
//! pdflatex (tex for AMS-TeX). Witnesses 2602.10008, 2606.11822 (fields); 2108.11527, 2602.09462, 2402.14202
//! (thm-restate); 0802.0707 (minipage notes); 1210.7805 (hyperref); math0402285, math9806089 (AMS-TeX styles).
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `tex` (and `files`) converted to HTML by the binary in a directory of their own, as a paper is (the bibliography
/// and the `\ref` text are the post-processor's): its HTML, the conversion free of errors and warnings.
fn html_of(tex: &str, files: &[(&str, &str)]) -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write a file");
  }
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

/// A `.bib` read through natbib + plainnat prints what plainnat prints: biblatex's own fields (`journaltitle`,
/// `location`, `addendum`, `urldate`, `maintitle`, `titleaddon`) are data no `.bst` reads, kept raw as in Perl
/// (BibTeX.pool `\bib@field@default@default`). Red (int47a): "Main Work Title . Paper . Addon Words . J Real,
/// \undefinedmacro Jt 3 … Note: An addendum text Accessed: 2020-01-01" and `Error:undefined:\undefinedmacro`.
/// Witnesses 2602.10008 (apsrev4-1, 54 "Accessed:"), 2606.11822. Repro
/// index-bib/natbib_biblatex_only_fields_are_raw_data.
#[test]
fn natbib_biblatex_only_fields_are_raw_data() {
  let html = html_of(
    repro!("index-bib/natbib_biblatex_only_fields_are_raw_data.tex"),
    &[(
      "natbib_biblatex_only_fields_are_raw_data.bib",
      repro!("index-bib/natbib_biblatex_only_fields_are_raw_data.bib"),
    )],
  );
  assert_element(
    &html,
    "li",
    &["id=\"bib.bib1\""],
    "<li class=\"ltx_bibitem ltx_bib_article\" id=\"bib.bib1\"><span class=\"ltx_tag ltx_bib_author-year \
     ltx_role_refnum ltx_tag_bibitem\">Able (2001)</span><span class=\"ltx_bibblock\"><span class=\"ltx_text \
     ltx_bib_author\">Ann Able</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">\
     Paper</span>.</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_journal\">J Real</span> <span \
     class=\"ltx_text ltx_bib_volume\">3</span>, <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}1–2</span>.</span><span \
     class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span>\
     </li>",
  );
}

/// The twin: under biblatex (a `.bib` without a `.bbl`, through the binding) the same fields print, as biblatex's
/// standard style prints them (standard.bbx; pdflatex + biber: "An addendum text.", "(visited on 01/01/2020)", "Main
/// Work Title. Volume Title. Addon Words. Berlin: Pub, 2002."). Repro index-bib/biblatex_prints_its_own_fields.
#[test]
fn biblatex_prints_its_own_fields() {
  let html = html_of(repro!("index-bib/biblatex_prints_its_own_fields.tex"), &[]);
  assert_element(
    &html,
    "li",
    &["id=\"bib.bib1\""],
    "<li class=\"ltx_bibitem ltx_bib_article\" id=\"bib.bib1\"><span class=\"ltx_tag ltx_bib_key ltx_role_refnum \
     ltx_tag_bibitem\">[1]</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Ann Able</span>\
     <span class=\"ltx_text ltx_bib_year\"> (2001)</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text \
     ltx_bib_title\">Paper</span>.</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_journal\">J Real\
     </span> <span class=\"ltx_text ltx_bib_volume\">3</span>, <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}1–2</span>.\
     </span><span class=\"ltx_bibblock\">Note: <span class=\"ltx_text ltx_bib_note\">An addendum text</span></span>\
     <span class=\"ltx_bibblock\">Accessed: <span class=\"ltx_text ltx_bib_accessed\">2020-01-01</span></span><span \
     class=\"ltx_bibblock\">External Links: <span class=\"ltx_text ltx_bib_links\"><a class=\"ltx_ref \
     ltx_bib_external\" href=\"https://example.org/a\" title=\"\">https://example.org/a</a></span></span><span \
     class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span>\
     </li>",
  );
  assert_element(
    &html,
    "li",
    &["id=\"bib.bib2\""],
    "<li class=\"ltx_bibitem ltx_bib_book\" id=\"bib.bib2\"><span class=\"ltx_tag ltx_bib_key ltx_role_refnum \
     ltx_tag_bibitem\">[2]</span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_author\">Bob Baker</span>\
     <span class=\"ltx_text ltx_bib_year\"> (2002)</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text \
     ltx_bib_maintitle\">Main Work Title</span>. <span class=\"ltx_text ltx_bib_title\">Volume Title</span>. <span \
     class=\"ltx_text ltx_bib_subtitle\">Addon Words</span>.</span><span class=\"ltx_bibblock\"> <span \
     class=\"ltx_text ltx_bib_publisher\">Pub</span>, <span class=\"ltx_text ltx_bib_place\">Berlin</span>.</span>\
     <span class=\"ltx_bibblock ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.\
     </span></li>",
  );
}

/// On the BibTeX route a biblatex field is read when the style declares it: ACM-Reference-Format.bst's `ENTRY` list has
/// `location` (:38, printed after the event, :1318-1342), so its entries print the place, and not `urldate`, which it
/// does not declare (pdflatex + bibtex: "In Proc. of the Conference (Berlin, Germany). ACM, New York, NY, USA, 1–2.").
/// Red (78i r1, the fields under biblatex only): no "Berlin, Germany". Witnesses 2603.25517, 2601.12610 (ACM),
/// 2607.28080 (sn-mathphys `urldate`). Repro index-bib/bst_declared_biblatex_field_is_printed.
#[test]
fn bst_declared_biblatex_field_is_printed() {
  let html = html_of(
    repro!("index-bib/bst_declared_biblatex_field_is_printed.tex"),
    &[],
  );
  assert_element(
    &html,
    "li",
    &["id=\"bib.bib1\""],
    "<li class=\"ltx_bibitem ltx_bib_inproceedings\" id=\"bib.bib1\"><span class=\"ltx_tag ltx_bib_author-year \
     ltx_role_refnum ltx_tag_bibitem\">Able (2001)</span><span class=\"ltx_bibblock\"><span class=\"ltx_text \
     ltx_bib_author\">Ann Able</span></span><span class=\"ltx_bibblock\"><span class=\"ltx_text ltx_bib_title\">\
     Paper</span>.</span><span class=\"ltx_bibblock\">In <span class=\"ltx_text ltx_bib_inbook\">Proc. of the \
     Conference</span>,</span><span class=\"ltx_bibblock\"> <span class=\"ltx_text ltx_bib_publisher\">ACM</span>, \
     <span class=\"ltx_text ltx_bib_place\">New York, NY, USA</span>, <span class=\"ltx_text ltx_bib_place\">Berlin, \
     Germany</span>, <span class=\"ltx_text ltx_bib_pages\">pp.\u{a0}1–2</span>.</span><span class=\"ltx_bibblock \
     ltx_bib_cited\">Cited by: <a class=\"ltx_ref\" href=\"#p1.1\" title=\"\">p1.1</a>.</span></li>",
  );
}

/// `\<name>*` prints the stored theorem's statement again (thm-restate.sty:103-184): its tag the original's, its
/// labels gobbled, its counter not stepped, its equation numbered as the original's and the equation counter put
/// back; the original keeps its labels and `\ref` targets it. The `restate=<name>` key stores the same way, with the
/// theorem's other keys for the copy. Red (int47a, Perl the same): each restatement `<p>See <ref
/// labelref="LABEL:restatable:…"/></p>`. Witnesses 2108.11527, 2602.09462, 2402.14202. Repro
/// package-bindings/thm_restate_restated_theorem_prints_its_statement.
#[test]
fn restated_theorem_prints_its_statement() {
  let (log, xml) = convert_with(
    repro!("package-bindings/thm_restate_restated_theorem_prints_its_statement.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // the original, labelled
  let original = latexml::util::test::xml_element(&xml, "theorem", &["xml:id=\"S1.Thmtheorem1\""])
    .expect("{xml}");
  assert!(
    original.contains("labels=\"LABEL:lem:k LABEL:restatable:keylem\""),
    "{original}"
  );
  let math = |id: &str| {
    latexml::util::test::xml_element(&xml, "Math", &[&format!("xml:id=\"{id}\"")])
      .expect("the equation's Math")
  };
  let restated_math = math("S2.E1.m1");
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"S2.Thmtheoremx1\""],
    &format!(
      "<theorem class=\"ltx_theorem_lemma\" inlist=\"thm theorem:lemma\" xml:id=\"S2.Thmtheoremx1\"><tags><tag>Lemma \
       1.1</tag><tag role=\"refnum\">1.1</tag><tag role=\"typerefnum\">Lemma 1.1</tag></tags><title \
       class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S2.Thmtheoremx1.1\">Lemma 1.1</text></tag><text \
       font=\"bold\" xml:id=\"S2.Thmtheoremx1.2\">.</text></title><para xml:id=\"S2.Thmtheoremx1.p1\"><p \
       xml:id=\"S2.Thmtheoremx1.p1.1\"><text font=\"italic\" xml:id=\"S2.Thmtheoremx1.p1.1.1\">First paragraph \
       with</text></p><equation xml:id=\"S2.E1\"><tags><tag>(1)</tag><tag role=\"refnum\">1</tag></tags>\
       {restated_math}</equation><p xml:id=\"S2.Thmtheoremx1.p1.2\"><text font=\"italic\" \
       xml:id=\"S2.Thmtheoremx1.p1.2.1\">Second paragraph.</text></p></para></theorem>"
    ),
  );
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"S2.Thmtheoremx2\""],
    "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" xml:id=\"S2.Thmtheoremx2\"><tags><tag>\
     Theorem 1.2</tag><tag role=\"refnum\">1.2</tag><tag role=\"typerefnum\">Theorem 1.2</tag></tags><title \
     class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S2.Thmtheoremx2.1\">Theorem 1.2</text></tag><text \
     font=\"bold\" xml:id=\"S2.Thmtheoremx2.2\"> </text>(Named)<text font=\"bold\" xml:id=\"S2.Thmtheoremx2.3\">.\
     </text></title><para xml:id=\"S2.Thmtheoremx2.p1\"><p xml:id=\"S2.Thmtheoremx2.p1.1\"><text font=\"italic\" \
     xml:id=\"S2.Thmtheoremx2.p1.1.1\">Keyed body.</text></p></para></theorem>",
  );
  // the keyed original keeps its label; the next equation is (3), as in the PDF
  assert!(
    xml.contains("labels=\"LABEL:restatable:mythm LABEL:thm:x\" xml:id=\"S1.Thmtheorem2\""),
    "{xml}"
  );
  let tags: Vec<&str> = xml
    .match_indices("<tag>(")
    .map(|(i, _)| &xml[i + 5..i + 8])
    .collect();
  assert_eq!(tags, ["(1)", "(2)", "(1)", "(3)"], "{xml}");
}

/// A minipage's notes are its own (latex.ltx:16323 `\def\@mpfn{mpfootnote}\def\thempfn{\thempfootnote}
/// \c@mpfootnote\z@`, :17649 `\footnote` steps `\@mpfn`): lettered "a" (:17638 `\thempfootnote`), the page's notes
/// going on 1, 2, 3, and the `\ref` to the body note printing 3, as pdflatex does. Red (int47a, Perl the same): the
/// minipage note 2 on the page's counter, the later notes 3 and 4. Witness 0802.0707. Repro
/// boxes-groups/minipage_footnotes_are_lettered.
#[test]
fn minipage_footnotes_are_lettered() {
  let (log, xml) = convert_with(
    repro!("boxes-groups/minipage_footnotes_are_lettered.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "note",
    &["mark=\"a\""],
    "<note mark=\"a\" role=\"footnote\" xml:id=\"footnotex1\"><tags><tag>a</tag><tag role=\"refnum\">a</tag><tag \
     role=\"typerefnum\">footnote a</tag></tags>Mini note.</note>",
  );
  let marks: Vec<&str> = xml
    .match_indices("<note ")
    .filter_map(|(i, _)| {
      let tag = &xml[i..i + xml[i..].find('>')?];
      let rest = &tag[tag.find(" mark=\"")? + 7..];
      Some(&rest[..rest.find('"')?])
    })
    .collect();
  assert_eq!(marks, ["1", "a", "2", "3"], "{xml}");
  let html = html_of(
    repro!("boxes-groups/minipage_footnotes_are_lettered.tex"),
    &[],
  );
  assert!(
    html.contains("class=\"ltx_ref\"><span class=\"ltx_text ltx_ref_tag\">3</span></a>"),
    "{html}"
  );
}

/// A `\footnotetext` in a minipage keeps to the page's counter, so it pairs with the `\footnotemark` it answers, which
/// LaTeX keeps there too (latex.ltx:17677); LaTeX letters the text (:17690-17699 `\thempfn`), its own mismatch, which the
/// pairing does not copy (KNOWN_PERL_ERRORS #1049). Red (78i r1, the text on the minipage's counter): a mark "1" and a
/// separate note "a". Witness 2304.06412. Repro boxes-groups/minipage_footnotetext_pairs_with_its_mark.
#[test]
fn minipage_footnotetext_pairs_with_its_mark() {
  let (log, xml) = convert_with(
    repro!("boxes-groups/minipage_footnotetext_pairs_with_its_mark.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<note ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "note",
    &["role=\"footnote\""],
    "<note mark=\"1\" role=\"footnote\" xml:id=\"footnotex1\"><tags><tag>1</tag><tag role=\"refnum\">1</tag><tag \
     role=\"typerefnum\">footnote 1</tag></tags>L is low.</note>",
  );
}

/// hyperref expands a PDF information string as it sets it (`\pdfstringdef\@pdftitle{#1}`, hyperref.sty:3543-3600):
/// the `ltx:rdf` metadata is its text. Red (60m, Perl the same: hyperref.sty.ltxml:133 `ToString`): `content="\mytitle"`.
/// Repro sectioning-frontmatter/hyperref_pdfinfo_expands_macros.
#[test]
fn hyperref_metadata_is_expanded() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_expands_macros.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "rdf",
    &["property=\"dcterms:title\""],
    "<rdf about=\"\" content=\"Real Title\" property=\"dcterms:title\"/>",
  );
  assert_element(
    &xml,
    "rdf",
    &["property=\"dcterms:creator\""],
    "<rdf about=\"\" content=\"Ann Alpha and Bob Bravo\" property=\"dcterms:creator\"/>",
  );
}

/// The markup of a PDF string is dropped as `\pdfstringdef` drops it: `\thanks` gobbled (the title's one note stays
/// the only one), `\emph` its text, `\texorpdfstring` its PDF branch, math its characters, `\"o` "ö", `\LaTeX`
/// "LaTeX", `~` a space, `\and` "; " (hyperref's pdfusetitle author, hyperref.sty:6255). pdfinfo: Title "A Study of
/// Things", Subject "alpha-rays and x2", Keywords "König, LaTeX". Witness 1210.7805 (`pdftitle=\OfficialTitle`).
/// Repro sectioning-frontmatter/hyperref_pdfinfo_drops_notes_and_markup.
#[test]
fn hyperref_metadata_drops_notes_and_markup() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_drops_notes_and_markup.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rdf: Vec<&str> = xml
    .match_indices("<rdf ")
    .map(|(i, _)| &xml[i..i + xml[i..].find("/>").unwrap_or(0) + 2])
    .collect();
  assert_eq!(rdf, [
    "<rdf about=\"\" content=\"Ann Able; Bob Baker\" property=\"dcterms:creator\"/>",
    "<rdf about=\"\" content=\"König, LaTeX, Martínez, Straße\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"alpha-rays and x2\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"A Study of Things\" property=\"dcterms:title\"/>",
  ]);
  assert_eq!(xml.matches("Funded by X.").count(), 1, "{xml}");
}

/// A PDF string's text symbols are their characters, as puenc.def declares them (`\textendash` :1082, `\textsection`
/// :184, the kernel's `\S`, `\dag`, `\copyright`, `\pounds`, hyperref.sty:416-418's `\P`, `\ldots`, `\dots`), and
/// `---`, `--`, `` !` ``, `` ?` `` are the dashes and inverted marks hyperref substitutes (hyperref.sty:620-623).
/// pdfinfo as asserted. Red (78i r2): "AB, 2, , , , , , x, , …", "Pre--post---end ``quoted'' and x y" (review probes
/// g2, g3). Repro sectioning-frontmatter/hyperref_pdfinfo_text_symbols.
#[test]
fn hyperref_metadata_text_symbols() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_text_symbols.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rdf: Vec<&str> = xml
    .match_indices("<rdf ")
    .map(|(i, _)| &xml[i..i + xml[i..].find("/>").unwrap_or(0) + 2])
    .collect();
  assert_eq!(rdf, [
    "<rdf about=\"\" content=\"A–B, §2, †, ©, £, …, ®, «x», ’, œuvre, ĳ, þ, ŋ\" property=\"dcterms:creator\"/>",
    "<rdf about=\"\" content=\"¶ ‡ ™ ° • ‘a’ “b” ‹c› ¡d ¿e\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"Pre–post—end ``quoted'' and — x … y\" property=\"dcterms:title\"/>",
  ]);
}

/// AMS-TeX's `\documentstyle{X}` is `\input X.sty` (amstex.tex:1731): a paper's own style with no binding is read raw,
/// so its top-matter macros (`\received`, `\startingpage`) are defined. Red (int47a, Perl the same: AmSTeX.pool.ltxml
/// :71 `notex => 1`): the amsppt fallback, `\title…\endtitle` running to the end (Fatal:Mouth:EoF), 0 words. The stand-in
/// amslocal.sty is written from scratch (amams.sty states no licence). Witnesses math0402285, math9806089. Repro
/// amstex/amstex_local_style_is_read.
#[test]
fn amstex_local_style_is_read() {
  let (log, xml) = convert_files_with(
    repro!("amstex/amstex_local_style_is_read.tex"),
    &[("amslocal.sty", repro!("amstex/amslocal.sty"))],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(&xml, "title", &[], "<title>Main Title</title>");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Body text of the paper.</p></para>",
  );
}

/// `{restatable*}` is not the numbered instance (thm-restate.sty:207-213 `\thmt@thisistheonefalse`, :130-145): its
/// theorem steps no counter, its labels are gobbled, and its number, like a `\<name>*`'s before the later `\<name>`,
/// is that instance's — a reference to the label the later `\<name>` sets, as `continues=` numbers its theorem. pdflatex
/// (second run): "Theorem 2.2. Main statement." twice in § 1, "See 2.2.", § 2 "Theorem 2.1", "2.2", "2.3". Red (78i
/// r1): the preview and `\main*` "Theorem 1.1", `thm:main` set twice; Perl: the environment undefined (KNOWN_PERL_ERRORS
/// #1052). Witness 1702.04225 (the introduction's
/// "Theorem 6.21", "Proposition 6.13"). Repro package-bindings/thm_restate_restatable_star_takes_the_later_number.
#[test]
fn restatable_star_takes_the_later_number() {
  let tex = repro!("package-bindings/thm_restate_restatable_star_takes_the_later_number.tex");
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for id in ["S1.Thmtheoremx1", "S1.Thmtheoremx2"] {
    assert_element(
      &xml,
      "theorem",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" xml:id=\"{id}\"><tags><tag>Theorem \
         <ref labelref=\"LABEL:restatable:main\"/></tag><tag role=\"refnum\"><ref \
         labelref=\"LABEL:restatable:main\"/></tag><tag role=\"typerefnum\">Theorem <ref \
         labelref=\"LABEL:restatable:main\"/></tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
         xml:id=\"{id}.1\">Theorem </text><ref font=\"bold\" labelref=\"LABEL:restatable:main\"/></tag><text \
         font=\"bold\" xml:id=\"{id}.2\">.</text></title><para xml:id=\"{id}.p1\"><p xml:id=\"{id}.p1.1\"><text \
         font=\"italic\" xml:id=\"{id}.p1.1.1\">Main statement.</text></p></para></theorem>"
      ),
    );
  }
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"S2.Thmtheorem2\""],
    "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" labels=\"LABEL:restatable:main \
     LABEL:thm:main\" xml:id=\"S2.Thmtheorem2\"><tags><tag>Theorem 2.2</tag><tag role=\"refnum\">2.2</tag><tag \
     role=\"typerefnum\">Theorem 2.2</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
     xml:id=\"S2.Thmtheorem2.1\">Theorem 2.2</text></tag><text font=\"bold\" xml:id=\"S2.Thmtheorem2.2\">.</text>\
     </title><para xml:id=\"S2.Thmtheorem2.p1\"><p xml:id=\"S2.Thmtheorem2.p1.1\"><text font=\"italic\" \
     xml:id=\"S2.Thmtheorem2.p1.1.1\">Main statement.</text></p></para></theorem>",
  );
  assert!(xml.contains("xml:id=\"S2.Thmtheorem3\""), "{xml}");
  assert_eq!(xml.matches("LABEL:thm:main\"").count(), 2, "{xml}"); // the label and the `\ref`
  // the post-processor prints the reference: "Theorem 2.2." in § 1
  let html = html_of(tex, &[]);
  assert_eq!(
    html
      .matches(
        "class=\"ltx_ref ltx_font_bold\"><span class=\"ltx_text ltx_ref_tag\">2.2</span></a></span>"
      )
      .count(),
    2,
    "{html}"
  );
}

/// Every `\<name>` sets the label `thmt@@<name>` (thm-restate.sty:158-160) and pdflatex reads the last, so a
/// `{restatable*}` followed by two `\<name>` previews the second one's number. pdflatex (second run): § 1 "Theorem 3.1.
/// Main statement.", "See 3.1.", § 2 "Theorem 2.1", § 3 "Theorem 3.1". Red (78i r2): the preview "Theorem 2.1", only
/// the first `\main` labelled. Review probe s1; witness 1702.04225. Repro
/// package-bindings/thm_restate_restatable_star_takes_the_last_number.
#[test]
fn restatable_star_takes_the_last_number() {
  let tex = repro!("package-bindings/thm_restate_restatable_star_takes_the_last_number.tex");
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, number) in [("S2.Thmtheorem1", "2.1"), ("S3.Thmtheorem1", "3.1")] {
    assert_element(
      &xml,
      "theorem",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" labels=\"LABEL:restatable:main \
         LABEL:thm:main\" xml:id=\"{id}\"><tags><tag>Theorem {number}</tag><tag role=\"refnum\">{number}</tag><tag \
         role=\"typerefnum\">Theorem {number}</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
         xml:id=\"{id}.1\">Theorem {number}</text></tag><text font=\"bold\" xml:id=\"{id}.2\">.</text></title><para \
         xml:id=\"{id}.p1\"><p xml:id=\"{id}.p1.1\"><text font=\"italic\" xml:id=\"{id}.p1.1.1\">Main \
         statement.</text></p></para></theorem>"
      ),
    );
  }
  // the preview: "Theorem 3.1." in § 1, by reference to the last `\main`
  let html = html_of(tex, &[]);
  assert_eq!(
    html
      .matches(
        "class=\"ltx_ref ltx_font_bold\"><span class=\"ltx_text ltx_ref_tag\">3.1</span></a></span>"
      )
      .count(),
    1,
    "{html}"
  );
  assert_eq!(html.matches("ltx_ref_tag\">2.1<").count(), 0, "{html}");
}

/// LaTeXML's own macros test with the kernel's `\lx@ifdefined` (etex.rs), so a document's `\ifdefined` of another
/// shape does not read them: amams.sty:1099 `\def\ifdefined#1#2#3{…}` made the footnote's typerefname test
/// (`\lx@latin@name`) read `\latinencoding` as its argument. pdflatex: "Text1 and no, no.", the note "1 A note.".
/// Red (78i r1): Error:undefined:\latinencoding, an extra `\else` and `\fi`, the ERROR in the typerefnum tag. Witness
/// math0402285. Repro parameter-conditional/kernel_ifdefined_survives_a_document_redefinition.
#[test]
fn kernel_ifdefined_survives_a_document_redefinition() {
  let (log, xml) = convert_with(
    repro!("parameter-conditional/kernel_ifdefined_survives_a_document_redefinition.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text<note labels=\"LABEL:x\" mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags><tag>1\
     </tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">footnote 1</tag></tags>A note.</note> and no, no.</p>",
  );
}
