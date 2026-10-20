//! OT2 font encoding: the Washington University Cyrillic fonts (wncyr, ot2enc.def), Cyrillic in the Latin slots.
//!
//! Each slot decodes to the glyph wncyr10 draws there (wncyr10.afm's names: `A` А, `C` Ц, `Q` Ч, `V` В, `W` Щ, `X` Ш,
//! `Y` Ы, `J` Ј …), the letters as ot2enc.def names them (`\DeclareTextSymbol{\CYRNJE}{OT2}{0}` … `{\cyrhrdsn}{OT2}
//! {127}`, Unicode from ot2enc.dfu): 32, 36, 38 and 64 are the accents of `\"`, `\U`, `\'` and `\u`, 61 the dotless i
//! of `\.{\i}` (і); the fonts have no upper half. The transliteration's digraphs are the fonts' ligatures (`sh` ш,
//! `ch` ч, `zh` ж, `shch` щ, `ya` я, `e1` э …: `ot2_ligature`). Perl has no OT2 map: the encoding fell back to OT1, so
//! `\CYRZH` printed nothing and `{\fontencoding{OT2}\selectfont Moskva}` printed "Moskva" where pdflatex prints
//! "Москва". OXIDIZED_DESIGN_DIVERGENCES #777.
use rustc_hash::FxHashMap;

use crate::prelude::*;

/// wncyr10.tfm's ligature program (tftopl's LIGTABLE; every OT2 font shares it): slot `a` followed by slot `b` is slot
/// `c`, TeX's `LIG` (`=:`), whose result is read again with the next character — `x c` is the digit `7`, and `7 h` is
/// щ. The dashes are the shared text ligatures (tex_fonts.rs), the same in wncyr.
const LIGATURES: [(u8, u8, u8); 78] = [
  (b'K', b'H', b'H'),
  (b'K', b'h', b'H'),
  (b'T', b'S', b'C'),
  (b'T', b's', b'C'),
  (b'\'', b'\'', 34),
  (b'-', b'-', 123),
  (123, b'-', 124),
  (b'6', b'h', b'W'),
  (b'6', b'H', b'W'),
  (b'7', b'h', b'w'),
  (b'C', b'1', 7),
  (b'C', b'H', b'Q'),
  (b'C', b'h', b'Q'),
  (b'D', b'1', 6),
  (b'D', b'2', 2),
  (b'D', b'3', 22),
  (b'D', b'J', 6),
  (b'D', b'j', 6),
  (b'E', b'0', 19),
  (b'E', b'1', 3),
  (b'E', b'2', 5),
  (b'I', b'0', 18),
  (b'I', b'1', 4),
  (b'J', b'1', 23),
  (b'J', b'2', 16),
  (b'L', b'1', 1),
  (b'L', b'J', 1),
  (b'L', b'j', 1),
  (b'N', b'0', 125),
  (b'N', b'1', 0),
  (b'N', b'J', 0),
  (b'N', b'j', 0),
  (b'P', b'1', 94),
  (b'P', b'2', 95),
  (b't', b's', b'c'),
  (b'S', b'H', b'X'),
  (b'S', b'h', b'X'),
  (b'X', b'Q', b'W'),
  (b'X', b'q', b'W'),
  (b'X', b'C', b'6'),
  (b'X', b'c', b'6'),
  (b'Y', b'A', 23),
  (b'Y', b'a', 23),
  (b'Y', b'U', 16),
  (b'Y', b'u', 16),
  (b'Z', b'1', 17),
  (b'Z', b'H', 17),
  (b'Z', b'h', 17),
  (b'[', b'[', 60),
  (b']', b']', 62),
  (b'`', b'`', 92),
  (b'e', b'0', 27),
  (b'e', b'1', 11),
  (b'e', b'2', 13),
  (b'c', b'1', 15),
  (b'c', b'h', b'q'),
  (b'd', b'1', 14),
  (b'd', b'2', 10),
  (b'd', b'3', 30),
  (b'd', b'j', 14),
  (b'i', b'0', 26),
  (b'i', b'1', 12),
  (b'j', b'1', 31),
  (b'j', b'2', 24),
  (b'k', b'h', b'h'),
  (b'l', b'1', 9),
  (b'l', b'j', 9),
  (b'n', b'1', 8),
  (b'n', b'j', 8),
  (b'p', b'1', 126),
  (b'p', b'2', 127),
  (b'x', b'q', b'w'),
  (b'x', b'c', b'7'),
  (b'y', b'a', 31),
  (b'y', b'u', 24),
  (b'z', b'1', 25),
  (b'z', b'h', 25),
  (b's', b'h', b'x'),
];

/// The OT2 map: each slot's glyph in the wncyr fonts.
fn ot2_map() -> Rc<[Option<char>]> {
  #[rustfmt::skip]
  let map = mixrc![
    // 0x00: Њ Љ Џ Э І Є Ђ Ћ
    '\u{040A}', '\u{0409}', '\u{040F}', '\u{042D}', '\u{0406}', '\u{0404}', '\u{0402}', '\u{040B}',
    // 0x08: њ љ џ э і є ђ ћ
    '\u{045A}', '\u{0459}', '\u{045F}', '\u{044D}', '\u{0456}', '\u{0454}', '\u{0452}', '\u{045B}',
    // 0x10: Ю Ж Й Ё Ѵ Ѳ Ѕ Я
    '\u{042E}', '\u{0416}', '\u{0419}', '\u{0401}', '\u{0474}', '\u{0472}', '\u{0405}', '\u{042F}',
    // 0x18: ю ж й ё ѵ ѳ ѕ я
    '\u{044E}', '\u{0436}', '\u{0439}', '\u{0451}', '\u{0475}', '\u{0473}', '\u{0455}', '\u{044F}',
    // 0x20: ¨ ! ” Ѣ ˘ % ´ ’
    '\u{00A8}', '\u{0021}', '\u{201D}', '\u{0462}', '\u{02D8}', '\u{0025}', '\u{00B4}', '\u{2019}',
    // 0x28: ( ) * ѣ , - . /
    '\u{0028}', '\u{0029}', '\u{002A}', '\u{0463}', '\u{002C}', '\u{002D}', '\u{002E}', '\u{002F}',
    // 0x30: 0 1 2 3 4 5 6 7
    '\u{0030}', '\u{0031}', '\u{0032}', '\u{0033}', '\u{0034}', '\u{0035}', '\u{0036}', '\u{0037}',
    // 0x38: 8 9 : ; « ı » ?
    '\u{0038}', '\u{0039}', '\u{003A}', '\u{003B}', '\u{00AB}', '\u{0131}', '\u{00BB}', '\u{003F}',
    // 0x40: ˘ А Б Ц Д Е Ф Г
    '\u{02D8}', '\u{0410}', '\u{0411}', '\u{0426}', '\u{0414}', '\u{0415}', '\u{0424}', '\u{0413}',
    // 0x48: Х И Ј К Л М Н О
    '\u{0425}', '\u{0418}', '\u{0408}', '\u{041A}', '\u{041B}', '\u{041C}', '\u{041D}', '\u{041E}',
    // 0x50: П Ч Р С Т У В Щ
    '\u{041F}', '\u{0427}', '\u{0420}', '\u{0421}', '\u{0422}', '\u{0423}', '\u{0412}', '\u{0429}',
    // 0x58: Ш Ы З [ “ ] Ь Ъ
    '\u{0428}', '\u{042B}', '\u{0417}', '\u{005B}', '\u{201C}', '\u{005D}', '\u{042C}', '\u{042A}',
    // 0x60: ‘ а б ц д е ф г
    '\u{2018}', '\u{0430}', '\u{0431}', '\u{0446}', '\u{0434}', '\u{0435}', '\u{0444}', '\u{0433}',
    // 0x68: х и ј к л м н о
    '\u{0445}', '\u{0438}', '\u{0458}', '\u{043A}', '\u{043B}', '\u{043C}', '\u{043D}', '\u{043E}',
    // 0x70: п ч р с т у в щ
    '\u{043F}', '\u{0447}', '\u{0440}', '\u{0441}', '\u{0442}', '\u{0443}', '\u{0432}', '\u{0449}',
    // 0x78: ш ы з – — № ь ъ
    '\u{0448}', '\u{044B}', '\u{0437}', '\u{2013}', '\u{2014}', '\u{2116}', '\u{044C}', '\u{044A}',
    // 0x80: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0x88: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0x90: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0x98: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xA0: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xA8: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xB0: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xB8: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xC0: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xC8: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xD0: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xD8: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xE0: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xE8: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xF0: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None,
    // 0xF8: - - - - - - - -
    None,      None,      None,      None,      None,      None,      None,      None
  ];
  map
}

/// A run of characters of the ligature program, as the font typesets it: decoded back to its slots, ligated as TeX does
/// (tex.web §1035-1040: the current character and the next form the ligature, which is current again), decoded again.
fn ot2_ligature(run: &str, map: &[Option<char>], slots: &FxHashMap<char, u8>) -> String {
  let mut codes = run.chars().filter_map(|c| slots.get(&c).copied());
  let Some(mut current) = codes.next() else {
    return run.to_string();
  };
  let mut out = String::new();
  let emit = |code: u8, out: &mut String| {
    if let Some(Some(c)) = map.get(usize::from(code)) {
      out.push(*c);
    }
  };
  for next in codes {
    match LIGATURES
      .iter()
      .find(|(a, b, _)| *a == current && *b == next)
    {
      Some((_, _, ligature)) => current = *ligature,
      None => {
        emit(current, &mut out);
        current = next;
      },
    }
  }
  emit(current, &mut out);
  out
}

fn is_ot2(font: &Font) -> bool {
  font
    .get_encoding()
    .is_some_and(|encoding| encoding == "OT2")
}

LoadDefinitions!({
  let map = ot2_map();
  DeclareFontMap!("OT2", map.clone());
  let slots: FxHashMap<char, u8> = map
    .iter()
    .enumerate()
    .filter_map(|(code, c)| Some(((*c)?, u8::try_from(code).ok()?)))
    .collect();
  let alphabet: String = LIGATURES
    .iter()
    .flat_map(|(a, b, _)| [*a, *b])
    .filter(|code| !matches!(*code, b'-' | 123))
    .filter_map(|code| map[usize::from(code)])
    .collect::<std::collections::BTreeSet<char>>()
    .into_iter()
    .map(|c| regex::escape(&c.to_string()))
    .collect();
  DefLigature!(
    &format!("[{alphabet}]{{2,}}"),
    |caps: &regex::Captures| ot2_ligature(&caps[0], &map, &slots),
    fontTest => sub[font] { is_ot2(font) }
  );
});
