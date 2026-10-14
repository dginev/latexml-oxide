//! Red/green guards for perfect-kernel batch 66w: OXIDIZED_DESIGN #553 narrowed (user ruling 2026-10-14). A `\par`
//! read in an inline formula looks ahead, unexpanded, to the formula's would-be closer: a math-only token before it
//! keeps the formula open across the blank line (Perl's content; hep-th9402135, the K1 cluster of run 345), and
//! otherwise tex.web §1047 ends it at the `\par` as 65n does. Repros `tools/perfect_kernel/repros/paragraph-mode/*.tex`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// The `Error:` lines of a log, ANSI-free, without their details.
fn error_lines(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|l| l.find("Error:").map(|i| &l[i..]))
    .collect()
}

/// The `tex` attributes of every `<Math>`, in document order.
fn math_tex(xml: &str) -> Vec<&str> {
  xml
    .split("<Math ")
    .skip(1)
    .filter_map(|m| m.split("tex=\"").nth(1).and_then(|t| t.split('"').next()))
    .collect()
}

/// A formula split by a blank line whose continuation holds math-only tokens (`\frac`, `^`, `_`) before its `$` stays
/// one formula, the `\par` reverting as nothing, as Perl's. Base (66t2): 5 errors, the formula ended at the blank line
/// and every later `$` of the paragraph out of step.
#[test]
fn formula_split_by_a_blank_line_continues_with_math() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/formula_split_by_blank_line_continues_with_math.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">The energy is "#,
      r#"<Math mode="inline" tex="E(x)=\frac{1}{2}mv^{2}+V_{0}" text="E@(x) = (1 / 2) * m * v ^ 2 + V _ 0" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.2"><XMApp xml:id="p1.m1.2.1"><XMTok meaning="equals" role="RELOP">=</XMTok>"#,
      r#"<XMApp xml:id="p1.m1.2.1.2"><XMTok font="italic" role="UNKNOWN">E</XMTok>"#,
      r#"<XMDual xml:id="p1.m1.2.1.2.2"><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.2.2.1"/>"#,
      r#"<XMWrap xml:id="p1.m1.2.1.2.2.2"><XMTok role="OPEN" stretchy="false">(</XMTok>"#,
      r#"<XMTok font="italic" role="UNKNOWN" xml:id="p1.m1.1">x</XMTok><XMTok role="CLOSE" stretchy="false">)"#,
      r#"</XMTok></XMWrap></XMDual></XMApp><XMApp xml:id="p1.m1.2.1.3"><XMTok meaning="plus" role="ADDOP">+"#,
      r#"</XMTok><XMApp xml:id="p1.m1.2.1.3.2"><XMTok meaning="times" role="MULOP">⁢</XMTok>"#,
      r#"<XMApp xml:id="p1.m1.2.1.3.2.2"><XMTok mathstyle="text" meaning="divide" role="FRACOP"/>"#,
      r#"<XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp>"#,
      r#"<XMTok font="italic" role="UNKNOWN">m</XMTok><XMApp xml:id="p1.m1.2.1.3.2.4">"#,
      r#"<XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" role="UNKNOWN">v</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="2" role="NUMBER">2</XMTok></XMApp></XMApp>"#,
      r#"<XMApp xml:id="p1.m1.2.1.3.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/>"#,
      r#"<XMTok font="italic" role="UNKNOWN">V</XMTok><XMTok fontsize="70%" meaning="0" role="NUMBER">0"#,
      r#"</XMTok></XMApp></XMApp></XMApp></XMath></Math>, where "#,
      r#"<Math mode="inline" tex="m" text="m" xml:id="p1.m2"><XMath xml:id="p1.m2.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">m</XMTok></XMath></Math> is the mass and "#,
      r#"<Math mode="inline" tex="v" text="v" xml:id="p1.m3"><XMath xml:id="p1.m3.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">v</XMTok></XMath></Math> the speed.</p>"#,
    ),
  );
}

/// hep-th9402135's braced continuation `{\cf^{1}/G}$`: the look-ahead goes into a brace group that is no command's
/// argument, and its `^` keeps the formula open. Base: 3 errors, "is smooth, and" a formula.
#[test]
fn formula_split_by_a_blank_line_continues_in_a_group() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/formula_split_by_blank_line_braced_continuation.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">The quotient map "#,
      r#"<Math mode="inline" tex="p:{\cal F}^{1}\rightarrow{{\cal F}^{1}/G}" text="p colon ℱ ^ 1 rightarrow ℱ ^ 1 / G" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok name="colon" role="METARELOP">:</XMTok>"#,
      r#"<XMTok font="italic" role="UNKNOWN">p</XMTok><XMApp xml:id="p1.m1.1.1.3">"#,
      r#"<XMTok name="rightarrow" role="ARROW">→</XMTok><XMApp xml:id="p1.m1.1.1.3.2">"#,
      r#"<XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok role="UNKNOWN">ℱ</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp><XMApp xml:id="p1.m1.1.1.3.3">"#,
      r#"<XMTok meaning="divide" role="MULOP">/</XMTok><XMApp xml:id="p1.m1.1.1.3.3.2">"#,
      r#"<XMTok role="SUPERSCRIPTOP" scriptpos="post2"/><XMTok role="UNKNOWN">ℱ</XMTok>"#,
      r#"<XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok></XMApp>"#,
      r#"<XMTok font="italic" role="UNKNOWN">G</XMTok></XMApp></XMApp></XMApp></XMath></Math> is smooth, and "#,
      r#"<Math mode="inline" tex="y" text="y" xml:id="p1.m2"><XMath xml:id="p1.m2.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math> is a point.</p>"#,
    ),
  );
}

/// The K1 repro (hep-th9402135:175-177): `$Z(\epsilon) =` <blank line> `\int_{A} … D A$, where $F_{A}$ …` is one
/// formula and every later `$…$` of the paragraph a formula of its own, 0 errors (Perl's count; pdflatex's §1046
/// re-opening gives 2). Base: 9 errors, the scripts after the blank line text and `$` parity flipped.
#[test]
fn split_formula_keeps_the_paragraphs_dollar_parity() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/script_in_text_after_par_closed_formula_reenters_math.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tex(&xml),
    vec![
      r"Z(\epsilon)=\int_{A}\exp(-F_{A})DA",
      "F_{A}",
      "A_{1}",
      "B_{2}",
      "C_{3}",
      "D_{4}"
    ],
    "{xml}"
  );
  assert!(xml.contains("</Math>, where <Math"), "{xml}");
  assert!(xml.contains("</Math> is the curvature of <Math"), "{xml}");
}

/// The 65n case kept: no math-only token before the next `$`, which opens the next paragraph's own formula, so the
/// `\par` ends this one ("Missing $ inserted", pdflatex's count), and the next paragraph's text and formulas are its
/// own. Perl: 4 errors, the formula ran on to that `$` and the paragraph's `$` parity flipped.
#[test]
fn par_before_a_text_paragraph_still_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_a_text_paragraph_ends_the_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["xafterone.", "y_{1}", "z"], "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    concat!(
      r#"<p xml:id="p2.1">Second para with <Math mode="inline" tex="y_{1}" text="y _ 1" xml:id="p2.m1">"#,
      r#"<XMath xml:id="p2.m1.1"><XMApp xml:id="p2.m1.1.1"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/>"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1"#,
      r#"</XMTok></XMApp></XMath></Math> here and <Math mode="inline" tex="z" text="z" xml:id="p2.m2">"#,
      r#"<XMath xml:id="p2.m2.1"><XMTok font="italic" role="UNKNOWN">z</XMTok></XMath></Math> there.</p>"#,
    ),
  );
}

/// A brace group right after a control sequence is its argument, not looked into: the `_` of `\ref{eq_a}` (a label
/// key, never typeset) is no math following the blank line, and the `\par` ends the formula as 65n.
#[test]
fn par_before_a_label_argument_still_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_a_label_argument_ends_the_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["a=b", "xafterone.", "y"], "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    concat!(
      r#"<p xml:id="p2.1">As in <ref labelref="LABEL:eq_a"/> we have "#,
      r#"<Math mode="inline" tex="y" text="y" xml:id="p2.m1"><XMath xml:id="p2.m1.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math> here.</p>"#,
    ),
  );
}

/// The first paragraph of the 66w r1 controls: `We have $a = b` with its formula ended at the blank line.
const FIRST_PARAGRAPH: &str = concat!(
  r#"<p xml:id="p1.1">We have <Math mode="inline" tex="a=b" text="a = b" xml:id="p1.m1">"#,
  r#"<XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok meaning="equals" role="RELOP">=</XMTok>"#,
  r#"<XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath>"#,
  r#"</Math></p>"#,
);

/// The formula `x` that closes the second paragraph of the 66w r1 controls, ` and $x$ here.`
const X_HERE: &str = concat!(
  r#" and <Math mode="inline" tex="x" text="x" xml:id="p2.m1"><XMath xml:id="p2.m1.1">"#,
  r#"<XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> here.</p>"#,
);

/// A 66w r1 control (review finding 1): `We have $a = b` ⏎⏎ `<command with arguments> and $x$ here.` — the
/// command's arguments are not looked into (a `*`, then every `[…]` and `{…}` in a row), or the command is one the
/// look-ahead cannot follow, so no math-only token precedes the `$` and the `\par` ends the formula as 66v1 and
/// pdflatex do: one "Missing $ inserted", the first paragraph and the second (`second`, up to ` and $x$ here.`)
/// whole, no warning. Returns the XML.
fn assert_par_ends_formula(source: &str, second: &str) -> String {
  let (log, xml) = convert_with(source, Some("ar5iv.sty"));
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["a=b", "x"], "{xml}");
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], FIRST_PARAGRAPH);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    &format!("{second}{X_HERE}"),
  );
  xml
}

/// Whether some `<Math>` of `xml` holds `needle`.
fn some_formula_holds(xml: &str, needle: &str) -> bool {
  xml.split("<Math ").skip(1).any(|m| {
    m.split("</Math>")
      .next()
      .is_some_and(|m| m.contains(needle))
  })
}

/// `\cite[p.~2]{smith_2001}`: the optional argument after the command, then the key, are its arguments. 66w read the
/// key's `_` as math, and the paragraph's next formula ran through `\begin{thebibliography}` — `<section>` and
/// `<bibliography>` inside `<XMath>` (Perl's failure, KNOWN_PERL_ERRORS #679).
#[test]
fn par_before_a_cite_with_a_note_ends_the_formula() {
  let xml = assert_par_ends_formula(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_a_cite_with_a_note_ends_the_formula.tex"
    ),
    concat!(
      r#"<p xml:id="p2.1">See <cite class="ltx_citemacro_cite">[<bibref bibrefs="smith_2001" separator="," "#,
      r#"yyseparator=","/>, "#,
      "p.\u{a0}2]</cite>",
    ),
  );
  assert!(xml.contains("<bibliography "), "{xml}");
  assert!(!some_formula_holds(&xml, "<bibliography"), "{xml}");
  assert!(!some_formula_holds(&xml, "<section"), "{xml}");
}

/// `\ref*{eq_1}`: a `*` right after the command keeps the look-ahead at its arguments.
#[test]
fn par_before_a_starred_ref_ends_the_formula() {
  assert_par_ends_formula(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_a_starred_ref_ends_the_formula.tex"
    ),
    r#"<p xml:id="p2.1">See <ref class="ltx_nolink" labelref="LABEL:eq_1"/>"#,
  );
}

/// `\verb|make_all|`: `\verb`'s expansion is code reading a delimited argument, which the look-ahead cannot follow.
#[test]
fn par_before_verb_ends_the_formula() {
  assert_par_ends_formula(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_verb_ends_the_formula.tex"
    ),
    r#"<p xml:id="p2.1">Run <verbatim font="typewriter" xml:id="p2.1.1">make_all</verbatim>"#,
  );
}

/// `\includegraphics[width=1cm]{fig_1}`: the key-value argument and the file name are the command's arguments.
#[test]
fn par_before_includegraphics_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_includegraphics_ends_the_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["a=b", "x"], "{xml}");
  assert_element(&xml, "p", &[r#"xml:id="p1.1""#], FIRST_PARAGRAPH);
  assert!(!some_formula_holds(&xml, "<graphics"), "{xml}");
  assert!(
    xml.contains(concat!(
      r#"<p xml:id="p2.1">See <graphics graphic="fig_1" options="width=28.45274pt,keepaspectratio=true" "#,
      r#"xml:id="p2.g1"/> and <Math mode="inline" tex="x""#
    )),
    "{xml}"
  );
}

/// TeX's alphabetic constant: the `_` after `\char`` is a character code, not a subscript, so it continues no formula
/// (review of 66w r1, finding 1).
#[test]
fn par_before_a_character_code_ends_the_formula() {
  let tex = "\\documentclass{article}\n\\begin{document}\nWe have $a = b\n\nSee \\char`_ and $x$ here.\n\\end{document}\n";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["a=b", "x"], "{xml}");
}

/// A quoted formula `` `$x_1$' ``: the `$` after the backquote is a math shift, not a character code, so it ends the
/// look-ahead and the formula before the `\par` (review of 66w r2, finding 1).
#[test]
fn par_before_a_backquoted_formula_ends_the_formula() {
  let tex = "\\documentclass{article}\n\\begin{document}\nWe have $a = b\n\nSee `$x_1$' here and $y$ there.\n\\end{document}\n";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(math_tex(&xml), vec!["a=b", "x_{1}", "y"], "{xml}");
}

/// A quoted reference `` `\ref{fig_1}' ``: the command after the backquote keeps its arguments, so the `_` of the
/// label continues no formula (review of 66w r2, finding 1).
#[test]
fn par_before_a_backquoted_ref_ends_the_formula() {
  let tex = "\\documentclass{article}\n\\begin{document}\nWe have $a = b\n\nSee `\\ref{fig_1}' and $x$ here.\n\\end{document}\n";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(math_tex(&xml), vec!["a=b", "x"], "{xml}");
  assert_eq!(warning_count(&log), 0, "{log}");
}

/// A `\catcode` change: TeX reads the `a_b` after it with `_` as other, so the look-ahead's tokens past it decide
/// nothing and the formula ends at the `\par` (review of 66w r2, probe bq2).
#[test]
fn par_before_a_catcode_change_ends_the_formula() {
  let tex = "\\documentclass{article}\n\\begin{document}\nWe have $a = b\n\nSee \\catcode`\\_=12 a_b and $x$ here.\n\
             \\end{document}\n";
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(
    error_lines(&log),
    vec!["Error:expected:$ Missing $ inserted"],
    "{log}"
  );
  assert_eq!(math_tex(&xml), vec!["a=b", "x"], "{xml}");
}

/// natbib's `\citep[e.g.][]{a_b}`: every `[…]` and `{…}` in a row after the command is its argument.
#[test]
fn par_before_citep_with_two_notes_ends_the_formula() {
  let xml = assert_par_ends_formula(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_citep_with_two_notes_ends_the_formula.tex"
    ),
    concat!(
      r#"<p xml:id="p2.1">See <cite class="ltx_citemacro_citep">(<bibref bibrefs="a_b" separator=";" "#,
      r#"show="AuthorsPhrase1Year" yyseparator=","><bibrefphrase>, </bibrefphrase></bibref>, e.g.)</cite>"#,
    ),
  );
  assert!(!some_formula_holds(&xml, "<bibliography"), "{xml}");
}

/// hyperref's `\hyperref[sec_a]{there}`: the optional label and the link text are the command's arguments.
#[test]
fn par_before_hyperref_ends_the_formula() {
  assert_par_ends_formula(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_hyperref_ends_the_formula.tex"
    ),
    r#"<p xml:id="p2.1">See <ref labelref="LABEL:sec_a">there</ref>"#,
  );
}

/// Commands the look-ahead cannot follow or that end a paragraph, by meaning: `\lstinline|make_all|` (its body hands
/// the input to code reading a verbatim argument), url.sty's `\url|a_b|` (the same) and `\medskip` (its body's
/// `\vskip`). Each formula ends at its `\par`; the `_` of `x_1` after `\medskip` is text's error, as in 66v1.
#[test]
fn par_before_unfollowable_commands_ends_the_formula() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/par_in_inline_math_before_unfollowable_commands_ends_the_formula.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(
    error_lines(&log),
    vec![
      "Error:expected:$ Missing $ inserted",
      "Error:expected:$ Missing $ inserted",
      "Error:expected:$ Missing $ inserted",
      "Error:unexpected:_ Script _ can only appear in math mode",
    ],
    "{log}"
  );
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    math_tex(&xml),
    vec!["a=b", "x", "c=d", "y", "g=h", "w"],
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    concat!(
      r#"<p xml:id="p2.1">Run <text class="ltx_lst_identifier ltx_lstlisting" xml:id="p2.1.1">make_all</text>"#,
      r#" and <Math mode="inline" tex="x" text="x" xml:id="p2.m1"><XMath xml:id="p2.m1.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> here.</p>"#,
    ),
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p4.1""#],
    concat!(
      r#"<p xml:id="p4.1">Get <ref class="ltx_nolink ltx_url" font="typewriter" href="a_b">a_b</ref>"#,
      r#" and <Math mode="inline" tex="y" text="y" xml:id="p4.m1"><XMath xml:id="p4.m1.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math> here.</p>"#,
    ),
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p6.1""#],
    concat!(
      r#"<p xml:id="p6.1">Then x_1 and <Math mode="inline" tex="w" text="w" xml:id="p6.m1"><XMath xml:id="p6.m1.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">w</XMTok></XMath></Math> here.</p>"#,
    ),
  );
}

/// Under `\obeylines` a line end in a formula is an active `^^M` let to `\par`: the look-ahead starts at the next line
/// (the current one is read to its end) and skips the leading tokens that mean `\par`, so the math there (`b_1`)
/// keeps the formula open, as for a blank line (hep-ph9809328, hep-ph9706250 with MTeXsis's `\unobeylines` missing).
/// 66v1: 3 errors, the formula ended at the line end and the paragraph's `$` parity flipped.
#[test]
fn formula_split_by_an_obeyed_line_end_continues() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/paragraph-mode/formula_split_by_an_obeyed_line_end_continues.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    concat!(
      r#"<p xml:id="p1.1">The pair <Math mode="inline" tex="(a+b_{1})" text="a + b _ 1" xml:id="p1.m1">"#,
      r#"<XMath xml:id="p1.m1.2"><XMDual xml:id="p1.m1.2.1"><XMRef idref="p1.m1.1" xml:id="p1.m1.2.1.1"/>"#,
      r#"<XMWrap xml:id="p1.m1.2.1.2"><XMTok role="OPEN" stretchy="false">(</XMTok><XMApp xml:id="p1.m1.1">"#,
      r#"<XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok>"#,
      r#"<XMApp xml:id="p1.m1.1.3"><XMTok role="SUBSCRIPTOP" scriptpos="post1"/>"#,
      r#"<XMTok font="italic" role="UNKNOWN">b</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok>"#,
      r#"</XMApp></XMApp><XMTok role="CLOSE" stretchy="false">)</XMTok></XMWrap></XMDual></XMath></Math> and "#,
      r#"<Math mode="inline" tex="c" text="c" xml:id="p1.m2"><XMath xml:id="p1.m2.1">"#,
      r#"<XMTok font="italic" role="UNKNOWN">c</XMTok></XMath></Math> here.</p>"#,
    ),
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p2.1""#],
    r#"<p xml:id="p2.1">Next line.</p>"#,
  );
}
