//! Red/green guards for batch 77u: letters of font encodings without a map (T4's African letters, X2's and OT2's
//! Cyrillic), the `\p@<counter>` reference form applied to the expanded number as `\refstepcounter` stores it (and
//! enumitem's reference forms), and the font reset at `\begin{document}`. Each guard asserts the diagnostics by count
//! and the authored content in its element.
use latexml::util::test::convert_files_with;

use super::{
  perfect_kernel_batch46::{convert_html, error_count, warning_count},
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
/// (a document-declared XX; tipa's T3 was such an encoding until 77x gave it a map: 2405.01972 `\textipa{\textltailn}`,
/// the schwa now prints). Repro fonts-nfss/text_symbol_of_an_unmapped_encoding_warns.
#[test]
fn text_symbol_of_an_unmapped_encoding_warns() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/text_symbol_of_an_unmapped_encoding_warns.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (0, 1),
    &["Font encoding XX has no font map: the text symbol \\textxxa has no glyph and is dropped"],
    &[(
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>Schwa \u{0259} and \u{0259}; slot  and .</p></para>",
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
  assert_elements(
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
    &[(
      "section",
      "S1",
      concat!(
        r#"<section inlist="toc" labels="LABEL:s" xml:id="S1"><tags><tag>1</tag><tag role="refnum">Sec."#,
        "\u{00A0}",
        r#"1</tag><tag role="typerefnum">"#,
        "\u{00A7}Sec.\u{00A0}",
        r#"1</tag></tags><title><tag close=" ">1</tag>S</title><para xml:id="S1.p1"><equation labels="LABEL:e" "#,
        r#"xml:id="S1.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><Math mode="display" tex="x" "#,
        r#"text="x" xml:id="S1.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation>"#,
        r#"<enumerate xml:id="S1.I1"><item xml:id="S1.I1.i1"><tags><tag>1.</tag><tag role="refnum">1</tag>"#,
        r#"<tag role="typerefnum">item"#,
        "\u{00A0}",
        r#"1</tag></tags><para xml:id="S1.I1.i1.p1"><p>A</p><enumerate xml:id="S1.I1.i1.I1"><item labels="LABEL:b" "#,
        r#"xml:id="S1.I1.i1.I1.i1"><tags><tag>(a)</tag><tag role="refnum">1a</tag><tag role="typerefnum">item"#,
        "\u{00A0}",
        r#"1a</tag></tags><para xml:id="S1.I1.i1.I1.i1.p1"><p>B</p></para></item></enumerate></para></item>"#,
        r#"</enumerate><p><ref labelref="LABEL:s"/> <ref labelref="LABEL:e"/> <ref labelref="LABEL:b"/></p></para>"#,
        r#"</section>"#,
      ),
    )],
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
  let item = nbsp(
    r#"<item labels="LABEL:in" xml:id="S0.I1.i1.I1.i1"><tags><tag>i.</tag><tag role="refnum">(a).i</tag><tag role="typerefnum">item~(a).i</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>inner</p></para></item>"#,
  );
  assert_elements(tex, RAW, (0, 0), &[("item", "S0.I1.i1.I1.i1", &item)]);
}

/// A guard's `~` is the no-break space the reference forms carry (`\lx@tag@texttilde`).
fn nbsp(markup: &str) -> String { markup.replace('~', "\u{00A0}") }

/// An `\item[…]` does not `\refstepcounter` (latex.ltx `\@item`, only `\if@noitemarg`), so a `\label` on it takes
/// `\@currentlabel` as the last numbered item left it: the enclosing item's "1" for the first nested `\item[(X)]`.
/// enumitem empties `\p@enumii` (enumitem.sty:551-556), which Perl's `\p@<counter>` form (latex_constructs.pool.ltxml:
/// 1383) relied on: the refnum was lost (77u r2; 2603.02347, 7 items). (Z), after a numbered item of its own nested
/// list whose `\p@enumii` prints nothing, takes that sibling's `\p@enumii\theenumii`, LaTeX's `\@currentlabel` "a"
/// (77u r3 had none; Perl's enumitem binding never empties `\p@`, so Perl and int35a print "1"). pdflatex
/// "Refs: [1] [a] [a]". KNOWN_PERL_ERRORS #1003. Repro
/// list-structure/enumitem_item_with_its_own_label_takes_the_enclosing_label.
#[test]
fn enumitem_item_with_its_own_label_takes_the_enclosing_label() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/enumitem_item_with_its_own_label_takes_the_enclosing_label.tex"
  );
  let x = nbsp(
    r#"<item labels="LABEL:x" xml:id="S0.I1.i1.I1.ix1"><tags><tag>(X)</tag><tag role="refnum">1</tag><tag role="typerefnum">item (X)</tag></tags><para xml:id="S0.I1.i1.I1.ix1.p1"><p>custom</p></para></item>"#,
  );
  let y = nbsp(
    r#"<item labels="LABEL:y" xml:id="S0.I1.i1.I1.i1"><tags><tag>(a)</tag><tag role="refnum">a</tag><tag role="typerefnum">item~a</tag></tags><para xml:id="S0.I1.i1.I1.i1.p1"><p>plain</p></para></item>"#,
  );
  let z = nbsp(
    r#"<item labels="LABEL:z" xml:id="S0.I1.i1.I1.ix2"><tags><tag>(Z)</tag><tag role="refnum">a</tag><tag role="typerefnum">item (Z)</tag></tags><para xml:id="S0.I1.i1.I1.ix2.p1"><p>after</p></para></item>"#,
  );
  assert_elements(tex, RAW, (0, 0), &[
    ("item", "S0.I1.i1.I1.ix1", &x),
    ("item", "S0.I1.i1.I1.i1", &y),
    ("item", "S0.I1.i1.I1.ix2", &z),
  ]);
}

/// The sibling's form reaches every reference form: hyperref's `\autoref` and cleveref's `\cref` of the (Z) item
/// print "item a", as pdflatex (77u r3: "item " dangling and "item (Z)", the item had no refnum; rev77u2 finding 1).
/// pdflatex "REFS [a] [item a] [item a]". Repro list-structure/enumitem_sibling_item_reference_forms.
#[test]
fn enumitem_sibling_item_reference_forms() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/enumitem_sibling_item_reference_forms.tex"
  );
  let z = nbsp(concat!(
    r#"<item labels="LABEL:z" xml:id="S0.I1.i1.I1.ix1"><tags><tag>(Z)</tag><tag role="autoref">item~a</tag>"#,
    r#"<tag role="creftype">item</tag><tag role="creftypecap">Item</tag><tag role="creftypeplural">items</tag>"#,
    r#"<tag role="creftypepluralcap">Items</tag><tag role="refnum">a</tag><tag role="typerefnum">item (Z)</tag>"#,
    r#"</tags><para xml:id="S0.I1.i1.I1.ix1.p1"><p>after</p></para></item>"#,
  ));
  assert_elements(tex, RAW, (0, 0), &[("item", "S0.I1.i1.I1.ix1", &z)]);
}

/// An itemize never steps (`\@nmbrlistfalse`): an `\item[…]` after a plain `\item` of an itemize nested in an
/// enumerate item still takes that enumerate item's "1", as pdflatex. 77u r3 counted the plain item as a numbered
/// sibling, so s2 had no refnum: `\the@itemii` is empty (sect06.rs), and an empty body is stored as no expansion,
/// which `prints_something` (dialect.rs) read as printing (rev77u2 finding 3). pdflatex "REFS [1] [1] [1]" (the plain
/// item keeps Perl's form, no refnum). Repro list-structure/item_after_an_itemize_sibling_takes_the_enclosing_label.
#[test]
fn item_after_an_itemize_sibling_takes_the_enclosing_label() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/item_after_an_itemize_sibling_takes_the_enclosing_label.tex"
  );
  let s2 = concat!(
    r#"<item labels="LABEL:s2" xml:id="S0.I1.i1.I1.ix2"><tags><tag><Math mode="inline" tex="\circ" text="compose" "#,
    r#"xml:id="S0.I1.i1.I1.ix2.m1"><XMath><XMTok meaning="compose" name="circ" role="MULOP">∘</XMTok></XMath>"#,
    r#"</Math></tag><tag role="refnum">1</tag><tag role="typerefnum">item <Math mode="inline" tex="\circ" "#,
    r#"text="compose" xml:id="S0.I1.i1.I1.ix2.m2"><XMath><XMTok meaning="compose" name="circ" role="MULOP">∘"#,
    r#"</XMTok></XMath></Math></tag></tags><para xml:id="S0.I1.i1.I1.ix2.p1"><p>circ</p></para></item>"#,
  );
  assert_elements(tex, RAW, (0, 0), &[("item", "S0.I1.i1.I1.ix2", s2)]);
}

/// The same in the kernel's lists: a first nested `\item[(W)]` takes its enclosing item's "1a" (pdflatex), where Perl's
/// `\p@enumiii` form gave "1(a)"; (X), after a numbered item of its own list whose `\p@enumii` (`\theenumi`) prints,
/// keeps Perl's "1" (pdflatex "1a"; KNOWN_PERL_ERRORS #1003). pdflatex
/// "Refs: [1] [1a] [1a] [1a] [1(a)i]". Repro list-structure/item_with_its_own_label_takes_the_enclosing_label.
#[test]
fn item_with_its_own_label_takes_the_enclosing_label() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/item_with_its_own_label_takes_the_enclosing_label.tex"
  );
  let x = nbsp(concat!(
    r#"<item labels="LABEL:x" xml:id="S0.I1.i1.I1.ix1"><tags><tag>(X)</tag><tag role="refnum">1</tag>"#,
    r#"<tag role="typerefnum">item (X)</tag></tags><para xml:id="S0.I1.i1.I1.ix1.p1"><p>custom</p>"#,
    r#"<enumerate xml:id="S0.I1.i1.I1.ix1.I1"><item labels="LABEL:w" xml:id="S0.I1.i1.I1.ix1.I1.ix1"><tags>"#,
    r#"<tag>(W)</tag><tag role="refnum">1a</tag><tag role="typerefnum">item (W)</tag></tags>"#,
    r#"<para xml:id="S0.I1.i1.I1.ix1.I1.ix1.p1"><p>w</p></para></item><item labels="LABEL:v" "#,
    r#"xml:id="S0.I1.i1.I1.ix1.I1.i1"><tags><tag>i.</tag><tag role="refnum">1(a)i</tag>"#,
    r#"<tag role="typerefnum">item~1(a)i</tag></tags><para xml:id="S0.I1.i1.I1.ix1.I1.i1.p1"><p>v</p></para></item>"#,
    r#"</enumerate></para></item>"#,
  ));
  assert_elements(tex, RAW, (0, 0), &[("item", "S0.I1.i1.I1.ix1", &x)]);
}

/// A formula's fonts are the math fonts, never the text family, series or shape around it (fontmath.ltx's math
/// versions): in a `\renewcommand\familydefault{\sfdefault}` document pdflatex sets `y`, `\alpha` and `\mathrm{c}` in
/// CMMI10/CMR10 and `\mathsf` in CMSS10 inside `\emph`, `\textbf`, `\textit` and a theorem body alike. Written relative
/// to the `sansserif italic` text element (Perl Document.pm:385-410) the italic `y` was `font="serif"`, which MathML
/// reads whole (MathML.pm:677) as upright (77u r2; 2608.14133, 2606.05381's theorems). The tokens are now relative to
/// the text default, the XML the same wherever the formula stands, and the MathML context font is read inside the
/// formula only. OXIDIZED_DESIGN_DIVERGENCES #780. Repro fonts-nfss/math_fonts_do_not_follow_a_sans_text_font.
#[test]
fn math_fonts_do_not_follow_a_sans_text_font() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/math_fonts_do_not_follow_a_sans_text_font.tex"
  );
  let (stderr, html) = convert_html(tex);
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  for (id, alt, body) in [
    (
      "p1.m1",
      r"y+\alpha",
      "<mrow><mi>y</mi><mo>+</mo><mi>\u{03B1}</mi></mrow>",
    ),
    (
      "p1.m2",
      r"\alpha+\mathsf{H}",
      "<mrow><mi>\u{03B1}</mi><mo>+</mo><mi>\u{1D5A7}</mi></mrow>",
    ),
    ("p1.m3", r"\mathsf{G}", "<mi>\u{1D5A6}</mi>"),
    (
      "p1.m4",
      r"y+\mathsf{K}",
      "<mrow><mi>y</mi><mo>+</mo><mi>\u{1D5AA}</mi></mrow>",
    ),
    (
      "Thmthm1.p1.m1",
      r"y+\mathrm{c}",
      "<mrow><mi>y</mi><mo>+</mo><mi mathvariant=\"normal\">c</mi></mrow>",
    ),
  ] {
    latexml::util::test::assert_element(
      &html,
      "math",
      &[&format!(r#"id="{id}""#)],
      &format!(
        r#"<math alttext="{alt}" class="ltx_Math" display="inline" id="{id}">{body}</math>"#
      ),
    );
  }
  // the XML: the `\emph` formula's tokens as outside any text element
  assert_elements(tex, RAW, (0, 0), &[(
    "Math",
    "p1.m1",
    r#"<Math mode="inline" tex="y+\alpha" text="y + alpha" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok><XMTok font="italic" name="alpha" role="UNKNOWN">α</XMTok></XMApp></XMath></Math>"#,
  )]);
}

/// The same in a serif document's `\textsf{\emph{…}}`, where Perl writes `y` as `font="serif"` against the
/// `sansserif italic` text and MathML.pm:677 sets it upright (KNOWN_PERL_ERRORS #1002): pdflatex CMMI10 for `y` and
/// `\alpha`, CMSS10 for `\mathsf{H}` in `\textbf`. Repro fonts-nfss/math_fonts_in_sans_italic_text_of_a_serif_document.
#[test]
fn math_fonts_in_sans_italic_text_of_a_serif_document() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/math_fonts_in_sans_italic_text_of_a_serif_document.tex"
  );
  let (stderr, html) = convert_html(tex);
  assert_eq!(
    (error_count(&stderr), warning_count(&stderr)),
    (0, 0),
    "{stderr}"
  );
  for (id, alt, body) in [
    ("p1.m1", "y", "<mi>y</mi>"),
    (
      "p1.m2",
      r"\alpha+\mathsf{H}",
      "<mrow><mi>\u{03B1}</mi><mo>+</mo><mi>\u{1D5A7}</mi></mrow>",
    ),
    ("p1.m3", "y", "<mi>y</mi>"),
  ] {
    latexml::util::test::assert_element(
      &html,
      "math",
      &[&format!(r#"id="{id}""#)],
      &format!(
        r#"<math alttext="{alt}" class="ltx_Math" display="inline" id="{id}">{body}</math>"#
      ),
    );
  }
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

/// `\normalfont` is latex.ltx's (:14113-14122): the document's `\familydefault`, `\seriesdefault`, `\shapedefault`
/// (and `\encodingdefault`), not Perl's `\rmdefault` (latex_constructs.pool.ltxml:5194). A
/// `\renewcommand\familydefault{\sfdefault}` document is sans from `\begin{document}` on, after a `\normalfont` and in
/// its footnotes (the note's neutral font is the same defaults): pdflatex CMSS10/CMSSI10/CMSS8. User ruling
/// 2026-10-20, OXIDIZED_DESIGN_DIVERGENCES #779. Repro fonts-nfss/family_default_sets_the_body.
#[test]
fn family_default_sets_the_body() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/family_default_sets_the_body.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    concat!(
      r#"<para xml:id="p1"><p><text font="sansserif">Body text. </text><text font="sansserif italic">it</text>"#,
      r#"<text font="sansserif"> </text><text font="typewriter">tt </text><text font="sansserif">back</text>"#,
      r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag><text font="sansserif">1</text></tag>"#,
      r#"<tag role="refnum"><text font="sansserif">1</text></tag><tag role="typerefnum"><text "#,
      r#"font="sansserif">footnote 1</text></tag></tags><text font="sansserif">Note.</text></note></p></para>"#,
    ),
  )]);
}

/// A preamble `\sffamily\bfseries` under `\familydefault` = `\sfdefault`: `\init@series@setup`'s `\reset@font` at
/// `\begin{document}` drops the bold but keeps the document's sans family (pdflatex: "Body text." CMSS10, `\textrm`
/// CMR10, `\textnormal` CMSS10). 77u r1 reset it to serif. Repro fonts-nfss/family_default_survives_a_preamble_switch.
#[test]
fn family_default_survives_a_preamble_switch() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/family_default_survives_a_preamble_switch.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r#"<para xml:id="p1"><p><text font="sansserif">Body text. </text>rm<text font="sansserif"> normal</text></p></para>"#,
  )]);
}

/// A `\familydefault` the font table does not know (`droidserif`, droidserif.sty:101 under `[default]`; cmpj3.sty's
/// papers, 2603.23615) resets to LaTeXML's serif, as Perl's `\rmdefault` did: `\normalfont` puts serif medium upright
/// in force before its `\selectfont` (`\lx@normalfont@base`), which keeps the font in force for an unknown family —
/// "b" stayed sans after `{\sffamily a \normalfont b}`. pdflatex: "a" CMSS10, "b" and the note DroidSerif. Repro
/// fonts-nfss/unknown_family_default_resets_to_serif.
#[test]
fn unknown_family_default_resets_to_serif() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/unknown_family_default_resets_to_serif.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    concat!(
      r#"<para xml:id="p1"><p><text font="sansserif">a </text>b<note mark="1" role="footnote" "#,
      r#"xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag>"#,
      r#"</tags>Note.</note></p></para>"#,
    ),
  )]);
}
