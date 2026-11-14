//! centernot.sty — a `\not` slash centred on the symbol it negates. No Perl binding.
use latexml_package::prelude::*;

LoadDefinitions!({
  // centernot.sty:30-44 `\newcommand*{\centernot}{\mathpalette\@centernot}`: the next math symbol with a `\not` slash
  // overprinted at its centre (`\rlap`, `\kern` by half widths) — the negated relation `\not` makes of it, placed
  // otherwise. The raw file's `\mathpalette`/`\rlap` boxes would split the symbol from its slash (a `\mathchoice` of
  // four boxed copies in the formula). newtxmath loads it (newtxmath.sty:36, the acmart chain): witnesses 2002.09451,
  // 2103.01489, 2104.13315 (`\centernot` undefined).
  DefMacro!("\\centernot", "\\not");
});
