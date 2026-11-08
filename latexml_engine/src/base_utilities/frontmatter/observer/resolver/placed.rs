//! The content invariant, checked on the placed output (FRONTMATTER_SINGLE_ENGINE_PLAN.md, the safety net's bar (2): every
//! word TeX printed in an author block is in the output exactly once, or counted in a Warning; OXIDIZED_DESIGN_DIVERGENCES
//! #980). The reading's word check (`words`) sees what the reading writes before the emit; what goes wrong after it — a
//! shared entry's address the Kind step keeps for every holder, digestion's junk in a name (2506.02663), a call written
//! whole beside its piece (2607.00684's `mtablo@postech.ac.kr` twice), a binding's lost entry text — shows only in the
//! frontmatter `insert_frontmatter` placed. So one pass after it (Perl's `insertFrontMatter` and `relocateAnnotations`,
//! Base_Utility.pool.ltxml:736-740, :891-900), under `LATEXML_FM_EMIT=2` only: the words the ledger's author blocks and
//! creator calls printed (their rehearsed events), against the words the placed creators hold (`writer::dom`'s reading of
//! the document). It counts, it never answers: placement is over.
//!
//! What counts, both sides by one definition ([`ink_words`]): a word of three characters or more, or of two letters; an
//! address is a unit of its own, its words none of the text's (1605.03019's `IDSIA` beside `@idsia.ch`). *Lost*: a word or
//! an address the inks print more often than the placed frontmatter holds it, every copy counted (two authors given one
//! printed line hold it twice), and the publication notes and a contact's label (its `name`) holding it too. *Excess*: a
//! word placed more often than printed — an item several creators share (one printed `\affiliation`, `annotate=all`)
//! counted once, as often as one creator holds it; an e-mail contact every time, an address being one person's (one address
//! on two creators, or twice on one). What may print beside the authors without being theirs only answers an excess: a
//! separator's word (`and`), a link's target (an ORCID's id behind a logo), a footnote's text (`\footnotetext`), a
//! publication note's, a call a block's own expansion made. An excess needs every live block and creator call read: a
//! block no rehearsal read prints words no event holds.

use std::collections::{BTreeMap, BTreeSet};

use super::{
  super::ledger::{Entry, Ledger, Outcome, SkipReason},
  words::{ink_words, live_blocks, words_of},
  *,
};
use crate::base_utilities::author_block::CUT_LIST;

/// One item of the placed frontmatter (`writer::dom`'s reading of a creator): its creator, what it is, its printed text.
pub(in crate::base_utilities::frontmatter::observer) struct PlacedItem {
  /// the creator holding it, by its place among the document's creators
  pub holder: usize,
  pub kind:   PlacedKind,
  pub text:   String,
}

/// What a [`PlacedItem`] is, for how often it counts: a name (every copy), an e-mail contact or any other item, by its
/// compared text (`dom::norm`): one each holder shares, an e-mail's addresses every copy (an address is one person's).
pub(in crate::base_utilities::frontmatter::observer) enum PlacedKind {
  Name,
  Email(String),
  Item(String),
}

/// The placed frontmatter as the check reads it: the creators' items, and the texts that hold a printed word apart from
/// them (a publication note, a contact's label).
#[derive(Default)]
pub(in crate::base_utilities::frontmatter::observer) struct Placed {
  pub items:  Vec<PlacedItem>,
  pub beside: Vec<String>,
}

/// The check's findings: the words (as first printed) and addresses lost, and those placed in excess.
#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::base_utilities::frontmatter::observer) struct ContentCheck {
  pub lost:   Vec<String>,
  pub excess: Vec<String>,
}

/// Whether a word counts ([`ink_words`]'s): three characters or more, or two letters (`Li`, `Wu`); a digit or two or a
/// letter is a mark's.
fn counted(word: &str) -> bool {
  let n = word.chars().count();
  n > 2 || (n == 2 && word.chars().all(char::is_alphabetic))
}

/// A multiset of the words and addresses texts print, each word with its first spelling.
#[derive(Default)]
struct Tally {
  words:     BTreeMap<String, (usize, String)>,
  addresses: BTreeMap<String, usize>,
}

impl Tally {
  fn add(&mut self, text: &str, times: usize) { self.add_parts(text, times, times); }

  /// `text`'s words `words` times, its addresses `addresses` times.
  fn add_parts(&mut self, text: &str, words: usize, addresses: usize) {
    let (printed, printed_addresses) = ink_words(text);
    for word in printed.into_iter().filter(|w| counted(w)) {
      let entry = self
        .words
        .entry(word.to_lowercase())
        .or_insert_with(|| (0, word.clone()));
      entry.0 += words;
    }
    for address in printed_addresses {
      *self.addresses.entry(address).or_default() += addresses;
    }
  }

  /// The words of `address`'s local part, `times` more.
  fn add_local(&mut self, address: &str, times: usize) {
    if times == 0 {
      return;
    }
    for word in local_words(address) {
      let entry = self.words.entry(word.clone()).or_insert((0, word));
      entry.0 += times;
    }
  }

  /// `word`, lowercased already, `times` more.
  fn add_word(&mut self, word: &str, times: usize) {
    let entry = self
      .words
      .entry(word.to_string())
      .or_insert_with(|| (0, word.to_string()));
    entry.0 += times;
  }

  fn word(&self, word: &str) -> usize { self.words.get(word).map_or(0, |(n, _)| *n) }

  fn address(&self, address: &str) -> usize { self.addresses.get(address).copied().unwrap_or(0) }
}

/// The texts `events` print as inks: each line's Names read whole, as the reading reads a line's addresses (`Switzerland
/// {adam`, `samuli`, `monaldo}@idsia.ch` one list, 1605.03019), a Contact or a Note on its own (a url beside an address,
/// `https://members.loria.fr/AClement alexandre.clement@loria.fr`, is no list, 2206.10577); a contact a mark's formula
/// holds beside them (an ORCID's logo in `$^{1\,\orcidicon{…}}$`, 2210.03748; the names a formula prints with their marks
/// are names, `$\text{Li Cheng}^{1}, …$`, 2001.05492, 2303.12302). Into `beside`, what prints beside them: a separator's
/// word, a link's target where its anchor prints no address of its own (`Written::excess`'s).
fn ink_texts(events: &[Event], beside: &mut Tally) -> Vec<String> {
  let marks: Vec<(u16, u16)> = events
    .iter()
    .filter(|m| m.kind == EventKind::Mark)
    .filter_map(|m| m.formula.and_then(|f| f.span))
    .collect();
  let in_a_mark = |e: &Event| {
    e.span.is_some_and(|(s, t)| {
      marks
        .iter()
        .any(|&(from, to)| from <= s && t <= to && (s, t) != (from, to))
    })
  };
  let mut texts = Vec::new();
  let mut line: Vec<&Event> = Vec::new();
  for e in events {
    match &e.kind {
      EventKind::Name | EventKind::Sep(SepKind::Comma) => line.push(e),
      EventKind::Contact if in_a_mark(e) => beside.add(&e.text, 1),
      EventKind::Contact => texts.push(e.text.clone()),
      EventKind::Note if !e.inkless => texts.push(e.text.clone()),
      EventKind::Sep(SepKind::Word(word)) => beside.add(word, 1),
      // (a line break inside a printed group of local parts parts no list: `\{yogarshi, …,\\ vittorca, drot, ballemig\}@amazon.com`,
      // 2305.13191, `Event::braces`)
      EventKind::Break if e.braces == 0 => {
        texts.push(events_text(&line));
        line.clear();
      },
      _ => {},
    }
    for contact in std::iter::once(e).chain(&e.inner) {
      if contact.kind == EventKind::Contact
        && let Some(target) = contact.target.as_deref()
        && addresses_in(&contact.text).is_empty()
      {
        beside.add(target, 1);
      }
    }
  }
  texts.push(events_text(&line));
  texts
}

/// The label an API call's class prints with its entry, its `name=` key (revtex's `\email[Corresponding author e-mails:
/// ]{…}`, 2404.07637, which the Kind step writes as each owner's role note).
fn call_label(attrs: &str) -> Option<&str> {
  attrs.split(',').find_map(|key| key.strip_prefix("name="))
}

/// The content check of `placed` against what the ledger's live blocks ([`live_blocks`]) and creator calls printed. A call
/// printing what another call prints too counts once (revtex's `\affiliation{\Argonne}` listed before the authors to number
/// the institutes and again under one, 2103.04797); the pieces a binding cut one printed list into (its calls' shared
/// `_cutlist`, [`CUT_LIST`]) are read as one text, the list's addresses with them (`{zeba.khanam, sehsan,
/// kdm}@essex.ac.uk`, 1808.00588).
pub(in crate::base_utilities::frontmatter::observer) fn content_check(
  ledger: &Ledger,
  placed: &Placed,
  kept_whole: &dyn Fn(usize) -> bool,
) -> ContentCheck {
  let (mut printed, mut beside) = (Tally::default(), Tally::default());
  let mut all_read = true;
  // (a block written whole is TeX's own digestion of it, printing what its separators print — a class's `\and` printing
  // "and", which the rehearsal binds to an event printing nothing: JAIR's, 1908.02923 —, words no event holds: no word of
  // it is counted in excess, its addresses still are)
  let mut whole = false;
  for (index, block) in live_blocks(ledger) {
    whole |= kept_whole(index);
    match &block.outcome {
      Outcome::Events { events, .. } => {
        for text in ink_texts(events, &mut beside) {
          printed.add(&text, 1);
        }
      },
      Outcome::Skipped(SkipReason::Empty) => {},
      _ => all_read = false,
    }
  }
  let mut calls: BTreeSet<Vec<String>> = BTreeSet::new();
  // (the pieces of a cut list so far, and its key)
  let mut list: (Option<&str>, Vec<String>) = (None, Vec::new());
  let aside = |events: &[Event], beside: &mut Tally| {
    for text in ink_texts(events, beside) {
      beside.add(&text, 1);
    }
  };
  for entry in ledger.entries() {
    let piece = match entry {
      Entry::Api(call) if !call.cleared && creator_call(&call.tag, &call.parent) => {
        if let Some(label) = call_label(&call.attrs) {
          beside.add(label, 1);
        }
        if call.inside {
          // (a block's own expansion's call: its block's events print it)
          aside(&call.events, &mut beside);
          None
        } else if !call.rehearsed {
          all_read = false;
          None
        } else if calls.insert(words_of(&events::printed_text(&call.events)).collect()) {
          let cut = call
            .attrs
            .split(',')
            .find_map(|key| key.strip_prefix(CUT_LIST)?.strip_prefix('='));
          Some((cut, ink_texts(&call.events, &mut beside)))
        } else {
          None
        }
      },
      Entry::Api(call) if !call.cleared && call.tag == "ltx:pubnote" => {
        aside(&call.events, &mut beside);
        None
      },
      Entry::NoteText(text) => {
        aside(&text.events, &mut beside);
        None
      },
      _ => None,
    };
    let Some((cut, texts)) = piece else {
      continue;
    };
    if cut.is_none() || cut != list.0 {
      printed.add(&list.1.join(", "), 1);
      list = (cut, Vec::new());
    }
    list.1.extend(texts);
    if cut.is_none() {
      printed.add(&list.1.join(", "), 1);
      list = (None, Vec::new());
    }
  }
  printed.add(&list.1.join(", "), 1);
  // (every copy placed, and the texts beside: what a lost word is held by; each text read once, with its copies — a shared
  // entry is one text under many creators, 2103.04797's 975)
  let mut copies: BTreeMap<&str, usize> = BTreeMap::new();
  let texts = placed.items.iter().map(|item| item.text.as_str());
  for text in texts.chain(placed.beside.iter().map(String::as_str)) {
    *copies.entry(text).or_default() += 1;
  }
  let mut held = Tally::default();
  for (text, n) in copies {
    held.add(text, n);
  }
  // (a shared item once, as often as one creator holds it; a name every copy, and an e-mail's addresses: what an excess is
  // counted by)
  let (mut names, mut emails): (BTreeMap<&str, usize>, BTreeMap<&str, usize>) = Default::default();
  let mut per_holder: BTreeMap<(&str, usize), (usize, &str, bool)> = BTreeMap::new();
  for item in &placed.items {
    let (key, email) = match &item.kind {
      PlacedKind::Name => {
        *names.entry(&item.text).or_default() += 1;
        continue;
      },
      PlacedKind::Email(key) => {
        *emails.entry(&item.text).or_default() += 1;
        (key, true)
      },
      PlacedKind::Item(key) => (key, false),
    };
    per_holder
      .entry((key.as_str(), item.holder))
      .or_insert((0, item.text.as_str(), email))
      .0 += 1;
  }
  let mut shared = Tally::default();
  for (text, n) in names {
    shared.add(text, n);
  }
  for (text, n) in emails {
    shared.add_parts(text, 0, n);
  }
  let mut most: BTreeMap<&str, (usize, &str, bool)> = BTreeMap::new();
  for ((key, _), (n, text, email)) in per_holder {
    let at = most.entry(key).or_insert((0, text, email));
    at.0 = at.0.max(n);
  }
  for (n, text, email) in most.into_values() {
    shared.add_parts(text, n, if email { 0 } else { n });
  }
  // (an address one side reads where the other reads its local part's words — a list's parts a note keeps, `{\it aable,
  // bbaker, cat@a.org}`, 2003.14069 —: the same ink, so neither lost nor in excess)
  let lost: Vec<(String, usize)> = printed
    .addresses
    .iter()
    .map(|(address, &n)| (address.clone(), n.saturating_sub(held.address(address))))
    .collect();
  for (address, gap) in lost {
    let n = same_ink(&address, gap, &held, &printed);
    *held.addresses.entry(address.clone()).or_default() += n;
    printed.add_local(&address, n);
  }
  let more: Vec<(String, usize)> = shared
    .addresses
    .iter()
    .map(|(address, &n)| {
      let gap = n.saturating_sub(printed.address(address) + beside.address(address));
      (address.clone(), gap)
    })
    .collect();
  for (address, gap) in more {
    let n = same_ink(&address, gap, &printed, &held);
    *printed.addresses.entry(address.clone()).or_default() += n;
    held.add_local(&address, n);
  }
  // (a word one side reads glued to the next, the other parted — a line's words the lexer reads across a break, `CanadaPerimeter`
  // (0803.3421), a name and its ORCID's id (`Kim0000`, 2307.03558); a block written whole parted at its layout's cells —: the
  // same ink)
  let gaps: Vec<(String, usize)> = printed
    .words
    .iter()
    .map(|(word, (n, _))| (word.clone(), n.saturating_sub(held.word(word))))
    .collect();
  for (word, gap) in gaps {
    if let Some((n, parts)) = glued(&word, gap, &held, &printed) {
      held.add_word(&word, n);
      parts.iter().for_each(|part| printed.add_word(part, n));
    }
  }
  let gaps: Vec<(String, usize)> = shared
    .words
    .iter()
    .map(|(word, (n, _))| {
      let gap = n.saturating_sub(printed.word(word) + beside.word(word));
      (word.clone(), gap)
    })
    .collect();
  for (word, gap) in gaps {
    if let Some((n, parts)) = glued(&word, gap, &printed, &held) {
      printed.add_word(&word, n);
      parts.iter().for_each(|part| held.add_word(part, n));
    }
  }
  let mut check = ContentCheck::default();
  for (word, (n, spelled)) in &printed.words {
    if *n > held.word(word) {
      check.lost.push(spelled.clone());
    }
  }
  for (address, n) in &printed.addresses {
    if *n > held.address(address) {
      check.lost.push(address.clone());
    }
  }
  if all_read && !whole {
    for (word, (n, spelled)) in &shared.words {
      if *n > printed.word(word) + beside.word(word) {
        check.excess.push(spelled.clone());
      }
    }
  }
  if all_read {
    for (address, n) in &shared.addresses {
      if *n > printed.address(address) + beside.address(address) {
        check.excess.push(address.clone());
      }
    }
  }
  check
}

/// How many of `gap` copies of `address` one side lacks are the same ink as its local part's words: as many as `words`
/// holds each of those words more often than `other`, at most `gap`.
fn same_ink(address: &str, gap: usize, words: &Tally, other: &Tally) -> usize {
  let local = local_words(address);
  let spare = local
    .iter()
    .map(|word| words.word(word).saturating_sub(other.word(word)))
    .min();
  spare.unwrap_or(0).min(gap)
}

/// The counted words of `address`'s local part, lowercased.
fn local_words(address: &str) -> Vec<String> {
  let local = address.split('@').next().unwrap_or_default();
  words_of(local).filter(|w| counted(w)).collect()
}

/// How many of `gap` copies of `word` one side lacks are two words of the other glued, and those two: the first cut of `word`
/// into two whose every counted part `words` holds more often than `other` (twice for a word glued to itself), a part a mark's
/// digit or letter (not [`counted`]) taken as printed; none where no cut has a counted part.
fn glued(word: &str, gap: usize, words: &Tally, other: &Tally) -> Option<(usize, [String; 2])> {
  if gap == 0 {
    return None;
  }
  let spare = |part: &str| words.word(part).saturating_sub(other.word(part));
  for (at, _) in word.char_indices().skip(1) {
    let (a, b) = word.split_at(at);
    if !counted(a) && !counted(b) {
      continue;
    }
    let n = match (counted(a), counted(b)) {
      (true, true) if a == b => spare(a) / 2,
      (true, true) => spare(a).min(spare(b)),
      (true, false) => spare(a),
      _ => spare(b),
    };
    if n > 0 {
      return Some((n.min(gap), [a, b].map(|part| part.to_string())));
    }
  }
  None
}
