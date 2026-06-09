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

LoadDefinitions!({
  #[rustfmt::skip]
  DeclareFontMap!("line", mixrc![
    // 0x00-0x0F: rising segments \line(1..2, 1..6) — ╱
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    // 0x10-0x17: mixed band (rising x=3 / falling x=1) — favour ╱, ╲ tail
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    '\u{2571}', '\u{2571}', '\u{2572}', '\u{2572}',
    // 0x18-0x1F: mixed; 0x1B ('33) = \@getlarrow(1,0) left arrowhead
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2190}',
    '\u{2571}', '\u{2571}', '\u{2572}', '\u{2572}',
    // 0x20-0x2F: steeper bands — falling ╲ dominates the upper half
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    '\u{2571}', '\u{2572}', '\u{2572}', '\u{2572}',
    '\u{2571}', '\u{2571}', '\u{2571}', '\u{2571}',
    '\u{2571}', '\u{2572}', '\u{2572}', '\u{2572}',
    // 0x30-0x3F: falling band; 0x36 ('66) up vector head, 0x3F ('77) down head
    '\u{2572}', '\u{2572}', '\u{2572}', '\u{2572}',
    '\u{2572}', '\u{2572}', '\u{2191}', '\u{2572}',
    '\u{2572}', '\u{2572}', '\u{2572}', '\u{2572}',
    '\u{2572}', '\u{2572}', '\u{2572}', '\u{2193}',
    // 0x40-0x4F: arrowheaded segments (letter range, \@getlarrow/\@getrarrow)
    '\u{2572}', '\u{2197}', '\u{2197}', '\u{2197}',
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    // 0x50-0x5F: arrowheaded segments (upper letters cont.)
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    '\u{2197}', '\u{2197}', '\u{2197}', '\u{2197}',
    // 0x60-0x6F: arrowheaded falling/left set (lowercase, +64 offset)
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    // 0x70-0x7F: arrowheaded falling/left set (cont.)
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    '\u{2198}', '\u{2198}', '\u{2198}', '\u{2198}',
    '\u{2198}', '\u{2198}', '\u{2198}', None
  ]);
});
