//! Red/green guards for batch 77u: letters of font encodings without a map (T4's African letters, X2's and OT2's
//! Cyrillic), the `\p@<counter>` reference form applied to the expanded number as `\refstepcounter` stores it (and
//! enumitem's reference forms), and the font reset at `\begin{document}`. Each guard asserts the diagnostics by count
//! and the authored content in its element.
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with},
};

/// t4enc.def's composites decode through the T4 map (t4_fontmap.rs, fccoding.mf's glyphs, fc.rme Appendix D): `\m{y}`
/// (t4enc.def:164, slot 175) is ƴ. Without the map the composite was never declared and `\m` reached its own
/// `\errmessage`, the letter lost (2405.01972: 44 Errors; pdflatex 0). Repro
/// fonts-nfss/t4_modified_letter_help_is_not_expanded.
#[test]
fn t4_modified_letter_prints() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t4_modified_letter_help_is_not_expanded.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>Text \u{01B4} here.</p></para>",
  )]);
}

/// The T4 letters of the upper half, its multi-character slots (E̱, Ɛ̃, fj) and its combining accents (`\I` the
/// vertical line above, `\G` the double grave: fc.rme Appendix D). Witness 2405.01972 (Pular `\m{b}`, `\m{d}`,
/// `\m{y}`). Repro fonts-nfss/t4_african_letters_print.
#[test]
fn t4_african_letters_print() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t4_african_letters_print.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>\u{0181}\u{0253} \u{018A}\u{0257} \u{0190}\u{025B} \u{018E}\u{01DD} \u{0186}\u{0254} \
     \u{014A}\u{014B} \u{01B4}\u{01B3} \u{0283} \u{0292}\n\u{0256}\u{0189} \u{0190}\u{0303} E\u{0331} \u{028B} \
     \u{02A7} fj a\u{030D} a\u{030F}</p></para>",
  )]);
}

/// X2 and OT2 have maps of their own (x2_fontmap.rs, ot2_fontmap.rs): the symbols 77s found dropped with no diagnostic
/// print, "A јіА B јЈ C." (pdflatex; pdftotext reads OT2's Ј ј as J j). Repro fonts-nfss/x2_ot2_cyrillic_symbols_print.
#[test]
fn x2_ot2_cyrillic_symbols_print() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/x2_ot2_cyrillic_symbols_print.tex"
  );
  assert_elements(tex, "ar5iv.sty", (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">A \u{0458}\u{0456}\u{0410} B \u{0458}\u{0408} C.</p></para>",
  )]);
}

/// X2's Cyrillic letters in the Latin slots and its T2A upper half, OT2's wncyr transliteration (`Moskva` is Москва);
/// `\U{i}` composes in OT2 although X2 declared `\U` first. Repro fonts-nfss/x2_ot2_cyrillic_letters_and_composites.
#[test]
fn x2_ot2_cyrillic_letters_and_composites() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/x2_ot2_cyrillic_letters_and_composites.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>X2: \u{0458} \u{0408} \u{0452} \u{051B} \u{051D} \u{0462} \u{04A9} \u{0448}\u{0430} \
     \u{0456}.\nOT2: \u{041C}\u{043E}\u{0441}\u{043A}\u{0432}\u{0430} \u{0416}\u{0436} \u{0458} \u{0463} \u{044A} \
     \u{0451} \u{0439}.</p></para>",
  )]);
}

/// OT2's transliteration digraphs are the wncyr fonts' ligatures (wncyr10.tfm's LIGTABLE): `sh` ш, `shch` щ through
/// the digit 7, `kh` х, `ya` я, `e1` э, `D1` Ђ, `N0` №, the quotes and guillemets; read letter by letter `sh` was "сх"
/// (1405.1988, 2510.25931). Repro fonts-nfss/ot2_transliteration_ligatures.
#[test]
fn ot2_transliteration_ligatures() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/ot2_transliteration_ligatures.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>\u{041C}\u{043E}\u{0441}\u{043A}\u{0432}\u{0430}, \u{0449}\u{0438}, \
     \u{0425}\u{0440}\u{0443}\u{0449}\u{0435}\u{0432}, \u{0429}\u{0423}\u{041A}\u{0410}, \
     \u{0436}\u{0443}\u{0440}\u{043D}\u{0430}\u{043B}, \u{044F}\u{0437}\u{044B}\u{043A}, \
     \u{042E}\u{0440}\u{0438}\u{0438}, \u{044D}\u{0442}\u{043E}, \u{0427}\u{0435}\u{0445}\u{043E}\u{0432}, \
     \u{0402}\u{043E}\u{0440}\u{0452}\u{0435},\n\u{0446}\u{0430}\u{0440}\u{2019}, \
     \u{201C}\u{043A}\u{0430}\u{0432}\u{044B}\u{0447}\u{043A}\u{0438}\u{201D} \
     \u{00AB}\u{0435}\u{0442}\u{043E}\u{00BB} \u{2116} 5 \u{2014} \u{0434}\u{0430}.</p></para>",
  )]);
}

/// A `\DeclareTextAccent` of a command another encoding's file declared first (T2A's `\U`, slot 20) still defines this
/// encoding's command (latex.ltx's `\DeclareTextCommand` always does): OT2's `\U{i}` is its composite й (ot2enc.def:47,
/// :156), not и.
#[test]
fn second_encoding_accent_composes() {
  assert_elements(
    r"\documentclass{article}
\usepackage[OT2,T2A,T1]{fontenc}
\begin{document}
{\fontencoding{OT2}\selectfont \U{i}\U{I}}
\end{document}",
    RAW,
    (0, 0),
    &[(
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>\u{0439}\u{0419}</p></para>",
    )],
  );
}

/// A text symbol of an encoding with no map at all prints nothing, as before, and says so once, naming the encoding
/// (tipa's T3; 2405.01972 `\textipa{\textltailn}`). Repro fonts-nfss/text_symbol_of_an_unmapped_encoding_warns.
#[test]
fn text_symbol_of_an_unmapped_encoding_warns() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/text_symbol_of_an_unmapped_encoding_warns.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (0, 1),
    &["Font encoding T3 has no font map: the text symbol \\textschwa has no glyph and is dropped"],
    &[(
      "para",
      "p1",
      r#"<para xml:id="p1"><p>Schwa  and .</p></para>"#,
    )],
  );
}

/// The reference form is latex.ltx's `\@currentlabel`: `\protected@edef` of `\p@equation\theequation`, so the robust
/// nine-argument `\p@equation` stays and keeps the ninth token of "Equation S1": refnum "S1", as pdflatex's `\ref`.
/// Typed unexpanded it read past the tag (1902.09345: 18 errors, the first SI equation lost). Repro
/// parameter-conditional/p_form_applies_to_the_expanded_number.
#[test]
fn p_form_applies_to_the_expanded_number() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/parameter-conditional/p_form_applies_to_the_expanded_number.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "equation",
      "S0.E1",
      r#"<equation labels="LABEL:a" xml:id="S0.E1"><tags><tag>(Equation S1)</tag><tag role="refnum">S1</tag></tags><Math mode="display" tex="x" text="x" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation>"#,
    ),
    (
      "equation",
      "S0.E2",
      r#"<equation xml:id="S0.E2"><tags><tag>(Equation S2)</tag><tag role="refnum">S2</tag></tags><Math mode="display" tex="y" text="y" xml:id="S0.E2.m1"><XMath><XMTok font="italic" role="UNKNOWN">y</XMTok></XMath></Math></equation>"#,
    ),
  ]);
}

/// The kernel's argument-taking and prefix forms are unchanged by the expansion: `\p@enumii` = `\theenumi`
/// ("1a"), `\labelformat` (an argument-taking `\p@section`), and an empty `\p@equation`: pdflatex "Sec. 1 1 1a".
#[test]
fn p_forms_of_the_kernel_keep_their_reference() {
  let xml = assert_elements(
    r"\documentclass{article}
\labelformat{section}{Sec.~#1}
\begin{document}
\section{S}\label{s}
\begin{equation}x\label{e}\end{equation}
\begin{enumerate}\item A\begin{enumerate}\item B\label{b}\end{enumerate}\end{enumerate}
\ref{s} \ref{e} \ref{b}
\end{document}",
    RAW,
    (0, 0),
    &[],
  );
  for refnum in ["Sec.\u{00A0}1", "1", "1a"] {
    let tag = format!(r#"<tag role="refnum">{refnum}</tag>"#);
    assert!(xml.contains(&tag), "{tag}: {xml}");
  }
  assert!(
    xml.contains("<tag role=\"typerefnum\">item\u{00A0}1a</tag>"),
    "{xml}"
  );
}

/// enumitem's short label is the list's `label=` (enumitem.sty:660-681 `\enit@first`), its mark replaced
/// (`\SetEnumerateShortLabel`, :633-652): `[(a)]` refers as "(a)", as `[label=(\alph*)]` does (pdflatex "Refs: (a);
/// (a)."; KNOWN_PERL_ERRORS #808). Repro package-bindings/enumitem_shortlabel_ref.
#[test]
fn enumitem_short_label_is_the_reference() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/enumitem_shortlabel_ref.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert_eq!(
    xml.matches(r#"<tag role="refnum">(a)</tag>"#).count(),
    2,
    "{xml}"
  );
}

/// enumitem's `ref=` (and `label=`) below the first level empties `\p@<counter>` (enumitem.sty:551-556): the inner
/// item refers as "(a).i", not "(a)(a).i". Repro list-structure/enumitem_ref_clears_the_parent_prefix.
#[test]
fn enumitem_ref_clears_the_parent_prefix() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/enumitem_ref_clears_the_parent_prefix.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  assert!(xml.contains(r#"<tag role="refnum">(a).i</tag>"#), "{xml}");
}

/// latex.ltx's `\@kernel@after@begindocument@before` runs `\init@series@setup` (:12513-12514, :13902-13920): the
/// preamble's `\itshape\bfseries` does not reach the body, which pdflatex sets upright medium CMR10 (Perl and 77s:
/// `<text font="bold italic">`). Repro fonts-nfss/preamble_font_switch_does_not_reach_the_body.
#[test]
fn preamble_font_switch_does_not_reach_the_body() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/preamble_font_switch_does_not_reach_the_body.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r#"<para xml:id="p1"><p>Body text. <text font="italic">it</text></p></para>"#,
  )]);
}

/// The same for a class body's `\usefont{T1}{cmss}{bx}{sl}`: pdflatex's body is CMR10, "it" CMTI10. Repro
/// fonts-nfss/class_body_font_does_not_reach_the_body.
#[test]
fn class_body_font_does_not_reach_the_body() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/class_body_font_does_not_reach_the_body.tex"
  );
  let start = tex
    .find("\\begin{filecontents*}[overwrite]{lxfontbody.cls}\n")
    .expect("the class");
  let end_marker = "\\end{filecontents*}\n";
  let end = tex.find(end_marker).expect("its end");
  let class = &tex[start + "\\begin{filecontents*}[overwrite]{lxfontbody.cls}\n".len()..end];
  let document = &tex[end + end_marker.len()..];
  let (stderr, xml) = convert_files_with(document, &[("lxfontbody.cls", class)], Some(RAW));
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Body text. <text font="italic">it</text></p></para>"#,
  );
}
