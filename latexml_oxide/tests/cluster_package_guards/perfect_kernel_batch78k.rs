//! Red/green guards for batch 78k (quality batch 11): thmtools' unnamed theorems headed by their environment's name;
//! algorithm2e's block ends as its `shortend`/`longend`/`noend` options print them; bibentry's `\bibentry` printing
//! the entry `\nobibliography` stored; footmisc's `symbol`/`symbol*` footnote marks; natbib's `\citetext` in the
//! citation brackets. Each guard converts its repro under `tools/perfect_kernel/repros/` and asserts no Error or
//! Warning and the authored content in whole elements, against pdflatex. Witnesses 1505.04294, 2311.15711, 1706.08429,
//! 2608.16854 (thmtools); 1810.03652, 1911.01815, 1903.04631, 1402.6889 and the ten other shippers of algorithm2e v5.00
//! (algorithm2e); 2210.16098, 2505.07606 (bibentry); 1907.00006 (footmisc).
use latexml::util::test::{assert_element, convert_files_with, convert_with, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `tex` (and `files`) converted to HTML by the binary in a directory of their own, as a paper is (the citation and
/// `\ref` texts are the post-processor's): its HTML, the conversion free of errors and warnings.
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

/// A clean conversion of `tex` to XML.
fn clean_xml(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// `\declaretheorem` without `name=` names the theorem `\thmt@modifycase{<env>}` (thm-kv.sty:359), `\MakeUppercase`
/// of the first token by default: `{defn}` is headed "Defn 1.2", as pdflatex prints it. Red (int49a): "1.2" with no
/// name, typerefnum " 1.2". Witness 1505.04294. Repro package-bindings/thmtools_unnamed_theorem_is_headed_by_its_environment.
#[test]
fn thmtools_unnamed_theorem_is_headed_by_its_environment() {
  let xml = clean_xml(repro!(
    "package-bindings/thmtools_unnamed_theorem_is_headed_by_its_environment.tex"
  ));
  assert_element(
    &xml,
    "theorem",
    &["xml:id=\"S1.Thmthm2\""],
    "<theorem class=\"ltx_theorem_defn\" inlist=\"thm theorem:defn\" labels=\"LABEL:d\" xml:id=\"S1.Thmthm2\"><tags>\
     <tag>Defn 1.2</tag><tag role=\"refnum\">1.2</tag><tag role=\"typerefnum\">Defn 1.2</tag></tags><title \
     class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"S1.Thmthm2.1\">Defn 1.2</text></tag><text font=\"bold\" \
     xml:id=\"S1.Thmthm2.2\">.</text></title><para xml:id=\"S1.Thmthm2.p1\"><p xml:id=\"S1.Thmthm2.p1.1\">B</p></para></theorem>",
  );
}

/// thmtools' `lowercase` option (passed to thm-kv, thm-kv.sty:30-32) lowercases the default name: "remark 1".
/// Red (int49a): "1". Repro package-bindings/thmtools_lowercase_option_lowercases_the_unnamed_theorem.
#[test]
fn thmtools_unnamed_theorem_follows_the_case_option() {
  let xml = clean_xml(repro!(
    "package-bindings/thmtools_lowercase_option_lowercases_the_unnamed_theorem.tex"
  ));
  assert_element(
    &xml,
    "tags",
    &[],
    "<tags><tag>remark 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">remark 1</tag></tags>",
  );
}

/// thm-kv.sty:241-242: the value-less `unnumbered` and `starred` declare an unnumbered theorem, as `numbered=no` does;
/// the last numbered key decides (`[unnumbered,numbered=yes]` is numbered). Red (78k r1, Perl the same): "Note 1",
/// "Note 2", "Proof Sketch 1". Witnesses 2311.15711, 1706.08429 ("Proof 161" for "Proof"). Repro
/// package-bindings/thmtools_unnumbered_and_starred_declare_unnumbered_theorems.
#[test]
fn thmtools_unnumbered_and_starred_declare_unnumbered_theorems() {
  let xml = clean_xml(repro!(
    "package-bindings/thmtools_unnumbered_and_starred_declare_unnumbered_theorems.tex"
  ));
  for (id, expected) in [
    (
      "Thmnotex1",
      "<theorem class=\"ltx_theorem_note\" inlist=\"thm theorem:note\" \
       xml:id=\"Thmnotex1\"><tags><tag>Note</tag><tag role=\"typerefnum\">Note</tag></tags><title \
       class=\"ltx_runin\"><tag><text font=\"italic\" xml:id=\"Thmnotex1.1\">Note</text></tag><text \
       font=\"italic\" xml:id=\"Thmnotex1.2\">.</text></title><para xml:id=\"Thmnotex1.p1\"><p \
       xml:id=\"Thmnotex1.p1.1\">N1</p></para></theorem>",
    ),
    (
      "Thmnotex2",
      "<theorem class=\"ltx_theorem_note\" inlist=\"thm theorem:note\" \
       xml:id=\"Thmnotex2\"><tags><tag>Note</tag><tag role=\"typerefnum\">Note</tag></tags><title \
       class=\"ltx_runin\"><tag><text font=\"italic\" xml:id=\"Thmnotex2.1\">Note</text></tag><text \
       font=\"italic\" xml:id=\"Thmnotex2.2\">.</text></title><para xml:id=\"Thmnotex2.p1\"><p \
       xml:id=\"Thmnotex2.p1.1\">N2</p></para></theorem>",
    ),
    (
      "Thmsketchx1",
      "<theorem class=\"ltx_theorem_sketch\" inlist=\"thm theorem:sketch\" \
       xml:id=\"Thmsketchx1\"><tags><tag>Proof Sketch</tag><tag role=\"typerefnum\">Proof \
       Sketch</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
       xml:id=\"Thmsketchx1.1\">Proof Sketch</text></tag><text font=\"bold\" \
       xml:id=\"Thmsketchx1.2\">.</text></title><para xml:id=\"Thmsketchx1.p1\"><p \
       xml:id=\"Thmsketchx1.p1.1\"><text font=\"italic\" \
       xml:id=\"Thmsketchx1.p1.1.1\">S</text></p></para></theorem>",
    ),
    (
      "Thmclaim1",
      "<theorem class=\"ltx_theorem_claim\" inlist=\"thm theorem:claim\" \
       xml:id=\"Thmclaim1\"><tags><tag>Claim 1</tag><tag role=\"refnum\">1</tag><tag \
       role=\"typerefnum\">Claim 1</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
       xml:id=\"Thmclaim1.1\">Claim 1</text></tag><text font=\"bold\" \
       xml:id=\"Thmclaim1.2\">.</text></title><para xml:id=\"Thmclaim1.p1\"><p \
       xml:id=\"Thmclaim1.p1.1\"><text font=\"italic\" \
       xml:id=\"Thmclaim1.p1.1.1\">K</text></p></para></theorem>",
    ),
  ] {
    assert_element(&xml, "theorem", &[&format!("xml:id=\"{id}\"")], expected);
  }
}

/// thm-kv.sty:333-348: `\declaretheorem[<opts>]{<a>, <b>}[<opts>]` declares each name of the list with both option
/// lists. Red (78k r1, Perl the same): one environment "corollary, lemma", `{lemma}` undefined. Witness 2608.16854.
/// Repro package-bindings/thmtools_declaretheorem_declares_each_listed_name.
#[test]
fn thmtools_declaretheorem_declares_each_listed_name() {
  let xml = clean_xml(repro!(
    "package-bindings/thmtools_declaretheorem_declares_each_listed_name.tex"
  ));
  for (id, expected) in [
    (
      "S1.Thmtheorem1",
      "<theorem class=\"ltx_theorem_theorem\" inlist=\"thm theorem:theorem\" \
       xml:id=\"S1.Thmtheorem1\"><tags><tag>Theorem 1.1</tag><tag role=\"refnum\">1.1</tag><tag \
       role=\"typerefnum\">Theorem 1.1</tag></tags><title class=\"ltx_runin\"><tag><text \
       font=\"bold\" xml:id=\"S1.Thmtheorem1.1\">Theorem 1.1</text></tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem1.2\">.</text></title><para xml:id=\"S1.Thmtheorem1.p1\"><p \
       xml:id=\"S1.Thmtheorem1.p1.1\"><text font=\"italic\" \
       xml:id=\"S1.Thmtheorem1.p1.1.1\">T</text></p></para></theorem>",
    ),
    (
      "S1.Thmtheorem2",
      "<theorem class=\"ltx_theorem_lemma\" inlist=\"thm theorem:lemma\" \
       xml:id=\"S1.Thmtheorem2\"><tags><tag>Lemma 1.2</tag><tag role=\"refnum\">1.2</tag><tag \
       role=\"typerefnum\">Lemma 1.2</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem2.1\">Lemma 1.2</text></tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem2.2\">.</text></title><para xml:id=\"S1.Thmtheorem2.p1\"><p \
       xml:id=\"S1.Thmtheorem2.p1.1\"><text font=\"italic\" \
       xml:id=\"S1.Thmtheorem2.p1.1.1\">L</text></p></para></theorem>",
    ),
    (
      "S1.Thmtheorem3",
      "<theorem class=\"ltx_theorem_corollary\" inlist=\"thm theorem:corollary\" \
       xml:id=\"S1.Thmtheorem3\"><tags><tag>Corollary 1.3</tag><tag role=\"refnum\">1.3</tag><tag \
       role=\"typerefnum\">Corollary 1.3</tag></tags><title class=\"ltx_runin\"><tag><text \
       font=\"bold\" xml:id=\"S1.Thmtheorem3.1\">Corollary 1.3</text></tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem3.2\">.</text></title><para xml:id=\"S1.Thmtheorem3.p1\"><p \
       xml:id=\"S1.Thmtheorem3.p1.1\"><text font=\"italic\" \
       xml:id=\"S1.Thmtheorem3.p1.1.1\">C</text></p></para></theorem>",
    ),
    (
      "S1.Thmtheorem4",
      "<theorem class=\"ltx_theorem_definition\" inlist=\"thm theorem:definition\" \
       xml:id=\"S1.Thmtheorem4\"><tags><tag>Definition 1.4</tag><tag role=\"refnum\">1.4</tag><tag \
       role=\"typerefnum\">Definition 1.4</tag></tags><title class=\"ltx_runin\"><tag><text \
       font=\"bold\" xml:id=\"S1.Thmtheorem4.1\">Definition 1.4</text></tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem4.2\">.</text></title><para xml:id=\"S1.Thmtheorem4.p1\"><p \
       xml:id=\"S1.Thmtheorem4.p1.1\">D</p></para></theorem>",
    ),
    (
      "S1.Thmtheorem5",
      "<theorem class=\"ltx_theorem_remark\" inlist=\"thm theorem:remark\" \
       xml:id=\"S1.Thmtheorem5\"><tags><tag>Remark 1.5</tag><tag role=\"refnum\">1.5</tag><tag \
       role=\"typerefnum\">Remark 1.5</tag></tags><title class=\"ltx_runin\"><tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem5.1\">Remark 1.5</text></tag><text font=\"bold\" \
       xml:id=\"S1.Thmtheorem5.2\">.</text></title><para xml:id=\"S1.Thmtheorem5.p1\"><p \
       xml:id=\"S1.Thmtheorem5.p1.1\">R</p></para></theorem>",
    ),
  ] {
    assert_element(&xml, "theorem", &[&format!("xml:id=\"{id}\"")], expected);
  }
}

/// The `<ltx:listingline>`s of `xml`, ids stripped, its `Math` collapsed to `M`.
fn listing_lines(xml: &str) -> Vec<String> {
  let mut lines = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find("<listingline") {
    let line = xml_element(&rest[at..], "listingline", &[]).expect("a listingline");
    let close = rest[at..].find("</listingline>").expect("its close") + "</listingline>".len();
    rest = &rest[at + close..];
    let mut flat = String::new();
    let mut s = line.as_str();
    while let Some(m) = s.find("<Math") {
      flat.push_str(&s[..m]);
      let end = s.find("</Math>").map_or(s.len(), |e| e + "</Math>".len());
      flat.push('M');
      s = &s[end..];
    }
    flat.push_str(s);
    let mut out = String::new();
    let mut t = flat.as_str();
    while let Some(i) = t.find(" xml:id=\"") {
      out.push_str(&t[..i]);
      let after = &t[i + " xml:id=\"".len()..];
      t = &after[after.find('"').map_or(after.len(), |e| e + 1)..];
    }
    out.push_str(t);
    lines.push(out);
  }
  lines
}

/// algorithm2e's default `shortend` prints a block's end keyword as `\KwSty{\@firstword …}` (algorithm2e.sty:1485,
/// :1868, :1890-1893): a bold "end". Red (int49a, Perl the same, KNOWN_PERL_ERRORS #1056): plain "end if", "end
/// for". Witness 1810.03652. Repro listings-algorithms/algorithm2e_block_end_is_a_bold_short_keyword.
#[test]
fn algorithm2e_block_end_is_a_bold_short_keyword() {
  let xml = clean_xml(repro!(
    "listings-algorithms/algorithm2e_block_end_is_a_bold_short_keyword.tex"
  ));
  let lines = listing_lines(&xml);
  assert_eq!(
    lines[1..],
    [
      "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}a;</listingline>",
      "<listingline><text font=\"bold\">else</text></listingline>",
      "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}b;</listingline>",
      "<listingline><text font=\"bold\">end</text></listingline>",
      "<listingline><text font=\"bold\">for</text> <emph font=\"italic\">M</emph> <text \
       font=\"bold\">do</text></listingline>",
      "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}c;</listingline>",
      "<listingline><text font=\"bold\">end</text></listingline>",
    ],
    "{lines:#?}"
  );
}

/// `longend` prints the whole keyword, bold: "end if", "end for". Repro
/// listings-algorithms/algorithm2e_block_end_longend_is_the_whole_keyword.
#[test]
fn algorithm2e_block_end_longend_is_the_whole_keyword() {
  let xml = clean_xml(repro!(
    "listings-algorithms/algorithm2e_block_end_longend_is_the_whole_keyword.tex"
  ));
  let lines = listing_lines(&xml);
  assert_eq!(lines.len(), 8, "{lines:#?}");
  assert_eq!(
    lines[4],
    "<listingline><text font=\"bold\">end if</text></listingline>"
  );
  assert_eq!(
    lines[7],
    "<listingline><text font=\"bold\">end for</text></listingline>"
  );
}

/// `noend` prints no end line at all (a CONTROL the three-argument block keeps). Repro
/// listings-algorithms/algorithm2e_block_end_noend_prints_none.
#[test]
fn algorithm2e_block_end_noend_prints_none() {
  let xml = clean_xml(repro!(
    "listings-algorithms/algorithm2e_block_end_noend_prints_none.tex"
  ));
  assert_eq!(listing_lines(&xml), [
    "<listingline><text font=\"bold\">if</text> <emph font=\"italic\">M</emph> <text \
       font=\"bold\">then</text></listingline>",
    "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}a;</listingline>",
    "<listingline><text font=\"bold\">else</text></listingline>",
    "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}b;</listingline>",
    "<listingline><text font=\"bold\">for</text> <emph font=\"italic\">M</emph> <text \
       font=\"bold\">do</text></listingline>",
    "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}c;</listingline>",
  ]);
}

/// With an empty end keyword algorithm2e.sty:1868 prints the side comment as ` #3\relax` and no `\strut\par`, so the
/// next statement continues its line: "// after b;" as pdflatex prints it. Red (78k r1): the line ended after the
/// comment, "b;" on a line of its own. Repro listings-algorithms/algorithm2e_block_side_comment_without_end_keyword_continues_the_line.
#[test]
fn algorithm2e_block_side_comment_without_end_keyword_continues_the_line() {
  let xml = clean_xml(repro!(
    "listings-algorithms/algorithm2e_block_side_comment_without_end_keyword_continues_the_line.tex"
  ));
  assert_eq!(listing_lines(&xml), [
    "<listingline><text font=\"bold\">loop</text> <emph font=\"italic\">M</emph> <text \
       font=\"bold\">do</text></listingline>",
    "<listingline>\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}a;</listingline>",
    "<listingline><text font=\"typewriter\">// </text><text font=\"typewriter\">after </text>b;</listingline>",
  ]);
}

/// A paper's own algorithm2e.sty of release 5.00 (2013/01/06; :1532-1534) defines the block macros with two
/// arguments, the body and the end keyword: the binding keeps that form (the arity of the loaded `\algocf@@@block`
/// picks it), so the end keyword closes its line and an empty one prints none. Red (78k r1): the three-argument form
/// read the next statement's first token as a side comment, "end loop b" then ";", "d" then ";". Witnesses 1402.6889,
/// 1406.0089, 1410.1699, 1410.6122, 1502.02887, 1507.00513, 1507.00980, 1511.03137, 1511.05741, 1805.07253,
/// 2208.10273 (each ships v5.00 or earlier). Repro listings-algorithms/algorithm2e_v500 (a from-scratch stand-in).
#[test]
fn algorithm2e_v500_two_argument_block_prints_the_end_keyword_on_its_line() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/algorithm2e_v500/algorithm2e_v500_block_takes_two_arguments.tex"),
    &[(
      "algorithm2e.sty",
      repro!("listings-algorithms/algorithm2e_v500/algorithm2e.sty"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(listing_lines(&xml), [
    "<listingline><text font=\"bold\">loop</text></listingline>",
    "<listingline><rule height=\"100%\" width=\"1px\"/>\u{2003}a;</listingline>",
    "<listingline><text font=\"bold\">end loop</text></listingline>",
    "<listingline>b;</listingline>",
    "<listingline><text font=\"bold\">body</text></listingline>",
    "<listingline><rule height=\"100%\" width=\"1px\"/>\u{2003}c;</listingline>",
    "<listingline>d;</listingline>",
  ]);
}

/// `\nobibliography*` stores each `.bbl` entry and prints no list; `\bibentry{k1}` prints the stored text without its
/// final period (bibentry.sty:42-66) and the later `\bibliography` the one list. Red (int49a): "Paper: ." and two
/// `ltx:bibliography`. Witnesses 2505.07606, 2210.16098. Repro index-bib/bibentry_prints_the_stored_entry.
#[test]
fn bibentry_prints_the_stored_entry() {
  let (log, xml) = convert_files_with(
    repro!("index-bib/bibentry_prints_the_stored_entry.tex"),
    &[(
      "t.bbl",
      repro!("index-bib/bibentry_prints_the_stored_entry.bbl"),
    )],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">Paper: A.\u{a0}Able. First paper. 2001. Also <cite class=\"ltx_citemacro_citet\"><bibref bibrefs=\"k2\" \
     separator=\";\" show=\"Authors Phrase1YearPhrase2\" yyseparator=\",\"><bibrefphrase>(</bibrefphrase>\
     <bibrefphrase>)</bibrefphrase></bibref></cite>.</p>",
  );
  assert_eq!(xml.matches("<bibliography ").count(), 1, "{xml}");
}

/// `\nobibliography{<db>}` without the star stores the entries too, and keeps the list where it stands as the
/// target its citations resolve to (OXIDIZED_DESIGN_DIVERGENCES #831): the section's `\bibentry`s print both
/// entries, the `\citet` reads "Baker (2002)". Red (int49a): both lines ".". Witness 2210.16098. Repro
/// index-bib/bibentry_nobibliography_keeps_the_list_its_citations_resolve_to.
#[test]
fn bibentry_nobibliography_keeps_the_list_its_citations_resolve_to() {
  let html = html_of(
    repro!("index-bib/bibentry_nobibliography_keeps_the_list_its_citations_resolve_to.tex"),
    &[(
      "t.bbl",
      repro!("index-bib/bibentry_nobibliography_keeps_the_list_its_citations_resolve_to.bbl"),
    )],
  );
  assert_element(
    &html,
    "p",
    &["id=\"Sx1.p1.1\""],
    "<p class=\"ltx_p\" id=\"Sx1.p1.1\">A.\u{a0}Able. First paper. 2001.<br class=\"ltx_break\">B.\u{a0}Baker. Second paper. \
     2002.</p>",
  );
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    "<p class=\"ltx_p\" id=\"p1.1\">Also <cite class=\"ltx_cite ltx_citemacro_citet\">Baker (<a class=\"ltx_ref\" \
     href=\"#bib.bib2\" title=\"\">2002</a>)</cite>.</p>",
  );
  assert_eq!(
    html.matches("class=\"ltx_bibliography\"").count(),
    1,
    "{html}"
  );
}

/// footmisc's `symbol` marks footnotes with `\fnsymbol{footnote}` (footmisc.sty:65), and `\footref` prints the mark.
/// Red (int49a, Perl the same, KNOWN_PERL_ERRORS #1057): marks 1, 2, `\footref` "1". Witness 1907.00006. Repro
/// package-bindings/footmisc_symbol_marks_footnotes_with_symbols.
#[test]
fn footmisc_symbol_marks_footnotes_with_symbols() {
  let html = html_of(
    repro!("package-bindings/footmisc_symbol_marks_footnotes_with_symbols.tex"),
    &[],
  );
  assert_element(
    &html,
    "p",
    &["id=\"p1.1\""],
    "<p class=\"ltx_p\" id=\"p1.1\">A<span class=\"ltx_note ltx_role_footnote\" \
     id=\"footnote1\"><sup class=\"ltx_note_mark\">*</sup><span class=\"ltx_note_outer\"><span \
     class=\"ltx_note_content\"><sup class=\"ltx_note_mark\">*</sup><span class=\"ltx_tag \
     ltx_tag_note\">*</span>First note.</span></span></span> B<span class=\"ltx_note \
     ltx_role_footnote\" id=\"footnote2\"><sup class=\"ltx_note_mark\">†</sup><span \
     class=\"ltx_note_outer\"><span class=\"ltx_note_content\"><sup \
     class=\"ltx_note_mark\">†</sup><span class=\"ltx_tag ltx_tag_note\">†</span>Second \
     note.</span></span></span> again<a class=\"ltx_ref\" href=\"#footnote1\" title=\"footnote \
     *\"><span class=\"ltx_text ltx_ref_tag\">*</span></a>.</p>",
  );
}

/// `symbol*` marks with `\@fnsymbol\c@footnote` in footmisc's `lamport*-robust` set, which prints the number past
/// its sixteen symbols (footmisc.sty:67-70, :522-531). Red (int49a): marks 1, 2, 17. Witness 1907.00006. Repro
/// package-bindings/footmisc_symbol_star_prints_the_number_past_the_symbols.
#[test]
fn footmisc_symbol_star_prints_the_number_past_the_symbols() {
  let xml = clean_xml(repro!(
    "package-bindings/footmisc_symbol_star_prints_the_number_past_the_symbols.tex"
  ));
  let marks: Vec<&str> = xml
    .match_indices("<note ")
    .filter_map(|(i, _)| {
      let tag = &xml[i..i + xml[i..].find('>')?];
      let rest = &tag[tag.find(" mark=\"")? + 7..];
      Some(&rest[..rest.find('"')?])
    })
    .collect();
  assert_eq!(marks, ["*", "†", "17"], "{xml}");
}

/// `\citetext{…}` is the text in natbib's current brackets (natbib.sty:739 `\NAT@open#1\NAT@close`): "(see Able et
/// al., 2001)" under `round`, "[see …]" under `square`. Red (int49a, Perl the same, KNOWN_PERL_ERRORS #1058): no
/// brackets. Repro index-bib/natbib_citetext_keeps_its_brackets.
#[test]
fn natbib_citetext_keeps_its_brackets() {
  let tex = repro!("index-bib/natbib_citetext_keeps_its_brackets.tex");
  let cite = |html: &str| {
    let p = xml_element(html, "p", &["id=\"p1.1\""]).expect("the paragraph");
    xml_element(&p, "cite", &["class=\"ltx_cite\""]).expect("the citetext cite")
  };
  let inner = "<cite class=\"ltx_cite ltx_citemacro_citep\">Able et\u{a0}al., <a class=\"ltx_ref\" \
               href=\"#bib.bib1\" title=\"\">2001</a></cite>";
  assert_eq!(
    cite(&html_of(tex, &[])),
    format!("<cite class=\"ltx_cite\">(see {inner})</cite>")
  );
  assert_eq!(
    cite(&html_of(&tex.replace("[round]", "[square]"), &[])),
    format!("<cite class=\"ltx_cite\">[see {inner}]</cite>")
  );
}

/// `\nobibliography*` in the preamble stores the entries and prints nothing: the one bibliography is the later
/// `\bibliography`'s, after the chapter, nothing bibliographic before "Cover". Red (67int13, 68s, int49a): two
/// `ltx:bibliography`, the first ahead of the cover, and bibentry's `\AtEndDocument` Warning. Witness 2010.07703
/// (806 bibitems at the head of the XML). Repro index-bib/nobibliography_star_prints_nothing.
#[test]
fn nobibliography_star_prints_nothing() {
  let xml = clean_xml(repro!("index-bib/nobibliography_star_prints_nothing.tex"));
  assert_eq!(xml.matches("<bibliography ").count(), 1, "{xml}");
  let cover = xml.find(">Cover</text>").expect("the cover");
  let chapter = xml.find("<chapter ").expect("the chapter");
  let bibliography = xml.find("<bibliography ").expect("the bibliography");
  assert!(cover < chapter && chapter < bibliography, "{xml}");
}
