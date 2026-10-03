//! LMC font encoding: montex's Mongolian Cyrillic (lmcenc.def, the kmr/kmss/kmtt fonts).
//!
//! Each slot decodes to the glyph the kmr fonts draw there (montex's METAFONT driver mcyrill.mf: mccoding.mf's slots,
//! mcyrsymb.mf's `<` `>` «» № ₮). The Cyrillic letters sit at the Latin slots they transliterate — `W` is В, `X` Х,
//! `J` Ж, `Q` Ч, `C` Ц, `H` Һ, `Y` Ы — and the rest of the alphabet above 127 (lmcenc.def's `\DeclareTextSymbol`s);
//! punctuation, digits and accents are cmr's, as OT1's (punct.mf, romanp.mf, accent.mf, comlig.mf), and `V`/`v` hold no
//! glyph. Perl
//! has no LMC map: the encoding falls back to OT1, so `{\mnr Xalx}` read "Xalx" where pdflatex prints "Халх", and
//! `\No`/`\MyTogrog` printed nothing (KNOWN_PERL_ERRORS #438). Witnesses: montex/montex, montex/mlsquick.
use crate::prelude::*;

/// kmr10's ligature program (kmr10.tfm LIGTABLE, from mcyrill.mf), over the decoded characters. Every LMC font has
/// the montex ligatures (mcyrligs): `"` (” in the roman fonts, `"` in kmtt) before a vowel is its umlaut letter —
/// lmcenc.def's `\DeclareTextComposite{\"}` — and the `y`/`Y` digraphs and `sh`/`qh`/`QH` are single letters.
/// The roman fonts add cmr's `` `` `` → “ and `''` → ” — which TeX's ligature program carries on into the vowel after
/// it (`''o` → ө) — and `<<`/`>>` → «»; the shared quote ligatures (tex_fonts.rs) are OT1's, T1's and TU's only. kmtt
/// has none of those (`ligs:=0`), nor dashes; the roman fonts' `--`/`---` are the shared text ligatures.
const UMLAUTS: [(char, char); 10] = [
  ('\u{0410}', '\u{042D}'),
  ('\u{0415}', '\u{0401}'),
  ('\u{0418}', '\u{0419}'),
  ('\u{041E}', '\u{04E8}'),
  ('\u{0423}', '\u{04AE}'),
  ('\u{0430}', '\u{044D}'),
  ('\u{0435}', '\u{0451}'),
  ('\u{0438}', '\u{0439}'),
  ('\u{043E}', '\u{04E9}'),
  ('\u{0443}', '\u{04AF}'),
];
const DIGRAPHS: [(&str, &str); 14] = [
  ("\u{044B}\u{0438}", "\u{0439}"),
  ("\u{044B}\u{043E}", "\u{0451}"),
  ("\u{044B}\u{0443}", "\u{044E}"),
  ("\u{044B}\u{0430}", "\u{044F}"),
  ("\u{042B}\u{0418}", "\u{0419}"),
  ("\u{042B}\u{041E}", "\u{0401}"),
  ("\u{042B}\u{043E}", "\u{0401}"),
  ("\u{042B}\u{0423}", "\u{042E}"),
  ("\u{042B}\u{0443}", "\u{042E}"),
  ("\u{042B}\u{0410}", "\u{042F}"),
  ("\u{042B}\u{0430}", "\u{042F}"),
  ("\u{0441}\u{04BB}", "\u{0448}"),
  ("\u{0447}\u{04BB}", "\u{0449}"),
  ("\u{0427}\u{04BA}", "\u{0429}"),
];

/// The ligature of a matched sequence: an umlaut letter for a quote and its vowel, else the digraph's or the
/// guillemet's letter.
fn lmc_ligature(matched: &str) -> String {
  match matched {
    "<<" => return "\u{00AB}".to_string(),
    ">>" => return "\u{00BB}".to_string(),
    "\u{2018}\u{2018}" => return "\u{201C}".to_string(),
    "\u{2019}\u{2019}" => return "\u{201D}".to_string(),
    _ => {},
  }
  if let Some(vowel) = matched.chars().last()
    && matched.starts_with(['\u{201D}', '"'])
    && let Some((_, umlaut)) = UMLAUTS.iter().find(|(plain, _)| *plain == vowel)
  {
    return umlaut.to_string();
  }
  DIGRAPHS
    .iter()
    .find(|(from, _)| *from == matched)
    .map_or_else(|| matched.to_string(), |(_, to)| to.to_string())
}

fn is_lmc(font: &Font) -> bool {
  font
    .get_encoding()
    .is_some_and(|encoding| encoding == "LMC")
}

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("LMC", mixrc![
    // 0x00: - - - -
    None, None, None, None,
    // 0x04: - - - -
    None, None, None, None,
    // 0x08: - - - -
    None, None, None, None,
    // 0x0C: - - - -
    None, None, None, None,
    // 0x10: - - '`' left-pointing_double_angle_quotation_mark
    None, None, Some('`'), Some('\u{00AB}'),
    // 0x14: right-pointing_double_angle_quotation_mark breve macron ring_above
    Some('\u{00BB}'), Some('\u{02D8}'), Some('\u{00AF}'), Some('\u{02DA}'),
    // 0x18: cedilla - - -
    Some('\u{00B8}'), None, None, None,
    // 0x1C: - - - -
    None, None, None, None,
    // 0x20: combining_short_stroke_overlay '!' right_double_quotation_mark '#'
    Some('\u{0335}'), Some('!'), Some('\u{201D}'), Some('#'),
    // 0x24: '$' '%' '&' right_single_quotation_mark
    Some('$'), Some('%'), Some('&'), Some('\u{2019}'),
    // 0x28: '(' ')' '*' '+'
    Some('('), Some(')'), Some('*'), Some('+'),
    // 0x2C: ',' '-' '.' '/'
    Some(','), Some('-'), Some('.'), Some('/'),
    // 0x30: '0' '1' '2' '3'
    Some('0'), Some('1'), Some('2'), Some('3'),
    // 0x34: '4' '5' '6' '7'
    Some('4'), Some('5'), Some('6'), Some('7'),
    // 0x38: '8' '9' ':' ';'
    Some('8'), Some('9'), Some(':'), Some(';'),
    // 0x3C: '<' '=' '>' '?'
    Some('<'), Some('='), Some('>'), Some('?'),
    // 0x40: '@' capital_a capital_be capital_tse
    Some('@'), Some('\u{0410}'), Some('\u{0411}'), Some('\u{0426}'),
    // 0x44: capital_de capital_ie capital_ef capital_ghe
    Some('\u{0414}'), Some('\u{0415}'), Some('\u{0424}'), Some('\u{0413}'),
    // 0x48: capital_shha capital_i capital_zhe capital_ka
    Some('\u{04BA}'), Some('\u{0418}'), Some('\u{0416}'), Some('\u{041A}'),
    // 0x4C: capital_el capital_em capital_en capital_o
    Some('\u{041B}'), Some('\u{041C}'), Some('\u{041D}'), Some('\u{041E}'),
    // 0x50: capital_pe capital_che capital_er capital_es
    Some('\u{041F}'), Some('\u{0427}'), Some('\u{0420}'), Some('\u{0421}'),
    // 0x54: capital_te capital_u - capital_ve
    Some('\u{0422}'), Some('\u{0423}'), None, Some('\u{0412}'),
    // 0x58: capital_ha capital_yeru capital_ze '['
    Some('\u{0425}'), Some('\u{042B}'), Some('\u{0417}'), Some('['),
    // 0x5C: left_double_quotation_mark ']' modifier_circumflex_accent dot_above
    Some('\u{201C}'), Some(']'), Some('\u{02C6}'), Some('\u{02D9}'),
    // 0x60: left_single_quotation_mark small_a small_be small_tse
    Some('\u{2018}'), Some('\u{0430}'), Some('\u{0431}'), Some('\u{0446}'),
    // 0x64: small_de small_ie small_ef small_ghe
    Some('\u{0434}'), Some('\u{0435}'), Some('\u{0444}'), Some('\u{0433}'),
    // 0x68: small_shha small_i small_zhe small_ka
    Some('\u{04BB}'), Some('\u{0438}'), Some('\u{0436}'), Some('\u{043A}'),
    // 0x6C: small_el small_em small_en small_o
    Some('\u{043B}'), Some('\u{043C}'), Some('\u{043D}'), Some('\u{043E}'),
    // 0x70: small_pe small_che small_er small_es
    Some('\u{043F}'), Some('\u{0447}'), Some('\u{0440}'), Some('\u{0441}'),
    // 0x74: small_te small_u - small_ve
    Some('\u{0442}'), Some('\u{0443}'), None, Some('\u{0432}'),
    // 0x78: small_ha small_yeru small_ze en_dash
    Some('\u{0445}'), Some('\u{044B}'), Some('\u{0437}'), Some('\u{2013}'),
    // 0x7C: em_dash double_acute_accent small_tilde diaeresis
    Some('\u{2014}'), Some('\u{02DD}'), Some('\u{02DC}'), Some('\u{00A8}'),
    // 0x80: - - - -
    None, None, None, None,
    // 0x84: - - - -
    None, None, None, None,
    // 0x88: - - - -
    None, None, None, None,
    // 0x8C: - - - -
    None, None, None, None,
    // 0x90: capital_shcha - capital_sha capital_hard_sign
    Some('\u{0429}'), None, Some('\u{0428}'), Some('\u{042A}'),
    // 0x94: capital_soft_sign capital_yu capital_ya -
    Some('\u{042C}'), Some('\u{042E}'), Some('\u{042F}'), None,
    // 0x98: - - - -
    None, None, None, None,
    // 0x9C: - - - -
    None, None, None, None,
    // 0xA0: - - - -
    None, None, None, None,
    // 0xA4: - - - -
    None, None, None, None,
    // 0xA8: - - - -
    None, None, None, None,
    // 0xAC: - - - -
    None, None, None, None,
    // 0xB0: small_shcha - small_sha small_hard_sign
    Some('\u{0449}'), None, Some('\u{0448}'), Some('\u{044A}'),
    // 0xB4: small_soft_sign small_yu small_ya -
    Some('\u{044C}'), Some('\u{044E}'), Some('\u{044F}'), None,
    // 0xB8: - - - -
    None, None, None, None,
    // 0xBC: - - - -
    None, None, None, None,
    // 0xC0: - - - -
    None, None, None, None,
    // 0xC4: capital_e - - -
    Some('\u{042D}'), None, None, None,
    // 0xC8: - - - capital_io
    None, None, None, Some('\u{0401}'),
    // 0xCC: - - - capital_short_i
    None, None, None, Some('\u{0419}'),
    // 0xD0: - - - -
    None, None, None, None,
    // 0xD4: - - capital_barred_o -
    None, None, Some('\u{04E8}'), None,
    // 0xD8: - - - -
    None, None, None, None,
    // 0xDC: capital_straight_u - - -
    Some('\u{04AE}'), None, None, None,
    // 0xE0: - - - -
    None, None, None, None,
    // 0xE4: small_e - - -
    Some('\u{044D}'), None, None, None,
    // 0xE8: - - - small_io
    None, None, None, Some('\u{0451}'),
    // 0xEC: - - - small_short_i
    None, None, None, Some('\u{0439}'),
    // 0xF0: - - - -
    None, None, None, None,
    // 0xF4: - - small_barred_o -
    None, None, Some('\u{04E9}'), None,
    // 0xF8: - numero_sign tugrik_sign tugrik_sign
    None, Some('\u{2116}'), Some('\u{20AE}'), Some('\u{20AE}'),
    // 0xFC: small_straight_u - - -
    Some('\u{04AF}'), None, None, None
  ]);
  // kmtt (lmccmtt.fd: LMC/cmtt), built with `ligs:=0`: romsub.mf's and sym.mf's ASCII glyphs where the roman fonts have
  // ligatures and accents — ↑ ↓ ' at 0x0B-0x0D, ␣ " \\ ^ _ { | } ~ — as OT1's typewriter map (tex_fonts.rs).
  #[rustfmt::skip]
  DeclareFontMap!("LMC", mixrc![
    // 0x00
    None, None, None, None,
    // 0x04
    None, None, None, None,
    // 0x08
    None, None, None, Some('\u{2191}'),
    // 0x0C
    Some('\u{2193}'), Some('\''), None, None,
    // 0x10
    None, None, Some('`'), Some('\u{00AB}'),
    // 0x14
    Some('\u{00BB}'), Some('\u{02D8}'), Some('\u{00AF}'), Some('\u{02DA}'),
    // 0x18
    Some('\u{00B8}'), None, None, None,
    // 0x1C
    None, None, None, None,
    // 0x20
    Some('\u{2423}'), Some('!'), Some('"'), Some('#'),
    // 0x24
    Some('$'), Some('%'), Some('&'), Some('\u{2019}'),
    // 0x28
    Some('('), Some(')'), Some('*'), Some('+'),
    // 0x2C
    Some(','), Some('-'), Some('.'), Some('/'),
    // 0x30
    Some('0'), Some('1'), Some('2'), Some('3'),
    // 0x34
    Some('4'), Some('5'), Some('6'), Some('7'),
    // 0x38
    Some('8'), Some('9'), Some(':'), Some(';'),
    // 0x3C
    Some('<'), Some('='), Some('>'), Some('?'),
    // 0x40
    Some('@'), Some('\u{0410}'), Some('\u{0411}'), Some('\u{0426}'),
    // 0x44
    Some('\u{0414}'), Some('\u{0415}'), Some('\u{0424}'), Some('\u{0413}'),
    // 0x48
    Some('\u{04BA}'), Some('\u{0418}'), Some('\u{0416}'), Some('\u{041A}'),
    // 0x4C
    Some('\u{041B}'), Some('\u{041C}'), Some('\u{041D}'), Some('\u{041E}'),
    // 0x50
    Some('\u{041F}'), Some('\u{0427}'), Some('\u{0420}'), Some('\u{0421}'),
    // 0x54
    Some('\u{0422}'), Some('\u{0423}'), None, Some('\u{0412}'),
    // 0x58
    Some('\u{0425}'), Some('\u{042B}'), Some('\u{0417}'), Some('['),
    // 0x5C
    Some('\\'), Some(']'), Some('^'), Some('_'),
    // 0x60
    Some('\u{2018}'), Some('\u{0430}'), Some('\u{0431}'), Some('\u{0446}'),
    // 0x64
    Some('\u{0434}'), Some('\u{0435}'), Some('\u{0444}'), Some('\u{0433}'),
    // 0x68
    Some('\u{04BB}'), Some('\u{0438}'), Some('\u{0436}'), Some('\u{043A}'),
    // 0x6C
    Some('\u{043B}'), Some('\u{043C}'), Some('\u{043D}'), Some('\u{043E}'),
    // 0x70
    Some('\u{043F}'), Some('\u{0447}'), Some('\u{0440}'), Some('\u{0441}'),
    // 0x74
    Some('\u{0442}'), Some('\u{0443}'), None, Some('\u{0432}'),
    // 0x78
    Some('\u{0445}'), Some('\u{044B}'), Some('\u{0437}'), Some('{'),
    // 0x7C
    Some('|'), Some('}'), Some('~'), Some('\u{00A8}'),
    // 0x80
    None, None, None, None,
    // 0x84
    None, None, None, None,
    // 0x88
    None, None, None, None,
    // 0x8C
    None, None, None, None,
    // 0x90
    Some('\u{0429}'), None, Some('\u{0428}'), Some('\u{042A}'),
    // 0x94
    Some('\u{042C}'), Some('\u{042E}'), Some('\u{042F}'), None,
    // 0x98
    None, None, None, None,
    // 0x9C
    None, None, None, None,
    // 0xA0
    None, None, None, None,
    // 0xA4
    None, None, None, None,
    // 0xA8
    None, None, None, None,
    // 0xAC
    None, None, None, None,
    // 0xB0
    Some('\u{0449}'), None, Some('\u{0448}'), Some('\u{044A}'),
    // 0xB4
    Some('\u{044C}'), Some('\u{044E}'), Some('\u{044F}'), None,
    // 0xB8
    None, None, None, None,
    // 0xBC
    None, None, None, None,
    // 0xC0
    None, None, None, None,
    // 0xC4
    Some('\u{042D}'), None, None, None,
    // 0xC8
    None, None, None, Some('\u{0401}'),
    // 0xCC
    None, None, None, Some('\u{0419}'),
    // 0xD0
    None, None, None, None,
    // 0xD4
    None, None, Some('\u{04E8}'), None,
    // 0xD8
    None, None, None, None,
    // 0xDC
    Some('\u{04AE}'), None, None, None,
    // 0xE0
    None, None, None, None,
    // 0xE4
    Some('\u{044D}'), None, None, None,
    // 0xE8
    None, None, None, Some('\u{0451}'),
    // 0xEC
    None, None, None, Some('\u{0439}'),
    // 0xF0
    None, None, None, None,
    // 0xF4
    None, None, Some('\u{04E9}'), None,
    // 0xF8
    None, Some('\u{2116}'), Some('\u{20AE}'), Some('\u{20AE}'),
    // 0xFC
    Some('\u{04AF}'), None, None, None
  ], "typewriter");

  let vowels: String = UMLAUTS.iter().map(|(vowel, _)| *vowel).collect();
  let digraphs: Vec<&str> = DIGRAPHS.iter().map(|(from, _)| *from).collect();
  DefLigature!(
    &format!("[\u{201D}\"][{vowels}]|{}", digraphs.join("|")),
    |caps: &regex::Captures| lmc_ligature(&caps[0]),
    fontTest => sub[font] { is_lmc(font) }
  );
  // Defined after the montex ligatures, so it runs first (`DefLigature` unshifts): `''` becomes ” and the umlaut
  // ligature then reads it, as TeX's LIG carries on with the new character (`''o` → ө; `'''o` → ”’о).
  DefLigature!(
    "\u{2018}\u{2018}|\u{2019}\u{2019}|<<|>>",
    |caps: &regex::Captures| lmc_ligature(&caps[0]),
    fontTest => sub[font] { is_lmc(font) && font.get_family().is_none_or(|family| family != "typewriter") }
  );
});
