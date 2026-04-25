//! tensor.sty (Mike Piff) — tensor index typesetting.
//!
//! No Perl binding upstream. ~7 sandbox papers (1608.02970 …
//! 1812.01559) hit `\tensor` undefined when the .sty isn't on the
//! texmf path. The real tensor.sty places sub/superscript indices at
//! specific positions relative to a base symbol; we approximate by
//! emitting base + indices concatenated, which produces serviceable
//! XMath since `^`/`_` in the indices arg still bind via the math
//! parser.
//!
//! Note: revsymb.sty defines a different `\tensor` (single-arg
//! overarrow); revsymb's load order overrides this stub when both
//! load — that's the desired precedence (revtex papers want the
//! over-arrow flavor).
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // \tensor{base}{indices} — base followed by index group.
  // \tensor*{base}{indices} — same; star is a "left-align" flag we ignore.
  DefMacro!("\\tensor OptionalMatch:* {}{}", "{#2#3}");
  // \indices{...} — used in tensor.sty 4.x to wrap the indices arg.
  DefMacro!("\\indices{}", "{#1}");
});
