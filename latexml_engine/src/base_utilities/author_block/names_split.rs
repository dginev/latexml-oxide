//! Splitting an author line into names: separators expanded, wrappers, leading spacing.

use super::*;

/// Second-level split of the author heuristic: divide ONE line that has already
/// been classified as an *author* line (its superscript marker sits far enough
/// from the front — `Some(p)` with `p >= 8` in `\lx@add@authors`) into the
/// individual creators it names.
///
/// This is guarded for affiliations by that upstream classification: an
/// *affiliation* line (marker near the front, `p < 8`) or a continuation line
/// (no marker) never reaches here — they take the other arms — so name-level
/// separators are only ever applied to author text, never to institution names.
/// That is why the literal word " and " is a separator here (it splits "Alice
/// and Bob") yet is deliberately absent from the line-level `author_affil_splits`
/// (where it would shred "Language and Intelligence").
///
/// The split proceeds by hierarchy (a separator at a wrapper's edge, or a wrapper holding only one, first moved out to
/// the line: `hoist_wrapped_separators`):
///  1. the name separators ("," and " and ", " \& ") at top level, each piece that is one wrapper holding names with
///     only marks and notes after it split into them (`names_in_wrapper`, KNOWN_PERL_ERRORS #552); if that already
///     yields more than one name we are done;
///  2. otherwise, if the whole line is a single font wrapper `\cmd{ a, b, c }`
///     (Perl's `SplitTokens` can't see the brace-hidden separators, so the
///     wrapper collapses into ONE creator that then hoards every `$^n$` marker
///     as a duplicate affiliation — arXiv 2605.00347), descend through the
///     wrapper, split its inner name list, and re-apply `\cmd` to each name so
///     every author becomes its own creator with the correct affiliation;
///  3. otherwise, if the line is one group opening with declarations whose content reads as names (`{\bf A, B}`,
///     hep-th9212083), the same with the group and its declarations.
///
/// A qualifier (`Jr.`, `PhD`, `\emph{Member, IEEE}`) or a piece of marks and notes only then rejoins the name before
/// it.
///
/// Public so a class binding whose `\authors{...}` is a comma/"and"-separated
/// name list (e.g. AGU's `agujournal2019`) reuses this canonical splitter
/// instead of re-listing the separators — the same split `\lx@add@authors`
/// applies to a superscript-marked author line.
pub fn split_author_line(line: Tokens) -> Vec<Tokens> {
  rejoin_name_qualifiers(split_author_names(line))
}

/// [`split_author_line`] for a binding whose `\author` Perl reads as one author (aastex, amsart, revtex, an IEEE
/// block, a labelled authblk `\author`): its names line splits only where it reads as two or more names
/// ([`name_count`]: `Klemens Fellner and Bao Quoc Tang`), and otherwise stays one author as written (`John Smith,
/// Max Planck Institute for Mathematics`; a name the shape test cannot read).
pub(in crate::base_utilities) fn split_author_line_cautious(line: Tokens) -> Vec<Tokens> {
  // (a byline's "by" before the names is no name: `\author{by T. Gisiger and M. B. Paranjape}`, hep-ph9507223,
  // hep-ph9404319)
  let line = without_leading_by(&line).unwrap_or(line);
  if separated_name_count(&line) >= 2 {
    split_author_line(line)
  } else {
    // (one name too opens with the "and" before it: jheppub's `\author[a,2]{and Fourth}`)
    vec![without_leading_and(&line).unwrap_or(line)]
  }
}

/// How many names an author line reads as ([`name_count`]) with its separators written as the split reads them
/// ([`spaced_name_separators`]): `{\bf Ann Able\break and Bob Baker}`, `Ann Able\thanks{…}and Bob Baker` are two, as
/// pdflatex prints them, where their text alone ("Ann Ableand Bob Baker") reads as one; so is `\parbox[t]{\textwidth}{
/// Philip F.~Hopkins\altaffilmark{1}, \&\ Jonathan Squire\altaffilmark{1}}` (1801.10166).
/// Names parted by horizontal space alone count as the split reads them too ([`space_parted_names`]: `J.
/// Wang\altaffilmark{1} Y. F. Mao \altaffilmark{1} and J. Y. Wei`, aastex 0811.3255).
fn separated_name_count(line: &Tokens) -> usize {
  name_count(&visible_name_text(
    space_parted_names(spaced_name_separators(line.clone())).unlist_ref(),
  ))
}

/// [`split_author_line`] before a qualifier rejoins its name: each piece with the separator written before it (none
/// for the first). An empty piece — between ", " and " and " in "A, B, and C" — names no one and is dropped, before a
/// wrapper is put back on the pieces: kept, it became an empty creator that took a copy of every affiliation given to
/// all the authors (`annotate=new`) and merged it onto B.
fn split_author_names(line: Tokens) -> Vec<(Vec<Token>, Tokens)> {
  // Name-level separators: comma, the literal word " and " ("Alice and Bob"), " \& " (`Zheng Zheng \& Jordi
  // Miralda-Escud\'e`, astro-ph0201275; 0908.0757) with its ties, and the `\and` family inside a wrapper; an "and" or
  // `\&` with no space token beside it (`\break and`, svmult's `and\\`) and a comma in a mark's math are written so
  // first ([`spaced_name_separators`]).
  let split = |tokens: Tokens| -> Vec<(Vec<Token>, Tokens)> {
    let mut pieces: Vec<(Vec<Token>, Tokens)> = Vec::new();
    // the separators of a dropped empty piece go on to the next (", and")
    let mut carried: Vec<Token> = Vec::new();
    let mut separators = vec![
      SplitDelim::Token(T_OTHER!(",")),
      literal_and(),
      literal_and_tie(),
      tie_and_literal(),
      tie_and_tie(),
      group_and(),
      literal_ampersand(),
      // (inside a wrapper, where the top level's `\and` split cannot reach: `{\bf {\large A \and B}}`, 2401.03555;
      // `\large{A \and B}`, 2402.15967)
      SplitDelim::Token(T_CS!("\\and")),
      SplitDelim::Token(T_CS!("\\And")),
      SplitDelim::Token(T_CS!("\\AND")),
    ];
    separators.extend(tied_ampersands());
    for (delimiter, piece) in split_tokens_delimited(tokens, separators) {
      carried.extend(delimiter);
      // (a doubled separator: the "and" after `\&` is the separator too, `Barrientos, \& and Omar López-Cruz`,
      // astro-ph9612009)
      let piece = if pieces.is_empty() {
        None
      } else {
        without_leading_and(&piece)
      }
      .unwrap_or(piece);
      if !piece.is_empty() {
        pieces.push((std::mem::take(&mut carried), piece));
      }
    }
    pieces
  };
  // a names line opening with "and" opens with the separator before its first name (mnras's `Bob Baker$^{2}$
  // \newauthor and Cat Cole$^{1}$`, 2401.11878); a byline's "By" before the names is none of them
  // (`By~~{\scshape Vitaly~Bergelson}~~and~~{\scshape Florian~K.~Richter}`, 2603.25922, 2604.12864)
  let line = without_leading_and(&line).unwrap_or(line);
  let line = without_leading_by(&line).unwrap_or(line);
  // (names parted by horizontal space alone, a wide space or a name's marks and the space after them, are parted as
  // by a comma: [`space_parted_names`])
  let line = marks_before_glued_commas(space_parted_names(spaced_name_separators(
    hoist_wrapped_separators(line),
  )));
  let top = split(line.clone());
  // a piece that is one wrapper holding names, its marks or notes after it (`\textbf{Cat Cole$^1$, Dan Dorn$^2$}\thanks{…}`,
  // 2303.16563; `\scalebox{.9}{…}`, 2412.05271; `\href{…}{A, B}`, 2312.07592): its names, each in the wrapper
  let descended: Vec<(Vec<Token>, Tokens)> = top
    .iter()
    .flat_map(|(delimiter, piece)| names_in_wrapper(delimiter, piece))
    .collect();
  if descended.len() > top.len() {
    return descended;
  }
  if top.len() > 1 {
    return top;
  }
  if let Some((cmd, inner)) = whole_line_cs_wrapper(&line) {
    // the marks after commas inside the wrapper are its names' too (`\textbf{Todd M. Tripp,\altaffilmark{2} …}`)
    let inner_split = split(marks_before_glued_commas(space_parted_names(
      spaced_name_separators(inner),
    )));
    if inner_split.len() > 1 {
      return inner_split
        .into_iter()
        .map(|(delimiter, piece)| {
          let mut v = vec![cmd, T_BEGIN!()];
          v.extend(piece.unlist());
          v.push(T_END!());
          (delimiter, Tokens::new(v))
        })
        .collect();
    }
  }
  // A line that is one group (`{\bf Daniel Boyanovsky, Da- Shin Lee}`, hep-th9212083; `{\bf Jianfei Ma$^{2}$,
  // \enspace Emmanuele Chersoni$^{2}$, …}`, 2608.16650; with no declarations, mnras's `\newauthor{…}` group in
  // 2409.07518, 2312.07625) is the same when what it holds reads as two or more names: each name in the group and its
  // declarations. A group holding one name together (`\author{{Smith, Jr., John}}`) reads as none.
  if let Some((declarations, inner)) = whole_line_group(&line)
    && separated_name_count(&inner) >= 2
  {
    let inner_split = split(marks_before_glued_commas(space_parted_names(
      spaced_name_separators(inner),
    )));
    if inner_split.len() > 1 {
      return inner_split
        .into_iter()
        .map(|(delimiter, piece)| {
          let mut v = vec![T_BEGIN!()];
          v.extend(declarations.iter().copied());
          v.extend(piece.unlist());
          v.push(T_END!());
          (delimiter, Tokens::new(v))
        })
        .collect();
    }
  }
  top
}

/// An author argument that is one parameterless macro, alone or with an empty group (`\author{\Authors}`,
/// 2401.13568; `\IEEEauthorblockN{\hpcaauthors{}}`, 2407.02944), read as TeX typesets it: expanded once, the macro's body
/// in its place, so its names and separators are seen; a body that is a `\newif` toggle through the branch TeX takes, any
/// other conditional body left whole (KNOWN_PERL_ERRORS #552).
pub(in crate::base_utilities) fn expand_sole_author_macro(stuff: Tokens) -> Tokens {
  let content: Vec<Token> = stuff
    .unlist_ref()
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  let cs = match content.as_slice() {
    [cs] => *cs,
    [cs, open, close]
      if open.get_catcode() == Catcode::BEGIN && close.get_catcode() == Catcode::END =>
    {
      *cs
    },
    _ => return stuff,
  };
  if cs.get_catcode() != Catcode::CS || lookup_conditional(&cs).is_some() {
    return stuff;
  }
  let Ok(Some(defn)) = lookup_definition(&cs) else {
    return stuff;
  };
  // (no parameter text at all: `\def\Authors.{…}` reads a delimiter TeX would demand)
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return stuff;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return stuff;
  };
  // a body holding a conditional is read through the branch a `\newif` toggle takes (`\ifanon Anonymous\else Ann Able,
  // Bob Baker\fi`, as the PDF shows it); any other is left whole: split at its commas, the `\if` and its `\else`/`\fi`
  // landed in different authors (`Error:expected:\fi`, the other branch's names leaking)
  let body = body.unlist_ref();
  let read = if body.iter().any(|t| lookup_conditional(t).is_some()) {
    match toggled_branch(body) {
      Some(branch) => branch,
      None => return stuff,
    }
  } else {
    body.to_vec()
  };
  if read.iter().any(|t| t.get_catcode() == Catcode::LETTER) {
    Tokens::new(read)
  } else {
    stuff
  }
}

/// An author line's parameterless macros that print only a name separator — `\newcommand{\authcomma}{\textmd{,}
/// \hskip0.5em}` (2403.07809), tibop-article.cls:41's `\lastand` = `, and ` (2401.13365) — written as that separator,
/// where the split reads it: hidden in the macro, the names around it were one author. The separator is written as the
/// macro prints it, a space where its body has spacing and no more (`AT\amp T Labs` stays "AT&T", `NSF\comma NIH`
/// "NSF,NIH"), and an empty group ending the macro's name (`\amp{}`) prints nothing.
pub(in crate::base_utilities) fn expand_separator_macros(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  if !v.iter().any(|t| t.get_catcode() == Catcode::CS) {
    return stuff;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut changed = false;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    i += 1;
    let Some(sep) = separator_macro_text(&t) else {
      out.push(t);
      continue;
    };
    changed = true;
    if sep.lead && out.last() != Some(&T_SPACE!()) {
      out.push(T_SPACE!());
    }
    for c in sep.text.chars() {
      out.push(match c {
        ' ' => T_SPACE!(),
        '&' => T_CS!("\\&"),
        ',' => T_OTHER!(","),
        _ => T_LETTER!(&c.to_string()),
      });
    }
    if v.get(i) == Some(&T_BEGIN!()) && v.get(i + 1) == Some(&T_END!()) {
      i += 2;
    }
    if sep.trail {
      out.push(T_SPACE!());
      while v.get(i) == Some(&T_SPACE!()) {
        i += 1;
      }
    }
  }
  if changed { Tokens::new(out) } else { stuff }
}

/// The font commands that set the upright roman text a separator prints anyway: around a separator's group they
/// print nothing of their own (`\authcomma` = `\textmd{,}\hskip0.5em`, 2403.07809).
const PLAIN_FONT_COMMANDS: [&str; 8] = [
  "\\textmd",
  "\\textup",
  "\\textrm",
  "\\textnormal",
  "\\normalfont",
  "\\rm",
  "\\rmfamily",
  "\\upshape",
];

/// The index in `body` past the glue specification starting at `i` (after `\hskip` or `\kern`): a dimension — a
/// number and a unit, two letters or `fil`, `fill`, `filll` — then `plus` and `minus` dimensions (`1em plus 1fil`).
pub(in crate::base_utilities) fn past_glue(body: &[Token], i: usize) -> usize {
  let letter_at = |j: usize, c: char| {
    body.get(j).is_some_and(|t| {
      t.get_catcode() == Catcode::LETTER && t.with_str(|s| s.len() == 1 && s.starts_with(c))
    })
  };
  let past_spaces = |mut j: usize| {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  let past_dimension = |j: usize| {
    let mut j = past_spaces(j);
    while body
      .get(j)
      .is_some_and(|u| u.with_str(|s| s.chars().all(|c| c.is_ascii_digit() || ".+-".contains(c))))
    {
      j += 1;
    }
    j = past_spaces(j);
    if letter_at(j, 'f') && letter_at(j + 1, 'i') && letter_at(j + 2, 'l') {
      j += 3;
      while letter_at(j, 'l') {
        j += 1;
      }
    } else {
      for _ in 0..2 {
        if body
          .get(j)
          .is_some_and(|u| u.get_catcode() == Catcode::LETTER)
        {
          j += 1;
        }
      }
    }
    j
  };
  let mut i = past_dimension(i);
  for keyword in ["plus", "minus"] {
    let j = past_spaces(i);
    if keyword
      .chars()
      .enumerate()
      .all(|(k, c)| letter_at(j + k, c))
    {
      i = past_dimension(j + keyword.len());
    }
  }
  i
}

/// What a separator macro prints: `text` (`,`, `and`, `&` or `, and`), and whether spacing comes before (`lead`) and
/// after it (`trail`) in the macro's body — written as one space.
struct SeparatorText {
  text:  &'static str,
  lead:  bool,
  trail: bool,
}

/// The separator a parameterless macro prints, read through its body past spacing (`\hskip0.5em`, `\quad`, ties), the
/// font commands around a group (`\textmd{,}`) and `\ignorespaces`, its letters in the case a `\MakeLowercase`,
/// `\MakeUppercase`, `\lowercase` or `\uppercase` gives its group (cimento's `\atque`); `None` for any other macro.
fn separator_macro_text(cs: &Token) -> Option<SeparatorText> {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return None;
  }
  // (an author-block delimiter is split on itself, whatever it prints: a class's `\renewcommand{\and}{{\normalfont
  // and}}`, ecca.cls:192, still parts its groups, as Perl splits the unexpanded `\and`, Base_Utility.pool.ltxml:682)
  let delimiter = |d: SplitDelim| matches!(d, SplitDelim::Token(t) if t == *cs);
  if author_group_splits()
    .into_iter()
    .chain(author_and_splits())
    .chain(author_affil_splits())
    .any(delimiter)
  {
    return None;
  }
  let defn = lookup_definition(cs).ok().flatten()?;
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return None;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return None;
  };
  let body = body.unlist_ref();
  let mut text = String::new();
  let (mut lead, mut spaced) = (false, false);
  // the case changes in force, each with the group depth it ends at (`\MakeLowercase{and\ \ignorespaces}`), and the
  // one a case command just asked of the group after it
  let mut cases: Vec<(usize, bool)> = Vec::new();
  let mut pending_case: Option<bool> = None;
  let mut depth = 0usize;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    i += 1;
    // (a case command's argument that is no group is its one token: `\MakeUppercase a`)
    let single_case = if t.get_catcode() == Catcode::BEGIN {
      None
    } else {
      pending_case.take()
    };
    let spacing = if t.get_catcode() == Catcode::BEGIN {
      depth += 1;
      if let Some(upper) = pending_case.take() {
        cases.push((depth, upper));
      }
      continue;
    } else if t.get_catcode() == Catcode::END {
      if cases.last().is_some_and(|(at, _)| *at == depth) {
        cases.pop();
      }
      depth = depth.saturating_sub(1);
      continue;
    } else if t == T_CS!("\\ignorespaces") {
      // (prints nothing: cimento.cls:1525 `\atque` = `{\normalfont\MakeLowercase{and\ \ignorespaces}}`, every shipped
      // copy; 0711.2584, 0801.3557, 1402.1844, 2109.08173, 2309.14694, hep-th0703055)
      continue;
    } else if let Some(upper) = t.with_str(|s| match s {
      "\\MakeLowercase" | "\\lowercase" => Some(false),
      "\\MakeUppercase" | "\\uppercase" => Some(true),
      _ => None,
    }) {
      // (the case its group's letters print in)
      pending_case = Some(upper);
      continue;
    } else if is_spacing(&t) {
      true
    } else if t == T_CS!("\\hskip") || t == T_CS!("\\kern") {
      i = past_glue(body, i);
      true
    } else if t == T_CS!("\\hspace") {
      if body.get(i) == Some(&T_OTHER!("*")) {
        i += 1;
      }
      i = skip_group(body, i);
      true
    } else {
      false
    };
    if spacing {
      if text.is_empty() {
        lead = true;
      } else {
        spaced = true;
      }
      continue;
    }
    if t.get_catcode() == Catcode::CS && t != T_CS!("\\&") {
      // an upright-roman font command around the separator's group (`\textmd{,}`, `\rm`) prints nothing of its own;
      // a visible one (`\textbf{\&}`, `\large`) is the separator's look, kept as written
      if !t.with_str(|s| PLAIN_FONT_COMMANDS.contains(&s)) {
        return None;
      }
      continue;
    }
    if !(t == T_CS!("\\&") || matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)) {
      // (an alignment `&`, a parameter, math: no printed separator)
      return None;
    }
    if std::mem::take(&mut spaced) {
      text.push(' ');
    }
    if t == T_CS!("\\&") {
      text.push('&');
    } else {
      t.with_str(
        |s| match single_case.or(cases.last().map(|(_, upper)| *upper)) {
          Some(true) => text.push_str(&s.to_uppercase()),
          Some(false) => text.push_str(&s.to_lowercase()),
          None => text.push_str(s),
        },
      );
    }
  }
  let text = match text.as_str() {
    "," => ",",
    "and" => "and",
    "&" => "&",
    ", and" => ", and",
    _ => return None,
  };
  Some(SeparatorText { text, lead, trail: spaced })
}

/// The branch of `body` = `\ifX … [\else …] \fi` whose `\ifX` means `\iftrue` or `\iffalse` (a `\newif` toggle) that TeX
/// takes, when that branch holds no conditional of its own; `None` for any other shape.
fn toggled_branch(body: &[Token]) -> Option<Vec<Token>> {
  let first = body.iter().position(|t| *t != T_SPACE!())?;
  let last = body.iter().rposition(|t| *t != T_SPACE!())?;
  let test = lookup_definition(&body[first]).ok().flatten()?;
  let truth = if Some(&test)
    == lookup_definition(&T_CS!("\\iftrue"))
      .ok()
      .flatten()
      .as_ref()
  {
    true
  } else if Some(&test)
    == lookup_definition(&T_CS!("\\iffalse"))
      .ok()
      .flatten()
      .as_ref()
  {
    false
  } else {
    return None;
  };
  if last <= first || lookup_conditional(&body[last]) != Some(ConditionalType::Fi) {
    return None;
  }
  let inner = &body[first + 1..last];
  let otherwise = inner
    .iter()
    .position(|t| lookup_conditional(t) == Some(ConditionalType::Else));
  let branch = match (truth, otherwise) {
    (true, Some(e)) => &inner[..e],
    (true, None) => inner,
    (false, Some(e)) => &inner[e + 1..],
    (false, None) => &[][..],
  };
  if branch.iter().any(|t| lookup_conditional(t).is_some()) {
    return None;
  }
  Some(branch.to_vec())
}

/// The names a piece of an author line holds inside one wrapper — a command over its last braced argument
/// (`\textbf{A, B}`, `\scalebox{0.9}{A, B}`, `\href{url}{A, B}`) or a group opening with declarations (`{\bf A, B}`) —
/// with only marks and notes after it: each name in the wrapper, the marks and notes after the last. Perl's
/// `SplitTokens` passes over groups (Base_Utility.pool.ltxml:152-165), so such a piece was one author holding them all.
/// A group with no declarations is a wrapper too (`{Benedikt Steinar Magnússon, Ragnar Sigurðsson}`, 2306.02486;
/// 2312.04684, 2501.04001): a piece reading as fewer than two names stays — a group holding one name together,
/// `{Smith, Jr., John}`, reads as none ("Smith" alone is no name) (KNOWN_PERL_ERRORS #552, #554).
fn names_in_wrapper(delimiter: &[Token], piece: &Tokens) -> Vec<(Vec<Token>, Tokens)> {
  let unchanged = || vec![(delimiter.to_vec(), piece.clone())];
  let v = piece.unlist_ref();
  // (spacing before the wrapper — `\ \textbf{…}`, `\vspace{0.2cm} \textbf{…}`: 2508.05004, 2512.22234 — stays with
  // the first name)
  let start = leading_spacing_end(v);
  if start >= v.len() {
    return unchanged();
  }
  let (open, close) = if v[start].get_catcode() == Catcode::CS {
    if is_footnote_mark(&v[start]) || is_name_annotation(&v[start]) || is_author_mark(&v[start]) {
      return unchanged();
    }
    let (mut i, mut last) = (start + 1, None);
    loop {
      match v.get(i) {
        Some(t) if *t == T_OTHER!("[") => match v[i..].iter().position(|u| *u == T_OTHER!("]")) {
          Some(k) => i += k + 1,
          None => return unchanged(),
        },
        Some(t) if t.get_catcode() == Catcode::BEGIN => {
          let end = skip_group(v, i);
          if end > v.len()
            || v
              .get(end - 1)
              .is_none_or(|u| u.get_catcode() != Catcode::END)
          {
            return unchanged();
          }
          last = Some((i, end - 1));
          i = end;
        },
        _ => break,
      }
    }
    match last {
      Some(group) => group,
      None => return unchanged(),
    }
  } else if v[start].get_catcode() == Catcode::BEGIN {
    let end = skip_group(v, start);
    if end > v.len()
      || v
        .get(end - 1)
        .is_none_or(|u| u.get_catcode() != Catcode::END)
    {
      return unchanged();
    }
    (start, end - 1)
  } else {
    return unchanged();
  };
  let tail = &v[close + 1..];
  // (spacing after it — `\textbf{…} \vspace{0.05in}`: 2404.00511, 2404.02905, 2408.13296, 2412.05271; between its notes,
  // with a paragraph break: `\emph{Wenzhuo Ouyang, Atilla Eryilmaz, and Ness B. Shroff} \vspace{-9pt}\par
  // \vspace{-5pt} \thanks{…}`, 1108.3768 — stays with the last)
  let marks = without_spacing(tail);
  // (the list's closing punctuation before them: `\uppercase{…, and Cat Cole}.$^{2}$`, 2201.07394)
  let marks = match marks.first() {
    Some(t) if *t == T_OTHER!(".") || *t == T_OTHER!(";") => &marks[1..],
    _ => &marks[..],
  };
  if !(marks.iter().all(|t| *t == T_SPACE!()) || only_marks_and_notes(marks)) {
    return unchanged();
  }
  let inner = &v[open + 1..close];
  // A box that only lays the names out (`\parbox[c]{\textwidth}{\centering A, B}`, 2504.03146; `\parbox{\textwidth}{A,
  // and B}`, mn2e 1205.5573) is no styling of theirs: each name stands without it, its paragraph layout (`\centering`)
  // left out with it, as a parbox per name a text width wide would lay them out anew. Its other opening declarations
  // style every name (`\mbox{\large A, B}`, `\parbox{…}{\centering\bfseries A, B}`): each name in a group of them.
  let layout_box = open != start && LAYOUT_BOXES.iter().any(|cs| v[start] == T_CS!(*cs));
  let declarations = if open == start || layout_box {
    opening_declarations(&Tokens::new(inner.to_vec()))
  } else {
    Vec::new()
  };
  let names = &inner[declarations.len()..];
  // (a layout box's styling: its declarations but its paragraph layout)
  let box_style: Vec<Token> = if layout_box {
    let style: Vec<Token> = declarations
      .iter()
      .copied()
      .filter(|t| !PARAGRAPH_LAYOUT.iter().any(|cs| *t == T_CS!(*cs)))
      .collect();
    if style.iter().all(|t| *t == T_SPACE!()) {
      Vec::new()
    } else {
      style
    }
  } else {
    Vec::new()
  };
  if separated_name_count(&Tokens::new(names.to_vec())) < 2 {
    return unchanged();
  }
  // a piece of marks and notes only is the name's before it, as after the wrapper (`Chia-Hsun Chuang,\thanks{…}
  // \thanks{…} and Yun Wang`, 1205.5573; `{Ann Able},~\IEEEmembership{Senior Member, IEEE},`, 2112.09769, 2603.10769)
  let pieces = notes_to_the_name_before(split_author_names(Tokens::new(names.to_vec())));
  if pieces.len() < 2
    || pieces
      .iter()
      .any(|(_, name)| visible_name_text(name.unlist_ref()).trim().is_empty())
  {
    return unchanged();
  }
  let declarations = if layout_box { Vec::new() } else { declarations };
  let count = pieces.len();
  pieces
    .into_iter()
    .enumerate()
    .map(|(k, (inner_delimiter, name))| {
      let mut wrapped = if layout_box {
        if k == 0 {
          v[..start].to_vec()
        } else {
          Vec::new()
        }
      } else if k == 0 {
        v[..=open].to_vec()
      } else {
        v[start..=open].to_vec()
      };
      if !box_style.is_empty() {
        wrapped.push(T_BEGIN!());
        wrapped.extend(box_style.iter().copied());
      }
      wrapped.extend(declarations.iter().copied());
      wrapped.extend(name.unlist());
      if !box_style.is_empty() {
        wrapped.push(T_END!());
      }
      if !layout_box {
        wrapped.push(v[close]);
      }
      if k + 1 == count {
        wrapped.extend_from_slice(tail);
      }
      (
        if k == 0 {
          delimiter.to_vec()
        } else {
          inner_delimiter
        },
        Tokens::new(wrapped),
      )
    })
    .collect()
}

/// Where a piece's leading spacing ends: spaces, ties, space commands (`\ `, `\quad`) and `\vspace`/`\hspace` with
/// their length.
pub(in crate::base_utilities) fn leading_spacing_end(v: &[Token]) -> usize {
  let mut i = 0;
  while i < v.len() {
    if is_spacing(&v[i]) || PARAMETERLESS_SPACING.iter().any(|cs| v[i] == T_CS!(*cs)) {
      i += 1;
    } else if v[i] == T_CS!("\\penalty") {
      // an integer follows (tex.web §1103 `\penalty` scans an int: `\penalty10000`, `\penalty -50`)
      let mut j = i + 1;
      while v
        .get(j)
        .is_some_and(|t| *t == T_SPACE!() || *t == T_OTHER!("-") || *t == T_OTHER!("+"))
      {
        j += 1;
      }
      while v.get(j).is_some_and(|t| {
        t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit()))
      }) {
        j += 1;
      }
      i = j;
    } else if ["\\vskip", "\\hskip", "\\kern"]
      .iter()
      .any(|cs| v[i] == T_CS!(*cs))
    {
      // a glue or dimension follows (tex.web §1057, §1061: `\kern2pt`, `\vskip 1em plus 1fil`)
      i = past_glue(v, i + 1);
    } else if v[i] == T_CS!("\\rule") {
      // latex.ltx:16359 `\rule[raise]{width}{height}`, a rule box, no glyph
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("[")) {
        while j < v.len() && v[j] != T_OTHER!("]") {
          j += 1;
        }
        j += 1;
      }
      for _ in 0..2 {
        if v.get(j).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
          break;
        }
        j = skip_group(v, j).min(v.len());
      }
      i = j;
    } else if v[i] == T_CS!("\\vspace") || v[i] == T_CS!("\\hspace") || v[i] == T_CS!("\\addvspace")
    {
      let mut j = i + 1;
      if v.get(j) == Some(&T_OTHER!("*")) {
        j += 1;
      }
      while v.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      // (only with its braced length: `\vspace\baselineskip` stops here, its length no wrapper)
      if v.get(j).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
        break;
      }
      i = skip_group(v, j).min(v.len());
    } else {
      break;
    }
  }
  i
}

/// The boxes that lay a names line out without styling it ([`names_in_wrapper`]): latex.ltx `\parbox[pos]{width}{…}`,
/// `\mbox{…}`, `\makebox[width][pos]{…}`, and the primitive `\hbox{…}`.
const LAYOUT_BOXES: [&str; 4] = ["\\parbox", "\\mbox", "\\makebox", "\\hbox"];

/// The paragraph-layout declarations a layout box's content may open with, which go with the box.
const PARAGRAPH_LAYOUT: [&str; 5] = [
  "\\centering",
  "\\raggedright",
  "\\raggedleft",
  "\\noindent",
  "\\par",
];

/// `tokens` without the spacing ([`leading_spacing_end`]) and paragraph breaks in them, which print nothing.
fn without_spacing(tokens: &[Token]) -> Vec<Token> {
  let mut out = Vec::with_capacity(tokens.len());
  let mut i = 0;
  while i < tokens.len() {
    let past = i + leading_spacing_end(&tokens[i..]);
    if past > i {
      i = past;
    } else {
      if tokens[i] != T_CS!("\\par") {
        out.push(tokens[i]);
      }
      i += 1;
    }
  }
  out
}

/// Name-list pieces with each piece of marks and notes only ([`only_marks_and_notes`]) joined to the piece before it,
/// without the separator written between them, which separated nothing visible; and a line break at a piece's edge
/// (`{Bob Baker},~\IEEEmembership{Member, IEEE},\\ {Cat Cole}`, 2112.09769) left out, as the layout between two names.
fn notes_to_the_name_before(pieces: Vec<(Vec<Token>, Tokens)>) -> Vec<(Vec<Token>, Tokens)> {
  let mut out: Vec<(Vec<Token>, Tokens)> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let v = piece.unlist_ref();
    let mut start = 0;
    let mut end = v.len();
    loop {
      let past = separator_gap_end(&v[..end], start, false);
      if past == start {
        break;
      }
      start = past;
    }
    while end > start
      && (is_spacing(&v[end - 1])
        || SEPARATOR_LINE_BREAKS
          .iter()
          .any(|cs| v[end - 1] == T_CS!(*cs)))
    {
      end -= 1;
    }
    let trimmed = &v[start..end];
    match out.last_mut() {
      Some((_, name)) if only_marks_and_notes(trimmed) => {
        let mut joined = name.clone().unlist();
        joined.extend_from_slice(trimmed);
        *name = Tokens::new(joined);
      },
      _ => out.push((delimiter, Tokens::new(trimmed.to_vec()))),
    }
  }
  out
}

/// A name separator written at the edge of a wrapper is the line's (`\textbf{Hang Xu$^{2}$,}`, 2310.10477;
/// `\textbf{, Jinyang Guo}`, 2505.02214), and a wrapper holding only one (`\textbf{,}`, 2403.11004; `{\rm and}`,
/// 2401.03205) is that separator: the font of a comma or an "and" is not a name's (KNOWN_PERL_ERRORS #552).
pub(in crate::base_utilities) fn hoist_wrapped_separators(line: Tokens) -> Tokens {
  let v = line.unlist();
  let and = [T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")];
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut in_math = false;
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    // (a separator in math or in a script's argument is the formula's: `$^{\dagger, 1}$`, 2301.11477)
    if t.get_catcode() == Catcode::MATH {
      in_math = !in_math;
    }
    // (nor is a group after a command it does not wrap: a mark's argument, `\textsuperscript{\faRuler,\faBalanceScale}`,
    // 2503.01996)
    let script = t.get_catcode() == Catcode::BEGIN
      && out
        .last()
        .is_some_and(|p| matches!(p.get_catcode(), Catcode::SUPER | Catcode::SUB | Catcode::CS));
    if in_math || t.get_catcode() == Catcode::MATH || script {
      let end = if t.get_catcode() == Catcode::BEGIN {
        skip_group(&v, i).min(v.len())
      } else {
        i + 1
      };
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    }
    let command = t.get_catcode() == Catcode::CS
      && v
        .get(i + 1)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
      && !is_footnote_mark(&t)
      && !is_name_annotation(&t)
      && !is_author_mark(&t);
    let open = if command {
      i + 1
    } else if t.get_catcode() == Catcode::BEGIN {
      i
    } else {
      out.push(t);
      i += 1;
      continue;
    };
    let end = skip_group(&v, open);
    if end > v.len()
      || v
        .get(end - 1)
        .is_none_or(|u| u.get_catcode() != Catcode::END)
    {
      out.push(t);
      i += 1;
      continue;
    }
    let close = end - 1;
    let inner = &v[open + 1..close];
    let declarations = if command {
      Vec::new()
    } else {
      opening_declarations(&Tokens::new(inner.to_vec()))
    };
    if !command && declarations.iter().all(|u| *u == T_SPACE!()) {
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    }
    let body: Vec<Token> = inner[declarations.len()..].to_vec();
    // (a tie or a space command beside the separator is spacing: `\textbf{Jiaming Ji$^{\diamond}$,~}`, 2505.20214;
    // `\textbf{Yuanpei Chen$^5$, \quad}`, 2407.15815)
    let spacing = is_spacing;
    let first = body.iter().position(|u| !spacing(u));
    let last = body.iter().rposition(|u| !spacing(u));
    let (Some(first), Some(last)) = (first, last) else {
      out.extend_from_slice(&v[i..end]);
      i = end;
      continue;
    };
    let trimmed = &body[first..=last];
    let rewrap = |out: &mut Vec<Token>, content: &[Token]| {
      out.extend_from_slice(&v[i..=open]);
      out.extend(declarations.iter().copied());
      out.extend_from_slice(content);
      out.push(v[close]);
    };
    if trimmed == [T_OTHER!(",")] {
      out.extend([T_OTHER!(","), T_SPACE!()]);
    } else if trimmed == and {
      out.push(T_SPACE!());
      out.extend(and);
      out.push(T_SPACE!());
    } else if trimmed.len() > 1 && trimmed[trimmed.len() - 1] == T_OTHER!(",") {
      rewrap(&mut out, &trimmed[..trimmed.len() - 1]);
      out.push(T_OTHER!(","));
    } else if trimmed.len() > 1 && trimmed[0] == T_OTHER!(",") {
      out.extend([T_OTHER!(","), T_SPACE!()]);
      rewrap(&mut out, &trimmed[1..]);
    } else if trimmed.len() > 4 && trimmed[..3] == and && spacing(&trimmed[3]) {
      // (the word "and" opening it, a word of its own: `\mbox{and {Thomas~P.} Krichbaum $^{5}$}`, mdpi 2108.09054)
      let rest = trimmed[3..]
        .iter()
        .position(|u| !spacing(u))
        .map_or(trimmed.len(), |k| 3 + k);
      if is_name_qualifier(&visible_name_text(&trimmed[rest..])) {
        // (…but not before what names no one, which stays with the name before it in its wrapper's font: `Ann Able
        // \emph{and others}`, 77n r2 review)
        out.extend_from_slice(&v[i..end]);
      } else {
        out.push(T_SPACE!());
        out.extend(and);
        out.push(T_SPACE!());
        rewrap(&mut out, &trimmed[rest..]);
      }
    } else if trimmed.len() > 4
      && trimmed[trimmed.len() - 3..] == and
      && spacing(&trimmed[trimmed.len() - 4])
    {
      let name_end = trimmed[..trimmed.len() - 3]
        .iter()
        .rposition(|u| !spacing(u))
        .map_or(0, |k| k + 1);
      rewrap(&mut out, &trimmed[..name_end]);
      out.push(T_SPACE!());
      out.extend(and);
      out.push(T_SPACE!());
    } else {
      out.extend_from_slice(&v[i..end]);
    }
    i = end;
  }
  Tokens::new(out)
}
