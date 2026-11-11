//! SVC: a `\chapter` call in a class without chapters is an Error, as TeX and Perl raise (user ruling 2026-11-11:
//! "\chapter follows TeX and Perl agreement"; OXIDIZED_DESIGN #734, #735), and the recovery — LaTeXML's numbered
//! chapter — is unchanged. One table guards both sides of the Error boundary (the user's 2026-11-11 rule): the Error
//! side, where pdflatex errs, and the safe side, the same construct where pdflatex is clean, at 0 errors 0 warnings.
//! Each row names its oracle (pdflatex `!` lines, Perl 0.8.8 errors on this host).
use super::perfect_kernel_batch46::{convert_files_with, error_count, warning_count};

/// One boundary row: name, oracle, document, (errors, warnings), the diagnostic lines that make the count, and the
/// whole elements (tag, `xml:id`, element).
type Row = (
  &'static str,
  &'static str,
  &'static str,
  (usize, usize),
  &'static [&'static str],
  &'static [(&'static str, &'static str, &'static str)],
);

const CHAPTERLESS: &str = "Error:undefined:\\chapter The class defines no \\chapter, so TeX stops here with an undefined \
                           control sequence; LaTeXML's \\chapter is used";

const CH1: &str = r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Text.</p></para></chapter>"#;

const PREFACE: &str = r#"<chapter xml:id="Chx1"><title>Preface</title><para xml:id="Chx1.p1"><p xml:id="Chx1.p1.1">Text.</p></para></chapter>"#;

const BOUNDARY: &[Row] = &[
  // --- Error side: TeX errs (and Perl, where named) ---
  (
    "article \\chapter",
    "pdflatex 1 `!` (Undefined control sequence); Perl 1 error (undefined:\\thechapter, KNOWN_PERL_ERRORS #962)",
    "\\documentclass{article}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    (1, 0),
    &[CHAPTERLESS],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "article \\chapter* (repro sectioning-frontmatter/chapter_star_in_a_chapterless_class)",
    "pdflatex 1 `!` (Undefined control sequence); Perl 0 errors (its \\chapter* reads no \\thechapter)",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_star_in_a_chapterless_class.tex"
    ),
    (1, 0),
    &[CHAPTERLESS],
    &[("chapter", "Chx1", PREFACE)],
  ),
  (
    "jmlr article \\chapter (OD #735)",
    "pdflatex 1 `!` (Class jmlr Error: Chapters not permitted in articles); Perl 1 error (undefined:\\pt@size, no jmlr \
     binding)",
    "\\documentclass{jmlr}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    (1, 0),
    &[
      "Error:latex:\\chapter Class jmlr Error: Chapters not permitted in articles; LaTeXML's \\chapter is used",
    ],
    &[(
      "chapter",
      "Ch1",
      r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="autoref">chapter 1<text xml:id="Ch1.1"/></tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Text.</p></para></chapter>"#,
    )],
  ),
  // --- Safe side: pdflatex clean, 0 errors 0 warnings ---
  (
    "report \\chapter",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{report}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "book \\chapter",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{book}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "report \\chapter*",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{report}\n\\begin{document}\n\\chapter*{Preface}\nText.\n\\end{document}\n",
    (0, 0),
    &[],
    &[("chapter", "Chx1", PREFACE)],
  ),
  (
    "amsbook \\chapter",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{amsbook}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
    (0, 0),
    &[],
    &[("chapter", "Ch1", CH1)],
  ),
  (
    "scrbook \\chapter (a binding that provides \\chapter; memoir's is RED for its own Warning, repro \
     sectioning-frontmatter/memoir_first_load_warns_misdefined_iffirstamp_red)",
    "pdflatex 0 `!`; Perl 0 errors (10 warnings, no scrbook binding)",
    "\\documentclass{scrbook}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n",
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
    (0, 0),
    &[],
    &[(
      "para",
      "p1",
      r#"<para class="ltx_noindent" xml:id="p1"><p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Intro</text></p></para>"#,
    )],
  ),
];

/// The `\chapter` Error boundary, both sides (rows above, each with its oracle). Error side: article `\chapter` and
/// `\chapter*` are `Error:undefined:\chapter` (int160a1: one Warning), jmlr's article `\chapter` its class error as an
/// Error (int160a1: a Warning); each keeps LaTeXML's chapter. Safe side: report, book, report `\chapter*`, amsbook,
/// scrbook and a document's own `\chapter` under article convert with 0 errors and 0 warnings.
#[test]
fn chapter_error_boundary() {
  for (name, oracle, tex, (errors, warnings), messages, elements) in BOUNDARY {
    let (stderr, xml) = convert_files_with(tex, &[], Some("ar5iv.sty"));
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
