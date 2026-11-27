//! `manfnt` font map, Knuth's font for the TeX and METAFONT manuals, declared under both of its names: the encoding
//! `manfnt` of plain's `\font\x=manfnt` (common/font.rs, Perl Common/Font.pm:117; `\char127` was dropped) and the NFSS
//! family `U/manual` of manfnt.sty, reached through `\selectfont`'s family-as-encoding branch (`\dbend` printed "¨").
//! Perl ships none. Slots from knuth-lib/manfnt.mf's `beginchar` lines, each glyph its closest character; the cubes
//! have no Unicode form and stand in as ⧉. Witnesses 1012.0265, 2008.09297, 2410.22549 (`\font`), 2004.07097,
//! 2011.01162, 2403.06925 (`\mancube`), 2105.09724 (`\textdbend`).
use crate::prelude::*;

/// The METAFONT logo's letters M E T A F O N, at each size's slots (manfnt.mf:583-707, `code.M` … `code.N`).
#[rustfmt::skip]
const LOGO_SIZES: [&[u8; 7]; 9] = [
  b"METAFON", b"89:;<=>", b"hijklmn", b"opqrstu", b"/012345", b"\\]^_efg", b"()*+,-.", b"BCDGHIJ", b"KLUVWvw",
];

LoadDefinitions!({
  let mut map = [None; 128];
  for size in LOGO_SIZES {
    for (slot, letter) in size.iter().zip("METAFON".chars()) {
      map[usize::from(*slot)] = Some(letter);
    }
  }
  #[rustfmt::skip]
  let glyphs = [
    (0, '\u{2621}'), (126, '\u{2621}'), (127, '\u{2621}'), // \reversedvideodbend, \lhdbend, \dbend ☡
    (1, '\u{2B21}'), (2, '\u{2B21}'), (3, '\u{2B21}'), (4, '\u{2B21}'), (5, '\u{2B21}'), // hexes ⬡
    (17, '\u{2B2D}'), (18, '\u{2B2D}'), (19, '\u{2B2D}'), (20, '\u{2B2D}'), // kidney beans ⬭
    (21, '\u{2712}'), (22, '\u{2712}'), (23, '\u{2712}'), // pen nibs ✒
    (25, 'S'), (26, 'O'), (27, 'I'), // Ionian letters
    (28, '\u{29C9}'), (29, '\u{29C9}'), // \mancube, \manimpossiblecube ⧉
    (30, '\u{2606}'), // \manstar ☆
    (31, '\u{25E1}'), // \oneu, a u-shaped stroke ◡
    (32, '\u{25DD}'), (33, '\u{25D4}'), (34, '\u{25DC}'), (35, '\u{2314}'), // quarter circles ◝ ◔ ◜, cone ⌔
    (36, '\u{25CE}'), (37, '\u{25C8}'), // concentric circles ◎, diamonds ◈
    (38, '\u{2723}'), (39, '\u{2723}'), // quadrifolia ✣
    (54, '\u{25B2}'), (55, '\u{25BC}'), // Addison-Wesley triangles ▲ ▼
    (63, '\u{1D521}'), // Gothic d 𝔡
    (80, '\u{25AA}'), (81, '\u{25AB}'), (82, '\u{25A0}'), (83, '\u{25A1}'), // pixels ▪ ▫ ■ □
    (88, '\u{2B2D}'), (89, '\u{25EF}'), (90, '\u{25CB}'), (91, '\u{25E1}'), // circles ⬭ ◯ ○, coin edge ◡
    (96, '\u{2150}'), // special 1/7 ⅐
    (97, '\u{25DD}'), (98, '\u{25DE}'), (99, '\u{25DF}'), (100, '\u{25DC}'), // \qc arcs ◝ ◞ ◟ ◜ (as pdflatex draws them)
    (120, '\u{25B6}'), // \mantriangleright ▶
    (121, '\u{2794}'), // \manerrarrow ➔
  ];
  for (slot, glyph) in glyphs {
    map[slot] = Some(glyph);
  }
  map[6..=16].fill(Some('A')); // stick A's
  let map: Rc<[Option<char>]> = Rc::from(map);
  DeclareFontMap!("manfnt", map.clone());
  DeclareFontMap!("manual", map);
});
