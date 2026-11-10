//! The content invariant, checked on the placed output (FRONTMATTER_SINGLE_ENGINE_PLAN.md, the safety net's bar (2): every
//! word TeX printed in an author block is in the output exactly once, or counted in a Warning; OXIDIZED_DESIGN_DIVERGENCES
//! #980). The reading's word check (`words`) sees what the reading writes before the emit; what goes wrong after it — a
//! shared entry's address the Kind step keeps for every holder, digestion's junk in a name (2506.02663), a call written
//! whole beside its piece (2607.00684's `mtablo@postech.ac.kr` twice), a binding's lost entry text — shows only in the
//! frontmatter `insert_frontmatter` placed. So one pass after it (Perl's `insertFrontMatter` and `relocateAnnotations`,
//! Base_Utility.pool.ltxml:736-740, :891-900): the words the ledger's author blocks and
//! creator calls printed (their rehearsed events), against the words the placed creators hold (`emitter::dom`'s reading of
//! the document). It counts, it never answers: placement is over.
//!
//! What counts, both sides by one definition ([`ink_words`]): every word, a mark excluded by its event, not by its
//! spelling (an inked letter, an initial or a two-digit number counts; a word a Mark event prints is that mark's, as
//! often as the events print it, wherever a text holds it, neither lost nor in excess); an address is a unit of its
//! own, its words none of the text's (1605.03019's `IDSIA` beside `@idsia.ch`). *Lost*: a word or an address the inks
//! print more often than the placed frontmatter holds it — a shared item (one printed `\affiliation`, `annotate=all`)
//! as often as the inks print its text, at least the most copies one creator holds and at most its copies in all, as
//! the excess counts it (a line printed under each author and kept as one shared item is no loss), a name every copy,
//! so a shared entry's copies hide no word lost on a line of other words (review revN2: `IDSIA` printed twice, one copy
//! lost, the shared affiliation printed once and held twice; a lost line printing the shared item's own words is still
//! covered, review revN2r3's `q1e`, SYNC leftovers 13f) —, a publication note, a contact's label (its `name`, with its item,
//! never its text's where the text opens with it: a class's `Email: `, which the rehearsal's `\email` doesn't print,
//! 2307.11138) and a link's target holding it too. *Excess*: a word placed more often than printed — a shared
//! item counted the same way; an e-mail contact every time, an address being one person's (one address on two creators,
//! or twice on one). What may print beside the authors without being theirs only answers an excess: a separator's word
//! (`and`), a mark's text, a link's target (an ORCID's id behind a logo), a footnote's text (`\footnotetext`), a publication
//! note's, a call a block's own expansion made whose words its block's events print (a call's words, else, are required:
//! its rehearsal's, or its content's text where none read it — [`content_check`]), and the words of the creator a block
//! written whole became (TeX's own digestion of it, its addresses still counted). An excess needs every live block and
//! creator call read: a block no rehearsal read prints words no event holds.

use std::collections::{BTreeMap, BTreeSet};

use super::{
  super::ledger::{ApiCall, ContentText, Entry, Ledger, Outcome, SkipReason},
  api::note_owned_contacts,
  words::{ink_words, live_blocks, words_of},
  *,
};
use crate::base_utilities::defs::frontmatter_api::CUT_LIST;

/// One item of the placed frontmatter (`emitter::dom`'s reading of a creator): its creator, what it is, its printed text.
pub(in crate::base_utilities::frontmatter::observer) struct PlacedItem {
  /// the creator holding it, by its place among the document's creators
  pub holder: usize,
  pub kind:   PlacedKind,
  pub text:   String,
  /// a contact's label (its `name`), which holds a printed word with it
  pub label:  Option<String>,
}

/// What a [`PlacedItem`] is, for how often it counts: a name (every copy), an e-mail contact or any other item, by its
/// kind and compared text (`dom::norm`): one each holder shares, an e-mail's addresses every copy (an address is one
/// person's).
pub(in crate::base_utilities::frontmatter::observer) enum PlacedKind {
  Name,
  Email(String),
  Item(String),
}

/// The placed frontmatter as the check reads it: the creators' items, and the texts that hold a printed word apart from
/// them (a publication note, a link's target, each target once); and the words the reading drops by its own rule
/// (`Resolved::drops`, a block's Name event by its ledger index, span and word), each a separator's.
#[derive(Default)]
pub(in crate::base_utilities::frontmatter::observer) struct Placed {
  pub items:   Vec<PlacedItem>,
  pub beside:  Vec<String>,
  pub dropped: BTreeSet<RuleDrop>,
  /// the document's publication notes' texts, each as written (in `beside` too): a note written more often than it
  /// printed is in excess ([`note_excess`])
  pub notes:   Vec<String>,
}

/// The check's findings: the words (as first printed) and addresses lost, and those placed in excess.
#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::base_utilities::frontmatter::observer) struct ContentCheck {
  pub lost:   Vec<String>,
  pub excess: Vec<String>,
}

/// A multiset of the words and addresses texts print, each word with its first spelling.
#[derive(Default, Clone)]
struct Tally {
  words:     BTreeMap<String, (usize, String)>,
  addresses: BTreeMap<String, usize>,
}

impl Tally {
  fn add(&mut self, text: &str, times: usize) { self.add_parts(text, times, times); }

  /// `text`'s words `words` times, its addresses `addresses` times.
  fn add_parts(&mut self, text: &str, words: usize, addresses: usize) {
    let (printed, printed_addresses) = ink_words(text);
    for word in printed {
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
/// word, a mark's text (into `marks` too, its words), a contact's label (`Event::name`), a link's target where its anchor prints no address of its own
/// (`Written::excess`'s). And the same texts with their separators' words, as an item's text holds them (`Japan Science and
/// Technology Corporation`, cond-mat0305063), each a print a shared item counts by ([`Prints`]).
fn ink_texts(
  events: &[Event],
  beside: &mut Tally,
  marks: &mut BTreeMap<String, usize>,
) -> (Vec<String>, Vec<String>) {
  let formulas: Vec<(u16, u16)> = events
    .iter()
    .filter(|m| m.kind == EventKind::Mark)
    .filter_map(|m| m.formula.and_then(|f| f.span))
    .collect();
  let in_a_mark = |e: &Event| {
    e.span.is_some_and(|(s, t)| {
      formulas
        .iter()
        .any(|&(from, to)| from <= s && t <= to && (s, t) != (from, to))
    })
  };
  let (mut texts, mut prints) = (Vec::new(), Vec::new());
  // (a line's Names, and with its separators' words)
  let (mut line, mut printed): (Vec<&Event>, Vec<&Event>) = (Vec::new(), Vec::new());
  fn end_line<'a>(
    (texts, prints): (&mut Vec<String>, &mut Vec<String>),
    line: &mut Vec<&'a Event>,
    printed: &mut Vec<&'a Event>,
  ) {
    texts.push(events_text(line));
    prints.push(events_text(printed));
    line.clear();
    printed.clear();
  }
  for e in events {
    match &e.kind {
      EventKind::Name | EventKind::Sep(SepKind::Comma) => {
        line.push(e);
        printed.push(e);
      },
      EventKind::Contact if in_a_mark(e) => beside.add(&e.text, 1),
      EventKind::Contact | EventKind::Note if e.kind == EventKind::Contact || !e.inkless => {
        texts.push(e.text.clone());
        prints.push(e.text.clone());
      },
      EventKind::Sep(SepKind::Word(word)) => {
        beside.add(word, 1);
        printed.push(e);
      },
      // (a line break inside a printed group of local parts parts no list: `\{yogarshi, …,\\ vittorca, drot, ballemig\}@amazon.com`,
      // 2305.13191, `Event::braces`; and a column ends one, `\and`'s or an alignment tab's: the next author's lines — `Ann
      // Able\\ IDSIA Lab, Lugano \and Bob Baker\\ IDSIA Lab, Lugano` prints that line twice, each its own, `Prints`)
      EventKind::Break if e.braces == 0 => {
        end_line((&mut texts, &mut prints), &mut line, &mut printed)
      },
      EventKind::Sep(SepKind::And(_) | SepKind::Column(_)) => {
        end_line((&mut texts, &mut prints), &mut line, &mut printed)
      },
      _ => {},
    }
    for inner in std::iter::once(e).chain(&e.inner) {
      // (a contact's label, printed with it, beside its address as a call's is (`call_label`): authblk's `Corresponding
      // author e-mail:` before the address of an affiliation's `\thanks`, cond-mat0202268)
      if inner.kind == EventKind::Contact
        && let Some(name) = inner.name.as_deref()
      {
        beside.add(name, 1);
      }
      if inner.kind == EventKind::Contact
        && let Some(target) = inner.target.as_deref()
        && addresses_in(&inner.text).is_empty()
      {
        beside.add(target, 1);
      }
      // (a mark by its event, whatever its spelling: an ordinal's suffix is the line's text, `events_text`)
      if inner.kind == EventKind::Mark && !inner.ordinal {
        beside.add(&inner.text, 1);
        for word in words_of(&inner.text) {
          *marks.entry(word).or_default() += 1;
        }
      }
    }
  }
  end_line((&mut texts, &mut prints), &mut line, &mut printed);
  (texts, prints)
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
  let mut marks: BTreeMap<String, usize> = BTreeMap::new();
  let mut all_read = true;
  // (the words a content's text read past an accent or a dotless letter in, with the places: `ContentText::read_past`)
  let mut past: BTreeMap<String, Vec<(usize, bool)>> = BTreeMap::new();
  // (the Name words of each block written whole: TeX's own digestion of it, printing what its separators print — a class's
  // `\and` printing "and", which the rehearsal binds to an event printing nothing: JAIR's, 1908.02923 —, words no event
  // holds; the creator it became is found by them below)
  let mut whole: Vec<BTreeMap<String, usize>> = Vec::new();
  // (the texts the inks print, each block's and call's in order, which a shared item's copies count by: `Prints`)
  let mut prints = Prints::default();
  // (each live block's printed words, which its own expansion's calls take theirs from: `own_print`)
  let mut block_print: BTreeMap<usize, Tally> = BTreeMap::new();
  let live = live_blocks(ledger);
  let live_index: BTreeSet<usize> = live.iter().map(|(index, _)| *index).collect();
  for (index, block) in live {
    let print = block_print.entry(index).or_default();
    match &block.outcome {
      Outcome::Events { events, unread, .. } => {
        // (a Name the reading drops by its own rule is the separator it reads it as, its word beside the names: a byline's
        // "by", `By~~{\scshape Bryna Kra}`, 2603.27258; 2603.25922, 2604.07186, 2604.12864)
        let separated: Vec<Event>;
        // (an earlier list setter call of the block the written one replaces prints nothing: `Ledger::replaced`)
        let unreplaced: Vec<Event>;
        let events = if ledger
          .replaced
          .iter()
          .any(|(at, run)| *at == index && run.is_some())
        {
          let within = |span: (u16, u16)| {
            ledger
              .replaced
              .iter()
              .any(|&(at, run)| at == index && run.is_some_and(|(s, t)| s <= span.0 && span.1 <= t))
          };
          unreplaced = events
            .iter()
            .filter(|e| !e.span.is_some_and(within))
            .cloned()
            .collect();
          &unreplaced
        } else {
          events
        };
        let events = if placed.dropped.iter().any(|(at, ..)| *at == index) {
          separated = events
            .iter()
            .map(|e| match e.span {
              Some(_)
                if e.kind == EventKind::Name
                  && placed
                    .dropped
                    .contains(&(index, e.span, e.text.trim().to_lowercase())) =>
              {
                let mut sep = Event::new(EventKind::Sep(SepKind::Word(e.text.trim().to_string())));
                sep.span = e.span;
                sep
              },
              _ => e.clone(),
            })
            .collect();
          &separated
        } else {
          events
        };
        // (a line of link pieces the reading joins into its addresses prints them, not the pieces' words:
        // `joined_address_pieces`, 1710.01968)
        let joined = joined_address_pieces(events);
        let (texts, units) = ink_texts(&joined, &mut beside, &mut marks);
        for text in texts {
          printed.add(&text, 1);
          print.add(&text, 1);
        }
        // (a rehearsal that left a group open — an `\end{tabular}\begin{tabular}{c}` juggle in the line, 1401.6981 — read
        // what follows into no box, so no event says it: the events are not trusted for the block, its content's text
        // (`Outcome::Events::unread`, `ledger::content_text`, as for a call no rehearsal read) counting the words they
        // lack, a possible loss)
        if let Some(text) = unread {
          read_past(&mut past, text);
          let mut content = Tally::default();
          content.add(text, 1);
          // (a content word the block prints as spelled; then a word it read past an accent or a dotless letter in, as the
          // word the block prints with them — `muller` for `müller`, `fsica` for `física` ([`reads_as`]) —, each print taken
          // once: a word lost beside a printed one it differs from by a letter is lost, and named, `Lan` beside `Lian`)
          let mut taken: BTreeMap<String, usize> = BTreeMap::new();
          let mut held: BTreeMap<String, usize> = BTreeMap::new();
          for (word, (n, _)) in &content.words {
            let take = (*n).min(print.word(word));
            taken.insert(word.clone(), take);
            held.insert(word.clone(), take);
          }
          for (word, (n, _)) in &content.words {
            let Some(places) = past.get(word) else {
              continue;
            };
            for (other, (m, _)) in &print.words {
              let have = held.get(word).copied().unwrap_or(0);
              if have == *n {
                break;
              }
              if other == word || !reads_as(word, places, other) {
                continue;
              }
              let take = m
                .saturating_sub(taken.get(other).copied().unwrap_or(0))
                .min(n - have);
              *taken.entry(other.clone()).or_default() += take;
              *held.entry(word.clone()).or_default() += take;
            }
          }
          // (then a word the block prints glued that is two content words joined — `ledger::content_text` parts a word at
          // every multi-letter control sequence, `Able\begingroup\bfseries Bob` read `able bob`, printed `ablebob` —: both
          // parts held, the document-level rule ([`glued`]) applied before a word is counted lost, as `CanadaPerimeter`'s)
          let mut have = Tally::default();
          held.iter().for_each(|(word, n)| have.add_word(word, *n));
          for (other, (m, _)) in &print.words {
            let gap = m.saturating_sub(taken.get(other).copied().unwrap_or(0));
            if let Some((n, parts)) = glued(other, gap, &content, &have, &marks) {
              *taken.entry(other.clone()).or_default() += n;
              for part in parts {
                have.add_word(&part, n);
                *held.entry(part).or_default() += n;
              }
            }
          }
          for (word, (n, spelled)) in content.words {
            let more = n.saturating_sub(held.get(&word).copied().unwrap_or(0));
            for tally in [&mut printed, &mut *print] {
              tally
                .words
                .entry(word.clone())
                .or_insert((0, spelled.clone()))
                .0 += more;
            }
          }
          for (address, n) in content.addresses {
            let more = n.saturating_sub(print.address(&address));
            for tally in [&mut printed, &mut *print] {
              *tally.addresses.entry(address.clone()).or_default() += more;
            }
          }
        }
        units.iter().for_each(|text| prints.add(index, text));
        if kept_whole(index) {
          let mut words: BTreeMap<String, usize> = BTreeMap::new();
          let names = events.iter().filter(|e| e.kind == EventKind::Name);
          for word in names.flat_map(|e| words_of(&e.text)) {
            *words.entry(word).or_default() += 1;
          }
          whole.push(words);
        }
      },
      Outcome::Skipped(SkipReason::Empty) => {},
      _ => all_read = false,
    }
  }
  let mut calls: BTreeSet<Vec<String>> = BTreeSet::new();
  // (the pieces of a cut list so far, and its key)
  let mut list: (Option<&str>, Vec<String>) = (None, Vec::new());
  let aside = |events: &[Event], beside: &mut Tally, marks: &mut BTreeMap<String, usize>| {
    for text in ink_texts(events, beside, marks).0 {
      beside.add(&text, 1);
    }
  };
  // (what TeX printed of a creator call, one rule for them all: its words count unless its block's events print them — a call
  // outside its block's expansion none of them. Its words are its rehearsal's, or its content's text where the rehearsal gave
  // up on it (`ApiCall::unread`; a queued call no call recorded, `Ledger::unrecorded`: texts the ledger reads, the check
  // reads no token); an inside call's, rehearsed for this check alone (`ApiCall::inside_events`), are its block's where
  // that block's events print every one of them, a block no longer live printing none that TeX printed. The block's own
  // creator entry is the block, and so is an inside call no flush could rehearse (a journal unavailable to the end). An inside call the ledger
  // misread — an input mark refilled to an equal token, IEEEtran's affiliations read as the author block's own, 2401.11731,
  // 1801.03314 — is no longer exempt on trust)
  let own_print = |call: &ApiCall, block_print: &mut BTreeMap<usize, Tally>| -> bool {
    let Some(block) = call.after_block.filter(|_| call.inside) else {
      return false;
    };
    if !live_index.contains(&block) {
      return true;
    }
    let mut words = Tally::default();
    if let Some(text) = &call.unread {
      words.add(text, 1);
    } else {
      let (mut beside, mut marks) = (Tally::default(), BTreeMap::new());
      for text in ink_texts(&call.inside_events, &mut beside, &mut marks).0 {
        words.add(&text, 1);
      }
    }
    let print = block_print.entry(block).or_default();
    let held = words
      .words
      .iter()
      .all(|(word, (n, _))| print.word(word) >= *n)
      && words
        .addresses
        .iter()
        .all(|(address, n)| print.address(address) >= *n);
    if held {
      for (word, (n, _)) in &words.words {
        if let Some(entry) = print.words.get_mut(word) {
          entry.0 -= n;
        }
      }
      for (address, n) in &words.addresses {
        if let Some(entry) = print.addresses.get_mut(address) {
          *entry -= n;
        }
      }
    }
    held
  };
  for (index, entry) in ledger.entries().iter().enumerate() {
    let piece = match entry {
      Entry::Api(call)
        if !call.cleared
          && !ledger.replaced.contains(&(call.index, None))
          && creator_call(&call.tag, &call.parent) =>
      {
        if let Some(label) = call_label(&call.attrs) {
          beside.add(label, 1);
        }
        let events: &[Event] = if call.inside {
          &call.inside_events
        } else {
          &call.events
        };
        if call.inside && call.parent.is_empty() {
          // (the block's own creator entry: the block's events are its print)
          None
        } else if own_print(call, &mut block_print) {
          aside(events, &mut beside, &mut marks);
          None
        } else if let Some(text) = &call.unread {
          read_past(&mut past, text);
          calls
            .insert(words_of(text).collect())
            .then(|| (None, (vec![text.to_string()], vec![text.to_string()])))
        } else if !call.rehearsed {
          // (the writer stays out for it, `writer::writer_stays_out` (deleted): the queue is digested as Perl's)
          all_read = false;
          None
        } else if calls.insert(words_of(&events::printed_text(events)).collect()) {
          let cut = call
            .attrs
            .split(',')
            .find_map(|key| key.strip_prefix(CUT_LIST)?.strip_prefix('='));
          Some((cut, ink_texts(events, &mut beside, &mut marks)))
        } else {
          None
        }
      },
      // (a document note's text is its own, beside; the contacts the reading takes as the creators' are placed or counted —
      // a late author's `\address` in amsart's `\thanks`, 0807.4720 — `api::note_owned_contacts`)
      Entry::Api(call) if !call.cleared && call.tag == "ltx:pubnote" => {
        let owned = note_owned_contacts(&call.events);
        let rest: Vec<Event> = call
          .events
          .iter()
          .filter(|e| !owned.iter().any(|o| std::ptr::eq(*o, *e)))
          .cloned()
          .collect();
        aside(&rest, &mut beside, &mut marks);
        let texts: Vec<String> = owned
          .iter()
          .map(|e| e.text.clone())
          .filter(|text| !text.trim().is_empty())
          .collect();
        (!texts.is_empty()).then(|| (None, (texts.clone(), texts)))
      },
      Entry::NoteText(text) => {
        aside(&text.events, &mut beside, &mut marks);
        None
      },
      _ => None,
    };
    let Some((cut, (texts, units))) = piece else {
      continue;
    };
    units.iter().for_each(|text| prints.add(index, text));
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
  // (the queued creator calls the emitter took that no call recorded: their contents' texts)
  for (n, text) in ledger.unrecorded.iter().enumerate() {
    if calls.insert(words_of(text).collect()) {
      read_past(&mut past, text);
      prints.add(ledger.entries().len() + n, text);
      printed.add(text, 1);
    }
  }
  // (what the placed frontmatter holds, both ways alike: a name every copy, and an e-mail's addresses; a shared item once,
  // as often as one creator holds it — its label with it)
  let (mut names, mut emails): (BTreeMap<&str, usize>, BTreeMap<&str, usize>) = Default::default();
  let mut per_holder: BTreeMap<(&str, usize), (usize, &PlacedItem)> = BTreeMap::new();
  // (each creator's name words: which one a block written whole became)
  let mut named: BTreeMap<usize, BTreeMap<String, usize>> = BTreeMap::new();
  for item in &placed.items {
    let key = match &item.kind {
      PlacedKind::Name => {
        *names.entry(&item.text).or_default() += 1;
        let words = named.entry(item.holder).or_default();
        for word in words_of(&item.text) {
          *words.entry(word).or_default() += 1;
        }
        continue;
      },
      PlacedKind::Email(key) => {
        *emails.entry(&item.text).or_default() += 1;
        key
      },
      PlacedKind::Item(key) => key,
    };
    per_holder
      .entry((key.as_str(), item.holder))
      .or_insert((0, item))
      .0 += 1;
  }
  let mut shared = Tally::default();
  for (text, n) in names {
    shared.add(text, n);
  }
  for (text, n) in emails {
    shared.add_parts(text, 0, n);
  }
  // (per shared item, the most copies one creator holds and its copies in all)
  let mut copies: BTreeMap<&str, (usize, usize, &PlacedItem)> = BTreeMap::new();
  for ((key, _), (n, item)) in per_holder {
    let at = copies.entry(key).or_insert((0, 0, item));
    at.0 = at.0.max(n);
    at.1 += n;
  }
  let mut held = Tally::default();
  // (the longest texts first: an entry printing two lines takes them before an entry printing one of them — revtex's
  // `\address{Dept, Univ\\ CREST, …}` and another author's `\address{Dept, Univ}`, cond-mat0305063)
  let mut copies: Vec<(usize, usize, &PlacedItem)> = copies.into_values().collect();
  copies.sort_by_cached_key(|&(_, _, item)| std::cmp::Reverse(Prints::key(labelled(item).1).len()));
  for (most, total, item) in copies {
    let email = matches!(item.kind, PlacedKind::Email(_));
    let (label, text) = labelled(item);
    let n = prints.take(text, most, total);
    shared.add_parts(text, n, if email { 0 } else { n });
    // (a contact's label, its `name`: the label's words, held with its item, never its text's where the text opens with it — a
    // class printing `Email: ` the rehearsal's `\email` doesn't, 2307.11138, 1602.07570; a label the text doesn't print answers
    // no word of it, `Email: ` beside an unprinted `email@domain`, 2607.28418)
    if let Some(label) = label {
      held.add(label, n);
    }
  }
  for text in &placed.beside {
    held.add(text, 1);
  }
  // (`held` is `shared` and the texts beside it)
  for (word, (n, spelled)) in &shared.words {
    let entry = held
      .words
      .entry(word.clone())
      .or_insert((0, spelled.clone()));
    entry.0 += n;
  }
  for (address, n) in &shared.addresses {
    *held.addresses.entry(address.clone()).or_default() += n;
  }
  // (a block written whole: the words of the creator it became — the one whose name holds the most of its Name words,
  // more than half — answer an excess, that creator's addresses still counted)
  for words in &whole {
    let total: usize = words.values().sum();
    let overlap = |name: &BTreeMap<String, usize>| -> usize {
      words
        .iter()
        .map(|(word, n)| (*n).min(name.get(word).copied().unwrap_or(0)))
        .sum()
    };
    let best = named
      .iter()
      .map(|(holder, name)| (overlap(name), *holder))
      .max_by_key(|&(n, holder)| (n, std::cmp::Reverse(holder)));
    if let Some((n, holder)) = best
      && 2 * n > total
    {
      for item in placed.items.iter().filter(|item| item.holder == holder) {
        beside.add_parts(&item.text, 1, 0);
      }
    }
  }
  // (an address one side reads where the other reads its local part's words — a list's parts a note keeps, `{\it aable,
  // bbaker, cat@a.org}`, 2003.14069 —: the same ink, so neither lost nor in excess; a lost one's from the words both sides
  // read, the placed creators' (`shared`), never a label's, a link's or a note's beside them)
  let lost: Vec<(String, usize)> = printed
    .addresses
    .iter()
    .map(|(address, &n)| (address.clone(), n.saturating_sub(held.address(address))))
    .collect();
  for (address, gap) in lost {
    let n = same_ink(&address, gap, &shared, &printed);
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
  // same ink; a lost one's parts from the words both sides read, as an address's)
  let gaps: Vec<(String, usize)> = printed
    .words
    .iter()
    .map(|(word, (n, _))| (word.clone(), n.saturating_sub(held.word(word))))
    .collect();
  for (word, gap) in gaps {
    if let Some((n, parts)) = glued(&word, gap, &shared, &printed, &marks) {
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
    if let Some((n, parts)) = glued(&word, gap, &printed, &held, &marks) {
      printed.add_word(&word, n);
      parts.iter().for_each(|part| held.add_word(part, n));
    }
  }
  // (a word one side reads without the accents or the dotless letters the other prints — the text of a call no rehearsal
  // read, which reads none (`ledger::content_text`), against its placed copy, `universitat` and `universität`, `fsica` and
  // `física` ([`reads_as`]) —: the same ink, as a glued word)
  let gaps: Vec<(String, usize)> = printed
    .words
    .iter()
    .map(|(word, (n, _))| (word.clone(), n.saturating_sub(held.word(word))))
    .filter(|(word, gap)| *gap > 0 && past.contains_key(word))
    .collect();
  for (word, mut gap) in gaps {
    let places = &past[&word];
    let accented: Vec<String> = shared
      .words
      .keys()
      .filter(|other| **other != word && reads_as(&word, places, other))
      .cloned()
      .collect();
    for other in accented {
      let spare = shared
        .word(&other)
        .saturating_sub(printed.word(&other) + beside.word(&other));
      let n = spare.min(gap);
      if n > 0 {
        held.add_word(&word, n);
        printed.add_word(&other, n);
        gap -= n;
      }
    }
  }
  // (a word a Mark event prints is that mark's, as often as the events print it, wherever a text holds it: an entry's
  // label in its contact's text, IEEEtran's nested `\IEEEauthorblockA{\IEEEauthorrefmark{2}Univ B, City}`, 1812.05961)
  let mut check = ContentCheck::default();
  for (word, (n, spelled)) in &printed.words {
    let mark = marks.get(word).copied().unwrap_or(0);
    if *n > held.word(word) + mark {
      check.lost.push(spelled.clone());
    }
  }
  for (address, n) in &printed.addresses {
    if *n > held.address(address) {
      check.lost.push(address.clone());
    }
  }
  if all_read {
    for (word, (n, spelled)) in &shared.words {
      if *n > printed.word(word) + beside.word(word) {
        check.excess.push(spelled.clone());
      }
    }
    for (address, n) in &shared.addresses {
      if *n > printed.address(address) + beside.address(address) {
        check.excess.push(address.clone());
      }
    }
  }
  check
    .excess
    .extend(note_excess(&placed.notes, &note_prints(ledger)));
  check
}

/// The texts the notes of the ledger's live blocks and creator calls print, and its document notes' (`\footnotetext`): a
/// Note event each, as the reading reads it — the one definition of a note printed once.
fn note_prints(ledger: &Ledger) -> Vec<String> {
  fn notes<'a>(events: &'a [Event], into: &mut Vec<&'a Event>) {
    for e in events {
      if e.kind == EventKind::Note {
        into.push(e);
      }
      notes(&e.inner, into);
    }
  }
  let mut found = Vec::new();
  for (_, block) in live_blocks(ledger) {
    if let Outcome::Events { events, .. } = &block.outcome {
      notes(events, &mut found);
    }
  }
  let mut texts: Vec<String> = found.iter().map(|e| e.text.clone()).collect();
  for entry in ledger.entries() {
    match entry {
      Entry::Api(call) if !call.cleared => {
        let mut found = Vec::new();
        notes(&call.events, &mut found);
        texts.extend(found.iter().map(|e| e.text.clone()));
        if call.tag == "ltx:pubnote" {
          texts.push(events::printed_text(&call.events));
        }
      },
      Entry::NoteText(text) => texts.push(events::printed_text(&text.events)),
      _ => {},
    }
  }
  texts
}

/// The publication notes `written` holds more often than the notes `printed` print them (each at least once: one whose
/// print no event read is its own), each extra copy's text: a `\thanks` digested twice (review rck2). Compared by their
/// letters' words, in any order (a mark's digits and symbols none of them: `$^{1}$Univ A` prints `1Univ A`, written
/// `<sup>1</sup>Univ A`, the note's text without its marks); a note of no letter is counted by no one.
fn note_excess(written: &[String], printed: &[String]) -> Vec<String> {
  fn key(text: &str) -> Vec<String> {
    let mut words: Vec<String> = words_of(text)
      .map(|word| {
        word
          .chars()
          .filter(|c| c.is_alphabetic())
          .collect::<String>()
      })
      .filter(|word| !word.is_empty())
      .collect();
    words.sort();
    words
  }
  let mut copies: BTreeMap<Vec<String>, (usize, &str)> = BTreeMap::new();
  for text in written {
    copies.entry(key(text)).or_insert((0, text)).0 += 1;
  }
  let mut excess = Vec::new();
  for (key_of, (n, text)) in copies {
    if key_of.is_empty() {
      continue;
    }
    let prints = printed.iter().filter(|p| key(p) == key_of).count();
    for _ in prints.max(1)..n {
      excess.push(text.trim().to_string());
    }
  }
  excess
}

/// The words `text` read past an accent or a dotless letter in, into `past` ([`ContentText::read_past`]).
fn read_past(past: &mut BTreeMap<String, Vec<(usize, bool)>>, text: &ContentText) {
  for (word, places) in &text.read_past {
    past.entry(word.clone()).or_insert_with(|| places.clone());
  }
}

/// Whether `word`, a content text's read past an accent or a one-letter control word at `places`
/// ([`ContentText::read_past`]: `F\'{\i}sica` read `fsica`, a dotless letter after its `f`), reads the word `other` an
/// ink prints (`física`): `other`'s base letters ([`base_letters`]) are `word`'s, with at most one `i` or `j` where a
/// one-letter control word was, the letter a dotless `\i`/`\j` printed — no other letter, and none elsewhere (`lu` is
/// no `liu`). The one test both content-text reconciles share.
fn reads_as(word: &str, places: &[(usize, bool)], other: &str) -> bool {
  fn fits(word: &[char], other: &[char], places: &[usize]) -> bool {
    let Some((&at, rest)) = places.split_first() else {
      return word == other;
    };
    if word.len() < at || other.len() < at || word[..at] != other[..at] {
      return false;
    }
    let (word, other) = (&word[at..], &other[at..]);
    let rest: Vec<usize> = rest.iter().map(|p| p - at).collect();
    fits(word, other, &rest)
      || other.first().is_some_and(|c| matches!(c, 'i' | 'j' | 'ȷ'))
        && fits(word, &other[1..], &rest)
  }
  let word: Vec<char> = base_letters(word).chars().collect();
  let other: Vec<char> = base_letters(other).chars().collect();
  let dotless: Vec<usize> = places
    .iter()
    .filter(|(_, letter)| *letter)
    .map(|(at, _)| *at)
    .collect();
  fits(&word, &other, &dotless)
}

/// The texts the inks print, each block's and call's (by its ledger index) in order, by their words ([`ink_words`], lowercased,
/// in order; an address a line prints beside them is an e-mail's, which the reading takes out of the entry: IEEEtran's
/// `\IEEEauthorblockA{…\\ Burwood 3125, Australia \\ ann@deakin.edu.au}`, 2311.03657): what a shared item's copies count
/// up to, each print taken once.
#[derive(Default)]
struct Prints {
  prints: Vec<(usize, Vec<String>, bool)>,
  /// each print's position by its first word, in order: where a run of prints spelling a text can start
  starts: BTreeMap<String, Vec<usize>>,
}

impl Prints {
  /// `text`'s words, lowercased.
  fn key(text: &str) -> Vec<String> {
    let (words, _) = ink_words(text);
    words.iter().map(|w| w.to_lowercase()).collect()
  }

  fn add(&mut self, source: usize, text: &str) {
    let key = Self::key(text);
    if let Some(first) = key.first() {
      self
        .starts
        .entry(first.clone())
        .or_default()
        .push(self.prints.len());
      self.prints.push((source, key, false));
    }
  }

  /// How often a shared item printing `text` counts: as often as the inks print its text — one print, or a run of one block's
  /// or call's prints in a row (an entry of two lines), none taken already —, at least `most` (the most copies one creator holds)
  /// and at most `total` (its copies in all): a line printed under each author and kept as one shared item is no loss, and a
  /// line printed once never counts more than one creator's copies (review revN2, the main session's ruling on N2r2). Each print
  /// it counts is taken: an item of another kind, or a shorter entry inside it, counts what is left.
  fn take(&mut self, text: &str, most: usize, total: usize) -> usize {
    let key = Self::key(text);
    let mut found = Vec::new();
    let mut at = 0;
    // (a run spelling `key` starts at a print its first word starts — a print's words are a prefix of what is left to spell —,
    // so only those are tried, in order, not every print for every item: 2103.04797's ~1,000 creators)
    let starts = key
      .first()
      .and_then(|first| self.starts.get(first))
      .map_or(&[][..], Vec::as_slice);
    for &start in starts {
      if found.len() >= total {
        break;
      }
      if start < at {
        continue;
      }
      // (how much of `key` the prints from `start` spell, in a row)
      let (mut spelled, mut end) = (0, start);
      while end < self.prints.len()
        && spelled < key.len()
        && !self.prints[end].2
        && self.prints[end].0 == self.prints[start].0
        && key[spelled..].starts_with(&self.prints[end].1)
      {
        spelled += self.prints[end].1.len();
        end += 1;
      }
      if spelled == key.len() {
        found.push(start..end);
        at = end;
      }
    }
    let n = found.len().clamp(most, total);
    for range in found.into_iter().take(n) {
      self.prints[range]
        .iter_mut()
        .for_each(|print| print.2 = true);
    }
    n
  }
}

/// A placed item's label (a contact's `name`) and its text without it, where the text opens with it (`Email: ann@a.org`).
fn labelled(item: &PlacedItem) -> (Option<&str>, &str) {
  let label = item
    .label
    .as_deref()
    .filter(|label| !label.trim().is_empty());
  let text = label
    .and_then(|label| item.text.trim_start().strip_prefix(label.trim()))
    .unwrap_or(&item.text);
  (label, text)
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

/// The words of `address`'s local part, lowercased.
fn local_words(address: &str) -> Vec<String> {
  let local = address.split('@').next().unwrap_or_default();
  words_of(local).collect()
}

/// How many of `gap` copies of `word` one side lacks are two words of the other glued, and those two: the first cut of `word`
/// into two whose every part not a mark's `words` holds more often than `other` (twice for a word glued to itself), a part
/// a mark printed (`marks`, a Mark event's words: `IDSIA1`) taken as printed; none where both parts are marks'.
fn glued(
  word: &str,
  gap: usize,
  words: &Tally,
  other: &Tally,
  marks: &BTreeMap<String, usize>,
) -> Option<(usize, [String; 2])> {
  if gap == 0 {
    return None;
  }
  let spare = |part: &str| words.word(part).saturating_sub(other.word(part));
  for (at, _) in word.char_indices().skip(1) {
    let (a, b) = word.split_at(at);
    let n = match (marks.contains_key(a), marks.contains_key(b)) {
      (true, true) => continue,
      (false, false) if a == b => spare(a) / 2,
      (false, false) => spare(a).min(spare(b)),
      (false, true) => spare(a),
      (true, false) => spare(b),
    };
    if n > 0 {
      return Some((n.min(gap), [a, b].map(|part| part.to_string())));
    }
  }
  None
}

#[cfg(test)]
mod note_tests {
  use super::note_excess;

  /// A note written twice that printed once is one copy in excess (`Bob Baker$^{2\thanks{$^{1}$Univ A; $^{2}$Univ B}}$`'s
  /// note digested twice, review rck2), its marks none of its words; two notes printed twice are no excess, and a note no
  /// event read is its own print, once.
  #[test]
  fn a_note_written_more_often_than_it_printed_is_in_excess() {
    let written = |texts: &[&str]| texts.iter().map(|t| t.to_string()).collect::<Vec<_>>();
    assert_eq!(
      note_excess(
        &written(&["1Univ A; 2Univ B", "1Univ A; 2Univ B"]),
        &written(&["1Univ A; 2Univ B"])
      ),
      ["1Univ A; 2Univ B"]
    );
    assert_eq!(
      note_excess(
        &written(&["Equal contribution.", "*Equal contribution."]),
        &written(&["Equal contribution.", "Equal contribution."])
      ),
      Vec::<String>::new()
    );
    assert_eq!(
      note_excess(
        &written(&["Funded by X.", "Funded by X.", "Funded by X."]),
        &[]
      ),
      ["Funded by X.", "Funded by X."]
    );
    assert_eq!(
      note_excess(&written(&["Funded by X."]), &[]),
      Vec::<String>::new()
    );
    assert_eq!(
      note_excess(&written(&["1", "1"]), &[]),
      Vec::<String>::new()
    );
  }
}

#[cfg(test)]
mod whole_tests {
  use latexml_core::{gullet::input_mark, tokens::Tokens};

  use super::{
    super::super::{
      events::{Event, EventKind, InkTable},
      ledger::{AuthorBlock, Entry, Ledger, Outcome, Stamp},
    },
    ContentCheck, Placed, PlacedItem, PlacedKind, content_check,
  };

  /// A rehearsed author block whose events are `names`, each a Name on a line of its own.
  fn block(index: usize, names: &[&str]) -> Entry {
    let mut events = Vec::new();
    for name in names {
      let mut event = Event::new(EventKind::Name);
      event.text = name.to_string();
      events.push(event);
      events.push(Event::new(EventKind::Break));
    }
    Entry::Author(Box::new(AuthorBlock {
      index,
      hook: "\\lx@add@authors@adaptive",
      policy: "kernel".to_string(),
      binding: "kernel".to_string(),
      owner: "kernel".to_string(),
      counter: None,
      replace: false,
      cleared: false,
      sectioned: false,
      late: false,
      cautious: false,
      keyvals: None,
      keyval_tokens: None,
      epoch: 0,
      queue_position: 0,
      stamp: Stamp::Projected,
      range: (0, 0),
      tokens: Tokens::new(Vec::new()),
      source: names.join(" \\\\ "),
      mark: input_mark(),
      window_open: false,
      outcome: Outcome::Events {
        events,
        formulas: Vec::new(),
        inks: Box::new(InkTable::default()),
        errors: 0,
        warnings: 0,
        declarations: Vec::new(),
        breaks: Vec::new(),
        held: Vec::new(),
        environments: Vec::new(),
        creators: Box::new([]),
        unread: None,
        counter_moves: None,
      },
      list_setter: None,
      queued: Vec::new(),
    }))
  }

  fn name(holder: usize, text: &str) -> PlacedItem {
    PlacedItem {
      holder,
      kind: PlacedKind::Name,
      text: text.to_string(),
      label: None,
    }
  }

  /// A block written whole exempts the words of the creator it became, never another creator's (review revN2 finding 2,
  /// revI8 finding 1): the whole block's creator holds an `and` its class's `\and` printed and no event holds (JAIR's,
  /// 1908.02923), which is no excess; the second creator, from a block of its own, holds its name twice where its block
  /// printed it once (until FX4, the ORCID link's text placed again as its contact: 2203.03523, 2306.13366), which is.
  /// The repro `emit_content_a_whole_block_exempts_only_itself` has no excess on this path since FX4.
  #[test]
  fn a_whole_block_exempts_only_its_own_creator() {
    let mut ledger = Ledger::default();
    ledger.push(block(0, &["Ann Able", "Bob Baker", "Univ A"]));
    ledger.push(block(1, &["Cat Cole"]));
    let placed = Placed {
      items: vec![
        name(0, "Ann Able and Bob Baker Univ A"),
        name(1, "Cat Cole"),
        name(1, "Cat Cole"),
      ],
      ..Placed::default()
    };
    assert_eq!(
      content_check(&ledger, &placed, &|index| index == 0),
      ContentCheck {
        lost:   vec![],
        excess: vec!["Cat".to_string(), "Cole".to_string()],
      }
    );
    // (the same words with the whole block read like any other: its `and` in excess too)
    assert_eq!(content_check(&ledger, &placed, &|_| false), ContentCheck {
      lost:   vec![],
      excess: vec!["and".to_string(), "Cat".to_string(), "Cole".to_string()],
    });
  }
}
