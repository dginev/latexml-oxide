//! The text rules on printed ink and the token primitives of `author_block/rows.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// The spacing commands a run of which parts two author names (user ruling 2026-10-19): the control space `\ ` — also
/// as the control return a line ending in `\ ` leaves, TeX dropping the line's trailing space (latex.ltx's `\^^M` is
/// `\ `; cs0207008's `K.V. Hindriks \affila\ \ ⏎ W. van der Hoek`) —, the medium and thick spaces `\:`, `\>`, `\;`,
/// the tie `~`, `\enspace` and `\enskip` (latex.ltx `\hskip.5em\relax`, a separator by itself only after a mark or note,
/// [`normalize_hspace_separators`]: `A.\enskip B. Smith` is one name).
pub(in crate::base_utilities) fn is_spacing_command(t: &Token) -> bool {
  *t == T_ACTIVE!('~')
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|s| {
        matches!(
          s,
          "\\ " | "\\\r" | "\\:" | "\\>" | "\\;" | "\\enspace" | "\\enskip"
        )
      }))
}

/// `tokens` after the markup a line or an entry may open with before its mark: spaces, group openings and the font and
/// size commands of [`NAME_TEXT_COMMANDS`] (`\normalsize{$^1$I3A - Universidad de Zaragoza} ~~~~~~
/// \normalsize{$^2$École Polytechnique Fédérale de Lausanne}`, 2308.10525).
pub(in crate::base_utilities) fn after_leading_markup(tokens: &[Token]) -> &[Token] {
  let lead = tokens
    .iter()
    .take_while(|t| {
      matches!(t.get_catcode(), Catcode::SPACE | Catcode::BEGIN)
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s)))
    })
    .count();
  &tokens[lead..]
}

/// A token that ends an author line or a name, before any spacing run reads it so: the `\and` family, `\quad`,
/// `\qquad`, a line break, the horizontal spaces [`normalize_hspace_separators`] makes `\quad`. Not `\par`: an author
/// block is set in a `c` column (latex.ltx's `\@maketitle`, `\begin{tabular}[t]{c}\@author\end{tabular}`; ACL's the
/// same), restricted horizontal mode, where a blank line's `\par` ends no paragraph and prints nothing (tex.web §1094
/// `hmode+par_end` calls `end_graf`, §1096 acts only when `mode=hmode`): 2009.12421's `Ehsan
/// Shareghi$^\spadesuit$$^\clubsuit$ <blank line> \ \ Nigel Collier$^\clubsuit$` prints the two names on one row.
pub(in crate::base_utilities) fn is_line_separator(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && (author_affil_splits()
      .iter()
      .chain(author_and_splits().iter())
      .chain(author_line_breaks().iter())
      .any(|d| matches!(d, SplitDelim::Token(d) if d == t))
      || t.with_str(|s| matches!(s, "\\qquad" | "\\hfill" | "\\hfil" | "\\hspace")))
}

/// The font and size commands an author line styles a name or an entry with: their argument, or the group after the
/// declaration, prints as the name it styles ([`shows_name_text`]), and an entry may open with them before its mark
/// ([`after_leading_markup`]).
pub(in crate::base_utilities) const NAME_TEXT_COMMANDS: &[&str] = &[
  "\\footnotesize",
  "\\scriptsize",
  "\\tiny",
  "\\LARGE",
  "\\huge",
  "\\Huge",
  "\\rm",
  "\\sf",
  "\\tt",
  "\\em",
  "\\sl",
  "\\upshape",
  "\\itshape",
  "\\slshape",
  "\\scshape",
  "\\bfseries",
  "\\mdseries",
  "\\rmfamily",
  "\\sffamily",
  "\\ttfamily",
  "\\normalfont",
  "\\textbf",
  "\\textit",
  "\\textsc",
  "\\textsl",
  "\\textup",
  "\\textmd",
  "\\textrm",
  "\\textsf",
  "\\textnormal",
  "\\emph",
  "\\bf",
  "\\it",
  "\\sc",
  "\\large",
  "\\Large",
  "\\normalsize",
  "\\small",
];
