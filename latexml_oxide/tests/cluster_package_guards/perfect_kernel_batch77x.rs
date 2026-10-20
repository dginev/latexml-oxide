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
