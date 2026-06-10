//! `line` font encoding — the LaTeX picture-mode line fonts (`line10`,
//! `linew10`), declared by `\font...=line10` and reached through
//! `Font.pm`-table family `line`/`linew` → encoding `line`.
//!
//! **Why this map exists (root cause of a canvas OOM cluster):** LaTeX-2.09-era
//! plain-TeX documents (math0102053, math0102089, math0212126, math0504436,
//! math0506088, math0604321, …) inline picture mode's `\@sline`, whose drawing
//! loop advances by the width of an `\hbox{\@linefnt\@getlinechar(x,y)}`:
//!
//! ```tex
//! \@clnwd=\wd\@linechar
//! \@whiledim \@clnwd <\@linelen \do {…\advance\@clnwd \wd\@linechar}
//! ```
//!
//! Real TeX gets nonzero char widths (2.5–10 pt) from `line10.tfm`, so the loop
//! terminates. Without a fontmap, `FontDecode` drops the char → empty box →
//! width 0 pt → the `\ifdim` NEVER flips → unbounded box accumulation (~1.9 M
//! boxes → 4.5 GB RSS). Perl LaTeXML ships no `line` fontmap
//! (`Info:fontmap:line Couldn't find fontmap`) and OOMs identically — this map
//! is a surpass-Perl reliability fix THROUGH the architecture's own mechanism
//! (no control-flow divergence; Perl with the same map would behave the same).
//! Modern latex.ltx even guards this exact hazard (`\ifdim\wd\@linechar=\z@
//! \setbox\@linechar\hbox{.}\@badlinearg\fi`, latex.ltx ~L13777) — but these
//! old documents inline the unguarded 2.09 macros, so the FONT's width is the
//! only lever that reaches them.
//!
//! **Slot layout** (kernel `\@getlinechar`, latex.ltx L13815): segment slots
//! are `8x-9+y` for rising `\line(x,y)` (y>0) and `8x-9+|y|+16` for falling
//! (y<0), x,y ∈ 1..6 coprime; `\@getlarrow`/`\@getrarrow` put arrowheaded
//! variants at 27 ('33, horizontal left head), 54 ('66, up head used by
//! `\@upvector`), 63 ('77, down head used by `\@downvector`) and across the
//! letter range (`16x-9±2y`, +64 for the falling/left set). The rising/falling
//! bands overlap structurally, so a glyph-exact map is impossible — we choose a
//! plausible diagonal per band. The semantically ESSENTIAL property is that
//! every populated slot maps to a NONZERO-width glyph; the picture itself is an
//! approximation either way (hand-rolled `\raise`/`\hskip` positioning).
use crate::prelude::*;

/// The `line` encoding slot table (0x00–0x7F), exposed for the TFM
/// slot-coverage test (every `line10.tfm`-populated slot must be `Some` —
/// a `None` on a populated slot silently resurrects the zero-width
/// `\@whiledim` infinite-loop OOM for that slope).
#[rustfmt::skip]
pub const LINE_SLOTS: [Option<char>; 128] = [
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2572}'), Some('\u{2572}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2190}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2572}'), Some('\u{2572}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'),
  Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'), Some('\u{2571}'),
  Some('\u{2571}'), Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'),
  Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'),
  Some('\u{2572}'), Some('\u{2572}'), Some('\u{2191}'), Some('\u{2572}'),
  Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'),
  Some('\u{2572}'), Some('\u{2572}'), Some('\u{2572}'), Some('\u{2193}'),
  Some('\u{2572}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'), Some('\u{2197}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'),
  Some('\u{2198}'), Some('\u{2198}'), Some('\u{2198}'), None,
];


LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("line", Rc::from(&LINE_SLOTS[..]));
});

#[cfg(test)]
mod tests {
  use super::LINE_SLOTS;
  use crate::package::lcircle_fontmap::LCIRCLE_SLOTS;
  use std::process::Command;

  /// The TFM-populated slots of a picture font, parsed from `tftopl` output
  /// (lines like `(CHARACTER O 27` / `(CHARACTER C a`). Returns None when the
  /// host TeX tree lacks the font or `tftopl` (test self-skips).
  fn tfm_slots(font: &str) -> Option<Vec<usize>> {
    let tfm = Command::new("kpsewhich").arg(format!("{font}.tfm")).output().ok()?;
    if !tfm.status.success() {
      return None;
    }
    let path = String::from_utf8_lossy(&tfm.stdout).trim().to_string();
    let out = Command::new("tftopl").arg(&path).output().ok()?;
    if !out.status.success() {
      return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut slots = Vec::new();
    for line in text.lines() {
      if let Some(rest) = line.strip_prefix("(CHARACTER ") {
        let mut it = rest.split_whitespace();
        match (it.next(), it.next()) {
          (Some("O"), Some(oct)) => {
            if let Ok(n) = usize::from_str_radix(oct, 8) {
              slots.push(n);
            }
          },
          (Some("C"), Some(ch)) => {
            if let Some(c) = ch.chars().next() {
              slots.push(c as usize);
            }
          },
          _ => {},
        }
      }
    }
    Some(slots)
  }

  /// Every TFM-populated slot MUST map to `Some` glyph: a `None` on a
  /// populated slot gives that char a zero-width box and silently resurrects
  /// the `\@whiledim` infinite-loop OOM (the canvas_3 cluster this map
  /// fixed) for documents drawing that slope/arc. PR #249 review P3-16.
  #[test]
  fn every_tfm_populated_slot_is_mapped() {
    for (font, map) in [("line10", &LINE_SLOTS), ("lcircle10", &LCIRCLE_SLOTS)] {
      let Some(slots) = tfm_slots(font) else {
        eprintln!("SKIP every_tfm_populated_slot_is_mapped: {font}.tfm / tftopl unavailable");
        continue;
      };
      assert!(!slots.is_empty(), "{font}.tfm parsed to zero slots?");
      for slot in slots {
        if slot < map.len() {
          assert!(
            map[slot].is_some(),
            "{font} slot {slot} (0o{slot:o}) is populated in the TFM but maps \
             to None — zero-width box, \\@whiledim OOM risk"
          );
        }
      }
    }
  }
}
