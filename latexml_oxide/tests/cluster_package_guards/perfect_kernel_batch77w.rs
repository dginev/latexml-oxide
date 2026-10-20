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
