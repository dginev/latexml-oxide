//! The reading's word check (FIDELITY_CHARTER §4, "no printed frontmatter word is lost"; OXIDIZED_DESIGN_DIVERGENCES #980):
//! the words an author block or a creator's call prints (its Names, Contacts and Notes) that the reading writes nowhere,
//! whatever lost them (review revZ4a: 1509.01480's creator and her note, 2607.28418's address). The block half is a pass after
//! the resolver, on the ledger and [`Resolved`] alone ([`unwritten_block_inks`]); the call half, which needs what the emit
//! wrote, is the emitter's, on [`Written`] (`emitter::emit`).

use std::collections::{BTreeMap, BTreeSet};

use super::{
  super::ledger::{AuthorBlock, Entry, Ledger, Outcome},
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

/// Text compared by its letters and digits alone, lowercased (the deleted `ab_audit fmshadow`'s `norm`): how the emitter
/// keys an item, and [`entry_span_in`] reads its entry.
pub(in crate::base_utilities::frontmatter::observer) fn norm(text: &str) -> String {
  text
    .chars()
    .filter(|c| c.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

/// The span of a block's entry the reading keys `key` (its text by [`norm`]), its source `span` its first event's (an entry
/// of several pieces, `$^1$Department of Physics, Univ X, Japan`, is sourced at its first Name: 2307.03384): through the
/// `events` printed after it, until their ink joined is the entry's text — over the lines an unmarked line continued, past
/// the layout between (an image: `Zebra University, \includegraphics{…} \url{…}` one affiliation, review revIMG1
/// finding 1) — `None` when it never is (a note, a mark, or other ink first). The span the emitter writes the entry from
/// (`emitter::repart::entry_span`), and what holds a block's pictures ([`Written::unwritten_pictures`]).
pub(in crate::base_utilities::frontmatter::observer) fn entry_span_in(
  events: &[Event],
  span: (u16, u16),
  key: &str,
) -> Option<(u16, u16)> {
  let first = events
    .iter()
    .position(|e| e.kind != EventKind::Layout && e.span.is_some_and(|s| s.0 == span.0))?;
  let (mut joined, mut end) = (String::new(), span.1);
  for event in &events[first..] {
    match &event.kind {
      EventKind::Name | EventKind::Contact => joined.push_str(&norm(&event.text)),
      // (the word "and" inside an entry is the entry's: `Computer Science and Engineering`, 1304.1574)
      EventKind::Sep(SepKind::Word(word)) => joined.push_str(&norm(word)),
      EventKind::Sep(_) | EventKind::Break | EventKind::Layout => continue,
      _ => return None,
    }
    if let Some((_, e)) = event.span {
      end = end.max(e);
    }
    if joined == key {
      return Some((span.0, end));
    }
    if !key.starts_with(&joined) {
      return None;
    }
  }
  None
}

/// The words a multiset of them counts: [`printed_words`], lowercased.
pub(in crate::base_utilities::frontmatter::observer) fn words_of(
  text: &str,
) -> impl Iterator<Item = String> + '_ {
  printed_words(text).map(str::to_lowercase)
}

/// Where the addresses of `run`, an address run of `text`, begin: after its label word (a label is the contact's name,
/// written or not, `email:` before `Duncan.Lorimer@mail.wvu.edu`, 0811.0762), else at the run.
fn address_start(text: &str, run: &AddressRun) -> usize {
  run
    .label
    .as_deref()
    .and_then(|label| {
      text[run.start..run.end]
        .find(label)
        .map(|at| run.start + at + label.len())
    })
    .unwrap_or(run.start)
}

/// What `text` prints, as the content check counts it (`placed`): its words ([`printed_words`]) outside its address runs
/// ([`address_runs`]), and the addresses those runs print, lowercased, a list's each (`{ann,bob}@x.org` two). An address's
/// local-part and domain words are the address's, never a word of the text: a domain `idsia` writes no `IDSIA`
/// (1605.03019, review revN1r3).
pub(in crate::base_utilities::frontmatter::observer) fn ink_words(
  text: &str,
) -> (Vec<String>, Vec<String>) {
  if !text.contains('@') {
    return (
      printed_words(text).map(str::to_string).collect(),
      Vec::new(),
    );
  }
  // (the braces of a group set in math, as [`addresses_in`] reads them: `$\{$fargyriou,mikebekos$\}$@math.ntua.gr`, 1106.2694;
  // and a tie a raw address text keeps, the space it prints — llncs's `\email{$\{$stomforde$|$bsick$\}$@uni-kassel.de;~cms@…}`,
  // whose `cms@sra.uni-hanover.de` the writer reads past the tie as the e-mail readers do (`email_block_calls`), was no run
  // here and its domain words counted as printed words, missing: 1701.08125, `perfect_kernel_batch85b::llncs_listed_local_parts_keep_their_braces`)
  let unshifted;
  let text = if text.contains(['$', '~']) {
    unshifted = text.replace('$', "").replace('~', " ");
    unshifted.as_str()
  } else {
    text
  };
  let runs = address_runs(text, &|_| false);
  let mut cut: Vec<(usize, usize)> = runs
    .iter()
    .map(|run| (address_start(text, run), run.end))
    .collect();
  let mut addresses: Vec<String> = runs
    .into_iter()
    .flat_map(|run| run.addresses)
    .map(|a| a.to_lowercase())
    .collect();
  // (and an address no run reads, glued to what prints before or after it: `——meizeh@cs.technion.ac.il`, 1503.06438; `†caprini@…`,
  // 1202.5401), from its first letter or digit to its last
  for token in text.split_whitespace() {
    let from = token.as_ptr() as usize - text.as_ptr() as usize;
    let to = from + token.len();
    let address = token.trim_matches(|c: char| !c.is_alphanumeric());
    let read = cut.iter().any(|&(s, t)| s < to && from < t);
    if !read
      && address
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
    {
      cut.push((from, to));
      addresses.push(address.to_lowercase());
    }
  }
  let words = words_at(text)
    .filter(|((from, to), _)| !cut.iter().any(|&(s, t)| s <= *from && *to <= t))
    .map(|(_, word)| word.to_string())
    .collect();
  (words, addresses)
}

/// The e-mail label the paper prints before `address` at `span` of a block's `events`, as the reading reads an address run's
/// label on its line ([`address_runs`]' `label_word_before`): the contact's `name`, written as printed. "E-mail:" before a
/// class's `\email{…}` (PoS's, 1412.5940, 1612.08209, 1812.02252, 1907.01311, 1005.2532, 0802.3384), a list's on each of its
/// addresses ("Emails:", 2511.22090), "email" without a colon (1209.0869), "e-mail:" in parentheses (1405.3793), "Email:" before a
/// tabular's marked addresses (2607.12142), a label and an address in one ink (`Email: wangfy@bnu.edu.cn`, 1009.1965; 1807.01821,
/// 1509.01480, 2205.04084, 0704.3639); in any language, by its place: one word or two and a colon, all the line prints before
/// the address's run (`Correo electrónico:`, `Courriel :`).
pub(in crate::base_utilities::frontmatter::observer) fn printed_email_label(
  events: &[Event],
  span: (u16, u16),
  address: &str,
) -> Option<String> {
  if let Some(name) = own_email_label(events, span) {
    return Some(name);
  }
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
  let run = after()
    .find(|run| run.addresses.iter().any(|a| a == address))
    .or_else(|| after().next())?;
  // (or, in any language, what its line prints before the run, one word or two before a colon: `Correo electrónico:`,
  // `Courriel :`, review revZ4br2)
  let before = text[..run.start].trim();
  run
    .label
    .clone()
    .or_else(|| reads_as_email_label(before).then(|| before.to_string()))
}

/// The label the class's own e-mail command prints with the address at `span` of a block's `events`, read as it prints —
/// jfm/jpp's `\email` printing "Email address for correspondence:" inside a `\thanks`, 2603.18265: the contact's `name`,
/// the rehearsal's `own_email_properties`. The command's call prints it with the address wherever it is written whole
/// (`insert::NAME_PRINTED`).
pub(in crate::base_utilities::frontmatter::observer) fn own_email_label(
  events: &[Event],
  span: (u16, u16),
) -> Option<String> {
  let at_span = |e: &&Event| e.kind == EventKind::Contact && e.span == Some(span);
  events
    .iter()
    .flat_map(|e| std::iter::once(e).chain(&e.inner))
    .find(at_span)
    .and_then(|e| e.name.as_deref())
    .map(|name| name.trim().to_string())
}

/// Whether `before`, what a line prints before an address's run, is the address's label by its place, in any language: one
/// word or two before a colon (`Correo electrónico:`, `Courriel :`, review revZ4br2).
pub(in crate::base_utilities::frontmatter::observer) fn reads_as_email_label(before: &str) -> bool {
  before.ends_with(':')
    && before.split_whitespace().count() <= 2
    && before.chars().any(char::is_alphabetic)
}

/// An ink of a block or a call whose words the reading writes nowhere ([`Written::lost_inks`]): its span, its text, the
/// words lost, as printed, where they print in it, the address runs it prints that the reading writes as its e-mail
/// items (cut out of a note of it), its owner, and whether it fails the block's names.
#[derive(Debug)]
pub(in crate::base_utilities::frontmatter::observer) struct LostInk {
  pub span:       Option<(u16, u16)>,
  pub text:       String,
  pub words:      Vec<String>,
  /// the byte ranges of `text` its lost words print at, in print order
  lost_at:        Vec<(usize, usize)>,
  /// the byte ranges of `text` its written address runs print at ([`address_runs`], each address an e-mail item the
  /// reading writes for the block)
  cut:            Vec<(usize, usize)>,
  /// the creator (its index in [`Resolved::creators`]) whose line it is, by the reading's own rule of an address
  /// ([`spelled_name`]): the one author the addresses its ink prints spell — `POC: K. Hobbs (kerianne.hobbs@us.af.mil)`
  /// Kerianne L. Hobbs's (2110.03506), `Germany (martin.waltz@tu-dresden.de)` Martin Waltz's (2203.10777),
  /// `yiliu.li@outlook` Aiden Yiliu Li's (2605.16481); not by its line's, a list's local parts printed apart owning none
  pub owner:      Option<usize>,
  /// whether it fails the block's names: a Name ink of a block losing a word of letters outside its addresses (a postcode
  /// is no name's, 2501.09580's `Hefei 230026`, its block of 168 names written whole for it; nor an address's word,
  /// 2601.00680), no author's line by its addresses, that is a name: on a line of its block's names, or one an address
  /// its ink, its line or its column prints spells by the reading's rule — a name the reading lost, JAIR's `\name Shaul Markovitch
  /// \email shaulm@…` (1109.6618; 1312.0286, 1111.6082, 1905.12960, 1807.03929, 2003.01013, 2008.13618; a column's own,
  /// 1805.08288, 2002.00084: review revN1); not a label (`Emails:- …`, 2605.21396) or an entry's line (1109.0220)
  pub names_fail: bool,
  /// its font as digested (a Name's first character's), which a note of it keeps
  pub font:       Option<Rc<Font>>,
}

impl LostInk {
  /// What a note of it holds: its text from its first lost word to its last, as printed, where no written address run
  /// prints among them (`POC: K. Hobbs` of `POC: K. Hobbs (kerianne.hobbs@us.af.mil)`, the address its contact's, 2110.03506;
  /// `Correo electrónico:` of `Correo electrónico: ann@a.org`, review revZ4br2), by position, not by word (a word the
  /// address repeats, `Hobbs`, is the address's where it prints in it: review revN1); its whole text, as the base wrote
  /// it, where an address it writes prints among them.
  pub fn note_text(&self) -> &str {
    match (self.lost_at.first(), self.lost_at.last()) {
      (Some(&(first, _)), Some(&(_, last)))
        if !self.cut.iter().any(|&(from, to)| first < to && from < last) =>
      {
        &self.text[first..last]
      },
      _ => &self.text,
    }
  }

  /// Whether it loses every word it prints, no address of it written: its tokens are its note.
  pub fn whole(&self) -> bool {
    self.cut.is_empty() && printed_words(&self.text).count() == self.words.len()
  }
}

/// `text`'s printed words ([`printed_words`]) with their byte ranges.
fn words_at(text: &str) -> impl Iterator<Item = ((usize, usize), &str)> {
  printed_words(text).map(move |word| {
    // (the word's offset: `printed_words` yields subslices of the text)
    let start = word.as_ptr() as usize - text.as_ptr() as usize;
    ((start, start + word.len()), word)
  })
}

/// What the reading writes, by source: the words of its names, of its items' texts (an item each owner holds a copy of
/// once) and of an e-mail's printed label ([`printed_email_label`]), as multisets per block or call; every word it or a
/// document's note writes (a call's note may go there: hep-ex0609057's `\thanks` in an `\affiliation`, a publication note);
/// where its items are read, and its names.
pub(in crate::base_utilities::frontmatter::observer) struct Written {
  counts:    BTreeMap<usize, BTreeMap<String, usize>>,
  anywhere:  BTreeSet<String>,
  /// the spans items are read at, and those of a contact the emit writes from its tokens (not an address, written as its
  /// text)
  spans:     BTreeSet<(usize, (u16, u16))>,
  contacts:  BTreeSet<(usize, (u16, u16))>,
  /// the spans of the items it writes, none a ruling dropped: what holds a block's pictures ([`Self::unwritten_pictures`])
  holding:   BTreeSet<(usize, (u16, u16))>,
  /// per source, its creators (their index in [`Resolved::creators`]) and names
  names:     BTreeMap<usize, Vec<(usize, String)>>,
  /// per source, the e-mail addresses it writes as items, lowercased, as often as one holder holds each: what an ink's
  /// address run is read as ([`Self::lost_inks`])
  addresses: BTreeMap<usize, BTreeMap<String, usize>>,
  /// per source, the words of its names and of the items it writes, as a multiset, and of its names alone: what
  /// [`Self::excess`] reads (no label; no note a ruling dropped; of an address its local part, a list's domain printed once
  /// for its addresses, `{a,b}@x`, and an address list written out not at all)
  read:      BTreeMap<usize, BTreeMap<String, usize>>,
  named:     BTreeMap<usize, BTreeMap<String, usize>>,
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
    let mut names: BTreeMap<usize, Vec<(usize, String)>> = BTreeMap::new();
    let (mut read, mut named): (BTreeMap<usize, BTreeMap<String, usize>>, BTreeMap<_, _>) =
      Default::default();
    let tally = |counts: &mut BTreeMap<String, usize>, text: &str| {
      for word in words_of(text) {
        *counts.entry(word).or_default() += 1;
      }
    };
    for (i, c) in resolved.creators.iter().enumerate() {
      tally(read.entry(c.source).or_default(), &c.name);
      tally(named.entry(c.source).or_default(), &c.name);
      write(c.source, &c.name);
      names.entry(c.source).or_default().push((i, c.name.clone()));
    }
    for (at, _, word) in &resolved.drops {
      write(*at, word);
    }
    // (an item as often as one holder holds it: a copy each owner holds counts once, two in one holder twice, 2607.00684's
    // `mtablo@postech.ac.kr` written twice for the block)
    let mut items: BTreeMap<(Source, &str, &str), usize> = BTreeMap::new();
    let mut unread: BTreeSet<(Source, &str, &str)> = BTreeSet::new();
    let mut holding = BTreeSet::new();
    let holders = resolved
      .creators
      .iter()
      .map(|c| {
        c.affiliations
          .iter()
          .chain(&c.contacts)
          .chain(&c.notes)
          .collect::<Vec<_>>()
      })
      .chain([
        resolved.block.iter().collect(),
        resolved.dropped.iter().collect(),
      ]);
    for held in holders {
      let mut here: BTreeMap<(Source, &str, &str), usize> = BTreeMap::new();
      for a in held {
        if let Some((at, span)) = a.source {
          let item = ((at, span), a.role.as_str(), a.text.as_str());
          *here.entry(item).or_default() += 1;
          if a.rule.contains("ruling3:written-out") || resolved.dropped.contains(a) {
            unread.insert(item);
          }
          // (what it writes: its entry's span, the emit's (`entry_span_in`), wider than its source span where its text
          // runs on past layout — an affiliation holding the URL after an image, `Zebra University, \includegraphics{…}
          // \url{…}`, review revIMG1 finding 1 — and its source span where the entry can't be read so)
          if let Some(span) = span
            && !resolved.dropped.contains(a)
          {
            let written = match ledger.entries().get(at) {
              Some(Entry::Author(block)) => match &block.outcome {
                Outcome::Events { events, .. } => entry_span_in(events, span, &norm(&a.text)),
                _ => None,
              },
              _ => None,
            };
            holding.insert((at, written.unwrap_or(span)));
          }
        }
      }
      for (item, n) in here {
        let most = items.entry(item).or_default();
        *most = (*most).max(n);
      }
    }
    let (mut spans, mut contacts) = (BTreeSet::new(), BTreeSet::new());
    let mut addresses: BTreeMap<usize, BTreeMap<String, usize>> = BTreeMap::new();
    let each = items
      .iter()
      .flat_map(|(item, n)| std::iter::repeat_n(*item, *n));
    for item @ ((at, span), role, text) in each {
      write(at, text);
      if role == "email" {
        *addresses
          .entry(at)
          .or_default()
          .entry(text.to_lowercase())
          .or_default() += 1;
      }
      if !unread.contains(&item) {
        let own = match role {
          "email" => text.split('@').next().unwrap_or(text),
          _ => text,
        };
        let counts = read.entry(at).or_default();
        tally(counts, own);
        // (a note's inner address the reading writes as its contact counts once, in the contact: llncs's `\thanks{…
        // \email{…}}`, 2406.13066; a note's call's too, siamart's `\thanks{… \email{…}}`, 1804.09455, whose `\email`
        // (OmniBus's, locked) the rehearsal reads as the binding's, `EmailMeaning::Binding` (2511.14713), and digestion routes
        // out. So an.cls's raw `\email` a `\thanks` keeps, 0807.0895, written in the note and its contact, goes uncounted)
        let inner: &[Event] = match (span, ledger.entries().get(at)) {
          (Some(span), Some(Entry::Author(block))) => match &block.outcome {
            Outcome::Events { events, .. } => events
              .iter()
              .find(|e| e.kind == EventKind::Note && e.span == Some(span))
              .map_or(&[], |note| &note.inner),
            _ => &[],
          },
          (None, Some(Entry::Api(call))) if call.tag == "ltx:note" => &call.events,
          _ => &[],
        };
        let routed = inner.iter().filter(|e| e.kind == EventKind::Contact);
        for word in routed.flat_map(|e| words_of(&e.text)) {
          counts.entry(word).and_modify(|n| *n = n.saturating_sub(1));
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
        && let Some(Entry::Author(block)) = ledger.entries().get(at)
        && let Outcome::Events { events, .. } = &block.outcome
        && let Some(label) = printed_email_label(events, span, text)
      {
        write(at, &label);
      }
    }
    let notes = ledger.entries().iter().filter_map(|entry| match entry {
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
      holding,
      names,
      addresses,
      read,
      named,
    }
  }

  /// The pictures of the block at ledger index `index` (an image, the Layout event of an `\includegraphics`; a picture note,
  /// `events::PICTURE_NOTE`) that no item it writes holds: each one's span inside no written item's span, or no span to
  /// tell (fail toward flagging). An image has no word, so the word check ([`Self::lost_inks`]) cannot miss it: the image
  /// leading a mark or an address on a later line, which `events::place_pictures` has no place for
  /// (`\includegraphics{…} $^{1}$Zebra University`, `\includegraphics{…} \texttt{ann@x.org}`), was lost silently. One a URL
  /// contact's span holds (2503.08161's `hf-logo.png`, 2505.21327's icons) or its own note's is written.
  pub(in crate::base_utilities::frontmatter::observer) fn unwritten_pictures(
    &self,
    index: usize,
    events: &[Event],
  ) -> usize {
    let of_index = || {
      self
        .holding
        .range((index, (0, 0))..=(index, (u16::MAX, u16::MAX)))
    };
    events
      .iter()
      .filter(|e| is_image(e) || events::is_picture_note(e))
      .filter(|e| {
        e.span
          .is_none_or(|(s, t)| !of_index().any(|&(_, (from, to))| from <= s && t <= to))
      })
      .count()
  }

  /// The words the reading writes of the block or call at `index` more times than its `events` print them (the other side
  /// of [`Self::lost_inks`]: a duplicated address, 2607.00684's `mtablo@postech.ac.kr` twice in one creator; an invented
  /// word), each with whether its names alone do: its names' and items' words against what its inks and its separators'
  /// words print, a link's target counting as printed where its anchor prints no address of its own: 2607.28418's
  /// `\href{mailto:email@domain}{ann@a.edu; bob@b.edu}` prints no `email@domain` (reviews revZ4br2, revN1). A word of two
  /// letters at most aside, a fragment or a mark.
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
      // (a link's target is what it carries where its anchor prints no address of its own, an ORCID's id behind a logo or a
      // name, 2210.03748, 2206.03602, 2102.04676; an anchor printing addresses prints only them, its target none:
      // `\href{mailto:email@domain}{ann@a.edu; bob@b.edu}`, 2607.28418, reviews revZ4br2, revN1)
      for contact in std::iter::once(e).chain(&e.inner) {
        if contact.kind == EventKind::Contact
          && let Some(target) = contact.target.as_deref()
          && addresses_in(&contact.text).is_empty()
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
  /// loses nothing, aa's two addresses in one `\email`, 2306.01880; "Ann" isn't held by "Annette", review revZ4ar2), an
  /// address run it prints that the reading writes as an e-mail item first the address's, by position (its words none of a
  /// name's: `POC: K. Hobbs (kerianne.hobbs@us.af.mil)`, 2110.03506; `Germany (martin.waltz@tu-dresden.de)`, 2203.10777;
  /// review revN1); a call's against every word written (a call printing an entry another prints too is read once:
  /// revtex's `\affiliation{\Argonne}` listed before the authors, to number the institutes, and again under one,
  /// 2103.04797), an ink [`Self::exempt`] none.
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
    // (each ink's line: a run between breaks, or a container's (a box, an environment, a cell: a column of names, one author
    // each, 1805.08288's minipages, 2002.00084))
    // (and its column: a container's events, its lines with them)
    let (mut unit, mut column) = (0usize, 0usize);
    let (units, columns): (Vec<usize>, Vec<usize>) = events
      .iter()
      .enumerate()
      .map(|(k, e)| {
        let container =
          e.kind == EventKind::Layout && events.get(k + 1).is_some_and(|next| next.nest > e.nest);
        unit += usize::from(e.kind == EventKind::Break || container);
        column += usize::from(container);
        (unit, column)
      })
      .unzip();
    let column_of: BTreeMap<usize, usize> =
      units.iter().copied().zip(columns.iter().copied()).collect();
    // (and its line of the block's names, a run between breaks: a unit is one line's)
    let mut line = 0usize;
    let line_of: BTreeMap<usize, usize> = events
      .iter()
      .zip(&units)
      .map(|(e, unit)| {
        line += usize::from(e.kind == EventKind::Break);
        (*unit, line)
      })
      .collect();
    let inks: Vec<(&Event, usize)> = events
      .iter()
      .zip(units.iter().copied())
      .filter(|(e, _)| {
        matches!(
          e.kind,
          EventKind::Name | EventKind::Contact | EventKind::Note
        ) && !self.exempt(index, e, &marks)
      })
      .collect();
    let mut counts = self.counts.get(&index).cloned().unwrap_or_default();
    let take = |counts: &mut BTreeMap<String, usize>, word: &str| match counts.get_mut(word) {
      Some(n) if *n > 0 => {
        *n -= 1;
        true
      },
      _ => false,
    };
    // (each ink's address runs first: one the reading writes, every address of it an e-mail item of the block, is cut out
    // of it, its words the address's; a word of it no item writes, its label's say, lost where it prints)
    let mut written = if a_block {
      self.addresses.get(&index).cloned().unwrap_or_default()
    } else {
      BTreeMap::new()
    };
    struct Read<'a> {
      cut:       Vec<(usize, usize)>,
      addresses: Vec<String>,
      lost:      Vec<((usize, usize), &'a str)>,
    }
    let mut reads: Vec<Read> = Vec::with_capacity(inks.len());
    for (e, _) in &inks {
      let mut read = Read {
        cut:       Vec::new(),
        addresses: Vec::new(),
        lost:      Vec::new(),
      };
      if e.text.contains('@') {
        for run in address_runs(&e.text, &|_| false) {
          let mut wanted: BTreeMap<String, usize> = BTreeMap::new();
          for address in &run.addresses {
            *wanted.entry(address.to_lowercase()).or_default() += 1;
          }
          if a_block
            && wanted
              .iter()
              .all(|(address, n)| written.get(address).is_some_and(|have| have >= n))
          {
            for (address, n) in wanted {
              written.entry(address).and_modify(|have| *have -= n);
            }
            // (its addresses, from after its label word: a label is the contact's name, written or not, `email:` before
            // `Duncan.Lorimer@mail.wvu.edu`, 0811.0762)
            let start = address_start(&e.text, &run);
            for ((from, to), word) in words_at(&e.text[start..run.end]) {
              if !take(&mut counts, &word.to_lowercase()) {
                read.lost.push(((start + from, start + to), word));
              }
            }
            read.cut.push((start, run.end));
          }
        }
        // (the addresses it prints, a run or not: `ann@a.org (corresponding)`; or the ink one word holding an `@`, its domain
        // no address's: `yiliu.li@outlook`, 2605.16481)
        read.addresses = addresses_in(&e.text);
        let word = e.text.trim();
        if read.addresses.is_empty() && !word.contains(char::is_whitespace) {
          read.addresses.push(word.to_string());
        }
      }
      reads.push(read);
    }
    let outside = |k: usize| {
      let (e, _) = inks[k];
      let cut = &reads[k].cut;
      words_at(&e.text)
        .filter(move |((from, to), _)| !cut.iter().any(|&(s, t)| s <= *from && *to <= t))
    };
    // (a block's ink every word of which is written takes its words first: a name the reading holds, before an address
    // spelling it printed above it, `\texttt{ann.able@outlook}\\ Ann Able`; an ink of a line an item other than an address
    // is read from (its source the line's first event) before any other, a line the reading lost taking no word of the
    // items it repeats: `Dresden, Saxony 01062, Germany (…)` under `Technische Universität Dresden,`, 2203.10777)
    let read_units: BTreeSet<usize> = events
      .iter()
      .zip(&units)
      .filter(|(e, _)| {
        e.span
          .is_some_and(|span| self.contacts.contains(&(index, span)))
      })
      .map(|(_, unit)| *unit)
      .collect();
    let read_from = |k: usize| read_units.contains(&inks[k].1);
    let mut held = vec![false; inks.len()];
    if a_block {
      let order = (0..inks.len())
        .filter(|&k| read_from(k))
        .chain((0..inks.len()).filter(|&k| !read_from(k)));
      for k in order {
        let mut words: BTreeMap<String, usize> = BTreeMap::new();
        for (_, word) in outside(k) {
          *words.entry(word.to_lowercase()).or_default() += 1;
        }
        if reads[k].lost.is_empty()
          && words
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
    let names = self.names.get(&index).map_or(&[][..], Vec::as_slice);
    let spelled_by: Vec<&str> = names.iter().map(|(_, name)| name.as_str()).collect();
    // (whom an address spells by the reading's rule, among the written names and `own`, a lost name the address may be its
    // own: `philips@` Andy Philips's beside Philip Laird, 1908.02923; review revN1r2)
    let holder = |address: &str, own: Option<&str>| -> Option<Holder> {
      let mut candidates = spelled_by.clone();
      candidates.extend(own);
      spelled_name(&candidates, address).map(|k| match names.get(k) {
        Some(&(creator, _)) => Holder::Author(creator),
        None => Holder::Own,
      })
    };
    // (the one holder addresses spell, every one of them spelling that one)
    let one_holder = |addresses: &mut dyn Iterator<Item = &String>, own: Option<&str>| {
      let spelled: Option<BTreeSet<Holder>> =
        addresses.map(|address| holder(address, own)).collect();
      match spelled.unwrap_or_default().into_iter().collect::<Vec<_>>()[..] {
        [one] => Some(one),
        _ => None,
      }
    };
    // (the lines its written names print every word of a name the reading reads from it on: its names' lines)
    let mut printed: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for (k, (e, unit)) in inks.iter().enumerate() {
      // (an address's ink prints no name: `ann.hobbs@a.org` beside `Contact: A. Hobbs,`)
      if held[k] && e.kind == EventKind::Name && reads[k].addresses.is_empty() {
        printed
          .entry(line_of[unit])
          .or_default()
          .extend(words_of(&e.text));
      }
    }
    let names_line = |unit: usize| {
      printed.get(&line_of[&unit]).is_some_and(|words| {
        names.iter().any(|(_, name)| {
          let mut name = words_of(name).peekable();
          name.peek().is_some() && name.all(|word| words.contains(&word))
        })
      })
    };
    let (mut by_unit, mut by_column): (BTreeMap<usize, Vec<usize>>, BTreeMap<usize, Vec<usize>>) =
      Default::default();
    for (k, (_, unit)) in inks.iter().enumerate() {
      by_unit.entry(*unit).or_default().push(k);
      by_column.entry(column_of[unit]).or_default().push(k);
    }
    let mut lost = Vec::new();
    for k in (0..inks.len()).filter(|&k| !held[k]) {
      let (e, unit) = inks[k];
      let mut at: Vec<((usize, usize), &str)> = reads[k].lost.clone();
      for (range, word) in outside(k) {
        let word_lc = word.to_lowercase();
        let written = if a_block {
          take(&mut counts, &word_lc)
        } else {
          self.anywhere.contains(&word_lc)
        };
        if !written {
          at.push((range, word));
        }
      }
      if at.is_empty() {
        continue;
      }
      at.sort_unstable_by_key(|(range, _)| *range);
      // (its lost words of letters outside a word printing an address, as the address reader reads one: what may be a name's
      // — `\texttt{\{firstname\}.\{lastname\}@kit.edu}`'s are an address's, 2601.00680)
      let address_words: Vec<(usize, usize)> = e
        .text
        .split_whitespace()
        .filter(|w| !addresses_in(w).is_empty() || reads[k].addresses.iter().any(|a| a == w))
        .map(|w| {
          let start = w.as_ptr() as usize - e.text.as_ptr() as usize;
          (start, start + w.len())
        })
        .collect();
      let named: Vec<&str> = at
        .iter()
        .filter(|((from, to), w)| {
          w.chars().any(char::is_alphabetic)
            && !address_words.iter().any(|&(s, t)| s <= *from && *to <= t)
        })
        .map(|(_, w)| *w)
        .collect();
      // (a Name's: what its addresses may spell as its own)
      let own =
        (a_block && e.kind == EventKind::Name && !named.is_empty()).then(|| named.join(" "));
      // (its owner by its own addresses, else by the addresses its line prints, read from the line's text whole: the line goes
      // with its other inks to the author its own address gives, `ext. 5` after `POC: A. Able (ann.able@a.org)`, 2203.10777's
      // `Dresden`, `Saxony 01062` before `Germany (martin.waltz@tu-dresden.de)`, review revN1r2; a list's local parts printed
      // apart are one run of addresses spelling its authors, owning none, `{pzou94, ozel, suresh}@gwu.edu`, 2003.14069;
      // 2102.08981, 1901.01753, 1708.00818, 2307.03645)
      let spelled = if !a_block {
        None
      } else if reads[k].addresses.is_empty() {
        let line: Vec<&Event> = events
          .iter()
          .zip(&units)
          .filter(|&(_, &u)| u == unit)
          .map(|(e, _)| e)
          .collect();
        let runs = address_runs(&events_text(&line), &|_| false);
        one_holder(
          &mut runs.iter().flat_map(|run| run.addresses.iter()),
          own.as_deref(),
        )
      } else {
        one_holder(&mut reads[k].addresses.iter(), own.as_deref())
      };
      let owner = match spelled {
        Some(Holder::Author(creator)) => Some(creator),
        _ => None,
      };
      // (the addresses its ink prints, else its line's)
      let addresses: Vec<&String> = if !a_block {
        Vec::new()
      } else if reads[k].addresses.is_empty() {
        let line = by_unit.get(&unit).into_iter().flatten();
        line.flat_map(|&j| reads[j].addresses.iter()).collect()
      } else {
        reads[k].addresses.iter().collect()
      };
      // (a name: on a line of its block's names, or one an address its line or its column prints spells as its own, by the
      // reading's rule of an address, ranked against the written names: a column's name above its address, 1805.08288's
      // minipages; an address of a written author's spells that author's line, whatever else it prints: `Contact: A. Hobbs,
      // ann.hobbs@a.org` under Ann Hobbs)
      let names_fail = own.as_deref().is_some_and(|own| {
        owner.is_none()
          && (names_line(unit) || {
            let column = by_column.get(&column_of[&unit]).into_iter().flatten();
            let mut all = addresses
              .iter()
              .copied()
              .chain(column.flat_map(|&j| reads[j].addresses.iter()));
            all.any(|address| holder(address, Some(own)) == Some(Holder::Own))
          })
      });
      lost.push(LostInk {
        span: e.span,
        text: e.text.clone(),
        words: at.iter().map(|(_, w)| (*w).to_string()).collect(),
        lost_at: at.iter().map(|(range, _)| *range).collect(),
        cut: reads[k].cut.clone(),
        owner,
        names_fail,
        font: e.font.clone(),
      });
    }
    lost
  }
}

/// Whom an address spells among a block's written names and a lost one ([`Written::lost_inks`]): a written author (its
/// index in [`Resolved::creators`]), or the lost name, its own.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Holder {
  Author(usize),
  Own,
}

/// One block's word check ([`check_blocks`]): its inks the reading writes nowhere, and the words it writes more times than
/// they print.
#[derive(Debug)]
pub(in crate::base_utilities::frontmatter::observer) struct BlockCheck {
  pub lost:     Vec<LostInk>,
  pub excess:   Vec<(String, bool)>,
  /// how many of its pictures no written item holds ([`Written::unwritten_pictures`])
  pub pictures: usize,
}

/// The author blocks of the ledger whose authors an output holds, with their ledger index: what a later call replaced or
/// cleared, as the kernel's `\lx@clear@frontmatter` does, prints in no output (a template's `\author{First Author}` before
/// the paper's own, 2412.15554).
pub(in crate::base_utilities::frontmatter::observer) fn live_blocks(
  ledger: &Ledger,
) -> Vec<(usize, &AuthorBlock)> {
  // (per entry, whether a later block of its epoch replaces the authors: the epochs a replacing block opens, from the end)
  let mut replacing: BTreeSet<usize> = BTreeSet::new();
  let mut replaced = vec![false; ledger.entries().len()];
  for (index, entry) in ledger.entries().iter().enumerate().rev() {
    if let Entry::Author(block) = entry {
      replaced[index] = replacing.contains(&block.epoch);
      if block.replace {
        replacing.insert(block.epoch);
      }
    }
  }
  let blocks = ledger.entries().iter().enumerate();
  let live = blocks.filter_map(|(index, entry)| match entry {
    Entry::Author(block) if !block.cleared && !replaced[index] => Some((index, &**block)),
    _ => None,
  });
  live.collect()
}

/// The block half of the word check, both ways: per author block (its ledger index) that fails it, its [`BlockCheck`],
/// against `written`, the reading's ([`Written::new`]), over the [`live_blocks`] (2412.15554's template block none).
pub(in crate::base_utilities::frontmatter::observer) fn check_blocks(
  ledger: &Ledger,
  written: &Written,
) -> BTreeMap<usize, BlockCheck> {
  let mut checks = BTreeMap::new();
  for (index, block) in live_blocks(ledger) {
    if let Outcome::Events { events, .. } = &block.outcome {
      let pictures = written.unwritten_pictures(index, events);
      // (a command's ink inside a word its letters, as the reading reads them: `lines::words_through_one_command`)
      let events = &words_through_one_command(events);
      let check = BlockCheck {
        lost: written.lost_inks(index, events, true),
        excess: written.excess(index, events),
        pictures,
      };
      if !check.lost.is_empty() || !check.excess.is_empty() || check.pictures > 0 {
        checks.insert(index, check);
      }
    }
  }
  checks
}
