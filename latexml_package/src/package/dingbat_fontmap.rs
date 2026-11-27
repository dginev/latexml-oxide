//! `dingbat` font map: dingbat.sty's `\dingbatfamily` (`\fontencoding{U}\fontfamily{dingbat}\selectfont`), reached
//! through `\selectfont`'s family-as-encoding branch as pifont's `pzd` is. Perl ships none: its slots printed OT1
//! letters (`\checkmark` as "D", witness 2303.10475, ar5iv #408). Slots and glyphs from dingbat.mf, each its closest
//! character as in ding.fontmap: the fleurons a floral heart facing their side, the border tiles box-drawing pieces,
//! and the `\Sborder`/`\Zborder` tiles, an angular S and Z with no Unicode form, their letters.
use crate::prelude::*;

LoadDefinitions!({
  let mut map = [None; 128];
  #[rustfmt::skip]
  let glyphs = [
    (b'B', '\u{25C7}'), // \squarewithdots ◇
    (b'C', '\u{25C6}'), // \filledsquarewithdots ◆
    (b'D', '\u{2713}'), // \checkmark ✓
    (b'E', '\u{2619}'), (b'F', '\u{2767}'), (b'G', '\u{2619}'), (b'H', '\u{2767}'), // fleurons ☙ ❧
    (b'I', '\u{1F4E1}'), // \satellitedish 📡
    (b'J', '\u{2619}'), (b'K', '\u{2767}'), (b'L', '\u{2619}'), (b'M', '\u{2767}'), // flowers ☙ ❧
    (b'N', '\u{261E}'), // \rightpointright ☞
    (b'O', '\u{2693}'), // \anchor ⚓
    (b'S', 'S'), (b'Z', 'Z'), // \Sborder, \Zborder
    (b'a', '\u{2554}'), (b'b', '\u{2550}'), (b'c', '\u{2557}'), // ╔ ═ ╗
    (b'd', '\u{2551}'), (b'e', '\u{2551}'), // ║ ║
    (b'f', '\u{255A}'), (b'g', '\u{2550}'), (b'h', '\u{255D}'), // ╚ ═ ╝
  ];
  for (slot, glyph) in glyphs {
    map[usize::from(slot)] = Some(glyph);
  }
  DeclareFontMap!("dingbat", Rc::from(map));
});
