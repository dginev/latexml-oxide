//! Red/green guards for batch 72b: `\chapter` exists only where the class has chapters (OXIDIZED_DESIGN #179 (a)-(c)),
//! and a document that calls it in a class without chapters gets LaTeXML's chapter with one Warning
//! (OXIDIZED_DESIGN #734; Perl: KNOWN_PERL_ERRORS #962).
//!
//! Every expectation is pdflatex's (the repro headers and the doc comments name the oracle run).
use super::{
  perfect_kernel_batch46::{convert_files_with, error_count, warning_count},
  perfect_kernel_batch57::{assert_elements, assert_elements_with},
};

/// The arXiv configuration's preload: a paper's own `.sty`/`.cls` is read raw.
const AR5IV: &str = "ar5iv.sty";

/// A raw package testing `\chapter` while it loads and at `\begin{document}` (bmvc2k_natbib.sty:779,
/// NJDnatbib.sty:1050).
const PROBE_STY: &str = r"\ProvidesPackage{lxchapprobe}
\@ifundefined{chapter}{\def\x{S}}{\def\x{C}}
\AtBeginDocument{\@ifundefined{chapter}{\gdef\y{S}}{\gdef\y{C}}}
";

/// The probe's paragraph under `\documentclass{<class>}`, with `files` beside the document; 0 errors and `warnings`
/// warnings.
fn probe_under(class: &str, files: &[(&str, &str)], warnings: usize) -> String {
  let tex = format!(
    "\\documentclass{{{class}}}\n\\usepackage{{lxchapprobe}}\n\\begin{{document}}\nX=\\x. Y=\\y.\n\\end{{document}}\n"
  );
  let mut all = vec![("lxchapprobe.sty", PROBE_STY)];
  all.extend_from_slice(files);
  let (stderr, xml) = convert_files_with(&tex, &all, Some(AR5IV));
  assert_eq!(error_count(&stderr), 0, "{class}: {stderr}");
  assert_eq!(warning_count(&stderr), warnings, "{class}: {stderr}");
  xml
}

fn assert_probe(xml: &str, answer: &str) {
  latexml::util::test::assert_element(
    xml,
    "p",
    &["xml:id=\"p1.1\""],
    &format!("<p xml:id=\"p1.1\">{answer}</p>"),
  );
}

/// OXIDIZED_DESIGN #179: a raw package's `\@ifundefined{chapter}`, at load and in `\AtBeginDocument`, under article,
/// under a named binding built on OmniBus (bmvc2k, WileyNJDv5, MRM: (c) is the fallback for an unknown class only) and
/// under report — pdflatex S / S / S / S / C (bmvc2k.cls of 2412.14509, WileyNJDv5.cls of 2602.03829, MRM.cls of
/// 2506.14995, whose `\newcounter{chapter}` is not a `\chapter`). 67int21 answered C under the three bindings, so the
/// natbib forks titled the bibliography `\chapter*{\bibname}` (2412.14509, 2212.00057, 2210.06270, 1501.07873,
/// 2311.17607, 2406.08486; 2101.06297, 2211.16474, 2411.11493, 2206.08798, 2602.03829, 2408.07575, 2609.08425,
/// 2506.14995). An unknown class keeps the kernel's chapter (C, its one Warning the missing binding).
#[test]
fn chapter_exists_where_the_class_has_chapters() {
  for class in ["article", "bmvc2k", "WileyNJDv5", "MRM"] {
    assert_probe(&probe_under(class, &[], 0), "X=S. Y=S.");
  }
  assert_probe(&probe_under("report", &[], 0), "X=C. Y=C.");
  assert_probe(&probe_under("lxunknownclass", &[], 1), "X=C. Y=C.");
  // repro sectioning-frontmatter/chapter_undefined_under_an_omnibus_based_binding
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_undefined_under_an_omnibus_based_binding.tex"
    ),
    AR5IV,
    (0, 0),
    &[("p", "p1.1", r#"<p xml:id="p1.1">P=S. B=S.</p>"#)],
  );
}

/// (a) and (b): a stand-in's copy (tlp_cls.rs reads the paper's `tlp.cls` raw) that makes the chapter counter and no
/// `\chapter` has no chapters (S, as WileyNJDv5.cls:5728 `\newcounter{chapter}[part]`); one that defines `\chapter`
/// has (C). A raw class that defines `\chapter` after `\LoadClass{article}`, even with the kernel's own body, or alone
/// (its `\def` refused by the kernel's lock, which keeps LaTeXML's chapter) has chapters (C). A stand-in's copy that
/// defines `\chapter` through its binding's owned name (a synthetic pasj00.cls as 0704.3654's, :478-504: its
/// `\def\chapter` kept aside or declined) has chapters (C); the copy's own `\LoadClass` settles outside the copy's
/// provides-mode, so its removal of `\chapter` is not taken for the copy's definition (tlp, S). pdflatex: S / C / C / C /
/// C.
#[test]
fn a_class_has_chapters_when_its_code_defines_chapter() {
  let copy_counter = "\\ProvidesClass{tlp}\n\\LoadClass{article}\n\\newcounter{chapter}[part]\n";
  assert_probe(
    &probe_under("tlp", &[("tlp.cls", copy_counter)], 0),
    "X=S. Y=S.",
  );
  let copy_chapter = "\\ProvidesClass{tlp}\n\\LoadClass{article}\n\\newcounter{chapter}\n\\newcommand\\chapter{\\section}\n";
  assert_probe(
    &probe_under("tlp", &[("tlp.cls", copy_chapter)], 0),
    "X=C. Y=C.",
  );
  let raw_on_article = "\\ProvidesClass{lxchapclass}\n\\LoadClass{article}\n\\newcounter{chapter}\n\
                        \\newcommand\\chapter{\\@startsection{chapter}{0}{}{}{}{}}\n";
  assert_probe(
    &probe_under("lxchapclass", &[("lxchapclass.cls", raw_on_article)], 0),
    "X=C. Y=C.",
  );
  let raw_alone =
    "\\ProvidesClass{lxchapclass}\n\\newcounter{chapter}\n\\def\\chapter{\\par\\noindent CHAP }\n";
  assert_probe(
    &probe_under("lxchapclass", &[("lxchapclass.cls", raw_alone)], 0),
    "X=C. Y=C.",
  );
  let pasj = "\\ProvidesClass{pasj00}\n\\LoadClass{article}\n\\newcounter{chapter}\n\
              \\def\\chapter{\\@startsection{chapter}{0}{\\z@}{1ex}{1ex}{\\bfseries}}\n";
  assert_probe(
    &probe_under("pasj00", &[("pasj00.cls", pasj)], 0),
    "X=C. Y=C.",
  );
}

/// OXIDIZED_DESIGN #734: in an article, `\chapter` stays undefined — in the preamble, in `\AtBeginDocument` and in the
/// body, as pdflatex's "P=S. B=S. D=S." (repro sectioning-frontmatter/chapter_undefined_in_a_chapterless_class) — and
/// a call of it reads LaTeXML's chapter, numbered by a counter of its own, with exactly one Warning per document; the
/// name is still undefined after it (pdflatex: 2 errors, "Still undefined."). 67int21: `Error:undefined:\chapter`
/// (0802.3014, 0909.2812, 0907.3938, 1403.2269, 1507.01872, 1511.03874, 1606.03650, 2504.00704); Perl: an
/// `undefined:\thechapter` Error.
#[test]
fn chapter_in_a_chapterless_class_is_the_kernel_chapter_with_a_warning() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_undefined_in_a_chapterless_class.tex"
    ),
    AR5IV,
    (0, 0),
    &[("p", "p1.1", r#"<p xml:id="p1.1">P=S. B=S. D=S.</p>"#)],
  );
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_call_in_a_chapterless_class.tex"
    ),
    AR5IV,
    (0, 1),
    &[
      "Warning:undefined:\\chapter The class defines no \\chapter, so TeX stops here with an undefined control sequence; LaTeXML's \\chapter is used",
    ],
    &[
      (
        "chapter",
        "Ch2",
        r#"<chapter inlist="toc" xml:id="Ch2"><tags><tag>Chapter 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Chapter 2</tag></tags><title><tag close=" ">Chapter 2</tag>Next</title><toctitle><tag close=" ">2</tag>Next</toctitle><para xml:id="Ch2.p1"><p xml:id="Ch2.p1.1">Still undefined.</p></para></chapter>"#,
      ),
      (
        "chapter",
        "Ch1",
        r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Text.</p></para><section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Sec</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">More.</p></para></section></chapter>"#,
      ),
    ],
  );
}

/// The document's files beside it: `expected` whole elements (tag, `xml:id`, element), exactly `errors` and `warnings`,
/// each of `messages` one of the diagnostic lines.
fn assert_files_elements(
  tex: &str,
  files: &[(&str, &str)],
  (errors, warnings): (usize, usize),
  messages: &[&str],
  expected: &[(&str, &str, &str)],
) {
  let (stderr, xml) = convert_files_with(tex, files, Some(AR5IV));
  assert_eq!(error_count(&stderr), errors, "{stderr}");
  assert_eq!(warning_count(&stderr), warnings, "{stderr}");
  for message in messages {
    assert!(
      stderr.lines().any(|line| line == *message),
      "{message}: {stderr}"
    );
  }
  for (tag, id, element) in expected {
    latexml::util::test::assert_element(&xml, tag, &[&format!("xml:id=\"{id}\"")], element);
  }
}

/// 72b review finding 3 (probe rev72b/p/r1): `\documentclass` clears the kernel's `\chapter` before the class loads,
/// so a raw class's own `\@ifundefined{chapter}` says undefined and its `\providecommand\chapter` defines — pdflatex
/// "CLS=S. DOC=C." (72b r1: "CLS=C."; Perl: C). The kernel's lock refuses the class's definition and puts LaTeXML's
/// chapter in its place, numbered by a counter of its own (the class made none): `<chapter>` "Chapter 1 Hello", where
/// pdflatex sets the class's "PROVIDEDCHAP Hello" — a raw class's chapters stay LaTeXML's, as scrbook's do
/// (`perfect_kernel_batch56::koma_part_is_a_part`). A class answered by a binding not its own words the Warning without
/// claiming TeX errs (72b review finding 1: 0909.2812, mn_thesis bound by the prefix alternate `mn`; here IEEEtranLX).
#[test]
fn a_raw_class_meets_chapter_undefined_while_it_loads() {
  let cls = "\\ProvidesClass{lxrawone}\n\\@ifundefined{chapter}{\\def\\clsx{S}}{\\def\\clsx{C}}\n\
             \\newcommand\\section{\\@startsection{section}{1}{\\z@}{1ex}{1ex}{\\bfseries}}\n\\newcounter{section}\n\
             \\providecommand\\chapter{\\par\\noindent PROVIDEDCHAP }\n\\renewcommand\\normalsize{}\n";
  let tex = "\\documentclass{lxrawone}\n\\makeatletter\n\\@ifundefined{chapter}{\\def\\x{S}}{\\def\\x{C}}\n\
             \\begin{document}\nCLS=\\clsx. DOC=\\x.\n\\chapter{Hello}\n\\end{document}\n";
  assert_files_elements(tex, &[("lxrawone.cls", cls)], (0, 0), &[], &[
    ("p", "p1.1", r#"<p xml:id="p1.1">CLS=S. DOC=C.</p>"#),
    (
      "chapter",
      "Ch1",
      r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Hello</title><toctitle><tag close=" ">1</tag>Hello</toctitle></chapter>"#,
    ),
  ]);
  let tex =
    "\\documentclass{IEEEtranLX}\n\\begin{document}\n\\chapter{Intro}\nText.\n\\end{document}\n";
  assert_files_elements(
    tex,
    &[],
    (0, 2),
    &[
      "Warning:missing_file:IEEEtranLX Can't find binding for class IEEEtranLX (using IEEEtran)",
      "Warning:undefined:\\chapter The binding answering class IEEEtranLX defines no \\chapter; LaTeXML's \\chapter is used",
    ],
    &[(
      "chapter",
      "Ch1",
      r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="Ch1.p1"><p xml:id="Ch1.p1.1">Text.</p></para></chapter>"#,
    )],
  );
}

/// 72b review finding 2 (repro sectioning-frontmatter/chapter_in_the_preamble_errs_and_the_body_chapter_converts): a
/// `\chapter` called in the preamble errs as TeX's undefined control sequence (one Error; pdflatex 3), and its
/// undefined-error stub does not stand at `\begin{document}`, so the body's call reads the stand-in — `<chapter>`
/// "Chapter 1 FromPreambleMacro" with the one Warning (72b r1: the stub's "\chapter FromPreambleMacro" as text).
#[test]
fn a_preamble_chapter_errs_and_the_body_chapter_reads_the_stand_in() {
  assert_elements_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/chapter_in_the_preamble_errs_and_the_body_chapter_converts.tex"
    ),
    AR5IV,
    (1, 1),
    &[
      "Error:undefined:\\chapter The token T_CS[\\chapter] is not defined.",
      "Warning:undefined:\\chapter The class defines no \\chapter, so TeX stops here with an undefined control sequence; LaTeXML's \\chapter is used",
    ],
    &[(
      "chapter",
      "Ch1",
      r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>FromPreambleMacro</title><toctitle><tag close=" ">1</tag>FromPreambleMacro</toctitle></chapter>"#,
    )],
  );
}

/// The one "has chapters" test numbers the appendices (72b review finding 6, 72a review r5 finding 1): after the
/// stand-in's chapter in an article (repro sectioning-frontmatter/appendix_after_a_chapter_in_a_chapterless_class; 72b
/// r1 "Appendix 1.A") and under a raw class whose `\newcounter{chapter}[part]` makes no `\chapter` (as WileyNJDv5.cls:5728,
/// MRM.cls:2160; probes rev72a/r6 retract, retract2: cimart, daj), `\appendix` and appendix.sty's `appendices` letter
/// the sections, "Appendix A" — pdflatex "A AppSec", "CHAP:UNDEF." and "A Extra".
#[test]
fn appendices_follow_the_class_chapters() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/appendix_after_a_chapter_in_a_chapterless_class.tex"
    ),
    AR5IV,
    (0, 1),
    &[(
      "appendix",
      "A1",
      r#"<appendix inlist="toc" xml:id="A1"><tags><tag>Appendix A</tag><tag role="refnum">A</tag><tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix A</tag>AppSec</title><toctitle><tag close=" ">A</tag>AppSec</toctitle></appendix>"#,
    )],
  );
  let cls = "\\ProvidesClass{lxctr}\n\\LoadClass{article}\n\\newcounter{chapter}[part]\n";
  let tex = "\\documentclass{lxctr}\n\\usepackage{appendix}\n\\makeatletter\n\\begin{document}\n\
             CHAP:\\@ifundefined{chapter}{UNDEF}{DEF}.\n\\section{One}\nText.\n\\begin{appendices}\n\\section{Extra}\n\
             More.\n\\end{appendices}\n\\end{document}\n";
  assert_files_elements(tex, &[("lxctr.cls", cls)], (0, 0), &[], &[
    ("p", "p1.1", r#"<p xml:id="p1.1">CHAP:UNDEF.</p>"#),
    (
      "appendix",
      "A1",
      r#"<appendix inlist="toc" xml:id="A1"><tags><tag>Appendix A</tag><tag role="refnum">A</tag><tag role="typerefnum">Appendix A</tag></tags><title><tag close=" ">Appendix A</tag>Extra</title><toctitle><tag close=" ">A</tag>Extra</toctitle><para xml:id="A1.p1"><p xml:id="A1.p1.1">More.</p></para></appendix>"#,
    ),
  ]);
}

/// Bindings whose real class has chapters define them (72b review findings 4, 7): jmlr (jmlr.cls:743-745
/// `\let\chapter\artchapter`, a ClassError; pdflatex "X=C." and 1 error) — a call is LaTeXML's chapter with jmlr's
/// error text as the one Warning; kluwer under its `chapter` option (kluwer.cls:135-136, :2464-2508; pdflatex "X=C.",
/// "1. Intro", "1.1. Sec"; without it "X=S."); ws-rv9x6 with its chapter counter (ws-rv9x6.cls:397-399; 1807.01378's
/// `\setcounter{chapter}` was "Counter 'chapter' was not defined"), "Chapter 3".
#[test]
fn bindings_whose_class_has_chapters_define_them() {
  let probe = |class: &str, body: &str| {
    format!(
      "\\documentclass{class}\n\\makeatletter\n\\@ifundefined{{chapter}}{{\\def\\x{{S}}}}{{\\def\\x{{C}}}}\n\
       \\begin{{document}}\nX=\\x.\n{body}\\end{{document}}\n"
    )
  };
  assert_files_elements(
    &probe("{jmlr}", "\\chapter{Intro}\nText.\n"),
    &[],
    (0, 1),
    &[
      "Warning:latex:\\chapter Class jmlr Error: Chapters not permitted in articles; LaTeXML's \\chapter is used",
    ],
    &[("p", "p1.1", r#"<p xml:id="p1.1">X=C.</p>"#)],
  );
  assert_files_elements(
    &probe(
      "[chapter]{kluwer}",
      "\\chapter{Intro}\n\\section{Sec}\nText.\n",
    ),
    &[],
    (0, 0),
    &[],
    &[
      ("p", "p1.1", r#"<p xml:id="p1.1">X=C.</p>"#),
      (
        "chapter",
        "Ch1",
        r#"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><section inlist="toc" xml:id="S1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title><tag close=" ">1.1</tag>Sec</title><para xml:id="S1.p1"><p xml:id="S1.p1.1">Text.</p></para></section></chapter>"#,
      ),
    ],
  );
  assert_files_elements(&probe("{kluwer}", ""), &[], (0, 0), &[], &[(
    "p",
    "p1.1",
    r#"<p xml:id="p1.1">X=S.</p>"#,
  )]);
  assert_files_elements(
    &probe(
      "{ws-rv9x6}",
      "\\setcounter{chapter}{2}\n\\chapter{Intro}\nText.\n",
    ),
    &[],
    (0, 0),
    &[],
    &[("p", "p1.1", r#"<p xml:id="p1.1">X=C.</p>"#)],
  );
}
