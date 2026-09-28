//! Red/green guards for perfect-kernel batch 56 (codebox manual:
//! \SetCatcodeRange / \setcatcoderange / \@setrangecatcode, \lstloadaspects,
//! \DeclareTCBListing nested inside \NewDocumentEnvironment with bare
//! environment invocation and outer listing scanning, and unicode-math table loading).
use super::perfect_kernel_batch46::{
  convert, convert_args, convert_files, convert_files_with, convert_html, convert_with,
  convert_with_then, error_count, warning_count,
};

/// Self-skip helper: is this file in the host TeX tree?
fn kpsewhich_has(name: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// Perl `State.pm:113-115` letters only ASCII and pdfTeX never letters a
/// non-ASCII char (utf8.def makes the bytes active), so under the default
/// profile `\xα` is `\x` followed by α, not one control sequence.
#[test]
fn non_ascii_letters_stay_other_under_pdftex() {
  let tex =
    "\\documentclass{article}\n\\def\\x{OK}\n\\begin{document}\n\\xα and \\x中.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("OKα and OK中."), "{xml}");
}

/// load-unicode-data.tex:134-135: the LuaTeX format letters every L/M code
/// point, Latin-1 included (the dump pins U+0080-U+00FF OTHER, the profile
/// re-letters them). Witnesses: circledtext, jnuexam, tikz-bagua.
#[test]
fn non_ascii_letters_are_letters_under_luatex() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\typeout{CC:\\the\\catcode`é:\\the\\catcode`α:\\the\\catcode`中:\\the\\catcode`Ⅳ}\n\\ifcat A中 ZH-LETTER\\else ZH-OTHER\\fi\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("CC:11:11:11:12"), "{stderr}");
  assert!(xml.contains("ZH-LETTER"), "{xml}");
}

/// A `\<type>name` that takes arguments (argumentation.sty:403 `\afname{…}`
/// draws a tikz node) is not the counter's name noun: `\refstepcounter{af}`
/// must format the tag from `\theaf` alone. KPE #194 (SHARED Perl failure;
/// pdflatex clean).
#[test]
fn counter_name_command_is_not_a_name_noun() {
  let tex = r"\documentclass{article}
\newcounter{af}
\NewDocumentCommand{\afname}{m}{\node[caption](x){#1};}
\newcounter{gadget}
\newcommand{\gadgetname}{Gadget}
\newcounter{zero}
\NewDocumentCommand{\zeroname}{}{Zero}
\makeatletter
\begin{document}
\refstepcounter{af}\label{a}\refstepcounter{gadget}\label{g}\refstepcounter{zero}\label{z}
See \ref{a}, \ref{g} and \ref{z}.
\typeout{NOUN:\iflx@namenoun\gadgetname Y\else N\fi:\iflx@namenoun\afname Y\else N\fi:\iflx@namenoun\zeroname Y\else N\fi:\iflx@namenoun\figurename Y\else N\fi}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("ERROR"), "{xml}");
  // plain macro noun / m-taking xparse command / zero-arg xparse noun / kernel noun
  assert!(stderr.contains("NOUN:Y:N:Y:Y"), "{stderr}");
}

/// A `\lstnewenvironment` listing used as `\begin{name}` terminates only at its
/// own `\end{name}`: a literal `\end{document}` in the body is verbatim content
/// (listings.sty:2211-2215 compares against `\@currenvir` = name). The bare
/// `\name` form (tcolorbox inside a wrapper environment) keeps terminating at
/// the enclosing environment's `\end`.
#[test]
fn lstnewenvironment_begin_form_keeps_literal_end_document() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\lstnewenvironment{mycode}{}{}
\begin{document}
\begin{mycode}
line one of code
\end{document}
line three of code
\end{mycode}
Tail text survives.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Tail text survives."), "{xml}");
  // base64 of "line one of code\n\end{document}\nline three of code"
  assert!(
    xml.contains("bGluZSBvbmUgb2YgY29kZQpcZW5ke2RvY3VtZW50fQpsaW5lIHRocmVlIG9mIGNvZGU="),
    "{xml}"
  );
}

/// pict2e.sty:742-774 path interface (dvips mode under `\pdfoutput=0`):
/// witnesses fancyqr-doc, curve2e-manual (RUST-ONLY: Perl raw-loads pict2e).
#[test]
fn pict2e_path_interface_strokes_a_polyline() {
  let tex = r"\documentclass{article}
\usepackage{pict2e}
\begin{document}
\setlength{\unitlength}{1mm}
\begin{picture}(40,40)
\moveto(0,0)
\lineto(40,0)
\lineto(40,40)
\closepath
\strokepath
\moveto(5,5)\curveto(10,20)(30,20)(35,5)\strokepath
\circlearc{20}{20}{10}{0}{90}\fillpath
\end{picture}
\newdimen\SIXR \SIXR=50pt
\begin{picture}(100,40)\moveto(\SIXR,20)\lineto(0,0)\strokepath\end{picture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<picture").count(), 2, "{xml}");
  // triangle + sampled curve + arc + the register-coordinate segment
  assert_eq!(xml.matches("<line ").count(), 4, "{xml}");
  // FramedSyntax.sty:189 shape: a dimen REGISTER coordinate is not 0
  assert!(!xml.contains("points=\"0,0 0,0\""), "{xml}");
}

/// hyperref.sty:3298-3311/3973-3979 storage macros and the :4092-4093
/// driver link pair (witnesses movie15 overlay-example, hrefhide-example,
/// ucalgmthesis sample-thesis; SHARED with Perl, pdflatex clean).
#[test]
fn hyperref_storage_and_driver_link_internals() {
  let tex = r"\documentclass{article}
\usepackage{hyperref}
\makeatletter
\begin{document}
\edef\z{/C [\@urlbordercolor] /H \@pdfhighlight (\@citebordercolor)(\@anchorcolor)}\typeout{Z:\z}
Text \hyper@linkstart{link}{target}anchor\hyper@linkend\ end.
\hyper@natlinkstart{k}cite\hyper@natlinkend.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // the space after `\@pdfhighlight` is a control-word space, eaten by the tokenizer
  assert!(
    stderr.contains("Z:/C [0 1 1] /H /I(0 1 0)(black)"),
    "{stderr}"
  );
  assert!(xml.contains("Text anchor end."), "{xml}");
  assert!(xml.contains("cite."), "{xml}");
}

/// showexpl.sty:58-61,115 switches: an undefined `\if@SX@…` inside a
/// skipped branch desyncs the skip (tex.web §510). Witness pst-exa-doc
/// (`\usepackage[tcb]{pst-exa}`), SHARED with Perl, pdflatex clean.
#[test]
fn showexpl_switches_balance_a_skipped_branch() {
  let tex = r"\documentclass{article}
\usepackage{showexpl}
\makeatletter
\newif\ifsw
\swfalse
\begin{document}
\ifsw
  \renewcommand*\Foo{%
    \ifx\a\@empty
      \if@SX@rangeaccept X\else Y\fi
    \else
      \begin{center}c\end{center}%
    \fi
  }%
\else
TCB-OK
\fi
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("TCB-OK"), "{xml}");
  assert!(!xml.contains(">c<"), "{xml}");
}

/// `\Umathcodenum` is an internal integer: `\the\Umathcodenum"2F` reads a
/// number (fixdif.sty:38; physics2, physics2-legacy).
#[test]
fn umathcodenum_is_an_internal_integer() {
  let tex = r#"\documentclass{article}
\begin{document}
\count0=\numexpr(\the\Umathcodenum"2F-"2F)/16777216\relax X\typeout{UM:\the\count0}
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("UM:0"), "{stderr}");
  assert!(xml.contains("X"), "{xml}");
}

/// caesar_book.cls:106-115 counts title lines with a `\lastbox` loop;
/// `\unpenalty` must not push a box per iteration (sidenotes caesar_example,
/// an unbounded runaway in Perl too; pdflatex terminates).
#[test]
fn unpenalty_does_not_grow_the_box_list() {
  let tex = r"\documentclass{article}
\makeatletter
\begin{document}
\setbox0\vbox{A title line here\par
  \count@\z@
  \loop
  \unskip\unpenalty\unskip\unpenalty\unskip
  \setbox0\lastbox
  \ifvoid0 \xdef\numlines{\the\count@}\else \advance\count@\@ne \repeat}%
numlines=\numlines
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("numlines="), "{xml}");
}

/// latex.ltx:17570-17591 allocates `\@marbox` before dispatching to
/// `\@ympar`; classes redefine `\@ympar` with the kernel idiom
/// (caesar_book.cls:84-87), so `\marginpar` must dispatch to private targets
/// (witness sidenotes caesar_example, 42 errors; RUST-ONLY, Perl and pdflatex clean).
#[test]
fn marginpar_ignores_a_class_redefined_ympar() {
  let tex = r"\documentclass{article}
\usepackage{graphicx}
\usepackage{sidenotes}
\makeatletter
\newcommand{\marginparstyle}{\footnotesize}
\long\def\@ympar#1{%
  \@savemarbox\@marbox{\marginparstyle#1}%
  \global\setbox\@currbox\copy\@marbox
  \@xympar}
\makeatother
\begin{document}
Text.
\begin{marginfigure}
  \includegraphics[width=\marginparwidth]{example-image-a}
  \caption{A margin figure.\label{f}}
\end{marginfigure}
More text~\ref{f}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("role=\"margin\""), "{xml}");
  assert!(xml.contains("<figure") && xml.contains("<caption"), "{xml}");
}

/// A `\lstnewenvironment` whose body is diverted to a file
/// (`\lst@BeginWriteFile`) still runs its end code (pst-exa.sty:163-170
/// closes a start-code `\hbox` there and reads the result back; pst-exa-doc,
/// RUST-ONLY).
#[test]
fn lstnewenvironment_writefile_runs_the_end_code() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\makeatletter
\def\SX@put@code@result{RESULTMARK}
\lstnewenvironment{myex}[1][]
 {\setbox\@tempboxa=\hbox\bgroup\lst@BeginWriteFile{\jobname.swpl}}
 {\lst@EndWriteFile\egroup\SX@put@code@result}
\makeatother
\begin{document}
\begin{myex}[pos=t]
hello world
\end{myex}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("RESULTMARK"), "{xml}");
}

/// latex.ltx:9247-9266 cr chain entered directly by brief.cls:496
/// `\@nobreakcr` (ntgclass brief-sample, RUST-ONLY: the raw `\@gnewline`
/// body dereferences `\reserved@f` when a parbox body is re-expanded).
#[test]
fn raw_newline_chain_is_the_native_newline() {
  let tex = r"\documentclass{brief}
\name{WG}
\begin{document}
\begin{brief}{Jan}\end{brief}
\begin{brief}{Jan}\opening{Hallo,} \ondertekening{Victor}\afsluiting{doei}\end{brief}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Victor") && xml.contains("doei"), "{xml}");
}

/// Perl locks both `\tabular` and `\endtabular`; a class redefining both
/// (jpsj2.cls:652,657) must have both dropped (witness jpsj injpsj2, RUST-ONLY).
#[test]
fn tabular_delegator_is_locked_with_endtabular() {
  let tex = r"\documentclass{article}
\makeatletter
\def\tabular{\begin{center}\let\@halignto\@empty\@tabular}
\def\endtabular{\crcr\egroup\egroup $\egroup\end{center}}
\makeatother
\begin{document}
\begin{center}
\begin{tabular}{cc} a & b \\ c & d \end{tabular}
\end{center}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  for cell in ["a", "b", "c", "d"] {
    assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
  }
}

/// `\ifdefined\Uchar` (and `\primitive`) are Unicode-engine detection
/// probes (ucharcat.sty, math-operator.sty); pdfTeX has neither, so the
/// default profile must leave them undefined (sweep-38 regression).
#[test]
fn unicode_engine_primitives_stay_undefined_under_pdftex() {
  let tex = r#"\documentclass{article}
\begin{document}
\ifdefined\Uchar \Umathchar"0"0"0 \fi
\ifdefined\primitive \Umathchar"0"0"0 \fi
Done.
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Done."), "{xml}");
}

/// latex.ltx:16560 `\@tabular` is parameterless with a literal `$` that
/// luababel.def:1895 `\bbl@replace\@tabular{$}{…}` rescans (lettrine-demo-arabic
/// and every babel `bidi=basic` document; sweep-38 TokenLimit regression).
#[test]
fn at_tabular_is_parameterless_with_a_patchable_math_shift() {
  let tex = r"\documentclass{article}
\makeatletter
\long\def\bbl@afterfi#1\fi{\fi#1}
\def\bbl@replace#1#2#3{%
  \toks@{}%
  \def\bbl@replace@aux##1#2##2#2{%
    \ifx\bbl@nil##2\toks@\expandafter{\the\toks@##1}%
    \else\toks@\expandafter{\the\toks@##1#3}\bbl@afterfi\bbl@replace@aux##2#2\fi}%
  \expandafter\bbl@replace@aux#1#2\bbl@nil#2%
  \edef#1{\the\toks@}}
\def\PATCHED{}
\bbl@replace\@tabular{$}{$\def\PATCHED{patched}}%
\makeatother
\begin{document}
\begin{tabular}{cc} \PATCHED & b \\ c & d \end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  // `\@tabular` is locked, so the rescanned `\edef` is dropped (the patch only
  // sets bidi layout state); what matters is that the `$` scan balanced and
  // the alignment still works.
  for cell in ["b", "c", "d"] {
    assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
  }
}

/// curve2e.sty raw-loads over pict2e's driver-level path builders: vector
/// algebra, `\Arc`, `\VectorARC`, `\Zbox`/`\Pbox`, `\xmultiput`, `\AutoGrid`
/// (witness curve2e-manual, 32 undefined-command errors with the old stub).
#[test]
fn curve2e_raw_load_renders_arcs_and_vectors() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{curve2e}
\begin{document}
\setlength{\unitlength}{1mm}
\CopyVect 3,4 to\V \ModOfVect\V to\M Mod=\M.
\begin{picture}(40,40)
\Arc(20,20)(30,20){90}
\VectorARC(20,20)(30,20){60}
\Zbox(40,0)[l]{40,0}[1]
\Pbox(0,0)[r]{C}[0.75ex]
\xmultiput(0,0)(8,0){5}{\circle*{1}}
\AutoGrid
\end{picture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Mod=5"), "{xml}");
  // the arc, the vector arc and the grid all render as lines
  assert!(xml.matches("<line ").count() >= 3, "{xml}");
  assert!(xml.matches("<circle").count() >= 5, "{xml}");
}

/// booktabs.sty:53-118 rule machinery for documents that copy the real
/// `\midrule` (l2kurz.tex:58-65): `\@BTendrule` closes the `\noalign{`
/// that `\ifnum0=`}\fi` opened (witness lshort-german l2kurz, 41 errors).
#[test]
fn booktabs_rule_machinery_closes_its_noalign() {
  let tex = r"\documentclass{article}
\usepackage{array,longtable,tabularx,booktabs}
\makeatletter
\def\midrule{\noalign{\ifnum0=`}\fi\penalty\@M
  \@aboverulesep=\aboverulesep \global\@belowrulesep=\belowrulesep
  \global\@thisruleclass=\@ne
  \@ifnextchar[{\@BTrule}{\@BTrule[\lightrulewidth]}}
\makeatother
\begin{document}
\begin{tabular}[t]{rl}
\toprule A & B \\ \midrule 1 & 2 \\ \bottomrule
\end{tabular}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  for cell in ["A", "B", "1", "2"] {
    assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
  }
  assert!(xml.contains("After."), "{xml}");
}

/// magyar.ldf:1882-1898 calls the removed caption3 internal
/// `\caption@setdefaultlabelsep` only when `\caption@lsep@default` is
/// undefined (witnesses elteikthesis ×3, elteiktdk ×2; RUST-ONLY).
#[test]
fn caption_lsep_default_keeps_magyar_off_the_removed_internal() {
  let tex = r"\documentclass{article}
\usepackage[hungarian]{babel}
\usepackage{caption}
\begin{document}
\begin{figure}
\centering Test
\caption{Teszt \'abra}
\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<caption"), "{xml}");
  assert!(!xml.contains("ERROR"), "{xml}");
}

/// tcolorbox's `\dispExample` runs its body via `\tcbusetemp` = `\input`
/// (tcolorbox.sty:2820), so a mid-body `\ExplSyntaxOn` applies to what
/// follows (witness csvsimple-l3; RUST-ONLY: the body was eagerly tokenized).
#[test]
fn dispexample_body_runs_with_live_catcodes() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{documentation}
\begin{document}
\begin{dispExample}
\ExplSyntaxOn
\tl_new:N \l_test_tl
\tl_set:Nn \l_test_tl {LI\csname VE\endcsname}
\tl_use:N \l_test_tl \gdef\EXECUTED{yes}
\ExplSyntaxOff
\end{dispExample}
\ifdefined\EXECUTED RAN-\else NOT-\fi
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("tl_new"), "{xml}");
  // the executed body defines a macro the listing display cannot
  assert!(xml.contains("RAN-"), "{xml}");
}

/// `\tikzexternalize` without shell escape: tikz's `mode=graphics if exists`
/// typesets the picture inline with no system-call error (witnesses
/// tikzviolinplots 591, causets 106, tilings 80, tikz-feynhand 55; SHARED).
#[test]
fn tikz_externalize_typesets_inline_without_a_system_call() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{external}
\tikzexternalize[prefix=ext/]
\begin{document}
\begin{tikzpicture}
\draw (0,0) circle (1);
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<picture") || xml.contains("<svg"), "{xml}");
}

/// codehigh's non-LuaTeX parser is O(n²) on the l3regex VM; a whole package
/// source through `\dochighinput` must still finish (fontscale-code and 6
/// more manuals timed out; SHARED with Perl, pdflatex fast).
#[test]
fn codehigh_dochighinput_is_bounded() {
  let tex = r"\documentclass{article}
\usepackage{codehigh}
\begin{document}
\dochighinput[language=latex/latex3]{fontscale.sty}
\end{document}
";
  // ~38 s alone; a full-suite run needs the wider budget (still bounded).
  let (stderr, xml) = super::perfect_kernel_batch46::convert_with_budget(tex, None, 300);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("ProvidesExplPackage") || xml.contains("fontscale"),
    "{xml}"
  );
}

/// `DefToken` skips blanks before the token being defined (tex.web §1215):
/// `\lstMakeShortInline [opts] {"}` must activate `"`, not the space
/// (install-latex-guide-zh-cn:111 → a `\maketitle` recursion Fatal; SHARED).
#[test]
fn deftoken_skips_a_leading_space() {
  let tex = r#"\documentclass{article}
\usepackage{listings}
\lstMakeShortInline [ x = 1 ] {"}
\newcommand {\foo} {FOO}
\begin{document}
SP[\the\catcode`\ ]DQ[\the\catcode`\"] \foo
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("SP[10]DQ[13]"), "{xml}");
  assert!(xml.contains("FOO"), "{xml}");
}

/// dhucs.sty:44 `\ifx 가가` takes the native-Unicode branch here, whose
/// `\dhucs@hu` lives behind LuaTeX/XeTeX probes; the engine-neutral subset
/// is supplied after the raw load (kotex-oblivoir manuals; SHARED, pdflatex clean).
#[test]
fn dhucs_native_branch_defines_the_hangul_skip() {
  let tex = r"\documentclass{article}
\makeatletter
\RequirePackage{dhucs}
\newdimen\x@hu \x@hu=\dhucs@hu
\setInterHangulSkip{1pt}
\makeatother
\begin{document}
ok
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ok"), "{xml}");
}

/// tcolorbox listings default to `listing and text`: the body is displayed
/// AND executed (tcblistingscore.code.tex:429/:205); `listing only` is not
/// (witnesses postit-doc-en/fr, 16 "No shape named" errors; RUST-ONLY).
#[test]
fn tcblisting_listing_and_text_executes_the_body() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newtcblisting{DemoCode}[1][]{listing options={commentstyle={\itshape}},#1}
\begin{document}
\begin{DemoCode}[]
\begin{tikzpicture}[remember picture]
  \coordinate (foo-N-W) at (0,0);
\end{tikzpicture}
\end{DemoCode}
\begin{tikzpicture}[remember picture,overlay]
  \draw (foo-N-W) circle[radius=2pt];
\end{tikzpicture}
\begin{DemoCode}[listing only]
\def\ONLYDISPLAYED{ran}
\end{DemoCode}
\ifdefined\ONLYDISPLAYED RAN\else NOTRUN\fi
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.matches("<svg").count() >= 2 || xml.matches("<picture").count() >= 2,
    "{xml}"
  );
  assert!(xml.contains("NOTRUN"), "{xml}");
}

/// beamerbasefont.sty:322-323 `\Tiny`/`\TINY` (font themes use them) and
/// caption3.sty:701 `\DeclareCaptionFormat*{name}{code}` consumed whole
/// (nostarch.cls:856 left `#1#2#3` in the stream). Both SHARED with Perl.
#[test]
fn beamer_tiny_sizes_and_starred_caption_format() {
  let tex = r"\documentclass{beamer}
\usepackage{caption}
\DeclareCaptionFormat*{myfmt}{\parbox{5cm}{#1#2#3}}
\DeclareCaptionFormat{plain2}[short]{#1#2#3\par}
\begin{document}
\begin{frame}
{\Tiny tiny text} {\TINY tinier text} Hello world.
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("tiny text") && xml.contains("tinier text") && xml.contains("Hello world."),
    "{xml}"
  );
  assert!(!xml.contains("#1"), "{xml}");
}

/// latex.ltx:15515-15521 `\@noligs` neutralises an active `<` (l3doc's
/// `function` shorthand) inside fancyvrb verbatim (witnesses interface3,
/// source3, source2e; SHARED with Perl, pdflatex clean).
#[test]
fn noligs_neutralises_active_chars_in_verbatim() {
  let tex = r"\documentclass{article}
\usepackage{fancyvrb}
\makeatletter
\catcode`\<=\active
\def<#1>{\textit{#1}}
\makeatother
\begin{document}
Meta <arg> outside.
\begin{Verbatim}
\dim_compare_p:n { #1 <= #2 }
next {line}
\end{Verbatim}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("dim_compare_p:n") && xml.contains("&lt;= #2") || xml.contains("<= #2"),
    "{xml}"
  );
  assert!(xml.contains("After."), "{xml}");
}

/// `\errmessage` counts as an error (tex.web §1283), so an expl3
/// `\msg_error` loop is cut by the error breaker (the total cap: 100 errors, then
/// `Fatal:TooManyErrors`, as pdflatex stops) instead of running to the token limit
/// (csvsimple-l3 `sort by=` with no sorter).
#[test]
fn errmessage_counts_toward_the_error_breaker() {
  let tex = r"\documentclass{article}
\usepackage{csvsimple-l3}
\begin{filecontents*}[overwrite]{grade.csv}
name,givenname
Maier,Hans
\end{filecontents*}
\begin{filecontents*}[overwrite]{namesort.xml}
<sortconfig/>
\end{filecontents*}
\ExplSyntaxOn \tl_gclear_new:N \csvline \ExplSyntaxOff
\begin{document}
Before.
\csvreader[sort by=namesort.xml]{grade.csv}{}{X}
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  // pdflatex also loops to its 100-error stop: 101 `\errmessage` errors, then the one Fatal.
  assert_eq!(stderr.matches("Fatal:TooManyErrors").count(), 1, "{stderr}");
  assert_eq!(stderr.matches("Error:errmessage:").count(), 101, "{stderr}");
  assert_eq!(error_count(&stderr), 102, "{stderr}");
  assert!(!stderr.contains("Fatal:Timeout"), "{stderr}");
  assert!(
    stderr.matches("not existent").count() < 700,
    "{}",
    stderr.matches("not existent").count()
  );
}

/// A preload that itself pulls in the LaTeX pool + dump pushes with the
/// native `\lx@pushfilename`; the pop must use the SAME decision (Perl's
/// `$pushpop`, Package.pm:2578/2637), not the dump's `\@popfilename` —
/// otherwise `\__hook_curr_name_pop:` underflows ("Extra \PopDefaultHookLabel").
#[test]
fn preload_that_pulls_in_the_format_pops_with_the_native_stack() {
  let tex = r"\documentclass{article}
\begin{document}
Plain text.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("PopDefaultHookLabel"), "{stderr}");
  assert!(xml.contains("Plain text."), "{xml}");
}

/// `\tcbuselibrary{listings}` raw-loads tcblistingscore.code.tex, whose
/// `\NewDocumentCommand \newtcblisting` must not find our override already
/// defined (ltcmd `command-already-defined` = counted `\errmessage`). The
/// family is installed by the code.tex binding after the raw load.
#[test]
fn tcbuselibrary_listings_installs_the_family_once() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newtcblisting{mybox}{listing only}
\NewTCBListing{exbox}{ O{} }{listing only,#1}
\begin{document}
\begin{mybox}
int alpha = 1;
\end{mybox}
\begin{exbox}
int beta = 2;
\end{exbox}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("already defined"), "{stderr}");
  // listings bodies are base64 `data=` payloads.
  assert!(xml.contains("aW50IGFscGhhID0gMTs="), "{xml}");
  assert!(xml.contains("aW50IGJldGEgPSAyOw=="), "{xml}");
  assert!(xml.contains("After."), "{xml}");
}

/// graphics.sty:189 `\Ginclude@graphics` (driver-level include, called
/// directly by pagelayout.cls:1494) routes to the `\includegraphics`
/// constructor.
#[test]
fn ginclude_graphics_internal_routes_to_the_constructor() {
  let tex = r"\documentclass{article}
\usepackage{graphicx}
\makeatletter
\begin{document}
\Ginclude@graphics{example-image}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<graphics ").count(), 1, "{xml}");
  assert!(xml.contains(r#"graphic="example-image""#), "{xml}");
}

/// A pgfmath string result carrying a control sequence stays executable
/// (pgfmathparser.code.tex:392-396 keeps `"…"` operands as real tokens);
/// braids.sty:276 sets its strand counter through `\pgfmathresult`.
#[test]
fn pgfmath_string_result_keeps_control_sequences() {
  let tex = r#"\documentclass{article}
\usepackage{tikz}
\newcounter{mytest}
\setcounter{mytest}{1}
\begin{document}
\pgfmathparse{\value{mytest} < 4 ? "\noexpand\setcounter{mytest}{4}" : ""}%
\pgfmathresult
\typeout{EXECVAL:\the\value{mytest}}
\pgfmathparse{2*3}\typeout{NUMVAL:\pgfmathresult}
\end{document}
"#;
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("EXECVAL:4"), "{stderr}");
  assert!(stderr.contains("NUMVAL:6"), "{stderr}");
}

/// xkeyval.tex:248 `\XKV@cc` (beamerposter.sty:55 calls it directly) and
/// xkvutils.tex:110-124 `\XKV@whilist` (powerdot) are verbatim ports.
#[test]
fn xkeyval_choice_check_and_whilist_internals() {
  let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\XKV@cc*+[\val\nr]{a1}{a0,a1,a2}{\typeout{XKVCC-OK val=\val\space nr=\nr}}{\typeout{XKVCC-BAD}}
\define@choicekey{fam}{shape}[\val\nr]{circle,square}{\typeout{shape=\val/\nr}}
\def\lst{alpha,beta,gamma}
\XKV@whilist\lst\itm\ifx\itm\@nnil\fi{\typeout{WH:\itm}}
\makeatother
\begin{document}
\setkeys{fam}{shape=square}
Done.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("XKVCC-OK val=a1 nr=1"), "{stderr}");
  assert!(stderr.contains("shape=square/1"), "{stderr}");
  assert!(xml.contains("Done."), "{xml}");
}

/// fonttext.ltx:57-68,93: a Unicode-engine format inputs tuenc.def and makes
/// TU the default encoding; xunicode-addon.sty:59-113 checks `\T@TU` exists.
#[test]
fn tu_encoding_is_declared_under_luatex() {
  let tex = r#"\documentclass{article}
\usepackage{xunicode-addon}
\makeatletter
\typeout{TUENC:\UnicodeEncodingName:\encodingdefault:\ifcsname T@TU\endcsname yes\else no\fi}
\makeatother
\begin{document}
Caf\'e na\"ive \textdollar\ \S
\end{document}
"#;
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("TUENC:TU:TU:yes"), "{stderr}");
  assert!(xml.contains("Café naïve $ §"), "{xml}");
}

/// tex.web §1063/§1064 `off_save`: `\endgroup` against an open math frame
/// inserts the missing `$`, closes the math and re-reads the `\endgroup`
/// (Perl leaves the frame open and every later closer re-errors).
#[test]
fn endgroup_against_open_math_inserts_the_missing_dollar() {
  let tex = r"\documentclass{article}
\begin{document}
Before \begingroup $x+1\endgroup after.

Next paragraph.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Missing $ inserted"), "{stderr}");
  assert!(!stderr.contains("Attempt to close"), "{stderr}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  assert!(xml.contains("after."), "{xml}");
  assert!(xml.contains("Next paragraph."), "{xml}");
  // The math closed inside the paragraph: no Math element after the last </para>.
  let last_para_end = xml.rfind("</para>").unwrap_or(0);
  assert!(!xml[last_para_end..].contains("<Math"), "{xml}");
}

/// `\newtcblisting{env}[1]{…,#1}`: `[1]` is one MANDATORY argument
/// (tcblistingscore.code.tex:318-323), so `\begin{env}{listing only}` reaches
/// the mode decision and the displayed preamble code is not executed.
#[test]
fn newtcblisting_mandatory_argument_stays_mandatory() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings,skins}
\newtcblisting{DemoCode}[1]{%
	enhanced,width=\linewidth,%
	listing options={breaklines=true,commentstyle={\itshape}},%
	#1
}
\newtcblisting{OptCode}[1][listing only]{#1}
\begin{document}
\begin{DemoCode}{listing only}
\usepackage{calculatoritems}
\end{DemoCode}
\begin{OptCode}
\usepackage{calculatoritems}
\end{OptCode}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml
      .matches("XHVzZXBhY2thZ2V7Y2FsY3VsYXRvcml0ZW1zfQ==")
      .count(),
    2,
    "{xml}"
  );
  assert!(xml.contains("After."), "{xml}");
}

/// nicematrix.sty:5772→5704: `\rowcolors`/`\rowlistcolors` absorb a trailing
/// `[keys]` optional (manual :2412 `[cols=2-3,restart]`, :2446 `[respect-blocks]`).
#[test]
fn nicematrix_rowcolors_trailing_optional_is_absorbed() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix}
\usepackage[table]{xcolor}
\begin{document}
\begin{NiceTabular}{lr}
\CodeBefore
  \rowcolors[gray]{2}{0.8}{}[cols=2-3,restart]
  \rowlistcolors{1}{blue!10}[respect-blocks]
\Body
a & 12 \\
b & 13 \\
\end{NiceTabular}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("cols=2-3"), "{xml}");
  assert!(!xml.contains("respect-blocks"), "{xml}");
  assert!(xml.contains("<tabular"), "{xml}");
  assert!(xml.contains(">13<") || xml.contains("13</"), "{xml}");
}

/// A pgfmath string result keeps its letters at catcode 11
/// (pgfmathparser.code.tex:35-40), so `\pgfmathresult` = `arc[…]` dispatches
/// in `\tikz@handle` (tikz.code.tex:2134-2163) instead of "Giving up".
#[test]
fn pgfmath_string_result_keeps_letter_catcodes() {
  let tex = r#"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
  \node (a) at (0,0) {A};
  \draw (a) \pgfextra{\pgfmathparse{"arc[start angle=90,end angle=180,radius=5pt]"}}%
    \pgfmathresult;
\end{tikzpicture}
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Giving up"), "{stderr}");
  assert!(xml.matches("<svg:path").count() >= 1, "{xml}");
}

/// tcblistingscore.code.tex:195-224: the listing mode is tcolorbox's resolved
/// state; `listing only` inside a user `.style` (tutodoc.cls:1208) must not
/// execute the body.
#[test]
fn tcb_listing_mode_hidden_in_a_style_is_honoured() {
  let tex = r#"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\tcbset{mystyle/.style={listing only}}
\NewTCBListing{mycode}{ m }{ mystyle }
\NewTCBListing{runcode}{ m }{ listing and text }
\begin{document}
\begin{mycode}{}
if ($name eq "") { print "hi $name"; }
\end{mycode}
\begin{runcode}{}
\gdef\RAN{yes}
\end{runcode}
\typeout{RAN:\ifdefined\RAN\RAN\else no\fi}
After.
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<XMath"), "{xml}");
  assert!(
    xml.contains("aWYgKCRuYW1lIGVxICIiKSB7IHByaW50ICJoaSAkbmFtZSI7IH0="),
    "{xml}"
  );
  assert!(stderr.contains("RAN:yes"), "{stderr}");
  assert!(xml.contains("After."), "{xml}");
}

/// codebox.sty:268 sets `listing only` from the ENCLOSING environment before
/// its `\DeclareTCBListing` box; the C body must stay a listing.
#[test]
fn tcb_listing_mode_set_by_the_enclosing_environment_is_honoured() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\DeclareTCBListing[]{codeviewaux}{m}{title={#1}}
\newenvironment{codeview}{\tcbset{listing only}\codeviewaux{X}}{\endcodeviewaux}
\begin{document}
\begin{codeview}{demo}
#include <stdio.h>
int main(){return 0;}
\end{codeview}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("misdefined"), "{stderr}");
  assert!(xml.contains("I2luY2x1ZGUgPHN0ZGlvLmg+"), "{xml}");
  assert!(xml.contains("After."), "{xml}");
}

/// tcolorbox.sty:2726-2735 `tcbverbatimwrite` writes the body without the
/// `\begin`-line remainder as an empty first line (csvsimple reads line 1).
#[test]
fn tcbverbatimwrite_has_no_leading_blank_line() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{documentation}
\usepackage{csvsimple-legacy}
\begin{document}
\begin{tcbverbatimwrite}{grade.csv}
name,givenname,matriculation,gender,grade
Maier,Hans,12345,m,1.0
\end{tcbverbatimwrite}
\csvautotabular{grade.csv}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("empty line"), "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  assert!(xml.contains("Maier"), "{xml}");
}

/// pdfpages.sty:205 `\includepdfset{…}` (tutodoc :1339) is absorbed.
#[test]
fn includepdfset_is_absorbed() {
  let tex = r"\documentclass{article}
\usepackage{pdfpages}
\includepdfset{pages=-,fitpaper=true}
\begin{document}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("After."), "{xml}");
}

/// tabularray.sty:2006/2008: `\hline[style]`/`\cline[style]` inside tblr
/// absorb their optional (manual :547 `\hline[dashed]\hline`).
#[test]
fn tblr_hline_style_optional_is_absorbed() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{lcr}
One & Two & Three \\
\hline[dashed]\hline
Four & Five & Six \\
\cline[dotted]{1-2}
Seven & Eight & Nine \\
\end{tblr}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("dashed"), "{xml}");
  assert!(!xml.contains("dotted"), "{xml}");
  assert!(xml.contains("Nine"), "{xml}");
  assert!(
    xml.contains(r#"border="tt""#) || xml.contains(r#"border="bb""#),
    "{xml}"
  );
}

/// beamer.cls:343 requires geometry; beamerposter.sty:176 calls `\geometry`.
#[test]
fn beamer_requires_geometry() {
  let tex = r"\documentclass{beamer}
\geometry{paperwidth=84.1cm,paperheight=118.9cm,hmargin=1cm}
\begin{document}
\begin{frame}\frametitle{Poster}Body text.\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Body text."), "{xml}");
  assert!(xml.contains("Poster"), "{xml}");
}

/// A `\lstnewenvironment` end code that displays a listing itself
/// (exsheets-listings.sty:89-112) runs once — it is not the postamble every
/// nested display re-reads.
#[test]
fn lstnewenvironment_end_code_with_a_listing_does_not_recurse() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\begin{filecontents*}[overwrite]{pre.lst}
preexisting line one
preexisting line two
\end{filecontents*}
\lstnewenvironment{myq}[1][]{}{\lstinputlisting{pre.lst}}
\begin{document}
\begin{myq}
hello listing
\end{myq}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert_eq!(xml.matches("<listing ").count(), 2, "{xml}");
  assert!(xml.contains("After."), "{xml}");
}

/// tagpdf-base.sty declares the tagging API with `\cs_new_protected`; the
/// no-op stubs for tagpdf-less documents are retracted before it loads.
#[test]
fn tagpdf_base_redeclares_the_stubbed_api_cleanly() {
  let tex = r"\RequirePackage{pdfmanagement}
\documentclass{article}
\begin{document}
\tagstructbegin{tag=P}\tagmcbegin{tag=P}Tagged.\tagmcend\tagstructend
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("already defined"), "{stderr}");
  assert!(xml.contains("Tagged."), "{xml}");
}

/// A `\luadef` slot whose Lua function scans its operand (luatexja-core.sty:
/// 408-421 `\ltjsetkanjiskip`, `token.scan_glue()`) reads and absorbs it; as a
/// bare no-op the glue was typeset as text, "0pt plus 0.25minus 0pt" on every
/// jlreq size change (`LUA_SLOT_OPERANDS`, latexml_sty/mod.rs).
#[test]
fn lua_slot_absorbs_its_scanned_operand() {
  let tex = "\\documentclass{article}\n\\luadef\\ltjsetkanjiskip 7\n\\begin{document}\nA\\ltjsetkanjiskip 0pt plus 0.25em minus 0pt B\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>AB</p>"), "{xml}");
}

/// expl3-code.tex:34944-34966 stubs `\lua_*` on a non-Lua format; the luatex
/// profile rebinds them to the bridge.
#[test]
fn lua_functions_are_live_under_the_luatex_profile() {
  let tex = r"\documentclass{article}
\ExplSyntaxOn
\lua_load_module:n { luaotfload-main }
\lua_now:n { tex.print('LUANOW') }
\ExplSyntaxOff
\begin{document}
Body.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("LuaTeX engine not in use"), "{stderr}");
  assert!(xml.contains("Body."), "{xml}");
}

/// tabularray's `\Set*` table commands are gobbled out of the cell
/// (tabularray.sty:3770-3860) and the `booktabs` environment takes the tblr
/// key-value spec through the same colspec extraction (:8163).
#[test]
fn tblr_table_commands_and_booktabs_env() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\UseTblrLibrary{booktabs}
\begin{document}
\begin{tblr}{colspec={lcr}}
 \SetRow{c}  Alpha   & Beta  & Gamma  \\
 \SetHline[1]{1-3}{solid}
 \SetColumn{c} Epsilon & Zeta  & Eta    \\
\end{tblr}
\begin{booktabs}{row{2}={c}}
\toprule
 One & Two & Three & Four \\
 Five & Six & Seven & Eight \\
\bottomrule
\end{booktabs}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert!(xml.contains("Alpha"), "{xml}");
  assert!(xml.contains("Eight"), "{xml}");
  assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
}

/// tex.web §1069/§1047 for a box reader: a box whose body left inline math
/// open (`\mbox{$x}`) closes the math into the box instead of running to the
/// end of the document; a balanced `\hbox{$x$}` stays error-free.
#[test]
fn box_end_over_leaked_math_closes_it_into_the_box() {
  let tex = r"\documentclass{article}
\begin{document}
Before \mbox{$x} after.

Next paragraph.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Missing $ inserted"), "{stderr}");
  assert!(!stderr.contains("malformed"), "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
  assert!(xml.contains("after."), "{xml}");
  assert!(xml.contains("Next paragraph."), "{xml}");
  let last_para_end = xml.rfind("</para>").unwrap_or(0);
  assert!(!xml[last_para_end..].contains("<Math"), "{xml}");
  let tex = r"\documentclass{article}
\begin{document}
Before \hbox{$x$} after.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
}

/// tcolorbox.sty:712 `tikz lower` wraps the box's executed lower part in a
/// `tikzpicture`; the executed listing body runs inside it.
#[test]
fn tcblisting_tikz_lower_wraps_the_executed_body() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage[most]{tcolorbox}
\tcbuselibrary{listings}
\newtcblisting{DemoCode}[1][]{#1}
\begin{document}
\begin{DemoCode}[tikz lower]
\draw (0,0) -- (2,1);
\coordinate (A) at (1,1);
\end{DemoCode}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<picture"), "{xml}");
  assert!(xml.contains("<svg:path"), "{xml}");
  assert!(xml.contains("After."), "{xml}");
}

/// beamerbasecompatibility.sty:517 `\beamertemplatedotitem` (called by the
/// miniframes outer theme) and beamerbasecolor.sty:149 `{beamercolorbox}`.
#[test]
fn beamer_theme_compat_aliases_and_colorbox() {
  let tex = r"\documentclass{beamer}
\usetheme[compress]{Singapore}
\begin{document}
\begin{frame}
\beamertemplatearticlebibitems
\begin{beamercolorbox}[wd=\textwidth,rounded=true]{block body}
Hello colored box.
\end{beamercolorbox}
\begin{itemize}\item One\end{itemize}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hello colored box."), "{xml}");
  assert!(xml.contains("<item"), "{xml}");
}

/// latex.ltx:9670 `\IfFileExists@` re-`\def`s the selected branch, halving
/// `##` once (chemexec.sty:274-289 defines `\react@##1` inside it).
#[test]
fn iffileexists_branch_halves_doubled_parameters() {
  let tex = r"\documentclass{article}
\begin{document}
\IfFileExists{article.cls}{%
  \long\def\reactx##1{[X ##1 Y]}%
}{}
\IfFileExists{no-such-file-xyz.sty}{}{\def\other##1{(O ##1)}}
\reactx{Z} \other{W}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[X Z Y]"), "{xml}");
  assert!(xml.contains("(O W)"), "{xml}");
}

/// xkeyval.tex:446-448: `\presetkeys` apply through `\ProcessOptionsX` and
/// `\ExecuteOptionsX` (powerdot.cls:52-92 sets `mode=present` only by preset);
/// head presets fill un-given keys, given keys win, tail presets follow.
#[test]
fn xkeyval_option_processing_applies_presetkeys() {
  let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\@namedef{opt@.}{size=12pt}
\define@choicekey*[pd]{class}{mode}[\pd@tempa\pd@mode]{present,print,handout}{}
\define@cmdkey[pd]{class}{size}{}
\define@cmdkey[pd]{class}{disp}{}
\presetkeys[pd]{class}{mode=present,size=10pt}{disp=tail}
\ProcessOptionsX[pd]<class>\relax
\typeout{PROBE:mode=\pd@mode:size=\cmdpd@class@size:disp=\cmdpd@class@disp}
\define@cmdkey[ex]{fam}{width}{}
\presetkeys[ex]{fam}{width=3cm}{}
\ExecuteOptionsX[ex]<fam>{}
\typeout{EXEC:width=\cmdex@fam@width}
\makeatother
\begin{document}
\makeatletter\ifnum\pd@mode>0 MODEGT\else MODELE\fi\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("PROBE:mode=0:size=12pt:disp=tail"),
    "{stderr}"
  );
  assert!(stderr.contains("EXEC:width=3cm"), "{stderr}");
  assert!(xml.contains("MODELE"), "{xml}");
}

/// The raw `{tcblisting}` environment (tcblistingscore.code.tex:275-283)
/// hands `\tcbverbatimwrite` the UNEXPANDED `\kvtcb@listingfile`; the body
/// must be stored under the expanded `\jobname.listing` name that
/// `\tcbinputlisting@core` reads back (sweep #40: 25 manuals regressed with
/// `missing_file:<job>.listing`; witness cistercian-doc).
#[test]
fn raw_tcblisting_environment_round_trips_its_listing_file() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\begin{document}
\begin{tcblisting}{title={Font scaling}}
\textbf{bold} Text
\end{tcblisting}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Can't find") && !stderr.contains("Can't read"),
    "{stderr}"
  );
  assert!(xml.contains("<listing"), "{xml}");
  assert!(xml.contains("font=\"bold\""), "{xml}");
}

/// The minted listing engine reads the listing back through `\inputminted`
/// from `\minted@outputdir <jobname>.listing` (tcbminted.code.tex:49-55);
/// the file name is expanded, so the listing holds the source (it was an
/// empty `<listing/>`: tkz-grapheur-examples-integrals, 32 manuals).
#[test]
fn raw_tcblisting_minted_engine_reads_its_listing_file() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/tcblisting_minted_listing_file.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // base64 of `\DrawZorp[Colors=blue]{h(x)}`
  assert!(
    xml.contains(r#"<listing class="ltx_lstlisting" data="XERyYXdab3JwW0NvbG9ycz1ibHVlXXtoKHgpfQ==" dataencoding="base64" datamimetype="text/plain">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r##"<text class="ltx_lst_identifier" color="#000000">DrawZorp</text>"##),
    "{xml}"
  );
}

/// `\newtcbinputlisting` reads its `listing file` as Perl's
/// listingsReadRawFile does (`FindFile(…, noltxml => 1)`): through kpathsea
/// and the source directory, not relative to the working directory. The
/// listing was an empty `<listing/>` with no diagnostic (commalists-tools-doc
/// `\DemoCodeFile{commalists-tools.sty}`, timeop-doc, csvsimple-legacy).
#[test]
fn newtcbinputlisting_finds_its_file_through_kpathsea() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/tcbinputlisting_kpathsea_file.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<listingline xml:id="lstnumberx15"><text class="ltx_lst_space">  </text>\<text class="ltx_lst_identifier">ProvidesPackage</text>{<text class="ltx_lst_identifier">ifpdf</text>}[2019/10/25<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">v3</text>.4<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">ifpdf</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">legacy</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">package</text>.<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">Use</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">iftex</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">instead</text>.]</listingline>"#),
    "{xml}"
  );
}

/// Perl `Gullet::readUntil` (Core/Gullet.pm:683-685): when the delimiter
/// never arrives, every scanned token is unread and the body is empty. A
/// `\verb` inside a pre-tokenized argument (`\footnote{…}`) can never meet
/// its ACTIVE delimiter (the `+` was frozen as OTHER), so Rust's reader ran
/// to the end of the argument and swallowed the rest of the note — a
/// `<item>` inside `<verbatim>` (platex-tools/plarray.tex:20, RUST-ONLY).
#[test]
fn verb_inside_pretokenized_argument_unreads_instead_of_running_away() {
  let tex = r"\documentclass{article}
\begin{document}
Body\footnote{intro
\begin{itemize}
\item Remove extra \verb+abc+ around tabular environment
\item Inhibit JFM glue
\end{itemize}
The package re-adds these.}.

After the footnote, top-level \verb+abc+ works fine.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Warning:"), "{stderr}");
  assert_eq!(xml.matches("<item ").count(), 2, "{xml}");
  // The footnote's `\verb` is an EMPTY verbatim (Perl's shape), not one
  // that swallowed the second item.
  assert!(
    xml.contains("<verbatim/>"),
    "the footnote \\verb is empty:\n{xml}"
  );
  assert!(
    xml.contains("The package re-adds these."),
    "the note tail survives:\n{xml}"
  );
  assert!(
    xml.contains(r#"<verbatim font="typewriter">abc</verbatim>"#),
    "top-level \\verb is unchanged:\n{xml}"
  );
}

/// latex.ltx:15504 `\verb@eol@error`: an unterminated `\verb` stops at the
/// end of its line with ONE recoverable error instead of scanning across
/// lines and swallowing a later `{verbatim}` (bigints manual; SHARED).
#[test]
fn verb_ended_by_end_of_line_recovers() {
  let tex = r"\documentclass{article}
\begin{document}
This package (\verb v1.1 ) helps you.

\begin{center}
\begin{verbatim}
\usepackage{bigints}
\end{verbatim}
\end{center}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("ended by end of line"), "{stderr}");
  assert!(xml.contains("<verbatim"), "{xml}");
  assert!(xml.contains(r"\usepackage{bigints}"), "{xml}");
  assert!(xml.contains("After."), "{xml}");
}

/// pdfTeX `\pdfmatch` (dataref.sty:374 `\let\dref@strmatch\pdfmatch`, then
/// `\ifnum\dref@strmatch{#1}{#2}=1`) expands to the match flag, and
/// `\pdflastmatch` to `<pos>-><text>` (dataref-doc; SHARED, pdflatex clean).
#[test]
fn pdfmatch_expands_to_a_match_flag() {
  let tex = r"\documentclass{article}
\begin{document}
\ifnum\pdfmatch{b}{abc}=1 yes\else no\fi.
\ifnum\pdfmatch{z}{abc}=0 none\else some\fi.
\ifnum\pdfmatch icase {B(C)}{abc}=1 \pdflastmatch0/\pdflastmatch1\fi.
\ifnum\pdfmatch{(}{abc}=-1 bad\fi.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("yes."), "{xml}");
  assert!(xml.contains("none."), "{xml}");
  assert!(xml.contains("bc/2-") && xml.contains("c.\nbad."), "{xml}");
  assert!(xml.contains("bad."), "{xml}");
}

/// A `\list` nested in a list that never rebound `\@listctr` (memoir.cls:4580
/// `\list` has no `\let\@listctr\@empty`; `adjustwidth` = `\begin{list}`,
/// memoir.cls:11267) inherits the OUTER list's counter. Routing that into
/// `begin_itemize` made `\the<ctr>@ID` point through the new list's id back
/// to itself — `Fatal:Timeout:PushbackLimit` (memman, dlfltxbcodetips in
/// s109). Perl's `\list` skips its list-start when the setup body left a
/// counter bound, so an inherited counter never starts a numbered list.
#[test]
fn list_inheriting_the_outer_counter_starts_an_unnumbered_list() {
  let tex = r"\documentclass{memoir}
\begin{document}
\begin{description}
\item[x] a
\begin{adjustwidth}{1cm}{1cm}
nested
\end{adjustwidth}
\end{description}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains("<item ") && xml.contains("nested"), "{xml}");
}

/// jmlrutils.sty:408 `{enumerate*}` = `\list{…}{\@nmbrlisttrue\def\@listctr{enumi}…}`
/// — continuous numbering, so a NESTED `enumerate*` reuses `enumi`. Same
/// self-referential id chain (pmlr-sample in s109).
#[test]
fn list_reusing_the_outer_counter_does_not_loop() {
  let tex = r"\documentclass[pmlr]{jmlr}
\jmlrvolume{1}\jmlryear{2010}\jmlrworkshop{W}
\title{T}\author{\Name{A} \Email{a@b.com}}
\begin{document}\maketitle
\begin{enumerate*}
  \item outer
  \begin{enumerate*}
    \item inner
    \begin{enumerate*}
      \item[] innermost
    \end{enumerate*}
  \end{enumerate*}
\end{enumerate*}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert_eq!(xml.matches("<item ").count(), 3, "{xml}");
  assert!(xml.contains("innermost"), "{xml}");
}

/// latex.ltx:16048 `\usecounter` is counter-only; LaTeXML made it the
/// list-start hook, whose `\let\item\list@item` clobbered fancybox's
/// `\let\item\Bitem` (fancybox.sty:251, 316: `\Benumerate` is an `\halign`
/// list) — three nested `<item>`s inside one `<td>` (fancybox-doc.tex:542).
#[test]
fn raw_halign_list_keeps_its_own_item() {
  let tex = r"\documentclass{article}
\usepackage{fancybox}
\begin{document}
\fbox{\begin{Benumerate}\item Groceries\item Hamster cages\end{Benumerate}}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains("<item"),
    "no <item> in an \\halign list:\n{xml}"
  );
  assert_eq!(xml.matches("<tr").count(), 2, "one row per \\Bitem:\n{xml}");
  assert!(
    xml.contains("Groceries") && xml.contains("Hamster cages"),
    "{xml}"
  );
}

/// The list-start moved from `\usecounter` to `\@trivlist` (latex.ltx:15862):
/// the counter value the setup body left (`\usecounter` zeroes it, a later
/// `\setcounter` continues a list) must reach the items unchanged.
#[test]
fn list_setup_counter_value_survives_into_the_items() {
  let tex = r"\documentclass{article}
\newcounter{ctr}
\begin{document}
\begin{list}{\arabic{ctr}.}{\usecounter{ctr}\setcounter{ctr}{5}}
\item Six
\item Seven
\end{list}
\begin{list}{\arabic{ctr}.}{\usecounter{ctr}}
\item One
\end{list}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<item ").count(), 3, "{xml}");
  for tag in ["<tag>6.</tag>", "<tag>7.</tag>", "<tag>1.</tag>"] {
    assert!(xml.contains(tag), "{tag}:\n{xml}");
  }
}

/// `\endlist` = `\endlx@list` = endMode('internal_vertical') (Perl
/// latex_constructs.pool.ltxml:1651-1653) also closes an enumerate opened
/// by its begin macro: nih/denselists.sty:16 `\newenvironment{Enumerate}
/// {\Onumerate\Nospacing}{\endlist}` (example-biosketch, polydemo; RUST-ONLY).
#[test]
fn endlist_closes_an_enumerate_opened_by_its_begin_macro() {
  let tex = r"\documentclass{article}
\let\Onumerate=\enumerate
\newenvironment{Enumerate}{\Onumerate}{\endlist}
\begin{document}
\begin{Enumerate}
\item x
\end{Enumerate}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<enumerate") && xml.contains("<item "),
    "{xml}"
  );
  assert!(xml.contains("After."), "{xml}");
}

/// latex.ltx:18901 `\AtBeginDocument` = `\AddToHook{begindocument}`, so a
/// `\RemoveFromHook{begindocument}[pkg]` cancels a package's
/// `\AtBeginDocument{\MakeShortVerb\"}` (source2edoc.cls:12 vs
/// l3doc.cls:511; base/source2e's ltoutenc macrocode leak; SHARED).
#[test]
fn atbegindocument_joins_the_l3_begindocument_hook() {
  let tex = r#"\documentclass{article}
\usepackage{doc}
\begin{filecontents}[overwrite,noheader,nosearch]{lxshortq.sty}
\AtBeginDocument{\MakeShortVerb\"}
\end{filecontents}
\usepackage{lxshortq}
\RemoveFromHook{begindocument}[lxshortq]
\AtBeginDocument{\def\lxhookran{ran}}
\begin{document}
Start "verb \textbf{bold" and more} done. \lxhookran
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("font=\"bold\""), "{xml}");
  assert!(!xml.contains("<verbatim"), "{xml}");
  assert!(xml.contains("ran"), "{xml}");
}

/// A listings example environment under `\DocInput` (forest-doc.sty:48
/// `forestexample`, `gobble=2`, `\lst@BeginAlsoWriteFile` + re-input): the
/// body's first line is read whole from column 0 so its `% ` survives like
/// lines 2+, and the write-file tee is gobbled like real listings', so the
/// re-input's `\end{forest}` is not commented out (forest-doc: 501
/// `readBalanced ran out of input` + Fatal; RUST-ONLY, pdflatex clean).
#[test]
fn forest_docinput_lstenv_writefile_gobbles_doc_percent() {
  let tex = r"\documentclass{ltxdoc}
\begin{filecontents*}{fdtx.dtx}
% \iffalse
% \fi
% \section{T}
% \begin{forestexample}
%   \begin{forest}
%     [VP[DP][V]]
%   \end{forest}
% \end{forestexample}
% \endinput
\end{filecontents*}
\usepackage[external]{forest}
\usepackage{forest-doc}
\begin{document}
\DocInput{fdtx.dtx}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(!stderr.contains("ran out of input"), "{stderr}");
  // Three binding gaps, pdflatex clean (SYNC_STATUS "forest binding gaps"): forest.sty:203
  // `\@escapeifif` and :8430 `\forest@file@copy` (forest-index.sty:407, 459, 477; RUST-ONLY, the
  // stub binding), lstmisc.sty:618 `\lst@InstallKeywords` (lstdoc.sty:178; SHARED with Perl).
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  for cs in [
    "\\lst@InstallKeywords",
    "\\@escapeifif",
    "\\forest@file@copy",
  ] {
    assert!(
      stderr.contains(&format!("Error:undefined:{cs}")),
      "{stderr}"
    );
  }
  assert!(xml.contains("<section"), "{xml}");
  // base64 of the gobbled first line "\begin{forest}" is what the listing
  // data starts with — line 1 lost neither its `\b` nor its indentation.
  assert!(xml.contains("<listing"), "{xml}");
}

/// polyglossia.sty:641 `\xpg_if_script:nTF` answers TRUE (there is no
/// OpenType font model to ask; a lualatex-clean document loaded a
/// script-capable font), so a non-Latin font switch no longer raises "The
/// current main roman font, cmr10, does not contain the Greek script!"
/// (fontsetup/fspsample ×2 11→0, greektonoi 16→0, latex-mr 98+Fatal→3).
#[test]
fn polyglossia_script_check_passes_without_font() {
  let tex = r"\documentclass{article}
\usepackage{polyglossia}
\setdefaultlanguage{english}
\setotherlanguage{greek}
\usepackage[default]{fontsetup}
\begin{document}
Hello \textgreek{ασδφ} world.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ασδφ"), "{xml}");
}

/// `\DocumentMetadata{tagging=on}` also loads the block and minipage
/// testphase modules (latex-lab-testphase-latest.sty:43,47): ltx-talk.cls
/// :1860 `\EditInstance{item}{basic}`, tagpdfdocu-patches.sty:127
/// `\DeclareInstance{blockenv}{docCommand}{display}` + `\UseInstance` +
/// `\endblockenv`, and :146 `\AssignSocketPlug{tagsupport/minipage/before}
/// {noop}` find their declarations (ltx-talk ×10, tagpdf manual 113 lines).
#[test]
fn testphase_tagging_sockets_and_block_templates_are_declared() {
  let tex = r"\DocumentMetadata{tagging = on}
\documentclass{article}
\ExplSyntaxOn
\EditInstance{item}{basic}{label-format = #1}
\DeclareInstance{blockenv}{docCommand}{display}{ name = docCommand, tag-name = Div, increment-level = false }
\AssignSocketPlug{tagsupport/minipage/before}{noop}
\ExplSyntaxOff
\begin{document}
A\UseTaggingSocket{minipage/before}B
\UseInstance{blockenv}{docCommand}{tag-name=Div,leftmargin=1pt,rightmargin=2pt}Hello\endblockenv
\begin{itemize}\item one\end{itemize}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("AB"), "{xml}");
  assert!(xml.contains("Hello"), "{xml}");
  assert!(xml.contains("<item "), "{xml}");
}

/// A `./`-prefixed name written by `\openout`/`\write` (fancyvrb
/// `{VerbatimOut}{./foo.tex}`) is read back by `\VerbatimInput{./foo.tex}`:
/// the VFS keys both sides without the `./` (xpicture-doc, checklistings;
/// RUST-ONLY).
#[test]
fn verbatimout_dotslash_round_trips_through_the_vfs() {
  let tex = r"\documentclass{article}
\usepackage{fancyvrb}
\begin{document}
\begin{VerbatimOut}{./foo.tex}
hello dotslash world
\end{VerbatimOut}
\VerbatimInput{./foo.tex}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("hello dotslash world"), "{xml}");
}

/// lthooks runs `top-level` chunks of `begindocument` AFTER package
/// chunks: pgfmanual-en-macros.tex:35's document-level
/// `\AtBeginDocument{\gdef|{\ifmmode…}}` must beat pgfmanual.pdflinks
/// .code.tex:413-416's `\let|=\pgfmanual@verb` (registered later, under
/// the `pgfmanual` label), so `\biggl|{r}\biggr|` in math is a fence, not
/// a verbatim collector opening a group inside the box (tikz-ext-manual:
/// ~950 of 1001 errors; SHARED). Follows from `\AtBeginDocument` joining
/// the L3 hook.
#[test]
fn pgfmanual_toplevel_atbegindocument_runs_last() {
  let tex = r"\documentclass[a4paper,doc2,landscape]{ltxdoc}
\usepackage{tikz}
\input{pgfmanual-en-macros}
\begin{document}
\[ \biggl| {r} \biggr| \]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("absolute-value"), "{xml}");
}

/// A standalone class's `\cs_new:Npn \thepage` (ltx-talk.cls:158) over the
/// pool's `\thepage` (article.cls material in real LaTeX) is quiet and
/// takes effect (ltx-talk ×10, 24 errors each; RUST-ONLY).
#[test]
fn l3_cs_new_over_a_pool_definition_is_quiet() {
  let tex = r"\begin{filecontents*}[overwrite]{poolc.cls}
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{poolc}[2026/01/01 pk-expl3]
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\ExplSyntaxOn
\cs_new:Npn \thepage { \@arabic \c@page }
\cs_new:Npn \figurename { Fig }
\ExplSyntaxOff
\normalsize
\end{filecontents*}
\documentclass{poolc}
\begin{document}
Page \thepage. \figurename.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Page 1. Fig."), "{xml}");
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// ltcmd's `\NewDocumentEnvironment{figure}` / `\NewDocumentCommand
/// \section` (ltx-talk.cls:1016-1033, :1574-1580) over the pool's
/// constructors: no error, and the pool's `<figure>`/`<section>` survive
/// (ltcmd keeps the existing definition after its check).
#[test]
fn ltcmd_declarators_keep_pool_constructors_quietly() {
  let tex = r"\begin{filecontents*}[overwrite]{poolc.cls}
\NeedsTeXFormat{LaTeX2e}\ProvidesClass{poolc}[2026/01/01 pk-expl3]
\renewcommand\normalsize{\fontsize{10pt}{12pt}\selectfont}
\ExplSyntaxOn
\NewDocumentEnvironment { figure } { } { } { }
\NewDocumentCommand \section { m } { }
\ExplSyntaxOff
\normalsize
\end{filecontents*}
\documentclass{poolc}
\begin{document}
\section{Hi}
\begin{figure}Body\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<section") && xml.contains("Hi</title>"),
    "{xml}"
  );
  assert!(xml.contains("<figure"), "{xml}");
}

/// The bindings' deferred begin-document code (cleveref_sty.rs `\let\label
/// \lx@cleverref@label`) runs AFTER the raw packages' `begindocument` hook
/// chunks, so raw cleveref.sty:66's `\def\label{\@ifnextchar[…}` does not
/// shadow it (its `[#1][#2]` scan ran to EOF: crossreftools_driver,
/// test-autonum fatal; RUST-ONLY).
#[test]
fn binding_begin_document_code_outranks_raw_hook() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\usepackage{cleveref}
\begin{document}
\begin{equation}a^2+b^2=c^2\label[section]{pyth}\end{equation}
See \cref{pyth}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert!(xml.contains("labels=\"LABEL:pyth\""), "{xml}");
}

/// `\NewTCBListing{E}{ O{} D<>{} }` / `{ !O{} !s }` / `{ !G{1} !O{} }`
/// (tutodoc.cls:1024, simplebnf-doc.tex:58, istgame-doc.tex:129): the
/// begin-line arguments the `\lstnewenvironment` arity cannot express are
/// absorbed, and the environment's own `\begin` line is never captured as
/// body (it re-entered the environment on `\input`-back without bound:
/// MemoryBudget fatal ×4, sweep #41; RUST-ONLY).
#[test]
fn tcb_listing_unmapped_begin_line_args_are_absorbed() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings,breakable}
\tcbset{listing engine=listings}
\NewTCBListing{mylst}{ O{} D<>{} }{ listing side text, #1 }
\NewTCBListing{example}{ !O{} !s }{ listing side text, #1 }
\DeclareTCBListing{doccode}{ !G{1} !O{} }{ listing only }
\begin{document}
\begin{mylst}<colback=red>
Some code line A
\end{mylst}
\begin{example}*
Some code line B
\end{example}
\begin{doccode}{colback=blue}
Some code line C
\end{doccode}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert_eq!(xml.matches("<listing ").count(), 3, "{xml}");
  assert!(!xml.contains("colback"), "{xml}");
  // `doccode` is `listing only`: its body is the base64 data, not text.
  assert!(
    xml.contains("Some code line A") && xml.contains("Some code line B"),
    "{xml}"
  );
  assert!(xml.contains("U29tZSBjb2RlIGxpbmUgQw=="), "{xml}");
}

/// `\mathitalicsmode` is a LuaTeX integer parameter (expl3-code.tex:996),
/// set by lualatex classes (homework, jwjournal).
#[test]
fn mathitalicsmode_is_a_register() {
  let tex = r"\documentclass{article}
\begin{document}
\mathitalicsmode=1 Mode \the\mathitalicsmode.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Mode 1."), "{xml}");
}

/// beamerbasetranslator.sty:14 loads translator: `\uselanguage` from a
/// language pack (ctex-scheme-chinese-beamer.def:71; mirage-beamer-zh).
#[test]
fn beamer_loads_translator() {
  let tex = r"\documentclass{beamer}
\uselanguage{English}\languagealias{en}{English}
\begin{document}
\begin{frame}Hi \translate{Theorem}\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hi"), "{xml}");
}

/// `\IfFontExistsTF` answers like luaotfload's case-insensitive database:
/// asmeconf.cls:650 asks for `TexGyreTermesX-regular.otf` (TeX Live ships
/// `TeXGyreTermesX-Regular.otf`), so the class must not take its
/// missing-font `\ClassErrorNoLine` branch (asmeconf/asmejour templates).
#[test]
fn font_exists_test_is_case_insensitive() {
  let tex = r"\documentclass{article}
\usepackage{fontspec}
\begin{document}
\IfFontExistsTF{TexGyreTermesX-regular.otf}{found}{missing}.
\IfFontExistsTF{NoSuchFontXyz.otf}{found}{missing}.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("found.") && xml.contains("missing."), "{xml}");
}

/// `\usepackage[authordate]{biblatex-chicago}` selects the author-date
/// family and chicago-dates-common.cbx:2966's `\gentextcite` renders as a
/// text cite (cms-dates-intro, cms-dates-sample; lualatex clean).
#[test]
fn biblatex_chicago_authordate_has_gentextcite() {
  let tex = r"\documentclass{article}
\usepackage[authordate,backend=biber]{biblatex-chicago}
\begin{document}
As \gentextcite{k1} shows; \Gentextcite{k1}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert!(xml.contains("<cite"), "{xml}");
}

/// A `#`-bearing `\AtBeginDocument` chunk registered under a package's own
/// label (pm-isomath.sty:150 `\providecommand\mathrmbf[1]{…}` and its
/// `\NewDocumentCommand…{…#2…}` blocks) takes the private store: lthooks'
/// labeled cleanup path is not yet reproduced by our gullet
/// (euclideangeometry-man: 100× `\csname g__hook_` errors + Fatal, sweep
/// #41). K3 correctness item; this guard pins the interim.
#[test]
fn hashful_begin_document_chunk_under_a_package_label() {
  let tex = r"\documentclass{article}
\begin{filecontents}[overwrite,noheader,nosearch]{lxhashpkg.sty}
\AtBeginDocument{\NewDocumentCommand\lxhashcmd{s m}{[#2]}\providecommand\lxhashplain[1]{(#1)}}
\end{filecontents}
\usepackage{lxhashpkg}
\begin{document}
\lxhashcmd{v} \lxhashplain{w}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("g__hook_"), "{stderr}");
  assert!(xml.contains("[v] (w)"), "{xml}");
}

/// The forest/diagrams discard stubs warn instead of erroring: the body is
/// discarded cleanly (forest-quickstart, fragoli_doc, milsymb; pdflatex clean).
#[test]
fn forest_stub_is_a_warning() {
  let tex = r"\documentclass{article}
\usepackage{forest}
\begin{document}
Before.
\begin{forest}
[VP [V [sees]] [NP [DP [the]] [NP [dog]]]]
\end{forest}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Before.") && xml.contains("After."), "{xml}");
}

/// forest.sty:1413-1655 (bracket reader), 8506-8515 (\NewDocumentEnvironment{forest}),
/// 8666-8680 (\Forest): parses the bracket grammar [label, options [child]...]
/// into a semantic tree of nested <ltx:inline-enumerate class="ltx_forest_children">
/// and <ltx:inline-item class="ltx_forest_node">.
#[test]
fn forest_three_level_semantic_tree() {
  let tex = r"\documentclass{article}
\usepackage{forest}
\begin{document}
\begin{forest}
[Root, for tree={draw}
  [Child1
    [Grandchild1]
    [$x^2$]
  ]
  [Child2
    [Grandchild3]
  ]
]
\end{forest}
\Forest(stages={foo}){ [TreeRoot [ChildLeaf]] }
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Root"), "Root node missing: {xml}");
  assert!(xml.contains("Child1"), "Child1 node missing: {xml}");
  assert!(xml.contains("Child2"), "Child2 node missing: {xml}");
  assert!(
    xml.contains("Grandchild1"),
    "Grandchild1 node missing: {xml}"
  );
  assert!(
    xml.contains("<Math"),
    "Math element for $x^2$ missing: {xml}"
  );
  assert!(
    xml.contains("Grandchild3"),
    "Grandchild3 node missing: {xml}"
  );
  assert!(
    xml.contains("TreeRoot"),
    "TreeRoot from \\Forest missing: {xml}"
  );
  assert!(
    xml.contains("ChildLeaf"),
    "ChildLeaf from \\Forest missing: {xml}"
  );
  assert!(
    xml.contains("<inline-enumerate class=\"ltx_forest\""),
    "inline forest list missing: {xml}"
  );
  // Sweep 63: a block wrapper was rejected inside text, cells and figures
  // (forest-doc 13→133, milsymb 0→1); every form's `ltx_forest_tree`
  // wrapper is an inline-block, as pdflatex draws the tree as an inline box
  // (forest.sty:8506-8514, P7).
  let tex = "\\documentclass{article}\n\\usepackage{forest}\n\\begin{document}\n\\fbox{\\begin{forest}[A[B][C]]\\end{forest}}\n\\begin{tabular}{c}\\begin{forest}[D[E]]\\end{forest}\\\\\\end{tabular}\n\\begin{figure}\\centering\\begin{forest}[F[G]]\\end{forest}\\caption{c}\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The `\fbox` frame lands on the wrapper, the cell holds the wrapper
  // directly, and `\centering` aligns it in the figure.
  assert!(
    xml.contains("<inline-block class=\"ltx_forest_tree\" framed=\"rectangle\">"),
    "fbox: {xml}"
  );
  assert!(
    xml.contains("<td align=\"center\"><inline-block class=\"ltx_forest_tree\">"),
    "cell: {xml}"
  );
  let fig = xml
    .find("<figure")
    .unwrap_or_else(|| panic!("figure: {xml}"));
  assert!(
    xml[fig..].contains("<inline-block align=\"center\" class=\"ltx_forest_tree\">"),
    "figure: {xml}"
  );
  if let Some(n) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(n, 0, "schema-invalid: {xml}");
  }
  for label in ["A", "B", "C", "D", "E", "F", "G"] {
    assert!(
      xml.contains(&format!("ltx_forest_node_content\">{label}<")),
      "{label}: {xml}"
    );
  }
  assert!(
    xml.contains("ltx_forest_children"),
    "ltx_forest_children missing: {xml}"
  );
  assert_eq!(
    xml.matches("ltx_forest_children").count(),
    3,
    "expected 3 child lists: {xml}"
  );
}

/// `{subeqnarray}` (subeqnarray.sty:33-41) is eqnarray with `\slabel`
/// subnumbers: `&` aligns, rows get `1a`/`1b` (subeqnarray-sample).
#[test]
fn subeqnarray_aligns_with_subnumbers() {
  let tex = r"\documentclass{article}
\usepackage{subeqnarray}
\begin{document}
\begin{subeqnarray}
\slabel{a} x & = & a \\
\slabel{b}   & = & b
\end{subeqnarray}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<equationgroup"), "{xml}");
  assert!(xml.contains("1a") && xml.contains("1b"), "{xml}");
}

/// `\blendcolors*{!60!white}` then `\textcolor{black!75}`: the blend is a
/// separate mix on the resolved color (gray .25 → .55 = #8C8C8C), not a
/// string-concatenated `black!75!60!white` (iodhbwm via ydoc-desc.sty:125).
#[test]
fn xcolor_blend_applies_after_the_local_mix() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\blendcolors*{!60!white}\textcolor{black!75}{hello}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("#8C8C8C"), "{xml}");
}

/// siunitx `per-mode=power` (its default) renders per-units as negative
/// exponents like our `reciprocal` (quantum-chemistry-bonn.sty:55).
#[test]
fn siunitx_per_mode_power_renders_reciprocal() {
  let tex = r"\documentclass{article}
\usepackage{siunitx}
\sisetup{per-mode=power}
\begin{document}
Energy: \qty{5}{\kilo\joule\per\mole}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("mol"), "{xml}");
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// `\documentclass[pdftex]` puts `pdfmode` in the backend request; naming
/// `dvips` at the `\document` backend load avoids expl3's "Backend request
/// inconsistent with engine" (elpres, scidoc), under both profiles.
#[test]
fn backend_load_names_the_dvi_backend() {
  let tex = r"\documentclass[pdftex]{article}
\usepackage{xcolor}
\begin{document}
Hello \textcolor{red}{world}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("world"), "{xml}");
  let (stderr, _) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// xpatch.sty:42 loads xparse, which restores ltcmd's legacy `g` argument
/// type (prtec.cls:316 `\NewDocumentCommand\entry{m g}`).
#[test]
fn xpatch_loads_xparse_for_legacy_arg_types() {
  let tex = r"\documentclass{article}
\usepackage{xpatch}
\NewDocumentCommand{\entry}{m g}{[#1/#2]}
\begin{document}
\entry{A}{B} \entry{C}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[A/B]"), "{xml}");
}

/// Under `\DocumentMetadata`, `\definecolor` also registers the color with
/// l3color (xcolor-patches-tmp-ltx.sty:56), so raw `\color_select:n`
/// (ltx-talk.cls:201) finds it.
#[test]
fn xcolor_definecolor_bridges_to_l3color_under_documentmetadata() {
  let tex = r"\DocumentMetadata{}
\documentclass{article}
\usepackage{xcolor}
\definecolor{alert}{RGB}{200,0,0}
\begin{document}
\ExplSyntaxOn \color_select:n {alert} \ExplSyntaxOff text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("text."), "{xml}");
}

/// K1 provenance: the l3/ltcmd leniency covers LaTeXML's OWN definitions
/// only — a genuine double declaration between two raw files (or in the
/// document) still reports "already defined", as pdflatex does.
#[test]
fn raw_double_declaration_still_errors() {
  let tex = r"\documentclass{article}
\begin{filecontents}[overwrite,noheader,nosearch]{lxdup.sty}
\ExplSyntaxOn
\cs_new:Npn \lxdupcmd { one }
\ExplSyntaxOff
\end{filecontents}
\usepackage{lxdup}
\ExplSyntaxOn
\cs_new:Npn \lxdupcmd { two }
\ExplSyntaxOff
\NewDocumentCommand\lxdupdoc{}{a}
\NewDocumentCommand\lxdupdoc{}{b}
\begin{document}
\lxdupcmd\ \lxdupdoc.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(stderr.contains("already defined"), "{stderr}");
  assert!(xml.contains("two a."), "{xml}");
}

/// Perl Expandable.pm:35: an unbalanced expansion is FATAL. jarticle.cls's
/// `\ds@tate` (ISO-2022-JP bytes whose `%` eats a brace) aborts in seconds
/// instead of proceeding into a 250 s loop (platexcheat; RUST-ONLY).
#[test]
fn unbalanced_expansion_is_fatal() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("jarticle.cls") {
    return;
  }
  // `\hour`/`\minute` are pLaTeX-format registers jarticle.cls:26-28 uses (the parked pTeX
  // format, §D9); declaring them leaves only the unbalanced expansion's error and its Fatal.
  let tex = r"\newcount\hour\newcount\minute
\documentclass[12pt,a4j,dvipdfmx]{jarticle}
\begin{document}
Hello
\end{document}
";
  let start = std::time::Instant::now();
  let (stderr, _xml) = convert(tex, true);
  assert!(stderr.contains("Fatal:Stomach:Misdefined"), "{stderr}");
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(start.elapsed().as_secs() < 60, "took {:?}", start.elapsed());
}

/// japanese-otf's ajmacros.sty (pTeX kanji token model, parked §D9) bails
/// with an explicit Fatal in under a second instead of an aperiodic
/// 250 s loop (platexsheet-jsclasses, wtref-ja, jpneduenumerate; SHARED).
#[test]
fn japanese_otf_kanji_scanners_bail_fast() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("otf.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{otf}
\begin{document}
Hello
\end{document}
";
  let start = std::time::Instant::now();
  let (stderr, _xml) = convert(tex, true);
  assert!(
    stderr.contains("Fatal:") && stderr.contains("ajmacros"),
    "{stderr}"
  );
  // Before the Fatal, the four undefined pTeX font declarations of the parked kanji model (§D9).
  assert_eq!(stderr.matches("Fatal:").count(), 1, "{stderr}");
  assert_eq!(error_count(&stderr), 5, "{stderr}");
  assert!(start.elapsed().as_secs() < 60, "took {:?}", start.elapsed());
}

/// lthooks is FIFO within a label: a raw `#`-bearing `\AtBeginDocument`
/// chunk registered first runs before a `#`-free one registered later
/// (alphabeta.sty then hep-math-font.sty; hep-paper-documentation fatal).
#[test]
fn raw_hashful_begin_document_chunk_keeps_fifo_order() {
  let tex = r"\documentclass{article}
\AtBeginDocument{\def\dummy#1{#1}\def\WHO{FIRST-param}}
\AtBeginDocument{\def\WHO{SECOND-plain}}
\begin{document}
Who: \WHO.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Who: SECOND-plain."), "{xml}");
}

/// `\NewTCBListing{egcite}{D(){teal} o m !o}{colframe=#1,…}` (oxyear-doc
/// .tex:216): the absorbed `D()` still owns `#1` (its default `teal`), so
/// the mandatory citation text never reaches `colframe`.
#[test]
fn tcb_listing_absorbed_specifiers_keep_positional_numbers() {
  let tex = r"\documentclass{article}
\usepackage[most]{tcolorbox}
\tcbuselibrary{listings}
\definecolor{teal}{rgb}{0,0.5,0.5}
\NewTCBListing{egcite}{D(){teal} o m !o}%
  {colframe = #1 ,colback = #1!5!white ,listing side text}
\begin{document}
\begin{egcite}{(Marx 1867), (Clarke, n.d.).}
Some text.
\end{egcite}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Can't find color"), "{stderr}");
  assert!(xml.contains("Some text."), "{xml}");
}

/// 300 tcolorbox pictures stay bounded under `--streaming --max-memory=800`
/// (fuse at 600 MB): each finished picture releases its node boxes, so a
/// 1,000-box manual no longer climbs to the fuse with a 39-byte XML
/// (glossaries-user, glossaries-extra-manual, datatool-user).
#[test]
fn tcolorbox_pictures_stay_memory_bounded() {
  let tex = r"\documentclass{article}
\usepackage[most]{tcolorbox}
\newtcolorbox{cb}{enhanced,breakable}
\newcount\ct \ct=0
\begin{document}
\loop\ifnum\ct<300
  \begin{cb}Sample code line \the\ct\end{cb}
  \advance\ct by 1
\repeat
\end{document}
";
  let (stderr, xml) = convert_args(tex, &["--streaming", "--max-memory=800"]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("MemoryBudget"), "{stderr}");
  assert_eq!(xml.matches("<picture").count(), 300, "{}", xml.len());
}

/// Perl `skipConditionalBody` (Core/Definition/Conditional.pm:127) reads the
/// CURRENT mouth only, so an `\ifX` left open by an `\input`ed preamble
/// file "falls off the end" of that file; Rust's skipper crossed into the
/// parent and devoured `\begin{document}` through EOF — a 39-byte output
/// with no root (thesis-sample; RUST-ONLY).
#[test]
fn conditional_skip_stops_at_the_input_file_boundary() {
  let tex = "\\documentclass{article}\n\\input{openif.tex}\n\\begin{document}\nBody survives.\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex, &[("openif.tex", "\\iffalse\n")]);
  assert!(stderr.contains("Error:expected:\\fi"), "{stderr}");
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains("<document"), "the root survives:\n{xml}");
  assert!(xml.contains("Body survives."), "{xml}");
}

/// A conversion whose document never opened its root (an undefined `\ifX`
/// in the main file skipped `\begin{document}` through EOF: xwatermark-guide,
/// skeyval-pokayoke2) wrote a bare XML declaration and reported status 2.
/// Empty output is a Fatal — the messages are the success signal.
#[test]
fn document_without_a_root_is_a_fatal() {
  let tex = "\\documentclass{article}\n\\ifdefTF\\relax{}{}\n\\begin{document}\nBody eaten.\n\\end{document}\n";
  // The CLI path (`Converter::convert`), which the corpus harness and
  // cortex_worker use; the editor's in-process fragment fallback is exempt.
  let (stderr, xml) = convert_args(tex, &[]);
  assert!(stderr.contains("Error:expected:\\fi"), "{stderr}");
  assert!(
    stderr.contains("Fatal:Document:Malformed"),
    "an output with no root element is a Fatal:\n{stderr}"
  );
  assert!(!xml.contains("<document"), "{xml}");
}

/// …but only a LaTeX document has a `\begin{document}` to miss. A source
/// that is all comments — arXiv's `%auto-ignore` withdrawal placeholders, 53
/// papers of cortex sandbox 2605 (e.g. 2605.00131) — loads no class and
/// lost nothing; Perl reports no problems, and so do we.
#[test]
fn comment_only_source_is_not_a_fatal() {
  let (stderr, xml) = convert_args("%auto-ignore", &[]);
  assert!(
    !stderr.contains("Fatal:") && !stderr.contains("Error:"),
    "a comment-only source is not a failure:\n{stderr}"
  );
  assert!(!xml.contains("<document"), "{xml}");
}

/// Streaming: the root's `xmlns:PREFIX` declarations were computed from the
/// RESIDENT DOM only (`apply_document_namespace_declarations`), so a prefix
/// used solely inside spilled segments — `xlink:href` on `svg:pattern`/
/// `svg:use` — was serialized unbound (six TikZ manuals flipped invalid in
/// s109 at the transition watermark: atableau, circuitikzmanual,
/// tikzlings-doc, tkz-grapheur-doc-en/-fr, tzplot-doc). Eager and Perl
/// declare it exactly when used.
#[test]
fn streaming_declares_namespaces_used_only_in_spilled_segments() {
  let mut tex = String::from(
    r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{patterns}
\newcommand\pic{\begin{center}\begin{tikzpicture}
\fill[pattern=north east lines] (0,0) rectangle (2,2);
\end{tikzpicture}\end{center}}
\begin{document}
This is a prose paragraph with real text content that stays resident.

",
  );
  for _ in 0..20 {
    tex.push_str("\\pic ");
  }
  tex.push_str("\n\\end{document}\n");
  let (stderr, xml) = convert_args(&tex, &["--streaming", "--max-memory=800"]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Warning:"), "{stderr}");
  assert!(xml.matches("xlink:href=").count() >= 20, "{}", xml.len());
  let root_end = xml
    .find("<document")
    .map(|i| i + xml[i..].find('>').unwrap())
    .expect("root");
  assert!(
    xml[..root_end].contains(r#"xmlns:xlink="http://www.w3.org/1999/xlink""#),
    "the root declares xlink:\n{}",
    &xml[..root_end]
  );
}

/// Streaming pass 1: an inline centered picture closes under
/// `ltx:para > ltx:p > ltx:text > ltx:picture` (no `\par` inside the
/// paragraph). `spill_prose_free_children` kept every such `ltx:p` whole
/// because its prose test matched the `ltx:p` itself, so nothing under a
/// closed root `ltx:para` spilled and each picture's digested box tree
/// stayed pinned in `node_boxes` (pgf-spectra LSE: 163,636 entries, ~4.4 GB
/// at the 6 GB fuse). A text-less `ltx:p` is never a leading-title
/// candidate and spills whole.
#[test]
fn inline_pictures_in_paragraphs_stay_memory_bounded() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\newcount\ct \ct=0
\newcount\dr
\begin{document}
A real leading paragraph of prose.

\loop\ifnum\ct<INLINE_N
  \noindent\makebox[\linewidth][c]{\begin{tikzpicture}
    \dr=0
    \loop\ifnum\dr<INLINE_D
      \draw[color=red!\the\dr!blue] (0,\the\dr pt) -- (2,\the\dr pt);
      \advance\dr by 1
    \repeat
  \end{tikzpicture}}\par
  \advance\ct by 1
\repeat
\end{document}
"
  .replace("INLINE_N", "300")
  .replace("INLINE_D", "40");
  let (stderr, xml) = convert_args(&tex, &["--streaming", "--max-memory=800"]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("MemoryBudget"), "{stderr}");
  assert_eq!(xml.matches("<picture").count(), 300, "{}", xml.len());
  assert!(
    xml.contains("A real leading paragraph of prose."),
    "{}",
    xml.len()
  );
}

/// Under the luatex profile `\DeclareUnicodeCharacter` declares nothing
/// (latex.ltx:22168/22203 — utf8.def is 8-bit-engine only), so a class's
/// `\cs_new_protected:Npn ·` finds the native character free
/// (einfart.cls:838-839; homework-demo-cn/-jp/-tc, jwjournal-demo-cn).
/// Repro `unicode-catcodes/declareunicodechar_middot_luatex_einfart.tex`.
#[test]
fn unicode_engine_keeps_middle_dot_native() {
  let tex = "\\documentclass{article}
\\ExplSyntaxOn
\\char_set_catcode_active:n { `\\· }
\\cs_new_protected:Npn · { \\ensuremath\\cdot }
\\ExplSyntaxOff
\\begin{document}
Middle dot active: $a·b$.
\\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("already defined"), "{stderr}");
  assert!(xml.contains("\u{22c5}") || xml.contains("\\cdot"), "{xml}");
  // The 8-bit profile still activates the LICR mapping.
  let tex8 = "\\documentclass{article}
\\begin{document}
Middle dot: ·.
\\end{document}
";
  let (stderr, xml) = convert(tex8, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Middle dot: ·."), "{xml}");
}

/// expl3-code.tex:985-986 alias `\tex_luatexversion:D`/`\tex_luatexrevision:D`
/// at format time; the luatex profile re-derives them from its own
/// `\luatexversion` (lua-widow-control.sty:153 compares `\tex_luatexversion:D`).
#[test]
fn luatex_profile_aliases_expl3_version_primitives() {
  let tex = "\\documentclass{article}
\\ExplSyntaxOn
\\int_compare:nNnTF { \\tex_luatexversion:D } > { 200 } { \\def\\x{NEW} } { \\def\\x{OLD-\\tex_luatexversion:D} }
\\ExplSyntaxOff
\\begin{document}
Version: \\x.
\\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Version: OLD-121."), "{xml}");
}

/// dhucs's Unicode-native branch (dhucs.sty:44 `\ifx가가` true) skips the
/// `\if@hangul` block that defines `\pdfstringdefPreHook` (:117) and
/// `\dhucs@emph@raise`, which memhangul-ucs.sty:509/:451 then read; the
/// overlay supplies them, and hyperref keeps an existing hook (Perl
/// hyperref.sty.ltxml:413). Repro
/// `loader/dhucs_native_pdfstringdefprehook_istgame.tex`.
#[test]
fn dhucs_native_branch_defines_pdfstringdefprehook() {
  let tex = r"\documentclass{article}
\usepackage{dhucs}
\makeatletter
\g@addto@macro\pdfstringdefPreHook{\def\lxprobe{kept}}
\makeatother
\usepackage{hyperref}
\begin{document}
\makeatletter\pdfstringdefPreHook
ok \lxprobe\ \the\dhucs@emph@raise\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ok kept 0.0pt"), "{xml}");
}

/// The begin-document backend loader follows expl3.ltx:130: skip when a
/// backend was already chosen, auto-select in PDF output, `dvips` in DVI.
/// Repros `backend-persona/{pdfoutput_inconsistent,backend_already_set}.tex`.
#[test]
fn backend_load_follows_pdfoutput_and_prior_choice() {
  let pdf = r"\documentclass[11pt]{article}
\ifx\pdfoutput\undefined\else
  \pdfoutput=1
\fi
\begin{document}
Hello world.
\end{document}
";
  let (stderr, xml) = convert(pdf, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hello world."), "{xml}");
  let set = r"\documentclass{article}
\pdfoutput=1
\makeatletter\ExplSyntaxOn
\sys_load_backend:n {pdftex}
\ExplSyntaxOff\makeatother
\begin{document}
Hello.
\end{document}
";
  let (stderr, xml) = convert(set, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hello."), "{xml}");
}

/// biblatex.sty:12862 `\citefield`, :3649 `\mkcomprange`, and the
/// :4371-4377 page-string family (`\pno`, `\psqq`) at document level
/// (oxref manuals, biblatex-german-legal, biblatex-true-citepages-omit).
/// Repro `index-bib/blx_toplevel_pagehelpers_oxref.tex`.
#[test]
fn biblatex_field_cites_and_page_strings() {
  let tex = r"\documentclass{article}
\usepackage[style=authoryear,backend=biber]{biblatex}
\begin{document}
Alpha \citefield{smith}{labelalpha}, range \mkcomprange{367-368}, first \mkfirstpage{367--368}.
See \cite[\pno~110]{smith}; also \cite[295 \psqq]{jones}.
Title \citefield{smith}{title}, editors \citename{smith}{editor}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("range 367-368, first 367."), "{xml}");
  assert!(
    xml.contains("p.\u{a0}110") || xml.contains("p.~110"),
    "{xml}"
  );
  assert!(xml.contains("sqq.</cite>"), "{xml}");
  assert!(xml.contains("show=\"Title\""), "{xml}");
  assert!(xml.contains("class=\"ltx_citemacro_citename\""), "{xml}");
}

/// `\usepackage[style = abnt]{biblatex}` with spaces around `=`
/// (biblatex-abnt.tex:53) still selects the style, so `abnt.cbx` loads and
/// its `\apud` exists.
#[test]
fn biblatex_style_option_tolerates_spaces() {
  let tex = r"\documentclass{article}
\usepackage[style = abnt, backend = biber]{biblatex}
\begin{document}
\makeatletter\typeout{[\meaning\apud]}\makeatother
Text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("[undefined]"), "{stderr}");
  assert!(xml.contains("Text."), "{xml}");
}

/// newpxmath (uantwerpenexam-example2 `\square`) and MnSymbol (univie-ling
/// `\blacktriangleright`, atableau `\bigcircle`) carry the AMS symbol set.
#[test]
fn font_symbol_packages_carry_amssymb() {
  for (pkg, sym) in [
    ("newpxmath", "\\square"),
    ("MnSymbol", "\\blacktriangleright"),
    ("MnSymbol", "\\bigcircle"),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage{{{pkg}}}\n\\begin{{document}}\n$a {sym} b$\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert(&tex, true);
    assert_eq!(error_count(&stderr), 0, "{pkg} {sym}: {stderr}");
    assert!(!xml.contains("<ERROR"), "{pkg} {sym}: {xml}");
  }
}

/// mdframed.sty:591 `\newmdtheoremenv` defines a theorem environment
/// (beautynote).
#[test]
fn mdframed_theorem_environments_are_theorems() {
  let tex = r"\documentclass{article}
\usepackage{amsthm}
\usepackage{mdframed}
\newmdtheoremenv[linewidth=1pt]{theorem}{Theorem}[section]
\newmdtheoremenv{lemma}[theorem]{Lemma}
\begin{document}
\section{One}
\begin{theorem}Thm body.\end{theorem}
\begin{lemma}Lemma body.\end{lemma}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<theorem") && xml.contains("Lemma body."),
    "{xml}"
  );
}

/// thm-restate.sty:191 `restatable*` (proof-at-the-end demo) and
/// lineno.sty:2881 `bframe` (ulineno).
#[test]
fn restatable_star_and_lineno_bframe() {
  let tex = r"\documentclass{article}
\usepackage{amsthm}
\usepackage{thm-restate}
\usepackage{lineno}
\newtheorem{theorem}{Theorem}
\begin{document}
\begin{restatable*}[Main]{theorem}{mainthm}Restated body.\end{restatable*}
\begin{bframe}Framed text.\end{bframe}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Restated body.") && xml.contains("Framed text."),
    "{xml}"
  );
}

/// subeqn.sty:51 `subeqnarray` (subeqn-sample).
#[test]
fn subeqn_subeqnarray_environment() {
  let tex = r"\documentclass{article}
\usepackage{subeqn}
\begin{document}
\begin{subeqnarray}\label{main}
a &=& b \\
c &=& d
\end{subeqnarray}
Eq.~\ref{main}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<equationgroup"), "{xml}");
}

/// unicode-math-table.tex rows become math symbols (derivative `\coloneq`,
/// rec-thy `\nvrightarrow`/`\mathhyphen`, shtthesis `\oiint`), while a
/// kernel-defined name keeps its own definition.
#[test]
fn unicode_math_symbol_table_defines_names() {
  let tex = r"\documentclass{article}
\usepackage{unicode-math}
\removenolimits{\sum}
\begin{document}
$a \coloneq b \nvrightarrow c \mathhyphen d \oiint_S f \le g$
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("\u{2254}") && xml.contains("\u{21f8}") && xml.contains("\u{222f}"),
    "{xml}"
  );
  assert!(xml.contains("less-than-or-equals"), "{xml}");
}

/// `\DeclareMathOperator` stores its body unexpanded like TeX: iidef.sty:147
/// names `\mathds` with dsfont unloaded and never uses the operator (ithw).
#[test]
fn declaremathoperator_body_stays_lazy() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\DeclareMathOperator{\one}{\mathds{1}}
\DeclareMathOperator{\Tr}{{\rm Tr}}
\begin{document}
$\Tr A$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // Perl's XMath, the body presented through `\operatorname` (57ab; amsopn.sty.ltxml:25).
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMApp><XMTok name="Tr" role="OPFUNCTION" scriptpos="post">Tr</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok></XMApp></XMath>"#,
  );
}

/// biblatex-chicago's `notes` style (the default) loads chicago-notes
/// bbx/cbx, whose `\DeclareCiteCommand`s define `\runcite` and
/// `\headlessfullcite` (cms-legal-sample, cms-notes-sample); internals a
/// document reaches directly (`\blx@opt@loccittracker@false`,
/// biblatex-sbl-ibid.tex:200; `\blx@refpatch@sect`, cmsendnotes.sty:121)
/// are consumed. Repros `index-bib/blx_chicago_cbx_citecmd_undefined.tex`,
/// `blx_loccittracker_internal_sbl.tex`, `blx_refpatch_sect_cmsendnotes.tex`.
#[test]
fn biblatex_chicago_notes_loads_its_cbx() {
  let tex = r"\documentclass{article}
\usepackage[notes,backend=biber]{biblatex-chicago}
\begin{document}
Text.\footnote{See \runcite{smith}; \headlessfullcite{jones}.} \Citetitle{smith}.
\makeatletter\blx@opt@loccittracker@false\blx@refpatch@sect{section}{}{1}\makeatother
\printshorthands
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<note"), "{xml}");
}

/// biblatex-cv.sty is raw-input on top of the binding, so its own
/// `\highlightname` (biblatex-cv.sty:565) exists. Repro
/// `index-bib/blx_variant_own_macro_cv.tex`.
#[test]
fn biblatex_cv_variant_overlay() {
  let tex = r"\documentclass{article}
\usepackage{biblatex-cv}
\highlightname{Doe}{Jon}{}{}
\begin{document}
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Body."), "{xml}");
}

/// tex.web §1131: `$` inside a group opened within math (`$\bm{\hat{m}$} b`,
/// kblocks-doc.tex:207) closes the group first; the math ends when the
/// bounded argument group does, and stays nested in the paragraph.
/// Repro `boxes-groups/mal_math_bm_group_close.tex`.
#[test]
fn math_end_inside_open_group_defers_to_group_end() {
  let tex = r"\documentclass{article}
\usepackage{bm}
\begin{document}
a $\bm{\hat{m}$} b

c $\bm{x}$ d
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<Math ").count(), 2, "{xml}");
  assert!(xml.contains("</Math> b"), "{xml}");
  assert!(!xml.contains("</p>\n<Math"), "{xml}");
}

/// A `\NewTCBListing` option value built from a substituted argument keeps
/// its control-word boundaries (`\dots ii` stayed `\dotsii`; oxnotes-doc).
#[test]
fn tcb_listing_option_tokens_keep_cs_boundaries() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\NewTCBListing{egcite}{m}{listing side text,before lower={#1\par}}
\begin{document}
\begin{egcite}{\dots ii (Brussels, 1867--88), 367--8}
\cite[367--368]{key}
\end{egcite}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("\u{2026} ii (Brussels") || xml.contains("\u{2026}ii (Brussels"),
    "{xml}"
  );
}

/// beamerfontthememetropolis.sty:278-308 `\patchcmd`s `\beamer@subsection`
/// and `\beamer@@frametitle`; the binding carries both bodies (never
/// invoked) so the patches apply. Repro `loader/beamer_metropolis_min.tex`.
#[test]
fn beamer_metropolis_font_theme_patches_apply() {
  let tex = r"\documentclass[10pt]{beamer}
\usetheme{metropolis}
\begin{document}
\section{S}
\subsection{Sub}
\begin{frame}{Title}x\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Patching"), "{stderr}");
  assert!(xml.contains("<subsection"), "{xml}");
}

/// aguplus.cls:524 probes `\@ifundefined{chapter}` right after its
/// `\LoadClass{article}`; the kernel `\chapter` is retracted at `\LoadClass`
/// return, not only at `\documentclass`. Control: book keeps chapters.
/// Repro `loader/aguplus_figcaps.tex`.
#[test]
fn loadclass_return_retracts_kernel_chapter() {
  let tex = r"\documentclass[twoside,agupp]{aguplus}
\begin{document}
ok
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ok"), "{xml}");
  let book = r"\documentclass{book}
\begin{document}
\chapter{One}
Text.
\end{document}
";
  let (stderr, xml) = convert(book, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<chapter"), "{xml}");
}

/// Perl Mouth.pm:98-117: an `at_letter` mouth saves and restores `@`'s
/// catcode LOCALLY, so a package that `\input`s a file inside
/// `\bgroup\catcode`\@0 … \egroup` (CoverPage.sty:60-70) gets `@` back as a
/// letter after the group. Repro
/// `macro-state/atletter_group_input_catcode_leak_coverpage.tex`.
#[test]
fn at_letter_mouth_keeps_group_catcode_undo() {
  let sty = r"\NeedsTeXFormat{LaTeX2e}
\ProvidesPackage{lxcatleak}
\bgroup
  \catcode`\@0
  \bgroup
    \def\article##1{\xdef\CP@ParseArg{##1}}%
    \input{lxcatleak.txt}%
  \egroup
\egroup
\define@key{cover}{title}{\gdef\CP@Title{#1}}
\endinput
";
  let txt = "@article{k,\n title = {Some Title}}\n";
  let tex = r"\documentclass{article}
\usepackage{keyval}
\usepackage{lxcatleak}
\begin{document}
\makeatletter\setkeys{cover}{title=T}\CP@Title\makeatother
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("lxcatleak.sty", sty), ("lxcatleak.txt", txt)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">T<") || xml.contains("T</p>"), "{xml}");
}

/// datetime.sty:181-188 `\newdateformat{name}{format}` defines `\name`
/// (chetdoc `\mydate`); jmlr.cls:593 `\abovestrut` (pmlr-sample); ejpecp.cls:156
/// `\BEMAIL` (sample).
#[test]
fn class_and_datetime_definitions_exist() {
  let tex = r"\documentclass{article}
\usepackage{datetime}
\newdateformat{mydate}{\THEYEAR-\THEMONTH-\THEDAY}
\begin{document}
\mydate Date: \formatdate{5}{9}{2026}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Date: 2026-9-5."), "{xml}");
  let jmlr = r"\documentclass{jmlr}
\title{T}\author{\Name{A}\Email{a@b}}
\begin{document}
\maketitle
\begin{tabular}{c}\abovestrut{2ex}x\belowstrut{1ex}\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(jmlr, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
}

/// Internals raw packages/documents reach that the replacing bindings
/// omitted (one witness manual each): lastpage `\lastpage@lastpage`,
/// geometry `\Gm@lmargin`, amsmath `\tag@true`, hyperref `\HyPsd@AMSclassfix`
/// and the dvips `\pdfmark`, colortbl `\therownum`, beamer's
/// `\pgfpagesuselayout`, siunitx v3 `\siunitx_number_format:nN`, fourier's
/// `\lefthand`.
#[test]
fn binding_internals_reached_by_raw_code() {
  let tex = r"\documentclass{article}
\usepackage{lastpage}
\usepackage{geometry}
\usepackage{amsmath}
\usepackage{hyperref}
\usepackage{colortbl}
\usepackage{siunitx}
\usepackage{fourier}
\makeatletter
\tag@true
\pdfmark[/ANN]{pdfmark=/OBJ,Raw={/_objdef {x} /type /stream}}
\HyPsd@AMSclassfix
\ExplSyntaxOn
\siunitx_number_format:nN {12.50} \l_tmpa_tl
\tl_set_eq:NN \lxnum \l_tmpa_tl
\ExplSyntaxOff
\begin{document}
Last \lastpage@lastpage; margin \the\Gm@lmargin; row \therownum; number \lxnum; hand \lefthand.
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Last ??; margin 0.0pt; row 0; number 12.50; hand"),
    "{xml}"
  );
  let beamer = r"\documentclass{beamer}
\pgfpagesuselayout{2 on 1}[a4paper]
\begin{document}
\begin{frame}x\end{frame}
\end{document}
";
  let (stderr, _) = convert(beamer, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// listings stores an undefined-yet colour instead of digesting it
/// (callouts: `\lstset{backgroundcolor=\color{…}}` before xcolor loads), and
/// its character-conversion internals exist for add-on styles
/// (lstfiracode `\lst@CCPutMacro`). Repros
/// `graphics-tikz/{callouts_lstset_color_eager,listings_CCPutMacro}.tex`.
#[test]
fn listings_deferred_colour_and_conversion_internals() {
  let tex = r#"\documentclass{article}
\usepackage{listings}
\lstset{backgroundcolor=\color{cyan!10}}
\usepackage{xcolor}
\makeatletter
\lst@CCPutMacro\lst@ProcessOther {"2D}{\lst@ttfamily{-{}}{-{}}}\@empty\z@\@empty
\makeatother
\begin{document}
\begin{lstlisting}
x = 1
\end{lstlisting}
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("data=\"eCA9IDE=\""), "{xml}"); // base64 of `x = 1`
}

/// `\only<handout>{…}` is discarded in presentation mode (beamerswitch.cls:
/// 226 runs `\pgfpagesuselayout` with pgfpages unloaded there); overlay
/// specs and beamer-mode specs still apply. seminar.cls:760 probes
/// `\ps@fancy` (semsamp1/2). Repro `beamer-stubs/beamer_only_modespec.tex`.
#[test]
fn beamer_only_discards_other_mode_specs() {
  let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}
\only<handout>{\undefinedhandoutonly}
\only<handout:0| trans:0>{\undefinedhandoutonly}
\only<2->{Overlay.}
\only<beamer>{Beamer.}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Overlay.") && xml.contains("Beamer."), "{xml}");
  let fancy = r"\documentclass{article}
\usepackage{fancyhdr}
\pagestyle{fancy}
\makeatletter\ifx\ps@fancy\@undefined MISSING\fi\makeatother
\begin{document}
Text.
\end{document}
";
  let (stderr, xml) = convert(fancy, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("MISSING"), "{xml}");
}

/// verbatim.sty:210-217: `\verbatiminput` of a file that does not exist
/// is `\typeout{No file …}`, not an error (msc.tex:287, lnosuppl.tex:89).
/// Repro `parameter-conditional/verbatiminput_missing_msc.tex`.
#[test]
fn verbatiminput_missing_file_is_not_an_error() {
  let tex = r"\documentclass{article}
\usepackage{verbatim}
\begin{document}
Before.
\verbatiminput{COPYRIGHT}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("No file COPYRIGHT"), "{stderr}");
  assert!(
    xml.contains("Before.") && xml.contains("After.") && !xml.contains("<ERROR"),
    "{xml}"
  );
}

/// lua-widow-control's user surface under the luatex profile (its Lua half
/// cannot run: homework-demo-*, jwjournal-demo-cn, abntexto).
#[test]
fn lua_widow_control_surface() {
  let tex = r"\documentclass{article}
\usepackage{lua-widow-control}
\lwcsetup{emergencystretch=1em, draft=false}
\begin{document}
\iflwc on\else off\fi; \lwcdisable\iflwc on\else off\fi.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("on; off."), "{xml}");
}

/// pict2e.sty:791 `\cbezier` (halloweenmath-man) and ejpecp.cls:467
/// `\realmathbb` (ejpecp sample).
#[test]
fn pict2e_cbezier_cubic() {
  let tex = r"\documentclass{article}
\usepackage{pict2e}
\begin{document}
\setlength{\unitlength}{1pt}
\begin{picture}(40,20)
\cbezier(0,0)(10,20)(30,20)(40,0)
\end{picture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // four point pairs → the post-processor's SVG `C` segment
  let points = xml
    .split("<bezier points=\"")
    .nth(1)
    .and_then(|r| r.split('"').next())
    .unwrap_or("");
  assert_eq!(points.split(' ').count(), 4, "{xml}");
  let ej = r"\documentclass{ejpecp}
\title{T}\author{A}
\begin{document}
\maketitle
$\realmathbb{R}$
\end{document}
";
  let (stderr, xml) = convert(ej, true);
  assert!(!stderr.contains("undefined:\\realmathbb"), "{stderr}");
  assert!(
    xml.contains("\u{211d}") || xml.contains("mathbb") || xml.contains("R<"),
    "{xml}"
  );
}

/// `\pgfmathparse{\l_x_dim}` reads the expl3 register as one name; the
/// alphabetic-only scanner split it at `_` and read `\l` (pgf-interference:
/// 200k warnings, 412 s). Repro `expl3/pgfmath_expl3_register_split.tex`.
#[test]
fn pgfmath_reads_expl3_register_names() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\ExplSyntaxOn
\dim_new:N \l_x_dim
\dim_set:Nn \l_x_dim { 3cm }
\NewDocumentCommand \showit {} { \pgfmathparse { \l_x_dim } RESULT=[\pgfmathresult] }
\ExplSyntaxOff
\begin{document}
\showit
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("is not a register"), "{stderr}");
  assert!(xml.contains("RESULT=[85.35826]"), "{xml}");
}

/// A deferred math end (#196) fires once at its group's end and never
/// re-defers: nicefrac's text-mode denominator `\nicefrac{1}{2$^{x}$}` puts
/// the inner `$` two groups below the math frame (egpeirce-doc.tex:1831);
/// the ender must not escape past the math frame and leak `<ltx:Math>`.
/// Repro `boxes-groups/math_defer_nicefrac_dollar_leak.tex`.
#[test]
fn deferred_math_end_never_escapes_the_math_frame() {
  let tex = r"\documentclass{article}
\usepackage{nicefrac}
\begin{document}
X \nicefrac{1}{2$^{\textrm{16}}$} Y

Z $a$ W
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert!(!stderr.contains("malformed"), "{stderr}");
  // Both errors are the binding's math-mode parts (pdflatex: 0; PERL-ORIGIN, RED
  // boxes-groups/nicefrac_text_argument_is_text), the path the deferred end needs.
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(xml.contains(" W</p>") || xml.contains(" W\n"), "{xml}");
  assert!(!xml.contains("</p>\n<Math"), "{xml}");
}

/// latex.ltx keeps one FIFO `\@begindocumenthook`: a `#`-bearing raw
/// `\AtBeginDocument` chunk registered AFTER a `#`-free one runs after it
/// (the italian.ldf/verifica.cls shape), and one registered BEFORE runs
/// before (the hep-paper shape, guarded separately). Repro
/// `macro-state/begindocument_hook_fifo.tex`.
#[test]
fn begin_document_hooks_run_in_registration_order() {
  let tex = r"\documentclass{article}
\makeatletter
\def\lxorder{}
\AtBeginDocument{\g@addto@macro\lxorder{A}}
\AtBeginDocument{\newcommand\lxhashed[1]{#1}\g@addto@macro\lxorder{B}}
\AtBeginDocument{\g@addto@macro\lxorder{C}}
\makeatother
\begin{document}
Order: \lxorder; \lxhashed{ok}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Order: ABC; ok."), "{xml}");
}

/// Sweep-44 singles: fontspec's `\latinencoding` (textalpha-doc), CJK.sty's
/// `\CJKspace`/`\CJKencfamily` (cjk-ko-doc, bxcjkjatype beamer), beamer
/// `\subject` (shipunov), pdfmanagement `\pdfmanagement_add:nee` under
/// `\DocumentMetadata` (zugferd), MnSymbol `\rcurvearrowse` (biblatex-apa6),
/// biblatex `\printorigdate` (cms-notes-sample).
#[test]
fn sweep44_single_name_gaps() {
  let luatex = r"\documentclass{article}
\usepackage{fontspec}
\usepackage{MnSymbol}
\begin{document}
Enc: \latinencoding; $a \rcurvearrowse b$.
\end{document}
";
  let (stderr, xml) = convert_with(luatex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Enc: TU;") && xml.contains("\u{21b7}"),
    "{xml}"
  );
  let cjk = r"\documentclass{article}
\usepackage{CJK}
\begin{document}
\CJKspace\CJKencfamily[UTF8]{mj}{}\CJKnospace Text.
\end{document}
";
  let (stderr, xml) = convert(cjk, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Text."), "{xml}");
  let beamer = r"\documentclass{beamer}
\subject{S}
\begin{document}
\begin{frame}x\end{frame}
\end{document}
";
  let (stderr, _) = convert(beamer, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let meta = r"\DocumentMetadata{}
\documentclass{article}
\ExplSyntaxOn
\pdfmanagement_add:nee {Catalog/AF}{}{x}
\pdfmanagement_add:nnx {Catalog}{AF}{\pdf_object_ref:n{zugferd/rechnung}}
\ExplSyntaxOff
\begin{document}
Meta.
\end{document}
";
  let (stderr, xml) = convert(meta, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Meta."), "{xml}");
}

/// Package state: newtxtext keeps its xpatch dependency so ltcmd's legacy
/// `g` type exists for `\NewDocumentCommand\entry{m g}` (prtec.cls:316);
/// psfrag allocates its `\newwrite\pfg@temp` (psfrag.sty:151) so psfragx's
/// read/write streams do not collide; verbatim's terminator reaches the
/// current `\end` macro (knowledge's scope areas). Repros
/// `macro-state/{newtxtext_drops_xpatch_xparse_g_prtec,psfrag_missing_newwrite_pfx_already_exists,knowledge_scope_verbatim_no_pop}.tex`.
#[test]
fn package_state_prtec_psfragx_knowledge() {
  let prtec = r"\documentclass{article}
\usepackage{newtxtext}
\NewDocumentCommand{\entry}{m g}{[#1/\IfNoValueTF{#2}{NO}{#2}]}
\begin{document}
\entry{A} \entry{B}{C}
\end{document}
";
  let (stderr, xml) = convert(prtec, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[A/NO]") && xml.contains("[B/C]"), "{xml}");
  let psfrag = r"\documentclass{article}
\usepackage{psfrag}
\usepackage{psfragx}
\begin{document}
\copypfxfromto{article.cls}{out.pfx}
Done.
\end{document}
";
  let (stderr, xml) = convert(psfrag, true);
  assert!(!stderr.contains("already exists"), "{stderr}");
  assert!(xml.contains("Done."), "{xml}");
  let knowledge = r"\documentclass{article}
\usepackage[scope,silent]{knowledge}
\begin{document}
Before.
\begin{verbatim}
x = 1
\end{verbatim}
After.
\end{document}
";
  let (stderr, xml) = convert(knowledge, true);
  assert!(!stderr.contains("Not allowed to close"), "{stderr}");
  assert!(xml.contains("<verbatim") && xml.contains("After."), "{xml}");
  // Control: the kernel `{verbatim}` hands its terminator to the CURRENT
  // `\end` (latex.ltx:15438 `\@xverbatim`), so a hooked `\end` sees it
  // once, after the verbatim, and the rest of the `\end` line still reads.
  let hooked = r"\documentclass{article}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{verbatim}
x = 1
\end{verbatim} tail
\begin{center}c\end{center}
\end{document}
";
  let (stderr, xml) = convert(hooked, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("[E:verbatim]").count(), 1, "{xml}");
  assert!(xml.contains("[E:center]") && xml.contains("tail"), "{xml}");
  let vpos = xml.find("</verbatim>").unwrap();
  assert!(xml[vpos..].contains("[E:verbatim]"), "{xml}");
}

/// Control for the Gemini G3 frame-body `#`-halving (DIVERGENCES #198): a
/// lone `#1` inside a non-fragile frame stays `#1` (leniency: real beamer
/// rejects it), `##1` and `####1` both reach `\newcommand` as `#1`, a
/// `[fragile]` frame is not halved, and a frame after the frame is unaffected.
#[test]
fn beamer_frame_single_hash_control() {
  let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}{One}
\newcommand\ha[1]{(a:#1)}\ha{x}
\newcommand\hb[1]{(b:##1)}\hb{y}
\newcommand\hc[1]{(c:####1)}\hc{z}
\end{frame}
\begin{frame}[fragile]{Two}
\newcommand\hd[1]{(d:#1)}\hd{w}
\end{frame}
\begin{frame}{Three}
Plain text.
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for s in ["(a:x)", "(b:y)", "(c:z)", "(d:w)", "Plain text."] {
    assert!(xml.contains(s), "missing {s}: {xml}");
  }
}

/// listings stores its length keys as macros (lstmisc.sty:1193
/// `\def\lst@numbersep{#1}`), so a value naming a macro defined only later
/// is fine; Perl's Dimension-typed key evaluated it at `\lstset` time
/// (abntexto-uece.tex:402/406, SHARED; pdflatex clean).
#[test]
fn listings_length_keys_are_lazy() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\lstset{numbers=left,numbersep=\dimexpr-5pt+\addnumbersep\relax,xleftmargin=\addnumbersep}
\def\addnumbersep{9pt}
\begin{document}
\begin{lstlisting}
x = 1
\end{lstlisting}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<listing") && xml.contains("After."), "{xml}");
}

/// LuaTeX `\csstring` (manual §2.8.3): `\string` without the escape
/// character; control: `\string` keeps it. Witness abntexto-uece.
#[test]
fn luatex_csstring_primitive() {
  let tex = r"\documentclass{article}
\edef\bslash{\csstring\\}
\begin{document}
[\csstring\foo][\bslash][\string\foo][\csstring a]
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The luatex profile's text encoding is TU (batch 56ja), so a catcode-12 `\`
  // prints as itself, as lualatex prints it.
  assert!(xml.contains("<p>[foo][\\][\\foo][a]</p>"), "{xml}");
}

/// A wrapper environment whose end code produces `\end{frame}` by expansion
/// (beamerthemeTorinoTh.sty:97 `tframe`) must still terminate the frame-body
/// collection at its own `\end{tframe}`; the following `[fragile]` frame's
/// `\verb` then reads raw characters. Witness beamer2thesis (4→83 in sweep 45).
#[test]
fn beamer_wrapper_frame_environment_terminates() {
  let tex = r"\documentclass{beamer}
\newenvironment{tframe}{\begin{frame}[t]}{\end{frame}}
\begin{document}
\begin{tframe}{General}
\begin{itemize}
\item All guides show options
\end{itemize}
\end{tframe}
\begin{frame}[t,fragile]{Config}
\begin{itemize}
\item It is the first thing
\item \verb!hello!
\end{itemize}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("All guides show options") && xml.contains("hello"),
    "{xml}"
  );
  assert_eq!(xml.matches("<subsection").count(), 2, "{xml}");
}

/// An `mdframed` box (here cnltx-example's `{example}`) inside a low-level
/// `\begin{list}` item must not close the outer list: items after it stay
/// siblings (schulmathematik 1→40 in sweep 45; pdflatex clean).
#[test]
fn mdframed_inside_low_level_list_keeps_items() {
  let tex = r"\documentclass{article}
\usepackage{cnltx-example}
\NewDocumentEnvironment {Liste} { }
  {\begin{list}{ }{\setlength{\leftmargin}{1em}}}{\end{list}}
\NewDocumentCommand \Desc {m}{\item \texttt{#1}\newline}
\begin{document}
\subsubsection*{Test}
\begin{Liste}
\Desc{Kosy}
Some description text.
\begin{example}
  \textbf{hello world}
\end{example}
\Desc{LGS}
\Desc{third}
\end{Liste}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The frame keeps cnltx's stray inner list (item i2) INSIDE itself; the
  // outer list's later items stay its children (i3, i4), never a section's.
  assert!(
    xml.contains(r#"<item xml:id="S0.I1.i3">"#) && xml.contains(r#"<item xml:id="S0.I1.i4">"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<subsubsection").count(), 1, "{xml}");
  let fb = xml.find("<logical-block").unwrap();
  let fe = xml.find("</logical-block>").unwrap();
  assert!(xml[fb..fe].contains("hello world"), "{xml}");
  assert!(
    xml[fe..].contains(r#"<item xml:id="S0.I1.i3">"#) && xml.contains("third"),
    "{xml}"
  );
}

/// A deferred math ender (#196) that fires with another REAL TeX group on
/// top re-defers to that group's end (tex.web §1131 off_save closes real
/// groups); titlecaps.sty:107 nests `\titlecap` in `\bgroup…\egroup` and
/// re-emits `$` two groups below the math frame (titlecaps 3→10, sweep 45).
/// Control: the nicefrac constructor-frame shape stays as #196 left it
/// (`deferred_math_end_never_escapes_the_math_frame`).
#[test]
fn deferred_math_end_walks_real_groups() {
  let tex = r"\documentclass{article}
\usepackage{titlecaps}
\def\bs{$\backslash$}
\begin{document}
\titlecap{\ttfamily \bs a\{b \{c\} d\}. \texttt{\bs x}}

After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("malformed"), "{stderr}");
  // pdflatex: "\a{B {C} D}.  \x", then "After." — both typewriter, since
  // `\titlecap` does not group its `\ttfamily` (`\f@family` is cmtt after it).
  // With `\ttfamily` robust (latex.ltx:13993) its `\protected@edef` keeps the
  // switch; as a plain macro it stored `\edef cmr{cmtt}`: "mtt\" and an upright
  // "After." (W17).
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    concat!(
      r#"<para xml:id="p1"><p><Math mode="inline" tex="\backslash" text="backslash" xml:id="p1.m1">"#,
      r#"<XMath><XMTok name="backslash" role="MULOP">\</XMTok></XMath></Math>"#,
      r#"<text font="typewriter">a{B {C} D}. "#,
      r#"<Math mode="inline" tex="\backslash" text="backslash" xml:id="p1.m2"><XMath>"#,
      r#"<XMTok font="serif" name="backslash" role="MULOP">\</XMTok></XMath></Math>x</text></p></para>"#
    ),
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p><text font="typewriter">After.</text></p></para>"#,
  );
}

/// Sweep-45 single-name gaps: cprotect's `\icprotect` (cprotect.sty:133;
/// LaTeX_RefSheet), newtxtext's xstring/ifthen/scalefnt requires
/// (newtxtext.sty:22; heria `\IfEq`), oup's `\ORCID` (cls:2733), beamer's
/// `\resetcounteronoverlays` (beamerbaseframe.sty:181; beamer2thesis).
#[test]
fn sweep45_single_name_gaps() {
  let tex = r"\documentclass{article}
\usepackage{cprotect}
\usepackage{newtxtext}
\begin{document}
\icprotect\textbf{hello \verb|world|}
[\IfEq{a}{a}{Y}{N}\IfEq{a}{b}{Y}{N}]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("world") && xml.contains("[YN]"), "{xml}");
  let oup = r"\documentclass{oup-authoring-template}
\begin{document}
\title{T}
\author{A \ORCID{0000-0001-2345-6789}}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert(oup, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("https://orcid.org/0000-0001-2345-6789"),
    "{xml}"
  );
  let beamer = r"\documentclass{beamer}
\resetcounteronoverlays{equation}
\begin{document}
\begin{frame}Reset ok.\end{frame}
\end{document}
";
  let (stderr, xml) = convert(beamer, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Reset ok."), "{xml}");
}

/// A `\DeclareMathOperator` body naming a protected `\NewDocumentCommand` (pm-isomath.sty:185,
/// `\DeclareMathOperator\eu{\MathLatin{e}(n)}`) is not expanded at definition — since 57ae the body
/// is Perl's token presentation (amsopn.sty.ltxml:25), expanded at use; a definition-time expansion
/// ran away (euclideangeometry-man 2→101 in sweep 45). Control: a body naming a defined macro
/// (`\newcommand\trname{tr}`). Both as Perl's output.
#[test]
fn declaremathoperator_keeps_protected_macros() {
  let tex = r"\documentclass{article}
\usepackage{pm-isomath}
\begin{document}
Text $\eu{3}$.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "XMApp",
    &[],
    r#"<XMApp><XMText name="eu" role="OPFUNCTION" scriptpos="post">e</XMText><XMTok meaning="3" role="NUMBER">3</XMTok></XMApp>"#,
  );
  let control = r"\documentclass{article}
\usepackage{amsmath}
\newcommand\trname{tr}
\DeclareMathOperator\tr{\trname}
\begin{document}
$\tr A$
\end{document}
";
  let (stderr, xml) = convert(control, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math content-tex="\tr A" mode="inline" tex="\operatorname{tr}A" text="tr@(A)" xml:id="p1.m1"><XMath><XMApp><XMTok name="tr" role="OPFUNCTION" scriptpos="post">tr</XMTok><XMTok font="italic" role="UNKNOWN">A</XMTok></XMApp></XMath></Math>"#,
  );
}

/// A `!O{}` LEADING optional of a tcolorbox listing environment reaches the
/// box options (`[listing only]` stops the body from executing, so the
/// `\errmessage` in it never runs), while a `[`-line inside the body stays
/// content (xparse `!`: no space skipping; istgame-doc). keytheorems-doc,
/// wordle ×2, simplebnf-doc flipped dirty in sweep 46 with a dropping eater.
#[test]
fn tcb_bang_leading_optional_reaches_options() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\NewTCBListing{ex}{ !O{} }{colback=red!5,#1}
\begin{document}
\begin{ex}[listing only]
\errmessage{executed}
\end{ex}
\begin{ex}
  [
    not an option
  ]
\end{ex}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("After.") && xml.matches("<listing class").count() == 2,
    "{xml}"
  );
}

/// amsmath.sty:211-216 `\@saveprimitive\over\@@over` …: a document that
/// restores the primitive (`\let\over=\@@over`, abntexto.tex:187) keeps a
/// working `\over`. Perl's binding omits the block (SHARED; lualatex clean).
#[test]
fn amsmath_saves_fraction_primitives() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\makeatletter \let\over=\@@over \let\atop=\@@atop \makeatother
\begin{document}
$1\over2$ and $a\atop b$.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("divide") || xml.contains("XMApp"), "{xml}");
}

/// tex.web §262 `print_cs`: `\meaning` prints a delimiter control word
/// with a trailing space (`macro:#1\foo #2->Q`); expkv-cs's aggregate keys
/// parse `\meaning` output delimited on `<space>#` (expkv-cs.tex:996-1001;
/// expkv-bundle 13 errors, Perl omits the space too).
#[test]
fn meaning_prints_delimiter_control_words_with_a_space() {
  let tex = r"\documentclass{article}
\begin{document}
\def\x#1\foo#2{Q}\def\y a\bar{R}
[\meaning\x][\meaning\y]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // OT1 text renders `\` as U+201C and `>` as U+00BF
  assert!(xml.contains("macro:#1\u{201C}foo #2-\u{BF}Q"), "{xml}");
  assert!(xml.contains("macro:a\u{201C}bar -\u{BF}R"), "{xml}");
  // expkv-cs aggregate keys parse `\meaning` output delimited on `<space>#`
  // (the manual's enverb-executed example, pkg-cs.tex:495-515).
  let agg = r"\documentclass{article}
\usepackage{xcolor}\usepackage{enverb}\usepackage[all]{expkv}\usepackage{expkv-cs}
\makeatletter
\begin{document}
\def\enverbBody{\ekvcSplit\foo{k-internal=0}{X}\ekvcSecondaryKeys\foo{aggregate k = {k-internal}{#1,#2}}\foo{k=1}}
\enverbExecute
\end{document}
";
  let (stderr, xml) = convert(agg, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("X"), "{xml}");
}

/// `\ensuremath{}` with an EMPTY argument in a text box inside math is
/// `$\relax$` (latex.ltx:15807), never an adjacent `$$` display shift
/// (polynom's Horner scheme: an empty p-column cell; polydemo 29→3).
#[test]
fn ensuremath_empty_argument_in_text_box() {
  let tex = r"\documentclass{article}
\usepackage{array}
\makeatletter
\begin{document}
\@tempdima=20pt
\[\leavevmode\hbox{$\vcenter{\offinterlineskip
  \halign{\hfil\ensuremath{##}&&\@startpbox\@tempdima\hfil\ensuremath{##}\@endpbox\cr
    a&\cr}}$}\]
Text \ensuremath{x+1} and \ensuremath{}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<equation") && xml.contains("<tabular"),
    "{xml}"
  );
  assert!(xml.contains(r#"tex="x+1""#), "{xml}");
}

/// Sweep-46 single-name gaps: ulem's `\UL@protected` (ulem.sty:46),
/// amsmath's `\std@minus`/`\overarrow@` internals (amsmath.sty:949/983),
/// ejpecp's supplement block (ejpecp.cls:350-366), stix2's AMS names
/// (stix2.sty:1316), and a missing `\include` file as a note (latex.ltx:9730).
#[test]
fn sweep46_single_name_gaps() {
  let tex = r"\documentclass{article}
\usepackage{ulem}
\usepackage{amsmath}
\usepackage{stix2}
\makeatletter
\let\x\UL@protected
\def\pfill{\rightarrowfill@}
\makeatother
\begin{document}
$a \nmid b$, $\twoheadrightarrowtail$, \makeatletter$\std@minus$ $\overarrow@\pfill\displaystyle{ab}$\makeatother.
\include{no-such-chapter}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("\u{2224}") && xml.contains("\u{2916}") && xml.contains("After."),
    "{xml}"
  );
  let ejp = r"\documentclass{ejpecp}
\begin{document}
\title{T}\author{A}\maketitle
\begin{supplement}
\stitle{Extra proofs}
\sdescription{The long proofs.}
\end{supplement}
Cite \MR{1234567 (2007e:60001)} and \ARXIV{2011.04706}.
\end{document}
";
  let (stderr, xml) = convert(ejp, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Supplementary Material") && xml.contains("Extra proofs"),
    "{xml}"
  );
  assert!(
    xml.contains("mr=1234567") && xml.contains("arXiv:2011.04706"),
    "{xml}"
  );
  // amsrefs titles keep their control-sequence names' case
  let refs = r"\documentclass{article}
\usepackage{amsrefs}
\begin{document}
Cite \cite{k}.
\begin{bibdiv}\begin{biblist}
\bib{k}{book}{author={A. B.}, title={Using \LaTeX{} and \LaTeXe{} well}, date={2020}}
\end{biblist}\end{bibdiv}
\end{document}
";
  let (stderr, xml) = convert(refs, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("well"), "{xml}");
}

/// `escapechar={}` clears an inherited `\lstset{escapechar=*}` (newpax's
/// `\lstinputlisting[escapechar={}]` executed a `*…*` span as LaTeX).
#[test]
fn listings_escapechar_empty_clears() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\lstset{escapechar=*}
\begin{document}
\begin{lstlisting}[escapechar={}]
* \typeout{never run} \undefinedcs *
\end{lstlisting}
\begin{lstlisting}
a *\textbf{bold}* b
\end{lstlisting}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("undefinedcs") && xml.contains("bold"), "{xml}");
  assert!(
    !xml.contains("never run") || xml.contains("typeout"),
    "{xml}"
  );
}

/// `escapeinside={}{}` clears an inherited escape too: `escapechar` and
/// `escapeinside` set the one `\lst@DefEsc` (lstmisc.sty:336-347).
/// codeanatomy.lstlisting's `\inputlisting` ran the file's `!…!` spans.
#[test]
fn listings_escapeinside_empty_clears() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/listings_escapeinside_empty_clears.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">alpha</text><text class="ltx_lst_space"> </text>!\<text class="ltx_lst_identifier">textbf</text>{<text class="ltx_lst_identifier">ESCAPED</text>}!<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">omega</text></listingline>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">fill</text>=<text class="ltx_lst_identifier">red</text>!50]<text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">tail</text></listingline>"#),
    "{xml}"
  );
}

/// `\inputminted` of an extensionless `filecontents` file: `find_file`
/// resolves `snip` to the virtual store's `snip.tex`, read from the store
/// (tutodoc-en/fr `\tdoclatexinput`, 8 empty listings).
#[test]
fn inputminted_reads_extensionless_vfs_file() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment/inputminted_extensionless_vfs_file.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">hello</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">world</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">listing</text><text class="ltx_lst_space"> </text><text class="ltx_lst_identifier">content</text></listingline>"#),
    "{xml}"
  );
}

/// `subequations` fixes the parent number under `\protected@edef`'s regime
/// (amsmath.sty:1134): a robust `\loop` with local `\edef`s inside
/// `\theequation` runs at digestion, not in the gullet (hep-paper's
/// oldstyle `\tstyle`; hep-math-documentation 39 errors). Control: the
/// `{\rm S}\arabic{equation}` shape (witness 2005.06712) keeps `S1a`/`S1b`.
#[test]
fn subequations_keep_robust_number_commands() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\makeatletter
\def\mfirst#1#2\@nil{#1}
\def\mrest#1#2\@nil{#2}
\newif\ifmytake
\DeclareRobustCommand{\myloopnum}{%
  \edef\mya{2j}\mytaketrue
  \loop
    \edef\myn{\expandafter\mfirst\mya\@nil}%
    \edef\mya{\expandafter\mrest\mya\@nil}%
    \ifx\mya\@empty \edef\myq{\myn}\mytakefalse \fi
  \ifmytake \repeat
}
\renewcommand{\theequation}{\myloopnum\arabic{equation}}
\makeatother
\begin{document}
\begin{subequations}
\begin{align}
a &= b \\
c &= d
\end{align}
\end{subequations}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<equationgroup").count(), 2, "{xml}");
  let control = r"\documentclass{article}
\usepackage{amsmath}
\renewcommand{\theequation}{{\rm S}\arabic{equation}}
\begin{document}
\begin{subequations}
\begin{align}
a &= b \\
c &= d
\end{align}
\end{subequations}
\end{document}
";
  let (stderr, xml) = convert(control, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("S1a") && xml.contains("S1b"), "{xml}");
}

/// eTeX `\iffontchar` (etex_man §3.7) answers from the font FILE: a
/// `\font`-declared TFM's populated `char_info` slots (cmr10 = 128 OT1
/// slots). Perl leaves the conditional undefined; the former stub said TRUE
/// for every slot.
#[test]
fn iffontchar_reads_tfm_coverage() {
  let tex = "\\documentclass{article}\n\\font\\x=cmr10 \n\\begin{document}\n\\x A:\\iffontchar\\x`A yes\\else no\\fi; 200:\\iffontchar\\x 200 yes\\else no\\fi; cur:\\iffontchar\\font`A yes\\else no\\fi/\\iffontchar\\font 200 yes\\else no\\fi.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("A:yes; 200:no; cur:yes/no."), "{xml}");
}

/// unicodefonttable's `\displayfonttable` walks every code point of the
/// range and keeps a cell only where `\iffontchar\font` is true; with real
/// OpenType `cmap` coverage the control blocks U+0000–001F and U+0080–009F
/// of Latin Modern Sans vanish (the samples manual emitted 262k cells and
/// hit the memory fuse). Self-skips without the font.
#[test]
fn iffontchar_bounds_unicodefonttable_to_font_coverage() {
  // Guard on the ENGINE's own resolver (font_index), not PATH kpsewhich: on a
  // Debian-split host lmsans10 lives in /usr/share/texmf, which kpsewhich sees
  // but font_index (TEXMFDIST+TEXMFLOCAL) does not — so `\iffontchar` coverage
  // is empty and the row bound this test asserts does not apply.
  if latexml_core::common::font::coverage::font_file_path("lmsans10-regular.otf").is_none()
    || !kpsewhich_has("unicodefonttable.sty")
  {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{fontspec}\n\\setmainfont{Latin Modern Sans}\n\\usepackage{unicodefonttable}\n\\begin{document}\n\\displayfonttable[range-start=0000,range-end=00FF]{Latin Modern Sans}\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let cells = xml.matches("<td").count();
  assert!(cells > 150 && cells < 260, "cells={cells}\n{xml}");
  assert!(
    !xml.contains("U+0000"),
    "control block row must be skipped:\n{xml}"
  );
  assert!(
    !xml.contains("U+0080"),
    "C1 block row must be skipped:\n{xml}"
  );
  assert!(xml.contains("U+0040") || xml.contains("U+0041"), "{xml}");
}

/// fontenc.sty un-marks itself loaded at its end, so a second
/// `\usepackage[<encs>]{fontenc}` loads again and inputs the new
/// encodings' .def files (montex: `\MyTogrog` from lmcenc.def).
#[test]
fn fontenc_reloads_with_new_encodings() {
  if !kpsewhich_has("lmcenc.def") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\usepackage[LMC,T1]{fontenc}\n\\makeatletter\n\\begin{document}\n\\typeout{FE:\\@ifpackageloaded{fontenc}{loaded}{unloaded}}\nTogrog: \\MyTogrog\\ done.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Option clash"), "{stderr}");
  assert!(stderr.contains("FE:unloaded"), "{stderr}");
  assert!(xml.contains("done."), "{xml}");
}

/// fontenc.sty:81-83 inputs `<enc>enc.def` only while `\T@<enc>` is
/// undefined, and the format preloads T1: `ș` stays a character (Perl too),
/// not t1enc.dfu's `\textcommabelow s` re-armed at document time. natbib
/// splits a bare label unexpanded (`\NAT@bare`, KNOWN_PERL_ERRORS #252), and
/// `\textcommabelow` is the combining comma below (DIVERGENCES #291). Before:
/// "Roșca" was a two-row tabular and both labels a PushbackLimit Fatal (arXiv
/// 2605.20924 / 2605.08338).
#[test]
fn fontenc_keeps_preloaded_encoding() {
  let tex = "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{natbib}\n\\begin{document}\nM[\\meaning ș] Roșca, \\textcommabelow{S}tefan and Țurcanu.\n\\begin{thebibliography}{99}\n\\bibitem[{Roșca et~al.(2024)Roșca and Li}]{k1} First.\n\\bibitem[{Mari\\textcommabelow{s} et~al.(2020)Mari\\textcommabelow{s}, Li, and Wu}]{k2} Second.\n\\end{thebibliography}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>M[the character ș] Roșca, Ștefan and Țurcanu.</p>"),
    "{xml}"
  );
  for tag in [
    "<tag role=\"authors\">Roșca et\u{a0}al.</tag>",
    "<tag role=\"fullauthors\">Roșca and Li</tag>",
    "<tag role=\"year\">2024</tag>",
    "<tag role=\"authors\">Mariș et\u{a0}al.</tag>",
    "<tag role=\"fullauthors\">Mariș, Li, and Wu</tag>",
    "<tag role=\"year\">2020</tag>",
  ] {
    assert!(xml.contains(tag), "missing {tag}:\n{xml}");
  }
}

/// `\psset` is pst-xkey's family-aware `\setkeys+[psset]`; a no-op lost the
/// key BODIES that define pst-node's `\psk@mnodesize` & co. (psmatrix under
/// pstricks-add: dsptricks 101 errors).
#[test]
fn psset_dispatches_family_key_bodies() {
  if !kpsewhich_has("pstricks-add.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{pstricks-add}\n\\begin{document}\n\\begin{psmatrix}\n  A & B \\\\\n  C & D\n\\end{psmatrix}\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  // pst-node's psmatrix internals now exist; the remaining psmatrix
  // `\halign` group cascade and pstricks-add's colour-key internals
  // (`\pst@getcolor` = xcolor's `\XC@getcolor`) are separate roots.
  assert!(!stderr.contains("psk@mnodesize"), "{stderr}");
  assert!(!stderr.contains("Error:undefined:\\psk@mnode"), "{stderr}");
  assert!(!stderr.contains("Error:undefined:\\psk@mcol"), "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  // Control: the idiomatic colour default through the real `\psset` runs
  // pstricks' `\pst@getcolor` over color's `\color@<name>` storage.
  let tex = "\\documentclass{article}\n\\usepackage{pstricks}\n\\psset{linewidth=2pt,linecolor=red}\n\\begin{document}\nok \\psframebox{boxed}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // (`\psframebox`'s content is dropped by the DVI-only binding — a
  // separate, pre-existing gap; the control is the colour key running clean.)
  assert!(xml.contains("ok"), "{xml}");
}

/// iftex.sty:272-291: LuaTeX always answers `\ifpdf` TRUE (PDF output
/// mode); tikzrput.sty defines `\rput` only inside that branch.
#[test]
fn ifpdf_is_true_under_the_luatex_profile() {
  let tex = "\\documentclass{article}\n\\usepackage{iftex}\n\\begin{document}\n\\ifpdf PDF\\else DVI\\fi\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">PDF<") || xml.contains("PDF\n"), "{xml}");
  // The pdfTeX persona answers from `\pdfoutput` (the K6 ruling,
  // 2026-09-24): PDF by default, DVI when the document selects it.
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("PDF"), "{xml}");
  let dvi = tex.replace("\\documentclass", "\\pdfoutput=0\n\\documentclass");
  let (stderr, xml) = convert(&dvi, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("DVI"), "{xml}");
}

/// listings.sty:2315-2316 inputs `listings.cfg` AND the user's
/// `lstlocal.cfg` (labyrinth ships `\pkgname` there).
#[test]
fn listings_reads_lstlocal_cfg() {
  let tex = "\\documentclass{article}\n\\usepackage{listings}\n\\begin{document}\nThe \\pkgname{labyrinth} package.\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex, &[(
    "lstlocal.cfg",
    "\\newcommand{\\pkgname}[1]{{\\normalfont\\textsf{#1}}}\n",
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("font=\"sansserif\"") && xml.contains("labyrinth"),
    "{xml}"
  );
}

/// Single-name gaps of sweep #47: lineno.sty:2214 `\firstlinenumber`,
/// l3draw.sty:1821 `\l_draw_default_linewidth_dim`, libertine.sty:456
/// `\biolinumLF` — each binding replaced the raw file without the name.
#[test]
fn sweep47_single_name_gaps() {
  let tex = "\\documentclass{article}\n\\usepackage{lineno}\n\\usepackage{l3draw}\n\\usepackage{libertine}\n\\ExplSyntaxOn\n\\dim_compare:nNnTF { \\l_draw_default_linewidth_dim } > { 0pt } { \\def\\lw{POS} } { \\def\\lw{ZERO} }\n\\ExplSyntaxOff\n\\begin{document}\n\\firstlinenumber{1}%\nlw:\\lw; {\\biolinumLF bio}.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("lw:POS;"), "{xml}");
  assert!(xml.contains("bio"), "{xml}");
}

/// Under `[utf8]{inputenc}` a decoded Latin-1 code point stays catcode 12
/// even after t1enc.dfu's `\DeclareUnicodeCharacter{00E1}` (utf8.def's
/// invariant); bibarts's raw-byte UTF-8 lead detector must not match the
/// document's `á`. Latin-1 input keeps the byte active.
#[test]
fn utf8_input_keeps_latin1_code_points_other() {
  let tex = "\\documentclass{article}\n\\usepackage[utf8]{inputenc}\n\\usepackage[T1]{fontenc}\n\\begin{document}\n\\typeout{CC:\\the\\catcode`á:\\the\\catcode`α}\ncaf\\'e café\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("CC:12:"), "{stderr}");
  assert!(xml.contains("café café"), "{xml}");
  if kpsewhich_has("bibarts.sty") {
    let tex = "\\documentclass[12pt,a4paper]{article}\n\\usepackage{bibarts}\\bacaptionsgerman\n\\usepackage{ngerman}\n\\usepackage[utf8]{inputenc}\n\\usepackage[T1]{fontenc}\n\\begin{document}\n\\textsc{\\hy á}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("á"), "{xml}");
  }
}

/// latex.ltx:15347/15362/15388/15391: `\begin`/`\end` fire the kernel's
/// `env/NAME/{before,begin,end,after}` hooks (lthooks store, real under the
/// raw-loaded latexml.sty) around both `\newenvironment` and
/// DefEnvironment-managed environments; functional.sty's
/// `\AddToHook{env/demohigh/before}{\MyDeleteShortVerb}` never ran.
#[test]
fn kernel_env_hooks_fire_around_environments() {
  let tex = "\\documentclass{article}\n\\newenvironment{foo}{[I}{J]}\n\\AddToHook{env/foo/before}{B}\n\\AddToHook{env/foo/begin}{G}\n\\AddToHook{env/foo/end}{E}\n\\AddToHook{env/foo/after}{A}\n\\AddToHook{env/quote/before}{QB}\n\\AddToHook{env/quote/begin}{QG}\n\\AddToHook{env/quote/end}{QE}\n\\AddToHook{env/quote/after}{QA}\n\\begin{document}\nx\\begin{foo}body\\end{foo}y\n\n\\begin{quote}qbody\\end{quote}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // latex.ltx:15362: `env/X/begin` runs before `\csname X\endcsname`.
  assert!(xml.contains("xBG[IbodyEJ]Ay"), "{xml}");
  // A DefEnvironment-managed env: `begin` before the constructor opens its
  // element (like `\quote` starting its list), `end` inside, `after` outside.
  assert!(xml.contains("QBQG"), "{xml}");
  assert!(xml.contains("qbodyQE"), "{xml}");
  let quote_end = xml.find("</quote>").expect("quote element");
  assert!(xml[quote_end..].contains("QA"), "{xml}");
  // The default (non-raw) profile keeps its no-op hooks: no errors, no marks.
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[Ibody"), "{xml}");
}

/// A `\caption` inside a `\parbox` whose insert context is inline
/// (`\rotatebox`) floats out to the enclosing figure (`insert_block`'s
/// float-out predicate keyed on the inline candidate set), instead of a
/// hard `ltx:block` rejecting it (heria-proposal, rubik).
#[test]
fn caption_in_inline_parbox_floats_to_figure() {
  if !kpsewhich_has("tcolorbox.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{graphicx}\n\\usepackage[raster]{tcolorbox}\n\\begin{document}\n\\begin{figure}[hbt]\n\\rotatebox{90}{\\parbox{10cm}{%\n\\begin{tcbraster}[raster columns=2]\n\\begin{tcolorbox}[title=A]x\\end{tcolorbox}\n\\begin{tcolorbox}[title=B]y\\end{tcolorbox}\n\\end{tcbraster}\n\\caption{Cap}\\label{fig:x}\n}}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let fig = xml.find("<figure").expect("figure");
  let fig_end = xml[fig..]
    .find("</figure>")
    .map(|i| fig + i)
    .expect("figure end");
  let body = &xml[fig..fig_end];
  assert_eq!(body.matches("<caption").count(), 1, "{xml}");
  assert!(
    !xml.contains("<block>\n      <caption") && !xml.contains("<block><caption"),
    "{xml}"
  );
}

/// A `\parbox` body that leaves a conditional open (jourcl.cls:145) must
/// not be digested inside the wrapper's own `\ifx` dispatch.
#[test]
fn parbox_body_dangling_conditional_is_not_the_wrappers() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\ifempty#1{\\def\\temp{#1} \\ifx\\temp\\empty }\n\\def\\RP#1{ \\ifempty{#1} \\else \\sbox0{#1}\\ifdim\\wd0=0pt {} \\else \\ifdim0pt=\\dimexpr\\ht0+\\dp0\\relax {} \\else {N:#1} \\fi \\fi }\n\\makeatother\n\\begin{document}\n\\parbox{3cm}{ \\RP{Reviewer} }\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("N:Reviewer"), "{xml}");
}

/// latex.ltx:18330 `\flushbottom` is a robust macro (the dump carries it);
/// scrlttr2.cls:5053 `\g@addto@macro`s it, which a primitive clobber turned
/// into a self-expanding loop.
#[test]
fn flushbottom_stays_the_kernel_macro() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\g@addto@macro\\flushbottom{\\relax}\n\\makeatother\n\\begin{document}\n\\flushbottom OK\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("OK"), "{xml}");
}

/// A trailing `\multicolumn` row with no `\\` before `\end{tabular}` must
/// still close the alignment (the DefEnvironment `env/tabular/end` hook
/// digests nothing when the hook has no code — latex.ltx:15386's
/// `\IfHookEmptyTF` guard).
#[test]
fn trailing_multicolumn_row_closes_the_alignment() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{tabular}{cc}\nc & d\\\\\n\\multicolumn{2}{l}{Total: 5}\n\\end{tabular}\n\\end{document}\n";
  for raw in [false, true] {
    let (stderr, xml) = convert(tex, raw);
    assert_eq!(error_count(&stderr), 0, "raw={raw}: {stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "raw={raw}: {xml}");
  }
}

/// An environment's closing tag closes only what its own replacement
/// opened; content the element could not hold (a tcolorbox directly in a
/// `{picture}`: pagelayout, xebaposter) auto-closed it already.
#[test]
fn environment_close_after_content_autoclose_is_not_an_error() {
  if !kpsewhich_has("tcolorbox.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{tcolorbox}\n\\begin{document}\n\\setlength{\\unitlength}{1pt}\n\\begin{picture}(200,200)\n\\begin{tcolorbox}A box directly in a picture.\\par Second para.\\end{tcolorbox}\n\\end{picture}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<picture") && xml.contains("Second para."),
    "{xml}"
  );
}

/// The real `\psset` runs pst key bodies through pstricks.tex's own helpers
/// (`\pst@getangle{#1}\psk@angleA`); a stub that ate the value left
/// `\psk@angleA` undefined (sweep #48 flips: hexgame, xcolor2, seminar).
#[test]
fn psset_angle_keys_run_the_raw_helpers() {
  if !kpsewhich_has("pst-coil.sty") || !kpsewhich_has("pst-node.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{pstricks,pst-node,pst-coil}\n\\psset{angleA=45,arcangleA=10,coilaspect=30,linewidth=1.5pt}\n\\makeatletter\n\\begin{document}\nA:\\psk@angleA;B:\\psk@arcangleA;C:\\psk@coilaspect.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // pstricks keeps angles in its PostScript form (`\pst@@getangle` →
  // "45. "): the raw helper ran, the target macros exist.
  assert!(xml.contains("A:45. ;B:10. ;C:30. ."), "{xml}");
}

/// `\inst` is provided around author content only (`\lx@author@withinst`)
/// with its frontmatter meaning — an affiliation-link request, or, for a
/// footnote-SYMBOL mark like `\inst{*}`, the kept glyph — so a class that
/// defines `\inst` only inside the title-box scope where `\@author` expands
/// (bfhsciposter.cls:445,476) still converts (witness
/// bfh-ci/DEMO-BFHSciPoster; Perl: `undefined:\inst`), and the class
/// binding's own linking `\inst` (llncs) still wins.
#[test]
fn author_inst_is_an_affiliation_link_request() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\@maketitle{\\begingroup\\def\\inst##1{\\textsuperscript{##1}}\\@author\\par\\endgroup}\n\\makeatother\n\\begin{document}\n\\author{Name\\inst{*}}\n\\title{T}\n\\maketitle\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // `\inst{*}` is a footnote-symbol mark, kept as its glyph (never an
  // affiliation number to link).
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &[],
    r##"<creator role="author"><personname>Name<sup>*</sup></personname></creator>"##,
  );
  // A class binding's own `\inst` (the affiliation-LINKING form) beats the
  // `\providecommand` fallback.
  let llncs = "\\documentclass{llncs}\n\\begin{document}\n\\title{T}\n\\author{Name\\inst{1}}\n\\institute{Univ A}\n\\maketitle\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(llncs, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("role=\"affiliation\"") && !xml.contains("<sup>1</sup>"),
    "{xml}"
  );
}

/// `\@nil` is undefined, as in latex.ltx, so the `\ifx\@nil#1` sentinel
/// test is false against an empty macro (polynom.sty:1695
/// `\pld@MeasureCells@`; witness polynom/polydemo stage=8 "Stray
/// alignment").
#[test]
fn at_nil_is_undefined_for_ifx_sentinels() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\es{}\n\\begin{document}\n\\ifx\\@nil\\es EQ\\else NE\\fi;\\ifx\\@nil\\@undefinedcs SAME\\else DIFF\\fi.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("NE;SAME."), "{xml}");
  if kpsewhich_has("polynom.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{polynom}\n\\begin{document}\n\\[\\polyhornerscheme[x=-2,stage=8]{x^3+x^2-1}\\]\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The Horner scheme is polynom's `\halign` inside display math: a
    // text-mode tabular of inline math cells.
    assert!(
      xml.contains("<tabular") && xml.matches("<tr").count() == 3,
      "{xml}"
    );
  }
}

/// CJK.sty:232 `\Unicode{hi}{lo}` typesets the code point `hi*256+lo`
/// (cjkutf8-ko.sty:65 `\dotemphchar` = U+02D9; witness cjk-ko/cjk-ko-doc).
#[test]
fn cjk_unicode_inserts_the_code_point() {
  let tex = "\\documentclass{article}\n\\usepackage{CJKutf8}\n\\begin{document}\n[\\Unicode{0}{\"B7}\\Unicode{\"02}{\"D9}]\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[\u{b7}\u{2d9}]"), "{xml}");
  if kpsewhich_has("kotex.sty") && kpsewhich_has("cjkutf8-ko.sty") {
    let tex = "\\documentclass{article}\n\\usepackage[cjk,hangul,usedotemph]{kotex}\n\\begin{document}\n\\dotemph{ABC}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // `\CJKfamily{nanummj}` (kotex's font setup) is a font switch, not text.
    assert!(xml.contains("ABC") && !xml.contains("nanummj"), "{xml}");
  }
}

/// ifpdf.sty is `\RequirePackage{iftex}`: `\ifpdf` has ONE profile-aware
/// source, so tikzrput.sty:66's `\ifpdf…\def\rput…\fi` defines `\rput`
/// under the luatex profile (pgfornament ornaments, tikzrput) and the
/// legacy `\pdffalse` setter still works.
#[test]
fn ifpdf_delegates_to_iftex() {
  let tex = "\\documentclass{article}\n\\usepackage{ifpdf}\n\\begin{document}\n\\ifpdf PDF\\else DVI\\fi;\\pdffalse\\ifpdf PDF\\else DVI\\fi.\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("PDF;DVI."), "{xml}");
  // The pdfTeX persona answers from `\pdfoutput`: PDF by default (the K6
  // ruling, 2026-09-24), DVI when the document selects it.
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("PDF;DVI."), "{xml}");
  let dvi = tex.replace("\\documentclass", "\\pdfoutput=0\n\\documentclass");
  let (stderr, xml) = convert(&dvi, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("DVI;DVI."), "{xml}");
  if kpsewhich_has("tikzrput.sty") && kpsewhich_has("tufte-handout.cls") {
    let tex = "\\RequirePackage{luatex85}\n\\documentclass{tufte-handout}\n\\usepackage{tikz}\n\\usepackage{tikzrput}\n\\begin{document}\n\\rput(0,0){X}\n\\end{document}\n";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<picture") || xml.contains("<svg"), "{xml}");
  }
}

/// beamerbasetitle.sty:214-215 `\keywords{…}` (PDF metadata) is provided
/// beside `\subject` (witness beamerswitch-example).
#[test]
fn beamer_keywords_is_provided() {
  let tex = "\\documentclass{beamer}\n\\title{T}\n\\author{A}\n\\keywords{CTAN, literate programming}\n\\begin{document}\n\\begin{frame}\\maketitle\\end{frame}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<title>T</title>") && !xml.contains("literate programming"),
    "{xml}"
  );
}

/// caption3.sty:446-457 `\SetCaptionDefault{name}{value}` binds
/// `\caption@<name>@default`, so bicaption.sty:92/132's biseparator
/// default resolves (witness shtthesis-user-guide).
#[test]
fn setcaptiondefault_binds_the_default() {
  let tex = "\\documentclass{article}\n\\usepackage{caption}\n\\makeatletter\n\\def\\caption@foo@bar{BAR}\n\\SetCaptionDefault{foo}{bar}\n\\begin{document}\n[\\caption@foo@default]\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[BAR]"), "{xml}");
  if kpsewhich_has("bicaption.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{keyval,caption}\n\\usepackage{bicaption}\n\\begin{document}\nx\n\\end{document}\n";
    let (stderr, _) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
  }
}

/// hyperref.sty:2077 `\Hy@writebookmark` (5 arguments) and
/// biblatex.sty:15506 `\BiblatexManualHyperrefOn` exist for classes that
/// call them directly (shtthesis.cls:330).
#[test]
fn hyperref_and_biblatex_manual_internals_exist() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\usepackage[hyperref=manual]{biblatex}\n\\makeatletter\n\\begin{document}\n\\Hy@writebookmark{0}{T}{a.1}{1}{toc}x\\BiblatexManualHyperrefOn y\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("x y") || xml.contains("xy"), "{xml}");
}

/// LuaTeX's `\glet` primitive (`\protected\def\glet{\global\let}`,
/// luatex-enhancements.tex:991) exists under the luatex profile only
/// (apa.cbx:645; witness biblatex-apa-test, lualatex oracle). Engine
/// check 2026-09-06: `pdflatex` on `\glet\myfoo\relax` = "Undefined
/// control sequence", `lualatex` = OK — each persona follows its engine.
#[test]
fn luatex_profile_defines_glet() {
  let tex = "\\documentclass{article}\n\\begin{document}\n{\\glet\\myfoo\\relax}\\ifx\\myfoo\\relax OK\\else BAD\\fi\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("OK"), "{xml}");
  // pdfTeX has no `\glet`.
  let (stderr, _) = convert(tex, true);
  assert!(stderr.contains("undefined:\\glet"), "{stderr}");
  assert_eq!(error_count(&stderr), 2, "{stderr}");
}

/// Cell mode follows `\@classz` (latex.ltx:16550/16561): a raw
/// `\let\@classz\@tabclassz … \@tabarray` scaffold gets TEXT cells whose
/// `$45^\circ$` is a clean inline math (witness aguplus planotable,
/// aguplus.tex:633; Perl 8).
#[test]
fn tabarray_cell_mode_follows_classz() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\begin{document}\n\\let\\@halignto\\@empty\n\\hbox{$\\let\\@acol\\@tabacol\\let\\@classz\\@tabclassz\n  \\let\\@classiv\\@tabclassiv\\let\\\\\\@tabularcr\n  \\@tabarray{lcc}A & $45^\\circ$ & b\\\\ C & $90^\\circ$ & d\\endarray$}\n\\makeatother\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<tabular") && xml.matches("<tr").count() == 2,
    "{xml}"
  );
  assert!(xml.contains("<td") && xml.contains("<Math"), "{xml}");
  // `\begin{array}` keeps math cells (t-angles: `\let\@classz\@arrayclassz`).
  let tex = "\\documentclass{article}\n\\begin{document}\n$\\begin{array}{cc}a&b\\\\c&d\\end{array}$\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<XMArray"), "{xml}");
}

/// latex.ltx:16554 `\endtabular` = `\crcr\egroup\egroup $\egroup`: a
/// deluxetable-style raw scaffold (`\hbox\bgroup$…\@tabarray` …
/// `\endtabular`, aguplus.cls:305 `\pt@tabular`) closes balanced, and the
/// template is `\edef`-expanded (`\string lcc`, aguplus.cls:314).
#[test]
fn raw_tabular_scaffold_closes_and_template_expands() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\newbox\\pt@box\n\\def\\pt@format{\\string lcc}\n\\def\\@halignto{}\n\\def\\@ptabacol{\\edef\\@preamble{\\@preamble\\hskip\\tabcolsep\\tabskip\\fill}}\n\\def\\pt@tabular{\\hbox\\bgroup$\\let\\@acol\\@ptabacol\n  \\let\\@classz\\@tabclassz\\let\\@classiv\\@tabclassiv\\let\\\\\\@tabularcr\\@tabarray}\n\\begin{document}\n\\setbox\\pt@box=\\pt@tabular{\\pt@format}%\nA & $45^\\circ$ & $90^\\circ$ \\\\\nB & $56^\\circ$ & $124^\\circ$\n\\crcr\\endtabular\n\\box\\pt@box\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Unrecognized tabular template"),
    "{stderr}"
  );
  assert!(
    xml.contains("<tabular") && xml.matches("<tr").count() == 2 && xml.matches("<td").count() == 6,
    "{xml}"
  );
  assert!(xml.contains("After.") && !xml.contains("<ERROR"), "{xml}");
  // The constructor `\begin{tabular}` opens no scaffold: nothing extra closes.
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\pt@format{\\string lcc}\n\\begin{document}\n\\begin{tabular}{\\pt@format}\nA & $45^\\circ$ & $90^\\circ$\\\\\nB & $56^\\circ$ & $124^\\circ$\n\\end{tabular}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Unrecognized tabular template"),
    "{stderr}"
  );
  assert!(
    xml.matches("<td").count() == 6 && xml.contains("After."),
    "{xml}"
  );
}

/// A class that redefines `\abstract` as an argument-taking command
/// (ryethesis.cls:344 `\newcommand{\abstract}[1]{…}`) reads `{…}` as one
/// stored argument, so `\Gls{LI}` before `\newacronym{LI}` resolves at
/// frontmatter time (witness ryethesis ryesample; Perl 0, Rust 1).
#[test]
fn class_redefined_abstract_defers_its_argument() {
  if !kpsewhich_has("glossaries.sty") {
    return;
  }
  let cls = "\\ProvidesClass{deferabs}\n\\LoadClass{report}\n\\newcommand{\\abstract}[1]{\\gdef\\my@theabstract{#1}}\n";
  let tex = "\\documentclass{deferabs}\n\\usepackage[acronym]{glossaries}\n\\begin{document}\n\\abstract{\\Gls{LI} dolor sit amet.}\n\\newglossaryentry{Lorem}{name={lorem},description={x}}\n\\newacronym{LI}{LI}{lorem ipsum}\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex, &[("deferabs.cls", cls)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<abstract") && xml.contains("dolor sit amet"),
    "{xml}"
  );
  // A `\def\abstract#1{…}` class (apa7.cls:785 shape) records the same.
  let cls2 =
    "\\ProvidesClass{deferabs2}\n\\LoadClass{report}\n\\def\\abstract#1{\\gdef\\@abstract{#1}}\n";
  let tex2 = "\\documentclass{deferabs2}\n\\begin{document}\n\\abstract{\\undefinedlater ipsum.}\n\\def\\undefinedlater{Lorem}\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex2, &[("deferabs2.cls", cls2)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // `\undefinedlater ipsum.`: the space after the control word is gobbled.
  assert!(xml.contains("Loremipsum."), "{xml}");
}

/// thumbs.sty loads in its own `hidethumbs` off-mode (:1537-1544), so the
/// shipout-reset state machine never raises "\thumbnewcolumn after
/// \addthumb" (:573; witness thumbs-example).
#[test]
fn thumbs_loads_in_its_own_hide_mode() {
  if !kpsewhich_has("thumbs.sty") {
    return;
  }
  let tex = "\\documentclass[twoside]{article}\n\\usepackage{thumbs}\n\\begin{document}\n\\section{A}\n\\addthumb{F mark}{\\Huge F}{magenta}{black}\nSome text.\n\\newpage\n\\thumbnewcolumn\n\\addthumb{New column}{\\Huge NC}{magenta}{black}\nThere.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Some text.") && xml.contains("There."),
    "{xml}"
  );
}

/// The hyperref PDF-form internal stubs carry their hpdftex.def /
/// hyperref.sty arity: a 0-argument `\HyField@AddToFields` (hpdftex.def:837)
/// no longer swallows the `\endgroup` that follows it (hyperbar.sty:175;
/// witness hyperbar/example `\end{Form}` mode error).
#[test]
fn hyperref_form_internals_keep_driver_arity() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\makeatletter\n\\begin{document}\n\\begin{Form}\nA\\begingroup\\leavevmode\\HyField@AddToFields\\endgroup B\\PDFForm@Name C\\HyField@UseFlag{Ff}{Multiline}D\n\\end{Form}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Each 0-argument stub gobbles only the space after its name (TeX's
  // control-word rule): the letters run together.
  assert!(xml.contains("ABCD") && !xml.contains("Multiline"), "{xml}");
}

/// tex.web §1335: an `\abstract{` whose brace never closes
/// (screenplay-pkg.tex:67) ends with the benign "(\end occurred inside a
/// group)" line, not a mode error; a `\section` inside the open group ends
/// the abstract (the section hook's terminal arrives inside the nested `{`
/// body and `until_terminal_inside_group` closes the runaway group), so the
/// section lands at document level as in the brace-less form.
#[test]
fn unbalanced_abstract_brace_unwinds_at_end() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\\author{A}\\date{}\n\\maketitle\n\\abstract{\\begin{quote}This is the abstract body.\n\\end{quote}\n\\section{Intro}\nSome following text.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<abstract") && xml.contains("</abstract>"),
    "{xml}"
  );
  let abs_end = xml.find("</abstract>").expect("abstract closes");
  let sec = xml.find("<section").expect("section present");
  assert!(
    sec > abs_end && xml.contains("Some following text."),
    "section nested in the abstract:\n{xml}"
  );
  let tex = "\\documentclass{article}\n\\begin{document}\n\\title{T}\\author{A}\\date{}\n\\maketitle\n\\abstract{\\begin{quote}Abstract body, no closing brace, no section.\n\\end{quote}\nTrailing text still inside the runaway group.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<abstract") && xml.contains("Trailing text"),
    "{xml}"
  );
}

/// A raw package's `\def\multicolumn` (agupp.sty:599 = latex.ltx's, whose
/// `\@mkpream` executes the `\let`-only `\@classz`/`\@acol`) is dropped by
/// the lock; the native alignment `\multicolumn` keeps the cell (witness
/// aguplus/aguplus:731; Perl shares the two undefined errors).
#[test]
fn raw_multicolumn_redefinition_is_dropped() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\multicolumn#1#2#3{\\multispan{#1}\\begingroup\\@mkpream{#2}\\def\\@sharp{#3}\\set@typeset@protect\\@arstrut\\@preamble\\hbox{}\\endgroup\\ignorespaces}\n\\makeatother\n\\begin{document}\n\\begin{tabular}{l@{~$\\Rightarrow$~}l}\na & b\\\\\n\\multicolumn{1}{c}{X} & c\\\\\n\\end{tabular}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<td align=\"center\">X</td>"), "{xml}");
  if kpsewhich_has("aguplus.cls") && kpsewhich_has("agupp.sty") {
    let tex = "\\documentclass[twoside,agupp]{aguplus}\n\\begin{document}\n\\begin{tabular}{l@{~$\\Rightarrow$~}l}\na & b\\\\\n\\multicolumn{1}{c}{X} & c\\\\\n\\end{tabular}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<td align=\"center\">X</td>"), "{xml}");
  }
}

/// Batch 56bz: `\addbibresource`'s argument is expanded before it is
/// recorded (biblatex.sty:1216-1222 `\blx@addbib` `\edef`s then
/// `\detokenize`s it), so the `\addbibresource{\jobname.bib}` idiom of the
/// manuals that ship their `.bib` through `filecontents` (biblatex-nejm,
/// cleanthesis, gitlog, shtthesis; 16 corpus docs) records the file NAME.
/// The literal control sequence was stored before, and no bibliography
/// stage can open `\jobname.bib`.
#[test]
fn biblatex_addbibresource_expands_its_argument() {
  let tex = "\\documentclass{article}\n\\usepackage{filecontents}\n\\begin{filecontents}{t.bib}\n@book{knuth84, author={Donald Knuth}, title={The TeXbook}, year={1984}, publisher={Addison-Wesley}}\n\\end{filecontents}\n\\usepackage[backend=biber]{biblatex}\n\\def\\mybibfile{t.bib}\n\\addbibresource{\\mybibfile}\n\\begin{document}\nCite \\cite{knuth84}.\n\\printbibliography\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<bibliography files=\"t.bib\"") || xml.contains("files=\"t.bib\""),
    "{xml}"
  );
  assert!(!xml.contains("\\mybibfile"), "{xml}");
}

/// 56ga: a raw package's `\addbibresource{\jobname.bib}` (incgraph-doc.sty:58,
/// gitlog.sty:111 `\gitLog@bibfile`) was recorded TWICE — expanded by the
/// native primitive and as the literal by the beyond-Perl dependency
/// scanner — and MakeBibliography then reported the literal as a missing
/// bibliography. The scanner leaves control-sequence names to the runtime.
/// (Core stage: the literal landed in the `files` attribute; the post error
/// was downstream of it.)
#[test]
fn addbibresource_macro_name_in_a_raw_package_resolves_once() {
  let tex = "\\documentclass{article}\n\\usepackage{filecontents}\n\\begin{filecontents}{t.bib}\n\
               @book{knuth84, author={Donald Knuth}, title={The TeXbook}, year={1984}}\n\\end{filecontents}\n\
               \\usepackage{adbres}\n\\begin{document}\nSee \\cite{knuth84}.\n\\printbibliography\n\\end{document}\n";
  let sty =
    "\\ProvidesPackage{adbres}\n\\RequirePackage{biblatex}\n\\addbibresource{\\jobname.bib}\n";
  let (stderr, xml) = convert_files(tex, &[("adbres.sty", sty)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("files=\"t.bib\""),
    "the expanded resource is recorded once:\n{xml}"
  );
  assert!(
    !xml.contains("\\jobname"),
    "no literal control sequence in the resources:\n{xml}"
  );
}

/// biblatex.sty:11277-11283 `\addglobalbib`/`\addsectionbib` record
/// resources like `\addbibresource` (biblatex-apa-test, shtthesis).
#[test]
fn biblatex_addglobalbib_records_resources() {
  let tex = "\\documentclass{article}\n\\usepackage{filecontents}\n\\begin{filecontents}{t.bib}\n@book{knuth84, author={Donald Knuth}, title={The TeXbook}, year={1984}, publisher={Addison-Wesley}}\n\\end{filecontents}\n\\usepackage[backend=biber]{biblatex}\n\\addglobalbib{t.bib}\n\\addsectionbib[label=x]{t.bib}\n\\begin{document}\nCite \\cite{knuth84}.\n\\printbibliography\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The resource was RECORDED: the bibliography element carries it (the
  // entries are filled at post-processing).
  assert!(
    xml.contains("<bibliography") && xml.contains("files=\"t.bib\""),
    "{xml}"
  );
}

/// xcolor.sty:1373-1396 contract: `\XC@getcolor{spec}\cs` leaves
/// `\xcolor@{}{drv}{model}{spec}` in `\cs` and `\XC@undeclaredcolor{model}
/// {spec}` sets the colour — the path lua-ul.sty:82-100 takes whenever
/// `\XC@getcolor` exists (witness gckanbun kanshi-sample under luwa-ul).
#[test]
fn xcolor_internal_api_matches_the_real_contract() {
  let tex = "\\documentclass{article}\n\\usepackage{xcolor}\n\\makeatletter\n\\def\\strip\\xcolor@#1#2{}\n\\begin{document}\n\\XC@getcolor{blue}\\x\\edef\\y{\\expandafter\\strip\\x}%\n{\\expandafter\\XC@undeclaredcolor\\y B}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("color=\"#0000FF\"") && xml.contains(">B<"),
    "{xml}"
  );
  if kpsewhich_has("lua-ul.sty") && kpsewhich_has("luacolor.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{luacolor,lua-ul}\n\\begin{document}\n\\underLine{under} and \\highLight[yellow]{high}.\n\\end{document}\n";
    let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("under") && xml.contains("high"), "{xml}");
  }
}

/// tex.web §15510 `align_peek`: the cell-head token is expanded in the
/// alignment's inter-row mode (internal vertical, §15350), before `init_row`
/// (§15532) enters the cell's restricted horizontal mode — so a cell head
/// `\ifhmode\else\expandafter\hbox\fi\bgroup…$…$…\egroup` (abntexto.tex:79-81)
/// keeps its `\hbox` and the display closes (witness abntexto/abntexto).
#[test]
fn alignment_cell_head_peeks_in_internal_vertical_mode() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\X{\\ifhmode\\else\\expandafter\\hbox\\fi\\bgroup $a$\\egroup}\n\\makeatother\n\\begin{document}\nBefore.\n$$\\offinterlineskip\\halign{$#$\\cr \\X + b\\cr}$$\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<tabular") && xml.contains("mode=\"inline\""),
    "{xml}"
  );
  assert!(xml.contains("After."), "{xml}");
  // Inside a cell tex.web init_row enters -hmode (§15532); LaTeXML's cell begins in a
  // vmode-ish galley, so the first `\\ifhmode` reads V — `VH || HH` accepts both.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{tabular}{l}\n\\ifhmode H\\else V\\fi\\ifhmode H\\else V\\fi\\\\\n\\end{tabular}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("VH") || xml.contains("HH"), "{xml}");
  // The override is frame-local: it must end in the frame that set it, before
  // the cell/row groups open — otherwise every LATER cell (and every row led
  // by `\multicolumn`'s `\omit`) stays in internal vertical mode, which the
  // goldens showed as spaces kept before `&` in `tex=` and diagbox cells
  // measured at the text width. Three cells after a group-opening peek must
  // all read the horizontal cell mode, and a space before `&` must vanish.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{tabular}{lll}\n\\multicolumn{1}{c}{M} & \\ifhmode H\\else V\\fi\\ifhmode H\\else V\\fi & \\ifhmode H\\else V\\fi\\ifhmode H\\else V\\fi\\\\\n\\hline\nA & \\ifhmode H\\else V\\fi\\ifhmode H\\else V\\fi & 2 \\\\\n\\end{tabular}\n$\\begin{array}{cc} 1 & 2 \\\\ 3 & 4 \\end{array}$\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("VH").count() + xml.matches("HH").count(),
    3,
    "{xml}"
  );
  assert!(xml.contains("1&amp;2\\\\"), "{xml}");
}

/// pgfsys-common-pdf.def:37-38: a graphics-state scope is `q`/`Q` output,
/// not a TeX group, so pgf may open it before a box and close it inside
/// (pgfsys.code.tex:572-611 `\pgfsys@begin@idscope`; witnesses msc/msc,
/// modernposter/demo).
#[test]
fn pgf_scope_straddling_a_box_is_not_a_tex_group() {
  let tex = "\\documentclass{article}\n\\usepackage{pgf}\n\\makeatletter\n\\begin{document}\n\\begin{pgfpicture}\n\\pgfsys@beginscope\n\\setbox0=\\hbox{X\\pgfsys@endscope}%\n\\box0\n\\end{pgfpicture}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg:svg") && xml.contains("After."), "{xml}");
  // msc's `\end{msc}` title node (`\pgf@maketext`) crosses driver scopes;
  // an instance (`\declinst`) still trips the parked fused mode-frame family
  // (DIFFICULT_CASES D12), so the guard stops at the empty chart.
  if kpsewhich_has("msc.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{msc}\n\\begin{document}\n\\begin{msc}{Chart}\n\\end{msc}\nAfter.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("<svg:svg") && xml.contains("<svg:g") && xml.contains("After."),
      "{xml}"
    );
  }
}

/// tex.web §21745 `start_eq_no`: the `\eqno`/`\leqno` tag is DIGESTED as a
/// math list, so an assignment in it executes at its position (mhequ.sty:184
/// `\@restoreMHComms` after `\eqno{…}`; witness mhequ/mhequ-example).
#[test]
fn eqno_digests_its_tag_material() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\let\\BAD\\undefinedxyz\n$$ a \\eqno \\let\\BAD\\relax \\BAD (1) $$\nAfter.\n$$ b \\leqno (2) $$\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tags>").count(), 2, "{xml}");
  assert!(xml.contains("After."), "{xml}");
  if kpsewhich_has("mhequ.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{mhequ}\n\\begin{document}\n\\begin{equ}[onelab]\n\te^{i\\pi} + 1 = 0 \\;.\n\\end{equ}\nAfter: a \\\\ b.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<equation") && xml.contains("<break"), "{xml}");
  }
}

/// latex.ltx:18637 `\@loadwithoptions`: `\LoadClassWithOptions` hands the
/// calling class's option list to the loaded class (witness istgame-doc:
/// `\documentclass[amsmath]{oblivoir}` → oblivoir-utf.cls:77 loads amsmath).
#[test]
fn load_class_with_options_forwards_the_calling_options() {
  // A local wrapper class with NO option handling of its own: the options
  // reach `article` only through `\LoadClassWithOptions`.
  let tex = "\\documentclass[twocolumn,fleqn]{wrapcls}\n\\makeatletter\n\\begin{document}\n\\@ifclasswith{article}{twocolumn}{TWO}{ONE}\\@ifclasswith{article}{fleqn}{-F}{-N}\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex, &[(
    "wrapcls.cls",
    "\\ProvidesClass{wrapcls}\n\\LoadClassWithOptions{article}\n",
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("TWO-F"), "{xml}");
  if kpsewhich_has("oblivoir.cls") {
    let tex = "\\documentclass[amsmath]{oblivoir}\n\\begin{document}\n$\\text{hello}$ $\\binom{n}{k}$\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("hello") && xml.contains("<Math"), "{xml}");
  }
}

/// A class/document `\renewcommand{\maketitle}[1]{…}` is dropped by the
/// lock and never replayed (the Gemini round-7 raw replay was reverted:
/// resphilosophica.cls:331's body needs amsart internals the binding lacks);
/// the kernel's frontmatter still lands and the raw `#1` never reaches the
/// stomach.
#[test]
fn maketitle_replay_skips_a_parameterized_body() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\renewcommand{\\maketitle}[1]{TITLEARG:#1:END}\n\\makeatother\n\\title{T}\\author{A}\n\\begin{document}\n\\maketitle{HELLO}\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<title>T</title>") && xml.contains("Body."),
    "{xml}"
  );
  assert!(!xml.contains("TITLEARG"), "{xml}");
}

/// tex.web §1214: `\globaldefs>0` globalizes the ASSIGNMENTS of
/// `prefixed_command`; the save stack (§274/§282) is untouched. Ours (and
/// Perl's, State.pm:144-151) routed the frame bookkeeping through the same
/// override, so under `\globaldefs=1` (msc.sty:2616 `\msc@global@set`) a
/// closed `{` group kept reporting itself as the current frame and every
/// later closer cascaded ("close non-boxing group"; the D12 family).
#[test]
fn globaldefs_does_not_globalize_the_save_stack() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\globaldefs=1 \\setbox0\\hbox{X{Y}}\\globaldefs=0 \\box0 {\\begingroup Z\\endgroup}A\n{\\globaldefs=1 \\def\\gl{G}}\\gl\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The box, the group content and the text after them all land …
  assert!(
    xml.contains("XY") && xml.contains("Z") && xml.contains("A"),
    "{xml}"
  );
  // … and `\globaldefs` still globalizes a real assignment made in a group.
  assert!(xml.contains("G</p>") || xml.contains("G\n"), "{xml}");
}

/// pgfmath `min`/`max` fold over EVERY argument
/// (pgfmathfunctions.misc.code.tex:292-336 `\pgfmathmin@@`); the native
/// arms were binary, so `min(55,74,35)` was 55 —
/// ribbonproofs.sty:1213 `min(\@leftPositions)` mis-stepped a block and a
/// ribbon re-started "already active" (ribbonproofsmanual 3, pdflatex 0).
#[test]
fn pgfmath_min_max_fold_over_every_argument() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\pgfmathparse{min(55,74,35)}A\\pgfmathresult.\n\\pgfmathparse{max(10,20,30)}B\\pgfmathresult.\n\\pgfmathparse{min(20)}C\\pgfmathresult.\n\\pgfmathparse{min(55,35)}D\\pgfmathresult.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // `min(20)` is `min(2,0)`: a lone argument is scanned one token at a time
  // (pdflatex: 0).
  assert!(
    xml.contains("A35.") && xml.contains("B30.") && xml.contains("C0.") && xml.contains("D35."),
    "{xml}"
  );
  if kpsewhich_has("ribbonproofs.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{ribbonproofs}\n\\begin{document}\n\\begin{ribbonproof}[start ribbons={c/{left=35,right=53},e/{left=74,right=86}}]\n\\startblock[extra left=33,fit ribbons={c,e},start ribbons={d/{left=55,right=70}}]{if}\\\\\n\\jus[finish ribbons={c,d}]{u}\n\\com[finish ribbons={e},start ribbons={e/{}}]{x}\\\\\n\\moveribbons{e/{left=4}}\\\\\n\\continueblock[repeat labels,start ribbons={d/{}}]{else}\n\\end{ribbonproof}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<picture").count(), 1, "{xml}");
  }
}

/// pgfmathcalc.code.tex:366-468 `\pgfmathpointintersectionoflineandarc`
/// bisects until pgf's fixed-point trig reaches an exact angle equality;
/// with float trig the loop never exits (a rounded-rectangle border query
/// for a self-loop wire: zx-calculus `\zxLoopAboveDots`, callout nodes
/// arXiv 2201.09268 — the 50,000-box cycle fatal). The binding solves the
/// line/ellipse intersection in closed form; the `rectangle` control has no
/// arc and never bisected.
#[test]
fn line_and_arc_intersection_is_closed_form() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\usetikzlibrary{shapes.misc,topaths}\n\\begin{document}\n\\begin{tikzpicture}\n\\node[rounded rectangle, draw, minimum width=1cm, minimum height=6mm] (a) at (0,0) {};\n\\draw (a) to[out=110,in=70,looseness=8] (a);\n\\end{tikzpicture}\n\\begin{tikzpicture}\n\\node[rectangle, draw, minimum width=1cm, minimum height=6mm] (b) at (0,0) {};\n\\draw (b) to[out=110,in=70,looseness=8] (b);\n\\end{tikzpicture}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<svg:path").count(), 4, "{xml}");
  if kpsewhich_has("tikzlibraryzx-calculus.code.tex") {
    let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\usetikzlibrary{zx-calculus}\n\\begin{document}\n\\zx{\\zxX{\\alpha} \\zxLoopAboveDots{}}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.matches("<svg:path").count() >= 2, "{xml}");
  }
}

/// latex.ltx:15255-15259 `\enddocument` runs the end-document hooks INLINE
/// before `\@checkend`, and etoolbox.sty:1774-1776 runs `\@afterendpreamblehook`
/// inline from `\document`: a box opened from the one hook and closed from the
/// other reads the document body as its contents. Digesting either hook in
/// an isolated mouth starved the box reader (modernposter.cls's
/// document-spanning `tikzpicture[overlay]`; Perl shares it).
#[test]
fn atenddocument_closer_reaches_the_galley_box() {
  let tex = "\\documentclass{article}\n\\AtEndDocument{X\\egroup}\n\\begin{document}\n\\setbox0=\\hbox\\bgroup A\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Warning:"), "{stderr}");
  assert!(xml.contains("</document>"), "{xml}");
  // The opener from `\AfterEndPreamble`, the closer from `\AtEndDocument`.
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\usepackage{etoolbox}\n\\AfterEndPreamble{\\begin{tikzpicture}[remember picture, overlay]}\n\\AtEndDocument{\\end{tikzpicture}}\n\\begin{document}\n\\node (title) at (0,0) {\\Huge Demo Title};\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<svg:g") && xml.contains("Demo Title"),
    "{xml}"
  );
  // Control: the same picture in the body.
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}[remember picture, overlay]\n\\node (title) at (0,0) {\\Huge Demo Title};\n\\end{tikzpicture}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<svg:g") && xml.contains("Demo Title"),
    "{xml}"
  );
  // latex.ltx:15278: the expansion ends with `\@@end`, and a hook may grab
  // the rest of it up to that token and re-emit it (morewrites.sty:550-557).
  let tex = "\\documentclass{article}\n\\makeatletter\n\\AtEndDocument{\\def\\grab#1\\@@end{TAIL#1\\@@end}\\grab}\n\\makeatother\n\\begin{document}\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Body.") && xml.contains("TAIL") && xml.contains("</document>"),
    "{xml}"
  );
  if kpsewhich_has("morewrites.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{morewrites}\n\\begin{document}\n\\newwrite\\w\\immediate\\openout\\w=t-extra.txt\\immediate\\write\\w{x}\nBody.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Body.") && xml.contains("</document>"),
      "{xml}"
    );
  }
}

/// pgf's matrices open with `\halign\bgroup` (tikz-cd, `\matrix`), for which
/// the gullet's ALIGN_STATE was never masked; `\lxSVG@halign` decremented
/// the align-group count unconditionally where the standard `\halign` does
/// it only for a `{` opener, so the ENCLOSING amsmath alignment lost a level:
/// its cell's closing hidden `$` was not recognized and the math frame stayed
/// open (zx-calculus 46 errors; tikz-cd inside `align`; Perl worse).
#[test]
fn tikzcd_matrix_inside_an_amsmath_cell_closes_its_math() {
  let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{tikz}\n\\usepackage{tikz-cd}\n\\begin{document}\n\\begin{align}\n  \\begin{tikzcd} A \\arrow[r] & B \\end{tikzcd} &= x\n\\end{align}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<equation") && xml.contains("<svg:g") && xml.contains("After."),
    "{xml}"
  );
  // Control: a plain picture in the same cell (no matrix).
  let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{align}\n  \\begin{tikzpicture}\\node{A};\\end{tikzpicture} &= x\n\\end{align}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<equation") && xml.contains("<svg:g") && xml.contains("After."),
    "{xml}"
  );
  if kpsewhich_has("tikzlibraryzx-calculus.code.tex") {
    let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{tikz}\n\\usetikzlibrary{zx-calculus}\n\\begin{document}\n\\begin{align}\n  \\zx{\\zxZ{}} &= \\zx{\\zxX{}}\n\\end{align}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<equation") && xml.contains("<svg:g"), "{xml}");
  }
}

/// amsmath.sty:1896 initializes `\maxcolumn@widths` to `\@empty`; the binding
/// lacked it, so cryptocode.sty:462-470's `\let\got@maxcolwd\maxcolumn@widths`
/// copied an undefined meaning and `\dimexpr\got@maxcolwd` errored on every
/// `\pseudocode` (zx-calculus manual; Perl identical, pdflatex clean).
#[test]
fn amsmath_maxcolumn_widths_is_initialized() {
  if kpsewhich_has("cryptocode.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{cryptocode}\n\\begin{document}\n\\[ \\pseudocode{a \\gets b \\\\ c \\gets d} \\]\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !xml.contains("got@maxcolwd") && xml.contains("ltx_eqn_align"),
      "{xml}"
    );
  }
  let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\makeatletter\n\\begin{document}\n\\ifx\\maxcolumn@widths\\@empty EMPTY\\else OTHER\\fi\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("EMPTY"), "{xml}");
}

/// xkeyval choice keys: the bin macro holds the chosen value with its original
/// catcodes (`\XKV@checkchoice`; `\lowercase` for `\define@choicekey*` keeps
/// them), so `\ifx`/`\in@` against a catcode-11 list matches. It was built
/// with `Explode!` (catcode-12 letters; Perl KeyVal.pm:143 too), which is why
/// powerdot.cls:353-387 never saw `\ifpd@ifsetup` true and every `\pd@@<key>`
/// stayed undefined (powerdot-fuberlin, 20 errors surfaced by batch 56ao).
/// KNOWN_PERL_ERRORS #215, batch 56ar.
#[test]
fn choicekey_bin_macro_keeps_letter_catcodes() {
  let tex = "\\documentclass{article}\n\\usepackage{xkeyval}\n\\makeatletter\n\\define@choicekey{lx}{mode}[\\lxval\\lxnr]{alpha,beta}{}\n\\define@choicekey*{lx}{mo}[\\lxv]{Alpha,Beta}{}\n\\setkeys{lx}{mode=beta,mo=BETA}\n\\def\\lxbeta{beta}\n\\edef\\lxres{[\\ifx\\lxval\\lxbeta same\\else diff\\fi][\\lxnr][\\ifx\\lxv\\lxbeta same\\else diff\\fi]}\n\\makeatother\n\\begin{document}\n\\lxres\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[same][1][same]"), "{xml}");
  if kpsewhich_has("powerdot.cls") {
    let tex = "\\documentclass{powerdot}\n\\begin{document}\n\\title{T}\\author{A}\\date{}\n\\maketitle\n\\begin{slide}{S}\nx\n\\end{slide}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert!(!stderr.contains("pd@@"), "{stderr}");
    assert!(xml.contains(">S<") || xml.contains(">S\n"), "{xml}");
  }
}

/// `\newpsstyle{X}{…}` defines `\pscs@X` (pstricks.tex:638-645) and the raw
/// `style` key (pstricks.tex:633-636) consults it; a leftover noop stub for
/// `\newpsstyle` made every `\psset{style=X}` raise "Custom style 'X'
/// undefined" (pst-calendar-doc 15→101 once `\rput` bodies were digested).
/// Batch 56as.
#[test]
fn newpsstyle_defines_the_custom_style_psset_consults() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("pstricks.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{pstricks}\n\\newpsstyle{september}{linewidth=2pt}\n\\begin{document}\nTop: \\psset{style=september}\n\\begin{pspicture}(0,0)(2,2)\n\\rput(1,1){\\psset{style=september}\\psframe(0,0)(1,1)}\n\\end{pspicture}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Custom style"), "{stderr}");
  assert!(xml.contains("<picture"), "{xml}");
}

/// `\fcolorbox{frame}{bg}{text}` reads its two color names undigested and
/// expands them to strings (like `\color`); the digesting `{}` reader
/// (Perl xcolor.sty.ltxml:878 too) raised "Script _ can only appear in math
/// mode" on a name with `_` (hobete_doc). A macro-valued name still expands.
/// Batch 56at (SHARED, surpassed).
#[test]
fn fcolorbox_color_names_are_expanded_not_digested() {
  let tex = "\\documentclass{article}\n\\usepackage{xcolor}\n\\definecolor{foo_bar}{rgb}{1,0,0}\n\\def\\mybg{yellow}\n\\begin{document}\n\\fcolorbox{foo_bar}{\\mybg}{Test}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("framed=\"rectangle\"") && xml.contains("framecolor=\"#FF0000\""),
    "{xml}"
  );
  assert!(xml.contains("backgroundcolor=\"#FFFF00\""), "{xml}");
}

/// `\@roman`/`\@alph`/… are latex.ltx:10206-10224 token macros: `#1` is one
/// token and `\romannumeral`/`\ifcase` scan the number from the stream, so
/// `\csname x\@roman\the\cnt\endcsname` builds a name (texmate.sty:546).
/// The `{Number}` closures re-parsed the one token in an isolated mouth and
/// leaked the register into the name ("Extra \endcsname"). DIVERGENCES
/// #221, batch 56av.
#[test]
fn at_roman_family_expands_inside_csname() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\newcount\\mycount \\mycount=3\n\\expandafter\\def\\csname chessdiagiii\\endcsname{ROMANOK}\n\\begin{document}\nS:\\csname chessdiag\\@roman\\mycount\\endcsname:\nR:\\csname chessdiag\\@roman\\the\\mycount\\endcsname:\nA:\\@Roman{7}\\@alph{3}\\@Alph{4}\\@arabic{12}:\n\\makeatother\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("S:ROMANOK:"), "{xml}");
  assert!(xml.contains("R:ROMANOK:"), "{xml}");
  assert!(xml.contains("A:VIIcD12:"), "{xml}");
}

/// chemnum.sty:51-55 loads translations, chemgreek and psfrag; the binding
/// loaded none, so chemgreek's preamble-only `\activatechemgreekmapping`
/// (chemgreek.sty:486) was undefined after `\usepackage{chemnum}`. The
/// `default` mapping needs no extra font package. Batch 56ax.
#[test]
fn chemnum_loads_chemgreek() {
  if !kpsewhich_has("chemgreek.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{chemnum}\n\\activatechemgreekmapping{default}\n\\begin{document}\n\\cmpd{a} and \\chemalpha\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ltx_cmpd"), "{xml}");
}

/// The xfrac binding raw-loads xfrac.sty; its old stub no-op'ed the KERNEL
/// `\DeclareInstance` (latex.ltx:8671) for every package loaded after it,
/// so tasks.sty:780's `alphabetize` instance never existed ("instance
/// unknown"; substances-index ×50, schulmathematik). Batch 56ay.
#[test]
fn xfrac_keeps_the_kernel_declareinstance() {
  if !kpsewhich_has("tasks.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{xfrac}\n\\usepackage{tasks}\n\\begin{document}\n\\begin{tasks}(2)\\task A\\task B\\end{tasks}\n$\\sfrac{1}{2}$\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("is unknown"), "{stderr}");
  assert!(xml.contains(">A<") && xml.contains(">B<"), "{xml}");
  assert!(xml.contains("<XMApp"), "sfrac: {xml}");
}

/// chemformula.sty is raw-loaded (batch 56az): the old `\ch`→mhchem `\ce`
/// alias rejected chemformula's `"text"` literals ("Assertion failed",
/// chemformula-manual ×50) and knew no `chemformula/*` key.
#[test]
fn chemformula_ch_is_the_real_parser() {
  if !kpsewhich_has("chemformula.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{chemformula}\n\\setchemformula{format=\\sffamily}\n\\begin{document}\n\\ch{\"text\" O2 + 2 H2 -> 2 H2O}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Assertion failed"), "{stderr}");
  // chemformula's own rendering: the `"…"` literal survives and a subscript
  // is its raised text box (`math-scripts=false`, chemformula.sty:3092).
  assert!(xml.contains("textO"), "{xml}");
  assert!(xml.contains("class=\"ltx_markedasmath\""), "{xml}");
}

/// tex.web §783: the `\halign` preamble's separators are recognized by
/// MEANING, so an active character `\let` to `\cr` (metre's
/// `\obeylines`+`\let\par=\cr`) terminates the template. The template
/// parser only accepted control sequences and ran the preamble away to the
/// end of the input (Perl's strict token equality drops the table too).
/// Batch 56bb.
#[test]
fn halign_template_accepts_active_char_separators() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\catcode`\\~=\\active\n\\let~=\\cr\n\\setbox0=\\vbox{\\halign{#\\hfil&#\\hfil~\naaa&bbb~\nccc&ddd~\n}}\n\\box0\n\\par XYZ\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  for cell in ["aaa", "bbb", "ccc", "ddd"] {
    assert!(xml.contains(&format!(">{cell}<")), "{cell}: {xml}");
  }
  assert!(xml.contains("XYZ"), "{xml}");
}

/// latex.ltx `\index` → `\@wrindex#1` reads ONE undelimited argument; a bare
/// `\index` in prose takes the next token instead of scanning to the next
/// `{` anywhere ahead (varindex.dtx:1497: the scan ate `\end{abstract}` and
/// the abstract swallowed the document). KNOWN_PERL_ERRORS #216, batch 56ba.
#[test]
fn bare_index_takes_one_token() {
  let tex = "\\documentclass{article}\n\\makeindex\n\\begin{document}\n\\begin{abstract}\nthe \\index command, twice \\index here.\n\\end{abstract}\n\\section{S}\nBody \\index{real entry} text TAILMARK.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let abs = xml.find("<abstract").unwrap_or_else(|| panic!("{xml}"));
  let abs_end = xml[abs..]
    .find("</abstract>")
    .map(|i| abs + i)
    .unwrap_or_else(|| panic!("{xml}"));
  assert!(!xml[abs..abs_end].contains("<section"), "{xml}");
  assert!(xml.contains("<section"), "{xml}");
  assert!(xml.contains("real entry"), "{xml}");
  // The braced form must stop at its own `}` (a runaway would eat the tail).
  assert!(xml.contains("text TAILMARK."), "{xml}");
}

/// etoolbox.sty:849-852 `\csdef`/`\csedef`/`\csgdef`/`\csxdef` are
/// `\newrobustcmd*` (protected): an `\edef` stores the call verbatim and the
/// `\noexpand`ed `\the` inside its name argument comes back plain
/// (yquantlanguage-groups.sty:241-245); the binding's expandable macros
/// expanded them in place. `\gundef` (etoolbox.sty:931) exists (Perl omits
/// it, KNOWN_PERL_ERRORS #217). Batch 56bd.
#[test]
fn etoolbox_cs_definers_are_protected_and_gundef_exists() {
  let tex = "\\documentclass{article}\n\\usepackage{etoolbox}\n\\makeatletter\n\\csgdef{yqreg}{2}\n\\edef\\splittext{\\csgdef{import@\\noexpand\\the\\numexpr\\csname yqreg\\endcsname+\\noexpand\\@ne\\relax}{VECBODY}}\n\\splittext\n\\edef\\t{\\csgdef{a}{b}}\n\\def\\gone{here}\\gundef\\gone\n\\makeatother\n\\begin{document}\n\\ifcsname import@3\\endcsname import3-ok\\else no3\\fi. \\ifdefined\\gone still\\else gone-ok\\fi. \\meaning\\t.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("import3-ok"), "{xml}");
  assert!(xml.contains("gone-ok"), "{xml}");
  // `\meaning\t` keeps the protected call; the braces render as typographic
  // characters, so assert on the name and on the absence of its expansion.
  assert!(xml.contains("csgdef") && !xml.contains("unhbox"), "{xml}");
}

/// `\endtrivlist` closes only the trivlist's own `_autoclose` itemize: a
/// `\trivlist\item\relax\par…\endtrivlist` inside a `\list` item (the
/// cnltx `{example}[outside=true]` shape) had its list auto-closed by the
/// `\par`, and the closer then climbed to the OUTER list — later `\item`s
/// fell into the section. Perl identical; pdflatex clean. Batch 56be.
#[test]
fn endtrivlist_closes_only_its_own_list() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\begin{document}\n\\subsubsection*{T}\n\\begin{list}{}{}\n\\item Kosy\n\\begingroup\\trivlist\\item\\relax\\par\\endtrivlist\\endgroup\n\\item LGS\n\\item third\n\\end{list}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // All three items belong to the one outer list (the trivlist's own
  // itemize nests inside it, so the outer list ends at the LAST `</itemize>`).
  let outer = xml.find("<itemize").unwrap_or_else(|| panic!("{xml}"));
  let outer_end = xml.rfind("</itemize>").unwrap_or_else(|| panic!("{xml}"));
  let body = &xml[outer..outer_end];
  for label in ["Kosy", "LGS", "third"] {
    assert!(body.contains(label), "{label} outside the list: {xml}");
  }
  assert!(!xml[outer_end..].contains("<item"), "{xml}");
}

/// pb-diagram.sty is raw-loaded (its registers exist: pb-diagram.sty:45
/// `\newskip\dgARROWLENGTH`); the `{diagram}` picture stays the ar5iv
/// placeholder. The stub refused the raw load and every register use was
/// "undefined" + "expected a Variable" (pb-manual). Batch 56bf.
#[test]
fn pb_diagram_raw_loads_its_registers() {
  if !kpsewhich_has("pb-diagram.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{pb-diagram}\n\\begin{document}\n\\divide\\dgARROWLENGTH by2\nL:\\the\\dgARROWLENGTH.\n\\begin{diagram}\\node{A}\\arrow{e}\\node{B}\\end{diagram}\nAfter.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // 2.5em in cmr10 = 25.00004pt, halved (Perl prints the same).
  assert!(xml.contains("L:12.50002pt."), "{xml}");
  assert!(xml.contains("After."), "{xml}");
}

/// multicol's `\newcolumn` (multicol.sty:936-950, a column break) is a
/// no-op like `\columnbreak`; it was undefined (tikz-ext-manual). Batch 56bg.
#[test]
fn multicol_newcolumn_is_a_column_break() {
  let tex = "\\documentclass{article}\n\\usepackage{multicol}\n\\begin{document}\n\\begin{multicols}{2}\nA\\newcolumn B\\columnbreak C\n\\end{multicols}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("A") && xml.contains("B") && xml.contains("C"),
    "{xml}"
  );
  assert!(xml.contains("start_2_columns"), "{xml}");
}

/// pstricks.sty:187-207 `\newrgbcolor{n}{…}` & co. define the switch macro
/// `\n` (= `\color{n}`) as well as the color; the binding only registered
/// the color (Perl's identical line is dead code under its raw load), so
/// `{\deepblue text}` was an undefined macro (ffslides-doc). Batch 56bh.
#[test]
fn pstricks_color_definitions_define_the_switch_macro() {
  let tex = "\\documentclass{article}\n\\usepackage{pstricks}\n\\newrgbcolor{deepblue}{.2 .2 .5}\\newgray{midgray}{.5}\n\\begin{document}\n{\\deepblue text is blue} and {\\midgray gray}.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("color=\"#333380\""), "{xml}");
  assert!(xml.contains("color=\"#808080\""), "{xml}");
}

/// tex.web §440: a tab or row end met by a number scan's lookahead is an
/// unexpandable token that ends the scan; the alignment action runs when
/// the main loop re-reads it. `\ifnum1<\nb\\\hline\fi` in an m-cell
/// (tabularcalc.sty:428 with a macro operand) broke the row mid-scan and
/// desynchronized the cell's frame (tabularcalc ×3, floatrow-rus, fepslatex).
/// Batch 56bi.
#[test]
fn number_scan_lookahead_does_not_fire_alignment_actions() {
  let tex = "\\documentclass{article}\n\\usepackage{array}\n\\newcommand\\nb{1}\\newcommand\\nc{2}\n\\begin{document}\n\\begin{tabular}{|>{\\centering\\arraybackslash}m{1cm}|}\\hline\nd\\ifnum1<\\nb\\\\\\hline\\fi \\\\ \\hline\n\\end{tabular}\n\\begin{tabular}{|c|}\\hline\ne\\ifnum1<\\nc\\\\\\hline\\fi f \\\\ \\hline\n\\end{tabular}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
  // The false branch skipped its `\\`: one row; the true branch kept it: two rows.
  let second = xml.rfind("<tabular").unwrap();
  assert_eq!(xml[..second].matches("<tr").count(), 1, "{xml}");
  assert_eq!(xml[second..].matches("<tr").count(), 2, "{xml}");
}

/// Batch 56bj: `\nopagecolor` (color.sty:110, a driver info message) and
/// beamer's `\trans…<overlay>[options]` family (beamerbaseoverlay.sty:755-774,
/// `\hypersetup{pdfpagetransition=…}` viewer effects) are no-ops that consume their
/// arguments. Witnesses: ffslides-doc, sample-bxcjkjatype-beamer.
#[test]
fn beamer_transitions_and_nopagecolor_are_noops() {
  let tex = "\\documentclass{beamer}\n\\begin{document}\n\\begin{frame}\n\\transdissolve<2>[duration=0.5]\\transwipe[direction=90]\\transduration<3>{2}\\transglitter\nkept\n\\end{frame}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("kept"), "{xml}");
  assert!(
    !xml.contains("duration")
      && !xml.contains("direction")
      && !xml.contains("transdissolve")
      && !xml.contains("Dissolve"),
    "{xml}"
  );
  let tex = "\\documentclass{article}\n\\usepackage{xcolor}\n\\begin{document}\n\\pagecolor{yellow}\\nopagecolor text\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("text") && !xml.contains("nopagecolor"),
    "{xml}"
  );
}

/// Batch 56bk: an `\item` digested inside a captured box (`minipage` in a
/// list item; tikz-ext-manual `{arrowtip}`) gets an auto-opened
/// `ltx:itemize` in the box instead of `malformed:ltx:item` (SHARED with
/// Perl; surpass). Repro `repros/boxes-groups/item_in_minipage_inside_list.tex`.
#[test]
fn item_in_a_captured_box_gets_an_itemize() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{itemize}\n\\item first\n\\begin{minipage}[t]{3cm}\n\\item boxed\n\\end{minipage}\n\\end{itemize}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<itemize").count(), 2, "{xml}");
  let inner = xml.rfind("<itemize").unwrap();
  let block = xml.find("ltx_minipage").unwrap();
  assert!(
    block < inner,
    "the inner list sits inside the minipage block: {xml}"
  );
  assert!(xml[inner..].contains("boxed"), "{xml}");
  // An item in an inline box still reports: \fbox holds text, not a list.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{itemize}\n\\item first \\fbox{\\item boxed}\n\\end{itemize}\n\\end{document}\n";
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
}

/// Batch 56bl (KERNEL_CAPABILITIES K10): the pdfTeX byte mouth, package-scoped.
/// kotexutf runs raw under it: josa control symbols over a lead byte
/// (kotexutf.sty `\DeclareRobustCommand*\^^ec[2]`) resolve, hangul funnels
/// through `\unihangulchar` (kotexutf-core.tex:213) and comes out as the
/// character, accents in the same document survive. Witnesses kotex-utf-doc,
/// kotex-doc, cjk-ko-doc.
#[test]
fn kotexutf_runs_under_the_byte_mouth() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("kotexutf.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{kotexutf}\n\\begin{document}\n한글\\은 문서\\를 café 만든다.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("한글은 문서를 café 만든다."), "{xml}");
}

/// Batch 56bl: dhucs-trivcj.sty:18 `\ifx 가가` is FALSE under the byte mouth
/// (two different bytes), so its legacy branch defines the `{japanese}`
/// environment (dhucs-trivcj.sty:117) instead of `\edef`-ing an undefined
/// luatexko `\japanese` (kotex-doc ×3).
#[test]
fn dhucs_trivcj_takes_the_byte_branch() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("dhucs-trivcj.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{kotexutf}\n\\usepackage{dhucs-trivcj}\n\\begin{document}\n\\begin{japanese}日本語\\end{japanese} 한글\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("日本語 한글"), "{xml}");
}

/// Batch 56bl: CJK's octet readers (`\CJK@XXX`, CJKutf8.sty:41-84) receive
/// real bytes under the byte mouth and emit the character; `\CJK@input`
/// (CJK.sty:76) feeds `UTF8.bdg`'s `\CJK@namedef` family
/// (sample-bxcjkjatype-beamer via bxcjkjatype.sty:932).
#[test]
fn cjk_octet_readers_emit_the_character() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("UTF8.bdg") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{CJKutf8}\n\\makeatletter\n\\begingroup\\CJK@input{UTF8.bdg}\\endgroup\n\\makeatother\n\\begin{document}\n\\begin{CJK}{UTF8}{mj}한글 café\\end{CJK}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("한글 café"), "{xml}");
}

/// Batch 56bl: under the byte mouth `\DeclareUnicodeCharacter{00ED}` defines
/// `\u8:<bytes>` (utf8.def:253-265) and leaves U+00ED — the lead byte of
/// `한` — alone; t1enc.dfu loaded after kotexutf (kotex-doc `[T1]{fontenc}`)
/// turned every hangul lead byte into `\'\i` (30 errors, mojibake title).
#[test]
fn byte_mouth_declare_unicode_character_defines_u8_names() {
  let tex = "\\documentclass{article}\n\\usepackage{kotexutf}\n\\usepackage[T1]{fontenc}\n\\title{한국어 텍}\n\\begin{document}\n\\maketitle\n본문 café.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("한국어 텍") && xml.contains("본문 café."),
    "{xml}"
  );
}

/// Batch 56bl: under the byte mouth a name has two spellings — the bytes
/// (`\detokenize`, cncolours.sty:21's comma list) and the characters the
/// octet readers emit; `color_sty::color_key` + `def_color` share one key
/// (pgfornament-han-doc: 16 → 1001 "Can't find color named '乌黑'" before).
#[test]
fn byte_mouth_color_names_share_one_key() {
  let tex = "\\documentclass{article}\n\\usepackage{xcolor}\n\\usepackage{CJKutf8}\n\\definecolorset{RGB}{}{}{素,240,240,240;青翠,0,224,158}\n\\begin{document}\n\\begin{CJK}{UTF8}{gbsn}\n\\expandafter\\definecolor\\expandafter{\\detokenize{乌黑}}{rgb}{0.1,0.1,0.1}\n\\textcolor{乌黑}{墨}\\textcolor{素}{a}\\textcolor{青翠}{b}\n\\end{CJK}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("color=\"#1A1A1A\"") && xml.contains("墨"),
    "{xml}"
  );
  // 素 = E7 B4 A0, 青翠 = … BF A0: byte 0xA0 is Unicode whitespace, so a
  // trim before the decode ate the last byte (`Token::from`, the colour
  // set parser).
  assert!(
    xml.contains("color=\"#F0F0F0\"") && xml.contains("color=\"#00E09E\""),
    "{xml}"
  );
}

/// Batch 56bl: strings stringified from byte tokens (hyperref's `pdftitle`
/// → `<ltx:rdf content=…>`, a theorem tag) are decoded at the document
/// boundary (`mouth::decode_byte_mouth_runs` in `open_text`/`set_attribute`;
/// cjk-ko-doc's `content="cjk-ko ê°ë¨…"`, inkpaper-cn's `<tag>å®ç 3.1</tag>`).
#[test]
fn byte_mouth_strings_decode_at_the_document_boundary() {
  let tex = "\\documentclass{article}\n\\usepackage{kotexutf}\n\\usepackage{hyperref}\n\\title{한국어 텍}\n\\author{김강수}\n\\begin{document}\n\\maketitle\n본문.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("한국어 텍"), "{xml}");
  assert!(
    !xml.contains('ê') && !xml.contains('í'),
    "byte spellings leaked: {xml}"
  );
}

/// Batch 56bm: the inline `\endnote` still opens `\jobname.ent`
/// (endnotes.sty:309 invariant), so a raw `\theendnotes` ending in
/// `\input{\jobname.ent}` (latex-doc-ptr.sty:86) finds the file.
#[test]
fn endnote_opens_the_ent_file_for_a_raw_theendnotes() {
  let tex = "\\documentclass{article}\n\\usepackage{endnotes}\n\\let\\footnote=\\endnote\n\\makeatletter\\def\\theendnotes{\\immediate\\closeout\\@enotes \\input{\\jobname.ent}}\\makeatother\n\\begin{document}\nText\\footnote{a note}.\n\\theendnotes\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("role=\"endnote\"").count(), 1, "{xml}");
}

/// Batch 56bo (`Stored::Opaque`): an `{animateinline}` deferred inside a
/// float constructs after the animate that follows it in the source; its
/// the frame count (`content=`) rides on its own whatsit (the shared context in
/// `anim_ctx`), so construction order no longer matters (the old LIFO pop
/// at construction handed the float's animate the later count).
#[test]
fn animateinline_in_a_float_keeps_its_own_frame_count() {
  let tex = "\\documentclass{article}\n\\usepackage{animate}\n\\begin{document}\n\\begin{figure}\\begin{animateinline}{2}A\\newframe B\\newframe C\\end{animateinline}\\caption{f}\\end{figure}\nText \\begin{animateinline}{2}X\\end{animateinline} more.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let fig = xml.find("<figure").unwrap();
  let fig_end = xml[fig..].find("</figure>").unwrap() + fig;
  assert!(xml[fig..fig_end].contains("content=\"3\""), "{xml}");
  assert!(xml[fig_end..].contains("content=\"1\""), "{xml}");
}

/// Batch 56bn review: a colour defined under a whitespace-padded name is
/// stored and looked up under the same trimmed key (`def_color` and
/// `color_sty::color_key` must not drift).
#[test]
fn whitespace_padded_color_name_resolves() {
  let tex = "\\documentclass{article}\n\\usepackage{color}\n\\definecolor{ foo }{rgb}{1,0,0}\n\\begin{document}\n\\textcolor{foo}{hello}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("color=\"#FF0000\""), "{xml}");
}

/// Batch 56bp: `\@pass@ptions` (latex.ltx:18514) stores an option list by
/// `\protected@xdef` — a CJK byte in `[piecechar={C}{炮}]` under the byte
/// mouth expands to ctex's PROTECTED `\CTEX@char@nnn` and stops there;
/// full expansion ran its body into CJKspace's `\futurelet\CJK@next@token`
/// and stubbed the lookahead name (chinesechess, sweep 70).
#[test]
fn package_options_are_stored_by_protected_xdef() {
  // chinesechess itself reduced to a package that stores its options (`lxopt.sty`); ctex's
  // `fontset=none` keeps the fontset check (pdflatex: "fandol is unavailable") out. Note
  // (2026-09-28): a full expansion in `protected_xdef_options` no longer turns this, or the
  // original chinesechess document, red — the byte mouth now keeps ctex's character macros
  // protected on its own — and `piecechar` is also in the `<?latexml options?>` PI; this stays
  // the chinesechess shape guard. The discriminating input (robust and `\protected` macros in
  // an option) is RED: loader/package_options_keep_protected_macros.
  let tex = "\\documentclass{article}\n\\usepackage[scheme=chinese,fontset=none]{ctex}\n\\usepackage[piecechar={C}{炮}]{lxopt}\n\\begin{document}\n中文 x\n\\end{document}\n";
  let sty = "\\ProvidesPackage{lxopt}\n\\DeclareOption*{}\n\\ProcessOptions\\relax\n";
  let (stderr, xml) = convert_files_with(
    tex,
    &[("lxopt.sty", sty)],
    Some("[rawstyles,rawclasses]latexml.sty"),
  );
  assert!(!stderr.contains("CJK@next@token"), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("piecechar"), "{xml}");
  assert!(xml.contains("中文 x"), "{xml}");
}

/// Batch 56bp: the byte-mouth decoder is sequence-wise — a decoded Latin-1
/// character (é = U+00E9, the image of a 3-byte lead) directly before a
/// fresh CJK sequence no longer poisons the whole run into an invalid
/// `from_utf8` that left both as bytes; and the text merge decodes only the
/// pending tail (`mouth::byte_mouth_pending_tail`).
#[test]
fn byte_mouth_latin1_before_cjk_decodes_sequence_wise() {
  let tex = "\\documentclass{article}\n\\usepackage{kotexutf}\n\\begin{document}\ncafé한글 naïve문서\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("café한글 naïve문서"), "{xml}");
}

/// Batch 56bp (manyind/mindsample): the index entry is a `.idx` string that
/// `\printindex` digests in order — the pre-expansion passes an undefined
/// control word inert (`\noexpand`), so manyind.sty:100/119's
/// `\protect\def \nwletre {…}` defines `\nwletre` before its use instead
/// of the bare name being force-expanded into `<ltx:ERROR/>`.
#[test]
fn index_entry_passes_undefined_words_inert() {
  let tex = "\\documentclass{book}\n\\usepackage{manyind}\n\\makeindex\n\\begin{document}\n\\setindex{main}\na\\index{\\\"N@\\protect\\nxtletre \\protect\\def \\nwletre {\\\"O}\\gobblepageref}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<indexmark"), "{xml}");
}

/// Batch 56bp: the sort key is a makeindex STRING — an undefined control
/// word in it is the literal characters (mindsample.tex:208
/// `\index{\AB@\relax…}`), and an undefined control word still carrying `@` after
/// the `\protected@write` expansion is re-read with document catcodes
/// (`@` OTHER), so `\AB@` is key `\AB` + separator, not one undefined
/// name (Perl's style-catcode re-tokenize; KPE #83).
#[test]
fn index_sort_key_undefined_word_is_text() {
  let tex = "\\documentclass{article}\n\\usepackage{makeidx}\\makeindex\n\\begin{document}\nA\\index{\\AB@\\relax x}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("key=\"\\AB\""), "{xml}");
}

/// Batch 56bq: a scanned dimension past the i64 range is TeX's arith_error
/// (tex.web §460 "Dimension too large" → max_dimen), not a wrapped i128
/// product (`100899720527872.0pt`; chinesechess.sty:2016-2020's coffin
/// scale by a `\dim_ratio:nn` with a zero box dimension). Conservative:
/// the clamp fires only past the i64 range — LaTeXML's headroom above
/// max_dimen (pgf intermediates re-scanned as `<factor><internal dimen>`)
/// stays, so `200000000000\dimen0` at -1pt prints as computed.
#[test]
fn dimension_overflow_clamps_to_max_dimen() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\dimen0=-16383pt\n\\dimen2=2000000000000000\\dimen0\nB=[\\the\\dimen2]\\par\n\\dimen1=\\dimexpr\\dimen0*100000000000\\relax\nA=[\\the\\dimen1]\\par\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("B=[-16383.99998pt]"), "{xml}");
  assert!(xml.contains("A=[-16383.99998pt]"), "{xml}");
}

/// Batch 56bq: `\@raw@opt@<file>` exists after an option-less load too
/// (latex.ltx:18521-18523 `\gdef`s it empty), so scrhack.sty:284-293's
/// forwarding of `\use:c{@raw@opt@setspace.sty}` into setspaceenhanced's
/// options is an empty first option, not the name `\@raw@opt@setspace .sty`
/// (ijsra, sweep 72).
#[test]
fn raw_option_record_exists_for_an_optionless_load() {
  let tex = "\\documentclass{article}\n\\usepackage{setspace}\n\\makeatletter\n\\RequirePackage[\\csname @raw@opt@setspace.sty\\endcsname,byselectfont,keepfontsize]{setspaceenhanced}\n\\edef\\x{\\@ifundefined{@raw@opt@setspace.sty}{UNDEF}{DEF}}\\typeout{RAW=\\x}\n\\makeatother\n\\begin{document}\nx\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("RAW=DEF"), "{stderr}");
  assert!(xml.contains("<p>x</p>"), "{xml}");
}

/// Batch 56bv: a forest node label with `#`, `&` or an alignment primitive is
/// not plain horizontal content — it is kept as a string instead of being
/// digested (forest-doc.tex:1055 `[…=#1]`, :3142 a tabular in a node), while
/// a plain label still digests (`$x^2$` → `<Math>`); `\bracketResume` exists.
#[test]
fn forest_non_label_streams_stay_strings() {
  let tex = "\\documentclass{article}\n\\usepackage{forest}\n\\begin{document}\n\\begin{forest}\n[root [a\\#1b] [x&y\\\\\\hline z] [$x^2$] [one\\\\two, align=center]]\n\\end{forest}\n\\end{document}\n".replace("a\\#1b", "a#1b");
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<Math"), "{xml}");
  assert!(xml.contains("a#1b"), "{xml}");
  // A multi-line node label still digests its `\\` to a break.
  assert!(xml.contains("<break"), "{xml}");
}

/// Batch 56bx: a `filecontents` capture ends at `\end{<current
/// environment>}` (latex.ltx:19047), so a wrapper environment that runs the
/// bare `\filecontents*` command (latexdemo.sty:97-101 `DefineCode`) ends at
/// its own `\end`; the fixed `\end{filecontents*}` marker swallowed the rest
/// of latex4wp (63% of the manual) with no diagnostic.
#[test]
fn filecontents_inside_a_wrapper_environment_ends_at_the_wrappers_end() {
  let tex = "\\documentclass{article}\n\\newenvironment{DemoCode}{\\csname filecontents*\\endcsname[overwrite]{democode}}{\\csname endfilecontents*\\endcsname}\n\\begin{document}\nBEFOREWORD\n\\begin{DemoCode}\ninside \\textbf{demo} body\n\\end{DemoCode}\nAFTERWORD\n\\input{democode}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("BEFOREWORD"), "{xml}");
  assert!(xml.contains("AFTERWORD"), "{xml}");
  // The captured body is the wrapper's content, readable back by name.
  assert!(xml.contains("demo"), "{xml}");
  assert!(
    stderr.contains("Cached filecontents for democode (1 lines)"),
    "{stderr}"
  );
}

/// Batch 56bx: after `\begin{document}` the kernel's `\@preamblecmds` is
/// `\let` to `\@notprerr` (latex.ltx:9521-9522, :1228), which lppl.tex:44
/// probes to choose its running-document branch; the standalone branch
/// `\let\endLPPLicense\enddocument` ended beameruserguide at the license.
#[test]
fn preamblecmds_is_notprerr_inside_a_running_document() {
  let tex = "\\documentclass{article}\n\\begin{document}\nBEFOREWORD\n\\makeatletter\n\\ifx\\@preamblecmds\\@notprerr \\let\\LPPLtest\\bgroup \\let\\endLPPLtest\\egroup \\else \\let\\LPPLtest\\document \\let\\endLPPLtest\\enddocument \\fi\n\\makeatother\n\\begin{LPPLtest}\nINSIDEWORD\n\\end{LPPLtest}\nAFTERWORD\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("INSIDEWORD"), "{xml}");
  assert!(xml.contains("AFTERWORD"), "{xml}");
}

/// Batch 56bx: `\vsplit` of a register holding a `\vbox` splits the box's
/// vertical list (tex.web §977) and stores the remainder back as a vbox;
/// treating the whatsit as one item swept the whole table into the
/// discard-top split of short-math-guide.tex:167 (`\splitlist`), losing
/// every amssymb symbol name at zero errors.
#[test]
fn vsplit_of_a_vbox_register_splits_its_lines() {
  let tex = "\\documentclass{article}\\makeatletter\n\\newcount\\cols\\newcount\\curcol\n\\def\\do{\\advance\\curcol1 \\setbox2=\\vsplit0 to\\dimen@\n  \\vtop{\\unvbox2}\\ifdim\\ht0>\\z@\\expandafter\\do\\fi}\n\\begin{document}\n\\setbox0\\vbox{\\hbox{alpha}\\hbox{beta}\\hbox{gamma}\\hbox{delta}}%\n\\cols=2 \\dimen@\\ht0 \\divide\\dimen@\\cols\n\\setbox2=\\vsplit0 to\\baselineskip\nBEGIN\\hbox to\\textwidth{\\curcol=0 \\do\\hfil}END\n\\makeatother\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  // The discard-top split takes the first line only; the rest survives.
  for w in ["beta", "gamma", "delta"] {
    assert!(xml.contains(w), "missing {w}:\n{xml}");
  }
}

/// Batch 56bx: `\blindtext` without babel is the Latin default
/// (blindtext.sty:356-369); the binding's forced `babel[english]` made
/// `\extrasenglish` swap in the English text (Perl-origin, #228).
#[test]
fn blindtext_default_is_latin_without_babel() {
  let tex = "\\documentclass{article}\n\\usepackage{blindtext}\n\\begin{document}\n\\blindtext\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("consectetuer"), "{xml}");
  assert!(!xml.contains("without a meaning"), "{xml}");
}

/// Batch 56bu: unicode-math re-binds the active math prime at begin-document
/// (unicode-math-luatex.sty:3405/3426), so hanging.sty:85/101's global
/// `\gdef'` no longer recurses on `$f'(x)$` (kaytannollista, 420 s).
#[test]
fn unicode_math_rebinds_the_math_prime_at_begin_document() {
  let tex = "\\documentclass{book}\n\\usepackage{fontspec}\n\\usepackage{amsmath}\n\\usepackage[math-style=ISO]{unicode-math}\n\\usepackage{hanging}\n\\begin{document}\n$f'(x)=6x-2$\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains("\u{2032}"), "{xml}");
}

/// Batch 56bt: an accent pushed through `\edef` + `\scantokens` (tkzexample.sty:
/// 352-357, a Latin-1 `é` in grafcet.tex:1136) round-trips because the
/// accent's inner is a letters-only private name (`\lxaccentacute`) that
/// re-tokenizes to itself under document catcodes; `\lx@accent@'` split
/// into the undefined `\lx` (sweep 74).
#[test]
fn accent_inner_survives_a_scantokens_round_trip() {
  let tex = "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\begin{document}\n\\edef\\x{caf\\'e}\\scantokens\\expandafter{\\x}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>café</p>"), "{xml}");
}

/// Batch 56bt: a delimited reader stores a matched group with its OWN
/// close token (tex.web §392) — chemfig scans `(…)` submols with `(`/`)`
/// as catcodes 1/2 (chemfig.tex:1315-1324); a canonical `}` in place of the
/// `)` re-tokenized as a real end-brace in the next `\scantokens` and
/// unbalanced the molecule (chemexec/chemnum, sweep 74).
#[test]
fn delimited_read_keeps_a_groups_own_close_token() {
  let tex = "\\documentclass{article}\n\\usepackage{chemfig}\n\\begin{document}\n\\chemfig{A(-B)(-C)D}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<svg:svg") || xml.contains("<picture"),
    "{xml}"
  );
}

/// Batch 56bs: eTeX reads the `\everyeof` payload as the pseudo-file's
/// LAST tokens (tex.web §362 + etex.ch `every_eof`/`eof_seen`) — at the
/// file's own level, before it closes — so a delimited scan started inside
/// a `\scantokens` sees its terminator.
#[test]
fn scantokens_everyeof_is_the_files_last_tokens() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\everyeof{\\ENDEOF}%\n\\def\\grab#1\\ENDEOF{[GOT:\\detokenize{#1}]}%\n\\expandafter\\grab\\scantokens{ABC}%\nDONE\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[GOT:ABC ]DONE") || xml.contains("[GOT:ABC]DONE"),
    "{xml}"
  );
}

/// Batch 56bs: spreadtab's macro-functions end their `\scantokens` scan on
/// `\everyeof{\ST_nil}` (spreadtab.sty:1493-1504); without the payload
/// every `sum(...)` cell re-pushed itself forever (spreadtab-en/-fr, 420 s).
#[test]
fn spreadtab_function_cell_evaluates() {
  let tex = "\\documentclass{article}\n\\usepackage{spreadtab}\n\\begin{document}\n\\begin{spreadtab}{{tabular}{c|c|c}}\n1 & 2 & :={sum(a1:b1)} \\\\\n\\end{spreadtab}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">3</td>"), "{xml}");
}

/// Batch 56bs: `DefAccent!`'s inner is a private name, so babel-spanish's
/// `\let\es@save@dot\.` + `\DeclareRobustCommand*\.` (spanish.ldf:329-331)
/// keeps the original accent, as LaTeX's `\OT1\.` inner does; the shared
/// `\. ` inner made `\.o` cycle forever (latexsheet-esmx, 420 s).
#[test]
fn babel_spanish_dot_accent_does_not_loop() {
  let tex = "\\documentclass{article}\n\\usepackage[spanish]{babel}\n\\begin{document}\n\\.o y \\.a fin.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ȯ y ȧ fin."), "{xml}");
}

/// Batch 56br: an option value carrying a control word from package/class
/// code (`@` a letter there — xebaposter.cls passes the register
/// `\xebaposter@finalpaperwidth` to geometry) survives the string round trip
/// of the option store; re-read with `@` OTHER it split into an undefined
/// `\xebaposter` (sweep 73).
#[test]
fn option_value_keeps_an_at_name_token() {
  let tex = "\\documentclass{article}\n\\makeatletter\\newlength\\my@w\\setlength\\my@w{5in}\n\\usepackage[paperwidth=\\my@w,paperheight=7in]{geometry}\n\\begin{document}\nW=[\\the\\Gm@pw]\\makeatother\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("W=[361.34999pt]"), "{xml}");
}

/// Batch 56bq: a package option whose handler the CLASS left `\let` to
/// `\relax` (`landscape`, `a4paper` after article's `\ProcessOptions`) is
/// undefined for latex.ltx `\@use@ption`'s `\@ifundefined`, so the
/// package's `\DeclareOption*` default sees it (geometry swallowed both;
/// Perl shares).
#[test]
fn class_cleared_option_handler_reaches_the_package_default() {
  let tex = "\\begin{filecontents}[overwrite]{lxoptprobe.sty}\n\\DeclareOption*{\\typeout{OPT=[\\CurrentOption]}}\n\\ProcessOptions*\n\\end{filecontents}\n\\documentclass{article}\n\\usepackage[landscape,a4paper,x={a,b}]{lxoptprobe}\n\\begin{document}\nx\n\\end{document}\n";
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for opt in ["OPT=[landscape]", "OPT=[a4paper]", "OPT=[x={a,b}]"] {
    assert!(stderr.contains(opt), "{opt} missing: {stderr}");
  }
}

/// Batch 56bq: geometry's `landscape` is a flag applied to the portrait
/// paper (geometry.sty:34/471-473), so `landscape,a4paper` and
/// `a4paper,landscape` agree; the eager swap of the kernel registers was
/// order-dependent (elzcards root-causer).
#[test]
fn geometry_landscape_is_order_independent() {
  let mk = |opts: &str| {
    format!(
      "\\documentclass{{article}}\n\\usepackage[{opts}]{{geometry}}\n\\begin{{document}}\n\\makeatletter W=[\\the\\Gm@pw] H=[\\the\\Gm@ph]\\makeatother\n\\end{{document}}\n"
    )
  };
  let (e1, x1) = convert(&mk("landscape,a4paper"), true);
  let (e2, x2) = convert(&mk("a4paper,landscape"), true);
  assert_eq!(error_count(&e1) + error_count(&e2), 0, "{e1}\n{e2}");
  assert!(x1.contains("W=[845.04684pt] H=[597.50787pt]"), "{x1}");
  assert!(x2.contains("W=[845.04684pt] H=[597.50787pt]"), "{x2}");
}

/// Batch 56bq: a package option's value keeps its argument tokens
/// (latex.ltx:18514), braces grouping — `vmargin={3mm,7mm}` reaches
/// `\setkeys` as one pair instead of splitting at its comma with OTHER
/// braces ("Missing number" ×4; elzcards root-causer).
#[test]
fn package_option_value_keeps_its_braces() {
  let tex = "\\documentclass{article}\n\\usepackage[vmargin={3mm,7mm}]{geometry}\n\\begin{document}\n\\makeatletter T=[\\the\\Gm@t] B=[\\the\\Gm@b]\\makeatother\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Missing number"), "{stderr}");
  assert!(xml.contains("T=[8.53581pt] B=[19.91692pt]"), "{xml}");
}

/// The hoist climb never leaves a list (Perl floats nothing out of one —
/// latex_constructs.pool.ltxml:3936 cannot cross a drawing's `svg:g`). A
/// bibliography alone in a tcolorbox inside an `\item` climbed to the
/// section, closed the itemize and nested the remaining items
/// (biblatex-ext.tex:1301/1326). The one remaining error is Perl's own
/// `ltx:bibliography` in `ltx:block`.
#[test]
fn bibliography_in_a_list_item_box_keeps_the_list() {
  let tex = r"\documentclass{article}
\usepackage[skins]{tcolorbox}
\newtcolorbox{bibexample}{enhanced}
\begin{document}
\begin{itemize}
\item First option.
\begin{bibexample}
\begin{thebibliography}{9}
\bibitem{a} Reference A.
\end{thebibliography}
\end{bibexample}
\item Second option.
\item Third option.
\end{itemize}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Error:malformed:ltx:bibliography"),
    "{stderr}"
  );
  assert!(!stderr.contains("malformed:ltx:item"), "{stderr}");
  assert_eq!(xml.matches("<itemize").count(), 1, "{xml}");
  assert_eq!(xml.matches("<item ").count(), 3, "{xml}");
  // The three items are siblings: no `<item` opens while another is open.
  let mut depth = 0i32;
  for tag in regex::Regex::new(r"<item |</item>")
    .unwrap()
    .find_iter(&xml)
  {
    if tag.as_str() == "<item " {
      depth += 1;
      assert_eq!(depth, 1, "nested item:\n{xml}");
    } else {
      depth -= 1;
    }
  }
}

/// The outermost list element of `xml`, with `xml:id`s and inter-tag
/// whitespace removed, for whole-element comparisons.
fn outer_list(xml: &str, tag: &str) -> String {
  let start = xml.find(&format!("<{tag}")).unwrap();
  let close = format!("</{tag}>");
  let end = xml.rfind(&close).unwrap() + close.len();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  gaps
    .replace_all(&ids.replace_all(&xml[start..end], ""), "><")
    .into_owned()
}

/// Batch 56gd (OXIDIZED_DESIGN #261): a block box opened in a list before an
/// `\item` has no room in the list's `item*` model; it gets an auto-opened
/// `item` → `para`, and the next `\item` closes it. Witnesses colorframed-doc
/// (`shaded` around each `\item`, 6 schema errors) and tableaux/exemples (a
/// minipage of `\item`s beside a table minipage, 2). Since 57f the minipage starts the
/// auto-item's paragraph (latex.ltx `\@iiiminipage` `\leavevmode`).
#[test]
fn block_in_a_list_before_an_item_gets_an_auto_item() {
  let tex = r"\documentclass{article}
\begin{document}
\begin{itemize}
\begin{minipage}{3cm}
Boxed text.
\end{minipage}
\item Real item.
\end{itemize}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    outer_list(&xml, "itemize"),
    concat!(
      r#"<itemize><item><para><p><inline-block class="ltx_minipage" vattach="middle" width="85.4pt">"#,
      r#"<p>Boxed text.</p></inline-block></p></para></item>"#,
      r#"<item><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags>"#,
      r#"<para><p>Real item.</p></para></item></itemize>"#
    ),
    "{xml}"
  );
  // Since 57f a minipage before an `\item` starts a paragraph (the auto-item comes from opening
  // it); a box that is no paragraph material still takes the #261 branch: a `\vbox`.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{itemize}\\vbox{\\hsize=3cm Boxed vbox.}\\item Real item A.\\end{itemize}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    outer_list(&xml, "itemize"),
    concat!(
      r#"<itemize><item><para><block vattach="bottom"><p>Boxed vbox.</p></block></para></item>"#,
      r#"<item><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags>"#,
      r#"<para><p>Real item A.</p></para></item></itemize>"#
    ),
    "{xml}"
  );
  // The tableaux shape: a minipage of `\item`s beside a second minipage — since 57f side by side
  // in the auto-item's one paragraph, as TeX sets them.
  let tex = r"\documentclass{article}
\begin{document}
\begin{enumerate}
\begin{minipage}{3cm}
\item First.
\end{minipage}\hfill
\begin{minipage}{3cm}
Side.
\end{minipage}
\end{enumerate}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    outer_list(&xml, "enumerate"),
    concat!(
      r#"<enumerate><item><para><p><inline-block class="ltx_minipage" vattach="middle" width="85.4pt">"#,
      r#"<itemize><item><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag>"#,
      r#"</tags><para><p>First.</p></para></item></itemize></inline-block>"#,
      r#"<inline-block class="ltx_minipage" vattach="middle" width="85.4pt"><p>Side.</p></inline-block>"#,
      r#"</p></para></item></enumerate>"#
    ),
    "{xml}"
  );
}

/// Batch 56gk (OXIDIZED_DESIGN #266): an 8-bit input byte that a package hands
/// back to the letter class (russ.sty:58-63 recatcodes cp1251's Cyrillic bytes so
/// words can name commands) bypasses its active inputenc definition. It is
/// decoded where it enters, as utf8 input is: the character its own declaration
/// produces (`\DeclareInputText{199}{\CYRZ}` → `\T2A\CYRZ` → `З`). Before, the
/// byte stayed its Latin-1 image: `Çäðàâåé` (russ_doc recall 69.9 → 82.7).
/// `^^c7` is byte 199 exactly as a file byte would be.
#[test]
fn recatcoded_input_bytes_decode_through_their_declaration() {
  let tex = r"\documentclass{article}
\usepackage[cp1251]{inputenc}
\usepackage[T2A]{fontenc}
\catcode`^^c7=11 \catcode`^^e4=11 \catcode`^^f0=11 \catcode`^^e0=11
\catcode`^^e2=11 \catcode`^^e5=11 \catcode`^^e9=11
\begin{document}
^^c7^^e4^^f0^^e0^^e2^^e5^^e9, ^^e4^^e0.

Café na é.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Здравей, да.</p>"), "{xml}");
  // A valid UTF-8 line is not bytes: its `é` (U+00E9, code 233 recatcoded to a
  // letter above) stays `é`, not cp1251 byte 233 `й`.
  assert!(xml.contains("<p>Café na é.</p>"), "{xml}");
  // Controls: an active byte and an accent composite are untouched — latin1 + T1
  // `\"y` stays `ÿ` (T1 slot 255 is `ß`), `\ss` stays `ß`.
  let tex = r#"\documentclass{article}
\usepackage[latin1]{inputenc}
\usepackage[T1]{fontenc}
\begin{document}
A[\"y] C[\ss] D[\"u]
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>A[ÿ] C[ß] D[ü]</p>"), "{xml}");
}

/// Batch 56hh (OXIDIZED_DESIGN #265): a class-body replay that raises an
/// error is dropped. A class that keeps its authors in its own store (pos.sty:88,
/// an expl3 seq the locked `\author` never fills) pops that empty store in its
/// `\maketitle` and expands `\q_no_value` (2× "Token \q_no_value expands into
/// itself!" in 20 papers of arXiv 2605, e.g. 2605.02049, and 154 of 2606;
/// pdflatex and Perl clean). The replay's diagnostics are held and dropped with
/// it; the frontmatter still carries the title and author.
#[test]
fn class_maketitle_replay_that_errors_is_dropped() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/class_maketitle_own_author_store_pos.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("q_no_value"), "{stderr}");
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  assert!(
    flat.contains(concat!(
      r#"<title>T</title><creator role="author"><personname>Ann</personname></creator>"#,
      r#"<para><p>Body text.</p></para>"#
    )),
    "{flat}"
  );
}

/// titling's `\pretitle`…`\postdate` hooks: Perl's binding stores them and
/// never reads them (titling.sty.ltxml:28-33), so a class that builds its
/// title page in them lost it (lion-msc/minimal 24 % → 70 % PDF recall).
/// The binding's `\@maketitle` is titling.sty:166-180's field/hook sequence,
/// deposited with the fields emptied: hook text lands after the frontmatter,
/// titling's default (layout-only) hooks add nothing. OXIDIZED_DESIGN #275.
#[test]
fn titling_hooks_reach_the_output() {
  let flat = |xml: &str| {
    let gaps = regex::Regex::new(r">\s+<").unwrap();
    let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
    gaps
      .replace_all(&ids.replace_all(xml, ""), "><")
      .into_owned()
  };
  let head = r#"<title>A Title</title><creator role="author"><personname>Ann Author</personname></creator><date role="creation">2026</date>"#;
  let tex = r"\documentclass{article}
\usepackage{titling}
\pretitle{\begin{center}Master Thesis\par\LARGE}
\posttitle{\par\end{center}in Physics\par}
\title{A Title}\author{Ann Author}\date{2026}
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let want = format!(
    r#"{head}<para><p align="center">Master Thesis</p></para><para><p>in Physics</p></para><para><p>Body.</p></para>"#
  );
  assert!(flat(&xml).contains(&want), "{}", flat(&xml));
  // The default hooks typeset only layout around the (emptied) fields.
  let tex = tex
    .replace("\\pretitle{\\begin{center}Master Thesis\\par\\LARGE}\n", "")
    .replace("\\posttitle{\\par\\end{center}in Physics\\par}\n", "");
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    flat(&xml).contains(&format!("{head}<para><p>Body.</p></para>")),
    "{}",
    flat(&xml)
  );
}

/// Keys the packages define but the bindings did not register, which the
/// Rust port reports as `Warning:undefined` (Perl: an Info): siunitx's
/// `detect-all`/`detect-none` meta choices (siunitx-v2.sty:393-437; 34
/// papers of arXiv 2605, e.g. 2605.00471) and hyperref.sty's newer `Hyp`
/// keys (`allcolors` in 4 papers of 2606, e.g. 2606.17809).
#[test]
fn package_keys_the_bindings_accept() {
  let tex = r"\documentclass{article}
\usepackage{siunitx}
\usepackage[allcolors=blue,linktoc=all,pdfborderstyle={/S/U/W 1}]{hyperref}
\sisetup{detect-all}
\hypersetup{allcolors=red,pdfusetitle}
\begin{document}
\SI{1}{\metre}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"text="1 * meter""#],
    r##"<Math mode="inline" tex="1\text{\,}\mathrm{m}" text="1 * meter" xml:id="p1.m1"><XMath><XMApp><XMText meaning="times" role="MULOP" xml:id="p1.m1.1"> </XMText><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok class="ltx_unit" meaning="meter" role="ID">m</XMTok></XMApp></XMath></Math>"##,
  );
}

/// compsci.sty:510's `\code` (verbatim typewriter through url's `\Url`) is
/// refused because modern doc.sty:623 made `\code` the identity, so the
/// frankenstein manuals executed the macros they document: `\cs\Wrapquotes`
/// ran titles.sty's quote wrapper, whose look-ahead looped
/// (`Fatal:Stomach:Recursion`). A `file/compsci.sty/after` first-aid hook
/// restores compsci's `\code` (OXIDIZED_DESIGN #276; 12 manuals 204 → 1
/// errors, titles and abbrevs 0 → 97 % PDF recall).
#[test]
fn compsci_code_is_verbatim() {
  if !kpsewhich_has("compsci.sty") {
    return;
  }
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/loader/compsci_code_identity_titles.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "ref",
    &[r#"href="\Wrapquotes""#],
    r##"<ref class="ltx_nolink ltx_Url" font="typewriter" href="\Wrapquotes">\Wrapquotes</ref>"##,
  );
}

/// natbib's `\setcitestyle` acts on the words it knows and ignores the rest
/// (natbib.sty:303-335): a package option passed there (`sort&compress`,
/// 2606.03886) warned "unknown KeyVals key" and — through Perl's fall-through
/// to authoryear — cost the `numbers` style; natbib's own `colon` (:315-316,
/// the `;` separator) was not known at all.
#[test]
fn natbib_setcitestyle_ignores_unknown_words() {
  let tex = r"\documentclass{article}
\usepackage{natbib}
\setcitestyle{numbers,sort&compress,colon,square}
\begin{document}
See \citep{a,b}.
\begin{thebibliography}{2}
\bibitem{a} A. Author. First. 2001.
\bibitem{b} B. Author. Second. 2002.
\end{thebibliography}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "cite",
    &[],
    r##"<cite class="ltx_citemacro_citep">[<bibref bibrefs="a,b" separator=";" show="Number" yyseparator=","/>]</cite>"##,
  );
}

/// `\NewTCBListing{…}{ s … }`: the absorbed star is a boolean to the options
/// (`\BooleanFalse` unstarred), never empty. The empty slot made
/// `IfBooleanT={#1}` run `\IfBooleanT{}` — l3's `cmd/if-boolean` expandable
/// error at every `\begin{macrodef}` (leporello-doc ×74, jsonparse-doc ×60,
/// visible under the strict 56gn delimiter check). Known gap: a starred call
/// reads `\BooleanFalse` too (the `*` is consumed as `\begin`-line leftover).
#[test]
fn tcb_listing_star_is_a_boolean() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/expl3/tcb_listing_star_is_a_boolean.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Slot: <text font=\"typewriter\">\\char\"0</text>."),
    "{xml}"
  );
}

/// The font-default codes are locked, as in Perl (latex_constructs.pool.ltxml:
/// 5146-5154). nunito.sty:64's `\renewcommand*{\rmdefault}{Nunito-TOsF}` took
/// effect, named a family `\selectfont` cannot map, and inside pgf's `\nullfont`
/// regime every node's plain text was dropped (bfh-ci DEMO-BFHSciPoster lost its
/// poster body).
#[test]
fn font_defaults_are_locked() {
  let (_, xml) = convert(
    "\\documentclass{article}\n\\renewcommand*{\\rmdefault}{Foo}\n\\begin{document}\n[\\rmdefault]\n\\end{document}\n",
    true,
  );
  assert!(xml.contains("<p>[cmr]</p>"), "{xml}");
  if !kpsewhich_has("nunito.sty") {
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/font_default_lock_nunito_node_text.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("PLAINNODE prose and"), "{xml}");
}

/// `\include{preamble.tex}` reads `preamble.tex`: latex.ltx:9557-9585 strips
/// a `.tex` extension before `\@include` appends one. Appending blindly
/// asked for `preamble.tex.tex`, and a whole preamble never loaded (arXiv
/// 2605.05505, 2605.24122 flooded into `Fatal:TooManyErrors`).
#[test]
fn include_strips_a_tex_extension() {
  let tex = "\\documentclass{article}\n\\include{sub.tex}\n\\begin{document}\nA \\mytext{ok}.\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex, &[("sub.tex", "\\newcommand\\mytext[1]{[#1]}\n")]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("sub.tex.tex"), "{stderr}");
  assert!(xml.contains("<p>A [ok].</p>"), "{xml}");
}

/// XeLaTeX documents convert under the default (pdfTeX) persona: the input is
/// Unicode-native, so `\RequireXeTeX` passes (Perl makes it a no-op) and
/// installs XeTeX's inter-character primitives as argument-reading no-ops for
/// the package that asked — ucharclasses' `\XeTeXcharclass` loops over whole
/// Unicode blocks ended in a 6.3 GB `alloc_failed` on the undefined primitive
/// (latexbangla). bidi, xeCJK and mathspec are bound. Without a request the
/// primitives stay undefined, since amsmath & co. probe them to detect XeTeX.
/// 11 papers of arXiv 2605 (2605.02089, 2605.16477 …) halted, 28 TL manuals.
#[test]
fn xetex_only_packages_load_under_the_default_persona() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/backend-persona/xelatex_packages_default_persona.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>A abc B 日本 C no.</p>"), "{xml}");
  if kpsewhich_has("ucharclasses.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{ucharclasses}\n\\begin{document}\nText \\ifx\\XeTeXcharclass\\undefined no\\else yes\\fi.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("<p>Text yes.</p>"), "{xml}");
  }
}

/// `\let\@left\left` + a `\left` wrapper (mathtools-style) must not loop: the
/// Rust `\left` trampoline re-emitted a constructor named `\@left`, which the
/// `\let` turned into the trampoline itself (arXiv 2605.21750,
/// `Fatal:Timeout:PushbackLimit`). TeX and Perl have no `\@left`.
#[test]
fn let_at_left_left_does_not_loop() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/let_at_left_left_loop.tex"
  );
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"role="OPEN" stretchy="true">(</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"role="CLOSE" stretchy="true">)</XMTok>"#),
    "{xml}"
  );
  // The `\mathopen{}\mathclose\bgroup\left` idiom is left unparsed, as in Perl,
  // not parsed into an empty `list@()` that drops the operands (56is).
  assert!(!xml.contains("list@()"), "{xml}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
}

/// bytedance_seed's binding defined `\author[]{}` as `\author{#2}` — itself: a
/// wall-clock hang on every ByteDance Seed paper (arXiv 2605.24117 and four
/// more; Perl and pdflatex complete). It is the frontmatter author API now.
#[test]
fn bytedance_author_is_a_creator() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bytedance_author_self_loop.tex"
  );
  let (stderr, xml) = convert_args(tex, &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<personname>Alice"), "{xml}");
}

/// Batch 56gj (OXIDIZED_DESIGN #265): a class that redefines `\maketitle`
/// itself had its body dropped by the lock, and with it every title-page field
/// the frontmatter API never sees (ryethesis.cls:282: degree, program,
/// "presented to Ryerson University", "Toronto, Ontario, Canada"). The dropped
/// body is kept as `\lx@dropped@maketitle` and deposited with the title/author
/// nulled: the fields follow the frontmatter, the title and author appear once.
#[test]
fn class_maketitle_body_deposits_its_fields() {
  let cls = r"\ProvidesClass{fieldthesis}
\LoadClass{report}
\newcommand\degree[1]{\gdef\ft@degree{#1}}
\renewcommand{\maketitle}{\begin{center}{\LARGE\@title}\\ by \\ {\@author}\\
  presented to Field University\\ for the degree of\\ \ft@degree\\ Toronto, Ontario, Canada\end{center}}
";
  let tex = r"\documentclass{fieldthesis}
\title{T}\author{A. E. Field}\degree{Doctor of Philosophy}
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("fieldthesis.cls", cls)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  assert!(
    flat.contains(concat!(
      r#"<title>T</title><creator role="author"><personname>A. E. Field</personname></creator>"#,
      r#"<logical-block><para><p align="center">by</p><p align="center">presented to Field University</p>"#,
      r#"<p align="center">for the degree of</p><p align="center">Doctor of Philosophy</p>"#,
      r#"<p align="center">Toronto, Ontario, Canada</p></para></logical-block><para><p>Body.</p></para>"#
    )),
    "{flat}"
  );
  // The real class, where the tree has it: its `{titlepage}` fields follow the
  // frontmatter (no `ltx:titlepage` — see the next guard).
  if kpsewhich_has("ryethesis.cls") {
    let tex = r"\documentclass{ryethesis}
\title{T}\author{A. E. Ryerson}
\degreeName{Doctor of Philosophy}\degreeYear{1847}\program{Education}
\begin{document}
\maketitle
\end{document}
";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("<titlepage"), "{xml}");
    for field in [
      "presented to Ryerson University",
      "Doctor of Philosophy",
      "Toronto, Ontario, Canada, 1847",
    ] {
      assert!(xml.contains(field), "{field}: {xml}");
    }
    assert_eq!(
      xml.matches("A. E. Ryerson").count(),
      1,
      "the author once: {xml}"
    );
  }
}

/// Batch 56ha: the deposit's vocabulary gate skips what a no-op macro
/// absorbs. uantwerpendocs' classes draw their title page inside eso-pic's
/// `\AddToShipoutPicture*{…}` (a no-op here), so the undefined tikz in that
/// argument rejected the whole dropped `\maketitle` body, with the flow
/// content beside it (uantwerpenexam's `\@extrainfo` rules, 42 % of the doc;
/// phdthesis jury and contact blocks). OXIDIZED_DESIGN #265.
#[test]
fn class_maketitle_deposit_skips_a_shipout_picture() {
  let tex = r"\documentclass{article}
\usepackage{eso-pic}
\makeatletter
\renewcommand\maketitle{\AddToShipoutPicture*{\put(0,0){\undefineddrawing (0,0) rectangle (1,1);}}\par Flow block kept.\par}
\makeatother
\title{T}
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The title, then the deposited flow block, then the body.
  assert!(
    latexml::util::test::normalize_markup(&xml).contains(
      r#"<title>T</title><para xml:id="p1"><p>Flow block kept.</p></para><para xml:id="p2"><p>Body.</p></para>"#
    ),
    "{xml}"
  );
}

/// Batch 56gm (56gj follow-up): a class `\maketitle` that lays its fields out on
/// `\begin{titlepage}` (edmaths.sty:166-181) is deposited with the title, author
/// and date nulled — so its titlepage must not become an `ltx:titlepage`: the
/// XSLT drops the document's title block whenever one exists (it takes a
/// titlepage to carry the title), and edmaths lost title/author/date from the
/// HTML. The fields follow the frontmatter; the title block stays.
#[test]
fn class_maketitle_titlepage_keeps_the_title_block() {
  let cls = r"\ProvidesClass{pagethesis}
\LoadClass{report}
\renewcommand{\maketitle}{\begin{titlepage}\begin{center}{\LARGE\@title}\\ \@author\\
  Doctor of Philosophy\\ The University of Nowhere\\ \@date\end{center}\end{titlepage}}
";
  let tex = r"\documentclass{pagethesis}
\title{T}\author{A. N. Author}\date{1999}
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("pagethesis.cls", cls)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<titlepage"), "{xml}");
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  assert!(
    flat.contains(concat!(
      r#"<title>T</title><creator role="author"><personname>A. N. Author</personname></creator>"#,
      r#"<date role="creation">1999</date>"#
    )),
    "{flat}"
  );
  for field in ["Doctor of Philosophy", "The University of Nowhere"] {
    assert_eq!(xml.matches(field).count(), 1, "{field}: {xml}");
  }
  assert_eq!(
    xml.matches("A. N. Author").count(),
    1,
    "the author once: {xml}"
  );
}

/// Batch 56gv: `\typeout` expands its argument PARTIALLY (Perl
/// `\typeout ExpandedPartially`, latex_constructs.pool.ltxml:4538), so a
/// `\protected` xparse command is written by name. Full expansion ran its
/// peek-based `s` grabber in an expansion-only context: `\IfBooleanTF` got an
/// empty argument (expl3's expandable `cmd/if-boolean` error) and the run fell
/// into an infinite-expansion recovery (leporello/jsonparse/euclideangeometry
/// class).
#[test]
fn typeout_leaves_protected_commands_unexpanded() {
  let tex = r"\documentclass{article}
\NewDocumentCommand{\foo}{s}{[\IfBooleanTF{#1}{STAR}{NOSTAR}]}
\begin{document}
\typeout{[\foo]}
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("[\\foo]"), "{stderr}");
}

/// Batch 56gu: beamer's list environments start the kernel's item machinery
/// (Perl beamer.cls.ltxml:1110-1179 `beginBeamerItemize`) and route `\item`
/// through `\beamer@item`, which consumes every overlay form. Without it
/// `\item` was raw beamerbaseoverlay.sty:485's (the kernel default `\par`):
/// each list was ONE tagless item, and every overlay spec leaked into the text
/// as OT1 glyphs (`¡2-¿ Two`, `[¡+-¿]`). Pinned here: `\item<2->`, a spec with
/// an action (`<3-| alert@3>`, metropolis), `\item[x]<4->`, `\item<5->[y]`, the
/// list default `[<+->]`, enumerate's label template alone (`[(a)]`) and after
/// a default overlay (`[<+->][i.]`), and `\item[Key]<2->` in a description.
#[test]
fn beamer_list_items_open_their_own_item() {
  let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}{T}
\begin{itemize}
\item One
\item<2-> Two
\item<3-| alert@3> Three
\item[x]<4-> Four
\item<5->[y] Five
\end{itemize}
\begin{itemize}[<+->]
\item A
\item B
\end{itemize}
\begin{enumerate}[(a)]
\item E1
\item E2
\end{enumerate}
\begin{enumerate}[<+->][i.]
\item F1
\end{enumerate}
\begin{description}
\item[Key]<2-> Value
\end{description}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  for list in [
    r##"<itemize><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para><p>One</p></para></item><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">2nd item</tag></tags><para><p>Two</p></para></item><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">3rd item</tag></tags><para><p>Three</p></para></item><item><tags><tag>x</tag><tag role="autoref">item </tag><tag role="typerefnum">item x</tag></tags><para><p>Four</p></para></item><item><tags><tag>y</tag><tag role="autoref">item </tag><tag role="typerefnum">item y</tag></tags><para><p>Five</p></para></item></itemize>"##,
    r##"<itemize><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para><p>A</p></para></item><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">2nd item</tag></tags><para><p>B</p></para></item></itemize>"##,
    r##"<enumerate><item><tags><tag>(a)</tag><tag role="autoref">item a</tag><tag role="refnum">a</tag><tag role="typerefnum">item a</tag></tags><para><p>E1</p></para></item><item><tags><tag>(b)</tag><tag role="autoref">item b</tag><tag role="refnum">b</tag><tag role="typerefnum">item b</tag></tags><para><p>E2</p></para></item></enumerate>"##,
    r##"<enumerate><item><tags><tag>i.</tag><tag role="autoref">item i</tag><tag role="refnum">i</tag><tag role="typerefnum">item i</tag></tags><para><p>F1</p></para></item></enumerate>"##,
    r##"<description><item><tags><tag><text font="bold">Key</text></tag><tag role="autoref">item </tag><tag role="typerefnum">item Key</tag></tags><para><p>Value</p></para></item></description>"##,
  ] {
    assert!(flat.contains(list), "{list}\n{flat}");
  }
  // No overlay spec reaches the text, in any spelling.
  for leak in ["¡", "¿", "alert@", "&lt;", "[i.]", "[(a)]"] {
    assert!(!xml.contains(leak), "{leak}: {xml}");
  }
  // A space before the overlay (`\@ifnextchar` skips it), both optionals with a
  // label template, and an overlay-only default that must keep the arabic labels.
  let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}{T}
\begin{itemize}
\item <2-> Spaced
\item Plain
\end{itemize}
\begin{enumerate}[<+->][(a)]
\item G1
\item G2
\end{enumerate}
\begin{enumerate}[<+->]
\item H1
\item H2
\end{enumerate}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  for list in [
    r##"<itemize><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para><p>Spaced</p></para></item><item><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">2nd item</tag></tags><para><p>Plain</p></para></item></itemize>"##,
    r##"<enumerate><item><tags><tag>(a)</tag><tag role="autoref">item a</tag><tag role="refnum">a</tag><tag role="typerefnum">item a</tag></tags><para><p>G1</p></para></item><item><tags><tag>(b)</tag><tag role="autoref">item b</tag><tag role="refnum">b</tag><tag role="typerefnum">item b</tag></tags><para><p>G2</p></para></item></enumerate>"##,
    r##"<enumerate><item><tags><tag>1.</tag><tag role="autoref">item 1</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para><p>H1</p></para></item><item><tags><tag>2.</tag><tag role="autoref">item 2</tag><tag role="refnum">2</tag><tag role="typerefnum">item 2</tag></tags><para><p>H2</p></para></item></enumerate>"##,
  ] {
    assert!(flat.contains(list), "{list}\n{flat}");
  }
}

/// Batch 56gt: etoolbox's `\patchcmd` re-tokenizes the patched body; a nested
/// macro's parameter (`##1` of an inner `\def`) came back as the OUTER `#1`
/// (pgfornament.sty:47-51's path operators in pgfornament-han's patched
/// ornament macro: every point collapsed, the ornaments drew nothing).
#[test]
fn patchcmd_keeps_nested_macro_parameters() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\def\outer#1{\def\inner##1{[#1|##1]}\inner{b}}
\patchcmd{\outer}{[}{(}{}{\typeout{PATCHFAIL}}
\begin{document}
\outer{a}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("PATCHFAIL"), "{stderr}");
  assert!(xml.contains("<p>(a—b]</p>"), "{xml}");
}

/// Batch 56gy: a space in a `\def`'s prefix delimiter matches exactly one space
/// token (tex.web §392). `read_match` skipped every following space, as Perl's
/// readMatch does (KPE #222), so `\again\Bdelim<sp><sp>X` lost the space xint's
/// zap loops leave for the next call (ipsum-doc: 70 `Match` errors under the
/// strict delimiter check). pdflatex: `( X) [a b]`.
#[test]
fn prefix_space_delimiter_matches_one_space() {
  let tex = r"\documentclass{article}
\usepackage{xinttools}
\long\def\myfirstofone#1{#1}
\long\def\showit#1\stop{(\detokenize{#1})}
\myfirstofone{\def\again\Bdelim} {\showit}
\def\Bdelim{}
\def\mk#1{\def\mk{\again\Bdelim#1#1X\stop}}\mk{ }
\begin{document}
\edef\z{\mk}\z\ [\xintZapSpaces{ a b }]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>( X) [a b]</p>"), "{xml}");
}

/// Batch 56he: a class `\maketitle[<options>]` (uni-titlepage.sty:97, 13
/// manuals) had its body dropped by the lock and its options leaked as a
/// paragraph. The kernel `\maketitle` now reads the options for the dropped body
/// and the deposit replays it with them. Inside the deposit `\title{…}` fills
/// `\@title` (uni-titlepage's `title=` key), and the frontmatter title stays.
#[test]
fn class_maketitle_deposit_threads_its_options() {
  let tex = r"\documentclass{article}
\begin{document}
\renewcommand{\maketitle}[1][]{\par DEKANLABEL: #1\par}
\maketitle[foo=bar]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    latexml::util::test::normalize_markup(&xml)
      .contains(r#"<para xml:id="p1"><p>DEKANLABEL: foo=bar</p></para></document>"#),
    "{xml}"
  );
  let tex = r"\documentclass{article}
\makeatletter
\renewcommand*{\maketitle}[1][]{\begingroup\title{#1}\par TITLE: \@title\par\endgroup}
\makeatother
\title{Kept}
\begin{document}
\maketitle[Hello]
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    latexml::util::test::normalize_markup(&xml).contains(
      r#"<title>Kept</title><para xml:id="p1"><p>TITLE: Hello</p></para><para xml:id="p2"><p>Body.</p></para>"#
    ),
    "{xml}"
  );
}

/// Batch 56hf: microtype with `babel` and `kerning` switches off French
/// babel's active `:;!?` at `\begin{document}` (microtype.sty:3162-3195), which
/// preamble code relies on: cahierprof.sty:366-388 freezes a `\tikzmath{…; …}`
/// run from its `\AtBeginDocument` hook. Our microtype stub never did, so once
/// 56hd made the class-option French real, tikzmath picked the active-`;`
/// delimiter and the statements leaked (cahierprof-doc 11 errors).
#[test]
fn microtype_babel_kerning_switches_off_french_shorthands() {
  if !kpsewhich_has("tikzlibrarymath.code.tex") {
    return;
  }
  let tex = r"\documentclass[french]{article}
\usepackage[T1]{fontenc}
\usepackage{babel}
\usepackage{tikz}\usetikzlibrary{math}
\usepackage[babel=true,kerning=true]{microtype}
\newcommand\calc{\tikzmath{\cc=int(5); \s=int(6);}}
\AtBeginDocument{\calc}
\begin{document}
x
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:lang="fr">"#),
    "{xml}"
  );
  assert!(
    latexml::util::test::normalize_markup(&xml)
      .contains(r#"<para xml:id="p1"><p>x</p></para></document>"#),
    "{xml}"
  );
}

/// Batch 56hd: a bare `\usepackage{babel}` takes its main language from the
/// class options (babel.sty:4197-4245), loads it last, and prepends it to
/// `\bbl@loaded` (:4130-4132). Our `.ldf` bindings never run `\main@language`,
/// so the main language stayed english: `xml:lang="en"`, English captions,
/// blindtext's English text (scrlttr2copy/letter-copy-test, recall 20.7; about
/// 36 French/German manuals). `\bbl@loaded`'s first language now wins when
/// babel has no options of its own.
#[test]
fn bare_babel_takes_the_class_language() {
  if !kpsewhich_has("blindtext.sty") {
    return;
  }
  let tex = r"\documentclass[ngerman]{article}
\usepackage{babel}
\usepackage{blindtext}
\begin{document}
\blindtext
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:lang="de">"#),
    "{xml}"
  );
  assert!(xml.contains("<p>Dies hier ist ein Blindtext"), "{xml}");
  let tex = r"\documentclass[french]{article}
\usepackage{babel}
\begin{document}
\tableofcontents
\section{A}
x
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:lang="fr">"#),
    "{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "TOC",
    &[],
    r#"<TOC lists="toc" scope="global" select="ltx:part | ltx:chapter | ltx:section | ltx:subsection | ltx:subsubsection | ltx:appendix | ltx:index | ltx:bibliography"><title>Table des matières</title></TOC>"#,
  );
}

/// Batch 56hc: verbatim.sty's `\verbatim@start#1` drops only the active line
/// end; a control sequence after it is prepended to line 1 and runs with the
/// line's remainder (verbatim.sty:107-112, tex.web §506). verbatimbox's
/// `\verbatim\verbbox@inner` (verbatimbox.sty:90/107) consumes the environment's
/// `[\footnotesize]` that way; our reader swallowed the command, so the
/// bracket leaked into the verbatim and the size was lost (readarray.tex:440).
#[test]
fn verbatim_start_runs_a_prepended_command() {
  let tex = r"\documentclass{article}
\usepackage{verbatim}
\newcommand\vbi[1][]{}
\newenvironment{vc}{\verbatim\vbi}{\endverbatim}
\begin{document}
\begin{vc}[X]
hello world
\end{vc}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<verbatim font=\"typewriter\">hello world\n</verbatim>"),
    "{xml}"
  );
  if !kpsewhich_has("verbatimbox.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{verbatimbox}
\begin{document}
Two:
\begin{verbbox}[\footnotesize]
gamma env
\end{verbbox}
\theverbbox
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<verbatim font="typewriter" fontsize="80%">gamma env</verbatim>"#),
    "{xml}"
  );
}

/// Batch 56hb: after a prefix TeX reads on to the next non-blank non-relax
/// token (tex.web §1211, §404), so `\protected\relax\def` and
/// `\protected<space>\def` define protected macros. Both engines cleared the
/// prefix at the `\relax`/space (Perl Stomach.pm:212, KPE #223): catoptions'
/// `\robust@def*` (catoptions.sty:375-381) left `\cpt@newv@riables`
/// unprotected, an `\edef` expanded it, 70 errors per `\usepackage{catoptions}`
/// (keyval2e, concepts). pdflatex: `XQYWZ macro:->\B`.
#[test]
fn relax_after_a_prefix_keeps_the_prefix() {
  let tex = r"\documentclass{article}
\makeatletter
\protected\relax\def\foo#1[#2]{X#1Y#2Z}
\edef\bad{\noexpand\@testopt{\foo{Q}}{}}
\protected\space\def\B{XB}\edef\PB{\B}
\makeatother
\begin{document}
\bad[W] \texttt{\meaning\PB}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<p>XQYWZ <text font="typewriter">macro:-&gt;\B </text></p>"#),
    "{xml}"
  );
}

/// Batch 56gz: a package or class load keeps ltfilehook's file-name stack
/// (`\@expl@@@filehook@file@push@@`/`…@pop@@` around the file hooks, latex.ltx
/// :18772/18789). scrlfile-hook.sty:146-158 seeds its own stack from the
/// kernel's and pops it once per `file/after`; with the kernel stack empty
/// every file already open when it loaded underflowed: nomencl → tocbasic →
/// scrbase → scrlfile → scrlfile-hook, 5 "More file names popped from stack
/// than put to" warnings (983 corpus manuals).
#[test]
fn file_hooks_keep_the_file_name_stack() {
  if !kpsewhich_has("scrlfile-hook.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{nomencl}
\begin{document}
x
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("popped from stack"), "{stderr}");
  assert!(!stderr.contains("should not happen"), "{stderr}");
  assert!(xml.contains("<p>x</p>"), "{xml}");
}

/// Batch 56gr: fancyvrb's `\VerbatimEnvironment` in a user environment that
/// wraps minted (RetoMatematico.cls:174 `{codigo}`) names the environment whose
/// `\end` closes the verbatim body (fancyvrb.sty:295-297/386-403); the minted
/// binding read to a literal `\end{minted}` and swallowed the rest of the
/// document (retomatematico-ejemplo, recall 53 %).
#[test]
fn verbatim_environment_wrapper_ends_minted() {
  if !kpsewhich_has("minted.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{minted}
\newenvironment{codigo}[1]{\VerbatimEnvironment\begin{minted}{#1}}{\end{minted}}
\begin{document}
Before.
\begin{codigo}{latex}
\textbf{x}
\end{codigo}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<listingline").count(), 1, "{xml}");
  assert!(xml.contains("<p>After.</p>"), "{xml}");
}

/// Batch 56gq: KOMA-Script replaces the (unlocked) `\part` with a typesetting
/// `\scr@startpart`, but still dispatches through LaTeX's part internals
/// (`\SecDef\@part\@spart`, scrartcl.cls:4884); the kernel's locked `\@part`/
/// `\@spart` make that an `ltx:part` (it was a bold paragraph — glossaries-
/// user, hvfloat, cnltx-doc manuals). exam.cls's question `\part` (its own
/// `\def\part`) is untouched.
#[test]
fn koma_part_is_a_part() {
  if !kpsewhich_has("scrartcl.cls") {
    return;
  }
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" (?:xml:id|inlist|labels)="[^"]*""#).unwrap();
  let flat = |xml: &str| {
    gaps
      .replace_all(&ids.replace_all(xml, ""), "><")
      .into_owned()
  };
  let tex = r"\documentclass{scrartcl}
\begin{document}
\part{Alpha}
\section{Beta}
Text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    flat(&xml).contains(concat!(
      r#"<part><tags><tag>Part I</tag><tag role="refnum">I</tag>"#,
      r#"<tag role="typerefnum">Part I</tag></tags>"#,
      r#"<title><tag close=" ">Part I</tag>Alpha</title>"#,
      r#"<toctitle><tag close=" ">I</tag>Alpha</toctitle><section><tags>"#
    )),
    "{xml}"
  );
  assert!(!xml.contains("sansserif bold"), "{xml}");
  let tex = r"\documentclass{scrbook}
\begin{document}
\part{Alpha}
\chapter{Gamma}
Text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<part") && xml.contains("<chapter"), "{xml}");
  if !kpsewhich_has("exam.cls") {
    return;
  }
  let tex = r"\documentclass{exam}
\begin{document}
\begin{questions}
\question First question
\begin{parts}
\part Part one
\end{parts}
\end{questions}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<part"), "{xml}");
  assert!(xml.contains("Part one"), "{xml}");
}

/// Batch 56gp: the dvips-backend pagecount hook (latexml_sty/mod.rs) defined
/// `\__graphics_backend_get_pagecount:n` with `##1` inside `\AddToHook`, which
/// stores its code verbatim — the constant came out `c__graphics_#1_pages_int`
/// and `\graphics_get_pagecount:nN` read back nothing (l3msg's silent
/// expandable "bad-variable" error; notebeamer-demo). `\pdfoutput=0` selects
/// the dvips backend the hook serves: since K6 (batch 56id) the default is
/// PDF mode, whose l3backend-pdftex never runs the hook.
#[test]
fn dvips_backend_pagecount_reads_back_the_count() {
  if !kpsewhich_has("notebeamer.sty") {
    return;
  }
  let tex = r"\pdfoutput=0
\documentclass{article}
\usepackage{notebeamer}
\begin{document}
\ExplSyntaxOn
\graphics_get_pagecount:nN {example-image-a4.pdf} \l_tmpa_tl
PAGES=\tl_use:N \l_tmpa_tl.
\ExplSyntaxOff
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>PAGES=1.</p>"), "{xml}");
}

/// Batch 56go: a character that starts a paragraph is backed up and
/// `\everypar` runs IN FRONT of it (tex.web §1090-1091), so the hook reads that
/// very token with its catcode already fixed. syntax.sty's grammar
/// (`\everypar{…\catcode`\<\active\gr@implitem}`, `\gr@implitem<#1> #2 `)
/// depends on it; digesting `\everypar` beside the absorbed `<` gave `¡ab¿`.
#[test]
fn everypar_reads_the_token_that_started_the_paragraph() {
  let tex = r"\documentclass{article}
\def\imp<#1>{[#1]}
\begin{document}
\catcode`\<=12
\everypar{\everypar{}\catcode`\<\active\imp}
<ab> text
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[ab] text</p>"), "{xml}");
}

/// …and a vertical command in horizontal mode runs a REDEFINED `\par` whole
/// before it reads its own arguments (tex.web §1094 `head_for_vmode`).
/// syntax.sty:264-273's grammar `\par` ends with `\@@par \catcode`\<12
/// \everypar{…}`; `leave_horizontal` ran it through `invoke_token`, which
/// stops at the first primitive, so `\vskip`'s glue scan read the leftover
/// `\@@par` ("Missing number") and the glue's stray digit fired the re-armed
/// `\everypar`, whose `\gr@implitem` never found its catcode-12 `<` (11
/// errors; arXiv 2605.07451 ended in `Fatal:Timeout:PushbackLimit`).
#[test]
fn everypar_grammar_survives_vertical_material_between_productions() {
  if !kpsewhich_has("syntax.sty") {
    return;
  }
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/block-model/leave_horizontal_runs_a_redefined_par_whole.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.ix2""#],
    r##"<item xml:id="S0.I1.ix2"><tags><tag><Math mode="inline" tex="\langle" text="langle" xml:id="S0.I1.ix2.m1"><XMath><XMTok name="langle" role="OPEN" stretchy="false">⟨</XMTok></XMath></Math><text font="italic">model<Math mode="inline" tex="\rangle" text="rangle" xml:id="S0.I1.ix2.m2"><XMath><XMTok font="upright" name="rangle" role="CLOSE" stretchy="false">⟩</XMTok></XMath></Math></text>  ::=</tag><tag role="typerefnum">item </tag></tags><para xml:id="S0.I1.ix2.p1"><p>bar</p></para></item>"##,
  );
}

/// Batch 56gm: a void box register is a box operand (TeXbook p.388) whatever
/// its SPELLING — expl3's `\box_use:N` is `\copy` (expl3-code.tex:30507), and
/// l3coffins `\raise`s it over a void coffin box (asmejour.cls:1388-1404).
#[test]
fn raise_accepts_an_aliased_void_box_register() {
  let tex = r"\documentclass{article}
\ExplSyntaxOn
\box_new:N \l_void_box
\ExplSyntaxOff
\begin{document}
X\ExplSyntaxOn\tex_raise:D 2pt \box_use:N \l_void_box \ExplSyntaxOff Y
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The void box raised, as `\raise1pt\copy\strutbox` is.
  assert!(xml.contains(r#"<p>X<text yoffset="2.0pt"/>Y</p>"#), "{xml}");
  // A non-box operand (an assignment, which yields nothing) is still an error.
  let tex = r"\documentclass{article}
\begin{document}
X\raise2pt\count0=1 Y
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(
    stderr
      .lines()
      .filter(|l| l.contains("Error:expected:<box>"))
      .count(),
    1,
    "{stderr}"
  );
}

/// Batch 56gm: unicode-math's `version=<name>` key declares the math version
/// (unicode-math-luatex.sty:1585-1592); the `\setmathfont` no-op left
/// `\mathversion{<name>}` unknown (asmeconf.cls:698-750, :1698).
#[test]
fn setmathfont_version_declares_the_math_version() {
  let tex = r"\documentclass{article}
\usepackage{unicode-math}
\setmathfont{XITSMath-Regular}[version=myver]
\setmathfont[version={other}, Scale=1]{XITSMath-Regular}
\begin{document}
Text. \mathversion{myver} more. \mathversion{other} end.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// Batch 56gm: the class `\maketitle` deposit relaxes the frontmatter SETTERS
/// as well as their stores — ukbill.cls:497 typesets `\textbf{\title}` (the
/// setter) where `\@title` was meant, and memoir's two-argument `\title` ate
/// the deposit's braces ("Attempt to close boxing group", immigration-bill).
#[test]
fn class_maketitle_deposit_relaxes_the_setters() {
  if !kpsewhich_has("memoir.cls") {
    return;
  }
  let tex = r"\documentclass{memoir}
\title{T}
\renewcommand{\maketitle}{\begin{center}{\Huge\textbf{\title}}\\ Fields\end{center}}
\begin{document}
\maketitle
Body text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // memoir's own `\@iffirstamp` (a conditional not spelled `\if…`).
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:misdefined:\\@iffirstamp"),
    "{stderr}"
  );
  assert!(xml.contains("<title>T</title>"), "{xml}");
  assert_eq!(xml.matches("Fields").count(), 1, "{xml}");
}

/// Batch 56gm (OXIDIZED_DESIGN #268): a counter allocated with a raw
/// `\newcount` (doc.sty:870 `\c@CodelineNo`) has no LaTeXML `UN` companion,
/// so resetting it from a reset list (l3doc.cls:463 `\@addtoreset{CodelineNo}
/// {part}`) must not assign one — "not a register" at every `\part`
/// (ltx-talk-code, 20 warnings; Perl `ResetCounter` shares it).
#[test]
fn reset_list_skips_a_missing_un_companion() {
  if !kpsewhich_has("l3doc.cls") {
    return;
  }
  let tex = r"\documentclass{l3doc}
\begin{document}
\part{Alpha}
Body one.
\part{Beta}
Body two.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<part ").count(), 2, "{xml}");
}

/// Batch 56gm: a class that renews the locked `{abstract}` keeps the abstract in
/// its own store, which the lock never lets it fill; inside the class
/// `\maketitle` deposit that store reads GIVEN AND EMPTY (the frontmatter
/// carries the abstract). A box store (wkmgr.cls:190 `\global\setbox
/// \abspagebox\vbox\bgroup`, :157 `\ifvoid` → "Nie podano streszczenia") and
/// the conventional `\@abstract` of an environ-collected body (mcmthesis.cls
/// :165, "\@abstract undefined").
#[test]
fn class_maketitle_reads_a_dropped_environment_store_as_given() {
  let cls = r"\ProvidesClass{boxthesis}
\LoadClass{report}
\newbox\abs@box
\renewenvironment{abstract}{\global\setbox\abs@box\vbox\bgroup}{\egroup}
\renewcommand{\maketitle}{\begin{center}Field Line\end{center}%
  \ifvoid\abs@box\ClassWarning{boxthesis}{No abstract given}\fi\unvbox\abs@box}
";
  let tex = r"\documentclass{boxthesis}
\title{T}
\begin{document}
\begin{abstract}The abstract.\end{abstract}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("boxthesis.cls", cls)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("Field Line").count(), 1, "{xml}");
  assert_eq!(xml.matches("The abstract.").count(), 1, "{xml}");
  if !kpsewhich_has("environ.sty") {
    return;
  }
  let cls = r"\ProvidesClass{envthesis}
\LoadClass{report}
\RequirePackage{environ}
\RenewEnviron{abstract}{\xdef\@abstract{\expandonce\BODY}}
\def\make@abstract{\begin{center}Summary Sheet\end{center}\@abstract\par}
\renewcommand{\maketitle}{\make@abstract}
";
  let tex = r"\documentclass{envthesis}
\title{T}
\begin{document}
\begin{abstract}The abstract.\end{abstract}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("envthesis.cls", cls)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("Summary Sheet").count(), 1, "{xml}");
  assert_eq!(xml.matches("The abstract.").count(), 1, "{xml}");
}

/// Batch 56gm: hyperref's `\NoHyper`/`\endNoHyper` are macros a class may call
/// directly (asmeconf.cls:2033 `\NoHyper\footnotemark[#1]\endNoHyper`); the
/// binding had only `\begin{NoHyper}`/`\end{NoHyper}`.
#[test]
fn nohyper_macros_are_defined() {
  let tex = r"\documentclass{article}
\usepackage{hyperref}
\begin{document}
A\NoHyper B\endNoHyper C
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>ABC</p>"), "{xml}");
}

/// Batch 56gi: doclicense loads raw — the stub made `\doclicenseThis` and every
/// accessor a no-op, so the license statement never reached the XML (beautynote;
/// PDF-to-XML recall 0 %). The statement, its link and the license image survive.
#[test]
fn doclicense_statement_reaches_the_xml() {
  if !kpsewhich_has("doclicense.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage[type={CC},modifier={by-sa},version={4.0}]{doclicense}
\begin{document}
\doclicenseThis
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(concat!(
      r#"This work is licensed under a <ref class="ltx_href" href="https://creativecommons.org/licenses/by-sa/4.0/deed.en">"#,
      "Creative Commons <text class=\"ltx_inline-quote ltx_outerquote\">\u{201c}Attribution-ShareAlike 4.0 International\u{201d}</text></ref> license.</p>"
    )),
    "{xml}"
  );
  assert!(
    xml.contains(r#"graphic="doclicense-CC-by-sa-88x31""#),
    "{xml}"
  );
}

/// Batch 56gh: `\@ifdefinable` tests Perl's `isDefinableLaTeX`
/// (latex_constructs.pool.ltxml:2512-2517, :5461-5468): a command the PLAIN
/// layer defined (`\proclaim`, `\beginsection` — plain_constructs pool) is
/// definable for LaTeX; a kernel LaTeX command (`\section`) is not. Rust pools
/// carry no `plain_*` locator, so the plain layer is read from the definition's
/// provenance (a plain pool, or the plain dump — embedded or on disk).
#[test]
fn ifdefinable_accepts_a_plain_macro() {
  let tex = r"\documentclass{article}
\makeatletter
\@ifdefinable\proclaim{\def\resultA{plain-ok}}
\@ifdefinable\beginsection{\def\resultB{plain-ok}}
\@ifdefinable\zzznew{\def\resultC{new-ok}}
\makeatother
\begin{document}
[\resultA][\resultB][\resultC]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>[plain-ok][plain-ok][new-ok]</p>"), "{xml}");
  // A kernel LaTeX command stays not-definable: `\@notdefinable` reports it.
  let tex = r"\documentclass{article}
\makeatletter
\@ifdefinable\section{\def\resultD{wrong}}
\providecommand\resultD{kept}
\makeatother
\begin{document}
[\resultD]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert!(
    stderr.contains("Command \\section already defined"),
    "{stderr}"
  );
  assert!(xml.contains("<p>[kept]</p>"), "{xml}");
}

/// Batch 56gg: a `\clearpage` inside a titlesec `\titleformat` runs while the
/// heading is built; its `<pagination>` floated out of `<title>` and stood
/// between the headings (`element "toctitle" not allowed here`, bmstu-example;
/// SHARED). As in TeX, the break now precedes the section.
#[test]
fn pagebreak_in_a_title_format_precedes_the_section() {
  let tex = r"\documentclass{article}
\usepackage{titlesec}
\titleformat{\section}[hang]{\clearpage\normalfont\bfseries\centering}{}{0em}{}
\begin{document}
\section[Short]{Terms}
Body text here.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  let section = flat.find("<section").unwrap();
  assert!(
    flat[..section].ends_with(r#"<pagination role="newpage"/>"#),
    "{flat}"
  );
  let body = &flat[section..];
  assert!(!body.contains("<pagination"), "{flat}");
  assert!(
    // 57o: the format's `\centering` aligns the heading itself (DIVERGENCES #344).
    body.contains(r#"<title class="ltx_align_center" font="bold">Terms</title><toctitle>"#),
    "the headings stay adjacent: {flat}"
  );
}

/// Batch 56gg: `\unwind@titlepage` (the `{titlepage}` env's `\maketitle` hook)
/// re-homes the unwound children the new parent cannot hold. jlreq's own
/// `\maketitle` opens a titlepage inside the document's `{titlepage}`
/// (jlreq.cls:5501-5527); its empty centered paragraphs were unwrapped straight
/// under `<document>` — `element "p" not allowed here` ×2, the only schema errors
/// of gckanbun-doc and kksymbols-doc. Now each is wrapped in a `<para>`.
#[test]
fn unwound_titlepage_rehomes_its_paragraphs() {
  if !kpsewhich_has("jlreq.cls") {
    return;
  }
  let tex = r"\documentclass[luatex,fontsize=10pt,paper=b5]{jlreq}
\title{\texttt{KKsymbols} Package Documentation}
\author{Kosei Kawaguchi}
\date{Version 1.1.1}
\begin{document}
\begin{titlepage}
  \maketitle
\end{titlepage}
\newpage
\section{Acknowledgements}
Hello.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The warning set is pinned exactly: jlreq's own `b5` notice (lualatex prints
  // it too) plus the OPEN jlreq heading-level lead — jlreq's `\NewBlockHeading`
  // finds the kernel's predefined `\section` ("Command \section already defined")
  // and never sets `\jlreq@heading@level@section`, so jlreq.cls:6584's
  // `\ifnum` reads `\relax` (LEDGER 2026-09-23 56gg row). When that lead lands,
  // this becomes `warning_count == 1`.
  let missing_number = stderr
    .matches("Warning:expected:<number> Missing number")
    .count();
  assert_eq!(warning_count(&stderr), 1 + missing_number, "{stderr}");
  assert!(
    stderr.contains("Class jlreq Warning: The option `b5' means"),
    "{stderr}"
  );
  // Structure, not indentation: flattened, the two empty centered paragraphs are
  // each wrapped, and no `<p` follows a root-level sibling's close directly.
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let flat = gaps
    .replace_all(&ids.replace_all(&xml, ""), "><")
    .into_owned();
  assert!(
    flat.contains(r#"<para><p/></para><para><p align="center"/></para>"#),
    "{flat}"
  );
  let bare = regex::Regex::new(r"</(?:title|creator|date|pagination|para)><p[ />]").unwrap();
  assert!(
    !bare.is_match(&flat),
    "no <p> directly under <document>:\n{flat}"
  );
  assert!(
    xml.contains("<title>KKsymbols Package Documentation</title>"),
    "{xml}"
  );
  assert!(xml.contains("<p>Hello.</p>"), "{xml}");
}

/// Batch 56gg: inside a frame `\frame` is the kernel box frame
/// (beamerbaseframe.sty:92 `\let\frame=\framelatex`), not a nested slide.
/// Witness beamertheme-trigon/trigon_demo (frames.tex:43-58,
/// `\frame{\includegraphics…}` in a subfigure): a `<subsection>` inside
/// `<figure>`, schema-invalid, RUST-ONLY. The top-level short form still opens
/// a frame.
#[test]
fn frame_inside_a_frame_is_the_kernel_box_frame() {
  let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}{Demo}
\begin{figure}
\frame{\rule{2cm}{1cm}}
\caption{plain}
\end{figure}
\end{frame}
\frame{Short form.}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let figure = &xml[xml.find("<figure").unwrap()..xml.find("</figure>").unwrap()];
  assert!(!figure.contains("<subsection"), "{xml}");
  assert!(
    figure.contains(r#"<rule height="28.5pt" width="56.9pt"/>"#) && figure.contains("<picture"),
    "the kernel \\frame boxes the rule: {xml}"
  );
  // Outside a frame the command form still builds a frame of its own.
  assert_eq!(xml.matches("<subsection").count(), 2, "{xml}");
  assert!(xml.contains("<p>Short form.</p>"), "{xml}");
}

/// Batch 56gf control: the flush ends a PARAGRAPH, never a frontmatter
/// container — `\begin{titlepage}\maketitle\end{titlepage}` (a common manual
/// shape) relies on `\unwind@titlepage` finding the titlepage open; closing it at
/// the flush left an empty `<titlepage/>` ahead of the title.
#[test]
fn titlepage_around_maketitle_leaves_no_empty_titlepage() {
  let tex = r"\documentclass{article}
\title{T}\author{A}\date{D}
\begin{document}
\begin{titlepage}
\maketitle
\end{titlepage}
\section{S}
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<titlepage"), "{xml}");
  // After the resources, the document opens with its title.
  let first = xml
    .lines()
    .skip_while(|l| !l.starts_with("<document"))
    .skip(1)
    .find(|l| !l.trim_start().starts_with("<resource"))
    .unwrap();
  assert_eq!(first.trim(), "<title>T</title>", "{xml}");
}

/// Batch 56gf (OXIDIZED_DESIGN #262): what the PREAMBLE typesets (an undefined
/// command's marker and argument — the leak behind ~33 frontmatter-after-body
/// manuals: `\mubytein`, `\XeTeXgenerateactualtext`, a missing sibling file's
/// macros) is preamble residue, marked at `\begin{document}`: the frontmatter
/// leads, the residue follows it whole, and `\maketitle` ends its paragraph.
#[test]
fn preamble_residue_follows_the_frontmatter() {
  let tex = r"\documentclass{article}
\undefinedpreamblecmd{leaked argument}
\title{The Title}\author{An Author}
\begin{document}
\maketitle
Body text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("Error:undefined:\\undefinedpreamblecmd"),
    "{stderr}"
  );
  let body = xml.find("<document").unwrap();
  let doc = xml[body..].split_once('>').unwrap().1;
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let ids = regex::Regex::new(r#" xml:id="[^"]*""#).unwrap();
  let doc = gaps
    .replace_all(&ids.replace_all(doc, ""), "><")
    .into_owned();
  assert!(
    doc.contains(concat!(
      r#"<title>The Title</title><creator role="author"><personname>An Author</personname></creator>"#,
      r#"<para><ERROR class="undefined">\undefinedpreamblecmd</ERROR><p>leaked argument</p></para>"#,
      r#"<para><p>Body text.</p></para></document>"#
    )),
    "{doc}"
  );
  // Control: `\begin{document}` issues no `\par` — without a flush the preamble
  // text and the first body words share one paragraph, as in TeX.
  let tex = r"\documentclass{article}
\undefinedpreamblecmd{leaked}
\begin{document}
body
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(xml.contains("<p>leaked\nbody</p>"), "{xml}");
}

/// Batch 56ge: `\global\read` keeps its read-mode bookkeeping local. The
/// `\global` prefix is still live inside the primitive, so `\read`'s unscoped
/// `PRESERVE_NEWLINES=2` became global and the MAIN mouth stayed in read mode:
/// stray `\par` tokens landed in every later label and id
/// (`labels="LABEL:sec:b\par\par"`, `xml:id="S1par"`). Witness
/// csvsimple/csvsimple-legacy (`\csvreadnext` = `\global\read`,
/// csvsimple-legacy.sty:296; ~1,600 corrupted attributes, 10 schema errors).
/// The target macro itself is still global.
#[test]
fn global_read_leaves_the_main_mouth_alone() {
  let tex = r"\documentclass{article}
\newread\myf
\begin{document}
\openin\myf=grade.csv
\begingroup\global\read\myf to\lineA\endgroup
\closein\myf
\section{B}\label{sec:b}
See \ref{sec:b} [\lineA].
\end{document}
";
  let (stderr, xml) = convert_files(tex, &[("grade.csv", "name,grade\nMaier,1.0\n")]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<section inlist="toc" labels="LABEL:sec:b" xml:id="S1">"#),
    "{xml}"
  );
  assert!(
    // The line's end-of-line space keeps its "\n" text (Perl's data model, #260).
    xml.contains("<p>See <ref labelref=\"LABEL:sec:b\"/> [name,grade\n].</p>"),
    "{xml}"
  );
}

/// Batch 56gd: refstyle loads the real refstyle.sty (with refstyle.cfg), in
/// raw and default mode alike: the `\newref` templates build `\secref`, the
/// amsmath `\eqref` is replaced without refstyle's "already defined" error,
/// and the internals a LyX preamble calls directly (`\RS@ifundefined`,
/// refstyle.sty:51-57) exist. The former stub left an `<ERROR>` in the
/// preamble that stranded the title (uspatent/PatentApplicationGuide, 3
/// schema errors). Witnesses arXiv:2009.10518, arXiv:1804.06350.
#[test]
fn refstyle_loads_raw_with_its_internals() {
  if !kpsewhich_has("refstyle.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{amsmath,refstyle}
\makeatletter
\RS@ifundefined{subref}{\newref{sub}{name=section~}}{}
\makeatother
\title{T}
\begin{document}
\maketitle
\section{A}\label{sec:a}
See \secref{a} and \eqref{e}.
\begin{equation}\label{eq:e}x\end{equation}
\end{document}
";
  for raw in [true, false] {
    let (stderr, xml) = convert(tex, raw);
    assert_eq!(error_count(&stderr), 0, "raw={raw}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "raw={raw}: {stderr}");
    assert!(!xml.contains("<ERROR"), "raw={raw}: {xml}");
    // The document title precedes the body: nothing stranded it.
    let title = xml.find("<title>T</title>").unwrap();
    assert!(title < xml.find("<section").unwrap(), "raw={raw}: {xml}");
    assert!(!xml[..title].contains("<para"), "raw={raw}: {xml}");
    assert!(
      xml.contains(concat!(
        r#"<p>See section §<ref labelref="LABEL:sec:a"/> and equation "#,
        r#"(<ref labelref="LABEL:eq:e"/>).</p>"#
      )),
      "raw={raw}: {xml}"
    );
  }
}

/// Batch 56gd: newverbs.sty:112-137 — `\MakeSpecialShortVerb\qverb\"` makes
/// `"…"` a short `\qverb` (macros2e.tex:10; undefined before, the preamble
/// `<ERROR>` stranded the frontmatter, 4 schema errors).
#[test]
fn make_special_short_verb_is_defined() {
  if !kpsewhich_has("newverbs.sty") {
    return;
  }
  let tex = r#"\documentclass{article}
\usepackage{newverbs}
\MakeSpecialShortVerb\qverb\"
\title{T}
\begin{document}
\maketitle
Use "x_y" here.
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert!(
    // Both quotes in the verbatim font, as pdflatex prints them: ‘‘x_y’’
    // (newverbs.sty:52-69;
    // `binding_singletons_56::qverb_quotes_share_the_verbatim_font`).
    xml.contains(concat!(
      r#"<p>Use <text font="typewriter">‘‘<verbatim>x_y</verbatim>"#,
      r#"’’</text> here.</p>"#
    )),
    "{xml}"
  );
}

/// Batch 56gd: a list opened directly inside a list of the same kind (iitem's
/// `\Pseudo@item` lost to a raw class's `\@item`, qworld: 24 schema errors)
/// is placed in an auto-opened `item` → `para`, the shape the empty item
/// gives it. The indirect model skips a same-tag route (Perl Document.pm:206),
/// so the find_insertion_point bridge table carries it.
#[test]
fn nested_list_before_an_item_gets_an_auto_item() {
  let tex = r"\documentclass{article}
\begin{document}
\begin{itemize}
\begin{itemize}
\item Inner.
\end{itemize}
\item Outer.
\end{itemize}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    outer_list(&xml, "itemize"),
    concat!(
      r#"<itemize><item><para><itemize><item><tags><tag><text font="bold">–</text></tag>"#,
      r#"<tag role="typerefnum">1st item</tag></tags><para><p>Inner.</p></para></item>"#,
      r#"</itemize></para></item><item><tags><tag>•</tag><tag role="typerefnum">1st item</tag>"#,
      r#"</tags><para><p>Outer.</p></para></item></itemize>"#
    ),
    "{xml}"
  );
}

/// Batch 56bq: a bibliography issued inside a pgf node (xebaposter's
/// References `\headerbox`) floats out of the drawing to the document,
/// as from a plain minipage, because nothing else in the box would be
/// stranded; `ltx:bibliography` is not Flow (LaTeXML-structure.rnc:677).
#[test]
fn bibliography_in_a_drawing_floats_to_the_document() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}\n\\node[text width=5cm]{%\n  \\begin{thebibliography}{1}\n  \\bibitem{a} Foo bar baz.\n  \\end{thebibliography}%\n};\n\\end{tikzpicture}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let bib = xml.find("<bibliography").expect("a bibliography");
  let svg_end = xml.rfind("</svg:svg>").unwrap_or(0);
  assert!(
    bib > svg_end,
    "the bibliography is still inside the drawing: {xml}"
  );
  assert!(xml.contains("bibitem"), "{xml}");
}

/// pstricks coordinates (Perl pstricks_support.sty.ltxml:85-113): a bare
/// number is scaled by `\psxunit`/`\psyunit`, an explicit dimension stands
/// as is, and a node reference is not a coordinate (placed at the origin, no
/// error); pst-node objects take an OPTIONAL pair (pst-node.tex:157-421).
/// Batch 56aq — sweep 61 regressions from 56ao: lsc (`\cnode{r}{n}` without a
/// pair), pst-eucl/egpeirce (`\rput(N){…}` derailing the picture).
#[test]
fn pstricks_coordinates_take_units_nodes_and_optional_pairs() {
  let tex = "\\documentclass{article}\n\\usepackage{pstricks,pst-node,pgffor}\n\\begin{document}\nA\\begin{pspicture}(4,3)\\foreach \\x in {1,2}{\\rput(\\x,1){P\\x}}\\end{pspicture}B\n\\begin{pspicture}(3cm,2cm)\\pnode(1,1){N}\\cnode{3pt}{n2}\\cnode(2,2){3pt}{n3}\\rput(N){at N}\\rput([nodesep=2pt]N){near N}\\uput[ur](N){$A_1$}\\rput(1cm,2mm){dims}\\end{pspicture}C\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<picture").count(), 2, "{xml}");
  assert!(
    xml.contains("<g transform=\"translate(39.37,39.37)\">"),
    "P1: {xml}"
  );
  assert!(
    xml.contains("<g transform=\"translate(78.74,39.37)\">"),
    "P2: {xml}"
  );
  assert!(
    xml.contains("width=\"85.36pt\"") && xml.contains("height=\"56.91pt\""),
    "{xml}"
  );
  assert!(
    xml.contains("<g transform=\"translate(39.37,7.87)\">"),
    "dims: {xml}"
  );
  for label in ["at N", "near N", "dims"] {
    assert!(
      xml.contains(&format!("<text>{label}</text>")),
      "{label} lost: {xml}"
    );
  }
  assert!(
    xml.contains("A<") && xml.contains(">B") && xml.contains(">C"),
    "{xml}"
  );
  assert!(
    !xml.contains("(1,1)N") && !xml.contains("3pt"),
    "leak: {xml}"
  );
}

/// pdfTeX manual §8.9: `\pdfximage{file}` sets `\pdflastximagepages` to the
/// PDF's page count (bitmaps: 1) and bumps `\pdflastximage`. Both were stubs
/// at 0, so pdfpages' `\AM@getpagecount` (pppdftex.def:79-82), the l3 backend
/// page count in the DVI persona and any `\ifnum\pdflastximagepages=…` test
/// were wrong. The reader takes the largest `/Type /Pages … /Count` (batch 56ap;
/// Gemini round 8 N3 design).
#[test]
fn pdfximage_reports_the_pdf_page_count() {
  // A minimal but well-formed 3-page PDF: two /Pages nodes (the LARGEST /Count
  // must be picked) and a nested /Resources dictionary BEFORE the root's /Count
  // (the scan must stay at the dictionary's own brace depth).
  let pdf = "%PDF-1.4\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Resources << /Font << /F1 9 0 R >> >> /Kids [3 0 R 6 0 R] /Count 3 >> endobj\n3 0 obj << /Type /Pages /Parent 2 0 R /Kids [4 0 R 5 0 R] /Count 2 >> endobj\n4 0 obj << /Type /Page /Parent 3 0 R /MediaBox [0 0 200 100] >> endobj\n5 0 obj << /Type /Page /Parent 3 0 R /MediaBox [0 0 200 100] >> endobj\n6 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] >> endobj\ntrailer << /Root 1 0 R >>\n%%EOF\n";
  let tex = "\\documentclass{article}\n\\begin{document}\n\\pdfximage{three.pdf}A=\\the\\pdflastximagepages;N=\\the\\pdflastximage;\n\\pdfximage{example-image-a4.pdf}B=\\the\\pdflastximagepages;N=\\the\\pdflastximage.\n\\end{document}\n";
  let (stderr, xml) = convert_files(tex, &[("three.pdf", pdf)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("A=3;N=1;"), "{xml}");
  if kpsewhich_has("example-image-a4.pdf") {
    assert!(xml.contains("B=1;N=2."), "{xml}");
  }
}

/// `{pspicture}` is an `<ltx:picture>` sized by its corner pairs (Perl
/// pstricks_support.sty.ltxml:520-536; a lone pair is the far corner) and
/// `\rput`/`\uput`/`\cput` keep their bodies inside `<ltx:g transform>`
/// (:879-888). The former `[]{}` signature leaked `x0,y0)(x1,y1)` as text in
/// every pstricks picture and the placement gobblers dropped every label
/// (batch 56ao).
#[test]
fn pspicture_is_a_picture_and_rput_keeps_its_body() {
  let tex = "\\documentclass{article}\n\\usepackage{pstricks}\n\\begin{document}\nA\\begin{pspicture}(4,3)\\rput(1,1){R1}\\end{pspicture}B\n\\begin{pspicture}(-1,-1)(4,3)\\rput[l]{90}(2,2){R2}\\uput[r](1,1){R3}\\end{pspicture}C\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<picture").count(), 2, "{xml}");
  assert!(
    xml.contains("width=\"113.81pt\"") && xml.contains("height=\"85.36pt\""),
    "{xml}"
  );
  assert!(
    xml.contains("origin-x=\"-28.45pt\"") && xml.contains("width=\"142.26pt\""),
    "{xml}"
  );
  // The `[l]` refpoint is Perl's `pos` (`\rput@start`'s digested `[]`, W12).
  assert!(
    xml.contains("<g pos=\"l\" transform=\"translate(78.74,78.74)\">"),
    "{xml}"
  );
  for label in ["R1", "R2", "R3"] {
    assert!(
      xml.contains(&format!("<text>{label}</text>")),
      "{label} lost: {xml}"
    );
  }
  for leak in ["4,3)", "-1,-1)", "(2,2)", "(1,1)"] {
    assert!(!xml.contains(leak), "leaked {leak}: {xml}");
  }
  assert!(
    xml.contains("A<") && xml.contains(">B") && xml.contains(">C"),
    "{xml}"
  );
}

/// pst-plot's refusing stub left `\psplot`/`\psaxes`… undefined (plot
/// expressions and coordinates leaked as text) and pst-all's stub loaded two
/// of its twelve packages (`\multido` undefined). K1 step 3 pass two, batch
/// 56an: both bindings now consume/provide what pst-plot.tex:107-1099 and
/// pst-all.sty:17-32 define.
#[test]
fn pst_plot_and_pst_all_consume_their_arguments() {
  if kpsewhich_has("pst-all.sty") && kpsewhich_has("pst-plot.tex") && kpsewhich_has("multido.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{pst-all}\n\\begin{document}\nBefore.\n\\begin{pspicture}(4,3)\\psaxes{->}(0,0)(4,3)\\psplot[linecolor=red]{0}{4}{x 2 div}\\parametricplot{0}{360}{t cos t sin}\\multido{\\i=1+1}{2}{\\psline(\\i,0)(\\i,1)}\\rput(1,1){R}\\end{pspicture}\nAfter.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("Before.") && xml.contains("After.") && xml.contains(">R<"),
      "{xml}"
    );
    for leak in ["x 2 div", "t cos", "(0,0)", "(4,3)", "->"] {
      assert!(!xml.contains(leak), "leaked {leak}: {xml}");
    }
  }
}

/// xcolor.sty:1373-1390 `\XC@getcolor` branches on the argument's first token:
/// `[model]{spec}` is parsed directly; only a bare name is a declared-colour
/// lookup. Our reimplementation handed `[cmyk]{…}` to the name lookup
/// (pst-3dplot.tex:245 `SegmentColor={[cmyk]{0.2,0.6,1,0}}` through
/// pstricks.sty:155 `\let\pst@getcolor\XC@getcolor`; neoschool-fr).
#[test]
fn xcolor_getcolor_parses_a_model_prefixed_spec() {
  let tex = "\\documentclass{article}\n\\usepackage{xcolor}\n\\makeatletter\n\\def\\lxstrip\\xcolor@#1#2#3#4{#3:#4}\n\\XC@getcolor{[cmyk]{0.2,0.6,1,0}}\\lxa\n\\XC@getcolor{red}\\lxb\n\\edef\\lxres{A=\\expandafter\\lxstrip\\lxa;B=\\expandafter\\lxstrip\\lxb;}\n\\makeatother\n\\begin{document}\n\\lxres\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("A=cmyk:0.2,0.6,1,0;"), "{xml}");
  assert!(xml.contains("B=rgb:") || xml.contains("B=named:"), "{xml}");
}

/// tex.web §484: a `\read` from the terminal below scroll mode is a fatal
/// error — the deliberate halt of iftex.sty:51 (`\Require<engine>`) and expl3's
/// `\__msg_fatal_exit:` (`\batchmode\read -1 to …`). Both were no-ops, so
/// XeTeX-only packages ran their bodies on undefined XeTeX primitives until a
/// 3.3 GB log-buffer allocation failed (23 docs in sweep 57: bidi, xepersian,
/// polyglossia-xetex, ucharclasses/latexbangla). KPE #213, DIVERGENCES #220.
#[test]
fn batchmode_terminal_read_halts_the_job() {
  let (stderr, _) = convert(
    "\\documentclass{article}\n\\begin{document}\nBefore.\n\\batchmode\\read-1 to\\x\nAfter.\n\\end{document}\n",
    true,
  );
  assert!(
    stderr.contains("Fatal:") && stderr.contains("cannot \\read from terminal"),
    "{stderr}"
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  if kpsewhich_has("iftex.sty") {
    // A non-Unicode-native engine still halts (`\RequireXeTeX` passes: see
    // `xetex_only_packages_load_under_the_default_persona`).
    let (stderr, _) = convert(
      "\\documentclass{article}\n\\usepackage{iftex}\n\\RequirepTeX\n\\begin{document}\nx\n\\end{document}\n",
      true,
    );
    assert!(
      stderr.contains("Fatal:") && stderr.contains("pTeX is required"),
      "{stderr}"
    );
    assert_eq!(error_count(&stderr), 1, "{stderr}");
    // The engine we DO present passes its own guard.
    let (stderr, xml) = convert(
      "\\documentclass{article}\n\\usepackage{iftex}\n\\RequirePDFTeX\\RequireeTeX\n\\begin{document}\nAfter.\n\\end{document}\n",
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      !stderr.contains("Fatal:") && xml.contains("After."),
      "{stderr}"
    );
  }
  // Scroll/errorstop mode (the default; no terminal here): a closed-stream
  // read stays a no-op, as in Perl.
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\begin{document}\n\\read16 to\\x After.\n\\end{document}\n",
    true,
  );
  assert!(
    !stderr.contains("Fatal:") && xml.contains("After."),
    "{stderr}"
  );
}

/// The K1-step-3 stub audit (2026-09-06, 8 packages) found two silent drops:
/// datetime's `\newdate`/`\displaydate` no-ops (datetime.sty:149-172) and
/// mdframed's `\mdfsubtitle` no-op (mdframed.sty:1312-1347). Both now carry
/// their text.
#[test]
fn datetime_named_dates_and_mdframed_subtitles_are_kept() {
  if kpsewhich_has("datetime.sty") && kpsewhich_has("mdframed.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{datetime}\n\\usepackage{mdframed}\n\\begin{document}\n\\newdate{lx}{5}{9}{2026}Date: \\displaydate{lx}.\n\\begin{mdframed}\nBody.\n\\mdfsubtitle{Sub Title}\nMore.\n\\end{mdframed}\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("September") && xml.contains("2026"), "{xml}");
    assert!(
      xml.contains("Sub Title") && xml.contains("Body.") && xml.contains("More."),
      "{xml}"
    );
  }
}

/// biblatex.sty:10757-10769 `\refsection` takes an OPTIONAL resource list only;
/// the binding's `[]{}` signature swallowed the `\begin` of the environment that
/// followed `\begin{refsection}[…]`, so `\end{otherlanguage}` closed the
/// refsection group and `\end{refsection}` hit the document frame ("Attempt to
/// close a group that switched to mode horizontal"; biblatex-apa-test:1203-1208).
#[test]
fn refsection_takes_only_an_optional_resource_list() {
  if kpsewhich_has("biblatex.sty") && kpsewhich_has("babel.sty") {
    let tex = "\\documentclass{article}\n\\usepackage[ngerman,american]{babel}\n\\usepackage[style=apa]{biblatex}\n\\begin{document}\n\\begin{refsection}[foo.bib]\n\\begin{otherlanguage}{ngerman}\nx\n\\end{otherlanguage}\n\\end{refsection}\nAfter.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(!xml.contains("otherlanguagengerman"), "{xml}");
    // babel's `otherlanguage` wraps the body it DID open: `<text xml:lang="de">x`.
    assert!(
      xml.contains("xml:lang=\"de\">x") && xml.contains("After."),
      "{xml}"
    );
    // A biber `.bbl` opens with `\refsection{0}` — the bbl-time variant with a
    // MANDATORY section number (biblatex.sty:8634, re-let by `\blx@bblstart`
    // :8999-9000). The old `[]{}` signature ate that `{0}` by accident; the
    // optional-only document macro must not typeset a stray "0".
    let bbl = include_str!("../cluster_regressions/biblatex_ay/declarecite.bbl");
    let tex = "\\documentclass{article}\n\\usepackage[style=authoryear]{biblatex}\n\\addbibresource{x.bib}\n\\begin{document}\nCite \\cite{smith2020}.\n\\printbibliography\n\\end{document}\n";
    let (stderr, xml) = convert_files(tex, &[("t.bbl", bbl)]);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    // The `.bbl`'s entries (biblatex_sty.rs `bbl_flush`).
    assert!(
      xml.contains("<bibentry") || xml.contains("<ltx:bibentry"),
      "{xml}"
    );
    let mut text = String::new();
    let mut in_tag = false;
    for c in xml.chars() {
      match c {
        '<' => in_tag = true,
        '>' => in_tag = false,
        _ if !in_tag => text.push(c),
        _ => {},
      }
    }
    assert!(
      !text.split_whitespace().any(|w| w == "0"),
      "stray bbl refsection number: {xml}"
    );
  }
}

/// xkeyval's EMPTY family is a family: pst-xkey.tex:53-57 accumulates
/// `\pst@famlist` as ",pstricks" and pstricks.tex:808-810 defines
/// `precode`/`postcode`/`exchange` in it (pst-node.tex the `Xnodesep` six).
/// `KeyVals::new` dropped the empty entry (Perl KeyVals.pm:52); now every
/// `\psset` searches it and the pstricks load emits no "unknown key" warning.
/// The `@`-count of the key macro (xkeyval.tex:83-88 header rule,
/// `\psset@precode`, leaving `\psset@@dash` to pstricks) is pinned by the unit
/// test `keyval_qname_normalizes_empty_prefix`, not here.
/// KNOWN_PERL_ERRORS #212, DIVERGENCES #219.
#[test]
fn xkeyval_empty_family_is_searched_by_psset() {
  if kpsewhich_has("pstricks.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{pstricks}\n\\makeatletter\n\\define@key[psset]{}{lxfoo}{\\gdef\\lxres{GOT-#1}}\n\\makeatother\n\\begin{document}\n\\psset{lxfoo=42,dash=3pt 2pt}\\lxres\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("GOT-42"), "{xml}");
    for key in ["precode", "postcode", "exchange", "lxfoo"] {
      assert!(
        !stderr.contains(&format!("unknown KeyVals key '{key}'")),
        "{stderr}"
      );
    }
  }
}

/// pst-grad.sty:3 is `\input{pst-grad.tex}`; the binding (like Perl's) stopped
/// at `\RequirePackage{pstricks}`, so once `\pst@object` dispatched for real a
/// raw object's `[fillstyle=gradient,…,addfillstyle=boxfill]` bracket raised
/// "Undefined fill style" and, with `\psk@fillstyle` left at its `\relax`
/// default, an `addfillstyle` self-recursion (arabi/big2, exposed by batch
/// 56af). DIVERGENCES #218.
#[test]
fn pst_grad_binding_loads_the_gradient_fill_styles() {
  if kpsewhich_has("pst-grad.tex") && kpsewhich_has("pst-char.sty") && kpsewhich_has("pst-fill.sty")
  {
    let tex = "\\documentclass{article}\n\\usepackage[tiling]{pst-fill}\n\\usepackage{pst-text,pst-char,pst-grad}\n\\begin{document}\nBefore.\n\\psboxfill{\\small x}\n\\begin{pspicture}(0,0)(6,2)\n\\pscharpath[linestyle=none,gradbegin=magenta,gradend=cyan,fillstyle=gradient,gradangle=-30,gradmidpoint=0.5,addfillstyle=boxfill]{\\rput[b](1,0){\\Huge Word}}\n\\end{pspicture}\nAfter.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("Before.") && xml.contains("After."), "{xml}");
  }
}

/// pstricks.tex:1453-1461 `\pst@object{name}` dispatches to `\<name>@i` after
/// the `*`/`[…]` options; the support binding's stub (`#1`, no Perl
/// counterpart) typeset the NAME instead, so pst-node's `\psm@beginnode`
/// never opened its node box and psmatrix's v-part closers popped the
/// alignment's own cell frame (dsptricks 98; every psmatrix).
#[test]
fn pst_object_dispatches_to_the_object_body() {
  if kpsewhich_has("pstricks-add.sty") {
    let tex = "\\documentclass{article}\n\\usepackage{pstricks-add}\n\\makeatletter\n\\def\\lxfoo@i{FOO-\\pst@par-END}\n\\makeatother\n\\begin{document}\n\\makeatletter\\pst@object{lxfoo}[linewidth=2pt]\\makeatother\n\\begin{psmatrix} A & B \\\\ C & D \\end{psmatrix}\nAfter.\n\\end{document}\n";
    let (stderr, xml) = convert(tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(
      xml.contains("FOO-linewidth=2pt-END") && xml.contains("After."),
      "{xml}"
    );
    assert_eq!(xml.matches("<td").count(), 4, "{xml}");
  }
}

/// \SetCatcodeRange and \lstloadaspects support (witness codebox-doc-en).
#[test]
fn luatex_catcoderange_and_listings_aspects() {
  let tex = r"\documentclass{article}
\usepackage{luatexbase}
\SetCatcodeRange{65}{90}{11}
\usepackage{listings}
\lstloadaspects{comments}
\begin{document}
Listing test.
\SetCatcodeRange{`A}{`Z}{12}\typeout{CAT:\the\catcode`Q:\the\catcode`q}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Listing test."), "{xml}");
  // ctablestack.sty:18: a real `\catcode` loop over the range, nothing else.
  assert!(stderr.contains("CAT:12:11"), "{stderr}");
}

/// \DeclareTCBListing invoked via bare macros inside \NewDocumentEnvironment
/// (witness codebox-doc-en \begin{codeview} calling \codeviewaux).
#[test]
fn declare_tcb_listing_nested_in_document_environment() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings,xparse}
\DeclareTCBListing{mycodeaux}{m}{title={Title #1},listing only}
\NewDocumentEnvironment{mycode}{O{} m}
  {\mycodeaux{#2}}
  {\endmycodeaux}
\begin{document}
\begin{mycode}{My Title}
#include <stdio.h>
int main() { return 0; }
\end{mycode}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("include"), "{xml}");
  assert!(xml.contains("stdio"), "{xml}");
}

/// unicode-math symbol table loading and ctex LuaTeX math letter hooks
#[test]
fn unicode_math_table_loading_and_ctex_hooks() {
  let tex = r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\cs_if_exist:NTF \__um_input_math_symbol_table: { \__um_input_math_symbol_table: } {}
\cs_if_exist:NTF \um_input_math_symbol_table: { \um_input_math_symbol_table: } {}
\cs_if_exist:NTF \__um_load_symbols: { \__um_load_symbols: } {}
\cs_if_exist:NTF \__um_switchto_literal: { \__um_switchto_literal: } {}
\ExplSyntaxOff
\begin{document}
Table hooks ok.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Table hooks ok."), "{xml}");
}

/// `\set@fontsize` reads its baselineskip argument through
/// `\@defaultunits…pt\relax\@nnil` (latex.ltx:12588), so a class's bare
/// `\@setfontsize\normalsize\@xipt{18}` (jpsj2.cls:317; KOMA's
/// scrsize11pt.clo:99 the same shape) is 18pt. The binding read
/// `\baselineskip#3\relax` bare — one "Illegal unit of measure (pt
/// inserted)" warning per size switch, surfaced by K11 typesetting a raw
/// class's `\@maketitle` (sweep #94: injpsj2 +6, TUDaPhD +3, BFHThesis +2).
#[test]
fn setfontsize_baselineskip_defaults_to_pt() {
  let tex = r"\documentclass{article}
\makeatletter
\renewcommand\normalsize{\@setfontsize\normalsize\@xpt{18}}
\makeatother
\begin{document}
\normalsize Body text. \the\baselineskip.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    stderr.matches("Illegal unit of measure").count(),
    0,
    "{stderr}"
  );
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>Body text. 18.0pt.</p>"##);
}

/// `\newif\if西暦` (jsarticle.cls:1927): the kanji is catcode OTHER in both
/// engines, so `\newif` names the bare `\if` — an EMPTY conditional name,
/// which Perl (Package.pm:1220, `defined $name`) lets to `\iffalse` like any
/// `\newif`. Rejecting it installed a test-less conditional and the
/// `\if`-heavy tikz→pgf→pgfkeys load collapsed (`\pgfeov` undefined at
/// pgfkeys.code.tex:931; jsarticle + tikz 544 errors, Perl 28 — the pTeX
/// residual; chuushaku 423/343).
#[test]
fn newif_with_an_empty_name_lets_if_to_iffalse() {
  let tex = std::fs::read_to_string("tests/cluster_regressions/newif/empty_name_tikz.tex")
    .expect("fixture");
  let (stderr, xml) = convert(&tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<picture"), "{xml}");
  assert!(xml.contains("<svg:path"), "{xml}");
}

/// Batch 56dy, the witness: the math parser queues every formula up front
/// and rebuilds each in place, freeing replaced originals only after the
/// whole parse (`replace_tree_deferred`). `replace_tree` freeing the
/// original immediately (56dx) handed a still-queued formula inside it to
/// the parser detached — a Fatal and an EMPTY document for four sweep-102
/// documents. The glosmathtools sample (TeX Live doc, two files) is the
/// smallest of them and reproduces only on the raw-styles path; the
/// synthetic nested-`\text` fixture below does not, so this is the guard.
#[test]
fn glosmathtools_sample_is_not_emptied_by_the_math_rebuild() {
  if !kpsewhich_has("glosmathtools.sty") || !kpsewhich_has("ulthese.cls") {
    eprintln!("skipping: glosmathtools.sty / ulthese.cls not in the host TeX tree");
    return;
  }
  let tex = include_str!("../cluster_regressions/glosmathtools/sample_glosmathtools_en.tex");
  let glos = include_str!("../cluster_regressions/glosmathtools/sample_glosmathtools_glos.tex");
  let (stderr, xml) = convert_files(tex, &[("sample_glosmathtools_glos.tex", glos)]);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  assert!(
    !stderr.contains("detached before it could be parsed"),
    "{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The whole document, not an empty result: sweep 101's output holds 47
  // formulae, 21 of them with nested text — plus, since the glossaries fields
  // are absorbed digested (batch 56hm), the 30 symbol formulae (3 with nested
  // text) of the glossary definitions, which were flattened text before. Since
  // batch 56jt a `\gls` term in math is typeset alone, not as a reference
  // holding a formula of its own: 27 body formulae, 1 with nested text.
  assert!(xml.contains("</document>"), "{xml}");
  assert_eq!(xml.matches("<Math ").count(), 57, "{xml}");
  assert_eq!(xml.matches("<XMText").count(), 4, "{xml}");
}

/// Nested `\text{…$x$…}` inside math: three inner formulae under XMText
/// survive their outer formulae's rebuild (a structural check; it does NOT
/// reproduce the 56dx failure, which needs the witness above).
#[test]
fn nested_text_math_survives_the_outer_rebuild() {
  let tex = include_str!("../cluster_regressions/math_nested_text_math.tex");
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("detached before it could be parsed"),
    "{stderr}"
  );
  // Two outer formulae and the three inner ones (each its own `ltx:Math`
  // under an `XMText`) all parsed: the inner relations survive.
  assert_eq!(xml.matches("<Math ").count(), 5, "{xml}");
  assert_eq!(xml.matches("<XMText>").count(), 3, "{xml}");
  assert!(
    xml.contains(r#"<XMTok meaning="greater-than" role="RELOP">&gt;</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok meaning="not-equals" name="neq" role="RELOP">≠</XMTok>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok meaning="element-of" name="in" role="RELOP">∈</XMTok>"#),
    "{xml}"
  );
}

/// K12: under a pLaTeX class (`\NeedsTeXFormat{pLaTeX2e}`, jsarticle.cls:14)
/// kanji and kana join control-word names as in pTeX (upTeX kcatcodes 16/17;
/// ptex-manual `\黄マーカー`), so `\newif\if西暦` (jsarticle.cls:1927) defines
/// `\if西暦` instead of letting the bare `\if` to `\iffalse` — the shared
/// degradation that broke every later `\if` (chuushaku 73 errors,
/// sample-bxjaprnind's runaway Fatal). Under `article` kanji stays OTHER
/// (`non_ascii_letters_stay_other_under_pdftex`). The pTeX engine
/// primitives (`\kanjiskip`…) stay undefined, PARKED, as in Perl.
#[test]
fn kanji_control_words_under_platex() {
  // The declaration sets the pTeX profile, in a class (as jsarticle.cls:14; jsarticle's own body
  // needs the parked pTeX-format registers `\hour`, `\jfam`, `\kanjiskip`…, §D9, so a local
  // class stands in for it) and in the document.
  let cls = "\\NeedsTeXFormat{pLaTeX2e}\n\\ProvidesClass{lxjs}\n\\LoadClass{article}\n";
  let body = r"\makeatletter
\newif\if西暦\西暦true\def\foo{}
\begin{document}
A:\if西暦 YES\else NO\fi. C:\expandafter\string\csname if西暦\endcsname. D:\foo々Y.
\end{document}
";
  let (stderr, xml) = convert_files_with(
    &format!("\\documentclass{{lxjs}}\n{body}"),
    &[("lxjs.cls", cls)],
    Some("[rawstyles,rawclasses]latexml.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>A:YES. C:“if西暦. D:々Y.</p>"##);
  let tex = r"\NeedsTeXFormat{pLaTeX2e}
\documentclass{article}
\makeatletter
\newif\if西暦\西暦true\def\foo{}
\begin{document}
A:\if西暦 YES\else NO\fi. C:\expandafter\string\csname if西暦\endcsname. D:\foo々Y.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("undefined:\\西").count(), 0, "{stderr}");
  assert_eq!(stderr.matches("undefined:\\if ").count(), 0, "{stderr}");
  assert_eq!(stderr.matches("undefined:\\foo々").count(), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>A:YES. C:“if西暦. D:々Y.</p>"##);
  let art = r"\documentclass{article}
\begin{document}
B:\ifcat A西 L\else O\fi.
\end{document}
";
  let (stderr, xml) = convert_with(art, Some("[rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>B:O.</p>"##);
}

/// Batch 56ek (OXIDIZED_DESIGN #240): a block box whose content is Para.class
/// (a `<float>`, here via framed's `{shaded}`) inserted while a `<para>` is
/// open from preceding inline text must END the paragraph and place the block
/// at the enclosing flow level — not rename the capture in place to a
/// schema-invalid `<para>/<logical-block>`. Witnesses tikz-network (88→1 jing),
/// numerica.
#[test]
fn logical_block_climbs_out_of_para() {
  let tex = "\\documentclass{article}\n\\usepackage{framed}\n\\usepackage{float}\n\\begin{document}\nIntro text.\n\\begin{shaded}\n\\begin{figure}[H]\\caption{c}\\end{figure}\n\\end{shaded}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let compact: String = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  // The logical-block must NOT sit inside the <para> (the schema violation):
  // after the fix a `</para>` closes before it.
  assert!(
    !compact.contains("</p> <logical-block"),
    "logical-block must not be a child of the open <para>\n{xml}"
  );
  // The framed block still emits a logical-block (holding the float).
  assert!(
    compact.contains("<logical-block"),
    "the framed block must still emit a <logical-block>\n{xml}"
  );
  // The whole climbed block, with text after the float (57g review).
  let (stderr, xml) = convert(
    "\\documentclass{article}\n\\usepackage{framed}\n\\usepackage{float}\n\\begin{document}\nIntro text.\n\\begin{shaded}\n\\begin{figure}[H]\\caption{c}\\end{figure} Some text after.\n\\end{shaded}\n\\end{document}\n",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "logical-block",
    &[r#"framed="rectangle""#],
    r##"<logical-block backgroundcolor="#000000" cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle"><figure inlist="lof" placement="H" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>c</toccaption><caption><tag close=": ">Figure 1</tag>c</caption></figure><para xml:id="p2"><p>Some text after.</p></para></logical-block>"##,
  );
}

/// Batch 56ep (OXIDIZED_DESIGN #242): a content-free leading `<pagination>`
/// (from `\clearpage`/`\newpage`/`\frontmatter` before `\maketitle`) must not
/// precede the document frontmatter. The schema requires frontmatter to lead
/// (`LaTeXML-structure.rnc:34`), so a `<pagination>` before `<title>` invalidates
/// every following frontmatter element. Render-safe surpass beyond Perl (Perl-raw
/// leaves this invalid too): a pagebreak reorders nothing visible, so
/// `\lx@frontmatterhere` hoists the QUEUED frontmatter above the content-free
/// leading nodes. Witnesses: amsmath/amsldoc, tkz-doc/tkz-doc, tuda-ci/DEMO-TUDaPhD,
/// tzplot/tzplot-doc (14 s105 docs recovered to 0 rng-errors). NOT hoisted here:
/// genuine pre-title content (corpus B-subclass — stays invalid by design); a
/// directly-built <titlepage> element after a leading pagination (toptesi
/// frontispiece) and a leading empty-box paragraph (gitinfo2/gitlog) — both now
/// settled by the `\end{document}` relocation pass of batch 56es (#245, below).
#[test]
fn frontmatter_hoists_above_content_free_leading_pagination() {
  let tex = "\\documentclass{book}\n\\title{T}\\author{A}\n\\begin{document}\n\
               \\frontmatter\n\\clearpage\n\\maketitle\n\\chapter{C}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let title = xml
    .find("<title>")
    .expect("frontmatter <title> must be present");
  let pagination = xml
    .find("<pagination")
    .expect("the \\clearpage <pagination> must still be present (not dropped)");
  assert!(
    title < pagination,
    "frontmatter <title> must lead — hoisted above the content-free leading \
       <pagination> pagebreak, not emitted after it:\n{xml}"
  );
}

/// Batch 56gf (OXIDIZED_DESIGN #262, user-approved surpass; supersedes the 56ep
/// control): visible content before `\maketitle` — a `\includegraphics` logo on
/// the cover — no longer holds the frontmatter back. The schema puts the front
/// group first, so `<title>` leads and the logo follows it, kept whole.
#[test]
fn frontmatter_leads_a_leading_graphic() {
  let tex = "\\documentclass{book}\n\\usepackage{graphicx}\n\\title{T}\\author{A}\n\
               \\begin{document}\n\\noindent\\includegraphics{logo}\n\\clearpage\n\
               \\maketitle\n\\chapter{C}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  let graphics = xml
    .find("<graphics")
    .expect("the leading \\includegraphics <graphics> must be present");
  let title = xml
    .find("<title>")
    .expect("frontmatter <title> must be present");
  assert!(
    title < graphics,
    "the frontmatter leads; the logo follows it:\n{xml}"
  );
  assert!(
    !xml[..title].contains("<para"),
    "nothing precedes the title:\n{xml}"
  );
}

/// Batch 56es (OXIDIZED_DESIGN #245): the `\end{document}` relocation pass. A
/// `\cleardoublepage` (memoir `\frontmatter`; gitinfo2/gitlog) emits `\clearpage
/// \hbox{} \newpage` → `<pagination>, <para><p/></para>, <pagination>` BEFORE the
/// `\maketitle` flush, and #242's construct-time gate declined because the empty
/// box's `<ltx:text>` is only folded into its `<p>` when the paragraph closes (now an
/// ink-free `<ltx:text>` is transparent to the gate too). The finalization pass sees
/// the settled tree and moves the content-free run past the frontmatter, so `<title>`
/// leads (0 rng-errors) and nothing is dropped.
#[test]
fn frontmatter_relocates_past_a_leading_empty_box_pagebreak() {
  let tex = "\\documentclass{report}\n\\begin{document}\n\\clearpage\\hbox{}\\newpage\n\
               \\title{T}\\author{A}\\date{D}\n\\maketitle\nBody text.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  // `error_count` only rules out gross binding errors: the core stage's model check
  // is order-insensitive, so the pre-fix misordering emitted no Error — the ORDER
  // asserts below are the teeth (the sweep's jing validation is what counts rng-errors).
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let title = xml
    .find("<title>")
    .expect("frontmatter <title> must be present");
  let date = xml
    .find("<date")
    .expect("frontmatter <date> must be present");
  let pagination = xml
    .find("<pagination")
    .expect("the pagebreak <pagination> must still be present (not dropped)");
  let empty_para = xml
    .find("<p/>")
    .expect("the empty-box paragraph must still be present (not dropped)");
  assert!(
    title < date && date < pagination && pagination < empty_para,
    "frontmatter (<title>…<date>) must lead, with the content-free pagination + \
       empty paragraph relocated after it in their original order:\n{xml}"
  );
  assert!(
    xml.find("Body text.").unwrap() > empty_para,
    "the body must stay after the relocated nodes:\n{xml}"
  );
}

/// Batch 56es (OXIDIZED_DESIGN #245): a `{titlepage}` environment is not queued
/// frontmatter — it is built in place, so a preceding `\clearpage` `<pagination>`
/// opens the body group and makes the `<titlepage>` (and the `\title`/`\author`
/// its `after_construct` flushes right behind it) schema-invalid (toptesi
/// frontispiece: FrontespizioScudo, toptesi-example-*). The relocation pass moves
/// the pagination past the whole contiguous frontmatter run.
#[test]
fn titlepage_relocates_past_a_leading_pagination() {
  let tex = "\\documentclass{report}\n\\begin{document}\n\\clearpage\n\
               \\begin{titlepage}\\title{T}\\author{A}\nCover line.\\end{titlepage}\n\
               \\chapter{C}\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let titlepage = xml.find("<titlepage").expect("<titlepage> must be present");
  let creator = xml
    .find("<creator")
    .expect("the flushed <creator> must be present");
  let pagination = xml
    .find("<pagination")
    .expect("the \\clearpage <pagination> must still be present (not dropped)");
  let chapter = xml.find("<chapter").expect("<chapter> must be present");
  assert!(
    titlepage < creator && creator < pagination && pagination < chapter,
    "the <titlepage> + flushed frontmatter must lead, the pagebreak relocated after \
       them and before the body:\n{xml}"
  );
}

/// Batch 56es control (OXIDIZED_DESIGN #245): a leading node with visible ink is
/// NOT content-free, so the pass leaves the tree alone — a hand-typeset cover line
/// above a `{titlepage}` (toptesi-it's TeX-logo cover) stays above it, invalid but
/// faithful; the pass never reorders visible content.
#[test]
fn titlepage_stays_below_a_visible_leading_cover() {
  let tex = "\\documentclass{report}\n\\begin{document}\n\\noindent Cover line.\\clearpage\n\
               \\begin{titlepage}Title page.\\end{titlepage}\n\\chapter{C}\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let cover = xml
    .find("Cover line.")
    .expect("the cover line must be present");
  // 56fw (#257): the stranded `{titlepage}` is demoted in place to a layout
  // paragraph — same text, same order, no wrapper the schema forbids there.
  assert!(!xml.contains("<titlepage"), "{xml}");
  let titlepage = xml
    .find(r#"<para class="ltx_titlepage""#)
    .expect("the demoted titlepage paragraph must be present");
  assert!(
    cover < titlepage && titlepage < xml.find("Title page.").unwrap(),
    "a visible leading cover line must stay ABOVE the title-page content — the \
       relocation pass must not move visible content:\n{xml}"
  );
}

/// 56gc (OXIDIZED_DESIGN #260): an end-of-line space token has character
/// code 32 like any space (tex.web §289/§349); Perl and this port reported
/// 10, so stringstrings' `\if\BlankSpace#1` took the wrong branch on a
/// `\@for` item that was a lone newline and desynchronised the conditional
/// stack (`Extra \else` ×4 in tikzviolinplots, then a runaway KDE loop).
#[test]
fn end_of_line_space_compares_equal_to_a_space_in_if() {
  let tex = "\\documentclass{article}\n\\usepackage{ifthen}\n\\usepackage{stringstrings}\n\\makeatletter\n\
               \\begin{document}\n\\def\\vao{xmin=0,\n}\n\\@for\\kdeoption:=\\vao\\do{\\whereisword[q]{\\kdeoption}{xmin}}\n\
               \\makeatother\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Extra"), "{stderr}");
  assert!(xml.contains("<document") && xml.contains("Body."), "{xml}");
  // The kernel form: `\if` on an end-of-line space and a typed space is TRUE.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\def\\a{\n}\\def\\b{ }\n\
               \\expandafter\\expandafter\\expandafter\\if\\expandafter\\a\\b YES\\else NO\\fi\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("YES") && !xml.contains("NO"), "{xml}");
}

/// 56gb: pgf's `divide(x, 0)` is `x` — TeX's `\divide` by zero leaves the
/// register unchanged (tex.web §107/§1240; pgfmathfunctions.basic.code.tex:66).
/// Our epsilon divisor returned `x / 0.00001`, so a decoration whose segment
/// length degraded to 0 computed a ~1 sp step from `len / int(len / 0)` and
/// pgf's automaton walked into the pushback limit (carbohydrates_en, RUST-ONLY).
#[test]
fn pgfmath_division_by_zero_returns_the_dividend() {
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\
               \\pgfmathparse{16.61/0}A\\pgfmathresult B\\pgfmathparse{int(16.61/0)}\\pgfmathresult C\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Perl warns on every zero divisor (pgfmath.code.tex.ltxml:255); so do we.
  assert!(
    stderr.contains("Warning:unexpected:<number> pgfmath: divisor should never be zero!"),
    "{stderr}"
  );
  // TeX gobbles the space after `\pgfmathresult`.
  assert!(xml.contains("A16.61B16C"), "{xml}");
  let tex = "\\documentclass{article}\n\\usepackage{tikz}\n\\usetikzlibrary{decorations}\n\
               \\pgfdeclaredecoration{cf loop}{initial}{\n\
               \\state{initial}[width=+0pt,next state=seg,persistent precomputation={\n\
               \\pgfmathsetmacro\\ml{\\pgfdecoratedinputsegmentlength/int(\\pgfdecoratedinputsegmentlength/\\pgfdecorationsegmentlength)}\n\
               \\setlength{\\pgfdecorationsegmentlength}{\\ml pt}}]{}\n\
               \\state{seg}[width=\\pgfdecorationsegmentlength]{\\pgfpathlineto{\\pgfpoint{\\pgfdecorationsegmentlength}{0pt}}}\n\
               \\state{final}{}}\n\\begin{document}\n\
               \\begin{tikzpicture}\\draw[decorate,decoration={cf loop,segment length=0pt}](0,0)--(3,0);\\end{tikzpicture}\n\
               \\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(
    xml.contains("<svg:svg") && xml.contains("<svg:path"),
    "the decorated path is drawn:\n{xml}"
  );
}

/// 56fy: `cleanup_xmtext` collapses a sole inline-block child into the
/// `XMText` and copied its attributes raw — a `\rotatebox`/`\raisebox` in a
/// `\text{}` of UNPARSED math left `angle`/`innerdepth`/… on `XMText`, which
/// its model forbids (principia's `\pmcexists`: six schema errors; Perl's
/// schema-gated setter drops them). Now the schema-gated setter.
#[test]
fn rotated_box_in_unparsed_math_text_keeps_no_transform_attributes() {
  let tex = "\\documentclass{article}\n\\usepackage{amsmath,graphicx}\n\\begin{document}\n\
               \\[ \\text{\\raisebox{5.0pt}{\\rotatebox{180.0}{{E}}}}\\hskip-1.00006pt\\mathop{\\textbf{!}} \\times\\times x \\]\n\
               \\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The defect lives on the unparsed path only (a parse rebuilds the XMText);
  // pin it so a future parser gain cannot make this guard vacuous. The bare
  // formula parses since 57w (`E` times the bigop, as Perl parses it), hence the
  // ungrammatical `\times\times x` tail, which warns as Perl's `not_parsed` does.
  assert!(xml.contains("ltx_math_unparsed"), "{xml}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(stderr.lines().any(|l| l.trim_end()
    == "Warning:unparsed_math:ATOM_BIGOP_MULOP_MULOP_UNKNOWN Unparsed math: no parse spans all input \
          tokens for: ATOM:E:1 BIGOP:!:2 MULOP:times:3 MULOP:times:4 UNKNOWN:x:5"), "{stderr}");
  let xmtext = xml.find("<XMText").expect("XMText");
  let end = xmtext + xml[xmtext..].find('>').unwrap();
  let tag = &xml[xmtext..end];
  for attr in [
    "angle=",
    "innerdepth=",
    "innerheight=",
    "innerwidth=",
    "xtranslate=",
    "ytranslate=",
  ] {
    assert!(!tag.contains(attr), "{attr} on XMText:\n{tag}\n{xml}");
  }
  assert!(xml.contains(">E<"), "the rotated glyph survives:\n{xml}");
}

/// OXIDIZED_DESIGN #258: a leading paragraph holding ONLY undefined-command
/// markers (`<ERROR class="undefined">\foo</ERROR>`, no text outside them) is
/// content-free for the frontmatter hoist — the marker is diagnostic ink, not
/// document content — so `\title`/`\author` placed after it lead the document
/// and the marker follows them (pst-calendar-doc, forest-doc, pmhanguljamo;
/// 8 docs). A marker whose argument was typeset as text is NOT content-free.
#[test]
fn frontmatter_hoists_over_an_error_marker_only_paragraph() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\ThisCommandIsUndefined\n\
               \\title{Sample Title}\n\\author{An Author}\n\\maketitle\n\\section{Intro}\nBody text here.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Error:undefined:\\ThisCommandIsUndefined"),
    "{stderr}"
  );
  let title = xml.find("<title>Sample Title</title>").expect("title");
  let marker = xml
    .find("<ERROR class=\"undefined\">\\ThisCommandIsUndefined</ERROR>")
    .expect("marker");
  let section = xml.find("<section").unwrap();
  assert!(
    title < marker && marker < section,
    "the marker follows the frontmatter:\n{xml}"
  );
  assert!(
    !xml[..title].contains("<para"),
    "no paragraph precedes the title:\n{xml}"
  );
  // Typeset argument text is visible content: since 56gf (#262) the frontmatter
  // leads it too, and the text follows, whole.
  let tex = "\\documentclass{article}\n\\begin{document}\n\\ThisCommandIsUndefined{Visible words}\n\
               \\title{Sample Title}\n\\author{An Author}\n\\maketitle\n\\section{Intro}\nBody text here.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    xml.contains("<ERROR class=\"undefined\">\\ThisCommandIsUndefined</ERROR>"),
    "the marker is still reported:\n{xml}"
  );
  let title = xml.find("<title>Sample Title</title>").expect("title");
  let words = xml.find("Visible words").expect("argument text");
  assert!(
    title < words,
    "the frontmatter leads the visible text (#262):\n{xml}"
  );
}

/// 56fw (OXIDIZED_DESIGN #257): a `{titlepage}` entered after leaked leading text
/// (chemexec_en:119, stanli, l2picfaq, pst-calendar-doc) was emitted as a stranded
/// `<titlepage>` after the body's first paragraph — Perl identical, one schema
/// error. It is now a layout paragraph; a titlepage at its proper leading position
/// keeps its element.
#[test]
fn stranded_titlepage_becomes_a_layout_paragraph() {
  let tex = "\\documentclass{article}\n\\begin{document}\nLeaked leading text.\n\
               \\begin{titlepage}\\centering{\\Large A Hand-Typeset Cover}\\end{titlepage}\n\
               \\section{Intro}\nBody text here.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<titlepage"), "{xml}");
  let leaked = xml.find("Leaked leading text.").unwrap();
  let para = xml
    .find(r#"<para class="ltx_titlepage""#)
    .expect("demoted paragraph");
  let cover = xml.find("A Hand-Typeset Cover").unwrap();
  let section = xml.find("<section").unwrap();
  assert!(leaked < para && para < cover && cover < section, "{xml}");
  // Control: at the leading position the element is kept.
  let tex = "\\documentclass{article}\n\\begin{document}\n\
               \\begin{titlepage}\\centering{\\Large A Hand-Typeset Cover}\\end{titlepage}\n\
               \\section{Intro}\nBody text here.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<titlepage>"), "{xml}");
}

/// Batch 56es control (OXIDIZED_DESIGN #245): the frontmatter run is the document
/// model's FIRST group only. `ltx:document` also admits BackMatter.class
/// (`bibliography`/`appendix`/`index`/`glossary`) that body-only content does not,
/// so a naive "document admits it, sectional-block doesn't" test would carry a
/// leading pagebreak past a whole bibliography. It must stay where the source put it.
#[test]
fn pagebreak_before_a_leading_bibliography_stays_put() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\clearpage\n\
               \\begin{thebibliography}{9}\\bibitem{a} An entry.\\end{thebibliography}\n\
               \\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let pagination = xml
    .find("<pagination")
    .expect("the \\clearpage <pagination> must be present");
  let bibliography = xml
    .find("<bibliography")
    .expect("<bibliography> must be present");
  assert!(
    pagination < bibliography,
    "a pagebreak before a leading <bibliography> (BackMatter, not frontmatter) must \
       not be relocated past it:\n{xml}"
  );
}

/// Batch 56gf (OXIDIZED_DESIGN #262; supersedes the 56es control): an ink-bearing
/// empty box before `\maketitle` (`\fbox{}`, an `<ltx:text framed="rectangle">`)
/// is visible body content; the frontmatter still leads and the box follows it.
#[test]
fn frontmatter_leads_a_leading_framed_empty_box() {
  let tex = "\\documentclass{report}\n\\begin{document}\n\\noindent\\fbox{}\\clearpage\n\
               \\title{T}\\author{A}\n\\maketitle\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  let framed = xml
    .find("framed=")
    .expect("the \\fbox{} <text framed> must be present");
  let title = xml
    .find("<title>")
    .expect("frontmatter <title> must be present");
  assert!(
    title < framed,
    "the frontmatter leads; the framed box follows it:\n{xml}"
  );
}

/// Batch 56et (OXIDIZED_DESIGN #246): with no `\maketitle`, an abstract-only
/// document's frontmatter is flushed by `\lx@frontmatter@fallback` at the first
/// `\section`. A beyond-Perl rescue flushed it at the CURRENT position, so the
/// `<abstract>` landed after every cover block, `<TOC>` and pagebreak the body had
/// emitted — schema-invalid (`Para.class |= TOC`) and later than the source order;
/// RUST-ONLY (Perl top-places: Base_Utility.pool.ltxml:927-945). Witnesses
/// tikz-mirror-lens, tipfr-doc, pgf-interference-{en,de}, axodraw2-man, derivative,
/// russ_doc, schulmathematik, jourcl, isosigns-docs, bootstrapicons-docs (16 s106 docs).
#[test]
fn abstract_only_fallback_floats_to_top() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{center}\n\
               {\\large A Package Manual}\n\\end{center}\n\\begin{abstract}\nThis is the \
               abstract of the manual.\n\\end{abstract}\n\\tableofcontents\n\
               \\section{Introduction}\nBody text.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let abstract_ = xml.find("<abstract").expect("<abstract> must be present");
  let toc = xml
    .find("<TOC")
    .expect("the \\tableofcontents <TOC> must be present");
  let section = xml.find("<section").expect("<section> must be present");
  assert!(
    abstract_ < toc && toc < section,
    "the queued <abstract> must lead the document (above the <TOC>), not trail the \
       body at the first-\\section flush point:\n{xml}"
  );
}

/// Batch 56et must-not-regress witness (arXiv 1609.07638, the case the old
/// current-position rescue was written for): a hand-formatted `\begin{center}` title
/// above an abstract with no `\maketitle`. `maybe_promote_leading_title` makes it a
/// real `<title>` (here the `\\`-joined author line is part of that one paragraph, so
/// it is folded INTO the title), and the flush lands the `<abstract>` right after it
/// via `insert_frontmatter_after_node`.
#[test]
fn promoted_title_stays_above_the_top_flushed_abstract() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{center}\n\
               {\\Large\\bfseries My Hand Made Title}\\\\[1ex]\nSome Author\n\\end{center}\n\
               \\begin{abstract}\nThis is the abstract body text.\n\\end{abstract}\n\
               \\section{Introduction}\nBody of intro.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let title = xml
    .find("<title>")
    .expect("the promoted hand-made <title> must be present");
  let abstract_ = xml.find("<abstract").expect("<abstract> must be present");
  let section = xml.find("<section").expect("<section> must be present");
  let author = xml
    .find("Some Author")
    .expect("the hand-formatted author line must survive");
  assert!(
    title < author && author < abstract_ && abstract_ < section,
    "the promoted <title> AND the rest of its hand-formatted block (the author line) \
       must stay ABOVE the flushed <abstract> — visible content is never reordered:\n{xml}"
  );
  let title_end = xml.find("</title>").expect("<title> must be closed");
  assert!(
    xml[title..title_end].contains("My Hand Made Title"),
    "the promoted title text must sit INSIDE the <title> element:\n{xml}"
  );
}

/// Batch 56et (OXIDIZED_DESIGN #246): a hand-formatted cover whose title is
/// unambiguous (plain `center`, first paragraph, unique display font) is promoted to
/// `<title>`; its REMAINDER (the author paragraph) is title-page LAYOUT, wrapped in
/// `<titlepage>` in its exact visible order, and the queued `<abstract>` follows —
/// `title, titlepage, abstract, section`, all first-group, schema-valid. Same shape as
/// the fixture `structure/promote_center_title`. A regular document has no body
/// `logical-block(author)` construct (user ruling 2026-09-20).
#[test]
fn promoted_title_remainder_becomes_titlepage_layout_before_the_abstract() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{center}\n\
               {\\Large A Hand-Formatted Title}\n\nAn Author\n\\end{center}\n\
               \\begin{abstract}\nThe abstract text.\n\\end{abstract}\n\
               \\section{Introduction}\nBody text.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let title = xml
    .find("<title>")
    .expect("the promoted <title> must be present");
  let title_end = xml.find("</title>").unwrap();
  assert!(
    xml[title..title_end].contains("A Hand-Formatted Title"),
    "{xml}"
  );
  let titlepage = xml
    .find("<titlepage")
    .expect("<titlepage> must wrap the cover remainder");
  let titlepage_end = xml.find("</titlepage>").unwrap();
  let author = xml.find("An Author").expect("the author line must survive");
  let abstract_ = xml.find("<abstract").expect("<abstract> must be present");
  let section = xml.find("<section").expect("<section> must be present");
  assert!(
    title_end < titlepage
      && titlepage < author
      && author < titlepage_end
      && titlepage_end < abstract_
      && abstract_ < section,
    "expected title, titlepage(author line), abstract, section:\n{xml}"
  );
  assert!(
    !xml[..abstract_].contains("<logical-block")
      && !xml[..abstract_].contains("<para ")
      && !xml[..abstract_].contains("<para>"),
    "no body para/logical-block may remain above the abstract — the cover is layout:\n{xml}"
  );
}

/// Batch 56et control (OXIDIZED_DESIGN #246): an author-first cover where `\Large`
/// leaks onto the title paragraph too (tikz-mirror-lens
/// `\FHZCapaArticleCabecalho`: `\Large{#1}` then `{#2}`) has TWO display-font
/// paragraphs — ambiguous, so nothing is promoted (no `<title>FHZ</title>`); the
/// whole cover is `<titlepage>` layout in order, and the abstract follows it.
#[test]
fn ambiguous_display_font_cover_is_titlepage_layout_not_a_title() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{center}\n\
               \\Large{\\textbf{FHZ}}\n\n{Spherical mirrors and lenses}\n\\end{center}\n\
               \\begin{abstract}\nThe abstract text.\n\\end{abstract}\n\
               \\section{Introduction}\nBody text.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let titlepage = xml
    .find("<titlepage")
    .expect("<titlepage> must wrap the cover");
  let abstract_ = xml.find("<abstract").expect("<abstract> must be present");
  let fhz = xml.find("FHZ").unwrap();
  let real = xml.find("Spherical mirrors").unwrap();
  assert!(
    !xml[..abstract_].contains("<title>"),
    "an ambiguous cover must not be promoted to a document <title>:\n{xml}"
  );
  assert!(
    titlepage < fhz && fhz < real && real < abstract_,
    "the cover must be titlepage layout in its visible order, above the abstract:\n{xml}"
  );
}

/// Batch 56et control (OXIDIZED_DESIGN #246): a display-font badge inside a nested box
/// (isosigns' `VERSION 2.1` tcolorbox; here a `\fbox{\parbox}`) is not a plain-center
/// paragraph, so it is never read as a title — layout, wrapped in `<titlepage>`.
#[test]
fn version_badge_in_a_box_is_not_promoted_to_a_title() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{flushright}\n\
               \\fbox{\\parbox{4cm}{\\centering\\Large\\textbf{VERSION 2.1}}}\n\\end{flushright}\n\
               \\begin{abstract}\nThe abstract text.\n\\end{abstract}\n\
               \\section{Introduction}\nBody text.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let abstract_ = xml.find("<abstract").expect("<abstract> must be present");
  assert!(
    !xml[..abstract_].contains("<title>"),
    "a boxed version badge must not become the document <title>:\n{xml}"
  );
  let titlepage = xml
    .find("<titlepage")
    .expect("<titlepage> must wrap the badge");
  let badge = xml.find("VERSION 2.1").unwrap();
  assert!(titlepage < badge && badge < abstract_, "{xml}");
}

/// Batch 56et control (OXIDIZED_DESIGN #246): the cover is what precedes the
/// abstract's OWN position (the `\lx@frontmatter@mark` marker), never what precedes
/// the flush point. A sectionless manual flushes at `\end{document}` — its whole body
/// would otherwise be swallowed into the titlepage (and the still-open last paragraph
/// renamed, `malformed`). Body after the abstract stays body; the promoted title +
/// abstract lead; no `<titlepage>` is made for a cover that is only the title.
#[test]
fn sectionless_body_after_the_abstract_is_not_cover_layout() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{center}\n\
               {\\Large A Hand-Formatted Title}\n\\end{center}\n\
               \\begin{abstract}\nThe abstract text.\n\\end{abstract}\n\
               First body paragraph.\n\nSecond body paragraph.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let title = xml
    .find("<title>")
    .expect("the promoted <title> must be present");
  let abstract_ = xml.find("<abstract").expect("<abstract> must be present");
  let first = xml.find("First body paragraph.").unwrap();
  assert!(title < abstract_ && abstract_ < first, "{xml}");
  assert!(
    !xml.contains("<titlepage"),
    "no titlepage for a title-only cover:\n{xml}"
  );
  assert!(
    !xml.contains("_Frontmatter_Capture_"),
    "markers must never reach the output:\n{xml}"
  );
  let body_start = xml.find("First body").unwrap();
  assert!(
    xml[body_start..].contains("<p>") || xml[..body_start].contains("<para"),
    "{xml}"
  );
}

/// Batch 56et regression control (s107: 8 docs leaked `<_Frontmatter_Capture_>`): an
/// abstract queued AFTER `\maketitle` placed the frontmatter (lips: a `\DocInput`-ed
/// `\begin{abstract}`) is a LATE abstract. No position marker is built once the
/// frontmatter is placed, the abstract-only fallback neither promotes nor wraps (what
/// precedes it is body, not cover), `insert_late_frontmatter` files the abstract beside
/// the existing frontmatter, and the core finalize drops any stranded `*_Capture_`
/// scaffolding even when `\end{document}` is never reached.
#[test]
fn late_abstract_after_maketitle_leaves_no_marker_and_wraps_nothing() {
  let tex = "\\documentclass{article}\n\\title{T}\\author{A}\n\\begin{document}\n\\maketitle\n\
               First body paragraph.\n\nSecond body paragraph.\n\
               \\begin{abstract}\nA late abstract.\n\\end{abstract}\n\
               Third body paragraph.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains("_Capture_"),
    "no internal marker may reach the output:\n{xml}"
  );
  assert!(
    !xml.contains("<titlepage"),
    "body before a LATE abstract is not cover:\n{xml}"
  );
  let creator = xml.find("<creator").expect("<creator> must be present");
  let abstract_ = xml
    .find("<abstract")
    .expect("the late abstract must be placed");
  let first = xml.find("First body paragraph.").unwrap();
  let third = xml.find("Third body paragraph.").unwrap();
  assert!(
    creator < abstract_ && abstract_ < first && first < third,
    "the late abstract joins the existing frontmatter (after <creator>), body stays body:\n{xml}"
  );
}

/// Batch 56ey: pdftexcmds' `\pdf@strcmp` (and siblings) must be `\let` to the
/// primitive, not a macro that re-invokes it by name — annotate-equations.sty:16
/// aliases the other way round (`\let\pdfstrcmp\pdf@strcmp` under `\ifluatex`), and
/// a by-name delegation then made `\pdfstrcmp` expand to itself without end
/// (`Fatal:Timeout:TokenLimit`; latex-via-exemplos under the luatex identity, 300 s).
#[test]
fn pdfstrcmp_realiased_through_pdftexcmds_does_not_recurse() {
  let tex = "\\documentclass{article}\n\\usepackage{pdftexcmds}\n\\makeatletter\n\
               \\let\\pdfstrcmp\\pdf@strcmp\n\\makeatother\n\\begin{document}\n\
               \\ifnum\\pdfstrcmp{south}{south}=0 EQUAL\\else DIFFERENT\\fi\n\
               \\ifnum\\pdfstrcmp{north}{south}=0 SAME\\else OTHER\\fi\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("EQUAL") && xml.contains("OTHER"), "{xml}");
  assert!(!xml.contains("DIFFERENT") && !xml.contains("SAME"), "{xml}");
}

/// Batch 56ez (RUST-ONLY parity fix): a math `\halign` cell that escapes the
/// `${}##{}$` template via `\multispan` holds its `\upbracefill` glyph as a bare text
/// node; the tabular→XMArray conversion in `cleanup_xmtext` filtered cell children to
/// elements, so the glyph stayed raw #PCDATA inside `<XMCell>` (schema: `XMCell_model
/// = XMath.class*`). Perl wraps every non-Math child, text included, in `<XMText>`.
/// Witnesses oubraces/oubraces, halloweenmath/halloweenmath-man, nath/nathguide.
#[test]
fn multispan_glyph_in_a_math_halign_cell_is_wrapped_in_xmtext() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\
               $\\vbox{\\halign{&\\hfil${}##{}$\\hfil\\cr\n  x&y\\cr\n  \\multispan{2}\\upbracefill\\cr\n  a&b\\cr}}=\\pi r^2$\n\
               \\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let cell = xml
    .find("colspan=\"2\"")
    .expect("the \\multispan cell must be present");
  let cell_start = xml[..cell]
    .rfind("<XMCell")
    .expect("the spanning cell must be an XMCell");
  let cell_end = xml[cell..].find("</XMCell>").map(|k| cell + k).unwrap();
  let inner = &xml[cell_start..cell_end];
  assert!(
    inner.contains("<XMText>") && inner.contains('\u{23DF}'),
    "the spanning cell's glyph must sit inside <XMText>, not as bare text:\n{inner}"
  );
  let after_open = &inner[inner.find('>').unwrap() + 1..];
  assert!(
    after_open.trim_start().starts_with("<XMText"),
    "no raw #PCDATA may precede the XMText in the XMCell:\n{inner}"
  );
}

/// SVG DOM invariant for the pgfsys marker constructors (`\lxSVG@setlinewidth`,
/// `buttcap`, `miterjoin`, `stroke`, …): whatever their definition kind, a `\draw`
/// yields exactly one `<svg:path>` and the paired `\lxSVG@begingroup` carries the
/// marker's state. (A 2026-09-20 flip of the markers to primitives left peak RSS
/// unchanged — 772 vs 769 MB on a 20k-path picture — and was reverted; the boxes on
/// the picture's box list are not the markers.)
#[test]
fn pgfsys_markers_leave_the_svg_dom_unchanged() {
  let mut tex = String::from(
    "\\documentclass{article}\n\\usepackage{tikz}\n\\begin{document}\n\\begin{tikzpicture}\n",
  );
  for i in 0..200 {
    tex.push_str(&format!(
      "\\draw[line width=0.5pt,line cap=round,line join=round,dashed] ({i}pt,0) -- ({i}pt,10pt);\n"
    ));
  }
  tex.push_str("\\end{tikzpicture}\n\\end{document}\n");
  let (stderr, xml) = convert(&tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("<svg:path").count(),
    200,
    "one svg:path per \\draw:\n{}",
    &xml[..xml.len().min(2000)]
  );
  assert_eq!(
    xml.matches("<svg:svg").count(),
    1,
    "one svg root:\n{}",
    &xml[..xml.len().min(2000)]
  );
  assert!(
    xml.contains("stroke-dasharray") && xml.contains("stroke-linecap=\"round\""),
    "the paired begingroup attributes still carry the marker's state:\n{}",
    &xml[..xml.len().min(2000)]
  );
}

/// Batch 56fb (OXIDIZED_DESIGN #250): a `\section*` inside a block-mode `minipage`
/// is a real section — the counter advanced, text after the box belongs to it. Perl
/// wraps the box as `<sectional-block>` INSIDE the enclosing `<section>` (invalid:
/// only `document.body.class` admits it). The unit floats out to where a live
/// `\section` opens — a sibling of the enclosing section — carrying the box as
/// `class="ltx_minipage"`; the post-box text nests inside it (user ruling
/// 2026-09-20: the box is presentation, the sectioning is semantics).
#[test]
fn section_in_a_block_minipage_floats_out_as_a_sibling_section() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\section{Outer}\nBefore.\n\n\
               \\begin{minipage}[t]{5cm}\n\n\\section*{In the box}\nInner text.\n\\end{minipage}\n\n\
               After box.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains("<sectional-block"),
    "no sectional-block wrapper may remain:\n{xml}"
  );
  let outer = xml.find("<section ").expect("the outer section");
  let outer_end = xml[outer..].find("</section>").map(|k| outer + k).unwrap();
  let inner = xml
    .find("ltx_minipage")
    .expect("the floated section carries the box class");
  assert!(
    inner > outer_end,
    "the boxed section must be a SIBLING after the outer one, not nested:\n{xml}"
  );
  let after = xml.find("After box.").unwrap();
  assert!(
    after > inner,
    "text after the box nests in the new section:\n{xml}"
  );
  assert!(xml[..outer_end].contains("Before."), "{xml}");
}

/// #250, review follow-up: `{center}` routes its captured body through
/// `insert_block` and stamps `class="ltx_centering"` on the nodes it gets back
/// (`aligning_environment`, sect06.rs). A floated sectioning unit must be among
/// those nodes, or the alignment class is silently lost on the float-out path.
#[test]
fn section_in_a_center_environment_floats_out_and_keeps_its_alignment_class() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\section{Outer}\nBefore.\n\n\
               \\begin{center}\n\\section*{Centered heading}\nInner text.\n\\end{center}\n\n\
               After.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<sectional-block"), "{xml}");
  let outer = xml.find("<section ").expect("the outer section");
  let outer_end = xml[outer..].find("</section>").map(|k| outer + k).unwrap();
  let inner = xml[outer_end..]
    .find("<section ")
    .map(|k| outer_end + k)
    .expect("the centered section is a later sibling");
  let inner_tag_end = xml[inner..].find('>').map(|k| inner + k).unwrap();
  assert!(
    xml[inner..inner_tag_end].contains("ltx_centering"),
    "the floated section keeps the {{center}} alignment class:\n{}",
    &xml[inner..inner_tag_end]
  );
  assert!(xml.find("After.").unwrap() > inner, "{xml}");
}

/// 56fd — `\epTeXinputencoding` (pTeX-only) raises `PTEX_PROFILE`, so kanji
/// join control words and jlreq.cls:6446 `\def\西暦{\西暦true}` is a control
/// WORD macro, not the self-referential `\西`+`暦`-delimited one that spun to
/// `Fatal:Timeout:PushbackLimit` (SHARED with Perl, which hangs). Witnesses
/// asternote, hideanswer-doc, inlinelabel, jpnedumathsymbols-doc.
#[test]
fn eptex_input_encoding_letters_kanji_so_jlreq_year_style_terminates() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\epTeXinputencoding utf8\n\
               \\newif\\if西暦\n\\def\\西暦{\\西暦true}\n\\makeatother\n\
               \\begin{document}\n\\西暦\\if西暦 seireki\\else wareki\\fi\n\n\
               done\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(
    xml.contains("<p>seireki</p>"),
    "the \\if西暦 conditional was set by \\西暦 → \\西暦true:\n{xml}"
  );
  assert!(xml.contains("<p>done</p>"), "{xml}");
}

/// 56fc — Perl `hardYankProcessing` (Common/Error.pm L320-348): a resource
/// Fatal ends digestion. The bodies digested before it are rescued into the
/// document, the input after it is never read, so no content is produced
/// whose diagnostics the post-Fatal latch would then mute — the asternote
/// shape (PushbackLimit inside jlreq.cls, then luatexja and the body
/// converted with `\kanjiskip` tallied but unlogged). Every message the run
/// prints is accounted for: exactly one `Fatal:` line, and the undefined CS
/// after the fatal appears neither as a line nor in the summary tally.
#[test]
fn a_resource_fatal_rescues_the_digested_bodies_and_reads_no_further_input() {
  // A macro that grows its argument and leaves a copy of it behind on every call: the
  // pushback grows without a repeating window, so this trips a RESOURCE fatal
  // (`Timeout:PushbackLimit` at the preload's limit, in well under a second) — the `Err` arm of
  // `digest_step_guarded` that calls `hard_yank_processing`. (`Timeout:Recursion`
  // and the stomach box cap take the older `Ok(false)` stop and would not
  // exercise it: the jlreq shape `\def\西暦{\西暦true}` this test first used
  // now stops at the gullet's cycle guard, gullet.rs `Recursion`.)
  let tex = "\\documentclass{article}\n\\begin{document}\nBefore the fatal.\n\n\
               \\def\\a#1{\\a{#1x}#1}\\a{x}\n\n\
               After the fatal. \\undefinedafterthefatal\n\\end{document}\n";
  let (stderr, xml) = convert_with(
    tex,
    Some("[rawstyles,rawclasses,pushbacklimit=1000000]latexml.sty"),
  );
  assert_eq!(
    stderr.matches("Fatal:Timeout:").count(),
    1,
    "one resource Fatal line, printed once:\n{stderr}"
  );
  assert!(
    stderr.contains("Fatal:Timeout:PushbackLimit"),
    "a resource Fatal, not the cycle guard's Recursion:\n{stderr}"
  );
  // The Fatal is the run's only diagnostic (`error_count` counts `Fatal:` lines too).
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  // The partial body in progress is not salvaged for a resource Fatal
  // (`digest_step_guarded`: reviving it re-entered the loop during build on
  // arXiv:2605.25400); only COMPLETED bodies are rescued, and a `document`
  // environment is one body — so "Before" is lost here, as in the old
  // 39-byte stub. What the yank guarantees is that nothing PAST the Fatal
  // is read: before it, the salvage pass re-entered the live mouth and
  // converted "After" with its Error records muted.
  assert!(
    !xml.contains("After the fatal"),
    "nothing after the fatal is digested:\n{xml}"
  );
  assert!(
    !stderr.contains("undefinedafterthefatal"),
    "an unread CS is neither reported nor tallied:\n{stderr}"
  );
}

/// 56ff (M2) — the kernel `\title`/`\author` set only the `@`-forms, as
/// Perl does (latex_constructs.pool.ltxml:1060/1077); the bare
/// `\shorttitle`/`\shortauthor` are aliases of them. Before, `\title` also
/// `\gdef`'d `\shorttitle{#1}`, turning a class's own `\def\shorttitle#1`
/// (gaceta.cls:744) into a 0-arg macro, so the class's later
/// `\shorttitle{RUNNING HEAD}` typeset its argument as a `<para>` before the
/// frontmatter (gaceta/plantilla-articulo-suelto; RUST-ONLY, Perl clean).
#[test]
fn a_class_shorttitle_survives_the_kernel_title() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\\def\\shorttitle#1{\\gdef\\@run{#1}}\n\\makeatother\n\
               \\begin{document}\n\\title{Full Title}\n\\author{A. Author}\n\\shorttitle{RUNNING HEAD}\n\\maketitle\n\
               Body.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains("RUNNING HEAD"),
    "the class's \\shorttitle argument is not ink:\n{xml}"
  );
  assert!(xml.contains("<title>Full Title</title>"), "{xml}");
  let title = xml.find("<title>Full Title</title>").unwrap();
  let first_para = xml.find("<para ").unwrap();
  assert!(title < first_para, "frontmatter precedes the body:\n{xml}");
}

/// Control for the alias: arxiv.sty:64 `\hypersetup{pdfauthor={\shortauthor}}`
/// (witness 2406.14142) reads the bare `\shortauthor` — before `\author` fires
/// it must be defined (empty), and afterwards it is the stored `\@shortauthor`,
/// i.e. the OPTIONAL short form of `\author[short]{long}` (Perl PR #2767:
/// `\def\@shortauthor{#1}`; empty when no short form was given).
#[test]
fn bare_shortauthor_resolves_to_the_stored_short_author() {
  let tex = "\\documentclass{article}\n\\begin{document}\nEarly: [\\shortauthor]\n\n\
               \\title{T}\n\\author[A. Author]{Ann Author}\n\\maketitle\n\
               Short: \\shortauthor.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>Early: []</p>"),
    "defined and empty before \\author:\n{xml}"
  );
  assert!(xml.contains("<p>Short: A. Author.</p>"), "{xml}");
}

/// 56ff (M3) — the oup-authoring-template contrib stub matched the raw class's
/// arities: `\authormark##1` only sets `\leftmark` (cls:810), `\corresp` is
/// `\@@corresp[2][]` (cls:1145), `\received#1#2#3` is a date triple (cls:1085).
/// The old one-arg stubs typeset "Author Name et al." / "]Corresponding…" /
/// "0Year 0Year 0Year" as leading `<para>`s before the title (RUST-ONLY; Perl
/// loads the raw class and emits the title first).
#[test]
fn oup_stub_arities_match_the_raw_class_so_nothing_leaks_before_the_title() {
  let tex = "\\documentclass{oup-authoring-template}\n\
               \\title{A Title}\n\\author{Author Name}\n\\authormark{Author Name et al.}\n\
               \\corresp[$\\ast$]{Corresponding author. mail@example.org}\n\
               \\received{20}{9}{2026}\n\\revised{21}{9}{2026}\n\\accepted{22}{9}{2026}\n\
               \\begin{document}\n\\maketitle\nBody.\n\\end{document}\n";
  let (_stderr, xml) = convert(tex, true);
  for leak in ["Author Name et al.", "]Corresponding", "0Year", "<p>20"] {
    assert!(!xml.contains(leak), "leaked `{leak}` into the body:\n{xml}");
  }
  assert!(xml.contains("<title>A Title</title>"), "{xml}");
  assert!(
    xml.contains("<note role=\"received\">20 9 2026</note>"),
    "the date triple is a frontmatter note:\n{xml}"
  );
  let title = xml.find("<title>A Title</title>").unwrap();
  let body = xml.find("Body.").unwrap();
  assert!(title < body, "{xml}");
}

/// 56fg — `document.body` admits `ltx:subparagraph` (user ruling 2026-09-22;
/// every other sectional container already admitted it, `document.body` alone
/// listed `paragraph` but not `subparagraph` — a model asymmetry shared with
/// Perl's schema). smflatex's `\tableofcontents` heading is an `\@startsection`
/// at level `\@M`, clamped to the deepest unit, so it lands at document level
/// (smflatex/smf-edoc, smf-fdoc). LaTeX permits a top-level `\subparagraph`.
#[test]
fn a_top_level_subparagraph_is_admitted_by_the_document_body() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\subparagraph{Contents}\nList.\n\n\\section{One}\nText.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(
    error_count(&stderr),
    0,
    "no malformed-content error:\n{stderr}"
  );
  let sp = xml
    .find("<subparagraph ")
    .expect("the subparagraph is emitted");
  let sec = xml.find("<section ").expect("the section follows");
  assert!(sp < sec, "{xml}");
  assert!(xml.contains("<title>Contents</title>"), "{xml}");
}

/// 56fh — `\left`/`\right` reached in text mode are plain characters, as in
/// Perl (`TeXDelimiter` digests the delimiter in the current mode; the XMTok
/// decoration applies only to an element). A beamer frame strips `$$`, so
/// `$$\left[ x \right]$$` is text there (hitszbeamer/main), and egpeirce's
/// `\marginnote{…\left\lfloor…\right\rfloor…}` likewise; the constructor used
/// to emit `<ltx:XMTok>` under `<p>` (RUST-ONLY, schema-invalid).
#[test]
fn left_right_in_text_mode_are_plain_text() {
  let tex = "\\documentclass{article}\n\\begin{document}\nText \\left[ x \\right] done.\n\n\
               Math $\\left[ x \\right]$ too.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let p_end = xml.find("done.</p>").expect("the text paragraph");
  assert!(
    !xml[..p_end].contains("<XMTok"),
    "no XMTok before/inside the text paragraph:\n{xml}"
  );
  assert!(xml.contains("<p>Text [ x ] done.</p>"), "{xml}");
  // The math-mode path is untouched: the fences are XMToks with roles.
  assert!(xml.contains("role=\"OPEN\""), "{xml}");
  assert!(xml.contains("role=\"CLOSE\""), "{xml}");
}

/// 56fh — ascmac's boxed environments are block boxes through `insert_block`
/// (as framed/mdframed): in a `<figure>` the box is a `<block>`, not a bare
/// `<para>` (schema-invalid; chemobabel-en/-ja, RUST-ONLY). The itembox title
/// rides along as the block's first `ltx:note`.
#[test]
fn ascmac_screen_inside_a_figure_is_a_block() {
  let tex = "\\documentclass{article}\n\\usepackage{ascmac}\n\\begin{document}\n\
               \\begin{figure}[ht]\n\\centering\nBelow is a short description:\\par\n\
               \\begin{screen}\nThis package provides a way to convert graphics.\n\\end{screen}\n\
               \\caption{A figure}\n\\end{figure}\n\n\
               \\begin{itembox}[l]{Note title}\nBoxed note body.\n\\end{itembox}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let fig = xml.find("<figure ").expect("figure");
  let fig_end = xml[fig..].find("</figure>").map(|k| fig + k).unwrap();
  let figure = &xml[fig..fig_end];
  assert!(
    !figure.contains("<para "),
    "no bare para in the figure:\n{figure}"
  );
  // `insert_block` picks the figure's own panel shape (Perl's raw ascmac run
  // yields the same `ltx_figure_panel` p), carrying the box class + frame.
  assert!(
    figure.contains(
      "<p align=\"center\" class=\"ltx_ascmac_screen ltx_figure_panel\" framed=\"rectangle\">\
         This package provides a way to convert graphics.</p>"
    ),
    "the screen is a framed figure panel:\n{figure}"
  );
  // The title is the FIRST child of the box content, before the body text
  // (`insert_block` returns the content `<p>`; the note is prepended there).
  assert!(
    xml.contains("<p><note role=\"itembox-title\">Note title</note>Boxed note body.</p>"),
    "the itembox title precedes its body inside the box:\n{xml}"
  );
}

/// 56fi — `\openin` is a probe (tex.web §1275): a file that cannot be
/// opened leaves `\ifeof` true, with NO diagnostic; a file `\openout`
/// registered but has not written yet is a real, empty file (`\ifeof` false
/// until read). Sweep s108 showed `Fatal:Mouth:MissingFile` on exactly this
/// probe (minitoc `.mtc0`, frenchle `.aux_LE`, chapterbib `.cb`: cahierprof-doc,
/// fepslatex, pst-eucl-docBG, ArsClassica-de) once the mouth's open-failure
/// `fatal!` logged at the raise (56fc); Perl reports nothing there.
#[test]
fn openin_of_a_missing_or_unwritten_file_is_silent() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\
               \\newread\\probe\n\
               \\openin\\probe=definitely-not-here.mtc0\\relax\n\
               \\ifeof\\probe Missing.\\else Present.\\fi\\closein\\probe\n\n\
               \\newwrite\\out\\immediate\\openout\\out=\\jobname.mtc0\\relax\n\
               \\openin\\probe=\\jobname.mtc0\\relax\n\
               \\ifeof\\probe Unopened.\\else Opened.\\fi\\closein\\probe\\immediate\\closeout\\out\n\
               \\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Fatal:"),
    "a probe is never fatal:\n{stderr}"
  );
  assert!(xml.contains("<p>Missing.</p>"), "{xml}");
  assert!(
    xml.contains("<p>Opened.</p>"),
    "an \\openout-registered file exists (empty):\n{xml}"
  );
}

/// 56fj — listings `gobble` drops CHARACTERS: a multibyte first character
/// (`本`, texproposal) panicked `[1..]` on a non-char boundary — the one
/// caught worker panic of sweep s108.
#[test]
fn listings_gobble_drops_a_character_not_a_byte() {
  let tex = "\\documentclass{article}\n\\usepackage{listings}\n\\begin{document}\n\
               \\begin{lstlisting}[gobble=1]\nxfirst line\n次行 two\n\\end{lstlisting}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  // The binding applies `gobble` from the second line on (the first line's
  // gobble is a separate, pre-existing gap); the multibyte line is the second.
  assert!(
    xml.contains(
      "<listingline xml:id=\"lstnumberx2\">行<text class=\"ltx_lst_space\"> </text>\
         <text class=\"ltx_lst_identifier\">two</text></listingline>"
    ),
    "the gobbled line keeps its second character and drops the first:\n{xml}"
  );
}

/// 56fl (#253) — els-cas shape: the class's `\RenewDocumentCommand\author{O{} m O{}}`
/// is refused by the lock, so its trailing `[keyval]` used to typeset as a
/// `<para>` before the title (SHARED with Perl). Now `orcid=` becomes a
/// contact and the presentational keys are dropped.
#[test]
fn raw_class_author_trailing_keyval_becomes_frontmatter() {
  let tex = "\\documentclass{article}\n\\usepackage{xparse}\n\\makeatletter\n\
               \\RenewDocumentCommand\\author{O{} m O{}}{\\def\\@author{#2}}\n\\makeatother\n\
               \\begin{document}\n\\title{T}\n\\author[1]{Jane Doe}[type=editor, orcid=0000-0001]\n\\maketitle\n\
               Body.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !xml.contains("type=editor"),
    "the keyval surplus is not ink:\n{xml}"
  );
  let title = xml.find("<title>T</title>").expect("title");
  let body = xml.find("Body.").unwrap();
  assert!(title < body, "frontmatter precedes the body:\n{xml}");
  assert!(xml.contains("<personname>Jane Doe</personname>"), "{xml}");
  assert!(
    xml.contains("<contact role=\"orcid\">") && xml.contains("0000-0001"),
    "the orcid is a contact:\n{xml}"
  );
}

/// 56fl (#253) — cnbwp shape: a class `\def\author` taking `{name}{affiliation}`
/// once per author. The kernel shape orphaned every `{affiliation}` and
/// dequeue-replaced the creators down to the last one (SHARED). Now each
/// call appends a creator carrying its affiliation.
#[test]
fn raw_class_author_per_author_calls_accumulate_with_affiliations() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\
               \\def\\author{\\@ifnextchar[{\\CNBs}{\\CNBl}}\n\\def\\CNBs[#1]#2#3{}\n\\def\\CNBl#1#2{\\def\\@author{#1}}\n\
               \\makeatother\n\\begin{document}\n\\title{T}\n\
               \\author{Ann One}{Alpha Institute}\n\\author{Bob Two}{Beta Lab}\n\\author{Cid Three}{Gamma Center}\n\
               \\maketitle\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("role=\"author\">").count(),
    3,
    "three creators:\n{xml}"
  );
  for (name, affil) in [
    ("Ann One", "Alpha Institute"),
    ("Bob Two", "Beta Lab"),
    ("Cid Three", "Gamma Center"),
  ] {
    assert!(
      xml.contains(&format!(
        "<personname>{name}</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{affil}</contact>"
      )),
      "the affiliation is a contact of its own creator:\n{xml}"
    );
    assert!(
      !xml.contains(&format!("<p>{affil}</p>")),
      "no orphaned affiliation paragraph:\n{xml}"
    );
  }
  let title = xml.find("<title>T</title>").unwrap();
  assert!(title < xml.find("Body.").unwrap(), "{xml}");
}

/// Control — a class redefinition of the SAME shape, one `\author`, and a
/// brace group in the next paragraph: one creator, the paragraph stays body
/// text (`\par` stops `\@ifnextchar`), nothing absorbed.
#[test]
fn same_shape_author_redefinition_absorbs_nothing() {
  let tex = "\\documentclass{article}\n\\makeatletter\n\
               \\renewcommand\\author[1]{\\gdef\\@author{#1}}\n\\makeatother\n\\begin{document}\n\
               \\title{T}\n\\author{Only One}\n\n{\\bfseries Braced} text.\n\n\\maketitle\nBody.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("role=\"author\">").count(), 1, "{xml}");
  assert!(
    !xml.contains("role=\"affiliation\""),
    "nothing absorbed:\n{xml}"
  );
  assert!(
    xml.contains("Braced</text> text."),
    "the braced paragraph is body text:\n{xml}"
  );
}

/// 56fh — a `\left.` / `\right.` reached in text mode emits nothing (its
/// math-only `<ltx:XMHint/>` would be as invalid under `<p>` as the XMTok).
#[test]
fn left_dot_in_text_mode_emits_no_math_hint() {
  let tex =
    "\\documentclass{article}\n\\begin{document}\nText \\left. x \\right. done.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<XMHint"), "{xml}");
  // The hint's own tokens leave nothing behind (the two spaces flank where
  // the `\left.`/`\right.` stood).
  assert!(xml.contains("<p>Text  x  done.</p>"), "{xml}");
}

/// Batch 56fb: a `\subsection` in the box NESTS in the enclosing section (a live
/// `\subsection` would), and the post-box text ends up inside that subsection.
#[test]
fn subsection_in_a_block_minipage_nests_in_the_enclosing_section() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\section{Outer}\nBefore.\n\n\
               \\begin{minipage}[t]{5cm}\n\n\\subsection{Sub}\nInner.\n\\end{minipage}\n\n\
               After box.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<sectional-block"), "{xml}");
  let outer_end = xml.find("</section>").unwrap();
  let sub = xml.find("<subsection").expect("the subsection");
  let sub_end = xml[sub..].find("</subsection>").map(|k| sub + k).unwrap();
  let after = xml.find("After box.").unwrap();
  assert!(
    sub < outer_end && sub_end < outer_end,
    "the subsection nests inside the outer section:\n{xml}"
  );
  assert!(
    sub < after && after < sub_end,
    "post-box text nests in the subsection:\n{xml}"
  );
  assert!(
    xml.contains("<tag>1.1</tag>"),
    "numbering is the digest-time counter (1.1):\n{xml}"
  );
}

/// Batch 56fb control: a box inside a `quote` has no auto-close path to a section
/// holder (`quote` does not auto-close), so the SHARED Perl shape is kept unchanged.
#[test]
fn section_in_a_minipage_inside_a_quote_keeps_perl_parity() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\section{Outer}\n\\begin{quote}\n\
               \\begin{minipage}[t]{5cm}\n\n\\section*{Boxed}\nInner.\n\\end{minipage}\n\\end{quote}\n\
               After.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(stderr.matches("Fatal:").count(), 0, "{stderr}");
  let quote = xml.find("<quote").expect("the quote");
  let quote_end = xml[quote..].find("</quote>").map(|k| quote + k).unwrap();
  let boxed = xml.find("Boxed").unwrap();
  assert!(
    quote < boxed && boxed < quote_end,
    "the boxed section stays inside the quote (Perl parity):\n{xml}"
  );
}

/// Batch 56eq (OXIDIZED_DESIGN #243): a `\put`-positioned `\parbox`/`\makebox` inside
/// a `picture` digests into the positioned `<g>` group. The box is an LR-box, but
/// `insert_block` emitted a schema-invalid `<block>` there (`g_model` is inline-only —
/// it holds `<inline-block>`, not `<block>`; LaTeXML-picture.rng). Fix: a container
/// that holds `inline-block` but neither `block` nor `#PCDATA` is treated as inline,
/// so the capture is renamed to `<inline-block>` IN PLACE (the `\put` position in the
/// enclosing `<g transform>` is preserved — fidelity-safe). SHARED with Perl
/// (`TeX_Box.pool.ltxml:493`), surpassed like #240; a sibling of 56ek's block-climb,
/// NOT the same move (climbing would destroy `\put` layout). Witnesses:
/// ticket/ex_flashcard (88→0 jing), elzcards/elzcards-examples (41→0) — 36 s105 docs.
#[test]
fn put_parbox_in_picture_is_inline_block_not_block() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\
               \\setlength{\\unitlength}{1mm}\n\\begin{picture}(60,40)\n\
               \\put(3,30){\\parbox{58mm}{\\textbf{word:} a definition here}}\n\
               \\end{picture}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let compact: String = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  // The \put'd \parbox in the picture <g> must be a schema-valid <inline-block>,
  // not the invalid <block> (g_model is inline-only).
  assert!(
    compact.contains("<inline-block") && !compact.contains("<block"),
    "a \\put'd \\parbox inside a picture must emit <inline-block> (valid in g_model), \
       not the schema-invalid <block>:\n{xml}"
  );
  // Content preserved (recall): the parbox body survives the rename.
  assert!(
    compact.contains("definition"),
    "the \\parbox body text must be preserved:\n{xml}"
  );
}

/// Batch 56er (OXIDIZED_DESIGN #244): a box-forming construct (`\begin{center}`/
/// minipage/`\parbox`) whose body auto-opens a `<para>` (a genuine block — here a
/// centered `tabular` — sits mid-content), placed in a Block.model container
/// (titlepage/quote/figure/abstract/inline-block), was renamed in place by
/// `insert_block` to a schema-invalid `<logical-block>` (Block.model has no Para.class;
/// LaTeXML-structure.rnc:585 etc.). Fix: when the context holds `<block>` but not the
/// Para.class element, emit `<block>` and recursively demote — rename descendant
/// logical-block/sectional-block → block (KEEPING the minipage `width`), unwrap
/// descendant `<para>`. In place, no reorder (ltx_para/block/logical-block are all
/// `display:block`). SHARED with Perl (TeX_Box.pool.ltxml:512), surpass like #240/#243.
/// Witnesses: webquiz, tabularcalc, short-math-guide, heria (~12 s105 docs). Since 57f a
/// minipage there starts the titlepage's paragraph (latex.ltx `\@iiiminipage` `\leavevmode`) and
/// is an `<inline-logical-block>` in it; the demotion is exercised by the `{center}` body.
#[test]
fn para_class_box_in_titlepage_demotes_to_block_not_logical_block() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{titlepage}\n\
               \\begin{center}\n\
               \\noindent\\textbf{Abstract}\\par\nGiven a list of numbers:\n\
               \\begin{center}\\begin{tabular}{|c|c|}\\hline $x$ & 1 \\\\\\hline\\end{tabular}\\end{center}\n\
               Other effects are possible.\n\\end{center}\n\\end{titlepage}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // No Para.class element may survive under the <titlepage> (Block.model): the
  // center capture is demoted to <block>, not left as <logical-block>.
  assert!(
    !xml.contains("<logical-block"),
    "Para.class box must demote to <block>, not emit a schema-invalid <logical-block>:\n{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "titlepage",
    &[],
    r#"<titlepage><block><p><text font="bold">Abstract</text></p><p align="center">Given a list of numbers:</p><tabular class="ltx_centering" vattach="middle"><tbody><tr><td align="center" border="b l r t"><Math mode="inline" tex="x" text="x" xml:id="m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td><td align="center" border="b r t">1</td></tr></tbody></tabular><p align="center">Other effects are possible.</p></block></titlepage>"#,
  );
  // Content preserved (recall): the abstract text and the tabular survive the demotion.
  assert!(
    xml.contains("Abstract") && xml.contains("<tabular") && xml.contains("Other effects"),
    "the box body (text + tabular) must be preserved through the demotion:\n{xml}"
  );
}

/// The 56er demotion keeps a minipage's `width` and class (OXIDIZED_DESIGN #244): in a figure a
/// minipage opens no paragraph (its content is panels), so a minipage whose body auto-opens a
/// `<para>` is demoted to `<block class="ltx_minipage" width=…>`.
#[test]
fn para_class_minipage_in_figure_demotes_keeping_its_width() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{figure}\n\
               \\begin{minipage}{0.85\\linewidth}\n\
               \\noindent\\textbf{Abstract}\\par\nGiven a list of numbers:\n\
               \\begin{center}\\begin{tabular}{|c|c|}\\hline $x$ & 1 \\\\\\hline\\end{tabular}\\end{center}\n\
               Other effects are possible.\n\\end{minipage}\n\\caption{F}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><block class="ltx_minipage" vattach="middle" width="293.3pt"><p><text font="bold">Abstract</text></p><p>Given a list of numbers:</p><tabular class="ltx_centering" vattach="middle"><tbody><tr><td align="center" border="b l r t"><Math mode="inline" tex="x" text="x" xml:id="S0.F1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td><td align="center" border="b r t">1</td></tr></tbody></tabular><p>Other effects are possible.</p></block><toccaption><tag close=" ">1</tag>F</toccaption><caption><tag close=": ">Figure 1</tag>F</caption></figure>"#,
  );
}

/// A macro call that does not match its `\def` (the parameter text's leading
/// delimiter absent: `\def\lp\x{…}` met with `\lp\y`) is reported and IGNORED,
/// as TeX does (tex.web §397-398 "Use of \lp doesn't match its definition").
/// The miss raised nothing and the macro expanded anyway, so a self-calling one
/// looped to the digestion fuse (frankenstein/titles via compsci's `\cs\def`).
/// A matching call still expands. Every expl3 expandable error goes through
/// this path (`\???`, expl3-code.tex:11596-11606).
#[test]
fn macro_delimiter_mismatch_ignores_the_call() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/parameter-conditional/macro_delimiter_mismatch.tex"
  );
  let (stderr, xml) = convert(tex, true);
  let mismatches = stderr
    .lines()
    .filter(|l| l.starts_with("Error:expected:Match"))
    .count();
  assert_eq!(mismatches, 1, "{stderr}");
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(
    xml.contains(r#"<p>A<ERROR class="undefined">\y</ERROR>B</p>"#),
    "{xml}"
  );
  assert!(xml.contains("<p>Y</p>"), "{xml}");
}

/// Convert raw bytes (a non-UTF-8 source) or a document with side files to the
/// given destination (`.xml` core, `.html` through the post stage), in a
/// tempdir with the raw preload. Returns (stderr, output).
fn convert_bytes_to(tex: &[u8], files: &[(&str, &[u8])], dest: &str) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
  let output = std::process::Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      dest,
      "--nocomments",
      "--timeout=110",
      "--preload=[rawstyles,rawclasses]latexml.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let out = std::fs::read_to_string(workdir.path().join(dest)).unwrap_or_default();
  (stderr, out)
}

/// `\afterassignment` before `\setbox<n>` whose operand is `\box`/`\copy` (no
/// body to take the token) fires right after the assignment (tex.web §1211
/// `done:`, §1269); it stayed pending and fired inside some later box.
/// luatexja's `\raise`/`\lower` via `\ltj@afterbox` (luatexja-core.sty:684-702)
/// emptied every pgf picture (suanpan-l3, qworld, codebox-doc-en). Perl
/// (TeX_Box.pool.ltxml:599-617) has the same gap. pdflatex prints both boxes.
#[test]
fn afterassignment_setbox_box_operand() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/afterassignment_setbox_box_operand.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p><text yoffset="2.0pt">Echo words</text><text yoffset="2.0pt">Foxtrot words</text></p>"#,
  );
}

/// A `{verbatim}` body is decoded through the document's 8-bit input encoding,
/// as inputenc's active characters keep their meaning in `\@verbatim`
/// (latex.ltx:15441-15459). A cp1251 listing came out as Latin-1 mojibake
/// (russ_doc, serbian-apostrophe; Perl drops the body). Inline `\verb` already
/// decoded.
#[test]
fn verbatim_decodes_the_input_encoding() {
  let tex =
    include_bytes!("../../../tools/perfect_kernel/repros/unicode-catcodes/verbatim_cp1251.tex");
  let (stderr, xml) = convert_bytes_to(tex, &[], "t.xml");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<verbatim font=\"typewriter\">\nКонец листинга\n</verbatim>"),
    "{xml}"
  );
  assert!(
    xml.contains("<verbatim font=\"typewriter\">Мир слово</verbatim>"),
    "{xml}"
  );
  // verbatim.sty, with the first body line pushed back by a wrapper's
  // optional-argument check (`read_raw_line_decoded`'s pushback branch).
  let tex = include_bytes!(
    "../../../tools/perfect_kernel/repros/unicode-catcodes/verbatim_cp1251_package.tex"
  );
  let (stderr, xml) = convert_bytes_to(tex, &[], "t.xml");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<verbatim font=\"typewriter\">Конец листинга один\nВторая строка\n</verbatim>"),
    "{xml}"
  );
}

/// `\psframebox[<params>]{<body>}` frames its BODY (Perl pstricks_support.sty.ltxml
/// :955-980 `DefPSConstructor`); the `#2` expansion printed the params and dropped
/// the body (ffslides.cls `\btext`, pst-poker-doc). In running text it is inline
/// framed text (DIVERGENCES #297) and the paragraph continues; inside a
/// `{pspicture}` it is a framed group.
#[test]
fn psframebox_keeps_its_body() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/psframebox_body.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p>Before <text framed="rectangle">Bravo framed words</text> after.</p>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "g",
    &["framed=\"true\""],
    r#"<g framed="true"><text>Golf boxed words</text></g>"#,
  );
  assert!(
    xml.contains(r#"<p><text framed="rectangle">Foxtrot words</text></p>"#),
    "{xml}"
  );
  assert!(
    !xml.contains("linecolor=red") && !xml.contains("fillframe"),
    "{xml}"
  );
}

/// `\setbox0 = \hbox{…}`: `scan_box` skips blanks and `\relax` before its box
/// (tex.web §1084, §404). The space after `=` was the operand, and the box was
/// typeset in place (KPE #254; 19 TL files spell it so). pdflatex prints `[xx]`.
#[test]
fn setbox_skips_blanks_before_the_box() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/setbox_blank_before_box.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[xx]</p>");
}

/// biber reads `%` to the end of the line as a comment between a `.bib`
/// entry's tokens (`@Book{a2004,% see also …`); the reader lost the entry
/// (windycity 27 entries, biblatex-iso690; Perl and bibtex 0.99d lose it too).
#[test]
fn biber_bib_percent_comments_keep_the_entry() {
  let tex =
    include_bytes!("../../../tools/perfect_kernel/repros/index-bib/biber_percent_comments.tex");
  let bib =
    include_bytes!("../../../tools/perfect_kernel/repros/index-bib/biber_percent_comments.bib");
  let (stderr, html) = convert_bytes_to(tex, &[("biber_percent_comments.bib", bib)], "t.html");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem").count(), 1, "{html}");
  assert!(
    html.contains(r#"<span class="ltx_text ltx_bib_title">Beloved Things</span>"#),
    "{html}"
  );
  assert!(
    html.contains(r#"<span class="ltx_text ltx_bib_year"> (2004)</span>"#),
    "{html}"
  );
  // The split post session (`--whatsin=xml`, post_sweep.sh's default) reads the
  // `.bib` with biblatex loaded from the preloads, not from `\printbibliography`.
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  std::fs::write(workdir.path().join("biber_percent_comments.bib"), bib).expect("write bib");
  let raw = "--preload=[rawstyles,rawclasses]latexml.sty";
  let core = std::process::Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      raw,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn core");
  assert!(
    core.status.success(),
    "{}",
    String::from_utf8_lossy(&core.stderr)
  );
  let post = std::process::Command::new(bin)
    .args([
      "--whatsin=xml",
      "t.xml",
      "--dest",
      "p.html",
      "--timeout=110",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn post");
  let stderr = String::from_utf8_lossy(&post.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("p.html")).unwrap_or_default();
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    html.contains(r#"<span class="ltx_text ltx_bib_title">Beloved Things</span>"#),
    "{html}"
  );
}

/// An empty fence parses only when its close balances its open (Perl
/// MathGrammar:463-464 `balancedClose`; MathParser.pm:1348-1384 `%balanced`,
/// `isMatchingClose`). `\mathopen{}\mathclose{\left(x\right)}` lexes as an empty
/// OPEN and a CLOSE that carries the `x`: the generic `open close` alternative
/// built `list@()` and dropped the `x` from the content tree, silently (arXiv
/// 2605.13448: 98 formulas; 2605.22010: 160). Perl leaves it unparsed with its
/// content kept, and warns once; so do we. A balanced empty fence still parses.
#[test]
fn empty_fence_needs_a_balanced_close() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/math-parse/empty_fence_balanced_close.tex");
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:unparsed_math:").count(),
    1,
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMTok meaning="absent" role="OPEN"/><XMDual role="CLOSE"><XMRef idref="p1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath>"#,
  );
  assert!(!xml.contains("list@()"), "{xml}");
  let (stderr, xml) = convert(
    "\\documentclass{article}\\begin{document}$\\lfloor\\rfloor$\\end{document}\n",
    false,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"text="list@()""#), "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMDual><XMApp><XMTok meaning="list"/></XMApp><XMWrap><XMTok name="lfloor" role="OPEN" stretchy="false">⌊</XMTok><XMTok name="rfloor" role="CLOSE" stretchy="false">⌋</XMTok></XMWrap></XMDual></XMath>"#,
  );
}

/// `\rule` sizes its box from Dimensions, as Perl stores them
/// (latex_constructs.pool.ltxml:4797-4799); the attribute strings measured 0×0,
/// so bfhsciposter.cls:171's `\box_gresize_to_ht_plus_dp` divided by zero (11 l3
/// `\???` errors, bfh-ci DEMO-BFHSciPoster). The raise follows latex.ltx
/// :16360-16368 `\@rule`, floored at 0 by hpack; pdflatex prints the same seven
/// sizes.
#[test]
fn rule_box_has_its_size() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/boxes-groups/rule_box_size.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p>[28.45274pt][1.0pt][0.0pt][26.45274pt][2.0pt][0.0pt][56.9055pt] <rule height="3.0pt" width="2.0pt"/></p>"#,
  );
}

/// A braced file name ends at its matching `}` (TeX Live's braced names); the
/// scan read on past it, expanding the next macro before the file was read, so
/// `\@@input{tfnini.tex}\tfnloaded` met an undefined `\tfnloaded`
/// (texnegar-luatex.sty:17 `\tex_input:D { texnegar-ini.tex }`; Perl alike).
#[test]
fn braced_file_name_stops_at_its_brace() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/loader/braced_file_name.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], "<p>LOADED X Y</p>");
}

/// `\advance`, `\multiply` and `\divide` are assignments: the `\afterassignment`
/// token goes in right after each (tex.web §1211 `done:`, §1269). It stayed
/// pending and fired at a later assignment (pstricks' raw `\psaddtolength`,
/// lsc; Perl never fires it, KNOWN_PERL_ERRORS #257). pdflatex prints the same.
#[test]
fn afterassignment_fires_after_arithmetic() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/afterassignment_arithmetic.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[F1.0pt][F2.0pt][F1.0pt][F3]</p>");
}

/// A braced `\input{…}` keeps its braces through the file-name scan, so `\input`
/// still strips them and loads LaTeX.pool (TeX_FileIO.pool.ltxml:164-169): a
/// LaTeX fragment input by a main file with no `\documentclass` keeps its
/// `\section`. Stopping the scan at the `}` must not drop that.
#[test]
fn braced_input_of_a_fragment_loads_latex() {
  let (stderr, xml) = convert_files_with(
    "\\input{tfnfrag}\n\\bye\n",
    &[("tfnfrag.tex", "\\section{Intro}Fragment text.\n")],
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[],
    r#"<section inlist="toc" xml:id="section1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>Intro</title><para xml:id="section1.p1"><p>Fragment text.</p></para></section>"#,
  );
}

/// A scanner puts back a `\noexpand`'d token it read but does not use as its
/// plain self: `\noexpand` suppresses expansion only for the read that met it
/// (tex.web:7509-7514, :8755-8757). The `\special_relax` marker survived the
/// optional-space check of `\romannumeral-`\q`, and `\csname` met it: trimspaces'
/// `\trim@spaces` on an argument starting with a macro (yquant-doc's 501-error
/// runaway; Perl alike). pdflatex prints `[macro:->FOO][Y]`.
#[test]
fn noexpand_marker_does_not_survive_a_scan() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/noexpand_marker_scan_putback.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[macro:-¿FOO][Y]</p>");
}

/// Under the XeTeX persona `\strcmp` is XeTeX's name for `\pdfstrcmp`
/// (expl3-code.tex:141-143); documents call it by that name
/// (cdcmd-test.tex:42-55, input by cdcmd-cn.tex:144).
#[test]
fn xetex_persona_has_strcmp() {
  let tex = "\\documentclass{article}\\begin{document}\
             [\\ifnum\\strcmp{ab}{ab}=0 same\\else different\\fi]\
             [\\number\\strcmp{a}{b}]\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,xetex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], "<p>[same][-1]</p>");
}

/// A tblr `*{<count>}{…}` whose count is not a literal is an integer
/// expression, as tabularray evaluates it (tabularray.sty:3361
/// `\prg_replicate:nn`): PixelArtTikz.sty:865's `*{\ListeCoulCaseslen}{Q[m,c]}`.
/// Read only as a literal, the whole spec fell back to the kernel template,
/// where "colspec"'s `p` took `e` as a width (calc: "`e' invalid",
/// PixelArtTikz-doc-fr, sweep #126). The binding pads a spec with 16 columns
/// continuing its last one, so only the `l` after `*{\n+1}{Q[r]}` (r r r l) and
/// `*{1+1}{Q[r]}` (r r l) shows the count evaluated right.
#[test]
fn tblr_colspec_count_is_an_integer_expression() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_colspec_count_expression.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p1.1\""],
    r#"<tabular vattach="middle" xml:id="p1.1"><tbody><tr xml:id="p1.1.1"><td align="center" xml:id="p1.1.1.1">a</td><td align="center" xml:id="p1.1.1.2">b</td></tr></tbody></tabular>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p2.1\""],
    r#"<tabular vattach="middle" xml:id="p2.1"><tbody><tr xml:id="p2.1.1"><td align="right" xml:id="p2.1.1.1">a</td><td align="right" xml:id="p2.1.1.2">b</td><td align="right" xml:id="p2.1.1.3">c</td><td align="left" xml:id="p2.1.1.4">d</td></tr></tbody></tabular>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "tabular",
    &["xml:id=\"p3.1\""],
    r#"<tabular vattach="middle" xml:id="p3.1"><tbody><tr xml:id="p3.1.1"><td align="right" xml:id="p3.1.1.1">a</td><td align="right" xml:id="p3.1.1.2">b</td><td align="left" xml:id="p3.1.1.3">c</td></tr></tbody></tabular>"#,
  );
}

/// A pgf path is kept as its segments and joined when it is drawn; Perl
/// concatenates one string (pgfsys-latexml.def.ltxml:225-232). Stored as a new
/// interned string at every step, each prefix of the path stayed in the arena,
/// quadratic in the path's length: 9000 segments took 4.1 GB and gave no output.
/// This is the memory half of tkz-grapheur-doc-en/-fr's 11,000-point plot; its
/// sweep-#126 Fatal is the box-cycle guard (batch 56kh). The path is unchanged,
/// drawn once, and the prefix through the 149th curveto was never interned (the
/// arena is checked to be live, so the absence is not vacuous).
#[test]
fn pgf_path_keeps_no_prefix_of_itself() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/pgf_long_path_linear_memory.tex"
  );
  let (stderr, xml, prefix_interned) = convert_with_then(tex, Some("ar5iv.sty"), |xml| {
    let d = xml
      .split(" d=\"")
      .nth(1)
      .and_then(|rest| rest.split('"').next())
      .unwrap_or_default();
    let cut = d
      .match_indices(" C ")
      .nth(149)
      .map_or(d.len(), |(at, _)| at);
    // The state key the path is stored under is interned by the conversion:
    // an arena reset before this point would make the check below vacuous.
    assert!(latexml_core::common::arena::get("pgf_SVGpath").is_some());
    latexml_core::common::arena::get(&d[..cut]).is_some()
  });
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // pgfsys-latexml's px: 100 dpi over 72.27pt, to two places.
  let px = |pt: f64| (pt * 100.0 / 72.27 * 100.0).round() / 100.0;
  let mut d = String::from("M 0 0");
  for i in 1..=300 {
    let x = px(f64::from(i));
    d.push_str(&format!(
      " C {x} {} {x} {} {x} {}",
      px(1.0),
      px(2.0),
      px(3.0)
    ));
  }
  latexml::util::test::assert_element(
    &xml,
    "svg:path",
    &[],
    &format!("<svg:path d=\"{d}\" style=\"fill:none\"/>"),
  );
  assert_eq!(xml.matches("<svg:path").count(), 1, "{xml}");
  assert!(!prefix_interned, "a prefix of the path was interned");
}

/// A finite long plot is no digestion loop. Every point of a pgf `plot[smooth]`
/// digests the same content-free boxes (empty `\pgf@process` groups, a stray
/// space; pgflibraryplothandlers.code.tex:37-77), so an 11,000-point plot read as
/// a cycle (tkz-grapheur-doc-en/-fr Fatal, sweep #126). A box period is a loop
/// only when the tokens read per box repeat too, and coordinates of varying
/// length do not; the soft path, expanded at once (pgfsyssoftpath.code.tex:
/// 122-131), fits the ar5iv pushback limit raised to the binary's 5,000,000. The
/// path is Perl's (without ar5iv, whose 599,999 limit stops it) byte for byte:
/// 347,233 bytes, 8977 curveto.
#[test]
fn smooth_plot_past_the_box_cycle_floor() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/smooth_plot_box_cycle_floor.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<svg:path").count(), 1);
  let d = xml
    .split(" d=\"")
    .nth(1)
    .and_then(|rest| rest.split('"').next())
    .unwrap_or_default();
  assert_eq!(d.len(), 347_233);
  assert_eq!(d.matches(" C ").count(), 8977);
  // FNV-1a of Perl's `d` for the same plot.
  let fnv = d.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
    (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
  });
  assert_eq!(fnv, 0x877e_57b1_c478_5814);
}

/// A runaway of content-free boxes read from the same input each time is still
/// a digestion loop, stopped at the cycle guard's floor (pdflatex never ends):
/// ` {}\x` and ` {} {}\relax\x` read the same tokens per period, and in
/// ` {} {} {}\y`/` {} {} {}\relax\x` the input repeats every 12 boxes over a
/// 2-box window; a `\loop` whose counter wraps every 33 iterations repeats its
/// input every 66 boxes. Excluding content-free boxes from the guard instead let
/// the first two run to the memory ceiling; mixing the span into the box
/// fingerprint left the third to the count cap (3.2 s, 3.7 GB); an input period
/// bound of 64 left the fourth to it (7.9 s, 2.9 GB).
#[test]
fn runaway_space_group_loop_is_still_a_loop() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/runaway_space_group_loop.tex"
  );
  let variant = "\\documentclass{article}\n\\begin{document}\nA \\def\\x{ {} {}\\relax\\x}\\x\n\\end{document}\n";
  let alternating = "\\documentclass{article}\n\\begin{document}\nA \\def\\x{ {} {} {}\\y}\\def\\y{ {} {} {}\\relax\\x}\\x\n\\end{document}\n";
  // A counter wrapping every 33 iterations: input period 66 boxes.
  let wrapping = "\\documentclass{article}\n\\newcount\\n\n\\begin{document}\nA \\loop\\advance\\n1 \\ifnum\\n=33 \\n=0 \\fi\\mbox{}\\ \\ifnum\\n>-1 \\repeat\n\\end{document}\n";
  for tex in [tex, variant, alternating, wrapping] {
    let (stderr, _xml) = convert_with(tex, Some("ar5iv.sty"));
    assert!(
      stderr.contains("Stomach:Recursion") && stderr.contains("Infinite digestion loop"),
      "{stderr}"
    );
    assert_eq!(error_count(&stderr), 1, "{stderr}");
  }
}

/// A long expansion is no runaway: `\edef\a{\a\a}` 17 times builds 1,310,720
/// tokens, which pdflatex holds (TeX pushes a pointer to a token list, tex.web
/// §323; its bound is `main_memory`, one word per token). The flat pushback
/// copies the body, so the ar5iv limit is the binary's own 5,000,000, not 650,000.
#[test]
fn a_long_expansion_fits_the_pushback() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/pushback_long_expansion.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">done</p>"#);
}

/// A finite loop printing the same letter is no digestion loop, even when its
/// input alternates (`\ifodd` branches: 60,000 x's). The box window's
/// uniform-run exemption judges the boxes alone; mixing the tokens read per box
/// into the box fingerprint made the run look periodic and lost the document.
#[test]
fn a_finite_loop_of_one_letter_is_no_digestion_loop() {
  let tex = "\\documentclass{article}\n\\newcount\\n\n\\begin{document}\n\\loop\\advance\\n1 \\ifodd\\n x\\else x\\fi\\ifnum\\n<60000 \\repeat\ndone\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let p = format!("{}done", "x".repeat(60_000));
  latexml::util::test::assert_element(&xml, "p", &[], &format!(r#"<p xml:id="p1.1">{p}</p>"#));
}

/// An environment's arguments revert one per valued parameter. Its leading
/// `SkipSpaces` (novalue; Perl Package.pm:1908-1912) stores no argument and
/// takes none (Perl Parameters.pm:51-57). Pairing every parameter with an
/// argument shifted each one a slot left and dropped the last:
/// `\begin{minipage}[t]{3cm}` reverted as `\begin{minipage}[85.35826pt]`, so
/// adjustbox's re-digestion (`\lx@RE@BOXCONTENT`) read `{second mini}` as the
/// width, and a picture lost its size in `tex`. Perl and pdflatex are clean.
#[test]
fn environment_arguments_revert_in_their_slots() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/minipage_reversion_width_as_position.tex"
  );
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r##"<para xml:id="p1"><inline-block depth="0.0pt" height="6.9pt" width="85.4pt" xscale="1" xtranslate="0.0pt" yscale="1" ytranslate="0.0pt" xml:id="p1.1"><p xml:id="p1.1.1"><text backgroundcolor="#FF0000" xml:id="p1.1.1.1"><inline-block class="ltx_minipage" vattach="middle" width="85.4pt" xml:id="p1.1.1.1.1"><p xml:id="p1.1.1.1.1.1">second mini</p></inline-block></text></p></inline-block></para>"##,
  );
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
$a\mbox{\begin{minipage}[t]{3cm}x\end{minipage}}$

$b\mbox{\begin{picture}(10,10)\put(0,0){p}\end{picture}}$

$c\mbox{\begin{minipage}[b][2cm][t]{3cm}z\end{minipage}}$

$k\mbox{\begin{minipage}{3cm}\begin{align}a&=b\end{align}\end{minipage}}$
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // The last argument is no longer dropped, and an alignment's body is reverted, as Perl's.
  for reverted in [
    r#"tex="a\mbox{\begin{minipage}[t]{85.35826pt}x\end{minipage}}""#,
    r#"tex="b\mbox{\begin{picture}(10.0,10.0)\put(0.0,0.0){p}\end{picture}}""#,
    r#"tex="c\mbox{\begin{minipage}[b][2cm][t]{85.35826pt}z\end{minipage}}""#,
    r#"tex="k\mbox{\begin{minipage}{85.35826pt}\@@amsalign a&amp;=b\end{minipage}}""#,
  ] {
    assert!(xml.contains(reverted), "{reverted} in {xml}");
  }
}

/// A box's measured size is its constructed size (theme 11, K18). `\wd`/`\ht`/`\dp`
/// read typed sizes only:
/// - `\framebox[w]` stored its width as an attribute string, ignored, so it
///   measured content plus padding (12.08pt; latex.ltx:16196-16229 puts the frame
///   inside the width);
/// - `\raisebox` had no sizer; it now has Perl's `raisedSizer`, the raise counted
///   (latex.ltx:16378-16393; its [height][depth]: `box_size_arguments_measure_the_box`);
/// - `\parbox` round-tripped its width through a 0.1pt string (50.0pt);
/// - the graphics boxes' sizes were strings, so the default sizer summed their
///   arguments as text (`\resizebox{1em}` 49.7pt; now Perl's 10.00002pt, pdflatex
///   10.00081pt through `\Gscale@div`).
///
/// pdflatex's values, except that last.
#[test]
fn box_sizes_are_their_constructed_sizes() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_dimensions_measured.tex");
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    "<p xml:id=\"p1.1\">[10.00002pt]\n[6.30554pt]\n[50.00008pt]\n[10.00002pt]</p>",
  );
}

/// A box's requested width is not its contents' paragraph width: Perl's
/// `computeBoxesSize` breaks lines only at the list's own width (Font.pm:683), so
/// `\makebox[1em]{aaa bbb ccc ddd}` is one line, 6.94444pt high (it was set as a
/// 1em paragraph: 4.3pt high, 36pt deep). A box that is a paragraph still breaks
/// at its own width, scaled or rotated: `\parbox{3cm}`, `\hbox to 3cm`. `\raisebox`
/// is Perl's `raisedSizer`, the raise counted.
/// All pdflatex's values.
#[test]
fn a_box_width_does_not_break_its_contents() {
  let tex = r"\documentclass{article}
\usepackage{graphicx}
\def\LT{aaa bbb ccc ddd eee fff ggg hhh iii jjj kkk lll mmm nnn ooo ppp}
\def\M#1{\setbox0\hbox{#1}[\the\wd0][\the\ht0][\the\dp0]\par}
\begin{document}
\M{\makebox[1em]{aaa bbb ccc ddd}}
\M{\raisebox{2pt}{x}}
\M{\parbox{3cm}{\LT}}
\M{\scalebox{2}{\parbox{3cm}{\LT}}}
\M{\rotatebox{90}{\parbox{3cm}{\LT}}}
\M{\hbox to 3cm{aaa bbb ccc ddd eee}}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let measured: Vec<&str> = xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| {
      p.split_once('>')
        .and_then(|(_, rest)| rest.split_once("</p>"))
    })
    .map(|(text, _)| text)
    .collect();
  assert_eq!(
    measured,
    [
      "[10.00002pt][6.94444pt][0.0pt]",
      "[5.2778pt][6.30554pt][0.0pt]",
      "[85.35826pt][24.94444pt][19.94444pt]",
      "[170.71652pt][49.88889pt][39.88889pt]",
      "[44.88889pt][85.35826pt][0.0pt]",
      "[85.35826pt][6.94444pt][0.0pt]",
    ],
    "{xml}"
  );
}

/// An EPS's DSC bounding box is read as bytes (batch 56kk): a
/// Latin-1 `%%Title` byte failed the strict UTF-8 decode of the whole 32 KB
/// window, and a CR-only file was one line, so both `<graphics>` went unsized.
/// pdflatex (epstopdf) sizes all four 144bp x 72bp. The `(atend)` trailer and
/// DOS EPS past the first 32 KB: `image::sizing_characterization_tests::postscript_boxes_are_read_as_bytes`.
#[test]
fn eps_bounding_boxes_are_read_as_bytes() {
  let (stderr, xml) = super::package_leads_56::convert_with_byte_files(
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/eps_bounding_box_bytes.tex"),
    &[
      (
        "eps_bbox_latin1.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/eps_bbox_latin1.eps"),
      ),
      (
        "eps_bbox_cr.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/eps_bbox_cr.eps"),
      ),
      (
        "eps_bbox_atend.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/eps_bbox_atend.eps"),
      ),
      (
        "eps_bbox_plain.eps",
        include_bytes!("../../../tools/perfect_kernel/repros/graphics-tikz/eps_bbox_plain.eps"),
      ),
    ],
    "ar5iv.sty",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (n, name) in ["latin1", "cr", "atend", "plain"].iter().enumerate() {
    let id = format!("p{}.g1", n + 1);
    latexml::util::test::assert_element(
      &xml,
      "graphics",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        r#"<graphics candidates="eps_bbox_{name}.eps" cssstyle="width:14.454em; height:7.227em" graphic="eps_bbox_{name}.eps" xml:id="{id}"/>"#
      ),
    );
  }
}

/// A box command's size arguments measure the box (theme 11, K18 step 2):
/// latex.ltx's `\@begin@tempboxa` (16085-16094) sets the box, then evaluates the
/// sizes with `\width`/`\height`/`\depth`/`\totalheight` = `\wd\@tempboxa`…
/// (`\@imakebox`, `\@iframebox`, `\@iiiparbox`'s height, `\raisebox`'s
/// [height][depth]). They were 0pt text read before the box (Perl alike):
/// `\makebox[2\width]` was 20pt, siamart's `\raisebox{0pt}[\height][0pt]`
/// (2605.00332) kept its depth, a counter stepped in the box went unseen.
/// pdflatex's values; calc reads them too, and a `{}` box is set in its own
/// group (`\Large` does not reach the `2em`). The raise itself:
/// `raisebox_raise_measures_the_box.tex` (RED).
#[test]
fn box_size_arguments_measure_the_box() {
  let rows = |xml: &str| -> Vec<String> {
    xml
      .split("<p xml:id=\"")
      .skip(1)
      .filter_map(|p| {
        p.split_once('>')
          .and_then(|(_, rest)| rest.split_once("</p>"))
      })
      .map(|(text, _)| text.to_string())
      .filter(|text| text.starts_with('['))
      .collect()
  };
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/box_size_arguments_measure_the_box.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(
    rows(&xml),
    [
      "[5.00002pt][4.30554pt][0.0pt]",
      "[5.00002pt][6.24998pt][1.94444pt]",
      "[10.5556pt][4.30554pt][0.0pt]",
      "[10.5556pt][4.30554pt][0.0pt]",
      "[10.5556pt][7.70554pt][3.4pt]",
      "[1.0pt][4.30554pt][0.0pt]",
      "[28.45274pt][8.61108pt][0.0pt]",
    ],
    "{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "p",
    &["xml:id=\"p8.1\""],
    r#"<p xml:id="p8.1">A<text align="center" width="10.6pt" xml:id="p8.1.1">x</text>C</p>"#,
  );
  // The siam form's optionals, and a [height] alone (latex.ltx:16378-16393).
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/raisebox_optionals_measure_the_box.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let heights: Vec<String> = xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| {
      p.split_once('>')
        .and_then(|(_, rest)| rest.split_once("</p>"))
    })
    .map(|(text, _)| text.to_string())
    .collect();
  assert_eq!(
    heights,
    [
      "[1.0pt][3.0pt]",
      "[4.30554pt][0.0pt]",
      "[0.0pt][0.0pt]",
      "[3.65973pt][0.0pt]"
    ],
    "{xml}"
  );
  // calc evaluates the width expression with `\width` bound; `\Gscale@box@dd`'s
  // plain `{}` box is set in a group, so its `\Large` leaves `2em` at 10pt
  // (without the group ~28.8pt; the last 0.0004pt is `\Gscale@div`'s).
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{calc}\n\\usepackage{graphicx}\n\
     \\def\\M#1{\\setbox0\\hbox{#1}[\\the\\wd0][\\the\\ht0][\\the\\dp0]\\par}\n\
     \\begin{document}\n\\M{\\makebox[\\width+2pt]{x}}\n\
     \\makeatletter\\M{\\Gscale@box@dd{2em}\\width{\\Large x}}\\makeatother\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let measured = rows(&xml);
  assert_eq!(
    measured.first().map(String::as_str),
    Some("[7.2778pt][4.30554pt][0.0pt]"),
    "{xml}"
  );
  let scaled_width: f64 = measured
    .get(1)
    .and_then(|row| row.strip_prefix('[')?.split_once("pt]"))
    .and_then(|(width, _)| width.parse().ok())
    .unwrap_or_default();
  assert!((scaled_width - 20.0).abs() < 0.01, "{xml}");
}

/// `\rotatebox[…]` turns the box about graphicx's point (`\Grot@box@kv`,
/// graphicx.sty:226-242): with `[…]` it starts at the centre, then the keys
/// apply in order (`origin` letters per axis, `c` a no-op, `x`/`y`); without
/// `[…]` it is the reference point. The keys were read and ignored (`origin=c`
/// at 90° came out 12.8pt high, 0pt deep; pdflatex 8.83/3.94pt), and Perl starts
/// at the reference point with `c` as a letter (DIVERGENCES #329). Witnesses
/// 2605.02583 (`valign=m` label raise), 2605.14476, 2605.22405, 2605.30813.
/// pdflatex's sizes; at 45° within trig.sty's 5-digit sine.
#[test]
fn rotatebox_turns_about_its_origin() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/rotatebox_origin_keys.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let rows: Vec<&str> = xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| {
      p.split_once('>')
        .and_then(|(_, rest)| rest.split_once("</p>"))
    })
    .map(|(text, _)| text.trim_start_matches('x'))
    .collect();
  let exact = [
    (0, "[14.05556pt][8.83334pt][3.94447pt]"),
    (1, "[14.05556pt][12.77782pt][0.0pt]"),
    (3, "[14.05556pt][11.77782pt][1.0pt]"),
    (4, "[14.05556pt][4.30554pt][10.33339pt]"),
    (5, "[14.05556pt][15.22224pt][0.0pt]"),
    (6, "[14.05556pt][4.44447pt][8.33334pt]"),
    (7, "[14.05556pt][6.38892pt][6.3889pt]"),
    (8, "[14.05556pt][8.83334pt][3.94447pt]"),
    (9, "[14.05556pt][13.22224pt][0.0pt]"),
    (10, "[14.05556pt][13.22224pt][0.0pt]"),
    (11, "[14.05556pt][24.7778pt][0.0pt]"),
  ];
  assert_eq!(rows.len(), 12, "{xml}");
  for (row, want) in exact {
    assert_eq!(rows[row], want, "row {row}: {xml}");
  }
  // origin=rb at 45°: pdflatex [20.51958pt][4.30554pt][10.97954pt], its sine 0.70709.
  let lengths: Vec<f64> = rows[2]
    .trim_matches(|c| c == '[' || c == ']')
    .split("][")
    .filter_map(|v| v.trim_end_matches("pt").parse().ok())
    .collect();
  assert_eq!(lengths.len(), 3, "{xml}");
  for (got, want) in lengths.iter().zip([20.51958, 4.30554, 10.97954]) {
    assert!((got - want).abs() < 0.001, "{xml}");
  }
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{graphicx}\n\\begin{document}\n\
     A\\rotatebox[origin=c]{90}{Qg}B\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "inline-block",
    &["xml:id=\"p1.1.1\""],
    r#"<inline-block angle="90" depth="3.9pt" height="8.8pt" innerdepth="1.9pt" innerheight="6.8pt" innerwidth="12.8pt" width="8.8pt" xtranslate="-2.0pt" ytranslate="-2.0pt" xml:id="p1.1.1"><p xml:id="p1.1.1.1">Qg</p></inline-block>"#,
  );
}

/// A `\unitlength` set through calc is the dimen assigned: calc's `\setlength`
/// assigns every length as a skip (calc.sty:86, 51-53) and TeX keeps its width
/// (tex.web §429). It read as 1pt, collapsing a paper's pictures toward the
/// origin: 201 pictures in 6 of 14 probed papers (2605.11190 ×93, 2605.05506
/// ×70, 2605.18952, 2605.10502, 2605.02197, 2605.15276). pdflatex:
/// `unitlength="2.0pt"`, a 20pt × 10pt picture, `a` at (10pt, 4pt); a stretch
/// is dropped (`[2.0pt]`, where Perl keeps the glue, KPE #300).
#[test]
fn unitlength_set_through_calc_is_the_length() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/unitlength_calc_glue.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "picture",
    &["xml:id=\"p1.pic1\""],
    r##"<picture fill="none" height="10.0pt" stroke="none" unitlength="2.0pt" width="20.0pt" xml:id="p1.pic1"><g innerdepth="0.0pt" innerheight="4.3pt" innerwidth="5.0pt" transform="translate(13.84,5.53)"><text xml:id="p1.pic1.1">a</text></g><line points="0,0 27.67,0" stroke="#000000" stroke-width="0.4"/></picture>"##,
  );
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{calc}\n\\begin{document}\n\
     \\setlength{\\unitlength}{2pt plus 1pt minus 1fil}[\\the\\unitlength]\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r#"<p xml:id="p1.1">[2.0pt]</p>"#);
}

/// A picture's size is its declared `(w,h)` in `\unitlength`s, depth 0 (Perl
/// latex_constructs.pool.ltxml:4966-4973 `picScale` Dimensions): stored as
/// strings, the size code measured its contents instead, so a resized picture
/// with a rotated label was scaled ×8.33 (2605.26285, 2605.15276). pdflatex: each
/// scaled ×0.5 to 100pt × 50pt.
#[test]
fn a_picture_measures_its_declared_size() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/picture_size_is_typed.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let scaled: Vec<&str> = xml
    .match_indices("<inline-block ")
    .filter_map(|(start, _)| xml[start..].split_once('>').map(|(tag, _)| tag))
    .filter(|tag| tag.contains("xscale="))
    .collect();
  let want = |n: u8| {
    format!(
      r#"<inline-block depth="0.0pt" height="50.0pt" width="100.0pt" xscale="0.5" xtranslate="-50.0pt" yscale="0.5" ytranslate="25.0pt" xml:id="p{n}.1""#
    )
  };
  assert_eq!(scaled, [want(1), want(2), want(3)], "{xml}");
}

/// An hbox's height and depth start at 0 (tex.web §649 `hpack`): a one-item
/// list, measured through `compute_boxes_size`'s bare-box shortcut, kept its
/// item's negative depth — `\hbox{\rotatebox[origin=l,y=1.2em]{90}{Qg}}` was
/// 12pt deep below 0, `\hbox{\box2}` with `\dp2=-5pt` −5pt (Perl the same, its
/// `List()` returning the lone box; DIVERGENCES #331; arXiv 2605.19974). A vbox
/// keeps its last box's negative depth (§670). pdflatex's values.
#[test]
fn a_one_item_hbox_has_no_negative_depth() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/hbox_single_child_negative_depth.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let rows: Vec<&str> = xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| {
      p.split_once('>')
        .and_then(|(_, rest)| rest.split_once("</p>"))
    })
    .map(|(text, _)| text)
    .collect();
  assert_eq!(
    rows,
    [
      "[8.77776pt][24.7778pt][0.0pt]",
      "[14.05556pt][24.7778pt][0.0pt]",
      "[5.2778pt][4.30554pt][0.0pt]",
      "[5.2778pt][4.30554pt][-5.0pt]"
    ],
    "{xml}"
  );
}

/// A box whose width, height and depth are all given keeps that size without
/// measuring its contents (Perl `computeSizeStore`, Box.pm:266-281). Since
/// 56kl `\makebox`'s box is measured for `\width`/`\height`/`\depth`, and a pgf
/// picture's `\lxSVG@insertpicture` declares its size, yet sizing the box
/// walked the picture's every drawing box (3,611 here; 11,732 sizings for three
/// pgf-spectra pictures, +33 % RSS): pgf-spectraPreviewDataLSE went from a
/// clean 126 s to `Fatal:Timeout:MemoryBudget`. `\makebox[2\width]` still
/// doubles the picture's width (28.85pt); the control, `\wd` of a text box, is
/// still measured, so the size trace is live.
#[test]
fn a_declared_size_is_not_measured() {
  let (stderr, xml) = super::perfect_kernel_gemini::convert_env_args(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/declared_size_not_measured.tex"
    ),
    &[],
    &[("LXML_SIZE_TRACE", "1")],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let widest = stderr
    .lines()
    .filter_map(|line| {
      line
        .split_once(" nboxes=")?
        .1
        .split(' ')
        .next()?
        .parse::<usize>()
        .ok()
    })
    .max();
  assert!(
    widest.is_some_and(|n| n < 100),
    "a picture's drawing was measured: widest sizing {widest:?}"
  );
  let start_tags = |name: &str| -> Vec<&str> {
    xml
      .match_indices(name)
      .map(|(at, _)| &xml[at..at + xml[at..].find('>').unwrap() + 1])
      .collect()
  };
  assert_eq!(
    start_tags("<picture "),
    [
      r#"<picture height="39.92" width="41.93" xml:id="p1.pic1">"#,
      r#"<picture height="39.92" width="39.92" xml:id="p2.pic1">"#
    ],
    "{xml}"
  );
  let texts = start_tags("<text ");
  assert_eq!(
    texts,
    [
      r#"<text align="center" width="345.0pt">"#,
      r#"<text align="left" width="57.7pt">"#
    ],
    "{xml}"
  );
}

/// A TikZ node's foreignObject takes its box's declared size (Perl's
/// `computeSizeStore` keeps a complete size, Box.pm:275-281): a `\rule` node
/// was 0×0, a `fit` node's minipage 0 high, a `\scalebox{0.3}` its unscaled
/// 13.84 high, a `\quad` unmeasured (arXiv A/B 56ko2 → 56kq, 73 papers, every
/// changed size Perl's; 2605.12644, 2605.00742, 2605.00859).
#[test]
fn a_node_takes_its_boxs_declared_size() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/node_size_is_the_declared_size.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let objects: Vec<&str> = xml
    .match_indices("<svg:foreignObject ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').unwrap() + 1])
    .collect();
  assert_eq!(
    objects,
    [
      r#"<svg:foreignObject height="0.42" overflow="visible" style="--ltx-fo-width:10.81em;--ltx-fo-height:0.03em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 0.42)" width="149.58">"#,
      r#"<svg:foreignObject height="10.19" overflow="visible" style="--ltx-fo-width:11.52em;--ltx-fo-height:0.37em;--ltx-fo-depth:0.37em;font-size:10pt;" transform="matrix(1 0 0 -1 0 5.1)" width="159.4">"#,
      r#"<svg:foreignObject height="4.15" overflow="visible" style="--ltx-fo-width:0.39em;--ltx-fo-height:0.23em;--ltx-fo-depth:0.08em;font-size:10pt;" transform="matrix(1 0 0 -1 0 3.11)" width="5.4">"#,
      r#"<svg:foreignObject height="14.76" overflow="visible" style="--ltx-fo-width:2.19em;--ltx-fo-height:0.71em;--ltx-fo-depth:0.2em;font-size:11.75pt;" transform="matrix(1 0 0 -1 0 11.53)" width="35.65">"#,
    ],
    "{xml}"
  );
}

/// A font's parameters are its TFM's, scaled to its size, from its `\font`
/// on (Perl TeX_Fonts.pool.ltxml:108-117, tex.web §575): `\fontdimen2` of
/// cmr10 is 3.33334pt where the stand-in read 5pt, cmsy10 at 20pt keeps its
/// 5pt axis, and cmr10's seventh parameter (extra space) is 1.11111pt.
#[test]
fn a_font_has_its_tfm_parameters() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/fontdimen_reads_font_parameters.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      "<p>[3.33334pt] [4.30556pt] [10.00003pt] [1.11111pt] [5.0pt] [3.33334pt]\n[3.33334pt]</p>"
    ),
    "{xml}"
  );
}

/// An intarray entry never set reads 0: `\intarray_new:Nn` sets entries 1-8
/// and the last (l3intarray.dtx:654-674) and TeX zero-fills the rest of the
/// font's parameters (tex.web §580) — the stand-ins read 9-12 and 22 nonzero,
/// where l3regex keeps its NFA's active steps.
#[test]
fn an_unset_intarray_item_is_zero() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/expl3/intarray_unset_item_is_zero.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let zeros = vec!["0"; 30].join(" ");
  assert!(xml.contains(&format!("<p>{zeros}</p>")), "{xml}");
}

/// A font with no metric of its own takes its family's at 10pt, else cmr10's,
/// scaled to its size (Perl `getMetricForName`, Common/Font.pm:551-562):
/// `cmr11`, small caps and bold italic read an empty array — 0pt — in 56kr's
/// first cut. An `at` size scales them exactly (Perl truncates, KPE #303). And
/// `\em` in italic or slanted text is upright (latex.ltx:14058), now that the
/// font's slant is its TFM's.
#[test]
fn a_font_without_its_own_metric_has_cmr_parameters() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/fontdimen_font_without_its_own_metric.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(concat!(
      r#"<p>[3.66667pt] [4.0pt] <text font="smallcaps">[3.33334pt]</text> <text font="bold italic">[3.33334pt]</text>"#,
      "\n",
      r#"<text fontsize="110%">[3.66667pt]</text></p>"#
    )),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<p><text font="italic">a </text>b <text font="slanted">c </text>d</p>"#),
    "{xml}"
  );
}

/// Whitespace in a pgf picture's SVG is ignorable, typewriter font or not
/// (DIVERGENCES #47): pgf pads every protocol literal with a `\space`
/// (pgfsysprotocol.code.tex:33), and under `\ttfamily` the verbatim-space
/// exception of `Document::open_text` kept it inside `svg:g`, auto-opening a
/// whitespace-only `svg:foreignObject` for each — 185 in 2605.01517,
/// 2605.07323, 2605.27124; Perl none. Without them pgf's stroke and fill
/// groups merge into one (`collapse_svg_group`), which holds the drawing.
#[test]
fn a_typewriter_space_opens_no_foreign_object() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/tt_space_opens_foreignobject.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<svg:foreignObject").count(), 0, "{xml}");
  let flat: String = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    flat.contains(concat!(
      r##"<svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" "##,
      r#"transform="translate(0,39.92) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)"> "#,
      r#"<svg:path d="M 0 0 L 39.37 39.37" style="fill:none"/> </svg:g>"#
    )),
    "the drawing is kept: {xml}"
  );
}

/// The sn-jnl binding runs Perl's dependency scan over the shipped class
/// (`maybeRequireDependencies`, Package.pm:2776-2813; Perl has no sn-jnl
/// binding, so OmniBus runs it): it had loaded a hand-picked subset, so
/// booktabs's `\toprule` was undefined (2606.00121, DIVERGENCES #333). The
/// class's program is not loaded (Perl binds none; it turns `\(`…`\)` into a
/// programbox), nor its article again — `\LoadClass[twoside,fleqn]{article}`
/// would set display equations flush left, where pdflatex centres them. The
/// class's xcolor leaves the paper's `[table]{xcolor}` its `m{}` column.
#[test]
fn the_sn_jnl_class_loads_its_packages() {
  let (stderr, xml) = super::perfect_kernel_batch51::convert_with_files(
    include_str!("../../../tools/perfect_kernel/repros/loader/snjnl_class_packages_loaded.tex"),
    &[(
      "sn-jnl.cls",
      include_str!("../../../tools/perfect_kernel/repros/loader/sn-jnl.cls"),
    )],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for element in [
    r#"<td align="left" border="tt">a</td>"#,
    r#"<td align="left" border="bb t">d</td>"#,
    r#"<listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain">"#,
    r#"<Math mode="inline" tex="c" text="c" xml:id="p2.m1">"#,
    r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">"#,
    r#"<td align="left" vattach="middle"><inline-block vattach="middle" width="28.5pt">"#,
  ] {
    assert!(xml.contains(element), "{element} missing:\n{xml}");
  }
  assert!(!xml.contains("ltx_minipage"), "{xml}");
}

/// amsmath processes its options (Perl amsmath.sty.ltxml:61): the binding
/// declared `fleqn`/`leqno`/`reqno` but never ran `\ProcessOptions`, so
/// `\usepackage[fleqn,leqno]{amsmath}` set neither document class (2605.06944)
/// and the handlers stayed defined for a later class re-load to fire (batch
/// 56kt).
#[test]
fn amsmath_processes_its_options() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/math-parse/amsmath_options_processed.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_fleqn ltx_leqno">"#),
    "{xml}"
  );
  // `reqno` removes `ltx_leqno` (Perl `ltx_leqno => undef`): an AMS class's
  // left default, overridden by `\documentclass[reqno]` (2605.02221).
  let (stderr, xml) = convert_with(
    "\\documentclass[reqno]{amsart}\n\\begin{document}\n\\begin{equation}x=y\\end{equation}\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">"#),
    "{xml}"
  );
  // A global `fleqn` reaches amsmath's handler, whose `\if@fleqn` sets a
  // `multline`'s middle rows flush left, as its first (2605.25642).
  let (stderr, xml) = convert_with(
    "\\documentclass[fleqn]{article}\n\\usepackage{amsmath}\n\\begin{document}\n\
     \\begin{multline}a+b\\\\c+d\\\\e+f\\end{multline}\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let aligns: Vec<&str> = xml
    .split("<XMCell align=\"")
    .skip(1)
    .map(|rest| &rest[..rest.find('"').unwrap()])
    .collect();
  assert_eq!(aligns, ["left", "left", "right"], "{xml}");
}

/// A class's `\LoadClass` options are its own (latex.ltx `\LoadClass` →
/// `\@onefilewithoptions`; only `\documentclass` sets `\@classoptionslist`),
/// and amsmath's `\newif\if@fleqn` (amsmath.sty:64) discards a class's `fleqn`
/// unless amsmath receives it. webofc and USG do `\LoadClass[fleqn]{article}`
/// and load amsmath bare; pdflatex centres their equations (KPE #304;
/// 2605.12407, 2605.00042).
#[test]
fn loadclass_options_are_the_class_own() {
  use super::perfect_kernel_batch46::{convert_files, convert_files_with};
  let class = [(
    "lcfleqn.cls",
    include_str!("../../../tools/perfect_kernel/repros/loader/lcfleqn.cls"),
  )];
  let repro =
    include_str!("../../../tools/perfect_kernel/repros/loader/loadclass_options_stay_local.tex");
  let bare = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">"#;
  // No preload: the unbound class falls to OmniBus, whose dependency scan
  // runs the class's `\LoadClass` (the one warning: no lcfleqn binding).
  let (stderr, xml) = convert_files_with(repro, &class, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(xml.contains(bare), "{xml}");
  assert!(xml.contains(r#"<equation xml:id="S0.E1">"#), "{xml}");
  // Raw classes: the `\LoadClass` runs article's `fleqn` handler; amsmath
  // discards it ...
  let (stderr, xml) = convert_files(repro, &class);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(bare), "{xml}");
  assert!(xml.contains(r#"<equation xml:id="S0.E1">"#), "{xml}");
  // ... and without amsmath the equation stays flush left, as in pdflatex.
  let (stderr, xml) = convert_files(&repro.replace("\\usepackage{amsmath}\n", ""), &class);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_fleqn">"#),
    "{xml}"
  );
  // The document's options stay the global ones (pdflatex `[french]`; Perl and
  // Rust 56ku `[fleqn]`, the nested load's).
  let list = "\\documentclass[french]{lcfleqn}\n\\makeatletter\n\\begin{document}\nList: \
              [\\@classoptionslist]\n\\end{document}\n";
  for (stderr, xml) in [
    convert_files_with(list, &class, None),
    convert_files(list, &class),
  ] {
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains("List: [french]"), "{xml}");
  }
}

/// elsarticle's equations are flush left only in the `5p` layout, or `3p` with
/// `twocolumn`: the class inputs fleqn.clo there alone (elsarticle.cls:1280,
/// 1295). Perl's binding loads fleqn for every layout (KPE #305); pdflatex
/// centres the default `preprint`, `1p` and `3p` (2605.05858). The layouts are
/// declared options, so the last declared wins: `[3p,5p]` is `3p`, centred.
#[test]
fn elsarticle_fleqn_follows_the_journal_type() {
  let repro = include_str!(
    "../../../tools/perfect_kernel/repros/loader/elsarticle_fleqn_follows_journal_type.tex"
  );
  for (options, class) in [
    ("", ""),
    ("[1p]", ""),
    ("[3p]", ""),
    ("[3p,twocolumn]", r#" class="ltx_fleqn""#),
    ("[5p]", r#" class="ltx_fleqn""#),
    ("[3p,5p]", ""),
  ] {
    let tex = repro.replace("\\documentclass{", &format!("\\documentclass{options}{{"));
    let (stderr, xml) = convert_with(&tex, None);
    assert_eq!(error_count(&stderr), 0, "{options}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{options}: {stderr}");
    let document = format!(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML"{class}>"#);
    assert!(
      xml.contains(&document),
      "{options}: {document} missing:\n{xml}"
    );
  }
}

/// amsmath.sty:53 `\newif\iftagsleft@`: amsmath's tags follow only its own
/// `leqno`/`reqno`, processed in its declared order (amsmath.sty:54-55), so a
/// class's own `leqno` (raw `\LoadClass[leqno]{article}`) stops applying, and
/// a local `reqno` beats a global `leqno`. amsart passes its default `leqno`
/// to amsmath (amsart.cls:159-162), as amsbook does (amsbook.cls:130-133);
/// acmart loads amsart with `reqno` (2605.02222); siamart, aomart and imsart's
/// `aos`-family journals pass `leqno` (2605.02838, 2605.16086). The pdflatex
/// tag side of each case is noted beside it.
#[test]
fn amsmath_tags_follow_its_own_options() {
  let bare = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">"#;
  let left = r#"<document xmlns="http://dlmf.nist.gov/LaTeXML" class="ltx_leqno">"#;
  // Right.
  let (stderr, xml) = convert_files(
    include_str!(
      "../../../tools/perfect_kernel/repros/loader/amsmath_tagsleft_discards_class_leqno.tex"
    ),
    &[(
      "lcleqno.cls",
      include_str!("../../../tools/perfect_kernel/repros/loader/lcleqno.cls"),
    )],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(bare), "{xml}");
  // imsart ships without TeX Live: one missing-class warning.
  for (preamble, document, warnings) in [
    (
      "\\documentclass[leqno]{article}\n\\usepackage[reqno]{amsmath}",
      bare,
      0,
    ), // right
    ("\\documentclass{amsart}", left, 0),        // left
    ("\\documentclass[reqno]{amsart}", bare, 0), // right
    (
      "\\documentclass[leqno]{article}\n\\usepackage{amsmath}",
      left,
      0,
    ), // left
    ("\\documentclass{acmart}", bare, 0),        // right
    ("\\documentclass{amsbook}", left, 0),       // left
    ("\\documentclass[reqno]{amsbook}", bare, 0), // right
    ("\\documentclass{siamart220329}", left, 0), // left
    ("\\documentclass{aomart}", left, 0),        // left
    ("\\documentclass[aos]{imsart}", left, 1),   // left
    ("\\documentclass{imsart}", bare, 1),        // right
  ] {
    let (stderr, xml) = convert_with(
      &format!(
        "{preamble}\n\\begin{{document}}\n\\begin{{equation}}x=1\\end{{equation}}\n\\end{{document}}\n"
      ),
      None,
    );
    assert_eq!(error_count(&stderr), 0, "{preamble}: {stderr}");
    assert_eq!(warning_count(&stderr), warnings, "{preamble}: {stderr}");
    assert!(
      xml.contains(document),
      "{preamble}: {document} missing:\n{xml}"
    );
  }
}

/// A natural-size vector figure's `em` box is its natural size under the ops
/// the post-processor applies to its pixels — `trim`/`viewport` (clamped, never
/// padded) and `angle` — so the cropped or turned figure is not stretched back
/// into the whole page's box (review of batch 56kx; 2605.06510).
#[test]
fn a_natural_figure_box_follows_its_crop_and_turn() {
  use super::perfect_kernel_batch46::convert_files;
  let eps = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 200 100\n%%EndComments\n\
             newpath 0 0 moveto 200 0 lineto 200 100 lineto 0 100 lineto closepath fill\nshowpage\n";
  let css = |options: &str| {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage{{graphicx}}\n\\begin{{document}}\n\
       \\includegraphics[{options}]{{quad.eps}}\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert_files(&tex, &[("quad.eps", eps)]);
    assert_eq!(error_count(&stderr), 0, "{options}: {stderr}");
    let at = xml
      .find(r#"cssstyle=""#)
      .unwrap_or_else(|| panic!("{options}: {xml}"));
    xml[at + 10..at + 10 + xml[at + 10..].find('"').unwrap()].to_string()
  };
  let whole = css("");
  let trimmed = css("trim=0 0 100 0,clip");
  let turned = css("angle=90");
  let padded = css("trim=0 -40 0 0,clip");
  assert_eq!(whole, "width:20.075em; height:10.037em");
  assert_eq!(trimmed, "width:10.037em; height:10.037em");
  // The turn's bounding box, through sin/cos: 10.0375em rounds up here.
  assert_eq!(turned, "width:10.038em; height:20.075em");
  assert_eq!(padded, whole, "a negative trim pads nothing");
}

/// `\meaning` of a macro coded in Rust prints Perl's `CODE(0x…)`, not the
/// debug form of a `dyn` pointer (`CODE(Pointer { addr: …, metadata:
/// DynMetadata(…) })`, into cnltx_en's text; review of batch 57a).
#[test]
fn meaning_of_a_closure_macro_is_perls_code_form() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/meaning_of_a_closure_macro.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let prefix = ">macro:-&gt;CODE(0x";
  let at = xml.find(prefix).unwrap_or_else(|| panic!("{xml}"));
  let rest = &xml[at + prefix.len()..];
  let hex = rest.find(')').unwrap();
  assert!(hex > 0, "an address: {xml}");
  assert!(rest[..hex].chars().all(|c| c.is_ascii_hexdigit()), "{xml}");
  assert!(rest[hex..].starts_with(")</text>"), "{xml}");
}

/// K13 stage 0: a constructor or primitive keeps what its binding's mode
/// options compile to, beside the closures built from it, so that the
/// binding-conformance audit can compare it with the real macro's prologue.
/// One witness per definer path: `\@makebox` enters horizontal mode and begins
/// restricted horizontal (sect12.rs; 56kb); `mode => "text"` implies the
/// entering (`\lx@frontmatter@keepsup`; Perl Package.pm:1459-1461); a
/// primitive leaves horizontal mode (`\vfil`, tex_glue.rs); an environment
/// begins its mode on `\begin{center}` and on the bare `\quote` (Perl
/// Package.pm:1902); a math constructor records its grouping — a group, as
/// Perl opens one unless `nogroup` is given (Package.pm:1707; 57ab) — and
/// requires math, as every Perl DefMath constructor's `requireMath`
/// (Package.pm:1706; 57n) (`\binom@content`, the content half of the dual that
/// amsmath's DefMath `\binom` expands to);
/// `\newline` declares nothing; a macro has no mode options at all.
#[test]
fn a_definition_keeps_its_declared_mode() {
  use latexml_core::state::lookup_definition;
  // (mode, enter_horizontal, leave_horizontal, bounded, require_math, forbid_math)
  type Record = (Option<String>, bool, bool, bool, bool, bool);
  let tex =
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\nx\n\\end{document}\n";
  // Arena symbols are resolved inside the conversion's thread, before its reset.
  let (stderr, _xml, records) = convert_with_then(tex, None, |_| {
    let record = |cs: &str| -> Option<Record> {
      lookup_definition(&latexml_core::T_CS!(cs))
        .expect("a definition lookup")
        .unwrap_or_else(|| panic!("{cs} is defined"))
        .declared_mode()
        .map(|m| {
          (
            m.mode.map(latexml_core::common::arena::to_string),
            m.enter_horizontal,
            m.leave_horizontal,
            m.bounded,
            m.require_math,
            m.forbid_math,
          )
        })
    };
    [
      "\\@makebox",
      "\\lx@frontmatter@keepsup",
      "\\vfil",
      "\\begin{center}",
      "\\quote",
      "\\binom@content",
      "\\newline",
      "\\@gobble",
    ]
    .map(record)
  });
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let rh = || Some("restricted_horizontal".to_string());
  let iv = || Some("internal_vertical".to_string());
  assert_eq!(records, [
    Some((rh(), true, false, false, false, false)),
    Some((rh(), true, false, false, false, false)),
    Some((None, false, true, false, false, false)),
    Some((iv(), false, false, false, false, false)),
    Some((iv(), false, false, false, false, false)),
    Some((None, false, false, true, true, false)),
    Some((None, false, false, false, false, false)),
    None,
  ]);
}

/// K13's first finding (57d): latex.ltx:16249-16250 `\@iiiparbox` begins with `\leavevmode`, so a
/// `\parbox` between paragraphs starts one and the text after it continues that paragraph; Perl
/// makes the box a block of its own and the text a new paragraph (KNOWN_PERL_ERRORS #309).
#[test]
fn a_parbox_starts_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/parbox_starts_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p><inline-block class="ltx_parbox" vattach="middle" width="85.4pt"><p>A</p></inline-block> text after.</p></para>"#,
  );
  // After `\noindent` (horizontal mode already) the parbox continues the paragraph it began.
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para class="ltx_noindent" xml:id="p3"><p><inline-block class="ltx_parbox" vattach="middle" width="85.4pt"><p>N</p></inline-block> text after N.</p></para>"#,
  );
}

/// 57f: latex.ltx:16305-16306 `\@iiiminipage` begins with `\leavevmode`, so a minipage between
/// paragraphs starts one and the text after it continues that paragraph, as `\parbox` does (57d);
/// Perl builds the minipage as a block of its own (KNOWN_PERL_ERRORS #309).
#[test]
fn a_minipage_starts_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/minipage_starts_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p><inline-block class="ltx_minipage" vattach="middle" width="85.4pt"><p>A</p></inline-block> text after.</p></para>"#,
  );
}

/// User rulings 2026-09-27 (57g): a box that is clearly a figure (an image and `\\captionof`,
/// which wraps only its caption) becomes that figure holding its image; figure-boxes that are
/// material of one paragraph (one line) form an uncaptioned outer figure of panels, while boxes
/// of successive paragraphs (a blank line between) stay apart.
#[test]
fn figure_boxes_become_figures_and_rows() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/figure_boxes_become_figures_and_rows.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, markup) in [
    (
      "S0.F1",
      r#"<figure align="center" class="ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><graphics class="ltx_centering" graphic="a" options="width=103.50105pt,keepaspectratio=true" xml:id="g1"/><toccaption><tag close=" ">1</tag>A</toccaption><caption><tag close=": ">Figure 1</tag>A</caption></figure>"#,
    ),
    (
      "S0.F2",
      r#"<figure align="center" class="ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><graphics class="ltx_centering" graphic="b" options="width=103.50105pt,keepaspectratio=true" xml:id="g2"/><toccaption><tag close=" ">2</tag>B</toccaption><caption><tag close=": ">Figure 2</tag>B</caption></figure>"#,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "figure", &[&format!(r#"xml:id="{id}""#)], markup);
  }
  assert_eq!(
    xml.matches(r#"<figure xml:id="p"#).count(),
    1,
    "one row:\n{xml}"
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="p5""#],
    r#"<figure xml:id="p5"><figure align="center" class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><graphics class="ltx_centering" graphic="c" options="width=103.50105pt,keepaspectratio=true" xml:id="g3"/><toccaption><tag close=" ">3</tag>C</toccaption><caption><tag close=": ">Figure 3</tag>C</caption></figure><figure align="center" class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F4"><tags><tag>Figure 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Figure 4</tag></tags><graphics class="ltx_centering" graphic="d" options="width=103.50105pt,keepaspectratio=true" xml:id="p5.g1"/><toccaption><tag close=" ">4</tag>D</toccaption><caption><tag close=": ">Figure 4</tag>D</caption></figure><table align="center" class="ltx_figure_panel ltx_minipage" inlist="lot" vattach="middle" width="103.5pt" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><graphics class="ltx_centering" graphic="e" options="width=103.50105pt,keepaspectratio=true" xml:id="p5.g2"/><toccaption><tag close=" ">1</tag>E</toccaption><caption><tag close=": ">Table 1</tag>E</caption></table></figure>"#,
  );
}

/// 57g's review: figure-boxes settle when their paragraph closes — a line of nothing but
/// float-boxes becomes a row (the author's `\\` kept as a row break); a float-box beside a text
/// box stays inline, side by side (arXiv 2605.18920's table and algorithm boxes); a float that
/// holds content of its own absorbs nothing, so the box's order stands.
#[test]
fn figure_box_lines_settle_when_the_paragraph_closes() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/figure_box_lines_settle_when_the_paragraph_closes.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="p2""#],
    r#"<figure xml:id="p2"><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><graphics graphic="c" xml:id="g1"/><toccaption><tag close=" ">1</tag>C</toccaption><caption><tag close=": ">Figure 1</tag>C</caption></figure><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><graphics graphic="d" xml:id="p2.g1"/><toccaption><tag close=" ">2</tag>D</toccaption><caption><tag close=": ">Figure 2</tag>D</caption></figure><break/><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><graphics graphic="e" xml:id="p2.g2"/><toccaption><tag close=" ">3</tag>E</toccaption><caption><tag close=": ">Figure 3</tag>E</caption></figure></figure>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p><inline-logical-block class="ltx_minipage" vattach="middle" width="172.5pt"><table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><graphics graphic="t" xml:id="g2"/><toccaption><tag close=" ">1</tag>T</toccaption><caption><tag close=": ">Table 1</tag>T</caption></table></inline-logical-block> <inline-block class="ltx_minipage" vattach="middle" width="138.0pt"><p>Algorithm text here.</p></inline-block></p></para>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p4""#],
    r#"<para xml:id="p4"><p><inline-logical-block class="ltx_minipage" vattach="middle" width="138.0pt"><para xml:id="p4.p1"><graphics graphic="first" xml:id="g3"/></para><figure inlist="lof" placement="H" xml:id="S0.F4"><tags><tag>Figure 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Figure 4</tag></tags><graphics graphic="second" xml:id="S0.F4.g1"/><toccaption><tag close=" ">4</tag>Second</toccaption><caption><tag close=": ">Figure 4</tag>Second</caption></figure></inline-logical-block></p></para>"#,
  );
}

/// 57g re-review: float-boxes keep their line's alignment — a `{center}` lone box and row, a
/// `{flushright}` box, a `\noindent\centering` line (arXiv 2605.03276 A2.F8-F10). The settle runs
/// once the document is built, after every alignment has been stamped.
#[test]
fn float_boxes_keep_their_line_alignment() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/float_boxes_keep_their_line_alignment.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, markup) in [
    (
      "S0.F1",
      r#"<figure align="center" class="ltx_minipage" inlist="lof" vattach="middle" width="172.5pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><graphics graphic="x" xml:id="g1"/><toccaption><tag close=" ">1</tag>X</toccaption><caption><tag close=": ">Figure 1</tag>X</caption></figure>"#,
    ),
    (
      "p3",
      r#"<figure align="center" xml:id="p3"><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><graphics graphic="y" xml:id="g2"/><toccaption><tag close=" ">2</tag>Y</toccaption><caption><tag close=": ">Figure 2</tag>Y</caption></figure><figure class="ltx_figure_panel ltx_minipage" inlist="lof" vattach="middle" width="103.5pt" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><graphics graphic="z" xml:id="g3"/><toccaption><tag close=" ">3</tag>Z</toccaption><caption><tag close=": ">Figure 3</tag>Z</caption></figure></figure>"#,
    ),
    (
      "S0.F4",
      r#"<figure align="right" class="ltx_minipage" inlist="lof" vattach="middle" width="172.5pt" xml:id="S0.F4"><tags><tag>Figure 4</tag><tag role="refnum">4</tag><tag role="typerefnum">Figure 4</tag></tags><graphics graphic="w" xml:id="g4"/><toccaption><tag close=" ">4</tag>W</toccaption><caption><tag close=": ">Figure 4</tag>W</caption></figure>"#,
    ),
    (
      "S0.F5",
      r#"<figure align="center" class="ltx_minipage" inlist="lof" vattach="middle" width="172.5pt" xml:id="S0.F5"><tags><tag>Figure 5</tag><tag role="refnum">5</tag><tag role="typerefnum">Figure 5</tag></tags><graphics graphic="v" xml:id="p5.g1"/><toccaption><tag close=" ">5</tag>V</toccaption><caption><tag close=": ">Figure 5</tag>V</caption></figure>"#,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "figure", &[&format!(r#"xml:id="{id}""#)], markup);
  }
}

/// 57g: `\noindent` before a box that is one float: the paragraph `\noindent` opened holds
/// nothing but that box, so it gives way to the float, and the box is its float.
#[test]
fn a_noindent_lone_float_box_stands_beside_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/noindent_lone_float_box_leaves_empty_para.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("ltx_noindent"), "no empty paragraph:\n{xml}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r#"<table class="ltx_minipage" inlist="lot" vattach="middle" width="345.0pt" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Only</toccaption><caption><tag close=": ">Table 1</tag>Only</caption></table>"#,
  );
}

/// User ruling 2026-09-27 (57f): a box whose whole content is one float becomes that float, as
/// Perl's fold makes it (`<figure class="ltx_parbox">`, `<figure class="ltx_minipage">`); the float
/// is a display element a paragraph would only enclose.
#[test]
fn a_box_holding_only_a_float_becomes_it() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/box_holding_only_a_float_becomes_it.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure class="ltx_parbox" inlist="lof" vattach="middle" width="345.0pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>In a parbox</toccaption><caption><tag close=": ">Figure 1</tag>In a parbox</caption></figure>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r#"<figure class="ltx_minipage" inlist="lof" vattach="middle" width="345.0pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">2</tag>In a minipage</toccaption><caption><tag close=": ">Figure 2</tag>In a minipage</caption></figure>"#,
  );
}

/// 57d's review: a float's content is panels, so no paragraph opens for a parbox through the
/// captures of the boxes built in it — minipage panels starting with a `\parbox` stay side by side
/// (arXiv 2605.27134 S5.F8) and captioned parbox panels in a `{center}` stay sub-figures.
#[test]
fn parbox_panels_stay_panels() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/parbox_panels_stay_panels.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><block class="ltx_figure_panel ltx_minipage ltx_parbox" vattach="middle" width="155.3pt"><p>Left panel.</p><p>Its second line.</p></block><block class="ltx_figure_panel ltx_minipage ltx_parbox" vattach="middle" width="155.3pt"><p>Right panel.</p><p>Its second line.</p></block><toccaption><tag close=" ">1</tag>Minipages.</toccaption><caption><tag close=": ">Figure 1</tag>Minipages.</caption></figure>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F3""#],
    r#"<figure inlist="lof" xml:id="S0.F3"><tags><tag>Figure 3</tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><figure align="center" class="ltx_figure_panel ltx_parbox" vattach="middle" width="155.3pt" xml:id="S0.F3.fig1"><graphics graphic="a" xml:id="S0.F3.g1"/><toccaption><tag close=" ">2</tag>A</toccaption><caption><tag close=": ">Figure 2</tag>A</caption></figure><figure align="center" class="ltx_figure_panel ltx_parbox" vattach="middle" width="155.3pt" xml:id="S0.F3.fig2"><graphics graphic="b" xml:id="S0.F3.g2"/><toccaption><tag close=" ">3</tag>B</toccaption><caption><tag close=": ">Figure 3</tag>B</caption></figure></figure>"#,
  );
}

/// 57e: Perl's `isVAttached` (TeX_Box.pool.ltxml:433-440) follows a node's only child down,
/// counting every child node, text included. A parbox whose one paragraph holds an inner parbox
/// and text is not v-attached, so the outer box keeps its own `vattach` (`insert_block`'s
/// one-node hack drops it only for a truly single v-attached child, as for the minipage below).
#[test]
fn a_nested_parbox_keeps_its_vattach() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/nested_parbox_keeps_its_vattach.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p><inline-block class="ltx_parbox" vattach="middle" width="142.3pt"><p><inline-block class="ltx_parbox" vattach="middle" width="56.9pt"><p>Inner</p></inline-block> after inner.</p></inline-block> after outer.</p></para>"#,
  );
  // The drop direction: a minipage holding only the parbox (one v-attached child) loses its own
  // vattach, as Perl's does; since 57f the minipage is its paragraph's inline block.
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p><inline-block class="ltx_minipage" width="142.3pt"><p><inline-block class="ltx_parbox" vattach="top" width="56.9pt"><p>Inner</p></inline-block></p></inline-block></p></para>"#,
  );
}

/// 57d's re-review: the parbox's paragraph opens only for content an inline box holds. A box
/// holding a bibliography keeps the hoist, which places the bibliography after the box's place,
/// and leaves no empty paragraph behind it.
#[test]
fn a_parbox_holding_a_bibliography_opens_no_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/parbox_holding_a_bibliography_opens_no_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r#"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title><tag close=" ">1</tag>S</title><para xml:id="S1.p1"><p>Before.</p></para><bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> Alpha.</bibblock></bibitem></biblist></bibliography><para xml:id="S1.p2"><p>After.</p></para></section>"#,
  );
}

/// 57d's review: the parbox's paragraph is decided after the sectioning float-out
/// (OXIDIZED_DESIGN #250), so a `\section*` in a `\parbox` between paragraphs is still a real
/// section, not a section inside a `<p>`.
#[test]
fn a_parbox_holding_a_section_floats_it_out() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/parbox_holding_a_section_floats_it_out.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="Sx1""#],
    r#"<section class="ltx_parbox" xml:id="Sx1"><title>Skills</title><para xml:id="Sx1.p1"><p>text</p></para><para xml:id="Sx1.p2"><p>After.</p></para></section>"#,
  );
}

/// 57d's gate, from its arXiv A/B (2605.20645's `\rotatebox{90}{\parbox…}` row labels): a
/// `\leavevmode` in restricted horizontal mode begins no paragraph (tex.web §1090-1091), so the
/// parbox is the rotated box's own content, while at the top of a minipage (internal vertical)
/// it starts one — and the minipage keeps its `vattach`, its one paragraph holding more than the
/// box (57e, Perl's `isVAttached`); since 57f the minipage is itself its paragraph's inline block.
#[test]
fn a_parbox_in_a_restricted_box_starts_no_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/parbox_in_a_restricted_box_starts_no_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>A <inline-block angle="90" depth="0.0pt" height="56.9pt" innerdepth="0.0pt" innerheight="6.8pt" innerwidth="56.9pt" width="6.8pt" xtranslate="-25.0pt" ytranslate="-25.0pt"><p class="ltx_parbox" vattach="middle" width="56.9pt">Rot</p></inline-block> B.</p></para>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p><inline-block class="ltx_minipage" vattach="middle" width="142.3pt"><p><inline-block class="ltx_parbox" vattach="middle" width="85.4pt"><p>Q</p></inline-block> after Q.</p></inline-block></p></para>"#,
  );
}

/// 57i (K13 stage-2 finding): the real `\orcidlinkX` is `\href{…}{…}` (orcidlink.sty:69), whose
/// `\hyper@linkurl` begins with `\leavevmode` (hpdftex.def:411): the link starts the paragraph, so
/// the space after it stays (DIVERGENCES #340).
#[test]
fn orcidlink_starts_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/orcidlink_starts_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r###"<para xml:id="p2"><p><ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097"><svg:svg class="ltx_orcidlogo" height="1em" version="1.1" viewBox="0 0 72 72" width="1em"><svg:path d="M72,36 C72,55.884375 55.884375,72 36,72 C16.115625,72 0,55.884375 0,36 C0,16.115625 16.115625,0 36,0 C55.884375,0 72,16.115625 72,36 Z" fill="#A6CE39"/><svg:g fill="#FFFFFF" transform="translate(18.868966, 12.910345)"><svg:polygon points="5.03734929 39.1250878 0.695429861 39.1250878 0.695429861 9.14431787 5.03734929 9.14431787 5.03734929 22.6930505 5.03734929 39.1250878"/><svg:path d="M11.409257,9.14431787 L23.1380784,9.14431787 C34.303014,9.14431787 39.2088191,17.0664074 39.2088191,24.1486995 C39.2088191,31.846843 33.1470485,39.1530811 23.1944669,39.1530811 L11.409257,39.1530811 L11.409257,9.14431787 Z M15.7511765,35.2620194 L22.6587756,35.2620194 C32.49858,35.2620194 34.7541226,27.8438084 34.7541226,24.1486995 C34.7541226,18.1301509 30.8915059,13.0353795 22.4332213,13.0353795 L15.7511765,13.0353795 L15.7511765,35.2620194 Z"/><svg:path d="M5.71401206,2.90182329 C5.71401206,4.441452 4.44526937,5.72914146 2.86638958,5.72914146 C1.28750978,5.72914146 0.0187670918,4.441452 0.0187670918,2.90182329 C0.0187670918,1.33420133 1.28750978,0.0745051096 2.86638958,0.0745051096 C4.44526937,0.0745051096 5.71401206,1.36219458 5.71401206,2.90182329 Z"/></svg:g></svg:svg></ref> text after.</p></para>"###,
  );
}

/// 57i (K13 stage-2 finding): `\externaldocument[prefix][nocite]{file}[url]` reads its trailing
/// `[url]` (xr.sty:47, xr-hyper.sty:41); it was left as text. The binding announces its stub (one
/// warning, `xr.sty is not implemented`).
#[test]
fn xr_externaldocument_reads_its_url() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/xr_externaldocument_reads_its_url.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Warning:missing_file:xr.sty"), "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>See the supplement.</p></para>"#,
  );
  // siamart loads xr-hyper (siamart220329.cls:1266); its binding's copy reads the same `[url]`.
  let (stderr, xml) = convert_with(
    "\\documentclass{siamart}\n\\externaldocument[][nocite]{ex_supplement}[https://example.org/ex_supp.pdf]\n\\begin{document}\nSee the supplement.\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>See the supplement.</p></para>"#,
  );
}

/// 57i (K13 stage-2 finding): physics's `\xmatrix*` subscripts each entry by its one-based index
/// (physics.sty:636 `s m m m`; Perl physics.sty.ltxml:600-615); the port had dropped the star. A
/// 1×1 matrix has no index (physics.sty:647; Perl's `x_1`, KPE #314). The whole start tag: the
/// source and the parsed structure of the display.
#[test]
fn xmatrix_star_subscripts_entries() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/xmatrix_star_subscripts_entries.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let math = latexml::util::test::xml_element(&xml, "Math", &[r#"mode="display""#])
    .unwrap_or_else(|| panic!("{xml}"));
  assert!(
    math.starts_with(r#"<Math mode="display" tex="\begin{pmatrix}a_{11}&amp;a_{12}\\&#10;a_{21}&amp;a_{22}\end{pmatrix}\quad\begin{pmatrix}b_{1}&amp;b_{2}&amp;b_{3}\end{pmatrix}\quad\begin{pmatrix}c\\&#10;c\end{pmatrix}\quad\begin{pmatrix}x\end{pmatrix}" text="fragments@(matrix@(Array[[a _ (list@(1, 1)), a _ (list@(1, 2))], [a _ (list@(2, 1)), a _ (list@(2, 2))]]), matrix@(Array[[b _ 1, b _ 2, b _ 3]]), matrix@(Array[[c], [c]]), matrix@(Array[[x]]))" xml:id="S0.Ex1.m1">"#),
    "{math}"
  );
}

/// 57i (review side finding): physics's `\zeromatrix` is `m g` (physics.sty:662), the column count
/// optional and defaulting to the row count; `{}{}` (Perl physics.sty.ltxml:619, KPE #314) took
/// `\end` as the second size of `\zmat{3}`.
#[test]
fn zeromatrix_defaults_its_columns() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/zeromatrix_defaults_its_columns.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let math = latexml::util::test::xml_element(&xml, "Math", &[r#"mode="display""#])
    .unwrap_or_else(|| panic!("{xml}"));
  assert!(math.starts_with(r#"<Math mode="display" tex="\begin{pmatrix}0&amp;0&amp;0\\&#10;0&amp;0&amp;0\\&#10;0&amp;0&amp;0\end{pmatrix}\quad\begin{pmatrix}0&amp;0\end{pmatrix}" text="fragments@(matrix@(Array[[0, 0, 0], [0, 0, 0], [0, 0, 0]]), matrix@(Array[[0, 0]]))" xml:id="S0.Ex1.m1">"#), "{math}");
}

/// 57i (K13 stage-2 finding): fontawesome's `\faBattery[1][4]` and `\faHourglass[1][]`
/// (fontawesome.sty:67-70) read their optional; it was left as text.
#[test]
fn fontawesome_icons_read_their_optional() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/fontawesome_icons_read_their_optional.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Charge <inline-block aria:hidden="true" class="fa fa-battery-2"/> and <inline-block aria:hidden="true" class="fa fa-battery-4"/> full, <inline-block aria:hidden="true" class="fa fa-hourglass-half"/> and <inline-block aria:hidden="true" class="fa fa-hourglass"/> now.</p></para>"#,
  );
}

/// 57i (K13 stage-2 finding): arydshln's `\ADLdrawingmode` reads its mode number
/// (`\def\ADLdrawingmode#1`, arydshln.sty:677); the digit was printed. The binding announces its
/// stub (one warning).
#[test]
fn adldrawingmode_reads_its_mode() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/adldrawingmode_reads_its_mode.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:missing_file:arydshln.sty"),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Text.</p></para>"#,
  );
}

/// 57j (K13 stage-2 finding): `{threeparttable}` and `{measuredfigure}` read their optional
/// placement (threeparttable.sty:107,122 `[1][t]`), which is never typeset; `[b]` was a text panel
/// (Perl shares it, KPE #315; witness 2605.04144).
#[test]
fn threeparttable_reads_its_placement() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/threeparttable_reads_its_placement.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r##"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Table 1</tag>Cap</caption><tabular vattach="middle"><tbody><tr><td align="center">x</td></tr></tbody></tabular></table>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Z</p></para>"##,
  );
}

/// 57j (K13 stage-2 finding): `\TPToverlap` gobbles its note outside `{threeparttable}`
/// (threeparttable.sty:281) and is `\relax` inside it (:118); a `\tnote` in a plain tabular printed
/// (Perl shares it, KPE #315; witnesses 2605.26854, 2605.23257).
#[test]
fn tnote_outside_threeparttable_prints_nothing() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/tnote_outside_threeparttable_prints_nothing.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><tabular vattach="middle"><tbody><tr><td align="center">4000</td></tr></tbody></tabular></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>AC</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><tabular vattach="middle"><tbody><tr><td align="center">x<sup>a</sup></td></tr></tbody></tabular></para>"##,
  );
}

/// 57j (K13 stage-2 finding): subfig's `\clearcaptionsetup*[option]{type}` (caption3.sty:275-280)
/// and `\listsubcaptions*` (subfig.sty:476-479) read their star and option (KPE #316).
#[test]
fn clearcaptionsetup_reads_its_options() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/clearcaptionsetup_reads_its_options.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure xml:id="S0.F1.sf1"><tags><tag>(a)</tag><tag role="refnum">1a</tag></tags><p>x</p><toccaption><tag close=" ">a</tag>A</toccaption><caption><tag close=" ">(a)</tag>A</caption></figure><toccaption><tag close=" ">1</tag>C</toccaption><caption><tag close=": ">Figure 1</tag>C</caption></figure>"##,
  );
}

/// 57j (K13 stage-2 finding): caption's `\clearcaptionsetup` reads `*[option]{type}`
/// (caption3.sty:275-280); the type was left as text (KPE #316).
#[test]
fn caption_clearcaptionsetup_reads_its_type() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/caption_clearcaptionsetup_reads_its_type.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Body.</p></para>"##,
  );
}

/// 57j (K13 stage-2 finding): titlesec's `\titleline*[align]{material}` (titlesec.sty:1088-1095);
/// the star was read as the material and `[c]` left in the title (KPE #317).
#[test]
fn titleline_reads_its_star_and_alignment() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titleline_reads_its_star_and_alignment.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title fontsize="120%">1   Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="S1.p1"><p>Body text.</p></para></section>"##,
  );
}

/// 57j (K13 stage-2 finding): `\iftitlemeasuring` is `\@secondoftwo` (titlesec.sty:1047); as a TeX
/// conditional it skipped the title to a missing `\fi` (KPE #317).
#[test]
fn iftitlemeasuring_takes_the_second_branch() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/iftitlemeasuring_takes_the_second_branch.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title font="bold">1  Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="S1.p1"><p>Body text.</p></para></section>"##,
  );
}

/// 57j (K13 stage-2 finding): titlesec's `\wordsep` is the font's interword glue
/// (titlesec.sty:1164-1165), not a 0pt register (KPE #317).
#[test]
fn wordsep_is_an_interword_space() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/wordsep_is_an_interword_space.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title class="ltx_runin" font="bold">1 Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="S1.p1"><p>Body text.</p></para></section>"##,
  );
}

/// 57j (K13 stage-2 finding): revsymb's `\biglb` … `\Biggrb` set the delimiter bold in the big sizes
/// (revsymb4-2.sty:136-154; Perl revsymb.sty.ltxml:31-54); the port's `\mathopen\big` macros
/// dropped the bold. The paragraph as Perl's.
#[test]
fn revsymb_delimiters_are_bold() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/math-parse/revsymb_delimiters_are_bold.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p><Math mode="inline" tex="\biglb(x\bigrb)" text="x" xml:id="p1.m1"><XMath><XMDual><XMRef idref="p1.m1.1"/><XMWrap><XMTok font="bold" fontsize="120%" role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok><XMTok font="bold" fontsize="120%" role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMath></Math> and <Math mode="inline" tex="\Biglb[y\Bigrb]" text="delimited-[]@(y)" xml:id="p1.m2"><XMath><XMDual><XMApp><XMTok meaning="delimited-[]"/><XMRef idref="p1.m2.1"/></XMApp><XMWrap><XMTok font="bold" fontsize="160%" role="OPEN" stretchy="false">[</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.1">y</XMTok><XMTok font="bold" fontsize="160%" role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual></XMath></Math></p></para>"##,
  );
}

/// 57k (K13 finding, kernel): a `DefPrimitive!` with a literal replacement is a string body, which
/// enters horizontal mode as Perl's `Primitive::invoke` does for a defined string (Primitive.pm:73);
/// compiled to a closure it did not, so the space after `\ataribox{}` and `\dag{}` was lost and
/// `\cent` left TeX in vertical mode (witness 2605.01273, `\copyright{} Author(s)`).
#[test]
fn literal_primitive_enters_horizontal() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/literal_primitive_enters_horizontal.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p><text backgroundcolor="#000000" color="#FFFFFF">⛋</text> text after the box.<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>† Equal contribution.</note></p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>¢ x.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><p>[a:V][b:H]</p></para>"##,
  );
}

/// 57k (review): the paragraph a glyph primitive starts begins before its `bounded` group and
/// `font`, as LaTeX's `\UseTextSymbol` (`\hmode@start@before@group`, latex.ltx:9967), so the
/// `\everypar` it fires runs in the paragraph's font: entered inside the group, `\textdagger`'s TS1
/// font lost the hook's "E" silently and `\ataribox` set it white on black.
#[test]
fn everypar_precedes_a_glyphs_group() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/everypar_precedes_a_glyphs_group.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p><text font="bold">E</text>† a.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p><text font="bold">E</text><text backgroundcolor="#000000" color="#FFFFFF">⛋</text> b.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><p><text font="bold">E</text>† c.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p4""#],
    r##"<para xml:id="p4"><p><text font="bold">E</text><text backgroundcolor="#000000" color="#FFFFFF">⛋</text> d.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p5""#],
    r##"<para xml:id="p5"><p><text font="bold">E</text>¢ e.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p6""#],
    r##"<para xml:id="p6"><p><text font="bold">E</text>© f.</p></para>"##,
  );
}

/// 57n: a parameterless DefMath whose options are Perl's simple ones — `stretchy`, `alias`
/// included (`$simpletoken_options`, Package.pm:1603-1607) — is a box taking the digestion font
/// (Package.pm:1665, 1815-1838), so `\big\langle`, `\bigl\{` keep their size and revsymb's
/// `\biglb\langle` its bold, as Perl; the box reverts to its alias (`\{`, Package.pm:1831).
#[test]
fn big_delimiter_keeps_its_size() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/big_delimiter_keeps_its_size.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for element in [
    r#"<XMTok fontsize="120%" name="langle" role="OPEN" stretchy="false">⟨</XMTok>"#,
    r#"<XMTok fontsize="120%" name="rangle" role="CLOSE" stretchy="false">⟩</XMTok>"#,
    r#"<XMTok fontsize="120%" role="OPEN" stretchy="false">{</XMTok>"#,
    r#"<XMTok fontsize="120%" role="CLOSE" stretchy="false">}</XMTok>"#,
    r#"<XMTok font="bold" fontsize="120%" name="langle" role="OPEN" stretchy="false">⟨</XMTok>"#,
    r#"<XMTok font="bold" fontsize="120%" name="rangle" role="CLOSE" stretchy="false">⟩</XMTok>"#,
    r#"tex="\bigl\{b\bigr\}""#,
  ] {
    assert!(xml.contains(element), "missing {element}\n{xml}");
  }
  assert!(
    !xml.contains(r#"<XMTok name="langle" role="OPEN" stretchy="false">⟨</XMTok>"#),
    "an unsized ⟨\n{xml}"
  );
}

/// 57n: every DefMath constructor requires math, as Perl's (`requireMath`, Package.pm:1706),
/// warning under the command's own name (`unexpected:\binom`, Package.pm:1069-1073); the gate on
/// `require_math` hid it. A text-mode `\rightarrowfill`, a box, stays silent.
#[test]
fn text_mode_defmath_constructor_is_reported() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/text_mode_defmath_constructor_is_reported.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_eq!(
    stderr
      .matches(r"Warning:unexpected:\binom \binom should only appear in math mode")
      .count(),
    1,
    "{stderr}"
  );
  assert!(
    xml.contains(r#"<XMTok meaning="binomial" name="binom" role="UNKNOWN"/>"#),
    "{xml}"
  );
}

/// 57n: `\hphantom` opens with `\protect`, as Perl's `\protect\ifmmode` (math_common.pool.ltxml:655),
/// so the look-ahead at an alignment row's start (tex.web §785) stops there and `\ifmmode` is read
/// after the template's `$`: the phantom stays math spacing, its argument math (witness 2605.21158).
#[test]
fn hphantom_at_an_align_row_start_stays_in_math() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/hphantom_at_an_align_row_start_stays_in_math.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for element in [
    r#"<Math tex="\displaystyle\hphantom{(\hat{x})}y=1." text="y = 1" xml:id="S0.E2.m3">"#,
    r#"<XMTok font="italic" lpadding="13.3pt" role="UNKNOWN">y</XMTok>"#,
  ] {
    assert!(xml.contains(element), "missing {element}\n{xml}");
  }
  assert!(
    !xml.contains("ltx_phantom"),
    "a text phantom in math\n{xml}"
  );
}

/// 57l: changepage's `{adjustwidth}` is a list (changepage.sty:110), whose end closes the paragraph
/// (`\endtrivlist`, latex.ltx:15915-15926); the text after it is a new `<p>` of the same logical
/// paragraph (`\@doendpe`, :15939). The transparent binding left the body's `<p>` open.
#[test]
fn adjustwidth_ends_its_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/adjustwidth_ends_its_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before text</p><p>Inner text.</p><p>After text.</p></para>"##,
  );
}

/// 57l: `{adjustwidth}`'s margins are read, not digested: a digested `-0.005\linewidth` assigned
/// `\linewidth` = 0pt inside (witness 2605.02723's zero-width figure).
#[test]
fn adjustwidth_margins_are_read_not_typeset() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/adjustwidth_margins_are_read_not_typeset.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>A</p><rule height="1.0pt" width="345.0pt"/><p>B</p><p>C</p></para>"##,
  );
}

/// 57l: changepage's page checks are the package's own TeX (changepage.sty:22, 29-32, 59-67; `\cp@tempcnt`
/// read by dgruyter.sty), loaded raw under the binding; `{adjustwidth*}` (:122; witness 2006.09676) and
/// `$$` inside (witness 2305.09826) keep working.
#[test]
fn changepage_page_checks_are_the_packages() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/changepage_page_checks_are_the_packages.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>EVEN; odd; 1.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>Star.</p><p>Tail.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><p>Wide</p><equation xml:id="S0.Ex1"><Math mode="display" tex="\log_{2}(x)" text="(logarithm _ 2)@(x)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMDual><XMRef idref="S0.Ex1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math></equation><p>text.</p><p>After.</p></para>"##,
  );
}

/// 57l: under memoir, changepage's page macros are memoir's (memoir.cls:12216 `\EmulatedPackage`;
/// changepage.sty:8-11 stops) — its `\checkoddpage` survives — while its list `{adjustwidth}`
/// (memoir.cls:11267-11297) gives way to the transparent one. memoir's `\@iffirstamp` warning stays.
#[test]
fn changepage_under_memoir_keeps_memoirs() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/changepage_under_memoir_keeps_memoirs.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains(r"Warning:misdefined:\@iffirstamp"),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before text</p><p>Inner text.</p><p>After text.
ODD</p><pagination role="newpage"/></para>"##,
  );
}

/// 57l: chngpage is its own package (chngpage.sty:21-54, `{adjustwidth}[3][\@empty]` :113), not an
/// alias of changepage's binding.
#[test]
fn chngpage_is_its_own_package() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/chngpage_is_its_own_package.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>even; S; 0; 1.</p><p>Wide text.</p><p>After.</p></para>"##,
  );
}

/// 57l: setspace's environments end with `\par` (setspace.sty:489-548); `{spacing}` and `{singlespace}` also
/// begin one, `{onehalfspace}`/`{doublespace}` go on in the current paragraph.
#[test]
fn setspace_environments_end_their_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/setspace_environments_end_their_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before</p><p>Inner.</p><p>After.

Inner2.</p><p>After2.</p></para>"##,
  );
}

/// 57l: subfig loads caption (subfig.sty:124-142), so caption's `\captionsetup` stays: a typed caption in a
/// minipage is its figure, and `\captionsetup*` prints nothing (Perl's stub, subfig.sty.ltxml:107).
#[test]
fn subfig_keeps_captions_captionsetup() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfig_keeps_captions_captionsetup.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p><inline-logical-block class="ltx_minipage" vattach="middle" width="138.0pt"><figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><toccaption><tag close=" ">1</tag>Typed</toccaption><caption><tag close=": ">Figure 1</tag>Typed</caption></figure></inline-logical-block>

Body.</p></para>"##,
  );
}

/// 57l: with subfig alone, caption's `\caption*` and subfig's keys (subfig.sty:163-167, 271-282) are defined.
#[test]
fn subfig_loads_caption() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/captions-floats/subfig_loads_caption.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="tab1""#],
    r##"<table xml:id="tab1"><caption>Unnumbered</caption></table>"##,
  );
}

/// 57l: caption keeps rotating's `\rotcaption` (caption.sty:1284 redefines it only with rotating; Perl's
/// no-op, caption.sty.ltxml:131, dropped the caption).
#[test]
fn rotcaption_is_a_caption() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/rotcaption_is_a_caption.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r##"<table angle="90" depth="0.0pt" height="550.0pt" inlist="lot" innerdepth="77.1pt" innerheight="8.5pt" innerwidth="550.0pt" width="85.6pt" xtranslate="-232.2pt" ytranslate="-232.2pt" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><tabular vattach="middle"><tbody><tr><td align="center">x</td></tr></tbody></tabular><toccaption><tag close=" ">1</tag>Rotated caption</toccaption><caption><tag close=": ">Table 1</tag><inline-block angle="90" depth="0.0pt" height="70.7pt" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="70.7pt" width="8.9pt" xtranslate="-30.9pt" ytranslate="-30.9pt"><p>Rotated caption</p></inline-block></caption></table>"##,
  );
}

/// 57l: `\rotcaption[short]{long}` (rotating.sty:260-270, `\@dblarg`); `{}` alone left `[Short]` as text.
#[test]
fn rotcaption_reads_its_short_form() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/rotcaption_reads_its_short_form.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><rule height="56.9pt" width="28.5pt"/><toccaption><tag close=" ">1</tag>Short</toccaption><caption><tag close=": ">Figure 1</tag><inline-block angle="90" depth="0.0pt" height="53.9pt" innerdepth="1.9pt" innerheight="6.9pt" innerwidth="53.9pt" width="8.9pt" xtranslate="-22.5pt" ytranslate="-22.5pt"><p>Side caption</p></inline-block></caption></figure>"##,
  );
}

/// 57l: setspace's `{doublespace}`/`{onehalfspace}` go on in the paragraph before them and
/// `{singlespace}` ends it (setspace.sty:489-548); all end theirs — begun mid-paragraph and in vertical
/// mode, pdflatex's paragraphs.
#[test]
fn setspace_environments_begin_as_setspace_does() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/setspace_environments_begin_as_setspace_does.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before

Inner2.</p><p>After2.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>Own para.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><p>Second.</p><p>Tail.
Mid</p><p>Single.</p><p>End.</p></para>"##,
  );
}

/// 57l: `\changetext` keeps LaTeXML's one column: `\ch@ngetext` (changepage.sty:80-88) no longer halves
/// `\columnwidth`, so `\linewidth` follows the changed `\textwidth`.
#[test]
fn changetext_keeps_one_column() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/changetext_keeps_one_column.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Width 373.45274pt and 373.45274pt.</p></para>"##,
  );
}

/// 57l: subcaption's `\subfloat` reads a lone optional as its caption and follows `\@captype`
/// (subcaption.sty:278-291; Perl's `\subfloat[][]{}`, subcaption.sty.ltxml:104, took the caption from the
/// second optional, KPE #323), and a list entry stays on its sub-float: the enclosing figure keeps
/// `inlist="lof"` (Perl's `^ inlist='#1'` floats from the current node, Document.pm:1080-1092). The
/// sub-float's `inlist="List"` — the list-entry text as a list name — is Perl's quirk (KPE #323
/// residual); a fix that makes the entry the `toccaption` updates it knowingly.
#[test]
fn subfloat_reads_a_lone_optional_as_its_caption() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfloat_reads_a_lone_optional_as_its_caption.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags><p>A</p><toccaption><tag close=" ">a</tag>One</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">One</text></caption></figure><break class="ltx_break"/><figure class="ltx_figure_panel" inlist="List" xml:id="S0.F1.sf2"><tags><tag><text fontsize="90%">(b)</text></tag><tag role="refnum">1b</tag></tags><p>B</p><toccaption><tag close=" ">b</tag>Two</toccaption><caption><tag close=" "><text fontsize="90%">(b)</text></tag><text fontsize="90%">Two</text></caption></figure><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r##"<table inlist="lot" xml:id="S0.T1"><tags><tag><text fontsize="90%">Table 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><table inlist="lot" xml:id="S0.T1.st1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags><p>X</p><toccaption><tag close=" ">a</tag>Tab one</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Tab one</text></caption></table><toccaption><tag close=" ">1</tag>Tabs</toccaption><caption><tag close=": "><text fontsize="90%">Table 1</text></tag><text fontsize="90%">Tabs</text></caption></table>"##,
  );
}

/// 57l: memoir's own `\changetext` keeps LaTeXML's one column: memoir's `\ch@ngetext`
/// (memoir.cls:11237-11245) halved `\columnwidth` under `[twocolumn]`; the memoir binding installs the
/// shared one-column override after the raw class.
#[test]
fn changetext_under_memoir_keeps_one_column() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/changetext_under_memoir_keeps_one_column.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains(r"Warning:misdefined:\@iffirstamp"),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Width 497.45274pt and 497.45274pt.</p><pagination role="newpage"/></para>"##,
  );
}

/// 57l: a `\subfloat` inside a `{subfigure}` is a subfigure: the sub-float type drops a leading `sub`
/// from `\@captype` (Perl's `\subcaption`, subcaption.sty.ltxml:50-53) and falls back to `figure`, so
/// no `{subsubfigure}` is opened. The ids read figure 2 where pdflatex numbers 1: the nested subfigure
/// pre-increments the figure counter a second time, as Perl's `beforeFloat` does (KPE #324).
#[test]
fn subfloat_in_a_subfigure_is_a_subfigure() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfloat_in_a_subfigure_is_a_subfigure.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2.sf2""#],
    r##"<figure inlist="lof" xml:id="S0.F2.sf2"><tags><tag><text fontsize="90%">(b)</text></tag><tag role="refnum">2b</tag></tags><figure inlist="lof" xml:id="S0.F2.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">2a</tag></tags><p>Nbody</p><toccaption><tag close=" ">a</tag>Nested</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Nested</text></caption></figure><toccaption><tag close=" ">b</tag>Outer sub</toccaption><caption><tag close=" "><text fontsize="90%">(b)</text></tag><text fontsize="90%">Outer sub</text></caption></figure>"##,
  );
}

/// 57m: apacite's macros print their own text (apacite.sty:1496-1499, 1378-1477, 2068-2156): the
/// binding read arguments the real macros do not (`\APACexlab{}{}`, `\APACciteatitle{}{}`) and
/// dropped strings (`\bibcomputersoftwaremanual`, `\APACorigED`), "XY. Z more. W tail." in 57i.
#[test]
fn apacite_macros_keep_their_text() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/apacite_macros_keep_their_text.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>X“Title”Y.
Z <text font="italic">Book</text> more.
W a tail.
Vol. 7, pp. 2–11, p. 5; [Computer software manual]. End.
(Original work published 1901; A note) and by E. D. Itor, Ed. ok.
QR.</p></para>"##,
  );
}

/// 57m: an apacite.bst `.bbl` formats as apacite.sty's reference-list layer does (:1243-1522,
/// 2068-2193, run verbatim): `\APACrefYearMonthDay`, `\APACjournalVolNumPages`, `{APACrefURL}[date]`,
/// `\APACorigbooknote`, `\BDBL`, `\BCntIP`, `\bibnodate`, `\nocitemeta`'s asterisk; the label's
/// `\APACyear{2001}\APACexlab{\BCnt{1}}` is the year "2001a". The `.bbl` is bibtex + apacite.bst
/// v6.03 output for the `.bib` beside the repro (0/19 entries as pdflatex's in 57k, 19/19 now).
#[test]
fn apacite_bbl_formats_like_pdflatex() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/apacite_bbl_formats_like_pdflatex.tex"
  );
  let bbl = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/apacite_bbl_formats_like_pdflatex.bbl"
  );
  let (stderr, xml) = convert_files_with(tex, &[("t.bbl", bbl)], None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<bibitem ").count(), 19, "{xml}");
  for (key, bibitem) in [
    (
      "smith2001",
      r##"<bibitem key="smith2001" xml:id="bib.bib16"><tags><tag role="number">16</tag><tag role="year">2001b</tag><tag role="authors">Smith &amp; Doe</tag><tag role="fullauthors">Smith &amp; Doe</tag><tag role="refnum">Smith &amp; Doe (2001b)</tag><tag role="key">smith2001</tag></tags><bibblock>
Smith, J. K., &amp; Doe, J. </bibblock><bibblock>(2001b).
</bibblock><bibblock>A study of things.
</bibblock><bibblock><emph font="italic">Journal of Studies</emph>, <emph font="italic">12</emph>(3), 45–67.

</bibblock></bibitem>"##,
    ),
    (
      "gray2010",
      r##"<bibitem key="gray2010" xml:id="bib.bib8"><tags><tag role="number">8</tag><tag role="year">2010</tag><tag role="authors">Gray</tag><tag role="fullauthors">Gray</tag><tag role="refnum">Gray (2010)</tag><tag role="key">gray2010</tag></tags><bibblock><sup>∗</sup>Gray, E. </bibblock><bibblock>(2010).
</bibblock><bibblock><emph font="italic">Report on reports</emph> (Tech. Rep. No. 42).
</bibblock><bibblock>Boston: Institute of Things.

</bibblock></bibitem>"##,
    ),
    (
      "many2020",
      r##"<bibitem key="many2020" xml:id="bib.bib14"><tags><tag role="number">14</tag><tag role="year">2020</tag><tag role="authors">One et al.</tag><tag role="fullauthors">One et al.</tag><tag role="refnum">One et al. (2020)</tag><tag role="key">many2020</tag></tags><bibblock>
One, A., Two, B., Three, C., Four, D., Five, E., Six, F., … Eight, H. </bibblock><bibblock>(2020).
</bibblock><bibblock>Many authors.
</bibblock><bibblock><emph font="italic">Big Science</emph>, <emph font="italic">1</emph>, 5.
</bibblock><bibblock>doi: 10.1000/xyz123 

</bibblock></bibitem>"##,
    ),
    (
      "web2015",
      r##"<bibitem key="web2015" xml:id="bib.bib19"><tags><tag role="number">19</tag><tag role="year">2015</tag><tag role="authors">Webber</tag><tag role="fullauthors">Webber</tag><tag role="refnum">Webber (2015)</tag><tag role="key">web2015</tag></tags><bibblock>
Webber, G. </bibblock><bibblock>(2015).
</bibblock><bibblock><emph font="italic">A web page.</emph></bibblock><bibblock>Retrieved March 3, 2016, from <ref class="ltx_nolink ltx_url" font="typewriter" href="http://example.com/page">http://example.com/page</ref></bibblock></bibitem>"##,
    ),
    (
      "reprintb1998",
      r##"<bibitem key="reprintb1998" xml:id="bib.bib9"><tags><tag role="number">9</tag><tag role="year">1890/1998</tag><tag role="authors">James</tag><tag role="fullauthors">James</tag><tag role="refnum">James (1890/1998)</tag><tag role="key">reprintb1998</tag></tags><bibblock>
James, W. </bibblock><bibblock>(1998).
</bibblock><bibblock><emph font="italic">Principles</emph>.
</bibblock><bibblock>New York: Dover.
</bibblock><bibblock>(Reprinted from <emph font="italic">The principles of psychology</emph>, Vol. 1, pp. 5–9, by E. Editor, Ed., 1890, Boston: Holt)

</bibblock></bibitem>"##,
    ),
    (
      "old1950",
      r##"<bibitem key="old1950" xml:id="bib.bib13"><tags><tag role="number">13</tag><tag role="year">1901/1950</tag><tag role="authors">Oldman</tag><tag role="fullauthors">Oldman</tag><tag role="refnum">Oldman (1901/1950)</tag><tag role="key">old1950</tag></tags><bibblock>
Oldman, I. </bibblock><bibblock>(1950).
</bibblock><bibblock><emph font="italic">Old classic</emph>.
</bibblock><bibblock>Berlin: Verlag.
</bibblock><bibblock>(Original work published 1901; A note)

</bibblock></bibitem>"##,
    ),
    (
      "blue2012",
      r##"<bibitem key="blue2012" xml:id="bib.bib4"><tags><tag role="number">4</tag><tag role="year">2012</tag><tag role="authors">Blue</tag><tag role="fullauthors">Blue</tag><tag role="refnum">Blue (2012)</tag><tag role="key">blue2012</tag></tags><bibblock>
Blue, F. </bibblock><bibblock>(2012).
 </bibblock><bibblock><emph font="italic">A dissertation</emph> (Unpublished doctoral dissertation).
 </bibblock><bibblock>University of Somewhere, Paris.

</bibblock></bibitem>"##,
    ),
    (
      "kim-ipa",
      r##"<bibitem key="kim-ipa" xml:id="bib.bib10"><tags><tag role="number">10</tag><tag role="year">in press-a</tag><tag role="authors">Kim</tag><tag role="fullauthors">Kim</tag><tag role="refnum">Kim (in press-a)</tag><tag role="key">kim-ipa</tag></tags><bibblock>
Kim, L. </bibblock><bibblock>(in press-a).
</bibblock><bibblock>First forthcoming
paper.
</bibblock><bibblock><emph font="italic">Future Journal</emph>.

</bibblock></bibitem>"##,
    ),
  ] {
    let selector = format!(r#"key="{key}""#);
    latexml::util::test::assert_element(&xml, "bibitem", &[selector.as_str()], bibitem);
  }
}

/// 57m: apacite's `\Bem` serves both `.bbl` generations: `\Bem{X}` is apacite.sty:1519's `\emph`, the
/// pre-2012 declaration `{\Bem X}` italicizes its group (theapa.sty:69 `\let\Bem\itshape`) where
/// `\emph` would take one letter (OXIDIZED_DESIGN_DIVERGENCES #342). The old binding's `\em`
/// toggled: "Command b" italic, "Declared words" upright.
#[test]
fn apacite_bem_reads_both_forms() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/index-bib/apacite_bem_reads_both_forms.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>A <emph font="italic">Command</emph> b <text font="italic">Declared words</text> c <emph font="italic"/> e.</p></para>"##,
  );
}

/// 57m: an old-format apacite `.bbl` (`\BCAY` label, `{\Bem …}` declarations, `\BBACOMMA`) keeps its
/// whole italic journal and volume (fixture of `06_cluster_bibliography::cluster_apacite_old_bbl_format_renders`).
#[test]
fn apacite_old_bbl_keeps_its_declared_italics() {
  let (stderr, xml) = convert_with(
    include_str!("../cluster_regressions/apacite_old_bbl.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "bibitem",
    &[r#"key="smith2020""#],
    r##"<bibitem key="smith2020" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="authors">Smith</tag><tag role="fullauthors">Smith</tag><tag role="refnum">(1)</tag><tag role="key">smith2020</tag></tags><bibblock>
Smith, J., &amp; Doe, J. (2020). A study of things. <text font="italic">Journal of Testing</text>, <text font="italic">12</text>, 1–20.
</bibblock></bibitem>"##,
  );
}

/// 57m: apacite's `\doi` reads the DOI as `\Url` does (apacite.sty:1812-1816): `_`, `~`, `^`, `<…>`
/// literal and in the ASCII encoding, not OT1's accents (`a˙b˜cˆd`, `¡…¿` with 57l's Semiverbatim).
#[test]
fn apacite_doi_reads_as_a_url() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/index-bib/apacite_doi_reads_as_a_url.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "bibitem",
    &[r#"key="a""#],
    r##"<bibitem key="a" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2001</tag><tag role="authors">A</tag><tag role="fullauthors">A</tag><tag role="refnum">A (2001)</tag><tag role="key">a</tag></tags><bibblock>
A, B. </bibblock><bibblock>(2001).
</bibblock><bibblock>doi: 10.1000/a_b~c^d 
</bibblock></bibitem>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "bibitem",
    &[r#"key="c""#],
    r##"<bibitem key="c" xml:id="bib.bib2"><tags><tag role="number">2</tag><tag role="year">2002</tag><tag role="authors">C</tag><tag role="fullauthors">C</tag><tag role="refnum">C (2002)</tag><tag role="key">c</tag></tags><bibblock>
C, D. </bibblock><bibblock>(2002).
</bibblock><bibblock>doi: 10.1175/1520-0485(2002)032&lt;0543:TROBFI&gt;2.0.CO;2 
</bibblock></bibitem>"##,
  );
}

/// 57m: OmniBus's `\doi` reads its DOI as a url (`HyperVerbatim`), so the `.bbl` of an apa6/apa7 paper —
/// classes with no `\doi` of their own, falling to OmniBus — keeps `\doi{10.1000/j_x~y.2002}` whole
/// (Perl OmniBus.cls.ltxml:154's `\doi{}` gives Error:unexpected:_).
#[test]
fn omnibus_doi_reads_as_a_url() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/index-bib/omnibus_doi_reads_as_a_url.tex"),
    None,
  );
  // OmniBus loads xparse raw; a host whose expl3 is not the dump's reports it (the dual-TL schism:
  // "Mismatched LaTeX support files", "Cannot run piped system commands") — any apa7 document does,
  // with or without a DOI; a host whose tree matches the dump (the vendor TL, CI) reports neither.
  // Every other error, `Error:unexpected:_` first, is the DOI's.
  let foreign = stderr
    .lines()
    .filter(|l| l.contains("Error:") || l.contains("Fatal:"))
    .filter(|l| {
      !(l.contains("Mismatched LaTeX support files")
        || l.contains("Cannot run piped system commands"))
    })
    .count();
  assert_eq!(foreign, 0, "{stderr}");
  assert!(stderr.contains("Warning:missing_file:apa7"), "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "bibitem",
    &[r#"key="c""#],
    r##"<bibitem key="c" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2002</tag><tag role="authors">C</tag><tag role="fullauthors">C</tag><tag role="refnum">C (2002)</tag><tag role="key">c</tag></tags><bibblock>
C, D. </bibblock><bibblock>(2002).
</bibblock><bibblock>doi: <ref href="https://doi.org/10.1000/j_x~y.2002">10.1000/j_x~y.2002</ref> 
</bibblock></bibitem>"##,
  );
}

/// 57m: OmniBus's frontmatter `\doi` (preamble → `\lx@add@pubnote[role=doi]`) keeps the DOI's own
/// characters: read as a url, they are set in the ASCII encoding, not OT1's accents (`pre˙a˜b¡c¿`).
#[test]
fn omnibus_frontmatter_doi_keeps_its_characters() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/index-bib/omnibus_frontmatter_doi_keeps_its_characters.tex"
    ),
    None,
  );
  // As `omnibus_doi_reads_as_a_url`: only the host's expl3 mismatch lines are tolerated.
  let foreign = stderr
    .lines()
    .filter(|l| l.contains("Error:") || l.contains("Fatal:"))
    .filter(|l| {
      !(l.contains("Mismatched LaTeX support files")
        || l.contains("Cannot run piped system commands"))
    })
    .count();
  assert_eq!(foreign, 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "pubnote",
    &[r#"role="doi""#],
    r##"<pubnote name="DOI: " role="doi">10.1000/pre_a~b&lt;c&gt;</pubnote>"##,
  );
}

/// 57m: a `\doi` with no braced argument reads one token, as the url-less `\doi[1]`
/// (apacite.sty:1814) does; `HyperVerbatim` would scan ahead to the next `{` and lose the text.
#[test]
fn bare_doi_reads_one_token() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bare_doi_reads_one_token.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>The command, then braced text.</p></para>"##,
  );
}

/// 57o: every framed.sty environment is `\MakeFramed`, whose body is a `\vbox` (framed.sty:289, 326):
/// internal vertical mode — `$$…$$` a display, a minipage in a paragraph (KPE #329; Perl's binding
/// declares no mode, framed.sty.ltxml:21-104).
#[test]
fn framed_body_is_vertical() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/block-model/framed_body_is_vertical.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "block",
    &[r#"framed="rectangle""#],
    r##"<block cssstyle="padding:9.0pt" framecolor="#000000" framed="rectangle"><p>First paragraph</p><equation xml:id="S0.Ex1"><Math mode="display" tex="x^{2}" text="x ^ 2" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></equation><p>and more.</p><p><inline-block class="ltx_minipage" vattach="middle" width="138.0pt"><p>Boxed text.</p></inline-block></p></block>"##,
  );
}

/// 57o: a titlesec format runs once, before the label (titlesec.sty:740-818): its material prints
/// once ("XX1 Intro"), its alignment reaches numbered titles, and `\titleformat*` replaces only
/// the format, keeping the number (titlesec.sty:672-683; KPE #328, witness 2605.21802).
#[test]
fn titlesec_format_runs_once() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_format_runs_once.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title font="bold">XX1  Intro</title><toctitle><tag close=" ">1</tag>Intro</toctitle><para xml:id="S1.p1"><p>A.</p></para><subsection inlist="toc" xml:id="S1.SS1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title class="ltx_align_left" font="bold">1.1  Sub</title><toctitle><tag close=" ">1.1</tag>Sub</toctitle><para xml:id="S1.SS1.p1"><p>B.</p></para><subsubsection inlist="toc" xml:id="S1.SS1.SSS1"><tags><tag>1.1.1</tag><tag role="refnum">1.1.1</tag><tag role="typerefnum">§1.1.1</tag></tags><title font="bold">1.1.1  Deep</title><toctitle><tag close=" ">1.1.1</tag>Deep</toctitle><para xml:id="S1.SS1.SSS1.p1"><p>C.</p></para></subsubsection></subsection></section>"##,
  );
}

/// 57o: `\titleline[align]{material}` prints its material as a line of the title (titlesec.sty:1088-1109),
/// once now that the format runs once; the group scopes its `\small`. 57q: the display label is a line of
/// its own (titlesec.sty:751), as pdflatex prints "PART / CHAPTER / 1 / Intro".
#[test]
fn titleline_prints_its_material() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titleline_prints_its_material.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r##"<title><text fontsize="90%">PART<break/></text>CHAPTER<break/>1<break/>Intro</title>"##,
  );
}

/// 57o: a minipage folded into its `<p>` is a panel at its declared width (latex.ltx:16305-16312), not
/// a standalone paragraph row: the two 0.45\textwidth text panels share a row, short or long text
/// (KPE #310, DIVERGENCES #343).
#[test]
fn minipage_text_panels_share_a_row() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/minipage_text_panels_share_a_row.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p class="ltx_figure_panel ltx_minipage" vattach="top" width="155.3pt">Left text.</p><p class="ltx_figure_panel ltx_minipage" vattach="top" width="155.3pt">Right text.</p><toccaption><tag close=" ">1</tag>Two panels.</toccaption><caption><tag close=": ">Figure 1</tag>Two panels.</caption></figure>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r##"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><p class="ltx_figure_panel ltx_minipage" vattach="top" width="155.3pt">Left text that is long enough to wrap over several lines in the printed minipage of this figure, surely more than one line.</p><p class="ltx_figure_panel ltx_minipage" vattach="top" width="155.3pt">Right text that is long enough to wrap over several lines in the printed minipage of this figure, surely more than one line.</p><toccaption><tag close=" ">2</tag>Long.</toccaption><caption><tag close=": ">Figure 2</tag>Long.</caption></figure>"##,
  );
}

/// 57o: a minipage folded into its one child keeps its declared width for panel layout, so the two
/// rows of label + two panels break before the second label (witnesses 2605.17146, 2605.13435,
/// 2605.05889; DIVERGENCES #343).
#[test]
fn minipage_panel_single_child_width() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/minipage_panel_single_child_width.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><inline-block angle="90" class="ltx_figure_panel ltx_minipage" depth="0.0pt" height="21.1pt" innerdepth="0.0pt" innerheight="6.8pt" innerwidth="21.1pt" vattach="middle" width="27.6pt" xtranslate="-7.1pt" ytranslate="-7.1pt"><p>EKF</p></inline-block><rule class="ltx_figure_panel ltx_minipage" height="56.9pt" vattach="middle" width="151.8pt"/><rule class="ltx_figure_panel ltx_minipage" height="56.9pt" vattach="middle" width="151.8pt"/><break class="ltx_break"/><inline-block angle="90" class="ltx_figure_panel ltx_minipage" depth="0.0pt" height="21.8pt" innerdepth="0.0pt" innerheight="6.8pt" innerwidth="21.8pt" vattach="middle" width="27.6pt" xtranslate="-7.5pt" ytranslate="-7.5pt"><p>UKF</p></inline-block><rule class="ltx_figure_panel ltx_minipage" height="56.9pt" vattach="middle" width="151.8pt"/><rule class="ltx_figure_panel ltx_minipage" height="56.9pt" vattach="middle" width="151.8pt"/><toccaption><tag close=" ">1</tag>Rows</toccaption><caption><tag close=": ">Figure 1</tag>Rows</caption></figure>"##,
  );
}

/// 57o: a titlesec `hang`/`runin` format takes label, separator and title as one group
/// (titlesec.sty:767, 788) — a trailing `\MakeUppercase` uppercases both, the shape's class goes
/// first; `\titleformat*` keeps a shaped format's class and, on a command never given a
/// `\titleformat`, builds titlesec's default composer (titlesec.sty:1541-1563; KPE #328).
#[test]
fn titlesec_format_takes_label_and_title() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_format_takes_label_and_title.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title class="ltx_runin" font="bold">1  FIRST</title><toctitle><tag close=" ">1</tag>First</toctitle><para xml:id="S1.p1"><p>A.</p></para><subsection inlist="toc" xml:id="S1.SS1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title class="ltx_runin" font="bold">1.1  Sub</title><toctitle><tag close=" ">1.1</tag>Sub</toctitle><para xml:id="S1.SS1.p1"><p>B.</p></para><subsubsection inlist="toc" xml:id="S1.SS1.SSS1"><tags><tag>1.1.1</tag><tag role="refnum">1.1.1</tag><tag role="typerefnum">§1.1.1</tag></tags><title font="bold" fontsize="120%">1.1.1  Deep</title><toctitle><tag close=" ">1.1.1</tag>Deep</toctitle><para xml:id="S1.SS1.SSS1.p1"><p>C.</p></para></subsubsection></subsection></section>"##,
  );
}

/// 57o: a centring switch in a title format aligns the heading itself (DIVERGENCES #344): the format's
/// size stays on the `<title>` (a `\huge` display chapter at 207%, not ≈517% on an aligned span).
/// 57q: the display label is a line of its own and the before-code `\Huge` sizes the title
/// (titlesec.sty:751-753 `{#2…\@@par}` … `#4{#8}`; `\Huge` inside `\huge` is 144%).
#[test]
fn centred_title_format_keeps_its_size() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/centred_title_format_keeps_its_size.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "chapter",
    &[r#"xml:id="Ch1""#],
    r##"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title class="ltx_align_center" font="bold" fontsize="207%">Chapter 1<break/><text fontsize="144%">Alpha</text></title><toctitle><tag close=" ">1</tag>Alpha</toctitle><section inlist="toc" xml:id="Ch1.S1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title class="ltx_align_center" font="bold" fontsize="144%">1.1  S one</title><toctitle><tag close=" ">1.1</tag>S one</toctitle><para xml:id="Ch1.S1.p1"><p>Text.</p></para></section></chapter>"##,
  );
}

/// 57p: pnas-new's `\doi` is its own setter (pnas-new.cls:422 `\def\@doi{#1}`, printed in the footer),
/// so `\doi{\url{…}}` keeps its url; OmniBus's verbatim `\doi` (KPE #326) printed "\urlwww.pnas…"
/// (witnesses 2605.03599, 2605.07504); a bibliography's `\doi` (pnas-new.bst:116-121) meets the same
/// setter and prints nothing.
#[test]
fn pnas_doi_is_the_classs_setter() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/pnas_doi_is_the_classs_setter.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The binding's mdframed stub announces itself; nothing else warns.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Warning:missing_file:mdframed.sty"),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "pubnote",
    &[r#"role="doi""#],
    r##"<pubnote name="DOI: " role="doi"><ref class="ltx_url" font="typewriter" href="www.pnas.org/cgi/doi/10.1073/pnas.XXXXXXXXXX">www.pnas.org/cgi/doi/10.1073/pnas.XXXXXXXXXX</ref></pubnote>"##,
  );
  // A bibliography's `\doi` meets the class setter: nothing printed (as in pdflatex).
  latexml::util::test::assert_element(
    &xml,
    "bibitem",
    &[r#"key="k""#],
    r##"<bibitem key="k" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="refnum">(1)</tag><tag role="key">k</tag></tags><bibblock> A. Author, Title. 
</bibblock></bibitem>"##,
  );
}

/// 57p: subcaption's sub-figure number is the bare letter and `\p@subfigure` the figure's
/// (caption3.sty:1803-1806); the parentheses are the caption label's (subcaption.sty:218-222):
/// refnum "1a", caption "(a)" — Perl's `(\alph{subfigure})` (subcaption.sty.ltxml:27-28) gave
/// "1(a)" (KPE #330; 16 papers of the 57l+57m A/B).
#[test]
fn subfigure_refnum_is_the_letter() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfigure_refnum_is_the_letter.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" labels="LABEL:m" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:a" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags><p>X</p><toccaption><tag close=" ">a</tag>Alpha</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Alpha</text></caption></figure><figure class="ltx_figure_panel" inlist="lof" labels="LABEL:b" xml:id="S0.F1.sf2"><tags><tag><text fontsize="90%">(b)</text></tag><tag role="refnum">1b</tag></tags><p>Y</p><toccaption><tag close=" ">b</tag>Beta</toccaption><caption><tag close=" "><text fontsize="90%">(b)</text></tag><text fontsize="90%">Beta</text></caption></figure><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure>"##,
  );
}

/// 57p: subcaption's package options are `\captionsetup[sub]` settings (subcaption.sty:214-217), and a
/// sub-caption label is its label format over `\thesubfigure` (caption3.sty:734-737): `labelformat=simple`
/// with a parenthesized `\thesubfigure` gives "(a)", not "((a))" (2605.01394).
#[test]
fn subcaption_labelformat_option_keeps_the_authors_number() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subcaption_labelformat_option_keeps_the_authors_number.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure inlist="lof" labels="LABEL:a" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1(a)</tag></tags><p>X</p><toccaption><tag close=" ">(a)</tag>Alpha</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Alpha</text></caption></figure><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure>"##,
  );
}

/// 57p: a `\captionsetup[subfigure]{labelformat=simple}` chooses the sub-caption label format.
#[test]
fn subcaption_labelformat_setup_keeps_the_authors_number() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subcaption_labelformat_setup_keeps_the_authors_number.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure inlist="lof" labels="LABEL:a" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1(a)</tag></tags><p>X</p><toccaption><tag close=" ">(a)</tag>Alpha</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Alpha</text></caption></figure><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure>"##,
  );
}

/// 57p: subcaption declares a sub-type only when its counter is new (subcaption.sty:226-230), so after
/// subfigure.sty its `(\alph{subfigure})` stays: refnum "1(a)" (2605.01846).
#[test]
fn subfigure_before_subcaption_keeps_its_numbers() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfigure_before_subcaption_keeps_its_numbers.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure labels="LABEL:a" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1(a)</tag></tags><p>X</p><toccaption><tag close=" ">(a)</tag>Alpha</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Alpha</text></caption></figure><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure>"##,
  );
}

/// 57q: titlesec sets a `display` label in a group on a line of its own (titlesec.sty:751-753): the
/// label's `\Large` stays in it, the title takes the format's `\Huge` and the before-code's `\filleft`
/// (57o ran them together, "Chapter 1Introducción"; unamth-template tesis).
#[test]
fn titlesec_display_label_is_a_line_of_its_own() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_display_label_is_a_line_of_its_own.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "chapter",
    &[r#"xml:id="Ch1""#],
    r##"<chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title class="ltx_align_right" fontsize="144%">Chapter 1<break/><text class="ltx_align_right" fontsize="207%">Introduction</text></title><toctitle><tag close=" ">1</tag>Introduction</toctitle><para xml:id="Ch1.p1"><p>Text.</p></para></chapter>"##,
  );
}

/// 57q: the before-code takes the title, `#4{#8}` (titlesec.sty:773), and a run-in title's after-code
/// follows it (:792); Perl's binding ignored both (KPE #332).
#[test]
fn titlesec_before_code_takes_the_title() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_before_code_takes_the_title.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title font="bold">1  INTRO</title><toctitle><tag close=" ">1</tag>Intro</toctitle><paragraph inlist="toc" xml:id="S1.SS0.SSS0.P1"><tags><tag>1.0.0.1</tag><tag role="refnum">1.0.0.1</tag><tag role="typerefnum">¶1.0.0.1</tag></tags><title class="ltx_runin" font="bold">Para.</title><toctitle><tag close=" ">1.0.0.1</tag>Para</toctitle><para xml:id="S1.SS0.SSS0.P1.p1"><p>Text.</p></para></paragraph></section>"##,
  );
}

/// 57q: under `explicit` the title is the before-code's `#1` (titlesec.sty:296-299, :714-719): `#1.` and
/// `\MakeUppercase{#1}` print as pdflatex does, the title once (KPE #332).
#[test]
fn titlesec_explicit_title_is_the_before_codes_argument() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_explicit_title_is_the_before_codes_argument.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "section",
    &[r#"xml:id="S1""#],
    r##"<section inlist="toc" xml:id="S1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">§1</tag></tags><title font="bold">1  Intro.</title><toctitle><tag close=" ">1</tag>Intro</toctitle><subsection inlist="toc" xml:id="S1.SS1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title font="italic">1.1 SCOPE</title><toctitle><tag close=" ">1.1</tag>Scope</toctitle><subsubsection inlist="toc" xml:id="S1.SS1.SSS1"><tags><tag>1.1.1</tag><tag role="refnum">1.1.1</tag><tag role="typerefnum">§1.1.1</tag></tags><title><tag close=" ">1.1.1</tag>Detail</title><para xml:id="S1.SS1.SSS1.p1"><p>Text.</p></para></subsubsection></subsection></section>"##,
  );
}

/// 57q: a set-width box keeps its own material when arrange_panels merges a narrow panel beside it:
/// the merge wraps picture minipage and label minipage, "(a)" stays out of the picture's box
/// (witness 2605.00042 S5.F4; KPE #331).
#[test]
fn panel_merge_keeps_boxes_closed() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/panel_merge_keeps_boxes_closed.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r##"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><block><block align="center" class="ltx_minipage" vattach="middle" width="241.5pt"><graphics class="ltx_centering" graphic="none.png" options="width=241.49895pt,keepaspectratio=true" xml:id="S0.F1.g1"/></block><p align="center" class="ltx_minipage" vattach="middle" width="13.8pt"><text fontsize="80%">(a)</text></p></block><toccaption class="ltx_centering"><tag close=" ">1</tag>Rows</toccaption><caption class="ltx_centering"><tag close=": ">Figure 1</tag>Rows</caption></figure>"##,
  );
  // Two rows: the second row's picture minipage (a row start, marked a panel by the overflow)
  // keeps no panel class inside its panel block.
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r##"<figure inlist="lof" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><block class="ltx_figure_panel"><block align="center" class="ltx_minipage" vattach="middle" width="241.5pt"><graphics class="ltx_centering" graphic="none.png" options="width=241.49895pt,keepaspectratio=true" xml:id="S0.F2.g1"/></block><p align="center" class="ltx_minipage" vattach="middle" width="13.8pt"><text fontsize="80%">(a)</text></p></block><break class="ltx_break"/><block class="ltx_figure_panel"><block align="center" class="ltx_minipage" vattach="middle" width="241.5pt"><graphics class="ltx_centering" graphic="none.png" options="width=241.49895pt,keepaspectratio=true" xml:id="S0.F2.g2"/></block><p align="center" class="ltx_minipage" vattach="middle" width="13.8pt"><text fontsize="80%">(b)</text></p></block><toccaption class="ltx_centering"><tag close=" ">2</tag>Two rows</toccaption><caption class="ltx_centering"><tag close=": ">Figure 2</tag>Two rows</caption></figure>"##,
  );
}

/// 57q review: the after-code runs after the title (titlesec.sty:756, :780, :816, :862; runin
/// `#5\unskip`, :792) — a before-code's `\list` is closed by its `[\endlist]` (AVT.sty `\part`), a
/// block format's `[\setcounter{equation}{0}]` resets the equations (2605.06498, 2605.11886), a
/// run-in `[.\quad]` leaves no trailing space — and not in `wrap`, `rightmargin`, `drop` (:901-1029)
/// (KPE #332).
#[test]
fn titlesec_after_code_follows_the_title() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_after_code_follows_the_title.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "part",
    &[r#"xml:id="Pt1""#],
    r##"<part inlist="toc" xml:id="Pt1"><tags><tag>Part I</tag><tag role="refnum">I</tag><tag role="typerefnum">Part I</tag></tags><title fontsize="144%">PART I<break/><inline-block><itemize><item xml:id="Ch0.S0.I1.ix1"><para align="center" xml:id="Ch0.S0.I1.ix1.p1"><p>Frame</p></para></item></itemize></inline-block></title><toctitle><tag close=" ">I</tag>Frame</toctitle><chapter inlist="toc" xml:id="Ch1"><tags><tag>Chapter 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Chapter 1</tag></tags><title><tag close=" ">Chapter 1</tag>One</title><toctitle><tag close=" ">1</tag>One</toctitle><para xml:id="Ch1.p1"><equation xml:id="Ch1.E1"><tags><tag>(1.1)</tag><tag role="refnum">1.1</tag></tags><Math mode="display" tex="a" text="a" xml:id="Ch1.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math></equation></para><section inlist="toc" xml:id="Ch1.S1"><tags><tag>1.1</tag><tag role="refnum">1.1</tag><tag role="typerefnum">§1.1</tag></tags><title font="bold">1.1  Block</title><toctitle><tag close=" ">1.1</tag>Block</toctitle><para xml:id="Ch1.S1.p1"><equation xml:id="Ch1.E1a"><tags><tag>(1.1)</tag><tag role="refnum">1.1</tag></tags><Math mode="display" tex="b" text="b" xml:id="Ch1.E1a.m1"><XMath><XMTok font="italic" role="UNKNOWN">b</XMTok></XMath></Math></equation></para><subsection inlist="toc" xml:id="Ch1.S1.SS1"><tags><tag>1.1.1</tag><tag role="refnum">1.1.1</tag><tag role="typerefnum">§1.1.1</tag></tags><title font="bold">1.1.1  Wrapped</title><toctitle><tag close=" ">1.1.1</tag>Wrapped</toctitle><paragraph inlist="toc" xml:id="Ch1.S1.SS1.SSS0.P1"><tags><tag>1.1.1.0.1</tag><tag role="refnum">1.1.1.0.1</tag><tag role="typerefnum">¶1.1.1.0.1</tag></tags><title class="ltx_runin" font="bold">Para.</title><toctitle><tag close=" ">1.1.1.0.1</tag>Para</toctitle><para xml:id="Ch1.S1.SS1.SSS0.P1.p1"><p>Text.</p></para></paragraph></subsection></section></chapter></part>"##,
  );
}

/// 57q review: without `explicit`, a `#` in a title format's before-code is TeX's "Illegal parameter
/// number in definition of \ttlf@section"; the error is raised and the author's `#1` is the title,
/// printed once (the first cut printed "1 IntroIntro" silently).
#[test]
fn titlesec_before_code_parameter_needs_explicit() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_before_code_parameter_needs_explicit.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains(
      "Error:misdefined:\\ttlf@section Illegal parameter number in definition of \\ttlf@section"
    ),
    "{stderr}"
  );
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r##"<title font="bold">1  Intro</title>"##,
  );
}

/// 57q re-review: `##` in a title format is an escaped parameter character (tex.web §476-479), not
/// a stray `#1`: an inner `\def\x##1{…}` in the before-code takes the title, with no error.
#[test]
fn titlesec_before_code_inner_definition_keeps_its_parameters() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/titlesec_before_code_inner_definition_keeps_its_parameters.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r##"<title font="bold">1  [Intro]</title>"##,
  );
}

/// 57r: `\autoref` starts the paragraph (Perl hyperref.sty.ltxml:367-370 `enterHorizontal`, dropped by
/// the port; hyperref.sty:8202-8204 `\leavevmode`), so the space after it stays (33 A/B papers,
/// witness 2605.02640).
#[test]
fn autoref_starts_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/autoref_starts_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p><ref class="ltx_refmacro_autoref" labelref="LABEL:sec:a" show="autoref"/> shows the claim.</p>"##,
  );
}

/// 57r: colortbl's `\rowcolor[model]{color}[left][right]` reads its overhangs (colortbl.sty:208-231); they
/// were cell text and misplaced a `\multicolumn`'s `\omit` (2605.22864, 2605.08915, 2605.04906; KPE #333).
#[test]
fn rowcolor_reads_its_overhangs() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/rowcolor_reads_its_overhangs.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "tabular",
    &[],
    r##"<tabular vattach="middle"><tbody><tr backgroundcolor="#E6E6E6"><td align="left" colspan="2">Group</td></tr><tr backgroundcolor="#E6E6E6"><td align="left"><text backgroundcolor="#E6E6E6">c</text></td><td align="left"><text backgroundcolor="#E6E6E6">d</text></td></tr></tbody></tabular>"##,
  );
}

/// 57r: `\pdfstringdefDisableCommands{…}` stores its argument for bookmark strings, read with `@` a
/// letter, never typeset (hyperref.sty:790-798; KPE #334).
#[test]
fn pdfstringdef_disable_commands_reads_its_argument() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/pdfstringdef_disable_commands_reads_its_argument.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>Body.</p>"##);
}

/// 57r: xcolor's `\fcolorbox{frame}[bg-model]{bg}{text}` (xcolor.sty:822-826), set as the `\hbox` it is:
/// in the paragraph, spaces kept (KPE #336).
#[test]
fn fcolorbox_reads_the_background_model() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/fcolorbox_reads_the_background_model.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>B <text backgroundcolor="#FFFF00" framecolor="#FF0000" framed="rectangle">boxed</text> end.</p>"##,
  );
}

/// 57r: xcolor's `\definecolor{name}[prefix]{model}{spec}` and `\providecolor` read the prefix
/// (xcolor.sty:519-522, :594-597; KPE #337).
#[test]
fn definecolor_reads_a_name_prefix() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/definecolor_reads_a_name_prefix.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>A <text color="#FF0000">red</text> and <text color="#0000FF">blue</text>.</p>"##,
  );
}

/// 57r: `\hyperdef[label]{cat}{name}{text}` reads its label (hyperref.sty:4833, :4864-4876; KPE #335).
#[test]
fn hyperdef_reads_its_label() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/hyperdef_reads_its_label.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "anchor",
    &[r#"xml:id="cat.nm""#],
    r##"<anchor xml:id="cat.nm">Target text</anchor>"##,
  );
  assert!(!xml.contains("lab]"), "{xml}");
  // hyperref.sty:4835-4839: an empty category names the anchor by its name alone.
  latexml::util::test::assert_element(
    &xml,
    "anchor",
    &[r#"xml:id="bare""#],
    r##"<anchor xml:id="bare">Bare</anchor>"##,
  );
}

/// 57r: `\AfterEndPreamble` after `begindocument/end` has run executes at once (etoolbox.sty:1745, a
/// one-time hook; KPE #338).
#[test]
fn afterendpreamble_in_the_body_runs_now() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/afterendpreamble_in_the_body_runs_now.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(&xml, "p", &[], r##"<p>A LateZ</p>"##);
}

/// 57r: babel's `\shorthandoff*` reads its star (babel.sty:1428-1430) and restores the original catcode.
#[test]
fn shorthandoff_star_reads_the_star() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/babel-lang/shorthandoff_star_reads_the_star.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r##"<p>XY a:b
c d <Math mode="inline" tex="x^{2}" text="x ^ 2" xml:id="p1.m1"><XMath><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMath></Math></p>"##,
  );
}

/// 57r: color.sty's `\fcolorbox` is `\color@b@x` (color.sty:159-164), an `\hbox` in the running
/// paragraph: its body is restricted horizontal, the paragraph is not split and the space after the
/// box stays (Perl's `internal_vertical` split it; KPE #336, #293).
#[test]
fn fcolorbox_stays_in_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/fcolorbox_splits_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>X <text backgroundcolor="#0000FF" framecolor="#FF0000" framed="rectangle">Mid</text> et B.</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p><text backgroundcolor="#0000FF" framecolor="#FF0000" framed="rectangle">Start</text> et B.</p></para>"##,
  );
}

/// 57s: enumitem's `\AddEnumerateCounter*` reads its star (enumitem.sty:575-591); the width sample was
/// typeset as a stray paragraph "9" (KPE #339).
#[test]
fn addenumeratecounter_reads_the_star() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/addenumeratecounter_reads_the_star.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">item *</tag></tags><para xml:id="S0.I1.i1.p1"><p>a</p></para></item></enumerate></para>"##,
  );
}

/// 57s: lineno's `{bframe}` begins and ends with `\par` (lineno.sty:2881-2908): its body is paragraphs of
/// its own (RUST-ONLY: the binding's plain `#body` ran it into the paragraph).
#[test]
fn bframe_is_paragraphs_of_its_own() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/bframe_is_paragraphs_of_its_own.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Text</p><p>framed</p><p>after.</p></para>"##,
  );
}

/// 57s: lineno's `\modulolinenumbers*[n]` reads its star and option (lineno.sty:2151-2158, :2182; KPE #340).
#[test]
fn modulolinenumbers_reads_the_star() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/modulolinenumbers_reads_the_star.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Text.</p></para>"##,
  );
}

/// 57s: lineno's `{numquote}`, `{numquotation}` and their starred forms are quotes whose
/// `\quotelinenumbers` reads `*` or `[n]` (lineno.sty:2849-2867; KPE #341).
#[test]
fn numquote_star_is_a_quote() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/block-model/numquote_star_is_a_quote.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><quote><p>quoted</p></quote><quote><p>numbered</p></quote><quote><p>long</p></quote></para>"##,
  );
}

/// 57s: pifont's `\dingline`/`\dingfill` are the package's `\Piline`/`\Pifill` (pifont.sty:30-34, :57-58): a
/// leader of the symbol, `\dingline` a paragraph of its own (KPE #342).
#[test]
fn dingline_is_a_line_of_its_own() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/block-model/dingline_is_a_line_of_its_own.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Text</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para class="ltx_noindent" xml:id="p2"><p><text align="center" class="ltx_leader" width="14.5pt">☞</text></p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r##"<para xml:id="p3"><p>more <text align="center" class="ltx_leader" width="14.5pt">✓</text> end.</p></para>"##,
  );
}

/// 57s: rotating's `\turnbox` is `\leavevmode` then an `\hbox` (rotating.sty:100-107): it stays in the
/// running paragraph (KPE #343).
#[test]
fn turnbox_stays_in_the_paragraph() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/turnbox_stays_in_the_paragraph.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before <inline-block angle="30" depth="0.0pt" height="13.9pt" innerdepth="0.0pt" innerheight="6.9pt" innerwidth="15.8pt" width="17.2pt" xtranslate="0.7pt" ytranslate="-3.5pt"><p>box</p></inline-block> after.</p></para>"##,
  );
}

/// 57s: ulem's `\UL@end *` is a delimited macro (ulem.sty:59): nothing after the `*` is read, so a custom
/// `\ULon` underline keeps the space after its argument (RUST-ONLY: `OptionalMatch:*` skipped it).
/// The custom mark itself is not rendered yet (RED `fonts-nfss/ulem_custom_mark_is_rendered`).
#[test]
fn ulem_end_keeps_the_following_space() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/ulem_end_keeps_the_following_space.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>Before struck after.</p></para>"##,
  );
}

/// 57t: amsmath's `\ams@start@box` (amsmath.sty:1441-1456): a bracket group at an aligned/gathered formula's
/// start that is no position goes back to the first cell, with amsmath's warning (KPE #344; 2605.04504 & al.).
#[test]
fn aligned_returns_bracket_group() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/aligned_returns_bracket_group.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert!(
    stderr.contains("Bracket group [\\alpha ,\\beta ] at formula start!"),
    "{stderr}"
  );
  assert!(
    stderr.contains("Bracket group [a,b] at formula start!"),
    "{stderr}"
  );
  // `gathered` takes no `vattach` from a data bracket group; `[a,b]` opens its first cell.
  latexml::util::test::assert_element(
    &xml,
    "XMArray",
    &[r#"name="gathered""#],
    r##"<XMArray name="gathered"><XMRow><XMCell align="center"><XMApp><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.2">=</XMTok><XMWrap><XMTok role="OPEN" stretchy="false" xml:id="S0.Ex1.m1.3">[</XMTok><XMWrap xml:id="S0.Ex1.m1.4"><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.5">a</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.7">b</XMTok></XMWrap><XMTok role="CLOSE" stretchy="false" xml:id="S0.Ex1.m1.8">]</XMTok></XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.9">c</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align="center"><XMApp><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.10">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.11">d</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.12">e</XMTok></XMApp></XMCell></XMRow></XMArray>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.E1X.m2""#],
    r##"<Math tex="\displaystyle[\alpha,\beta]" text="closed-interval@(alpha, beta)" xml:id="S0.E1X.m2"><XMath><XMDual><XMApp><XMTok meaning="closed-interval"/><XMRef idref="S0.E1X.m2.1"/><XMRef idref="S0.E1X.m2.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok font="italic" name="alpha" role="UNKNOWN" xml:id="S0.E1X.m2.1">α</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" name="beta" role="UNKNOWN" xml:id="S0.E1X.m2.2">β</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual></XMath></Math>"##,
  );
}

/// 57t: `\alignedat` reads `[pos]` before `{n}` (amsmath.sty:1518-1524; KPE #346), and returns a data bracket group.
#[test]
fn alignedat_reads_position_first() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/alignedat_reads_position_first.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1X.m2""#],
    r##"<Math tex="\displaystyle a" text="a" xml:id="S0.Ex1X.m2"><XMath><XMTok font="italic" role="UNKNOWN">a</XMTok></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2X.m2""#],
    r##"<Math tex="\displaystyle[x,y]" text="closed-interval@(x, y)" xml:id="S0.Ex2X.m2"><XMath><XMDual><XMApp><XMTok meaning="closed-interval"/><XMRef idref="S0.Ex2X.m2.1"/><XMRef idref="S0.Ex2X.m2.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2X.m2.1">x</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2X.m2.2">y</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual></XMath></Math>"##,
  );
}

/// 57t: mathtools' `\ArrowBetweenLines*[\Downarrow]` is a row holding the arrow and `\shortvdotswithin*`
/// reads its star (mathtools.sty:1299-1344; KPE #347).
#[test]
fn mathtools_starred_row_macros() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/mathtools_starred_row_macros.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m3""#],
    r##"<Math tex="\displaystyle\quad\Downarrow" text="Downarrow" xml:id="S0.Ex2.m3"><XMath><XMTok lpadding="10.0pt" name="Downarrow" role="ARROW">⇓</XMTok></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex4.m2""#],
    r##"<Math tex="\displaystyle\mathmakebox[7.7778pt][c]{\vdots}" text="vdots" xml:id="S0.Ex4.m2"><XMath><XMTok name="vdots" role="ID">⋮</XMTok></XMath></Math>"##,
  );
  // `\shortvdotswithin*{=}` is `\vdotswithin{=}&`: the vdots in the first (right-aligned) cell.
  latexml::util::test::assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S0.Ex4""#],
    r##"<equation xml:id="S0.Ex4"><MathFork><Math tex="\displaystyle\mathmakebox[7.7778pt][c]{\vdots}" text="vdots" xml:id="S0.Ex4.m2"><XMath><XMTok name="vdots" role="ID">⋮</XMTok></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle\mathmakebox[7.7778pt][c]{\vdots}" text="vdots" xml:id="S0.Ex4.m1"><XMath><XMTok name="vdots" role="ID">⋮</XMTok></XMath></Math></td><td/></MathBranch></MathFork></equation>"##,
  );
  // The starred arrow row: `&&\quad\Downarrow`, the arrow in the right-aligned column.
  latexml::util::test::assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S0.Ex2""#],
    r##"<equation xml:id="S0.Ex2"><MathFork><Math tex="\displaystyle\quad\Downarrow" text="Downarrow" xml:id="S0.Ex2.m3"><XMath><XMTok lpadding="10.0pt" name="Downarrow" role="ARROW">⇓</XMTok></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle\quad\Downarrow" text="Downarrow" xml:id="S0.Ex2.m1"><XMath><XMTok lpadding="10.0pt" name="Downarrow" role="ARROW">⇓</XMTok></XMath></Math></td></MathBranch></MathFork></equation>"##,
  );
}

/// 57t: `{multlined}`'s optionals are the position when t/c/b and the width otherwise, in either order
/// (mathtools.sty:677-688, 795-812; KPE #348).
#[test]
fn multlined_classifies_its_optionals() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/multlined_classifies_its_optionals.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\begin{multlined}a+b+c\\&#10;+d+e\end{multlined}=x" text="a + b + c + d + e = x" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMDual><XMApp><XMRef idref="S0.Ex1.m1.1"/><XMRef idref="S0.Ex1.m1.2"/><XMRef idref="S0.Ex1.m1.3"/><XMRef idref="S0.Ex1.m1.4"/><XMRef idref="S0.Ex1.m1.5"/><XMRef idref="S0.Ex1.m1.6"/></XMApp><XMArray name="multlined" vattach="bottom"><XMRow><XMCell align="left"><XMApp><XMTok meaning="plus" role="ADDOP" xml:id="S0.Ex1.m1.1">+</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">a</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.3">b</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.4">c</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align="right"><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.5">d</XMTok></XMApp><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.6">e</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"##,
  );
  // `[\mypos]` holding `t` is the position (`\MH_if:w t#1` expands it).
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\begin{multlined}a+b\\&#10;+c\end{multlined}=y" text="a + b + c = y" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMDual><XMApp><XMRef idref="S0.Ex2.m1.1"/><XMRef idref="S0.Ex2.m1.2"/><XMRef idref="S0.Ex2.m1.3"/><XMRef idref="S0.Ex2.m1.4"/></XMApp><XMArray name="multlined" vattach="top"><XMRow><XMCell align="left"><XMApp><XMTok meaning="plus" role="ADDOP" xml:id="S0.Ex2.m1.1">+</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.2">a</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.3">b</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align="right"><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.4">c</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"##,
  );
}

/// 57t: `\bmdefine` defines globally and overwrites, a self-reference reading the old meaning
/// (bm.sty:229, :307-316; KPE #350).
#[test]
fn bmdefine_is_global() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/bmdefine_is_global.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\bm{x}+\bm{\alpha}" text="x + alpha" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="bold italic" role="UNKNOWN">x</XMTok><XMTok font="bold italic" name="alpha" role="UNKNOWN">α</XMTok></XMApp></XMath></Math>"##,
  );
  // A body naming the command itself is bold over its meaning at that moment, not a recursion,
  // declared twice too (each save has a name of its own).
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\bm{\bm{\nabla}}" text="nabla" xml:id="S0.Ex2.m1"><XMath><XMTok font="bold" name="nabla" role="OPERATOR">∇</XMTok></XMath></Math>"##,
  );
}

/// 57t: `\cfrac[l]` reads its alignment (amsmath.sty:912; KPE #349).
#[test]
fn cfrac_reads_its_alignment() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/cfrac_reads_its_alignment.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="x=\cfrac[l]{1}{2+\cfrac[r]{3}{4}}" text="x = continued-fraction@(1, 2 + continued-fraction@(3, 4))" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp><XMTok mathstyle="display" meaning="continued-fraction" name="cfrac"/><XMTok meaning="1" role="NUMBER">1</XMTok><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok><XMApp><XMTok mathstyle="display" meaning="continued-fraction" name="cfrac"/><XMTok meaning="3" role="NUMBER">3</XMTok><XMTok meaning="4" role="NUMBER">4</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57t: physics' optional arguments are found past spaces, as ltcmd's `g`/`t`/`s`/`d` grabbers find them
/// (xparse.sty:161-165; KPE #345; witness 2605.08402).
#[test]
fn physics_optional_args_skip_spaces() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_optional_args_skip_spaces.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\bra{a}\ket{b}\quad\expectationvalue{A}{\psi}\quad\outerproduct{a}{b}\quad\derivative{f}{x}" text="fragments@(inner-product@(a, b), expectation-value@(A, psi, psi), outer-product@(a, b), (derivative@(x))@(f))" xml:id="S0.Ex1.m1"><XMath><XMDual><XMApp><XMTok meaning="fragments"/><XMRef idref="S0.Ex1.m1.1"/><XMRef idref="S0.Ex1.m1.4"/><XMRef idref="S0.Ex1.m1.8"/><XMRef idref="S0.Ex1.m1.11"/></XMApp><XMWrap><XMDual xml:id="S0.Ex1.m1.1"><XMApp><XMTok meaning="inner-product"/><XMRef idref="S0.Ex1.m1.2"/><XMRef idref="S0.Ex1.m1.3"/></XMApp><XMWrap><XMTok name="langle" role="OPEN" stretchy="true">⟨</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">a</XMTok><XMTok role="MIDDLE" role_side="middle" stretchy="true">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.3">b</XMTok><XMTok name="rangle" role="CLOSE" stretchy="true">⟩</XMTok></XMWrap></XMDual><XMHint name="quad" role="PUNCT" width="10pt"/><XMDual xml:id="S0.Ex1.m1.4"><XMApp><XMTok meaning="expectation-value"/><XMRef idref="S0.Ex1.m1.6"/><XMRef idref="S0.Ex1.m1.5"/><XMRef idref="S0.Ex1.m1.7"/></XMApp><XMWrap><XMTok name="langle" role="OPEN" stretchy="true">⟨</XMTok><XMTok font="italic" name="psi" role="UNKNOWN" xml:id="S0.Ex1.m1.5">ψ</XMTok><XMTok role="MIDDLE" role_side="middle" stretchy="true">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.6">A</XMTok><XMTok role="MIDDLE" role_side="middle" stretchy="true">|</XMTok><XMTok font="italic" name="psi" role="UNKNOWN" xml:id="S0.Ex1.m1.7">ψ</XMTok><XMTok name="rangle" role="CLOSE" stretchy="true">⟩</XMTok></XMWrap></XMDual><XMHint name="quad" role="PUNCT" width="10pt"/><XMDual xml:id="S0.Ex1.m1.8"><XMApp><XMTok meaning="outer-product"/><XMRef idref="S0.Ex1.m1.9"/><XMRef idref="S0.Ex1.m1.10"/></XMApp><XMWrap><XMTok role="VERTBAR" role_side="left" stretchy="true">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.9">a</XMTok><XMTok name="rangle" role="MIDDLE" role_side="middle" stretchy="true">⟩</XMTok><XMTok name="langle" role="OPEN" stretchy="false">⟨</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.10">b</XMTok><XMTok role="VERTBAR" role_side="right" stretchy="true">|</XMTok></XMWrap></XMDual><XMHint name="quad" role="PUNCT" width="10pt"/><XMDual xml:id="S0.Ex1.m1.11"><XMApp><XMApp><XMTok meaning="derivative"/><XMRef idref="S0.Ex1.m1.13"/></XMApp><XMRef idref="S0.Ex1.m1.12"/></XMApp><XMApp><XMTok mathstyle="display" meaning="divide" role="FRACOP"/><XMApp><XMTok role="DIFFOP">d</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.12">f</XMTok></XMApp><XMApp><XMTok role="DIFFOP">d</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.13">x</XMTok></XMApp></XMApp></XMDual></XMWrap></XMDual></XMath></Math>"##,
  );
}

/// 57t: physics' `\q<word>*` reads its star (physics.sty:383-401; RUST-ONLY: the port dropped Perl's
/// `OptionalMatch:*`).
#[test]
fn physics_qword_reads_star() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_qword_reads_star.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="x=1\mbox{if\quad}y=2\mbox{\quad and\quad}z=3\mbox{or\quad}w=4" text="x = 1 * [if ] * y = 2 * [ and ] * z = 3 * [or ] * w = 4" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="multirelation"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok><XMText>if </XMText><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok><XMText> and </XMText><XMTok font="italic" role="UNKNOWN">z</XMTok></XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="3" role="NUMBER">3</XMTok><XMText>or </XMText><XMTok font="italic" role="UNKNOWN">w</XMTok></XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok meaning="4" role="NUMBER">4</XMTok></XMApp></XMath></Math>"##,
  );
}

/// 57t: `\pmqty{…}` and its kin keep their body (physics.sty:70-73, 105-108; KPE #351).
#[test]
fn physics_matrix_keeps_its_body() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_matrix_keeps_its_body.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "XMArray",
    &[r#"xml:id="S0.Ex1.m1.2""#],
    r##"<XMArray xml:id="S0.Ex1.m1.2"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow></XMArray>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "XMArray",
    &[r#"xml:id="S0.Ex1.m1.4""#],
    r##"<XMArray xml:id="S0.Ex1.m1.4"><XMRow><XMCell align="center"><XMTok meaning="1" role="NUMBER">1</XMTok></XMCell><XMCell align="center"><XMTok meaning="0" role="NUMBER">0</XMTok></XMCell></XMRow></XMArray>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "XMArray",
    &[r#"xml:id="S0.Ex2.m1.2""#],
    r##"<XMArray xml:id="S0.Ex2.m1.2"><XMRow><XMCell align="center"><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMCell></XMRow></XMArray>"##,
  );
}

/// 57t: `\var[I]` and `\dd[2] x` keep their degree, the dual's `#2` (physics.sty.ltxml:370-388).
#[test]
fn physics_differential_keeps_its_degree() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_differential_keeps_its_degree.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\variation[I]f+\differential[2]x" text="(functional-power@(variation, I))@(f) + (functional-power@(differential, 2))@(x)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMApp role="DIFFOP"><XMTok meaning="functional-power" role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" meaning="variation" name="delta" role="DIFFOP">δ</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">I</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMApp><XMApp role="DIFFOP"><XMTok meaning="functional-power" role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="differential" role="DIFFOP">d</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57u: `\@listdepth`/`\@itemdepth`/`\@enumdepth` follow the lists (latex.ltx:15852, :15913; KPE #353).
#[test]
fn list_depth_registers_follow_the_lists() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/list_depth_registers_follow_the_lists.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>a[1]</p><enumerate xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>b[2][1]</p><itemize xml:id="S0.I1.i1.I1.i1.I1"><item xml:id="S0.I1.i1.I1.i1.I1.i1"><tags><tag><text font="bold">–</text></tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.I1.i1.I1.i1.p1"><p>c[3][2]</p></para></item></itemize></para></item></enumerate></para></item></itemize><p>d[0][0][0]</p></para>"##,
  );
}

/// 57u: the kernel `\list` advances only `\@listdepth`; enumitem's inline lists advance it too (enumitem.sty:1188-1191; KPE #353).
#[test]
fn list_depth_registers_across_list_kinds() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/list_depth_registers_across_list_kinds.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><itemize><item xml:id="S0.I1.i1"><tags><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>x[1]</p><itemize xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>y[2][1]</p></para></item></itemize><p>z[1]</p></para></item></itemize><p>w[0]</p><enumerate xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I2.i1.p1"><p>p[1][1] <inline-enumerate xml:id="S0.I2.i1.I2"><inline-item xml:id="S0.I2.i1.I2.i1"><tags><tag>(a)</tag><tag role="refnum">1a</tag><tag role="typerefnum">item 1a</tag></tags><text>q[2][2]</text></inline-item></inline-enumerate></p></para></item></enumerate><description xml:id="S0.I3"><item xml:id="S0.I3.ix1"><tags><tag><text font="bold">t</text></tag><tag role="typerefnum">item t</tag></tags><para xml:id="S0.I3.ix1.p1"><p>r[1]</p></para></item></description><p>s[0][0][0]</p></para>"##,
  );
}

/// 57u: `\setlist` sorts its names into lists and levels; `\setenumerate` defaults to level 0 (enumitem.sty:1674-1705; KPE #354).
#[test]
fn enumitem_setlist_levels_and_names() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/enumitem_setlist_levels_and_names.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I1""#],
    r##"<enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>(a)</tag><tag role="refnum">(a)</tag><tag role="typerefnum">item (a)</tag></tags><para xml:id="S0.I1.i1.p1"><p>y</p></para></item></enumerate>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "itemize",
    &[r#"xml:id="S0.I2""#],
    r##"<itemize xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>–</tag><tag role="refnum">–</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I2.i1.p1"><p>x</p></para></item></itemize>"##,
  );
}

/// 57u: `\setlist` replaces and `\setlist*` appends, `list<depth>` keys apply, only a counter command takes `*`, a `\newlist` name is a list (enumitem.sty:573-598, :977-980, :1597-1612; KPE #354).
#[test]
fn enumitem_setlist_replaces_appends_and_depth() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/enumitem_setlist_replaces_appends_and_depth.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I1""#],
    r##"<enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag><text font="bold">1.</text></tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.p1"><p>one</p><enumerate xml:id="S0.I1.i1.I1"><item xml:id="S0.I1.i1.I1.i1"><tags><tag><text font="bold">**</text></tag><tag role="refnum">1**</tag><tag role="typerefnum">item 1**</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>two</p></para></item></enumerate></para></item></enumerate>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "itemize",
    &[r#"xml:id="S0.I2""#],
    r##"<itemize xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag><text font="italic">+</text></tag><tag role="refnum">+</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I2.i1.p1"><p>three</p></para></item></itemize>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I3""#],
    r##"<enumerate xml:id="S0.I3"><item xml:id="S0.I3.i1"><tags><tag>S1</tag><tag role="refnum">S1</tag></tags><para xml:id="S0.I3.i1.p1"><p>four</p></para></item></enumerate>"##,
  );
}

/// 57u: txfonts' `\varmathbb` is its own alphabet, not an alias of the `\mathbb` trigger (txfonts.sty:920; witness 1205.4484; KPE #355).
#[test]
fn varmathbb_is_its_own_alphabet() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/varmathbb_is_its_own_alphabet.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r##"<Math mode="inline" tex="\varmathbb{E}+\varmathbb{R}" text="E + R" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="blackboard" role="UNKNOWN">E</XMTok><XMTok font="blackboard" role="UNKNOWN">R</XMTok></XMApp></XMath></Math>"##,
  );
}

/// 57u: fourier makes `\mathbb` its blackboard alphabet at `\begin{document}` (fourier.sty:300-303; witness 2406.06884; KPE #355).
#[test]
fn fourier_mathbb_is_its_blackboard() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/fourier_mathbb_is_its_blackboard.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // fourier-orns has no binding and stays unloaded (its raw `futs` slots would print as text).
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains(
      "Warning:missing_file:fourier-orns Can't find binding or file for 'fourier-orns.sty'. no \
       dispatcher entry, and raw TeX loading is off (enable with --includestyles)."
    ),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r##"<Math mode="inline" tex="\mathbb{R}" text="R" xml:id="p1.m1"><XMath><XMTok font="blackboard" role="UNKNOWN">R</XMTok></XMath></Math>"##,
  );
}

/// 57u (57t review): an empty `\bmdefine` target is an error, not a silent no-op.
#[test]
fn bmdefine_empty_target_is_reported() {
  let (stderr, _xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/bmdefine_empty_target_is_reported.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    stderr.contains("Error:expected:\\bmdefine Missing the command to define"),
    "{stderr}"
  );
}

/// 57u: physics' trig family takes only a `(…)` argument (`\trigbraces{ m o d() }`, physics.sty:300);
/// 57t's space skip let `\sin[\ell] {e}` take `{e}` (witness 2605.20398; KPE #356).
#[test]
fn physics_trig_takes_no_braced_argument() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_trig_takes_no_braced_argument.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\sin[\ell]{e}^{x}" text="sine@(delimited-[]@(ell) * e ^ x)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="sine" role="OPFUNCTION" scriptpos="post">sin</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual><XMApp><XMTok meaning="delimited-[]"/><XMRef idref="S0.Ex1.m1.1"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok name="ell" role="UNKNOWN" xml:id="S0.Ex1.m1.1">ℓ</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex3.m1""#],
    r##"<Math mode="display" tex="\sin{y}" text="sine@(y)" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="sine" role="OPFUNCTION" scriptpos="post">sin</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\sin[2](x)+\exp{z}" text="(power@(sine, 2))@(x) + exponential@(z)" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMDual><XMApp><XMApp><XMTok meaning="power"/><XMTok meaning="sine"/><XMRef idref="S0.Ex2.m1.1"/></XMApp><XMRef idref="S0.Ex2.m1.2"/></XMApp><XMWrap><XMApp role="OPFUNCTION"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="OPFUNCTION" scriptpos="post">sin</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER" xml:id="S0.Ex2.m1.1">2</XMTok></XMApp><XMTok role="OPEN" stretchy="true">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.2">x</XMTok><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual><XMDual><XMApp><XMTok meaning="exponential"/><XMRef idref="S0.Ex2.m1.3"/></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">exp</XMTok><XMWrap><XMTok role="OPEN" stretchy="true">{</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.3">z</XMTok><XMTok role="CLOSE" stretchy="true">}</XMTok></XMWrap></XMApp></XMDual></XMApp></XMath></Math>"##,
  );
  // A user macro in the group stays expanded, as Perl reverts a math group (57x A/B witness 2605.11861).
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex4.m1""#],
    r##"<Math mode="display" tex="\sin{(4\varphi_{\mathrm{S}})}" text="sine@(4 * varphi _ S)" xml:id="S0.Ex4.m1"><XMath><XMDual><XMApp><XMRef idref="S0.Ex4.m1.1"/><XMRef idref="S0.Ex4.m1.2"/></XMApp><XMApp><XMTok meaning="sine" role="OPFUNCTION" scriptpos="post" xml:id="S0.Ex4.m1.1">sin</XMTok><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="S0.Ex4.m1.2"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="4" role="NUMBER">4</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post2"/><XMTok font="italic" name="varphi" role="UNKNOWN">φ</XMTok><XMTok fontsize="70%" role="UNKNOWN">S</XMTok></XMApp></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMApp></XMDual></XMath></Math>"##,
  );
}

/// 57u: `\pgfsetdash` expands its phase argument as tokens: tikz's `\tikz@dashphase` is a macro
/// (tikz.code.tex:119-137), not the register pgfmath read its string as (RUST-ONLY; beautybook,
/// 4,304 warnings over the 3,003-paper A/B).
#[test]
fn tikz_dash_phase_is_a_macro() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/graphics-tikz/tikz_dash_phase_is_a_macro.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "svg:g",
    &[r#"stroke-dashoffset="1.0pt""#],
    r##"<svg:g fill="#000000" stroke="#000000" stroke-dasharray="3.0pt,2.0pt" stroke-dashoffset="1.0pt" stroke-width="0.4pt" transform="translate(0,1) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)"><svg:path d="M 0 0 L 39.37 0" style="fill:none"/></svg:g>"##,
  );
  // a pattern length held in a macro (`\pgfsetdash{{\dl}{2pt}}{0pt}`) is expanded as tokens
  latexml::util::test::assert_element(
    &xml,
    "svg:g",
    &[r#"stroke-dasharray="3.0pt,2.0pt" stroke-dashoffset="0.0pt""#],
    r##"<svg:g fill="#000000" stroke="#000000" stroke-dasharray="3.0pt,2.0pt" stroke-dashoffset="0.0pt" stroke-width="0.4pt" transform="translate(0,1) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)"><svg:path d="M 0 0 L 39.37 0" style="fill:none"/></svg:g>"##,
  );
}

/// 57u review: txfonts' `\varmathbb` is its own command, so a `\renewcommand{\mathbb}{\varmathbb}` before the load does not loop (txfonts.sty:920; KPE #355).
#[test]
fn varmathbb_survives_an_earlier_renewcommand() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/varmathbb_survives_an_earlier_renewcommand.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r##"<Math mode="inline" tex="\varmathbb{E}" text="E" xml:id="p1.m1"><XMath><XMTok font="blackboard" role="UNKNOWN">E</XMTok></XMath></Math>"##,
  );
}

/// 57u review: newtxmath's `\vmathbb` and `\vvmathbb` (newtxmath.sty:2466-2467, :2577; KPE #355).
#[test]
fn newtxmath_blackboard_variants() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/newtxmath_blackboard_variants.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m1""#],
    r##"<Math mode="inline" tex="\vvmathbb{R}\vmathbb{N}\varmathbb{Z}" text="R * N * Z" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="blackboard" role="UNKNOWN">R</XMTok><XMTok font="blackboard" role="UNKNOWN">N</XMTok><XMTok font="blackboard" role="UNKNOWN">Z</XMTok></XMApp></XMath></Math>"##,
  );
}

/// 57u review: `\setlist` stores with a local `\def` (enumitem.sty:1597-1612; KPE #354).
#[test]
fn enumitem_setlist_in_a_group_is_local() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/enumitem_setlist_in_a_group_is_local.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I1""#],
    r##"<enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.p1"><p>a</p></para></item></enumerate>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I2""#],
    r##"<enumerate xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>(a)</tag><tag role="refnum">(a)</tag><tag role="typerefnum">item (a)</tag></tags><para xml:id="S0.I2.i1.p1"><p>b</p></para></item></enumerate>"##,
  );
}

/// 57u review: `enumerate*` reads `\setlist[enumerate]`'s keys (enumitem.sty:1796-1805; KPE #354).
#[test]
fn enumitem_inline_list_reads_its_base_keys() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/enumitem_inline_list_reads_its_base_keys.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "inline-enumerate",
    &[r#"xml:id="S0.I1""#],
    r##"<inline-enumerate xml:id="S0.I1"><inline-item xml:id="S0.I1.i1"><tags><tag>(a)</tag><tag role="refnum">(a)</tag><tag role="typerefnum">item (a)</tag></tags><text>one </text></inline-item><inline-item xml:id="S0.I1.i2"><tags><tag>(b)</tag><tag role="refnum">(b)</tag><tag role="typerefnum">item (b)</tag></tags><text>two</text></inline-item></inline-enumerate>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I2""#],
    r##"<enumerate xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>(a)</tag><tag role="refnum">(a)</tag><tag role="typerefnum">item (a)</tag></tags><para xml:id="S0.I2.i1.p1"><p>three</p></para></item></enumerate>"##,
  );
}

/// 57u review: a binding list closed by the kernel `\endlist` restores `\@listdepth` once (nih/denselists; latex.ltx:15913; KPE #353).
#[test]
fn list_depth_survives_an_endlist_close() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/list_depth_survives_an_endlist_close.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.i1.p1"><p>a[1]</p></para></item></enumerate><p>b[0]</p><itemize xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I2.i1.p1"><p>c[1]</p></para></item></itemize><p>d[0]</p></para>"##,
  );
}

/// 57v: a physics matrix reverts through its source body, not the `\lx@physics@matrix` wrapper (physics.sty.ltxml:677-692 `I_arg(1)`; RUST-ONLY).
#[test]
fn physics_matrix_reverts_to_its_source() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_matrix_reverts_to_its_source.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\matrixquantity(a &amp; b)" text="matrix@(Array[[a, b]])" xml:id="S0.Ex1.m1"><XMath><XMDual><XMRef idref="S0.Ex1.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMDual xml:id="S0.Ex1.m1.1"><XMApp><XMTok meaning="matrix"/><XMRef idref="S0.Ex1.m1.2"/></XMApp><XMArray xml:id="S0.Ex1.m1.2"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell></XMRow></XMArray></XMDual><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\pmqty{c &amp; d}" text="matrix@(Array[[c, d]])" xml:id="S0.Ex2.m1"><XMath><XMDual><XMRef idref="S0.Ex2.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMDual xml:id="S0.Ex2.m1.1"><XMApp><XMTok meaning="matrix"/><XMRef idref="S0.Ex2.m1.2"/></XMApp><XMArray xml:id="S0.Ex2.m1.2"><XMRow><XMCell align="center"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell><XMCell align="center"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell></XMRow></XMArray></XMDual><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math>"##,
  );
}

/// 57v: a bare `\mqty` prints `()` (physics.sty:74-101; KPE #352).
#[test]
fn physics_bare_mqty_prints_parentheses() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_bare_mqty_prints_parentheses.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="()=x" text="list@() = x" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMDual><XMApp><XMTok meaning="list"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="()=y" text="list@() = y" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMDual><XMApp><XMTok meaning="list"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"##,
  );
}

/// 57v: `\opbraces`' `{…}` argument prints between braces (physics.sty:279-298; KPE #356).
#[test]
fn physics_opbraces_keeps_its_braces() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/parameter-conditional/physics_opbraces_keeps_its_braces.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\det{M}+\tr{A}" text="determinant@(M) + trace@(A)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMDual><XMApp><XMTok meaning="determinant"/><XMRef idref="S0.Ex1.m1.1"/></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">det</XMTok><XMWrap><XMTok role="OPEN" stretchy="true">{</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">M</XMTok><XMTok role="CLOSE" stretchy="true">}</XMTok></XMWrap></XMApp></XMDual><XMDual><XMApp><XMTok meaning="trace"/><XMRef idref="S0.Ex1.m1.2"/></XMApp><XMApp><XMTok role="OPFUNCTION" scriptpos="post">tr</XMTok><XMWrap><XMTok role="OPEN" stretchy="true">{</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">A</XMTok><XMTok role="CLOSE" stretchy="true">}</XMTok></XMWrap></XMApp></XMDual></XMApp></XMath></Math>"##,
  );
}

/// 57v: a generic message is written as `\write` writes it, a control word followed by a space (tex.web §262; KPE #357).
#[test]
fn package_warning_keeps_the_space_after_a_control_word() {
  let (stderr, _xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/string-mouth/package_warning_keeps_the_space_after_a_control_word.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  for line in [
    "Warning:latex:(test) Package test Warning: A \\foo b",
    "Warning:latex:(test) Package test Warning: B \\foo c",
  ] {
    assert!(
      stderr.lines().any(|l| l.trim_end() == line),
      "{line}\n{stderr}"
    );
  }
}

/// 57v review: a message read through the byte mouth (CJKutf8) is written decoded, as `Tokens`'
/// Display writes byte-mouth runs (KPE #357). The repro writes its `part.tex` with
/// `filecontents*` (into the in-memory file store, opened lazily decoded, so CJKutf8's byte mouth
/// still reads it); the tempdir holds `--dest t.xml`.
#[test]
fn package_warning_decodes_byte_mouth_text() {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(
    workdir.path().join("t.tex"),
    include_str!(
      "../../../tools/perfect_kernel/repros/string-mouth/package_warning_decodes_byte_mouth_text.tex"
    ),
  )
  .expect("write t.tex");
  let output = std::process::Command::new(bin)
    .args(["t.tex", "--dest", "t.xml", "--nocomments"])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(output.status.success(), "{stderr}");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    stderr
      .lines()
      .any(|l| l.trim_end().ends_with("Package test Warning: ab é cd")),
    "{stderr}"
  );
}

/// 57w: a leading sign takes a whole term — `-a/b`, `x^{-1/2}`, `-J\sum…` (Perl MathGrammar:246 `SignedTerm : AddOp Term`; RUST-ONLY).
#[test]
fn signed_term_is_a_whole_term() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/math-parse/signed_term_is_a_whole_term.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="-a/b" text="- a / b" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="divide" role="MULOP">/</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex3.m1""#],
    r##"<Math mode="display" tex="E=-J\sum_{ij}s_{i}s_{j}" text="E = - J * (sum _ (i * j))@(s _ i * s _ j)" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">E</XMTok><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">J</XMTok><XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">j</XMTok></XMApp></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">s</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">s</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">j</XMTok></XMApp></XMApp></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="x^{-1/2}+e^{-x/2}" text="x ^ (- 1 / 2) + e ^ (- x / 2)" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">x</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok fontsize="70%" meaning="divide" role="MULOP">/</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">e</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok fontsize="70%" meaning="divide" role="MULOP">/</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex4.m1""#],
    r##"<Math mode="display" tex="S=-\int f" text="S = - integral@(f)" xml:id="S0.Ex4.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">S</XMTok><XMApp><XMTok meaning="minus" role="ADDOP">-</XMTok><XMApp><XMTok mathstyle="display" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57w: a formula the recognizer rejects before its last token warns `unparsed_math` (Perl MathParser.pm:891 `not_parsed`; RUST-ONLY).
#[test]
fn unparsed_formula_is_reported() {
  let (stderr, _xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/unparsed_formula_is_reported.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.lines().any(|l| l.trim_end()
      == "Warning:unparsed_math:UNKNOWN_MULOP_MULOP_UNKNOWN Unparsed math: no parse spans all input \
          tokens for: UNKNOWN:a:1 MULOP:times:2 MULOP:times:3 UNKNOWN:b:4"),
    "{stderr}"
  );
}

/// 57w: a bare bigop is an operand — `\var + y`, `\var + \dd x` (Perl MathGrammar:292, :605-609 `addOpArgs`; RUST-ONLY).
#[test]
fn diffop_before_addop_is_an_operand() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/diffop_before_addop_is_an_operand.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\variation+y" text="variation + y" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" meaning="variation" name="delta" role="DIFFOP">δ</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\variation+\differential x" text="variation + differential@(x)" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" meaning="variation" name="delta" role="DIFFOP">δ</XMTok><XMApp><XMTok meaning="differential" role="DIFFOP">d</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57w: a math ligature may start its container — `\[...\]` is one ldots, a `&:= b` cell an assign (Perl Package.pm:3116-3121; RUST-ONLY).
#[test]
fn ligature_starts_its_container() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/ligature_starts_its_container.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="..." text="ldots" xml:id="S0.Ex1.m1"><XMath><XMTok name="ldots" role="ID">…</XMTok></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"tex="\displaystyle:=b""#],
    r##"<Math mode="inline" tex="\displaystyle:=b" text="absent assign b" xml:id="S0.Ex2.m2"><XMath><XMApp><XMTok meaning="assign" role="RELOP">:=</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"##,
  );
}

/// 57w: an alignment cell's closing punctuation is presentation — the cell dual's content is its bare XMRef (Perl Core/Alignment.pm:372 `rule => 'Anything,'`; RUST-ONLY).
#[test]
fn cell_punctuation_is_presentation() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/cell_punctuation_is_presentation.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\begin{array}[]{ll}x=1,&amp;y=2,\\&#10;z=3.&amp;w.\end{array}" text="Array[[x = 1, y = 2], [z = 3, w]]" xml:id="S0.Ex1.m1"><XMath><XMArray role="ARRAY" vattach="middle"><XMRow><XMCell align="left"><XMDual><XMRef idref="S0.Ex1.m1.1"/><XMWrap><XMApp xml:id="S0.Ex1.m1.1"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMCell><XMCell align="left"><XMDual><XMRef idref="S0.Ex1.m1.2"/><XMWrap><XMApp xml:id="S0.Ex1.m1.2"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok meaning="2" role="NUMBER">2</XMTok></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMDual></XMCell></XMRow><XMRow><XMCell align="left"><XMDual><XMRef idref="S0.Ex1.m1.3"/><XMWrap><XMApp xml:id="S0.Ex1.m1.3"><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">z</XMTok><XMTok meaning="3" role="NUMBER">3</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMCell><XMCell align="left"><XMDual><XMRef idref="S0.Ex1.m1.4"/><XMWrap><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.4">w</XMTok><XMTok role="PERIOD">.</XMTok></XMWrap></XMDual></XMCell></XMRow></XMArray></XMath></Math>"##,
  );
}

/// 57w: a cell of nothing but punctuation keeps it — `XMWrap(absent, .)` (Perl MathParser.pm:683, :696-699; RUST-ONLY).
#[test]
fn punctuation_only_cell_keeps_its_punctuation() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/punctuation_only_cell_keeps_its_punctuation.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\begin{gathered}a=b\\&#10;.\end{gathered}" text="a = b" xml:id="S0.Ex1.m1"><XMath><XMDual><XMDual><XMRef idref="S0.Ex1.m1.1"/><XMWrap><XMApp xml:id="S0.Ex1.m1.1"><XMRef idref="S0.Ex1.m1.2"/><XMRef idref="S0.Ex1.m1.3"/><XMRef idref="S0.Ex1.m1.4"/></XMApp><XMTok role="PERIOD"/></XMWrap></XMDual><XMArray name="gathered"><XMRow><XMCell align="center"><XMApp><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.2">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.3">a</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.4">b</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align="center"><XMWrap><XMTok meaning="absent"/><XMTok role="PERIOD">.</XMTok></XMWrap></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
  );
}

/// 57x: a gathered row's closing `,`/`.` is set aside through its XMRef, so the rows parse as formulae (Perl MathParser.pm:661 realizeXMNode; RUST-ONLY).
#[test]
fn gathered_rows_set_their_punctuation_aside() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/gathered_rows_set_their_punctuation_aside.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\begin{gathered}a=b,\\&#10;c=d.\end{gathered}" text="formulae@(a = b, c = d)" xml:id="S0.Ex1.m1"><XMath><XMDual><XMDual><XMRef idref="S0.Ex1.m1.1"/><XMWrap><XMDual xml:id="S0.Ex1.m1.1"><XMApp><XMTok meaning="formulae"/><XMRef idref="S0.Ex1.m1.2"/><XMRef idref="S0.Ex1.m1.3"/></XMApp><XMWrap><XMApp xml:id="S0.Ex1.m1.2"><XMRef idref="S0.Ex1.m1.5"/><XMRef idref="S0.Ex1.m1.6"/><XMRef idref="S0.Ex1.m1.7"/></XMApp><XMTok role="PUNCT"/><XMApp xml:id="S0.Ex1.m1.3"><XMRef idref="S0.Ex1.m1.9"/><XMRef idref="S0.Ex1.m1.10"/><XMRef idref="S0.Ex1.m1.11"/></XMApp></XMWrap></XMDual><XMTok role="PERIOD"/></XMWrap></XMDual><XMArray name="gathered"><XMRow><XMCell align="center"><XMWrap><XMApp xml:id="S0.Ex1.m1.4"><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.5">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.6">a</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.7">b</XMTok></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMCell></XMRow><XMRow><XMCell align="center"><XMWrap><XMApp xml:id="S0.Ex1.m1.8"><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.9">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.10">c</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.11">d</XMTok></XMApp><XMTok role="PERIOD">.</XMTok></XMWrap></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
  );
}

/// 57x: an XMRef to set-aside punctuation becomes the bare mark, as Perl's shallow `cloneNode` (MathParser.pm:281-286).
#[test]
fn stray_punctuation_ref_is_replaced() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/stray_punctuation_ref_is_replaced.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\begin{gathered}a=b,\\&#10;c=d\end{gathered}" text="formulae@(a = b, c = d)" xml:id="S0.Ex1.m1"><XMath><XMDual><XMDual><XMApp><XMTok meaning="formulae"/><XMRef idref="S0.Ex1.m1.1"/><XMRef idref="S0.Ex1.m1.2"/></XMApp><XMWrap><XMApp xml:id="S0.Ex1.m1.1"><XMRef idref="S0.Ex1.m1.4"/><XMRef idref="S0.Ex1.m1.5"/><XMRef idref="S0.Ex1.m1.6"/></XMApp><XMTok role="PUNCT"/><XMApp xml:id="S0.Ex1.m1.2"><XMRef idref="S0.Ex1.m1.7"/><XMRef idref="S0.Ex1.m1.8"/><XMRef idref="S0.Ex1.m1.9"/></XMApp></XMWrap></XMDual><XMArray name="gathered"><XMRow><XMCell align="center"><XMWrap><XMApp xml:id="S0.Ex1.m1.3"><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.4">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.5">a</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.6">b</XMTok></XMApp><XMTok role="PUNCT">,</XMTok></XMWrap></XMCell></XMRow><XMRow><XMCell align="center"><XMApp><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.7">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.8">c</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.9">d</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math>"##,
  );
}

/// 57x: adjacent bigops apply in turn — `\partial\partial f` is ∂@(∂@(f)) (Perl MathGrammar:292; RUST-ONLY `composed_bigop` removed).
#[test]
fn stacked_bigops_apply_in_turn() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/stacked_bigops_apply_in_turn.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\partial\partial f" text="partial-differential@(partial-differential@(f))" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="partial-differential" name="partial" role="DIFFOP">∂</XMTok><XMApp><XMTok meaning="partial-differential" name="partial" role="DIFFOP">∂</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57x: a pre-scripted bigop is an operand — `{}^a\sum + b` (Perl MathGrammar:292 preScripted['bigop']; RUST-ONLY).
#[test]
fn prescripted_bigop_is_an_operand() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/prescripted_bigop_is_an_operand.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="{}^{a}\sum+b" text="^ a sum + b" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre1"/><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="{}^{a}\sum\sum b" text="(^ a sum)@(sum@(b))" xml:id="S0.Ex2.m1"><XMath><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="pre1"/><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp><XMApp><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57x: `\setlist` expands its names before sorting lists from levels (enumitem.sty:1678
/// `\protected@edef`; 2605.31510's clevethm.sty; RUST-ONLY, a 57u regression).
#[test]
fn setlist_names_are_expanded() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/setlist_names_are_expanded.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.i1""#],
    r##"<item xml:id="S0.I1.i1"><tags><tag>(i)</tag><tag role="refnum">(i)</tag><tag role="typerefnum">item (i)</tag></tags><para xml:id="S0.I1.i1.p1"><p>x</p></para></item>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I2.i1""#],
    r##"<item xml:id="S0.I2.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I2.i1.p1"><p>y</p></para></item>"##,
  );
}

/// 57x: a script on the base before a bigop stays the base's — `\|f\|_2\int g` is norm_2 * ∫g (only a FLOAT script starts a pre-scripted bigop; Perl MathGrammar:292; RUST-ONLY).
#[test]
fn base_script_is_not_a_bigop_prescript() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/base_script_is_not_a_bigop_prescript.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\|f\|_{2}\int g" text="(norm@(f)) _ 2 * integral@(g)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex1.m1.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">f</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp><XMApp><XMTok mathstyle="display" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok font="italic" role="UNKNOWN">g</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\|h\|^{3}\int_{0}^{1}f\,dt" text="(norm@(h)) ^ 3 * ((integral _ 0) ^ 1)@(f * differential-d@(t))" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMDual><XMApp><XMTok meaning="norm"/><XMRef idref="S0.Ex2.m1.1"/></XMApp><XMWrap><XMTok name="||" role="OPEN">‖</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.1">h</XMTok><XMTok name="||" role="CLOSE">‖</XMTok></XMWrap></XMDual><XMTok fontsize="70%" meaning="3" role="NUMBER">3</XMTok></XMApp><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok mathstyle="display" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok fontsize="70%" meaning="0" role="NUMBER">0</XMTok></XMApp><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">f</XMTok><XMApp><XMTok font="italic" meaning="differential-d" role="DIFFOP">d</XMTok><XMTok font="italic" role="UNKNOWN">t</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57x: a function or operator, scripted or not, before a bigop is a factor — `\min_\theta\sum_i \ell_i` is min_θ * ∑… (Perl `Factor moreFactors`; RUST-ONLY).
#[test]
fn function_before_a_bigop_is_a_factor() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/function_before_a_bigop_is_a_factor.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    r##"<Math mode="display" tex="\min_{\theta}\sum_{i=1}^{n}\ell_{i}" text="minimum _ theta * ((sum _ (i = 1)) ^ n)@(ell _ i)" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok meaning="minimum" role="OPFUNCTION" scriptpos="mid">min</XMTok><XMTok font="italic" fontsize="70%" name="theta" role="UNKNOWN">θ</XMTok></XMApp><XMApp><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="mid1"/><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="mid1"/><XMTok mathstyle="display" meaning="sum" role="SUMOP" scriptpos="mid">∑</XMTok><XMApp><XMTok fontsize="70%" meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp></XMApp><XMTok font="italic" fontsize="70%" role="UNKNOWN">n</XMTok></XMApp><XMApp><XMTok role="SUBSCRIPTOP" scriptpos="post1"/><XMTok name="ell" role="UNKNOWN">ℓ</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex2.m1""#],
    r##"<Math mode="display" tex="\log\int f" text="logarithm * integral@(f)" xml:id="S0.Ex2.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="logarithm" role="OPFUNCTION">log</XMTok><XMApp><XMTok mathstyle="display" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex3.m1""#],
    r##"<Math mode="display" tex="\nabla\int f" text="nabla * integral@(f)" xml:id="S0.Ex3.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok name="nabla" role="OPERATOR">∇</XMTok><XMApp><XMTok mathstyle="display" meaning="integral" name="int" role="INTOP">∫</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMApp></XMath></Math>"##,
  );
}

/// 57y: an itemize inside a kernel `\list` labels by `\@itemdepth` — `\labelitemi` (latex.ltx:16068-16075; PERL-ORIGIN, pdflatex the oracle).
#[test]
fn itemize_in_a_list_takes_the_first_label() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/itemize_in_a_list_takes_the_first_label.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.i1.I1.i1""#],
    r##"<item xml:id="S0.I1.i1.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>y</p></para></item>"##,
  );
}

/// 57y: `\ArrowBetweenLines` ends an empty row, then sets the arrow row; `align`/`gather` number both (mathtools.sty:1299-1322; RUST-ONLY).
#[test]
fn arrow_rows_in_align_are_numbered() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/arrow_rows_in_align_are_numbered.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S0.E9""#],
    r##"<equation xml:id="S0.E9"><tags><tag>(9)</tag><tag role="refnum">9</tag></tags><Math mode="display" tex="e=f" text="e = f" xml:id="S0.E9.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">e</XMTok><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMath></Math></equation>"##,
  );
}

/// 57y: svjour3's `{description}[]` counts in `@desc` and labels by `\descriptionlabel` (Perl sv_support.sty.ltxml:311-315; RUST-ONLY).
#[test]
fn svjour3_description_is_a_description_list() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/svjour3_description_is_a_description_list.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "description",
    &[r#"xml:id="S0.I1""#],
    r##"<description xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag><text font="bold">x</text></tag><tag role="typerefnum">item x</tag></tags><para xml:id="S0.I1.ix1.p1"><p>a</p><itemize xml:id="S0.I1.ix1.I1"><item xml:id="S0.I1.ix1.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.ix1.I1.i1.p1"><p>b</p></para></item></itemize></para></item></description>"##,
  );
}

/// 57y: enumitem labels star by expansion — `\roman{*}`, `\csname roman\endcsname*`, a macro alias; `**` stays text (enumitem.sty:910-936; RUST-ONLY, a 57u regression).
#[test]
fn enumitem_label_star_is_read_by_expansion() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/enumitem_label_star_is_read_by_expansion.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "enumerate",
    &[r#"xml:id="S0.I1""#],
    r##"<enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>(i)</tag><tag role="refnum">(i)</tag><tag role="typerefnum">item (i)</tag></tags><para xml:id="S0.I1.i1.p1"><p>one</p></para></item><item xml:id="S0.I1.i2"><tags><tag>(ii)</tag><tag role="refnum">(ii)</tag><tag role="typerefnum">item (ii)</tag></tags><para xml:id="S0.I1.i2.p1"><p>two</p></para></item></enumerate>"##,
  );
  for (id, whole) in [
    (
      "S0.I2",
      r##"<enumerate xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>(i)</tag><tag role="refnum">(i)</tag><tag role="typerefnum">item (i)</tag></tags><para xml:id="S0.I2.i1.p1"><p>three</p></para></item></enumerate>"##,
    ),
    (
      "S0.I3",
      r##"<enumerate xml:id="S0.I3"><item xml:id="S0.I3.i1"><tags><tag>(i)</tag><tag role="refnum">(i)</tag><tag role="typerefnum">item (i)</tag></tags><para xml:id="S0.I3.i1.p1"><p>four</p></para></item></enumerate>"##,
    ),
    (
      "S0.I4",
      r##"<enumerate xml:id="S0.I4"><item xml:id="S0.I4.i1"><tags><tag>**</tag><tag role="refnum">**</tag><tag role="typerefnum">item **</tag></tags><para xml:id="S0.I4.i1.p1"><p>five</p></para></item></enumerate>"##,
    ),
    (
      "S0.I5",
      r##"<enumerate xml:id="S0.I5"><item xml:id="S0.I5.i1"><tags><tag>a)</tag><tag role="refnum">a)</tag><tag role="typerefnum">item a)</tag></tags><para xml:id="S0.I5.i1.p1"><p>six</p></para></item></enumerate>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "enumerate", &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 57y: `\renewlist` registers its list for `\setlist` (enumitem.sty:1730-1731; RUST-ONLY, a 57u regression).
#[test]
fn renewlist_registers_its_list() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/renewlist_registers_its_list.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "description",
    &[r#"xml:id="S0.I1""#],
    r##"<description xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag><text font="bold">1</text></tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I1.ix1.p1"><p>note</p></para></item></description>"##,
  );
}

/// 57y review: enumitem reads an itemize label as written — only an enumerate's label is expanded for `*` counters (`\enit@enumerate@i`, enumitem.sty:1396-1397); a TikZ bullet does not survive `\xdef` (RUST-ONLY, a 57y first-cut regression).
#[test]
fn itemize_label_is_used_as_written() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/itemize_label_is_used_as_written.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.i1""#],
    r##"<item xml:id="S0.I1.i1"><tags><tag><picture height="6.09" width="6.09" xml:id="S0.I1.i1.pic1"><svg:svg height="6.09" overflow="visible" version="1.1" viewBox="0 0 6.09 6.09" width="6.09"><svg:g fill="#FF0000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,6.09) matrix(1 0 0 -1 0 0) translate(3.04,0) translate(0,3.04)"><svg:path d="M 0 0 M 2.77 0 C 2.77 1.53 1.53 2.77 0 2.77 C -1.53 2.77 -2.77 1.53 -2.77 0 C -2.77 -1.53 -1.53 -2.77 0 -2.77 C 1.53 -2.77 2.77 -1.53 2.77 0 Z M 0 0"/></svg:g></svg:svg></picture></tag><tag role="refnum"><picture height="6.09" width="6.09" xml:id="S0.I1.i1.pic2"><svg:svg height="6.09" overflow="visible" version="1.1" viewBox="0 0 6.09 6.09" width="6.09"><svg:g fill="#FF0000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,6.09) matrix(1 0 0 -1 0 0) translate(3.04,0) translate(0,3.04)"><svg:path d="M 0 0 M 2.77 0 C 2.77 1.53 1.53 2.77 0 2.77 C -1.53 2.77 -2.77 1.53 -2.77 0 C -2.77 -1.53 -1.53 -2.77 0 -2.77 C 1.53 -2.77 2.77 -1.53 2.77 0 Z M 0 0"/></svg:g></svg:svg></picture></tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>tikzlabel</p></para></item>"##,
  );
}

/// 57y review: an itemize inside a kernel `\list`/`\trivlist` takes enumitem's first-level keys and keeps its own `label=` (enumitem `\@itemdepth`, :519-521; PERL-ORIGIN, KNOWN_PERL_ERRORS #358).
#[test]
fn itemize_label_follows_the_itemize_depth() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/itemize_label_follows_the_itemize_depth.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, whole) in [
    (
      "S0.I1.i1.I1.i1",
      r##"<item xml:id="S0.I1.i1.I1.i1"><tags><tag>–</tag><tag role="refnum">–</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>dashlabel</p></para></item>"##,
    ),
    (
      "S0.I3.i1",
      r##"<item xml:id="S0.I3.i1"><tags><tag><Math mode="inline" tex="\star" text="star" xml:id="S0.I3.i1.m1"><XMath><XMTok name="star" role="MULOP">⋆</XMTok></XMath></Math></tag><tag role="refnum"><Math mode="inline" tex="\star" text="star" xml:id="S0.I3.i1.m2"><XMath><XMTok name="star" role="MULOP">⋆</XMTok></XMath></Math></tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I3.i1.p1"><p>starlabel</p></para></item>"##,
    ),
    (
      "S0.I4.i1.I1.i1",
      r##"<item xml:id="S0.I4.i1.I1.i1"><tags><tag><Math mode="inline" tex="\circ" text="compose" xml:id="S0.I4.i1.I1.i1.m1"><XMath><XMTok meaning="compose" name="circ" role="MULOP">∘</XMTok></XMath></Math></tag><tag role="refnum"><Math mode="inline" tex="\circ" text="compose" xml:id="S0.I4.i1.I1.i1.m2"><XMath><XMTok meaning="compose" name="circ" role="MULOP">∘</XMTok></XMath></Math></tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I4.i1.I1.i1.p1"><p>inner</p></para></item>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "item", &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 57y review: paralist's `[<label>]` names `\labelitem<\@itemdepth>` (paralist.sty:281-282), so a compactitem inside a `\list` keeps it (PERL-ORIGIN, KNOWN_PERL_ERRORS #358).
#[test]
fn paralist_label_follows_the_itemize_depth() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/paralist_label_follows_the_itemize_depth.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "item",
    &[r#"xml:id="S0.I1.i1.I1.i1""#],
    r##"<item xml:id="S0.I1.i1.I1.i1"><tags><tag>–</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>dashlabel</p></para></item>"##,
  );
}

/// 57y review: elsarticle's itemize labels by `\@itemdepth` like the kernel's (elsarticle.cls:1143-1150); Perl counted it as an enumerate, "1."/"(a)" (SURPASS, OXIDIZED_DESIGN #346, KNOWN_PERL_ERRORS #360).
#[test]
fn elsarticle_itemize_has_bullets() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/elsarticle_itemize_has_bullets.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (tag, id, whole) in [
    (
      "itemize",
      "S0.I1",
      r##"<itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>top</p></para></item></itemize>"##,
    ),
    (
      "enumerate",
      "S0.I2",
      r##"<enumerate xml:id="S0.I2"><item xml:id="S0.I2.i1"><tags><tag>1.</tag><tag role="autoref">item 1</tag><tag role="refnum">1</tag><tag role="typerefnum">item 1</tag></tags><para xml:id="S0.I2.i1.p1"><p>one</p><itemize xml:id="S0.I2.i1.I1"><item xml:id="S0.I2.i1.I1.i1"><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I2.i1.I1.i1.p1"><p>nested</p></para></item></itemize></para></item></enumerate>"##,
    ),
    (
      "itemize",
      "S0.I3",
      r##"<itemize xml:id="S0.I3"><item xml:id="S0.I3.i1"><tags><tag>•</tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I3.i1.p1"><p>a</p><itemize xml:id="S0.I3.i1.I2"><item xml:id="S0.I3.i1.I2.i1"><tags><tag><text font="bold">–</text></tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I3.i1.I2.i1.p1"><p>b</p></para></item></itemize></para></item></itemize>"##,
    ),
    (
      "itemize",
      "S0.I4",
      r##"<itemize xml:id="S0.I4"><item xml:id="S0.I4.i1"><tags><tag><Math mode="inline" tex="\ast" text="ast" xml:id="S0.I4.i1.m1"><XMath><XMTok name="ast" role="MULOP">∗</XMTok></XMath></Math></tag><tag role="autoref">item </tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I4.i1.p1"><p>star</p></para></item></itemize>"##,
    ),
  ] {
    latexml::util::test::assert_element(&xml, tag, &[&format!(r#"xml:id="{id}""#)], whole);
  }
}

/// 57y A/B: under the pdfTeX byte mouth the continuation bytes are protected (utf8.def:176), so an `\edef` over `•` keeps the character, and an enumerate label `•\arabic*` under CJKutf8 is "•1" (RUST-ONLY).
#[test]
fn byte_mouth_continuation_bytes_are_protected() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/unicode-catcodes/byte_mouth_continuation_bytes_are_protected.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>a • b • c</p><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•1</tag><tag role="refnum">•1</tag><tag role="typerefnum">item •1</tag></tags><para xml:id="S0.I1.i1.p1"><p>one</p></para></item></enumerate></para>"##,
  );
}

/// 57y: `\textcircled` circles its typeset argument, a text accent (latex.ltx:10057); Perl circled the raw tokens, "\small2⃝" (PERL-ORIGIN, KNOWN_PERL_ERRORS #361; witness latex-via-exemplos).
#[test]
fn textcircled_circles_its_typeset_argument() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/textcircled_circles_its_typeset_argument.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>② Ⓐ</p><enumerate xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>①</tag><tag role="refnum">①</tag><tag role="typerefnum">item ①</tag></tags><para xml:id="S0.I1.i1.p1"><p>one</p></para></item><item xml:id="S0.I1.i2"><tags><tag>②</tag><tag role="refnum">②</tag><tag role="typerefnum">item ②</tag></tags><para xml:id="S0.I1.i2.p1"><p>two</p></para></item></enumerate><p>ⓒ</p></para>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r##"<para xml:id="p2"><p>[EP]① first.</p></para>"##,
  );
}

/// 57y review: an enumerate label whose `\protected@xdef` leaves an unbalanced body (an unprotected `\tikz`, where pdflatex errors too) is kept as written instead of ending the document with `Fatal:Stomach:Misdefined` (RUST-ONLY).
#[test]
fn unbalanced_label_expansion_keeps_the_label() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/list-structure/unbalanced_label_expansion_keeps_the_label.tex"
    ),
    None,
  );
  // The expansion's own errors stay (pdflatex errors and hangs on this input); what the fallback
  // guarantees is that no Fatal ends the document.
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(error_count(&stderr) > 0, "{stderr}");
  for id in ["S0.I1.i1", "S0.I1.i2"] {
    assert!(
      xml.contains(&format!(
        r#"<tag><picture height="41.6" width="41.6" xml:id="{id}.pic1">"#
      )),
      "{xml}"
    );
  }
  assert!(xml.contains("after</p>"), "{xml}");
}

/// 57aa: a `\let` copy still identical to its source is dumped as an alias and re-`\let` at load
/// (Perl TeX_Job.pool.ltxml:185-197 `Lt`), so expl3's `\tex_jobname:D` — and `\c_sys_jobname_str`
/// — are the run's `\jobname`, not the dump run's empty one (RUST-ONLY). Was `[t][][macro:-¿]…`.
#[test]
fn tex_jobname_is_the_jobname() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/expl3/tex_jobname_is_the_jobname.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r##"<para xml:id="p1"><p>[t][t][macro:-¿t][t]</p></para>"##,
  );
}

/// 57aa: under a preload (`ar5iv.sty`, as cortex runs; `latexml.sty`) the LaTeX format — and with
/// it expl3's copies of `\jobname` — loads before the job's `\jobname` is installed; the install
/// re-points those copies (`state::rebind_let_copies`). Without it: `[t][][macro:-¿][]`.
#[test]
fn tex_jobname_is_the_jobname_under_a_preload() {
  for (preload, p) in [
    ("ar5iv.sty", r#"<p xml:id="p1.1">"#),
    ("[rawstyles,rawclasses]latexml.sty", "<p>"),
  ] {
    let (stderr, xml) = convert_with(
      include_str!("../../../tools/perfect_kernel/repros/expl3/tex_jobname_is_the_jobname.tex"),
      Some(preload),
    );
    assert_eq!(error_count(&stderr), 0, "{preload}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{preload}: {stderr}");
    latexml::util::test::assert_element(
      &xml,
      "para",
      &[r#"xml:id="p1""#],
      &format!(r#"<para xml:id="p1">{p}[t][t][macro:-¿t][t]</p></para>"#),
    );
  }
}

/// 57ab: a single char in `<mi>` is italic by default (MathML Core `text-transform: math-auto`),
/// so a font-less one — `\mathrm{d}`, `\mathrm{e}` — says `mathvariant="normal"`, as Perl's
/// MathML.pm:717-719 does for every single-char `mi` without an italic variant; Rust did so only
/// for a token with a `name`, rendering every upright differential italic (RUST-ONLY).
#[test]
fn upright_single_letter_is_normal() {
  let (stderr, html) = convert_html(include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/upright_single_letter_is_normal.tex"
  ));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "mrow",
    &[],
    "<mrow><mi mathvariant=\"normal\">d</mi><mo>\u{2062}</mo><mi>x</mi></mrow>",
  );
  latexml::util::test::assert_element(
    &html,
    "msup",
    &[],
    r#"<msup><mi mathvariant="normal">e</mi><mi>x</mi></msup>"#,
  );
}

/// 57ab: sticky-font letters join into one token only while the text is ASCII `[0-9a-zA-Z]` and
/// starts with a letter (Perl Base_XMath.pool.ltxml:443-458), so `\mathrm{éa}` stays two
/// upright letters; Rust tested Unicode `is_alphanumeric` and made one token "éa" (RUST-ONLY).
#[test]
fn letters_ligature_is_ascii() {
  let (stderr, xml) = convert_with(
    include_str!("../../../tools/perfect_kernel/repros/math-parse/letters_ligature_is_ascii.tex"),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, tex, text, first, second) in [
    ("p1.m1", "éa", "é * a", "é", "a"),
    ("p1.m2", "aé", "a * é", "a", "é"),
  ] {
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<Math mode="inline" tex="\mathrm{{{tex}}}" text="{text}" xml:id="{id}"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok role="UNKNOWN">{first}</XMTok><XMTok role="UNKNOWN">{second}</XMTok></XMApp></XMath></Math>"#
      ),
    );
  }
}

/// 57ab: a DefMath constructor digests in its own group unless `nogroup` (Perl Package.pm:1706-1711)
/// and takes its font there (Constructor.pm:97-104), so the upright font of a `\DeclareMathOperator`
/// whose body names a control sequence stays with the operator: the letters after `\E` and `\argmax`
/// keep `font="italic"` (and render as plain italic `<mi>`), as in Perl; `\argmax`'s own letters are
/// upright, its body presented through `\operatorname` (amsopn.sty.ltxml:25). 57ae: the wrapped
/// DefMath (Package.pm:1786-1813) gives Perl's `<XMApp name="argmax" …>` and `tex=` (the scripted
/// `XMApp`'s `role="OPERATOR"` is Rust's post-parse role copy, `parser.rs`, not Perl's).
#[test]
fn declaremathoperator_keeps_the_following_letters_italic() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/declaremathoperator_keeps_the_following_letters_italic.tex"
  );
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok font="blackboard" name="E" role="OPFUNCTION" scriptpos="post">E</XMTok><XMTok font="italic" role="UNKNOWN">X</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="p1.m2""#],
    concat!(
      r#"<Math content-tex="\argmax_{z}f(z)" mode="inline" tex="\operatorname*{arg\,max}_{z}f(z)" text="(argmax _ z)@(f) * z" xml:id="p1.m2"><XMath><XMApp>"#,
      "<XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMApp><XMApp role=\"OPERATOR\">",
      r#"<XMTok role="SUBSCRIPTOP" scriptpos="post1"/>"#,
      "<XMApp name=\"argmax\" role=\"OPERATOR\" scriptpos=\"post\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>",
      r#"<XMTok role="UNKNOWN" rpadding="1.7pt">arg</XMTok><XMTok role="UNKNOWN">max</XMTok></XMApp>"#,
      r#"<XMTok font="italic" fontsize="70%" role="UNKNOWN">z</XMTok></XMApp>"#,
      r#"<XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp><XMDual><XMRef idref="p1.m2.1"/><XMWrap>"#,
      r#"<XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m2.1">z</XMTok>"#,
      r#"<XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#
    ),
  );
  let (stderr, html) = convert_html(tex);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "mrow",
    &[],
    "<mrow><mrow><mi>\u{1D53C}</mi><mo lspace=\"0.167em\">\u{2061}</mo><mi>X</mi></mrow><mo>+</mo><mi>y</mi></mrow>",
  );
  latexml::util::test::assert_element(
    &html,
    "msub",
    &[],
    "<msub><mrow><mi>arg</mi><mo lspace=\"0.170em\">\u{2062}</mo><mi>max</mi></mrow><mi>z</mi></msub>",
  );
}

/// 57ab: the bold of `\boldmath` / `\boldsymbol` reaches a DefMath constructor's accent, read as
/// Perl reads it, at digestion (Constructor.pm:97-104; Package.pm:1726-1728).
#[test]
fn defmath_accent_takes_the_digestion_font() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/defmath_accent_takes_the_digestion_font.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, tex, name, glyph, base) in [
    ("p1.m1", r"\hat{x}", "hat", "^", "x"),
    ("p1.m2", r"\boldsymbol{\tilde{y}}", "tilde", "~", "y"),
  ] {
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<Math mode="inline" tex="{tex}" text="{name}@({base})" xml:id="{id}"><XMath><XMApp><XMTok font="bold" name="{name}" role="OVERACCENT" stretchy="false">{glyph}</XMTok><XMTok font="bold italic" role="UNKNOWN">{base}</XMTok></XMApp></XMath></Math>"#
      ),
    );
  }
}

/// 57ab: in an italic context the italic is the context's (`<text font="italic">`, a theorem
/// body), so a letter there is a plain `<mi>`, found by the inherited font as Perl's
/// `$LaTeXML::MathML::FONT` (MathML.pm:278); an upright one says `normal`.
#[test]
fn letters_in_an_italic_context_stay_plain_mi() {
  let (stderr, html) = convert_html(
    "\\documentclass{article}\n\\newtheorem{thm}{Theorem}\n\\begin{document}\n\\textit{$x$} $\\mathrm{d}$\n\\begin{thm}$y$\\end{thm}\n\\end{document}\n",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, alt, mi) in [
    ("p1.m1", "x", "<mi>x</mi>"),
    ("p1.m2", "\\mathrm{d}", "<mi mathvariant=\"normal\">d</mi>"),
    ("Thmthm1.p1.m1", "y", "<mi>y</mi>"),
  ] {
    latexml::util::test::assert_element(
      &html,
      "math",
      &[&format!(r#"id="{id}""#)],
      &format!(r#"<math alttext="{alt}" class="ltx_Math" display="inline" id="{id}">{mi}</math>"#),
    );
  }
}

/// 57ab: a comment between sticky-font letters does not stop them joining (Perl
/// Base_XMath.pool.ltxml:453-456 steps over comment nodes; the applier keeps them,
/// Core/Document.pm:1194-1199) — under `--comments`, `\mathrm{a%c` + `b}` is one token "ab".
#[test]
fn letters_ligature_joins_across_a_comment() {
  let (stderr, xml) = convert_args(
    "\\documentclass{article}\n\\begin{document}\n$\\mathrm{a%c\nb}$\n\\end{document}\n",
    &["--comments"],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "XMath",
    &[],
    r#"<XMath><XMTok role="UNKNOWN">ab</XMTok></XMath>"#,
  );
}

/// 57ab: a starred `\DeclareMathOperator` takes Perl's `scriptpos => \&doScriptpos`
/// (amsopn.sty.ltxml:26; TeX_Math.pool.ltxml:350) on the constructor path too: `mid` in display,
/// so the subscript sits below (`mid1`), `post` inline. Rust read it only for a one-token box.
/// 57ae: Perl's wrapped DefMath — the name and `tex="\operatorname*{…}"` (the scripted `XMApp`'s
/// `role` is Rust's post-parse copy); a body naming a control sequence:
/// `declaremathoperator_is_perls_wrapped_operator`.
#[test]
fn starred_operator_puts_limits_below_in_display() {
  let tex = "\\documentclass{article}\n\\usepackage{amsmath}\n\\DeclareMathOperator*{\\argmin}{argmin}\n\\begin{document}\n\\[\\argmin_x f\\] $\\argmin_x f$\n\\end{document}\n";
  let (stderr, xml) = convert_with(tex, None);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, mode, pos) in [("S0.Ex1.m1", "display", "mid"), ("p1.m1", "inline", "post")] {
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<Math content-tex="\argmin_{{x}}f" mode="{mode}" tex="\operatorname*{{argmin}}_{{x}}f" text="(argmin _ x)@(f)" xml:id="{id}"><XMath><XMApp><XMApp role="OPERATOR"><XMTok role="SUBSCRIPTOP" scriptpos="{pos}1"/><XMTok name="argmin" role="OPERATOR" scriptpos="{pos}">argmin</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">f</XMTok></XMApp></XMath></Math>"#
      ),
    );
  }
}

/// 57ac: a counter reset list is assigned globally — latex.ltx:10156-10172 builds `\cl@<within>`
/// with `\@cons`/`\xdef`, Perl assigns it 'global' (latex_constructs.pool.ltxml:3011, :3020) — so a
/// reset registered or removed inside a group outlives it (`\@addtoreset`, `\@removefromreset`,
/// `\counterwithin`, `\counterwithout`, one pair of helpers now). RUST-ONLY: "[2][0]".
#[test]
fn counter_reset_lists_are_global() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/counter_reset_lists_are_global.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>[0][5]</p></para>"#,
  );
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\newcounter{foo}\\newcounter{bar}\n\\begin{document}\n\\setcounter{foo}{2}{\\counterwithin{foo}{bar}}\\stepcounter{bar}[\\arabic{foo}]\n\\setcounter{foo}{3}{\\counterwithout{foo}{bar}}\\stepcounter{bar}[\\arabic{foo}]\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>[0]\n[3]</p></para>",
  );
}

/// Assert each float's whole `<tags>` element, found after its `xml:id` and before any nested
/// or following float starts (so a float without tags of its own cannot borrow the next one's).
fn assert_float_tags(xml: &str, expected: &[(&str, &str)]) {
  for (id, tags) in expected {
    let at = xml
      .find(&format!(r#"xml:id="{id}""#))
      .unwrap_or_else(|| panic!("{id}\n{xml}"));
    let rest = &xml[at..];
    let own = rest
      .find("<tags>")
      .unwrap_or_else(|| panic!("{id}: no <tags>\n{xml}"));
    let next_float = ["<figure", "<table", "<float"]
      .iter()
      .filter_map(|start| rest.find(start))
      .min()
      .unwrap_or(usize::MAX);
    assert!(
      own < next_float,
      "{id}: its <tags> belongs to a later float\n{xml}"
    );
    latexml::util::test::assert_element(rest, "tags", &[], tags);
  }
}

/// 57ad: caption's `\ContinuedFloat` (caption.sty:496-536) keeps the float's number and the
/// sub-float letters going: the continued part is Figure 1 again with (c), (d), its ids suffixed
/// by `\@alph\c@continuedfloat` (caption's `\theH` append), and the next float is Figure 2
/// (pdflatex). caption suppresses the continued float's step (`step_float_counter`); Perl's
/// binding is a no-op (KNOWN_PERL_ERRORS #362): refnums ran 1, 1a, 1b, 2, 2a, 2b, 3.
#[test]
fn continuedfloat_keeps_the_number_and_the_letters() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/continuedfloat_keeps_the_number_and_the_letters.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let figure = |n: &str| {
    format!(
      r#"<tags><tag><text fontsize="90%">Figure {n}</text></tag><tag role="refnum">{n}</tag><tag role="typerefnum">Figure {n}</tag></tags>"#
    )
  };
  let (f1, f2) = (figure("1"), figure("2"));
  assert_float_tags(&xml, &[
    ("S0.F1", f1.as_str()),
    (
      "S0.F1.sf1",
      r#"<tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags>"#,
    ),
    ("S0.F1a", f1.as_str()),
    (
      "S0.F1a.sf3",
      r#"<tags><tag><text fontsize="90%">(c)</text></tag><tag role="refnum">1c</tag></tags>"#,
    ),
    ("S0.F2", f2.as_str()),
  ]);
}

/// 57ad: a continuation without a numbered caption (none, or `\caption*`) consumes no number —
/// the next figure is Figure 2 (pdflatex); `\ContinuedFloat*` starts a new number (Figure 3) that
/// a later `\ContinuedFloat` continues; a continuation's count restarts at every real step, so the
/// first continuation in each chapter is suffixed "a" (caption.sty:577-579). subfig's
/// `\ContinuedFloat` (subfig.sty:581-590) continues the letters, the sub-floats nested under the
/// continued float's id.
#[test]
fn continuedfloat_suppresses_one_step_per_real_float() {
  let figure = |n: &str| {
    format!(
      r#"<tags><tag>Figure {n}</tag><tag role="refnum">{n}</tag><tag role="typerefnum">Figure {n}</tag></tags>"#
    )
  };
  for (tex, expected) in [
    (
      "\\documentclass{article}\n\\usepackage{caption}\n\\begin{document}\n\\begin{figure}X\\caption{A}\\end{figure}\n\\begin{figure}\\ContinuedFloat Y\\end{figure}\n\\begin{figure}\\ContinuedFloat Z\\caption*{Starred}\\end{figure}\n\\begin{figure}W\\caption{B}\\label{fb}\\end{figure}\n\\begin{figure}\\ContinuedFloat* V\\caption{C}\\end{figure}\n\\begin{figure}\\ContinuedFloat U\\caption{D}\\end{figure}\nSee \\ref{fb}.\n\\end{document}\n",
      vec![
        ("S0.F2", figure("2")),
        ("S0.F3a", figure("3")),
        ("S0.F3b", figure("3")),
      ],
    ),
    (
      "\\documentclass{report}\n\\usepackage{caption}\n\\begin{document}\n\\chapter{One}\n\\begin{figure}A\\caption{a}\\end{figure}\\begin{figure}B\\caption{b}\\end{figure}\\begin{figure}C\\caption{c}\\end{figure}\n\\begin{figure}\\ContinuedFloat C2\\caption{c2}\\end{figure}\n\\chapter{Two}\n\\begin{figure}A\\caption{a}\\end{figure}\\begin{figure}B\\caption{b}\\end{figure}\\begin{figure}C\\caption{c}\\end{figure}\n\\begin{figure}\\ContinuedFloat C2\\caption{c2}\\end{figure}\n\\end{document}\n",
      vec![("Ch1.F3a", figure("1.3")), ("Ch2.F3a", figure("2.3"))],
    ),
    (
      "\\documentclass{article}\n\\usepackage{graphicx}\n\\usepackage{subfig}\n\\begin{document}\n\\begin{figure}\\subfloat[One]{\\rule{1cm}{1cm}}\\subfloat[Two]{\\rule{1cm}{1cm}}\\caption{First}\\end{figure}\n\\begin{figure}\\ContinuedFloat\\subfloat[Three]{\\rule{1cm}{1cm}}\\caption{Second}\\end{figure}\n\\begin{figure}\\rule{1cm}{1cm}\\caption{Next}\\end{figure}\n\\end{document}\n",
      vec![
        ("S0.F1a", figure("1")),
        (
          "S0.F1a.sf3",
          r#"<tags><tag>(c)</tag><tag role="refnum">1c</tag></tags>"#.to_string(),
        ),
        ("S0.F2", figure("2")),
      ],
    ),
  ] {
    let (stderr, xml) = convert_with(tex, None);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    let expected: Vec<(&str, &str)> = expected.iter().map(|(id, t)| (*id, t.as_str())).collect();
    assert_float_tags(&xml, &expected);
  }
}

/// 57ad review: only the main float's own counter restarts a continuation. A sub-float's counter
/// steps through plain `\refstepcounter` in caption (caption.sty:668), not through
/// `\caption@prepare@stepcounter` (:577-579), so a `\subcaption` in a minipage or a subfig
/// `\subfloat` after the main caption does not reset `\continuedfloat@captype` — the first cut
/// logged "Continued `table' after `subtable'" and numbered 1, 2, 3. The continuation mark is
/// global (caption's `caption@flags` counter, :173-192), so a `\ContinuedFloat` inside a `center`
/// still reaches the float's `\caption`. pdflatex: Table 1, 1, 2; subfig's uncaptioned continuation
/// leaves its step-back, so its next table is Table 1 again; Figure 1, 1, 2.
#[test]
fn continuedfloat_counts_only_the_main_floats_steps() {
  let tags = |kind: &str, n: &str, sized: bool| {
    let label = if sized {
      format!(r#"<text fontsize="90%">{kind} {n}</text>"#)
    } else {
      format!("{kind} {n}")
    };
    format!(
      r#"<tags><tag>{label}</tag><tag role="refnum">{n}</tag><tag role="typerefnum">{kind} {n}</tag></tags>"#
    )
  };
  for (tex, expected) in [
    (
      "\\documentclass{article}\n\\usepackage{caption}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{table}\\caption{Top}\n\\begin{minipage}{.4\\linewidth}\\subcaption{One}X\\end{minipage}\n\\end{table}\n\\begin{table}\\ContinuedFloat\\caption{Top again}\n\\begin{minipage}{.4\\linewidth}\\subcaption{Two}Y\\end{minipage}\n\\end{table}\n\\begin{table}\\caption{Next}Z\\end{table}\n\\end{document}\n",
      vec![
        ("S0.T1", tags("Table", "1", true)),
        ("S0.T1a", tags("Table", "1", true)),
        ("S0.T2", tags("Table", "2", true)),
      ],
    ),
    (
      "\\documentclass{article}\n\\usepackage{graphicx}\n\\usepackage{subfig}\n\\begin{document}\n\\begin{table}\\caption{Top}\\subfloat[One]{\\rule{1cm}{1cm}}\\subfloat[Two]{\\rule{1cm}{1cm}}\\end{table}\n\\begin{table}\\ContinuedFloat\\subfloat[Three]{\\rule{1cm}{1cm}}\\end{table}\n\\begin{table}\\caption{Next}\\rule{1cm}{1cm}\\end{table}\n\\end{document}\n",
      vec![
        ("S0.T1", tags("Table", "1", false)),
        (
          "S0.T1a.sf3",
          r#"<tags><tag>(c)</tag><tag role="refnum">1c</tag></tags>"#.to_string(),
        ),
        // The next table, Table 1 again: its id `S0.T1` is taken, so it is repaired to `S0.T1a`
        // (the uncaptioned continued table keeps no element of its own, only its sub-float).
        ("S0.T1a", tags("Table", "1", false)),
      ],
    ),
    (
      "\\documentclass{article}\n\\usepackage{caption}\n\\begin{document}\n\\begin{figure}A\\caption{A}\\end{figure}\n\\begin{figure}\\begin{center}\\ContinuedFloat B\\end{center}\\caption{B}\\end{figure}\n\\begin{figure}C\\caption{C}\\end{figure}\n\\end{document}\n",
      vec![
        ("S0.F1", tags("Figure", "1", false)),
        // The continued figure: its `\@alph` id suffix is local to the `center` (as caption's),
        // so `S0.F1a` is the duplicate-id repair of its `S0.F1`.
        ("S0.F1a", tags("Figure", "1", false)),
        ("S0.F2", tags("Figure", "2", false)),
      ],
    ),
  ] {
    let (stderr, xml) = convert_with(tex, None);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    let expected: Vec<(&str, &str)> = expected.iter().map(|(id, t)| (*id, t.as_str())).collect();
    assert_float_tags(&xml, &expected);
  }
}

/// 57ad second review: caption clears a pending continuation wherever `\caption@settype` runs
/// (caption.sty:300-303) — every float's begin but a sub-float's, `\captionof` (:389-391),
/// `\captionsetup{type=…}` (:283) and a longtable (:1186-1187) — and its flags are one global
/// counter for every type (:173-192) (`latex_constructs::begin_float_continuation`). pdflatex: a
/// `\captionof{table}` in a figure's minipage is Table 1, which the next table continues; a
/// `\caption*` continuation followed by a longtable, or by a table holding `\captionof{figure}`,
/// leaves no pending step (Table 2, Figure 2); a `\captionof{figure}` in a continued figure, or a
/// `\captionsetup{type=figure}` after `\ContinuedFloat`, clears it (Figure 2); a minipage's
/// `\captionsetup{type=figure}` before `\ContinuedFloat` continues (Figure 1 twice). The second
/// cut opened a scope at the outermost float only: an error in the first case and a number lost
/// in the next four. Third review: caption never types a float it does not build — an
/// uncaptioned longtable (its begin is the kernel's plain step, longtable.sty:115; caption types
/// and prepares only at `\LT@makecaption`, caption.sty:1167, :1171), a listing, subfigure.sty's
/// sub-floats — so a continuation survives them (the third cut logged "Continued `figure' after
/// `table'" / "after `lstlisting'" / "`table' after `subfigure'"); a top-level `\captionof` does
/// not stop the next figure's begin from clearing a `\caption*` continuation (Figure 2).
#[test]
fn continuedfloat_scope_opens_where_caption_sets_the_type() {
  let tags = |kind: &str, n: &str| {
    format!(
      r#"<tags><tag>{kind} {n}</tag><tag role="refnum">{n}</tag><tag role="typerefnum">{kind} {n}</tag></tags>"#
    )
  };
  let doc = |preamble: &str, body: &str| {
    format!(
      "\\documentclass{{article}}\n\\usepackage{{caption}}\n{preamble}\\begin{{document}}\n{body}\\end{{document}}\n"
    )
  };
  for (tex, expected) in [
    (
      doc(
        "",
        "\\begin{figure}\n\\begin{minipage}{.45\\linewidth}Y\\caption{F one}\\end{minipage}\n\\begin{minipage}{.45\\linewidth}X\\captionof{table}{T one}\\end{minipage}\n\\end{figure}\n\\begin{table}\\ContinuedFloat Z\\caption{T one cont}\\end{table}\n\\begin{table}W\\caption{T two}\\end{table}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.T1", tags("Table", "1")),
        ("S0.T1a", tags("Table", "1")),
        ("S0.T2", tags("Table", "2")),
      ],
    ),
    (
      doc(
        "\\usepackage{longtable}\n",
        "\\begin{table}A\\caption{A}\\end{table}\n\\begin{table}\\ContinuedFloat B\\caption*{B}\\end{table}\n\\begin{longtable}{l}\\caption{LT}\\\\ x\\\\\\end{longtable}\n\\begin{table}C\\caption{C}\\end{table}\n",
      ),
      vec![
        ("S0.T1", tags("Table", "1")),
        ("S0.T2", tags("Table", "2")),
        ("S0.T3", tags("Table", "3")),
      ],
    ),
    (
      doc(
        "",
        "\\begin{figure}A\\caption{A}\\end{figure}\n\\begin{figure}\\ContinuedFloat B\\caption*{B}\\end{figure}\n\\begin{table}\\caption{T}\\begin{minipage}{.4\\linewidth}X\\captionof{figure}{In table}\\end{minipage}\\end{table}\n\\begin{figure}C\\caption{C}\\end{figure}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.T1", tags("Table", "1")),
        ("S0.F2", tags("Figure", "2")),
        ("S0.F3", tags("Figure", "3")),
      ],
    ),
    (
      doc(
        "",
        "\\begin{figure}A\\caption{A}\\end{figure}\n\\begin{figure}\\ContinuedFloat\\begin{minipage}{.4\\linewidth}B\\captionof{figure}{B}\\end{minipage}\\end{figure}\n\\begin{figure}C\\caption{C}\\end{figure}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.F2", tags("Figure", "2")),
        ("S0.F3", tags("Figure", "3")),
      ],
    ),
    (
      doc(
        "",
        "\\begin{figure}A\\caption{A}\\end{figure}\n\\begin{figure}\\ContinuedFloat B\\captionsetup{type=figure}\\caption{B}\\end{figure}\n\\begin{figure}C\\caption{C}\\end{figure}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.F2", tags("Figure", "2")),
        ("S0.F3", tags("Figure", "3")),
      ],
    ),
    (
      doc(
        "\\usepackage{longtable}\n",
        "\\begin{figure}A\\caption{A}\\end{figure}\n\\begin{longtable}{l} x\\\\ y\\\\\\end{longtable}\n\\begin{figure}\\ContinuedFloat B\\caption{B}\\end{figure}\n\\begin{figure}C\\caption{C}\\end{figure}\n\\begin{table}T\\caption{T}\\end{table}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.F1a", tags("Figure", "1")),
        ("S0.F2", tags("Figure", "2")),
        ("S0.T2", tags("Table", "2")),
      ],
    ),
    (
      doc(
        "\\usepackage{listings}\n",
        "\\begin{figure}A\\caption{A}\\end{figure}\n\\begin{lstlisting}[caption=L]\nx = 1\n\\end{lstlisting}\n\\begin{figure}\\ContinuedFloat B\\caption{B}\\end{figure}\n\\begin{figure}C\\caption{C}\\end{figure}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.F1a", tags("Figure", "1")),
        ("S0.F2", tags("Figure", "2")),
      ],
    ),
    (
      doc(
        "\\usepackage{subfigure}\n",
        "\\begin{table}\\caption{T}\\subfigure[a]{X}\\subfigure[b]{Y}\\end{table}\n\\begin{table}\\ContinuedFloat\\caption{T cont}Z\\end{table}\n\\begin{table}W\\caption{Next}\\end{table}\n",
      ),
      vec![
        ("S0.T1", tags("Table", "1")),
        ("S0.T1a", tags("Table", "1")),
        ("S0.T2", tags("Table", "2")),
      ],
    ),
    (
      doc(
        "",
        "X\\captionof{figure}{Top-level}\n\n\\begin{figure}\\ContinuedFloat B\\caption*{B}\\end{figure}\n\\begin{figure}C\\caption{C}\\end{figure}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.F2", tags("Figure", "2")),
      ],
    ),
    (
      doc(
        "",
        "\\noindent\\begin{minipage}{.4\\linewidth}\\captionsetup{type=figure}X\\caption{A}\\end{minipage}\n\n\\noindent\\begin{minipage}{.4\\linewidth}\\captionsetup{type=figure}\\ContinuedFloat Y\\caption{B}\\end{minipage}\n\n\\begin{figure}Z\\caption{C}\\end{figure}\n",
      ),
      vec![
        ("S0.F1", tags("Figure", "1")),
        ("S0.F1a", tags("Figure", "1")),
        ("S0.F2", tags("Figure", "2")),
      ],
    ),
  ] {
    let (stderr, xml) = convert_with(&tex, None);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    let expected: Vec<(&str, &str)> = expected.iter().map(|(id, t)| (*id, t.as_str())).collect();
    assert_float_tags(&xml, &expected);
  }
}

/// 57ad fourth review: the float `\@captionof@` wraps a caption in is marked so that its begin
/// opens no new type (`lx@float@captionof`, one-shot); when the `\begin` never reaches a float — a
/// `\captionof` of a non-float environment, which pdflatex rejects too ("No counter", "No float
/// type") — the mark is cleared after its `\end`, else the next figure skipped its type and a
/// `\ContinuedFloat` two figures on logged "Continued `figure' after `table'" and shifted every
/// later number. pdflatex: Figure 1, 1, 2. The three errors are the invalid `\captionof`'s own.
#[test]
fn continuedfloat_captionof_wrapper_does_not_leak() {
  let (stderr, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{caption}\n\\newenvironment{myfig}{\\par}{\\par}\n\\begin{document}\n\\begin{table}A\\caption{A}\\end{table}\n\\noindent X\\captionof{myfig}{Non-float env}\n\n\\begin{figure}C\\caption{C}\\end{figure}\n\\begin{figure}\\ContinuedFloat D\\caption{D}\\end{figure}\n\\begin{figure}E\\caption{E}\\end{figure}\n\\end{document}\n",
    None,
  );
  assert_eq!(error_count(&stderr), 3, "{stderr}");
  assert!(!stderr.contains("Continued `figure'"), "{stderr}");
  let figure = |n: &str| {
    format!(
      r#"<tags><tag>Figure {n}</tag><tag role="refnum">{n}</tag><tag role="typerefnum">Figure {n}</tag></tags>"#
    )
  };
  let (f1, f2) = (figure("1"), figure("2"));
  assert_float_tags(&xml, &[
    ("S0.F1", f1.as_str()),
    ("S0.F1a", f1.as_str()),
    ("S0.F2", f2.as_str()),
  ]);
}

/// 57ae: `\DeclareMathOperator` is Perl's wrapped DefMath (amsopn.sty.ltxml:23-29 passes the token
/// presentation `\operatorname*?{body}`; Package.pm:1656-1660, 1786-1813; `def_math_wrapped`): the
/// control sequence's name is kept (it differs from the presentation), the wrapper's `XMWrap`
/// hands name, role and scriptpos to the parsed operator, a starred operator's scriptpos is read
/// per use (`mid` in display, `post` inline — 57ab's dual path had put `mid` on inline tokens, 84
/// in 8 papers of the 3,003-paper A/B, mostly `\E`), and `tex=` is `\operatorname{…}` with the
/// author's form in `content-tex=`, byte for byte as Perl. The scripted `XMApp`'s `role` is Rust's
/// post-parse copy. Repros math-parse/{starred_operator_with_a_command_body_puts_limits_below,
/// declaremathoperator_plain_body_keeps_its_name}.
#[test]
fn declaremathoperator_is_perls_wrapped_operator() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/starred_operator_with_a_command_body_puts_limits_below.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, mode, pos) in [("S0.Ex1.m1", "display", "mid"), ("p1.m1", "inline", "post")] {
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        "<Math content-tex=\"\\argmax_{{x}}g\" mode=\"{mode}\" tex=\"\\operatorname*{{arg\\,max}}_{{x}}g\" text=\"(argmax _ x)@(g)\" xml:id=\"{id}\"><XMath><XMApp><XMApp role=\"OPERATOR\"><XMTok role=\"SUBSCRIPTOP\" scriptpos=\"{pos}1\"/><XMApp name=\"argmax\" role=\"OPERATOR\" scriptpos=\"{pos}\"><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok role=\"UNKNOWN\" rpadding=\"1.7pt\">arg</XMTok><XMTok role=\"UNKNOWN\">max</XMTok></XMApp><XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">x</XMTok></XMApp><XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok></XMApp></XMath></Math>"
      ),
    );
  }
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/declaremathoperator_plain_body_keeps_its_name.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[],
    r#"<Math content-tex="\Tr B" mode="inline" tex="\operatorname{Tr}B" text="Tr@(B)" xml:id="p1.m1"><XMath><XMApp><XMTok name="Tr" role="OPFUNCTION" scriptpos="post">Tr</XMTok><XMTok font="italic" role="UNKNOWN">B</XMTok></XMApp></XMath></Math>"#,
  );
}

/// 57ae: the letters and number ligatures walk back while the fonts are equal as Perl's
/// `Font::equals` (Base_XMath.pool.ltxml:443-458, :486-510; Font.pm:333-337; `Font::perl_equals`):
/// `\mathrm{ab}2` is `ab2`, `1\mathrm{2}` the number 12 and `3.\mathrm{5}` 3.5, as Perl — Rust's
/// `Font` equality compared its own `name` and the math default's unset encoding (Perl's is `OT1`).
/// A letter run read back through leading digits joins its letters (`2`, `KL`; `10`, `log`) where
/// Perl joins nothing (`l * o * g`; KNOWN_PERL_ERRORS #364, OXIDIZED_DESIGN #348). Witness
/// 2605.31599; repro math-parse/letters_ligature_reads_back_through_a_digit.
#[test]
fn letters_ligature_reads_back_through_a_digit() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/letters_ligature_reads_back_through_a_digit.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, tex, text, digits, letters) in [
    ("p1.m1", "2\\mathrm{KL}", "2 * KL", "2", "KL"),
    ("p1.m2", "5\\mathrm{mm}", "5 * mm", "5", "mm"),
    ("p1.m3", "10\\mathrm{log}", "10 * log", "10", "log"),
  ] {
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        "<Math mode=\"inline\" tex=\"{tex}\" text=\"{text}\" xml:id=\"{id}\"><XMath><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok meaning=\"{digits}\" role=\"NUMBER\">{digits}</XMTok><XMTok role=\"UNKNOWN\">{letters}</XMTok></XMApp></XMath></Math>"
      ),
    );
  }
  for (id, math) in [
    (
      "p1.m4",
      r#"<Math mode="inline" tex="\mathrm{KL}(f)" text="KL@(f)" xml:id="p1.m4"><XMath><XMApp><XMTok role="UNKNOWN">KL</XMTok><XMDual><XMRef idref="p1.m4.1"/><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m4.1">f</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m5",
      r#"<Math mode="inline" tex="\mathrm{ab}2" text="ab2" xml:id="p1.m5"><XMath><XMTok role="UNKNOWN">ab2</XMTok></XMath></Math>"#,
    ),
    (
      "p1.m6",
      r#"<Math mode="inline" tex="1\mathrm{2}" text="12" xml:id="p1.m6"><XMath><XMTok meaning="12" role="NUMBER">12</XMTok></XMath></Math>"#,
    ),
    (
      "p1.m7",
      r#"<Math mode="inline" tex="3.\mathrm{5}" text="3.5" xml:id="p1.m7"><XMath><XMTok meaning="3.5" role="NUMBER">3.5</XMTok></XMath></Math>"#,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}

/// 57af: a bar with a name but no meaning — `\arrowvert`, amsmath's `\lvert`/`\rvert` and
/// `\lVert`/`\rVert` after `\bigl`/`\bigr` — lexes by its glyph (`|` single, `∥`/`‖` double), as
/// Perl's content-agnostic `VERTBAR` terminal (MathGrammar:797) and content-keyed fences
/// (MathParser.pm:1361) read it: absolute value and norm, where Rust's `VERTBAR:lvert` lexeme left
/// the formula unparsed. In a gathered row's content branch the bar is an XMRef, read through its
/// target. Repro math-parse/named_vertbar_parses_as_absolute_value (7 papers of the 3,003-paper A/B;
/// arXiv 2605.02713, 2605.19292).
#[test]
fn named_vertbar_parses_as_absolute_value() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/named_vertbar_parses_as_absolute_value.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for (id, tex, meaning, attrs, open, close, bar) in [
    (
      "p1.m1",
      "\\bigl\\arrowvert y\\bigr\\arrowvert",
      "absolute-value",
      " fontsize=\"120%\"",
      "name=\"arrowvert\"",
      "name=\"arrowvert\"",
      "|",
    ),
    (
      "p1.m2",
      "\\arrowvert y\\arrowvert",
      "absolute-value",
      "",
      "name=\"arrowvert\"",
      "name=\"arrowvert\"",
      "|",
    ),
    (
      "p1.m6",
      "\\bigl\\lvert y\\bigr\\rvert",
      "absolute-value",
      " fontsize=\"120%\"",
      "name=\"lvert\"",
      "name=\"rvert\"",
      "|",
    ),
    (
      "p1.m7",
      "\\bigl\\lVert y\\bigr\\rVert",
      "norm",
      " fontsize=\"120%\"",
      "name=\"lVert\"",
      "name=\"rVert\"",
      "\u{2016}",
    ),
    (
      "p1.m8",
      "\\Arrowvert y\\Arrowvert",
      "norm",
      "",
      "name=\"Arrowvert\"",
      "name=\"Arrowvert\"",
      "\u{2016}",
    ),
  ] {
    let stretchy = if id == "p1.m6" {
      " stretchy=\"false\""
    } else {
      ""
    };
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        "<Math mode=\"inline\" tex=\"{tex}\" text=\"{meaning}@(y)\" xml:id=\"{id}\"><XMath><XMDual><XMApp><XMTok meaning=\"{meaning}\"/><XMRef idref=\"{id}.1\"/></XMApp><XMWrap><XMTok{attrs} {open} role=\"OPEN\"{stretchy}>{bar}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"{id}.1\">y</XMTok><XMTok{attrs} {close} role=\"CLOSE\"{stretchy}>{bar}</XMTok></XMWrap></XMDual></XMath></Math>"
      ),
    );
  }
  latexml::util::test::assert_element(
    &xml,
    "Math",
    &[r#"xml:id="S0.Ex1.m1""#],
    concat!(
      r#"<Math mode="display" tex="\begin{gathered}\bigl\lvert y\bigr\rvert=1\end{gathered}" text="absolute-value@(y) = 1" xml:id="S0.Ex1.m1"><XMath><XMDual>"#,
      r#"<XMApp><XMRef idref="S0.Ex1.m1.1"/><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.2"/></XMApp><XMRef idref="S0.Ex1.m1.3"/></XMApp>"#,
      r#"<XMArray name="gathered"><XMRow><XMCell align="center"><XMApp><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.1">=</XMTok>"#,
      r#"<XMWrap><XMTok fontsize="120%" name="lvert" role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.2">y</XMTok><XMTok fontsize="120%" name="rvert" role="CLOSE" stretchy="false">|</XMTok></XMWrap>"#,
      r#"<XMTok meaning="1" role="NUMBER" xml:id="S0.Ex1.m1.3">1</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math>"#
    ),
  );
}

/// 57af: in a gathered/split/multline row the content branch (XMRefs to the row's tokens) is parsed
/// too (MathParser.pm:378-392): the INTOP that licenses a differential `d` is the realized role of
/// an XMRef's target (the lexer's `XDIFFUNK` downgrade and `diffop_apply` read raw roles, so the
/// content read `f * d * x`), and the `d` keeps its node — its xml:id, so the content's XMRef to it
/// resolves, and on an XMRef Perl `Annotate`'s role and meaning (MathParser.pm:1206-1235). The `\,`
/// before it is padding once: `rpadding` on the `f` and on its content XMRef (the content branch's
/// copy of the hint, Perl `createXMRefs`), none on the `d` (which a Rust-only `lpadding` transfer
/// doubled). Perl: `integral@(f * differential-d@(x)) = d`. Repro
/// math-parse/gathered_row_keeps_its_differential (47 papers of the 3,003-paper A/B showed
/// `integral@(… * [] * …)`; arXiv 2605.01547, 2605.23309).
#[test]
fn gathered_row_keeps_its_differential() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/gathered_row_keeps_its_differential.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "XMDual",
    &[],
    concat!(
      r#"<XMDual><XMApp><XMRef idref="S0.Ex1.m1.1"/><XMApp><XMRef idref="S0.Ex1.m1.2"/><XMApp>"#,
      "<XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>",
      r#"<XMRef idref="S0.Ex1.m1.3" rpadding="1.7pt"/><XMApp><XMRef idref="S0.Ex1.m1.4" meaning="differential-d" role="DIFFOP"/><XMRef idref="S0.Ex1.m1.5"/></XMApp></XMApp></XMApp><XMRef idref="S0.Ex1.m1.6"/></XMApp>"#,
      r#"<XMArray name="gathered"><XMRow><XMCell align="center"><XMApp><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex1.m1.1">=</XMTok><XMApp><XMTok mathstyle="display" meaning="integral" name="int" role="INTOP" xml:id="S0.Ex1.m1.2">∫</XMTok><XMApp>"#,
      "<XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>",
      r#"<XMTok font="italic" role="UNKNOWN" rpadding="1.7pt" xml:id="S0.Ex1.m1.3">f</XMTok><XMApp><XMTok font="italic" meaning="differential-d" role="DIFFOP" xml:id="S0.Ex1.m1.4">d</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.5">x</XMTok></XMApp></XMApp></XMApp><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.6">d</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual>"#
    ),
  );
}

/// 57af: a gathered/split/multline row's content branch holds XMRefs to the rows' tokens, and Perl
/// reads a fence's delimiters and punctuation through `realizeXMNode` (Fence, MathParser.pm:1398-1402):
/// reading the XMRef's own empty content made every bracket there `delimited-@(…)`. The inner XMHints
/// reach the content branch as id-less copies (Perl `createXMRefs`, Package.pm:1551-1556), so a
/// `\quad` there is padding on the preceding XMRef — not a second punctuation after a `,`: the
/// PUNCT test reads the realized role (Perl reads the XMRef's own and leaves the formula unparsed,
/// KNOWN_PERL_ERRORS #365). A phantom's width never becomes a PUNCT (MathParser.pm:443, :482-483), and a negative pending
/// space is the next node's `lpadding` (:466). Repro
/// math-parse/content_branch_reads_its_delimiters.
#[test]
fn content_branch_reads_its_delimiters() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/math-parse/content_branch_reads_its_delimiters.tex"
    ),
    None,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let gathered = |row: &str| {
    format!(
      r#"<XMArray name="gathered"><XMRow><XMCell align="center">{row}</XMCell></XMRow></XMArray>"#
    )
  };
  for (id, math) in [
    (
      "S0.Ex1.m1",
      format!(
        r#"<Math mode="display" tex="\begin{{gathered}}|y|\end{{gathered}}" text="absolute-value@(y)" xml:id="S0.Ex1.m1"><XMath><XMDual><XMApp><XMTok meaning="absolute-value"/><XMRef idref="S0.Ex1.m1.1"/></XMApp>{}</XMDual></XMath></Math>"#,
        gathered(
          r#"<XMWrap><XMTok role="OPEN" stretchy="false">|</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex1.m1.1">y</XMTok><XMTok role="CLOSE" stretchy="false">|</XMTok></XMWrap>"#
        )
      ),
    ),
    (
      "S0.Ex2.m1",
      format!(
        r#"<Math mode="display" tex="\begin{{gathered}}\lfloor x\rfloor\end{{gathered}}" text="floor@(x)" xml:id="S0.Ex2.m1"><XMath><XMDual><XMApp><XMTok meaning="floor"/><XMRef idref="S0.Ex2.m1.2"/></XMApp>{}</XMDual></XMath></Math>"#,
        gathered(
          r#"<XMWrap><XMTok name="lfloor" role="OPEN" stretchy="false">⌊</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex2.m1.2">x</XMTok><XMTok name="rfloor" role="CLOSE" stretchy="false">⌋</XMTok></XMWrap>"#
        )
      ),
    ),
    (
      "S0.Ex3.m1",
      format!(
        r#"<Math mode="display" tex="\begin{{gathered}}\{{a,b\}}\end{{gathered}}" text="set@(a, b)" xml:id="S0.Ex3.m1"><XMath><XMDual><XMApp><XMTok meaning="set"/><XMRef idref="S0.Ex3.m1.2"/><XMRef idref="S0.Ex3.m1.4"/></XMApp>{}</XMDual></XMath></Math>"#,
        gathered(
          r#"<XMWrap><XMTok role="OPEN" stretchy="false">{</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.2">a</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex3.m1.4">b</XMTok><XMTok role="CLOSE" stretchy="false">}</XMTok></XMWrap>"#
        )
      ),
    ),
    (
      "S0.Ex4.m1",
      format!(
        r#"<Math mode="display" tex="\begin{{gathered}}(a,b]\end{{gathered}}" text="open-closed-interval@(a, b)" xml:id="S0.Ex4.m1"><XMath><XMDual><XMApp><XMTok meaning="open-closed-interval"/><XMRef idref="S0.Ex4.m1.2"/><XMRef idref="S0.Ex4.m1.4"/></XMApp>{}</XMDual></XMath></Math>"#,
        gathered(
          r#"<XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.2">a</XMTok><XMTok role="PUNCT">,</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex4.m1.4">b</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap>"#
        )
      ),
    ),
    (
      "S0.Ex5.m1",
      format!(
        r#"<Math mode="display" tex="\begin{{gathered}}a=b,\quad c=d\end{{gathered}}" text="formulae@(a = b, c = d)" xml:id="S0.Ex5.m1"><XMath><XMDual><XMDual><XMApp><XMTok meaning="formulae"/><XMRef idref="S0.Ex5.m1.1"/><XMRef idref="S0.Ex5.m1.2"/></XMApp><XMWrap><XMApp xml:id="S0.Ex5.m1.1"><XMRef idref="S0.Ex5.m1.4"/><XMRef idref="S0.Ex5.m1.5"/><XMRef idref="S0.Ex5.m1.6"/></XMApp><XMRef idref="S0.Ex5.m1.7" rpadding="10.0pt"/><XMApp xml:id="S0.Ex5.m1.2"><XMRef idref="S0.Ex5.m1.9"/><XMRef idref="S0.Ex5.m1.10"/><XMRef idref="S0.Ex5.m1.11"/></XMApp></XMWrap></XMDual>{}</XMDual></XMath></Math>"#,
        gathered(concat!(
          r#"<XMWrap><XMApp xml:id="S0.Ex5.m1.3"><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex5.m1.4">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.5">a</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.6">b</XMTok></XMApp>"#,
          r#"<XMTok role="PUNCT" rpadding="10.0pt" xml:id="S0.Ex5.m1.7">,</XMTok>"#,
          r#"<XMApp xml:id="S0.Ex5.m1.8"><XMTok meaning="equals" role="RELOP" xml:id="S0.Ex5.m1.9">=</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.10">c</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="S0.Ex5.m1.11">d</XMTok></XMApp></XMWrap>"#
        ))
      ),
    ),
    (
      "S0.Ex7.m1",
      concat!(
        r#"<Math mode="display" tex="\begin{gathered}(\!x)\\&#10;\!y\end{gathered}" text="x * y" xml:id="S0.Ex7.m1"><XMath><XMDual><XMApp>"#,
        "<XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>",
        r#"<XMRef idref="S0.Ex7.m1.2"/><XMRef idref="S0.Ex7.m1.4"/></XMApp><XMArray name="gathered">"#,
        r#"<XMRow><XMCell align="center"><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok font="italic" lpadding="-1.7pt" role="UNKNOWN" xml:id="S0.Ex7.m1.2">x</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMCell></XMRow>"#,
        r#"<XMRow><XMCell align="center"><XMTok font="italic" lpadding="-1.7pt" role="UNKNOWN" xml:id="S0.Ex7.m1.4">y</XMTok></XMCell></XMRow></XMArray></XMDual></XMath></Math>"#
      )
      .to_string(),
    ),
    (
      "S0.Ex6.m1",
      concat!(
        r#"<Math mode="display" tex="x\hphantom{abcdefgh}y" text="x * y" xml:id="S0.Ex6.m1"><XMath><XMApp>"#,
        "<XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>",
        r#"<XMTok font="italic" role="UNKNOWN" rpadding="40.6pt">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math>"#
      )
      .to_string(),
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], &math);
  }
}
