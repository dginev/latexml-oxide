//! Red/green guards for batch 77x: tipa's T3 encoding prints — its input letters and named symbols through the T3 font
//! map (t3_fontmap.rs, the tipa fonts' glyphs), its diacritics as combining marks (t3enc_def.rs, t3enc.def's
//! `\upper@accent`/`\lower@accent`), its tone letters (tone_sty.rs), its accents outside T3 text (latex.ltx's
//! `\DeclareTextAccentDefault`/`\UseTextAccent`), and the glyphs Unicode lacks warn. Each guard asserts the diagnostics
//! by count and the authored content in its element. Witnesses 2405.01972, 1802.05444.
use super::perfect_kernel_batch57::{RAW, assert_elements, assert_elements_with};

/// tipa's input convention (tipaman §3): in T3 the digits, capitals and `@ : ; " |` are IPA letters, `g` the
/// single-storey ɡ, `""` and `||` the ligatures ˌ ǁ (tipa10.tfm), and tipa.sty's `\;` `\:` `\!` `\*` composites small
/// capitals, retroflexes, implosives and turned letters. Red (int36a): the OT1 letters, "[D@”lIm@] 0123456789 ABC…".
/// Repro fonts-nfss/t3_tipa_input_letters_print.
#[test]
fn t3_tipa_input_letters_print() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_input_letters_print.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>[\u{00F0}\u{0259}\u{02C8}l\u{026A}m\u{0259}] \
     \u{0289}\u{0268}\u{028C}\u{025C}\u{0265}\u{0250}\u{0252}\u{0264}\u{0275}\u{0258} \
     \u{0251}\u{03B2}\u{0255}\u{00F0}\u{025B}\u{0278}\u{0263}\u{0266}\u{026A}\u{029D}\u{0281}\u{028E}\u{0271}\u{014B}\
     \u{0254}\u{0294}\u{0295}\u{027E}\u{0283}\u{03B8}\u{028A}\u{028B}\u{026F}\u{03C7}\u{028F}\u{0292} \u{0259} \
     \u{0261} a\u{02D0} e\u{02D1} \u{02CC}a \u{02CC}\u{02C8}a \u{01C0} \u{01C1} \u{1D07} \u{0288} \u{0253} \
     \u{029E}</p></para>",
  )]);
}

/// tipa's named symbols (t3enc.def's slots), the macro-made letters Unicode has (ƀ đ ʣ), `\tipaencoding`, the
/// IPA environment and T3 in an `\mbox` in a formula. Red (int36a): the symbols dropped with 8 Warnings, `\textbarb` a
/// two-row table, "@" in the formula. Witness 2405.01972 (`\textipa{\textltailn}`). Repro
/// fonts-nfss/t3_tipa_symbols_print.
#[test]
fn t3_tipa_symbols_print() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_symbols_print.tex");
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>A \u{0259}, \u{0283}, \u{028C}, \u{0272} and \
       \u{025A}\u{0298}\u{02A4}\u{0264}.</p></para>",
    ),
    (
      "para",
      "p2",
      "<para xml:id=\"p2\"><p>B \u{0283}\u{0259}\u{014B} and t\u{0283} C \u{0259} <Math mode=\"inline\" \
       tex=\"x\\mbox{{@}}\" text=\"x * [\u{0259}]\" xml:id=\"p2.m1\"><XMath><XMApp><XMTok meaning=\"times\" \
       role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMText>\u{0259}</XMText>\
       </XMApp></XMath></Math> D \u{0180}\u{0111}\u{02A3}.</p></para>",
    ),
  ]);
}

/// t3enc.def's diacritics are the tipa font's accent slots set above or below the letter: the combining marks, the
/// `*` forms below (`\~*` U+0330, `\r*` U+0325, `\t*` U+035C), `\|[` the subscript bridge, the kernel accents in
/// T3 precomposed (á), nested accents in order (`\textdotbreve` ă̇), the tone accents (U+1DC4). Red (int36a): "n *̃a
/// *̊n ∥[t a t͡s *͡ts …" — the accent boxes lost, the native accents on the `*`. Repro
/// fonts-nfss/t3_tipa_diacritics_compose.
#[test]
fn t3_tipa_diacritics_compose() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_diacritics_compose.tex");
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>n\u{0325} a\u{0330} n\u{0325} t\u{032A} \u{0203} t\u{0361}s t\u{035C}s \u{1E73} \
       o\u{032C} \u{1E0D} \u{0201}</p></para>",
    ),
    (
      "para",
      "p2",
      "<para xml:id=\"p2\"><p>\u{00E1} \u{00E8} \u{00F4} \u{016B} \u{010D} \u{017C} \u{00E5} \u{0115} \u{0151} \
       \u{00FC} \u{00E7} \u{0103}\u{0307} a\u{1DC4}\nl\u{0334} m\u{0329} t\u{032A} e\u{031E}</p></para>",
    ),
  ]);
}

/// tone.sty's contours are Unicode's tone letters, one per level: `\tone{51}` ˥˩, `\rtone` the left-stem letters.
/// Red (int36a): empty raised boxes. Repro fonts-nfss/t3_tone_letters.
#[test]
fn t3_tone_letters() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_tone_letters.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>Tones \u{02E5}\u{02E9}, \u{02E7}\u{02E5}, \u{A716}\u{A712} and \u{02E7}.</p></para>",
  )]);
}

/// A T3 accent in T1 text is latex.ltx's `\UseTextAccent{T3}` (tipa.sty's `\DeclareTextAccentDefault`): the mark from
/// T3, the letter in T1 (`g` stays g); in a formula tipa's `\;` `\!` `\:` set the letters alone, as pdflatex. Red
/// (int36a): "Voiceless n and g, a" — the default undefined, the marks lost. Repro
/// fonts-nfss/t3_accent_outside_t3_text.
#[test]
fn t3_accent_outside_t3_text() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_accent_outside_t3_text.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>Voiceless n\u{0325} and g\u{0325}, \u{0203}; <Math mode=\"inline\" tex=\"abcd\" \
     text=\"a * b * c * d\" xml:id=\"p1.m1\"><XMath><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\
     <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok><XMTok \
     font=\"italic\" role=\"UNKNOWN\">c</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">d</XMTok></XMApp></XMath>\
     </Math> stays.</p></para>",
  )]);
}

/// A T3 glyph Unicode has no character for — the hooktop (slot 35), the subscript W set above (`\textoverw`) — is
/// dropped with a Warning each. Red (int36a): one Warning ("T3 has no font map"), the over-w lost silently. Repro
/// fonts-nfss/t3_slot_without_unicode_warns.
#[test]
fn t3_slot_without_unicode_warns() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t3_slot_without_unicode_warns.tex"
  );
  assert_elements_with(
    tex,
    RAW,
    (0, 2),
    &[
      "Font encoding T3 has no Unicode glyph for it: the text symbol \\texthooktop has no glyph and is dropped",
      "Font encoding T3 has no combining mark for the accent in slot 22 set Above",
    ],
    &[(
      "para",
      "p1",
      "<para xml:id=\"p1\"><p>Hook b and a.</p></para>",
    )],
  );
}

// ---- 77x r2 (rev77x findings) ----

/// amsmath.sty:155-175 re-declares `\,` `\!` `\:` `\;` (`\let\!\@undefined` …), so loaded after tipa.sty, whose
/// :429-458 made `\!` `\;` `\:` T3 text accents, they are the spacing commands again: the align row and the inline
/// formulas are what they are without tipa (amsmath_sty.rs). Red (77x r1): 5 Errors, `\!` read the next token as
/// its argument. Witness 2603.03017. Repro fonts-nfss/t3_tipa_then_amsmath_align_negspace.
#[test]
fn t3_tipa_then_amsmath_align_negspace() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_then_amsmath_align_negspace.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle\!\!x\leq\!y" text="x &lt;= y" xml:id="S0.E1.m3"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP" rpadding="-1.7pt">≤</XMTok><XMTok font="italic" lpadding="-3.4pt" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle\!\!x" text="x" xml:id="S0.E1.m1"><XMath><XMTok font="italic" lpadding="-3.4pt" role="UNKNOWN">x</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle\leq\!y" text="absent &lt;= y" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP" rpadding="-1.7pt">≤</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup><p>Inline <Math mode="inline" tex="a\!b" text="a * b" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="-1.7pt">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>, <Math mode="inline" tex="a\;b" text="a * b" xml:id="p1.m2"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="2.8pt">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>, <Math mode="inline" tex="a\&gt;b" text="a * b" xml:id="p1.m3"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="2.2pt">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math> and <Math mode="inline" tex="a\,b" text="a * b" xml:id="p1.m4"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="italic" role="UNKNOWN" rpadding="1.7pt">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math>.</p></para>"##,
  )]);
}

/// After amsmath re-declares `\!`, a `\!\! &` in a tabular cell is two negative thin spaces and the cell ends at the
/// `&`, as without tipa. Red (77x r1): 7 Errors, the `&` read as an accent argument, "After text." lost. Witness
/// 2605.00529. Repro fonts-nfss/t3_tipa_then_amsmath_tabular_negspace.
#[test]
fn t3_tipa_then_amsmath_tabular_negspace() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_then_amsmath_tabular_negspace.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "para",
      "p1",
      "<para xml:id=\"p1\"><tabular vattach=\"middle\"><tbody><tr><td align=\"left\">a \u{200B}\u{200B}<text \
       fontsize=\"90%\">x</text></td><td align=\"left\">b</td></tr><tr><td align=\"left\">c</td><td \
       align=\"left\">d</td></tr></tbody></tabular></para>",
    ),
    (
      "para",
      "p2",
      "<para xml:id=\"p2\"><p>After text.</p></para>",
    ),
  ]);
}

/// After amsmath re-declares `\;`, `,\;$x$` is a comma, a thick space and the formula. Red (77x r1): the `$` read as
/// an accent argument. Witness 2608.02769. Repro fonts-nfss/t3_tipa_then_amsmath_thickspace_before_math.
#[test]
fn t3_tipa_then_amsmath_thickspace_before_math() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_then_amsmath_thickspace_before_math.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>Values a,\u{2004}<Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"p1.m1\"><XMath><XMTok \
     font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math> and b.</p></para>",
  )]);
}

/// amsmath before tipa: `\!` `\;` stay tipa's T3 text accents, and latex.ltx:9958 `\UseTextAccent` would read the `&`,
/// `}` or `$` after one as its argument (pdflatex: 101 errors). The accent takes no argument there and says so (a
/// Warning each: the recovery keeps the cell, group and formula boundaries); before a letter it sets the letter in its
/// own encoding, as pdflatex (`\;E` is E); in a formula the letters alone. Red (77x r1): 5 Errors, the cells and the
/// formula lost. Witness 2609.13770 (`\crefname{section}{\S\!}`); 2605.00132 and 2607.12821 exercise the robust
/// `\UseTextAccent` and its math path. Repro fonts-nfss/t3_tipa_after_amsmath_accent_at_boundary.
#[test]
fn t3_tipa_after_amsmath_accent_at_boundary() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t3_tipa_after_amsmath_accent_at_boundary.tex"
  );
  let accent_without_argument = "has no argument before";
  assert_elements_with(
    tex,
    RAW,
    (0, 3),
    &[
      accent_without_argument,
      accent_without_argument,
      accent_without_argument,
    ],
    &[
      (
        "para",
        "p1",
        "<para xml:id=\"p1\"><tabular vattach=\"middle\"><tbody><tr><td align=\"left\">a</td><td \
         align=\"left\">b</td></tr><tr><td align=\"left\">c</td><td align=\"left\">d</td></tr></tbody></tabular>\
         </para>",
      ),
      (
        "para",
        "p2",
        r##"<para xml:id="p2"><p>Values a,<Math mode="inline" tex="x" text="x" xml:id="p2.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math> and E.</p><equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"><equation xml:id="S0.E1"><tags><tag>(1)</tag><tag role="refnum">1</tag></tags><MathFork><Math tex="\displaystyle x\leq y" text="x &lt;= y" xml:id="S0.E1.m3"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math><MathBranch><td align="right"><Math mode="inline" tex="\displaystyle x" text="x" xml:id="S0.E1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td><td align="left"><Math mode="inline" tex="\displaystyle\leq y" text="absent &lt;= y" xml:id="S0.E1.m2"><XMath><XMApp><XMTok meaning="less-than-or-equals" name="leq" role="RELOP">≤</XMTok><XMTok meaning="absent"/><XMTok font="italic" role="UNKNOWN">y</XMTok></XMApp></XMath></Math></td></MathBranch></MathFork></equation></equationgroup></para>"##,
      ),
    ],
  );
}

/// t3enc.def read a second time (after fontenc's T3 and tipa.sty) keeps the kernel accents the binding saved the
/// first time: it does not save its own wrappers as them, which then expanded to themselves. Red (77x r1):
/// Fatal:Timeout. Repro fonts-nfss/t3enc_def_loaded_twice.
#[test]
fn t3enc_def_loaded_twice() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3enc_def_loaded_twice.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>X: \u{00E4} \u{00E9} \u{00F1} \u{00E4} \u{00E9} a\u{0330} \u{0259} n\u{0325}.</p></para>",
  )]);
}

/// The T3 slots whose drawn glyph and IPA meaning differ follow the IPA meaning (tipauni.sty; user ruling 2026-10-20):
/// `!` the alveolar click ǃ, `&` ɞ, `\textcorner` the combining no-audible-release mark U+031A and `\textsubbar` the
/// minus sign below U+0320; the corner and the hooks (`\textrthook`, `\textpalhook`) attach to the letter before them,
/// the tie bars typed `<` `>` join the letters either side;
/// `\textopencorner` has no IPA mapping and stays ˹. Red (77x r1): "!a", ̴, "k˺", the hooks on a no-break space, a̱.
/// Witnesses 2606.11542, 2601.02906, 2609.04232. Repro fonts-nfss/t3_ipa_meaning_slots.
#[test]
fn t3_ipa_meaning_slots() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_ipa_meaning_slots.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>Click \u{01C3}a, \u{025E}; k\u{031A} and t\u{0322} t\u{0321}; a\u{0320}; \u{02F9}x\u{031A}; t\u{035C}s \
     and t\u{0361}s.</p></para>",
  )]);
}

/// t3enc.def's `\ipabar` sets a stroke over a character run (`\textbarc` c̵, `\textcrg` ɡ̵); over anything else —
/// `\textdoublebaresh`'s `\hb@xt@` box of another `\ipabar` — it keeps t3enc.def's construction, with no TeX source in
/// the text. Red (77x r1): "\̵hboxto7.5pt{\hss\ipabar{{\textesh}}\hss}". Repro fonts-nfss/t3_ipabar_built_symbols.
/// The prefix and no-source checks are deliberate: the rest of the paragraph is t3enc.def's box construction (the
/// bars as rules beneath ʃ, a SYNC_STATUS 77x residual), which a whole-element pin would freeze.
#[test]
fn t3_ipabar_built_symbols() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/t3_ipabar_built_symbols.tex");
  let xml = assert_elements(tex, RAW, (0, 0), &[]);
  let para = latexml::util::test::xml_element(&xml, "para", &["xml:id=\"p1\""]).expect("para p1");
  assert!(
    para.starts_with("<para xml:id=\"p1\"><p>Barred c\u{0335} and \u{0261}\u{0335}; esh "),
    "{para}"
  );
  assert!(!para.contains('\\') && !para.contains("hbox"), "{para}");
}

/// latex.ltx:9958-9965 `\UseTextAccent{OT1}{\'}{e}`: `\add@accent` (:9906-9912) sets `\accent19 \fontencoding{OT1}
/// \selectfont e`, and the font switch between the accent number and the letter is a run of assignments (tex.web
/// §1123 make_accent, §1270 do_assignments: protected macros expanded, `\relax` skipped). Red (77x r1, int36a, Perl):
/// "A ´e B." (KNOWN_PERL_ERRORS #1004). Repro fonts-nfss/use_text_accent_font_assignments.
#[test]
fn use_text_accent_font_assignments() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/use_text_accent_font_assignments.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>A \u{00E9} B.</p></para>",
  )]);
}

/// ot4enc.def:82 and ly1enc.def:42 put the dotless j at slot 17: ȷ U+0237. Red (77x r1): dropped, a Warning each.
/// Repro fonts-nfss/ot4_ly1_dotless_j.
#[test]
fn ot4_ly1_dotless_j() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/ot4_ly1_dotless_j.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>OT4 \u{0237}, LY1 \u{0237}.</p></para>",
  )]);
}

/// laeenc.def:83-97's shadda vowels (slots 198-203) are shadda U+0651 with the vowel, as arabi's DTP Naskh LAE font
/// draws them and the LFE map's 0xC6-0xCB. Red (77x r1): dropped, 6 Warnings. Repro fonts-nfss/lae_shadda_vowels.
#[test]
fn lae_shadda_vowels() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/fonts-nfss/lae_shadda_vowels.tex");
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    "<para xml:id=\"p1\"><p>\u{0628}\u{0651}\u{064B} \u{0628}\u{0651}\u{064C} \u{0628}\u{0651}\u{064D} \
     \u{0628}\u{0651}\u{064E} \u{0628}\u{0651}\u{064F} \u{0628}\u{0651}\u{0650}.</p></para>",
  )]);
}

/// tone.sty's `\tone` with a level outside 1-5 has no tone letter: tone.sty's construction (tone-letter parts with no
/// character) is kept and the loss says so. Red (77x r1): dropped silently. Witnesses 2601.19932, 2602.10003. Repro
/// fonts-nfss/t3_tone_level_outside_range_warns. The prefix check is deliberate: what follows "and" is tone.sty's
/// box construction for the level with no tone letter, the dropped content its Warning reports.
#[test]
fn t3_tone_level_outside_range_warns() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/t3_tone_level_outside_range_warns.tex"
  );
  let xml = assert_elements_with(tex, RAW, (0, 1), &["has no Chao tone letter"], &[]);
  let para = latexml::util::test::xml_element(&xml, "para", &["xml:id=\"p1\""]).expect("para p1");
  assert!(
    para.starts_with("<para xml:id=\"p1\"><p>Tones \u{02E5}\u{02E9} and "),
    "{para}"
  );
}

/// A number before `\!` stays in its formula: base_xmath.rs's number ligature (Perl Base_XMath.pool.ltxml:484-522)
/// reads a spacing hint as part of a number only where its `%space_chars` entry is a character, and `\!`'s is the empty
/// string, false in Perl — so the scan stops there. Taken as a separator, the hint ended a two-node "number" and the
/// NUMBER was folded into the hint the parser then drops. Red (int36a, 77x r1/r2 before the fix): `$2\!x$` text "x",
/// `$2\!\cdot 3$` "absent cdot 3". Witnesses 2609.36534 (`$+5.9\!\cdot\!10^{-7}$`), 2608.02769 (`\mathbf 1\!\left(`),
/// 2603.03017 (`(1 \! - \! f)`). Repro math-parse/number_before_negative_thin_space_kept.
#[test]
fn number_before_negative_thin_space_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/number_before_negative_thin_space_kept.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>A <Math mode="inline" tex="2\!x" text="2 * x" xml:id="p1.m1"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok meaning="2" role="NUMBER" rpadding="-1.7pt">2</XMTok><XMTok font="italic" role="UNKNOWN">x</XMTok></XMApp></XMath></Math>, <Math mode="inline" tex="2\!\cdot 3" text="2 cdot 3" xml:id="p1.m2"><XMath><XMApp><XMTok name="cdot" role="MULOP">⋅</XMTok><XMTok meaning="2" role="NUMBER" rpadding="-1.7pt">2</XMTok><XMTok meaning="3" role="NUMBER">3</XMTok></XMApp></XMath></Math>, <Math mode="inline" tex="+5.9\!\cdot\!10^{-7}" text="+ 5.9 cdot 10 ^ (- 7)" xml:id="p1.m3"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMApp><XMTok name="cdot" role="MULOP" rpadding="-1.7pt">⋅</XMTok><XMTok meaning="5.9" role="NUMBER" rpadding="-1.7pt">5.9</XMTok><XMApp><XMTok role="SUPERSCRIPTOP" scriptpos="post1"/><XMTok meaning="10" role="NUMBER">10</XMTok><XMApp><XMTok fontsize="70%" meaning="minus" role="ADDOP">-</XMTok><XMTok fontsize="70%" meaning="7" role="NUMBER">7</XMTok></XMApp></XMApp></XMApp></XMApp></XMath></Math> and <Math mode="inline" tex="\mathbf{1}\!\left(x\right)" text="1 * x" xml:id="p1.m4"><XMath><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok font="bold" meaning="1" role="NUMBER" rpadding="-1.7pt">1</XMTok><XMDual><XMRef idref="p1.m4.1"/><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMTok font="italic" role="UNKNOWN" xml:id="p1.m4.1">x</XMTok><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMApp></XMath></Math>.</p></para>"##,
  )]);
}
