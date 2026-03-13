//! t2c font encoding
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("T2C", mixrc![
    // 0x00-0x07
    '\u{0060}', '\u{00B4}', '\u{02C6}', '\u{02DC}', '\u{00A8}', '\u{02DD}', '\u{02DA}', '\u{02C7}',
    // 0x08-0x0F
    '\u{02D8}', '\u{00AF}', '\u{02D9}', '\u{00B8}', '\u{02DB}', '\u{0406}', '\u{2039}', '\u{203A}',
    // 0x10-0x17
    // NOTE pos 0x12: Perl has UTF(0xA0)."\x{0311}" (NBSP + COMBINING INVERTED BREVE); using single char U+0311
    '\u{201C}', '\u{201D}', '\u{0311}', '\u{2036}', '\u{02D8}', '\u{2013}', '\u{2014}', '\u{200C}',
    // 0x18-0x1F
    '0', '\u{0131}', '\u{0237}', '\u{FB00}', '\u{FB01}', '\u{FB02}', '\u{FB03}', '\u{FB04}',
    // 0x20-0x27
    '\u{2423}', '!', '"', '#', '$', '%', '&', '\u{2019}',
    // 0x28-0x2F
    '(', ')', '*', '+', ',', '-', '.', '/',
    // 0x30-0x37
    '0', '1', '2', '3', '4', '5', '6', '7',
    // 0x38-0x3F
    '8', '9', ':', ';', '<', '=', '>', '?',
    // 0x40-0x47
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
    // 0x48-0x4F
    'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    // 0x50-0x57
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W',
    // 0x58-0x5F
    'X', 'Y', 'Z', '[', '\\', ']', '\u{02C6}', '\u{005F}',
    // 0x60-0x67
    '\u{2018}', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    // 0x68-0x6F
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    // 0x70-0x77
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w',
    // 0x78-0x7F
    'x', 'y', 'z', '{', '|', '}', '\u{02DC}', '\u{2010}',
    // 0x80-0x87
    // pos 0x85: multi-char "\x{0420}\x{0349}" handled by DeclareFontMapMultichar
    '\u{04A6}', '\u{04B4}', '\u{04AC}', '\u{0494}', '\u{04BA}', '\u{0420}', '\u{048E}', '\u{04E0}',
    // 0x88-0x8F
    // pos 0x8E: multi-char "[CYRMHK]" handled by DeclareFontMapMultichar
    '\u{04CD}', '\u{049A}', '\u{04C5}', '\u{049E}', '\u{0512}', '\u{04A2}', 'M', '\u{04C7}',
    // 0x90-0x97
    '\u{04E8}', '\u{04BC}', '\u{04BE}', '\u{048C}', '\u{048A}', '\u{04B2}', '\u{040F}', '\u{04A8}',
    // 0x98-0x9F
    // pos 0x99, 0x9B: multi-char placeholders handled by DeclareFontMapMultichar
    '\u{04B6}', 'N', '\u{04D8}', 'R', '\u{0401}', '\u{2116}', '\u{00A4}', '\u{00A7}',
    // 0xA0-0xA7
    // pos 0xA5: multi-char "\x{0440}\x{0349}" handled by DeclareFontMapMultichar
    '\u{04A7}', '\u{04B5}', '\u{04AD}', '\u{0495}', '\u{04BB}', '\u{0440}', '\u{048F}', '\u{04E1}',
    // 0xA8-0xAF
    // pos 0xAE: multi-char "[cyrmhk]" handled by DeclareFontMapMultichar
    '\u{04CE}', '\u{049B}', '\u{04C6}', '\u{049F}', '\u{0513}', '\u{04A3}', 'm', '\u{04C8}',
    // 0xB0-0xB7
    '\u{04E9}', '\u{04BD}', '\u{04BF}', '\u{048D}', '\u{048B}', '\u{04B3}', '\u{045F}', '\u{04A9}',
    // 0xB8-0xBF
    // pos 0xB9, 0xBB: multi-char placeholders handled by DeclareFontMapMultichar
    '\u{04B7}', 'n', '\u{04D9}', 'r', '\u{0451}', '\u{201E}', '\u{00AB}', '\u{00BB}',
    // 0xC0-0xC7: Cyrillic uppercase А-З
    '\u{0410}', '\u{0411}', '\u{0412}', '\u{0413}', '\u{0414}', '\u{0415}', '\u{0416}', '\u{0417}',
    // 0xC8-0xCF: Cyrillic uppercase И-П
    '\u{0418}', '\u{0419}', '\u{041A}', '\u{041B}', '\u{041C}', '\u{041D}', '\u{041E}', '\u{041F}',
    // 0xD0-0xD7: Cyrillic uppercase Р-Ч
    '\u{0420}', '\u{0421}', '\u{0422}', '\u{0423}', '\u{0424}', '\u{0425}', '\u{0426}', '\u{0427}',
    // 0xD8-0xDF: Cyrillic uppercase Ш-Я
    '\u{0428}', '\u{0429}', '\u{042A}', '\u{042B}', '\u{042C}', '\u{042D}', '\u{042E}', '\u{042F}',
    // 0xE0-0xE7: Cyrillic lowercase а-з
    '\u{0430}', '\u{0431}', '\u{0432}', '\u{0433}', '\u{0434}', '\u{0435}', '\u{0436}', '\u{0437}',
    // 0xE8-0xEF: Cyrillic lowercase и-п
    '\u{0438}', '\u{0439}', '\u{043A}', '\u{043B}', '\u{043C}', '\u{043D}', '\u{043E}', '\u{043F}',
    // 0xF0-0xF7: Cyrillic lowercase р-ч
    '\u{0440}', '\u{0441}', '\u{0442}', '\u{0443}', '\u{0444}', '\u{0445}', '\u{0446}', '\u{0447}',
    // 0xF8-0xFF: Cyrillic lowercase ш-я
    '\u{0448}', '\u{0449}', '\u{044A}', '\u{044B}', '\u{044C}', '\u{044D}', '\u{044E}', '\u{044F}'
  ]);
  // Multi-char overrides: base char + combining char or placeholder strings
  DeclareFontMapMultichar!("T2C", {
    0x85u8 => "\u{0420}\u{0349}",   // CYR P + COMBINING RIGHT HALF RING BELOW
    0x8Eu8 => "[CYRMHK]",           // No known Unicode codepoint
    0x99u8 => "[CYRNLHK]",          // No known Unicode codepoint
    0x9Bu8 => "[CYRRHK]",           // No known Unicode codepoint
    0xA5u8 => "\u{0440}\u{0349}",   // cyr er + COMBINING RIGHT HALF RING BELOW
    0xAEu8 => "[cyrmhk]",           // No known Unicode codepoint
    0xB9u8 => "[cyrnlhk]",          // No known Unicode codepoint
    0xBBu8 => "[cyrrhk]",           // No known Unicode codepoint
  });
});
