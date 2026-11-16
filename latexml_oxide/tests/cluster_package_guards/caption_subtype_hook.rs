//! caption.sty's `\caption@subtypehook` (caption.sty:639, run on entering a sub-type at :314): defined by the caption
//! binding and run by subcaption's sub-floats (`run_caption_subtype_hook`). The kernel's first aid for subcaption v1.6b
//! (latex2e-first-aid-for-external-files.ltx:197-201, TL 2026) adds to it, so did every paper's own caption copy; the
//! first aid's line is written into the source here, so the guards hold on any TeX Live.
use latexml::util::test::{assert_element, convert_with, error_count, warning_count};

fn convert(tex: &str) -> String {
  let (log, xml) = convert_with(tex, None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// The first aid's addition (`\let\ltx@label\label`) is no Error, and the sub-figure keeps its label (pdflatex: "See a.",
/// no `!`).
#[test]
fn first_aid_adds_to_the_subtype_hook() {
  let xml = convert(
    r"\documentclass{article}
\usepackage{subcaption}
\makeatletter\g@addto@macro\caption@subtypehook{\let\ltx@label\label}\makeatother
\begin{document}
\begin{figure}\begin{subfigure}{.4\textwidth}A\caption{a}\label{fa}\end{subfigure}\caption{F}\end{figure}
See \subref{fa}.
\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1.sf1""#],
    r#"<figure inlist="lof" labels="LABEL:fa" xml:id="S0.F1.sf1">
      <tags>
        <tag><text fontsize="90%">(a)</text></tag>
        <tag role="refnum">1a</tag>
      </tags>
      <p>A</p>
      <toccaption><tag close=" ">a</tag>a</toccaption>
      <caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">a</text></caption>
    </figure>"#,
  );
  assert_element(
    &xml,
    "ref",
    &[r#"labelref="LABEL:fa""#],
    r#"<ref labelref="LABEL:fa"/>"#,
  );
}

/// What a package adds to the hook runs inside the sub-float, in its group: defined there, undefined after the float.
#[test]
fn subtype_hook_runs_inside_the_sub_float() {
  let xml = convert(
    r"\documentclass{article}
\usepackage{subcaption}
\makeatletter\g@addto@macro\caption@subtypehook{\def\lxmark{IN}}\makeatother
\begin{document}
\begin{figure}\begin{subfigure}{.4\textwidth}[\lxmark]\caption{a}\end{subfigure}\caption{F}\end{figure}
After: \ifdefined\lxmark defined\else undefined\fi.
\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1.sf1""#],
    r#"<figure inlist="lof" xml:id="S0.F1.sf1">
      <tags>
        <tag><text fontsize="90%">(a)</text></tag>
        <tag role="refnum">1a</tag>
      </tags>
      <p>[IN]</p>
      <toccaption><tag close=" ">a</tag>a</toccaption>
      <caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">a</text></caption>
    </figure>"#,
  );
  assert!(xml.contains("<p>After: undefined.</p>"), "{xml}");
}

/// caption.sty:325-330 `\@setcaptionsubtype`: entering a sub-type runs the hook once, and not again inside one
/// (`\caption@ifsubtype`): a `{subfigure}` (its `\phantomsubcaption` inside it skips), a `\phantomsubcaption` and a
/// `\subcaption` in a minipage (subcaption.sty:251-252), a `{subcaptiongroup}` with a sub-float and a `\subcaption` in it.
/// pdflatex: "R1: 1.", "R2: 2.", "R3: 3.", "R4: 4.".
#[test]
fn subtype_hook_runs_once_per_sub_type() {
  let xml = convert(
    r"\documentclass{article}
\usepackage{subcaption}
\newcounter{hk}
\makeatletter\g@addto@macro\caption@subtypehook{\stepcounter{hk}}\makeatother
\begin{document}
\begin{figure}\begin{subfigure}{.3\textwidth}A\phantomsubcaption\end{subfigure}\caption{F}\end{figure}
R1: \thehk.

\begin{figure}\begin{minipage}{.3\textwidth}C\phantomsubcaption\end{minipage}\caption{F}\end{figure}
R2: \thehk.

\begin{figure}\begin{minipage}{.3\textwidth}D\subcaption{d}\end{minipage}\caption{F}\end{figure}
R3: \thehk.

\begin{figure}\begin{subcaptiongroup}\begin{subfigure}{.3\textwidth}A\caption{a}\end{subfigure}
\begin{minipage}{.3\textwidth}E\subcaption{e}\end{minipage}\end{subcaptiongroup}\caption{F}\end{figure}
R4: \thehk.
\end{document}",
  );
  let runs: Vec<&str> = xml
    .lines()
    .map(str::trim)
    .filter(|line| line.starts_with("<p>R"))
    .collect();
  assert_eq!(
    runs,
    [
      "<p>R1: 1.</p>",
      "<p>R2: 2.</p>",
      "<p>R3: 3.</p>",
      "<p>R4: 4.</p>"
    ],
    "{xml}"
  );
}

/// What the hook typesets is kept where it runs (pdflatex: "INKA" in the sub-float, "INKC" in the minipage).
#[test]
fn subtype_hook_material_is_kept() {
  let xml = convert(
    r"\documentclass{article}
\usepackage{subcaption}
\makeatletter\g@addto@macro\caption@subtypehook{INK}\makeatother
\begin{document}
\begin{figure}\begin{subfigure}{.4\textwidth}A\caption{a}\end{subfigure}
\begin{minipage}{.4\textwidth}\phantomsubcaption C\end{minipage}\caption{F}\end{figure}
\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1""#],
    r#"<figure class="ltx_panels_gallery" inlist="lof" panelcols="2" panelrows="1" xml:id="S0.F1">
      <tags>
        <tag>Figure 1</tag>
        <tag role="refnum">1</tag>
        <tag role="typerefnum">Figure 1</tag>
      </tags>
      <figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F1.sf1">
        <tags>
          <tag><text fontsize="90%">(a)</text></tag>
          <tag role="refnum">1a</tag>
        </tags>
        <p>INKA</p>
        <toccaption><tag close=" ">a</tag>a</toccaption>
        <caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">a</text></caption>
      </figure>
      <p class="ltx_figure_panel ltx_minipage" panelwidth="0.4" vattach="middle" width="138.0pt">INKC</p>
      <toccaption><tag close=" ">1</tag>F</toccaption>
      <caption><tag close=": ">Figure 1</tag>F</caption>
    </figure>"#,
  );
}
