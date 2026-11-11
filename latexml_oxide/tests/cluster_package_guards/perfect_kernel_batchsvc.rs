//! SVC: the severities of a `\chapter` call around the Error boundary, pinned (OXIDIZED_DESIGN #734, #735). A call in a
//! class without chapters is LaTeXML's numbered chapter with one Warning, though pdflatex errs there (user 2026-11-11:
//! no author-facing value in an Error; the recovery is semantic, reach tiny — SVC's Error was withdrawn). One table
//! covers both sides (the user's 2026-11-11 rule): the shapes where pdflatex errs (a Warning, never silent) and the same
//! construct where pdflatex is clean (0 errors 0 warnings, or only a binding's own Warnings). Each row names its oracle
//! (pdflatex `!` lines, Perl 0.8.8 errors on this host), so the table records where TeX errs.
use super::perfect_kernel_batch46::{convert_files_with, error_count, warning_count};

/// One boundary row: name, oracle, document, the files on disk beside it (name, content), (errors, warnings), the
/// diagnostic lines that make the count, and the whole elements (tag, `xml:id`, element).
type Row = (
  &'static str,
  &'static str,
  &'static str,
  &'static [(&'static str, &'static str)],
  (usize, usize),
  &'static [&'static str],
  &'static [(&'static str, &'static str, &'static str)],
);

const CHAPTERLESS: &str = "Warning:undefined:\\chapter The class defines no \\chapter, so TeX stops here with an undefined \
                           control sequence; LaTeXML's \\chapter is used";

const CH1: &str = r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Text.</p></para></chapter>"#;

const PREFACE: &str = r#"<chapter xml:id="Chx1"><title>Preface</title><para xml:id="Chx1.p1"><p xml:id="Chx1.p1.1">Text.</p></para></chapter>"#;

const BOUNDARY: &[Row] = &[
  // --- TeX errs (and Perl, where named): one Warning (user 2026-11-11) ---
  (
    "article \\chapter",
    "pdflatex 1 `!` (Undefined control sequence); Perl 1 error (undefined:\\thechapter, KNOWN_PERL_ERRORS #962)",
    "\\documentclass{article}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 1),
    &[CHAPTERLESS],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "article \\chapter* (repro sectioning-frontmatter/chapter_star_in_a_chapterless_class)",
    "pdflatex 1 `!` (Undefined control sequence); Perl 0 errors (its \\chapter* reads no \\thechapter)",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_star_in_a_chapterless_class.tex"
    ),
    &[],
    (0, 1),
    &[CHAPTERLESS],
    &[("chapter", "Chx1", PREFACE)],
  ),
  (
    "jmlr article \\chapter (OD #735)",
    "pdflatex 1 `!` (Class jmlr Error: Chapters not permitted in articles); Perl 1 error (undefined:\\pt@size, no jmlr \
     binding)",
    "\\documentclass{jmlr}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 1),
    &[
      "Warning:latex:\\chapter Class jmlr Error: Chapters not permitted in articles; LaTeXML's \\chapter is used",
    ],
    &[(
      "chapter",
      "Ch1",
      r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="autoref">chapter 1<text xml:id="Ch1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Text.</p></para></chapter>"#,
    )],
  ),
  // --- Safe side: pdflatex clean, 0 errors ---
  (
    "report \\chapter",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{report}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "book \\chapter",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{book}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "report \\chapter*",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{report}\n\\begin{document}\n\\chapter*{Preface}\nText.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[("chapter", "Chx1", PREFACE)],
  ),
  (
    "amsbook \\chapter",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{amsbook}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "scrbook \\chapter (a binding that provides \\chapter; memoir's is RED for its own Warning, repro \
     sectioning-frontmatter/memoir_first_load_warns_misdefined_iffirstamp_red)",
    "pdflatex 0 `!`; Perl 0 errors (10 warnings, no scrbook binding)",
    "\\documentclass{scrbook}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "article with the document's own \\newcommand\\chapter",
    "pdflatex 0 `!`, \"Intro\" in bold; Perl 1 error (its locked kernel \\chapter refuses the document's, \
     undefined:\\thechapter)",
    "\\documentclass{article}\n\\newcommand\\chapter[1]{\\par\\noindent\\textbf{#1}\\par}\n\\begin{document}\n\
     \\chapter{Intro}\nText.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[(
      "para",
      "p1",
      r#"<para class="ltx_noindent" xml:id="p1"><p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Intro</text></p></para>"#,
    )],
  ),
  (
    "a shipped raw class over report, answered by the prefix alternate `mn` (repro \
     sectioning-frontmatter/chapter_in_a_class_a_prefix_binding_answers_warns; 0909.2812's shape)",
    "pdflatex 0 `!`, \"Chapter 1 Intro\"; Perl 0 errors (it loads the shipped class)",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_in_a_class_a_prefix_binding_answers_warns.tex"
    ),
    // On disk, as the witness ships them (and as a pdflatex run of the repro leaves them): its filecontents* then
    // writes nothing. From filecontents* alone, the dependency scan reads the cached lxphd.cls and loads report, so
    // `\chapter` is defined and only the missing_file Warning remains (a separate asymmetry, SVCb report).
    &[
      (
        "lxphd.cls",
        "\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{lxphd}\n\\LoadClass{report}\n",
      ),
      (
        "mn_lxthesis.cls",
        "\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{mn_lxthesis}\n\\LoadClass{lxphd}\n",
      ),
    ],
    (0, 2),
    &[
      "Warning:missing_file:mn_lxthesis Can't find binding for class mn_lxthesis (using mn)",
      "Warning:undefined:\\chapter The binding answering class mn_lxthesis defines no \\chapter; LaTeXML's \\chapter is used",
    ],
    &[("chapter", "Ch1", CH1)],
  ),
];

/// The `\chapter` Error boundary, both sides (rows above, each with its oracle). Where pdflatex errs: article `\chapter`
/// and `\chapter*` are one `Warning:undefined:\chapter`, jmlr's article `\chapter` its class error as one Warning; each
/// keeps LaTeXML's chapter (SVC's Error withdrawn, user 2026-11-11). Safe side: report, book, report `\chapter*`,
/// amsbook, scrbook and a document's own `\chapter` under article convert with 0 errors and 0 warnings; 0909.2812's
/// shape with no Error, only the binding's two Warnings (SVC: an Error, which pdflatex contradicts).
#[test]
fn chapter_error_boundary() {
  for (name, oracle, tex, files, (errors, warnings), messages, elements) in BOUNDARY {
    let (stderr, xml) = convert_files_with(tex, files, Some("ar5iv.sty"));
    assert_eq!(
      error_count(&stderr),
      *errors,
      "{name} ({oracle}):\n{stderr}"
    );
    assert_eq!(
      warning_count(&stderr),
      *warnings,
      "{name} ({oracle}):\n{stderr}"
    );
    for message in *messages {
      assert!(
        stderr.lines().any(|line| line == *message),
        "{name}: {message}:\n{stderr}"
      );
    }
    for (tag, id, element) in *elements {
      latexml::util::test::assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
    }
  }
}
