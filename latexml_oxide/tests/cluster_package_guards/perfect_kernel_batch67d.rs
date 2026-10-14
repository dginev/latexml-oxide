//! Red/green guards for batch 67d: Paul Taylor's `diagrams` package (not in TeX Live, in arXiv's tree; ~470 arXiv
//! papers) as a native binding (latexml_contrib/src/diagrams_tex.rs): a `{diagram}` is a math array of its objects
//! and labelled arrows, where Perl's ar5iv binding put `<ERROR>{diagram}</ERROR>` (OXIDIZED_DESIGN_DIVERGENCES
//! #589). Witnesses: math0001049, math-ph0001006, math0003019, math-ph0007001, math0010298, math0012073,
//! math0102115, math0104065, math0104175, math0104206, math0403203, math0404431.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The XML without its ids and the white space between elements, so a whole element reads as one string.
fn squashed(xml: &str) -> String {
  let mut out = String::with_capacity(xml.len());
  let mut rest = xml;
  while let Some(at) = rest.find(" xml:id=\"") {
    out.push_str(&rest[..at]);
    let after = &rest[at + 9..];
    rest = &after[after.find('"').map_or(after.len(), |e| e + 1)..];
  }
  out.push_str(rest);
  let mut squashed = String::with_capacity(out.len());
  let mut chars = out.chars().peekable();
  while let Some(c) = chars.next() {
    squashed.push(c);
    if c == '>' {
      let mut gap = String::new();
      while let Some(&w) = chars.peek() {
        if !w.is_whitespace() {
          break;
        }
        gap.push(w);
        chars.next();
      }
      if chars.peek() != Some(&'<') {
        squashed.push_str(&gap);
      }
    }
  }
  squashed
}

const RIGHT: &str = r#"<XMTok class="ltx_horizontally_stretchy" role="ARROW" stretchy="true" width="30.0pt">→</XMTok>"#;

/// math0001049's exact rows: the objects and arrows are the cells of one array, a horizontal arrow's labels are
/// its scripts, a vertical's stand beside it; the inline `$A \rTo^f B$` is a relation.
#[test]
fn diagrams_commutative_square() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/diagrams_commutative_square.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_eq!(
    xml
      .matches(r#"<XMTok meaning="commutative-diagram"/>"#)
      .count(),
    1,
    "{xml}"
  );
  assert_eq!(xml.matches("<XMRow>").count(), 3, "{xml}");
  assert_eq!(
    xml
      .matches(&format!(r#"<XMCell align="center">{RIGHT}</XMCell>"#))
      .count(),
    6,
    "{xml}"
  );
  for cell in [
    // \rTo^{i}_{\cong}
    format!(
      r#"<XMCell align="center"><XMApp role="ARROW"><XMTok role="SUBSCRIPTOP" scriptpos="mid5"/><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="mid5"/>{RIGHT}<XMTok font="italic" fontsize="70%" role="UNKNOWN">i</XMTok></XMApp><XMTok fontsize="70%" meaning="approximately-equals" name="cong" role="RELOP">≅</XMTok></XMApp></XMCell>"#
    ),
    // \dTo<{\alpha}
    String::from(
      r#"<XMCell align="center"><XMWrap role="ARROW" rule="Anything,"><XMTok font="italic" fontsize="70%" name="alpha" role="UNKNOWN" width="0.0pt">α</XMTok><XMTok role="ARROW" stretchy="true">↓</XMTok></XMWrap></XMCell>"#,
    ),
    // \dEqual (\newarrow{Equal} =====)
    String::from(
      r#"<XMCell align="center"><XMTok role="ARROW" stretchy="true">‖</XMTok></XMCell>"#,
    ),
    // \dOnto>{\beta} (\newarrow{Onto} ----{>>})
    String::from(
      r#"<XMCell align="center"><XMApp role="ARROW"><XMTok role="ARROW" stretchy="true">↡</XMTok><XMTok meaning="absent"/><XMTok font="italic" fontsize="70%" name="beta" role="UNKNOWN" width="0.0pt">β</XMTok></XMApp></XMCell>"#,
    ),
    // \rInto (\newarrow{Into} C--->)
    String::from(
      r#"<XMCell align="center"><XMTok class="ltx_horizontally_stretchy" role="ARROW" stretchy="true" width="30.0pt">↪</XMTok></XMCell>"#,
    ),
  ] {
    assert!(xml.contains(&cell), "{cell}\n{xml}");
  }
  let inline = format!(
    r#"<Math mode="inline" tex="A\rTo^{{f}}B" text="A → ^ f B"><XMath><XMApp><XMApp role="ARROW"><XMTok role="SUPERSCRIPTOP" scriptpos="mid1"/>{RIGHT}<XMTok font="italic" fontsize="70%" role="UNKNOWN">f</XMTok></XMApp><XMTok font="italic" role="UNKNOWN">A</XMTok><XMTok font="italic" role="UNKNOWN">B</XMTok></XMApp></XMath></Math>"#
  );
  assert!(xml.contains(&inline), "{inline}\n{xml}");
}

/// Outside maths a diagram is a display, `[LaTeXeqno]` a numbered equation with the diagram's `\label`; `\pile`
/// is a nested array; diagonals take a span, options and a middle label; `(…)` after a text arrow that is no
/// span stays the arrow's target (math0104065, math0104175, math0012073, math0404431, cs0204036).
#[test]
fn diagrams_display_pile_diagonals() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/diagrams_display_pile_diagonals.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_eq!(
    xml
      .matches(r#"<XMTok meaning="commutative-diagram"/>"#)
      .count(),
    2,
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<Math class="ltx_math_unparsed" mode="display" tex="\begin{diagram}X&amp;\pile{\rTo^{a}\\&#10;\rTo_{b}}&amp;Y\\&#10;&amp;\rdTo^{h}\ldTo~{k}&amp;\uTo&lt;{p}&gt;{q}\\&#10;U&amp;\rDash~{x}&amp;\end{diagram}""#
    ),
    "{xml}"
  );
  let pile = format!(
    r#"<XMCell align="center"><XMArray><XMRow><XMCell align="center"><XMApp role="ARROW"><XMTok role="SUPERSCRIPTOP" scriptpos="mid9"/>{RIGHT}<XMTok font="italic" fontsize="70%" role="UNKNOWN">a</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align="center"><XMApp role="ARROW"><XMTok role="SUBSCRIPTOP" scriptpos="mid9"/>{RIGHT}<XMTok font="italic" fontsize="70%" role="UNKNOWN">b</XMTok></XMApp></XMCell></XMRow></XMArray></XMCell>"#
  );
  assert!(xml.contains(&pile), "{pile}\n{xml}");
  for cell in [
    r#"<XMApp role="ARROW"><XMTok role="SUPERSCRIPTOP" scriptpos="mid5"/><XMTok role="ARROW">↘</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">h</XMTok></XMApp>"#,
    r#"<XMApp role="ARROW"><XMTok role="SUPERSCRIPTOP" scriptpos="mid5"/><XMTok role="ARROW">↙</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">k</XMTok></XMApp>"#,
    r#"<XMCell align="center"><XMWrap role="ARROW" rule="Anything,"><XMTok font="italic" fontsize="70%" role="UNKNOWN" width="0.0pt">p</XMTok><XMTok role="ARROW" stretchy="true">↑</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN" width="0.0pt">q</XMTok></XMWrap></XMCell>"#,
    r#"<XMCell align="center"><XMApp role="ARROW"><XMTok role="SUPERSCRIPTOP" scriptpos="mid5"/><XMTok class="ltx_horizontally_stretchy" role="ARROW" stretchy="true" width="30.0pt">╌</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">x</XMTok></XMApp></XMCell>"#,
  ] {
    assert!(xml.contains(cell), "{cell}\n{xml}");
  }
  assert!(
    xml.contains(
      r#"<equation labels="LABEL:eq:d"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="\begin{diagram}P&amp;\rMapsto&amp;Q\end{diagram}" text="commutative-diagram@(Array[[P, ↦, Q]])">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"as (<ref labelref="LABEL:eq:d"/>) shows;"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<Math mode="inline" tex="\mathrm{Sel}" text="Sel">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<Math mode="inline" tex="f(x)\rTo(\mathcal{F})" text="f@(x) → F">"#),
    "{xml}"
  );
}

/// `\newarrow` draws each arrow with the Unicode arrow of its components, in every direction; the source's
/// synonyms (`\hLine`, `\hEq`, `\rInto`) keep their names (math-ph0007001, math0010298, math0102115).
#[test]
fn diagrams_newarrow_glyphs() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/diagrams_newarrow_glyphs.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert!(
    xml.contains(
      r#"text="commutative-diagram@(Array[[A, ↣, B, ↠, C, ↔, D, ←, E, ⇔, F], [⋮, , ⇣, , ⦀, , ↧, , ⇓, , ↖], [G, —, H, =, I, ↪, J, ⇀, K]])""#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"G&amp;\hLine&amp;H&amp;\hEq&amp;I&amp;\rInto&amp;J&amp;\rPto&amp;K\end{diagram}""#
    ),
    "{xml}"
  );
}

/// `\newarrow` in maths is "ignored within maths" (diagrams.tex:615), with the package's warning.
#[test]
fn diagrams_newarrow_in_math_is_ignored() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{diagrams}\n\\begin{document}\n$x \\newarrow{Foo}---->y$ \
     and $A\\rTo B$.\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("diagrams: Foo ignored within maths"), "{log}");
  let xml = squashed(&xml);
  assert!(
    xml.contains(r#"<Math mode="inline" tex="xy" text="x * y">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<Math mode="inline" tex="A\rTo B" text="A → B">"#),
    "{xml}"
  );
}

/// The package is plain TeX's too (`\input diagrams`): a text-mode diagram is a `$$` display, not LaTeX's `\[`
/// (math0105166).
#[test]
fn diagrams_plain_tex() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/diagrams_plain_tex.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert_eq!(xml.matches("<equation>").count(), 2, "{xml}");
  assert!(
    xml.contains(
      r#"<Math mode="display" tex="\begin{diagram}A&amp;\rTo^{f}&amp;B\end{diagram}" text="commutative-diagram@(Array[[A, → ^ f, B]])">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<p>and a text-mode</p><equation><Math mode="display" tex="\begin{diagram}C&amp;\rTo&amp;D\end{diagram}" text="commutative-diagram@(Array[[C, →, D]])">"#
    ),
    "{xml}"
  );
  assert!(
    xml.contains(r#"</equation><p>and more text here.</p>"#),
    "{xml}"
  );
}

/// An arrow typed in text is the package's error (diagrams.tex `\CD@nB` → `\CD@E`, pdflatex `! Commutative Diagram:
/// "$" inserted before horizontal map.`); the arrow alone is set as a `$` formula, not the paragraph's rest
/// (OXIDIZED_DESIGN_DIVERGENCES #589; `$`, not `\ensuremath`, for plain TeX, math0105166).
#[test]
fn diagrams_text_arrow_is_the_package_error() {
  for (preamble, end) in [
    (
      "\\documentclass{article}\n\\usepackage{diagrams}\n\\begin{document}\n",
      "\\end{document}\n",
    ),
    ("\\input diagrams\n", "\\bye\n"),
  ] {
    let (log, xml) = convert_counted(&format!("{preamble}Text \\rTo{{}} word.\n{end}"));
    assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
    assert!(
      log.contains(
        r#"Error:unexpected:\rTo Commutative Diagram: "$" inserted before horizontal map"#
      ),
      "{log}"
    );
    let xml = squashed(&xml);
    assert!(
      xml.contains(&format!(
        r#"<p>Text <Math mode="inline" tex="\rTo" text="→"><XMath>{RIGHT}</XMath></Math> word.</p>"#
      )),
      "{xml}"
    );
  }
}

/// Outside a diagram an arrow reads `^ _ [ . ~`, a space or a `{` (diagrams.tex:301-306 `\switch@arg`); `(x,y)`, `<`
/// and `>` are read only inside one (:290-299, :729), so `$X\rTo (0,1)$` keeps its pair (pdflatex "X → (0, 1)").
/// A `{` after `^f` is the lower label, after a lower label content (`\positional@`); a `.` (dotted shaft) is read.
#[test]
fn diagrams_arrows_outside_a_diagram() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/diagrams_arrows_outside_a_diagram.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  for math in [
    format!(
      r#"<Math mode="inline" tex="X\rTo(0,1)" text="X → vector@(0, 1)"><XMath><XMApp>{RIGHT}<XMTok font="italic" role="UNKNOWN">X</XMTok><XMDual><XMApp><XMTok meaning="vector"/><XMRef idref="p1.m1.1"/><XMRef idref="p1.m1.2"/></XMApp><XMWrap><XMTok role="OPEN" stretchy="false">(</XMTok><XMTok meaning="0" role="NUMBER">0</XMTok><XMTok role="PUNCT">,</XMTok><XMTok meaning="1" role="NUMBER">1</XMTok><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>"#
    ),
    String::from(
      r#"<Math mode="inline" tex="f\colon X\rTo(0,\infty)" text="f colon X → open-interval@(0, infinity)">"#,
    ),
    String::from(r#"<Math mode="inline" tex="A\rTo&lt;B" text="A → absent &lt; B">"#),
    String::from(r#"<Math mode="inline" tex="C\rTo&gt;D" text="C → absent &gt; D">"#),
    format!(
      r#"<Math mode="inline" tex="P\rTo Q" text="P → Q"><XMath><XMApp>{RIGHT}<XMTok font="italic" role="UNKNOWN">P</XMTok><XMTok font="italic" role="UNKNOWN">Q</XMTok></XMApp></XMath></Math>"#
    ),
    String::from(r#"<Math mode="inline" tex="R\rTo^{f}_{g}S" text="R (→ ^ f) _ g S">"#),
    String::from(r#"<Math mode="inline" tex="T\rTo_{g}{h}U" text="T → _ g h * U">"#),
    String::from(
      r#"<Math mode="display" tex="f\colon X\rTo(0,1)" text="f colon X → vector@(0, 1)">"#,
    ),
  ] {
    assert!(xml.contains(&math), "{math}\n{xml}");
  }
  // In the diagram the span is read and a vertical's `<`/`>` are its labels.
  assert!(
    xml.contains(r#"text="commutative-diagram@(Array[[G, →, H, x@↓@y, K]])""#),
    "{xml}"
  );
}

/// The `amstex` option, as a package option or in `\diagramstyle`, enables AmS-TeX's `{CD}` (manual "Options",
/// diagrams.tex `\cds@amstex`), the amscd binding's; `\diagramstyle{…}` reads one braced argument when no `[` follows
/// (diagrams.tex:282-285), so a body `\diagramstyle{centredisplay}` prints nothing (math0104206, math0403203).
#[test]
fn diagrams_amstex_option_enables_cd() {
  for setup in [
    "\\usepackage{diagrams}\n\\diagramstyle{PostScript=dvips,nohug,amstex,labelstyle=\\scriptstyle}\n",
    "\\usepackage{diagrams}\n\\diagramstyle[amstex]\n",
    "\\usepackage[amstex]{diagrams}\n",
  ] {
    let (log, xml) = convert_counted(&format!(
      "\\documentclass{{article}}\n{setup}\\begin{{document}}\n$\\begin{{CD}} A @>f>> B \\end{{CD}}$\n\
       \\diagramstyle{{centredisplay}}done.\n\\end{{document}}\n"
    ));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{setup}\n{log}"
    );
    let xml = squashed(&xml);
    assert!(
      xml.contains(
        r#"<Math mode="inline" tex="\begin{CD}A@&gt;{f}&gt;{}&gt;B\end{CD}" text="commutative-diagram@(Array[[A, rightarrow ^ f, B]])">"#
      ),
      "{setup}\n{xml}"
    );
    assert!(
      xml.contains("done.") && !xml.contains("centredisplay"),
      "{setup}\n{xml}"
    );
  }
}

/// In restricted horizontal mode (a `c` cell, an LR box) a diagram is an inline formula, `$$` being an empty
/// formula there; in a minipage, a `p` cell or a float it is a display (math0010298, math0109209).
#[test]
fn diagrams_in_cells_and_boxes() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/diagrams_in_cells_and_boxes.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert_eq!(
    xml
      .matches(r#"<XMTok meaning="commutative-diagram"/>"#)
      .count(),
    5,
    "{xml}"
  );
  for (mode, objects) in [
    ("inline", "A * ddot@(o), → ^ similar-to, B"),
    ("inline", "C, →, D"),
    ("display", "E, →, F"),
    ("display", "G, →, H"),
    ("display", "I, →, J"),
  ] {
    let math = format!(r#"<Math mode="{mode}" tex="\begin{{diagram}}"#);
    let text = format!(r#"text="commutative-diagram@(Array[[{objects}]])""#);
    assert!(
      xml
        .split("<Math ")
        .any(|m| format!("<Math {m}").starts_with(&math) && m.contains(&text)),
      "{mode} {objects}\n{xml}"
    );
  }
  assert_eq!(xml.matches("<equation>").count(), 3, "{xml}");
}

/// An arrow of no components draws nothing; its label stays (`\newarrow{Nul}{}{}{}{}{}`, math0310311).
#[test]
fn diagrams_component_less_arrow_draws_nothing() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{diagrams}\n\\newarrow{Nul}{}{}{}{}{}\n\\begin{document}\n\
     \\[\\begin{diagram}A&&B\\\\&\\ruNul^{e}&\\end{diagram}\\]\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert!(
    xml.contains(
      r#"<XMCell align="center"><XMApp role="ARROW"><XMTok role="SUPERSCRIPTOP" scriptpos="mid5"/><XMTok meaning="absent"/><XMTok font="italic" fontsize="70%" role="UNKNOWN">e</XMTok></XMApp></XMCell>"#
    ),
    "{xml}"
  );
}

/// In a diagram `\\` is `\cr` (diagrams.tex:729, :739) and reads no `[dim]`: a row may begin with a bracket, and a
/// last `\\` makes no empty row (math0109209 `\\ [x,yK]`, math0403203 `\\ [0]`).
#[test]
fn diagrams_row_may_begin_with_a_bracket() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{diagrams}\n\\begin{document}\n\\begin{diagram}\n\
     &&& [x,y] \\\\\n& \\ldMapsto & &&& \\rdMapsto \\\\\n[x,yK] &&&&&& [x,ygK]\\\\\n\\end{diagram}\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  assert!(
    xml.contains(
      r#"text="commutative-diagram@(Array[[, , , closed-interval@(x, y)], [, ↙, , , , ↘], [closed-interval@(x, y * K), , , , , , closed-interval@(x, y * g * K)]])""#
    ),
    "{xml}"
  );
}

/// An arrow or a diagram at the start of an alignment cell is in that cell's maths: the mode is tested when it is
/// digested, not when the look-ahead for `\omit` expands it before the cell's math begins (the source's arrows begin
/// with `\relax`; `&\rTo` in `align*`, math0404431).
#[test]
fn diagrams_at_alignment_cell_starts() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\usepackage{diagrams}\n\\begin{document}\n\
     \\begin{align*}\n \\phi: H^0 &\\rTo H^1,\\\\\n f &\\rMapsto g.\n\\end{align*}\n\
     \\begin{align}\nx &\\begin{diagram} A & \\rTo & B\\end{diagram}\\\\\n\\diagram C & \\rTo & D\\enddiagram & y\n\
     \\end{align}\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squashed(&xml);
  for math in [
    r#"<Math tex="\displaystyle\phi:H^{0}\rTo H^{1}," text="phi colon H ^ 0 → H ^ 1">"#,
    r#"<Math tex="\displaystyle f\rMapsto g." text="f ↦ g">"#,
    r#"<Math tex="\displaystyle x\begin{diagram}A&amp;\rTo&amp;B\end{diagram}" text="x * commutative-diagram@(Array[[A, →, B]])">"#,
    r#"<Math tex="\displaystyle\begin{diagram}C&amp;\rTo&amp;D\end{diagram}y" text="commutative-diagram@(Array[[C, →, D]]) * y">"#,
  ] {
    assert!(xml.contains(math), "{math}\n{xml}");
  }
}
