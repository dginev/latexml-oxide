//! The old reader's owner logic, moved unchanged from `author_block/` into the one engine's library (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08): the spelling keys of a name ([`NameKeys`], `as_printed` with them), the
//! owners an address's local part spells or elimination leaves, the notes a name holds, the marks a name requests. It
//! reads the names the old reader exports, as it did there (`crate::base_utilities`), not the resolver's own of the same
//! name (its `text_tokens`, `printed_words`, …); `text_prints_name` stays with `name_words`, which it calls.

use crate::base_utilities::*;

/// A note an author's own name holds ([`author_notes`]).
pub(in crate::base_utilities) struct AuthorNote {
  /// the author (index among the names read) whose name holds it
  pub(in crate::base_utilities) owner:                usize,
  /// for a jheppub-family `\note`, which prints its text under a mark it shows beside no name (jheppub.sty:84-89), the
  /// label of that mark, which the authors it is about write on their names: its optional argument's (`\note[*]{…}`),
  /// else the n-th such note's number, the footnote counter it steps (`\author[a,1]{M.V. Smirnov,\note{Corresponding
  /// author.}}` … `\author[a,1]{{\rm and} J.J. Ling}`, 1802.03677); `None` for a `\thanks` or `\footnote`, which
  /// prints its own mark on its owner's name
  pub(in crate::base_utilities) mark:                 Option<String>,
  /// whether `mark` is the counter's number rather than the note's own optional argument: a number an affiliation's
  /// label also is, the authors who write it request that affiliation (`\author[1]{Ann Able,\note{…}}`
  /// `\author[1]{Bob Baker}` `\affiliation[1]{Univ A}`: Bob Baker's 1 is Univ A's), so it is no legend of the note's
  pub(in crate::base_utilities) numbered:             bool,
  /// whether its text names the corresponding author ([`names_correspondence`])
  pub(in crate::base_utilities) names_correspondence: bool,
}

/// The notes of the commands `notes` that the names `authors` hold, in their order (a binding's own note command too,
/// jheppub's `\note{Corresponding author.}`, `\note[*]{…}`).
pub(in crate::base_utilities) fn author_notes<'a>(
  authors: impl Iterator<Item = &'a Tokens>,
  notes: &[Token],
) -> Vec<AuthorNote> {
  let mut found = Vec::new();
  let mut numbered = 0;
  for (owner, author) in authors.enumerate() {
    let toks = author.unlist_ref();
    let mut i = 0;
    while i < toks.len() {
      if !notes.contains(&toks[i]) {
        i += 1;
        continue;
      }
      // (past an optional argument: `\thanks[1]{…}`, `\note[*]{…}`)
      let mut at = i + 1;
      let mut mark = None;
      if toks.get(at) == Some(&T_OTHER!("["))
        && let Some(close) = toks[at..].iter().position(|t| *t == T_OTHER!("]"))
      {
        mark = clean_frontmatter_labels(
          &Tokens::new(toks[at + 1..at + close].to_vec()).to_string(),
          "affiliation",
        )
        .into_iter()
        .next();
        at += close + 1;
      }
      if !toks
        .get(at)
        .is_some_and(|b| b.get_catcode() == Catcode::BEGIN)
      {
        i += 1;
        continue;
      }
      let end = skip_group(toks, at);
      // (a `\thanks`/`\footnote` prints its own mark on the name, whatever its optional number: its owner alone, and
      // its number no legend of marks the authors write — `$^1$` there is an affiliation's)
      let mut counted = false;
      if toks[i] != T_CS!("\\note") {
        mark = None;
      } else if mark.is_none() {
        numbered += 1;
        counted = true;
        mark = clean_frontmatter_labels(&numbered.to_string(), "affiliation")
          .into_iter()
          .next();
      }
      found.push(AuthorNote {
        owner,
        mark,
        numbered: counted,
        names_correspondence: names_correspondence(&visible_name_text(&toks[at..end])),
      });
      i = end;
    }
  }
  found
}

/// A list of local parts written without spaces before one domain, `text` whole (spaces round it aside), whose every
/// part spells exactly one author, each a different one (`owner`, letter case ignored as the spelling reads it), each
/// part at the domain, as the source writes it: `M.R.Pestalozzi,a.chrysostomou,j.l.collett@herts.ac.uk` under the three
/// names, astro-ph0612088 — a capitalised part, which [`write_out_unspaced_lists`] leaves whole, is a local part when it
/// spells its author. `None` for any other text: an address is never invented.
pub(in crate::base_utilities) fn spelled_unspaced_list(
  text: &str,
  owner: &dyn Fn(&str) -> Option<usize>,
) -> Option<Vec<String>> {
  let text = text.trim();
  if text.contains(|c: char| c.is_whitespace() || "{};|/".contains(c)) {
    return None;
  }
  let (list, domain) = text.split_once('@')?;
  let domain_ok = domain.contains('.')
    && !domain.starts_with(['.', '-'])
    && !domain.ends_with(['.', '-'])
    && domain
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
  let parts: Vec<&str> = list.split(',').collect();
  if !domain_ok || parts.len() < 2 {
    return None;
  }
  let mut owners: Vec<usize> = Vec::with_capacity(parts.len());
  for part in &parts {
    if part.chars().nth(1).is_none()
      || part.starts_with('.')
      || !part.chars().all(is_local_char)
      || !part.chars().any(|c| c.is_ascii_alphabetic())
    {
      return None;
    }
    let one = owner(part)?;
    if owners.contains(&one) {
      return None;
    }
    owners.push(one);
  }
  Some(parts.iter().map(|part| s!("{part}@{domain}")).collect())
}

/// The marks an author's tokens request: those outside its notes, a `\thanks{...}` or `\footnote{...}` whose own
/// superscripts are its text or label it (`note_body`) — `Ann Able\thanks{$^{a}$ Supported by ...}` requests nothing.
pub(in crate::base_utilities) fn author_mark_operands(tokens: &[Token]) -> Vec<Vec<Token>> {
  mark_operands(&outside_notes(tokens))
}

/// The words that can stand for a person in an email address ([`spelled_owners`]), lowercased, of the name as printed,
/// its notes aside (1706.03762's `Ashish Vaswani\thanks{… Noam proposed …}` is not "noam").
#[derive(Clone, Debug, Default)]
pub(in crate::base_utilities) struct NameKeys {
  /// the name's words after the first that have three letters or more, counted in their base spelling ([`SpelledWord`]:
  /// `Ph.~Boucaud` "boucaud", `J.~Rodr\'iguez-Quintero` "rodriguez", "quintero")
  surnames: Vec<SpelledWord>,
  /// the name's words after the first of two letters (`Weili An` "an", `Wei Li` "li"), which spell only a local part of
  /// those letters alone, digits aside (`an107@purdue.edu` Weili An's, not Anusuya Nallathambi's initials "an", 2608.22602)
  short_surnames: Vec<String>,
  /// its first word, when it has three letters or more (`Ahna R. Girshick` "ahna"; not the initial of `J.~Smith`)
  given: Option<SpelledWord>,
  /// the spellings an address opens with that join two of its words: the first word's initial and a later word of two
  /// letters or more (`Prathamjyot Singh` "psingh", 2608.10670's `psingh1_be22@…` among three Singhs; `Tianjian Li`
  /// "tli", 2606.23525's `tli104@…`; `Karamvir Singh Batra` "kbatra"), a later word before the whole first, the
  /// family name first (`Fei Wu` "wufei", `Yu Cheng` "chengyu": 2509.10377), or after it (`Di Wu` "diwu", 2604.01354;
  /// `Mo Yu` "moyu", 2608.18082 — 77p r10, where an address no name spells is the block's), and the initials of the
  /// words before the last with the last (`Kai-Wei Chang` "kwchang", 2604.01354); and the initials of every word, two or more
  /// (`Ann Able` "aa"), or of the first and the last when more stand between (`Joseph G. Wallwork` "jgw" and "jw",
  /// 2407.00382's `jw2423@cam.ac.uk`), which only a local part of those letters alone spells
  pub(in crate::base_utilities) joined: Vec<String>,
  pub(in crate::base_utilities) initials: Vec<String>,
  /// the same keys of the name's transliterated spelling, where it differs from the base letters (`Schäffler`
  /// "schaeffler" beside "schaffler", [`resolver::transliterated`]): a name scores the better of its two spellings
  transliterated: Option<Box<NameKeys>>,
}

/// A word of a name, as [`spelled_owners`] scores it: its letters (as an address spells them), and its length class —
/// the letters of its base spelling, which decide whether it is a surname whose first six letters spell it, a given
/// name a local part may hold anywhere, or a short word only a local part of its letters alone spells: the
/// transliteration never lifts a word over a threshold (`Wei L\"u`'s "lue" spells no `bluesky@`, review rev107a
/// finding 3; OXIDIZED_DESIGN_DIVERGENCES #970).
#[derive(Clone, Debug, Default)]
pub(in crate::base_utilities) struct SpelledWord {
  text:    String,
  letters: usize,
}

impl NameKeys {
  /// The keys of a name as an address spells it: its letters folded to their base letters (`Martínez` "martinez",
  /// [`resolver::base_letters`]) and, beside those, transliterated (`Schäffler` "schaeffler", `Wei{\ss}enborn`
  /// "weissenborn", `Hj\o rth` "hjoerth", [`resolver::transliterated`]; html_feedback #7233, OXIDIZED_DESIGN_DIVERGENCES
  /// #970) — a letter written with an accent or letter command read as the letter it prints ([`spelling_name_text`]).
  pub(in crate::base_utilities) fn of(name: &Tokens) -> NameKeys {
    let text = spelling_name_text(name.unlist_ref());
    let base = resolver::base_letters(&text);
    let transliterated = resolver::base_letters(&resolver::transliterated(&text));
    let base_words = name_words(&base);
    let classes: Vec<usize> = base_words.iter().map(|word| word.chars().count()).collect();
    let mut keys = NameKeys::of_words(&base_words, &classes);
    if transliterated != base {
      // (each transliterated word in its base spelling's length class, the words one for one; else its own)
      let words = name_words(&transliterated);
      let own: Vec<usize> = words.iter().map(|word| word.chars().count()).collect();
      let classes = if words.len() == classes.len() {
        &classes
      } else {
        &own
      };
      let mut spelled = NameKeys::of_words(&words, classes);
      spelled.joined.extend(hyphenated_surname(&transliterated));
      keys.transliterated = Some(Box::new(spelled));
    }
    keys.joined.extend(hyphenated_surname(&base));
    keys
  }

  /// The keys of a name as it prints, its letters unfolded, and of its transliteration (`Müller` "mueller", the
  /// letters [`resolver::transliteration`] spells, the others as printed): whom `insert::respell_emails` and the
  /// frontmatter observer's `Resolver::respell_emails` move an author's e-mail to, an address spelling that author as
  /// printed — a transliterated spelling is one (user ruling 2026-11-05: `joerg.mueller@` is Jörg Müller's, "also
  /// written Joerg Mueller"; `\email{mueller-group@x.de}` on Anna Smith's own call moves to `\author{J\"org M\"uller}`
  /// as its ASCII twin does, FIDELITY_CHARTER §4 #420), a base-letter fold is not (`muller-group@` stays). Each word in
  /// its printed length class ([`SpelledWord`]); a tie between a literal and a transliterated spelling is a tie
  /// (`mueller@` beside Hans Mueller and Jörg Müller moves to neither, user ruling 2026-11-05). It expects the name's
  /// digested tokens, as both respelling sites pass them: an undigested `Sch\"affler` reads through
  /// [`visible_name_text`]'s accent fold as "Schaffler", whose base fold it then spells (review rev107a2 N3).
  pub(in crate::base_utilities) fn as_printed(name: &Tokens) -> NameKeys {
    let printed = name_words(&visible_name_text(name.unlist_ref()));
    let classes: Vec<usize> = printed.iter().map(|word| word.chars().count()).collect();
    let mut keys = NameKeys::of_words(&printed, &classes);
    let transliterated = name_words(&resolver::transliterated(&spelling_name_text(
      name.unlist_ref(),
    )));
    if transliterated != printed {
      let own: Vec<usize> = transliterated
        .iter()
        .map(|word| word.chars().count())
        .collect();
      let classes = if transliterated.len() == classes.len() {
        &classes
      } else {
        &own
      };
      keys.transliterated = Some(Box::new(NameKeys::of_words(&transliterated, classes)));
    }
    keys
  }

  /// The name's spellings: its base letters', and its transliterated one where it has one.
  pub(in crate::base_utilities) fn spellings(&self) -> impl Iterator<Item = &NameKeys> {
    std::iter::once(self).chain(self.transliterated.as_deref())
  }

  /// The keys of a name's `words`, each in its length class (`classes`, its letters in the base spelling: [`SpelledWord`]).
  fn of_words(words: &[String], classes: &[usize]) -> NameKeys {
    let Some((first, later)) = words.split_first() else {
      return NameKeys::default();
    };
    let class = |at: usize| classes.get(at).copied().unwrap_or_default();
    let initial: String = first.chars().take(1).collect();
    let mut joined: Vec<String> = Vec::new();
    for (_, word) in later
      .iter()
      .enumerate()
      .filter(|(at, _)| class(at + 1) >= 2)
    {
      joined.push(format!("{initial}{word}"));
      if class(0) >= 2 {
        joined.push(format!("{word}{first}"));
        joined.push(format!("{first}{word}"));
      }
    }
    // (the initials of every word before the last, then the last: `Kai-Wei Chang` "kwchang", 2604.01354)
    if let Some((last, before)) = words.split_last()
      && before.len() >= 2
      && class(words.len() - 1) >= 2
    {
      let initials: String = before
        .iter()
        .filter_map(|word| word.chars().next())
        .collect();
      joined.push(format!("{initials}{last}"));
    }
    let initials: String = words
      .iter()
      .filter_map(|word| word.chars().next())
      .collect();
    NameKeys {
      surnames: later
        .iter()
        .enumerate()
        .filter(|(at, _)| class(at + 1) >= 3)
        .map(|(at, word)| SpelledWord {
          text:    word.clone(),
          letters: class(at + 1),
        })
        .collect(),
      // (a name particle another word of the name follows is no surname of its own: `Ana de Souza` spells no `de@`,
      // review rev100b S3; the last word is the surname whatever it reads, `Thanh Le`'s `le@`, review rev100b2 S3)
      short_surnames: later
        .iter()
        .enumerate()
        .filter(|(at, word)| {
          class(at + 1) == 2
            && (*at + 1 == later.len()
              || !matches!(word.as_str(), "de" | "la" | "da" | "le" | "di" | "du"))
        })
        .map(|(_, word)| word.clone())
        .collect(),
      given: (class(0) >= 3).then(|| SpelledWord {
        text:    first.clone(),
        letters: class(0),
      }),
      joined,
      initials: [
        Some(initials.clone()),
        (words.len() >= 3).then(|| {
          initials
            .chars()
            .take(1)
            .chain(initials.chars().last())
            .collect()
        }),
      ]
      .into_iter()
      .flatten()
      .filter(|initials: &String| initials.chars().count() >= 2)
      .collect(),
      transliterated: None,
    }
  }
}

/// A name's words, lowercased: its runs of letters, an apostrophe between two letters inside its word (`D'Amico`
/// "damico", whose address `damicos@` holds it whole, 2606.17317; `Gol'dshtein` "goldshtein" — brief 107f, where a
/// surname spells only a run of the local part's words).
/// A hyphen-linked surname with its hyphen dropped, a spelling as printed (`L. Tal-Or` "talor": `levtalor@`, 2112.09029,
/// review rev107g), as a hyphenated given name's already is (`Wang-Chiew` "wangchiew"). The name's last word only, its
/// parts all letters: spaced words stay apart (`de Vries`). A joined key, so where it ends a word it does not open it is
/// a weak spelling ([`LocalPart::joined_standing`]) and owns only an address no other name spells.
fn hyphenated_surname(text: &str) -> Option<String> {
  let last = text.split_whitespace().last()?;
  let parts: Vec<&str> = last.split(['-', '\u{2010}', '\u{2011}']).collect();
  (parts.len() >= 2
    && parts
      .iter()
      .all(|part| !part.is_empty() && part.chars().all(char::is_alphabetic)))
  .then(|| parts.concat().to_lowercase())
}

fn name_words(text: &str) -> Vec<String> {
  let chars: Vec<char> = text.chars().collect();
  let text: String = chars
    .iter()
    .enumerate()
    .filter(|&(at, c)| {
      !(matches!(c, '\'' | '\u{2019}' | '\u{02bc}')
        && at > 0
        && chars[at - 1].is_alphabetic()
        && chars.get(at + 1).is_some_and(|next| next.is_alphabetic()))
    })
    .map(|(_, c)| *c)
    .collect();
  text
    .split(|c: char| !c.is_alphabetic())
    .filter(|word| !word.is_empty())
    .map(str::to_lowercase)
    .collect()
}

/// An address's letters and other characters, as [`spelled_owners`] reads them.
pub(in crate::base_utilities) fn address_text(address: &Tokens) -> String {
  // (a colour, a font, a scale is no letter of it, [`without_non_printing_arguments`]: 2604.19440, 2609.25669)
  let printed = without_non_printing_arguments(address.unlist_ref());
  let v = printed.as_slice();
  // (an escaped underscore prints one, and parts the local part's words: `ricky\_ang@`, 2604.02636 — [`spelled_owners`])
  let text = |v: &[Token]| -> String {
    v.iter()
      .filter_map(|t| match t.code {
        Catcode::LETTER | Catcode::OTHER => Some(t.to_string()),
        _ if *t == T_CS!("\\_") || *t == T_CS!("\\textunderscore") => Some("_".to_string()),
        _ => None,
      })
      .collect()
  };
  // `\href{mailto:t}{p}`: the address it prints, not its target and the printed one run together; its target's, past
  // `mailto:`, when it prints none (a placeholder target, `\href{mailto:email@domain}{yuxi.xia@univie.ac.at}`,
  // 2601.07974)
  if let Some(at) = v.iter().position(|t| *t == T_CS!("\\href")) {
    let open = at + 1 + v[at + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
    let close = skip_group(v, open).min(v.len());
    let target = text(&v[open..close]);
    let printed = text(&v[..at]) + &text(&v[close..]);
    return if printed.contains('@') {
      printed
    } else {
      target.trim_start_matches("mailto:").to_string()
    };
  }
  text(v)
}

/// Every name (index into `name_keys`) whose surname the local part of `address` spells ([`spelled_owners`]).
pub(in crate::base_utilities) fn address_owners(
  name_keys: &[NameKeys],
  address: &Tokens,
) -> Vec<usize> {
  spelled_owners(name_keys, &address_text(address))
}

/// The fewest letters a key spells an address with strictly inside a word of its local part, between letters that are
/// none of the name's initials ([`LocalPart::standing`]) — a weak spelling, which owns only an address no other name
/// spells (brief 107g: 4, chosen on the pop107g A/B, OXIDIZED_DESIGN_DIVERGENCES #970).
const WEAK_KEY_LETTERS: usize = 4;

/// The fewest letters a joined spelling ending a word it does not open spells an address with
/// ([`LocalPart::joined_standing`]) — a weak spelling (brief 107f r2).
const WEAK_JOINED_LETTERS: usize = 5;

/// Whether `text` prints the name `name` whole, a name of two words at least, its words in a row (`insert::
/// link_entry_addresses`: `David Macêdo, Pedro Dreyer, … are with …`, 2102.08716).
pub(in crate::base_utilities) fn text_prints_name(text: &str, name: &str) -> bool {
  let name = name_words(name);
  name.len() >= 2
    && name_words(text)
      .windows(name.len())
      .any(|words| words == name)
}

/// How a key's letters stand among a local part's words ([`LocalPart::standing`]): at full strength, or weakly — a
/// spelling that owns an address only no other name spells, and never decides between names that do ([`spelled_owners`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Standing {
  Weak,
  Full,
}

/// An address's local part as [`spelled_owners`] reads it: its letters, lowercased, and where its words meet (`bounds`,
/// byte offsets into `letters`): its separators — `.` `_` `-`, a digit, any other character —, its start and its end.
struct LocalPart {
  letters: String,
  bounds:  Vec<usize>,
}

impl LocalPart {
  fn of(address: &str) -> Option<LocalPart> {
    let raw = address.split('@').next()?;
    let mut letters = String::new();
    let mut bounds: Vec<usize> = vec![0];
    for c in raw.chars() {
      if c.is_alphabetic() {
        letters.extend(c.to_lowercase());
      } else if bounds.last() != Some(&letters.len()) {
        bounds.push(letters.len());
      }
    }
    if bounds.last() != Some(&letters.len()) {
      bounds.push(letters.len());
    }
    Some(LocalPart { letters, bounds })
  }

  /// Every place `key` stands in the letters, overlapping ones too.
  fn places(&self, key: &str) -> Vec<usize> {
    if key.is_empty() {
      return Vec::new();
    }
    (0..self.letters.len())
      .filter(|&at| self.letters.is_char_boundary(at) && self.letters[at..].starts_with(key))
      .collect()
  }

  /// The word holding the letters at `start..end`, or `None` when they run across a separator.
  fn word_around(&self, start: usize, end: usize) -> Option<(usize, usize)> {
    if self.bounds.iter().any(|&b| start < b && b < end) {
      return None;
    }
    let before = self.bounds.iter().copied().filter(|&b| b <= start).max()?;
    let after = self.bounds.iter().copied().filter(|&b| b >= end).min()?;
    Some((before, after))
  }

  /// THE boundary predicate (brief 107f r2, on review rev107f's Q1): how a name's key at `start..end` stands. At full
  /// strength when it is a whole word, or a word's prefix or suffix (`ricky_ang@` Lay Kee Ang's "ang", `yang91@`,
  /// `kee.ang@`), or strictly inside a word between the name's own initials (`own`), at most two letters on each side
  /// (`rjosem@` José M. Rodríguez-Nogales', 1809.06911; `kmfonsecar@` K. M. Fonseca Romero's, 1202.4210); weakly when a
  /// key of [`WEAK_KEY_LETTERS`] letters or more stands there after two letters at most, whatever follows it (`vemarinc@`
  /// Víctor Marín's, 2311.18024; `tpengqy@` Q. Y. Peng's, 2004.06253; `eetanggz@` Guozhi Tang's, 2106.10681;
  /// `ylwanguestc@` Yinglong Wang's, 2208.05114); else not at all — a shorter key between other letters is a word's
  /// letters (`yvanwy@` no "van", 2202.02002; `byang@` no "yan", 2602.00582; `bangkok@` no "ang"), and no key runs across
  /// a separator (`ricky_ang@` is no Shengyuan A. Yang's "yang", 2604.02636). The sides are counted in characters (review
  /// rev107f finding 5).
  fn standing(&self, start: usize, end: usize, own: &str) -> Option<Standing> {
    let (before, after) = self.word_around(start, end)?;
    if start == before || end == after {
      return Some(Standing::Full);
    }
    let (lead, trail) = (&self.letters[before..start], &self.letters[end..after]);
    if lead.chars().count() > 2 {
      return None;
    }
    let initials = |letters: &str| letters.chars().all(|c| own.contains(c));
    if trail.chars().count() <= 2 && initials(lead) && initials(trail) {
      Some(Standing::Full)
    } else if self.letters[start..end].chars().count() >= WEAK_KEY_LETTERS {
      Some(Standing::Weak)
    } else {
      None
    }
  }

  /// How a joined spelling (`NameKeys::joined`) at `start..end` stands: at full strength opening a word, its separators
  /// read through (`jhuang@`, `j.huang@`, `nlp.wangyi@`); weakly ending a word it does not open, at
  /// [`WEAK_JOINED_LETTERS`] letters or more (`sechenqi@` Qi Chen's "chenqi", 2003.00397; `xjhuang@`'s "jhuang", which is
  /// no tie-break between Jixuan Huang and Xuanjing Huang, 2510.24320 — review rev107f B1; `pyli@` no Yanan Li's "yli",
  /// 2105.07637); else not at all.
  fn joined_standing(&self, start: usize, end: usize) -> Option<Standing> {
    if self.bounds.contains(&start) {
      Some(Standing::Full)
    } else if self.bounds.contains(&end)
      && self.letters[start..end].chars().count() >= WEAK_JOINED_LETTERS
    {
      Some(Standing::Weak)
    } else {
      None
    }
  }
}

/// A name's score for the local part `part`, and its surname part ([`spelled_owners`], which reads the scores): with its
/// weak spellings or without (`weak`, [`Standing`]).
fn spelling_score(keys: &NameKeys, part: &LocalPart, weak: bool) -> (usize, usize) {
  let local = part.letters.as_str();
  let counts = |standing: Option<Standing>| standing.is_some_and(|s| weak || s == Standing::Full);
  // (a surname's first six letters only outside the letters its given name matched (`given`, their place in the local
  // part): `andrea.rossi@` is Andrea Rossi's, not Andrea Andreassen's "andrea" counted twice, `soeren.hansen@` Søren
  // Hansen's, not Søren Sørensen's — review rev107a finding 2). A truncated surname stands as a whole one does: the
  // longest of the surname's first letters standing there, six or more (`mkaufman@` for Kaufmann, `leyaouan@` for Le
  // Yaouanc)
  let surname_score = |key: &SpelledWord, given: Option<(usize, usize)>, own: &str| {
    let len = key.text.len();
    if part
      .places(&key.text)
      .into_iter()
      .any(|at| counts(part.standing(at, at + len, own)))
    {
      return key.text.chars().count();
    }
    if key.letters < 6 {
      return 0;
    }
    let prefix: String = key.text.chars().take(6).collect();
    let outside = |at: usize| match given {
      Some((start, end)) => at + prefix.len() <= start || at >= end,
      None => true,
    };
    let truncated = part.places(&prefix).into_iter().any(|at| {
      let held = local[at..]
        .chars()
        .zip(key.text.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
      outside(at) && counts(part.standing(at, at + held, own))
    });
    if truncated { 6 } else { 0 }
  };
  // (where the given name stands in the local part: opening it, or anywhere when it has four letters or more)
  let given_place = |given: &SpelledWord, own: &str| {
    let len = given.text.len();
    let fits = |at: usize| counts(part.standing(at, at + len, own));
    if local.starts_with(given.text.as_str()) && fits(0) {
      Some((0, len))
    } else if given.letters >= 4 {
      part
        .places(&given.text)
        .into_iter()
        .find(|&at| fits(at))
        .map(|at| (at, at + len))
    } else {
      None
    }
  };
  // (a joined spelling standing in the local part, its length; the initials, the local part's letters alone)
  // (or its first six letters or more, as a surname's: a truncated family-first spelling, `zhuzhenh22@` for Zhenhao
  // Zhu, `chenyuxi22@` for Yuxiang Chen, 2602.03328, 2609.11716; the r7 re-review's finding 9 — scored the letters it
  // holds less one, below a spelling whole of as many: `wangyu@` is Yu Wang's "wangyu", not Yuxin Wang's "wangyuxin",
  // `liyang@` Yang Li's, not Yangfan Li's; the r8 re-review's finding 3 — where the local part ends there; one going on
  // past a shared start scores its six letters, as before)
  let joined_score = |keys: &NameKeys| {
    keys
      .joined
      .iter()
      .filter_map(|key| {
        if part
          .places(key)
          .into_iter()
          .any(|at| counts(part.joined_standing(at, at + key.len())))
        {
          Some(key.chars().count())
        } else {
          let held = local
            .chars()
            .zip(key.chars())
            .take_while(|(a, b)| a == b)
            .count();
          // (a truncation only when the local part's letters are used up, else its first six letters: `zhangweiwei@`
          // parts from "zhangweiwen" and is Wei Zhang's "zhangwei", the 77y review's finding 5, probe n1)
          if held == local.chars().count() {
            (held >= 6).then(|| held - 1)
          } else {
            (held >= 6).then_some(6)
          }
        }
      })
      .max()
      .unwrap_or(0)
      + keys
        .initials
        .iter()
        .find(|initials| **initials == local)
        .map_or(0, |initials| initials.chars().count())
  };
  // (the score, then its surname part: a tie goes to the surname, `chen@…` Wei Chen's before Chen Li's). A name scores
  // the better of its words spelled apart and a joined spelling, never both added: one spelling of the letters is
  // counted once (`khan.muhammad@ieee.org` scores 12 for Khan Muhammad, "muhammad" and "khan" apart, and 12 for
  // Muhammad Haris Khan, "khanmuhammad" joined, and goes to the surname's spelling, 2602.07835).
  // (a name's score the better of its spellings', its base letters' and its transliterated one's: `mo.schaeffler@`
  // Moritz Schäffler's, html_feedback #7233. A tie between a name spelled literally and one spelled through the
  // transliteration is a tie — an address carries no umlaut, so `mueller@` spells Hans Mueller and Jörg Müller alike
  // and is neither's by its spelling (user ruling 2026-11-05, on review rev107a finding 1); the rest of the placement
  // order decides it)
  keys
    .spellings()
    .map(|keys| {
      // (the initials of the name's words, which a part of it inside a word may stand between: `standing`)
      let own = keys.initials.first().map_or("", String::as_str);
      let given = keys
        .given
        .as_ref()
        .and_then(|given| Some((given, given_place(given, own)?)));
      let place = given.map(|(_, place)| place);
      let surname: usize = keys
        .surnames
        .iter()
        .map(|key| surname_score(key, place, own))
        .sum::<usize>()
        + keys
          .short_surnames
          .iter()
          .find(|short| **short == local)
          .map_or(0, |short| short.chars().count());
      let given = given.map_or(0, |(given, _)| given.text.chars().count());
      ((surname + given).max(joined_score(keys)), surname)
    })
    .max()
    .unwrap_or((0, 0))
}

/// The names (indices into `name_keys`) the local part of `address` spells best: each name scores the letters of its
/// keys the local part holds — a surname whole, or its first six letters (`leyaouan@…` for "A.~Le~Yaouanc"); the given
/// name it opens with, or holds when the given name has four letters or more (2603.24730's
/// `\{yuqihu,~ahna,~vasha\}@berkeley.edu` for Yuqi Hu, Ahna R. Girshick, Vasha DuTell; 2603.20851's
/// `nlp2ct.shanshan@…` for Shanshan Wang, `lidiasc@…` for Lidia S. Chao), or a joined spelling it opens with when
/// that scores better (the better of the two, never their sum: 2602.07835) — and the highest scores own it: a longer
/// spelling outranks a key it holds (`fucheng.fuc@…` is Cheng Fu's, not Jiawei Chen's, 2403.00801; `wangchiew@…`
/// Wang-Chiew Tan's, not Xiaolan Wang's, 2104.01371; `yipengzhang97@…` Yipeng Zhang's, not Xiaoying Zhang's,
/// 2406.06326), a tie going to the surname's spelling. No name when none scores.
///
/// Each key counts only where the one boundary predicate places it ([`LocalPart::standing`], [`LocalPart::joined_standing`];
/// census138 EMAILSPELL, OXIDIZED_DESIGN_DIVERGENCES #970 "107f"). A weak spelling owns an address only no other name
/// spells; where another does, the names are ranked by their full-strength spellings, and a weak one never decides
/// against a full one: the two rankings agreeing, their owner; else every name either ranks first — a tie, which the rest
/// of the placement order (the list's order, the elimination, the block) decides. All of them weak, the one ranking
/// decides. Nothing is guessed (FIDELITY_CHARTER §4): `xjhuang@`
/// beside Jixuan Huang and Xuanjing Huang is a tie (each "huang" ending the word; Jixuan's "jhuang" weak, review rev107f
/// B1, 2510.24320), and so is `xywang@` beside Yi Wang and Xiaoyu Wang, which the elimination gives to Xiaoyu Wang once
/// `yiwang@` is Yi Wang's.
pub(in crate::base_utilities) fn spelled_owners(
  name_keys: &[NameKeys],
  address: &str,
) -> Vec<usize> {
  let Some(part) = LocalPart::of(address) else {
    return Vec::new();
  };
  let scores = |weak: bool| -> Vec<(usize, usize)> {
    name_keys
      .iter()
      .map(|keys| spelling_score(keys, &part, weak))
      .collect()
  };
  let all = scores(true);
  let spelling: Vec<usize> = (0..all.len()).filter(|&at| all[at].0 > 0).collect();
  if spelling.len() < 2 {
    return spelling;
  }
  // (several names spell it: ranked with their weak spellings and without; where a weak one would decide against a full
  // one, a tie. All of them weak, the longer spelling ranks first, with no tie-break by the surname: `sechenqi@` Qi Chen's
  // "chenqi" beside Wei Chen's "chen"; `kkwongar@` no one's beside Hiu Tung Wong and Wong Kam-Kwai, 2608.26715)
  let first = |scores: &[(usize, usize)]| -> Vec<usize> {
    let best = scores.iter().copied().max().unwrap_or((0, 0));
    (0..scores.len()).filter(|&at| scores[at] == best).collect()
  };
  let full = scores(false);
  if full.iter().all(|score| score.0 == 0) {
    let best = all.iter().map(|score| score.0).max().unwrap_or(0);
    return (0..all.len()).filter(|&at| all[at].0 == best).collect();
  }
  let mut owners = first(&all);
  for at in first(&full) {
    if !owners.contains(&at) {
      owners.push(at);
    }
  }
  owners.sort_unstable();
  owners
}

/// A name's full-strength score for the local part `part` ([`spelling_score`], no weak spelling counted).
fn full_score(part: &LocalPart, keys: &NameKeys) -> usize { spelling_score(keys, part, false).0 }

/// Whether an author's own call keeps `address` from `owner`, the name spelling it best (FIDELITY_CHARTER §4 #420): it
/// moves only if the owner's full-strength score ([`spelling_score`], no weak spelling counted) is strictly higher than
/// the holder's (`holder`), so a tie never moves an own call — amsart's `\author{Yang Zhang}…\email{yang91@…}` beside
/// Hengyun Yang, 1707.09730; `minhnnhat@` under Nhat Ho beside Anh-Minh Phan, 2511.18493; `\author{Lay Kee
/// Ang}\email{ricky\_ang@…}`, 2604.02636 — and a longer spelling does, whichever part of it the holder's key is:
/// `sanderson@` on Mark Anderson's call is Lee Sanderson's, `anderson@` on Peter Andersen's is Maria Anderson's (review
/// rev107f finding 3), `angela.rossi@` on Lay Kee Ang's is Angela Rossi's (rev107f2 finding 1), `dongsu.kim@` on Jang
/// Soo Kim's is Dongsu Kim's (0902.4311), `giordano.pola@` on Gianni Pola's Giordano's (0907.3301), `zx.liang55@` on Yi
/// Liang's Zhu-Xing's (1008.3764), `filippo.neri.email@` on the misread "Italy. email"'s Filippo Neri's (2107.06981).
/// So a weak spelling moves no own call (`yhsheng@` on Husheng Yu's stays, no Xiangmin Shen's "shen", 2411.01273;
/// `zhonggong.hc@` on Chao Huang's, no Wang Hong's "hong", 2609.11148) — but for an address set after the last author
/// (`last`), which no name's position claims: it is the name's that spells it, the holder's not standing there even weakly
/// (`tpengqy@` after Y. J. Shang, Q. Y. Peng's, 2004.06253; #420's count, 22 of 24 addresses set after the last author his;
/// `kkwongar@` stays on Wong Kam-Kwai beside Hiu Tung Wong, 2608.26715). Read by
/// `insert::respell_emails` and the observer's `Resolver::respell_emails`, which else move it to the name spelling it
/// better.
pub(in crate::base_utilities) fn own_call_keeps(
  holder: &NameKeys,
  owner: &NameKeys,
  address: &str,
  last: bool,
) -> bool {
  let Some(part) = LocalPart::of(address) else {
    return false;
  };
  full_score(&part, owner) <= full_score(&part, holder)
    && !(last && spelling_score(holder, &part, true).0 == 0)
}

/// The owner (index into `name_keys`) of each of a line's `addresses`: the one name it spells ([`spelled_owner`]); and,
/// by elimination, an address no name spells goes to a name no address spells — on this line, or on another e-mail line
/// of the block (`taken`, by index; 2605.06223's `kyle.min@…` on the next line leaves `treejhk@…` Junhyuk Kwon's) — only
/// where the order is confirmed (user decision 2026-10-20, batch 77p r4):
///  - exactly one address and one name are left (`\{lzhang,zhangw,smith\}@…` under Wei Zhang, Li Zhang, Bob Smith:
///    "zhangw" Wei Zhang's once "lzhang" is Li Zhang's); or
///  - the addresses the names spell stand in name order (one at least), and the addresses left are as many as the names
///    left (2605.26032's `\{jzxchen,chenzhuo,archerdw,gore,congyued,billf,soljacic\}@mit.edu`), or those between two
///    of them as many as the names between their owners (never a run before the first or after the last, which no
///    spelled address bounds: user ruling R10 2026-10-20, 2509.10377), every one of those
///    names counted, spelled on another line or not, and none of them spelled or given another address (`\{aszlam,jase,
///    robfergus\}@fb.com`: "jase" to Jason Weston, 1503.08895; 2603.24730's `j8675309@mit.edu` to Jennifer E. Corbett;
///    2303.11225's `xuta@…`, `musclewu@…` between `tianyuhe@…`, `tabaltru@…`, `runnan.li@…`).
///
/// A name left over is no one's to give while another e-mail line of the block holds an address no name spells
/// (`elsewhere_open`): that address may be the name's, so neither the one-left nor the as-many-left rule places one
/// (the r5 re-review's q6, `\{aable,q77,ddorn\}@x.edu \\ \{bbaker,z99\}@y.edu`: `q77@` is not Cat Cole's;
/// 2602.23777's `dongsli@` on the next line). The run between two spelled owners is read by the order alone, every
/// author between them counted, spelled on another line or not: 2602.23777's `nnwang@` between `xu_zhipeng@` and
/// `dcheng@` has Zilong Wang, Xinyang Jiang and Dongsheng Li between Zhipeng Xu and De Cheng, so it goes to none of
/// them; 2303.11225's `xuta@` has Xu Tan alone between Tianyu He and Tadas Baltrušaitis, as its PDF prints the list
/// in the authors' order (and its `\thanks` names `xuta@microsoft.com` Xu Tan's), though `ztk21@` on the line above
/// spells no one (batch 77p r6).
///
/// Not in order — a list by institution (2509.10377), or none spelled (`\{bb,aa\}@x.edu` reads by the initials, not
/// the order) — the rest stay `None`, as does a role mailbox (RFC 2142's `info@`, `support@`, …, [`is_role_mailbox`]),
/// which is no person's: the block's, on its shared creator (user ruling R10 2026-10-20).
pub(in crate::base_utilities) fn owners_by_elimination(
  name_keys: &[NameKeys],
  addresses: &[String],
  taken: &[bool],
  elsewhere_open: bool,
) -> Vec<Option<usize>> {
  let mut owners: Vec<Option<usize>> = addresses
    .iter()
    .map(|address| spelled_owner(name_keys, address))
    .collect();
  let free = |name: usize| !taken.get(name).copied().unwrap_or(false);
  let left: Vec<usize> = (0..name_keys.len())
    .filter(|name| !owners.contains(&Some(*name)) && free(*name))
    .collect();
  let open: Vec<usize> = (0..owners.len())
    .filter(|at| owners[*at].is_none() && !is_role_mailbox(&addresses[*at]))
    .collect();
  if let ([at], [name]) = (open.as_slice(), left.as_slice())
    && !elsewhere_open
  {
    owners[*at] = Some(*name);
    return owners;
  }
  let spelled: Vec<usize> = owners.iter().flatten().copied().collect();
  if open.is_empty() || spelled.is_empty() || !spelled.windows(2).all(|pair| pair[0] < pair[1]) {
    return owners;
  }
  // (as many left as there are names left: in order, FIDELITY_CHARTER §4's condition, so only when the order holds with
  // the spelled addresses. It fails on 2606.04646, whose position read swapped `dishkw@` and `e0556732@`, and on 2605.26032,
  // `congyued@` before `billf@` with Deng after Freeman, where it placed `jzxchen@` and `billf@` right by luck; those and
  // 2102.08716's list stay on the block, rev108a3 B3)
  if open.len() == left.len() && !elsewhere_open {
    let mut placed = owners.clone();
    for (at, name) in open.iter().zip(&left) {
      placed[*at] = Some(*name);
    }
    let in_order: Vec<usize> = placed.iter().flatten().copied().collect();
    if in_order.windows(2).all(|pair| pair[0] < pair[1]) {
      return placed;
    }
  }
  // (each run of open addresses, with the owners of the spelled addresses around it, and every name between them)
  let mut run: Vec<usize> = Vec::new();
  let mut below: Option<usize> = None;
  for at in 0..=owners.len() {
    let bound = if at == owners.len() {
      Some(name_keys.len())
    } else {
      owners[at]
    };
    match bound {
      Some(above) => {
        let between = below.map_or(0, |below| below + 1)..above;
        // (a run before the first spelled owner or after the last is bounded by the list's end alone, which no spelled
        // address confirms: no position placement at a list's end (user ruling R10 2026-10-20; 2509.10377's `12421181@`
        // first in its list, the r5 re-review's q7 `z99@`); a run between two spelled owners is read by the order,
        // 2303.11225)
        let at_an_end = below.is_none() || at == owners.len();
        if !run.is_empty() && between.len() == run.len() && between.clone().all(free) && !at_an_end
        {
          for (address, name) in run.iter().zip(between) {
            owners[*address] = Some(name);
          }
        }
        run.clear();
        below = Some(above);
      },
      None if open.contains(&at) => run.push(at),
      None => {},
    }
  }
  owners
}

/// The owners FIDELITY_CHARTER §4's chain gives the e-mail addresses printed inside an affiliation-kind entry (an
/// affiliation, alternate affiliation or address: its lines of addresses, its e-mail calls), the one Link step both
/// engines run — the author block's at the flush (`insert::link_entry_addresses`) and the frontmatter observer's
/// reading (`Resolver::take_out_entry_addresses`, `Resolver::answer_inner_emails`). `spelled` is each address of the
/// entry's the author it spells (any live author: [`spelled_owner`]), `run` the addresses read here (one run of them,
/// in print order), `carriers` the authors holding the entry, in order, and `symbol_line` whether a footnote symbol
/// leads the entry, `addressed` the carriers holding an address of their own beside it, and `sole_author` whether the
/// document has one author, whose every address is (the charter's "the group's only author"). Each address of `run`, in
/// order: the author it spells (`owner:spelled`); else the entry's only carrier, when it is the entry's one address no
/// one spells, no other address of the entry spells that carrier and the carrier holds none of its own
/// (`owner:only-carrier`: `{steen, sund}@…` under Steen alone is two people's); else, on a symbol-led line, every carrier
/// no other address of the entry spells (`owner:carriers`, the charter's "every symbol
/// carrier", 2026-10-30 — before the order, so `$^\dagger$alpha@x.org, beta@y.org` under two carriers of `\dagger` is
/// both carriers'); else, the run's addresses one per carrier that every spelled one agrees with, the carrier at its
/// place (`owner:order`, OD #805); else `None`, the block's, once (`owner:block`: ruling E, main session 2026-11-02 —
/// the charter's step 4 wins over the "stays in the entry" fail-safe of OD #938; astro-ph0512555's `Email:dzg@,
/// xywang@, hyf@` under four carriers of entry 2). Surname-first initials (`dzg` for Z. G. Dai) are not read: a false
/// match is worse than the block.
pub fn entry_address_owners(
  spelled: &[Option<usize>],
  run: std::ops::Range<usize>,
  carriers: &[usize],
  symbol_line: bool,
  addressed: &[usize],
  sole_author: bool,
  facts: &EntryOwnerFacts,
) -> Vec<(Option<Vec<usize>>, &'static str)> {
  // (the only carrier's only where the entry prints that one address no one spells and the carrier holds none of its own:
  // several unspelled under one name are no one name's — 2403.17706's `1022041122@, 1223045440@, hhp@` under Haiping
  // Huang's second institute, 2506.06908's welded entry under Chenzhi Xu —, nor is one under a name already holding its
  // own — 2411.17073's `bull@` under Awais Naeem, `awais.naeem@` his)
  let unspelled = spelled.iter().filter(|s| s.is_none()).count();
  let in_order = run.len() == carriers.len()
    && carriers.len() >= 2
    && facts.spellers[run.clone()]
      .iter()
      .zip(carriers)
      .all(|(names, c)| names.len() < 2 || names.contains(c))
    && spelled[run.clone()]
      .iter()
      .zip(carriers)
      .all(|(s, c)| s.is_none_or(|s| s == *c));
  // (the symbol carriers no other address of the entry spells, as the only-carrier step excludes them: 1907.01388's
  // `* chinarentao@163.com, zhutou@ustc.edu` under Tao Ren* and Tao Zhou* gives `zhutou@` to Zhou alone, review rev100b
  // S1; none left, the order and the block decide)
  let free: Vec<usize> = carriers
    .iter()
    .copied()
    .filter(|c| !spelled.contains(&Some(*c)))
    .collect();
  let mut owners: Vec<(Option<Vec<usize>>, &'static str)> = run
    .clone()
    .map(|at| match (spelled[at], carriers) {
      (Some(owner), _) => (Some(vec![owner]), "owner:spelled"),
      (None, [only])
        if sole_author
          || (!spelled.contains(&Some(*only)) && unspelled == 1 && !addressed.contains(only)) =>
      {
        (Some(vec![*only]), "owner:only-carrier")
      },
      (None, [] | [_]) => (None, "owner:block"),
      (None, _) if symbol_line && !free.is_empty() => (Some(free.clone()), "owner:carriers"),
      (None, _) if in_order => (Some(vec![carriers[at - run.start]]), "owner:order"),
      (None, _) => (None, "owner:block"),
    })
    .collect();
  // (the one address left on the block of several carriers: the carrier marked the corresponding author's, when they own no
  // other address of the run — the old split's row e-mail owners, `shared_row::row_email_calls`)
  let left: Vec<usize> = (0..owners.len())
    .filter(|&at| owners[at].0.is_none())
    .collect();
  if let ([at], Some(author)) = (left.as_slice(), facts.corresponding)
    && !facts.no_ones[run.start + *at]
    && carriers.len() > 1
    && carriers.contains(&author)
    && owners
      .iter()
      .all(|(to, _)| to.as_ref().is_none_or(|to| !to.contains(&author)))
  {
    owners[*at] = (Some(vec![author]), "owner:corresponding");
  }
  owners
}

/// What [`entry_address_owners`] reads beyond the spelling (the old split's row e-mail owners, `shared_row::row_email_calls`'s
/// `row_address_owners`, Z18c): every name each address spells (`spellers`) — one spelling several names alike, none of
/// them the carrier at its place, puts the run in no order: `x1@, x2@, lee@` under Ann Lee, Bob Lee and Cid Cole is the
/// block's (repro llncs_ambiguous_spelling_is_no_place), while 2102.08148's `jtchen721@` at Jintai Chen's place beside
/// Danny Z. Chen stays his —; and the one carrier marked the corresponding author (a note naming them so),
/// whose the one address left on the block of several is (`\author{Ann Able\thanks{Corresponding author} \and Bob
/// Baker}\institute{Univ X\\ \email{zz@x.edu}}`, repro llncs_corresponding_author_takes_the_left_address).
/// A template's address or a list the reader could not part (`{first.last}@x.org`, `is_pattern_address`,
/// `lists_local_parts`) is no one's, never the corresponding author's (`no_ones`: 2408.14427's `\{first.last\}@uii-ai.com`
/// beside a corresponding Meng Zheng).
pub struct EntryOwnerFacts<'a> {
  pub spellers:      &'a [Vec<usize>],
  pub no_ones:       &'a [bool],
  pub corresponding: Option<usize>,
}

/// The owners of the addresses of a `{…}@domain` list set over several lines, which the Kind step for an address line
/// took out of an affiliation-kind entry (`insert::link_entry_addresses`, the list's `LINE_LIST`), placed as the author
/// block's reader places the list on one line (FIDELITY_CHARTER §4's order, review rev107a finding 4; html_feedback
/// #7233): `name_keys` every author's, in order; `addresses` the list's, in print order; `spelled` each one's owner as the
/// Link step reads it ([`spelled_owner`], an entry's word aside: [`is_entry_word_mailbox`]); `taken` the authors an
/// address outside the list spells or who hold one; `spelled_elsewhere` whether an address outside the list spells an
/// author; `elsewhere_open` whether one outside it spells no one and is placed on no author. Each address: the author it
/// spells (`owner:spelled`); else, the list one address per author that every spelled one agrees with and no address
/// outside it spells an author, the author at its place (`owner:order`, OXIDIZED_DESIGN_DIVERGENCES #805 — the
/// synthetic `email_list_split_order_only`); else the only author's, where there is one (`owner:only-author`, the
/// charter's "the group's only author", as [`entry_address_owners`]' `sole_author` — 2311.03226); else by elimination
/// (`owner:elimination`, batch 77p r4, [`owners_by_elimination`] — 1808.00588's `kdm@`; 2305.13191's `wshui@`, `drot@`,
/// `ballemig@`, 9 entries for 10 names); else `None`, the block's
/// (`owner:block`, OD #159's block creator).
pub(in crate::base_utilities) fn list_owners(
  name_keys: &[NameKeys],
  addresses: &[String],
  spelled: &[Option<usize>],
  taken: &[bool],
  spelled_elsewhere: bool,
  elsewhere_open: bool,
) -> Vec<(Option<usize>, &'static str)> {
  let in_order = addresses.len() == name_keys.len()
    && name_keys.len() >= 2
    && !spelled_elsewhere
    && spelled
      .iter()
      .enumerate()
      .all(|(at, owner)| owner.is_none_or(|owner| owner == at));
  if in_order {
    return spelled
      .iter()
      .enumerate()
      .map(|(at, owner)| match owner {
        Some(owner) => (Some(*owner), "owner:spelled"),
        None => (Some(at), "owner:order"),
      })
      .collect();
  }
  if let [_] = name_keys {
    return spelled
      .iter()
      .map(|owner| match owner {
        Some(owner) => (Some(*owner), "owner:spelled"),
        None => (Some(0), "owner:only-author"),
      })
      .collect();
  }
  owners_by_elimination(name_keys, addresses, taken, elsewhere_open)
    .into_iter()
    .zip(spelled)
    .zip(addresses)
    .map(|((owner, spelled), address)| match (spelled, owner) {
      (Some(owner), _) => (Some(*owner), "owner:spelled"),
      // (an address the elimination reads as spelled that the Link step does not — an entry's word — is the block's)
      (None, Some(owner)) if spelled_owner(name_keys, address).is_none() => {
        (Some(owner), "owner:elimination")
      },
      _ => (None, "owner:block"),
    })
    .collect()
}

/// The one name (index into `name_keys`) the local part of `address` spells ([`spelled_owners`]), or `None` when no name or
/// several do.
pub(in crate::base_utilities) fn spelled_owner(
  name_keys: &[NameKeys],
  address: &str,
) -> Option<usize> {
  if lists_local_parts(address) {
    return None;
  }
  match spelled_owners(name_keys, address).as_slice() {
    [owner] => Some(*owner),
    _ => None,
  }
}

/// A mark's operand without its footnote symbols (`is_symbol_mark`), comma-separated pieces kept apart: `1,\dagger` is
/// `1`, a number or letter glued to a symbol (`1\ast`, `1{\dagger}`: `split_glued_symbol`) its label; `None` when only
/// symbols are left (`*`, `\dagger`).
pub(in crate::base_utilities) fn label_pieces(operand: &[Token]) -> Option<Vec<Token>> {
  let mut pieces: Vec<&[Token]> = Vec::new();
  let (mut start, mut i) = (0, 0);
  while i < operand.len() {
    if operand[i].get_catcode() == Catcode::BEGIN {
      i = skip_group(operand, i);
      continue;
    }
    if operand[i] == T_OTHER!(",") {
      pieces.push(&operand[start..i]);
      start = i + 1;
    }
    i += 1;
  }
  pieces.push(&operand[start.min(operand.len())..]);
  let mut kept: Vec<Token> = Vec::with_capacity(operand.len());
  for piece in pieces {
    let piece = {
      let lead = piece.iter().take_while(|t| **t == T_SPACE!()).count();
      let trail = piece[lead..]
        .iter()
        .rev()
        .take_while(|t| **t == T_SPACE!())
        .count();
      &piece[lead..piece.len() - trail]
    };
    let unbraced = match piece {
      [open, inner @ .., close]
        if open.get_catcode() == Catcode::BEGIN
          && close.get_catcode() == Catcode::END
          && skip_group(piece, 0) == piece.len() =>
      {
        inner
      },
      _ => piece,
    };
    let label = if unbraced.is_empty() || is_symbol_mark(unbraced) {
      continue;
    } else if let Some((label, _)) = split_glued_symbol(piece) {
      label
    } else {
      piece.to_vec()
    };
    if !kept.is_empty() {
      kept.push(T_OTHER!(","));
    }
    kept.extend(label);
  }
  (!kept.is_empty()).then_some(kept)
}

/// The place among a row's `names` of the owner of each of `addresses`, the e-mail addresses printed under the row
/// ([`row_email_owners`]): the name it spells; else, in a list of one address per name that every spelled address
/// agrees with, the name at its place; else `None`, the row's names' (FIDELITY_CHARTER §4). An address several names
/// spell agrees with the list only when the name at its place is one of them: `lee@x.edu` at Cid Cole's place under
/// "Ann Lee, Bob Lee and Cid Cole" spells another author than the one there, so the list is in no author order (the
/// 85b review's r01; [`addresses_in_order`] reads such an address as none's). A list of local parts the reader could
/// not part (`{zz|yy}@x.edu`, [`lists_local_parts`]) is several authors' address, so no list holding one is one address
/// per name (user ruling R10: the block's).
pub(in crate::base_utilities) fn row_address_owners(
  names: &[Tokens],
  addresses: &[String],
) -> Vec<Option<usize>> {
  let keys: Vec<NameKeys> = names.iter().map(NameKeys::of).collect();
  let in_order = addresses.len() == names.len()
    && addresses.iter().enumerate().all(|(at, address)| {
      let spelled = spelled_owners(&keys, address);
      !lists_local_parts(address) && (spelled.is_empty() || spelled.contains(&at))
    });
  addresses
    .iter()
    .enumerate()
    .map(|(at, address)| spelled_owner(&keys, address).or(in_order.then_some(at)))
    .collect()
}

/// The owner of each of `addresses` among a row's `names` by its spelling alone ([`spelled_owner`]), when the
/// addresses printed under the row are not all read, so no list of them is known to be one per name.
pub(in crate::base_utilities) fn row_address_spelled_owners(
  names: &[Tokens],
  addresses: &[String],
) -> Vec<Option<usize>> {
  let keys: Vec<NameKeys> = names.iter().map(NameKeys::of).collect();
  addresses
    .iter()
    .map(|address| spelled_owner(&keys, address))
    .collect()
}
