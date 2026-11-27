//! caption.sty's `\caption@subtypehook` (caption.sty:639, run on entering a sub-type at :314): defined by the caption
//! binding and run once on entering a sub-type, in its group, its material kept (`run_caption_subtype_hook`;
//! OXIDIZED_DESIGN_DIVERGENCES #853). The kernel's first aid for subcaption v1.6b
//! (latex2e-first-aid-for-external-files.ltx:197-201, TL 2026) adds to it; the first aid's line is written into the
//! source here, so the guards hold on any TeX Live.
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

/// What a package adds to the hook runs inside the sub-float, in its group: defined there, undefined after the
/// sub-float, before the float's own caption (pdflatex: "[IN]", "Mid: undefined.").
#[test]
fn subtype_hook_runs_inside_the_sub_float() {
  let xml = convert(
    r"\documentclass{article}
\usepackage{subcaption}
\makeatletter\g@addto@macro\caption@subtypehook{\def\lxmark{IN}}\makeatother
\begin{document}
\begin{figure}\begin{subfigure}{.4\textwidth}[\lxmark]\caption{a}\end{subfigure}
Mid: \ifdefined\lxmark defined\else undefined\fi.
\caption{F}\end{figure}
\end{document}",
  );
  assert_element(
    &xml,
    "figure",
    &[r#"xml:id="S0.F1.sf1""#],
    r#"<figure class="ltx_figure_panel" inlist="lof" xml:id="S0.F1.sf1">
      <tags>
        <tag><text fontsize="90%">(a)</text></tag>
        <tag role="refnum">1a</tag>
      </tags>
      <p>[IN]</p>
      <toccaption><tag close=" ">a</tag>a</toccaption>
      <caption><tag close=" "><text fontsize="90%">(a)</text></tag><text fontsize="90%">a</text></caption>
    </figure>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"class="ltx_figure_panel""#],
    r#"<p class="ltx_figure_panel">Mid: undefined.</p>"#,
  );
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

/// Each figure and table, nested as in the XML: `id[labels]=refnum`.
fn float_numbers(xml: &str) -> String {
  fn walk(node: roxmltree::Node, out: &mut Vec<String>) {
    for child in node.children().filter(|c| c.is_element()) {
      let mut inner = Vec::new();
      walk(child, &mut inner);
      let id = child.attribute(("http://www.w3.org/XML/1998/namespace", "id"));
      if let (Some(id), "figure" | "table") = (id, child.tag_name().name()) {
        let refnum = child
          .descendants()
          .find(|t| t.attribute("role") == Some("refnum"))
          .and_then(|t| t.text())
          .unwrap_or("");
        let labels = child
          .attribute("labels")
          .unwrap_or("")
          .replace("LABEL:", "");
        let inner = if inner.is_empty() {
          String::new()
        } else {
          format!("({})", inner.join(" "))
        };
        out.push(format!("{id}[{labels}]={refnum}{inner}"));
      } else {
        out.extend(inner);
      }
    }
  }
  let doc = roxmltree::Document::parse(xml).expect("well-formed XML");
  let mut out = Vec::new();
  walk(doc.root_element(), &mut out);
  out.join(" ")
}

/// The hook steps the float's counter once, on entering the sub-type (caption.sty:639-642): a sub-float opened in a
/// sub-float, the first of its float, steps nothing (:325-331). Red (INT081AU, Perl the same, KNOWN_PERL_ERRORS #324):
/// 2502.06601's Figure 3 was S4.F4, every later figure one too high. Rows: nested sub-figures (deep, `figure*`, in a
/// group, a `\subcaption` within, after the caption); a sub-table in a sub-figure, whose leaked table step numbered the
/// next section's table 1.1; a sibling closed first (2208.13192, a control).
#[test]
fn nested_subfigure_steps_figure_once() {
  const ROWS: &[(&str, &str)] = &[
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/captions-floats/nested_subfigure_steps_the_figure_once.tex"
      ),
      "S0.F1[]=1 S0.F2[fig:two]=2(S0.F2.sf1[sub:in]=2a) \
       S0.F3[fig:three]=3(S0.F3.sf1[sub:deep]=3a S0.F3.sf2[sub:right]=3b) \
       S0.F4[fig:four]=4(S0.F4.sf1[sub:wide]=4a) S0.F5[fig:five]=5(S0.F5.sf1[sub:grp]=5a) \
       S0.F6[fig:six]=6(S0.F6.sf1[sub:sc]=6a) S0.F7[fig:seven]=7(S0.F7.sf1[sub:after]=7a) S0.F8[fig:eight]=8",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/captions-floats/subtable_in_a_subfigure_steps_no_table.tex"
      ),
      // The inner panel's "0a" is known-wrong: pdflatex makes it the sub-figure 2a (subtable_in_a_figure_is_a_subfigure_red).
      "S1.F1[]=1 S1.F2[fig:two]=2(S1.F2.sf1[sub:out]=2a(S1.T0.st1[sub:in]=0a)) S2.T1[tab:one]=2.1 \
       S2.F3[fig:three]=3",
    ),
    (
      include_str!(
        "../../../tools/perfect_kernel/repros/captions-floats/a_subfigure_closed_before_a_nested_one_steps_once.tex"
      ),
      "S0.F1[]=1 S0.F2[fig:two]=2(S0.F2.sf1[sub:left]=2a S0.F2.sf2[sub:in]=2b) S0.F3[fig:three]=3",
    ),
  ];
  for (tex, expected) in ROWS {
    let numbers = float_numbers(&convert(tex));
    assert_eq!(
      numbers.split_whitespace().collect::<Vec<_>>(),
      expected.split_whitespace().collect::<Vec<_>>()
    );
  }
}
