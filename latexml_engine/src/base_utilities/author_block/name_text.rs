//! The visible text of a name: name words, particles, qualifiers, name groups and merged creators.

use super::*;

/// Whether a name's last word is one its period belongs to: an initial (a single capital, `K`) or a suffix written with
/// a period (`Jr`, `Esq`, [`NAME_SUFFIXES`]). A two-letter word is a surname as often as an initial (`Wei Li.`, `Andrew
/// Ng.`: sentence punctuation), so its period goes, as Perl's.
pub(in crate::base_utilities) fn ends_with_abbreviation(name: &str) -> bool {
  let Some(word) = name.split_whitespace().next_back() else {
    return false;
  };
  let mut chars = word.chars();
  let initial = chars.next().is_some_and(char::is_uppercase) && chars.next().is_none();
  // (and "et al." ending a list, `is_name_qualifier`)
  initial || NAME_SUFFIXES.contains(&format!("{word}.").as_str()) || word == "al" || word == "et.al"
}

/// Whether `text` is a professional membership grade as an author writes it after the name and its comma: the words of
/// one ([`grade_words`]: `Fellow, IEEE`, `Senior Member, IEEE`, `Member, OSA`), or an abbreviated grade before its
/// society (`M.ASCE`, `A. M. ASCE`; [`NAME_GRADE_ABBREVIATIONS`]).
pub(in crate::base_utilities) fn is_membership_grade(text: &str) -> bool {
  let text = strip_glued_marks(text);
  let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
  grade_words(text, true)
    || NAME_GRADE_ABBREVIATIONS.iter().any(|grade| {
      compact
        .strip_prefix(grade)
        .is_some_and(|society| NAME_GRADE_SOCIETIES.contains(&society))
    })
}

/// A name with the membership grade written after its comma taken out as the author's `membership` contact: `Cat
/// Cole\textsuperscript{1}, \textit{Fellow, IEEE}` is "Cat Cole" and the contact "Fellow, IEEE" in its italics, the
/// contact IEEEtran's `\IEEEmembership` makes of the grade it prints (ieeetran_cls.rs `\lx@IEEE@membership`;
/// 2107.05093, 2302.11403, 2602.16174), and so are `Bob Baker, Member, OSA` and ascelike's `Matthew R. Kuhn, M.ASCE`
/// (ascexmpl.tex:48). The grade is the text after the first comma of the name's own (outside its groups and math) after
/// which only a grade is printed ([`is_membership_grade`]); the marks and notes written after it stay the name's, after
/// the contact, as they print after the grade. `None` when the name ends in no such grade (`John Smith, Jr.`, `Jane
/// Doe, MD, PhD` qualify the name).
pub(in crate::base_utilities) fn written_membership_out(name: &[Token]) -> Option<Vec<Token>> {
  let (mut depth, mut in_math, mut found) = (0, false, None);
  let commas: Vec<usize> = (name.iter().enumerate())
    .filter_map(|(c, t)| {
      depth += t.group_step();
      in_math ^= t.get_catcode() == Catcode::MATH;
      (depth == 0 && !in_math && *t == T_OTHER!(",")).then_some(c)
    })
    .collect();
  // (a grade is a suffix of grade words, so its comma is read from the last leftward, the first tail holding a word no grade
  // has ending the reading: every tail before it holds that word too. The first comma after an alphabetic head opening a
  // grade is the one read from the first comma on, each tail read once — not the rest of the name per comma, quadratic on
  // Perl's whole-block creator of a block of 418 names, 1507.02068)
  for c in commas.into_iter().rev() {
    let (grade, annotations) = grade_and_annotations(&name[c + 1..]);
    let text = visible_name_text(&grade);
    if is_membership_grade(&text)
      && visible_name_text(&name[..c])
        .chars()
        .any(char::is_alphabetic)
    {
      found = Some((c, grade, annotations));
    } else if text.chars().any(char::is_alphanumeric)
      && !grade_words(strip_glued_marks(&text), false)
    {
      break;
    }
  }
  // (one grade, the line's last: a names line printing a grade before it holds a name after a grade, `Ann Able,
  // \textit{Member, IEEE}; Bob Baker, \textit{Fellow, IEEE}`, whose grades all stay — taking out the last alone would
  // leave the names before it one creator: 1510.03162, 2507.19818)
  let (c, grade, annotations) = found.filter(|(c, ..)| !holds_grade(&name[..*c]))?;
  let head = &name[..c];
  let end = head
    .iter()
    .rposition(|t| !is_spacing(t))
    .map_or(0, |k| k + 1);
  let mut out = head[..end].to_vec();
  out.extend(TokenizeInternal!(r"\lx@add@contact[role=membership]").unlist());
  out.push(T_BEGIN!());
  out.extend(grade);
  out.push(T_END!());
  out.extend(annotations);
  Some(out)
}

/// Whether a names line prints a membership grade or a part of one ([`is_membership_part`]) between its names'
/// separators — a comma, a semicolon, an "and".
fn holds_grade(line: &[Token]) -> bool {
  visible_name_text(line)
    .replace('~', " ")
    .split([',', ';'])
    .flat_map(|piece| piece.split(" and "))
    .map(str::trim)
    .any(|piece| !piece.is_empty() && is_membership_part(piece))
}

/// Where the name line a creator's content holds begins and ends: inside the block reader's `\lx@author@withsup{…}` and
/// a command the name line is set in as its one argument (a raw IEEE class's `\IEEEauthorblockN{…}`, which the kernel's
/// split puts round each name: icrc2009, 0907.0548, 1111.2718), each holding the whole content; the whole content
/// otherwise.
pub(in crate::base_utilities) fn name_line_span(v: &[Token]) -> (usize, usize) {
  let (mut start, mut end) = (0, v.len());
  for _ in 0..2 {
    while end > start && v[end - 1] == T_SPACE!() {
      end -= 1;
    }
    if end - start < 3 {
      break;
    }
    let cs = v[start];
    if cs.get_catcode() != Catcode::CS
      || v[start + 1].get_catcode() != Catcode::BEGIN
      || skip_group(&v[..end], start + 1) != end
      || (cs != T_CS!("\\lx@author@withsup")
        && (is_name_annotation(&cs)
          || is_footnote_mark(&cs)
          || is_author_mark(&cs)
          || cs.with_str(|n| n.starts_with("\\lx@"))))
    {
      break;
    }
    start += 2;
    end -= 1;
  }
  (start, end)
}

/// A creator's content as `\lx@add@author` reads it (frontmatter_fields.rs): the membership grade its name line writes
/// after the name's comma the author's contact ([`written_membership_out`]), inside what the line is set in
/// ([`name_line_span`]) as in a bare name.
pub(in crate::base_utilities) fn membership_read_out(content: Tokens) -> Tokens {
  let v = content.unlist_ref();
  let (start, end) = name_line_span(v);
  match written_membership_out(&v[start..end]) {
    Some(name) => {
      let mut out = v[..start].to_vec();
      out.extend(name);
      out.extend_from_slice(&v[end..]);
      Tokens::new(out)
    },
    None => content,
  }
}

/// What follows a name's comma parted into the text it prints, its spacing trimmed, and the marks and notes written in
/// it (a note or mark command with its arguments, [`is_name_annotation`], [`is_footnote_mark`], [`is_mark_command`]; a
/// formula, `$^{1}$`; a block reader's contact call after the name, `\lx@add@affiliation[annotate=3]{…}`: 1611.04834's
/// `and Warren J. Gross, \emph{Senior Member, IEEE}\\ Department …`).
fn grade_and_annotations(tail: &[Token]) -> (Vec<Token>, Vec<Token>) {
  let kernel_call = |t: &Token| t.with_str(|s| s.starts_with("\\lx@add@"));
  let (mut text, mut annotations, mut i) = (Vec::new(), Vec::new(), 0);
  while i < tail.len() {
    let t = tail[i];
    let start = i;
    i += 1;
    if t.get_catcode() == Catcode::MATH {
      while i < tail.len() && tail[i].get_catcode() != Catcode::MATH {
        i += 1;
      }
      i = (i + 1).min(tail.len());
      annotations.extend_from_slice(&tail[start..i]);
    } else if t.get_catcode() == Catcode::CS
      && (is_name_annotation(&t) || is_footnote_mark(&t) || is_mark_command(&t) || kernel_call(&t))
    {
      if tail.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = tail[i..].iter().position(|u| *u == T_OTHER!("]"))
      {
        i += close + 1;
      }
      if kernel_call(&t) {
        while tail
          .get(i)
          .is_some_and(|u| u.get_catcode() == Catcode::BEGIN)
        {
          i = skip_group(tail, i).min(tail.len());
        }
      } else if !is_footnote_mark(&t) {
        i = skip_group(tail, i).min(tail.len());
      }
      annotations.extend_from_slice(&tail[start..i]);
    } else if t.get_catcode() == Catcode::BEGIN {
      i = skip_group(tail, start).min(tail.len());
      text.extend_from_slice(&tail[start..i]);
    } else {
      text.push(t);
    }
  }
  let from = text
    .iter()
    .position(|t| !is_spacing(t))
    .unwrap_or(text.len());
  let to = text
    .iter()
    .rposition(|t| !is_spacing(t))
    .map_or(from, |k| k + 1);
  (text[from..to].to_vec(), annotations)
}

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
