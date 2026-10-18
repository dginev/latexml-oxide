//! Red/green guards for int26 and int27, from the silent-loss inventory (agent_reports/2026-10-18_silent_loss_inventory.md) and
//! the `undefined:\thetitle` cluster: a frontmatter body still open at `\end{document}` ends there and keeps its text
//! (sect02.rs `\lx@end@open@frontmatter`; was an empty `<document/>` with 0 errors), and a refused `\title` replays for
//! its side effects as `\author`'s does (sect05.rs `\title`, the support bindings' `\title`; OXIDIZED_DESIGN #482). Each
//! green asserts the three targets: the log's tally, the authored text, and the element that carries it.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A `sectioning-frontmatter` repro's log and squashed XML.
macro_rules! convert_repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert!(!log.lines().any(|l| l.starts_with("Fatal:")), "{log}");
    (log, squash(&xml))
  }};
}

/// The abstract both repros below keep, whole.
const TWO_PARAGRAPH_ABSTRACT: &str = r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract only text.</p> <p xml:id="abstract1.2">More abstract text.</p> </abstract>"#;

/// A brace-less `\abstract` no sectioning command ended: both paragraphs in the abstract, clean, as pdflatex prints them
/// (was `<document/>`, 2 warnings).
#[test]
fn braceless_abstract_unclosed_at_end_document_keeps_its_text() {
  let (log, xml) =
    convert_repro!("braceless_abstract_unclosed_at_end_document_keeps_the_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(TWO_PARAGRAPH_ABSTRACT), "{xml}");
}

/// An unbalanced `\abstract{`: the same abstract; its open group is the one warning, as pdflatex's "\end occurred inside
/// a group".
#[test]
fn braced_abstract_unclosed_at_end_document_keeps_its_text() {
  let (log, xml) =
    convert_repro!("braced_abstract_unclosed_at_end_document_keeps_the_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log
      .lines()
      .any(|l| l.starts_with("Warning:unexpected:\\lx@end@abstract")),
    "{log}"
  );
  assert!(xml.contains(TWO_PARAGRAPH_ABSTRACT), "{xml}");
}

/// An unclosed `\begin{abstract}`: its `\section` closes it (user ruling 2026-10-18), so the section and its text follow
/// the abstract; the `\end{abstract}` it still owes is LaTeX's error at `\end{document}`, raised once on purpose and
/// pinned by count (pdflatex and Perl give 1).
#[test]
fn environment_abstract_unclosed_at_end_document_errs_and_keeps_its_text() {
  let (log, xml) = convert_repro!("unclosed_abstract_at_end_document_keeps_the_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.lines().any(|l| l
      .starts_with("Error:unexpected:\\end{document} \\begin{abstract} ended by \\end{document}")),
    "{log}"
  );
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract only text.</p> </abstract>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title><tag close=" ">1</tag>Body</title> <para xml:id="S1.p1"> <p xml:id="S1.p1.1">Body text.</p> </para>"#), "{xml}");
}

/// A `\section*` inside a balanced abstract closes it (user ruling 2026-10-18; was kept nested, schema-invalid; Perl
/// errs): the heading and its text follow the abstract, and the `\end{abstract}` after it is a no-op. Clean, as pdflatex.
#[test]
fn section_inside_balanced_abstract_closes_it() {
  let (log, xml) = convert_repro!("abstract_section_closes_the_abstract.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract text.</p> </abstract>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title>Keywords</title>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="Sx1.p1.1">alpha, beta</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<title><tag close=" ">1</tag>Introduction</title>"#),
    "{xml}"
  );
}

/// Keywords left open inside an abstract left open: both bodies end at `\end{document}` and keep their text; LaTeX's
/// one error names the innermost open environment (`\@checkend` against `\@currenvir`, as pdflatex), raised on purpose
/// and pinned by count (int26 review p8; was an empty `<document/>`).
#[test]
fn keywords_unclosed_inside_unclosed_abstract_keep_their_text() {
  let (log, xml) = convert_repro!("keywords_unclosed_inside_unclosed_abstract.tex");
  assert_eq!(error_count(&log), 1, "{log}");
  assert!(
    log.lines().any(|l| l.starts_with(
      "Error:unexpected:\\end{document} \\begin{IEEEkeywords} ended by \\end{document}"
    )),
    "{log}"
  );
  for text in ["Abstract text here.", "alpha, beta", "Body text here."] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<keywords name="Index Terms: ">"#), "{xml}");
}

/// The cvpr template's `\renewcommand{\title}[1]{\titleold{#1}\newcommand{\thetitle}{#1}}` in a package: the refused
/// `\title` replays, so `\maketitlesupplementary` prints the title (was `undefined:\thetitle`, 2308.03900 and 15 more).
#[test]
fn refused_package_title_defines_thetitle() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/title_replay_defines_thetitle.tex"
    ),
    &[(
      "ttl.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/ttl.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(
    xml.matches("<title>Universal Widgets</title>").count(),
    1,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text font="bold" fontsize="144%" xml:id="p2.1.1">Universal Widgets<break/><text font="medium" xml:id="p2.1.1.1">Supplementary Material</text></text>"#),
    "{xml}"
  );
}

/// The same idiom under acmart, whose `\title` is ams_support's: it replays there too (2506.01799).
#[test]
fn refused_document_title_defines_thetitle_under_acmart() {
  let (log, xml) = convert_repro!("class_title/title_replay_defines_thetitle_acmart.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    xml.matches("<title>Universal Widgets</title>").count(),
    1,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p1.1">Body Universal Widgets.</p>"#),
    "{xml}"
  );
}

/// A package's refused `\title` that typesets, inside `{titlepage}`: it runs once on the cover, not again as the kernel
/// part's replay (int26 review p10): its counter counts 1, as in pdflatex.
#[test]
fn titlepage_class_title_runs_once() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/title_in_titlepage_runs_once.tex"
    ),
    &[(
      "tp.sty",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_title/tp.sty"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(xml.contains("Count is 1."), "{xml}");
  assert_eq!(xml.matches("Cover Title").count(), 2, "{xml}"); // the cover print and the document's ltx:title
}

/// physics `\exp[…]` whose optional argument holds a matrix `&`, inside an `align`: read align-safe as ltcmd's `o`
/// argument, so both rows stand and the text after the display is kept (2606.03447; was 11 Errors, Perl the same).
#[test]
fn physics_optional_argument_amp_in_align_keeps_the_rows() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_optional_argument_amp_in_align.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(
    xml
      .matches(r#"<equationgroup class="ltx_eqn_align""#)
      .count(),
    1,
    "{xml}"
  );
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
  assert!(xml.contains("<XMArray"), "{xml}");
  assert!(xml.contains("TAILTEXT survives."), "{xml}");
}

/// An abstract whose first line is a heading (scrartcl `\begin{abstract}\section*{Abstract:} …`, 1507.08575): the
/// heading closes it before any text, the text follows with the heading in the body, and the body that typesets
/// nothing adds no `ltx:abstract` (no empty element; was three sections nested in the abstract, 2 Errors).
#[test]
fn abstract_opening_with_a_section_leaves_no_empty_abstract() {
  let (log, xml) = convert_repro!("abstract_opening_with_a_section_leaves_no_empty_abstract.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<abstract"), "{xml}");
  assert!(xml.contains(r#"<title>Abstract:</title>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="Sx1.p1.1">Structured abstract text.</p>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<title>Availability:</title>"#), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="Sx2.p1.1">Code is online.</p>"#),
    "{xml}"
  );
}

/// A begin code called as a command has no extent: mnras's `\keywords{…}` (a `\list` no `\endkeywords` closes) ends
/// where the paper's first section begins, and the sections are the document's (was the whole document inside
/// `<ltx:keywords>`, 3 Errors; 1004.2791, 2201.07008, 2309.07195).
#[test]
fn keywords_command_form_ends_at_first_section() {
  let (log, xml) = convert_repro!("keywords_command_form_ends_at_first_section.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<keywords name="Keywords: ">planets and satellites: interiors </keywords>"#),
    "{xml}"
  );
  let after = xml.split("</keywords>").nth(1).unwrap_or_default();
  for text in ["<section", "Body text here.", "More text."] {
    assert!(after.contains(text), "{text}: {xml}");
  }
}

/// ieeeconf's `\keywords` inside `{abstract}` is a label: it ends with the abstract, and the section after it is the
/// document's (was the abstract kept open past its `\end{abstract}`, the whole document inside `<ltx:keywords>`, 3
/// Errors; 1401.5657).
#[test]
fn keywords_label_inside_abstract_ends_with_it() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/keywords_label_inside_abstract_ends_with_it.tex"
    ),
    &[(
      "ieeeconf.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_keywords/ieeeconf.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abstract text.</p> </abstract>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<keywords name="Index Terms: ">alpha, beta </keywords>"#),
    "{xml}"
  );
  let after = xml.split("</keywords>").nth(1).unwrap_or_default();
  for text in ["<section", "Body text here."] {
    assert!(after.contains(text), "{text}: {xml}");
  }
}

/// deepseek.cls prints its `\correspondingauthor` toks only in the title page's footer (`firststyle`, which its
/// `\maketitle` selects); footers are not set, but the note is frontmatter and is kept (deepseek_cls.rs; 2401.14196).
#[test]
fn deepseek_corresponding_author_note_is_kept() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/deepseek_corresponding_author_footer_note.tex"
    ),
    &[(
      "deepseek.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/deepseek.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<creator role="author"> <personname>Ann Author</personname> </creator> <note role="corresponding" xml:id="id1">*Core contributors, ordered alphabetically.</note>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<p xml:id="p1.1">Body text.</p>"#), "{xml}");
}

/// mdframed's `frametitle=` (from the env's options, a `\newmdenv` preset or `\mdfsetup`) is the frame's first
/// paragraph, in `\normalfont\mdf@frametitlefont`; a nested frame inherits the outer title, as in pdflatex
/// (mdframed_sty.rs; 2401.03158).
#[test]
fn mdframed_frametitle_is_kept() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/mdframed_frametitle_is_kept.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("Warning:missing_file:mdframed.sty"), "{log}");
  let xml = squash(&xml);
  for element in [
    r#"<p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Outer Title</text></p> <p xml:id="p1.2">Outer body.</p>"#,
    r#"<block framed="rectangle" xml:id="p1.3"> <p xml:id="p1.3.1"><text font="bold" xml:id="p1.3.1.1">Outer Title</text></p> <p xml:id="p1.3.2">Inner body.</p> </block>"#,
    r#"<p xml:id="p2.1"><text font="bold" xml:id="p2.1.1">Preset Title</text></p> <p xml:id="p2.2">Boxed body.</p>"#,
    r#"<p xml:id="p3.1"><text font="italic" xml:id="p3.1.1">Styled <Math mode="inline" tex="x^{2}""#,
    r#"</Math> Title</text></p> <p xml:id="p3.2">Third body.</p>"#,
  ] {
    assert!(xml.contains(element), "{element}: {xml}");
  }
}

/// An empty `{titlepage}` (a blank cover) before `\maketitle` no longer hides the document title in the HTML: only a
/// titlepage with ink stands for the title (LaTeXML-structure-xhtml.xsl `ltx:title`; OXIDIZED_DESIGN #761).
#[test]
fn empty_titlepage_keeps_the_title() {
  let (log, xml, (html, post_log)) = latexml::util::test::convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/empty_titlepage_keeps_the_title.tex"
    ),
    &[],
    Some("ar5iv.sty"),
    |xml, source_dir| {
      super::perfect_kernel_batch64zf::post_logged(xml, source_dir.to_str().expect("a UTF-8 path"))
    },
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(error_count(&post_log), 0, "{post_log}");
  assert!(
    squash(&xml).contains(r#"<titlepage xml:id="id1"/> <title>The Real Title</title>"#),
    "{xml}"
  );
  assert!(
    html.contains(r#"<h1 class="ltx_title ltx_title_document">The Real Title</h1>"#),
    "{html}"
  );
}

/// AASTeX's `\figcaption` outside a figure is that figure's numbered caption, its `\label` the figure's
/// (aas_support_sty.rs `\@figcaption`; aastex701.cls:8111-8114; astro-ph9806172). Was: the bare text in the figure.
#[test]
fn aastex_figcaption_outside_a_figure_is_its_caption() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/aastex_figcaption_outside_a_figure_is_its_caption.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<figure inlist="lof" labels="LABEL:f1" xml:id="S0.F1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=": ">Figure 1</tag>The first figure’s caption.</caption> </figure>"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p xml:id="p2.1">See Figure <ref labelref="LABEL:f1"/>.</p>"#),
    "{xml}"
  );
}

/// A glossary entry used only in display math is listed: its location-only reference goes in a `ltx:text` of the
/// equation after the formula (glossaries_sty.rs `\lx@glossaries@glsadd`; silent-loss inventory #20).
#[test]
fn gls_only_in_display_math_is_listed() {
  let (log, xml, (html, post_log)) = latexml::util::test::convert_bytes_files_then(
    include_bytes!(
      "../../../tools/perfect_kernel/repros/index-bib/gls_only_in_display_math_is_listed.tex"
    ),
    &[],
    Some("ar5iv.sty"),
    |xml, source_dir| {
      super::perfect_kernel_batch64zf::post_logged(xml, source_dir.to_str().expect("a UTF-8 path"))
    },
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(error_count(&post_log), 0, "{post_log}");
  assert!(
    squash(&xml).contains(r#"<text font="italic" xml:id="S0.E1.1"><glossaryref inlist="main" key="energy" show="none"/></text> </equation>"#),
    "{xml}"
  );
  for description in ["the energy of the system", "the mass"] {
    assert!(html.contains(description), "{description}: {html}");
  }
}

/// asmejour's `\keywords`, stored for the abstract's coffin the locked `{abstract}` never builds, are the document's
/// keywords under the class's label; the class's own date footnote keeps `\date` (asmejour_cls.rs; 2510.10694).
#[test]
fn asmejour_keywords_are_kept() {
  if !latexml::util::test::kpse_has("asmejour.cls") {
    eprintln!("skipped: asmejour.cls is not installed");
    return;
  }
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/asmejour_keywords_kept.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<keywords name="Keywords: ">alpha, beta</keywords>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"role="footnotetext""#) && xml.contains("Version 2, 1 May 2025</note>"),
    "{xml}"
  );
  assert!(
    xml.contains("<p xml:id=\"abstract1.1\">Abstract text.</p>"),
    "{xml}"
  );
}

/// an.cls's `\abstract` is its `\aaabstract` (an.cls:98, :414), a store only its title page reads: the abstract, its
/// text once, where it was a plain paragraph of the deposited title block (an_cls.rs; astro-ph0611016). Clean, as
/// pdflatex beside the trimmed class.
#[test]
fn an_abstract_store_is_the_abstract() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an_abstract_store_is_the_abstract.tex"
    ),
    &[(
      "an.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">We consider the contribution of microlensing.</p> </abstract>"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("We consider the contribution").count(),
    1,
    "{xml}"
  );
  assert!(xml.contains("lensing – quasars"), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="S1.p1.1">Body text.</p>"#),
    "{xml}"
  );
}

/// an.cls v1.03's title page prints the class's label before its abstract store (an.cls:662 `\noindent{\bfseries
/// \abstractname}\ \@abstract`): the label is the abstract's name and the deposited title block keeps no lone bold
/// "Abstract" paragraph (an_cls.rs, sect05.rs `note_title_page_part`; astro-ph0306340). Clean, as pdflatex beside the
/// trimmed class.
#[test]
fn an_v103_abstract_label_is_its_name() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an_v103/an_v103_abstract_label_is_its_name.tex"
    ),
    &[(
      "an.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an_v103/an.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">We discuss a new member candidate.</p> </abstract>"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("We discuss a new member candidate.").count(),
    1,
    "{xml}"
  );
  assert!(!xml.contains(">Abstract</text>"), "{xml}");
  assert!(
    xml.contains(r#"<p xml:id="S1.p1.1">Body text.</p>"#),
    "{xml}"
  );
}

/// an.cls's `{abstract}` environment calls the same store (`\begin{abstract}` → `\abstract` = `\aaabstract`): the
/// abstract once, and the deposited title block keeps the class's keywords line, which the deposit dropped while
/// `\@abstract` was undefined there (an_cls.rs; reviewer probe anenv, which keeps the © line too). Clean, as pdflatex
/// beside the trimmed class.
#[test]
fn an_abstract_environment_keeps_the_title_block() {
  let (log, xml) = convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an_abstract_environment_keeps_the_title_block.tex"
    ),
    &[(
      "an.cls",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/an.cls"
      ),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">We consider the contribution of microlensing.</p> </abstract>"#),
    "{xml}"
  );
  assert_eq!(
    xml.matches("We consider the contribution").count(),
    1,
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text font="bold" xml:id="p1.1.1">Key words</text> lensing – quasars"#),
    "{xml}"
  );
}

/// A paper asking for an.cls without shipping it: OmniBus, with the loader's missing-binding Warning, which the
/// stand-in binding's presence had kept back (`omnibus_without_copy`; pdflatex stops on the missing file).
#[test]
fn an_without_its_class_warns() {
  let (log, xml) = convert_with(
    "\\documentclass{an}\n\\begin{document}\n\\title{A Title}\n\\author{A. Author}\n\\maketitle\nBody text.\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log
      .lines()
      .any(|l| l == "Warning:missing_file:an Can't find binding for class an (using OmniBus)"),
    "{log}"
  );
  assert!(squash(&xml).contains("<title>A Title</title>"), "{xml}");
}

/// apa.cls's `\acknowledgements` stores for the doc and jou title pages, which the refused `\maketitle` never set (man
/// mode: a closing section): in every mode the paper's acknowledgements under "Author Note", its text once, beside the
/// title page's `\note` (apa6_cls.rs `define_notes`). The one warning is the class's, as pdflatex's.
#[test]
fn apa_acknowledgements_are_kept_in_every_mode() {
  if !latexml::util::test::kpse_has("apa.cls") {
    eprintln!("skipped: apa.cls is not installed");
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/apa_acknowledgements_kept.tex"
  );
  assert!(
    tex.contains("[doc]{apa}"),
    "the mode replacements read the repro's class line"
  );
  for mode in ["doc", "jou", "man"] {
    let (log, xml) = convert_with(
      &tex.replace("[doc]{apa}", &format!("[{mode}]{{apa}}")),
      Some("ar5iv.sty"),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 1),
      "{mode}: {log}"
    );
    assert!(
      log.lines().any(|l| l.starts_with(
        "Warning:latex:(apa)  Class apa Warning: Using BibTeX with apacite for citations and references"
      )),
      "{mode}: {log}"
    );
    let xml = squash(&xml);
    assert!(
      xml.contains(r#"<acknowledgements inlist="toc" name="Author Note" xml:id="acknowledgements1">We thank the funding agency.</acknowledgements> <note role="note" xml:id="id1">Draft note for reviewers.</note>"#),
      "{mode}: {xml}"
    );
    assert_eq!(
      xml.matches("We thank the funding agency.").count(),
      1,
      "{mode}: {xml}"
    );
    assert!(!xml.contains("<title>Author Note</title>"), "{mode}: {xml}");
  }
}

/// cclayout.sty's `\Abstract` and `{subject}` store for cc.cls's title page, which is not set: the abstract and the
/// subject classification under the package's labels, each text once, beside K11's `{keywords}` (cclayout_sty.rs). The
/// three warnings are cc's two and a "has changed" one, as pdflatex's three.
#[test]
fn cclayout_abstract_and_subject_are_kept() {
  if !latexml::util::test::kpse_has("cc.cls") {
    eprintln!("skipped: cc.cls is not installed");
    return;
  }
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_notes/cclayout_abstract_and_subject_kept.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  for (prefix, warning) in [
    ("Warning:latex:(cc)", "Who is the contact author?"),
    ("Warning:latex:(cc)", "Missing submission/received date."),
    ("Warning:latex:(ccalgo)", "Command \\@item has changed."),
  ] {
    assert!(
      log
        .lines()
        .any(|l| l.starts_with(prefix) && l.contains(warning)),
      "{warning}: {log}"
    );
  }
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">The abstract text here.</p> </abstract> <keywords name="Keywords: ">lower bounds, circuits</keywords> <classification name="Subject classification">68Q15, 68Q17</classification>"#),
    "{xml}"
  );
  for text in [
    "The abstract text here.",
    "lower bounds, circuits",
    "68Q15, 68Q17",
  ] {
    assert_eq!(xml.matches(text).count(), 1, "{text}: {xml}");
  }
  assert!(
    xml.contains(r#"<p xml:id="S1.p1.1">Body text.</p>"#),
    "{xml}"
  );
}
