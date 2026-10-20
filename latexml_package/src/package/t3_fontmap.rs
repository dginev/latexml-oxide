//! T3 font encoding: tipa's IPA fonts (tipa10 …, t3enc.def, FUKUI Rei).
//!
//! Each slot decodes to the glyph the tipa fonts draw there — tipa.mf's driver and its parts, whose `cmchar` names
//! identify the glyph (tipadiac.mf accents and diacritics 0-31 and the marks at 34-38, 58-62, 92-95, 126-127,
//! 152-159, 189-191; tipapnct.mf punctuation; tiparoml.mf letters; tipasym1-4.mf, tipagerm.mf, tipasc.mf IPA letters;
//! tipatone.mf tone letters 128-151) — as the IPA letter Unicode has for it, by t3enc.def's command names
//! (`\textschwa` 64, `\textesh` 83, `\textltailn` 241 …), cross-checked against tipauni.sty's Unicode table (TL
//! tipauni 0.7a: `\textltailn` "272, the input letters `@` ə, `N` ŋ, `:` ː …). The ASCII slots are tipa's input
//! convention (tipaman §3): digits, capitals and `@ : ; " |` are IPA letters, so `\textipa{[D@"lIm@]}` reads
//! `[ðəˈlɪmə]`; `g` (103) is the single-storey ɡ (`\textscriptg`), the two-storey g is `\textg` (165). The fonts'
//! ligature program (tipa10.tfm LIGTABLE, tipa.mf:66-67) makes `""` the secondary stress ˌ (127) and `||` the double
//! pipe ǁ (123): `t3_ligature`.
//!
//! An accent slot holds the combining mark it is set as (decoded alone on a no-break space, the Perl fontmaps' shape),
//! or the spacing accent where T1 has one (0-12); [`t3_accent_mark`] gives the mark above or below a letter that
//! t3enc.def's `\upper@accent`/`\lower@accent` set it as (t3enc_def.rs). Slots with no Unicode character — the tone
//! letters' parts 128-145 and stem 191 (tone.sty assembles them; tone_sty.rs maps `\tone{51}` to ˥˩), the Celtic
//! palatalization mark 32, the hooktop 35, the Viby and left-hooktop long letters 166, 167, 182, 183 — decode to
//! nothing, and a text symbol at one warns where it is used (`\lx@unmapped@text@symbol`, sect08.rs).
//!
//! Perl has no T3 map: tipa's symbols printed nothing and `\textipa`'s input letters decoded as OT1 ("D@"lIm@" for
//! ðəˈlɪmə). OXIDIZED_DESIGN_DIVERGENCES #781. Witnesses 1802.05444, 2405.01972 (`\textipa{\textltailn}` ɲ, `\!b` ɓ).
use crate::prelude::*;

/// The T3 map: each slot's glyph in the tipa fonts.
fn t3_map() -> Rc<[Option<char>]> {
  #[rustfmt::skip]
  let map = mixrc![
    // 0x00: grave, acute, circumflex, tilde, umlaut, double acute, ring, caron (tipadiac.mf:24-160, as T1's)
    '\u{0060}', '\u{00B4}', '\u{02C6}', '\u{02DC}', '\u{00A8}', '\u{02DD}', '\u{02DA}', '\u{02C7}',
    // 0x08: breve, macron, dot, cedilla, Polish hook, double grave, grave below, acute below (:162-273)
    '\u{02D8}', '\u{00AF}', '\u{02D9}', '\u{00B8}', '\u{02DB}', '\u{030F}', '\u{0316}', '\u{0317}',
    // 0x10: round cap / subscript arch, subscript bridge, inverted bridge, square, right/left half-ring, subscript W,
    // seagull (:275-413)
    '\u{0311}', '\u{032A}', '\u{033A}', '\u{033B}', '\u{0339}', '\u{031C}', '\u{032B}', '\u{033C}',
    // 0x18: over-cross, ı ȷ (tiparoml.mf:774-788), plus, raising, lowering, advancing, retracting signs (:415-503)
    '\u{033D}', '\u{0131}', '\u{0237}', '\u{031F}', '\u{031D}', '\u{031E}', '\u{0318}', '\u{0319}',
    // 0x20: Celtic palatalization mark, !, primary stress ˈ, hooktop, right hook, palatal hook, superimposed tilde, ’
    None,       '\u{0021}', '\u{02C8}', None,       '\u{0322}', '\u{0321}', '\u{0334}', '\u{2019}',
    // 0x28: ( ) * + , - . / (tipapnct.mf)
    '\u{0028}', '\u{0029}', '\u{002A}', '\u{002B}', '\u{002C}', '\u{002D}', '\u{002E}', '\u{002F}',
    // 0x30: the digits: ʉ ɨ ʌ ɜ ɥ ɐ ɒ ɤ (tipasym1.mf:21-244; t3enc.def:47-54)
    '\u{0289}', '\u{0268}', '\u{028C}', '\u{025C}', '\u{0265}', '\u{0250}', '\u{0252}', '\u{0264}',
    // 0x38: ɵ ɘ, length mark ː, half-length ˑ, bottom tie bar, =, top tie bar, ? (t3enc.def:55-56, :101-102)
    '\u{0275}', '\u{0258}', '\u{02D0}', '\u{02D1}', '\u{035C}', '\u{003D}', '\u{0361}', '\u{003F}',
    // 0x40: ə ɑ β ɕ ð ɛ ɸ ɣ (t3enc.def:60-70; `D` is `\dh`, :242)
    '\u{0259}', '\u{0251}', '\u{03B2}', '\u{0255}', '\u{00F0}', '\u{025B}', '\u{0278}', '\u{0263}',
    // 0x48: ɦ ɪ ʝ ʁ ʎ ɱ ŋ ɔ (:71-78; `N` is `\ng`, :243)
    '\u{0266}', '\u{026A}', '\u{029D}', '\u{0281}', '\u{028E}', '\u{0271}', '\u{014B}', '\u{0254}',
    // 0x50: ʔ ʕ ɾ ʃ θ ʊ ʋ ɯ (:79-87)
    '\u{0294}', '\u{0295}', '\u{027E}', '\u{0283}', '\u{03B8}', '\u{028A}', '\u{028B}', '\u{026F}',
    // 0x58: χ ʏ ʒ [, reversed apostrophe ʽ, ], corner ˺, open corner ˹ (:87-89, :218-220; tipadiac.mf:567-602)
    '\u{03C7}', '\u{028F}', '\u{0292}', '\u{005B}', '\u{02BD}', '\u{005D}', '\u{02FA}', '\u{02F9}',
    // 0x60: ‘ and the lower-case letters (tiparoml.mf), `g` the single-storey ɡ (t3enc.def:93 `\textscriptg`)
    '\u{2018}', '\u{0061}', '\u{0062}', '\u{0063}', '\u{0064}', '\u{0065}', '\u{0066}', '\u{0261}',
    '\u{0068}', '\u{0069}', '\u{006A}', '\u{006B}', '\u{006C}', '\u{006D}', '\u{006E}', '\u{006F}',
    '\u{0070}', '\u{0071}', '\u{0072}', '\u{0073}', '\u{0074}', '\u{0075}', '\u{0076}', '\u{0077}',
    // 0x78: x y z, double pipe ǁ, pipe ǀ, double-barred pipe ǂ, rhoticity ˞, secondary stress ˌ (t3enc.def:106,
    // :224-227)
    '\u{0078}', '\u{0079}', '\u{007A}', '\u{01C1}', '\u{01C0}', '\u{01C2}', '\u{02DE}', '\u{02CC}',
    // 0x80: the tone letters' parts (tipatone.mf:30-99): drawn at the height tone.sty raises them to, no character
    None,       None,       None,       None,       None,       None,       None,       None,
    None,       None,       None,       None,       None,       None,       None,       None,
    // 0x90: two more parts; minor and major group | ‖, downstep ꜜ, upstep ꜛ, global rise ↗, global fall ↘
    // (t3enc.def:231-236)
    None,       None,       '\u{007C}', '\u{2016}', '\u{A71C}', '\u{A71B}', '\u{2197}', '\u{2198}',
    // 0x98: the tone accents (tipatone.mf:203-282: mid acute, grave mid, rising-falling, falling-rising), vertical
    // bar, double vertical bar, grave dot, acute dot accents (tipadiac.mf:685-752)
    '\u{1DC4}', '\u{1DC5}', '\u{1DC8}', '\u{1DC9}', '\u{030D}', '\u{030E}', '\u{1DC0}', '\u{1DC1}',
    // 0xA0: ƀ đ ȡ ᶑ ᴇ g, left-hooktop long i, Viby i (t3enc.def:176-183, tipasym4.mf:21-149)
    '\u{0180}', '\u{0111}', '\u{0221}', '\u{1D91}', '\u{1D07}', '\u{0067}', None,       None,
    // 0xA8: ᴊ ʞ ƚ λ ƛ ɮ ȵ ꭢ (:184-191; 175 is tipasym4.mf:299 "Open O-E Ligature")
    '\u{1D0A}', '\u{029E}', '\u{019A}', '\u{03BB}', '\u{019B}', '\u{026E}', '\u{0235}', '\u{AB62}',
    // 0xB0: ω ꭥ ʆ ƫ ȶ ʦ, left-hooktop long y, Viby y (:192-199)
    '\u{03C9}', '\u{AB65}', '\u{0286}', '\u{01AB}', '\u{0236}', '\u{02A6}', None,       None,
    // 0xB8: ʓ ƹ ь ъ ˀ ˂ ˃, the tone letter stem (:200-207)
    '\u{0293}', '\u{01B9}', '\u{044C}', '\u{044A}', '\u{02C0}', '\u{02C2}', '\u{02C3}', None,
    // 0xC0: ᴀ ƈ ʗ ʤ ɚ ʚ ɞ ɝ (:141-148)
    '\u{1D00}', '\u{0188}', '\u{0297}', '\u{02A4}', '\u{025A}', '\u{029A}', '\u{025E}', '\u{025D}',
    // 0xC8: baby gamma ɤ, ʛ ɧ ʜ ɩ ɟ ƙ ʟ (:149-156)
    '\u{0264}', '\u{029B}', '\u{0267}', '\u{029C}', '\u{0269}', '\u{025F}', '\u{0199}', '\u{029F}',
    // 0xD0: ɮ ɷ ƥ ʠ ɼ ɺ ƭ ɶ (:157-164)
    '\u{026E}', '\u{0277}', '\u{01A5}', '\u{02A0}', '\u{027C}', '\u{027A}', '\u{01AD}', '\u{0276}',
    // 0xD8: ʇ ʧ ᴜ ʖ ʡ ʢ ȥ ƿ (:165-172)
    '\u{0287}', '\u{02A7}', '\u{1D1C}', '\u{0296}', '\u{02A1}', '\u{02A2}', '\u{0225}', '\u{01BF}',
    // 0xE0: ʙ ɓ ɗ ɖ ɠ ɢ æ ç (:110-115, :245; tipasym2.mf:590-647)
    '\u{0299}', '\u{0253}', '\u{0257}', '\u{0256}', '\u{0260}', '\u{0262}', '\u{00E6}', '\u{00E7}',
    // 0xE8: ħ ɟ ʄ ɫ ɬ ɭ ɰ ɳ (:116-123)
    '\u{0127}', '\u{025F}', '\u{0284}', '\u{026B}', '\u{026C}', '\u{026D}', '\u{0270}', '\u{0273}',
    // 0xF0: ɴ ɲ ʘ ɽ ɹ ɻ ʀ œ (:124-130, :246)
    '\u{0274}', '\u{0272}', '\u{0298}', '\u{027D}', '\u{0279}', '\u{027B}', '\u{0280}', '\u{0153}',
    // 0xF8: ø ʂ ʈ ʍ ʐ ʑ þ ƕ (:131-137, :247)
    '\u{00F8}', '\u{0282}', '\u{0288}', '\u{028D}', '\u{0290}', '\u{0291}', '\u{00FE}', '\u{0195}'
  ];
  map
}

/// Where t3enc.def sets an accent slot: above the letter (`\upper@accent`), below it (`\lower@accent`), or over it
/// (`\textsuperimposetilde`'s `\ipaclap`, `\ipabar`'s bar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccentPlace {
  /// `\upper@accent`, t3enc.def:268-280.
  Above,
  /// `\lower@accent`, t3enc.def:282-294.
  Below,
  /// Superimposed, t3enc.def:330-354.
  Over,
}

/// The combining mark of T3 slot `slot` set `place` on a letter, the glyph tipadiac.mf/tipatone.mf draw there as
/// t3enc.def:368-516 sets it: the tilde (3) above is U+0303, below (`\~*`, `\textsubtilde`) U+0330; the round cap (16)
/// above is the inverted breve U+0311, below (`\textsubarch`, `\u*`) U+032F; the stroke of `"` (34) below is the
/// syllabicity mark U+0329 (`\textsyllabic`, `\s`). `None` where Unicode has no mark: the subscript W set above
/// (`\textoverw`).
pub fn t3_accent_mark(slot: i64, place: AccentPlace) -> Option<char> {
  use AccentPlace::{Above, Below, Over};
  let mark = match (slot, place) {
    (0, Above) => '\u{0300}',
    (0 | 14, Below) => '\u{0316}',
    (1, Above) => '\u{0301}',
    (1 | 15, Below) => '\u{0317}',
    (2, Above) => '\u{0302}',
    (2, Below) => '\u{032D}',
    (3, Above) => '\u{0303}',
    (3, Below) => '\u{0330}',
    (4, Above) => '\u{0308}',
    (4, Below) => '\u{0324}',
    (5, Above) => '\u{030B}',
    (6, Above) => '\u{030A}',
    (6, Below) => '\u{0325}',
    (7, Above) => '\u{030C}',
    (7, Below) => '\u{032C}',
    (8, Above) => '\u{0306}',
    (8, Below) => '\u{032E}',
    (9, Above) => '\u{0304}',
    (9, Below) => '\u{0331}',
    (10, Above) => '\u{0307}',
    (10, Below) => '\u{0323}',
    (11, _) => '\u{0327}',
    (12, _) => '\u{0328}',
    (13, Above) => '\u{030F}',
    (14, Above) => '\u{0300}',
    (15, Above) => '\u{0301}',
    (16, Above) => '\u{0311}',
    (16, Below) => '\u{032F}',
    (17, Below) => '\u{032A}',
    (17, Above) => '\u{0346}',
    (18, Below) => '\u{033A}',
    (19, Below) => '\u{033B}',
    (20, Below) => '\u{0339}',
    (21, Below) => '\u{031C}',
    (22, Below) => '\u{032B}',
    (23, Below) => '\u{033C}',
    (24, Above) => '\u{033D}',
    (27, Below) => '\u{031F}',
    (28, Below) => '\u{031D}',
    (29, Below) => '\u{031E}',
    (30, Below) => '\u{0318}',
    (31, Below) => '\u{0319}',
    (34, Below) => '\u{0329}',
    (34, Above) => '\u{030D}',
    (38, Over) => '\u{0334}',
    (60, Below) => '\u{035C}',
    (62, Above) => '\u{0361}',
    (152, Above) => '\u{1DC4}',
    (153, Above) => '\u{1DC5}',
    (154, Above) => '\u{1DC8}',
    (155, Above) => '\u{1DC9}',
    (156, Above) => '\u{030D}',
    (157, Above) => '\u{030E}',
    (158, Above) => '\u{1DC0}',
    (159, Above) => '\u{1DC1}',
    _ => return None,
  };
  Some(mark)
}

/// tipa10.tfm's ligatures (tipa.mf:66-67), over the decoded characters: two primary stresses are the secondary stress,
/// two pipes the double pipe. TeX reads the ligature again with the next character and has no ligature for it, so a
/// third `"` follows the ˌ as ˈ.
fn t3_ligature(run: &str) -> String {
  let mut out = String::new();
  let mut chars = run.chars().peekable();
  while let Some(c) = chars.next() {
    match (c, chars.peek()) {
      ('\u{02C8}', Some('\u{02C8}')) => {
        chars.next();
        out.push('\u{02CC}');
      },
      ('\u{01C0}', Some('\u{01C0}')) => {
        chars.next();
        out.push('\u{01C1}');
      },
      _ => out.push(c),
    }
  }
  out
}

fn is_t3(font: &Font) -> bool { font.get_encoding().is_some_and(|encoding| encoding == "T3") }

LoadDefinitions!({
  DeclareFontMap!("T3", t3_map());
  DefLigature!(
    "\u{02C8}{2,}|\u{01C0}{2,}",
    |caps: &regex::Captures| t3_ligature(&caps[0]),
    fontTest => sub[font] { is_t3(font) }
  );
});
