//! The emitter and its vocabulary: the one added step at the flush ([`emit`], OXIDIZED_DESIGN_DIVERGENCES #980), which
//! writes Perl's own API calls ([`calls`]) from the observer's reading; the token slices it cuts a call's content into
//! (`plan`, `repart`, `content`, `address_cut`); the reading of the placed creators ([`dom`]) and the content check on
//! them ([`check_placed_content`]). The S2b writer that once lived here (OD #837 "S2b": `write_observed_frontmatter`,
//! which re-wrote a placed `written` family's frontmatter by DOM operations after `insert_frontmatter`) and the census's
//! read-back were deleted with the census (DEL4): since DEL3 the emitter reads every block a rehearsal read, and a block
//! none read is Perl's whole-block creator, so the writer had no paper left (0 of the 1,434 stage2 papers and the E0
//! population changed without it).

use std::collections::BTreeSet;

use super::{
  events::{Event, Formula},
  ledger::{Entry, Ledger, Outcome, SkipReason, with_ledger},
  resolver::{
    Assoc, Placed, PlacedItem, PlacedKind, RCreator, Resolved, content_check, ledger_inputs,
    mark_labels, resolve,
  },
  *,
};

mod address_cut;
// (the emitter's vocabulary, moved unchanged from the old reader by the plan's row "Knowledge in, driver out, one pass"; re-exported by `observer/mod.rs`)
pub(in crate::base_utilities) mod calls;
mod content;
mod dom;
mod emit;
mod plan;
mod repart;

// (the writer's cut of a block's entry, which the reading checks an in-block contact's entries by: `events::labelled_entries`)
pub(super) use content::{Field, aligned};
pub(in crate::base_utilities) use emit::{
  Analysis, EMITTING, NAMES_SHOWN_LABELS, SETTERS_RECOVERED, block_creator_written, emit_commands,
  emitted, names_show_label, setters_recovered, take_digested,
};
pub(super) use plan::{entry_forms, labelled_start, span_tokens};

/// The content invariant's check on the placed output (`resolver::placed`; the safety net's bar (2)), the last step of
/// `insert_frontmatter`: the words the ledger's author blocks and creator calls printed
/// against the placed creators' (their names, contacts and notes, as [`dom::read`] reads them; a publication note's, a title's
/// note's and a contact's label holding a word too), each finding counted in one Warning per paper naming its words — a printed word or
/// address the output lacks (1605.03019's `IDSIA`), one placed more often than printed (2607.00684's `mtablo@postech.ac.kr`
/// twice, 2506.02663's footnote junk). It counts; it changes nothing.
pub(in crate::base_utilities) fn check_placed_content(document: &mut Document) {
  if rehearsal::rehearsing() {
    return;
  }
  let front = dom::read(document);
  let mut placed = Placed::default();
  // (a link's target, which an ink may print: an ORCID's id behind its logo, 2607.00684; each target once, a shared
  // entry's link under every holder one print)
  let mut targets = BTreeSet::new();
  for (holder, creator) in front.creators.iter().chain(&front.nameless).enumerate() {
    let mut links = Vec::new();
    dom::descendants(&creator.node, "ltx:ref", &mut links);
    targets.extend(links.iter().filter_map(|link| link.get_attribute("href")));
    if !creator.name.trim().is_empty() {
      // (its words parted at every layout part, as the block's lines print them: a block written whole, 1908.02923; read again
      // only where a layout part holds them)
      let names = creator.node.get_child_nodes().into_iter();
      let names: Vec<Node> = names
        .filter(|child| dom::is(child, "ltx:personname"))
        .collect();
      let text = if names.iter().any(dom::holds_a_layout_part) {
        let words: Vec<String> = names.iter().map(dom::words_without_marks).collect();
        words.join(" ")
      } else {
        creator.name.clone()
      };
      placed.items.push(PlacedItem {
        holder,
        kind: PlacedKind::Name,
        text,
        label: None,
      });
    }
    // (what a personname holds besides its notes, an ORCID link, its name's text holds)
    for item in creator
      .items
      .iter()
      .filter(|item| item.movable || item.kind == dom::Kind::Note)
    {
      // (one shared item is one kind's under its text: a note on one creator and an affiliation on another printing the same
      // words are two, each printed)
      let kind = match &item.kind {
        dom::Kind::Contact(role) if role == "email" => PlacedKind::Email(item.key.clone()),
        kind => PlacedKind::Item(s!("{kind:?}|{}", item.key)),
      };
      placed.items.push(PlacedItem {
        holder,
        kind,
        text: item.text.clone(),
        label: item.node.get_attribute("name"),
      });
    }
  }
  placed.beside.extend(targets);
  // (a creator call's note relocated to the title, elsarticle's `\tnotetext[t1]{…}` labelled `tnote:t1`, 1611.08809)
  for note in document.findnodes("//ltx:pubnote | //ltx:title//ltx:note", None) {
    placed.beside.push(dom::text_without_marks(&note));
  }
  // (each publication note as written: one written more often than it printed is counted, review rck2)
  placed.notes = document
    .findnodes("//ltx:pubnote", None)
    .iter()
    .map(dom::text_without_marks)
    .collect();
  placed.dropped = emit::reading_drops();
  let check = with_ledger(|ledger| content_check(ledger, &placed, &emit::kept_whole));
  for (words, what) in [
    (
      &check.lost,
      "words the author blocks print are missing from the placed frontmatter",
    ),
    (
      &check.excess,
      "words or addresses appear in the placed frontmatter more often than the author blocks print them",
    ),
  ] {
    if !words.is_empty() {
      Warn!(
        "unexpected",
        "frontmatter",
        s!(
          "Frontmatter content check: {} {what}: {}.",
          words.len(),
          words.join(" ")
        )
      );
    }
  }
  // (a note printing no word, which the word check can't see: written nowhere, `emit::unwritten_notes`)
  let notes = emit::unwritten_notes();
  if !notes.is_empty() {
    Warn!(
      "unexpected",
      "frontmatter",
      s!(
        "Frontmatter content check: {} notes the author blocks print with no word are missing from the placed frontmatter: {}.",
        notes.len(),
        notes.join(" ")
      )
    );
  }
}

/// The entry `taken` given back in the reading `out`: its addresses, taken out, no contacts of its holders.
pub(super) fn give_back_entry(out: &mut Resolved, taken: &resolver::Taken) {
  for creator in &mut out.creators {
    creator
      .contacts
      .retain(|a| !(a.source == taken.source && a.rule.contains("owner:entry-address")));
  }
  for (holder, at, assoc) in &taken.holders {
    let slots = match holder {
      Some(c) => &mut out.creators[*c].affiliations,
      None => &mut out.block,
    };
    match slots.iter().position(|a| {
      a.source == taken.source
        && a.text == taken.stripped
        && a.rule.ends_with(" ruling3:address-out")
    }) {
      Some(i) => slots[i] = assoc.clone(),
      None => slots.insert((*at).min(slots.len()), assoc.clone()),
    }
  }
}
