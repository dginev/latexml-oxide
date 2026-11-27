//! `ccicons` font map: ccicons.sty's icons (`\usefont{U}{ccicons}{m}{n}\char<n>`), reached through `\selectfont`'s
//! family-as-encoding branch as pifont's `pzd` is. Perl ships none: its slots printed OT1 Greek (`\ccby` as "Γ Δ",
//! witnesses 2602.07369, 2203.09493, 2210.04484). Slots from ccicons-u.enc, each icon its Unicode 13 CC symbol; the
//! NC variants nceu/ncjp take the NC symbol, and sampling, share and remix (no Unicode form) their glyph's letter.
use crate::prelude::*;

LoadDefinitions!({
  let mut map = [None; 128];
  #[rustfmt::skip]
  let glyphs = [
    (0, '\u{1F16D}'), // c: \ccLogo 🅭
    (1, '\u{1F16F}'), // b: \ccAttribution 🅯
    (2, '\u{1F10E}'), // a: \ccShareAlike 🄎
    (3, '\u{229C}'), // d: \ccNoDerivatives ⊜
    (4, '\u{1F10F}'), (5, '\u{1F10F}'), (6, '\u{1F10F}'), // n, e, y: \ccNonCommercial(EU|JP) 🄏
    (7, '\u{1F16E}'), // p: \ccPublicDomain 🅮
    (8, '\u{1F10D}'), // z: \ccZero 🄍
    (9, 'm'), (10, 's'), (11, 'r'), // \ccSampling, \ccShare, \ccRemix
    (12, '\u{A9}'), // copyright: \ccCopy ©
    (13, '\u{1F16E}'), // P: \ccPublicDomainAlt 🅮
  ];
  for (slot, glyph) in glyphs {
    map[slot] = Some(glyph);
  }
  DeclareFontMap!("ccicons", Rc::from(map));
});
