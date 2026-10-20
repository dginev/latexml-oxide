//! Red/green guards for batch 77w: "a shipped file that `\Provides` a bound name loads that binding" (user ruling
//! 2026-10-20; OXIDIZED_DESIGN_DIVERGENCES #731, the declaration route). A `.sty`/`.cls` the paper ships under a name
//! with no binding of its own that declares itself another file a binding stands for (`\ProvidesPackage{natbib}` in
//! natbib_vw.sty, `\ProvidesClass{revtex4}` in qm2008_abs.cls, `\ProvidesPackage{aistats2023}` in a renamed aistats
//! style) loads that binding with the request's options, then is read after it as the stand-in's copy: its own commands
//! are added, the binding's kept. Repros `tools/perfect_kernel/repros/stand-in-copies/provides_route/`.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/stand-in-copies/provides_route/",
      $path
    ))
  };
}

const NATBIB_VW: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/mn2e_natbib_copy/natbib_vw.sty");

/// A renamed natbib copy (`\ProvidesPackage{natbib}`, natbib_vw.sty; 0802.4100, 1003.3017, astro-ph0512042) is the
/// natbib binding: author-year citations with their label text, where the copy read raw printed "?" with a "Citation
/// undefined" Warning per citation. pdflatex: "Schneider et al. (2010), Smith (2001), (Smith, 2001)."
#[test]
fn natbib_copy_is_natbib() {
  let (log, xml) = convert_files_with(
    repro!("natbib_copy_is_natbib.tex"),
    &[("natbib_vw.sty", NATBIB_VW)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:fallback:natbib_vw The paper's natbib_vw.sty declares itself natbib: loading the natbib.sty binding, then \
       the copy as its stand-in's"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\"><cite class=\"ltx_citemacro_citet\"><bibref bibrefs=\"s10\" separator=\";\" \
     show=\"Authors Phrase1YearPhrase2\" yyseparator=\",\"><bibrefphrase>(</bibrefphrase><bibrefphrase>)</bibrefphrase>\
     </bibref></cite>, <cite class=\"ltx_citemacro_citet\"><bibref bibrefs=\"a\" separator=\";\" \
     show=\"Authors Phrase1YearPhrase2\" yyseparator=\",\"><bibrefphrase>(</bibrefphrase><bibrefphrase>)</bibrefphrase>\
     </bibref></cite>, <cite class=\"ltx_citemacro_citep\">(<bibref bibrefs=\"a\" separator=\";\" \
     show=\"AuthorsPhrase1Year\" yyseparator=\",\"><bibrefphrase>, </bibrefphrase></bibref>)</cite>.</p>",
  );
  assert!(
    xml.contains("<tag role=\"refnum\">Schneider et\u{a0}al. (2010)</tag>"),
    "{xml}"
  );
}

/// The witnesses' own shape: mn2e with the natbib_vw.sty copy (index-bib/mn2e_natbib_copy; SYNC 77s row).
#[test]
fn natbib_renamed_copy_citations_resolve() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/mn2e_natbib_copy/natbib_renamed_copy_citations_resolve.tex"
    ),
    &[("natbib_vw.sty", NATBIB_VW)],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains(">?<"), "{xml}");
  assert!(
    xml.contains("<tag role=\"refnum\">Schneider et al. (2010)</tag>"),
    "{xml}"
  );
  assert!(
    xml.contains("<tag role=\"refnum\">Smith (2001)</tag>"),
    "{xml}"
  );
}

/// A renamed REVTeX 4 class (`\ProvidesClass{revtex4}`: qm2008_abs.cls, snp.cls; 0805.2253, 0806.0481, 0805.4759) is
/// the revtex4 binding — the creator keeps its affiliation — and its own `\qmextra` stands. Also with the declaration
/// naming a macro (`\def\ClassName{revtex4}\ProvidesClass{\ClassName}`) and as `\ProvidesFile{revtex4.cls}`. pdflatex:
/// "Title / A. Author / Univ", "Text QM-EXTRA."
#[test]
fn revtex4_copy_class_is_revtex4() {
  let class = repro!("qmcopy.cls");
  let declaration = "\\ProvidesClass{revtex4}";
  for class in [
    class.to_string(),
    class.replace(
      declaration,
      "\\def\\ClassName{revtex4}\n\\ProvidesClass{\\ClassName}",
    ),
    class.replace(declaration, "\\ProvidesFile{revtex4.cls}"),
  ] {
    let (log, xml) = convert_files_with(
      repro!("revtex4_copy_class.tex"),
      &[("qmcopy.cls", &class)],
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert!(
      log.contains("Info:fallback:qmcopy The paper's qmcopy.cls declares itself revtex4"),
      "{log}"
    );
    assert_element(
      &xml,
      "creator",
      &["role=\"author\""],
      "<creator role=\"author\"><personname>A. Author</personname><contact name=\"Affiliation:\u{a0}\" \
       role=\"affiliation\">Univ</contact></creator>",
    );
    assert_element(
      &xml,
      "p",
      &["xml:id=\"p1.1\""],
      "<p xml:id=\"p1.1\">Text QM-EXTRA.</p>",
    );
  }
}

/// A renamed aistats style (`\ProvidesPackage{aistats2023}`, which the versioned fallback reduces to aistats; SYNC row
/// "aistats renamed copies", 77o) is the aistats binding: a title and a creator where the copy read raw gave centred
/// paragraphs; its own `\styleextra` stands. pdflatex: "A Title / Ann Author / Univ", "Text STYLE-EXTRA."
#[test]
fn aistats_renamed_copy_is_aistats() {
  let (log, xml) = convert_files_with(
    repro!("aistats_renamed_copy.tex"),
    &[("mystyle.sty", repro!("mystyle.sty"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:fallback:mystyle The paper's mystyle.sty declares itself aistats2023: loading the aistats.sty binding"
    ),
    "{log}"
  );
  assert_element(&xml, "title", &[], "<title>A Title</title>");
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>Ann Author</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ</contact></creator>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text STYLE-EXTRA.</p>",
  );
}

/// No route: a copy declaring its own name, one declaring a name no binding answers, a `.sty` declaring a bound class
/// (another kind), and a request with its own binding (a shipped url.sty declaring natbib loads the url binding). The
/// shipped style is read raw, as before. pdflatex: "SELF".
#[test]
fn provides_route_exclusions() {
  let style = repro!("selfnamed.sty");
  let declaration = "\\ProvidesPackage{selfnamed}";
  let tex = repro!("not_routed.tex");
  for (tex, file, style) in [
    (tex.to_string(), "selfnamed.sty", style.to_string()),
    (
      tex.to_string(),
      "selfnamed.sty",
      style.replace(declaration, "\\ProvidesPackage{nosuchfile}"),
    ),
    (
      tex.to_string(),
      "selfnamed.sty",
      style.replace(declaration, "\\ProvidesClass{revtex4}"),
    ),
    (
      tex.replace(
        "\\usepackage{selfnamed}",
        "\\usepackage{url}\\newcommand\\selfextra{SELF}",
      ),
      "url.sty",
      style
        .replace(declaration, "\\ProvidesPackage{natbib}")
        .replace("\\newcommand\\selfextra{SELF}", ""),
    ),
  ] {
    let (log, xml) = convert_files_with(&tex, &[(file, &style)], Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{style}: {log}"
    );
    assert!(!log.contains("declares itself"), "{style}: {log}");
    assert_element(
      &xml,
      "p",
      &["xml:id=\"p1.1\""],
      "<p xml:id=\"p1.1\">SELF</p>",
    );
  }
}

/// A routed copy re-saving a command its binding already saved and wrapped (`\let\jmlr@org@newtheorem\newtheorem`,
/// jmlr.cls:753-754, which the jmlr binding's jmlrutils.sty makes too) keeps the saved original: the `\let` is declined
/// in the copy's code too, so the copy's own `\newtheorem{example}` does not call itself (1506.02582, 2304.14217:
/// timeouts). pdflatex: "Example 1 Text."
#[test]
fn copy_resaves_a_wrapped_command() {
  let (log, xml) = convert_files_with(
    repro!("copy_resaves_a_wrapped_command.tex"),
    &[("jmlrcopy.cls", repro!("jmlrcopy.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:provide:\\jmlr@org@newtheorem \\jmlr@org@newtheorem keeps its saved \\newtheorem: the paper's \
       jmlrcopy.cls saves \\newtheorem again after its binding redefined it"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "title",
    &["class=\"ltx_runin\""],
    "<title class=\"ltx_runin\"><tag><text font=\"bold\" xml:id=\"Thmtheorem1.2\">Example 1</text></tag><text \
     font=\"bold\" xml:id=\"Thmtheorem1.3\">.</text></title>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"Thmtheorem1.p1.1\""],
    "<p xml:id=\"Thmtheorem1.p1.1\"><text font=\"italic\" xml:id=\"Thmtheorem1.p1.1.1\">Text.</text></p>",
  );
}

/// A routed copy's own load of a package its binding loaded on the class's behalf is that load: the copy's options
/// apply, with no "Option clash" (1811.10063's elsarticle.cls copy `\RequirePackage[square,…]{natbib}`). pdflatex:
/// "See [1]."
#[test]
fn copy_loads_its_binding_package() {
  let (log, xml) = convert_files_with(
    repro!("copy_loads_its_binding_package.tex"),
    &[("elscopy.cls", repro!("elscopy.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("Applying options 'square' from a repeat load of natbib.sty"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">See <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a\" separator=\",\" \
     show=\"Number\" yyseparator=\",\"/>]</cite>.</p>",
  );
}

/// A routed copy's own write to LaTeX's document hook (`\g@addto@macro\@begindocumenthook`, ltxgrid's
/// `\prepdef\@begindocumenthook` in qm2008_abs.cls:3233 and snp.cls) is document-time code, which a stand-in's copy does
/// not register (OD #731 rule 5), as its `\AtBeginDocument` code is not: ltxgrid's page grid ran over the revtex4
/// binding's names at `\begin{document}` (0805.2253, 0806.0481, 0805.4759: 7 "Missing number" Warnings each).
/// pdflatex: "Text HOOKED." (the ruled divergence).
#[test]
fn copy_writes_document_hook() {
  let (log, xml) = convert_files_with(
    repro!("copy_writes_document_hook.tex"),
    &[("hookcopy.cls", repro!("hookcopy.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "The paper's hookcopy.cls defers code to the document (\\@begindocumenthook); not registered: its binding \
       stands in for that code"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Text UNHOOKED.</p>",
  );
}

/// A class the paper ships that declares one of LaTeX's own classes (aip-cp.cls:24 `\ProvidesClass{article}`;
/// 1506.04050's five authors, 1604.08112's two) is a derivative of it, read as a raw class over the article binding (OD
/// #731 rule 13): its accumulating `\author[aff1]{…}` keeps every author — the kernel `\author` appends each call and
/// replays the class's own setter — and its `\affil` list reaches the page through its title page's deposit. Read as
/// the binding's stand-in copy (77w r1), the locked `\author` kept only the last call, silently. pdflatex: "A Paper /
/// M. Bruemmer^aff1, S. Sekula^aff1 / ^aff1 Department of Physics, SMU".
#[test]
fn standard_class_derivative_keeps_every_author() {
  let (log, xml) = convert_files_with(
    repro!("standard_class_derivative_authors.tex"),
    &[("aipcopy.cls", repro!("aipcopy.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:provide:aipcopy.cls The paper's aipcopy.cls derives from a LaTeX class: read as a raw class over its binding"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>M. Bruemmer</personname></creator>",
  );
  assert_element(
    &xml,
    "creator",
    &["before=\"\u{2003}\u{2003}\"", "role=\"author\""],
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>S. Sekula</personname></creator>",
  );
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p align=\"center\" xml:id=\"p1.1\"><sup xml:id=\"p1.1.1\">aff1</sup><text font=\"italic\" \
     xml:id=\"p1.1.2\">Department of Physics, SMU</text></p></para>",
  );
}

/// The derivative's other patches of the kernel's class stand as it makes them (OD #731 rule 13): its size file
/// (`\input{size1\@ptsize.clo}` under `[11pt]`) sets the nominal font size (0709.4467's mf.cls: 10.95; 1808.10740's
/// antonio.cls: 12), and its `\newcommand`s of the names the article binding assigned define them, as the class read in
/// article.cls's place does (aip-cp.cls:875-878, 1506.04050: "REFERENCES", "FIGURE 1"; HERON.cls:682, 1903.06504).
/// pdflatex: "FIGURE 1: A caption.", "REFERENCES".
#[test]
fn standard_class_derivative_patches_stand() {
  let (log, xml) = convert_files_with(
    repro!("standard_class_derivative_patches.tex"),
    &[("stdderiv.cls", repro!("stdderiv.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<?latexml nominal-font-size=\"10.95\"?>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\": \">FIGURE\u{a0}1</tag>A caption.</caption>",
  );
  assert_element(
    &xml,
    "title",
    &[],
    "<title><tag close=\" \">1</tag>Introduction</title>",
  );
  assert!(xml.contains("<title>REFERENCES</title>"), "{xml}");
  // Its `\newcommand\appendix` (article.cls's, stdderiv.cls) does not replace the binding's, which makes the
  // appendices LaTeXML's `<appendix>` (the 77w r2 re-review's finding 1: 0705.2210, hep-ph0008011, cond-mat0402505,
  // quant-ph0409078 lost theirs), and its `\newcounter{figure}` keeps the binding's ids (`S1.F1`, not `F1`: probe1).
  assert_element(
    &xml,
    "appendix",
    &["inlist=\"toc\"", "xml:id=\"A1\""],
    "<appendix inlist=\"toc\" xml:id=\"A1\"><tags><tag>Appendix A</tag><tag role=\"refnum\">A</tag><tag \
     role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix A</tag>Proofs</title><toctitle><tag \
     close=\" \">A</tag>Proofs</toctitle><para xml:id=\"A1.p1\"><p xml:id=\"A1.p1.1\">Appendix \
     text.</p></para></appendix>",
  );
  assert!(
    xml.contains("<figure inlist=\"lof\" placement=\"h\" xml:id=\"S1.F1\">"),
    "{xml}"
  );
}

/// A derivative of the book class (a full copy declaring `\ProvidesClass{book}`: hep-ex0405054's book_modified.cls,
/// math0301023's khbook.cls) keeps the book binding's structural commands — its `\newcommand\frontmatter`,
/// `\mainmatter`, `\backmatter` and `\appendix` do not replace them (OD #731 rule 13: only the binding's text names
/// yield; the 77w r2 re-review's finding 1, its probe4). pdflatex: "Preface", "Chapter 1 Intro", "1.1 Sec",
/// "Appendix A Extra", "Afterword"; the last is "Appendix B Afterword" as under `\documentclass{book}`, whose
/// `\backmatter` numbers on (Perl book.cls.ltxml:121 alike).
#[test]
fn standard_class_derivative_keeps_book_structure() {
  let (log, xml) = convert_files_with(
    repro!("standard_class_derivative_book.tex"),
    &[("bookcopy.cls", repro!("bookcopy.cls"))],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:provide:bookcopy.cls The paper's bookcopy.cls derives from a LaTeX class: read as a raw class over its binding"
    ),
    "{log}"
  );
  let titles: Vec<&str> = xml
    .match_indices("<title>")
    .map(|(at, _)| {
      let rest = &xml[at..];
      &rest[..rest
        .find("</title>")
        .map_or(rest.len(), |end| end + "</title>".len())]
    })
    .collect();
  assert_eq!(
    titles,
    [
      "<title>Preface</title>",
      "<title><tag close=\" \">Chapter 1</tag>Intro</title>",
      "<title><tag close=\" \">1.1</tag>Sec</title>",
      "<title><tag close=\" \">Appendix A</tag>Extra</title>",
      "<title><tag close=\" \">Appendix B</tag>Afterword</title>",
    ],
    "{xml}"
  );
  assert_eq!(xml.matches("<appendix ").count(), 2, "{xml}");
  assert!(
    xml.contains("<chapter inlist=\"toc\" xml:id=\"Chx1\">"),
    "{xml}"
  );
}

/// A file that declares three files or more is a bundle, not a copy (cauchy.sty: theorem, exscale, epsfig, apacite;
/// 1312.3078), and a declaration after a file's top-level `\endinput` is its documentation's (the 77w review's
/// finding 8): both are read raw as before the route. pdflatex: "EXSCALE & TOOL."
#[test]
fn provides_route_skips_bundles_and_trailers() {
  let (log, xml) = convert_files_with(
    repro!("provides_route_bundles.tex"),
    &[
      ("bundle.sty", repro!("bundle.sty")),
      ("mytools.sty", repro!("mytools.sty")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("declares itself"), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">EXSCALE &amp; TOOL.</p>",
  );
}

/// A package a stand-in binding's load loaded first is the copy's own load for that binding's copy only (the 77w
/// review's finding 6): another stand-in's copy that loads it with other options is LaTeX's "Option clash", the first
/// load's options kept (the elsarticle copy's own repeat, 1811.10063, stays `copy_loads_its_binding_package`).
/// pdflatex: 1 error, "See [1]."
#[test]
fn another_copys_repeat_load_clashes() {
  let (log, xml) = convert_files_with(
    repro!("another_copy_loads_the_package.tex"),
    &[
      ("elscopy0.cls", repro!("elscopy0.cls")),
      ("ijcaicopy.sty", repro!("ijcaicopy.sty")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains(
      "Error:latex:options Option clash for package natbib: options 'round' of a repeat load dropped, as LaTeX keeps \
       the first load's"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">See <cite class=\"ltx_citemacro_citep\">[<bibref bibrefs=\"a\" separator=\",\" \
     show=\"Number\" yyseparator=\",\"/>]</cite>.</p>",
  );
}

/// A standard-class derivative defines over its binding only the standard class binding's own names (OD #731 rule
/// 13), not those of a stand-in it loads (the 77w r2 re-review's finding 4): `\providecommand\jname` after
/// `\RequirePackage{jheppub}` keeps jheppub's "JHEP", as LaTeX does. pdflatex: "Published in JHEP."
#[test]
fn standard_class_derivative_defines_only_its_class_names() {
  let (log, xml) = convert_files_with(
    repro!("standard_class_derivative_scope.tex"),
    &[
      ("scopederiv.cls", repro!("scopederiv.cls")),
      ("jheppub.sty", repro!("jheppub.sty")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">Published in JHEP.</p>",
  );
}

/// A `.tex` stand-in binding records itself as the loader of the packages its load loads first, as a package or class
/// stand-in does (`<file>:loaded_by_stand_in`, `StandInBindingName`; the 77w r2 re-review's finding 3): mssymb_tex.rs's
/// amsfonts. pdflatex: "The reals R."
#[test]
fn tex_stand_in_names_its_package_loads() {
  let files: [(&str, &[u8]); 1] = [("mssymb.tex", repro!("mssymb.tex").as_bytes())];
  let (log, _xml, loader) = latexml::util::test::convert_bytes_files_then(
    repro!("tex_stand_in_loads_packages.tex").as_bytes(),
    &files,
    Some("ar5iv.sty"),
    |_, _| latexml_core::state::lookup_string("amsfonts.sty:loaded_by_stand_in"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(loader, "mssymb", "{log}");
}

/// A derivative beside a self-named `article.cls` the paper also ships still declares the LaTeX class (the 77w r2
/// re-review's finding 6): the base class is looked up in the TeX tree, not the paper's directory first, so the
/// derivative is read as a raw class over its binding and keeps every author (as
/// `standard_class_derivative_keeps_every_author`).
#[test]
fn standard_class_derivative_beside_a_self_named_article() {
  let (log, xml) = convert_files_with(
    repro!("standard_class_derivative_beside_article.tex"),
    &[
      ("aipcopy.cls", repro!("aipcopy.cls")),
      ("article.cls", repro!("stdderiv.cls")),
    ],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains(
      "Info:provide:aipcopy.cls The paper's aipcopy.cls derives from a LaTeX class: read as a raw class over its binding"
    ),
    "{log}"
  );
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>M. Bruemmer</personname></creator>",
  );
  assert_eq!(xml.matches("<creator ").count(), 2, "{xml}");
}
