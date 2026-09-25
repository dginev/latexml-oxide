//! Red/green guards for the perfect-kernel batches 40-43 root-cause fixes.
//! Each test is the minimal reproduction distilled during triage; the
//! doc-comment names the ORIGINAL corpus witness (TeX Live doc corpus,
//! `bundle/doc`) whose larger conversion was vetted separately.

/// Convert an inline snippet in a tempdir under the perfect-kernel preload;
/// return (ANSI-stripped stderr, XML string).
fn convert(tex: &str) -> (String, String) { convert_with_files(tex, &[]) }

/// Like `convert`, with sibling files (`.cls`/`.sty` under test) written
/// next to `t.tex` and reachable through `TEXINPUTS`.
pub(crate) fn convert_with_files(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  if files.is_empty() {
    super::perfect_kernel_batch46::convert(tex, true)
  } else {
    super::perfect_kernel_batch46::convert_files(tex, files)
  }
}

fn error_count(stderr: &str) -> usize {
  // Any `Error:`/`Fatal:` diagnostic, anywhere in the line (WISDOM 85).
  let re = regex::Regex::new(r"(Error|Fatal):[A-Za-z_]+:").unwrap();
  stderr.lines().filter(|l| re.is_match(l)).count()
}

/// Batch 40: the xparse TCB listing trio takes a LEADING `[init-options]`
/// optional (tcblistingscore.code.tex:329). RED: the three `{}` args
/// grabbed `[`, `u`, `s`; the options body digested raw (`misdefined:#`
/// storm, `\thetcbcounter` undefined). Witness: atableau/atableau
/// (1001-cap → 0 together with the stub removal).
// `listing only`: the body is C-like text that the default `listing and
// text` mode would also execute in real LaTeX (codebox.sty declares its
// listings `listing only` for the same reason).
#[test]
fn newtcblisting_xparse_leading_optional() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage[skins]{tcolorbox}
\tcbuselibrary{listings}
\newcounter{example}
\NewTCBListing[use counter=example, number within=section]{example}{ O{} s m }{ title={\thetcbcounter}, listing only, #1 }
\begin{document}
\begin{example}{tst}
verbatim_body^here
\end{example}
\end{document}
",
  );
  assert!(
    !stderr.contains("misdefined"),
    "leading [init-options] mis-grabbed again:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The lexer marks identifiers up, so match the split body parts.
  assert!(
    xml.contains("verbatim_body") && xml.contains("here"),
    "listing body was not captured verbatim:\n{xml}"
  );
}

/// Batch 40: beamer's full `\newif` surface. An UNDEFINED `\if…` inside a
/// skipped branch is invisible to the meaning-counting skipper (tex.web
/// §366; Conditional.pm:117 — the skipper is correct, the definition was
/// missing), so its `\fi` closed the outer frame early. RED: 2 errors
/// (`unexpected:\else` + `unexpected:fi`). Witnesses:
/// beamerthemecelestia/Celestia-demo-* (2 → 0 each, ×6 docs).
#[test]
fn beamer_newif_invisible_in_skipped_branch() {
  let (stderr, xml) = convert(
    r"\documentclass{beamer}
\makeatletter
\iffalse
  \ifbeamer@plainframe X\fi
\else
  \def\elsebranch{ran}
\fi
\makeatother
\begin{document}
\begin{frame}ok \elsebranch\end{frame}
\end{document}
",
  );
  assert!(
    !stderr.contains("unexpected:fi") && !stderr.contains("unexpected:\\else"),
    "orphan fi/else came back:\n{stderr}"
  );
  assert!(xml.contains("ran"), "else-branch did not execute:\n{xml}");
}

/// Batch 41: beamer's COMMAND form `\frame{content}` must route through
/// the frame environment. RED: the env-installed bare `\frame` opened the
/// `_noautoclose` subsection and never closed it — later `\section`s
/// nested inside (malformed:ltx beamer-sectioning family, 185 errors /
/// 40 docs). Witness: beamerauxtheme (16 → 0).
#[test]
fn beamer_frame_command_form_closes() {
  let (stderr, xml) = convert(
    r"\documentclass{beamer}
\begin{document}
\section{S}\frame{f}\section{T}
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("<section").count(),
    2,
    "a section was swallowed by a dangling frame subsection:\n{xml}"
  );
}

/// Batch 41: amsopn re-asserts the 34 log-like operators exactly like
/// real amsopn.sty L56-89. RED: amsldoc.cls makes `\arg{1}` doc-markup;
/// `$\arg$` then ate its closing `$` — 101-error cascade (Perl shares the
/// omission; pdflatex is the oracle). Witnesses: amsmath/amsldoc
/// (101 → 0), amsldoc-it/itamsldoc + amsldoc-vn/amsldoc-vi (101 → 0 with
/// the `\@nobslash` binding below).
#[test]
fn amsopn_reasserts_clobbered_operators() {
  let (stderr, _xml) = convert(
    r"\documentclass{amsldoc}
\usepackage{amsmath}
\begin{document}
text $\arg$ text $\det_n$.
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// Batch 43: amsldoc/amsdtx `\@nobslash` resolved at expansion time. RED:
/// the raw `\ifnum`#1=\bslchar` test rode inside `\index` arguments; the
/// SanitizedVerbatim untex→retokenize roundtrip welded its catcode-12 `\`
/// into fake CSes (`Expected a relational token … Got \bslchar` + empty
/// index entries). Witnesses: amsldoc-it/itamsldoc, amsldoc-vn/amsldoc-vi
/// (2 residual errors each → 0).
#[test]
fn amsldoc_nobslash_expansion_time() {
  let (stderr, xml) = convert(
    r"\documentclass{amsldoc}
\begin{document}
\cn{\|} and \cn{\bslash} in text.
\end{document}
",
  );
  assert!(
    !stderr.contains("relational"),
    "\\bslchar reached the relational reader again:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Index-entry content is vetted on the full itamsldoc witness (the class
  // only opens its index stream in the full-manual configuration).
  let _ = xml;
}

/// Batch 40: any stub replacing a raw .sty must register `\ver@<file>`
/// WITH the real ` v.` pattern. RED: `\GetFileInfo{curve2e.sty}`'s
/// delimited ` v.` scan ran away over an undefined `\ver@curve2e.sty`,
/// poisoning the whole document (locator-less pushback; shortverb regions
/// executed raw). Witness: curve2e/curve2e-manual (88 errors + fatal →
/// 41 honest stub-gap errors, all Match/Pair gone).
#[test]
fn curve2e_stub_registers_ver() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{curve2e}
\makeatletter
\providecommand\GetFileInfo[1]{%
  \def\@tempb##1 v.##2 ##3\relax##4\relax{\def\filedate{##1}\def\fileversion{##2}}%
  \edef\@tempa{\csname ver@#1\endcsname}%
  \expandafter\@tempb\@tempa\relax? ? \relax\relax}
\makeatother
\begin{document}
\GetFileInfo{curve2e.sty}
version \fileversion\ works.
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
}

/// Batch 42: `use counter=`/`listing file=` honored by generated TCB
/// listing envs, with catcode-robust `\lxlstbeginwritefile` injection
/// (a doc-level definition site has `@` = OTHER — the raw
/// `\lst@BeginAlsoWriteFile` name split there). RED: nothing was ever
/// written; `\input{\jobname.1.listing}` was a missing file. Witness:
/// incgraph/incgraph (`\inputlisting{\n}` reading 12 such files).
#[test]
fn tcb_listing_file_written_and_input_back() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newcounter{texexp}
\newtcblisting[use counter=texexp]{texexptitled}[2][]{listing file={\jobname.\thetcbcounter.listing}}
\begin{document}
\begin{texexptitled}{t}{l}
replayed\_marker
\end{texexptitled}
\input{t.1.listing}
\end{document}
",
  );
  assert!(
    !stderr.contains("missing_file"),
    "listing file was not written to the VFS:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Once as the verbatim listing, once replayed via \input.
  assert!(
    xml.matches("replayed").count() >= 2,
    "the written listing did not replay via \\input:\n{xml}"
  );
}

/// Batch 44: `\newtcbinputlisting` — the defined command INPUTS its
/// `listing file=` (after #-substitution) as a listing and shares the
/// referenced env's counter via `use counter from=`. RED: the command was
/// undefined; incgraph's `\inputexamplelisting` cascaded
/// (`\tcb@cnt@texexptitled` csname errors ×15). Witness:
/// incgraph/incgraph.
#[test]
fn tcbinputlisting_inputs_file() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\newcounter{texexp}
\newtcblisting[use counter=texexp]{texexptitled}[2][]{listing file={\jobname.\thetcbcounter.listing}}
\newtcbinputlisting[use counter from=texexptitled]{\inputexamplelisting}[3][]{listing file={#2}}
\begin{document}
\begin{texexptitled}{t}{l}
roundtrip\_marker
\end{texexptitled}
\inputexamplelisting{t.1.listing}{lbl}
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.matches("roundtrip").count() >= 2,
    "\\inputexamplelisting did not display the recorded listing:\n{xml}"
  );
}

/// Batch 42: pgfmath `array({e0,e1,…}, i)` — brace-list first argument
/// parsed in place, 0-based select. RED: "Unimplemented pgfmath operator
/// 'array'" (Perl silently no-ops). Witness: colorblind/colorblind_doc
/// (17 → 0).
#[test]
fn pgfmath_array_selects() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\pgfmathparse{array({3,7,5},1)}\edef\picked{\pgfmathresult}
picked=[\picked]
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("picked=[7"),
    "array(...,1) did not select the second element:\n{xml}"
  );
}

/// Batch 43: xkeyval presets implemented (OXIDIZED_DESIGN #173). RED: the
/// six preset front-ends were warn-stubs and `\setkeys` bypassed the
/// preset hooks — key code that only runs from presets never ran
/// (cntperchap's "section level … is unknown"). Witness:
/// cntperchap/cntperchap_doc (6 → 3, residual is unrelated surface).
#[test]
fn xkeyval_presets_apply_on_setkeys() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{xkeyval}
\makeatletter
\define@key{cpskeys}{tracklevel}[section]{\gdef\@cps@@keymacro@@tracklevel{#1}}
\presetkeys{cpskeys}{tracklevel=section}{}
\setkeys{cpskeys}{}
\begin{document}
\makeatletter
\expandafter\ifx\csname @cps@@keymacro@@tracklevel\endcsname\relax
LEVEL-UNKNOWN\else DEFINED: \@cps@@keymacro@@tracklevel\fi
\makeatother
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("DEFINED: section"),
    "preset key code did not run on \\setkeys:\n{xml}"
  );
}

/// Batch 43: Perl-faithful balanced-pair color-spec trim. RED: the old
/// `trim_matches('{','}')` stripped braces from either end independently,
/// so a name whose T1-mangled form ends in `}` (`é` →
/// `…\lx@applyaccent…{e}`) lost its tail at LOOKUP while DEFINE stored it
/// intact — "Can't find color named 'xFuchsiaFonc…'". Witness:
/// couleurs-fr/couleurs-fr-doc (1 → 0).
#[test]
fn color_name_accent_symmetric_keys() {
  let (stderr, xml) = convert(
    "\\documentclass{article}
\\usepackage[T1]{fontenc}
\\usepackage{xcolor}
\\definecolor{caf\u{e9}}{HTML}{112233}
\\begin{document}
\\textcolor{caf\u{e9}}{x} \\textcolor[rgb]{0.5,0.5,0.5}{y}
\\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("#112233"),
    "accented color name failed to round-trip define→lookup:\n{xml}"
  );
  assert!(xml.contains("#808080"), "plain rgb spec regressed:\n{xml}");
}

/// Batch 43: `\index` sort keys brace-protected through the
/// `\@indexphrase[]` re-parse (OXIDIZED_DESIGN #174, KPE #83 sibling).
/// RED: a `]` inside the sort key truncated the optional-arg re-parse —
/// key attribute cut at `gradetable[v`, display phrase spilled as illegal
/// indexmark children. Witness: exam/examdoc (39-error family → 0),
/// pgfornament docs.
#[test]
fn index_sortas_bracket_protected() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{makeidx}\makeindex
\newcommand{\indc}[1]{\index{#1@\texttt{\char`\\#1}}}
\begin{document}
x\indc{gradetable[v]} y\index{plain}\index{sorted@\textbf{shown}}
\printindex
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("key=\"gradetable[v]\""),
    "sort key with ] was truncated again:\n{xml}"
  );
  assert!(
    xml.contains("key=\"sorted\"") && xml.contains("key=\"plain\""),
    "plain sort keys must be byte-unchanged:\n{xml}"
  );
}

/// Batch 40: `\pscircle`'s coordinate pair is OPTIONAL (Perl
/// pstricks_support ZeroPSCoord = ReadPSCoord || ZeroPair). RED:
/// `Error:expected:Pair` on dsptricks.sty L535
/// `\pscircle[…]{\PZCROC\dspUnitX}`. Witness: dsptricks/dspTricksManual.
#[test]
fn pscircle_pair_optional() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{pstricks}
\begin{document}
\begin{pspicture}(2,2)
\pscircle[linewidth=1pt]{0.5}
\pscircle(1,1){0.3}
\end{pspicture}
\end{document}
",
  );
  assert!(
    !stderr.contains("expected:Pair"),
    "pair-less \\pscircle regressed:\n{stderr}"
  );
}

/// Batch 45: `\DeclareMathOperator`'s text is expanded before the
/// String round-trip into `def_math`. RED: numerica.sty:50-51 declares
/// `\DeclareMathOperator{\asinh}{\cs_to_str:N \asinh}` under expl3
/// catcodes; the stringified body re-tokenized as `\cs_to_str` `_` `:N`
/// at every use → 100 malformed:ltx + Fatal Stomach:Recursion. Witness:
/// numerica/numerica (manual line 148).
#[test]
fn declaremathoperator_expands_expl3_name() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{amsmath}
\ExplSyntaxOn
\DeclareMathOperator{\asinh}{\cs_to_str:N \asinh}
\ExplSyntaxOff
\begin{document}
$\asinh$
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<XMTok role="OPFUNCTION" scriptpos="post">asinh</XMTok>"#),
    "expl3-named operator did not resolve to plain letters:\n{xml}"
  );
}

/// Batch 45: `\PassOptionsToPackage`/`\PassOptionsToClass` store one
/// `Stored::String` per option (Perl Package.pm:2435 spreads). RED: the
/// whole list landed as ONE nested element, which the `\opt@<file>`
/// rebuild skips, so options routed through the primitives read back
/// EMPTY — a kvoptions class forwarding `\CurrentOption` to its own .sty
/// after `\LoadClass[12pt]` (which clobbers `\@classoptionslist`) never
/// saw `scheme`. Witness: brandeis-problemset/example (87 → tabu residual).
#[test]
fn passoptions_spreads_options() {
  let (stderr, xml) = convert_with_files(
    r"\documentclass[scheme]{pod}
\begin{document}\begin{scheme}hi\end{scheme}\end{document}
",
    &[
      (
        "pod.cls",
        r"\ProvidesClass{pod}
\RequirePackage{kvoptions}
\SetupKeyvalOptions{family=po,prefix=po@}
\DeclareVoidOption{scheme}{\PassOptionsToPackage{\CurrentOption}{pod}}
\ProcessKeyvalOptions*
\LoadClass[12pt]{article}
\RequirePackage{pod}
",
      ),
      (
        "pod.sty",
        r"\ProvidesPackage{pod}
\RequirePackage{kvoptions}
\SetupKeyvalOptions{family=po,prefix=po@}
\DeclareBoolOption{scheme}
\ProcessKeyvalOptions*
\ifpo@scheme\newenvironment{scheme}{}{}\fi
",
      ),
    ],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("hi") && !xml.contains("ERROR"),
    "option passed via \\PassOptionsToPackage was lost:\n{xml}"
  );
}

/// Batch 45: amsmath's `\newif\if@display` (amsmath.sty:649) exists.
/// RED: gaceta.cls:1666 redefines `\mod` with amsmath's own
/// `\if@display…\else…\fi` body via babel's `\addto`; the undefined
/// conditional orphaned 2×`\else`, 2×`\fi` and leaked `#1`
/// (`misdefined:#`). SHARED with Perl. Witnesses: gaceta
/// plantilla-articulo-suelto / -de-seccion (10 → 0 each).
#[test]
fn amsmath_if_display_defined() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{amsmath}
\usepackage[spanish]{babel}
\makeatletter
\addto\es@operators{%
  \renewcommand{\mod}[1]{\allowbreak\if@display\mkern18mu
  \else\mkern12mu\fi{\operator@font mod}\,\,#1}%
}
\makeatother
\begin{document}x $a \mod b$\end{document}
",
  );
  assert!(
    !stderr.contains("unexpected:fi") && !stderr.contains("misdefined"),
    "\\if@display went missing again:\n{stderr}"
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("mod"), "\\mod body did not digest:\n{xml}");
}

/// Batch 45: natbib's full `\newif` surface (`\ifNAT@full` :683,
/// `\ifNAT@longnames` :284, …). RED: nmbib.sty:267 `\ifNAT@full` was
/// undefined, so the `\fi` of its skipped branch closed the outer
/// conditional (`unexpected:fi`). SHARED with Perl. Witness:
/// nmbib/nmbib-sample (4 fi lines; the doc keeps a SHARED residual on
/// ~20 other natbib internals).
#[test]
fn natbib_newif_surface_complete() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{nmbib}
\begin{document}
\citeall{X}
\end{document}
",
  );
  assert!(
    !stderr.contains("unexpected:fi") && !stderr.contains("undefined:\\ifNAT@"),
    "natbib \\newif surface regressed:\n{stderr}"
  );
}

/// Batch 45: `\@enumctr` is defined by enumerate lists (latex.ltx:16057
/// `\edef\@enumctr{enum\romannumeral\the\@enumdepth}`). RED: beginItemize
/// (Perl pool:1314, SHARED) only defined `\@listctr`; bullenum.sty:58/61
/// `\csname the\@enumctr\endcsname` cascaded to the 100-error cap.
/// Witness: bullcntr/bullcntr-man.
#[test]
fn enumerate_defines_enumctr() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{bullenum}
\begin{document}
\begin{bullenum}
\item First
\item Second
\end{bullenum}
\end{document}
",
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(
    xml.matches("<item").count(),
    2,
    "bullenum items did not materialize:\n{xml}"
  );
}
