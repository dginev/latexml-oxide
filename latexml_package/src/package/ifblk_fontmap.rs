//! ifblk font encoding (from ifblk.fontmap.ltxml)
//! Block element symbols.
use crate::prelude::*;

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("ifblk", mixrc![
    // 0x00-0x07
    None, None, None, None, None, None, None, None,
    // 0x08-0x0F
    None, None, None, None, None, None, None, None,
    // 0x10-0x17
    None, None, None, None, None, None, None, None,
    // 0x18-0x1F
    None, None, None, None, None, None, None, None,
    // 0x20-0x27
    None, None, None, None, None, None, None, None,
    // 0x28-0x2F
    None, None, None, None, None, None, None, None,
    // 0x30-0x37: block elements at 0x31-0x32, 0x35-0x36
    None,         '\u{2598}', '\u{259D}', '\u{2580}',
    None,         '\u{2590}', '\u{259C}', '\u{2584}',
    // 0x38-0x3A
    '\u{2599}', '\u{259F}', '\u{2588}'
  ]);
});
