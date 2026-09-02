//! newverbs.sty (M. Scharrer) — `\newverbcommand{\cmd}[\verb]{before}{after}`.
//!
//! newverbs.sty:52-69 `\new@@verbcommand`: the created command opens a
//! `\bgroup`, runs `before`, then the verb macro; `after` and the matching
//! `\egroup` are appended to `\verb@egroup` (:58,64), which real `\verb`
//! (latex.ltx:15501) executes when its verbatim scan ends. Our native `\verb`
//! reads its body directly and never runs `\verb@egroup`, so the extra group
//! leaked past every `\end{…}` ("Attempt to end mode internal_vertical" —
//! homework.cls demos ×4 via `\newverbcommand{\cverb}{\color{…}}{}`; Perl's
//! `\verb` is the same reader and fails identically). Here the command reads
//! the verbatim argument through the same reader as `\verb`
//! (`read_verb_invocation`) and puts `before`/`after` inside the one hidden
//! group. `\fverb`/`\qverb` (:100-108) are provided the same way.
//! Guard: `perfect_kernel_batch54::newverbcommand_wraps_the_verb_body`.
use latexml_engine::latex_constructs::read_verb_invocation;

use crate::prelude::*;

fn define_verb_command(cmd: Token, before: Tokens, after: Tokens) -> Result<()> {
  let mut body = vec![T_CS!("\\lx@hidden@bgroup")];
  body.extend(before.unlist());
  body.push(T_CS!("\\lx@newverbs@verb"));
  body.push(T_BEGIN!());
  body.extend(after.unlist());
  body.push(T_END!());
  def_macro(cmd, None, Tokens::new(body), None)?;
  Ok(())
}

LoadDefinitions!({
  RequirePackage!("shortvrb");
  DefMacro!("\\lx@newverbs@verb{}", sub[(after)] {
    match read_verb_invocation()? {
      Some(mut inner) => {
        inner.extend(after.unlist());
        inner.push(T_CS!("\\lx@hidden@egroup"));
        Ok(Tokens::new(inner))
      },
      None => Ok(Tokens!()),
    }
  });
  DefMacro!("\\lx@newverbs@define DefToken []{}{}", sub[(cmd, _verb, before, after)] {
    define_verb_command(cmd, before, after)?;
    Ok(Tokens!())
  });
  for definer in [
    "\\newverbcommand",
    "\\renewverbcommand",
    "\\provideverbcommand",
  ] {
    Let!(&T_CS!(definer), "\\lx@newverbs@define");
  }
  // newverbs.sty:100-108 `\qverb` (quoted) and `\fverb` (framed).
  DefMacro!("\\qverbbeginquote", "``");
  DefMacro!("\\qverbendquote", "''");
  RawTeX!(r"\newverbcommand{\qverb}{\qverbbeginquote}{\qverbendquote}\newverbcommand{\fverb}{}{}");
});
