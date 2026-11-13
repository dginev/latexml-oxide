//! The Error boundary, both sides, in one table (EB1; the user 2026-11-11: "red/green TDD on minimal tests that
//! captured desired Error raising cases as well as safe cases"). A row is a construct where TeX or Perl errs, or its
//! safe twin where pdflatex is clean; each names its oracle (pdflatex `!` lines, Perl's errors on this host) and pins
//! our exact counts, the diagnostics that make them and whole elements. The staged rule (2026-10-11): where Perl
//! reports, its severity; where only TeX errs, a Warning if the recovery is semantic, else an Error.
use latexml::util::test::{assert_element, convert_files_with, rng_error_count};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One boundary row: name, oracle, document, the files on disk beside it (name, content), (errors, warnings), the
/// first lines of the diagnostics that make the count (each as often as it occurs), and the whole elements (tag,
/// `xml:id` — or an attribute `name="value"`, or "" for the first of the tag —, element).
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

const IFPDF: &str = r"\documentclass{article}
\usepackage[final]{graphicx}
\usepackage{lineno}
\let\ifpdf\relax
\def\lnk#1{\iftrue\ifpdf[pdf #1]\else[dvi #1]\fi\fi}
\begin{document}
See \lnk{x}.
\end{document}";

const IFPDF_SAFE: &str = r"\documentclass{article}
\usepackage[final]{graphicx}

\let\ifpdf\relax
\def\lnk#1{\iftrue\ifpdf[pdf #1]\else[dvi #1]\fi\fi}
\begin{document}
See \lnk{x}.
\end{document}";

const SEE_PDF: &str = r#"<p xml:id="p1.1">See [pdf x].</p>"#;

const REFS: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bibunits_chapterbib/refs.bib");

const CALC_WIDTHOF: &str = "Warning:unexpected:\\widthof pgf's parser errs on calc's \\widthof in an expression; \
                            calc's value is used";

const SBX2_THANKS_BOLD: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"><title>T</title><creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A</contact></creator><creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"><text font="bold" xml:id="id1">Univ B</text></contact></creator><para xml:id="p1"><p xml:id="p1.1">Hi.</p></para></document>"#;

const SBX2_THANKS_GROUP: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"><title>T</title><creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation">Ann Lee <text font="typewriter" fontsize="90%" xml:id="id1">al@itu.dk</text></contact></creator><creator before="  " role="author"><personname>Bob Roe</personname><contact name="Affiliation: " role="affiliation">Bob Roe <text font="typewriter" fontsize="90%" xml:id="id2">br@itu.dk</text> are with the IT-University.</contact></creator><para xml:id="p1"><p xml:id="p1.1">Hi.</p></para></document>"#;

const SBX2_THANKS_TABLE: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"><title>T</title><creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A</contact></creator><creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ B</contact></creator><para xml:id="p1"><p xml:id="p1.1">Hi.</p></para></document>"#;

const SBX2_THANKS_TABLE_MORE: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"><title>T</title><creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A</contact></creator><creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ B more</contact></creator><para xml:id="p1"><p xml:id="p1.1">Hi.</p></para></document>"#;

const SBX2_ANN_UNIV_A: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"><title>T</title><creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation">Univ A</contact></creator><para xml:id="p1"><p xml:id="p1.1">Hi.</p></para></document>"#;

const SBX2_LLNCS: &str = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_authors_1line"><title>T</title><creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation">Amazon</contact></creator><creator before="  " role="author"><personname>Bob Roe</personname><contact name="Affiliation: " role="affiliation">UW</contact></creator><para xml:id="p1"><p xml:id="p1.1">Hi.</p></para></document>"#;

const BOUNDARY: &[Row] = &[
  // --- `\chapter` (SVC, OD #734, #735). TeX errs (and Perl, where named): one Warning (user 2026-11-11) ---
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
  // --- `\chapter`, safe side: pdflatex clean, 0 errors ---
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
  // --- RC1306 (1306.0471, JINST): a document's `\let\ifpdf\relax` holds when no package reloads iftex at
  // `\begin{document}` (lineno loads kvoptions, lineno.sty:264-265, leaving `\@curroptions` empty, so epstopdf-base
  // skips pdftexcmds → iftex; KNOWN_PERL_ERRORS #487); then `\iftrue\ifpdf…\else…\fi\fi` has a stray `\fi` ---
  (
    "a disabled \\ifpdf under lineno",
    "pdflatex 1 `!` (Extra \\fi); Perl 1 error",
    IFPDF,
    &[],
    (1, 0),
    &[],
    &[("p", "p1.1", SEE_PDF)],
  ),
  (
    "a disabled \\ifpdf without lineno (iftex resets it)",
    "pdflatex 0 `!`; Perl 1 error",
    IFPDF_SAFE,
    &[],
    (0, 0),
    &[],
    &[("p", "p1.1", SEE_PDF)],
  ),
  // --- Out of TeX's range (OD #519): TeX errs and holds its bound or 0, not the author's value, so an Error by rule (3);
  // each bound itself reads silently (DTL1d; DTL1's dimension row is the error side of 2010.05701's tikzscale overflow) ---
  (
    "\\dimen0=20000pt (repro graphics-tikz/pgfmath_dimension_too_large_still_reported)",
    "pdflatex 1 `!` (Dimension too large), [16383.99998pt]; Perl 0 errors, [20000.0pt]",
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_dimension_too_large_still_reported.tex"
    ),
    &[],
    (1, 0),
    &["Error:expected:<dimen> Dimension too large."],
    &[("p", "p1.1", r#"<p xml:id="p1.1">[16383.99998pt]</p>"#)],
  ),
  (
    "\\count255=3000000000 (repro expansion-primitives/out_of_range_number_errs)",
    "pdflatex 1 `!` (Number too big), [2147483647]; Perl 0 errors, [3000000000]",
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/out_of_range_number_errs.tex"
    ),
    &[],
    (1, 0),
    &["Error:expected:<number> Number too big."],
    &[("p", "p1.1", r#"<p xml:id="p1.1">A=[2147483647]</p>"#)],
  ),
  (
    "\\numexpr past 2^31-1 (repro expansion-primitives/numexpr_overflow_errs)",
    "pdflatex 1 `!` (Arithmetic overflow), [0]; Perl 0 errors, [2147483648]",
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/numexpr_overflow_errs.tex"
    ),
    &[],
    (1, 0),
    &["Error:expected:<number> Arithmetic overflow."],
    &[("p", "p1.1", r#"<p xml:id="p1.1">A=[0]</p>"#)],
  ),
  (
    "\\dimexpr past \\maxdimen (repro expansion-primitives/dimexpr_overflow_errs)",
    "pdflatex 1 `!` (Arithmetic overflow), [0.0pt]; Perl 0 errors, [32766.0pt]",
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/dimexpr_overflow_errs.tex"
    ),
    &[],
    (1, 0),
    &["Error:expected:<number> Arithmetic overflow."],
    &[("p", "p1.1", r#"<p xml:id="p1.1">A=[0.0pt]</p>"#)],
  ),
  (
    "TeX's bounds as constants and e-TeX results (repro expansion-primitives/out_of_range_bounds_are_clean)",
    "pdflatex 0 `!`; Perl 0 errors",
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/out_of_range_bounds_are_clean.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[(
      "p",
      "p1.1",
      "<p xml:id=\"p1.1\">[2147483647]\n[16383.99998pt]\n[16383.99998pt]\n[2147483647]\n[16383.99997pt]</p>",
    )],
  ),
  (
    "a pgfmath result past 16383.99998 (repro graphics-tikz/pgfmath_overflow_errs)",
    "pdflatex 1 `!` (Dimension too large), [16383.99998pt]; Perl 1 error (pgfmath:overflow), [16383.99998pt]",
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_overflow_errs.tex"),
    &[],
    (1, 0),
    &["Error:pgfmath:overflow Dimension too large: 20000. Input was: \\wd\\mb*200"],
    &[("p", "p1.1", r#"<p xml:id="p1.1">[16383.99998pt]</p>"#)],
  ),
  (
    "pgfmath at its bound (`\\maxdimen` too: tikzpingus), and past it as a plain number (pgf's own number path)",
    "pdflatex 0 `!`, [16383.99998] [-16383.99998] [16382.0] [20000] [16383.99998] [-16383.99998]; Perl 0 errors",
    "\\documentclass{article}\n\\usepackage{pgf}\n\\begin{document}\n\\pgfmathparse{16383.99998}[\\pgfmathresult] \
     \\pgfmathparse{-16383.99998}[\\pgfmathresult] \\pgfmathparse{8191*2}[\\pgfmathresult] \
     \\pgfmathparse{20000}[\\pgfmathresult] \\pgfmathparse{\\maxdimen}[\\pgfmathresult] \
     \\pgfmathparse{-\\maxdimen}[\\pgfmathresult]\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[(
      "p",
      "p1.1",
      r#"<p xml:id="p1.1">[16383.99998] [-16383.99998] [16382.0] [20000] [16383.99998] [-16383.99998]</p>"#,
    )],
  ),
  // --- pgfmath: the overflow test on the dimension TeX holds (2^30−1 sp after rounding), none on the fpu's floats
  // (DTL1h; 1811.10950, 2109.14286, the manuals ddphonism, tikzbrickfigurines-doc) ---
  (
    "pgfmath's `16383.99999pt` is \\maxdimen; the fpu's sums are floats (repro \
     graphics-tikz/pgfmath_maxdimen_spelling_and_fpu_in_range)",
    "pdflatex 0 `!`, [16383.99998] [-16383.99998] [-16383.99998] [16383.99998], [16,761] [18,765]; Perl 14 errors",
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_maxdimen_spelling_and_fpu_in_range.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[
      (
        "p",
        "p1.1",
        "<p xml:id=\"p1.1\">[16383.99998] [-16383.99998]\n[-16383.99998] [16383.99998]</p>",
      ),
      (
        "p",
        "p3.1",
        r#"<p xml:id="p3.1">[<Math mode="inline" tex="16{,}761" text="16761" xml:id="p3.m1"><XMath xml:id="p3.m1.1"><XMTok meaning="16761" role="NUMBER">16,761</XMTok></XMath></Math>] [<Math mode="inline" tex="18{,}765" text="18765" xml:id="p3.m2"><XMath xml:id="p3.m2.1"><XMTok meaning="18765" role="NUMBER">18,765</XMTok></XMath></Math>]</p>"#,
      ),
    ],
  ),
  (
    "pgfmath past \\maxdimen: a literal, a sum, each unit'd operand, the fpu's unit'd operand (repro \
     graphics-tikz/pgfmath_past_maxdimen_errs)",
    "pdflatex 7 `!` (Dimension too large), [16383.99998] [16383.99998] [6383.99998] [16383.99998] [16,384][16,384] \
     [−20,000]; Perl 5 errors",
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_past_maxdimen_errs.tex"
    ),
    &[],
    (7, 0),
    &[
      "Error:pgfmath:overflow Dimension too large: 16383.999993. Input was: 16383.999993pt",
      "Error:pgfmath:overflow Dimension too large: 16761. Input was: 12319.0+4442",
      "Error:pgfmath:overflow Dimension too large: 20000. Input was: 20000pt",
      "Error:pgfmath:overflow Dimension too large: 20000. Input was: 20000pt",
      "Error:pgfmath:overflow Dimension too large: 20000. Input was: 20000pt",
      "Error:pgfmath:overflow Dimension too large: 16384.99998474121. Input was: 20000pt+1",
      "Error:expected:<dimen> Dimension too large.",
    ],
    &[
      (
        "p",
        "p1.1",
        r#"<p xml:id="p1.1">[16383.99998] [16383.99998]</p>"#,
      ),
      (
        "p",
        "p2.1",
        r#"<p xml:id="p2.1">[6383.99998] [16383.99998]</p>"#,
      ),
      (
        "p",
        "p3.1",
        r#"<p xml:id="p3.1">[<Math mode="inline" tex="16{,}384" text="16384" xml:id="p3.m1"><XMath xml:id="p3.m1.1"><XMTok meaning="16384" role="NUMBER">16,384</XMTok></XMath></Math>][<Math mode="inline" tex="16{,}384" text="16384" xml:id="p3.m2"><XMath xml:id="p3.m2.1"><XMTok meaning="16384" role="NUMBER">16,384</XMTok></XMath></Math>] [<Math mode="inline" tex="-20{,}000" text="- 20000" xml:id="p3.m3"><XMath xml:id="p3.m3.1"><XMApp xml:id="p3.m3.1.1"><XMTok meaning="minus" role="ADDOP">-</XMTok><XMTok meaning="20000" role="NUMBER">20,000</XMTok></XMApp></XMath></Math>]</p>"#,
      ),
    ],
  ),
  // --- bibunits' `\putbib` reads `\@startbibunitorrelax`, undefined before the first unit (bibunits.sty:251-256,
  // 324-325): an Error by rule (3), the cite dangling (revBU1b PE1); a unit's `plain` sorts no citation (GA3) ---
  (
    "\\putbib outside every unit (repro index-bib/bibunits_chapterbib/bibunits_putbib_outside_every_unit_errs)",
    "pdflatex 1 `!` (Undefined control sequence), \"Main [?]\"; Perl 0 errors 0 warnings",
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/bibunits_chapterbib/bibunits_putbib_outside_every_unit_errs.tex"
    ),
    &[("refs.bib", REFS)],
    (1, 0),
    &[
      "Error:undefined:\\@startbibunitorrelax bibunits' \\putbib outside every unit: TeX stops on the undefined \
       \\@startbibunitorrelax",
    ],
    &[
      (
        "para",
        "p1",
        r#"<para xml:id="p1"><p xml:id="p1.1">Main <cite class="ltx_citemacro_cite">[<bibref bibrefs="alpha" separator="," yyseparator=","/>]</cite>.</p></para>"#,
      ),
      (
        "para",
        "p3",
        r#"<para xml:id="p3"><p xml:id="p3.1">Two <cite class="ltx_citemacro_cite">[<bibref bibrefs="beta" inlist="bu2" separator="," yyseparator=","/>]</cite>.</p></para>"#,
      ),
    ],
  ),
  (
    "\\putbib in a unit and after it (`\\relax` from the unit's start on)",
    "pdflatex 0 `!`, \"One [?]. After the unit.\"; Perl 0 errors 0 warnings",
    "\\documentclass{article}\n\\usepackage{bibunits}\n\\begin{document}\n\\begin{bibunit}[plain]\nOne \\cite{alpha}.\n\
     \\putbib[refs]\n\\end{bibunit}\nAfter the unit.\n\\putbib[refs]\n\\end{document}\n",
    &[("refs.bib", REFS)],
    (0, 0),
    &[],
    &[
      (
        "para",
        "p1",
        r#"<para xml:id="p1"><p xml:id="p1.1">One <cite class="ltx_citemacro_cite">[<bibref bibrefs="alpha" inlist="bu1" separator="," yyseparator=","/>]</cite>.</p></para>"#,
      ),
      (
        "bibliography",
        "biba",
        r#"<bibliography citestyle="numbers" files="refs" inlist="toc" lists="bu1" xml:id="biba"><title>References</title></bibliography>"#,
      ),
    ],
  ),
  (
    "\\putbib before the first \\section unit, then in it (`\\bibliographyunit[\\section]`, the `\\@bibunit` start)",
    "pdflatex 1 `!` (Undefined control sequence), at the first \\putbib; Perl 0 errors 0 warnings",
    "\\documentclass{article}\n\\usepackage{bibunits}\n\\begin{document}\n\\bibliographyunit[\\section]\nBefore \
     \\cite{alpha}.\n\\putbib[refs]\n\\section{One}\nOne \\cite{beta}.\n\\putbib[refs]\n\\end{document}\n",
    &[("refs.bib", REFS)],
    (1, 0),
    &[
      "Error:undefined:\\@startbibunitorrelax bibunits' \\putbib outside every unit: TeX stops on the undefined \
       \\@startbibunitorrelax",
    ],
    &[(
      "para",
      "S1.p1",
      r#"<para xml:id="S1.p1"><p xml:id="S1.p1.1">One <cite class="ltx_citemacro_cite">[<bibref bibrefs="beta" inlist="bu1" separator="," yyseparator=","/>]</cite>.</p></para>"#,
    )],
  ),
  // --- pgfmath: calc's names, which pgf `\let`s for a parse (pgfmathparser.code.tex:124-130) and TL 2025's pgf errs
  // on: calc's value is the author's, a Warning per parse by rule (2) (revDTL1f S1); a user's `\ratio` is a safe twin
  // in `batchdtl1_pgfmath_rows` (pgfmath_expands_before_calc_lets) ---
  (
    "pgfmath calc names (repro graphics-tikz/pgfmath_calc_names_warn)",
    "pdflatex 7 `!` (each paragraph alone 9, 4, 9); Perl 0 errors 0 warnings, [10.56][21.12] [10.56pt] [6.0][5.0]",
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/pgfmath_calc_names_warn.tex"),
    &[],
    (0, 5),
    &[
      CALC_WIDTHOF,
      CALC_WIDTHOF,
      CALC_WIDTHOF,
      "Warning:unexpected:\\maxof pgf's parser errs on calc's \\maxof in an expression; calc's value is used",
      "Warning:unexpected:\\real pgf's parser errs on calc's \\real in an expression; calc's value is used",
    ],
    &[
      ("p", "p1.1", r#"<p xml:id="p1.1">[21.11121][21.11121]</p>"#),
      ("p", "p2.1", r#"<p xml:id="p2.1">[21.1112pt]</p>"#),
      ("p", "p3.1", r#"<p xml:id="p3.1">[6.0][5.0]</p>"#),
    ],
  ),
  (
    "pgfmath's own width() and max() beside calc's \\widthof in \\setlength",
    "pdflatex 0 `!`, [21.1112][6.0] [21.1112pt]; Perl 0 errors, 1 warning (calc's `\\widthof` in \\setlength)",
    "\\documentclass{article}\n\\usepackage{calc}\n\\usepackage{pgf}\n\\newdimen\\mydim\n\\begin{document}\n\
     \\pgfmathparse{2*width(\"xx\")}[\\pgfmathresult]\\pgfmathsetmacro\\g{max(1,3)*2}[\\g]\n\n\
     \\setlength\\mydim{\\widthof{xx}*2}[\\the\\mydim]\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[
      ("p", "p1.1", r#"<p xml:id="p1.1">[21.11121][6.0]</p>"#),
      ("p", "p2.1", r#"<p xml:id="p2.1">[21.1112pt]</p>"#),
    ],
  ),
  // --- SBX2: frontmatter pieces cut whole (`content::Field::whole`; a table's line, `Field::past_open_environment`), a replaced author never digested
  // (`reading::superseded`), float.sty's `float@type` counter. A row's element by `xml:id`, an attribute, or "" the
  // first of its tag ---
  (
    "a `$^n$` list in `\\thanks` ending in `\\textbf{…}` (repro sectioning-frontmatter/thanks_mark_list_last_entry_ends_in_a_braced_command)",
    "pdflatex 0 `!`; Perl 0 errors 0 warnings",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_mark_list_last_entry_ends_in_a_braced_command.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[("document", "", SBX2_THANKS_BOLD)],
  ),
  (
    "a group in `\\thanks` holding two `$^n$` entries (repro sectioning-frontmatter/thanks_mark_list_group_holds_two_entries)",
    "pdflatex 0 `!`; Perl 0 errors",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_mark_list_group_holds_two_entries.tex"
    ),
    &[],
    (0, 1),
    &[
      "Warning:unexpected:frontmatter Frontmatter content check: 1 words the author blocks print are missing from the \
       placed frontmatter: and.",
    ],
    &[("document", "", SBX2_THANKS_GROUP)],
  ),
  (
    "a `\\newtcbox` command in `\\author` takes its argument (repro sectioning-frontmatter/author_box_command_takes_its_argument)",
    "pdflatex 0 `!`; Perl 0 errors",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_box_command_takes_its_argument.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[
      (
        "ref",
        "href=\"https://a.org/\"",
        r##"<ref class="ltx_href" color="#000000" href="https://a.org/">Website</ref>"##,
      ),
      (
        "ref",
        "href=\"https://b.org/\"",
        r##"<ref class="ltx_href" color="#000000" href="https://b.org/">Code</ref>"##,
      ),
    ],
  ),
  (
    "a `$^n$` list in `\\thanks` as a table's lines: each piece its line (revSBX2 B1) (repro sectioning-frontmatter/thanks_mark_list_in_a_tabular_one_line_each)",
    "pdflatex 0 `!`; Perl not run",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_mark_list_in_a_tabular_one_line_each.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[("document", "", SBX2_THANKS_TABLE)],
  ),
  (
    "the table's opening frame covers an `@{}` column spec (repro sectioning-frontmatter/thanks_mark_list_in_a_tabular_column_spec_at)",
    "pdflatex 0 `!`; Perl not run",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_mark_list_in_a_tabular_column_spec_at.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[("document", "", SBX2_THANKS_TABLE)],
  ),
  (
    "both entries on one table line: the last piece ends before `\\end{tabular}` (repro sectioning-frontmatter/thanks_mark_list_in_a_tabular_one_line)",
    "pdflatex 0 `!`; Perl not run",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_mark_list_in_a_tabular_one_line.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[("document", "", SBX2_THANKS_TABLE)],
  ),
  (
    "a piece holding the table's `\\end` and text after it: the counted text path (repro sectioning-frontmatter/thanks_mark_list_in_a_tabular_text_after)",
    "pdflatex 0 `!`; Perl not run",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/thanks_mark_list_in_a_tabular_text_after.tex"
    ),
    &[],
    (0, 1),
    &[
      "Warning:unexpected:frontmatter The frontmatter emitter writes \"Univ B more\" as its plain text, its markup lost: \
       span letters differ",
    ],
    &[("document", "", SBX2_THANKS_TABLE_MORE)],
  ),
  (
    "a replaced `\\author` with undefined `\\name`, `\\email` (repro sectioning-frontmatter/author_replaced_template_never_digested)",
    "pdflatex 0 `!`; Perl 0 errors",
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_replaced_template_never_digested.tex"
    ),
    &[],
    (0, 0),
    &[],
    &[("document", "", SBX2_ANN_UNIV_A)],
  ),
  (
    "llncs: a template `\\author` of `\\IEEEauthorblockN`s replaced (2605.25890)",
    "pdflatex 0 `!`; Perl 0 errors",
    "\\documentclass{llncs}\n\\title{T}\n\\author{\\IEEEauthorblockN{}\\IEEEauthorblockA{\\textit{Amazon}\\\\ a@b.com} \
     \\and \\IEEEauthorblockN{}\\IEEEauthorblockA{\\textit{UW}\\\\ c@d.edu}}\n\\author{Ann Lee\\inst{1} \\and Bob \
     Roe\\inst{2}}\n\\institute{Amazon \\and UW}\n\\begin{document}\n\\maketitle\nHi.\n\\end{document}\n",
    &[],
    (0, 0),
    &[],
    &[("document", "", SBX2_LLNCS)],
  ),
  (
    "the replacing `\\author`'s own undefined `\\foo` still errs",
    "pdflatex 1 `!` (Undefined control sequence); Perl 1 error (undefined:\\foo)",
    "\\documentclass{article}\n\\title{T}\n\\author{\\name Ann Lee \\email a@b.edu}\n\\author{Ann Lee \\\\ Univ A \
     \\foo}\n\\begin{document}\n\\maketitle\nHi.\n\\end{document}\n",
    &[],
    (1, 0),
    &["Error:undefined:\\foo The token T_CS[\\foo] is not defined."],
    &[("document", "", SBX2_ANN_UNIV_A)],
  ),
  (
    "float.sty's `\\c@float@type` read by a document (repro package-bindings/float_type_counter)",
    "pdflatex 0 `!`, \"Type 8.\"; Perl 3 errors (undefined:\\c@float@type, \\the\\c@float@type, \\thefloat@type)",
    include_str!("../../../tools/perfect_kernel/repros/package-bindings/float_type_counter.tex"),
    &[],
    (0, 0),
    &[],
    &[("p", "p1.1", r#"<p xml:id="p1.1">Type 8.</p>"#)],
  ),
];

/// Every row converted once, in-process: the exact counts, each named diagnostic as often as listed, a schema-valid
/// document and the whole elements.
#[test]
fn perfect_kernel_error_boundary() {
  for (name, oracle, tex, files, (errors, warnings), messages, elements) in BOUNDARY {
    let (log, xml) = convert_files_with(tex, files, Some("ar5iv.sty"));
    assert_eq!(error_count(&log), *errors, "{name} ({oracle}):\n{log}");
    assert_eq!(warning_count(&log), *warnings, "{name} ({oracle}):\n{log}");
    for message in *messages {
      let wanted = messages.iter().filter(|m| *m == message).count();
      let found = log.lines().filter(|line| line == message).count();
      assert_eq!(found, wanted, "{name}: {message}:\n{log}");
    }
    if let Some(lines) = rng_error_count(&xml) {
      assert_eq!(lines, 0, "{name}: schema:\n{xml}");
    }
    for (tag, id, element) in *elements {
      let attr = match *id {
        "" => None,
        id if id.contains('=') => Some(id.to_string()),
        id => Some(format!("xml:id=\"{id}\"")),
      };
      assert_element(
        &xml,
        tag,
        attr
          .as_slice()
          .iter()
          .map(String::as_str)
          .collect::<Vec<_>>()
          .as_slice(),
        element,
      );
    }
  }
}
