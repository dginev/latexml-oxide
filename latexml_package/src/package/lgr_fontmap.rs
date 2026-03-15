//! LGR font encoding (from lgr.fontmap.ltxml)
//! Greek font encoding with ligatures for polytonic accents.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("LGR", mixrc![
    // 0x00-0x07
    '\u{2013}', None,       None,       None,       None,       None,       '\u{03DA}', '\u{03DB}',
    // 0x08-0x0F
    '\u{1FBE}', '\u{1FBC}', '\u{1FCC}', '\u{1FFC}', '\u{0391}', '\u{03D4}', '\u{03B1}', '\u{1FE7}',
    // 0x10-0x17
    None,       None,       '\u{03DE}', '\u{03D9}', None,       '\u{03DA}', None,       '\u{03E0}',
    // 0x18-0x1F
    '\u{20AC}', '\u{2030}', '\u{018F}', '\u{03E1}', '\u{2018}', '\u{2019}', '\u{02D8}', '\u{00AF}',
    // 0x20-0x27
    '\u{1FC1}', '!',        '\u{1FBD}', '\u{1FEE}', '\u{1FED}', '%',        '.',        '\u{1FFD}',
    // 0x28-0x2F
    '(',        ')',        '*',        '+',        ',',        '-',        '.',        '/',
    // 0x30-0x37
    '0',        '1',        '2',        '3',        '4',        '5',        '6',        '7',
    // 0x38-0x3F
    '8',        '9',        ':',        '\u{0387}', '\u{201B}', '=',        '\u{2019}', ';',

    // 0x40-0x47
    '\u{1FDF}', '\u{0391}', '\u{0392}', '\u{1FDD}', '\u{0394}', '\u{0395}', '\u{03A6}', '\u{0393}',
    // 0x48-0x4F
    '\u{0397}', '\u{0399}', '\u{0398}', '\u{039A}', '\u{039B}', '\u{039C}', '\u{039D}', '\u{039F}',
    // 0x50-0x57
    '\u{03A0}', '\u{03A7}', '\u{03A1}', '\u{03A3}', '\u{03A4}', '\u{03D2}', '\u{1FDE}', '\u{03A9}',
    // 0x58-0x5F
    '\u{039E}', '\u{03A8}', '\u{0396}', '[',        '\u{1FCF}', ']',        '\u{1FCE}', '\u{1FCD}',
    // 0x60-0x67
    '\u{1FEF}', '\u{03B1}', '\u{03B2}', '\u{03C2}', '\u{03B4}', '\u{03B5}', '\u{03C6}', '\u{03B3}',
    // 0x68-0x6F
    '\u{03B7}', '\u{03B9}', '\u{03D1}', '\u{03F0}', '\u{03BB}', '\u{03BC}', '\u{03BD}', '\u{03BF}',
    // 0x70-0x77
    '\u{03C0}', '\u{03C7}', '\u{03C1}', '\u{03C2}', '\u{03C4}', '\u{03C5}', None,       '\u{03C9}',
    // 0x78-0x7F
    '\u{03BE}', '\u{03C8}', '\u{03B6}', '\u{00AB}', '\u{1FBE}', '\u{00BB}', '\u{1FC0}', '\u{2014}',

    // 0x80-0x87
    '\u{1F70}', '\u{1F01}', '\u{1F00}', '\u{1F03}', '\u{1FB2}', '\u{1F81}', '\u{1F80}', '\u{1F83}',
    // 0x88-0x8F
    '\u{1F71}', '\u{1F05}', '\u{1F04}', '\u{1F02}', '\u{1FB4}', '\u{1F85}', '\u{1F84}', '\u{1F82}',
    // 0x90-0x97
    '\u{1FB6}', '\u{1F07}', '\u{1F06}', '\u{03DD}', '\u{1FB7}', '\u{1F87}', '\u{1F86}', None,
    // 0x98-0x9F
    '\u{1F74}', '\u{1F21}', '\u{1F20}', None,       '\u{1FC2}', '\u{1F91}', '\u{1F90}', None,
    // 0xA0-0xA7
    '\u{1F75}', '\u{1F25}', '\u{1F24}', '\u{1F23}', '\u{1FC4}', '\u{1F95}', '\u{1F94}', '\u{1F93}',
    // 0xA8-0xAF
    '\u{1FC6}', '\u{1F27}', '\u{1F26}', '\u{1F22}', '\u{1FC7}', '\u{1F97}', '\u{1F96}', '\u{1F92}',
    // 0xB0-0xB7
    '\u{1F7C}', '\u{1F61}', '\u{1F60}', '\u{1F63}', '\u{1FF2}', '\u{1FA1}', '\u{1FA0}', '\u{1FA3}',
    // 0xB8-0xBF
    '\u{1F7C}', '\u{1F65}', '\u{1F64}', '\u{1F62}', '\u{1FF4}', '\u{1FA5}', '\u{1FA4}', '\u{1FA2}',

    // 0xC0-0xC7
    '\u{1FF6}', '\u{1F67}', '\u{1F66}', '\u{03DC}', '\u{1FF7}', '\u{1FA7}', '\u{1FA6}', None,
    // 0xC8-0xCF
    '\u{1F76}', '\u{1F31}', '\u{1F30}', '\u{1F33}', '\u{1F7A}', '\u{1F51}', '\u{1F50}', '\u{1F53}',
    // 0xD0-0xD7
    '\u{1F77}', '\u{1F35}', '\u{1F34}', '\u{1F32}', '\u{1F7B}', '\u{1F55}', '\u{1F54}', '\u{1F52}',
    // 0xD8-0xDF
    '\u{1FD6}', '\u{1F37}', '\u{1F36}', '\u{03AA}', '\u{1FE6}', '\u{1F57}', '\u{1F56}', '\u{03D4}',
    // 0xE0-0xE7
    '\u{1F72}', '\u{1F11}', '\u{1F10}', '\u{1F13}', '\u{1F78}', '\u{1F41}', '\u{1F40}', '\u{1F43}',
    // 0xE8-0xEF
    '\u{1F73}', '\u{1F15}', '\u{1F14}', '\u{1F12}', '\u{1F79}', '\u{1F45}', '\u{1F44}', '\u{1F42}',
    // 0xF0-0xF7
    '\u{03CA}', '\u{1FD2}', '\u{1FD3}', '\u{1FD7}', '\u{03CB}', '\u{1FE2}', '\u{1FE3}', '\u{1FE7}',
    // 0xF8-0xFF
    '\u{1FB3}', '\u{1FC3}', '\u{1FF3}', '\u{1FE5}', '\u{1FE4}', None,       '\u{0374}', '\u{0375}'
  ]);

  // Greek polytonic accent ligatures.
  // These map sequences of LGR-encoded characters (accents + base letters) to precomposed forms.
  // Generated from the Perl ligature computation in lgr.fontmap.ltxml.
  // Sorted by length then lexicographically, matching Perl's sort order.

  // 2-char ligatures
  DefLigature!("\u{03B1}\u{1FBE}", "\u{1FB3}");
  DefLigature!("\u{03B7}\u{1FBE}", "\u{1FC3}");
  DefLigature!("\u{03C9}\u{1FBE}", "\u{1FF3}");
  DefLigature!("\u{1FBD}\u{0399}", "\u{03AA}");
  DefLigature!("\u{1FBD}\u{03B9}", "\u{03CA}");
  DefLigature!("\u{1FBD}\u{03C5}", "\u{03CB}");
  DefLigature!("\u{1FBD}\u{03D2}", "\u{03D4}");
  DefLigature!("\u{1FBD}\u{1FC0}", "\u{1FC1}");
  DefLigature!("\u{1FBD}\u{1FEF}", "\u{1FED}");
  DefLigature!("\u{1FBD}\u{1FFD}", "\u{1FEE}");
  DefLigature!("\u{1FC0}\u{03B1}", "\u{1FB6}");
  DefLigature!("\u{1FC0}\u{03B7}", "\u{1FC6}");
  DefLigature!("\u{1FC0}\u{03B9}", "\u{1FD6}");
  DefLigature!("\u{1FC0}\u{03C9}", "\u{1FF6}");
  DefLigature!("\u{1FEF}\u{03B1}", "\u{1F70}");
  DefLigature!("\u{1FEF}\u{03B5}", "\u{1F72}");
  DefLigature!("\u{1FEF}\u{03B7}", "\u{1F74}");
  DefLigature!("\u{1FEF}\u{03B9}", "\u{1F76}");
  DefLigature!("\u{1FEF}\u{03C9}", "\u{1F7C}");
  DefLigature!("\u{1FFD}\u{03B1}", "\u{1F71}");
  DefLigature!("\u{1FFD}\u{03B5}", "\u{1F73}");
  DefLigature!("\u{1FFD}\u{03B7}", "\u{1F75}");
  DefLigature!("\u{1FFD}\u{03B9}", "\u{1F77}");
  DefLigature!("\u{1FFD}\u{03C9}", "\u{1F7C}");
  DefLigature!("\u{2019}\u{03B1}", "\u{1F00}");
  DefLigature!("\u{2019}\u{03B5}", "\u{1F10}");
  DefLigature!("\u{2019}\u{03B7}", "\u{1F20}");
  DefLigature!("\u{2019}\u{03B9}", "\u{1F30}");
  DefLigature!("\u{2019}\u{03C1}", "\u{1FE4}");
  DefLigature!("\u{2019}\u{03C9}", "\u{1F60}");
  DefLigature!("\u{2019}\u{1FC0}", "\u{1FCF}");
  DefLigature!("\u{2019}\u{1FEF}", "\u{1FCD}");
  DefLigature!("\u{2019}\u{1FFD}", "\u{1FCE}");
  DefLigature!("\u{201B}\u{03B1}", "\u{1F01}");
  DefLigature!("\u{201B}\u{03B5}", "\u{1F11}");
  DefLigature!("\u{201B}\u{03B7}", "\u{1F21}");
  DefLigature!("\u{201B}\u{03B9}", "\u{1F31}");
  DefLigature!("\u{201B}\u{03C1}", "\u{1FE5}");
  DefLigature!("\u{201B}\u{03C9}", "\u{1F61}");
  DefLigature!("\u{201B}\u{1FC0}", "\u{1FDF}");
  DefLigature!("\u{201B}\u{1FEF}", "\u{1FDD}");
  DefLigature!("\u{201B}\u{1FFD}", "\u{1FDE}");

  // 3-char ligatures
  DefLigature!("\u{1FBD}\u{1FC0}\u{03B9}", "\u{1FD7}");
  DefLigature!("\u{1FBD}\u{1FC0}\u{03C5}", "\u{1FE7}");
  DefLigature!("\u{1FBD}\u{1FEF}\u{03B9}", "\u{1FD2}");
  DefLigature!("\u{1FBD}\u{1FEF}\u{03C5}", "\u{1FE2}");
  DefLigature!("\u{1FBD}\u{1FFD}\u{03B9}", "\u{1FD3}");
  DefLigature!("\u{1FBD}\u{1FFD}\u{03C5}", "\u{1FE3}");
  DefLigature!("\u{1FC0}\u{03B1}\u{1FBE}", "\u{1FB7}");
  DefLigature!("\u{1FC0}\u{03B7}\u{1FBE}", "\u{1FC7}");
  DefLigature!("\u{1FC0}\u{03C5}\u{1FBE}", "\u{1FE6}");
  DefLigature!("\u{1FC0}\u{03C9}\u{1FBE}", "\u{1FF7}");
  DefLigature!("\u{1FC0}\u{1FBD}\u{03B9}", "\u{1FD7}");
  DefLigature!("\u{1FC0}\u{1FBD}\u{03C5}", "\u{1FE7}");
  DefLigature!("\u{1FC0}\u{2019}\u{03B1}", "\u{1F06}");
  DefLigature!("\u{1FC0}\u{2019}\u{03B7}", "\u{1F26}");
  DefLigature!("\u{1FC0}\u{2019}\u{03B9}", "\u{1F36}");
  DefLigature!("\u{1FC0}\u{2019}\u{03C9}", "\u{1F66}");
  DefLigature!("\u{1FC0}\u{201B}\u{03B1}", "\u{1F07}");
  DefLigature!("\u{1FC0}\u{201B}\u{03B7}", "\u{1F27}");
  DefLigature!("\u{1FC0}\u{201B}\u{03B9}", "\u{1F37}");
  DefLigature!("\u{1FC0}\u{201B}\u{03C9}", "\u{1F67}");
  DefLigature!("\u{1FEF}\u{03B1}\u{1FBE}", "\u{1FB2}");
  DefLigature!("\u{1FEF}\u{03B7}\u{1FBE}", "\u{1FC2}");
  DefLigature!("\u{1FEF}\u{03BF}\u{1FBE}", "\u{1F78}");
  DefLigature!("\u{1FEF}\u{03C5}\u{1FBE}", "\u{1F7A}");
  DefLigature!("\u{1FEF}\u{03C9}\u{1FBE}", "\u{1FF2}");
  DefLigature!("\u{1FEF}\u{1FBD}\u{03B9}", "\u{1FD2}");
  DefLigature!("\u{1FEF}\u{1FBD}\u{03C5}", "\u{1FE2}");
  DefLigature!("\u{1FEF}\u{2019}\u{03B1}", "\u{1F02}");
  DefLigature!("\u{1FEF}\u{2019}\u{03B5}", "\u{1F12}");
  DefLigature!("\u{1FEF}\u{2019}\u{03B7}", "\u{1F22}");
  DefLigature!("\u{1FEF}\u{2019}\u{03B9}", "\u{1F32}");
  DefLigature!("\u{1FEF}\u{2019}\u{03C9}", "\u{1F62}");
  DefLigature!("\u{1FEF}\u{201B}\u{03B1}", "\u{1F03}");
  DefLigature!("\u{1FEF}\u{201B}\u{03B5}", "\u{1F13}");
  DefLigature!("\u{1FEF}\u{201B}\u{03B7}", "\u{1F23}");
  DefLigature!("\u{1FEF}\u{201B}\u{03B9}", "\u{1F33}");
  DefLigature!("\u{1FEF}\u{201B}\u{03C9}", "\u{1F63}");
  DefLigature!("\u{1FFD}\u{03B1}\u{1FBE}", "\u{1FB4}");
  DefLigature!("\u{1FFD}\u{03B7}\u{1FBE}", "\u{1FC4}");
  DefLigature!("\u{1FFD}\u{03BF}\u{1FBE}", "\u{1F79}");
  DefLigature!("\u{1FFD}\u{03C5}\u{1FBE}", "\u{1F7B}");
  DefLigature!("\u{1FFD}\u{03C9}\u{1FBE}", "\u{1FF4}");
  DefLigature!("\u{1FFD}\u{1FBD}\u{03B9}", "\u{1FD3}");
  DefLigature!("\u{1FFD}\u{1FBD}\u{03C5}", "\u{1FE3}");
  DefLigature!("\u{1FFD}\u{2019}\u{03B1}", "\u{1F04}");
  DefLigature!("\u{1FFD}\u{2019}\u{03B5}", "\u{1F14}");
  DefLigature!("\u{1FFD}\u{2019}\u{03B7}", "\u{1F24}");
  DefLigature!("\u{1FFD}\u{2019}\u{03B9}", "\u{1F34}");
  DefLigature!("\u{1FFD}\u{2019}\u{03C9}", "\u{1F64}");
  DefLigature!("\u{1FFD}\u{201B}\u{03B1}", "\u{1F05}");
  DefLigature!("\u{1FFD}\u{201B}\u{03B5}", "\u{1F15}");
  DefLigature!("\u{1FFD}\u{201B}\u{03B7}", "\u{1F25}");
  DefLigature!("\u{1FFD}\u{201B}\u{03B9}", "\u{1F35}");
  DefLigature!("\u{1FFD}\u{201B}\u{03C9}", "\u{1F65}");
  DefLigature!("\u{2019}\u{03B1}\u{1FBE}", "\u{1F80}");
  DefLigature!("\u{2019}\u{03B7}\u{1FBE}", "\u{1F90}");
  DefLigature!("\u{2019}\u{03BF}\u{1FBE}", "\u{1F40}");
  DefLigature!("\u{2019}\u{03C5}\u{1FBE}", "\u{1F50}");
  DefLigature!("\u{2019}\u{03C9}\u{1FBE}", "\u{1FA0}");
  DefLigature!("\u{2019}\u{1FC0}\u{03B1}", "\u{1F06}");
  DefLigature!("\u{2019}\u{1FC0}\u{03B7}", "\u{1F26}");
  DefLigature!("\u{2019}\u{1FC0}\u{03B9}", "\u{1F36}");
  DefLigature!("\u{2019}\u{1FC0}\u{03C9}", "\u{1F66}");
  DefLigature!("\u{2019}\u{1FEF}\u{03B1}", "\u{1F02}");
  DefLigature!("\u{2019}\u{1FEF}\u{03B5}", "\u{1F12}");
  DefLigature!("\u{2019}\u{1FEF}\u{03B7}", "\u{1F22}");
  DefLigature!("\u{2019}\u{1FEF}\u{03B9}", "\u{1F32}");
  DefLigature!("\u{2019}\u{1FEF}\u{03C9}", "\u{1F62}");
  DefLigature!("\u{2019}\u{1FFD}\u{03B1}", "\u{1F04}");
  DefLigature!("\u{2019}\u{1FFD}\u{03B5}", "\u{1F14}");
  DefLigature!("\u{2019}\u{1FFD}\u{03B7}", "\u{1F24}");
  DefLigature!("\u{2019}\u{1FFD}\u{03B9}", "\u{1F34}");
  DefLigature!("\u{2019}\u{1FFD}\u{03C9}", "\u{1F64}");
  DefLigature!("\u{201B}\u{03B1}\u{1FBE}", "\u{1F81}");
  DefLigature!("\u{201B}\u{03B7}\u{1FBE}", "\u{1F91}");
  DefLigature!("\u{201B}\u{03BF}\u{1FBE}", "\u{1F41}");
  DefLigature!("\u{201B}\u{03C5}\u{1FBE}", "\u{1F51}");
  DefLigature!("\u{201B}\u{03C9}\u{1FBE}", "\u{1FA1}");
  DefLigature!("\u{201B}\u{1FC0}\u{03B1}", "\u{1F07}");
  DefLigature!("\u{201B}\u{1FC0}\u{03B7}", "\u{1F27}");
  DefLigature!("\u{201B}\u{1FC0}\u{03B9}", "\u{1F37}");
  DefLigature!("\u{201B}\u{1FC0}\u{03C9}", "\u{1F67}");
  DefLigature!("\u{201B}\u{1FEF}\u{03B1}", "\u{1F03}");
  DefLigature!("\u{201B}\u{1FEF}\u{03B5}", "\u{1F13}");
  DefLigature!("\u{201B}\u{1FEF}\u{03B7}", "\u{1F23}");
  DefLigature!("\u{201B}\u{1FEF}\u{03B9}", "\u{1F33}");
  DefLigature!("\u{201B}\u{1FEF}\u{03C9}", "\u{1F63}");
  DefLigature!("\u{201B}\u{1FFD}\u{03B1}", "\u{1F05}");
  DefLigature!("\u{201B}\u{1FFD}\u{03B5}", "\u{1F15}");
  DefLigature!("\u{201B}\u{1FFD}\u{03B7}", "\u{1F25}");
  DefLigature!("\u{201B}\u{1FFD}\u{03B9}", "\u{1F35}");
  DefLigature!("\u{201B}\u{1FFD}\u{03C9}", "\u{1F65}");

  // 4-char ligatures
  DefLigature!("\u{1FC0}\u{2019}\u{03B1}\u{1FBE}", "\u{1F86}");
  DefLigature!("\u{1FC0}\u{2019}\u{03B7}\u{1FBE}", "\u{1F96}");
  DefLigature!("\u{1FC0}\u{2019}\u{03C5}\u{1FBE}", "\u{1F56}");
  DefLigature!("\u{1FC0}\u{2019}\u{03C9}\u{1FBE}", "\u{1FA6}");
  DefLigature!("\u{1FC0}\u{201B}\u{03B1}\u{1FBE}", "\u{1F87}");
  DefLigature!("\u{1FC0}\u{201B}\u{03B7}\u{1FBE}", "\u{1F97}");
  DefLigature!("\u{1FC0}\u{201B}\u{03C5}\u{1FBE}", "\u{1F57}");
  DefLigature!("\u{1FC0}\u{201B}\u{03C9}\u{1FBE}", "\u{1FA7}");
  DefLigature!("\u{1FEF}\u{2019}\u{03B1}\u{1FBE}", "\u{1F82}");
  DefLigature!("\u{1FEF}\u{2019}\u{03B7}\u{1FBE}", "\u{1F92}");
  DefLigature!("\u{1FEF}\u{2019}\u{03BF}\u{1FBE}", "\u{1F42}");
  DefLigature!("\u{1FEF}\u{2019}\u{03C5}\u{1FBE}", "\u{1F52}");
  DefLigature!("\u{1FEF}\u{2019}\u{03C9}\u{1FBE}", "\u{1FA2}");
  DefLigature!("\u{1FEF}\u{201B}\u{03B1}\u{1FBE}", "\u{1F83}");
  DefLigature!("\u{1FEF}\u{201B}\u{03B7}\u{1FBE}", "\u{1F93}");
  DefLigature!("\u{1FEF}\u{201B}\u{03BF}\u{1FBE}", "\u{1F43}");
  DefLigature!("\u{1FEF}\u{201B}\u{03C5}\u{1FBE}", "\u{1F53}");
  DefLigature!("\u{1FEF}\u{201B}\u{03C9}\u{1FBE}", "\u{1FA3}");
  DefLigature!("\u{1FFD}\u{2019}\u{03B1}\u{1FBE}", "\u{1F84}");
  DefLigature!("\u{1FFD}\u{2019}\u{03B7}\u{1FBE}", "\u{1F94}");
  DefLigature!("\u{1FFD}\u{2019}\u{03BF}\u{1FBE}", "\u{1F44}");
  DefLigature!("\u{1FFD}\u{2019}\u{03C5}\u{1FBE}", "\u{1F54}");
  DefLigature!("\u{1FFD}\u{2019}\u{03C9}\u{1FBE}", "\u{1FA4}");
  DefLigature!("\u{1FFD}\u{201B}\u{03B1}\u{1FBE}", "\u{1F85}");
  DefLigature!("\u{1FFD}\u{201B}\u{03B7}\u{1FBE}", "\u{1F95}");
  DefLigature!("\u{1FFD}\u{201B}\u{03BF}\u{1FBE}", "\u{1F45}");
  DefLigature!("\u{1FFD}\u{201B}\u{03C5}\u{1FBE}", "\u{1F55}");
  DefLigature!("\u{1FFD}\u{201B}\u{03C9}\u{1FBE}", "\u{1FA5}");
  DefLigature!("\u{2019}\u{1FC0}\u{03B1}\u{1FBE}", "\u{1F86}");
  DefLigature!("\u{2019}\u{1FC0}\u{03B7}\u{1FBE}", "\u{1F96}");
  DefLigature!("\u{2019}\u{1FC0}\u{03C5}\u{1FBE}", "\u{1F56}");
  DefLigature!("\u{2019}\u{1FC0}\u{03C9}\u{1FBE}", "\u{1FA6}");
  DefLigature!("\u{2019}\u{1FEF}\u{03B1}\u{1FBE}", "\u{1F82}");
  DefLigature!("\u{2019}\u{1FEF}\u{03B7}\u{1FBE}", "\u{1F92}");
  DefLigature!("\u{2019}\u{1FEF}\u{03BF}\u{1FBE}", "\u{1F42}");
  DefLigature!("\u{2019}\u{1FEF}\u{03C5}\u{1FBE}", "\u{1F52}");
  DefLigature!("\u{2019}\u{1FEF}\u{03C9}\u{1FBE}", "\u{1FA2}");
  DefLigature!("\u{2019}\u{1FFD}\u{03B1}\u{1FBE}", "\u{1F84}");
  DefLigature!("\u{2019}\u{1FFD}\u{03B7}\u{1FBE}", "\u{1F94}");
  DefLigature!("\u{2019}\u{1FFD}\u{03BF}\u{1FBE}", "\u{1F44}");
  DefLigature!("\u{2019}\u{1FFD}\u{03C5}\u{1FBE}", "\u{1F54}");
  DefLigature!("\u{2019}\u{1FFD}\u{03C9}\u{1FBE}", "\u{1FA4}");
  DefLigature!("\u{201B}\u{1FC0}\u{03B1}\u{1FBE}", "\u{1F87}");
  DefLigature!("\u{201B}\u{1FC0}\u{03B7}\u{1FBE}", "\u{1F97}");
  DefLigature!("\u{201B}\u{1FC0}\u{03C5}\u{1FBE}", "\u{1F57}");
  DefLigature!("\u{201B}\u{1FC0}\u{03C9}\u{1FBE}", "\u{1FA7}");
  DefLigature!("\u{201B}\u{1FEF}\u{03B1}\u{1FBE}", "\u{1F83}");
  DefLigature!("\u{201B}\u{1FEF}\u{03B7}\u{1FBE}", "\u{1F93}");
  DefLigature!("\u{201B}\u{1FEF}\u{03BF}\u{1FBE}", "\u{1F43}");
  DefLigature!("\u{201B}\u{1FEF}\u{03C5}\u{1FBE}", "\u{1F53}");
  DefLigature!("\u{201B}\u{1FEF}\u{03C9}\u{1FBE}", "\u{1FA3}");
  DefLigature!("\u{201B}\u{1FFD}\u{03B1}\u{1FBE}", "\u{1F85}");
  DefLigature!("\u{201B}\u{1FFD}\u{03B7}\u{1FBE}", "\u{1F95}");
  DefLigature!("\u{201B}\u{1FFD}\u{03BF}\u{1FBE}", "\u{1F45}");
  DefLigature!("\u{201B}\u{1FFD}\u{03C5}\u{1FBE}", "\u{1F55}");
  DefLigature!("\u{201B}\u{1FFD}\u{03C9}\u{1FBE}", "\u{1FA5}");
});
