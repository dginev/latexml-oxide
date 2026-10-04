use super::perfect_kernel_batch46::{convert, error_count, warning_count};

fn kpsewhich_has(name: &str) -> bool {
  std::process::Command::new("kpsewhich")
    .arg(name)
    .output()
    .map(|o| o.status.success() && !o.stdout.is_empty())
    .unwrap_or(false)
}

/// lineno.sty manual surface (witness lineno/ulineno): \linenumberwidth,
/// \bframesep, \bframerule, \linerefp, \linerefr, and bare \internallinenumbers
/// inside \parbox.
#[test]
fn lineno_manual_surface() {
  let tex = r"\documentclass{article}
\usepackage{lineno}
\begin{document}
\setlength\linenumberwidth{1cm}
\setlength\bframesep{10pt}
\setlength\bframerule{1pt}
\linenumbers
First line.\linelabel{l1}
Second line references \lineref{l1}, offset \lineref[+1]{l1}, \linerefp[+2]{l1}, \linerefr[+3]{l1}.
\begin{center}
\fbox{\parbox{0.8\textwidth}{
  \internallinenumbers \resetlinenumber[13]
  Internal linenumbers in a box.
}}
\end{center}
\begin{bframe}Framed text.\end{bframe}
\begin{internallinenumbers}Environment block.\end{internallinenumbers}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("First line.") && xml.contains("Internal linenumbers in a box."),
    "{xml}"
  );
}

/// caption hook surface for class and extension package patches
/// (witness shtthesis/shtthesis-user-guide via raw bicaption.sty):
/// \caption@beginhook, \caption@endhook, \caption@LT@setup, \caption@dblarg,
/// \captionsetup[type][subtype], and faithful \caption@ifundefined.
#[test]
fn caption_hook_surface_for_class_patches() {
  let tex = r"\documentclass{article}
\usepackage{caption}
\makeatletter
\g@addto@macro\caption@beginhook{\def\hook@ran{1}}
\g@addto@macro\caption@endhook{\def\hook@ended{1}}
\g@addto@macro\caption@LT@setup{\relax}
\caption@ifundefined\undefined@cmd{\def\undef@branch{1}}{\def\undef@branch{0}}
\caption@ifundefined\caption@beginhook{\def\def@branch{0}}{\def\def@branch{1}}
\def\test@dblarg[#1]#2{\def\dbl@got{#1:#2}}
\caption@dblarg\test@dblarg{My Title}
\captionsetup[figure][bi-second]{name=Figure}
\captionsetup*[table][bi-second]{name=Table}
\makeatother
\begin{document}
\begin{figure}
  \caption{Test caption}
\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Test caption"), "{xml}");
}

/// babel-italian ISO compliance unit definition via deferred \AtBeginDocument
/// (witness: verifica/example4.tex, example5.tex).
#[test]
fn babel_italian_iso_compliance_unit() {
  let tex = r"\documentclass{article}
\usepackage[italian]{babel}
\AtBeginDocument{
  \setISOcompliance
}
\begin{document}
$25\unit{m}$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("25"), "{xml}");
}

/// \openout, \write, \closeout, then \input within the same run via VFS,
/// including filenames formed by protected macros (witness: proof-at-the-end/proof-at-the-end_demo).
#[test]
fn openout_then_input_same_run() {
  let tex = r"\documentclass{article}
\usepackage{xparse}
\NewDocumentCommand\prefixMacro{m}{#1-vfs}
\newwrite\testout
\begin{document}
\immediate\openout\testout=\prefixMacro{\jobname}out.tex
\immediate\write\testout{Hello from VFS with protected macro}
\immediate\closeout\testout
\input{\prefixMacro{\jobname}out.tex}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Hello from VFS with protected macro"), "{xml}");
}

pub(crate) fn convert_env_args(
  tex: &str,
  extra: &[&str],
  envs: &[(&str, &str)],
) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  assert!(
    std::path::Path::new(bin).is_file(),
    "binary not staged at {bin}"
  );
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let mut args = vec![
    "t.tex",
    "--dest",
    "t.xml",
    "--nocomments",
    "--timeout=110",
    "--preload=[rawstyles,rawclasses]latexml.sty",
  ];
  args.extend_from_slice(extra);
  let mut cmd = std::process::Command::new(bin);
  cmd.args(&args).current_dir(workdir.path());
  for (k, v) in envs {
    cmd.env(k, v);
  }
  let output = cmd.output().expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  (stderr, xml)
}

/// node_boxes stays bounded under streaming (K8 memory lever): spilled subtrees
/// purge their own entries as they spill. This fixture leaves no stale entries
/// (its finishing sweep drops none), so it guards spill-time purging; the sweep
/// itself is guarded by `align_stale_node_boxes_are_swept`.
#[test]
fn spill_gated_node_boxes_stays_bounded() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\newtcolorbox{cb}{colback=red!5,colframe=red!75!black,title=Boxed}
\newcount\ct \ct=0
\begin{document}
\loop\ifnum\ct<300
  \ifnum\numexpr\ct/5*5=\ct
    \section{Section \the\ct}
  \fi
  \begin{cb}Box \the\ct\ with some text $x_{\the\ct}$.\end{cb}
  \advance\ct by 1
\repeat
\end{document}
";
  let (stderr, xml) = convert_env_args(tex, &["--streaming", "--max-memory=768"], &[(
    "LXML_TRACE_NODE_BOXES",
    "1",
  )]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  let sizes: Vec<usize> = stderr
    .lines()
    .filter_map(|line| line.rsplit_once("; node_boxes ")?.1.trim().parse().ok())
    .collect();
  assert!(!sizes.is_empty(), "no streaming progress report:\n{stderr}");
  assert!(sizes.iter().all(|&n| n < 256), "node_boxes grew: {sizes:?}");
  assert!(
    stderr.contains("node_boxes sweep"),
    "no finishing sweep:\n{stderr}"
  );
  assert_eq!(xml.matches("<picture").count(), 300, "{}", xml.len());
}

/// The backstop sweep reclaims `node_boxes` entries that a build-time discard
/// path detached without purging: alignment rearrangement leaves ~23 stale
/// entries per `align` (7,203 → 303 here). The finishing yield sweeps, so none
/// is pinned through pass 2 and the spine tail (batch 56it). This document
/// yields only when its RSS crosses the spill watermark, so the watermark is
/// pinned (`LATEXML_SPILL_AT_MIB`) well below the run's footprint (~130 MB):
/// derived from `--max-memory` (a third of the fuse, 192 MB at 768), it sat
/// 7 MB under a 199 MB run, and batch 56kp's 67 MB saving stopped the yield.
#[test]
fn align_stale_node_boxes_are_swept() {
  let body: String = (0..300)
    .map(|i| format!("Text {i}.\n\\begin{{align}}a_{{{i}}}&=b+c\\\\d&=e\\end{{align}}\n\n"))
    .collect();
  let tex = format!(
    "\\documentclass{{article}}\n\\usepackage{{amsmath}}\n\\begin{{document}}\n{body}\\end{{document}}\n"
  );
  let (stderr, xml) = convert_env_args(&tex, &["--streaming", "--max-memory=768"], &[
    ("LXML_TRACE_NODE_BOXES", "1"),
    ("LATEXML_SPILL_AT_MIB", "64"),
  ]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let dropped: usize = stderr
    .lines()
    .filter_map(|line| {
      line
        .split_once("dropped: ")?
        .1
        .split(')')
        .next()?
        .parse::<usize>()
        .ok()
    })
    .sum();
  assert!(dropped > 0, "no stale entry was swept:\n{stderr}");
  let sizes: Vec<usize> = stderr
    .lines()
    .filter_map(|line| line.rsplit_once("; node_boxes ")?.1.trim().parse().ok())
    .collect();
  assert!(
    sizes.iter().all(|&n| n < 1000),
    "node_boxes grew: {sizes:?}"
  );
  assert_eq!(xml.matches("<equation ").count(), 600, "{}", xml.len());
}

/// A resident glossary block does not make every yield sweep the whole live
/// DOM: the backstop sweep is gated on node_boxes growth, not on each spill
/// (roadmap stream C, batch 56it). The repro swept 301 times for 300 yields;
/// datatool-user ran 149.8 s → 103.3 s and glossaries-extra-manual 217.5 s →
/// 149.3 s with identical XML. Streaming output stays identical to eager.
#[test]
fn resident_glossary_does_not_sweep_every_yield() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/streaming/resident_glossary_sweeps.tex");
  let (stderr, streamed) =
    convert_env_args(tex, &["--streaming"], &[("LXML_TRACE_NODE_BOXES", "1")]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  // It must actually stream: 300 yields, reported at powers of two.
  assert!(stderr.contains("fragment 256 absorbed"), "{stderr}");
  let sweeps = stderr.matches("sweep took").count();
  assert!(sweeps <= 10, "{sweeps} sweeps (301 before 56it):\n{stderr}");
  assert_eq!(streamed.matches("<glossarydefinition").count(), 600);
  assert_eq!(streamed.matches("<picture").count(), 300);
  let (stderr, eager) = convert_env_args(tex, &[], &[]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(streamed == eager, "streaming XML differs from eager");
}

/// Native ctable binding: \ctable with keyvals, captions, tabular/tabularx,
/// rule macros (\NN, \FL, \ML, \LL), and footnotes block (\tnote, \tmark).
/// Witnesses: proofread/example, arXiv:2011.04706.
#[test]
fn ctable_native_table_with_caption() {
  let tex = r"\documentclass{article}
\usepackage{ctable}
\begin{document}
\ctable[
  botcap,
  caption=Sample Table with Ctable,
  label=tab:sample,
  pos=htbp,
  width=80mm,
]{ccc}{
  \tnote[a]{First footnote.}
  \tnote[b]{Second footnote.}
}{
  \FL
  Col 1 & Col 2 & Col 3 \ML
  A\tmark[a] & B & C\tmark[b] \NN
  D & E & F \LL
}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Sample Table with Ctable"), "{xml}");
  assert!(xml.contains("<table"), "{xml}");
  assert!(xml.contains("<tabular"), "{xml}");
  assert!(xml.contains("First footnote."), "{xml}");
}

/// Beamer frame body parameter halving (\def-collect level halving):
/// non-fragile beamer frames collect the body inside \loop ... \def\beamer@doifinframe ... \repeat,
/// requiring two levels of parameter-hash halving so that ####1 becomes #1 at definition time.
/// Witnesses: beamer-theme-albi/beamer-theme-albi-doc, tuda-ci/DEMO-TUDaBeamer.
#[test]
fn beamer_frame_hash_halving() {
  let tex = r"\documentclass{beamer}
\usepackage{etoolbox}
\begin{document}
\begin{frame}{Hash Halving Test}
  \renewcommand*{\do}[1]{[X ####1 Y]}
  \docsvlist{a,b,c}
\end{frame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[X a Y][X b Y][X c Y]"), "{xml}");
}

/// mdframed with block-level content (e.g. \printbibliography / \thebibliography)
/// mid-subsection followed by sectioning commands:
/// mdframed breaks paragraph before opening, chooses logical-block outside floats,
/// permits auto-closing so backmatter can place at section level, and auto-closes
/// gracefully without error.
/// Witness: biblatex-juradiss/biblatex-juradiss.
#[test]
fn mdframed_block_bibliography_juradiss() {
  let tex = r"\documentclass{article}
\usepackage{mdframed}
\begin{document}
\section{A}
Intro text before the frame.
\begin{mdframed}
\begin{thebibliography}{9}\bibitem{x}An entry.\end{thebibliography}
\end{mdframed}
\subsection{B}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<bibliography"), "{xml}");
  assert!(xml.contains("<subsection"), "{xml}");
  assert!(xml.contains("An entry."), "{xml}");
}

/// mdframed retains support for in-float frames (arXiv 1907.05772) and nested frames
/// (arXiv 1712.00062).
#[test]
fn mdframed_in_float_and_nested() {
  let tex = r"\documentclass{article}
\usepackage{mdframed}
\begin{document}
\begin{figure}
\begin{mdframed}
Framed float.
\end{mdframed}
\end{figure}
\begin{mdframed}
\begin{mdframed}
Nested frame.
\end{mdframed}
\end{mdframed}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Framed float."), "{xml}");
  assert!(xml.contains("Nested frame."), "{xml}");
}

/// gauss.sty `gmatrix` inside outer alignment environments (e.g. `alignat*`):
/// opens the amsmath matrix natively without gullet delimited-scan failures,
/// with row and column operations closing the inner matrix alignment cleanly.
/// Witness: tools/perfect_kernel/repros/beamer-stubs/gauss_in_alignat.tex.
#[test]
fn gauss_gmatrix_in_alignat() {
  let tex = r"\documentclass{article}
\usepackage{amsmath,gauss}
\begin{document}
\begin{alignat*}1
A=\begin{gmatrix}[p]
 1 & 1 \\
 t & 2t
\rowops
 \add[-t]{0}{1}
\end{gmatrix}&\\
\end{alignat*}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<XMArray"), "{xml}");
  assert!(
    xml.contains("←") || xml.contains("&#8592;") || xml.contains("leftarrow"),
    "{xml}"
  );
}

/// listings self-terminating environments hand \end{lstlisting} to the current \end macro
/// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn listings_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{listings}
\AfterEndEnvironment{lstlisting}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{lstlisting}
x = 1
\end{lstlisting} tail
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:lstlisting]"), "{xml}");
  assert!(xml.contains("[AFTER]"), "{xml}");
  assert!(xml.contains("tail"), "{xml}");
  assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:lstlisting]").count(), 1, "{xml}");
}

/// fancyvrb self-terminating environments hand \end{Verbatim}, \end{BVerbatim}, \end{LVerbatim}
/// to current \end macro (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn fancyvrb_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{fancyvrb}
\AfterEndEnvironment{Verbatim}{[AFTER-V]}
\AfterEndEnvironment{BVerbatim}{[AFTER-B]}
\AfterEndEnvironment{LVerbatim}{[AFTER-L]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{Verbatim}
v = 1
\end{Verbatim}
\begin{BVerbatim}
b = 1
\end{BVerbatim}
\begin{LVerbatim}
l = 1
\end{LVerbatim}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:Verbatim]"), "{xml}");
  assert!(xml.contains("[AFTER-V]"), "{xml}");
  assert!(xml.contains("[E:BVerbatim]"), "{xml}");
  assert!(xml.contains("[AFTER-B]"), "{xml}");
  assert!(xml.contains("[E:LVerbatim]"), "{xml}");
  assert!(xml.contains("[AFTER-L]"), "{xml}");
  assert_eq!(xml.matches("[AFTER-V]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:Verbatim]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[AFTER-B]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:BVerbatim]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[AFTER-L]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:LVerbatim]").count(), 1, "{xml}");
}

/// minted self-terminating environments hand \end{minted} to current \end macro
/// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn minted_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{minted}
\AfterEndEnvironment{minted}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{minted}{python}
x = 1
\end{minted} tail
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:minted]"), "{xml}");
  assert!(xml.contains("[AFTER]"), "{xml}");
  assert!(xml.contains("tail"), "{xml}");
  assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:minted]").count(), 1, "{xml}");
}

/// comment.sty self-terminating environments hand \end{comment} to current \end macro
/// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn comment_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{comment}
\AfterEndEnvironment{comment}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{comment}
ignored
\end{comment}
Tail.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:comment]"), "{xml}");
  assert!(xml.contains("[AFTER]") && xml.contains("Tail."), "{xml}");
  assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:comment]").count(), 1, "{xml}");
  // comment.sty ends only on a WHOLE `\end{comment}` line (pdflatex aborts on
  // `\end{comment} tail`: the comment runs to EOF); we report TeX's error.
  let midline = tex.replace("\\end{comment}\n", "\\end{comment} tail\n");
  let (stderr, xml) = convert(&midline, true);
  assert!(stderr.contains("File ended while scanning"), "{stderr}");
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(!xml.contains("Tail."), "{xml}");
}

/// verbatim.sty self-terminating environments hand \end{verbatim} to current \end macro
/// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn verbatim_sty_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{verbatim}
\AfterEndEnvironment{verbatim}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{verbatim}
x = 1
\end{verbatim}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:verbatim]"), "{xml}");
  assert!(xml.contains("[AFTER]"), "{xml}");
  assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:verbatim]").count(), 1, "{xml}");
}

/// alltt self-terminating environments hand \end{alltt} to current \end macro
/// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn alltt_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{alltt}
\AfterEndEnvironment{alltt}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{alltt}
x = 1
\end{alltt} tail
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:alltt]"), "{xml}");
  assert!(xml.contains("[AFTER]"), "{xml}");
  assert!(xml.contains("tail"), "{xml}");
  assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:alltt]").count(), 1, "{xml}");
}

/// tcolorbox dispListing self-terminating environments hand \end{dispListing} to current \end macro
/// (witness: s44 manuals with hooked \end, \AfterEndEnvironment, knowledge scope areas).
#[test]
fn tcolorbox_self_terminating_hands_to_end() {
  let tex = r"\documentclass{article}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\usepackage{etoolbox}
\AfterEndEnvironment{dispListing}{[AFTER]}
\let\SUPERend\end
\def\end#1{\SUPERend{#1}[E:#1]}
\begin{document}
\begin{dispListing}
x = 1
\end{dispListing} tail
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[E:dispListing]"), "{xml}");
  assert!(xml.contains("[AFTER]"), "{xml}");
  assert!(xml.contains("tail"), "{xml}");
  assert_eq!(xml.matches("[AFTER]").count(), 1, "{xml}");
  assert_eq!(xml.matches("[E:dispListing]").count(), 1, "{xml}");
}

/// endnotes internals and raw-load overlay for cmsendnotes / biblatex-chicago
/// (witness: biblatex-chicago/cms-noteref-demo, cms-notes-intro, cms-notes-sample; s44).
/// Exposes \@enotes, \if@enotesopen, \@openenotes, \@doanenote, \@endanenote,
/// \enotesize, \enoteformat, \enoteheading, \theendnotes with .ent replay,
/// and biblatex \MakeCapital + xstring dependency.
#[test]
fn endnotes_internals_and_cmsendnotes_overlay() {
  let tex = r"\documentclass{article}
\usepackage{biblatex-chicago}
\usepackage[split=section]{cmsendnotes}
\begin{document}
\section{First Section}
Some text with an endnote.\endnote{This is the first endnote.}
Another sentence.\endnote{Second endnote.}
\theendnotesbypart
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("This is the first endnote."), "{xml}");
  assert!(xml.contains("Second endnote."), "{xml}");
}

/// standard endnotes.sty standalone with \theendnotes TOC output
/// (tests/structure/endnote.xml).
#[test]
fn endnotes_standard_standalone() {
  let tex = r"\documentclass{article}
\usepackage{endnotes}
\begin{document}
Some text with an endnote.\endnote{This is an endnote.}
\theendnotes
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<TOC"), "{xml}");
  assert!(xml.contains("This is an endnote."), "{xml}");
}

/// istgame and TikZ trees child nodes (\tikzparentnode, \tikzchildnode,
/// tikzlibrarytrees, and tcolorbox !O{} listing signature without space-skipping).
#[test]
fn istgame_and_tikz_trees_child_nodes() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{trees}
\usepackage{tcolorbox}
\tcbuselibrary{listings}
\usepackage{istgame}
\DeclareTCBListing{docplain}{ !O{} }{colback=white,colframe=gray!15,listing only,#1}
\begin{document}
\begin{docplain}
  % tikz-qtree conflict resolution (only with \usepackage{tikz-qtree})
  [
    edge from parent path={(\tikzparentnode) -- (\tikzchildnode)}
  ]
\end{docplain}
\begin{tikzpicture}[edge from parent path={(\tikzparentnode) -- (\tikzchildnode)}]
\node {root}
  child { node {left} }
  child { node {right} };
\end{tikzpicture}
\begin{istgame}
\istroot(0){Alice}
  \istb{L}[al]{(-1,1)}
  \istb{R}[ar]{(1,-1)}
  \endist
\end{istgame}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("tikzparentnode"), "{xml}");
  assert!(xml.contains("tikzchildnode"), "{xml}");
  assert!(xml.contains("root"), "{xml}");
  assert!(xml.contains("Alice"), "{xml}");
}

/// expkv \ekvcsvloop delimiter matching with adjacent spaces
/// (witness expkv-bundle/expkv-bundle: \ekv@stop undefined and TokenLimit fatal):
/// \ekv@csv@loop@end matches literal delimiter prefix tokens containing
/// adjacent space tokens (\ekv@mark  \ekv@nil). read_match must not greedily
/// swallow subsequent spaces from input when to_match expects another space.
#[test]
fn expkv_ekvcsvloop_delimiter_adjacent_spaces() {
  let tex = r"\documentclass{article}
\usepackage{expkv}
\newcommand*\myprocessor[1]{(#1)}
\begin{document}
\ekvcsvloop\myprocessor{abc,def,ghi}
\ekvcsvloop\myprocessor{1,,2,,3,,4}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("(abc)(def)(ghi)"), "{xml}");
  assert!(xml.contains("(1)(2)(3)(4)"), "{xml}");
  // Control (the preserved Perl Gullet.pm:614 branch): a LEADING literal
  // delimiter compiles to `Match:` → `read_match` (a delimiter after `#n` is
  // `Until:` and never reaches it); its space token followed by a non-space
  // still matches plain source.
  let control = r"\documentclass{article}
\def\foo a b#1{[#1]}
\begin{document}
\foo a b Z.
\end{document}
";
  let (stderr, xml) = convert(control, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[Z]."), "{xml}");
}

/// xy curve option and curved arrow handling (witness: amshelp manual)
/// \usepackage[curve]{xy} or \usepackage[all,cmtip]{xy} loads xycurve;
/// marks xycurveloaded, neutralizes \curve@check, and defines \curve inside
/// xymatrix arrows with minimal real semantics (renders as the arrow without error).
#[test]
fn xy_curve_option_and_curved_arrows() {
  let tex = r"\documentclass{article}
\usepackage[all,cmtip]{xy}
\begin{document}
\begin{displaymath}
  \xymatrix{
    {A} \ar@/^/[drr]^{p} \ar@{.>}[dr]|{\exists!} \ar@/_/[ddr]_{q}\\
    & {B} \ar[r] \ar[d]
    & {C} \ar[d]\\
    & {D} \ar[r]
    & {E}
  }
\end{displaymath}
\begin{displaymath}
  \xymatrix{
    {X} \ar \curve{+(0,2)} [r] & {Y}
  }
\end{displaymath}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    !stderr.contains("Info:xy:error"),
    "stderr had xy error:\n{stderr}"
  );
  assert!(xml.contains("<svg:svg"), "{xml}");
  assert!(
    xml.contains("XMTok font=\"italic\" role=\"UNKNOWN\">A</XMTok>"),
    "{xml}"
  );
  assert!(
    xml.contains("XMTok font=\"italic\" role=\"UNKNOWN\">Y</XMTok>"),
    "{xml}"
  );

  // Control: standard xymatrix without curve option unchanged
  let control_tex = r"\documentclass{article}
\usepackage{xy}
\xyoption{matrix}
\xyoption{arrow}
\begin{document}
\begin{displaymath}
  \xymatrix{
    {A} \ar[r]^f & {B}
  }
\end{displaymath}
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("<svg:svg"), "{c_xml}");
  assert!(
    c_xml.contains("XMTok font=\"italic\" role=\"UNKNOWN\">B</XMTok>"),
    "{c_xml}"
  );
}

/// CJK active UTF-8 octet binding (witness: kotex cjk_envstart_protect_utf8_octets).
/// Under CJK UTF8, active octets 0x80..0xF4 are given valid expansion meanings
/// without modifying their catcodes (retaining Catcode::OTHER for Latin accents).
#[test]
fn cjk_utf8_active_octets_binding() {
  let tex = r"\documentclass{article}
\usepackage[cjk,hangul]{kotex}
\begin{document}
소개
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("소개"), "{xml}");

  // Control: Latin accents with plain CJK
  // Control under the SAME octet bindings (kotex installs them): the
  // accented Latin code points keep their catcode-12 identity.
  let latin_tex = r"\documentclass{article}
\usepackage[cjk,hangul]{kotex}
\begin{document}
café résumé
\end{document}
";
  let (l_stderr, l_xml) = convert(latin_tex, true);
  assert_eq!(error_count(&l_stderr), 0, "{l_stderr}");
  assert!(l_xml.contains("café résumé"), "{l_xml}");
}

/// srdp-tables.sty routes directly to tabu binding (witness: srdp-mathematik).
#[test]
fn srdp_tables_routes_to_tabu() {
  let tex = r"\documentclass{article}
\usepackage{srdp-tables}
\begin{document}
\begin{tabu}{cc}
a & b \\
1 & 2 \\
\end{tabu}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<tabular"), "{xml}");
  assert!(xml.contains("1") && xml.contains("2"), "{xml}");

  // Control: tabu directly
  let control_tex = r"\documentclass{article}
\usepackage{tabu}
\begin{document}
\begin{tabu}{cc}
a & b \\
1 & 2 \\
\end{tabu}
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("<tabular"), "{c_xml}");
}

/// oup-authoring-template class constructs (witness: oup-authoring-template.tex).
#[test]
fn oup_authoring_template_constructs() {
  let tex = r"\documentclass{oup-authoring-template}
\begin{document}
\address[1]{\orgaddress{\state{California}}}
\begin{table}
\caption{Table}\label{tab}
\begin{tabular}{cc}
\toprule
a & b \\
\botrule
\end{tabular}
\begin{tablenotes}
\item Note
\end{tablenotes}
\end{table}
\begin{algorithm}
\caption{Alg}\label{alg}
\begin{algorithmic}[1]
\State $x \Leftarrow 1$
\end{algorithmic}
\end{algorithm}
\begin{unlist}
\item item
\end{unlist}
\begin{appendices}
\section{App}
\end{appendices}
\begin{biography}{}{\author{Author.} Bio text}
\end{biography}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("California"), "{xml}");
  assert!(xml.contains("<tabular"), "{xml}");
  assert!(xml.contains("Bio text"), "{xml}");

  // Control: standard article
  let control_tex = r"\documentclass{article}
\begin{document}
Hello world
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("Hello world"), "{c_xml}");
}

/// `$\LaTeXe$` in ejpecp (sample.tex:128, :171): the logo constructor's
/// `</ltx:text>` after its content auto-closed the text is a scoped close
/// (batch 56x, DIVERGENCES #202) — no class-level `\mbox` wrapping (which
/// collapses the whole formula into a marked-as-math text node).
#[test]
fn ejpecp_latexe_math_mode() {
  let tex = r"\documentclass{ejpecp}
\begin{document}
$\LaTeXe$ and $\LaTeX$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("ltx_LaTeX_logo"), "{xml}");
  assert!(xml.contains("<Math"), "{xml}");
  assert!(!xml.contains("ltx_markedasmath"), "{xml}");

  // Control: text mode in standard article works without error
  let control_tex = r"\documentclass{article}
\begin{document}
\LaTeXe\ and \LaTeX
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("ltx_LaTeX_logo"), "{c_xml}");
}

/// verbatim.sty: \verbatim@readfile (witness: kotex-utf/kotex-utf-doc.tex).
/// Tests reading an external/filecontents file through \verbatim@readfile,
/// with \verbatiminput as control twin.
#[test]
fn verbatim_readfile_macro() {
  let tex = r"\begin{filecontents*}{readfile_test.txt}
hello verbatim world
line 2
\end{filecontents*}
\documentclass{article}
\usepackage{verbatim}
\begin{document}
\makeatletter
\verbatim@readfile{readfile_test.txt}
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("hello verbatim world"), "{xml}");
  assert!(xml.contains("line 2"), "{xml}");

  // Control: standard \verbatiminput on the same file
  let control_tex = r"\begin{filecontents*}{readfile_test2.txt}
hello verbatim world
line 2
\end{filecontents*}
\documentclass{article}
\usepackage{verbatim}
\begin{document}
\verbatiminput{readfile_test2.txt}
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("hello verbatim world"), "{c_xml}");
}

/// xcolor: \XC@getcolor & \XC@usecolor (L1, witness dsptricks/dspTricksManual).
/// Verifies \XC@getcolor yields xcolor's real `\xcolor@…` shape in the target macro,
/// \XC@usecolor consumes the color argument without error, and that loading
/// pstricks after/with xcolor aliases \pst@getcolor / \pst@usecolor to them.
#[test]
fn xcolor_pst_getcolor_and_usecolor() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\usepackage{pstricks}
\begin{document}
\makeatletter
\XC@getcolor{red}\mycolorA
\pst@getcolor{blue}\mycolorB
\XC@usecolor\mycolorA
\pst@usecolor\mycolorB
ColorA:\mycolorA;ColorB:\mycolorB.
\makeatother
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // xcolor.sty:1373-1396: `\cs` holds `\xcolor@{}{drv}{model}{spec}` and this
  // binding's `\xcolor@` expands to the driver spec.
  assert!(xml.contains("ColorA:1,0,0;ColorB:0,0,1."), "{xml}");

  // Control: standard \definecolor + \color still emits color attribute
  let control_tex = r"\documentclass{article}
\usepackage{xcolor}
\definecolor{mytestcolor}{rgb}{1,0,0}
\begin{document}
{\color{mytestcolor}Hello Red World}
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("color=\"#FF0000\""), "{c_xml}");
  assert!(c_xml.contains("Hello Red World"), "{c_xml}");
}

/// etoolbox: \AtBeginEnvironment & co. routing to lthooks env hooks (L2 / K3 step)
/// with label support, firing in lthooks order, plus verbatim private store compatibility.
#[test]
fn etoolbox_env_hooks_onto_lthooks() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\BeforeBeginEnvironment{center}{[BEFORE-C]}
\AtBeginEnvironment[label1]{center}{[BEGIN-C1]}
\AtBeginEnvironment[label2]{center}{[BEGIN-C2]}
\AtEndEnvironment{center}{[END-C]}
\AfterEndEnvironment{center}{[AFTER-C]}
\begin{document}
\begin{center}
Center text.
\end{center}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[BEFORE-C]"), "{xml}");
  assert!(xml.contains("[BEGIN-C1]"), "{xml}");
  assert!(xml.contains("[BEGIN-C2]"), "{xml}");
  assert!(xml.contains("[END-C]"), "{xml}");
  assert!(xml.contains("[AFTER-C]"), "{xml}");
  assert!(xml.contains("Center text."), "{xml}");

  // Control: center environment without hooks
  let control_tex = r"\documentclass{article}
\begin{document}
\begin{center}
Control center text.
\end{center}
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("Control center text."), "{c_xml}");
}

#[test]
fn apptocmd_pretocmd_constructor_env_and_control() {
  // 1. Repro case from tools/perfect_kernel/repros/expansion-primitives/apptocmd_endminipage_error.tex:
  // \apptocmd on constructor-backed \endminipage takes success branch, not {\ERROR}.
  let repro_tex = r"\documentclass{article}
\usepackage{etoolbox}
\begin{document}
\apptocmd{\endminipage}{\relax}{}{\ERROR}%
ok
\end{document}
";
  let (r_stderr, r_xml) = convert(repro_tex, true);
  assert_eq!(error_count(&r_stderr), 0, "{r_stderr}");
  assert!(r_xml.contains("<p>ok</p>"), "{r_xml}");

  // 2. Functional test: \apptocmd and \pretocmd on \endminipage correctly append and prepend
  // to the environment's end hook (with pretocmd running first).
  let mp_tex = r"\documentclass{article}
\usepackage{etoolbox}
\apptocmd{\endminipage}{[MP-APP]}{}{}
\pretocmd{\endminipage}{[MP-PRE]}{}{}
\begin{document}
\begin{minipage}{5cm}
Inside minipage
\end{minipage}
\end{document}
";
  let (mp_stderr, mp_xml) = convert(mp_tex, true);
  assert_eq!(error_count(&mp_stderr), 0, "{mp_stderr}");
  assert!(mp_xml.contains("[MP-PRE][MP-APP]"), "{mp_xml}");

  // 3. Control case: \apptocmd and \pretocmd on a plain \def macro still patch the macro body.
  let ctrl_tex = r"\documentclass{article}
\usepackage{etoolbox}
\def\mymacro{HELLO}
\pretocmd{\mymacro}{[PRE-]}{}{}
\apptocmd{\mymacro}{[-APP]}{}{}
\begin{document}
\mymacro
\end{document}
";
  let (ctrl_stderr, ctrl_xml) = convert(ctrl_tex, true);
  assert_eq!(error_count(&ctrl_stderr), 0, "{ctrl_stderr}");
  assert!(ctrl_xml.contains("[PRE-]HELLO[-APP]"), "{ctrl_xml}");
}

/// enumitem.sty:705-725 list-level `before=`/`after=`/`first=` key code
/// (rec-thy.sty:574 \setlist[pfcasesnonum,1]{before=\def\pfcasecounter@pmg{…}}).
#[test]
fn enumitem_before_after_first_key_code() {
  // 1. Witness case: \newlist + \setlist[...,1]{before=...} defines macro read inside items.
  let witness_tex = r"\documentclass{article}
\usepackage{enumitem}
\newlist{pfcasesnonum}{enumerate}{3}
\setlist[pfcasesnonum,1]{
    before=\def\pfcasecounter@pmg{pfcasesnonumi},
}
\begin{document}
\begin{pfcasesnonum}
\item \pfcasecounter@pmg
\end{pfcasesnonum}
\end{document}
";
  let (w_stderr, w_xml) = convert(witness_tex, false);
  assert_eq!(error_count(&w_stderr), 0, "{w_stderr}");
  assert!(w_xml.contains("pfcasesnonumi"), "{w_xml}");

  // 2. Inline key execution: before, before*, first, after.
  let keys_tex = r"\documentclass{article}
\usepackage{enumitem}
\begin{document}
\begin{itemize}[before=\def\testb{B1},before*=\def\testbb{B2},first=\def\testf{F},after=\def\testa{A}]
\item \testb-\testbb-\testf
\end{itemize}
\testa
\end{document}
";
  let (k_stderr, k_xml) = convert(keys_tex, false);
  assert_eq!(error_count(&k_stderr), 0, "{k_stderr}");
  assert!(k_xml.contains("B1-B2-F"), "{k_xml}");
  assert!(k_xml.contains("<p>A</p>"), "{k_xml}");

  // 3. Control case: label= and itemsep= guards unchanged.
  let ctrl_tex = r"\documentclass{article}
\usepackage{enumitem}
\begin{document}
\begin{enumerate}[label=(\alph*),itemsep=2pt]
\item First
\item Second
\end{enumerate}
\end{document}
";
  let (c_stderr, c_xml) = convert(ctrl_tex, false);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("(a)"), "{c_xml}");
  assert!(c_xml.contains("(b)"), "{c_xml}");
}

#[test]
fn font_size_currsize_maintenance() {
  // Task L5: font-size commands maintain \@currsize and initial document size
  // is \let to \normalsize so \ifx\@currsize\normalsize chains succeed.

  // 1. Initial document state equals \normalsize without explicit switch.
  let init_tex = r"\documentclass{article}
\makeatletter
\begin{document}
\ifx\@currsize\normalsize Y\else N\fi
\end{document}
";
  let (i_stderr, i_xml) = convert(init_tex, false);
  assert_eq!(error_count(&i_stderr), 0, "{i_stderr}");
  assert!(i_xml.contains("<p>Y</p>"), "{i_xml}");

  // 2. Switching to \small updates \@currsize to \small.
  let small_tex = r"\documentclass{article}
\makeatletter
\begin{document}
\small\ifx\@currsize\small Y\else N\fi
\end{document}
";
  let (s_stderr, s_xml) = convert(small_tex, false);
  assert_eq!(error_count(&s_stderr), 0, "{s_stderr}");
  assert!(s_xml.contains("Y"), "{s_xml}");

  // 3. Control: \normalsize branch.
  let norm_tex = r"\documentclass{article}
\makeatletter
\begin{document}
\small small text
\normalsize\ifx\@currsize\normalsize Y\else N\fi
\end{document}
";
  let (n_stderr, n_xml) = convert(norm_tex, false);
  assert_eq!(error_count(&n_stderr), 0, "{n_stderr}");
  assert!(n_xml.contains("Y"), "{n_xml}");

  // 4. Scoped group restoration: {\small ...} restores \normalsize on group exit.
  let grp_tex = r"\documentclass{article}
\makeatletter
\begin{document}
{\small\ifx\@currsize\small S\fi}\ifx\@currsize\normalsize N\fi
\end{document}
";
  let (g_stderr, g_xml) = convert(grp_tex, false);
  assert_eq!(error_count(&g_stderr), 0, "{g_stderr}");
  assert!(g_xml.contains("S"), "{g_xml}");
  assert!(g_xml.contains("N"), "{g_xml}");
}

/// pdfpages.sty:262 `\includepdfmerge[opts]{file-page-list}` delegates each
/// comma-separated file and optional page spec to `\includepdf`.
/// Witness: latex-refsheet/LaTeX_RefSheet.tex:1395 (Task L6).
#[test]
fn pdfpages_includepdfmerge_multi_and_opts() {
  let tex = r"\documentclass{article}
\usepackage{pdfpages}
\begin{document}
\includepdfmerge[pages=-, nup=5x2, frame=true, scale=0.97]{thesis.pdf, 1-9, acknowledgements.pdf}
\includepdfmerge{single.pdf}
\includepdfmerge{docA.pdf, 1, docB.pdf, 2-, docC.pdf}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // thesis.pdf with page spec 1-9
  assert!(
    xml.contains(r#"<resource src="thesis.pdf" type="application/pdf"/>"#),
    "{xml}"
  );
  assert!(xml.contains("pages 1-9 of "), "{xml}");
  assert!(
    xml.contains(r#"<ref href="thesis.pdf">thesis.pdf</ref>"#),
    "{xml}"
  );
  // acknowledgements.pdf with default pages=- from options
  assert!(
    xml.contains(r#"<resource src="acknowledgements.pdf" type="application/pdf"/>"#),
    "{xml}"
  );
  assert!(xml.contains("pages - of "), "{xml}");
  assert!(
    xml.contains(r#"<ref href="acknowledgements.pdf">acknowledgements.pdf</ref>"#),
    "{xml}"
  );
  // single.pdf with no options
  assert!(
    xml.contains(r#"<resource src="single.pdf" type="application/pdf"/>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<ref href="single.pdf">single.pdf</ref>"#),
    "{xml}"
  );
  // docA.pdf with page 1, docB.pdf with pages 2-, docC.pdf with no pages
  assert!(
    xml.contains(r#"<resource src="docA.pdf" type="application/pdf"/>"#),
    "{xml}"
  );
  assert!(xml.contains("pages 1 of "), "{xml}");
  assert!(
    xml.contains(r#"<resource src="docB.pdf" type="application/pdf"/>"#),
    "{xml}"
  );
  assert!(xml.contains("pages 2- of "), "{xml}");
  assert!(
    xml.contains(r#"<resource src="docC.pdf" type="application/pdf"/>"#),
    "{xml}"
  );
}

/// bookmark.sty / hyperref.sty: `\bookmark[options]{text}` and `\bookmarksetup{options}`
/// are PDF outline metadata macros that become no-ops in XML/HTML conversion without
/// emitting navigation elements or erroring.
/// Witness: tagpdf/tagpdf.tex:129 (Task L7).
#[test]
fn bookmark_and_setup_absorbed_in_hyperref_and_bookmark() {
  // 1. In hyperref (witness usage where bookmark is implicitly available)
  let hyp_tex = r"\documentclass{article}
\usepackage{hyperref}
\begin{document}
\bookmarksetup{depth=2}
\bookmarksetupnext{level=section}
\bookmark[dest=toc,level=section]{Table of Contents}
\bookmark{Unadorned Bookmark}
\bookmarkdefinestyle{mystyle}{color=blue}
\bookmarkget{dest}
\BookmarkAtEnd{\bookmark{End Bookmark}}
\section{First Section}
Hello
\end{document}
";
  let (hyp_stderr, hyp_xml) = convert(hyp_tex, false);
  assert_eq!(error_count(&hyp_stderr), 0, "{hyp_stderr}");
  assert!(hyp_xml.contains("First Section"), "{hyp_xml}");
  assert!(!hyp_xml.contains("<ltx:navigation"), "{hyp_xml}");

  // 2. In bookmark.sty
  let bkm_tex = r"\documentclass{article}
\usepackage{bookmark}
\begin{document}
\bookmarksetup{depth=2}
\bookmark[dest=toc,level=section]{Table of Contents}
\bookmark{Unadorned Bookmark}
\end{document}
";
  let (bkm_stderr, _bkm_xml) = convert(bkm_tex, false);
  assert_eq!(error_count(&bkm_stderr), 0, "{bkm_stderr}");
}

/// uspatent.cls: redefines \maketitle to run \patentTitlePage and \patentStart,
/// the latter of which defines `\newcounter{parnum}` (used by \patentParagraph).
/// Kernel \maketitle is locked, which dropped the redefinition, leaving `parnum`
/// undefined (`undefined:\theparnum`, `undefined:counter:parnum`).
/// Witness: uspatent/PatentApplication.tex (Task L9).
#[test]
fn uspatent_maketitle_defines_parnum_counter() {
  let tex = r"\documentclass{uspatent}
\begin{document}
\title{Test Patent}
\author{Test Inventor}
\maketitle
\patentParagraph First paragraph.
\patentParagraph Second paragraph.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("0001"), "{xml}");
  assert!(xml.contains("0002"), "{xml}");
  assert!(xml.contains("First paragraph."), "{xml}");
  assert!(xml.contains("Second paragraph."), "{xml}");
}

/// \maketitle honours class's dropped body after structured frontmatter
/// (witness: uspatent/PatentApplication, PatentApplicationGuide).
/// uspatent's binding unlocks `\maketitle` so the class's own
/// `\renewcommand{\maketitle}{\patentTitlePage\patentStart}` (uspatent.cls:188-191)
/// runs and defines the `parnum` counter; the article control keeps the
/// kernel `\maketitle`. (A generic raw replay of any dropped body was reverted
/// at the round-7 merge: resphilosophica.cls:331 needs amsart internals.)
#[test]
fn maketitle_executes_dropped_class_body_after_frontmatter() {
  let tex = r"\documentclass{uspatent}
\title{Test Patent}
\author{Test Inventor}
\begin{document}
\maketitle
\patentParagraph First paragraph.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Test Patent"), "{xml}");
  assert!(xml.contains("Test Inventor"), "{xml}");
  assert!(xml.contains("First paragraph."), "{xml}");
  assert!(xml.contains("0001"), "{xml}");

  // Control: standard article where \maketitle redefinition only adds \thispagestyle{empty}
  let control_tex = r"\documentclass{article}
\renewcommand{\maketitle}{\thispagestyle{empty}}
\title{Foo}
\author{Bar}
\begin{document}
\maketitle
Hello world.
\end{document}
";
  let (c_stderr, c_xml) = convert(control_tex, true);
  assert_eq!(error_count(&c_stderr), 0, "{c_stderr}");
  assert!(c_xml.contains("Foo"), "{c_xml}");
  assert!(c_xml.contains("Bar"), "{c_xml}");
  assert!(c_xml.contains("Hello world."), "{c_xml}");
}

/// xcolor \XC@getcolor normalises color spec to \xcolor@ {}{<drv_spec>}{<model>}{<spec_comma>}
/// (witness: dsptricks/dspTricksManual; oracle: pdflatex \meaning\x).
#[test]
fn xcolor_getcolor_faithful_normalization() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\begin{document}
\makeatletter
\XC@getcolor{red!50}\x
\typeout{MEANING_XC=\meaning\x}
\pst@getcolor{red!50}\y
\typeout{MEANING_PST=\meaning\y}
\XC@usecolor\x
\makeatother
\end{document}
";
  let (stderr, _xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The binding's `\XC@getcolor` (xcolor_sty.rs, batch 56aa) yields the real
  // contract shape `\xcolor@{}{drv}{model}{spec}` with the model spec standing
  // in for the driver literal (our colours are attributes, not `rg`/`RG`
  // operators); pstricks aliases `\pst@getcolor` to it when xcolor is loaded.
  assert!(
    stderr.contains(r"MEANING_XC=macro:->\xcolor@ {}{1,0.5,0.5}{rgb}{1,0.5,0.5}"),
    "{stderr}"
  );
  assert!(
    stderr.contains(r"MEANING_PST=macro:->\xcolor@ {}{1,0.5,0.5}{rgb}{1,0.5,0.5}"),
    "{stderr}"
  );
}

/// algpseudocodex.sty binding: clean inline comments, LComment, and boxed blocks
/// (witness: arXiv 2511.21969; Divergence #214).
#[test]
fn algpseudocodex_produces_clean_comments_and_boxes() {
  let tex = r"\documentclass{article}
\usepackage[italicComments=false]{algpseudocodex}
\begin{document}
\begin{algorithmic}[1]
\State $x \gets 1$ \Comment{First comment}
\LComment{Wide comment}
\BeginBox[draw=blue,dashed,thick]
\If{$x > 0$}
  \State $y \gets 2$
\EndBox
\EndIf
\State \BoxedString[draw=red]{boxed text}
\end{algorithmic}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // Comment sits in the \State's own <listingline>
  assert!(
    xml.contains("<listingline xml:id=\"algx1.l1\">"),
    "missing first listingline:\n{xml}"
  );
  assert!(
    xml.contains("ltx_algpx_comment"),
    "expected right-flushed comment:\n{xml}"
  );
  assert!(xml.contains("First comment"), "{xml}");
  // `[italicComments=false]` reached its handler (algpseudocodex.sty:43): the
  // comment text is not italic.
  assert!(
    !xml.contains("font=\"italic\">First comment") && !xml.contains("font=\"italic\">Wide comment"),
    "{xml}"
  );
  // LComment is on its own listingline with delimiters
  assert!(
    xml.contains("<listingline xml:id=\"algx1.l2\">"),
    "missing LComment listingline:\n{xml}"
  );
  assert!(xml.contains("Wide comment"), "{xml}");
  // Box styling:
  assert!(xml.contains("ltx_border_blue"), "expected blue box:\n{xml}");
  assert!(xml.contains("ltx_dashed"), "expected dashed box:\n{xml}");
  assert!(xml.contains("ltx_thick"), "expected thick box:\n{xml}");
  assert!(
    xml.contains("ltx_border_red"),
    "expected inline red box:\n{xml}"
  );
}

/// algpseudocodex defensive fallback when old algorithmic.sty loaded first
/// (algorithmicx bails, leaving \algrenewcomment undefined; witness: 2410.03000).
#[test]
fn algpseudocodex_defensive_when_algorithmic_loaded_first() {
  let tex = r"\documentclass{article}
\usepackage{algorithmic}
\usepackage{algpseudocodex}
\begin{document}
\begin{algorithmic}
\STATE $x \leftarrow 1$
\end{algorithmic}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains("<listingline") && xml.contains("<Math"),
    "{xml}"
  );
}

/// algpseudocodex.sty completeness: indLines, spaceRequire, keywords, structures, statements, and comments.
#[test]
fn algpseudocodex_completeness_and_controls() {
  // 1. With indLines=true: <listing> has ltx_algpx_indlines
  let tex_true = r"\documentclass{article}
\usepackage[indLines=true]{algpseudocodex}
\begin{document}
\begin{algorithmic}[1]
\Require input A
\Require input B
\Structure{Point}
  \Properties
    \State $x, y$
  \EndProperties
  \Methods
    \State \Call{Dist}{$p$}
  \EndMethods
\EndStructure
\Class{Graph}
  \State \Return 0
  \State \Output \Call{Find}{$v$}
\EndClass
\State $x \gets 1$ \Comment{Inline}
\end{algorithmic}
\end{document}
";
  let (stderr_true, xml_true) = convert(tex_true, true);
  assert_eq!(error_count(&stderr_true), 0, "{stderr_true}");
  assert!(
    xml_true.contains("class=\"ltx_algpx_indlines\""),
    "{xml_true}"
  );
  assert!(xml_true.contains(">structure<"), "{xml_true}");
  assert!(xml_true.contains(">properties<"), "{xml_true}");
  assert!(xml_true.contains(">methods<"), "{xml_true}");
  assert!(xml_true.contains(">class<"), "{xml_true}");
  assert!(xml_true.contains(">return<"), "{xml_true}");
  assert!(xml_true.contains(">output<"), "{xml_true}");
  assert!(xml_true.contains(">Dist<"), "{xml_true}");
  assert!(xml_true.contains(">Find<"), "{xml_true}");

  // 2. Control: indLines=false: <listing> does NOT have ltx_algpx_indlines
  let tex_false = r"\documentclass{article}
\usepackage[indLines=false]{algpseudocodex}
\begin{document}
\begin{algorithmic}
\State $x \gets 1$
\end{algorithmic}
\end{document}
";
  let (stderr_false, xml_false) = convert(tex_false, true);
  assert_eq!(error_count(&stderr_false), 0, "{stderr_false}");
  assert!(!xml_false.contains("ltx_algpx_indlines"), "{xml_false}");

  // 3. Control: rightComments=false converts cleanly without malformed descendant errors
  let tex_comm = r"\documentclass{article}
\usepackage[rightComments=false]{algpseudocodex}
\begin{document}
\begin{algorithmic}
\State $x \gets 1$ \Comment{Inline comment}
\end{algorithmic}
\end{document}
";
  let (stderr_comm, xml_comm) = convert(tex_comm, true);
  assert_eq!(error_count(&stderr_comm), 0, "{stderr_comm}");
  assert!(xml_comm.contains("Inline comment"), "{xml_comm}");
}

/// lltjfont fontfamily redefined without leaking trailing arguments into gullet (witness: kksymbols/kksymbols-doc).
#[test]
fn kksymbols_fontfamily_no_text_leak() {
  let tex = r"\documentclass[luatex,fontsize=10pt,paper=b5,twoside]{jlreq}
\usepackage{KKsymbols}
\usepackage{listings}
\begin{document}
\begin{lstlisting}
hello
\end{lstlisting}
\end{document}
";
  let (stderr, xml) = super::perfect_kernel_batch46::convert_with(
    tex,
    Some("[rawstyles,rawclasses,luatex]latexml.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("hello"), "{xml}");
  assert!(!xml.contains("cmtttrue"), "{xml}");
}

/// l3backend-dvips pagecount hook prevents "Cannot run piped system commands" (witness: notebeamer/notebeamer-demo).
/// `\pdfoutput=0` selects the dvips backend: the K6 PDF-mode default loads
/// l3backend-pdftex, which never runs the hook.
#[test]
fn notebeamer_pagecount_dvips_fallback() {
  let tex = r"\pdfoutput=0
\documentclass{article}
\usepackage{notebeamer}
\begin{document}
\includebeamer[nup=1,pages=1]{example-image-a4.pdf}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<graphics"), "{xml}");
}

/// hypdestopt binding and svn-multi's \svnrev, \svnmonth, \svnauthor (witness: biblatex-cheatsheet/biblatex-cheatsheet).
/// svn-multi loads raw since 57ah (the stub's intrinsic `missing_file` warning is gone).
#[test]
fn biblatex_cheatsheet_hypdestopt_and_svn_multi() {
  let tex = r"\documentclass{article}
\usepackage{hypdestopt}
\usepackage{svn-multi}
\begin{document}
\svnrev\ \svnmonth\ \svnauthor
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // svn-multi.sty:255/261 defaults before any keyword is registered.
  assert!(xml.contains("-2 00"), "{xml}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
}

/// xcolor.sty \XC@undeclaredcolor used by lua-ul.sty (witness: gckanbun/kanshi-sample).
#[test]
fn xcolor_undeclaredcolor_macro() {
  let tex = r"\documentclass{article}
\usepackage{xcolor}
\makeatletter
\begin{document}
\XC@undeclaredcolor{rgb}{1,0,0}{Red text}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Red text"), "{xml}");
}

/// listings.sty \lst@TestEOLChar internal used by tagpdfdocu-patches.sty
/// (witness: tagpdf/tagpdf).
#[test]
fn listings_test_eol_char_exists() {
  let tex = r"\documentclass{article}
\usepackage{listings}
\makeatletter
\begin{document}
\lst@TestEOLChar{foo}
OK
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("OK"), "{xml}");
}

/// pax.sty definitions patched by doc-use-pax.tex (witness: newpax/doc-use-pax).
#[test]
fn pax_patchcmd_targets_defined() {
  let tex = r"\documentclass{article}
\usepackage{etoolbox}
\usepackage{pax}
\makeatletter
\patchcmd\PAX@pdf@annot{\PAX@pagellx}{\PAX@page@llx}{}{\fail}
\patchcmd\PAX@AddAnnots{\InputIfFileExists\PAX@file{}{\typeout{* Missing: \PAX@file}}}
 {\begingroup \catcode`\#=12 \catcode`\%=12
  \InputIfFileExists\PAX@file{}{\typeout{* Missing: \PAX@file}}\endgroup}{}{\fail}
\makeatother
\begin{document}
Patched
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Patched"), "{xml}");
  assert!(!stderr.contains("undefined:\\fail"), "{stderr}");
}

/// updatemarks.sty out-of-scope stub (witness: updatemarks/updatemarks).
#[test]
fn updatemarks_stub_loads_cleanly() {
  let tex = r"\documentclass{article}
\usepackage{updatemarks}
\begin{document}
Marks stub
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("Marks stub"), "{xml}");
  assert_eq!(
    stderr
      .matches("Warning:missing_file:updatemarks.sty")
      .count(),
    1,
    "{stderr}"
  );
}

/// pgf layers macro list expansion (witness: pgf-periodictable/pgf-PeriodicTableManual).
///
/// \pgfsetlayers passes its argument unexpanded to \pgf@dosetlayer, matching
/// `#1,#2,\relax`. When a package passes a macro containing a comma list,
/// \expanded{#1} ensures layers are split into \pgf@layerlist, preventing
/// "layer not part of the layer list" errors and rendering ordered <svg:g> elements.
#[test]
fn pgf_layers_macro_list_renders_ordered_svg_groups() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\pgfdeclarelayer{bg}
\def\mylayers{bg,main}
\pgfsetlayers{\mylayers}
\begin{document}
\begin{tikzpicture}
  \fill[blue] (0,0) rectangle (2,2);
  \begin{pgfonlayer}{bg}
    \fill[red] (0,0) rectangle (1,1);
  \end{pgfonlayer}
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("part of the layer list"), "{stderr}");
  // Verify that bg (red) is shipped out in an SVG group before main (blue)
  let red_pos = xml.find("#FF0000").expect("red background layer present");
  let blue_pos = xml.find("#0000FF").expect("blue main layer present");
  assert!(
    red_pos < blue_pos,
    "background layer must precede main layer in SVG output"
  );
}

/// modernposter class semantic blocks and frontmatter (witness: modernposter/demo).
///
/// modernposter wraps documents in an overlay tikzpicture spanning the page,
/// causing "No shape named sep is known" when nodes are looked up across scopes.
/// The modernposter binding drops the overlay tikzpicture and outputs semantic
/// containers (postercolumn, posterbox, doubleposterbox) with title and author frontmatter.
#[test]
fn modernposter_semantic_poster_blocks_and_frontmatter() {
  let tex = r"\documentclass{modernposter}
\title{Demo Title}
\author{A. Author}
\email{a@author.org}
\begin{document}
\maketitle
\begin{postercolumn}
  \posterbox{Intro}{Some intro text.}
  \doubleposterbox{Box A}{Body A}{Box B}{Body B}
\end{postercolumn}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(!stderr.contains("No shape named"), "{stderr}");
  assert!(xml.contains("<title>Demo Title</title>"), "{xml}");
  assert!(xml.contains("<personname>A. Author</personname>"), "{xml}");
  assert!(
    xml.contains("<note role=\"email\">a@author.org</note>"),
    "{xml}"
  );
  assert!(xml.contains(r#"<block class="ltx_postercolumn">"#), "{xml}");
  assert!(xml.contains(r#"<block class="ltx_posterbox">"#), "{xml}");
  assert!(
    xml.contains(r#"<p class="ltx_posterbox_title">Intro</p>"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<block class="ltx_doubleposterbox">"#),
    "{xml}"
  );
}

/// PGF functional shading fallback and gradient stops (witness: tikzpingus/tikzpingus-doc.tex Figure 1).
/// Functional shadings (e.g. \pgfuseshading{bilinear interpolation}) must define \@pgfshading<name>!
/// via \pgf@sys@noshading to provide \lxSVG@sh@defs, \lxSVG@sh, and \lxSVG@pos, rather than
/// leaving them undefined in Gullet. Also asserts valid SVG linear and radial gradients with >= 2 stops.
#[test]
fn pgf_functional_shading_and_gradients() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{shadings}
\pgfdeclareradialshading{testradial}{\pgfpointorigin}{%
  color(0bp)=(red); color(20bp)=(yellow); color(40bp)=(blue)%
}
\pgfdeclarehorizontalshading{testhori}{100bp}{%
  color(0bp)=(red); color(50bp)=(yellow); color(100bp)=(blue)%
}
\begin{document}
\begin{tikzpicture}
\pgfuseshading{bilinear interpolation}
\pgfuseshading{testradial}
\pgfuseshading{testhori}
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg:radialGradient"), "{xml}");
  assert!(xml.contains("<svg:linearGradient"), "{xml}");
  assert!(xml.matches("<svg:stop").count() >= 2, "{xml}");
}

/// tikzpingus shading regeneration with cloak and functional-shading left wing grab
/// (witness: tikzpingus/tikzpingus-doc.tex Figure 1 Table 3).
#[test]
fn tikzpingus_cloak_functional_shading() {
  if !kpsewhich_has("tikzpingus.sty") {
    return;
  }
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage{tikzpingus}
\begin{document}
\tikz{\pingu[cloak=gray,cup,left wing grab]}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<svg:linearGradient"), "{xml}");
  assert!(xml.matches("<svg:stop").count() >= 2, "{xml}");
}

/// pgfplots scatter markers group balance (witness: ualberta 05_Plots_And_Graphs.tex:325-337).
/// Scatter marker post-marker code calls colormap routines which trigger \pgfmathmultiply@{0.0}{...}.
/// Under LaTeXML/latexml-oxide, integer formatting strips the decimal point ('0' instead of '0.0'),
/// breaking \pgfplotscolormap@floor@unforgiving#1.#2\relax delimiter matching and corrupting the group stack.
#[test]
fn pgfplots_scatter_marker_group_balance() {
  let tex = r"\documentclass{article}
\usepackage{pgfplots}
\pgfplotsset{compat=1.18}
\begin{document}
\begin{tikzpicture}
\begin{axis}
\addplot+[only marks,scatter,mark=*] coordinates {(1,1)(2,4)(3,9)};
\end{axis}
\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.matches("<svg:g").count() >= 3, "{xml}");
}

/// animate: single representative frame for multi-frame animations (witness: among-us repro.tex).
/// Exposes the frame count (RDFa `content`) on the wrapper block, avoiding memory budget exhaustion.
#[test]
fn animate_multiframe_single_frame() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usepackage{animate}
\begin{document}
\begin{animateinline}[controls]{30}
\multiframe{10}{x=0+1}{%
  \begin{tikzpicture}
    \draw (0,0) rectangle (\x,2);
  \end{tikzpicture}%
}
\end{animateinline}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<svg:svg").count(), 1, "{xml}");
  assert!(xml.contains("content=\"10\""), "{xml}");
  assert!(xml.contains("class=\"ltx_animate\""), "{xml}");
  // Batch 56aw (liftarm 1→813, sweep 63): the option list's `begin`/`end`
  // code wraps EVERY frame (animate.sty:2314-2340) and the first `\newframe`
  // closes the representative frame; the remaining frames are discarded but
  // still counted.
  let tex = r"\documentclass{article}
\usepackage{tikz}
\usetikzlibrary{calc}
\usepackage{animate}
\begin{document}
\begin{animateinline}[begin={\begin{tikzpicture}\def\r{1}},end={\end{tikzpicture}}]{20}
\path let \p1=(1,1) in (\p1) node{a};\draw (0,0) circle (\r);
\newframe
\draw (0,0) circle (2);
\newframe
\draw (0,0) circle (3);
\end{animateinline}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<svg:svg").count(), 1, "{xml}");
  assert!(xml.contains("content=\"3\""), "{xml}");

  // Task N4: \newframe* and \newframe[fps] consumed silently, counting frames.
  let tex = r"\documentclass{article}
\usepackage{animate}
\begin{document}
\begin{animateinline}{10}
Frame 1
\newframe*
Frame 2
\newframe[20]
Frame 3
\newframe*[15]
Frame 4
\end{animateinline}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("content=\"4\""), "{xml}");

  // Task N4: reverse playback \animategraphics with first > last (|last-first|+1 frames).
  // Missing frame file emits a warning, not an error.
  let tex = r"\documentclass{article}
\usepackage{animate}
\begin{document}
\animategraphics[controls]{12}{missing_frame_}{10}{1}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("content=\"10\""), "{xml}");

  // Reverse playback selecting an existing file candidate (e.g. example-image-a).
  let tex = r"\documentclass{article}
\usepackage{animate}
\begin{document}
\animategraphics{12}{example-image-a}{5}{1}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("content=\"5\""), "{xml}");
}

/// chemnum: sequential compound numbering model (Task N5, Task N2).
/// Guard: first use = target <text>, references = <ref idref=...>,
/// \refcmpd of undeclared label emits <ref> with ?? without crashing; 0 errors.
#[test]
fn chemnum_compound_numbering() {
  let tex = r"\documentclass{article}
\usepackage{chemnum}
\cmpdinit{initA, initB}
\begin{document}
\cmpd{first}
\cmpd{second}
\refcmpd{first}
\cmpd{first.a}
\refcmpd{first.a}
\cmpd{first,second}
\cmpd{initA} and \cmpd{initB}
\refcmpd{undeclared}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(
    xml.contains(r#"<text class="ltx_cmpd" xml:id="cmpd.first">3</text>"#),
    "first use must be 3 (after initA=1, initB=2) with xml:id target: {xml}"
  );
  assert!(
    xml.contains(r#"<text class="ltx_cmpd" xml:id="cmpd.second">4</text>"#),
    "second label must be 4 with xml:id target: {xml}"
  );
  assert!(
    xml.contains(r#"<ref class="ltx_cmpd" idref="cmpd.first">3</ref>"#),
    "refcmpd of first must be 3 with ltx:ref link: {xml}"
  );
  assert!(
    xml.contains(r#"<text class="ltx_cmpd" xml:id="cmpd.first.a">3a</text>"#),
    "sub-compound must be 3a with xml:id target: {xml}"
  );
  assert!(
    xml.contains(r#"<ref class="ltx_cmpd" idref="cmpd.first.a">3a</ref>"#),
    "refcmpd of sub-compound must be 3a with ltx:ref link: {xml}"
  );
  assert!(
    xml.contains(r#"<ref class="ltx_cmpd" idref="cmpd.first">3</ref>, <ref class="ltx_cmpd" idref="cmpd.second">4</ref>"#),
    "list must emit comma-separated references: {xml}"
  );
  assert!(
    xml.contains(r#"<text class="ltx_cmpd" xml:id="cmpd.initA">1</text>"#),
    "initA must be 1 with xml:id target: {xml}"
  );
  assert!(
    xml.contains(r#"<text class="ltx_cmpd" xml:id="cmpd.initB">2</text>"#),
    "initB must be 2 with xml:id target: {xml}"
  );
  assert!(
    xml.contains(r#"<ref class="ltx_cmpd" idref="cmpd.undeclared">??</ref>"#),
    "refcmpd of undeclared label must emit ref with ?? without crashing: {xml}"
  );
}

/// afterpage: \afterpage defined as macro that un-doubles ## parameters (Task N5).
/// Guard: \afterpage with nested \newcommand with ##1/##2 parameters converts with 0 errors
/// and substitutes correctly. The package must be loaded: an undefined `\afterpage` stays an error (LaTeX, Perl).
#[test]
fn afterpage_body_undoubles_parameter_hashes() {
  let tex = r"\documentclass{article}
\usepackage{afterpage}
\begin{document}
\afterpage{%
  \newcommand\C[2]{[#1:#2]}%
  \C{A1}{1.00}%
}
Hello
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[A1:1.00]"), "{xml}");

  let tex = r"\documentclass{article}
\usepackage{afterpage}
\begin{document}
\afterpage{%
  \newcommand\C[2]{[##1:##2]}%
  \C{B2}{2.50}%
}
World
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("[B2:2.50]"), "{xml}");
}

/// Batch 56bz: an undefined control word in an `\index` entry is inerted at
/// STATE level for the `\protected@write` pre-expansion, never by an
/// injected `\noexpand` token — `\string` reads the NEXT token literally, so
/// `\string\noexpand\cmd` stringified "\noexpand" and re-exposed `\cmd`
/// (beamerug-macros.tex:44 `\gdef\stripcommand#1{\expandafter\@gobble\string#1}`;
/// beameruserguide 167 errors, Perl 0). The `\if…` names are the worst case:
/// an exposed one is auto-defined as a conditional and scans for `\fi` off
/// the end of the entry.
/// A separator inside math is phrase material: `\index{arroba@$@$}`
/// (latex-via-exemplos.tex:1042 `\arrobasymbforindex` = `$@$`) is the key
/// `arroba` with the display `$@$`; splitting at the inner `@` left a lone
/// `$` opening math the bounded `\@index` box never closed (three errors
/// per entry, a leaked `<XMath>` swallowing the paragraph; pdflatex clean).
#[test]
fn index_separator_inside_math_is_phrase_material() {
  let tex = "\\documentclass{article}\n\\usepackage{makeidx}\\makeindex\n\\begin{document}\nX\\index{arroba@$@$} Y\n\\end{document}\n";
  let (stderr, xml) = convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r##"<para xml:id="p1"><p>X<indexmark><indexphrase key="arroba"><Math mode="inline" tex="@" text="@" xml:id="p1.m1"><XMath><XMTok role="UNKNOWN">@</XMTok></XMath></Math></indexphrase></indexmark> Y</p></para>"##,
  );
}

/// beameruserguide: beamerug-macros.tex:58 builds the entry in a macro (`\index{\stripcommand#1@…}`), so the write
/// expands `\stripcommand` before makeindex sees the `@`.
#[test]
fn index_string_of_undefined_command_stringifies_its_name() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}\makeindex
\makeatletter
\gdef\stripcommand#1{\expandafter\@gobble\string#1}
\makeatother
\def\myprintcommand#1{\texttt{\char`\\#1}}
\def\indexcommand#1{\index{\stripcommand#1@\protect\myprintcommand{\stripcommand#1}}}
\begin{document}
\indexcommand\insertframetitle
\indexcommand\ifbeamercolorempty
X\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // The whole index phrase: the key is the STRINGIFIED name and the display
  // is `\myprintcommand`'s typewriter text, run at typesetting time.
  assert!(
    xml.contains(
      "<indexphrase key=\"insertframetitle\"><text font=\"typewriter\">\\insertframetitle</text></indexphrase>"
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      "<indexphrase key=\"ifbeamercolorempty\"><text font=\"typewriter\">\\ifbeamercolorempty</text></indexphrase>"
    ),
    "{xml}"
  );
  assert!(!xml.contains("<ERROR"), "{xml}");
}

/// Gemini round 13, Q1: amsart's `\uppercasenonmath` (amsart.cls:405-426; amsproc.cls and
/// amsbook.cls identical) uppercases a macro's text in place and leaves its math as is. Neither
/// Perl binding ports it (`Error:undefined:\uppercasenonmath`, PERL-ORIGIN); pdflatex "T: TITLE
/// x HERE.". Repro: tools/perfect_kernel/repros/sectioning-frontmatter/amsart_uppercasenonmath_is_defined.tex.
#[test]
fn amsart_uppercasenonmath_is_defined() {
  for class in ["amsart", "amsbook"] {
    let tex = format!(
      "\\documentclass{{{class}}}\n\\begin{{document}}\n\\makeatletter\\def\\x{{Title $x$ here}}\\uppercasenonmath\\x\\makeatother\nT: \\x.\n\\end{{document}}\n"
    );
    let (stderr, xml) = convert(&tex, true);
    assert_eq!(error_count(&stderr), 0, "{class}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{class}: {stderr}");
    latexml::util::test::assert_element(
      &xml,
      "p",
      &[],
      r#"<p>T: TITLE <Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> HERE.</p>"#,
    );
  }
  // Control (passed before the fix): without the call the text keeps its case.
  let tex = "\\documentclass{amsart}\n\\begin{document}\n\\makeatletter\\def\\x{Title $x$ here}\\makeatother\nT: \\x.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p>T: Title <Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> here.</p>"#,
  );
}

/// Gemini round 13, Q2: `\captionof{lstlisting}` inside a figure's minipage numbers its own
/// counter. caption's `\captionof` sets the caption type, `\@captype` included (caption.sty:391,
/// :296-313); the binding recorded only the continuation, and a verbatim type (no wrapper float,
/// OXIDIZED_DESIGN #89) was stepped as the enclosing `figure` by `\@@add@caption@counters`:
/// "Listing 0" tagged "Figure 2", and the next `\ContinuedFloat` accepted. pdflatex: Figure 1,
/// Listing 1, Figure 2, Figure 3 and caption's one error, "Continued `figure' after
/// `lstlisting'". The caption opens the generic `{@float}{lstlisting}` (57bu re-review), so the
/// listing is numbered where it stands — its tags, `LST1`, the list of listings — and collapses with
/// the minipage and the figure into one element.
/// Repro: tools/perfect_kernel/repros/captions-floats/captionof_verbatim_type_numbers_its_own_counter.tex.
#[test]
fn captionof_verbatim_type_numbers_its_own_counter() {
  let tex = r"\documentclass{article}
\usepackage{caption}
\usepackage{listings}
\begin{document}
\begin{figure}A\caption{A}\end{figure}
\begin{figure}\begin{minipage}{.8\linewidth}\captionof{lstlisting}{L}\end{minipage}\end{figure}
\begin{figure}\ContinuedFloat B\caption{B}\end{figure}
\begin{figure}C\caption{C}\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(
    stderr.contains("Continued `figure' after `lstlisting'"),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="LST1""#],
    concat!(
      r#"<figure class="ltx_float_lstlisting ltx_minipage" inlist="lol" vattach="middle" width="276.0pt" xml:id="LST1">"#,
      "<tags><tag>Listing\u{a0}1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Listing 1</tag></tags>",
      r#"<toccaption><tag close=" ">1</tag>L</toccaption>"#,
      "<caption><tag close=\": \">Listing\u{a0}1</tag>L</caption></figure>"
    ),
  );
  // Control (passed before the fix): a figure-typed `\captionof` in the same place is Figure 2.
  let tex = r"\documentclass{article}
\usepackage{caption}
\begin{document}
\begin{figure}A\caption{A}\end{figure}
\begin{figure}\begin{minipage}{.8\linewidth}\captionof{figure}{L}\end{minipage}\end{figure}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r#"<figure class="ltx_minipage" inlist="lof" vattach="middle" width="276.0pt" xml:id="S0.F2"><tags><tag>Figure 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><toccaption><tag close=" ">2</tag>L</toccaption><caption><tag close=": ">Figure 2</tag>L</caption></figure>"#,
  );
}

/// Gemini round 13, Q3: `\PackageWarning` writes its text as `\immediate\write` expands it
/// (latex.ltx:8780-8786 `\GenericWarning`), keeping what `\unexpanded` yields unexpanded: the
/// undefined `\foo` inside it is printed, never run. `make_generic_message` expanded fully, as
/// Perl (latex_constructs.pool.ltxml:5586-5587; `Error:undefined:\foo`, PERL-ORIGIN). pdflatex:
/// "Package test Warning: \foo x #### y" (`##` is two `#` tokens, each doubled by `\write`).
/// Repro: tools/perfect_kernel/repros/string-mouth/package_warning_keeps_unexpanded_text.tex.
#[test]
fn package_warning_keeps_unexpanded_text() {
  let tex = "\\documentclass{article}\n\\PackageWarning{test}{\\unexpanded{\\foo x ## y}}\n\\begin{document}\nx\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr
      .lines()
      .any(|l| l.trim_end() == r"Warning:latex:(test) Package test Warning: \foo x #### y"),
    "{stderr}"
  );
  latexml::util::test::assert_element(&xml, "para", &[], r#"<para xml:id="p1"><p>x</p></para>"#);
  // Control (passed before the fix): an expandable macro in the text is expanded.
  let tex = "\\documentclass{article}\n\\def\\bar{B}\n\\PackageWarning{test}{\\bar\\space x}\n\\begin{document}\nx\n\\end{document}\n";
  let (stderr, _) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert!(
    stderr
      .lines()
      .any(|l| l.trim_end() == "Warning:latex:(test) Package test Warning: B x"),
    "{stderr}"
  );
}

/// Gemini round 13, Q4: listings' `name=` (the file name of `\lstinputlisting`) re-encodes `_` and
/// `$` as `\textunderscore`/`\textdollar` before it is typeset (Perl listings.sty.ltxml:170-178,
/// `%lstFilenameRPL`). Digested as catcode-12 characters under the default OT1 encoding, the `_`
/// took slot 0x5F, the dot accent: `dataname="lstu˙x.txt"`. Repro:
/// tools/perfect_kernel/repros/singletons/listings_dataname_ot1_underscore.tex.
#[test]
fn listings_name_keeps_its_underscore() {
  let body = "\\begin{filecontents*}{lstu_x.txt}\nx = 1\n\\end{filecontents*}\n\\documentclass{article}\nFONTENC\\usepackage{listings}\n\\begin{document}\n\\lstinputlisting{lstu_x.txt}\n\\end{document}\n";
  // The fix, then the control that passed before it (T1 has `_` at 0x5F).
  for fontenc in ["", "\\usepackage[T1]{fontenc}\n"] {
    let (stderr, xml) = convert(&body.replace("FONTENC", fontenc), true);
    assert_eq!(error_count(&stderr), 0, "{fontenc}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{fontenc}: {stderr}");
    latexml::util::test::assert_element(
      &xml,
      "toccaption",
      &[],
      "<toccaption>lstu_x.txt</toccaption>",
    );
    latexml::util::test::assert_element(
      &xml,
      "listing",
      &[],
      concat!(
        r#"<listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain" dataname="lstu_x.txt">"#,
        r#"<listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing>"#
      ),
    );
  }
}

/// Gemini round 13, Q5: `\hyperdef`/`\hypertarget` anchor their own text (hyperref.sty:4834-4845,
/// `\hyper@@anchor{…}{#3}`). The bindings (and Perl, hyperref.sty.ltxml:238-258) walked from the
/// insertion point for the first node an anchor may hold, which mid-paragraph already held the
/// words before them: `<anchor xml:id="cat.nm">A Target</anchor><anchor xml:id="tt"> b. T3</anchor>`
/// (SHARED; this beats Perl). Repro:
/// tools/perfect_kernel/repros/block-model/hyperdef_anchor_holds_only_its_text.tex.
#[test]
fn hyperdef_anchor_holds_only_its_text() {
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\begin{document}\nA \\hyperdef{cat}{nm}{Target} b. \\hypertarget{tt}{T3} d.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    r#"<p>A <anchor xml:id="cat.nm">Target</anchor> b. <anchor xml:id="tt">T3</anchor> d.</p>"#,
  );
  // Control (passed before the fix): where the insertion point admits no anchor, the walk still
  // places it (Pandoc-style `\hypertarget{n}{\section{T}}`: inside the title), and an empty
  // target mid-paragraph is a bare destination.
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\begin{document}\n\\hypertarget{sec}{\\section{T}}\nA \\hypertarget{e}{} b.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "title",
    &[],
    r#"<title><tag close=" ">1</tag><anchor xml:id="sec">T</anchor></title>"#,
  );
  latexml::util::test::assert_element(&xml, "p", &[], r#"<p>A <anchor xml:id="e"/> b.</p>"#);
  // Control (review of the first cut): display material in the text is a block, not an
  // inline-block inside the anchor, and the equation stays between the paragraph's two halves.
  // Since 59m the anchor is a point destination before the display (hyperref's nesting-false
  // `\hyper@@anchor{#1}{\relax}#2`, hyperref.sty:4805-4810); the walk had wrapped the display's
  // `ltx:Math`, an anchor inside `ltx:equation` (schema-invalid, philexmanual).
  let tex = "\\documentclass{article}\n\\usepackage{hyperref}\n\\begin{document}\nA \\hypertarget{d}{\\[x\\]} b.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    concat!(
      r#"<para xml:id="p1"><p>A <anchor xml:id="d"/></p><equation xml:id="S0.Ex1">"#,
      r#"<Math mode="display" tex="x" text="x" xml:id="S0.Ex1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>"#,
      r#"</equation><p>b.</p></para>"#
    ),
  );
}

/// Gemini round 13, Q6: babel-french's high punctuation replaces the space typed before it
/// (french3.ldf:277-318, `\ifdim\lastskip>1sp \unskip\penalty\@M\FBthinspace`): "Mid
/// \textbf{Bold} ;" kept U+0020 before babel's thin space (SHARED). Witness matapli/matapli-doc
/// (6 places, `\Verb+…+ ;`). Repro: tools/perfect_kernel/repros/babel-lang/french_highpunct_unskips_space.tex.
#[test]
fn french_high_punctuation_unskips_the_space() {
  let tex = "\\documentclass{article}\n\\usepackage[french]{babel}\n\\begin{document}\nMid \\textbf{Bold} ; suite.\n\nMid bold ; suite.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    "<para xml:id=\"p1\"><p>Mid <text font=\"bold\">Bold</text>\u{2006}; suite.</p></para>",
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    "<para xml:id=\"p2\"><p>Mid bold\u{2006}; suite.</p></para>",
  );
  // Control (passed before the fix): punctuation right after a word gets the thin space (`:` a
  // normal one), no space removed.
  let tex = "\\documentclass{article}\n\\usepackage[french]{babel}\n\\begin{document}\nOui! Non; peut-etre? Voila: fin.\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[],
    "<p>Oui\u{2006}! Non\u{2006}; peut-etre\u{2006}? Voila : fin.</p>",
  );
}

/// Gemini round 13, Q7: a bare `\subfloat{…}` steps the sub-float counter and prints no caption
/// — subcaption boxes it with a `\phantomcaption` (subcaption.sty:293-300), subfig tests for its
/// `[\@empty]` caption (subfig.sty:348-349, :391, :410) — and caption's `\phantomcaption` is
/// `\caption@refstepcounter\@captype` (caption.sty:392-395), a no-op stub before. pdflatex: the
/// bare form prints nothing, `\subfloat[]{…}` "(b)", `\subfloat[Cap C]{…}` "(c) Cap C"; the List of
/// Figures has "Main" only. Rust and Perl printed a "(a)" caption for the bare form (SHARED).
/// Repro: tools/perfect_kernel/repros/captions-floats/bare_subfloat_has_a_phantom_caption.tex.
#[test]
fn bare_subfloat_has_a_phantom_caption() {
  let body = "\\documentclass{article}\n\\usepackage{graphicx}\n\\usepackage{PKG}\n\\begin{document}\n\\begin{figure}\n\\subfloat{\\rule{1cm}{1cm}}\n\\subfloat[]{\\rule{1cm}{1cm}}\n\\subfloat[Cap C]{\\rule{1cm}{1cm}}\n\\caption{Main}\n\\end{figure}\n\\end{document}\n";
  // subcaption sets its sub-captions small and lists them; subfig does neither.
  let rule = r#"<rule height="28.5pt" width="28.5pt"/>"#;
  for (pkg, small, inlist) in [
    (
      "subcaption",
      (r#"<text fontsize="90%">"#, "</text>"),
      r#" inlist="lof""#,
    ),
    ("subfig", ("", ""), ""),
  ] {
    let (stderr, xml) = convert(&body.replace("PKG", pkg), true);
    assert_eq!(error_count(&stderr), 0, "{pkg}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{pkg}: {stderr}");
    let (open, close) = small;
    // The bare form: its tags and its rule; no caption, toccaption or list entry.
    latexml::util::test::assert_element(
      &xml,
      "figure",
      &[r#"xml:id="S0.F1.sf1""#],
      &format!(
        r#"<figure class="ltx_figure_panel" xml:id="S0.F1.sf1"><tags><tag>{open}(a){close}</tag><tag role="refnum">1a</tag></tags>{rule}</figure>"#
      ),
    );
    // Control (passed before the fix): the captioned third sub-figure.
    latexml::util::test::assert_element(
      &xml,
      "figure",
      &[r#"xml:id="S0.F1.sf3""#],
      &format!(
        concat!(
          r#"<figure class="ltx_figure_panel"{inlist} xml:id="S0.F1.sf3"><tags><tag>{open}(c){close}</tag><tag role="refnum">1c</tag></tags>"#,
          r#"{rule}<toccaption><tag close=" ">c</tag>Cap C</toccaption>"#,
          r#"<caption><tag close=" ">{open}(c){close}</tag>{open}Cap C{close}</caption></figure>"#
        ),
        inlist = inlist,
        open = open,
        close = close,
        rule = rule
      ),
    );
  }
  // A figure opening with `\phantomcaption` (2503.21681): the figure is numbered, its sub-figures
  // follow that number and do not pre-increment it again, the next figure is Figure 2.
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\n\\phantomcaption\n\\begin{subfigure}{.4\\textwidth}A\\caption{A}\\end{subfigure}\n\\begin{subfigure}{.4\\textwidth}C\\caption{C}\\end{subfigure}\n\\end{figure}\n\\begin{figure}B\\caption{B}\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let small = |t: &str| format!(r#"<text fontsize="90%">{t}</text>"#);
  let panel = |n: &str, l: &str, body: &str| {
    format!(
      concat!(
        r#"<figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F1.sf{n}"><tags><tag>{tag}</tag><tag role="refnum">1{l}</tag></tags>"#,
        r#"<p>{body}</p><toccaption><tag close=" ">{l}</tag>{body}</toccaption><caption><tag close=" ">{tag}</tag>{text}</caption></figure>"#
      ),
      n = n,
      l = l,
      body = body,
      tag = small(&format!("({l})")),
      text = small(body)
    )
  };
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    &format!(
      r#"<figure xml:id="S0.F1"><tags><tag>{}</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags>{}{}</figure>"#,
      small("Figure 1"),
      panel("1", "a", "A"),
      panel("2", "b", "C")
    ),
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    &format!(
      concat!(
        r#"<figure inlist="lof" xml:id="S0.F2"><tags><tag>{fig}</tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags>"#,
        r#"<p>B</p><toccaption><tag close=" ">2</tag>B</toccaption><caption><tag close=": ">{fig}</tag>{b}</caption></figure>"#
      ),
      fig = small("Figure 2"),
      b = small("B")
    ),
  );
}

/// Gemini round 13, Q8: subfig's sub-caption label follows the caption label format. subfig
/// passes its package options to `\captionsetup[subfloat]` (subfig.sty:208-225, skipping the
/// `caption` and `config` keys, :188-195), default `labelformat=parens` (:285-288); the binding
/// hard-coded `\fnum@subfigure` as `(\thesubfigure)`, so `labelformat=simple` over a parenthesized
/// `\thesubfigure` printed "((a))" (SHARED with Perl); pdflatex "(a) Cap A". Repro:
/// tools/perfect_kernel/repros/captions-floats/subfig_label_follows_the_caption_label_format.tex.
#[test]
fn subfig_label_follows_the_caption_label_format() {
  let body = "\\documentclass{article}\n\\usepackage[OPTS]{subfig}\nTHESUB\\begin{document}\n\\begin{figure}\n\\subfloat[Cap A\\label{sa}]{\\rule{1cm}{1cm}}\n\\caption{Main}\n\\end{figure}\nSee \\ref{sa}.\n\\end{document}\n";
  // The repro, then the control that passed before (plain options, the default parens).
  for (opts, thesub) in [
    (
      "caption=false,labelformat=simple",
      "\\renewcommand\\thesubfigure{(\\alph{subfigure})}\n",
    ),
    ("", ""),
  ] {
    let tex = body.replace("OPTS", opts).replace("THESUB", thesub);
    let (stderr, xml) = convert(&tex, true);
    assert_eq!(error_count(&stderr), 0, "{opts}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{opts}: {stderr}");
    latexml::util::test::assert_element(
      &xml,
      "caption",
      &[],
      r#"<caption><tag close=" ">(a)</tag>Cap A</caption>"#,
    );
  }
}

/// Gemini round 13, Q9: natbib's `thebibliography` typesets `\bibpreamble` after its heading
/// (natbib.sty:1063-1066), and apacite adds `\bibliographyprenote` and `\nocitemeta`'s note to it
/// (apacite.sty:1835-1850); the kernel bibliography never ran it, so both sentences were lost
/// (Perl the same, and 5 errors: KNOWN_PERL_ERRORS #325). The schema allows `Para.model` before
/// `ltx:biblist`. Repro: tools/perfect_kernel/repros/index-bib/bibpreamble_is_printed.tex.
#[test]
fn bibpreamble_is_printed() {
  let tex = r"\documentclass{article}
\usepackage[natbibapa]{apacite}
\renewcommand{\bibliographyprenote}{Preamble note.}
\begin{document}
See \citet{smith2001}.
\nocitemeta{smith2001}
\begin{thebibliography}{}
\bibitem [\protect \citeauthoryear {Smith}{Smith}{{\protect \APACyear {2001}}}]{smith2001}
\APACinsertmetastar {smith2001}%
\begin{APACrefauthors}Smith, J.\end{APACrefauthors}
\newblock \APACrefYearMonthDay{2001}{}{}.
\newblock T.
\end{thebibliography}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "bibliography",
    &[],
    concat!(
      r#"<bibliography inlist="toc" xml:id="bib"><title>References</title>"#,
      r#"<para xml:id="bib.p1"><p>Preamble note.References marked with an asterisk indicate studies included in the meta-analysis.</p></para>"#,
      r#"<biblist><bibitem key="smith2001" xml:id="bib.bib1"><tags><tag role="number">1</tag><tag role="year">2001</tag>"#,
      r#"<tag role="authors">Smith</tag><tag role="fullauthors">Smith</tag><tag role="refnum">Smith (2001)</tag><tag role="key">smith2001</tag></tags>"#,
      r#"<bibblock><sup>∗</sup>Smith, J.</bibblock><bibblock>(2001).</bibblock><bibblock>T.</bibblock></bibitem></biblist></bibliography>"#
    ),
  );
  // Control (passed before the fix): natbib alone, no preamble — no paragraph before the list.
  let tex = "\\documentclass{article}\n\\usepackage{natbib}\n\\begin{document}\nSee \\cite{a}.\n\\begin{thebibliography}{1}\n\\bibitem{a} A.\n\\end{thebibliography}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "bibliography",
    &[],
    concat!(
      r#"<bibliography inlist="toc" xml:id="bib"><title>References</title><biblist><bibitem key="a" xml:id="bib.bib1">"#,
      r#"<tags><tag role="number">1</tag><tag role="refnum">(1)</tag><tag role="key">a</tag></tags><bibblock> A.</bibblock></bibitem></biblist></bibliography>"#
    ),
  );
  // Control (review of the first cut): a font switch in the preamble reaches the entries but not
  // the heading, which natbib typesets before `\bibpreamble` (natbib.sty:1063-1066).
  let tex = "\\documentclass{article}\n\\usepackage{natbib}\n\\renewcommand{\\bibpreamble}{\\small Note.}\n\\begin{document}\nSee \\cite{a}.\n\\begin{thebibliography}{1}\n\\bibitem{a} A.\n\\end{thebibliography}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "bibliography",
    &[],
    concat!(
      r#"<bibliography inlist="toc" xml:id="bib"><title>References</title>"#,
      r#"<para xml:id="bib.p1"><p><text fontsize="90%">Note.</text></p></para><biblist><bibitem key="a" xml:id="bib.bib1"><tags>"#,
      r#"<tag role="number"><text fontsize="90%">1</text></tag><tag role="refnum"><text fontsize="90%">(1)</text></tag>"#,
      r#"<tag role="key"><text fontsize="90%">a</text></tag></tags><bibblock><text fontsize="90%"> A.</text></bibblock></bibitem></biblist></bibliography>"#
    ),
  );
}

/// Round-13 A/B (57bu), R3: `\captionof` in a float sets `\@captype` only inside its own wrapper —
/// `\caption@settype` there left it set in the enclosing figure, whose `after_float` then rescued the
/// table's counters: the figure lost its number and every later figure and table shifted
/// (2605.19656, 2605.20199). `\setcaptiontype` (caption.sty:288-298) sets the type back.
#[test]
fn captionof_in_a_float_keeps_the_float_type() {
  let tex = "\\documentclass{article}\n\\usepackage{caption}\n\\begin{document}\n\\begin{figure}\nx\n\\caption{Fig cap}\\label{fig:a}\n\\captionof{table}{Tab cap}\\label{tab:a}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" labels="LABEL:fig:a" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p class="ltx_figure_panel">x</p><toccaption><tag close=" ">1</tag>Fig cap</toccaption><caption><tag close=": ">Figure 1</tag>Fig cap</caption><table class="ltx_figure_panel" inlist="lot" labels="LABEL:tab:a" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Tab cap</toccaption><caption><tag close=": ">Table 1</tag>Tab cap</caption></table></figure>"#,
  );
  // `\setcaptiontype{figure}` after a `\captionof{table}`: the figure's own caption is Figure 1.
  let tex = "\\documentclass{article}\n\\usepackage{caption}\n\\begin{document}\n\\begin{figure}\n\\captionof{table}{Tab cap}\n\\setcaptiontype{figure}\nx\n\\caption{Fig cap}\\label{fig:a}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" labels="LABEL:fig:a" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><table class="ltx_figure_panel" inlist="lot" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Tab cap</toccaption><caption><tag close=": ">Table 1</tag>Tab cap</caption></table><break class="ltx_break"/><p class="ltx_figure_panel">x</p><toccaption><tag close=" ">1</tag>Fig cap</toccaption><caption><tag close=": ">Figure 1</tag>Fig cap</caption></figure>"#,
  );
}

/// Round-13 A/B (57bu), R4: subcaption's `{subcaptiongroup}` (subcaption.sty:60-69) makes `\@captype`
/// the sub-type, so its `\phantomcaption`s number the panels, not the figure — undefined, each
/// stepped the figure counter (2605.01925: Figure 3 for 1, every later figure shifted).
/// The panels' labels sit on the figure, which has no panel element to carry `1a` (KPE #386
/// residual): `\ref{a}` reads 1 where pdflatex prints 1a.
#[test]
fn subcaptiongroup_numbers_the_panels() {
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\n\\begin{subcaptiongroup}\n\\phantomcaption\\label{a}\n\\phantomcaption\\label{b}\n\\end{subcaptiongroup}\nx\n\\caption{Grouped}\\label{f}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" labels="LABEL:a LABEL:b LABEL:f" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p>x</p><toccaption><tag close=" ">1</tag>Grouped</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Grouped</text></caption></figure>"#,
  );
}

/// Round-13 A/B (57bu), R1/R2: a panel numbered by a phantom caption (its own `tags`, no `caption`) is
/// captioned in caption.sty's terms and is not collapsed into its figure — the collapse appended its
/// content after the figure's caption (2605.04869, 2605.17547, 2605.20770, 2605.27546) and copied its
/// `labels` over the figure's (2605.28276). A collapse puts the content where the inner float stood
/// (Perl collapseFloat, latex_constructs.pool.ltxml:3454-3462) and keeps both labels.
#[test]
fn phantom_numbered_panel_keeps_its_place_and_labels() {
  let tex = "\\documentclass{article}\n\\usepackage{subfig}\n\\begin{document}\n\\begin{figure}\n\\subfloat{\\rule{1cm}{1cm}}\n\\caption{Main caption}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure xml:id="S0.F1.sf1"><tags><tag>(a)</tag><tag role="refnum">1a</tag></tags><rule height="28.5pt" width="28.5pt"/></figure><toccaption><tag close=" ">1</tag>Main caption</toccaption><caption><tag close=": ">Figure 1</tag>Main caption</caption></figure>"#,
  );
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\n\\begin{subfigure}{\\linewidth}\nx\n\\phantomsubcaption\\label{a}\n\\end{subfigure}\n\\caption{Main}\\label{main}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure inlist="lof" labels="LABEL:main" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure labels="LABEL:a" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags><p>x</p></figure><toccaption><tag close=" ">1</tag>Main</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Main</text></caption></figure>"#,
  );
}

/// Round-13 A/B (57bu), R5: with subfig's binding read, subcaption's sub-labels are subcaption's —
/// subfig's `labelformat=empty` package option dropped the "(a)" (2605.20200, which keeps subfig out
/// with `\@namedef{ver@subfig.sty}{…}`; our loader does not honour a `\ver@` mark, as memoir's
/// `\EmulatedPackage` marks packages whose bindings are its semantic layer).
#[test]
fn subcaption_labels_win_over_a_read_subfig() {
  let tex = "\\makeatletter\n\\@namedef{ver@subfig.sty}{9999/99/99}\n\\makeatother\n\\documentclass{article}\n\\usepackage[labelformat=empty]{subfig}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\n\\begin{subfigure}{.4\\linewidth}x\\caption{Left}\\end{subfigure}\n\\caption{Main}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "caption",
    &[],
    r#"<caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Left</text></caption>"#,
  );
}

/// 57bu re-review: a collapse leaves the inner float's content where the inner float stood — between
/// the outer's material before and after it (Perl collapseFloat, latex_constructs.pool.ltxml:3454-3462);
/// the round-13 merge appended it after the outer's last child. The inner was one panel beside others, so
/// its `ltx_figure_panel` class is not copied onto the figure (57by, OXIDIZED_DESIGN_DIVERGENCES #375): its
/// content stays that panel (57cp review 5, #372).
#[test]
fn collapsed_panel_content_stays_in_place() {
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\nBefore\n\\begin{subfigure}{\\linewidth}Inner\\end{subfigure}\nAfter\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[],
    r#"<figure xml:id="fig2"><p class="ltx_figure_panel">Before</p><break class="ltx_break"/><p class="ltx_figure_panel">Inner</p><break class="ltx_break"/><p class="ltx_figure_panel">After</p></figure>"#,
  );
}

/// 57bu.2 review: `{subcaptiongroup}` pre-increments the figure counter once per figure, as
/// caption's sub-type hook does — the first cut stepped it again at every later group or sub-float
/// of the same figure (Figure 2, then 4 and 5, for 1, 2, 3). A group outside a float is subcaption's
/// "outside float" error; the later figure keeps its number.
#[test]
fn subcaptiongroup_steps_the_figure_counter_once() {
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\n\\begin{subcaptiongroup}\n\\phantomcaption\\label{a}\n\\end{subcaptiongroup}\n\\begin{subcaptiongroup}\n\\phantomcaption\\label{b}\n\\end{subcaptiongroup}\nx\n\\caption{Grouped}\\label{f}\n\\end{figure}\n\\begin{figure}\n\\begin{subcaptiongroup}\n\\phantomcaption\\label{c}\n\\end{subcaptiongroup}\n\\begin{subfigure}{.4\\linewidth}y\\caption{Sub}\\label{d}\\end{subfigure}\n\\caption{Mixed}\\label{g}\n\\end{figure}\n\\begin{figure}z\\caption{Third}\\label{h}\\end{figure}\n\\begin{subcaptiongroup}w\\end{subcaptiongroup}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 1, "{stderr}");
  assert!(stderr.contains("subcaptiongroup outside float"), "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r#"<figure inlist="lof" labels="LABEL:c LABEL:g" xml:id="S0.F2"><tags><tag><text fontsize="90%">Figure 2</text></tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><figure inlist="lof" labels="LABEL:d" xml:id="S0.F2.sf2"><tags><tag><text fontsize="90%">(b)</text></tag><tag role="refnum">2b</tag></tags><p>y</p><toccaption><tag close=" ">b</tag>Sub</toccaption><caption><tag close=" "><text fontsize="90%">(b)</text></tag><text fontsize="90%">Sub</text></caption></figure><toccaption><tag close=" ">2</tag>Mixed</toccaption><caption><tag close=": "><text fontsize="90%">Figure 2</text></tag><text fontsize="90%">Mixed</text></caption></figure>"#,
  );
  for (id, figure) in [
    (
      "S0.F1",
      r#"<figure inlist="lof" labels="LABEL:a LABEL:b LABEL:f" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p>x</p><toccaption><tag close=" ">1</tag>Grouped</toccaption><caption><tag close=": "><text fontsize="90%">Figure 1</text></tag><text fontsize="90%">Grouped</text></caption></figure>"#,
    ),
    (
      "S0.F3",
      r#"<figure inlist="lof" labels="LABEL:h" xml:id="S0.F3"><tags><tag><text fontsize="90%">Figure 3</text></tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><p>z</p><toccaption><tag close=" ">3</tag>Third</toccaption><caption><tag close=": "><text fontsize="90%">Figure 3</text></tag><text fontsize="90%">Third</text></caption></figure>"#,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "figure", &[&format!(r#"xml:id="{id}""#)], figure);
  }
  // A sub-float inside the group is of the sub-type already: it does not step the figure again
  // (57bu.2 re-review: Figure 3 with 3a, the next figure 4, where pdflatex prints 2, 2a, 3).
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\\rule{1cm}{1cm}\\caption{One}\\label{f1}\\end{figure}\n\\begin{figure}\n\\begin{subcaptiongroup}\n\\begin{subfigure}{.4\\linewidth}\\rule{1cm}{1cm}\\caption{A}\\label{a}\\end{subfigure}\n\\end{subcaptiongroup}\n\\caption{Two}\\label{f2}\n\\end{figure}\n\\begin{figure}\\rule{1cm}{1cm}\\caption{Three}\\label{f3}\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F2""#],
    r#"<figure inlist="lof" labels="LABEL:f2" xml:id="S0.F2"><tags><tag><text fontsize="90%">Figure 2</text></tag><tag role="refnum">2</tag><tag role="typerefnum">Figure 2</tag></tags><figure inlist="lof" labels="LABEL:a" xml:id="S0.F2.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">2a</tag></tags><rule height="28.5pt" width="28.5pt"/><toccaption><tag close=" ">a</tag>A</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">A</text></caption></figure><toccaption><tag close=" ">2</tag>Two</toccaption><caption><tag close=": "><text fontsize="90%">Figure 2</text></tag><text fontsize="90%">Two</text></caption></figure>"#,
  );
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F3""#],
    r#"<figure inlist="lof" labels="LABEL:f3" xml:id="S0.F3"><tags><tag><text fontsize="90%">Figure 3</text></tag><tag role="refnum">3</tag><tag role="typerefnum">Figure 3</tag></tags><rule height="28.5pt" width="28.5pt"/><toccaption><tag close=" ">3</tag>Three</toccaption><caption><tag close=": "><text fontsize="90%">Figure 3</text></tag><text fontsize="90%">Three</text></caption></figure>"#,
  );
}

/// 57bu.2 review: the outer float's tags are its own number — a figure numbered by `\phantomcaption`
/// keeps its captioned panel nested, the panel its own number (DIVERGENCES #372).
#[test]
fn phantom_numbered_outer_keeps_its_captioned_panel() {
  let tex = "\\documentclass{article}\n\\usepackage{subcaption}\n\\begin{document}\n\\begin{figure}\\phantomcaption\\label{outer}\\begin{subfigure}{.4\\linewidth}y\\caption{Sub}\\label{sub}\\end{subfigure}\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure labels="LABEL:outer" xml:id="S0.F1"><tags><tag><text fontsize="90%">Figure 1</text></tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><figure inlist="lof" labels="LABEL:sub" xml:id="S0.F1.sf1"><tags><tag><text fontsize="90%">(a)</text></tag><tag role="refnum">1a</tag></tags><p>y</p><toccaption><tag close=" ">a</tag>Sub</toccaption><caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">Sub</text></caption></figure></figure>"#,
  );
}

/// 57bu A/B (2605.26653, 2605.18937): an inner float's tags that are not a panel number leave it
/// collapsible, as Perl's caption-only test does — a caption-less table that took the outer caption's
/// counters by closing first, and ltablex's longtable, which steps the table counter itself. Kept
/// apart, the captioned table lost its number to the inner one and `\label{L}` named Table 2 where
/// pdflatex prints 1. The longtable's own tags ride along into the collapsed table, after the
/// caption, as in Perl (its `\addtocounter{table}{-1}` keeps the next table 2). Since 57cp the
/// caption-less inner table takes nothing (it sets the outer caption's pending counters aside,
/// KNOWN_PERL_ERRORS #395): the outer table keeps its number and its `\label{t}`, which fell to the
/// document root before.
/// The ltablex half is skipped where `ltablex.sty` is not installed (a trimmed TeX Live).
#[test]
fn inner_float_without_a_panel_number_collapses() {
  let tex = "\\documentclass{article}\n\\usepackage{float}\n\\begin{document}\n\\begin{table}[H]\\caption{Outer}\\label{t}\n\\begin{minipage}{\\linewidth}\\begin{table}[H]\\centering\\begin{tabular}{c}a\\end{tabular}\\end{table}\\end{minipage}\n\\end{table}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r#"<table class="ltx_minipage" inlist="lot" labels="LABEL:t" placement="H" width="345.0pt" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Outer</toccaption><caption><tag close=": ">Table 1</tag>Outer</caption><tabular class="ltx_centering" vattach="middle"><tbody><tr><td align="center">a</td></tr></tbody></tabular></table>"#,
  );
  if !kpsewhich_has("ltablex.sty") {
    return;
  }
  let tex = "\\documentclass{article}\n\\usepackage{ltablex}\n\\begin{document}\n\\begin{table}\\caption{Outer}\n\\begin{tabularx}{\\linewidth}{X}a\\\\\\end{tabularx}\\label{L}\\addtocounter{table}{-1}\n\\end{table}\n\\begin{table}x\\caption{Next}\\end{table}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "table",
    &[r#"xml:id="S0.T1""#],
    r#"<table inlist="lot" labels="LABEL:L" xml:id="S0.T1"><tags><tag>Table 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Table 1</tag></tags><toccaption><tag close=" ">1</tag>Outer</toccaption><caption><tag close=": ">Table 1</tag>Outer</caption><tags><tag>Table 2</tag><tag role="refnum">2</tag><tag role="typerefnum">Table 2</tag></tags><tabular><tr><td align="left"><inline-block vattach="top"><p>a</p></inline-block></td></tr></tabular></table>"#,
  );
}

/// 57bu.2 A/B: a `\caption` in a minipage (or `\parbox`) of a figure — the box becomes a tagless
/// figure, whose caption's counters the outer figure rescues — collapses into one figure, as Perl
/// and pdflatex's one float. Counting the outer's rescued tags as a caption left the number on the
/// outer and the caption on the inner in 40 papers (104 floats; 2605.01437 Fig. 1). The figure keeps its
/// own id, which its tags carry (`S0.F1`; Perl, promoting the captioned inner's, `S0.F1.fig1`; 57cp, #372).
#[test]
fn caption_in_minipage_collapses_into_its_float() {
  let tex = "\\documentclass{article}\n\\begin{document}\n\\begin{figure}\n\\begin{minipage}{\\linewidth}\nx\n\\caption{Cap}\\label{f}\n\\end{minipage}\n\\end{figure}\n\\end{document}\n";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure class="ltx_minipage" inlist="lof" labels="LABEL:f" vattach="middle" width="345.0pt" xml:id="S0.F1"><tags><tag>Figure 1</tag><tag role="refnum">1</tag><tag role="typerefnum">Figure 1</tag></tags><p>x</p><toccaption><tag close=" ">1</tag>Cap</toccaption><caption><tag close=": ">Figure 1</tag>Cap</caption></figure>"#,
  );
}
