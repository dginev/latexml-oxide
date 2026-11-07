//! The reading's word check (FIDELITY_CHARTER §4, "no printed frontmatter word is lost"; OXIDIZED_DESIGN_DIVERGENCES #980):
//! the words an author block or a creator's call prints (its Names, Contacts and Notes) that the reading writes nowhere,
//! whatever lost them (review revZ4a: 1509.01480's creator and her note, 2607.28418's address). The block half is a pass after
//! the resolver, on the ledger and [`Resolved`] alone ([`unwritten_block_inks`]); the call half, which needs what the emit
//! wrote, is the emitter's, on [`Written`] (`writer::emit`).

use std::collections::{BTreeMap, BTreeSet};

use super::{
  super::ledger::{Entry, Ledger, Outcome},
  *,
};

/// `text`'s runs of letters and digits, as printed.
pub(in crate::base_utilities::frontmatter::observer) fn printed_words(
  text: &str,
) -> impl Iterator<Item = &str> {
  text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
}

/// The words a multiset of them counts: [`printed_words`], lowercased.
fn words_of(text: &str) -> impl Iterator<Item = String> + '_ {
  printed_words(text).map(str::to_lowercase)
}

/// The e-mail label the paper prints before `address` at `span` of a block's `events`, as the reading reads an address run's
/// label on its line ([`address_runs`]' `label_word_before`): the contact's `name`, written as printed. "E-mail:" before a
/// class's `\email{…}` (PoS's, 1412.5940, 1612.08209, 1812.02252, 1907.01311, 1005.2532, 0802.3384), a list's on each of its
/// addresses ("Emails:", 2511.22090), "email" without a colon (1209.0869), "e-mail:" in parentheses (1405.3793), "Email:" before a
/// tabular's marked addresses (2607.12142), a label and an address in one ink (`Email: wangfy@bnu.edu.cn`, 1009.1965; 1807.01821,
/// 1509.01480, 2205.04084, 0704.3639).
pub(in crate::base_utilities::frontmatter::observer) fn printed_email_label(
  events: &[Event],
  span: (u16, u16),
  address: &str,
) -> Option<String> {
  let at = events.iter().position(|e| {
    matches!(e.kind, EventKind::Name | EventKind::Contact)
      && e.span.is_some_and(|s| s.0 <= span.0 && span.1 <= s.1)
  })?;
  let start = events[..at]
    .iter()
    .rposition(|e| e.kind == EventKind::Break)
    .map_or(0, |at| at + 1);
  let end = events[at..]
    .iter()
    .position(|e| e.kind == EventKind::Break)
    .map_or(events.len(), |n| at + n);
  let line: Vec<&Event> = events[start..end].iter().collect();
  let text = events_text(&line);
  // (the run holding the address, from the ink the reading read it at on: its label's, `E-mail:` before `\email{…}`, or its
  // own, the run's)
  let ink = events_text(&line[..at - start]).len();
  let runs = address_runs(&text, &|_| false);
  let after = || runs.iter().filter(|run| run.end > ink);
  after()
    .find(|run| run.addresses.iter().any(|a| a == address))
    .or_else(|| after().next())
    .and_then(|run| run.label.clone())
}

/// An ink of a block or a call whose words the reading writes nowhere ([`Written::lost_inks`]): its span, its text, the
/// words lost, as printed, and whether it is a name's ink on a line of its block's names (one its written names print every word
/// of a name the reading reads from it on), where it may be a name's.
pub(in crate::base_utilities::frontmatter::observer) struct LostInk {
  pub span:        Option<(u16, u16)>,
  pub text:        String,
  pub words:       Vec<String>,
  pub among_names: bool,
}

/// What the reading writes, by source: the words of its names, of its items' texts (an item each owner holds a copy of
/// once) and of an e-mail's printed label ([`printed_email_label`]), as multisets per block or call; every word it or a
/// document's note writes (a call's note may go there: hep-ex0609057's `\thanks` in an `\affiliation`, a publication note);
/// where its items are read, and its names.
pub(in crate::base_utilities::frontmatter::observer) struct Written {
  counts:   BTreeMap<usize, BTreeMap<String, usize>>,
  anywhere: BTreeSet<String>,
  /// the spans items are read at, and those of a contact the emit writes from its tokens (not an address, written as its
  /// text)
  spans:    BTreeSet<(usize, (u16, u16))>,
  contacts: BTreeSet<(usize, (u16, u16))>,
  /// per source, its names' words
  names:    BTreeMap<usize, Vec<BTreeSet<String>>>,
  /// per source, the words of its names and of the items it writes, as a multiset, and of its names alone: what
  /// [`Self::excess`] reads (no label; no note a ruling dropped; of an address its local part, a list's domain printed once
  /// for its addresses, `{a,b}@x`, and an address list written out not at all)
  read:     BTreeMap<usize, BTreeMap<String, usize>>,
  named:    BTreeMap<usize, BTreeMap<String, usize>>,
}

impl Written {
  pub(in crate::base_utilities::frontmatter::observer) fn new(
    ledger: &Ledger,
    resolved: &Resolved,
  ) -> Self {
    let mut counts: BTreeMap<usize, BTreeMap<String, usize>> = BTreeMap::new();
    let mut write = |at: usize, text: &str| {
      let counts = counts.entry(at).or_default();
      for word in words_of(text) {
        *counts.entry(word).or_default() += 1;
      }
    };
    let mut names: BTreeMap<usize, Vec<BTreeSet<String>>> = BTreeMap::new();
    let (mut read, mut named): (BTreeMap<usize, BTreeMap<String, usize>>, BTreeMap<_, _>) =
      Default::default();
    let tally = |counts: &mut BTreeMap<String, usize>, text: &str| {
      for word in words_of(text) {
        *counts.entry(word).or_default() += 1;
      }
    };
    for c in &resolved.creators {
      tally(read.entry(c.source).or_default(), &c.name);
      tally(named.entry(c.source).or_default(), &c.name);
      write(c.source, &c.name);
      names
        .entry(c.source)
        .or_default()
        .push(words_of(&c.name).collect());
    }
    let mut items: BTreeSet<(Source, &str, &str)> = BTreeSet::new();
    let mut unread: BTreeSet<(Source, &str, &str)> = BTreeSet::new();
    for a in resolved
      .creators
      .iter()
      .flat_map(|c| c.affiliations.iter().chain(&c.contacts).chain(&c.notes))
      .chain(&resolved.block)
      .chain(&resolved.dropped)
    {
      if let Some((at, span)) = a.source {
        items.insert(((at, span), a.role.as_str(), a.text.as_str()));
        if a.rule.contains("ruling3:written-out") || resolved.dropped.contains(a) {
          unread.insert(((at, span), a.role.as_str(), a.text.as_str()));
        }
      }
    }
    let (mut spans, mut contacts) = (BTreeSet::new(), BTreeSet::new());
    for &item @ ((at, span), role, text) in &items {
      write(at, text);
      if !unread.contains(&item) {
        let own = match role {
          "email" => text.split('@').next().unwrap_or(text),
          _ => text,
        };
        let counts = read.entry(at).or_default();
        tally(counts, own);
        // (a note's inner address a binding's call routes out of it prints in its contact, not the note: llncs's `\thanks{…
        // \email{…}}`, 2406.13066; a document's own `\email` stays the note's text, `EmailMeaning::Own`, 0807.0895)
        if let Some(span) = span
          && let Some(Entry::Author(block)) = ledger.entries.get(at)
          && let Outcome::Events { events, .. } = &block.outcome
          && let Some(note) = events
            .iter()
            .find(|e| e.kind == EventKind::Note && e.span == Some(span))
        {
          let routed = note
            .inner
            .iter()
            .filter(|e| e.kind == EventKind::Contact && e.meaning != EmailMeaning::Own);
          for word in routed.flat_map(|e| words_of(&e.text)) {
            counts.entry(word).and_modify(|n| *n = n.saturating_sub(1));
          }
        }
      }
      let Some(span) = span else {
        continue;
      };
      spans.insert((at, span));
      if role != "email" {
        contacts.insert((at, span));
      }
      if role == "email"
        && let Some(Entry::Author(block)) = ledger.entries.get(at)
        && let Outcome::Events { events, .. } = &block.outcome
        && let Some(label) = printed_email_label(events, span, text)
      {
        write(at, &label);
      }
    }
    let notes = ledger.entries.iter().filter_map(|entry| match entry {
      Entry::Api(call) if call.tag == "ltx:pubnote" => Some(events::printed_text(&call.events)),
      _ => None,
    });
    let notes: Vec<String> = notes.collect();
    let anywhere = counts
      .values()
      .flat_map(|counts| counts.keys().cloned())
      .chain(notes.iter().flat_map(|note| words_of(note)))
      .collect();
    Self {
      counts,
      anywhere,
      spans,
      contacts,
      names,
      read,
      named,
    }
  }

  /// The words the reading writes of the block at `index` more times than its `events` print them (the other side of
  /// [`Self::lost_inks`]: a duplicated address, an invented word), each with whether its names alone do: its names' and
  /// items' words against what its inks and its separators' words print, a contact's link target counting as printed
  /// (`\href{mailto:x}{…}`). A word of two letters at most aside, a fragment or a mark.
  pub(in crate::base_utilities::frontmatter::observer) fn excess(
    &self,
    index: usize,
    events: &[Event],
  ) -> Vec<(String, bool)> {
    let mut printed: BTreeMap<String, usize> = BTreeMap::new();
    let mut count = |text: &str| {
      for word in words_of(text) {
        *printed.entry(word).or_default() += 1;
      }
    };
    for e in events {
      match &e.kind {
        EventKind::Name | EventKind::Contact | EventKind::Note => count(&e.text),
        EventKind::Sep(SepKind::Word(word)) => count(word),
        _ => {},
      }
      for contact in std::iter::once(e).chain(&e.inner) {
        if contact.kind == EventKind::Contact
          && let Some(target) = contact.target.as_deref()
        {
          count(target);
        }
      }
    }
    let printed_of = |word: &str| printed.get(word).copied().unwrap_or_default();
    let named = self.named.get(&index);
    let read = self.read.get(&index).into_iter().flatten();
    let over = read.filter(|(word, n)| word.chars().count() > 2 && printed_of(word) < **n);
    over
      .map(|(word, _)| {
        let by_names = named
          .and_then(|named| named.get(word))
          .is_some_and(|&n| printed_of(word) < n);
        (word.clone(), by_names)
      })
      .collect()
  }

  /// Whether the ink `e` of the block or call at `index` is no loss: one a container in an item's span holds (a picture's in an
  /// entry), a contact read at its own span (an ORCID link's logo, 2409.13798; not an address, 2607.28418), one a mark's
  /// formula holds (its label or its logo: an ORCID icon, 2210.03748), and an ink of two letters at most, a fragment the
  /// reading joins or reads as a mark (an accent's letter, `Karlu$\rm{\check{s}i\acute{c}}$`, 0911.1045; a label letter).
  fn exempt(&self, index: usize, e: &Event, marks: &[(u16, u16)]) -> bool {
    let within = |&(from, to): &(u16, u16)| {
      e.span
        .is_some_and(|(s, t)| from <= s && t <= to && (s, t) != (from, to))
    };
    let mut of_index = self
      .spans
      .range((index, (0, 0))..=(index, (u16::MAX, u16::MAX)));
    e.text.chars().filter(|c| c.is_alphanumeric()).count() <= 2
      || (e.kind == EventKind::Contact
        && e.role.as_deref() != Some("email")
        && e
          .span
          .is_some_and(|span| self.contacts.contains(&(index, span))))
      || (e.nest > 0 && of_index.any(|(_, span)| within(span)))
      || marks.iter().any(within)
  }

  /// The inks of `events`, a block's or a call's at ledger index `index`, whose words it doesn't write: a block's as a
  /// multiset against the names and items read from it, which the emit writes from its tokens (an ink the reading parts
  /// loses nothing, aa's two addresses in one `\email`, 2306.01880; "Ann" isn't held by "Annette", review revZ4ar2); a
  /// call's against every word written (a call printing an entry another prints too is read once: revtex's
  /// `\affiliation{\Argonne}` listed before the authors, to number the institutes, and again under one, 2103.04797), an ink
  /// [`Self::exempt`] none.
  pub(in crate::base_utilities::frontmatter::observer) fn lost_inks(
    &self,
    index: usize,
    events: &[Event],
    a_block: bool,
  ) -> Vec<LostInk> {
    let marks: Vec<(u16, u16)> = events
      .iter()
      .filter(|m| m.kind == EventKind::Mark)
      .filter_map(|m| m.formula.and_then(|f| f.span))
      .collect();
    // (each ink's line)
    let mut line = 0usize;
    let lines: Vec<usize> = events
      .iter()
      .map(|e| {
        let at = line;
        line += usize::from(e.kind == EventKind::Break);
        at
      })
      .collect();
    let inks: Vec<(&Event, usize)> = events
      .iter()
      .zip(lines.iter().copied())
      .filter(|(e, _)| {
        matches!(
          e.kind,
          EventKind::Name | EventKind::Contact | EventKind::Note
        ) && !self.exempt(index, e, &marks)
      })
      .collect();
    let mut counts = self.counts.get(&index).cloned().unwrap_or_default();
    // (a block's ink every word of which is written takes its words first: a name the reading holds, before an address
    // spelling it printed above it, `\texttt{ann.able@outlook}\\ Ann Able`)
    let mut held = vec![false; inks.len()];
    if a_block {
      for (k, (e, _)) in inks.iter().enumerate() {
        let mut words: BTreeMap<String, usize> = BTreeMap::new();
        for word in words_of(&e.text) {
          *words.entry(word).or_default() += 1;
        }
        if words
          .iter()
          .all(|(word, n)| counts.get(word).is_some_and(|have| have >= n))
        {
          for (word, n) in words {
            counts.entry(word).and_modify(|have| *have -= n);
          }
          held[k] = true;
        }
      }
    }
    // (the lines its written names print every word of a name the reading reads from it on: its names' lines)
    let mut printed: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for ((e, line), _) in inks.iter().zip(&held).filter(|(_, held)| **held) {
      if e.kind == EventKind::Name {
        printed.entry(*line).or_default().extend(words_of(&e.text));
      }
    }
    let names = self.names.get(&index).map_or(&[][..], Vec::as_slice);
    let names_line = |line: usize| {
      printed.get(&line).is_some_and(|words| {
        names
          .iter()
          .any(|name| !name.is_empty() && name.is_subset(words))
      })
    };
    let mut lost = Vec::new();
    for ((e, line), _) in inks.iter().zip(&held).filter(|(_, held)| !**held) {
      let mut words = Vec::new();
      for printed in printed_words(&e.text) {
        let word = printed.to_lowercase();
        match counts.get_mut(&word) {
          Some(n) if *n > 0 && a_block => *n -= 1,
          _ if !a_block && self.anywhere.contains(&word) => {},
          _ => words.push(printed.to_string()),
        }
      }
      if !words.is_empty() {
        lost.push(LostInk {
          span: e.span,
          text: e.text.clone(),
          words,
          among_names: a_block && e.kind == EventKind::Name && names_line(*line),
        });
      }
    }
    lost
  }
}

/// One block's word check ([`check_blocks`]): its inks the reading writes nowhere, and the words it writes more times than
/// they print.
pub(in crate::base_utilities::frontmatter::observer) struct BlockCheck {
  pub lost:   Vec<LostInk>,
  pub excess: Vec<(String, bool)>,
}

/// The block half of the word check, both ways: per author block (its ledger index) that fails it, its [`BlockCheck`]. What a
/// later call replaced or cleared, as the kernel's `\lx@clear@frontmatter` does, prints in no output: a template's
/// `\author{First Author}` before the paper's own, 2412.15554.
pub(in crate::base_utilities::frontmatter::observer) fn check_blocks(
  ledger: &Ledger,
  resolved: &Resolved,
) -> BTreeMap<usize, BlockCheck> {
  let written = Written::new(ledger, resolved);
  let replaced = |at: usize, epoch: usize| {
    ledger.entries[at + 1..]
      .iter()
      .any(|later| matches!(later, Entry::Author(later) if later.replace && later.epoch == epoch))
  };
  let mut checks = BTreeMap::new();
  for (index, entry) in ledger.entries.iter().enumerate() {
    if let Entry::Author(block) = entry
      && !block.cleared
      && !replaced(index, block.epoch)
      && let Outcome::Events { events, .. } = &block.outcome
    {
      let check = BlockCheck {
        lost:   written.lost_inks(index, events, true),
        excess: written.excess(index, events),
      };
      if !check.lost.is_empty() || !check.excess.is_empty() {
        checks.insert(index, check);
      }
    }
  }
  checks
}
