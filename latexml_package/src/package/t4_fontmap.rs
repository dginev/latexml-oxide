//! T4 font encoding: the fc ("African Computer") fonts of t4enc.def (fc package, Jörg Knappen).
//!
//! The lower half is T1's (the fc fonts are EC-shaped: fcaccent.mf, fcpunct.mf, fcrlig.mf draw T1's accents,
//! quotes, dashes and ligatures at T1's slots). The upper half is fccoding.mf's African Latin letters, each decoded to
//! the Unicode letter fccoding.mf's glyph name and t4enc.def's input command name (`\m{y}` hooktop y, slot 175 →
//! ƴ), checked against fc.rme's Appendix D "T4-to-UCS correspondence" — whose slips are not followed: 139 Ń (the
//! table's U+0134 is Ĵ), 150 Ʒ (U+0167 ŧ), 152 Ė (U+0166 Ŧ), 162 ɛ (U+0258 ɘ, the open e of `\m e`), 172 ʃ (U+03A3 Σ),
//! 173 ŋ / 174 ʋ (swapped), 226 ɛ̃ (ɘ̃). The letters with no precomposed character (fc.rme: E̱ O̱ e̱ o̱, Ɛ̃ Ɔ̃ ɛ̃ ɔ̃) and the
//! fj ligature are multi-character slots; the accents at 159 (`\G`), 188 (`\U`), 191 (`\I`) and the Polish stroke at
//! 223 (`\L`, `\l`: t4enc.def:62-74) are their combining marks.
//!
//! Perl has no T4 map, and its LaTeX.pool `\DeclareTextComposite` is ignored, so neither engine printed `\m{y}`: the
//! encoding fell back to OT1, the composite was never declared, and the command reached t4enc.def's own
//! `\errmessage` (2405.01972: 44 Errors, the letters lost; pdflatex 0). OXIDIZED_DESIGN_DIVERGENCES #777.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("T4", mixrc![
    // 0x00: the T1 lower half — the fc fonts' accents, quotes, dashes, ligatures (fcaccent.mf, fcpunct.mf, fcrlig.mf)
    '\u{0060}', '\u{00B4}', '\u{02C6}', '\u{02DC}', '\u{00A8}', '\u{02DD}', '\u{02DA}', '\u{02C7}',
    '\u{02D8}', '\u{00AF}', '\u{02D9}', '\u{00B8}', '\u{02DB}', '\u{201A}', '\u{2039}', '\u{203A}',
    '\u{201C}', '\u{201D}', '\u{201E}', '\u{00AB}', '\u{00BB}', '\u{2013}', '\u{2014}', '\u{200C}',
    '\u{0030}', '\u{0131}', '\u{0237}', '\u{FB00}', '\u{FB01}', '\u{FB02}', '\u{FB03}', '\u{FB04}',
    // 0x20: ASCII as T1 (fcpunct.mf, fcrompct.mf)
    '\u{2423}', '\u{0021}', '\u{0022}', '\u{0023}', '\u{0024}', '\u{0025}', '\u{0026}', '\u{2019}',
    '\u{0028}', '\u{0029}', '\u{002A}', '\u{002B}', '\u{002C}', '\u{002D}', '\u{002E}', '\u{002F}',
    '\u{0030}', '\u{0031}', '\u{0032}', '\u{0033}', '\u{0034}', '\u{0035}', '\u{0036}', '\u{0037}',
    '\u{0038}', '\u{0039}', '\u{003A}', '\u{003B}', '\u{003C}', '\u{003D}', '\u{003E}', '\u{003F}',
    '\u{0040}', '\u{0041}', '\u{0042}', '\u{0043}', '\u{0044}', '\u{0045}', '\u{0046}', '\u{0047}',
    '\u{0048}', '\u{0049}', '\u{004A}', '\u{004B}', '\u{004C}', '\u{004D}', '\u{004E}', '\u{004F}',
    '\u{0050}', '\u{0051}', '\u{0052}', '\u{0053}', '\u{0054}', '\u{0055}', '\u{0056}', '\u{0057}',
    '\u{0058}', '\u{0059}', '\u{005A}', '\u{005B}', '\u{005C}', '\u{005D}', '\u{005E}', '\u{005F}',
    '\u{2018}', '\u{0061}', '\u{0062}', '\u{0063}', '\u{0064}', '\u{0065}', '\u{0066}', '\u{0067}',
    '\u{0068}', '\u{0069}', '\u{006A}', '\u{006B}', '\u{006C}', '\u{006D}', '\u{006E}', '\u{006F}',
    '\u{0070}', '\u{0071}', '\u{0072}', '\u{0073}', '\u{0074}', '\u{0075}', '\u{0076}', '\u{0077}',
    '\u{0078}', '\u{0079}', '\u{007A}', '\u{007B}', '\u{007C}', '\u{007D}', '\u{007E}', '\u{2010}',
    // 0x80: Ɓ Ɗ Ɛ Ǝ Ƒ Ě Ɣ Ħ (t4enc.def:118-125 `\m B` `\m D` `\m E` `\M E` `\m F` `\v E` `\m G` `\B H`)
    '\u{0181}', '\u{018A}', '\u{0190}', '\u{018E}', '\u{0191}', '\u{011A}', '\u{0194}', '\u{0126}',
    // 0x88: Ƙ Ɲ Ɔ Ń Ʃ Ŋ Ʋ Ƴ (:126-134; `\m J` is fccoding.mf's Enj, `\m U`/`\m V` its round V)
    '\u{0198}', '\u{019D}', '\u{0186}', '\u{0143}', '\u{01A9}', '\u{014A}', '\u{01B2}', '\u{01B3}',
    // 0x90: Ƈ Ƥ Š Ṅ Ṉ Ṣ Ʒ Ŧ (:135-142)
    '\u{0187}', '\u{01A4}', '\u{0160}', '\u{1E44}', '\u{1E48}', '\u{1E62}', '\u{01B7}', '\u{0166}',
    // 0x98: Ė Ẹ Ƭ Ʈ ʧ fj đ, the double grave accent (:143-147, :116 `\tsh`, :22 `\G`)
    '\u{0116}', '\u{1EB8}', '\u{01AC}', '\u{01AE}', '\u{02A7}', 'f',        '\u{0111}', '\u{030F}',
    // 0xA0: ɓ ɗ ɛ ǝ ƒ ě ɣ ħ (:148-155)
    '\u{0253}', '\u{0257}', '\u{025B}', '\u{01DD}', '\u{0192}', '\u{011B}', '\u{0263}', '\u{0127}',
    // 0xA8: ƙ ɲ ɔ ń ʃ ŋ ʋ ƴ (:156-164)
    '\u{0199}', '\u{0272}', '\u{0254}', '\u{0144}', '\u{0283}', '\u{014B}', '\u{028B}', '\u{01B4}',
    // 0xB0: ƈ ƥ š ṅ ṉ ṣ ʒ ŧ (:165-172)
    '\u{0188}', '\u{01A5}', '\u{0161}', '\u{1E45}', '\u{1E49}', '\u{1E63}', '\u{0292}', '\u{0167}',
    // 0xB8: ė ẹ ƭ ʈ, the double universal accent, ¡ ¿, the universal accent (:173-176, :20-21 `\I` `\U`)
    '\u{0117}', '\u{1EB9}', '\u{01AD}', '\u{0288}', '\u{030E}', '\u{00A1}', '\u{00BF}', '\u{030D}',
    // 0xC0: Ɩ Ị Ɛ̃ Ã Ḿ Ɔ̃ Æ Ç (:177-183)
    '\u{0196}', '\u{1ECA}', '\u{0190}', '\u{00C3}', '\u{1E3E}', '\u{0186}', '\u{00C6}', '\u{00C7}',
    // 0xC8: È É Ê Ë E̱ Ē Ẽ Ĩ
    '\u{00C8}', '\u{00C9}', '\u{00CA}', '\u{00CB}', 'E',        '\u{0112}', '\u{1EBC}', '\u{0128}',
    // 0xD0: Ɖ Ñ Ò Ȯ Ô Õ Ö Œ (`\M D`, `\B D`, `\DH` share 208)
    '\u{0189}', '\u{00D1}', '\u{00D2}', '\u{022E}', '\u{00D4}', '\u{00D5}', '\u{00D6}', '\u{0152}',
    // 0xD8: Ø Ọ O̱ Ō Ǒ Ụ Ũ, the Polish stroke (`\L` `\l`)
    '\u{00D8}', '\u{1ECC}', 'O',        '\u{014C}', '\u{01D1}', '\u{1EE4}', '\u{0168}', '\u{0337}',
    // 0xE0: ɩ ị ɛ̃ ã ḿ ɔ̃ æ ç
    '\u{0269}', '\u{1ECB}', '\u{025B}', '\u{00E3}', '\u{1E3F}', '\u{0254}', '\u{00E6}', '\u{00E7}',
    // 0xE8: è é ê ë e̱ ē ẽ ĩ
    '\u{00E8}', '\u{00E9}', '\u{00EA}', '\u{00EB}', 'e',        '\u{0113}', '\u{1EBD}', '\u{0129}',
    // 0xF0: ɖ ñ ò ȯ ô õ ö œ
    '\u{0256}', '\u{00F1}', '\u{00F2}', '\u{022F}', '\u{00F4}', '\u{00F5}', '\u{00F6}', '\u{0153}',
    // 0xF8: ø ọ o̱ ō ǒ ụ ũ ß (:234)
    '\u{00F8}', '\u{1ECD}', 'o',        '\u{014D}', '\u{01D2}', '\u{1EE5}', '\u{0169}', '\u{00DF}'
  ]);
  // fc.rme Appendix D's combining sequences for the letters Unicode has no character for, and the fj ligature
  // (fcrlig.mf), "mapped to the two characters f j".
  DeclareFontMapMultichar!("T4", {
    157u8 => "fj",
    194u8 => "\u{0190}\u{0303}",
    197u8 => "\u{0186}\u{0303}",
    204u8 => "E\u{0331}",
    218u8 => "O\u{0331}",
    226u8 => "\u{025B}\u{0303}",
    229u8 => "\u{0254}\u{0303}",
    236u8 => "e\u{0331}",
    250u8 => "o\u{0331}",
  });
});
