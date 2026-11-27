//! `ark` font map: dingbat.sty's `\arkfamily` (`\fontencoding{U}\fontfamily{ark}\selectfont`, font ark10), reached
//! through `\selectfont`'s family-as-encoding branch as pifont's `pzd` is. Perl ships none: its slots printed OT1 letters.
//! Slots and glyphs from ark10.mf; a hand's side (left/right) has no Unicode form, its pointing direction does.
//! Witness 2303.10475 (ar5iv #408, dingbat.sty's `\checkmark` in the companion `dingbat` map).
use crate::prelude::*;

LoadDefinitions!({
  let mut map = [None; 128];
  #[rustfmt::skip]
  let glyphs = [
    (0o103, '\u{21B5}'), // \carriagereturn ↵
    (0o104, '\u{1F44E}'), // \leftthumbsdown 👎
    (0o105, '\u{1F441}'), // \eye 👁
    (0o114, '\u{261C}'), // \rightpointleft ☜
    (0o120, '\u{270E}'), // \smallpencil ✎
    (0o122, '\u{261E}'), // \leftpointright ☞
    (0o125, '\u{1F44D}'), // \leftthumbsup 👍
    (0o127, '\u{270E}'), // \largepencil ✎
    (0o144, '\u{1F44E}'), // \rightthumbsdown 👎
    (0o165, '\u{1F44D}'), // \rightthumbsup 👍
  ];
  for (slot, glyph) in glyphs {
    map[slot] = Some(glyph);
  }
  DeclareFontMap!("ark", Rc::from(map));
});
