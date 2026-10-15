//! Red/green guards for batch 67r: latex.ltx's `\@fontswitch` (14312-14321) — in maths an old font command
//! (`\DeclareOldFontCommand{\rm}{\normalfont\rmfamily}{\mathrm}`) or `\@fontswitch\relax\mathcal` (`\cal`) switches
//! the math alphabet to the end of the enclosing group, as the native `\rm`/`\bf` switches do; a non-alphabet `#2`
//! reads the `\relax`; an alphabet reached through a parameterless macro or `\protect` is a switch too (round 2).
//! Witnesses: jpsj2 cond-mat0510381, 0711.1066, 1004.2823; the ptptex, paper and artikel1 classes.
use super::perfect_kernel_batch46::{error_count, warning_count};

/// `{\rm LP-HF}` is upright through its end, `\cal H` caligraphic, `{\rm ^{8}Be}` a prescript, `{\rm \left(x\right)}`
/// a fenced x, `{\bf v} w` a bold v only, and `{\my a}` (a user macro as `#3`) reads `\relax`, not the `a`; in text
/// `{\cal x}` is `\relax` with no space of its own. Perl (latex_constructs.pool.ltxml:5200, :5315) and 67o1 read the
/// next token as the alphabet's argument: `\mathrm{L}P-HF`, `\mathcal{}H`, "Missing sub/superscript argument",
/// "Extra \right".
#[test]
fn old_font_command_switches_the_math_alphabet() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/old_font_command_switches_math_alphabet.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let maths = [
    (
      "p1.m1",
      r#"<Math mode="inline" tex="\chi^{\rm LP-HF}" text="chi ^ (LP - HF)" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="italic" name="chi" role="UNKNOWN">χ</XMTok><XMApp xml:id="p1.m1.1.1.3"><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" role="UNKNOWN">LP</XMTok><XMTok fontsize="70%" role="UNKNOWN">HF</XMTok></XMApp></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m2",
      r#"<Math mode="inline" tex="\mathcal H" text="H" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMTok font="caligraphic" role="UNKNOWN">H</XMTok></XMath></Math>"#,
    ),
    (
      "p1.m3",
      r#"<Math mode="inline" tex="{\rm{}^{8}Be}" text="^ 8 Be" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMApp xml:id="p1.m3.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="pre2"/><XMTok role="UNKNOWN">Be</XMTok><XMTok fontsize="70%" meaning="8" role="NUMBER">8</XMTok></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m4",
      r#"<Math mode="inline" tex="{\rm\left(x\right)}" text="x" xml:id="p1.m4"><XMath xml:id="p1.m4.2"><XMDual xml:id="p1.m4.2.1"><XMRef idref="p1.m4.1" xml:id="p1.m4.2.1.1"/><XMWrap xml:id="p1.m4.2.1.2"><XMTok role="OPEN" stretchy="true">(</XMTok><XMTok role="UNKNOWN" xml:id="p1.m4.1">x</XMTok><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math>"#,
    ),
    (
      "p1.m5",
      r#"<Math mode="inline" tex="{\bf v}w" text="v * w" xml:id="p1.m5"><XMath xml:id="p1.m5.1"><XMApp xml:id="p1.m5.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="bold" role="UNKNOWN">v</XMTok><XMTok font="italic" role="UNKNOWN">w</XMTok></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m6",
      r#"<Math mode="inline" tex="{[]a}" text="list@() * a" xml:id="p1.m6"><XMath xml:id="p1.m6.1"><XMApp xml:id="p1.m6.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMDual xml:id="p1.m6.1.1.2"><XMApp xml:id="p1.m6.1.1.2.1"><XMTok meaning="list"/></XMApp><XMWrap xml:id="p1.m6.1.1.2.2"><XMTok role="OPEN" stretchy="false">[</XMTok><XMTok role="CLOSE" stretchy="false">]</XMTok></XMWrap></XMDual><XMTok font="italic" role="UNKNOWN">a</XMTok></XMApp></XMath></Math>"#,
    ),
  ];
  for (id, math) in maths {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
  // The whole paragraph: the six formulas, then `{\rm text}` plain, `{\bf bold}` bold and `{\cal x}` a bare x with no
  // space of its own after the `\relax`.
  let formulas: Vec<&str> = maths.iter().map(|(_, math)| *math).collect();
  latexml::util::test::assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    &format!(
      r#"<p xml:id="p1.1">{}and text <text font="bold" xml:id="p1.1.1">bold</text> x</p>"#,
      formulas.join(" ")
    ),
  );
}

/// An alphabet reached through a macro is a switch too, as latex.ltx's `\math@bgroup` let to `\relax` makes any
/// alphabet `#2` expands to: `\newcommand\Bbbx{\mathbb}`, a robust `\DeclareRobustCommand\Bbby{\mathbb}` and a `\let`
/// copy all give a blackboard R and Z and an italic c (pdflatex `\showbox`: `\U/msb/m/n/10` R and Z, `\OML/cmm/m/it/10`
/// c). Round 1 of 67r gave `tex="{\mathbb{}RZ}"` and both letters italic; 67o1 a blackboard R only.
#[test]
fn old_font_command_alphabet_through_a_macro() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/fonts-nfss/old_font_command_alphabet_through_a_macro.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (n, cmd) in [(1, "bbx"), (2, "bby"), (3, "bbz")] {
    latexml::util::test::assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="p1.m{n}""#)],
      &format!(
        r#"<Math mode="inline" tex="{{\{cmd} RZ}}c" text="R * Z * c" xml:id="p1.m{n}"><XMath xml:id="p1.m{n}.1"><XMApp xml:id="p1.m{n}.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="blackboard" role="UNKNOWN">R</XMTok><XMTok font="blackboard" role="UNKNOWN">Z</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"#
      ),
    );
  }
}

/// CONTROL: an alphabet from a package binding (`\mathbb`) and a `\DeclareMathAlphabet`'d one (a font-carrying
/// primitive) switch to the group's end too; an `\edef` in maths keeps the old font command finite (its reversion name
/// is a character string, never the command itself), and `\mathrm{a{\rm b}c}` stays one upright run.
#[test]
fn old_font_command_switches_package_and_declared_alphabets() {
  let (log, xml) = latexml::util::test::convert_with(
    r"\documentclass{article}
\usepackage{amsfonts}
\DeclareOldFontCommand{\rm}{\normalfont\rmfamily}{\mathrm}
\DeclareOldFontCommand{\bb}{\normalfont}{\mathbb}
\DeclareMathAlphabet{\mathsfx}{OT1}{cmss}{m}{n}
\DeclareOldFontCommand{\sx}{\sffamily}{\mathsfx}
\begin{document}
${\bb R}^n$ ${\sx ab}c$ $\edef\x{\rm a}\x b$ $\mathrm{a{\rm b}c}$
\end{document}
",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, math) in [
    (
      "p1.m1",
      r#"<Math mode="inline" tex="{\bb R}^{n}" text="R ^ n" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMApp xml:id="p1.m1.1.1"><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok font="blackboard" role="UNKNOWN">R</XMTok><XMTok font="italic" fontsize="70%" role="UNKNOWN">n</XMTok></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m2",
      r#"<Math mode="inline" tex="{\sx ab}c" text="ab * c" xml:id="p1.m2"><XMath xml:id="p1.m2.1"><XMApp xml:id="p1.m2.1.1"><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="sansserif" role="UNKNOWN">ab</XMTok><XMTok font="italic" role="UNKNOWN">c</XMTok></XMApp></XMath></Math>"#,
    ),
    (
      "p1.m3",
      r#"<Math mode="inline" tex="\rm ab" text="ab" xml:id="p1.m3"><XMath xml:id="p1.m3.1"><XMTok role="UNKNOWN">ab</XMTok></XMath></Math>"#,
    ),
    (
      "p1.m4",
      r#"<Math mode="inline" tex="\mathrm{a{\rm b}c}" text="abc" xml:id="p1.m4"><XMath xml:id="p1.m4.1"><XMTok role="UNKNOWN">abc</XMTok></XMath></Math>"#,
    ),
  ] {
    latexml::util::test::assert_element(&xml, "Math", &[&format!(r#"xml:id="{id}""#)], math);
  }
}
