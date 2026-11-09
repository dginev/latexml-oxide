//! The visible text of a name: name words, particles, qualifiers, name groups and merged creators.

use super::*;

/// Whether `line` continues the author names `names` instead of starting their affiliations: styled as the names are,
/// all of it names (or their qualifiers), and either the names read as an unfinished list — ending in a comma, "and"
/// or "&" (`Joshua N.\ Winn\altaffilmark{1}, Andrew W.\ Howard\altaffilmark{2},\\ Avi Shporer\altaffilmark{1}`,
/// 1010.1318) — or the line opens with "and" (`…, \emph{Member, IEEE},\\ and Warren J. Gross, \emph{Senior Member,
/// IEEE}`, 1611.04834) — or the line is a comma list of two or more names and nothing else (`…, Wenhan Liu\\ Haonan
/// Chen, Zheng Liu, Zhicheng Dou, and Ji-Rong Wen`, 2308.07107; 2312.11514, 2406.10513). "and Max--Planck Institut für
/// Mathematik, Bonn" (math0208081), `John Smith,\\ {\it Bell Labs, Murray Hill}` and `Pierre Fernandez\\ Meta FAIR \&
/// Inria Rennes` (2402.14904: joined by "&" alone, as institutions are) stay affiliations.
pub fn names_continue(names: &Tokens, line: &Tokens) -> bool {
  leading_markup(line) == leading_markup(names) && names_list_continues(names, line)
}

/// [`names_continue`]'s text test alone, whatever markup either line opens with: in a block whose authors carry marks,
/// a row of names after names continues them however it is styled (`Ann Able$^{1}$, Bob Baker$^{2}$,\\
/// \textbf{Cat Cole}$^{\dagger}$, \textbf{Dan Dee}$^{\dagger}$`, 2307.00040; `\textbf{Cat Cole, Dan Dee,}`, 2510.11639) —
/// its words tell names from places there, as the unmarked block's styled-affiliation alternation needs the markup too
/// (hep-ph9306253, 0811.1526).
pub(in crate::base_utilities) fn names_list_continues(names: &Tokens, line: &Tokens) -> bool {
  // (a separator in a wrapper of its own is the line's: `\textbf{Seth Ebner}$^{\diamondsuit}$\textrm{,}\\ \textbf{Brandon
  // Smock}`, 2512.18601)
  let names_text = visible_name_text(hoist_wrapped_separators(names.clone()).unlist_ref());
  let names_text = names_text.trim_end();
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim();
  let unfinished =
    names_text.ends_with(',') || names_text.ends_with(" and") || names_text.ends_with('&');
  let count = name_count(text);
  ((unfinished || text.starts_with("and ") || text.starts_with("& ")) && count > 0)
    || (count >= 2 && text.contains(',') && name_count(names_text) >= 2)
}

/// The markup an author line opens with, before its first letter: its control sequences and the groups they open
/// (`{\small\em Bartol …}` → `{`, `\small`, `\em`; `T. Stelzer` → none). Lines styled alike share it.
pub(in crate::base_utilities) fn leading_markup(line: &Tokens) -> Vec<Token> {
  line
    .unlist_ref()
    .iter()
    .take_while(|t| t.get_catcode() != Catcode::LETTER)
    .filter(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::BEGIN))
    .copied()
    .collect()
}

/// `line` without the byline word "by" (in any case) that opens it, when names follow it (see [`without_leading_and`]), and the
/// spaces after it (ties and control spaces too: `By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922).
pub(in crate::base_utilities) fn without_leading_by(line: &Tokens) -> Option<Tokens> {
  let v = line.unlist_ref();
  let first = v.iter().position(|t| t.get_catcode() == Catcode::LETTER)?;
  let spacing = |t: Option<&Token>| t.is_some_and(|t| *t == T_SPACE!() || is_spacing_command(t));
  // (in any case: "by", "By", "BY")
  if !(v[first] == T_LETTER!("b") || v[first] == T_LETTER!("B"))
    || !(v.get(first + 1) == Some(&T_LETTER!("y")) || v.get(first + 1) == Some(&T_LETTER!("Y")))
    || !spacing(v.get(first + 2))
  {
    return None;
  }
  let mut rest = first + 2;
  while spacing(v.get(rest)) {
    rest += 1;
  }
  let mut out = v[..first].to_vec();
  out.extend_from_slice(&v[rest..]);
  let out = Tokens::new(out);
  (name_count(&visible_name_text(out.unlist_ref())) >= 1).then_some(out)
}

/// `line` without the word "and" its text opens with — after its leading markup, which is kept (`{\it and Zhenghan
/// Wang}` → `{\it Zhenghan Wang}`) — and the spaces after it; `None` when its first word is not "and".
pub(in crate::base_utilities) fn without_leading_and(line: &Tokens) -> Option<Tokens> {
  let v = line.unlist_ref();
  let first = v.iter().position(|t| t.get_catcode() == Catcode::LETTER)?;
  if v.get(first..first + 3) != Some(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")][..])
    || v
      .get(first + 3)
      .is_some_and(|t| t.get_catcode() == Catcode::LETTER)
  {
    return None;
  }
  let mut rest = first + 3;
  while v.get(rest) == Some(&T_SPACE!()) {
    rest += 1;
  }
  let mut out = v[..first].to_vec();
  out.extend_from_slice(&v[rest..]);
  Some(Tokens::new(out))
}

/// A name-list piece that qualifies the name before it ([`is_name_qualifier`]: `Jr.`, `PhD`, `\emph{Member, IEEE}`)
/// rejoins that name with the separator written between them, instead of becoming an author of its own; so does a
/// piece of marks and notes only ([`only_marks_and_notes`]: `~\IEEEmembership{Senior Member,~IEEE,}\thanks{…}`,
/// 1510.02728), without its separator, which separated nothing visible.
pub(in crate::base_utilities) fn rejoin_name_qualifiers(
  pieces: Vec<(Vec<Token>, Tokens)>,
) -> Vec<Tokens> {
  rejoin_name_qualifiers_keeping_separators(pieces)
    .into_iter()
    .map(|(_, name)| name)
    .collect()
}

/// [`rejoin_name_qualifiers`], each name with the separator written before it (none for the first).
pub(in crate::base_utilities) fn rejoin_name_qualifiers_keeping_separators(
  pieces: Vec<(Vec<Token>, Tokens)>,
) -> Vec<(Vec<Token>, Tokens)> {
  let mut out: Vec<(Vec<Token>, Tokens)> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let unseen = only_marks_and_notes(piece.unlist_ref());
    match out.last_mut().map(|(_, name)| name) {
      Some(name) if unseen || is_name_qualifier(&visible_name_text(piece.unlist_ref())) => {
        let mut joined = name.clone().unlist();
        if !unseen {
          // the comma's space went with the piece's trimming
          let comma = delimiter.last() == Some(&T_OTHER!(","));
          joined.extend(delimiter);
          if comma {
            joined.push(T_SPACE!());
          }
        }
        joined.extend(piece.unlist());
        *name = Tokens::new(joined);
      },
      _ => out.push((delimiter, piece)),
    }
  }
  out
}

/// Whether `tokens` hold nothing to read but marks and notes: spaces (`~`, `\ `, `\,`), a note or mark command with
/// its arguments ([`NAME_ANNOTATIONS`], `\footnotemark[…]`, the mark commands of [`mark_commands`]:
/// `\textsuperscript`, `\IEEEauthorrefmark`, `\authorrefmark`), a superscript or subscript, or math holding only those
/// (`$^{*}$`). An author written as a macro (`\A`) is something to read.
pub(in crate::base_utilities) fn only_marks_and_notes(tokens: &[Token]) -> bool {
  let mut i = 0;
  let mut any = false;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::SPACE => i += 1,
      Catcode::ACTIVE if t.with_str(|s| s == "~") => i += 1,
      Catcode::CS if t == T_CS!("\\ ") || t == T_CS!("\\,") => i += 1,
      // `${}^{1}$`'s empty group
      Catcode::BEGIN
        if tokens
          .get(i + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::END) =>
      {
        i += 2
      },
      // (a mark command too: ieeeaccess's `\authorrefmark{2}` after a list's closing period, 2201.07394)
      Catcode::CS if is_footnote_mark(&t) || is_name_annotation(&t) || is_mark_command(&t) => {
        any = true;
        i += 1;
        if tokens.get(i) == Some(&T_OTHER!("[")) {
          while i < tokens.len() && tokens[i] != T_OTHER!("]") {
            i += 1;
          }
          i += 1;
        }
        if !is_footnote_mark(&t) {
          i = skip_group(tokens, i);
        }
      },
      Catcode::SUPER | Catcode::SUB => {
        any = true;
        i = skip_group(tokens, i + 1);
      },
      Catcode::MATH => {
        let Some(close) = tokens[i + 1..]
          .iter()
          .position(|t| t.get_catcode() == Catcode::MATH)
        else {
          return false;
        };
        if !only_marks_and_notes(&tokens[i + 1..i + 1 + close])
          || tokens[i + 1..i + 1 + close].is_empty()
        {
          return false;
        }
        any = true;
        i += close + 2;
      },
      _ => return false,
    }
  }
  any
}
