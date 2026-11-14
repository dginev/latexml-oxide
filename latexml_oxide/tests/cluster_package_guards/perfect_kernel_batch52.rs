//! Red/green guards for perfect-kernel batch 52 (sweep 28 tikzlings `hsb`
//! cluster, the memoir/dpfloat `\csname` runaway, the xspace pending-space
//! exception, and the l3prg `Until` runaway at end of input). Each test is
//! the minimal reproduction distilled during triage; the doc-comment names
//! the ORIGINAL corpus witness (TeX Live doc corpus) whose larger
//! conversion was vetted separately.
use super::{
  perfect_kernel_batch46::{convert, error_count},
  perfect_kernel_batch51::convert_with_files,
};

/// `\selectcolormodel{rgb}` (xcolor.sty:137-147) sets `\convertcolorsDtrue`
/// so every later `\definecolor`/`\colorlet` is CONVERTED to the target
/// model at definition time (`\XC@definecolor`, xcolor.sty:535-537). pgf
/// only knows rgb/cmy/cmyk/gray (pgfcoregraphicstate.code.tex:133-137), so
/// an `hsb` colour reaching `\draw[fill=…]` unconverted is
/// "Unsupported color model" — pdflatex exit 0 with the selection, and
/// errors without it. RED: the stub `\selectcolormodel` was a no-op (3
/// errors). Witness: tikzlings/tikzlings-doc (tikzlings-doc.tex:36 +
/// tikzlings-bears.sty:124, 29× hsb across the animal styles).
#[test]
fn selectcolormodel_converts_definecolor() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{tikz}
\selectcolormodel{rgb}
\definecolor{bb}{hsb}{0.1,0.5,0.5}
\begin{document}
\tikz\draw[bb,fill=bb] (0,0)--(1,1);
\textcolor{bb}{text}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("Unsupported color model"), "{stderr}");
  assert!(xml.contains("<text") && xml.contains(">text<"), "{xml}");
}

/// `\@currbox` is a BOX REGISTER (latex.ltx:17443 `\@next\@currbox
/// \@freelist`, the free list being `\newbox`es at :424/442); dpfloat.sty
/// :82-88 keys its per-box store on `\expandafter\string\@currbox`. Perl
/// (latex_constructs.pool.ltxml:1025) makes it an EMPTY macro, so
/// `\csname LP:\endcsname`-style lookups hit `\@namedef` with a `\string`
/// of nothing and the float body plus everything after it was swallowed
/// into a `\csname` scan (memoir/memman, oxref ×4: 1001 errors).
/// KNOWN_PERL_ERRORS #115.
#[test]
fn currbox_is_a_box_register() {
  let (stderr, xml) = convert(
    r"\documentclass{memoir}
\usepackage{dpfloat}
\newfloat[chapter]{tegresult}{loe}{Typeset Example}
\begin{document}
Before float.
\begin{tegresult}
Inside custom float.
\end{tegresult}
SWALLOWED text one. SWALLOWED text two. SWALLOWED text three.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("between \\csname and \\endcsname"),
    "{stderr}"
  );
  assert!(xml.contains("Inside custom float."), "{xml}");
  assert!(xml.contains("SWALLOWED text three."), "{xml}");
}

/// xspace.sty:49 lists `\@sptoken` — a pending SPACE token — among the
/// exceptions, so `\bazA[x] and` (the space after a `]`-delimited argument
/// survives) gets exactly one space. Perl's @XSPACES compares the literal
/// CS `\@sptoken` and doubles it (KNOWN_PERL_ERRORS #116). Surfaced by the
/// batch-51 non-space-skipping `\new@ifnextchar` through glossaries
/// `\gls{potato} and` (structure/glossary golden).
#[test]
fn xspace_pending_space_token_is_an_exception() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{xspace}
\def\bazA[#1]{baz#1\xspace}
\begin{document}
D \bazA[x] and E \bazA[x]{} and G.
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("D bazx and E bazx and G."), "{xml}");
  assert!(!xml.contains("bazx  and"), "{xml}");
}

/// A delimited scan that runs off the TRUE end of all input is reported
/// ONCE and then ends the job: tex.web §338 abandons the macro call after
/// the "File ended while scanning" report and §360 aborts with no `\end`
/// left to find (pdflatex: one report, emergency stop). Perl hands the
/// caller an empty argument instead, so l3prg's self-recursive
/// `\prg_map_break:Nn` (expl3-code.tex:2452-2458; reached when
/// `\prop_map_inline:cn` names an undefined prop) re-misses to the error
/// cap (Perl 100 → too_many_errors; ours 513 under tikz's raised
/// `MAX_ERRORS`). Witness: stex/stex-doc.
#[test]
fn until_miss_at_eof_is_fatal_once() {
  let (stderr, _xml) = convert(
    r"\documentclass{article}
\usepackage{expl3}
\ExplSyntaxOn
\prop_map_inline:cn { l_no_such_prop_xyz } { [#1=#2] }
\ExplSyntaxOff
\begin{document}
text
\end{document}
",
    true,
  );
  // One `Error:` (the missing Until argument) plus the one `Fatal:Mouth:EoF`.
  assert_eq!(error_count(&stderr), 2, "{stderr}");
  assert!(
    stderr.contains("Missing argument Until:\\prg_break_point:Nn at end of input"),
    "{stderr}"
  );
  assert!(stderr.contains("Fatal:Mouth:EoF"), "{stderr}");
}

/// `\input{pkg.sty}` of a package an earlier `\usepackage` already
/// raw-loaded must READ THE FILE AGAIN — real TeX's `\input` always reads,
/// and Perl re-reads it too (Package.pm:2270 `loadTeXDefinitions
/// (reloadable=>1)`). RED: the binding-only reloadable probe in `input()`
/// returned a silent `Ok` for a `_raw_loaded` file, so nothing was read
/// and tikzlings-doc's `\tex_input:D` comment harvester
/// (tikzlings-doc.tex:60-72, `\__tikzlings_process_line:w #1^^M` under
/// `\c_other_cctab`) scanned the enclosing document to its end.
#[test]
fn input_rereads_loaded_sty() {
  let (stderr, xml) = convert_with_files(
    r"\documentclass{article}
\usepackage{mypkg}
\begin{document}
A\input{mypkg.sty}B
\end{document}
",
    &[("mypkg.sty", "\\typeout{MYPKG READ}\n")],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(stderr.matches("MYPKG READ").count(), 2, "{stderr}");
  assert!(xml.contains("<p>A\nB</p>"), "{xml}");
}

/// latex.ltx:19677-19716 `\declare@file@substitution{orig}{repl}` makes
/// `\input{orig}` read `repl` (applied by `\set@curr@file`,
/// latex.ltx:19794); `\undeclare@file@substitution` restores the original.
/// pdflatex: `[REPLACEMENT ] [REPLACEMENT ] [ORIGINAL ]`. Perl's `\input`
/// binding bypasses `\set@curr@file` and re-reads the original.
/// Witness: tikzlings-doc.tex:73-75 substitutes each animal `.sty` by the
/// `\jobname.cif` comment harvest and `\input`s it — without the
/// substitution the raw `.sty` re-executes in the body and the per-animal
/// documentation is lost.
#[test]
fn input_honors_file_substitution() {
  let (stderr, xml) = convert_with_files(
    r"\documentclass{article}
\begin{document}
\makeatletter
\declare@file@substitution{orig.tex}{repl.tex}
\makeatother
[\input{orig}]
[\input{orig.tex}]
\makeatletter
\undeclare@file@substitution{orig.tex}
\makeatother
[\input{orig}]
\end{document}
",
    &[("orig.tex", "ORIGINAL\n"), ("repl.tex", "REPLACEMENT\n")],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    flat.contains("[REPLACEMENT ] [REPLACEMENT ] [ORIGINAL ]"),
    "{flat}"
  );
}

/// xcolor.sty:1033-1036: the plural `\extractcolorspecs{c}{\m}{\s}` stores
/// the BARE spec (`1,0,0`), unlike the singular `\extractcolorspec`
/// (`{rgb}{1,0,0}`), so `\definecolor{x}{\m}{\s}` round-trips. Perl's
/// xcolor.sty.ltxml:808 braces the plural too (KNOWN_PERL_ERRORS #117;
/// witness pgfPT.colorSchemes.info).
#[test]
fn extractcolorspecs_plural_is_unbraced() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\definecolor{src}{rgb}{0.5,0.25,0}
\extractcolorspecs{src}{\m}{\s}
[\m;\s]
\definecolor{dst}{\m}{\s}
\textcolor{dst}{X}
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[rgb;0.5,0.25,0]"), "{xml}");
  assert!(xml.contains("color=\"#804000\""), "{xml}");
}

/// tagpdf.sty:1594-1665 fills `\g__tag_role_NS_pdf_prop` from
/// `tagpdf-ns-pdf.def` with `{role}{}` per tag; the manual
/// (tagpdf.tex:2161-2200) lists the standard structure names by iterating
/// it. The binding must populate the same tables from the TL data files
/// rather than leave the props undefined.
#[test]
fn tagpdf_role_namespace_props_are_populated() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\usepackage{tagpdf}
\begin{document}
\ExplSyntaxOn
\clist_clear:N\l_tmpa_clist
\prop_map_inline:cn {g__tag_role_NS_pdf_prop}
  { \str_if_eq:eeT {#1} {\use_i:nn #2} { \clist_put_right:Nn \l_tmpa_clist {#1} } }
[\clist_use:Nn \l_tmpa_clist {,\c_space_tl}]
\clist_clear:N\l_tmpa_clist
\prop_map_inline:cn { g__tag_role_NS_pdf_prop }
  { \prop_if_in:cnF { g__tag_role_NS_pdf2_prop } {#1} { \clist_put_right:Nn \l_tmpa_clist {#1} } }
[\clist_use:Nn \l_tmpa_clist {,\c_space_tl}]
\ExplSyntaxOff
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let flat = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  assert!(
    flat.contains("[StructTreeRoot, Document, Part, Sect, Div, Caption,"),
    "{flat}"
  );
  // PDF 1.7 names dropped by the PDF 2.0 namespace (tagpdf-ns-pdf2.def).
  assert!(
    flat.contains("[Art, BlockQuote, TOC, TOCI, Index, Private,"),
    "{flat}"
  );
}
