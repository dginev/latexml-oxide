//! Red/green guards for perfect-kernel batch 54 (wave-4 root-causer
//! reports over the sweep-28 residuals). Each test is the minimal
//! reproduction distilled during triage; the doc-comment names the
//! ORIGINAL corpus witness (TeX Live doc corpus) whose larger conversion
//! was vetted separately.
use super::{
  perfect_kernel_batch40_43::convert_with_files,
  perfect_kernel_batch46::{convert, convert_with, error_count, warning_count},
  perfect_kernel_batch53::convert_with_sty,
};

fn kpsewhich_has(name: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// biblatex.sty:4407-4425 defines `\DeclareIndex{Name,List,Field}Format`
/// through the same `\blx@defformat` as their non-Index siblings, and
/// :14133 `\DeclareDriverSourcemap[2][]`. The native binding no-ops the
/// siblings but omitted these, so an undefined-CS stub (zero args) left
/// each declaration BODY in the document: `#1` reached the Stomach
/// (`misdefined:#`) and `\nameparts`/`\usebibmacro`/`\actualoperator`/
/// `\map`/`\step` fired as undefined (cnltx.bbx:131-210; witnesses
/// cnltx_en, endiagram_en, chemformula-manual — 7 `#` + 11 undefined each).
#[test]
fn bbx_declaration_bodies_are_absorbed() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{biblatex}
\DeclareIndexFieldFormat[package]{title}{#1}
\DeclareIndexListFormat{cnltx}{#1}
\DeclareIndexNameFormat{cnltx}{\nameparts{#1}\usebibmacro{index:entry}{#1}\actualoperator}
\DeclareDriverSourcemap[datatype=bibtex]{\map{\step[fieldsource=info, fieldtarget=subtitle]}}
\begin{document}
Hello.\usebibmacro*{index:entry}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("misdefined"), "{stderr}");
  assert!(
    !xml.contains("ltx:ERROR") && !xml.contains("<ERROR"),
    "{xml}"
  );
  assert!(!xml.contains("[package]title"), "{xml}");
  assert!(xml.contains("<p>Hello.</p>"), "{xml}");
}

/// biblatex.sty:9436 `\defbibcheck[2]`, :7029 `\DeclareRedundantLanguages[2]`
/// and :9784 `\printbibheading` (one `\@ifnextchar[` optional). Undefined
/// `\defbibcheck` (arthistory-bonn.bbx:199) leaked its check body, whose
/// `\ifcsdef{\strfield{series}}` mis-nested into a live `\iffalse` that
/// scanned to the .bbx end of file (`expected:\fi`), eating the document's
/// `\printbibheading` (witness rub-kunstgeschichte-example: 4 errors → 0).
#[test]
fn biblatex_check_and_heading_commands_absorb_args() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{biblatex}
\DeclareRedundantLanguages{german}{german,ngerman}
\defbibcheck{shortseries}{\iffieldundef{series}{\skipentry}{\ifcsdef{\strfield{series}}{\skipentry}{}}}
\begin{document}
A.\printbibheading[title=Works]B.\printbibheading C.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("iffalse"), "{stderr}");
  assert!(xml.contains("A.B.C."), "{xml}");
}

/// `\usetikzlibrary{hobby}` → tikzlibraryhobby.code.tex:16 → pgflibraryhobby
/// .code.tex:16 `\input{hobby.code.tex}`. A `Warn!`-only refusal stub
/// (`hobby_code_tex.rs`, added for arXiv 2111.02755 "until our LaTeX3 support
/// is ready") intercepted that `\input`, so `\hobbyVersion`/`\hobbyDate`
/// (hobby.code.tex:36/40) and `\hobbyinit` (:668) were undefined and the
/// zero-arg ERROR stub for `\hobbyinit` left `\curvethrough`'s
/// (tikzlibraryhobby.code.tex:210) `\relax`-delimited point scan to run to
/// end of input (witness hobby/hobby: 5 errors + EoF Fatal). The real file
/// (expl3 + pml3array) now raw-loads clean; the stub is retired.
#[test]
fn hobby_code_tex_raw_loads() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{hobby}
\begin{document}
V=\hobbyVersion\ from \hobbyDate.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("hobby.code.tex is not implemented"),
    "{stderr}"
  );
  assert!(xml.contains("V=1.12 from 2023-09-01."), "{xml}");
}

/// tex.web §373: a non-character CS inside `\csname…\endcsname` is
/// "Missing \endcsname inserted" via `back_error` — the name ENDS there and
/// the offending token is re-read after the constructed CS. The gullet
/// reported the error but kept scanning to the real `\endcsname`, so every
/// token in between was lost (witness tikzpingus-doc: `\csname …\relax…`
/// style key builders dropped their content). Two errors remain, exactly
/// real TeX's: "Missing \endcsname inserted" then "Extra \endcsname"
/// (tex.web §1135) for the orphaned closer.
#[test]
fn csname_missing_endcsname_reinserts_after_offender() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\begin{document}
X\csname foo\relax bar\endcsname Y
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(stderr.contains("Extra \\endcsname"), "{stderr}");
  assert!(xml.contains("bar"), "{xml}");
  assert!(xml.contains("Y"), "{xml}");
}

/// The forest binding replaces the raw forest.sty, whose :1
/// `\ProvidesPackage{forest}` is what `\@ifpackageloaded{forest}` in
/// dependants keys on (forest-doc preamble, neoschool.cls).
#[test]
fn forest_binding_registers_as_loaded() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{forest}
\begin{document}
\makeatletter\@ifpackageloaded{forest}{LOADED}{ABSENT}\makeatother
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("LOADED") && !xml.contains("ABSENT"), "{xml}");
}

/// xltabular.sty:19-21 user toggles `\normalLTpagebreak`/`\specialLTpagebreak`
/// (page-break policy only) were dropped by the binding that replaces the
/// raw .sty (witness xltabular-doc: 2 undefined).
#[test]
fn xltabular_pagebreak_toggles() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{xltabular}
\begin{document}
Special: \specialLTpagebreak Normal: \normalLTpagebreak Done.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Special: Normal: Done."), "{xml}");
}

/// memoir.cls:8811 redefines only `\endminipage` (raw latex.ltx closer);
/// tcolorbox `\let\endtcb@lrbox=\endminipage` (tcolorbox.sty:1118) then
/// closed our NATIVE minipage with it: the live dump `\@iiiparbox` got an
/// undefined `\@mpargs` and its `Until:[` scan ate the next box's option
/// list (witness biblatex-oxref/oxalph-doc: 983× `\csname bm@bicolor,…`
/// + Fatal TooManyErrors). The binding now keeps the native pair paired.
#[test]
fn memoir_keeps_native_endminipage() {
  let (stderr, xml) = convert(
    r"\documentclass[oneside]{memoir}
\usepackage{tcolorbox}
\begin{document}
\begin{tcolorbox}[colframe=red]
A\par B\tcblower C
\end{tcolorbox}
\begin{tcolorbox}[colframe=blue]
D
\end{tcolorbox}
\end{document}
",
    true,
  );
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(!stderr.contains("bm@"), "{stderr}");
  assert!(xml.contains("ltx_minipage"), "{xml}");
  assert!(xml.contains(">D<") || xml.contains("D\n"), "{xml}");
}

/// latex.ltx:15913 `\endlist` decrements `\@listdepth`; our `\endlist`
/// never did, so a raw class whose `\list` is latex.ltx's (memoir.cls:4580,
/// with the `>5 → \@toodeep` check) hit "Too deeply nested" on the seventh
/// list (witness memman: 88 errors from `adjustwidth`, memoir.cls:11268).
#[test]
fn endlist_decrements_listdepth() {
  let (stderr, xml) = convert(
    r"\documentclass{memoir}
\begin{document}
\begin{adjustwidth}{1em}{1em}A\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}B\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}C\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}D\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}E\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}F\end{adjustwidth}
\begin{adjustwidth}{1em}{1em}G\end{adjustwidth}
\makeatletter\the\@listdepth\makeatother
\end{document}
",
    true,
  );
  assert!(!stderr.contains("Too deeply nested"), "{stderr}");
  assert!(xml.contains("G"), "{xml}");
  // The lists are real lists since OXIDIZED_DESIGN #180; the depth
  // reads 0 after the seventh has closed.
  assert!(xml.contains("<p>0</p>"), "{xml}");
}

/// listings.sty:320 `\let\lst@UserCommand\gdef`; patches such as
/// tagpdfdocu-patches.sty:65 `\lst@UserCommand\lstrenewenvironment#1#2#{…}`
/// otherwise leaked their `#` PARAM tokens into digestion (tagpdf manual:
/// 7× "should never reach Stomach").
#[test]
fn lst_usercommand_is_gdef() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{listings}
\makeatletter
\lst@UserCommand\lst@mytest#1#2#{[#1/#2]}
\makeatother
\begin{document}
\makeatletter\lst@mytest ab{x}\makeatother DONE
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("misdefined"), "{stderr}");
  assert!(
    xml.contains("[a/b]xDONE") || xml.contains("[a/b]x DONE"),
    "{xml}"
  );
}

/// `\lx@lstinline` opened its group with a direct `bgroup()` but closed it
/// with a raw `T_END`, which the gullet counts as −1 on the alignment
/// ledger; with a non-brace delimiter nothing compensated, so inside a
/// `p{}` cell the row's `\\` was never recognised and the tabular never
/// closed (witness bibleref-parse L172: 28-error cascade).
#[test]
fn lstinline_pipe_in_p_column() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{listings}
\begin{document}
\begin{tabular}{p{3cm}p{3cm}}
A & \lstinline|\foo| here\\
C & \lstinline{\bar} third\\
\end{tabular}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  assert!(xml.contains("ltx_lst_identifier\">foo"), "{xml}");
  assert!(xml.contains("third"), "{xml}");
}

/// Real `\DocumentMetadata` always loads tagpdf (documentmetadata-support
/// .ltx:72 → latex-lab-testphase-latest.sty:39); the tagpdf manual iterates
/// `\g__tag_role_NS_pdf_prop` (tagpdf.tex:2163) and an undefined prop turned
/// `\prop_map_inline:cn` into a `\prg_break_point:Nn` runaway to EOF.
#[test]
fn documentmetadata_loads_tagpdf() {
  let (stderr, xml) = convert(
    r"\DocumentMetadata{tagging=on}
\documentclass{article}
\begin{document}
\ExplSyntaxOn
\clist_clear:N \l_tmpa_clist
\prop_map_inline:cn { g__tag_role_NS_pdf_prop }
  { \clist_put_right:Nn \l_tmpa_clist {#1} }
\clist_use:Nn \l_tmpa_clist {,}
\ExplSyntaxOff
DONE
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert!(xml.contains("StructTreeRoot"), "{xml}");
  assert!(xml.contains("DONE"), "{xml}");
}

/// nicematrix.sty:1644/3745 `\NotEmpty` (flags a cell for `hvlines`, no
/// content) and :394 public `\g_nicematrix_code_before_tl` were missing
/// from the binding that replaces the raw .sty (witness cahierprof.sty:619
/// and :519/531 — cahierprof-exemple 2 errors).
#[test]
fn nicematrix_notempty_and_code_before_hook() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{nicematrix}
\ExplSyntaxOn
\tl_gput_right:Nn \g_nicematrix_code_before_tl { x }
\ExplSyntaxOff
\begin{document}
\begin{NiceTabular}{cc}
a & b \\
c & \NotEmpty \\
\end{NiceTabular}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
}

/// tikz-network.sty L400 presets `Network=false` on family `[NW]{vertex}`,
/// then `\@vertex` (L414-433) sets every key explicitly. Our beyond-Perl
/// `\XKV@setkeys` runs the Rust reader once per `preseth` hook before the
/// main list; the reader wipes `\XKV@fams`/`\XKV@prefix` on exit (faithful
/// to Perl KeyVals.pm L389-400), so the main call used to be rebuilt from an
/// empty family list and every real key became "unknown" (1001 errors on
/// tikz-network.tex, all `\cmdNW@vertex@*`). Real `\XKV@s@tkeys` never
/// mutates them (xkeyval.tex L464-469); the shim now saves/restores both.
#[test]
fn xkeyval_preset_hook_keeps_family_for_main_list() {
  let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@cmdkey  [NW] {vertex} {color}{}
\define@cmdkey  [NW] {vertex} {fontcolor}{}
\define@boolkey [NW] {vertex} {Network}[true]{}
\presetkeys     [NW] {vertex} {Network = false,}{}
\begin{document}
\setkeys[NW]{vertex}{color={red}, fontcolor={blue}}
C=[\cmdNW@vertex@color] F=[\cmdNW@vertex@fontcolor]
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("C=[red] F=[blue]"), "{xml}");
}

/// KNOWN_PERL_ERRORS #123: `` `<char> `` must take the character code of
/// any character token (tex.web §442). Perl strips a leading `\` from the
/// token *string* — fine for `\a`, but a catcode-12 backslash (what
/// `\detokenize`/`\string` produce) becomes "" → 0. Witness
/// bibleref-parse.sty L481-486 `\brp@ifcs` (backslash test → every
/// `\foreach`-variable book name "unknown"). Same root aborts every
/// `\fpeval{\dimen0 > \dimen1}`: l3fp's comparison chain-detect
/// (expl3-code.tex L17662-17673) routes `\if_case:w` on
/// `` ` \token_to_str:N <register> `` → 0 instead of 92 → the `@` sentinel
/// is never emitted → `Missing argument Until:@` + Fatal EoF (witness
/// swfigure `\fptest`/`\DFscalefactor`).
#[test]
fn backquote_charcode_of_other_backslash() {
  let tex = r"\documentclass{article}
\begin{document}
\def\name{x}
\def\first#1#2\end{[\number`#1]}
A\expandafter\first\detokenize{\name}aa\end
B\expandafter\first\string\name aa\end
C[\number`\\]D[\number`\a]E[\number`a]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Missing number"), "{stderr}");
  assert!(xml.contains("A[92]"), "{xml}");
  assert!(xml.contains("B[92]"), "{xml}");
  assert!(xml.contains("C[92]D[97]E[97]"), "{xml}");
}

#[test]
fn fpeval_register_right_operand_of_comparison() {
  let tex = r"\documentclass{book}
\usepackage{xfp}
\newdimen\Ah\newdimen\Bt\Ah=10pt\Bt=5pt
\begin{document}
\edef\x{\fpeval{\Ah > \Bt}}[\x]
\edef\y{\fpeval{\Ah < \Bt}}[\y]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert!(xml.contains("[1]\n[0]"), "{xml}");
}

/// OXIDIZED_DESIGN #170's named residual: `\angle` as a `\tikzmath`
/// variable (sunpath.sty L44-47). Real LaTeX's `\angle` is a robust
/// command, so `\meaning` starts with `macro:` and tikzmath's sniff
/// (tikzlibrarymath.code.tex L22-46) treats it as assignable; a primitive
/// math atom hits the keyword path and `\csname pgfmath\angle\endcsname`
/// loops to the error cap (Rust 1001, Perl 101). The math meaning must
/// survive in the space-suffixed inner CS.
#[test]
fn angle_tikzmath_variable() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{math}
\begin{document}
$\angle ABC$
\tikzmath{ \angle = 90 - 30; }
V=[\angle]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">\u{2220}</XMTok>"), "{xml}");
  assert!(xml.contains("V=[60.0]"), "{xml}");
}

/// Real listings sets its line counter at register level
/// (listings.sty L1516 `\global\c@lstnumber\lst@firstnumber`), never via
/// `\setcounter`; our block emitter used user-level `\setcounter`, which
/// xassoccnt.sty L2553 wraps in an expl3 body that (under our engine) runs
/// away into the following `\@lst@startline` → the first line leaks as
/// loose text under `<ltx:listing>` (518 malformed errors, xassoccnt_doc).
#[test]
fn listings_line_counter_init_is_register_level() {
  let tex = r"\documentclass{article}
\usepackage{xassoccnt}
\usepackage{listings}
\begin{document}
\section{S}
\begin{lstlisting}
Hello world
Second line
\end{lstlisting}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<listingline").count(), 2, "{xml}");
}

/// tabularray.sty:3461-3470 `\NewTblrEnviron{name}` creates a `tblr`-alias
/// environment; the binding had it as a no-op so `\begin{MPMtache}` was
/// undefined. Witness: profsio ProfSio-doc-fr (ProfSio.sty:98).
#[test]
fn tabularray_newtblrenviron_defines_environment() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\NewTblrEnviron{MPMtache}
\SetTblrInner[MPMtache]{colspec={Q[c]Q[c]}}
\begin{document}
\begin{MPMtache}{hlines={wd=1pt},vlines={wd=1pt}}
\SetCell[c=2]{c} {X} & \\
a & b \\
\end{MPMtache}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  assert!(xml.matches("<tr").count() >= 2, "{xml}");
}

/// xcolor.sty L1461 runs `\color{black}` at load, which defines the current
/// color `.` (`\color@.`); `\draw[.]` resolves via tikz's colour fallback.
/// Witness: twoxtwogame_doc (twoxtwogame.sty:493 `row player color=.`).
#[test]
fn xcolor_current_color_dot_defined_at_load() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\draw[line width=1pt, ., ] (0,0) -- (1,1);
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg:path"), "{xml}");
}

/// pgf driver handler for xcolor's core model `hsb` (xcolor.sty L1121-1132
/// folds Hsb/HSB/tHsb/wave into it); pgfcoregraphicstate.code.tex L195-202
/// errors "Unsupported color model" when `\pgfsys@color@hsb` is missing.
/// Witness: tikz-3dplot_documentation (tikz-3dplot.sty:731).
#[test]
fn pgfsys_hsb_color_model_supported() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\definecolor{tdplotfillcolor}{hsb}{0.5, 1, 1}
\fill[tdplotfillcolor] (0,0) rectangle (1,1);
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Unsupported color model"), "{stderr}");
  assert!(xml.contains("#00FFFF"), "{xml}");
}

/// eTeX `quotient` (etex.ch, `scan_expr`) rounds half AWAY from zero on
/// magnitudes: `\numexpr -1/2` = -1, `-7/2` = -4 (pdflatex-probed). Perl
/// Number.pm `int(0.5 + n/d)` truncates toward zero (KNOWN_PERL_ERRORS
/// #124), and l3fp's `\__fp_mul_cases_o:NnNnww` case index
/// (expl3-code.tex:18724-18760) relies on the TeX rounding — with the
/// Perl rounding `0 * x` inside a `+`/`-` expression collapsed the whole
/// `\fp_eval:n` to 0. Witness: wheelchart (wheelchart.sty:2423 transform
/// determinant → 1001 errors).
#[test]
fn numexpr_division_rounds_half_away_from_zero() {
  let tex = r"\documentclass{article}
\begin{document}
K[\the\numexpr -1/2\relax][\the\numexpr 1/2\relax][\the\numexpr -3/2\relax][\the\numexpr -7/2\relax][\the\numexpr 7/2\relax][\the\numexpr -5/-2\relax][\the\numexpr 5/-2\relax]

\ExplSyntaxOn
F[\fp_eval:n { 800 - 0 * 3 }][\fp_eval:n { (0*3) + 800 }][\fp_eval:n { -0 * 3 }]
\ExplSyntaxOff
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("K[-1][1][-2][-4][4][3][-3]"), "{xml}");
  assert!(xml.contains("F[800][800][-0]"), "{xml}");
  // `\dimexpr` shares `quotient` (pdflatex-probed 2026-09-02; the xy
  // `\dimexpr(\X@p+2\A@)/3` curve-control shape, xytest golden re-blessed).
  let tex = r"\documentclass{article}
\begin{document}
\newdimen\A \A=-107.6pt
D[\the\dimexpr -1sp/2\relax][\the\dimexpr 1sp/2\relax][\the\dimexpr -3sp/2\relax][\number\dimexpr\A/3\relax][\number\dimexpr(-1pt+2\A)/3\relax]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("D[-0.00002pt][0.00002pt][-0.00003pt][-2350558][-4722961]"),
    "{xml}"
  );
}

/// `\read` past end-of-file reads the synthetic empty line + `\endlinechar`
/// in state N (tex.web §345-349): an IGNORE (catcode 9) endline char can
/// never become a token. Perl Mouth.pm:303-307 emits it and the Stomach
/// reports `misdefined` (KNOWN_PERL_ERRORS #125). Witness: liftarm
/// (pgfmanual `codeexample` sets `\catcode`\^^M=9` around `\scantokens`,
/// animate.sty `\@anim@buildtmln` `\read`s the timeline to EOF — 501
/// errors capped).
#[test]
fn read_at_eof_drops_ignored_endlinechar() {
  let tex = r"\documentclass{article}
\begin{document}
\newread\myr
\openin\myr=rdtest.dat
\catcode`\^^M=9\relax
\read\myr to \la
\read\myr to \lb
\catcode`\^^M=5\relax
\closein\myr
X\lb X\la X
\end{document}
";
  let (stderr, xml) = convert_with_sty(tex, "rdtest.dat", "lineone\n");
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("should never reach Stomach"), "{stderr}");
  assert!(xml.contains("XXlineoneX"), "{xml}");
}

/// xkeyval.tex:518-529 + 560-583: `\XKV@s@tk@ys@` saves the raw value and
/// then `\XKV@replacepointers` splices every `\usevalue{X}` EAGERLY, under
/// the key's own header, before the key code runs. The binding resolved
/// `\usevalue` lazily at expansion time, so a value stored by the key code
/// and expanded later (inside another `\setkeys`) looked the pointer up
/// under the wrong family. Witness: pmdraw (pmdraw.sty:1704-1706 stores
/// `\pmdraw@tikz`, consumed at :1857-1860 — 501 errors capped). Perl
/// stubs the pointer system outright (xkeyval.sty.ltxml:397-432).
#[test]
fn xkeyval_usevalue_is_replaced_eagerly_at_setkeys() {
  let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key{d}{v}{\def\stored{#1}}
\define@key{dDefault}{v}{\setkeys{d}{\savevalue{v}=#1}}
\makeatother
\begin{document}
\makeatletter
\setkeys{dDefault}{v=42}
\setkeys{d}{v=\usevalue{v}}
\setkeys{e}{whatever}
[\stored]
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[42]"), "{xml}");
}

/// lstmisc.sty:60-64 `\lst@WFBegin` runs `\immediate\openout\lst@WF=#2`
/// on every FRESH `\lst@BeginWriteFile`/`\lst@BeginAlsoWriteFile`, so the
/// file is truncated per begin/end span (pdflatex: only the second write
/// survives). The display-time tee appended across spans, so
/// forest-doc.sty:59's per-example `\jobname.tmp` kept a stale
/// `\usepackage[linguistics]{forest}` line that every later
/// `\lst@sampleInput` re-`\input` — 440 "can only appear in the preamble"
/// errors. Witness: forest-doc (1001 errors + TooManyErrors).
#[test]
fn listings_writefile_truncates_on_fresh_begin() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\makeatletter
\lst@BeginAlsoWriteFile{\jobname.tmp}
\begin{lstlisting}
\usepackage[foo]{bar}
\end{lstlisting}
\lst@EndWriteFile
\lst@BeginAlsoWriteFile{\jobname.tmp}
\begin{lstlisting}
hello world
\end{lstlisting}
\lst@EndWriteFile
\makeatother
[\input{\jobname.tmp}]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The displayed listings still show both bodies (AlsoWriteFile); the
  // `\input`-back must see only the second span.
  assert!(xml.contains("[hello world"), "{xml}");
  assert_eq!(
    xml.matches("foo").count(),
    1,
    "stale first write re-input: {xml}"
  );
}

/// etoolbox.sty:1740-1746: under a 2020-10+ format `\AtEndPreamble` IS
/// `\AddToHook{begindocument/before}`, so it takes the hook system's
/// optional `[label]`. tcbdocumentation.code.tex:69 defines `\meta`
/// inside `\AtEndPreamble[tcolorbox]{…}`; the binding read `[tcolorbox]`
/// as the hook code. Witness: xassoccnt_doc (`undefined:\meta`).
#[test]
fn etoolbox_atendpreamble_accepts_hook_label() {
  let tex = r"\documentclass{article}
\usepackage[most,documentation]{tcolorbox}
\begin{document}
Syntax: \meta{true,false}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("true,false"), "{xml}");
  assert!(!xml.contains("tcolorbox]"), "label leaked as text: {xml}");
}

/// forest.sty:8506 `\NewDocumentEnvironment{forest}{D(){}}` also defines
/// the bare `\forest … \endforest` pair, and :1413 `\bracketset`;
/// neoschool.cls:8567-8581 builds `neotree` on the bare form and calls
/// `\bracketset{action character=@}` at load. The stub knew only
/// `\begin{forest}`, so the tree body (`w=\frac{1}{3}`) leaked into text
/// as XMApp errors. Witness: neoschool (4 errors). The stub's own
/// one-per-kind report is the single expected error.
#[test]
fn forest_bare_cs_form_discards_body() {
  let tex = r"\documentclass{article}
\usepackage{forest}
\bracketset{action character=@}
\begin{document}
A\forest [root [w=\frac{1}{3}] [b]]\endforest B
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  // The stub diagnostic is a Warn since batch 56k (`forest_stub_is_a_warning`).
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(stderr.contains("stub binding"), "{stderr}");
  assert!(!stderr.contains("undefined:\\forest "), "{stderr}");
  assert!(!stderr.contains("bracketset"), "{stderr}");
  assert!(!xml.contains("XMApp"), "tree body leaked: {xml}");
  assert!(xml.contains("A") && xml.contains("B"), "{xml}");
}

/// latex.ltx:14103-14107 `\@setfontsize` only `\let\@currsize#1` under
/// `\ifx\protect\@typeset@protect`, so it is inert inside `\protected@edef`.
/// Our binding (and Perl latex_constructs.pool:5622, which OOMs same-host)
/// dropped the guard: a raw class routing its size commands through
/// `\@setfontsize` (tufte-common.def:368-405) re-expanded
/// `\@currsize`→`\normalsize`→`\@setfontsize\normalsize…` without bound
/// once pgf edef'd tikz-network's `font=\normalsize` label. Witness:
/// tikz-network manual (PushbackLimit Fatal, no output).
#[test]
fn setfontsize_is_inert_inside_protected_edef() {
  let tex = r"\documentclass{article}
\makeatletter
\renewcommand\normalsize{\@setfontsize\normalsize\@xpt{14}}
\protected@edef\lx@probe{\normalsize}
\makeatother
\begin{document}
probe ok
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("PushbackLimit"), "{stderr}");
  assert!(xml.contains("<p>probe ok</p>"), "{xml}");
}

/// xkeyval.tex:569 `\XKV@ifundefined{XKV@<header><key>@value}` tests
/// DEFINEDNESS: a key saved with an EMPTY value (`\savevalue{k}={}`, L525-527)
/// is `\let` to an empty-bodied macro and `\usevalue{k}` splices nothing.
/// `replace_pointers` read `get_expansion()`, which is `None` for an empty
/// body, and reported "no value recorded". Witness: pmdraw
/// (pmdraw.sty:2191-2264 sets 16 defaults to `{}` — 501 errors + cap).
#[test]
fn xkeyval_usevalue_of_empty_saved_value_is_empty() {
  let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key{fam}{k}{\def\myval{#1}}
\define@key{famDefault}{k}{\setkeys{fam}{\savevalue{k}=#1}}
\setkeys{famDefault}{k={}}
\setkeys{fam}{k=\usevalue{k}}
\makeatother
\begin{document}
START\myval END
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("START END") || xml.contains("STARTEND"),
    "{xml}"
  );
}

/// Perl TeX_Debugging.pool.ltxml:110-113 reduces a primitive / conditional /
/// constructor to its cs-or-alias token before rendering `\meaning`; our
/// `DefMath` atoms are a separate `Stored::MathPrimitive` and fell to the
/// catch-all `Stored[??]`. Witness: sunpath (tikzmath `\meaning` sniffing).
#[test]
fn meaning_of_defmath_atom_is_its_cs() {
  let tex = r"\documentclass{article}
\begin{document}
[\expandafter\detokenize\expandafter{\meaning\forall}][\expandafter\detokenize\expandafter{\meaning\infty}]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // (the backslash itself is OT1-decoded in text; the names are the signal)
  assert!(xml.contains("forall][") && xml.contains("infty]"), "{xml}");
  assert!(!xml.contains("Stored["), "{xml}");
}

/// delarray.sty:43-58 — `\@@array[pos]` peeks with `\@ifnextchar\bgroup`;
/// a non-brace is a delimiter pair around the column spec,
/// `\begin{array}({cc})…\end{array}` = `\left(` array `\right)`. Both
/// engines' own `\array[]{}` read `(` as the template, so every `&`
/// reported "Extra alignment tab" (memoir manual, memoir.cls:5468
/// `\RequirePackage{delarray}`: 33 errors; SHARED). pdflatex clean.
#[test]
fn delarray_delimited_array_form() {
  let tex = r"\documentclass{article}
\usepackage{delarray}
\begin{document}
$\begin{array}({cc}) a & b \\ c & d \end{array}$
$\begin{array}[t]\{{lL}. x \\ y \end{array}$
$\begin{array}{c} p \\ q \end{array}$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Extra alignment tab"), "{stderr}");
  assert_eq!(xml.matches("<XMArray").count(), 3, "{xml}");
  // the delimiters survive as fence tokens around the arrays
  assert!(
    xml.contains(r#"role="OPEN">(</XMTok>"#) || xml.contains(r#">(</XMTok>"#),
    "{xml}"
  );
  assert!(xml.contains(r#">{</XMTok>"#), "{xml}");
}

/// etoolbox.sty:1743: `\AtEndPreamble` IS `\AddToHook{begindocument/before}`,
/// so it queues in order with doc.sty:907-910's chunk that loads hypdoc
/// (→ hyperref) at `\begin{document}`; a private list that fired before
/// the L3 hook saw `\hypersetup` undefined. Witnesses: liftarm.tex:39,
/// wheelchart.tex:128 (ltxdoc manuals; SHARED with Perl).
#[test]
fn etoolbox_atendpreamble_runs_after_earlier_begindocument_before_chunks() {
  let tex = r"\documentclass{ltxdoc}
\usepackage{etoolbox}
\AtEndPreamble{\hypersetup{colorlinks=true}}
\begin{document}
Hello.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Hello.</p>"), "{xml}");
}

/// pgfsys-latexml.def.ltxml:392-398 opens a self-contained `svg:svg` when
/// `\lxSVG@begingroup@` fires inside an `ltx:` box within a picture; a
/// BARE-style path (no dash/color) never passes through the group opener,
/// so `\phantom{\draw …}` inside a tikzpicture relocated its `svg:path` up
/// to the picture group and desynced every later close — pmdraw manual
/// (`vertices top phantom`, pmdraw.sty:56-66): 64 errors. SHARED (Perl 7 on
/// this repro); `ensure_svg_context` now guards the path emitters too.
#[test]
fn pgf_bare_path_inside_phantom_stays_in_its_box() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{center}\begin{minipage}{0.85\textwidth}\begin{minipage}[c]{0.4\linewidth}
\raisebox{0.5cm}{\begin{tikzpicture}\phantom{\draw (0,0)--(1,1);}\draw (0,0)--(2,0);\end{tikzpicture}}
\end{minipage}\end{minipage}\end{center}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // the phantom's drawing nests inside its own foreignObject box
  let fo = xml
    .find("<svg:foreignObject")
    .expect("phantom foreignObject");
  let after = &xml[fo..];
  let close = after.find("</svg:foreignObject>").unwrap();
  assert!(
    after[..close].contains("<svg:path"),
    "phantom path escaped its box: {xml}"
  );
  assert_eq!(xml.matches("<svg:svg").count(), 2, "{xml}");
}

/// End-to-end hobby curve: the raw `hobby.code.tex` load (stub retired) plus
/// l3fp's comparison chain-detect (`\__fp_parse_compare_auxi:NNNNNNN`,
/// expl3-code.tex:17662 — the backquote of a detokenized backslash must
/// read 92, KPE #123) yield a Hobby-smoothed cubic. Witness: hobby manual
/// (373 paths, was ~empty + `Until:@` Fatal; Perl 101 errors + Fatal).
#[test]
fn hobby_shortcut_draws_a_cubic_path() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{hobby}
\begin{document}
\begin{tikzpicture}[use Hobby shortcut]
\draw (0,0) .. (1,1) .. (2,0);
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Until:@"), "{stderr}");
  assert!(xml.contains(r#"d="M 0 0 C "#), "no Hobby cubic: {xml}");
}

/// tabularray.sty:3472-3477 builds `longtblr`/`talltblr` with the same
/// factory as `tblr`; the binding knew only `tblr`, so `{longtblr}` was an
/// undefined environment whose body cascaded (panda manual: 149
/// `<relationaltoken>` errors + `Until:` EoF Fatal).
#[test]
fn tabularray_longtblr_and_talltblr_are_tblr() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{longtblr}[theme=naked]{colspec={Xll}, rowhead=1}
A & B & C \\
1 & 2 & 3 \\
\end{longtblr}
\begin{talltblr}{colspec={cc}}
x & y \\
\end{talltblr}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
  assert_eq!(xml.matches("<tr").count(), 3, "{xml}");
}

/// expl3-code.tex:3758-3790: `\tl_set_rescan:Nnn` captures the WHOLE
/// `\scantokens` output (PARAM tokens included) through `\everyeof` +
/// `\__tl_rescan:NNw`'s delimited scan. Our `\scantokens` cannot carry the
/// `\everyeof` payload (P15 dead-end), so the scan ran to EOF and a
/// rescanned macro MEANING leaked its `#`s to digestion — substances.sty:452
/// (substances manual, 720 `misdefined:#`; Perl identical). The core now
/// rescans atomically under the caller's catcodes.
#[test]
fn tl_set_rescan_captures_param_tokens() {
  let tex = r"\documentclass{article}
\ExplSyntaxOn
\cs_new:Npn \FooEntry #1#2#3 { #1@#3|see{#2} }
\cs_new_protected:Npn \contains_see:N #1
  {
    \tl_set_rescan:Nnx \l_tmpa_tl {} {\cs_meaning:N #1 }
    \tl_if_in:VnT \l_tmpa_tl { |see } { YESSEE }
  }
\tl_set_rescan:Nnn \l_tmpb_tl { \char_set_catcode_other:N \\ } { A\B }
\ExplSyntaxOff
\begin{document}
\ExplSyntaxOn \contains_see:N \FooEntry [\tl_use:N \l_tmpb_tl] \ExplSyntaxOff
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("misdefined"), "{stderr}");
  assert!(xml.contains("YESSEE"), "rescan lost the meaning: {xml}");
  // the caller's catcode setup governs the rescan: `\` as OTHER is text
  assert!(xml.contains("[A") && xml.contains("B]"), "{xml}");
}

/// tex.web §442: a brace read as a character constant (`` `} ``) undoes the
/// `align_state` step `get_token` applied, so `\iffalse{\fi\ifnum0=`}\fi`
/// (expl3 `\group_align_safe_begin:`, amsmath) leaves ALIGN_STATE +1 with no
/// group open. Without the undo the idiom netted 0 and an alignment-catcode
/// token in a delimited-macro definition inside a cell — l3tl
/// `\tl_replace_all` with a rescanned `_`(4), l3doc `\marg` inside `syntax`
/// (every l3doc manual) — was taken as the cell end (`Until:…@after_` EoF
/// Fatal). Perl Gullet.pm:926 shares the gap.
#[test]
fn backquote_brace_charcode_keeps_align_state() {
  let tex = r"\documentclass{article}
\usepackage{expl3}
\begin{document}
\begin{tabular}{l}
\begin{minipage}{3cm}
\ExplSyntaxOn
\tl_set_rescan:Nnn \l_tmpa_tl { \char_set_catcode:nn { `_ } {4} } { _ }
\tl_set:Nn \l_tmpb_tl { a_b }
\tl_replace_all:NVn \l_tmpb_tl \l_tmpa_tl { X }
[\tl_use:N \l_tmpb_tl]
\ExplSyntaxOff
\end{minipage}
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Until:"), "{stderr}");
  // the letter-catcode `_` in `a_b` is not the catcode-4 pattern: untouched
  assert!(xml.contains("[a") && xml.contains("b]"), "{xml}");
  assert_eq!(xml.matches("<td").count(), 1, "{xml}");
}

/// l3doc `\marg`/`\oarg` inside `{syntax}` (a tabular+minipage) — the
/// corpus-wide face of `backquote_brace_charcode_keeps_align_state`
/// once `\tl_set_rescan` captures alignment-catcode tokens.
#[test]
fn l3doc_marg_inside_syntax_env() {
  let tex = r"\documentclass{l3doc}
\begin{document}
\begin{function}{\zcheck}
\begin{syntax}
\cs{zcheck} \oarg{options} \marg{labels}
\end{syntax}
Typesets \meta{text}.
\end{function}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("labels"), "{xml}");
}

/// latex-lab-testphase-tikz.sty:228-260 adds `/tikz/alt` and friends when
/// `\DocumentMetadata` is active; the picture's alt text is recorded (no
/// XML slot yet) and `\tagtool`/`\DebugBlocksOff` are PDF-structure-only.
/// Witness: tagpdf manual (9× "I do not know the key '/tikz/alt'").
#[test]
fn documentmetadata_tikz_alt_key_and_tagging_tools() {
  let tex = r"\DocumentMetadata{tagging=on}
\documentclass{article}
\usepackage{tikz}
\DebugBlocksOff
\begin{document}
\tagtool{para/tag=P}
\begin{tikzpicture}[alt={A red circle}]
\fill[red] (0,0) circle (2pt);
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg:path"), "{xml}");
}

/// tabularray.sty:3444 `\SetTblrInner[<envs>]{keys}` records per-environment
/// inner defaults that every `\begin{<env>}` prepends, and a table with no
/// colspec anywhere takes its column count from the rows. The combination
/// `\NewTblrEnviron` with `\SetTblrInner[spectblr]{hlines…}` and
/// `\begin{spectblr}[…]{}` had become a zero-column template (pegmatch
/// manual: 52 "Extra alignment tab").
#[test]
fn tabularray_settblrinner_defaults_and_inferred_columns() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\NewTblrEnviron{spectblr}
\SetTblrInner[spectblr]{hlines, rowhead=1}
\SetTblrInner[tblr]{colspec={lc}}
\begin{document}
\begin{spectblr}[caption=Basic]{}
Command & Description & More \\
a & b & c \\
\end{spectblr}
\begin{tblr}{}
x & y \\
\end{tblr}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
  assert_eq!(xml.matches("<tr").count(), 3, "{xml}");
  assert_eq!(xml.matches("<td").count(), 8, "{xml}");
  assert!(
    xml.contains(r#"align="center""#),
    "stored colspec lost: {xml}"
  );
}

/// tabularray colspec inter-column material `@{…}`/`!{…}` translates through
/// (not a column). A bailed `colspec={@{}Xll@{}}` made the WHOLE inner spec
/// the tabular template, whose `cell{…}={cmd={…}}` value was edef-expanded
/// in the preamble — panda manual (`\BusyPanda` fp → `Until:\__fp_sep:`
/// EoF Fatal).
#[test]
fn tabularray_colspec_intercolumn_material() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{colspec={@{}Xll@{}}, cell{2-Z}{2}={cmd={\textbf}}}
a & b & c \\
d & e & f \\
\end{tblr}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  assert_eq!(xml.matches("<td").count(), 6, "{xml}");
}

/// Begin-document hook code runs RE-LOCKED (Perl State.pm:502-514 ignores a
/// redefinition of a `:locked` cs under `$UNLOCKED=0`; Perl never fires the
/// L3 `begindocument` hook at all). Our `\hook_use:n{begindocument}` digest
/// ran unlocked, so polyglossia.sty:1442-1456's `\cs_set:Npn \@caption
/// #1[#2]#3` replaced the locked `\@caption` and its `[`-scan overshot every
/// figure. Witness: beamerdarkthemes user guide (101 caption errors).
#[test]
fn begindocument_hook_code_cannot_redefine_locked_caption() {
  let tex = r"\documentclass{article}
\makeatletter
\AddToHook{begindocument}{%
  \let\xpgsave\@caption
  \long\def\@caption#1[#2]#3{\xpgsave{#1}[{\ignorespaces#2}]{#3}}}
\makeatother
\begin{document}
\begin{figure}\caption{cormorant color theme}\label{fig:a}\end{figure}
\begin{figure}[p]\caption{magpie}\label{fig:b}\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<caption").count(), 2, "{xml}");
  assert!(xml.contains("cormorant color theme"), "{xml}");
}

/// latex.ltx:15392 `\end` ends with `\if@ignore\@ignorefalse\ignorespaces\fi`
/// after the `env/<name>/after` hook; noindentafter.sty:44
/// `\nia@afterendenv#1\ignorespaces\fi` is delimited by those tokens and
/// otherwise scans to EOF (pkgloader manual, 102 errors; Perl identical).
#[test]
fn end_environment_emits_the_ignorespaces_epilogue() {
  let tex = r"\documentclass{article}
\usepackage{noindentafter}
\NoIndentAfterEnv{itemize}
\begin{document}
\begin{itemize}\item a\end{itemize}
Text after list. $x$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Until:"), "{stderr}");
  assert!(xml.contains("<itemize"), "{xml}");
  assert!(
    xml.contains("Text after list.") && xml.contains("<Math"),
    "{xml}"
  );
}

/// ulem.sty:232-233 extension contract: `\bgroup\markoverwith{…}\ULon{text}`
/// — the word machinery closes the group (`\UL@end *`, :59). The binding's
/// inert internals lacked `\markoverwith`/`\ULon` and never closed it, so
/// CJKfntef's `\CJKunderline` (CJKfntef.sty:258-283) unbalanced the
/// enclosing list. Witness: jnuexam examfc-a-answer (+8 CJKfntef manuals).
#[test]
fn ulem_markoverwith_ulon_contract_closes_its_group() {
  let tex = r"\documentclass{article}
\usepackage{ulem}
\makeatletter
\def\myul{\bgroup\markoverwith{\hbox{x}}\ULon}
\makeatother
\begin{document}
\begin{description}
\item[A] before \myul{XYZ} after \myul{second}.
\end{description}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("XYZ") && xml.contains("second"), "{xml}");
  assert_eq!(xml.matches("<description").count(), 1, "{xml}");
}

/// tex.web §370: an UNDEFINED control sequence met inside `\csname…\endcsname`
/// is reported and discarded; the name scan continues to `\endcsname`. The
/// error stub made it look defined, so the §373 `back_error` path ended the
/// name early and the real `\endcsname` went stray ("Extra \endcsname":
/// 1693 lines / 65 manuals — beamer2thesis's babel `\csname l@\beamer@…`,
/// gckanbun's pgf arrow declarations). One error, like pdflatex.
#[test]
fn csname_discards_an_undefined_cs_and_keeps_scanning() {
  let tex = r"\documentclass{article}
\makeatletter
\begin{document}
A\csname l@\beamer@torinoth@language\endcsname B
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(!stderr.contains("Extra \\endcsname"), "{stderr}");
  assert!(!stderr.contains("should not appear"), "{stderr}");
  assert!(xml.contains("A") && xml.contains("B"), "{xml}");
}

/// etex.sty's register allocators (`\globcount`…`\loctoks`…, etex.sty:332-348)
/// are PACKAGE macros; Perl's eTeX pool defines none. In the always-on pool
/// they made l3sort freeze the `\cs_if_exist:NT \loctoks` branch of
/// `\__sort_compute_range:` (expl3-code.tex:23356-23364, `\count265`/
/// `\count275`) into the dump, and once a package load left `\count265` > 0
/// every `\seq_sort` ran an inverted range to the TokenLimit (spath3
/// `insert gaps after components`, tabularray/testidx/cistercian manuals).
/// Needs the regenerated dump (base `\count15` branch, Perl
/// latex_dump.pool:8589).
#[test]
fn l3sort_after_package_registers_terminates() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{spath3}
\begin{document}
\ExplSyntaxOn
\seq_set_from_clist:Nn \l_tmpa_seq { 3 , 1 , 2 }
\seq_sort:Nn \l_tmpa_seq { \int_compare:nNnTF {#1} < {#2} { \sort_return_same: } { \sort_return_swapped: } }
[\seq_use:Nn \l_tmpa_seq { - }]
\ExplSyntaxOff
\begin{tikzpicture}
\draw[spath/save=p] (0,0) -- (1,0) (2,0) -- (3,0);
\tikzset{spath/.cd, insert gaps after components={p}{10pt}{1}}
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("TokenLimit"), "{stderr}");
  assert!(xml.contains("[1-2-3]"), "{xml}");
  assert!(xml.contains("<svg:path"), "{xml}");
  // no etex.sty loaded: the allocators stay undefined, like Perl/LaTeX
  let (stderr2, xml2) = convert(
    r"\documentclass{article}\begin{document}[\ifdefined\loctoks Y\else N\fi]\end{document}",
    false,
  );
  assert_eq!(error_count(&stderr2), 0, "{stderr2}");
  assert!(xml2.contains("[N]"), "{xml2}");
}

/// A `fnum@font@<type>` value wraps the number as a braced argument
/// (enumitem.sty:451/1478 `\enit@format{<label>}`); as a bare prefix an
/// argument-taking font command grabbed the following `\@ifundefined`
/// (non-decimal-units manual: `\setlist[description]{font=\docAuxKey}`,
/// 40 errors; Perl Base_Utility.pool:1041 identical).
#[test]
fn enumitem_font_wraps_the_item_tag() {
  let tex = r"\documentclass{article}
\usepackage{enumitem}
\setlist[description]{font=\textbf}
\begin{document}
\begin{description}
\item[british] Currencies
\item[danish] Areas
\end{description}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("british") && xml.contains("danish"), "{xml}");
  assert!(
    xml.contains(r#"font="bold""#),
    "font not applied to the tag: {xml}"
  );
}

/// memoir.cls:5477-5719 auto-tables reduce to `\tabular`: `\autorows` fills
/// `num` columns row-major, `\autocols` column-major with the `\linespercol`
/// heights (:5665-5675, greedy ceil — column 0 tallest; the manual's own
/// `\showit` mock is wrong), `{ctabular}`'s `[pos]` is horizontal. The raw
/// code drives `\valign`/`\@mkpream` internals the engine never provides
/// (memman: ~157 errors; SHARED).
#[test]
fn memoir_auto_tables_reduce_to_tabular() {
  let tex = r"\documentclass{memoir}
\begin{document}
\autorows{c}{5}{c}{one, two, three, four, five, six, seven, eight, nine, ten,
eleven, twelve, thirteen, fourteen}
\autocols{c}{5}{l}{one, two, three, four, five, six, seven, eight, nine, ten,
eleven, twelve, thirteen, fourteen}
\begin{ctabular}[l]{lcr}
LEFT & CENTER & RIGHT \\
l & c & r \\
\end{ctabular}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tabular").count(), 3, "{xml}");
  assert_eq!(xml.matches("<tr").count(), 8, "{xml}");
  // autorows: first row one…five; autocols: first row one,four,seven,ten,thirteen
  let rows: Vec<&str> = xml.split("<tr").skip(1).collect();
  let cells = |r: &str| -> Vec<String> {
    r.split("<td")
      .skip(1)
      .map(|c| {
        c.split('>')
          .nth(1)
          .unwrap_or("")
          .split('<')
          .next()
          .unwrap_or("")
          .trim()
          .to_string()
      })
      .collect()
  };
  assert_eq!(
    cells(rows[0]),
    ["one", "two", "three", "four", "five"],
    "{xml}"
  );
  assert_eq!(
    cells(rows[2])[..4],
    ["eleven", "twelve", "thirteen", "fourteen"],
    "{xml}"
  );
  assert_eq!(
    cells(rows[3]),
    ["one", "four", "seven", "ten", "thirteen"],
    "{xml}"
  );
  assert_eq!(
    cells(rows[5])[..4],
    ["three", "six", "nine", "twelve"],
    "{xml}"
  );
  assert_eq!(cells(rows[6]), ["LEFT", "CENTER", "RIGHT"], "{xml}");
}

/// pdfTeX `\pdfuniformdeviate <n>` expands to a random integer in [0,n);
/// the empty macro (Perl pdfTeX.pool:110) also ate the next token, so expl3's
/// `\int_rand:nn` (`\tex_uniformdeviate:D 268435456 \__fp_sep:`) lost its
/// separator and returned the midpoint — rejection-sampling loops never
/// terminated (randintlist-l3 manual, TokenLimit). Deterministic seed:
/// the same document converts identically; `\pdfsetrandomseed` re-seeds.
#[test]
fn pdfuniformdeviate_is_a_random_integer() {
  let tex = r"\documentclass{article}
\begin{document}
\ExplSyntaxOn
[\int_rand:nn{1}{1000},\int_rand:nn{1}{1000},\int_rand:nn{1}{1000},\int_rand:nn{1}{1000}]
[\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ,\pdfuniformdeviate 10 ]
\pdfsetrandomseed 42 [\the\pdfrandomseed]
\ExplSyntaxOff
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let (_, xml2) = convert(tex, false);
  assert_eq!(
    xml, xml2,
    "random stream must be deterministic per conversion"
  );
  let first = xml.split('[').nth(1).unwrap().split(']').next().unwrap();
  let vals: Vec<i64> = first
    .split(',')
    .map(|v| v.trim().parse().unwrap())
    .collect();
  assert_eq!(vals.len(), 4, "{xml}");
  assert!(vals.iter().all(|&v| (1..=1000).contains(&v)), "{xml}");
  assert!(
    vals.windows(2).any(|w| w[0] != w[1]),
    "constant stream: {xml}"
  );
  let second = xml.split('[').nth(2).unwrap().split(']').next().unwrap();
  assert!(
    second
      .split(',')
      .all(|v| (0..10).contains(&v.trim().parse::<i64>().unwrap())),
    "{xml}"
  );
  assert!(xml.contains("[42]"), "{xml}");
}

/// listings `\lstnewenvironment{x}{<begin>}{<end>}`: the end code runs at
/// the environment's group level AFTER the listing (listings.sty
/// `\lst@EndProcess`/`\lstnewenvironment`→`\newenvironment`), so a
/// mode-switching begin/end pair (`\mdframed`…`\endmdframed`, cnltx's
/// `sourcecode` env, `\begin{minipage}`…) balances. The display wrapper
/// `{\def\lstname{…} <block>}` had the postamble INSIDE its braces, so
/// `\endmdframed` met the wrapper's `{` frame ("Attempt to end mode
/// internal_vertical" — cnltx_en 921×, chemnum 654×, pixelart 703×,
/// modiagram 896×, tasks 171×; Perl listings.sty.ltxml:205-212 identical).
#[test]
fn lstnewenvironment_end_code_runs_outside_the_listing_group() {
  let tex = r"\documentclass{article}
\usepackage{mdframed,listings}
\lstnewenvironment{foo}{\mdframed}{\endmdframed}
\lstnewenvironment{bar}{\begin{minipage}{3cm}}{\end{minipage}}
\begin{document}
Before.
\begin{foo}
code here
\end{foo}
\begin{bar}
more code
\end{bar}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<listing class").count(), 2, "{xml}");
  assert!(
    xml.contains("framed=\"rectangle\""),
    "mdframed block lost: {xml}"
  );
  // the frame covers the listing: either as a wrapper element or, since
  // batch 56u routes mdframed through `insert_block` (Perl insertBlock's
  // single-node rule), as `framed="rectangle"` on the listing element
  // itself — the same way the minipage's `ltx_minipage` lands on the second
  let frame = xml.find("framed=\"rectangle\"").unwrap();
  let first_listing = xml.find("<listing class").unwrap();
  let first_tag_end = first_listing + xml[first_listing..].find('>').unwrap();
  assert!(frame < first_tag_end, "{xml}");
  assert!(xml.contains("<p>After.</p>"), "{xml}");
}

/// tex.web §982/§987: `\pagegoal` is `\vsize` once the page has content;
/// with no page builder the standing value must serve every "free space"
/// probe — Perl's 0 loops fullwidth.sty:243-273, `\maxdimen` sent
/// fillwith.sty:319's coffin stacking after a 16384pt goal (TokenLimit).
/// `\strutbox` is the real latex.ltx:12596 strut (.7/.3 `\baselineskip`),
/// not void, so `\strut`-based line heights are honest.
#[test]
fn pagegoal_is_vsize_and_strutbox_is_real() {
  let tex = r"\documentclass{article}
\begin{document}
[\the\pagegoal][\the\vsize][\the\ht\strutbox,\the\dp\strutbox]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[1000.0pt][1000.0pt]"), "{xml}");
  assert!(xml.contains("[8.39996pt,3.60004pt]"), "{xml}");
}

/// fullwidth.sty:243-273 `\fwd@freepagevspace` retries `\vfill\eject` while
/// `\pagegoal - \pagetotal < 2\baselineskip`; with `\pagegoal=\vsize` the
/// frame is placed at once (Perl's `\pagegoal=0` loops).
#[test]
fn fullwidth_frame_does_not_retry_forever() {
  let tex = r"\documentclass{article}
\usepackage{fullwidth}
\begin{document}
Before.
\begin{fullwidth}
Wide text.
\end{fullwidth}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Not enough space"), "{stderr}");
  assert!(
    xml.contains("Wide text.") && xml.contains("After."),
    "{xml}"
  );
}

/// OXIDIZED_DESIGN #176 / KNOWN_PERL_ERRORS #131: a zero-width `\vrule` is a
/// strut, not a column rule — with the real `\strutbox` every TeXbook
/// `\halign{\strut#&\vrule#&…}` template (halignatt.tex) otherwise grew an
/// empty bordered cell per row, and Perl marks the explicit idiom
/// `border="ll"`.
#[test]
fn zero_width_vrule_is_a_strut_not_a_border() {
  let tex = r"\documentclass{article}
\begin{document}
\halign{\vrule height 12pt width 0pt#&\vrule#&#\cr &&a\cr}
\halign{\strut#&\vrule#&#\cr &&b\cr}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<td align="left" border="l" class="ltx_nopad_l ltx_nopad_r">a</td>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<td align="left" border="l" class="ltx_nopad_l ltx_nopad_r">b</td>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<td").count(), 2, "{xml}");
}

/// latex.ltx:18856 runs the `-h@@k` before `\@popfilename` restores
/// `\catcode`\@`, so `\AtEndOfPackage` code reads `@`-names as single
/// control sequences: europecv.cls:27 inputs `ecven.def` from the hook and
/// its `\ecv@utf` split into `\ecv`+`@utf` looped the title row to the
/// pushback limit (KNOWN_PERL_ERRORS #132).
#[test]
fn at_end_of_package_hook_runs_with_at_letter() {
  let cls = "\\ProvidesClass{hookcls}\n\
       \\AtEndOfPackage{\\InputIfFileExists{hookcls.def}{}{}}\n\
       \\newcommand\\hook@one{ONE}\n\
       \\LoadClass{article}\n";
  let def = "\\providecommand\\hooktwo{[\\hook@one]}\n";
  let tex = "\\documentclass{hookcls}\n\\begin{document}\n\\hooktwo\n\\end{document}\n";
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(tex, &[
    ("hookcls.cls", cls),
    ("hookcls.def", def),
  ]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[ONE]"), "{xml}");
}

/// catchfile.sty:264-296: `\CatchFileDef\cs{file}{setup}` runs `setup`
/// inside a group and reads the file's tokens under the catcodes it left —
/// `\catcode`\#=12` (codehigh/fontscale `\dochighinput` reads a .sty whose
/// `#` would otherwise be a parameter token) and `\endlinechar=-1` — while
/// `\CatchFileEdef` expands the contents. Both define the target at the
/// outer level (`\let#1` after `\endgroup`). Witnesses: fontscale-code,
/// cistercian manuals (codehigh); arXiv 2210.08043, 1611.01359.
#[test]
fn catchfiledef_reads_under_setup_catcodes_and_edef_expands() {
  let txt = "A#1\\foo B\nC\n";
  let tex = "\\documentclass{article}\\usepackage{catchfile}\n\
       \\def\\foo{FOO}\n\
       \\begin{document}\n\
       \\CatchFileDef\\raw{caught.txt}{\\catcode`\\#=12 \\endlinechar=-1 }\n\
       \\CatchFileEdef\\exp{caught.txt}{\\catcode`\\#=12 \\endlinechar=-1 }\n\
       [\\detokenize\\expandafter{\\raw}][\\detokenize\\expandafter{\\exp}]\n\
       \\end{document}\n";
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(tex, &[("caught.txt", txt)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // `\detokenize`'s backslash renders through OT1 as `“`.
  assert!(xml.contains("[A#1“foo BC][A#1FOOBC ]"), "{xml}");
}

/// OXIDIZED_DESIGN #177: `\usepackage{../tex/pkg}` (CTAN source layout,
/// tikzpingus-doc.tex:16 and 60 more manuals) resolves nowhere in the
/// installed tree; the basename does, so it is loaded instead. A relative
/// path that DOES resolve still loads the local file.
#[test]
fn relative_package_path_falls_back_to_basename() {
  let xspace = "\\ProvidesPackage{xspace}\\newcommand\\localmarker{LOCALXSPACE}\n";
  let tex = "\\documentclass{article}\n\
       \\usepackage{../tex/xcolor}\n\
       \\usepackage{./local/xspace}\n\
       \\begin{document}\n\
       \\textcolor{red}{R}\\localmarker\n\
       \\end{document}\n";
  let (stderr, xml) =
    super::perfect_kernel_batch46::convert_files(tex, &[("local/xspace.sty", xspace)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"color="#FF0000""##), "{xml}");
  assert!(xml.contains("LOCALXSPACE"), "{xml}");
}

/// OXIDIZED_DESIGN #178: `\clearpage`/`\newpage` advance `\c@page`
/// (latex.ltx:15271), so knowledge.tex:803-809's pad-to-page loop
/// terminates — Perl hangs, Rust's box-cycle guard fataled.
#[test]
fn clearpage_advances_the_page_counter() {
  let tex = r"\documentclass{article}
\begin{document}
\newcommand{\filluptopage}[1]{\clearpage\loop\ifnum\value{page}<#1\relax\null\clearpage\repeat}
\filluptopage{4}
Done [\thepage].
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal:"), "{stderr}");
  assert_eq!(
    xml.matches("<pagination role=\"newpage\"").count(),
    3,
    "{xml}"
  );
  assert!(xml.contains("Done [4]."), "{xml}");
}

/// tex.web §977: `\vsplit` stores the remainder at the register's existing
/// eq_level, so a drain inside `{…}` survives the group — eledmac.sty:1363
/// `\do@line` relies on it (eledform example: box-list runaway). The
/// `\ifnum>50` cap turns a regression into a failed assertion, not a hang.
#[test]
fn vsplit_drain_survives_the_enclosing_group() {
  let tex = r"\documentclass{article}
\begin{document}
\newbox\rawt\setbox\rawt=\vbox{a\par b\par c}\count255=0
\loop\ifvbox\rawt {\global\setbox0=\vsplit\rawt to 100pt}\advance\count255 by1
  \ifnum\count255>50 \global\setbox\rawt=\box\voidb@x\fi\repeat
[\the\count255]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[1]"), "{xml}");
}

/// biblatex.sty:8995-9024 binds the `.bbl` commands (`\list`, `\name`,
/// `\field`…) only while the `.bbl` is read; a document-wide `\list{}{}{}`
/// (KNOWN_PERL_ERRORS #133) shadowed LaTeX's `\list{label}{setup}` for
/// every list environment (cnltx-doc `commands` under `add-bib`).
#[test]
fn biblatex_bbl_commands_do_not_shadow_list() {
  let tex = r"\documentclass{article}
\usepackage{biblatex}
\newenvironment{mylist}{\list{}{\leftmargin=0pt}}{\endlist}
\begin{document}
\begin{mylist}\item one\end{mylist}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<itemize"), "{xml}");
  assert!(xml.contains("<item"), "{xml}");
}

/// fontspec-xetex.sty:755-767: `\newfontfamily\Foo{…}` DEFINES `\Foo` as a
/// robust font switch (papiergurvan `\BelleAllureGras`; unicodefonttable's
/// `\setfontface` target must be non-empty for `\tl_if_empty:NF`).
#[test]
fn fontspec_definers_define_a_font_switch() {
  let tex = r"\documentclass{article}
\usepackage{fontspec}
\newfontfamily\Foo[Scale=1.1]{Belle Allure}[Ligatures=TeX]
\setfontface\Bar{Some Font.otf}
\begin{document}
{\Foo abc}{\Bar def}[\ifx\Bar\empty EMPTY\else BODY\fi]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("abcdef[BODY]"), "{xml}");
}

/// latex.ltx:18843-18875: `package/<name>/before|after` (and the
/// `file/<name>.sty/after` form) fire around a package load whichever way
/// it loads — binding (xspace) or raw (a local .sty). tudapub.cls hooks
/// scrbook's `\addchap` via `class/scrbook/after` (DEMO-TUDaPhD).
#[test]
fn package_after_hook_fires_for_a_binding_load() {
  let sty = "\\ProvidesPackage{rawpkg}\\newcommand\\rawmark{RAW}\n";
  let tex = "\\documentclass{article}\n\
       \\AddToHook{package/xspace/after}{\\def\\afterx{AX}}\n\
       \\AddToHook{package/xspace/before}{\\def\\beforex{BX}}\n\
       \\AddToHook{file/rawpkg.sty/after}{\\def\\afterraw{AR}}\n\
       \\AddToHook{package/rawpkg/after}{\\let\\rawmarktwo\\rawmark}\n\
       \\usepackage{xspace}\\usepackage{rawpkg}\n\
       \\begin{document}\n\
       [\\beforex\\afterx\\afterraw\\rawmarktwo]\n\
       \\end{document}\n";
  let (stderr, xml) = super::perfect_kernel_batch46::convert_files(tex, &[("rawpkg.sty", sty)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[BXAXARRAW]"), "{xml}");
}

/// chemmacros raw-loads (its stub's `\ch` → `\ensuremath{\mathrm{#1}}`
/// overrode chemformula's `\ch`: chemformula manual, 90+ errors) and its
/// `formula=chemformula` method finds the chemformula l3 API
/// (chemmacros.sty:1358-1366 → `\chemformula_chcpd:nn`).
#[test]
fn chemmacros_raw_load_keeps_chemformula_ch() {
  let tex = r"\documentclass{article}
\usepackage{chemformula}
\usepackage{chemmacros}
\begin{document}
\ch{CrO4^2-} \ox{+1,Na} \NMR{1,H} \pH
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<Math"), "{xml}");
  assert!(xml.contains("NMR"), "{xml}");
}

/// datetime2 raw-loads (the stub left `\DTMsetup`/`\DTMdate` undefined —
/// cnltx/chemformula manuals) once `\pdfcreationdate` is pdfTeX's
/// `D:YYYYMMDD…` stamp (pdfTeX manual §8.11; datetime2.sty:46-48).
#[test]
fn datetime2_raw_dates_render() {
  let tex = r"\documentclass{article}
\usepackage[en-GB]{datetime2}
\begin{document}
\DTMsetup{datesep=/}[\DTMdate{2026-09-02}][\DTMdisplaydate{2020}{3}{7}{-1}][\DTMsetdatestyle{iso}\DTMdate{2026-09-02}]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[2nd September 2026][7th March 2020][2026-09-02]"),
    "{xml}"
  );
}

/// OXIDIZED_DESIGN #179: a chapterless class leaves `\chapter` undefined
/// (latex.ltx/article define none), so `\@ifundefined{chapter}` takes the
/// article branch — blindtext.sty:243 `\blinddocument` under scrartcl
/// (hvfloat ×50, coseoul, xassoccnt: `undefined:\thechapter`). A class
/// with a chapter counter keeps it.
#[test]
fn chapter_is_undefined_in_a_chapterless_class() {
  let tex = r"\documentclass{article}
\begin{document}
\makeatletter[\@ifundefined{chapter}{NOCHAP}{CHAP}]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[NOCHAP]"), "{xml}");
  let tex = r"\documentclass{report}
\begin{document}
\makeatletter[\@ifundefined{chapter}{NOCHAP}{CHAP}]
\chapter{One}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[CHAP]") && xml.contains("<chapter"), "{xml}");
  let tex = r"\documentclass{scrartcl}
\usepackage{blindtext}
\begin{document}
\blinddocument
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<section"), "{xml}");
}

/// OXIDIZED_DESIGN #180 (P38): a raw `\list` (latex.ltx:15848 verbatim, as
/// memoir.cls:4580 redefines it) ends in `\@trivlist`, which now opens the
/// list our `\endlist` closes — memoir `adjustwidth` (digiconfigs, memman)
/// and hand-rolled `\@trivlist`…`\endtrivlist` pairs (0802.2207
/// `mathtrivlist`) both nest cleanly.
#[test]
fn raw_list_opens_through_trivlist() {
  let tex = r"\documentclass{article}
\makeatletter
\renewcommand*{\list}[2]{\ifnum\@listdepth>5\relax\@toodeep\else\global\advance\@listdepth\@ne\fi
  \rightmargin\z@\listparindent\z@\itemindent\z@
  \csname @list\romannumeral\the\@listdepth\endcsname\def\@itemlabel{#1}\let\makelabel\@mklab
  \@nmbrlistfalse#2\@trivlist\parskip\parsep\parindent\listparindent\advance\linewidth-\rightmargin
  \advance\linewidth-\leftmargin\advance\@totalleftmargin\leftmargin\parshape\@ne\@totalleftmargin\linewidth\ignorespaces}
\newenvironment{adjw}[2]{\begin{list}{}{\topsep\z@}\item[]}{\end{list}}
\newenvironment{mtl}{\@trivlist\item[]}{\endtrivlist}
\makeatother
\begin{document}
\begin{adjw}{1em}{0pt}Inside A\end{adjw}
\begin{mtl}Inside B\end{mtl}
\begin{enumerate}\item one\end{enumerate}
After
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<itemize").count(), 2, "{xml}");
  assert!(
    xml.contains("Inside A") && xml.contains("Inside B") && xml.contains("<enumerate"),
    "{xml}"
  );
}

/// tcolorbox.sty:2339 `\tcb@proc@options@init` processes a listing env's
/// `[init]` (`auto counter`, `number within`), so a later
/// `\newtcolorbox[use counter from=<env>]` finds `\tcb@cnt@<env>`
/// (tcolorbox manual preamble D: `texexptitledspec` from `texexptitled`).
#[test]
fn tcblisting_init_counter_is_shared_by_use_counter_from() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}\tcbuselibrary{listings}
\tcbset{example/.style 2 args={title={Example \thetcbcounter: #1},label={#2}}}
\newtcblisting[auto counter,number within=section]{texexptitled}[3][]{example={#2}{#3},#1}
\newtcolorbox[use counter from=texexptitled]{texexptitledspec}[3][]{example={#2}{#3},#1}
\begin{document}
\section{S}
\begin{texexptitled}{T1}{l1}
x
\end{texexptitled}
\begin{texexptitledspec}{T2}{l2}y\end{texexptitledspec}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The listing box renders no title (presentation-only in the native env);
  // the shared counter stepped once for it, so the tcolorbox is 1.2.
  assert!(xml.contains("Example 1.2"), "{xml}");
}

/// nicematrix.sty:1953/3665: `NiceArray` takes `[opts]{cols}[opts]`; the
/// leading optional was read as the preamble (nicematrix.tex:409 →
/// `Unrecognized tabular template "["`, 57 extra `&`, `Until:\Body` EOF).
#[test]
fn nicearray_takes_a_leading_option_list() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
$\begin{NiceArray}[t]{lcc}[no-cell-nodes]
n & 0 & 1 \\
u & 2 & 3 \\
\end{NiceArray}$
$\begin{pNiceArray}{cc}[first-col] a & b \\ \end{pNiceArray}$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Unrecognized tabular template"),
    "{stderr}"
  );
  assert_eq!(xml.matches("<XMArray").count(), 2, "{xml}");
}

/// OXIDIZED_DESIGN #181: a `\\` inside a brace group of a cell is an
/// in-cell break, not a row end (latex.ltx:16583 `{\ifnum0=`}\fi` keeps
/// `\cr` from firing at align_state≠0; tabularray makes it a line break —
/// ProfSio.sty:2917 `\SetCell{l}{… \\ …}`). Ending the row misread the
/// cell's `}` as the alignment's `\egroup` (3 errors per cell; Perl
/// truncates the table). An empty cell before `\\` still ends the row.
#[test]
fn newline_inside_a_cell_group_is_an_in_cell_break() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{colspec={XQ[3cm]},hlines}
\SetCell[c=2]{c}{S} & \\
NOM : X & \SetCell{l}{A\\B} \\
{Y \\} & C \\
\end{tblr}
\begin{tabular}{ll}
S & \\
{Y \\} & C \\
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("<tr>").count() + xml.matches("<tr ").count(),
    5,
    "{xml}"
  );
  assert!(xml.contains("A<break/>B"), "{xml}");
  assert!(
    xml.contains("Y <break/>") || xml.contains("Y<break/>"),
    "{xml}"
  );
}

/// latex.ltx:14060/14131: a `\newcommand` optional default passes through
/// two `\def` bodies, so `[########1]` reaches the macro as `##1`
/// (pdflatex-probed). etoolbox/biditools `\patchcmd` builds on it;
/// biditools' load errored `misdefined:#` (crbox, lineno, multiple-choice …).
#[test]
fn newcommand_default_halves_param_tokens_twice() {
  let tex = r"\documentclass{article}
\newcommand{\foo}[2][########1]{[\detokenize{#1}|#2]}
\begin{document}
\foo{A} \foo[x]{B}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // OT1 renders `|` as an em-dash.
  assert!(xml.contains("[####1—A] [x—B]"), "{xml}");
  let tex = r"\documentclass{article}
\usepackage{biditools}
\begin{document}
x
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// A binding-loaded package exposes the installed file's
/// `\ProvidesPackage` version in `\ver@<name>.sty` (setspace-doc.tex:60-64
/// splits it at spaces with `\def\pkginfo#1 #2 #3\relax`; a space-free
/// `\fmtversion` ran to EOF).
#[test]
fn binding_ver_macro_carries_the_installed_provides_version() {
  let tex = r"\documentclass{article}
\usepackage{setspace}
\makeatletter
\def\pkginfo#1 #2 #3\relax{\def\filedate{#1}\def\fileversion{#2}}
\expandafter\expandafter\expandafter\pkginfo\csname ver@setspace.sty\endcsname\relax
\makeatother
\begin{document}
[\filedate][\fileversion]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[20") && xml.contains("][v"), "{xml}");
}

/// latex.ltx:16187 `\secdef#1#2` = `\@ifstar{#2}{\@dblarg{#1}}` (Perl drops
/// the `\@dblarg`; memoir.cls:2787 `\book` ran to EOF, srbook-mem ×3).
#[test]
fn secdef_doubles_the_title_for_the_unstarred_form() {
  let tex = r"\documentclass{article}
\makeatletter
\long\def\@bk[#1]#2{[BK:#1|#2]}
\def\@sbk#1{[SBK:#1]}
\newcommand*{\bk}{\secdef\@bk\@sbk}
\makeatother
\begin{document}
\bk{Ovo} \bk[short]{Long} \bk*{Star}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[BK:Ovo—Ovo] [BK:short—Long] [SBK:Star]"),
    "{xml}"
  );
}

/// Real microtype.sty:80 defines only the `\microtypecontext{…}`
/// declaration — no environment, so `\endmicrotypecontext` is undefined
/// and synthslant.sty:302's `\ifcsdef{endmicrotypecontext}` takes the
/// false branch (a live env-end errored "Attempt to end mode", ×101 in
/// synthslant-gauge). The env form still works through `\begin`/`\end`.
#[test]
fn microtypecontext_is_a_declaration_not_an_environment() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{microtype}
\NewDocumentEnvironment{slantenv}{}
  {\ifcsdef{microtypecontext}{\microtypecontext{tracking=x}}{}}
  {\ifcsdef{endmicrotypecontext}{\endmicrotypecontext}{}}
\begin{document}
\begin{slantenv}Hello\end{slantenv}
\begin{microtypecontext}{tracking=y}World\end{microtypecontext}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hello") && xml.contains("World"), "{xml}");
}

/// beamerbaseoptions.sty:34-38: theme options are keyval, and the themes'
/// internals come from `\ExecuteOptionsBeamer` defaults
/// (beamerouterthemesidebar.sty:30-32 `\beamer@sidebarside`,
/// beamerinnerthemerounded.sty:11-12 `\beamer@themerounded@shadow`);
/// beamerbaseframe.sty:730 creates the `framenumber` counter
/// (appendixnumberbeamer.sty:43).
#[test]
fn beamer_theme_option_defaults_define_their_internals() {
  for theme in ["Berkeley", "Madrid"] {
    let tex = format!(
      "\\documentclass{{beamer}}\n\\usetheme{{{theme}}}\n\\begin{{document}}\n\\begin{{frame}}{{Title}}Hello \\theframenumber\\end{{frame}}\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert(&tex, false);
    assert_eq!(error_count(&stderr), 0, "{theme}: {stderr}");
    assert!(xml.contains("Hello"), "{xml}");
  }
}

/// beamerthemeVerona.sty:174-190 uses `\addtobeamertemplate{background}{...}{}`
/// and `\newcommand<>{\sidegraphics}[3][]{...}` with an optional default argument.
/// `\addtobeamertemplate` executes at frame start and `\newcommand<>` / `\newenvironment<>`
/// pack and remap overlay and optional-default arguments so the frame closes cleanly.
#[test]
fn beamer_frame_sidebar_overlay_template() {
  let tex = r"\documentclass{beamer}
\usetheme[sidebar]{Verona}
\begin{document}
\begin{frame}\sidegraphics<1>{plato}{scale=1.1}\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<subsection"), "{xml}");
}

/// numprint.sty:779 `\DeclareRobustCommand*\numprint`: a `\the\toks255`
/// register-number lookahead (tex.web §440-448) stops at `\protect`
/// instead of pre-expanding the `\ifmmode` dispatch into the stored list
/// (calctab.sty:334-335; calctab manual: 94 "Extra \or already saw \else").
#[test]
fn numprint_is_robust_under_a_the_toks_lookahead() {
  let tex = r"\documentclass{article}
\usepackage{numprint}
\begin{document}
\toks0={}\edef\r{\noexpand\numprint{12500.90}}
\toks0=\expandafter\expandafter\expandafter{\expandafter\the\expandafter\toks0\r}
[\the\toks0]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("ltx_number").count(), 1, "{xml}");
}

/// showexpl.sty:66-86 load-time state survives a document that rebuilds
/// `LTXexample` from the internals (lshort-german l2kurz.tex:73-100 —
/// `\def\SX@codefile{\SX@codefile}` "expands into itself" ×96).
#[test]
fn showexpl_internals_exist_for_rebuilt_ltxexample() {
  let tex = r"\documentclass{article}
\usepackage{showexpl}
\makeatletter
\begingroup
\edef\x{\endgroup\def\noexpand\SX@codefile{\SX@codefile}}
\x
\begin{document}
Codefile:[\SX@codefile]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(".tmp]"), "{xml}");
}

/// newverbs.sty:52-69: `\newverbcommand{\cverb}{before}{after}` wraps a
/// verbatim argument; the real command's extra `\bgroup` is closed by
/// `\verb@egroup`, which a native `\verb` never runs (homework.cls demos:
/// "Attempt to end mode internal_vertical" at the next `\end{…}`).
#[test]
fn newverbcommand_wraps_the_verb_body() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{newverbs}
\newverbcommand{\cverb}{\color{red}}{}
\begin{document}
\begin{quote}
Use \cverb|\qedhere| here and \qverb|x|.
\end{quote}
done
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<quote"), "{xml}");
  assert!(xml.contains(r"\qedhere") && xml.contains("done"), "{xml}");
  assert!(xml.contains("color=\"#FF0000\""), "{xml}");
}

/// Bare `\flushleft`…`\endflushleft` (comment.tex:12-18 `noverb`,
/// bidicode.sty:195 `BDef`): the declaration opens no frame, so its
/// `\end…` partner is a no-op; `\begin{flushleft}` still aligns.
#[test]
fn bare_endflushleft_is_a_noop() {
  let tex = r"\documentclass{article}
\newenvironment*{noverb}{\flushleft}{\endflushleft}
\begin{document}
Text.
\begin{noverb}
content
\end{noverb}
\begin{flushleft}left\end{flushleft}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("content") && xml.contains(r#"<p align="left">left</p>"#),
    "{xml}"
  );
}

/// listings.sty:1968 `\lst@InlineG`: a `{`-delimited `\lstinline` ends at
/// the balanced `}` (coolfn `\mintinline{latex}{\renewcommand{\fnindent}{1.25em}}`).
#[test]
fn lstinline_brace_delimiter_is_balanced() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\lstinline{\renewcommand{\fnindent}{1.25em}}. Then \lstinline|a{b|.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The inline listing is token-marked-up; the group and the unit survive.
  assert!(xml.contains("fnindent</text>}{1.25"), "{xml}");
  assert!(xml.contains("a</text>{<text"), "{xml}");
}

/// `\centering`/`\raggedright` are macros (latex.ltx:16419-16433): expl3's
/// V-expansion register test (expl3-code.tex:2507-2517) must not `\the` a
/// `\let\raggedsignature=\centering` (DIN.lco:130; scrlttr2.cls:5095
/// `\closing`: KOMA letters ×5).
#[test]
fn centering_is_expandable_for_expl3_v_expansion() {
  let tex = r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\let\raggedsignature=\centering
\tl_if_in:nVTF { \raggedright\LaTeXraggedright } \raggedsignature
  { \def\got{L} } { \def\got{NOTL} }
\ExplSyntaxOff
\begin{document}
[\got]\begin{center}c\end{center}{\raggedleft r\par}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[NOTL]"), "{xml}");
  assert!(
    xml.contains(r#"align="center""#) && xml.contains("ltx_align_right"),
    "{xml}"
  );
}

/// OXIDIZED_DESIGN #182 → batch 56gs: a `\caption` with `\@captype` set inside
/// an `lrbox` minipage (tufte-common.def:1110-1133 `marginfigure`; pgfornament
/// ornaments ×40, memman) has no float ancestor. It was degraded to inline
/// `ltx_caption` text (unnumbered, its `\label` a dangling target); it is now
/// the float of its type, placed where the box admits one, numbered and
/// labelled as in LaTeX ("Figure 1:"). A real `figure` keeps its own float.
#[test]
fn caption_outside_a_float_becomes_its_float() {
  let tex = r"\documentclass{article}
\makeatletter
\newsavebox\mybox
\newenvironment{marginfig}{\begin{lrbox}{\mybox}\begin{minipage}{3cm}\def\@captype{figure}}{\end{minipage}\end{lrbox}\marginpar{\usebox{\mybox}}}
\makeatother
\begin{document}
\begin{marginfig}
X
\caption{A caption}\label{fig:m}
\end{marginfig}
See \ref{fig:m}.
\begin{figure}\centering Y\caption{Real float}\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let flat = gaps.replace_all(&xml, "><").into_owned();
  assert!(
    flat.contains(concat!(
      r#"<p>X</p></para><figure inlist="lof" labels="LABEL:fig:m" xml:id="S0.F1"><tags>"#,
      r#"<tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags>"#,
      r#"<caption><tag close=": ">Figure 1</tag>A caption</caption></figure>"#
    )),
    "{flat}"
  );
  assert!(!xml.contains("ltx_caption"), "{xml}");
  assert!(
    xml.contains(
      r#"<caption class="ltx_centering"><tag close=": ">Figure 2</tag>Real float</caption>"#
    ),
    "{xml}"
  );
  assert!(xml.contains("<toccaption"), "{xml}");
}

/// KNOWN_PERL_ERRORS #140: `\index{packages!#1@\texttt{#1}}` with `#1` =
/// `\TIKZ` (pgfornament usefulcommands.tex:93) must re-read as `\TIKZ` + `@`
/// (the `.idx` `\write` form), not the undefined `\TIKZ@`.
#[test]
fn index_control_word_before_at_is_not_glued() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}
\makeindex
\newcommand*{\TIKZ}{Ti\emph{k}Z}
\newcommand{\docpkg}[1]{\texttt{#1}\index{#1 package@\texttt{#1} package}\index{packages!#1@\texttt{#1}}}
\begin{document}
Uses \docpkg{\TIKZ} here.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<indexphrase key="packages">packages</indexphrase>"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<indexphrase key="Ti\emph{k}Z">"#), "{xml}");
}

/// KNOWN_PERL_ERRORS #141: `\renewcommand\part{\secdef\@part\@spart}` and a
/// document-made `\chapter` (source3body.tex:96-123: l3kernel interface3 +
/// source3, 2 → 101 errors) find the class-level workers and an unlocked
/// `\chapter` in a chapterless class.
#[test]
fn secdef_part_and_chapter_workers_exist() {
  let tex = r"\documentclass{article}
\makeatletter
\renewcommand\part{\par\secdef\@part\@spart}
\newcounter{chapter}
\renewcommand\thesection{\thechapter.\@arabic\c@section}
\newcommand\chapter{\clearpage\secdef\@chapter\@schapter}
\makeatother
\begin{document}
\part{First part}
\chapter{A chapter}
\section{A section}
\chapter*{Unnumbered}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<part inlist="toc" xml:id="Pt1">"#), "{xml}");
  assert!(
    xml.contains(r#"<chapter inlist="toc" xml:id="chapter1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<tag close=" ">1.1</tag>A section"#),
    "{xml}"
  );
  assert!(xml.contains("<title>Unnumbered</title>"), "{xml}");
}

/// The German `"` shorthands belong to babel's German, not only to
/// `\usepackage{german}`: `\usepackage[ngerman]{babel}` (80 TL manuals)
/// rendered `Sch"one` as `Sch”one`, and `\mdqon` errored `T_ACTIVE["]`.
/// Non-shorthand follow-characters print the quote itself
/// (pdflatex `A "x" B "1"` → `A "x" B "1"`), `"ck`/`"ff` the letter.
#[test]
fn babel_ngerman_umlaut_shorthands() {
  let tex = r#"\documentclass{article}
\usepackage[ngerman]{babel}
\begin{document}
Sch"one Gr"u"se "`Zitat"' A "x" B "1" C "ck D "ff E {\mdqoff "y"} \mdqon "a
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Schöne Grüße „Zitat“ A \"x\" B \"1\" C ck D ff E ”y” ä"),
    "{xml}"
  );
}

/// german.sty's `\germanTeX` (german.sty:666-671) — run by the kernel first
/// aid `file/german.sty/after` (latex2e-first-aid-for-external-files.ltx:160)
/// and by documents written for german.sty (a0poster a0/a0_eng, adrconv,
/// akletter … 16 TL manuals with `\ngermanTeX`).
#[test]
fn german_sty_germantex_switch_is_defined() {
  let tex = r#"\documentclass{article}
\usepackage{german}
\begin{document}
Sch"one \germanTeX Gr"u"se
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Schöne Grüße"), "{xml}");
  let tex = tex
    .replace("{german}", "{ngerman}")
    .replace(r"\germanTeX", r"\ngermanTeX");
  let (stderr, xml) = convert(&tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Schöne Grüße"), "{xml}");
}

/// beamer builds on article (Perl beamer.cls.ltxml:1361 `LoadClass`); the
/// binding's `RequirePackage!("article")` missed silently, leaving
/// `\subsection` with `undefined:\thesubsection` (bfh-ci DEMO-BFHBeamer,
/// metropolis/gotham demos).
#[test]
fn beamer_has_article_sectioning_counters() {
  let tex = r"\documentclass{beamer}
\begin{document}
\section{Introduction}
\begin{frame}{A}x\end{frame}
\subsection{Sub}
\begin{frame}{B}y\end{frame}
\section{Second}
\subsection{Sub two}
\begin{frame}{C}z\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<subsection inlist="toc" xml:id="S1.SS1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<subsection inlist="toc" xml:id="S2.SS1">"#),
    "{xml}"
  );
}

/// A `\bibliography` inside a beamer frame (metropolis demo, simpleplus /
/// simpledarkblue / pure-minimalistic samples): the frame's `ltx:subsection`
/// never auto-closes, so placing the bibliography "as an `ltx:section`" erred
/// `<ltx:section> isn't allowed in <ltx:p>` and left it inside the `<p>`.
/// The subsection may hold an `ltx:bibliography`, which is where beamer
/// typesets it (`backmatter_insertion_target`).
#[test]
fn bibliography_inside_a_beamer_frame_stays_in_the_frame() {
  let tex = r"\documentclass{beamer}
\begin{document}
\section{Intro}
\begin{frame}{A}x\end{frame}
\begin{frame}{References}
  \bibliography{nonexistent}
  \bibliographystyle{abbrv}
\end{frame}
\begin{frame}{After}z\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("</p>\n      </para>\n      <bibliography"),
    "{xml}"
  );
  assert!(xml.contains("</bibliography>\n    </subsection>"), "{xml}");
}

/// nameref.sty:189-192 `\NR@gettitle` (memoir.cls:7025 routes `\M@gettitle`
/// — heads, `\PoemTitle` — through it; srbook-mem Test/TestLight/
/// SerbianBookMem, serbian-apostrophe ×2: sole error).
#[test]
fn nameref_gettitle_records_the_title() {
  let tex = r"\documentclass{article}
\usepackage{nameref}
\begin{document}
\makeatletter
\NR@gettitle{Guarded Title}[\@currentlabelname]
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[Guarded Title]"), "{xml}");
}

/// fancyhdr.sty:577-608 `\f@nch@initialise` — executed by ctex's
/// end-of-package hook (ctex-heading-article.def:686; inkpaper, sduthesis,
/// shtthesis, caspervector) after patching it.
#[test]
fn fancyhdr_initialise_is_defined() {
  let tex = r"\documentclass{article}
\usepackage{fancyhdr}
\pagestyle{fancy}
\makeatletter
\f@nch@initialise
\makeatother
\begin{document}
\section{One}
x
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>x</p>"), "{xml}");
}

/// biblatex.sty:16439-16440 loads the bbx before the cbx (oxref.bbx:489
/// `\newtoggle` vs oxnum.cbx:26 `\providetoggle`), and the raw style chain's
/// declaration-only commands (`\DeclareDataInheritance`, `\NumCheckSetup`,
/// `\defbibfilter`, `\defbibnote` …) are accepted (biblatex-oxref ×4,
/// biblatex-cse-doc, biblatex-musuos).
#[test]
fn biblatex_loads_bbx_before_cbx() {
  let tex = r"\documentclass{article}
\usepackage[style=oxnum]{biblatex}
\defbibfilter{books}{type=book}
\defbibnote{pre}{A note.}
\begin{document}
Hello.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Hello.</p>"), "{xml}");
}

/// titlesec.sty:112-165 `\titleclass` defines a NEW heading command
/// (regulatory.sty:116/121 `\article`/`\para`; regulatory example1/2 ×4).
#[test]
fn titleclass_defines_a_new_heading_command() {
  let tex = r"\documentclass{article}
\usepackage{titlesec}
\newcounter{article}
\titleclass{\article}[0]{straight}
\newcounter{para}
\titleclass{\para}{straight}[\article]
\begin{document}
\article{Hello}
\para{World}
Text.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<section"), "{xml}");
  assert!(xml.contains("Hello</title>"), "{xml}");
  assert!(xml.contains("<subsection"), "{xml}");
  assert!(xml.contains("World</title>"), "{xml}");
}

/// caption3.sty:1753 `\DeclareCaptionType` lazy-loads newfloat and delegates,
/// and newfloat.sty:117-125 reads the trailing `[singular][listname]`
/// (pygmentex.sty:23; pygmentex ×2, hvpygmentex).
#[test]
fn declare_caption_type_makes_a_float() {
  let tex = r"\documentclass{article}
\usepackage{caption}
\DeclareCaptionType{pygcode}[Listagem][Lista de listagens]
\begin{document}
\begin{pygcode}code\caption{A code listing}\end{pygcode}
[\pygcodename/\listpygcodename]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<float class="ltx_float_pygcode""#), "{xml}");
  assert!(xml.contains("[Listagem/Lista de listagens]"), "{xml}");
  assert!(!xml.contains("[Listagem][Lista"), "{xml}");
}

/// KNOWN_PERL_ERRORS #142: `\valign{…}` consumes its alignment
/// (fancyvrb.sty:570 `\FancyVerbTab`: one `#`-reaches-stomach error per
/// tab-bearing `Verbatim` line under `showtabs`, pygmentex_demo).
#[test]
fn valign_swallows_its_alignment() {
  let tex = "\\documentclass{article}
\\usepackage{fancyvrb}
\\begin{document}
\\begin{Verbatim}[showtabs,tabsize=1]
A\tB
\\end{Verbatim}
\\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"class="ltx_verbatim""#) && xml.contains("A<text"),
    "{xml}"
  );
}

/// The `@`-is-a-letter sibling of KNOWN_PERL_ERRORS #140 (pgfmanual-en-macros
/// .tex:281 `\index{Internals!\strippedat @…}` under `\makeatletter`:
/// tikz-cd-doc, tikz-dependency-doc, pdfmarginpar): the print_cs space must
/// not depend on `@`'s catcode.
#[test]
fn index_control_word_before_letter_at_is_not_glued() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}
\makeindex
\makeatletter
\def\strippedat{foo}
\def\extractinternalcommand{\index{Internals!\strippedat @\protect\texttt{\strippedat}}}
\makeatother
\begin{document}
\extractinternalcommand Text.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<indexphrase key="foo">"#), "{xml}");
}

/// latex.ltx:9512/9537: `\document` is preamble-only and its hooks one-time —
/// a second `\begin{document}` (ltnews.tex:236/296, l3news.tex:109/177
/// `\renewenvironment{document}` + per-issue `\input`) re-fired csquotes'
/// end-preamble block whose hooks are `\undef`ed after use (csquotes.sty:2434-2446).
#[test]
fn second_begin_document_fires_no_hooks() {
  let tex = r"\documentclass{article}
\usepackage{csquotes}
\usepackage{hyperref}
\begin{document}
Hello \enquote{world}.
\begin{document}
Second begin.
\end{document}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Second begin."), "{xml}");
  assert_eq!(xml.matches("<document ").count(), 1, "{xml}");
}

/// pdfcomment annotations become `ltx:note`s (pdfcomment example ×3: raw
/// pdfcomment.sty took the dvips `\pdfmark` branch and dumped PDF
/// dictionaries into the text); `\pdfstringdef` is global (hyperref.sty:386).
#[test]
fn pdfcomment_annotations_are_notes() {
  let tex = r"\documentclass{article}
\usepackage[author={Me}]{pdfcomment}
\begin{document}
A\pdfcomment[color=red,subject={S},deadline={2009/11/11}]{Hello comment.} B
\pdftooltip{visible}{tip text} $x\pdftooltip{y}{math tip}$
\pdfmarkupcomment[markup=Highlight]{marked}{note}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<note role="pdfcomment">Hello comment.</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"visible<note role="tooltip">tip text</note>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"marked<note role="pdfmarkupcomment">note</note>"#),
    "{xml}"
  );
  assert!(!xml.contains("pdfmark="), "{xml}");
}

/// memoir.cls:2640-2672 patches `\title`/`\author` to set `\thetitle`/
/// `\theauthor` (biblatex-oxref docs typeset them on their own title page).
#[test]
fn memoir_title_defines_thetitle() {
  let tex = r"\documentclass{memoir}
\title{My Title\thanks{T}}\author{An Author}
\begin{document}
[\thetitle/\theauthor]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[My Title/An Author]"), "{xml}");
}

/// mathtools.sty:1576 `\mathmakebox[<width>]`: the width is a <dimen>
/// (`\widthof{$x$}` measured), not content (optidef `\bodySubjectTo` in
/// `align*`, 58 errors).
#[test]
fn mathmakebox_width_is_measured_not_typeset() {
  let tex = r"\documentclass{article}
\usepackage{amsmath,mathtools,calc}
\begin{document}
\begin{align*}
a &= \mathmakebox[\widthof{$x$}][c]{y} b \\
c &= \mathmakebox[2em]{d} \mathmakebox[][c]{e}
\end{align*}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(">y<") && xml.contains(">d<") && xml.contains(">e<"),
    "{xml}"
  );
}

/// scrlfile.sty raw: `\BeforePackage`/`\AfterPackage` are the kernel file
/// hooks (scrlfile-hook.sty:85-230). scrbook.cls:5466-5477 pairs them to
/// save/restore `\@addchap` around hyperref — the absorbed "before" left
/// `\addchap` undefined after `\usepackage{hyperref}` (cleanthesis, bfh-ci).
#[test]
fn scrlfile_before_and_after_package_hooks_fire() {
  let tex = r"\documentclass{article}
\usepackage{scrlfile}
\makeatletter
\BeforePackage{hyperref}{\def\before@ran{yes}}
\AfterPackage{hyperref}{\def\after@ran{yes}}
\AfterPackage*{hyperref}{\def\afterstar@early{yes}}
\makeatother
\usepackage{hyperref}
\makeatletter
\AfterPackage*{hyperref}{\def\afterstar@late{yes}}
\makeatother
\begin{document}
\makeatletter
[\before@ran/\after@ran/\afterstar@early/\afterstar@late]
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[yes/yes/yes/yes]"), "{xml}");
  let tex = r"\documentclass{scrbook}
\usepackage{hyperref}
\begin{document}
\addchap{Declaration}
Text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<title>Declaration</title>"), "{xml}");
}

/// makeidx.sty defines no `\makeindex` (makeidx.sty:44-51); the binding's
/// no-op clobbered the kernel's `\@indexfile` allocation that manyind /
/// robustindex write to (mindsample, robustmanual, multisample).
#[test]
fn makeidx_keeps_the_allocating_makeindex() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}
\makeindex
\begin{document}
\makeatletter
\protected@write\@indexfile{}{payload}%
\ifdefined\@indexfile STREAMDEFINED\fi
\makeatother
\index{alpha}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("STREAMDEFINED"), "{xml}");
  assert!(xml.contains(r#"<indexphrase key="alpha">"#), "{xml}");
  assert!(!xml.contains("payload"), "{xml}");
}

/// PLANS P37: a `{lstlisting}` in a tabbing field / `l` cell is wrapped in
/// an auto-opened `ltx:inline-block` (the `p{}`-column shape) instead of
/// `<ltx:listing> isn't allowed in <ltx:td>` (engtlc ×2, lexref,
/// expex-glossonly).
#[test]
fn listing_in_a_tabular_cell_gets_an_inline_block() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\begin{tabbing}
\hspace{3cm}\=\kill
\begin{lstlisting}
$x$
\end{lstlisting} \> value
\end{tabbing}
\begin{tabular}{ll}
\begin{lstlisting}
code
\end{lstlisting} & right \\
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<inline-block").count(), 2, "{xml}");
  assert!(
    xml.contains("<inline-block>\n            <listing") || xml.contains("<inline-block><listing"),
    "{xml}"
  );
}

/// tex.web §1211: the register reader skips spaces/`\relax` and absorbs
/// `\global` (a0poster.cls.ltxml `\setlength { \paperwidth }{…}`:
/// modernposter; xtab.sty:146 `\setlength{\global\ST@toadd}{#1}`: rec-thy,
/// altverse).
#[test]
fn variable_reader_skips_spaces_and_takes_prefixes() {
  let tex = r"\documentclass{article}
\usepackage{xtab}
\newlength{\mylen}
\begin{document}
\setlength { \mylen }{ 5pt }%
{\setlength{\global\mylen}{7pt}}[\the\mylen]
\begin{xtabular}{l}a\\[6pt]b\\ \end{xtabular}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[7.0pt]"), "{xml}");
  assert_eq!(xml.matches("<tr>").count(), 2, "{xml}");
}

/// beamerbasetitle.sty:148/169/233/238: `\inst{n}` is `\textsuperscript{n}`
/// (detlevcm, beamerstructure2), `\partpage` (beamerbasetitle.sty:30) re-shows
/// the part page, and beamer.cls:32-49 declares the sidebar/margin dimension
/// family themes read (beamerthemeVerona.sty:287).
#[test]
fn beamer_inst_partpage_and_sidebar_dimens() {
  let tex = r"\documentclass{beamer}
\title{T}
\author{Alice\inst{1} \and Bob\inst{2}}
\institute{\inst{1}Univ A \and \inst{2}Univ B}
\makeatletter
\newlength{\myx}
\setlength{\myx}{\dimexpr(\paperwidth-\beamer@rightsidebar-2mm)}
\makeatother
\begin{document}
\begin{frame}\titlepage\end{frame}
\part{Background}
\begin{frame}\partpage\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // `\inst{n}` in an institute SETS the affiliation label and in an author
  // REQUESTS it: Alice links to Univ A, Bob to Univ B (no typeset `<sup>`).
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &[],
    r##"<creator role="author"><personname>Alice</personname><contact name="Affiliation: " role="affiliation">Univ A</contact></creator>"##,
  );
  latexml::util::test::assert_element(
    &xml,
    "creator",
    &["before="],
    r##"<creator before="  " role="author"><personname>Bob</personname><contact name="Affiliation: " role="affiliation">Univ B</contact></creator>"##,
  );
  assert!(xml.contains("<tag>Part I</tag>"), "{xml}");
}

/// tabu.sty:6-8 `\begin{tabu} to <dimen>{cols}`: the `to` prefix and `X`
/// columns (brandeis-problemset example.tex:228, 41 errors).
#[test]
fn tabu_to_width_and_x_columns() {
  let tex = r"\documentclass{article}
\usepackage{tabu}
\begin{document}
\begin{tabu} to 0.25\linewidth{X[1,$]rr}
a & b & c \\
\end{tabu}
\begin{tabu}{lX}
d & e \\
\end{tabu}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 5, "{xml}");
}

/// xkeyval.tex:38 `\let\XKeyValLoaded\endinput`: expex.tex:65 must not
/// re-input raw xkeyval over the binding (fragoli, rainbowbrackets:
/// `undefined:\ep@preambleanchor` on a `\pex` with preamble text).
#[test]
fn xkeyval_sets_the_loaded_sentinel() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("expex.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{expex}
\begin{document}
\pex
This is a preamble.
\a First item.
\b Second item.
\xe
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("This is a preamble."), "{xml}");
  assert!(xml.contains("First item."), "{xml}");
}

/// biblatex.sty:809-870 declares its counters with `\newcounter` (fiwi.bbx:59
/// `\defcounter{lownamepenalty}` → "No counter defined"; biblatex-fiwi ×3);
/// blx-compat.def:155 `\AtBeginShorthands` (philosophy/windycity styles);
/// hyperref.sty:237 `\Hy@AtBeginDocument` (biblatex2bibitem ×2).
#[test]
fn biblatex_counters_hooks_and_hy_atbegindocument() {
  let tex = r"\documentclass{article}
\usepackage[colorlinks]{hyperref}
\usepackage{biblatex}
\AtBeginShorthands{\relax}
\makeatletter
\defcounter{lownamepenalty}{0}
\Hy@AtBeginDocument{\def\@pdfborder{0 0 1}}
\makeatother
\setcounter{lownamepenalty}{5}
\begin{document}
[\arabic{lownamepenalty}/\arabic{maxnames}] \href{http://x}{y}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[5/3]"), "{xml}");
  assert!(xml.contains(r#"href="http://x""#), "{xml}");
}

/// KNOWN_PERL_ERRORS #145: the frontmatter copy of `\title`/`\author`/`\date`
/// comes from the stored (once-halved) macro — the RCS-keyword idiom
/// `\date{\def\$##1: ##2 ##3${##2}…}` (ulineno.tex:16) put a literal `#`
/// in the stomach.
#[test]
fn frontmatter_copies_the_halved_macro() {
  let tex = r"\documentclass{article}
\date{\def\$##1: ##2 ##3${##2}%$
   Version \$Revision: 3.1 $, \$Date: 2001/08/03 03:29:19 $
}
\title{T\def\x##1{##1}\x{ok}}\author{A \and B\thanks{t}}
\begin{document}
\maketitle
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Version 3.1, 2001/08/03"), "{xml}");
  assert!(xml.contains("<title>Tok</title>"), "{xml}");
  assert_eq!(xml.matches("<personname>").count(), 2, "{xml}");
}

/// pgfplotscore.code.tex:74-89 `\pgfplotsenablelua{0}`: under the `[luatex]`
/// profile `\directlua` exists but pgfplots' Lua bootstrap cannot run here
/// (colorblind_doc: `\pgfplotsglobalretval`, `\pgfplotsutil@savecatcodetable`).
#[test]
fn pgfplots_lua_backend_is_off_under_luatex_profile() {
  let tex = r"\documentclass{article}
\usepackage{pgfplots}
\pgfplotsset{compat=1.18}
\begin{document}
\begin{tikzpicture}\begin{axis}\addplot {x^2};\end{axis}\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg"), "{xml}");
}

/// url.sty:84: `\` is literal inside `\url`/`\path` (latex4wp.tex:451
/// `\path{C:\localtexmf\tex\}` swallowed the rest of the manual).
#[test]
fn url_backslash_is_literal() {
  let tex = r"\documentclass{article}
\usepackage{url}
\begin{document}
See the path \path{C:\localtexmf\tex\} here. \url{http://x/a_b#c\}
More text after it.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r"C:\localtexmf\tex\</text>") || xml.contains(r"C:\localtexmf\tex\<"),
    "{xml}"
  );
  assert!(xml.contains("More text after it."), "{xml}");
}

/// `\index{foo@\string\verb\string"bar}` (amsldoc.cls `\cs`; amsldoc-it/-vn):
/// a `\verb` "delimited" by a control sequence is index text, not verbatim.
#[test]
fn index_verb_followed_by_cs_is_text() {
  let tex = r#"\documentclass{article}
\usepackage{makeidx}
\makeindex
\begin{document}
Text\index{foo@\string\verb\string"bar}. More text here.
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<indexmark"), "{xml}");
  assert!(xml.contains("More text here."), "{xml}");
}

/// physics2 is its own package, not "physics v2": the glued-suffix fallback
/// loaded the physics binding (`undefined:\usephysicsmodule`, every
/// `\ab`/`\bra`/`\ket`; physics2 manuals, whatsnote). Registered
/// INTERPRETABLE, it raw-loads even without `--includestyles`.
#[test]
fn physics2_is_not_a_version_of_physics() {
  let tex = r"\documentclass{article}
\usepackage{physics2}
\usephysicsmodule{ab,braket}
\begin{document}
\[ \ab(x) \quad \bra{\psi}\ket{\phi} \braket{\psi}{\phi} \]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"name="rangle""#), "{xml}");
  assert!(xml.contains(r#"role="MIDDLE""#), "{xml}");
  let (stderr, _) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// beamerbaseframe.sty:91 sets `\ifbeamer@inframe` inside a frame (the BFH
/// theme's `\sectionpage` otherwise nests a `\frame[plain]`: DEMO-BFHBeamer
/// ×2), and beamerbasesection.sty:45-93's lecture layer captures the
/// `\AtBeginLecture` body instead of running it (beamerthemeVerona.sty:354).
#[test]
fn beamer_inframe_flag_and_lecture_layer() {
  let tex = r"\documentclass{beamer}
\makeatletter
\def\sectionpage{\ifbeamer@inframe\else\frame{X}\fi}
\AtBeginLecture{\begin{frame}[plain]\thelecture.\quad \insertlecture\end{frame}}
\makeatother
\begin{document}
\section{S}
\frame{\sectionpage}
\begin{frame}{T}x\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<subsection").count(), 2, "{xml}");
  assert!(!xml.contains("plain"), "{xml}");
}

/// `\maketitle` inside a box capture (ltx-talk.cls:515 frames, unifront,
/// `\parbox{…}{\maketitle}`): the flush goes to the document head (56gf,
/// OXIDIZED_DESIGN #262) as real `<title>`/`<creator>` elements — never
/// `<ltx:title> isn't allowed in <ltx:_CaptureBlock_>` — and the box keeps the
/// rest of its own content. (Before 56gf it degraded to `ltx:text` in the box,
/// leaking `\lx@personname{Alice}` as text.)
#[test]
fn maketitle_inside_a_box_goes_to_the_head() {
  let tex = r"\documentclass{article}
\title[Short]{My Title}
\author{Alice}
\begin{document}
\parbox{\textwidth}{\maketitle}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("ltx_title"), "{xml}");
  let title = xml.find("<title>My Title</title>").expect("a real title");
  assert!(!xml[..title].contains("<para"), "the title leads:\n{xml}");
  assert!(xml.contains("<personname>Alice</personname>"), "{xml}");
  assert!(!xml.contains("lx@personname"), "{xml}");
  // Content around `\maketitle` in the box stays in the box, in order.
  let tex = r"\documentclass{article}
\title{My Title}
\author{Alice}
\begin{document}
Before.
\parbox{5cm}{Inside before. \maketitle Inside after.}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(concat!(
      r#"<inline-block class="ltx_parbox" vattach="middle" width="142.3pt">"#,
      "\n        <p>Inside before. Inside after.</p>\n      </inline-block>"
    )),
    "{xml}"
  );
}

/// keyval.sty reads each option as a delimited argument, so a `{…}` inside
/// a KEY is opaque (enumitem shortlabels expanding to a box: verifica.cls
/// `\setlist[test]{\@risp,leftmargin=*}`, 3 mode errors × 5 docs).
#[test]
fn keyval_key_is_brace_aware() {
  let tex = r"\documentclass{article}
\usepackage[shortlabels,inline]{enumitem}
\makeatletter
\newcommand{\labelbox}[1]{\fbox{\parbox[][.2cm][c]{.2cm}{#1}}}
\def\@risp{\labelbox{\alph*}}
\newlist{test}{enumerate}{1}
\setlist[test]{\@risp,leftmargin=*}
\setlist[esercizi]{\bfseries 1.,leftmargin=*}
\makeatother
\begin{document}
x
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>x</p>"), "{xml}");
}

/// ngermanb.ldf:123-127 / french.ldf: the babel `\extras<lang>` hooks must
/// exist, or cleveref's `\cref@addto` `\edef`s a self-referential hook that
/// loops at `\begin{document}` (homework-demo-de/-fr, jwjournal-demo-de).
#[test]
fn babel_extras_hooks_are_defined() {
  for lang in ["ngerman", "french"] {
    let tex = format!(
      r#"\documentclass[{lang}]{{article}}
\usepackage[{lang}]{{babel}}
\usepackage{{cleveref}}
\begin{{document}}
\selectlanguage{{{lang}}}
Sch\"one Gr\"u\ss e
\end{{document}}
"#
    );
    let (stderr, xml) = convert(&tex, false);
    assert_eq!(error_count(&stderr), 0, "{lang}: {stderr}");
    assert!(!stderr.contains("expands into itself"), "{lang}: {stderr}");
    assert!(xml.contains("Schöne Grüße"), "{lang}: {xml}");
  }
}

/// xkeyval.tex:497/618 fetch `\XKV@rm` one step: a leftover value may name
/// a macro defined only when its key code finally runs (chessboard.sty:1439
/// `trimarea=\board`, `\board` \edef'd at :1087 — chessboard-skakps).
#[test]
fn setrmkeys_keeps_leftover_values_unexpanded() {
  let tex = r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key[p]{A}{k}{\def\got{#1}}
\setkeys*[p]{B}{k=\m}
\def\m{VAL}
\setrmkeys[p]{A}
\makeatother
\begin{document}
[\got]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[VAL]"), "{xml}");
}

/// amsopn.sty:90 `\operatorfont` (glosmathtools `\sbu`, ~54× per manual).
#[test]
fn amsopn_operatorfont_is_defined() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\newcommand*{\sbu}[1]{_{\operatorfont{#1}}}
\begin{document}
$x\sbu{i}$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<XMApp>") && xml.contains("i</XMTok>"),
    "{xml}"
  );
}

/// fontspec-luatex.sty:3980 `\strong`; under the `luatex` profile
/// nlctuserguide.sty:177 relies on fontspec for it (glossariesbegin,
/// mfirstuc-manual: their only error).
#[test]
fn fontspec_strong_is_bold() {
  let tex = r"\documentclass{article}
\usepackage{fontspec}
\begin{document}
\strong{hi} there
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<text font="bold">hi</text>"#), "{xml}");
}

/// scalerel.sty:56 loads graphicx; :152-186 is the documented low-level
/// API (`\ThisStyle`, `\SavedStyle`, `\@obj`, `\LMex`); `\@obj` re-enters
/// math so a math-mode `\scaleobj` keeps its scripts (scalerel.tex:422-508,
/// hwemoji, stackengine).
#[test]
fn scalerel_low_level_api_and_math_objects() {
  let tex = r"\documentclass{article}
\usepackage{scalerel}
\begin{document}
\scalebox{2}{X}
\(\scaleobj{2}{\sum_{i=0}^{n}}\)
\makeatletter
$\ThisStyle{\hbox{\@obj{\LMex=1ex \SavedStyle x}}}$
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert!(xml.contains(r#"xscale="2.0""#), "{xml}");
  assert!(xml.contains("∑") && xml.contains("SUBSCRIPTOP"), "{xml}");
}

/// cas-common.sty:1560 `{graphicalabstract}` (cas-sc / cas-dc).
#[test]
fn cas_graphicalabstract_is_a_note() {
  let tex = r"\documentclass{cas-sc}
\begin{document}
\begin{graphicalabstract}
Some abstract figure.
\end{graphicalabstract}
Body text.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<note role="graphicalabstract">"#), "{xml}");
}

/// cas-common.sty:895 `\author{O{} m O{}}` — the trailing `[keyvals]` block
/// (cas-sc-sample.tex:58) is frontmatter (`orcid=` → contact), never a
/// `<para>` before the title; four calls give four creators.
#[test]
fn cas_author_trailing_keyvals_are_frontmatter() {
  let tex = "\\documentclass{cas-sc}
\\begin{document}
\\title{T}
               \\author[1,3]{J.K. Krishnan}[type=editor,
 auid=000,bioid=1,
 prefix=Sir,
 role=Researcher,
 orcid=0000-0001-0000-0000]
               \\author[2,4]{Han Thane}[style=chinese]
               \\author[2,3]{William {J. Hansen}}[%
   role=Co-ordinator,
   suffix=Jr,
   ]
               \\author[1,3]{T. Rafeeq}
\\maketitle
Body.
\\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for leak in ["type=editor", "style=chinese", "suffix=Jr"] {
    assert!(!xml.contains(leak), "`{leak}` is not ink:\n{xml}");
  }
  let title = xml.find("<title>T</title>").expect("title");
  assert!(
    !xml[..title].contains("<para"),
    "no paragraph precedes the title:\n{xml}"
  );
  assert_eq!(
    xml.matches("role=\"author\">").count(),
    4,
    "four creators:\n{xml}"
  );
  for name in [
    "J.K. Krishnan",
    "Han Thane",
    "William J. Hansen",
    "T. Rafeeq",
  ] {
    assert!(
      xml.contains(&format!("<personname>{name}</personname>")),
      "{name}:\n{xml}"
    );
  }
  assert!(
    xml.contains("<contact role=\"orcid\">") && xml.contains("0000-0001-0000-0000"),
    "the orcid is a contact:\n{xml}"
  );
  assert_eq!(
    xml.matches("role=\"orcid\"").count(),
    1,
    "one orcid:\n{xml}"
  );
}

/// spanish.ldf:680 `\deactivatetilden` (gaceta.cls:1612).
#[test]
fn babel_spanish_deactivatetilden_is_defined() {
  let tex = r"\documentclass{article}
\usepackage[spanish]{babel}
\makeatletter
\deactivatetilden
\makeatother
\begin{document}
Espa\~nol
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Español"), "{xml}");
}

/// xcolor.sty:168 `\XC@@names`, called by xcolor-patches-tmp-ltx.sty:83
/// under pdfmanagement's `package/xcolor/after` hook (doc-use-newpax).
#[test]
fn xcolor_names_hook_is_defined() {
  let tex = r"\RequirePackage{pdfmanagement}
\documentclass{article}
\usepackage{xcolor}
\begin{document}
\textcolor{red}{hello}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"color="#FF0000""##), "{xml}");
}

/// tabu.sty:1066/1081 `X[1,$]` is a MATH column (brandeis-problemset
/// example.tex:228).
#[test]
fn tabu_math_x_column() {
  let tex = r"\documentclass{article}
\usepackage{tabu}
\begin{document}
\begin{tabu} to 0.5\linewidth{X[1,$]rr}
P_1 & 10 & 3 \\
P_2 & 1 & 1 \\
\end{tabu}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("SUBSCRIPTOP"), "{xml}");
  assert!(xml.contains("<td") && xml.contains(">10</td>"), "{xml}");
}

/// biblatex.sty defines its `\if<test>` commands as BRANCH-SELECTING
/// macros (`\iffieldundef{f}{true}{false}`, :6205), not TeX conditionals;
/// plus the round-3 declarations (`\DeclareLabeltitle`, `\letbibmacro`,
/// `\uspunctuation`, `\footfullcite`).
#[test]
fn biblatex_tests_are_branch_macros() {
  let tex = r"\documentclass{article}
\usepackage{biblatex}
\DeclareLabeltitle{\field{title}}
\DeclareLabelalphaTemplate{\labelelement{\field{label}}}
\letbibmacro{foo}{bar}
\uspunctuation
\begin{document}
[\iffieldundef{title}{U}{D}]
[\ifcitation{C}{N}]
[\ifentrytype{book}{B}{N}]
[\ifuseauthor{A}{N}]
[\ifhyperref{H}{N}]
\stdpunctuation
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[U]\n[N]\n[N]\n[A]\n[H]"), "{xml}");
}

/// biditools.sty:792 `\bidi@ifscanable` rebuilds a macro from its
/// `\meaning`; a native (closure) `\begin`/`\end` must fail that `\ifx`
/// round-trip as in Perl, or the patched `\begin` loses its `\begingroup`
/// (crbox-doc, ghab-doc: "close a group that switched to mode horizontal").
#[test]
fn biditools_env_patch_leaves_begin_end_intact() {
  let tex = r"\documentclass{article}
\usepackage{biditools}
\begin{document}
\begin{tabular}{ll}a & b\\\end{tabular}
\begin{center}x\end{center}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">a</td>") && xml.contains(">b</td>"), "{xml}");
  assert!(xml.contains(r#"<p align="center">x</p>"#), "{xml}");
}

/// enumitem.sty:108 `\enitkv@key` adds a list key (verifica.cls:307);
/// italian.ldf:155/179 `\setISOcompliance`, `\IntelligentComma`.
#[test]
fn enitkv_key_and_babel_italian_extras() {
  let tex = r"\documentclass{article}
\usepackage[italian]{babel}
\usepackage{enumitem}
\makeatletter
\enitkv@key{}{mykey}{\gdef\gotkey{#1}}
\makeatother
\setISOcompliance
\begin{document}
\IntelligentComma
\begin{enumerate}[mykey=7]
\item a
\end{enumerate}
[\gotkey]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[7]"), "{xml}");
}

/// latex.ltx:16061/16072: itemize/enumerate locally reset `\makelabel`, so
/// a document's global 2-argument `\makelabel` (mathfont-user-guide.tex:85)
/// never receives the item labels (Perl errs the same way).
#[test]
fn global_makelabel_does_not_reach_list_items() {
  let tex = r"\documentclass{article}
\usepackage{enumitem}
\makeatletter
\def\makelabel#1#2{\expandafter\gdef\csname fig@#1\endcsname{#2}}
\makeatother
\begin{document}
\begin{itemize}
\item First bullet item.
\item Second item.
\end{itemize}
\begin{enumerate}[label=(\alph*)]
\item a
\end{enumerate}
\makelabel{x}{y}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<item ").count(), 3, "{xml}");
}

/// LuaTeX manual §7.3 `\Udelimiter`/`\Uradical`/`\Umathcodenum` +
/// `\mathnolimitsmode`/`\scantextokens` (mathfont.sty:670,1405,2818-2925;
/// mathfont-symbol-list).
#[test]
fn umath_delimiter_radical_and_codenum_under_luatex_profile() {
  let tex = r"\documentclass{article}
\mathnolimitsmode=4\relax
\begin{document}
$\Umathcharnumdef\myrel=\Umathcodenum`\- \relax$
$\Udelimiter+4+0+123\relax$
$\Uradical+0+8730\relax{x}$
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("{") && xml.contains("<XMApp"), "{xml}");
}

/// beamerbasethemes.sty:25 `\usefonttheme` loads its theme file (the
/// uantwerpen font theme carries `\usetikzlibrary{calc}`), and
/// beamerbasecompatibility.sty:309 `\beamer@ifempty` (graphbox's
/// `\includegraphics`). Witness beamerthemeuantwerpenuserguide.
#[test]
fn beamer_font_theme_loads_and_ifempty_is_defined() {
  let tex = r"\documentclass{beamer}
\usepackage{tikz}
\usepackage{graphbox}
\usefonttheme{serif}
\makeatletter
\begin{document}
\begin{frame}
\beamer@ifempty{}{EMPTY}{FULL}
\includegraphics[width=1cm]{example-image}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("EMPTY"), "{xml}");
  assert!(xml.contains("<graphics"), "{xml}");
}

/// italian.ldf:156-171: with ISO compliance on, `\unit` is the babel-italian
/// unit macro (verifica example4/5 `$25\unit{m}$`).
#[test]
fn babel_italian_unit_under_iso_compliance() {
  // `\setISOcompliance` must precede babel's own `begindocument` chunk
  // (italian.ldf:155-165 tests `\it@ISOcompliance` there). A document-level
  // `\AtBeginDocument{\setISOcompliance}` runs LAST in lthooks (`top-level`
  // after package labels) — pdflatex then also reports `\unit` undefined
  // (probed TL2025) — so the compliance switch is set in the preamble.
  let tex = r"\documentclass{article}
\usepackage[italian]{babel}
\setISOcompliance
\begin{document}
$25\unit{m}$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"font="upright""#) || xml.contains("mathrm"),
    "{xml}"
  );
}

/// latex_constructs.pool.ltxml:2588-2605: the bare text command is the
/// call-time encoding dispatcher, so textalpha's `normalize-symbols`
/// override of `\LGR\textbetasymbol` reaches `\textbetasymbol`
/// (greek-fontenc char-list, hyperref-with-greek); `\UseTextSymbol` runs
/// the encoding-specific body inside its encoding (`\textsigma` under T1
/// is σ, not a Latin `s`; KPE #148 slot 0x73).
#[test]
fn provide_text_command_dispatches_on_encoding() {
  let tex = r"\documentclass{article}
\usepackage[LGR,T1]{fontenc}
\usepackage[normalize-symbols]{textalpha}
\begin{document}
X\textbetasymbol Y\textthetasymbol Z \textsigma\textalpha
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>XβYϑZ σα</p>"), "{xml}");
}

/// ctex-heading-article.def:747 makes `\p@section` argument-taking; the
/// refnum formatter must close its `\csname` first (KPE #149; caspervector,
/// sduthesis, tabular2, inkpaper-en).
#[test]
fn ctex_argument_taking_p_macro_keeps_the_refnum() {
  let tex = r"\documentclass{article}
\makeatletter
\def\p@section#1{\thesection}
\makeatother
\begin{document}
\section{X}
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<tag role="refnum">1</tag>"#), "{xml}");
}

/// latex.ltx:107-110 + :896-1058: the lualatex format surface (attributes,
/// catcode tables, lua-function allocators, hyphenation chars) under the
/// `luatex` profile — luaotfload's `\input ltluatex`, luacolor's
/// `\setattribute`, tuenc.def's `\newprotectedluacmd`, babel's
/// `\prehyphenchar` (17 lualatex-oracle manuals).
#[test]
fn ltluatex_format_surface_under_luatex_profile() {
  let tex = r"\documentclass{article}
\usepackage{luaotfload}
\usepackage[TU]{fontenc}
\usepackage{luainputenc}
\makeatletter
\begin{document}
\prehyphenchar=`\- \newattribute\myattr \setattribute\myattr{7}[\the\myattr]
\newprotectedluacmd\mycmd \newcatcodetable\mytable \catcodetable\mytable
[\the\e@alloc@attribute@count]
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[7]"), "{xml}");
}

/// OXIDIZED_DESIGN #184: `\DeclareTextAccent` (lgrenc.def:439-470) defines
/// the Greek diacritics as combining-mark accents with an encoding
/// dispatcher (Perl ignores it: teubner.sty:165 `\let\~\accperispomeni`
/// then made `\~` undefined; textalpha's `\<`/`\>` breathings errored);
/// the dispatcher is `\fi`-free so an argument-taking text command sees
/// its argument. Also the section-type name of `\@@numbered@section` is
/// taken from the reverted tokens, not the LGR-decoded text
/// (`\theσεςτιον`).
#[test]
fn declare_text_accent_defines_greek_diacritics() {
  let tex = r"\documentclass{article}
\usepackage[LGR,T1]{fontenc}
\usepackage{textalpha}
\begin{document}
\fontencoding{LGR}\selectfont
\section{A}
[\<a][\accperispomeni{a}][\>'\textalpha][\accdialytika{i}][\accpsili{}]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[ἁ][ᾶ][ἄ][ϊ][\u{0313}]"), "{xml}");
  assert!(xml.contains(r#"<tag role="refnum">1</tag>"#), "{xml}");
}

/// multicol.sty.ltxml:22 closed an `ltx:p` a block spanning text had
/// already closed (KPE #150; thuaslogos-doc).
#[test]
fn multicols_spanning_section_is_not_double_closed() {
  let tex = r"\documentclass{article}
\usepackage{multicol}
\begin{document}
Intro.
\begin{multicols}{2}[\section*{Contents}]
Column text.
\end{multicols}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<title>Contents</title>"), "{xml}");
}

/// latex.ltx:9525 `begindocument/end` and :15257 `enddocument` fire; the
/// former is UNREAD so a `+b` environment opened from it (jwjournal.cls:643
/// wraps the whole body) reads the body from the file.
#[test]
fn begindocument_end_and_enddocument_hooks_fire() {
  let tex = r"\documentclass{article}
\ExplSyntaxOn
\NewDocumentEnvironment{wrapall}{+b}{[\regex_replace_all:nnN{\#\#}{\c{section}\*}\l_tmpa_tl\tl_set:Nn\l_tmpa_tl{#1}\regex_replace_all:nnN{\#\#}{\c{section}\*}\l_tmpa_tl\tl_use:N\l_tmpa_tl]}{}
\hook_gput_code:nnn{begindocument/end}{t}{\begin{wrapall}}
\hook_gput_code:nnn{enddocument}{t}{END-HOOK}
\ExplSyntaxOff
\begin{document}
Body text.

## {A New Section}

More.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<title>A New Section</title>"), "{xml}");
  assert!(xml.contains("END-HOOK"), "{xml}");
}

/// PLANS P37 (svg half): block content in a TikZ node (`\verb`) gets an
/// auto-opened `svg:foreignObject` (Flow model) — makeshape, optikz.
#[test]
fn verbatim_in_a_tikz_node_gets_a_foreign_object() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}
\node at (0,0) [draw] (a) {\verb|x  x|};
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<svg:foreignObject") && xml.contains("x  x</verbatim>"),
    "{xml}"
  );
}

/// The trivial-recursion guard anchors on the INVOKING token: a `\let`
/// alias of a macro whose body starts with the original CS is not a loop
/// by itself (musixlyr.tex:709-722 `\der@kontext`; recorder-fingering,
/// undar-digitacion-doc), while `\def\x{\x}` invoked as `\x` still is.
#[test]
fn recursion_guard_anchors_on_the_invoking_token() {
  let tex = r"\documentclass{article}
\begin{document}
\def\selfx{\selfx}
\def\ctx{\ctx A}
\let\alias\ctx
\def\ctx{}
\edef\zz{\alias}[\zz]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[A]"), "{xml}");
  let tex2 = r"\documentclass{article}
\begin{document}
\def\selfx{\selfx}\edef\zz{\selfx}
\end{document}
";
  let (stderr2, _) = convert(tex2, false);
  assert!(stderr2.contains("expands into itself"), "{stderr2}");
}

/// codehigh.sty:508 takes its `\directlua` parser under the luatex profile;
/// the binding degrades that path to plain verbatim text in bounded time
/// (the l3regex parser is O(n²), PLANS P65 — CreationBoites-doc,
/// tkz-bernoulli, tabularray-abnt, functional all timed out on it).
#[test]
fn codehigh_highlights_without_lua() {
  let tex = r"\documentclass{article}
\usepackage{codehigh}
\CodeHigh{language=latex/latex2}
\begin{document}
\begin{codehigh}
\foo{bar}
\end{codehigh}
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("bar") && xml.contains("foo"), "{xml}");
}

/// beamer internals the themes reach: beamerbasesection's `\secname`
/// family, `\beamer@slideinframe`, the gotham font theme's `\patchcmd`
/// targets, and `\titlegraphic` STORING its argument (Verona's `\node`).
#[test]
fn beamer_section_names_slide_counter_and_patch_targets() {
  let tex = r"\documentclass{beamer}
\usetheme{gotham}
\title{T}
\titlegraphic{\node[anchor=north]at(0,0){G};}
\makeatletter
\begin{document}
\section{Intro}
\begin{frame}\frametitle{\secname}[\number\beamer@slideinframe]\framebreak Body\end{frame}
\begin{frame}\titlepage\end{frame}
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[1]") && xml.contains("Body"), "{xml}");
}

/// Wave-11 package internals: `\captionbox` (caption.sty:454), pict2e's
/// `\polyline` family under curve2e, graphics' `\Grot@setangle`/`\Grot@box`
/// (isorot), xcolor's `\xcolor@`, hyperref's `\IfHyperBoolean`, biblatex's
/// `\AtUsedriver`/`\delimcontext`/`\DeclareAutoCiteCommand`, cas's xspace.
#[test]
fn wave11_package_internals_are_defined() {
  let tex = r"\documentclass{article}
\usepackage{caption}
\usepackage{curve2e}
\usepackage{isorot}
\usepackage{hyperref}
\usepackage{xspace}
\usepackage{xcolor}
\makeatletter
\begin{document}
\begin{figure}\captionbox{A caption\label{f}}[\linewidth]{Content}\end{figure}
\begin{picture}(10,10)\polyline(0,0)(10,10)(20,0)\polygon(0,0)(5,5)(10,0)\end{picture}
\begin{sideways}Hi\end{sideways}
[\IfHyperBoolean{hyperfootnotes}{yes}{no}][\xcolor@{}{X}{}{}]
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<caption>") && xml.contains("A caption"),
    "{xml}"
  );
  assert!(
    xml.contains("<line points=\"0,0 13.84,13.84 27.67,0\""),
    "{xml}"
  );
  assert!(xml.contains("angle=\"90"), "{xml}");
  assert!(xml.contains("[no][X]"), "{xml}");
}

/// tuenc.def:106-121 `\DeclareUnicodeAccent` under the luatex profile
/// (tipauni.sty:349) and the LuaTeX PDF primitives beside `\directlua`
/// (`\pdfvariable pageattr`, multimedia.sty:30).
#[test]
fn unicode_accent_and_pdfvariable_under_luatex_profile() {
  let tex = r#"\documentclass{article}
\usepackage{multimedia}
\begin{document}
\DeclareUnicodeAccent{\textsyllabic}{TU}{"0329}
[\textsyllabic{n}]
\end{document}
"#;
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[n\u{0329}]"), "{xml}");
}

/// ltxtable.sty:9 `\LTXtable{width}{file}` — a longtable with `X` columns
/// from a file; the raw macro reaches `\TX@target`/`\LT@echunk` internals
/// the bindings do not model (tikzcodeblocks-documentation, vhistory).
#[test]
fn ltxtable_inputs_a_longtable_with_x_columns() {
  let tex = r"\documentclass{article}
\usepackage{ltxtable}
\begin{document}
\LTXtable{\textwidth}{mytab.tex}
\end{document}
";
  let table = r"\begin{longtable}{lX}
a & some longer text that would wrap \\
b & more \\
\end{longtable}
";
  let (stderr, xml) = convert_with_files(tex, &[("mytab.tex", table)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<tabular") && xml.contains("<p>more</p>"),
    "{xml}"
  );
}

/// tex.web §1091 `new_graf` fires `\everypar` when a *list* starts a
/// paragraph; a constructor's digested `{}` argument is macro-parameter
/// text, not a list. An armed `\everypar` (latex.ltx:8090 `\@afterheading`'s
/// `{\setbox\z@\lastbox}`, left by ltugboat.cls:1214 `\aftergroup\@afterheading`
/// in `\@maketitle`) used to fire inside `\@@numbered@section`'s *type*
/// argument and revert as `{}section` — counter `\c@{}section`, tag
/// `ltx:{}section` (lazylist, parnotes). The body paragraph after the
/// heading still fires it.
#[test]
fn everypar_does_not_fire_inside_a_constructor_argument() {
  let tex = r"\documentclass{article}
\begin{document}
A\everypar{{\setbox0\lastbox}}
\section{Why lists?}
Text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("{}section"), "{stderr}");
  assert!(
    xml.contains(r#"<section inlist="toc" xml:id="S1">"#) && xml.contains("<tag>1</tag>"),
    "{xml}"
  );
  // A paragraph of the current list is `new_graf`: it still fires (the
  // algorithm2e `\nl` numbering rides on this).
  let tex = r"\documentclass{article}
\begin{document}
\everypar{EP:}Text.

More.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<p>EP:Text.</p>") && xml.contains("<p>EP:More.</p>"),
    "{xml}"
  );
}

/// lineno.sty:1077 `\newif\ifLineNumbers` — the binding lacked lineno's
/// switches, and `\lx@deposit@maketitle` (OD #124) runs a class's
/// `\@maketitle`, which for homework.cls:128 reaches minimalist.sty:144
/// `\LocallyStopLineNumbers` = `…\ifLineNumbers\LNturnsONtrue\fi…`
/// (homework-demo-{cn,de,en,es,fr,jp}).
#[test]
fn lineno_binding_defines_the_line_number_switches() {
  let tex = r"\documentclass{article}
\usepackage{lineno}
\makeatletter
\renewcommand{\@maketitle}{\ifLineNumbers\fi\ifoddNumberedPage\fi\ifcolumnwiselinenumbers\fi Body}
\makeatother
\title{X}\author{Y}
\begin{document}
\maketitle
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<title>X</title>") && xml.contains("<personname>Y</personname>"),
    "{xml}"
  );
}

/// examdesign.cls:323-344 owns `\section` as a *non-sectioning* macro and
/// `\begin{section}…\end{section}` wraps every question block
/// (examdesign.cls:802-812); the locked kernel `\section` ran
/// `\@startsection` on the environment body instead (examplea/b/c: Perl 67
/// errors, Rust Fatal after 100). The class binding unlocks it before the
/// raw load.
#[test]
fn examdesign_owns_section_as_an_environment() {
  let tex = r"\documentclass{examdesign}
\begin{document}
\begin{matching}[title={T}]
  \pair{Elvis}{Spike}
  \pair{Nirvana}{Nevermind}
\end{matching}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("<section"), "{xml}");
  assert!(xml.contains("Elvis") && xml.contains("Nevermind"), "{xml}");
  assert!(xml.matches("<item").count() >= 4, "{xml}");
}

/// latex.ltx:9551 `\protected@write` freezes `\protect`ed macros into the
/// index entry (`\let\protect\@unexpandable@protect`); expanding them at
/// `\index` time ran manyind.sty:100/119's `\protect\def\nwletre{…}` and
/// `\protect\nxtletre` (`\proc@letter`'s caller-closing `\fi`) in the
/// gullet (mindsample: `undefined \nwletre`, stray `\fi`).
#[test]
fn index_entry_defers_protected_macros() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}\makeindex
\long\def\ltest#1{\ifx#1\ltest\else X\fi}
\newcommand\nxt{\def\item{\ltest}}
\begin{document}
A\index{key@\protect\nxt}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<indexphrase key="key"/>"#), "{xml}");
}

/// pdfTeX `annot type spec` = `[useobjnum n] [rule spec] general text`;
/// the `(width|height|depth) dimen` rule spec was never read (Perl
/// pdfTeX.pool:156-171 too; KPE #151). pdfmarginpar.sty:142 passes it
/// whenever a `width=`/`height=` key is set (pdfmarginpar doc).
#[test]
fn pdfannot_reads_its_rule_spec() {
  let tex = r"\documentclass{article}
\begin{document}
Hi\pdfannot width 4cm height 0.5cm {/Subtype /Text /Contents (x)} there
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Hi there</p>"), "{xml}");
}

/// robustindex.sty:201-216 `\gobblepageref`/`\wrappageref` scan for the
/// `, \indpageref{N}` makeindex writes into an `.ind` line; LaTeXML's index
/// has no such line (robustsample.tex:82; multisample, robustmanual).
#[test]
fn robustindex_page_reference_hooks_are_inert() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}
\usepackage{robustindex}
\makeindex
\begin{document}
A\index{alpha!see also gamma\gobblepageref}
B\index{beta\wrappageref\textbf}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">alpha</indexphrase>"), "{xml}");
  assert!(xml.contains(">see also gamma</indexphrase>"), "{xml}");
  assert!(xml.contains(">beta</indexphrase>"), "{xml}");
}

/// `\abstract{…}` is the environment's begin code plus a plain group in
/// LaTeX, read incrementally — a `\makeatletter` inside it precedes the
/// `\patch@level` that follows (char-list-alphabeta.tex:88-103; PLANS P74,
/// SHARED). It was taken as one pre-tokenized `{}` argument.
#[test]
fn braced_abstract_reads_its_body_incrementally() {
  let tex = r"\documentclass{article}
\makeatletter\def\patch@level{7}\makeatother
\title{T}
\begin{document}
\maketitle
\abstract{ \noindent Test.
\makeatletter
patch-level \patch@level{} here.
\makeatother
}
Body text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<abstract"), "{xml}");
  assert!(xml.contains("patch-level 7 here."), "{xml}");
  assert!(xml.contains("<p>Body text.</p>"), "{xml}");
}

/// latex.ltx:10140-10156 keep a counter's reset list as the macro
/// `\cl@<ctr>` = `\@elt{child}…`; raw code expands and rewrites it
/// (contract.sty:336 `\edef\cl@Clause{\cl@Clause\cl@contractClause}`,
/// afthesis.cls:44-49 `\@removefromreset` re-`\edef`). LaTeXML's State value
/// stays authoritative; the macro mirrors it after every mutation.
#[test]
fn reset_list_is_an_expandable_cl_macro() {
  let tex = r"\documentclass{article}
\makeatletter
\newcounter{Clause}\newcounter{contractClause}[Clause]
\newcounter{Extra}\@addtoreset{Extra}{Clause}
\edef\cl@Clause{\cl@Clause\cl@contractClause}
\def\@elt#1{[#1]}
\begin{document}
A\cl@Clause B\cl@contractClause C
\stepcounter{contractClause}\stepcounter{Clause}\thecontractClause
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[contractClause]") && xml.contains("[Extra]"),
    "{xml}"
  );
  assert!(xml.contains("BC"), "{xml}");
  // the Value list still drives the reset: Clause stepped -> contractClause back to 0
  assert!(xml.contains("0</p>") && !xml.contains("1</p>"), "{xml}");
}

/// `\mbox\bgroup A … B\egroup` (syntax.sty:158 `\syn@assist`; the newcommand
/// manual's `grammar` environment): TeX hands `\bgroup` to `\mbox#1` as its
/// one-token argument and the box then runs to the `\egroup`. A `{}` argument
/// that is exactly an implicit begin-group reads its group by digestion.
#[test]
fn implicit_bgroup_argument_reads_its_group() {
  let tex = r"\documentclass{article}
\def\OPEN{\mbox\bgroup A}
\def\CLOSE{ B\egroup}
\begin{document}
X\OPEN\CLOSE Y
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("XA BY"), "{xml}");
  let tex = r#"\documentclass{article}
\usepackage{syntax}
\begin{document}
\begin{grammar}
<decl> ::= \[[ "MACRO" <ident> \]]
\end{grammar}
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("MACRO"), "{xml}");
}

/// amsldoc.cls:84-89 `\indexcs` writes the sort key of `\cn{\\*}` as the
/// `\string`ed `\*` — catcode-12 after `\@sanitize` (latex.ltx:1778); the
/// whole-string re-tokenization welded it into the live `\*` (amsldoc.cls:213)
/// which ate the entry (itamsldoc, amsldoc-vi; PLANS P73, SHARED).
#[test]
fn index_sanitized_backslash_symbol_stays_literal() {
  let tex = r#"\documentclass{amsldoc}
\usepackage{guit}
\usepackage{makeidx}\makeindex
\begin{document}
Il comando \cn{\\*} qui.
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<indexmark").count(), 1, "{xml}");
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// expl3-code.tex:7846-7861 fixes `\c_sys_engine_str` and the
/// `\sys_if_engine_<e>` conditionals at format-build time; the `luatex`
/// profile must re-derive them (polyglossia gloss-latin.ldf:125 else takes
/// the XeTeX branch — hang, sample), and unicode-math's `\math<style>`
/// aliases (unicode-math-luatex.sty:2273-2306; toptesi topcoman.sty:76
/// `\mathup`) must exist.
#[test]
fn l3sys_engine_identity_under_luatex_profile() {
  let tex = r"\documentclass{article}
\usepackage{unicode-math}
\begin{document}
\ExplSyntaxOn
[\c_sys_engine_str][\sys_if_engine_luatex:TF{L}{X}][\sys_if_engine_pdftex:TF{P}{N}][\c_sys_engine_format_str]
\ExplSyntaxOff
$\mathup{\mu}+\mathbfit{x}+\symscr{S}$
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[luatex][L][N][lualatex]"), "{xml}");
  assert!(xml.contains(r#"tex="\mathrm{\mu}+"#), "{xml}");
}

/// tex.web §368: `\expandafter` expands the second token FIRST and only
/// then `back_input`s the saved one, so a saved `{` still counts in
/// `align_state` while the expansion reads its arguments. Rust retracted
/// the brace before the expansion: in `\exp_after:wN { \use_none:nn & …}`
/// (numerica.sty:1748 `\__nmc_delim_arg:` on the slash path of
/// `\eval{1/8}`) the `&` was read at ledger 0 and fired the cell template
/// mid-cell — the amsmath after-`$` inserted early, the before-`$` frame
/// left open (numerica 83, mhchem `\ce` 14, tablists-rus 101; Perl shares
/// it). The package-free shape is the second document. The plain `{$b$}`
/// in a cell stays a TeX error (§1065 `off_save`). NB tex.web `macro_call`
/// keeps `align_state` LIVE during parameter scanning — a freeze there
/// broke `columncolor_lbrack_cell_does_not_cascade_the_column_mode`.
#[test]
fn argument_scan_is_align_state_neutral() {
  let tex = r"\documentclass{article}\usepackage{amsmath}
\ExplSyntaxOn
\newcommand\doit{\exp_after:wN { \use_none:nn & Z } }
\ExplSyntaxOff
\begin{document}
\begin{align*}
a &= 1 \doit + 2
\end{align*}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<equationgroup"), "{xml}");
  let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{numerica}
\begin{document}
\begin{align*}
a & =\eval{1/8} \\
b & =\eval{1/8} & c &= 2
\end{align*}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<equationgroup") && xml.contains(">0.125</XMTok>"),
    "{xml}"
  );
  let tex = r"\documentclass{article}\usepackage{amsmath}
\begin{document}
\begin{align*}
a &= {$b$} + c
\end{align*}
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  assert!(
    error_count(&stderr) > 0,
    "a $ under a simple group must stay an error:\n{stderr}"
  );
}

/// latex.ltx:1729-1737 `\@ifundefined` probes with `\ifcsname` and leaves the
/// name undefined; the `\relax` pollution broke every reentrancy-guarded
/// `.def` loaded as `\@ifundefined{sentinel}{\input file}{}` — polyglossia's
/// gloss-latin.ldf:591 + babelsh.def:1 (hang, sample; Perl pollutes too).
#[test]
fn ifundefined_does_not_define_the_name() {
  let tex = r"\documentclass{article}
\makeatletter
\@ifundefined{zz@undef}{}{}
\ifx\zz@undef\@undefined [STILL-UNDEFINED]\else [POLLUTED]\fi
\@ifundefined{zz@undef}{[U]}{[D]}
\def\zz@def{}\@ifundefined{zz@def}{[U]}{[D]}
\makeatother
\begin{document}
\makeatletter\ifx\zz@undef\@undefined [BODY-UNDEFINED]\fi\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[BODY-UNDEFINED]"), "{xml}");
  let tex = r"\documentclass{article}
\usepackage{polyglossia}
\setdefaultlanguage{english}
\setotherlanguage{latin}
\begin{document}
Text \textlatin{lingua latina} here.
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[luatex,rawstyles,rawclasses]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("lingua latina"), "{xml}");
}

/// latex.ltx:16369 `\DeclareRobustCommand\underline`: robust, so an
/// `\edef`/`\write` freezes the whole `\ifmmode…\fi` body (bibarts.sty:2231
/// `\edef\@tempa{\write\@auxout{…\underline{Publ.}…}}`; Perl's non-robust
/// body tears at `\else`).
#[test]
fn underline_is_robust_in_an_edef_write() {
  let tex = r"\documentclass{article}
\makeatletter
\begin{document}
\let\protect\@unexpandable@protect
\edef\x{\underline{Publ.}\overline{X}}
\let\protect\relax
\x
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<text framed="underline">Publ.</text>"#) || xml.contains(">Publ.</text>"),
    "{xml}"
  );
}

/// `\overrightarrow`/`\overleftarrow` are `protected` like their siblings
/// `\overline`/`\underline` (fontmath.ltx:424 `\DeclareRobustCommand`): the
/// `\index` phrase pass froze only their `\ifmmode` and ran the bare
/// `\else`/`\fi` against an empty conditional stack (two errors and a
/// doubled key `→abc→abc`; latex-via-exemplos' accent table).
#[test]
fn overrightarrow_is_robust_in_an_index_phrase() {
  let tex = "\\documentclass{article}\n\\usepackage{makeidx}\\makeindex\n\\begin{document}\nX\\index{$\\overrightarrow{abc}$} Y\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "indexmark",
    &[],
    r##"<indexmark><indexphrase key="→abc"><Math mode="inline" tex="\overrightarrow{abc}" text="overrightarrow@(a * b * c)" xml:id="p1.m1"><XMath><XMApp><XMTok name="overrightarrow" role="OVERACCENT" stretchy="true">→</XMTok><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMApp></XMath></Math></indexphrase></indexmark>"##,
  );
}

/// A raw class's full `\@maketitle` (ascelike.cls:406-411 `\AB@authlist`)
/// runs under `\lx@deposit@maketitle` (OD #124); the bindings' semantic
/// `\author` never fills authblk's visual accumulators, which therefore exist
/// at their package-initial empty value so the layout collapses to nothing.
#[test]
fn class_maketitle_layout_over_binding_accumulators() {
  let tex = r"\documentclass{article}
\usepackage{authblk}
\author{Alice}
\title{T}
\makeatletter
\renewcommand{\@maketitle}{\begin{center}\@title\\ \AB@authlist\thankses\end{center}}
\makeatother
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<personname>Alice</personname>"), "{xml}");
  assert!(
    !xml.contains("<ERROR") && !xml.contains("AB@authlist"),
    "{xml}"
  );
}

/// tex.web §1083 `begin_box` pushes nest and save level TOGETHER for
/// `\hbox\bgroup`; Perl `readBoxContents` (TeX_Box.pool:164-185) uses one
/// frame. The two-frame hbox reader left ulem's open-here/close-there word
/// boxes (examdesign.cls:186-200 `\UL@start`/`\UL@stop`) around a
/// `\makebox` meeting the wrong frame (examdesign examplea/b/c; Perl shares).
#[test]
fn hbox_reader_is_one_frame() {
  let tex = r"\documentclass[10pt]{examdesign}
\Fullpages
\ContinuousNumbering
\DefineAnswerWrapper{}{}
\NumberOfVersions{2}
\class{{\Large A sample exam}}
\begin{document}
\begin{truefalse}[title={T/F}]
\begin{question}
  \answer{True} This sentence is not false.
\end{question}
\end{truefalse}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("This sentence is not false."), "{xml}");
  // `\hbox{a}` still reverts with its braces
  let tex = r"\documentclass{article}
\begin{document}
$\hbox{ab}$ \setbox0\hbox\bgroup x\egroup\box0
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ab</text> x"), "{xml}");
}

/// latex.ltx:18557 `\ProcessOptions` reads `\@ptionlist{\@currname.\@currext}`
/// = the MACRO `\opt@<pkg>.<ext>`, which babel.sty:316-347 rewrites to strip
/// its `language.modifier` syntax (`greek.polutoniko` → `greek`,
/// `\bbl@mod@greek`=polutoniko). Reading the loader's State list instead
/// raised "Unknown option 'greek.polutoniko'" (alphabeta-doc,
/// hyperref-with-greek; Perl shares it).
#[test]
fn processoptions_reads_the_rewritten_opt_macro() {
  let tex = r"\documentclass{article}
\usepackage[greek.polutoniko,english]{babel}
\begin{document}
\makeatletter[\bbl@mod@greek]\makeatother \textgreek{a}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[polutoniko]"), "{xml}");
  let tex = r"\documentclass{article}
\makeatletter
\def\lx@rewriter@sty{}
\DeclareOption{alpha}{\gdef\seen{ALPHA}}\DeclareOption{beta}{\gdef\seen{BETA}}
\def\@currname{article}\def\@currext{cls}
\expandafter\def\csname opt@article.cls\endcsname{beta}
\ProcessOptions\relax
\makeatother
\begin{document}
[\seen]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[BETA]"), "{xml}");
}

/// latex.ltx `\mbox{#1}` = `\leavevmode\hbox{#1}`: the content is an hbox
/// BODY read in the same list, so ulem's `\hss` (`\UL@hskip` →
/// `\afterassignment\UL@reskip` → `\UL@stop` `\egroup\egroup` … `\UL@start`)
/// inside `\makebox[.5in][r]{\hss}` (examdesign.cls:1210) closes the makebox
/// and the makebox's own `}` closes the box ulem reopened (OD #188). The
/// common shapes keep their structure.
#[test]
fn box_constructor_content_is_a_live_hbox_body() {
  let tex = r"\documentclass{article}
\begin{document}
\makebox[2cm][r]{mk} \mbox{x y} \fbox{fb} \raisebox{1pt}{rb} \framebox[3cm]{fr} $\fbox{$op$}$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<text align="right" width="56.9pt">mk</text>"#),
    "{xml}"
  );
  assert!(xml.contains("x y"), "{xml}");
  assert!(xml.contains(r#"framed="rectangle">fb</text>"#), "{xml}");
  assert!(xml.contains(r#"<text yoffset="1.0pt">rb</text>"#), "{xml}");
  assert!(
    xml.contains("<XMArg enclose=\"box\">") || xml.contains(r#"tex="\framebox{$op$}""#),
    "{xml}"
  );
}

/// latex.ltx:14978 `\labelformat#1` = `\expandafter\def\csname p@#1\endcsname##1`
/// (kernel since 2019-10-01; varioref only re-exports it). contract.sty:978
/// probes it with `\scr@ifundefinedorrelax{labelformat}` and, when it is
/// missing, falls back to the pre-2019 `\p@sentence`=`\expandafter\p@@sentence`
/// prefix, whose one-token grab of `\thesentence`'s expansion (`\arabic`)
/// leaves `{sentence}` behind and ends `\refstepcounter`'s `\@currentlabel`
/// with `\arabic}` ("You can't use } after \the" ×3 per sentence,
/// contract-example-en 44 errors; Perl shares it, KPE #160). With the kernel
/// macro the `\labelformat` branch wins and `\p@sentence` takes
/// `\thesentence` whole, as it does under pdflatex.
#[test]
fn labelformat_is_a_kernel_macro() {
  let tex = r"\documentclass{article}
\makeatletter
\newcounter{par}\newcounter{sentence}[par]
\renewcommand*{\thesentence}{\arabic{sentence}}
\def\p@par{[P]}
\@ifundefined{labelformat}{%
  \renewcommand*{\p@sentence}{\expandafter\p@@sentence}%
  \newcommand*{\p@@sentence}[1]{\p@par{{\thepar}-}{S:#1}}%
}{\labelformat{sentence}{\p@par{{\thepar}-}{S:#1}}}
\makeatother
\labelformat{equation}{[E:#1]}
\newtheorem{thm}{Theorem}\labelformat{thm}{[T:#1]}
\begin{document}
\refstepcounter{par}\refstepcounter{sentence}\label{s}
X Y \ref{s}
\begin{equation}\label{e}x\end{equation}
\begin{thm}\label{t}x\end{thm}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<tag role="refnum">[E:1]</tag>"#), "{xml}");
  // typerefnum goes through the same `\p@<ctr>\the<ctr>` helper.
  assert!(
    xml.contains(r#"<tag role="typerefnum">Theorem [T:1]</tag>"#),
    "{xml}"
  );
}

/// LuaTeX's `\matheqdirmode` integer parameter (LuaTeX manual §6) beside
/// its profile siblings (`\matheqnogapstep`, `\breakafterdirmode`);
/// minim-math.tex:19 sets it (lettrine-demo-arabic, 1 error).
#[test]
fn luatex_profile_defines_matheqdirmode() {
  let tex = r"\documentclass{article}
\matheqdirmode=1
\begin{document}
[\the\matheqdirmode]
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[1]"), "{xml}");
}

/// pdfTeX `\pdfoutline [attr spec] action spec [count N] general text` and
/// `\pdfdest <id> <dest type>` (manual §8.13-8.14) produce PDF navigation
/// only, but the specs must be CONSUMED: tools-overview.tex:93 `\pdfoutline
/// attr {…} user {…} {[#1]}` leaked `attr`/`user` into the text (Perl
/// pdfTeX.pool:179-180 only comments them, KPE #162).
#[test]
fn pdfoutline_and_pdfdest_consume_their_specs() {
  let tex = r"\documentclass{article}
\begin{document}
\pdfoutline attr {/C[0 0 1]} user {<< /S/GoToR /F(x.pdf) >>} {[Section 1]}\relax
\pdfoutline goto name {sec1} count -2 {Sec}\pdfoutline goto file {o.pdf} page 3 {top} newwindow {Other}%
\pdfdest name {sec1} xyz zoom 1000 \pdfdest num 7 fitr width 2cm height 1cm \pdfdest name {a} fith
Body text.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>Body text.</p>"), "{xml}");
  assert!(
    !xml.contains("attr") && !xml.contains("user") && !xml.contains("zoom"),
    "{xml}"
  );
}

/// utf8.def:253-265 `\parse@UTFviii@a`/`@b` are KERNEL macros (latex.ltx:
/// 22224 inputs utf8.def at format time); paresse-utf8.sty:203-204 `\let`s
/// them to build its own UTF-8 sequences (paresse-eng 3, -fra 6 errors;
/// Perl utf8.def.ltxml omits them too, KPE #163).
#[test]
fn utf8_octet_parsers_are_defined() {
  let tex = r"\documentclass{article}
\makeatletter
\count@=233 \parse@UTFviii@a;\parse@UTFviii@b C\UTFviii@two@octets.;
\edef\x{\expandafter\meaning\csname UTFviii@tmp\endcsname}
\makeatother
\begin{document}
[\x]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // 233 = 0xE9 → octets C3 A9, uppercased to the bytes' glyphs.
  assert!(xml.contains("UTFviii@two@octets Ã©"), "{xml}");
}

/// ltablex.sty makes `tabularx` a multi-page (longtable-driven) table and
/// defines `\keepXColumns`/`\convertXColumns` (:146-153) as toggles of
/// `\ifTX@convertX@`; the former stub defined neither (milsymb.tex, 44
/// errors; Perl raw-loads the file). `\endhead` is legal inside.
#[test]
fn ltablex_tabularx_is_a_longtable_with_toggles() {
  let tex = r"\documentclass{article}
\usepackage{ltablex}
\keepXColumns
\begin{document}
\begin{tabularx}{\textwidth}{|c|l|X|}
 h1 & h2 & h3 \\ \hline \endhead
 a & b & c \\ \hline
 d & e & f \\ \hline
\end{tabularx}
\makeatletter\ifTX@convertX@ [CONVERT]\else [KEEP]\fi\convertXColumns\ifTX@convertX@ [CONVERT]\fi\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<thead"), "{xml}");
  assert!(xml.matches("<td").count() >= 9, "{xml}");
  assert!(xml.contains("[KEEP]") && xml.contains("[CONVERT]"), "{xml}");
}

/// pgfmath functions that real pgf defines with an integer literal result
/// (`sign`, `iseven`/`isodd`/`isprime`, `gcd`, `div`, `scalar`, `true`/
/// `false`, `!`) must print without `.0`, because packages feed them to
/// `\ifnum`: tikzbricks.sty:146-151 `\ifnum\brick@sin<0` on `sign(sin(…))`
/// broke at the `.` ("Expected a relational token"; tikzbricks doc 90
/// errors, Perl identical). Probed against pdflatex/pgf TL2025.
#[test]
fn pgfmath_integer_functions_yield_integers() {
  let tex = r"\documentclass{article}
\usepackage{pgfmath}
\begin{document}
\def\P#1{\pgfmathparse{#1}[\pgfmathresult]}
\P{sign(-2.5)}\P{sign(0)}\P{iseven(4)}\P{isodd(4)}\P{isprime(7)}\P{gcd(12,18)}\P{div(7,2)}\P{scalar(3)}\P{true}\P{false}\P{!0}%
\P{floor(3.7)}\P{abs(-3)}\P{2+sign(1)}
\pgfmathparse{sign(-3)}\let\s\pgfmathresult \ifnum\s<0 [NEG]\fi
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[-1][0][1][0][1][6][3][3][1][0][1][3.0][3.0][3.0]"),
    "{xml}"
  );
  assert!(xml.contains("[NEG]"), "{xml}");
}

/// xcolor's `\color@<name>` storage is `\xcolor@{}{}{model}{spec}` and
/// `\xcolor@` is a real macro (xcolor.sty:603 `\def\xcolor@#1#2#3#4{#2}`),
/// so the fallback lookup must read the REPLACEMENT TEXT, not an
/// expansion (which collapsed to ""): ydoc-desc.sty:22's empty `none`
/// color raised "Can't find color named 'none'" (iodhbwm; Perl identical).
#[test]
fn xcolor_storage_macro_is_read_as_a_body() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\makeatletter
\expandafter\def\csname\string\color@none\endcsname{\xcolor@ {}{}{}{}}
\expandafter\def\csname\string\color@myred\endcsname{\xcolor@ {}{}{rgb}{1,0,0}}
\makeatother
\colorlet{cls}{none}
\begin{document}
\textcolor{cls}{hello} \textcolor{myred}{red}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("hello"), "{xml}");
  assert!(xml.contains(r##"color="#FF0000">red"##), "{xml}");
}

/// When xcolor is loaded, `\definecolor` registers `\\color@<name>` using the
/// standard LaTeX shape `\xcolor@{}{<driver_spec>}{<model>}{<spec_comma>}`.
/// Packages like colorspace.sty hook into `\xcolor@` inside `\definespotcolor`
/// to inspect components and driver commands (colorspace.tex).
#[test]
fn def_color_macro_emits_xcolor_representation() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\definecolor{testc}{cmyk}{0.8,0.2,0.5,0.3}
\makeatletter
\def\spctest#1{%
  \begingroup
    \def\xcolor@##1##2##3##4{%
      \gdef\extractedmodel{##3}%
      \gdef\extractedspec{##4}}%
    \csname\string\color@#1\endcsname
  \endgroup}
\spctest{testc}
\makeatother
\begin{document}
Model: \extractedmodel, Spec: \extractedspec
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Model: cmyk, Spec: 0.8,0.2,0.5,0.3"), "{xml}");
}

/// LaTeX runs `\section`/`\paragraph` inside an `\item` or a float body (the
/// heading is set in the list's indentation; ddphonism, phonrule, prerex,
/// pdfmarginpar — pdflatex clean). Both engines build the nested
/// `<ltx:item><ltx:subsection>`; Perl errors and inserts anyway
/// (Document.pm openElement), so only the diagnostic differed. The builder's
/// sectioning-in-frontmatter leniency now covers the whole sectioning
/// family inside `ltx:item`/`ltx:figure` (OD #189).
#[test]
fn sectioning_unit_inside_item_or_figure_is_lenient() {
  let tex = r"\documentclass{article}
\begin{document}
\begin{itemize}
\item First item.
\subsection{Heading inside item}
More text.
\end{itemize}
\begin{figure}
Figure body text.
\paragraph{Notes} inside the figure.
\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<subsection"), "{xml}");
  assert!(xml.contains("<paragraph"), "{xml}");
  assert!(xml.contains("<figure"), "{xml}");
}

/// A math node arriving in an Inline-model element opened in math mode — a
/// `\hyperref[l]{b}` or glossaries' `\glsdisp{k}{k}` under `\ensuremath`
/// (glosmathtools.sty:74; `<ltx:XMTok> isn't allowed in <ltx:glossaryref>`,
/// sample_glosmathtools ×2 53 errors; Perl TeX_Math.pool:42 autoOpens only
/// XMText, so it shares the error) — takes the `\text{$k$}` shape: an
/// auto-opened inline `ltx:Math`/`ltx:XMath` inside the ref (OD #190).
#[test]
fn math_content_in_a_ref_gets_an_inline_math() {
  let tex = r"\documentclass{article}
\usepackage{hyperref}
\usepackage{glossaries}
\newglossaryentry{k}{name={\ensuremath{k}},description={discrete time}}
\begin{document}
\label{s}$x+\hyperref[s]{b}$ and \(a = \glsdisp{k}{k} + 1\).
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<ref font="italic" labelref="LABEL:s"><Math mode="inline""#),
    "{xml}"
  );
  assert!(xml.contains(r#"key="k"><Math mode="inline""#), "{xml}");
}

/// hyperref.sty:8183-8203 `\autopageref{label}` = `\hyperref[{label}]
/// {\HyRef@autopagerefname\pageref*{label}}` — "page <n>" through the
/// language's `\pageautorefname`; absent in Perl's hyperref.sty.ltxml
/// (abntex2cite.tex:1367; KPE #164).
#[test]
fn autopageref_is_a_page_reference() {
  let tex = r"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\section{A}\label{s}
See \autopageref{s} and \autopageref*{s}.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(
      "See page\u{a0}<ref labelref=\"LABEL:s\"/> and page\u{a0}<ref labelref=\"LABEL:s\"/>."
    ),
    "{xml}"
  );
}

/// LaTeX's tabular entry template is a brace group (latex.ltx `\@classz`:
/// `{\hfil\hskip1sp\ignorespaces\@sharp\unskip\hfil}`), so `\aftergroup`
/// in a cell fires at the entry's `}` — inside the cell, before `&`/`\cr`
/// is acted on. The cell frame's tokens used to be unread after the column
/// ended, so babel's `\selectlanguage` (`\aftergroup\bbl@pop@language`) in
/// a non-first cell ran as the NEXT cell and, after the last cell, opened a
/// spurious one ("`\@end@tabular` Attempt to close boxing group";
/// uantwerpenexam-example2 41, derivative 101; Perl identical).
#[test]
fn aftergroup_in_a_tabular_cell_fires_inside_the_cell() {
  let tex = r"\documentclass{article}
\usepackage[dutch,english]{babel}
\def\foo{\gdef\fired{[FIRED]}}
\begin{document}
\begin{tabular}{cc}%
\selectlanguage{english}A%
&
\selectlanguage{dutch}B%
\end{tabular}
\begin{tabular}{cc} a & \aftergroup\foo b \\ c & d \end{tabular}\fired
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tabular").count(), 2, "{xml}");
  assert!(xml.matches("<td").count() >= 6, "{xml}");
  assert!(xml.contains("[FIRED]"), "{xml}");
}

/// After a sectioning unit is leniently nested in a list item (OD #189),
/// the NEXT sectioning command closes it and becomes its SIBLING inside the
/// item — latex.ltx's `\@startsection` ends the previous heading's scope,
/// not the list; a `\section` after `\end{itemize}` is at the outer level
/// (ddphonism; Perl nests Y inside X with a second error).
#[test]
fn next_sectioning_unit_in_an_item_is_a_sibling() {
  let tex = r"\documentclass{article}
\begin{document}
\begin{itemize}
\item A \subsection{X} text \subsection{Y} more
\end{itemize}
\section{Z}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let x = xml
    .find(r#"<subsection inlist="toc" xml:id="S0.SS1">"#)
    .expect("X");
  let x_end = xml[x..].find("</subsection>").expect("X end") + x;
  let y = xml
    .find(r#"<subsection inlist="toc" xml:id="S0.SS2">"#)
    .expect("Y");
  assert!(y > x_end, "Y must follow X's close as a sibling:\n{xml}");
  let item_end = xml.find("</item>").expect("item end");
  assert!(y < item_end, "Y stays inside the item:\n{xml}");
  let z = xml
    .find(r#"<section inlist="toc" xml:id="S1">"#)
    .expect("Z");
  assert!(z > xml.find("</itemize>").unwrap(), "{xml}");
}

/// beamer.cls:144-156 `\beamer@size` = the size .clo the class inputs (:363);
/// themes read it (beamerthemeAlbi.sty:192 `size/.expanded=\beamer@size`
/// as a pgfkeys choice). The binding's option remap never set it
/// (beamer-theme-albi-doc; Perl identical).
#[test]
fn beamer_size_option_is_recorded() {
  let tex = r"\documentclass[14pt]{beamer}
\makeatletter
\def\showsize{[\expandafter\@firstofone\beamer@size]}
\makeatother
\begin{document}
\begin{frame}\showsize\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("[size14.clo]") || xml.contains("[size11.clo]"),
    "{xml}"
  );
}

/// beamerbaseoverlay.sty:590-597 wraps `\color` and the `\text<font>`
/// commands with an `<overlay>` reader (Perl beamer.cls.ltxml:1345-1356
/// `%BEAMER_WRAPPED`); without it `\color<2>{red}` read `<` as the color
/// ("Can't find color named '<'", xskak_and_beamer 34 errors).
#[test]
fn beamer_color_and_text_commands_take_an_overlay() {
  let tex = r"\documentclass{beamer}
\begin{document}
\begin{frame}
\color<2>{red}Hello \textbf<2->{bold} \textcolor{blue}{b}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"color="#FF0000""##), "{xml}");
  assert!(xml.contains(r#"font="bold">bold"#), "{xml}");
}

/// latex.ltx:15438 `\@xverbatim` is delimited by the catcode-12 string
/// `\end{verbatim}` and `\end` runs `\endgroup` BEFORE the rest of that
/// line is tokenized, so a `\verb` on the same line scans with restored
/// catcodes. The pre-tokenized remainder (Perl latex_constructs.pool:1777)
/// handed `\verb` frozen tokens: its delimiter never matched and the rest
/// of the DOCUMENT was re-read under `\dospecials` (ddphonism:87; KPE #165).
/// A TAB keeps catcode 10 in verbatim, so it is a space, not OT1 slot 9.
#[test]
fn verb_on_the_endverbatim_line_scans_raw() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{itemize}\n\\item A\n\\begin{verbatim}\n\tx\n\\end{verbatim} same \\verb|z| y.\n\\item B\n\\end{itemize}\n\\section{Next}\nT\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"same <verbatim font="typewriter">z</verbatim> y."#),
    "{xml}"
  );
  assert_eq!(xml.matches("<item xml:id").count(), 2, "{xml}");
  assert!(xml.contains("<section"), "{xml}");
  assert!(!xml.contains('Ψ'), "{xml}");
  assert!(
    xml.contains("\n x\n") || xml.contains("\n\u{2423}x\n") || xml.contains(">\n x"),
    "{xml}"
  );
}

/// marginnote.sty:319-343 routes the note body through three macro-argument
/// layers (`\@dblarg\@mn@marginnote` → `\@mn@@marginnote` →
/// `\@mn@@@marginnote`); a binding that expands straight to `\marginpar`
/// (Perl marginnote.sty.ltxml:37-40) is one layer short, so skdoc.cls:631's
/// `\marginnote{…\clist_map_inline:Nn…{\index@option*{####1}}}` leaked a
/// literal `#1` and mis-keyed every glossary entry (iodhbwm 146 errors).
#[test]
fn marginnote_body_rides_three_argument_layers() {
  let tex = r"\documentclass{article}
\usepackage{marginnote}
\usepackage{xparse}
\ExplSyntaxOn
\DeclareDocumentCommand\Options{m}{
  \clist_set:Nn\l_tmpa_clist{#1}
  \marginnote{
    \clist_map_inline:Nn\l_tmpa_clist{ [####1] }
  }
}
\ExplSyntaxOff
\begin{document}
Body.\Options{alpha,beta} \marginnote[L]{R}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[alpha]") && xml.contains("[beta]"), "{xml}");
  assert!(!xml.contains("#1"), "{xml}");
  assert!(xml.contains(">R<") || xml.contains("R</note>"), "{xml}");
}

/// A registered contrib binding REPLACES the raw file: the schooldocs
/// binding must load schooldocs.sty first (`\RequirePackage{xcolor}` :32,
/// `titlecolor` :100, `\subject`…) and patch on top (schooldocs-examples
/// 17 errors: `\definecolor` undefined; Perl raw-loads it clean).
#[test]
fn schooldocs_binding_loads_the_real_style() {
  let tex = r"\documentclass{article}
\usepackage{schooldocs}
\definecolor{darkbrown}{rgb}{0.5,0.1,0.1}
\begin{document}
\textcolor{darkbrown}{y}\textcolor{titlecolor}{t}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"color="#801A1A">y"##), "{xml}");
}

/// soul-ori.sty:557-567 `\SOUL@setup` resets the scanner's redefinable
/// hooks; highlightx.sty:193 / proofread.sty:74 run it, redefine the hooks
/// and hand text to the scanner `\SOUL@` (:131). The binding has no
/// character scanner, so the hooks are plain macros and `\SOUL@` sets its
/// argument as text (Perl: `\SOUL@setup` undefined; KPE #167).
#[test]
fn soul_scanner_surface_is_defined() {
  let tex = r"\documentclass{article}
\usepackage{soul}
\makeatletter
\begin{document}
\SOUL@setup\def\SOUL@preamble{}\SOUL@{highlighted text} \SOUL@ X \so{spaced}
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("highlighted text"), "{xml}");
  assert!(xml.contains("letter-spacing"), "{xml}");
}

/// latex.ltx:1832 `\g@addto@macro` appends at DIGESTION (its `\xdef`); an
/// expandable side-effecting version (Perl :968) was executed by the
/// `\ifnum` number scan's look-ahead (tex.web §444) even in a false branch
/// (numspell-english.sty:79-105 `\ifnum…>0\numspell@{ hundred}\fi`; KPE #170).
#[test]
fn g_addto_macro_appends_at_digestion() {
  let tex = r"\documentclass{article}
\makeatletter
\def\out{}%
\def\g{\ifnum0>0\g@addto@macro\out{WRONG}\else\g@addto@macro\out{RIGHT}\fi}%
\g
\g@addto@macro\out{+MORE}
\AtBeginDocument{\g@addto@macro\out{+ABD}}
\makeatother
\begin{document}
[\out]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[RIGHT+MORE+ABD]"), "{xml}");
}

/// tabularray parses its own body and tolerates a row wider than the
/// colspec (circularglyphs-doc.tex:196 `*{13}{X[m,c]}` with a 14-cell
/// row; pdflatex and Perl clean); the kernel template is only a cap, so
/// the tblr translation carries a margin of fallback columns. A plain
/// tabular keeps erroring on an extra `&`.
#[test]
fn tblr_row_wider_than_the_colspec_is_tolerated() {
  let tex = r"\documentclass{article}
\usepackage{tabularray}
\begin{document}
\begin{tblr}{colspec={*{2}{c}}}
a & b \\
Null & & \\
\end{tblr}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">Null<"), "{xml}");
  let tex = r"\documentclass{article}
\begin{document}
\begin{tabular}{cc} a & b & c \end{tabular}
\end{document}
";
  let (stderr, _xml) = convert(tex, false);
  assert!(
    error_count(&stderr) > 0,
    "a plain tabular's extra & stays an error:\n{stderr}"
  );
}

/// latex.ltx's `\nocite` writes `\citation{#1}` through
/// `\protected@write` at the call site, so a key held in a transient
/// macro is expanded there; the deferred raw key (Perl :4214) was expanded
/// at `\end{document}` when tufte-common.def:934's `\@for\@temp@bibkeyx`
/// loop variable no longer existed (tufte sample-book; KPE #171).
#[test]
fn nocite_expands_its_key_at_the_call_site() {
  let tex = r"\documentclass{article}
\makeatletter
\begin{document}
\def\keys{key1,key2}\marginpar{\@for\@temp@bibkeyx:=\keys\do{\nocite{\@temp@bibkeyx}}}
\nocite{*}
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"bibrefs="key1""#) && xml.contains(r#"bibrefs="key2""#),
    "{xml}"
  );
  assert!(xml.contains(r#"bibrefs="*""#), "{xml}");
}

/// report/book define `{titlepage}` with `\newenvironment`, so a class may
/// `\def\titlepage{…}` as a plain vertical macro (uwthesis.cls:610, used as
/// `{… \titlepage }`); the locked environment refused the `\def` and the
/// bare `\titlepage` opened an environment frame the `}` then met
/// (KPE #172). The environment itself still works.
#[test]
fn titlepage_environment_is_overridable() {
  let tex = r"\documentclass{report}
\makeatletter
\def\titlepage{\par TITLE STUFF\par}
\makeatother
\begin{document}
{\titlepage}After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("TITLE STUFF") && !xml.contains("<titlepage"),
    "{xml}"
  );
  let tex = r"\documentclass{report}
\begin{document}
\begin{titlepage}\title{T}\author{A}\maketitle\end{titlepage}
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<title>T</title>"), "{xml}");
}

/// The l3draw binding carries the full public surface; the path/state
/// functions are absorbed but `\draw_box_use:N`/`\draw_coffin_use:Nnn`
/// (l3draw.sty:40/:98) typeset their CONTENT (circledtext, tabular2,
/// suanpan-l3 under lualatex).
#[test]
fn l3draw_surface_keeps_box_content() {
  let tex = r"\documentclass{article}
\usepackage{l3draw}
\ExplSyntaxOn
\box_new:N \l_tmp_box \hbox_set:Nn \l_tmp_box { INSIDE-BOX }
\NewDocumentCommand \mydraw { } {
  \draw_begin:
    \draw_set_linewidth:n { 1pt }
    \draw_path_scope_begin: \draw_path_circle:nn {0pt,0pt}{5pt} \draw_path_scope_end:
    \draw_box_use:N \l_tmp_box
  \draw_end: }
\ExplSyntaxOff
\begin{document}Before \mydraw{} After\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Before INSIDE-BOX After"), "{xml}");
}

/// The babel language stubs are FALLBACKS for a missing `.ldf`: when the
/// real file is installed it is raw-loaded, so its `\DeclareOption
/// {mexico}` (spanish.ldf:66-88) and `\bbl@declare@ttribute{czech}{split}`
/// (czech.ldf:328) are honoured — the stub shadowed them ("Unknown option
/// 'mexico'", unamthesis; "attribute split", csbulletin).
#[test]
fn installed_ldf_outranks_the_language_stub() {
  // Host-portability: skip when the exercised package is absent from this
  // TeX Live tree (the behavior under test needs the real file).
  if !kpsewhich_has("spanish.ldf") || !kpsewhich_has("czech.ldf") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage[english,spanish,mexico]{babel}
\begin{document}
\selectlanguage{spanish}Hola \selectlanguage{english}Hello
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hola") && xml.contains("Hello"), "{xml}");
  let tex = r"\documentclass{article}
\usepackage[czech,english]{babel}
\languageattribute{czech}{split}
\begin{document}
\selectlanguage{czech}Ahoj
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Ahoj"), "{xml}");
}

/// Real soul resolves a stored color name through `\color` at use time,
/// which expands a macro-valued name (europasscv.cls:560 `\setulcolor
/// {\ecv@textcolor}`); the binding stored it unexpanded (Perl
/// soul.sty.ltxml:75 too; KPE #173). Same for `\setstcolor`/`\sethlcolor`.
#[test]
fn soul_color_setters_expand_a_macro_name() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{soul}
\definecolor{mycol}{HTML}{3E3A38}
\def\mycolname{mycol}
\begin{document}
\setulcolor{\mycolname}\ul{underlined text} \sethlcolor{\mycolname}\hl{hi}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"framecolor="#3E3A38""##), "{xml}");
  assert!(xml.contains(r##"backgroundcolor="#3E3A38""##), "{xml}");
}

/// nmbib.sty's `\citeall` (:343) runs natbib's low-level engine
/// (`\NAT@reset@parser`, natbib.sty:780) that the natbib binding — a
/// high-level `<ltx:cite>` emulation, like Perl's — does not carry (nmbib-
/// sample 22 errors); the binding emulates it as `\citet*`.
#[test]
fn nmbib_citeall_is_a_cite() {
  let tex = r"\documentclass{article}
\usepackage{nmbib}
\begin{document}
Text \citeall{Markey:Tame_the_BeaST} and \citealn{Markey:Tame_the_BeaST}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"bibrefs="Markey:Tame_the_BeaST""#), "{xml}");
}

/// latex.ltx:16585-16594's array/tabular row CONTINUATION macros carry the
/// closing half of `\@arraycr`'s `${` trick; reached directly (tablists.sty's
/// `\TeXr@arraycr` inside its own raw `\halign`) the `$` had no partner and
/// opened inline math the row's `\cr` could not balance (tablists-rus 101;
/// Perl 12; KPE #174).
#[test]
fn array_continuation_macros_carry_no_math_shift() {
  let tex = r"\documentclass{article}\usepackage{tablists}
\begin{document}
\begin{tabenum}[\bfseries1)]
\tabenumitem aa;\\
\tabenumitem bb;\\[2pt]
\tabenumitem $c$;
\end{tabenum}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("aa") && xml.contains("bb"), "{xml}");
  assert_eq!(xml.matches("<Math ").count(), 1, "{xml}");
}

/// A plain content `.tex` re-`\InputIfFileExists`ed while a `.sty` is being
/// read is re-read every time (TeX; Perl `Input`); the once-only package
/// guard skipped the second read, so babel's second `babel-french.tex` scan
/// (french as BOTH class option and `main=`) never recorded french and
/// french.ldf (→ `\og`, `\ieme`) never loaded (paresse-fra; KPE #175).
#[test]
fn content_tex_reinput_during_definitions_rereads() {
  let tex = r"\documentclass{article}
\usepackage{reinstyx}
\begin{document}
[\afterone][\aftertwo]
\end{document}
";
  let (stderr, xml) = convert_with_files(tex, &[
    ("helperx.tex", "\\def\\hmarker{SET}\\endinput\n"),
    (
      "reinstyx.sty",
      "\\ProvidesPackage{reinstyx}\n\\def\\hmarker{INIT}\\InputIfFileExists{helperx.tex}{}{}\\edef\\afterone{\\hmarker}%\n\\def\\hmarker{RESET}\\InputIfFileExists{helperx.tex}{}{}\\edef\\aftertwo{\\hmarker}%\n",
    ),
  ]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[SET][SET]"), "{xml}");
  let tex = r"\documentclass[french]{article}
\usepackage[english,main=french]{babel}
\begin{document}
\og guillemets\fg{} 1\ier{} 2\ieme
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("«") || xml.contains("guillemets"), "{xml}");
}

/// pdfmanagement's l3pdffile/l3pdfdict user surface (`\pdffile_embed_file:nnn`
/// pdfmanagement.ltx:3389, `\pdfdict_put:nnn`) builds PDF/A associated-file
/// objects nothing reads back (tagpdf's ex-AF-file.tex:29-32) — absorbed.
#[test]
fn pdffile_and_pdfdict_are_absorbed() {
  let tex = r"\DocumentMetadata{tagging=on,pdfversion=2.0,lang=de}
\documentclass{article}
\ExplSyntaxOn
\pdffile_embed_file:nnn{t.tex}{}{tag/AFtest}
\pdfdict_put:nnn {l_pdffile/Filespec} {AFRelationship}{/Supplement}
\ExplSyntaxOff
\begin{document}AF done\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("AF done"), "{xml}");
}

/// nicematrix's `\CodeAfter` grab must be environment-balanced: a
/// `\begin{tikzpicture}…\end{tikzpicture}` inside it has its own `\end`
/// (nicematrix-french: 23 stray `\endgroup`s + leaked pgf node errors).
#[test]
fn nicematrix_codeafter_grab_is_environment_balanced() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\usetikzlibrary{fit}
\begin{document}
\[\begin{pNiceMatrix}
121 & 23 & 345 \\ 45 & 346 & 863 \\ 3462 & 38458 & 34
\CodeAfter
\SubMatrix\{{2-2}{3-3}\}[name=A]
\begin{tikzpicture}
\node [fit = (A),fill = red!15] {} ;
\end{tikzpicture}
\end{pNiceMatrix}\]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("38458"), "{xml}");
}

/// The executed `\CodeBefore` block keeps its color commands but drops the
/// drawing `tikzpicture`/`scope` overlays that reference cell nodes LaTeXML
/// never materializes (`create-cell-nodes`, nicematrix-french ×280).
#[test]
fn nicematrix_codebefore_drops_drawing_environments() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\usetikzlibrary{fit}
\begin{document}
\[\begin{pNiceMatrix}
\CodeBefore [create-cell-nodes]
\cellcolor{red}{1-1}
\begin{tikzpicture}
\node [fit = (2-2), fill=red!15] {} ;
\end{tikzpicture}
\Body
a & a + b \\ a & a
\end{pNiceMatrix}\]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("backgroundcolor="), "{xml}");
}

/// Under `ampersand-in-blocks` a `\Block` body holding `&` is a sub-grid
/// (nicematrix.sty:7592 `\__nicematrix_Block_vii`, a tabular in text / an
/// array in math split on `&`); emitting it bare re-exposed the `&` to the
/// outer alignment (nicematrix.tex:1152; "Extra alignment tab").
#[test]
fn nicematrix_block_ampersand_body_is_a_subgrid() {
  let tex = r"\documentclass{article}
\usepackage[ampersand-in-blocks]{nicematrix}
\begin{document}
\begin{NiceTabular}{ll}
\Block{}{one & two & three} & x \\
a & b
\end{NiceTabular}
$\begin{pNiceMatrix}
\Block{}{1 & 2} & c \\ d & e
\end{pNiceMatrix}$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(">three<"), "{xml}");
  assert!(xml.matches("<tabular").count() >= 2, "{xml}");
}

/// A raw class redefining a locked frontmatter command as a plain setter
/// (afthesis.cls:520 `\def\author#1{\def\auth@r{#1}}`) is dropped
/// (Perl State.pm:502-517), and its readers then fail on the internal it
/// would have defined (`\flyleaf`/`\titlepage` :637/:688; usethesis). The
/// dropped body's `\def`-targets are defined EMPTY, the class's own default
/// convention (:494-495), so the locked binding stays the single source.
#[test]
fn locked_setter_internals_are_defined_empty() {
  let tex = r"\documentclass{article}
\makeatletter
\def\author#1{\def\auth@r{#1}\gdef\auth@rtwo{#1}}
\makeatother
\author{First Author}
\begin{document}
\makeatletter[\auth@r][\auth@rtwo]\makeatother\maketitle
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[][]"), "{xml}");
  assert!(xml.contains("First Author"), "{xml}");
}

/// xcolor.sty:762-763 `\color` = `\@ifnextchar[\@undeclaredcolor\@declaredcolor`;
/// fancyqr.sty:20-22 calls the named-color branch directly. Both engines
/// bind `\color` monolithically and lacked the branches (KPE #177).
#[test]
fn color_switch_branches_are_defined() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\definecolor{tl}{HTML}{FF0000}\definecolor{br}{HTML}{3D3A38}
\begin{document}
\makeatletter{\@declaredcolor{tl!50!br}Hello} {\@undeclaredcolor[rgb]{0,0,1}Blue}\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r##"color="#9E1D1C">Hello"##), "{xml}");
  assert!(xml.contains(r##"color="#0000FF">Blue"##), "{xml}");
}

/// A raw biblatex style `.def` (biblatex-sbl.def:663) replaces
/// `\printbibliography` with biblatex's real body, which reaches the
/// `\blx@key@bibcheck` / `\blx@printbibliography` internals the binding
/// stands in for (biblatex.sty:9643/:9820). Witness biblatex-sbl/sbl-paper.
#[test]
fn style_def_printbibliography_override_routes_to_binding() {
  let tex = r"\documentclass{article}
\usepackage[style=sbl,backend=biber]{biblatex}
\begin{document}
Text.
\printbibliography[heading=bibintoc]
\end{document}
";
  let (stderr, _xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// The siunitx `S`/`s` cell is read with expansion under LaTeX's
/// `\protected@edef` context (`\protect` = `\@unexpandable@protect`,
/// latex.ltx:1384): a raw class's size command then stays `\protect\small `
/// instead of expanding — `\@setfontsize` (latex.ltx:14103) reaches
/// `\@currsize` → `\normalsize` → itself under `\@typeset@protect`, the same
/// overflow as pdflatex's `\edef\x{\small}`. The cell is emitted as ONE
/// GROUP (LaTeX's column template wraps every entry in `{…}`) so the size
/// stays scoped to the cell. Witness zugferd-invoice.sty:113 `\small\emph
/// {Pos.}&…` in an `S` column under scrartcl (`PushbackLimit`; pdflatex
/// clean). The number still parses; under article the size is applied.
#[test]
fn s_column_unbraced_size_command_is_scoped() {
  let tex = r"\documentclass{scrartcl}
\usepackage{siunitx}
\begin{document}
\begin{tabular}{S}
\small a \\
1.5 \\
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<Math mode="inline" tex="1.5""#), "{xml}");
  let article = tex.replace("scrartcl", "article");
  let (stderr, xml) = convert(&article, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<text fontsize="90%">a</text>"#), "{xml}");
  assert!(xml.contains(r#"<Math mode="inline" tex="1.5""#), "{xml}");
}

/// A pure size switch is LaTeX's `\@setfontsize` (latex.ltx:14103), whose
/// first act is `\let\@currsize#1`; packages test the identity with
/// `\ifx\@currsize\small` (ltugboat's `\SMC` cascade in
/// latex-doc-ptr.sty:203-215, else `\TBWarning`). With the class
/// binding's primitive alone `\@currsize` never matched any size.
#[test]
fn size_switch_lets_currsize() {
  let tex = r"\documentclass{article}
\makeatletter
\DeclareRobustCommand{\SMC}{\ifx\@currsize\normalsize\small\else
 \ifx\@currsize\small\footnotesize\else
  \ifx\@currsize\large\normalsize\else NOSIZE\fi\fi\fi}
\makeatother
\begin{document}
{\small A\SMC B}
{\large C\SMC D}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("NOSIZE"), "{xml}");
  // `\small`→`\footnotesize` (B), `\large`→`\normalsize` (D, base size,
  // no wrapper).
  assert!(xml.contains(r#"<text fontsize="89%">B</text>"#), "{xml}");
  assert!(
    xml.contains(r#"<text fontsize="120%">C</text>D</p>"#),
    "{xml}"
  );
}

/// caption3.sty:1595 `\providecommand*\caption@prepareslc{}` is an empty
/// hook other packages extend (hep-bibliography.sty:108; 9 hep-* docs).
#[test]
fn caption_prepareslc_hook_is_defined() {
  let tex = r"\documentclass{article}
\usepackage{caption}
\makeatletter
\g@addto@macro\caption@prepareslc{\relax}
\makeatother
\begin{document}
Hello.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hello."), "{xml}");
}

/// titlesec.sty:1039-1041 `\newdimen\titlewidth…` (titlesec.tex:1780).
#[test]
fn titlesec_title_width_registers_exist() {
  let tex = r"\documentclass{article}
\usepackage{titlesec}
\titleformat{\section}[block]
  {\addtolength{\titlewidth}{2pc}\normalfont\sffamily}
  {\thesection}{1em}{}
\begin{document}
\section{Hello}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<title font=\"sansserif\">") && xml.contains("Hello</title>"),
    "{xml}"
  );
}

/// ntheorem.sty:714-715 `\newskip\thm@topsep`/`\thm@topsepadd`
/// (dlfltxbcodetips.sty:102-106 copies ntheorem's code).
#[test]
fn ntheorem_topsep_registers_exist() {
  let tex = r"\documentclass{article}
\usepackage{amsmath,amssymb}
\usepackage[amsmath,thmmarks,framed]{ntheorem}
\makeatletter
\thm@topsepadd \theorempostskipamount
\advance\thm@topsepadd\partopsep
\makeatother
\begin{document}
Hi.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hi."), "{xml}");
}

/// mathtools.sty:1897-1907 `\xmathstrut` is a `\vphantom` strut
/// (numerica.tex:3431 inside `\eval{\[\frac…\]}`).
#[test]
fn xmathstrut_is_a_vphantom() {
  let tex = r"\documentclass{article}
\usepackage{mathtools}
\begin{document}
\[ \frac{\xmathstrut{0.1} a}{\xmathstrut{0.4} b} \]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<XMApp"#) && xml.contains("phantom"),
    "{xml}"
  );
}

/// isorot's raw `\@xrotfloat` (isorot.sty:139-147) builds the sideways
/// float as an lrbox + minipage capture, inside which `\caption`'s float-up
/// finds no float ("`<ltx:caption>` isn't allowed in `<ltx:block>`";
/// isorot/rotman, Perl identical, pdflatex clean). The binding gives the
/// float environments rotating's shape, so the caption is the float's child.
#[test]
fn isorot_sideways_float_holds_its_caption() {
  let tex = r"\documentclass{article}
\usepackage{isorot}
\begin{document}
\begin{sidewaystable}
\centering
\caption{The rotation facilities}
\begin{tabular}{|l|l|}\hline A & B \\\hline\end{tabular}
\end{sidewaystable}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<table") && xml.contains("<caption class=\"ltx_centering\">"),
    "{xml}"
  );
  assert!(!xml.contains("<block>"), "{xml}");
}

/// adjmulticol.sty:151 raw-calls multicol.sty:172 `\mult@@cols`, the
/// column balancer LaTeXML never emulates; bound, `adjmulticols` emits the
/// same pagination markers as `multicols` (adjmulticol/sample).
#[test]
fn adjmulticols_are_pagination_markers() {
  let tex = r"\documentclass{book}
\usepackage{adjmulticol}
\begin{document}
\begin{adjmulticols}{2}{12pt}{-2in}
Some text flowing across two adjusted columns. More text here.
\end{adjmulticols}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<pagination role="start_2_columns"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<pagination role="end_2_columns"/>"#),
    "{xml}"
  );
  assert!(xml.contains("Some text flowing"), "{xml}");
}

/// Raw biblatex style files reach the biblatex.sty internal/public surface
/// at cite/bibliography time (windycity data-model declarations, sbl's
/// `\citeshorthand` control flow, juradiss' `\AtDataInput`); the binding
/// stands in for biblatex.sty and carries that surface.
#[test]
fn biblatex_style_internal_surface() {
  for (style, body) in [
    ("windycity", r"Text.\par \printbibliography"),
    ("sbl", r"See \citeshorthand{SBL} and \cite{SBLHS}."),
    ("biblatex-juradiss", r"Text.\par \printbibliography"),
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage[style={style}]{{biblatex}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert(&tex, false);
    assert_eq!(error_count(&stderr), 0, "{style}: {stderr}");
    assert!(
      xml.contains("Text.") || xml.contains("<cite class="),
      "{style}: {xml}"
    );
  }
}

/// hyperref's low-level URL chain (`\hyper@normalise` :4604 → `\url@`
/// :4802 → `\hyper@linkurl`/`\Hurl`) reached by biblatex.tex's `\fnurl`;
/// the neutralised read keeps `#`/`%`/`~`.
#[test]
fn hyperref_normalise_chain_links_urls() {
  let tex = r"\documentclass{article}
\usepackage{hyperref}
\makeatletter
\newcommand\fnurl@[1]{\footnote{\url@{#1}}}
\DeclareRobustCommand\fnurl{\hyper@normalise\fnurl@}
\makeatother
\begin{document}
See the docs.\fnurl{https://ctan.org/pkg/biblatex#frag~x}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r##"href="https://ctan.org/pkg/biblatex#frag~x""##),
    "{xml}"
  );
  assert!(xml.contains("biblatex#frag~x</ref>"), "{xml}");
}

/// longtable.sty:135-137 redefine `\newpage`/`\pagebreak`/`\nopagebreak`
/// inside the table to `\noalign{…}`, so tex.web §785 `align_peek` takes
/// the no_align branch instead of starting a row (harmony: `\newpage`
/// between `\hline` rows; `\clearpage` is NOT redefined and errors in
/// pdflatex too).
#[test]
fn longtable_page_commands_between_rows_are_noalign() {
  for cmd in [r"\newpage", r"\nopagebreak", r"\pagebreak[2]"] {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage{{longtable}}\n\\begin{{document}}\n\\begin{{longtable}}{{ll}}\n\\hline a & b \\\\ \\hline\n{cmd}\n\\hline c & d \\\\ \\hline\n\\end{{longtable}}\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert(&tex, false);
    assert_eq!(error_count(&stderr), 0, "{cmd}: {stderr}");
    assert_eq!(xml.matches("<td").count(), 4, "{cmd}: {xml}");
  }
}

/// threeparttable.sty:110 (`\def\@captype{table}` if undefined) and :126
/// (measuredfigure → `figure`) let `\caption` work outside a float; both
/// bindings bound a bare `#body` and dropped it (threeparttablex;
/// PERL-ORIGIN, threeparttable.sty.ltxml:31,36).
#[test]
fn threeparttable_sets_captype_outside_a_float() {
  let tex = r"\documentclass{article}
\usepackage{threeparttable}
\begin{document}
\begin{threeparttable}
\caption{A table}
\begin{tabular}{l} a \\ \end{tabular}
\end{threeparttable}
\begin{measuredfigure}
\caption{A figure}
\end{measuredfigure}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // `\@captype` is defined, so no "outside any known float" error; outside a
  // float each caption is its type's float (batch 56gs, guard
  // `caption_outside_a_float_becomes_its_float`), the tabular kept beside it.
  let gaps = regex::Regex::new(r">\s+<").unwrap();
  let flat = gaps.replace_all(&xml, "><").into_owned();
  assert!(
    flat.contains(concat!(
      r#"<table inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag>"#,
      r#"<tag role="typerefnum">Table 1</tag></tags><caption><tag close=": ">Table 1</tag>A table</caption></table>"#,
      r#"<para xml:id="p1"><tabular vattach="middle">"#
    )),
    "{flat}"
  );
  assert!(
    flat.contains(r#"<caption><tag close=": ">Figure 1</tag>A figure</caption></figure>"#),
    "{flat}"
  );
}

/// A block listing in a `p{}` cell: the listing's group must close with
/// an implicit `\egroup` (tex.web §347: only `{`/`}` characters move
/// `align_state`), else the cell's `&`/`\\` stop being column ends
/// (pfdicons-doc, tikzcodeblocks-documentation, shipunov; pdflatex clean).
/// The `\parbox` form errors in pdflatex too and stays an error.
#[test]
fn block_listing_in_a_paragraph_cell() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\begin{document}
\begin{tabular}{p{4cm}l}
\begin{lstlisting}[numbers=none]
x=1;
\end{lstlisting} & b \\
c & d \\
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 4, "{xml}");
  assert!(xml.contains("<listing"), "{xml}");
  let control = tex
    .replace(
      r"\begin{tabular}{p{4cm}l}",
      r"\begin{tabular}{ll}\parbox{4cm}{",
    )
    .replace(r"\end{lstlisting} & b", r"\end{lstlisting}} & b");
  let (stderr, _xml) = convert(&control, false);
  assert!(
    error_count(&stderr) > 0,
    "CONTROL: pdflatex errors here too\n{stderr}"
  );
}

/// latex.ltx `\@tabarray` = `\m@th\@ifnextchar[\@array{\@array[c]}` — the
/// full array setup; a package building its own array on it (t-angles.sty:491)
/// nested in an outer array cell under `\begingroup` broke the outer cell's
/// group (t-angles/t-manual, 101 errors; Perl identical, pdflatex clean).
#[test]
fn tabarray_is_the_full_array_setup() {
  let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{t-angles}
\def\SHOW#1#2{\begin{array}{c}\begin{tangle}#1\end{tangle}\\ \hbox{\tt\string#2}\end{array}}
\def\Show#1{\SHOW#1#1}
\begin{document}
$$ \Show\id \quad \Show\n $$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.matches("<XMArray").count() >= 2, "{xml}");
}

/// colortbl's `\CT@*` internal surface for raw derivatives (tabu.sty:720
/// assigns to and `\the`s `\CT@everycr`, colortbl.sty:116 `\let…\everycr`).
#[test]
fn colortbl_internal_surface_is_defined() {
  let tex = r"\documentclass{article}\usepackage{colortbl}
\makeatletter
\CT@everycr\expandafter{\expandafter\relax\the\CT@everycr}
\CT@arc@\CT@column@color\CT@row@color\CT@cell@color\CT@do@color
\makeatother
\begin{document}
x
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<p>x</p>"), "{xml}");
}

/// A box capture (`insert_block`'s `ltx:_CaptureBlock_`) is a completed
/// box: its non-auto-closeable descendants (a `verbatim`, listing lines)
/// are closed by the box, not reported (testnumberedblock; Perl emitted
/// the same spurious error over the same tree).
#[test]
fn capture_box_closes_its_descendants() {
  let tex = r"\documentclass{article}
\usepackage{numberedblock}
\begin{document}
\begin{numVblock}
This is a labeled numVblock
program test
\end{numVblock}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("program test"), "{xml}");
  assert!(!xml.contains("_CaptureBlock_"), "{xml}");
}

/// physics2 `ab.braket`: the active `|` in `\braket<a|b>` is a
/// `\middle\vert` without the `\egroup…\bgroup` atom split that
/// LaTeXML's token-level `\left` capture cannot pair (physics2,
/// physics2-legacy; lualatex clean).
#[test]
fn physics2_braket_active_bar_is_a_middle_fence() {
  let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{physics2}
\usephysicsmodule{ab,ab.braket}
\begin{document}
\[ \bra<\phi| \quad \ket|\psi> \quad \braket<\phi> \]
\[ \braket<\phi|\psi> \quad \braket<\phi|A|\psi> \quad \ketbra|\phi><\psi| \]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"role="MIDDLE""#) || xml.contains("∣"),
    "{xml}"
  );
}

/// physics2 + unicode-math: `physics2`'s `\vert` is redefined via `\Udelimiter`
/// when `unicode-math` defines `\symrm`. Multi-dot dispatch loads
/// `phy-ab.braket_sty.rs` and `\Udelimiter` constructor avoids mathcode 8000
/// recursion on `\middle\vert` (witness: egroup_braket_physics2.tex).
#[test]
fn physics2_braket_with_unicode_math_delimiters() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}\usepackage{unicode-math}\usepackage{physics2}
\usephysicsmodule{ab,ab.braket}
\begin{document}
\[ \braket< a | b > \]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"role="MIDDLE""#), "{xml}");
}

/// physics2 `\delopen` and `\delclose` paired with active pipe inside
/// `\bgroup`..`\egroup` delimited math (witness: egroup_delopen_activepipe_reduced.tex).
#[test]
fn physics2_delopen_delclose_active_pipe_reduced() {
  let tex = r#"\documentclass{article}
\usepackage{amsmath}\usepackage{unicode-math}\usepackage{physics2}
\begingroup\catcode`\|=\active
\gdef\mytest{\begingroup\mathcode`\|="8000\def|{\egroup\vert\bgroup}%
  \delopen\langle\bgroup a|b\egroup\delclose\rangle\endgroup}
\endgroup
\begin{document}
\[ \mytest \]
\end{document}
"#;
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<XMWrap"), "{xml}");
}

/// tex.web §1206: a `\noalign` body is EXECUTED to the `}` closing its
/// group; latex.ltx's `\hline` brace hack (`\noalign{\ifnum0=`}\fi…`) has a
/// char-constant `}` a token pre-scan miscounted, leaking the rule into the
/// alignment (boldline `\hlineB`, shipunov/boldline-ex-en; Perl identical).
#[test]
fn noalign_body_is_executed_to_its_group_end() {
  let tex = r"\documentclass{article}\usepackage{array}
\makeatletter
\def\myhline{\noalign{\ifnum0=`}\fi\hrule \@height \arrayrulewidth \futurelet\reserved@a\@xmyhline}
\def\@xmyhline{\ifx\reserved@a\myhline\fi\ifnum0=`{\fi}}
\makeatother
\begin{document}
\begin{tabular}{c}a\\\myhline b\\\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // the raw full-width `\hrule \@height…` is the next row's top border
  assert!(
    xml.contains(">a") && xml.contains(">b") && xml.contains(r#"border="t""#),
    "{xml}"
  );
  let boldline = r"\documentclass{article}\usepackage{boldline}
\begin{document}
\begin{tabular}{cc}\hlineB{2.5} a & b \\ \hlineB{2.5} c & d \\ \hlineB{2.5}\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(boldline, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for cell in [">a<", ">b<", ">c<", ">d<"] {
    assert!(xml.contains(cell), "{cell}: {xml}");
  }
}

/// tabu's remaining user surface: `\everyrow`, `\rowfont`, and the
/// `\extrarowsep` assignment syntax (tabu.sty:232) over
/// `\extrarowheight`/`\extrarowdepth`.
#[test]
fn tabu_row_surface_is_covered() {
  let tex = r"\documentclass{article}\usepackage{tabu}
\begin{document}
\extrarowsep=2pt \extrarowsep^=3pt \extrarowsep=^1pt_2pt
\everyrow{\hline}
\begin{tabu}{ll}\rowfont[c]{\bfseries} a & b \\ c & d \\\end{tabu}
\the\extrarowheight/\the\extrarowdepth
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 4, "{xml}");
  assert!(xml.contains("1.0pt/2.0pt"), "{xml}");
}

/// latex.ltx:10005 `\@tabacckludge`: inside tabbing `\a=`/`\a<`/`\a>` reach
/// the encoding-level accents although `\=`/`\<`/`\>` are tab operators,
/// and an accent tabbing never rebinds (`\a"`) is the accent itself
/// (encguide, greek-fontenc; Perl saved only `'` and `` ` ``).
#[test]
fn tabbing_accent_kludge_recovers_rebound_accents() {
  let tex = r#"\documentclass{article}
\begin{document}
\begin{tabbing}
xxx \= yyy \\
\a=o \> \a'e \a"u \\
\end{tabbing}
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("ō") && xml.contains("é") && xml.contains("ü"),
    "{xml}"
  );
}

/// beamerbasetemplates.sty:26 `\ifbeamertemplateempty` gates theme code on
/// whether a template is set (beamerthemeAlbi; 43-error `\fi` cascade).
#[test]
fn beamer_template_empty_test_is_defined() {
  let tex = r"\documentclass{beamer}
\makeatletter
\ifbeamertemplateempty{logo}{EMPTY}{NONEMPTY}
\makeatother
\begin{document}
\begin{frame}Hi\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("EMPTY") && !xml.contains("NONEMPTY"), "{xml}");
}

/// amsart's raw `\maketitle` internals reached by a derivative class that
/// redefines `\maketitle` over `\LoadClass{amsart}` (resphilosophica).
#[test]
fn amsart_maketitle_internals_are_defined() {
  let tex = r"\documentclass{resphilosophica}
\author{Alice}
\title{T}
\dedicatory{For X}
\begin{document}
\begin{abstract}Abs.\end{abstract}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<creator") && xml.contains("<abstract"),
    "{xml}"
  );
}

/// quantumview's `\renewcommand{\author}` cannot override the locked
/// kernel `\author`, so its author-group list init never runs and the raw
/// `\maketitle` loop meets an undefined `\@authorgroup`; the class
/// binding initialises the lists (creators still captured).
#[test]
fn quantumview_author_group_lists_are_initialised() {
  let tex = r"\documentclass{quantumview}
\title{T}
\author{Alice}
\affiliation{Somewhere}
\begin{document}
\maketitle
Body.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<creator"), "{xml}");
}

/// A full-width `\hrule` with an explicit height inside `\noalign` is a
/// horizontal rule → the next row's top border (it was silently dropped).
#[test]
fn noalign_rule_with_height_is_a_border() {
  let tex = r"\documentclass{article}
\begin{document}
\begin{tabular}{ll}
\noalign{\hrule height 1pt}
a & b \\
c & d \\
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 4, "{xml}");
  assert!(xml.contains(r#"border="t""#), "{xml}");
}

/// gauss `gmatrix`: the amsmath matrix its delimiter names plus the
/// row/column operations as a math annotation (raw gauss measures the box
/// with a `\lastbox` recursion whose termination is a physical width).
#[test]
fn gauss_gmatrix_renders_with_operation_lines() {
  let tex = r"\documentclass{article}\usepackage{amsmath}\usepackage{gauss}
\begin{document}
\[ \begin{gmatrix}[p] 1 & 2 \\ 3 & 4
\rowops \mult{0}{\cdot 2} \add[3]{0}{1} \swap{0}{1}
\end{gmatrix} \]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<XMArray"), "{xml}");
  assert!(
    xml.contains("←") || xml.contains("&#8592;") || xml.contains("leftarrow"),
    "{xml}"
  );
}

/// latex.ltx `\marginpar` is a macro (`\@ifnextchar[\@xmpar\@ympar`); a
/// package prepending to it by expansion (marginfix.sty:91) must capture
/// its body, not the bare token.
#[test]
fn marginpar_is_a_macro_over_its_constructor() {
  let tex = r"\documentclass{article}
\makeatletter
\edef\marginpar{\unexpanded{\typeout{pre}}\expandafter\unexpanded\expandafter{\marginpar}}
\makeatother
\begin{document}
Text\marginpar{Note}\marginpar[L]{R} more.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // `\marginpar[L]{R}` yields a left and a right note
  assert_eq!(xml.matches(r#"role="margin""#).count(), 3, "{xml}");
  assert!(xml.contains("ltx_marginpar_left"), "{xml}");
}

/// subfiles.sty:171 `\ifSubfilesClassLoaded{yes}{no}` (sshrc-insight).
#[test]
fn subfiles_class_loaded_test_is_defined() {
  let tex = r"\documentclass{article}\usepackage{subfiles}
\begin{document}
\ifSubfilesClassLoaded{CLASS}{PACKAGE}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("PACKAGE") && !xml.contains("CLASS"), "{xml}");
}

/// xparse `O{}` options nest brackets: nicematrix.tex:1364
/// `[rules/color=[gray]{0.9},…]` was cut at the inner `]`, spilling the rest
/// into the table (16 "Extra alignment tab" cascades in nicematrix).
#[test]
fn optional_balanced_nests_brackets() {
  let tex = r"\documentclass{article}\usepackage{nicematrix,xcolor}
\begin{document}
\begin{NiceTabular}{|ccc|}[rules/color=[gray]{0.9},rules/width=1pt,no-cell-nodes]
\hline
a & b & c \\
\hline
\end{NiceTabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 3, "{xml}");
}

/// fontspec's `\IfFontExistsTF` is a texmf-tree lookup, not a constant
/// false (asmeconf's class-level font checks under the luatex profile).
#[test]
fn font_exists_test_consults_the_texmf_tree() {
  let tex = r"\documentclass{article}\usepackage{fontspec}
\begin{document}
\IfFontExistsTF{lmroman10-regular.otf}{YES1}{NO1}
\IfFontExistsTF{nonsense-font-xyz.otf}{YES2}{NO2}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("YES1") && xml.contains("NO2"), "{xml}");
}

/// Singleton internals reached by raw packages over the standing-in
/// bindings: hyperref's driver sentinel (hrefhide.sty:154), doclicense's
/// layout wrapper (beautynote), listings' `\lst@XConvert` consumer.
#[test]
fn singleton_internal_surface() {
  // doclicense needs its type/modifier/version options (the real package errors
  // without them, as pdflatex does; the former stub accepted anything).
  let tex = r"\documentclass{article}\usepackage{hyperref}\usepackage[type={CC},modifier={by},version={4.0}]{doclicense}\usepackage{listings}
\makeatletter
\def\hrefhide@driver{hpdftex}
\begin{document}
\ifx\Hy@driver\hrefhide@driver DRIVER-OK\fi
\lst@XConvert{abc}\@nil
\doclicenseThis
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Default mode: raw doclicense requires ccicons, which has no binding and is
  // not raw-loaded there — one honest missing_file warning, nothing else.
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("Warning:missing_file:ccicons"), "{stderr}");
  assert!(xml.contains("DRIVER-OK"), "{xml}");
}

/// `\DeclareMathVersion{name}` registers a version `\mathversion{name}`
/// may select (oz, askmaps, iwonamath, zed); an undeclared one still errors.
#[test]
fn declared_math_versions_are_selectable() {
  let tex = r"\documentclass{article}
\makeatletter
\DeclareMathVersion{oz}
\makeatother
\begin{document}
\mathversion{oz}$x=1$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<Math"), "{xml}");
  let (stderr, _xml) = convert(&tex.replace(r"\DeclareMathVersion{oz}", ""), false);
  assert_eq!(
    error_count(&stderr),
    1,
    "CONTROL: undeclared version errors\n{stderr}"
  );
}

/// array.sty's `\@mkpream` templates the cell as `\@sharp` (a cs `\let` to
/// `#`); a package-assembled `\ialign` (sgame, tabularcalc, tabvar) is a real
/// alignment once the raw `\halign` reader recognises the meaning (tex.web
/// §783). A `\noalign` outside any alignment still errors.
#[test]
fn ialign_template_accepts_the_sharp_placeholder() {
  let tex = r"\documentclass{article}\makeatletter
\begin{document}
\let\@sharp=#
\ialign{\hfil\@sharp\hfil&&\hfil\@sharp\hfil\cr a&b\cr c&d\cr}
\makeatother\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<td").count(), 4, "{xml}");
  let control = r"\documentclass{article}\begin{document}
x \noalign{\hrule} y
\end{document}
";
  let (stderr, _xml) = convert(control, false);
  assert!(
    error_count(&stderr) >= 1,
    "CONTROL: \\noalign outside an alignment errors\n{stderr}"
  );
}

/// `\usetheme[opts]{name}` passes its options to the theme as package
/// options and `\ProcessOptionsBeamer` applies them (beamerbasethemes.sty:
/// 18, beamerbaseoptions.sty:15): Verona's `sidebar` option installs the
/// real `\sidegraphics` instead of its "defined only with the 'sidebar'
/// option" stub. A theme without options (Albi) loads as before.
#[test]
fn usetheme_options_reach_the_theme() {
  let tex = r"\documentclass{beamer}
\usetheme[sidebar]{Verona}
\begin{document}
\begin{frame}\sidegraphics<1>{plato}{scale=1.1}\end{frame}
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  assert!(
    !stderr.contains("defined only with the 'sidebar' option"),
    "{stderr}"
  );
  let albi = r"\documentclass{beamer}\usetheme{Albi}\begin{document}\begin{frame}Hi\end{frame}\end{document}
";
  let (stderr, xml) = convert(albi, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hi"), "{xml}");
}

/// tex.web §211: `\ifinner` is the box/inline-math interior sign — false
/// at the main galley (paracol.sty:1996 `\ifinner\@parmoderr`; tidyres),
/// true inside `\parbox`/`$…$`, false in display math.
#[test]
fn ifinner_is_the_box_frame_sign() {
  let tex = r"\documentclass{article}\usepackage{paracol}
\begin{document}
\par\ifinner INNER1\else OUTER1\fi
{\par\ifinner INNER2\else OUTER2\fi}
\parbox{5cm}{\par\ifinner INNER3\else OUTER3\fi}
$\ifinner I4\else O4\fi$ \[\ifinner I5\else O5\fi\]
\begin{paracol}{2}Left.\switchcolumn Right.\end{paracol}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  for want in ["OUTER1", "OUTER2", "INNER3", "I4", "O5"] {
    assert!(xml.contains(want), "{want}: {xml}");
  }
}

/// Under the `[luatex]` profile pgf takes its LuaTeX branch (keyed on
/// `\directlua`) and expects Lua to define `\pgfutil@luaescapestring`; the
/// binding supplies pgf's own TeX fallback (neoschool, beamerthemeCelestia).
#[test]
fn pgf_lua_entry_points_have_their_tex_fallback() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{graphdrawing,graphs}
\usegdlibrary{trees}
\begin{document}
\tikz \graph[tree layout] { a -> {b, c} };
\end{document}
";
  let (stderr, xml) = convert_with(tex, Some("[rawstyles,rawclasses,luatex]latexml.sty"));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg:svg") || xml.contains("<svg"), "{xml}");
}

/// A float inside a Block container escapes to the enclosing `ltx:para`
/// (the `^` float-up marker), as LaTeX floats escape their environment
/// (isorot/rotman, bashful; Perl placed it in the quote).
#[test]
fn floats_escape_block_containers() {
  let tex = r"\documentclass{article}
\begin{document}
\begin{quote}
\begin{figure}
\caption{X}
\end{figure}
\end{quote}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<figure") && !xml.contains("<quote>\n    <figure"),
    "{xml}"
  );
  let plain = r"\documentclass{article}\begin{document}
Text.
\begin{figure}\caption{Y}\end{figure}
\end{document}
";
  let (stderr, xml) = convert(plain, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<figure"), "{xml}");
  // float.sty custom floats (bashful's `program`) go the same way.
  let custom = r"\documentclass{article}\usepackage{float}
\newfloat{program}{tbp}{lop}
\begin{document}
\begin{itemize}\item
\begin{program}\caption{Z}\end{program}
\end{itemize}
\end{document}
";
  let (stderr, xml) = convert(custom, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<float") && !xml.contains("<item>\n      <float"),
    "{xml}"
  );
}

/// achemso.cls:1022-1030 declares `scheme`/`chart`/`graph` floats through
/// float.sty; the binding must too, or `\caption` inside `scheme` cascades
/// (achemso-demo; RUST-ONLY, Perl raw-loads the class).
#[test]
fn achemso_declares_its_scheme_floats() {
  let tex = r"\documentclass{achemso}
\author{A}\title{T}
\begin{document}
\begin{scheme}
\caption{An example scheme}
\end{scheme}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("ltx_float_scheme") && xml.contains("<caption>"),
    "{xml}"
  );
}

/// Stub class bindings must issue the float-package requires of the real
/// class: jmlr.cls:155 algorithm2e, oup.cls:137 rotating (pmlr-sample,
/// oup-authoring-template; RUST-ONLY, Perl raw-loads both). jmlr's
/// `\floatconts` keeps its caption (jmlrutils.sty:166).
#[test]
fn class_stubs_require_their_float_packages() {
  let jmlr = r"\documentclass[pmlr]{jmlr}
\title{T}\author{\Name{A}}
\begin{document}
\begin{algorithm2e}
\caption{Computing Net Activation}
\end{algorithm2e}
\begin{table}\floatconts{tab:a}{\caption{Cap A}}{\begin{tabular}{l}x\end{tabular}}\end{table}
\end{document}
";
  let (stderr, xml) = convert(jmlr, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Computing Net Activation") && xml.contains("Cap A"),
    "{xml}"
  );
  let oup = r"\documentclass[unnumsec,webpdf,contemporary,large]{oup-authoring-template}
\begin{document}
\begin{sidewaystable}
\caption{X\label{t3}}
\begin{tabular}{ll}a&b\end{tabular}
\end{sidewaystable}
\end{document}
";
  let (stderr, xml) = convert(oup, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<caption>"), "{xml}");
}

/// A misplaced `\omit` (tex.web §1128) is one error and nothing else: no
/// group is left open to swallow the next `}` (nicematrix manual's
/// `\multicolumn` off-alignment ran to a `\Body` EoF runaway), and `&`
/// keeps working afterwards.
#[test]
fn misplaced_omit_does_not_open_a_group() {
  let tex = r"\documentclass{article}
\begin{document}
A{\multicolumn{1}{c}{B}}C

\begin{tabular}{ll}x&y\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert!(
    xml.contains(">C<") || xml.contains("BC") || xml.contains("C</p>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<td").count(), 2, "{xml}");
}

/// latex.ltx:16576: `\@array` lets `\tabularnewline` to `\\` for `array`
/// too, so a column template that re-lets `\\` inside a box it opened
/// (tabvar's varwidth cells) still ends the row (tabvar demo).
#[test]
fn math_array_lets_tabularnewline_to_the_row_break() {
  let tex = r"\documentclass{article}
\usepackage{array,varwidth}
\newcolumntype{C}{>{\begin{varwidth}{3cm}\let\\=\tabularnewline$}c<{$\end{varwidth}}}
\begin{document}
\[\begin{array}{cC}a&b\\ c&d\end{array}\]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<XMRow").count(), 2, "{xml}");
}

/// varwidth.sty:308-314 defines the `V{width}` column when array is
/// loaded; without it the template loses a column (numerica).
#[test]
fn varwidth_v_column_is_defined() {
  let tex = r"\documentclass{article}
\usepackage{array,varwidth,booktabs}
\begin{document}
\begin{tabular}{lccV{\linewidth}l}\toprule
env & rem & eq & vv & sep\tabularnewline\midrule
a & b & c & d & e\tabularnewline\bottomrule
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  assert_eq!(xml.matches("<td").count(), 10, "{xml}");
}

/// algpseudocodex ends a line's varwidth box only at the next `\State`;
/// `\Statex` (= `\item[]`) sets its text inside the open box, so it is a
/// break within the open line, not a nested `listingline` (manual,
/// coloredtheorem; pdflatex clean).
#[test]
fn statex_continues_the_open_line_box() {
  for wrap in [
    ("", ""),
    (r"\begin{minipage}[t]{0.45\textwidth}", r"\end{minipage}"),
  ] {
    let tex = format!(
      r"\documentclass{{article}}
\usepackage{{algpseudocodex}}
\begin{{document}}
{}
\begin{{algorithmic}}[1]
\State first line
\Statex continuing line
\State second line
\end{{algorithmic}}
{}
\end{{document}}
",
      wrap.0, wrap.1
    );
    let (stderr, xml) = convert(&tex, true);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<listingline").count(), 2, "{xml}");
    assert!(xml.contains("<break"), "{xml}");
  }
}

/// nicematrix's `\CodeBefore`/`\Body` must carry unique meanings: `\let` to
/// `\relax`, `\@ifnextchar\CodeBefore` (meaning comparison) matched any
/// `\relax` at a matrix start and the `Until:\Body` grab ran to EoF
/// (nicematrix manual's Fatal). A genuine `\CodeBefore` still grabs.
#[test]
fn nicematrix_relax_at_matrix_start_is_not_codebefore() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
$\begin{bNiceMatrix}\relax 9 & 17 \\ -2 & 5\end{bNiceMatrix}$
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert_eq!(xml.matches("<XMArray").count(), 1, "{xml}");
  let control = r"\documentclass{article}
\usepackage{nicematrix}
\begin{document}
$\begin{bNiceMatrix}\CodeBefore \rowcolor{blue!15}{1} \Body 9 & 17 \\ -2 & 5\end{bNiceMatrix}$
\end{document}
";
  let (stderr, xml) = convert(control, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("backgroundcolor="), "{xml}");
}

/// achemso.cls:144-165 `\bibnote` (notes2bib) files a note into the
/// bibliography; rendered as a numbered in-place note, and mciteplus's
/// `\mciteSubRef` (mciteplus.sty:780-782) is defined (achemso-demo).
#[test]
fn achemso_bibnote_is_a_numbered_note() {
  let tex = r"\documentclass{achemso}
\author{A}\title{T}
\begin{document}
Text\bibnote{This is a note.} and ref.~\mciteSubRef{Key2005}.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("role=\"bibnote\"") && xml.contains("This is a note."),
    "{xml}"
  );
}

/// A nested `\halign …\bgroup` (oz.sty's `op` schema inside `class`) must
/// not decrement the outer alignment's align_state at its end: `\bgroup`
/// never incremented it (tex.web §347), so the outer's `\crcr\noalign`
/// stayed recognizable (ozguide).
#[test]
fn nested_halign_bgroup_keeps_the_outer_align_state() {
  let tex = r"\documentclass{article}
\usepackage{oz}
\begin{document}
\begin{class}{Point}
\begin{op}{Translate}
dx? : \real
\ST
x' = x + dx?
\end{op}
\end{class}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Translate"), "{xml}");
}

/// amsmath.sty:52 `\newif\ifctagsplit@` exists for documents that poke it
/// (testmath.tex:1796); SHARED, Perl's binding lacks it too.
#[test]
fn amsmath_ctagsplit_switch_exists() {
  let tex = r"\documentclass{article}
\usepackage{amsmath}
\begin{document}
{\makeatletter\ctagsplit@true
\begin{equation}\begin{split} a&=b\\ &=c \end{split}\end{equation}}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<equation"), "{xml}");
}

/// latex.ltx:13531 `\DeclareMathDelimiter` takes six arguments and defines
/// a control-sequence symbol through `\DeclareMathSymbol` (oz.sty:261
/// corner delimiters from the AMSa symbol font; ozguide).
#[test]
fn declare_math_delimiter_defines_the_symbol() {
  let tex = r#"\documentclass{article}
\DeclareSymbolFont{AMSa}{U}{msa}{m}{n}
\DeclareMathDelimiter\ulcorner{4}{AMSa}{"70}{AMSa}{"70}
\DeclareMathDelimiter\urcorner{5}{AMSa}{"71}{AMSa}{"71}
\begin{document}
$\ulcorner a\urcorner$
\end{document}
"#;
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("\u{231C}") && xml.contains("\u{231D}"),
    "{xml}"
  );
  assert!(xml.contains("role=\"OPEN\""), "{xml}");
}

/// Long-tail singletons: aastex701.cls:13637 `\digitalasset`, amsbook.cls:1779
/// `\markleft`, t5enc's `\textdotbelow` (aastex701-sample,
/// Author_Handbook_Memo, amsldoc-vi; Perl's bindings lack all three).
#[test]
fn long_tail_class_and_encoding_singletons() {
  for (tex, needle) in [
    (
      r"\documentclass{aastex701}\begin{document}\digitalasset Text.\end{document}",
      "Text.",
    ),
    (
      r"\documentclass{amsbook}\begin{document}\markleft{RUNNING}Body.\end{document}",
      "Body.",
    ),
    (
      r"\documentclass{article}\usepackage[T5]{fontenc}\begin{document}\textdotbelow{a}\end{document}",
      "\u{1EA1}",
    ),
  ] {
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(needle), "{xml}");
  }
}

/// e-TeX `\ifincsname` is true inside `\csname…\endcsname`, so utf8.def's
/// guard keeps `§` literal in a name (clefval `\TheValue{a§b}`; Perl's
/// constant-false shortcut expanded it to `\textsection` and errored).
#[test]
fn ifincsname_keeps_utf8_chars_literal_in_names() {
  let tex = r"\documentclass{article}
\begin{document}
\expandafter\def\csname V@a§b\endcsname{VALUE}%
[\csname V@a§b\endcsname][\ifincsname yes\else no\fi][§]
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[VALUE][no][§]"), "{xml}");
  let clef = r"\documentclass{article}
\usepackage{clefval}
\begin{document}
\TheKey{a§b}{value-here}
\TheValue{a§b}
\end{document}
";
  // Single pass: clefval resolves values through the .aux file, so the
  // lookup prints `?? a§b ??` (pdflatex's first run does the same); the
  // point is that the key survived the `\csname` intact and no error fired.
  let (stderr, xml) = convert(clef, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("should not appear between"), "{stderr}");
  assert!(xml.contains("a§b"), "{xml}");
}

/// Long-tail singletons, batch 2: article.cls:585 `\@openbib@code`,
/// lettrine.sty:143 `\LettrineTextFont`, hyperref.sty:229
/// `\AfterBeginDocument` (mciteplus_doc, ijsra, iodhbwm).
#[test]
fn long_tail_bib_lettrine_hyperref_singletons() {
  for (tex, needle) in [
    (
      r"\documentclass{article}\begin{document}\makeatletter\@openbib@code Text.\end{document}",
      "Text.",
    ),
    (
      r"\documentclass{article}\usepackage{lettrine}\renewcommand*{\LettrineTextFont}{\itshape}\begin{document}\lettrine{A}{bc} def.\end{document}",
      "def.",
    ),
    (
      r"\documentclass{article}\usepackage{hyperref}\AfterBeginDocument{\def\x{Hooked.}}\begin{document}\x\end{document}",
      "Hooked.",
    ),
  ] {
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(needle), "{xml}");
  }
}

/// afterpackage.sty's patched `\@popfilename` reads `\@currname` as the
/// package being finished; a NESTED load (ncc.cls → ncclatex → nccsect) must
/// still fire the `\AfterPackage{nccsect}` hook that defines
/// `\openrightorany` (nccdefaults.sty:41; ncclatex manual).
#[test]
fn afterpackage_hook_fires_for_a_nested_load() {
  let tex = r"\documentclass[11pt]{ncc}
\begin{document}
\makeatletter\ifx\openrightorany\@undefined UNDEF\else DEF\fi\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("DEF") && !xml.contains("UNDEF"), "{xml}");
}

/// `\inputminted` inside a minipage/tcolorbox must not close the box: the
/// listing's trailer already balances its own group (sweep-37 regression
/// from 54x: algxpar-doc, tikzducks-doc, biblatex-oxref, tcolorbox posters).
#[test]
fn inputminted_inside_a_minipage_keeps_its_box() {
  let tex = r"\documentclass{article}
\usepackage{minted}
\begin{document}
\begin{figure}
\begin{minipage}{5cm}
\inputminted{tex}{no-such-file-for-this-guard.tex}
Still inside.
\end{minipage}
\end{figure}
After.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("Still inside.") && xml.contains("After."),
    "{xml}"
  );
}

/// Long-tail singletons, batch 3: lineno.sty:1445 `\linelabel`, verbatim's
/// `\verbatim@in@stream`, hyperref.sty:3331 `\@baseurl` default (lineno
/// manual, ltug notes-for-authors, cms-dates-intro).
#[test]
fn long_tail_lineno_verbatim_baseurl_singletons() {
  for (tex, needle) in [
    (
      r"\documentclass{article}\usepackage{lineno}\begin{document}\linenumbers x\linelabel{a} y (\lineref{a})\end{document}",
      "y",
    ),
    (
      r"\documentclass{article}\usepackage{verbatim}\begin{document}\makeatletter\ifx\verbatim@in@stream\@undefined NO\else OK\fi\makeatother\end{document}",
      "OK",
    ),
    (
      r"\documentclass{article}\usepackage{hyperref}\begin{document}\makeatletter[\@baseurl]\makeatother Text.\end{document}",
      "[]",
    ),
  ] {
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(needle), "{xml}");
  }
}

/// A `\lstnewenvironment` cell leaves align_state balanced, so the `&`
/// after it is the column end (lexref's ltxdockit `ltxcode` cells; 54x
/// regression: `{` opener with an `\lx@hidden@egroup` closer).
#[test]
fn listings_environment_cell_ends_at_the_tab() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\lstnewenvironment{ltxcode}{}{}
\begin{document}
\begin{tabular}{llll}
\begin{ltxcode}
a
\end{ltxcode} & \begin{ltxcode}
b
\end{ltxcode} & \begin{ltxcode}
c
\end{ltxcode} & \begin{ltxcode}
d
\end{ltxcode} \\
1 & 2 & 3 & 4 \\
\end{tabular}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  assert_eq!(xml.matches("<td").count(), 8, "{xml}");
}

/// `\marginpar[{…[1][1-4]…}]{…}`: the braced optional is re-passed braced,
/// so its own brackets are not read as the optional's end (Test-flexipage,
/// a 55a regression; latex.ltx:17591 uses `{#1}`).
#[test]
fn marginpar_optional_keeps_its_own_brackets() {
  let tex = r"\documentclass{article}
\usepackage{lipsum}
\begin{document}
Text\marginpar[{\lipsum[1][1-1]}]{Y} more.
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("role=\"margin\"") || xml.contains("margin"),
    "{xml}"
  );
  assert!(xml.contains("more."), "{xml}");
}

/// A Semiverbatim read inertizes active shorthands as `\url`'s
/// `\dospecials` loop does: babel-czech's active `-` in a hyperref url no
/// longer runs its word scanner inside the attribute (csbulletin; a 55c
/// regression once `\ifinner` became correct).
#[test]
fn semiverbatim_inertizes_babel_shorthands() {
  let tex = r"\documentclass{csbulletin}
\usepackage{hyperref}
\begin{document}
\nolinkurl{a-b}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert!(xml.contains("a-b"), "{xml}");
  let tilde =
    r"\documentclass{article}\usepackage{hyperref}\begin{document}\nolinkurl{a~b}\end{document}";
  let (stderr, xml) = convert(tilde, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("a~b"), "{xml}");
}

/// minted's displays use the `\begingroup`/`\endgroup` listing group like
/// lstlisting, so inside a `p{}` cell their closer meets no mode-switch
/// frame (kernel-alignment locus probe: the only red construct).
#[test]
fn minted_in_a_p_column_keeps_the_cell() {
  for body in [
    "\\begin{minted}{tex}\nzz\n\\end{minted}",
    "\\inputminted{tex}{no-such-file-for-this-guard.tex}",
  ] {
    let tex = format!(
      "\\documentclass{{article}}\n\\usepackage{{minted}}\n\\begin{{document}}\n\\begin{{tabular}}{{p{{4cm}}l}}\n{body} & next \\\\\nrow2 & x \\\\\n\\end{{tabular}}\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert(&tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
    assert_eq!(xml.matches("<td").count(), 4, "{xml}");
  }
}

/// Long-tail singletons, batch 4: physics.sty:23 `\vnabla`,
/// quantumarticle.cls:22 `\quantumarticleversion`, latex.ltx:18349
/// `\@normalsize`, graphics.sty:156-158 `\Ginput@path` (physics manual,
/// quantum-template, UNAMThesis, upmethodology; Perl lacks all four).
#[test]
fn long_tail_physics_quantum_normalsize_ginput_singletons() {
  for (tex, needle) in [
    (
      r"\documentclass{article}\usepackage{physics}\begin{document}$\vnabla f$\end{document}",
      "<Math",
    ),
    (
      r"\documentclass{quantumarticle}\begin{document}v\quantumarticleversion.\end{document}",
      "v6.",
    ),
    (
      r"\documentclass{report}\begin{document}\makeatletter\@normalsize\makeatother x\end{document}",
      "x",
    ),
    (
      r"\documentclass{article}\usepackage{graphicx}\graphicspath{{figs/}}\begin{document}\makeatletter[\Ginput@path]\makeatother\end{document}",
      "[figs/]",
    ),
  ] {
    let (stderr, xml) = convert(tex, false);
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert!(xml.contains(needle), "{xml}");
  }
}

/// tex.web §485-486: `\read` consumes the whole physical line, so a header
/// read under `\ExplSyntaxOn` (space = IGNORE) followed by an
/// `\ior_map_inline` under `\ExplSyntaxOff` yields no spurious empty row
/// (l3prefixes' `Until:,` runaway; Perl shares the empty row).
#[test]
fn read_consumes_the_physical_line_across_catcode_regimes() {
  let tex = r"\documentclass{article}
\usepackage{expl3}
\begin{filecontents}[overwrite,noheader,nosearch]{guard-twocol.csv}
h1,h2,h3,h4
r1,r2,r3,r4
s1,s2,s3,s4
\end{filecontents}
\ExplSyntaxOn
\cs_new_protected:Npn \__guard_row:w #1 , #2 , #3 , #4 \q_stop { [#1/#2/#3/#4] }
\ior_new:N \g_guard_ior
\ior_open:Nn \g_guard_ior { guard-twocol.csv }
\ior_get:NN \g_guard_ior \l_tmpa_tl
\cs_new_protected:Npn \GuardTable
  { \ior_map_inline:Nn \g_guard_ior { \__guard_row:w ##1 \q_stop } }
\ExplSyntaxOff
\begin{document}
\GuardTable
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Fatal"), "{stderr}");
  assert!(
    xml.contains("[r1/r2/r3/r4") && xml.contains("[s1/s2/s3/s4"),
    "{xml}"
  );
}

/// mdwtab.sty:765 `\hlx{vhv}` ends the row and rules it (talkdoc);
/// german.sty:375 `\def@dqmacro` exists for germkorr's patch.
#[test]
fn mdwtab_hlx_ends_the_row_and_rules() {
  let tex = r"\documentclass{article}\usepackage{mdwtab}
\begin{document}\begin{tabular}{cc}a&b\hlx{vhv}c&d\end{tabular}\end{document}";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<tr").count(), 2, "{xml}");
  let de =
    r"\documentclass{article}\usepackage{german,germkorr}\begin{document}Text.\end{document}";
  let (stderr, xml) = convert(de, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Text."), "{xml}");
}

/// jmlr.cls:246-247 `\titlebreak`/`\titletag` and the raw jmlrutils.sty
/// surface (`\subfigure`, `\subfigref`, `\includeteximage`) reach the
/// jmlr binding (pmlr-sample).
#[test]
fn jmlr_has_the_jmlrutils_surface() {
  let tex = r"\documentclass[pmlr]{jmlr}
\title[Short]{A Long\titlebreak Title \titletag{x}}\author{\Name{A}}
\begin{document}
\begin{figure}\floatconts{fig:a}{\caption{Two}}{\subfigure[one]{\rule{1cm}{1cm}}\subfigure[two]{\rule{1cm}{1cm}}}\end{figure}
See \subfigref{fig:a}{a}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<title") && xml.contains("<figure"), "{xml}");
}

/// microtype.sty:36 `\MT@MT` marks the package for typog.sty:68's
/// `\ifdefined\MT@MT` (typog-example under `trackingttspacing`).
#[test]
fn microtype_marker_satisfies_typog() {
  let tex = r"\documentclass{article}\usepackage[activate=true]{microtype}\usepackage[trackingttspacing]{typog}
\begin{document}Text.\end{document}";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Text."), "{xml}");
}

/// An autoload stub must not satisfy a class-detection probe:
/// projlib-author.sty:38 `\cs_if_exist:NT \subjclass {\endinput}` (homework).
#[test]
fn autoload_stubs_do_not_satisfy_class_probes() {
  let tex = r"\documentclass{article}\usepackage{expl3}\begin{document}
\ExplSyntaxOn[\cs_if_exist:NTF\subjclass{AMS}{NOAMS}]\ExplSyntaxOff
\end{document}";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[NOAMS]"), "{xml}");
  let ams = r"\documentclass{amsart}\begin{document}\subjclass{03B05}Text.\end{document}";
  let (stderr, _xml) = convert(ams, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// titlesec.sty:420 sets `\thetitle` per typeset title, so mla.cls:196's
/// `\titleformat{\section}{}{\thetitle.\enspace}…` label expands.
#[test]
fn titlesec_thetitle_in_format_label() {
  let tex = r"\documentclass{article}\usepackage{titlesec}
\titleformat{\section}{}{\thetitle.\enspace}{0pt}{}
\begin{document}\section{Intro}Body.\end{document}";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Intro"), "{xml}");
}

/// latex.ltx:19172-19280 release rollback: `\RequirePackage{doc}[=v2]` loads
/// doc-2021-06-01.sty, so dox.sty's v2-era `\let\SpecialMacroIndex
/// \SpecialUsageIndex` is not a self-loop (testidx-manual and every
/// nlctdoc manual; Perl hangs identically).
#[test]
fn package_release_rollback_loads_the_named_release() {
  let tex = r"\documentclass{article}
\RequirePackage{doc}[=v2]
\usepackage{dox}
\begin{document}
\makeatletter[\csname ver@doc.sty\endcsname]\makeatother
\DescribeMacro\foo Text.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("2021") && xml.contains("Text."), "{xml}");
}

/// article.cls's `\maketitle` disables `\title`/`\maketitle` after use; a
/// class that `\renewcommand`s `\maketitle` without that cleanup
/// (schooldocs.sty:136, `\correct` :168-178 chaining `\@title`) had the
/// redefinition dropped by the lock and the kernel cleanup made later
/// `\title`s no-ops, so the second `\correct` built a self-referential
/// `\@originaltitle` (`PushbackLimit`; schooldocs-examples). The
/// self-disabling half now yields when the class took `\maketitle` over.
#[test]
fn maketitle_cleanup_yields_to_a_class_redefinition() {
  let tex = r"\documentclass{article}
\usepackage{schooldocs}
\begin{document}
\schooldocstitles
\title{Standard}
\maketitle
\correct
\title{Exam}
\maketitle
\correct
\title{Small}
\schooldocstitles
\makesmalltitle
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Small"), "{xml}");
  // Without a class redefinition the standard cleanup still applies.
  let tex = r"\documentclass{article}
\title{T}\author{A}
\begin{document}
\maketitle
\title{Again}\makeatletter[\@title]\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[]"), "{xml}");
}

/// Conditionals inside dimension/glue arguments (e.g. \hspace, \raisebox)
/// must cleanly expand remaining tokens (\else, \fi) when reparsed in a
/// temporary mouth so they do not leak unclosed if-frames into enclosing
/// macros like \parbox.
/// Witness: typog-example / parbox_dimen_conditional_double.tex
#[test]
fn dimension_conditional_in_parbox_does_not_leak_or_duplicate() {
  let tex = r"\documentclass{article}
\makeatletter
\newlength{\Lreg}\newlength{\U}\setlength{\U}{.001em}\def\a{0}\def\b{*}
\makeatother
\begin{document}
\parbox[t]{0pt}{s\hspace{\ifx\a\b\Lreg\else\a\U\fi}e}
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("class=\"ltx_parbox\"").count(),
    1,
    "Expected exactly one ltx_parbox: {xml}"
  );
  assert!(xml.contains("se"), "{xml}");

  // True branch case (\else ... \fi tail in mouth)
  let tex_true = r"\documentclass{article}
\begin{document}
\parbox[t]{0pt}{s\hspace{\iftrue 10pt\else 20pt\fi}e}
\end{document}
";
  let (stderr_true, xml_true) = convert(tex_true, false);
  assert_eq!(error_count(&stderr_true), 0, "{stderr_true}");
  assert_eq!(
    xml_true.matches("class=\"ltx_parbox\"").count(),
    1,
    "Expected exactly one ltx_parbox: {xml_true}"
  );

  // typog.sty \raisebox shape
  let tex_raisebox = r"\documentclass{article}
\begin{document}
\raisebox{\iftrue 5pt\else 10pt\fi}{test}
\end{document}
";
  let (stderr_raise, xml_raise) = convert(tex_raisebox, false);
  assert_eq!(error_count(&stderr_raise), 0, "{stderr_raise}");
  assert!(xml_raise.contains("test"), "{xml_raise}");
}

/// nicematrix environments with `name=...` or `create-cell-nodes` materialize
/// coordinate nodes for PGF/TikZ overlays (`ma-matrice-2-2`), avoiding
/// `Package pgf Error: No shape named '...' is known` (witness nicematrix-french:5986).
#[test]
fn nicematrix_cell_nodes_materialized_for_pgf_overlay() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\begin{document}
$\begin{pNiceMatrix}[name=ma-matrice]
1 & 2 & 3 \\ 4 & 5 & 6 \\ 7 & 8 & 9
\end{pNiceMatrix}$
\tikz[remember picture,overlay] \draw (ma-matrice-2-2) circle (2mm) ;
\end{document}
";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<svg:path"),
    "Expected overlay svg path: {xml}"
  );
  assert!(
    xml.contains("matrix@(Array"),
    "Expected pNiceMatrix math: {xml}"
  );
}

/// P52: nicematrix + shortvrb verbatim footnotes. Under \VerbatimFootnotes,
/// \footnote captures its body live so active verbatim tokens (e.g. `|` from
/// shortvrb) and inner unescaped braces digest cleanly into <note>
/// without triggering misplaced \omit or premature argument termination.
#[test]
fn nicematrix_shortvrb_verbatim_footnotes() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix,shortvrb,fancyvrb}
\MakeShortVerb{\|}
\VerbatimFootnotes
\begin{document}
Plain: |\multicolumn|.
X\footnote{Footnote with |\multicolumn| and a brace |}| here.}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains(r#"<note mark="1" role="footnote""#), "{xml}");
  assert!(xml.contains(r">\multicolumn<"), "{xml}");
}

/// P53: nicematrix AutoNiceMatrix, tabularnote, braces, and CodeBefore overlays.
#[test]
fn nicematrix_autonicematrix_delimiters_and_overlays() {
  let tex = r"\documentclass{article}
\usepackage{nicematrix,tikz}
\begin{document}
\[ C = \pAutoNiceMatrix{2-2}{C_{\arabic{iRow},\arabic{jCol}}} \]
\begin{NiceTabular}{cc}
A\tabularnote{A note} & B \\
\end{NiceTabular}
$\begin{NiceArray}{cc}[first-col]
\Hbrace{2}{top} \\
\Vbrace{2}{left} & 1 & 2 \\
& \Hspace{5mm} & \Vdotsfor{1}
\end{NiceArray}$
\[\begin{NiceArray}{cc}
\CodeBefore [create-cell-nodes]
  \chessboardcolors{red!15}{blue!15}
  \SubMatrix({1-1}{2-2})
  \tikz \draw (1-1) -- (2-2) ;
\Body
1 & 2 \\
3 & 4
\end{NiceArray}\]
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("C_{1,1}"), "{xml}");
  assert!(xml.contains("C_{2,2}"), "{xml}");
  assert!(xml.contains(r#"role="footnote""#), "{xml}");
  assert!(xml.contains("A note"), "{xml}");
  assert!(xml.contains("top"), "{xml}");
  assert!(xml.contains("left"), "{xml}");
}
