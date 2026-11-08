//! The text rules on printed ink and the token primitives of `author_block/collaborations.rs`, moved unchanged (MV1,
//! user ruling 2026-11-08 "knowledge in, driver out").

use super::*;

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
pub(in crate::base_utilities) fn collaboration_core(text: &str) -> (String, Framing, bool) {
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
pub(in crate::base_utilities) fn bare_word(word: &str) -> String {
  word
    .trim_matches(|c: char| ".,;:()[]".contains(c))
    .to_lowercase()
}

/// Whether a word names a collaboration, a team or a group ([`COLLABORATION_WORDS`], [`TEAM_WORDS`]).
pub(in crate::base_utilities) fn collaboration_word(word: &str) -> bool {
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
pub(in crate::base_utilities) fn collaboration_framing(
  text: &str,
  listed: bool,
  line: bool,
) -> Option<Framing> {
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
