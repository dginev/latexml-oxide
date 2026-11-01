//! A collaboration or team named in an author line: a `<contact role="collaboration">` on the names it covers, no
//! creator of its own and no part of a name (FIDELITY_CHARTER §4, user ruling 2026-11-01; OXIDIZED_DESIGN_DIVERGENCES
//! #946). The contact is the one Perl's aastex binding makes of `\collaboration{n}{name}`: `\lx@add@contact[role=
//! collaboration,annotate=new]` (aas_support.sty.ltxml:126), on the authors before it that carry none — here those of
//! the author call's `\and` group, `annotate=new:N`.

use super::*;

/// The command a names-line piece read as a collaboration is wrapped in ([`collaboration_split`]), for the author block's
/// reader to make its contact ([`Collaborations`]); `\lx@author@collaboration[k]{…}` in a group's lines, `k` the names
/// of the group after it. Defined as its text (`author_macros.rs`), so a reader that does not know it prints the piece
/// where it stood.
pub(in crate::base_utilities) fn collaboration_wrapper() -> Token {
  T_CS!("\\lx@author@collaboration")
}

/// What frames a collaboration's name on the line ([`collaboration_core`]): nothing, a bare "the" before it, or more —
/// "for", "on behalf of", "representing", brackets, dashes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(in crate::base_utilities) enum Framing {
  /// the name alone: `\textit{WIDE Consortium}` (2501.13770), `\center BESIII Collaboration` (2212.06489)
  Unframed,
  /// a bare "the": `the ATLAS Collaboration`, `The Robotics Group`
  The,
  /// "for", "on behalf of", "representing", brackets or dashes: `On Behalf Of The ATLAS Collaboration` (0910.2149),
  /// `Representing the BES Collaboration` (hep-ex0011032), `(XYZ Collaboration)`, `-- QCDSF-UKQCD Collaboration --`
  Framed,
}

/// `text` without what frames a collaboration's name — a list's "and", brackets, dashes (`-- QCDSF-UKQCD Collaboration
/// --`, hep-ph0502212), "for", "on behalf of", "representing", "the" — and what framed it ([`Framing`]); and whether
/// brackets held a name opening with "The" (`(The Nearby Supernova Factory)`, 1511.01102).
fn collaboration_core(text: &str) -> (String, Framing, bool) {
  let dash = |c: char| matches!(c, '-' | '–' | '—');
  let mut t = text.trim().to_string();
  let (mut framing, mut the_in_brackets) = (Framing::Unframed, false);
  loop {
    let before = t.clone();
    for p in ["and ", "And ", "& "] {
      if let Some(rest) = t.strip_prefix(p) {
        t = rest.trim_start().to_string();
      }
    }
    if t.len() >= 2
      && ((t.starts_with('(') && t.ends_with(')')) || (t.starts_with('[') && t.ends_with(']')))
    {
      t = t[1..t.len() - 1].trim().to_string();
      framing = Framing::Framed;
      the_in_brackets |= t.get(..4).is_some_and(|p| p.eq_ignore_ascii_case("the "));
    }
    if t.starts_with(dash) && t.ends_with(dash) {
      t = t.trim_matches(dash).trim().to_string();
      framing = Framing::Framed;
    }
    for p in ["for ", "on behalf of ", "representing ", "the "] {
      if t.get(..p.len()).is_some_and(|q| q.eq_ignore_ascii_case(p)) {
        t = t[p.len()..].trim_start().to_string();
        framing = framing.max(if p == "the " {
          Framing::The
        } else {
          Framing::Framed
        });
        break;
      }
    }
    if t == before {
      break;
    }
  }
  (t, framing, the_in_brackets)
}

/// A word without the punctuation around it, lowercased (as [`word_forms`] reads it).
fn bare_word(word: &str) -> String {
  word
    .trim_matches(|c: char| ".,;:()[]".contains(c))
    .to_lowercase()
}

/// Whether a word names a collaboration, a team or a group ([`COLLABORATION_WORDS`], [`TEAM_WORDS`]).
fn collaboration_word(word: &str) -> bool {
  let bare = bare_word(word);
  COLLABORATION_WORDS.contains(&bare.as_str()) || TEAM_WORDS.contains(&bare.as_str())
}

/// Whether the visible text of an author-line piece names a collaboration, a team or a consortium and no person: its
/// name ends with a collaboration word ([`COLLABORATION_WORDS`]: `XYZ Collaboration`, `for the ATLAS, CDF, CMS and D0
/// Collaborations`, 1311.7305) or with a team word ([`TEAM_WORDS`]) the line frames — "the", "for", "on behalf of",
/// brackets: `and the GAPS Team`, 1708.04166 — or a names line lists with "and" (`listed`: `and LHD experimental group`,
/// 1711.07097), and no word before it names an institution (`The ATLAS Collaboration Office`, `Theory Group, Department
/// of Physics` stay affiliations); or brackets hold a name opening with "The" that no word names an institution in
/// (`(The Nearby Supernova Factory)`, 1511.01102). A name the collaboration follows is a name and its remark
/// ([`collaboration_split`]), no collaboration: `Bob Baker for the ATLAS Collaboration`; nor is a surname holding such a
/// word (`M.~Huertas-Company`, 1212.4143: "Company" no collaboration's).
pub fn reads_as_collaboration(text: &str, listed: bool) -> bool {
  collaboration_framing(text, listed, false).is_some()
}

/// Whether a line of its own under or over the names names a collaboration ([`reads_as_collaboration`]), and how its
/// name is framed: a team word's only by more than a bare "the" (`for the GAPS Team`, `(The Robotics Group)`) — `The
/// Robotics Group`, `the Applied Mathematics Group` under a name are its institution's, an affiliation.
pub fn collaboration_line_framing(text: &str) -> Option<Framing> {
  collaboration_framing(text, false, true)
}

/// [`reads_as_collaboration`] (`line` false) and [`collaboration_line_framing`] (`line` true): how the collaboration a
/// piece or line names is framed, `None` when it names none.
fn collaboration_framing(text: &str, listed: bool, line: bool) -> Option<Framing> {
  if text.contains('@') {
    return None;
  }
  let (core, framing, the_in_brackets) = collaboration_core(text);
  let lower = core.to_lowercase();
  if [" for ", " on behalf of "]
    .iter()
    .any(|p| lower.contains(p))
    || core.contains(['(', '['])
  {
    return None;
  }
  let words: Vec<&str> = core.split_whitespace().collect();
  let last = words.last()?;
  if words.len() > 12 {
    return None;
  }
  let before = &words[..words.len() - 1];
  if before
    .iter()
    .any(|w| non_name_word(w) && !collaboration_word(w))
  {
    return None;
  }
  // (a name, not a sentence or a person's: a lowercase word only right before the collaboration word, `LHD experimental
  // group` — not `Monica Brigida\\on behalf of the \emph{Fermi}-LAT Collaboration`, whose line break the text does
  // not part, 0907.0548)
  if before.iter().enumerate().any(|(at, w)| {
    !["and", "of", "the", "&"].contains(&bare_word(w).as_str())
      && w.chars().next().is_some_and(char::is_lowercase)
      && at + 1 != before.len()
  }) {
    return None;
  }
  let head = bare_word(last);
  if COLLABORATION_WORDS.contains(&head.as_str()) {
    return Some(framing);
  }
  if TEAM_WORDS.contains(&head.as_str()) {
    let framed = if line {
      framing == Framing::Framed
    } else {
      framing != Framing::Unframed
    };
    return (framed || listed).then_some(framing);
  }
  (the_in_brackets && !before.is_empty() && !words.iter().any(|w| non_name_word(w)))
    .then_some(framing)
}

/// Whether a piece of a names line names a collaboration beside the names ([`reads_as_collaboration`]) the way a line
/// lists one: framed — "the", "for", "on behalf of", brackets — or listed with "and" (`listed`). A piece the commas alone
/// part from the names is no collaboration of theirs: revtex's `\collaboration{Fermilab Lattice, HPQCD, and MILC
/// Collaborations}` (2412.18491, its `\author{#1}`) one collaboration, no name "Fermilab Lattice".
pub fn reads_as_listed_collaboration(text: &str, listed: bool) -> bool {
  reads_as_collaboration(text, listed)
    && (listed || collaboration_core(text).1 != Framing::Unframed)
}

/// Whether a collaboration's text ends with a colon, a heading over the names after it (`Tam\'as Cs\"org\H{o}$^{4}$,
/// for the TOTEM Collaboration:\\ G.~Antchev …`, 1204.5689; `JLQCD Collaboration: S.~Aoki$^1$, …`, hep-lat0212039).
pub fn collaboration_heads(text: &str) -> bool { text.trim_end().ends_with(':') }

/// Whether a line of an author block, a line of its own, names a collaboration ([`reads_as_collaboration`], a team word
/// framed): `(XYZ Collaboration)` under the names (2410.11607), `on behalf of the LHCb collaboration` (1905.11825).
pub(in crate::base_utilities) fn collaboration_line(line: &Tokens) -> bool {
  collaboration_line_reading(line).is_some()
}

/// How the collaboration a line of an author block names is framed ([`collaboration_line_framing`]), `None` for a line
/// naming none or marked.
pub(in crate::base_utilities) fn collaboration_line_reading(line: &Tokens) -> Option<Framing> {
  if author_mark_position(line).is_some() {
    return None;
  }
  collaboration_line_framing(&visible_name_text(line.unlist_ref()))
}

/// Whether a line of an author block that names a collaboration unframed ([`Framing::Unframed`]) is an institution,
/// the line under it (`next`) its address going on — a place, no e-mail, link, mark or collaboration of its own: an
/// employer's line (`\textit{WIDE Consortium}\\ \.Zabbar\\ Malta`, 2501.13770, as the next column's `\textit{ACURRAENT
/// UG}\\ Frankfurt (Oder)\\ Germany`), an affiliation as before. A framed one is the collaboration whatever follows
/// (`On Behalf Of The ATLAS Collaboration \\ University of Wisconsin`, 0910.2149).
pub(in crate::base_utilities) fn collaboration_line_before(
  line: &Tokens,
  next: Option<&Tokens>,
) -> bool {
  match collaboration_line_reading(line) {
    None => false,
    Some(Framing::Unframed) => !next.is_some_and(|next| {
      author_mark_position(next).is_none()
        && !prints_nothing(next)
        && !line_is_email(next)
        && !is_link_line(next)
        && !collaboration_line(next)
    }),
    Some(_) => true,
  }
}

/// A line of an author block ending in "and" or "&" and the line under it, joined when together they name one
/// collaboration ([`collaboration_line`]): `the \textit{Fermi} Pulsar Search Consortium and \\ the \textit{Fermi} Pulsar
/// Timing Consortium` (1210.5341) is one collaboration, not the affiliation "… Search Consortium and" beside the
/// collaboration "… Timing Consortium". The line break prints as a space.
pub(in crate::base_utilities) fn joined_collaboration(
  line: &Tokens,
  next: &Tokens,
) -> Option<Tokens> {
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim_end();
  let ends_with_and = text == "and"
    || text.ends_with(" and")
    || text.ends_with('&')
    || line.unlist_ref().iter().rev().find(|t| !is_spacing(t)) == Some(&T_CS!("\\&"));
  if !ends_with_and || author_mark_position(next).is_some() {
    return None;
  }
  let mut v = line.unlist_ref()[..spacing_end(line.unlist_ref())].to_vec();
  v.push(T_SPACE!());
  let next = next.unlist_ref();
  v.extend(next[leading_spacing_end(next)..].iter().copied());
  let joined = Tokens::new(v);
  collaboration_line(&joined).then_some(joined)
}

/// Where a heading's collaboration goes in a name's tokens ([`Collaborations::author`]): before the first note or mark
/// command the name carries, as the heading prints above the name and its notes' marks (`G.~Antchev\footnote{INRNE-BAS,
/// …}` under `for the TOTEM Collaboration:`, 1204.5689), and after the name's words, which the readers that place a
/// later address by the name's words read first (iopart's `\ead{awest@…}` after `\author{ACME Collaboration: … A~D~West$^3$,
/// E~P~West$^1$}`, 1612.09318: a contact before the name read as its first words, the address no one's; after them
/// its words are no name's, [`visible_name_text`] reading no `\lx@add@contact`).
pub(in crate::base_utilities) fn heading_place(name: &[Token]) -> usize {
  let mut depth = 0i32;
  for (at, t) in name.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::CS if depth == 0 && (is_name_annotation(t) || is_footnote_mark(t)) => return at,
      _ => {},
    }
  }
  name.len()
}

/// Whether a names-line piece reads as one person's name: by its shape, or marked after the name and no institution or
/// address, as the cautious split reads a list by its marks (`marked_names`).
fn reads_as_one_name(piece: &Tokens) -> bool {
  let text = visible_name_text(piece.unlist_ref());
  name_count(&text) == 1
    || (marked_after_name(piece) && !reads_as_affiliation(piece) && !text.contains('@'))
}

/// Whether a names-line separator joins with "and" (`and`, `\&`).
fn joined_by_and(delimiter: &[Token]) -> bool {
  let letters: String = delimiter
    .iter()
    .filter(|t| t.get_catcode() == Catcode::LETTER)
    .map(Token::to_string)
    .collect();
  letters == "and" || delimiter.iter().any(|t| *t == T_CS!("\\&"))
}

/// Whether a piece is one more collaboration's acronym listed before the collaboration word (`ATLAS, CDF, CMS and D0
/// Collaborations`): one word of capitals and digits, or two capitals at least (`LHCb`), no institution's.
fn listed_acronym(text: &str) -> bool {
  let w = text.trim().trim_matches(|c: char| ".,;:".contains(c));
  !w.is_empty()
    && !w.contains(char::is_whitespace)
    && w.chars().all(|c| c.is_alphanumeric() || c == '-')
    && (w
      .chars()
      .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '-')
      || w.chars().filter(|c| c.is_uppercase()).count() >= 2)
    && w.chars().any(char::is_uppercase)
    && !non_name_word(w)
}

/// Whether a piece opens a collaboration's name or holds the remark that does: "for", "on behalf of", "the" (`for the
/// XYZ`, `Bob Baker for the XYZ` before `and ABC Collaborations`).
fn opens_collaboration(text: &str) -> bool {
  let lower = text.trim().to_lowercase();
  ["for ", "on behalf of ", "the "]
    .iter()
    .any(|p| lower.starts_with(p))
    || [" for ", " on behalf of ", "(for ", "(on behalf of "]
      .iter()
      .any(|p| lower.contains(p))
}

/// The name a piece holds before the collaboration it speaks for or that brackets set after it, and that
/// collaboration: `J.~Zapatero$^{5}$ (The MAGIC Collaboration)` (0910.4381), `C.~Cole$^{1,2}$ \vspace{0.2cm} (XYZ
/// Collaboration)` (2212.06489), `Bob Baker for the ATLAS Collaboration` (1502.03246's `for the Hall A DVCS
/// collaboration`); `None` when no such remark follows a name.
fn name_and_remark(piece: &Tokens) -> Option<(Tokens, Tokens)> {
  let v = piece.unlist_ref();
  let (mut depth, mut in_math) = (0i32, false);
  for j in 0..v.len() {
    let t = v[j];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::MATH => in_math = !in_math,
      _ => {},
    }
    if j == 0 || depth != 0 || in_math {
      continue;
    }
    // (a word after a space, a tie, a mark or a command: `J.K. de Jong \IEEEauthorrefmark{2}for the MINOS
    // collaboration.`, 1111.2718)
    let word_start = !matches!(v[j - 1].get_catcode(), Catcode::LETTER | Catcode::OTHER);
    let candidate = t == T_OTHER!("(")
      || t == T_OTHER!("[")
      || (word_start && (t == T_LETTER!("f") || t == T_LETTER!("o")));
    if !candidate {
      continue;
    }
    let name = Tokens::new(v[..spacing_end(&v[..j])].to_vec());
    let remark = Tokens::new(v[j..].to_vec());
    if reads_as_collaboration(&visible_name_text(remark.unlist_ref()), false)
      && reads_as_one_name(&name)
    {
      return Some(name_takes_annotations(name, remark));
    }
  }
  None
}

/// A name and its collaboration remark with the name's annotations the remark holds moved onto the name — a note
/// reference, a footnote, a `\thanks` (`is_name_annotation`, `is_footnote_mark`) and its arguments: `Jan Fiete
/// Grosse-Oetringhaus (for the ALICE\fnref{col1} Collaboration)` (1408.0414, 1408.5384) is the name's `\fnref`, which
/// in the contact asked for a label no creator answers (two `Warning:unexpected:annotation`).
fn name_takes_annotations(name: Tokens, remark: Tokens) -> (Tokens, Tokens) {
  let v = remark.unlist_ref();
  let (mut kept, mut moved) = (Vec::with_capacity(v.len()), Vec::new());
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if t.get_catcode() != Catcode::CS || !(is_name_annotation(&t) || is_footnote_mark(&t)) {
      kept.push(t);
      i += 1;
      continue;
    }
    let start = i;
    i += 1;
    if v.get(i) == Some(&T_OTHER!("["))
      && let Some(close) = v[i..].iter().position(|u| *u == T_OTHER!("]"))
    {
      i += close + 1;
    }
    while v.get(i).is_some_and(|u| u.get_catcode() == Catcode::BEGIN) {
      i = skip_group(v, i);
    }
    moved.extend_from_slice(&v[start..i.min(v.len())]);
  }
  if moved.is_empty() {
    return (name, remark);
  }
  let mut named = name.unlist();
  named.extend(moved);
  (Tokens::new(named), Tokens::new(kept))
}

/// The collaboration a piece opens with before a colon and the name after it: `Euclid Collaboration: Y.~Fu\inst{…}`
/// (2512.08803), `AMI Consortium: Carmen Rodr{\'i}guez-Gonz{\'a}lvez$^{1,2}$` (1205.7067); `None` for any other piece.
fn heading_and_name(piece: &Tokens) -> Option<(Tokens, Tokens)> {
  let v = piece.unlist_ref();
  // (a colon of the line's own, not one inside a group or a formula: `MeerKLASS Collaboration\thanks{Corresponding
  // authors:}: Matilde Barberi-Squarotti$^{1,2,3,4}$`, 2407.21626; `\parbox{\textwidth}{\Large{ MAGIC Collaboration:
  // K.~Abe$^{1}$, …`, 2601.03873 — cut there, the pieces' braces did not close)
  let (mut depth, mut in_math) = (0i32, false);
  let colon = v.iter().position(|t| {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::MATH => in_math = !in_math,
      _ => {},
    }
    depth == 0 && !in_math && *t == T_OTHER!(":")
  })?;
  let heading = Tokens::new(v[..colon].to_vec());
  let mut start = colon + 1;
  while v.get(start).is_some_and(is_spacing) {
    start += 1;
  }
  let name = Tokens::new(v[start..].to_vec());
  (reads_as_collaboration(&visible_name_text(heading.unlist_ref()), false)
    && reads_as_one_name(&name))
  .then_some((heading, name))
}

/// A names line's pieces ([`split_author_names`]) with the collaborations among them read out, each piece with the
/// separator before it and whether it is a collaboration: a piece naming one ([`reads_as_collaboration`]; one the
/// separators parted put together again — `for the XYZ` and `ABC Collaborations`, `ATLAS`, `CMS` and `D0
/// Collaborations`), or a name's remark ([`name_and_remark`]); `None` when no piece is one, or no name stands beside
/// them.
fn names_and_collaborations(
  pieces: Vec<(Vec<Token>, Tokens)>,
) -> Option<Vec<(Vec<Token>, Tokens, bool)>> {
  let text = |piece: &Tokens| visible_name_text(piece.unlist_ref());
  let mut joined: Vec<(Vec<Token>, Tokens)> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let piece_text = text(&piece);
    let ends_collaboration = piece_text
      .split_whitespace()
      .last()
      .is_some_and(collaboration_word);
    if !ends_collaboration
      || opens_collaboration(&piece_text)
      || piece_text.trim_start().starts_with(['(', '['])
    {
      joined.push((delimiter, piece));
      continue;
    }
    let mut taken: Vec<(Vec<Token>, Tokens)> = Vec::new();
    let (mut merged, mut merged_delimiter) = (piece.clone().unlist(), delimiter.clone());
    while let Some((before_delimiter, before)) = joined.last().cloned() {
      let before_text = text(&before);
      let opens = opens_collaboration(&before_text);
      if !(opens || listed_acronym(&before_text)) {
        break;
      }
      let mut v = before.unlist();
      let comma = merged_delimiter.last() == Some(&T_OTHER!(","));
      v.extend(merged_delimiter.iter().copied());
      if comma {
        v.push(T_SPACE!());
      }
      v.extend(merged);
      merged = v;
      merged_delimiter = before_delimiter;
      taken.push(joined.pop()?);
      if opens {
        break;
      }
    }
    let merged = Tokens::new(merged);
    if taken.is_empty()
      || reads_as_collaboration(&text(&merged), false)
      || name_and_remark(&merged).is_some()
    {
      joined.push((merged_delimiter, merged));
    } else {
      // (no collaboration made of them: the pieces as they were)
      joined.extend(taken.into_iter().rev());
      joined.push((delimiter, piece));
    }
  }
  // (a collaboration's name listed with "and" after unmarked names, neither framed nor a team's, may be the whole line
  // one collaboration's name: revtex's `\collaboration{Fermilab Lattice and MILC Collaborations}`, 1211.0751, 1411.1651
  // — no person "Fermilab Lattice")
  let marked_line = joined.iter().any(|(_, piece)| marked_after_name(piece));
  let listed_alone = |text: &str| {
    collaboration_core(text).1 == Framing::Unframed
      && text
        .split_whitespace()
        .last()
        .is_some_and(|w| COLLABORATION_WORDS.contains(&bare_word(w).as_str()))
  };
  let mut out: Vec<(Vec<Token>, Tokens, bool)> = Vec::with_capacity(joined.len() + 1);
  for (delimiter, piece) in joined {
    let piece_text = text(&piece);
    if reads_as_listed_collaboration(&piece_text, joined_by_and(&delimiter))
      && (marked_line || !listed_alone(&piece_text))
    {
      out.push((delimiter, piece, true));
    } else if let Some((name, remark)) = name_and_remark(&piece) {
      out.push((delimiter, name, false));
      out.push((Vec::new(), remark, true));
    } else if let Some((heading, name)) = heading_and_name(&piece) {
      out.push((Vec::new(), heading, true));
      out.push((delimiter, name, false));
    } else {
      out.push((delimiter, piece, false));
    }
  }
  (out.iter().any(|(.., c)| *c) && out.iter().any(|(.., c)| !*c)).then_some(out)
}

/// An author line split into its names with the collaborations it names read out, each such piece wrapped in
/// [`collaboration_wrapper`] for the contact it makes: `Ann Able$^{1}$, Bob Baker$^{2}$ and the GAPS Team` is two
/// names and the GAPS Team (1708.04166), `A.~Able$^{1}$, …, J.~Zu$^{68,55}$ \vspace{0.2cm} (BESIII Collaboration)` the
/// names and BESIII (2212.06489, 581 names). A per-author class's line (`cautious`) is read so only when every other
/// piece reads as one name, by its shape or its mark ([`split_author_line_cautious`]'s list by its marks), its names
/// then led by the separators printed before them as that split leads them (`\lx@author@printed@before`); `None` when
/// the line names no collaboration beside its names, for the split it would have had.
pub(in crate::base_utilities) fn collaboration_split(
  line: &Tokens,
  cautious: bool,
) -> Option<Vec<Tokens>> {
  let line = without_leading_by(line).unwrap_or_else(|| line.clone());
  // (the names a whole-line group holds read inside it, each piece in the group and its declarations: `{\slshape
  // Stefanie Adomeit$^1$, Reinhild Yvonne Peters$^{2,3}$ for the ATLAS, CDF, CMS and D0 Collaborations}`, 1311.7305)
  let (wrap, pieces) = match whole_line_group(&line) {
    Some((declarations, inner)) => (
      Some(declarations),
      names_and_collaborations(rejoin_name_qualifiers_keeping_separators(
        split_author_names(inner),
      ))?,
    ),
    None => (
      None,
      names_and_collaborations(rejoin_name_qualifiers_keeping_separators(
        split_author_names(line.clone()),
      ))?,
    ),
  };
  let names: Vec<&Tokens> = pieces
    .iter()
    .filter(|(.., c)| !*c)
    .map(|(_, name, _)| name)
    .collect();
  // (every other piece a name, beside a piece that is a collaboration — `Content \& Services Team`, 2109.03031, names
  // one team, no person "Content" —, and on a per-author class's line whatever the collaboration; a remark after a name
  // is read on any line the kernel splits as it splits it: `Ann Able, University of Toronto and Bob Baker for the CMS
  // Collaboration`)
  let piece = pieces
    .iter()
    .any(|(delimiter, _, collaboration)| *collaboration && !delimiter.is_empty());
  if (cautious || piece) && !names.iter().all(|name| reads_as_one_name(name)) {
    return None;
  }
  let printed = cautious && names.iter().all(|name| marked_after_name(name));
  let wrapped = |piece: Tokens| match &wrap {
    Some(declarations) => {
      let mut v = vec![T_BEGIN!()];
      v.extend(declarations.iter().copied());
      v.extend(piece.unlist());
      v.push(T_END!());
      Tokens::new(v)
    },
    None => piece,
  };
  let mut out = Vec::with_capacity(pieces.len());
  let mut named: Vec<(Vec<Token>, Tokens)> = Vec::new();
  let flush = |named: &mut Vec<(Vec<Token>, Tokens)>, out: &mut Vec<Tokens>, first: bool| {
    let run = std::mem::take(named);
    if printed {
      // (a run after a collaboration is led by its separator as any later name is)
      let mut led = with_printed_separators(
        std::iter::once((Vec::new(), Tokens::default()))
          .filter(|_| !first)
          .chain(run)
          .collect(),
      );
      if !first {
        led.remove(0);
      }
      out.extend(led.into_iter().map(wrapped));
    } else {
      out.extend(run.into_iter().map(|(_, name)| wrapped(name)));
    }
  };
  let mut first = true;
  for (delimiter, piece, collaboration) in pieces {
    if collaboration {
      let had = !named.is_empty();
      flush(&mut named, &mut out, first);
      first &= !had;
      let mut v = vec![collaboration_wrapper(), T_BEGIN!()];
      v.extend(wrapped(piece).unlist());
      v.push(T_END!());
      out.push(Tokens::new(v));
    } else {
      named.push((delimiter, piece));
    }
  }
  flush(&mut named, &mut out, first);
  Some(out)
}

/// The collaboration a piece wrapped by [`collaboration_split`] holds, and the names of its group after it when the
/// wrapper says (`[k]`, counted from the group's end, which the names of a group joined before it leave as it is);
/// `None` for any other piece.
pub(in crate::base_utilities) fn unwrapped_collaboration(
  piece: &Tokens,
) -> Option<(Tokens, Option<usize>)> {
  let v = piece.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!())?;
  if v[start] != collaboration_wrapper() {
    return None;
  }
  let mut i = start + 1;
  let mut after = None;
  if v.get(i) == Some(&T_OTHER!("[")) {
    let close = v[i..].iter().position(|t| *t == T_OTHER!("]"))? + i;
    after = Tokens::new(v[i + 1..close].to_vec())
      .to_string()
      .trim()
      .parse()
      .ok();
    i = close + 1;
  }
  if v.get(i).is_none_or(|t| t.get_catcode() != Catcode::BEGIN) {
    return None;
  }
  let end = skip_group(v, i);
  v[end..]
    .iter()
    .all(|t| *t == T_SPACE!())
    .then(|| (Tokens::new(v[i + 1..end - 1].to_vec()), after))
}

/// A collaboration wrapped for a group's lines, `after` the names of the group after it ([`unwrapped_collaboration`]):
/// none for a line under the names.
pub(in crate::base_utilities) fn collaboration_in_lines(text: Tokens, after: usize) -> Tokens {
  let mut v = vec![collaboration_wrapper(), T_OTHER!("[")];
  v.extend(mouth::tokenize_internal(TeXString::assembled(after.to_string())).unlist());
  v.push(T_OTHER!("]"));
  v.push(T_BEGIN!());
  v.extend(text.unlist());
  v.push(T_END!());
  Tokens::new(v)
}

/// The text a collaboration's contact holds: as written, without the layout around it (`\center`, `\vspace{0.2cm}`)
/// and the affiliation marks on it (`LHD experimental group$^{2}$`, 1711.07097: a mark no contact answers); a footnote
/// symbol it shows stays (`on behalf of the "Flat Blazars" collaboration$^*$`, astro-ph9712234).
fn collaboration_content(text: &Tokens) -> Tokens {
  let v = text.unlist_ref();
  let mut start = 0;
  loop {
    let past = start + leading_spacing_end(&v[start..]);
    let past = past
      + v[past..]
        .iter()
        .take_while(|t| {
          t.get_catcode() == Catcode::CS
            && t.with_str(|s| {
              matches!(
                s,
                "\\center"
                  | "\\centering"
                  | "\\raggedright"
                  | "\\raggedleft"
                  | "\\noindent"
                  | "\\par"
              )
            })
        })
        .count();
    if past == start {
      break;
    }
    start = past;
  }
  let v = &v[start..];
  let mut v = &v[..spacing_end(v)];
  // (a heading's colon is no part of the collaboration's name, 1204.5689)
  while let [rest @ .., last] = v
    && (*last == T_OTHER!(":") || *last == T_SPACE!())
  {
    v = rest;
  }
  let mut out = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    if t.get_catcode() == Catcode::MATH
      && let Some(close) = v[i + 1..]
        .iter()
        .position(|u| u.get_catcode() == Catcode::MATH)
      && only_marks_and_notes(&v[i + 1..i + 1 + close])
    {
      i += close + 2;
      continue;
    }
    if is_author_mark(&t) || t == T_CS!("\\altaffilmark") {
      i = skip_group(v, i + 1);
      continue;
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// How two readings of one collaboration compare: its framed name's words, lowercased ([`collaboration_core`]).
fn collaboration_key(text: &Tokens) -> String {
  collaboration_core(&visible_name_text(text.unlist_ref()))
    .0
    .to_lowercase()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

/// The contact a collaboration makes (Base_Utility.pool.ltxml:517-521): on the last `covers` authors made
/// (`annotate=N`), the names it covers counted where it is read ([`Collaborations`]) — `Some(1)` the creator being made,
/// read in its content —; with `None`, on every author before it that carries no collaboration yet (`annotate=new`, as
/// aastex's `\collaboration` reaches them, aas_support.sty.ltxml:126).
fn collaboration_contact_call(text: &Tokens, covers: Option<usize>) -> Result<Vec<Token>> {
  let annotate = covers.map_or_else(|| s!("new"), |covers| covers.max(1).to_string());
  let keyvals = mouth::tokenize_internal(TeXString::assembled(s!(
    "role=collaboration,annotate={annotate}"
  )));
  Ok(
    Invocation!(T_CS!("\\lx@add@contact"), vec![
      Some(keyvals),
      Some(collaboration_content(text))
    ])
    .unlist(),
  )
}

/// The collaborations of an author call, read in the order its entries are made ([`collaboration_split`],
/// [`collaboration_line`]), each contact made where the collaboration prints among the creator's other contacts (they
/// render in document order, LaTeXML-structure-xhtml.xsl:795). One after names — a piece of the names line, a line under
/// them — covers the names of its `\and` group made since the group's last collaboration (`… and the GAPS Team`,
/// 1708.04166; `(XYZ Collaboration)` under the names, 2410.11607; `J.~Zapatero$^{5}$ (The MAGIC Collaboration),
/// A.~Falcone$^{30}`, 0910.4381: the 144 names before it, not the six after), read where it stands — under the names,
/// in the last name's content at its line's place (`Ann Able \\ for the XYZ Collaboration \\ Univ X`: the
/// collaboration, then the institution; 0910.2149, hep-ex0011032). One before any name of its group, a heading, is
/// every name's after it in the call, made in each name's content right after the name (`\center BESIII
/// Collaboration\\~\\ M.~Ablikim$^{1}$, …`, 2212.06489; `\small H.E.S.S. Collaboration \and H.~Abdalla \inst{…} \and
/// …`, 1912.05868), once when the same collaboration closes them (`… J.~Zu$^{68,55}$ (BESIII Collaboration)`); one no
/// name comes after, a trailing `\and` group, every name's before it that carries none (`annotate=new`). A new `\and`
/// group starts its own count: a collaboration after another covers its names only within one group (rev97cd D4).
#[derive(Default)]
pub(in crate::base_utilities) struct Collaborations {
  /// the names of the group made since its last collaboration
  uncovered:     usize,
  /// the headings over the names to come
  headings:      Vec<Tokens>,
  /// the names made since the headings were read
  heading_names: usize,
  /// the last collaboration made in the group, by [`collaboration_key`], and the names it covered
  last:          Option<(String, usize)>,
}

impl Collaborations {
  /// An author made: the contacts its headings make on it, for its content right after its name.
  pub(in crate::base_utilities) fn author(&mut self) -> Result<Vec<Token>> {
    self.uncovered += 1;
    self.heading_names += 1;
    let mut calls = Vec::new();
    for heading in &self.headings {
      calls.extend(collaboration_contact_call(heading, Some(1))?);
    }
    Ok(calls)
  }

  /// A collaboration read where it stands: the contact calls it makes now.
  pub(in crate::base_utilities) fn read(&mut self, text: Tokens) -> Result<Vec<Token>> {
    let key = collaboration_key(&text);
    let heads = collaboration_heads(&visible_name_text(text.unlist_ref()));
    let heading_of = |headings: &[Tokens]| {
      headings
        .iter()
        .position(|heading| collaboration_key(heading) == key)
    };
    if self.uncovered == 0 || heads {
      // (the same collaboration again, with no name between, once; one right after another, no name between, covers its
      // names too — `… I.C. Luhmann$^7$, the ASDEX Upgrade Team and the EUROfusion MST1 Team$^*$`, 1603.09150)
      match &self.last {
        Some((last, _)) if *last == key && !heads => return Ok(Vec::new()),
        Some((_, covers)) if !heads && self.headings.is_empty() => {
          let covers = *covers;
          let calls = collaboration_contact_call(&text, Some(covers))?;
          self.last = Some((key, covers));
          return Ok(calls);
        },
        _ => {},
      }
      if heading_of(&self.headings).is_some() {
        return Ok(Vec::new());
      }
      // (a heading over the names to come; the ones before it closed once names came under them)
      if self.heading_names > 0 {
        self.headings.clear();
      }
      self.headings.push(text);
      self.heading_names = 0;
      self.uncovered = 0;
      return Ok(Vec::new());
    }
    // (the heading's own name closing its names: they carry it already)
    if let Some(at) = heading_of(&self.headings) {
      self.headings.remove(at);
      self.last = Some((key, self.heading_names));
      self.uncovered = 0;
      return Ok(Vec::new());
    }
    let covers = self.uncovered;
    let calls = collaboration_contact_call(&text, Some(covers))?;
    self.uncovered = 0;
    self.last = Some((key, covers));
    Ok(calls)
  }

  /// A new `\and` group of the call: the names after it are no longer the last group's, nor is a collaboration after
  /// them the last one's companion (rev97cd D4: `… \\ (ATLAS Collaboration) \\ … \and CMS Collaboration \\ Cid Cole …`
  /// heads the names after it, not Ann's and Bob's).
  pub(in crate::base_utilities) fn close_group(&mut self) {
    self.uncovered = 0;
    self.last = None;
  }

  /// The call read to its end: a heading no name came after covers every name before it that carries no collaboration
  /// yet (`\author{Ann Able \and Bob Baker \and The XYZ Collaboration}`, `annotate=new`).
  pub(in crate::base_utilities) fn finish(&mut self) -> Result<Vec<Token>> {
    let mut calls = Vec::new();
    if self.heading_names == 0 {
      for heading in &self.headings {
        calls.extend(collaboration_contact_call(heading, None)?);
      }
    }
    *self = Self::default();
    Ok(calls)
  }
}
