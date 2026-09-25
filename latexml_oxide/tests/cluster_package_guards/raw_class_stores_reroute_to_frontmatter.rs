//! K11 (batch 56dj): after a `.cls` loads raw, every setter in the surveyed
//! table whose macro body is a pure one-argument store (`\gdef\@x{#1}`) is
//! rerouted to the frontmatter API of its kind, and the class's
//! `\@maketitle` — their typesetter, which LaTeXML's locked `\maketitle`
//! never runs — is discarded as Perl does. Perl drops every such store
//! (OmniBus's generic table is bypassed under raw class loading).
//! `frontmatter_stores.rs`.

/// jpsj2.cls:841-847 stores `\abst`/`\inst`/`\kword`/`\recdate` with
/// `\long\def\abst#1{\long\gdef\@abst{#1}}`; the frontmatter carries them.
#[test]
fn jpsj2_stores_become_frontmatter() {
  if !latexml::util::test::kpse_has("jpsj2.cls") {
    return;
  }
  let tex = std::fs::read_to_string("tests/cluster_regressions/jpsj2_stores_frontmatter.tex")
    .expect("fixture");
  let (stderr, xml) = super::convert_with(&tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &[],
    r##"<creator role="author"><personname>Ann Author</personname><contact name="Affiliation: " role="affiliation">Department of Physics, Tokyo</contact></creator>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "abstract",
    &[],
    r##"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>The abstract text.</p></abstract>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "keywords",
    &[],
    r##"<keywords name="Keywords: ">electrons, phonons</keywords>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "date",
    &[],
    r##"<date name="Received " role="received">May 1, 2024</date>"##,
  );
  let title_at = xml.find("<title>").expect("title");
  let first_para = xml.find("<para").unwrap_or(usize::MAX);
  assert!(title_at < first_para, "frontmatter first:\n{xml}");
}

/// Only table names with a store body are touched: a store named `\logo`
/// (not metadata) and a `\kword` whose body is not a store stay the
/// class's own, while a real `\kword` store becomes keywords.
#[test]
fn only_table_names_with_store_bodies_are_rerouted() {
  let cls = "\\ProvidesClass{pkstore}\n\\LoadClass{article}\n\
\\newcommand\\kword[1]{\\gdef\\@kword{#1}}\n\
\\newcommand\\logo[1]{\\gdef\\@logo{#1}}\n\
\\newcommand\\pacs[1]{\\textbf{#1}}\n\
\\def\\@kword{}\\def\\@logo{}\n\
\\def\\@maketitle{\\begin{center}{\\Large\\@title}\\par\\@author\\par\\@kword\\par\\@logo\\end{center}}\n";
  let tex = "\\documentclass{pkstore}\n\\title{T}\\author{A}\\kword{alpha, beta}\\logo{LOGO}\n\
\\begin{document}\\maketitle\\pacs{12.34}\\end{document}\n";
  let (stderr, xml) = super::convert_files_with(
    tex,
    &[("pkstore.cls", cls)],
    Some("[rawstyles,rawclasses]latexml.sty"),
  );
  assert_eq!(super::error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "keywords",
    &[],
    r##"<keywords name="Keywords: ">alpha, beta</keywords>"##,
  );
  assert_eq!(
    xml.matches("LOGO").count(),
    1,
    "the uncaptured store deposits once:\n{xml}"
  );
  assert_eq!(
    xml.matches("alpha, beta").count(),
    1,
    "a captured store is not deposited again:\n{xml}"
  );
  latexml::util::test::assert_element(&xml, "p", &["align="], r##"<p align="center">LOGO</p>"##);
  assert!(
    xml.contains(r##"<p><text font="bold">12.34</text></p>"##),
    "the non-store \\pacs stays the class's own:\n{xml}"
  );
}
