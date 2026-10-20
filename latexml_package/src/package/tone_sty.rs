//! tone.sty (tipa's tone letters, FUKUI Rei): the raw file, with its tone letters as Unicode's.
//!
//! `\tone{51}` (tone.sty:13-14) draws a contour from tipa's tone-letter parts — slots 128-145, each a segment raised to
//! its level (`\@outputtone`, :37-58), not a character — and the stem (slot 191) at its right; `\rtone` (:16-17) puts
//! the stem at the left, `\stone` (:19-20) draws short segments. Unicode writes the contour as its tone letters, one
//! per level (UAX: Chao tone letters, U+02E5-02E9 with the stem at the right, U+A712-A716 at the left), which fonts
//! join into the contour: `\tone{51}` ˥˩, `\rtone{15}` ꜖꜒. A level outside 1-5 keeps tone.sty's construction, whose
//! tone-letter parts have no character, and says so. OXIDIZED_DESIGN_DIVERGENCES #781. Witnesses 2601.19932,
//! 2602.10003 (`\usepackage[tone]{tipa}`, `\tone{…}`).
use crate::prelude::*;

/// The tone letters of `levels`, extra-low 1 to extra-high 5, each `stem + 5 - level`; `None` if a level is not one.
fn tone_letters(levels: &str, stem: u32) -> Option<String> {
  levels
    .chars()
    .filter(|c| !c.is_whitespace())
    .map(|c| match c.to_digit(10) {
      Some(level @ 1..=5) => char::from_u32(stem + 5 - level),
      _ => None,
    })
    .collect()
}

LoadDefinitions!({
  InputDefinitions!("tone", extension => Some("sty".into()), noltxml => true);
  // tone.sty read again (`\input{tone.sty}`) keeps the commands saved the first time: saving these wrappers would
  // make a level outside 1-5 expand to itself.
  if IsDefined!(&T_CS!("\\lx@tone@raw@tone")) {
    return Ok(());
  }
  for (cs, raw, stem) in [
    ("\\tone", "\\lx@tone@raw@tone", 0x02E5),
    ("\\stone", "\\lx@tone@raw@stone", 0x02E5),
    ("\\rtone", "\\lx@tone@raw@rtone", 0xA712),
  ] {
    Let!(T_CS!(raw), T_CS!(cs));
    DefMacro!(T_CS!(cs), "{}", sub[(levels)] {
      let tokens = levels.unlist_cow().into_owned();
      match tone_letters(&Tokens::new(tokens.clone()).to_string(), stem) {
        Some(letters) if !letters.is_empty() => {
          let mut out = vec![T_CS!("\\leavevmode")];
          out.extend(letters.chars().map(|c| Token::new(c.to_string(), Catcode::OTHER)));
          Tokens::new(out)
        },
        _ => {
          let levels = Tokens::new(tokens.clone()).to_string();
          if !already_reported(&s!("tone_letter_{cs}{levels}")) {
            Warn!("missing_font_encoding", "T3",
              s!("tone.sty's {cs}{{{levels}}} has no Chao tone letter (levels 1-5): its tone-letter parts have no \
                  character and are dropped"));
          }
          let mut out = vec![T_CS!(raw), T_BEGIN!()];
          out.extend(tokens);
          out.push(T_END!());
          Tokens::new(out)
        },
      }
    });
  }
});
