//! ifclk font encoding (from ifclk.fontmap.ltxml)
//! Clock face symbols.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("ifclk", mixrc![
    // 12:00 and :30
    '\u{1F55B}', None, None, None, None, None,
    '\u{1F567}', None, None, None, None, None,
    // 1:00 and :30
    '\u{1F550}', None, None, None, None, None,
    '\u{1F55C}', None, None, None, None, None,
    // 2:00 and :30
    '\u{1F551}', None, None, None, None, None,
    '\u{1F55D}', None, None, None, None, None,
    // 3:00 and :30
    '\u{1F552}', None, None, None, None, None,
    '\u{1F55E}', None, None, None, None, None,
    // 4:00 and :30
    '\u{1F553}', None, None, None, None, None,
    '\u{1F55F}', None, None, None, None, None,
    // 5:00 and :30
    '\u{1F554}', None, None, None, None, None,
    '\u{1F560}', None, None, None, None, None,
    // 6:00 and :30
    '\u{1F555}', None, None, None, None, None,
    '\u{1F561}', None, None, None, None, None,
    // 7:00 and :30
    '\u{1F556}', None, None, None, None, None,
    '\u{1F562}', None, None, None, None, None,
    // 8:00 and :30
    '\u{1F557}', None, None, None, None, None,
    '\u{1F563}', None, None, None, None, None,
    // 9:00 and :30
    '\u{1F558}', None, None, None, None, None,
    '\u{1F564}', None, None, None, None, None,
    // 10:00 and :30
    '\u{1F559}', None, None, None, None, None,
    '\u{1F565}', None, None, None, None, None,
    // 11:00 and :30
    '\u{1F55A}', None, None, None, None, None,
    '\u{1F566}', None, None, None, None, None,
    // 0x90-0x97: random clocks
    None,         None, None,         None,
    '\u{231A}',  None, '\u{231A}',  '\u{23F1}',
    // 0x98-0x9F
    '\u{23F1}',  None, '\u{23F0}',  None,
    None,         None, None,         None
  ]);
});
