//! ts1 font encoding
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("TS1", mixrc![
    // 0x00-0x07: grave, acute, circumflex, tilde, diaeresis, double acute, ring, caron
    '\u{0060}', '\u{00B4}', '\u{005E}', '\u{007E}', '\u{00A8}', '\u{02DD}', '\u{02DA}', '\u{02C7}',
    // 0x08-0x0F: breve, macron, dot above, cedilla, ogonek, single low-9 quote, undef, undef
    '\u{02D8}', '\u{00AF}', '\u{02D9}', '\u{00B8}', '\u{02DB}', '\u{201A}', None, None,
    // 0x10-0x17: undef, undef, double quote, undef, undef, en dash, em dash, undef
    None, None, '"', None, None, '\u{2013}', '\u{2014}', None,
    // 0x18-0x1F: left arrow, right arrow, tie(combining double inverted breve), tie(same),
    //   inverted breve, inverted breve, undef, undef
    // Perl: positions 26-27 are UTF(0xA0)."\x{0361}" (NBSP + COMBINING DOUBLE INVERTED BREVE)
    // Perl: positions 28-29 are UTF(0xA0)."\x{0311}" (NBSP + COMBINING INVERTED BREVE)
    // Rust fontmap is Option<char>, so we store only the combining character.
    '\u{2190}', '\u{2192}', '\u{0361}', '\u{0361}', '\u{0311}', '\u{0311}', None, None,
    // 0x20-0x27: blank symbol, undef, undef, undef, dollar, undef, undef, apostrophe
    '\u{2422}', None, None, None, '$', None, None, '\'',
    // 0x28-0x2F: undef, undef, asterism, undef, comma, equals, period, fraction slash
    None, None, '\u{204E}', None, ',', '=', '.', '\u{2044}',
    // 0x30-0x37: 0-7
    '0', '1', '2', '3', '4', '5', '6', '7',
    // 0x38-0x3F: 8, 9, undef, undef, left angle bracket, hyphen, right angle bracket, undef
    '8', '9', None, None, '\u{2329}', '-', '\u{232A}', None,
    // 0x40-0x47: undef x8
    None, None, None, None, None, None, None, None,
    // 0x48-0x4F: undef, undef, undef, undef, undef, mho, undef, large circle
    None, None, None, None, None, '\u{2127}', None, '\u{25EF}',
    // 0x50-0x57: undef x8
    None, None, None, None, None, None, None, '\u{2126}',
    // 0x58-0x5F: undef, undef, undef, left white square bracket, undef, right white square bracket, up arrow, down arrow
    None, None, None, '\u{27E6}', None, '\u{27E7}', '\u{2191}', '\u{2193}',
    // 0x60-0x67: grave, undef, star operator, unmarried partnership, latin cross, undef, undef, undef
    '\u{0060}', None, '\u{22C6}', '\u{26AE}', '\u{271D}', None, None, None,
    // 0x68-0x6F: undef, undef, undef, undef, undef, marriage symbol, music note, undef
    None, None, None, None, None, '\u{26AD}', '\u{266A}', None,
    // 0x70-0x77: undef x8
    None, None, None, None, None, None, None, None,
    // 0x78-0x7F: undef, undef, undef, undef, undef, undef, tilde, equals
    None, None, None, None, None, None, '~', '=',
    // 0x80-0x87: breve, caron, double acute, combining double grave (Perl: NBSP+\x{030F}), dagger, double dagger, double vertical line, per mille
    // Perl: position 131 is UTF(0xA0)."\x{030F}" (NBSP + COMBINING DOUBLE GRAVE ACCENT)
    // Rust fontmap is Option<char>, so we store only the combining character.
    '\u{02D8}', '\u{02C7}', '\u{02DD}', '\u{030F}', '\u{2020}', '\u{2021}', '\u{2016}', '\u{2030}',
    // 0x88-0x8F: bullet, degree celsius, dollar, cent, latin small f with hook, colon sign, won, naira
    '\u{2022}', '\u{2103}', '$', '\u{00A2}', '\u{0192}', '\u{20A1}', '\u{20A9}', '\u{20A6}',
    // 0x90-0x97: guarani sign (Perl: "G\x{20D2}", G + combining long vertical line overlay),
    //   peso, lira, prescription, interrobang, gnaborretni, dong, trademark
    // pos 0x90: multi-char "G\x{20D2}" handled by DeclareFontMapMultichar
    'G', '\u{20B1}', '\u{20A4}', '\u{211E}', '\u{203D}', '\u{2E18}', '\u{20AB}', '\u{2122}',
    // 0x98-0x9F: per ten thousand, pilcrow, baht, numero, discount percentage, estimated, white bullet, service mark
    '\u{2031}', '\u{00B6}', '\u{0E3F}', '\u{2116}', '\u{2052}', '\u{212E}', '\u{25E6}', '\u{2120}',
    // 0xA0-0xA7: left square bracket with quill, right square bracket with quill, cent, pound, currency, yen, broken bar, section
    '\u{2045}', '\u{2046}', '\u{00A2}', '\u{00A3}', '\u{00A4}', '\u{00A5}', '\u{00A6}', '\u{00A7}',
    // 0xA8-0xAF: diaeresis, copyright, feminine ordinal, undef, not, sound recording copyright, registered, macron
    '\u{00A8}', '\u{00A9}', '\u{00AA}', None, '\u{00AC}', '\u{2117}', '\u{00AE}', '\u{00AF}',
    // 0xB0-0xB7: degree, plus-minus, superscript 2, superscript 3, acute, micro, pilcrow, middle dot
    '\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00B3}', '\u{00B4}', '\u{00B5}', '\u{00B6}', '\u{00B7}',
    // 0xB8-0xBF: reference mark, superscript 1, masculine ordinal, square root, vulgar 1/4, vulgar 1/2, vulgar 3/4, euro
    '\u{203B}', '\u{00B9}', '\u{00BA}', '\u{221A}', '\u{00BC}', '\u{00BD}', '\u{00BE}', '\u{20AC}',
    // 0xC0-0xC7: undef x8
    None, None, None, None, None, None, None, None,
    // 0xC8-0xCF: undef x8
    None, None, None, None, None, None, None, None,
    // 0xD0-0xD7: undef x6, multiplication, undef
    None, None, None, None, None, None, '\u{00D7}', None,
    // 0xD8-0xDF: undef x8
    None, None, None, None, None, None, None, None,
    // 0xE0-0xE7: undef x8
    None, None, None, None, None, None, None, None,
    // 0xE8-0xEF: undef x8
    None, None, None, None, None, None, None, None,
    // 0xF0-0xF7: undef x6, division, undef
    None, None, None, None, None, None, '\u{00F7}', None,
    // 0xF8-0xFF: undef x8
    None, None, None, None, None, None, None, None
  ]);
  // Multi-char overrides: base char + combining char
  DeclareFontMapMultichar!("TS1", {
    0x90u8 => "G\u{20D2}",   // G + COMBINING LONG VERTICAL LINE OVERLAY (guarani sign)
  });
});
