//! OT4 font encoding (from ot4.fontmap.ltxml)
//! Polish extension of OT1 encoding.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("OT4", mixrc![
    // 0x00-0x07
    '\u{0393}', '\u{0394}', '\u{0398}', '\u{039B}', '\u{039E}', '\u{03A0}', '\u{03A3}', '\u{03A5}',
    // 0x08-0x0F
    '\u{03A6}', '\u{03A8}', '\u{03A9}', '\u{FB00}', '\u{FB01}', '\u{FB02}', '\u{FB03}', '\u{FB04}',
    // 0x10-0x17: Perl "\x{}" at 0x14 is empty/null — treated as None
    '\u{0131}', None,       '\u{0060}', '\u{00B4}', None,       '\u{02D8}', '\u{00AF}', '\u{02DA}',
    // 0x18-0x1F
    '\u{00B8}', '\u{00DF}', '\u{00E6}', '\u{0153}', '\u{00F8}', '\u{00C6}', '\u{0152}', '\u{00D8}',
    // 0x20-0x27
    None,       '!',        '\u{201D}', '#',        '$',        '%',        '&',        '\u{2018}',
    // 0x28-0x2F
    '(',        ')',        '*',        '+',        ',',        '-',        '.',        '/',
    // 0x30-0x37
    '0',        '1',        '2',        '3',        '4',        '5',        '6',        '7',
    // 0x38-0x3F
    '8',        '9',        ':',        ';',        '\u{00A1}', '=',        '\u{00BF}', '?',
    // 0x40-0x47
    '@',        'A',        'B',        'C',        'D',        'E',        'F',        'G',
    // 0x48-0x4F
    'H',        'I',        'J',        'K',        'L',        'M',        'N',        'O',
    // 0x50-0x57
    'P',        'Q',        'R',        'S',        'T',        'U',        'V',        'W',
    // 0x58-0x5F
    'X',        'Y',        'Z',        '[',        '\u{201C}', ']',        '\u{005F}', '\u{02D9}',
    // 0x60-0x67
    '\u{2019}', 'a',        'b',        'c',        'd',        'e',        'f',        'g',
    // 0x68-0x6F
    'h',        'i',        'j',        'k',        'l',        'm',        'n',        'o',
    // 0x70-0x77
    'p',        'q',        'r',        's',        't',        'u',        'v',        'w',
    // 0x78-0x7F
    'x',        'y',        'z',        '\u{2013}', '\u{2014}', '\u{02DD}', '\u{007E}', '\u{00A8}',

    // 0x80-0x87: Polish characters
    None,       '\u{0104}', '\u{0106}', None,       None,       None,       '\u{0118}', None,
    // 0x88-0x8F
    None,       None,       '\u{0141}', '\u{0143}', None,       None,       None,       None,
    // 0x90-0x97
    None,       '\u{015A}', None,       None,       None,       None,       None,       None,
    // 0x98-0x9F
    None,       '\u{0179}', None,       '\u{017B}', None,       None,       None,       None,
    // 0xA0-0xA7
    None,       '\u{0105}', '\u{0107}', None,       None,       None,       '\u{0119}', None,
    // 0xA8-0xAF
    None,       None,       '\u{0142}', '\u{0144}', None,       None,       '\u{00AB}', '\u{00BB}',
    // 0xB0-0xB7
    None,       '\u{015B}', None,       None,       None,       None,       None,       None,
    // 0xB8-0xBF
    None,       '\u{017A}', None,       '\u{017C}', None,       None,       None,       None,
    // 0xC0-0xC7
    None,       None,       None,       None,       None,       None,       None,       None,
    // 0xC8-0xCF
    None,       None,       None,       None,       None,       None,       None,       None,
    // 0xD0-0xD7
    None,       None,       None,       '\u{00D3}', None,       None,       None,       None,
    // 0xD8-0xDF
    None,       None,       None,       None,       None,       None,       None,       None,
    // 0xE0-0xE7
    None,       None,       None,       None,       None,       None,       None,       None,
    // 0xE8-0xEF
    None,       None,       None,       None,       None,       None,       None,       None,
    // 0xF0-0xF7
    None,       None,       None,       '\u{00F3}', None,       None,       None,       None,
    // 0xF8-0xFF
    None,       None,       None,       None,       None,       None,       None,       '\u{201E}'
  ]);
});
