//! LaTeX's text-font state follows every text font switch (W8). The bindings
//! switch the font directly while LaTeX goes through `\f@family`,
//! `\f@series`, `\f@shape`, `\f@size` and `\selectfont`. So the merge now
//! writes those codes back (content.rs `merge_font_ref`), a class binding's
//! size switch ends in `\selectfont` as `\@setfontsize` does
//! (latex.ltx:14103-14107; dialect.rs), and `\the\font` names the current
//! font (latex.ltx:12576-12579; tex_fonts.rs `current_font_identifier`).
//! Repros are in `tools/perfect_kernel/repros/fonts-nfss/`. The controls pin
//! switches that must survive a size switch, the `\emph` toggle, which Perl
//! renders the same way, math, whose font a `\small` leaves alone, and a pgf
//! picture, whose `\selectfont` keeps `\nullfont`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert, error_count, warning_count};

#[test]
fn macrofont_small_selects_the_pending_typewriter_family() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/macrofont_small_selects_tt.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p><text font="typewriter" fontsize="90%">\defbeamertemplate{footline}</text></p></para>"#,
  );
}

#[test]
fn old_font_switch_survives_a_raw_size_switch() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/old_font_switch_then_size.tex");
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  // scrartcl's own `\@startsection`/`\@sect` checks (scrartcl.cls), raised
  // with or without this change.
  assert_eq!(warning_count(&stderr), 2, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p><text font="typewriter" fontsize="91%">\foo</text> <text font="bold" fontsize="91%">x</text></p></para>"#,
  );
}

#[test]
fn the_font_names_the_current_font() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/the_font_names_the_current_font.tex"
  );
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p><text font="typewriter">\foo{x}</text></p></para>"#,
  );
}

/// `\fontname`, `\hyphenchar` and `\meaning` see the current font through
/// `\font`, as pdflatex does: the typewriter font is cmtt10, loaded with
/// `\hyphenchar` -1 (ot1cmtt.fd:3), `\small\bfseries` is cmbx9, and a
/// `\hyphenchar` set in one font stays with that font (TeX's font memory),
/// where every write used to land on cmr10. The kernel format leaves
/// `\OT1/cmr/m/n/10` `\relax`; it is defined as a font then, as `\pickup@font`
/// does (latex.ltx:10582-10585).
#[test]
fn font_primitives_see_the_current_font() {
  let tex = r"\documentclass{article}
\begin{document}
{\ttfamily \fontname\font; \the\hyphenchar\font}

{\small\bfseries \fontname\font}

{\ttfamily\edef\x{\the\font}\expandafter\meaning\x}

{\bfseries \hyphenchar\font=-1 }\fontname\font; \the\hyphenchar\font

{\ttfamily\edef\x{\the\font}\rmfamily\expandafter\meaning\the\font}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let expected = [
    r#"<para xml:id="p1"><p><text font="typewriter">cmtt10; -1</text></p></para>"#,
    r#"<para xml:id="p2"><p><text font="bold" fontsize="90%">cmbx9</text></p></para>"#,
    r#"<para xml:id="p3"><p><text font="typewriter">select font cmtt10</text></p></para>"#,
    r#"<para xml:id="p4"><p>cmr10; 45</p></para>"#,
    r#"<para xml:id="p5"><p>select font cmr10</p></para>"#,
  ];
  for (n, want) in expected.iter().enumerate() {
    let id = format!(r#"xml:id="p{}""#, n + 1);
    assert_element(&xml, "para", &[&id], want);
  }
}

/// The same path as the scrartcl repro, warning-free: a raw `\small`
/// (`\@setfontsize`, whose `\fontsize…\selectfont` re-selects the NFSS
/// codes) after plain's `\tt`/`\bf`, which now write them.
#[test]
fn raw_size_switch_after_old_font_switches() {
  let tex = r"\documentclass{article}
\makeatletter\renewcommand\small{\@setfontsize\small\@ixpt{11}}\makeatother
\begin{document}
{\tt\small \string\foo} {\bf\small x}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p><text font="typewriter" fontsize="90%">\foo</text> <text font="bold" fontsize="90%">x</text></p></para>"#,
  );
}

/// A note and an equation tag reset their font (`neutralize_font`), and the
/// NFSS codes with it, as `\@footnotetext`'s `\reset@font\footnotesize` does
/// (latex.ltx:17658-17659): a size switch inside re-selects upright, medium
/// roman — not the italic, bold or typewriter around the note — as in Perl and
/// pdflatex (cmr8, cmr9).
#[test]
fn font_reset_in_notes_and_tags_resets_nfss() {
  let tex = r"\documentclass{article}
\renewcommand\theequation{\small\arabic{equation}}
\begin{document}
\textit{A\footnote{\footnotesize note}} \textbf{B\footnote{\small bnote}} \texttt{C\footnote{\small cnote}}

{\bfseries\itshape Bold
\begin{equation}a=b\end{equation}
}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "note",
    &[r#"xml:id="footnote1""#],
    r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag><text font="upright">1</text></tag><tag role="refnum"><text font="upright">1</text></tag><tag role="typerefnum"><text font="upright">footnote 1</text></tag></tags><text font="upright" fontsize="80%">note</text></note>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"xml:id="footnote2""#],
    r#"<note mark="2" role="footnote" xml:id="footnote2"><tags><tag><text font="medium">2</text></tag><tag role="refnum"><text font="medium">2</text></tag><tag role="typerefnum"><text font="medium">footnote 2</text></tag></tags><text font="medium" fontsize="90%">bnote</text></note>"#,
  );
  assert_element(
    &xml,
    "note",
    &[r#"xml:id="footnote3""#],
    r#"<note mark="3" role="footnote" xml:id="footnote3"><tags><tag><text font="serif">3</text></tag><tag role="refnum"><text font="serif">3</text></tag><tag role="typerefnum"><text font="serif">footnote 3</text></tag></tags><text font="serif" fontsize="90%">cnote</text></note>"#,
  );
  assert_element(
    &xml,
    "equation",
    &[r#"xml:id="S0.E1""#],
    r#"<equation xml:id="S0.E1"><tags><tag>(<text fontsize="90%">1)</text></tag><tag role="refnum"><text fontsize="90%">1</text></tag></tags><Math mode="display" tex="a=b" text="a = b" xml:id="S0.E1.m1"><XMath><XMApp><XMTok meaning="equals" role="RELOP">=</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></equation>"#,
  );
}

/// Codes a size switch now re-selects must name the font: `\textnormal` in
/// math sets `\f@family` to `\rmdefault` — Perl's `cmtt`
/// (latex_constructs.pool.ltxml:5267) turns `y` typewriter in Perl and the
/// tip, and would turn `x` too now that `\small` re-selects; pdflatex: cmr9,
/// cmr10 — and yfonts' families have codes (`ygoth`, kept by pdflatex).
#[test]
fn size_switch_reselects_named_families() {
  let tex = r"\documentclass{article}
\usepackage{yfonts}
\begin{document}
{\ttfamily $\textnormal{\small x}$}

{\gothfamily\small G} \textgoth{\small H}

$\textnormal{\selectfont y}$
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p><text class="ltx_markedasmath" fontsize="90%">x</text></p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p><text font="gothic" fontsize="90%">G</text> <text font="gothic" fontsize="90%">H</text></p></para>"#,
  );
  // Perl and the tip: `font="typewriter"`.
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p3""#],
    r#"<para xml:id="p3"><p><text class="ltx_markedasmath">y</text></p></para>"#,
  );
}

/// Font switches a size switch must keep, now that it re-selects the NFSS
/// codes: bold, typewriter, `\em`'s italic (plain_base.rs routes it through
/// the merge), plain's `\bf`, a family+series pair; in math a `\small` leaves
/// the math font alone. The `\emph` cases are the toggle the merge now keeps
/// in `\f@shape` (Perl's `\f@shape` hook, latex_constructs.pool.ltxml:414-416,
/// is gone), identical to same-host Perl.
#[test]
fn size_switch_keeps_the_font_in_force() {
  let tex = r"\documentclass{article}
\begin{document}
A \textbf{\small x}.

B {\ttfamily\small y}.

C {\ttfamily $\small z$}.

D {\em\small w}.

E \emph{a \emph{b} c}.

F \textit{\emph{d}}.

G \emph{\small e}.

H {\itshape\emph{f}}.

I {\bf\small g}.

J {\sffamily\bfseries\small h}.
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let expected = [
    r#"<para xml:id="p1"><p>A <text font="bold" fontsize="90%">x</text>.</p></para>"#,
    r#"<para xml:id="p2"><p>B <text font="typewriter" fontsize="90%">y</text>.</p></para>"#,
    r#"<para xml:id="p3"><p>C <Math mode="inline" tex="\small z" text="z" xml:id="p3.m1"><XMath><XMTok font="italic" fontsize="90%" role="UNKNOWN">z</XMTok></XMath></Math>.</p></para>"#,
    r#"<para xml:id="p4"><p>D <text font="italic" fontsize="90%">w</text>.</p></para>"#,
    r#"<para xml:id="p5"><p>E <emph font="italic">a <emph font="upright">b</emph> c</emph>.</p></para>"#,
    r#"<para xml:id="p6"><p>F <emph>d</emph>.</p></para>"#,
    r#"<para xml:id="p7"><p>G <emph font="italic" fontsize="90%">e</emph>.</p></para>"#,
    r#"<para xml:id="p8"><p>H <emph>f</emph>.</p></para>"#,
    r#"<para xml:id="p9"><p>I <text font="bold" fontsize="90%">g</text>.</p></para>"#,
    r#"<para xml:id="p10"><p>J <text font="sansserif bold" fontsize="90%">h</text>.</p></para>"#,
  ];
  for (n, want) in expected.iter().enumerate() {
    let id = format!(r#"xml:id="p{}""#, n + 1);
    assert_element(&xml, "para", &[&id], want);
  }
}

/// A size switch ends in whatever `\selectfont` means where it runs: inside a
/// pgf picture that is `\pgf@selectfont`, back to `\nullfont`
/// (pgfcorescopes.code.tex:243, 309), so stray picture text stays dropped as
/// in pdflatex, while node text keeps its size and family.
#[test]
fn size_switch_in_a_pgf_picture_keeps_nullfont() {
  let tex = r"\documentclass{article}
\usepackage{tikz}
\begin{document}
\begin{tikzpicture}\small stray \node[font=\small]{A}; \node at (1,0) {\ttfamily\small B};\end{tikzpicture}
\end{document}
";
  let (stderr, xml) = convert(tex, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert!(!xml.contains("stray"), "stray picture text leaked:\n{xml}");
  assert_element(
    &xml,
    "text",
    &[r#"fontsize="90%""#],
    r#"<text fontsize="90%">A</text>"#,
  );
  assert_element(
    &xml,
    "text",
    &[r#"font="typewriter""#],
    r#"<text font="typewriter" fontsize="90%">B</text>"#,
  );
}
