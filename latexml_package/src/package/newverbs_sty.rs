//! newverbs.sty (M. Scharrer) — `\newverbcommand{\cmd}[\verb]{before}{after}`.
//!
//! newverbs.sty:52-69 `\new@@verbcommand`: the created command opens a
//! `\bgroup`, runs `before`, then the verb macro; `after` and the matching
//! `\egroup` are appended to `\verb@egroup` (:58,64), which real `\verb`
//! (latex.ltx:15501) executes when its verbatim scan ends. Our `\verb` closes
//! its group through `\verb@egroup` too (sect06.rs), so the protocol is ported
//! as is and the verb is whatever `\verb` (or the optional `[\verbcmd]`) means
//! at the call: a package that redefines the `\verb`/`\verb@egroup` pair
//! (accessibility.sty:1566-1577) keeps both halves. The command's group is a
//! hidden one. It once leaked past every `\end{…}` ("Attempt to end mode
//! internal_vertical" — homework.cls demos ×4 via
//! `\newverbcommand{\cverb}{\color{…}}{}`), when `\verb` read its body without
//! running `\verb@egroup`. `\fverb`/`\qverb` (:100-108) are provided the same
//! way, and the `\MakeShortVerb`/`\MakeSpecialShortVerb` family (:112-137)
//! verbatim. Guards: `perfect_kernel_batch54::newverbcommand_wraps_the_verb_body`,
//! `binding_singletons_56::qverb_keeps_a_redefined_verb_pair`.
use crate::prelude::*;

/// newverbs.sty:52-69 `\new@@verbcommand`'s body, `\bgroup`/`\egroup` being
/// the hidden pair: `\lx@newverbs@setup{after}` is :56-64 (after `\verb`'s own
/// group closes, `\verb@egroup` runs `after` and closes the command's group),
/// then `\verbatim@font` is selected BEFORE `before` runs (:65-66), so
/// `before`, the body and `after` share the verbatim font — `\qverb`'s opening
/// quote was roman (macros2e's `\MakeSpecialShortVerb\qverb\"`).
/// `\lx@newverbs@font` keeps the document's encoding across `\verbatim@font`,
/// whose `\fontencoding{ASCII}` (OXIDIZED_DESIGN #144) is meant for verbatim
/// text: the body still gets it from the `\verb` constructor's own font
/// (sect06.rs `\@internal@text@verb`), and the user's code prints as pdflatex
/// prints it — `\qverb`'s quotes are OT1 typewriter ‘‘ ’’, not ASCII `` ''.
/// Guard: `binding_singletons_56::qverb_quotes_share_the_verbatim_font`.
fn define_verb_command(
  cmd: Token,
  verb: Option<Tokens>,
  before: Tokens,
  after: Tokens,
) -> Result<()> {
  let mut body = vec![
    T_CS!("\\lx@hidden@bgroup"),
    T_CS!("\\newverbcommand@settings"),
    T_CS!("\\lx@newverbs@setup"),
    T_BEGIN!(),
  ];
  body.extend(after.unlist());
  body.push(T_END!());
  body.push(T_CS!("\\lx@newverbs@font"));
  body.extend(before.unlist());
  // :47-50: the verb command, `\verb` unless `[\verbcmd]` names another.
  match verb {
    Some(verb) if !verb.is_empty() => body.extend(verb.unlist()),
    _ => body.push(T_CS!("\\verb")),
  }
  def_macro(cmd, None, Tokens::new(body), None)?;
  Ok(())
}

LoadDefinitions!({
  RequirePackage!("shortvrb");
  // newverbs.sty:51, :56-64 (`#5` passed as the argument) and :96-99.
  RawTeX!(
    r"\let\newverbs@end\@empty
\def\lx@newverbs@setup#1{%
  \ifx\newverbs@end\@empty
  \expandafter\def\expandafter\verb@egroup\expandafter{\verb@egroup\newverbs@end}%
  \fi
  \begingroup\def\@tempa{#1}%
  \expandafter\expandafter\expandafter\endgroup
  \expandafter\expandafter\expandafter\def
  \expandafter\expandafter\expandafter\newverbs@end
  \expandafter\expandafter\expandafter{\expandafter\@tempa\newverbs@end\lx@hidden@egroup}%
  \def\newverbs@txend{#1\lx@hidden@egroup}}
\def\newverbcommand@settings{\let\verb@orig@egroup\verb@egroup\let\verbbox\@tempboxa}
\def\lx@newverbs@font{\edef\lx@newverbs@encoding{\f@encoding}%
  \verbatim@font\fontencoding{\lx@newverbs@encoding}\selectfont
  \let\verbatim@font\relax}"
  );
  DefMacro!("\\lx@newverbs@define DefToken []{}{}", sub[(cmd, verb, before, after)] {
    define_verb_command(cmd, verb, before, after)?;
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
  // newverbs.sty:112-137 verbatim: `\MakeShortVerb` takes an optional verb
  // command, and `\MakeSpecialShortVerb\cmd\char` makes `\char` a short form of
  // any verb command (`\qverb`, `\fverb`), through shortvrb's `\@MakeShortVerb`
  // reading `\@shortvrbdef`. macros2e.tex:10 `\MakeSpecialShortVerb\qverb\"`
  // (undefined before: an `<ERROR>` in the preamble stranded the frontmatter,
  // 4 schema errors). Guard
  // `perfect_kernel_batch56::make_special_short_verb_is_defined`.
  RawTeX!(
    r"\def\MakeShortVerb{\@ifstar{\newverbs@MakeShortVerb*}{\newverbs@MakeShortVerb{}}}
\def\newverbs@MakeShortVerb#1{\@ifnextchar[{\newverbs@@MakeShortVerb{#1}}{\@MakeSpecialShortVerb{#1}{\verb}}}
\def\newverbs@@MakeShortVerb#1[#2]{\@MakeSpecialShortVerb{#1}{#2}}
\def\@MakeSpecialShortVerb#1#2#3{\def\@shortvrbdef{#2#1}\@MakeShortVerb{#3}}
\newcommand*\MakeSpecialShortVerb{\@ifstar{\@MakeSpecialShortVerb{*}}{\@MakeSpecialShortVerb{}}}"
  );
});
