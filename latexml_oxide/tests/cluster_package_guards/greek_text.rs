//! Greek text: the font-encoding machinery greek-fontenc, textalpha and
//! babel-greek build on. LGR slots decode to the glyphs of the CB fonts; a
//! kernel accent takes the encoding's declared composite for a key it cannot
//! precompose (`\~>` → psili + perispomeni) and composes with the letter as the
//! font prints it (LGR `\"i` → ϊ); a raw definitions file `\input` twice is read
//! twice (tuenc-greek.def re-reads greek-fontenc.def for TU after lgrenc.def
//! read it for LGR), and a re-declared text command is wrapped for its
//! composites again. Expected output is the intended engine's (lualatex for the
//! TU docs, pdflatex for LGR).
//! Witnesses: greek-fontenc test-tuenc-greek, hyperref-with-greek,
//! char-list-alphabeta, alphabeta-doc, test-lgrenc, char-list; teubner-doc;
//! arXiv 2605.01889.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{
  convert, convert_files, convert_with, error_count, warning_count,
};

const LUATEX: &str = "[rawstyles,rawclasses,luatex]latexml.sty";

const LGR_SLOTS: &str =
  include_str!("../../../tools/perfect_kernel/repros/unicode-catcodes/lgr_slot_glyphs_cbfonts.tex");

const TILDE_GREATER: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/text_composite_tilde_greater_psili.tex"
);

const LGR_ACCENT: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/lgr_accent_on_decoded_letter.tex"
);

const TUENC_GREEK_REREAD: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/input_rereads_raw_def_tuenc_greek.tex"
);

const UNICODE_DEFAULT: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/unicode_composite_default_is_the_accent.tex"
);

const REWRAP: &str = include_str!(
  "../../../tools/perfect_kernel/repros/unicode-catcodes/text_composite_rewrap_after_redeclare.tex"
);

/// LGR slots decode to the CB fonts' glyphs (CB.enc, lgrenc.def): `U` Υ, `j`
/// θ, `k` κ, `\accdialytika U` Ϋ (slot 223), `'w` ώ; the omicron and upsilon
/// accent ligatures (`'o`, `>'u`, `<o`) compose; a lone `>` is the psili ᾿;
/// ϛ ϟ Ϙ Ϛ ə. pdflatex prints the same (teubner-doc's ΑΫΛΟΣ came out ΑϔΛΟΣ).
#[test]
fn lgr_slots_decode_to_the_font_glyphs() {
  let (stderr, xml) = convert(LGR_SLOTS, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    concat!(
      "<p>[\u{3a5}] [\u{3b8}\u{3ba}] [\u{1f7d}] [\u{3ab}] [\u{3bb}\u{1f79}\u{3b3}\u{3bf}\u{3c5}]",
      " [\u{1f54}] [\u{1f41}] [\u{1fbf}\u{395}\u{3bd}]\n[\u{3db} \u{3df} \u{3d8} \u{3da} \u{259}]</p>"
    ),
  );
}

/// greek-fontenc.def:221/226 `\DeclareTextCompositeCommand{\~}{…}{>}
/// {\accpsiliperispomeni}`: `\~>{\alpha}` is alpha with psili and perispomeni,
/// not a tilde on `>` — GREEN modulo NFC: tuenc-greek.def's
/// `\accpsiliperispomeni` appends `\char"0313\char"0342`, so the text is α
/// U+0313 U+0342 where lualatex's PDF extracts the precomposed ἆ.
/// Control: Latin accents, where the letter has a precomposed form, and a
/// punctuation key with no declared composite are unchanged under OT1 and T1.
#[test]
fn declared_composite_on_a_punctuation_key() {
  let (stderr, xml) = convert_with(TILDE_GREATER, Some(LUATEX));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p>[\u{3b1}\u{313}\u{342}] [\u{3b1}\u{314}\u{342}]</p>",
  );

  // OT1 prints `>` as ¿ (slot 62), T1 as `>`.
  for (fontenc, greater) in [("", "\u{bf}"), (r"\usepackage[T1]{fontenc}", "&gt;")] {
    let (stderr, xml) = convert(
      &format!(
        r"\documentclass{{article}}
{fontenc}
\begin{{document}}
[\~n] [\'e] [\~>] [\'\i] [\c{{c}}] [\v{{s}}] [\~{{}}]
\end{{document}}
"
      ),
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{fontenc}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{fontenc}: {stderr}");
    assert_element(
      &xml,
      "p",
      &[],
      &format!("<p>[\u{f1}] [\u{e9}] [{greater}\u{303}] [\u{ed}] [\u{e7}] [\u{161}] [\u{2dc}]</p>"),
    );
  }
}

/// A kernel accent on an LGR letter combines with the letter the font prints
/// (`i` is ι): `\"i` ϊ, `\'a` ά, `` \`e `` ὲ, `\"U` Ϋ, as pdflatex and Perl
/// print them — not the Latin ï á è Ü of composing the raw letter. `\~a` stays
/// α + U+0303 as in Perl (pdflatex's LGR `\~` is the perispomeni, ᾶ).
#[test]
fn lgr_accent_combines_with_the_decoded_letter() {
  let (stderr, xml) = convert(LGR_ACCENT, true);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p>[\u{3ca}] [\u{3ac}] [\u{1f72}] [\u{3ab}] [\u{3b1}\u{303}]</p>",
  );
}

/// Two packages `\input` the same raw `.def`: TeX reads it twice (Perl skips
/// the second read, Package.pm:2363). Under `[LGR,TU]{fontenc}` the second read
/// of greek-fontenc.def declares `\TU\greekscript` and `\TU\LGR@hiatus`, so
/// babel-greek keeps TU (`\MakeUppercase{\'u}` is Ú, not LGR's ϒ) and the
/// uppercase hiatus places its dialytika (ΑΫΛΟΣ). A binding stays once-only:
/// `\input{amsmath.sty}` after `\usepackage{amsmath}` loads nothing again.
#[test]
fn input_rereads_a_raw_definitions_file() {
  let (stderr, xml) = convert_files(
    r"\documentclass{article}
\makeatletter\count@=0 \gdef\seenlist{}\makeatother
\usepackage{amsmath}
\usepackage{w11pa,w11pb}
\begin{document}
\seenlist
\end{document}
",
    &[
      (
        "w11reread.def",
        "\\global\\advance\\count@ 1 \\xdef\\seenlist{\\seenlist[\\the\\count@]}\n",
      ),
      (
        "w11pa.sty",
        "\\ProvidesPackage{w11pa}\\input{w11reread.def}\n",
      ),
      (
        "w11pb.sty",
        "\\ProvidesPackage{w11pb}\\input{w11reread.def}\\input{amsmath.sty}\n",
      ),
    ],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[1][2]</p>");

  // The re-read is `\input`'s file lookup, not a binding's: from
  // `\usepackage{sub/w11d}` it reads `w11d.def`, not `sub/w11d.def` (the
  // `\@currname` directory resolution of `noltxml`); pdflatex `[top][top]`.
  let (stderr, xml) = convert_files(
    r"\documentclass{article}
\gdef\seen{}
\usepackage{sub/w11d}
\begin{document}
\seen
\end{document}
",
    &[
      (
        "sub/w11d.sty",
        "\\ProvidesPackage{sub/w11d}\\input{w11d.def}\\input{w11d.def}\n",
      ),
      ("w11d.def", "\\xdef\\seen{\\seen[top]}\n"),
      ("sub/w11d.def", "\\xdef\\seen{\\seen[dir]}\n"),
    ],
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[top][top]</p>");

  let (stderr, xml) = convert_with(TUENC_GREEK_REREAD, Some(LUATEX));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(
    &xml,
    "p",
    &[],
    "<p>[\u{da}] [\u{391}\u{3ab}\u{39b}\u{39f}\u{3a3}]</p>",
  );
}

/// A text command re-declared after its composites were declared is wrapped
/// again by the next composite declaration (latex.ltx:9923-9932): the old
/// composite (`x`) and the new one (`y`) both apply, anything else takes the
/// NEW body (pdflatex `[X] [Y] [[z]]`). Under the luatex profile greek.ldf and
/// textalpha both read tuenc-greek.def, so `\TU\LGR@hiatus` is re-declared: the
/// uppercase `\>` + grave drops the grave (lualatex Α), and the grave accent
/// on a grave is its declared composite ‘ (greek-fontenc.def:209-210).
#[test]
fn redeclared_text_command_is_rewrapped() {
  let (stderr, xml) = convert(
    r"\documentclass{article}
\makeatletter
\DeclareTextCommand{\wfoo}{OT1}[1]{(#1)}
\DeclareTextCompositeCommand{\wfoo}{OT1}{x}{X}
\DeclareTextCommand{\wfoo}{OT1}[1]{[#1]}
\DeclareTextCompositeCommand{\wfoo}{OT1}{y}{Y}
\makeatother
\begin{document}
[\wfoo x] [\wfoo y] [\wfoo z]
\end{document}
",
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[X] [Y] [[z]]</p>");

  let (stderr, xml) = convert_with(REWRAP, Some(LUATEX));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[\u{391}] [\u{391}] [\u{2018}]</p>");
}

/// A glyph composite reached from a kernel accent is the character itself,
/// decoded through the encoding, not the composite's key: the math `tex=`
/// keeps valid TeX (was `x\text{Mart\LY1\'-\i nez}` under LY1), and a loop
/// that expands the first token until a character comes back (bibleref-parse
/// `\brp@@expandcs`) ends on `\"\i` with ï. T1's composites are the format's dumped
/// t1enc.def keys, found under the kernel's key spelling.
#[test]
fn glyph_composite_is_a_character() {
  let (stderr, xml) = convert(
    r#"\documentclass{article}
\usepackage{amsmath}
\usepackage[LY1,T1]{fontenc}
\begin{document}
$x\text{Mart\'{\i}nez}$ {\fontencoding{LY1}\selectfont $x\text{Mart\'{\i}nez}$}

\def\firstcs#1#2\end{\ifcat\noexpand#1\relax\expandafter\firstcs#1#2\end\else[#1#2]\fi}\firstcs\"\i\end
\end{document}
"#,
    true,
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  for id in ["p1.m1", "p1.m2"] {
    assert_element(
      &xml,
      "Math",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        r#"<Math mode="inline" tex="x\text{{Martínez}}" text="x * [Martínez]" xml:id="{id}">
        <XMath>
          <XMApp>
            <XMTok meaning="times" role="MULOP">⁢</XMTok>
            <XMTok font="italic" role="UNKNOWN">x</XMTok>
            <XMText>Martínez</XMText>
          </XMApp>
        </XMath>
      </Math>"#
      ),
    );
  }
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>[ï]</p></para>"#,
  );
}

/// latex.ltx:9938-9947: a composite is looked up from the argument's FIRST
/// token and replaces the whole argument. T1 declares `\"a` and `\'e`
/// (t1enc.def), so `X\"{ab}Y X\'{ei}Y` is `XäY XéY`; OT1 declares neither and
/// accents the whole group, `XäbY XéiY`. pdflatex prints both so.
#[test]
fn composite_drops_the_rest_of_the_argument() {
  for (fontenc, expected) in [
    (r"\usepackage[T1]{fontenc}", "<p>XäY XéY</p>"),
    ("", "<p>XäbY XéiY</p>"),
  ] {
    let (stderr, xml) = convert(
      &format!(
        r#"\documentclass{{article}}
{fontenc}
\begin{{document}}
X\"{{ab}}Y X\'{{ei}}Y
\end{{document}}
"#
      ),
      true,
    );
    assert_eq!(error_count(&stderr), 0, "{fontenc}: {stderr}");
    assert_eq!(warning_count(&stderr), 0, "{fontenc}: {stderr}");
    assert_element(&xml, "p", &[], expected);
  }
}

/// tuenc.def:122-145: `\DeclareUnicodeComposite{\'}{\i}{"00ED}` takes its
/// fallback from the `\TU\'` wrapper through `\extract@default@composite`,
/// which recognises only the kernel's `\@text@composite` head. With a font
/// lacking í (`\iffontchar` false) `\'{\i}` is then the accent on `\i`, ı́, as
/// in lualatex; Rust's former wrapper made the fallback the composite itself
/// (PushbackLimit). Without the font file `\iffontchar` is true: í.
#[test]
fn unicode_composite_default_is_the_accent() {
  let (stderr, xml) = convert_with(UNICODE_DEFAULT, Some(LUATEX));
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let expected = if latexml::util::test::kpse_has("FontAwesome5Free-Solid-900.otf") {
    "<p>[N] [\u{131}\u{301}] [\u{e9}]</p>"
  } else {
    "<p>[Y] [\u{ed}] [\u{e9}]</p>"
  };
  assert_element(&xml, "p", &[], expected);
}

/// latex.ltx:9933-9940 key an empty argument as `-\@empty`: tuenc-greek.def:275
/// `\DeclareUnicodeComposite{\accdialytikaperispomeni}{}{"1FC1}` is the
/// spacing ῁ for `\accdialytikaperispomeni{}`, and tuenc.def:318-319 make
/// `\^{}` and `\~{}` the ASCII ^ and ~ under TU (lualatex prints both so).
#[test]
fn empty_argument_takes_the_empty_composite() {
  let (stderr, xml) = convert_with(
    r"\documentclass{article}
\usepackage{fontspec}
\usepackage{textalpha}
\begin{document}
[\accdialytikaperispomeni{}] [\~{}] [\^{}] [a\~{}b]
\end{document}
",
    Some(LUATEX),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_element(&xml, "p", &[], "<p>[\u{1fc1}] [~] [^] [a~b]</p>");
}
