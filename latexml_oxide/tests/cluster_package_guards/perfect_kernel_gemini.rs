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

fn convert_env_args(tex: &str, extra: &[&str], envs: &[(&str, &str)]) -> (String, String) {
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
/// is pinned through pass 2 and the spine tail (batch 56it).
#[test]
fn align_stale_node_boxes_are_swept() {
  let body: String = (0..300)
    .map(|i| format!("Text {i}.\n\\begin{{align}}a_{{{i}}}&=b+c\\\\d&=e\\end{{align}}\n\n"))
    .collect();
  let tex = format!(
    "\\documentclass{{article}}\n\\usepackage{{amsmath}}\n\\begin{{document}}\n{body}\\end{{document}}\n"
  );
  let (stderr, xml) = convert_env_args(&tex, &["--streaming", "--max-memory=768"], &[(
    "LXML_TRACE_NODE_BOXES",
    "1",
  )]);
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
#[test]
fn notebeamer_pagecount_dvips_fallback() {
  let tex = r"\documentclass{article}
\usepackage{notebeamer}
\begin{document}
\includebeamer[nup=1,pages=1]{example-image-a4.pdf}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert!(xml.contains("<graphics"), "{xml}");
}

/// hypdestopt binding and svn-multi \svnrev, \svnmonth, \svnauthor stubs (witness: biblatex-cheatsheet/biblatex-cheatsheet).
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
  // The svn-multi binding announces itself as a stub (one intrinsic warning).
  assert_eq!(
    stderr.matches("Warning:missing_file:svn-multi.sty").count(),
    1,
    "{stderr}"
  );
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

#[test]
fn index_string_of_undefined_command_stringifies_its_name() {
  let tex = r"\documentclass{article}
\usepackage{makeidx}\makeindex
\makeatletter
\gdef\stripcommand#1{\expandafter\@gobble\string#1}
\makeatother
\def\myprintcommand#1{\texttt{\char`\\#1}}
\begin{document}
\index{\stripcommand\insertframetitle @\protect\myprintcommand{\stripcommand\insertframetitle}}
\index{\stripcommand\ifbeamercolorempty @\protect\myprintcommand{\stripcommand\ifbeamercolorempty}}
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
