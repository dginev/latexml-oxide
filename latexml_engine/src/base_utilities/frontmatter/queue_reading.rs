//! What the frontmatter queue's names ask of their marks, read once per entry and kept with the queue.
//!
//! Every contact of an author block asks what the queued names request, so reading the names again at each call made the
//! block's cost the product of its names and its contacts: 28 % of 2103.04797's instructions (975 authors, the Cut,
//! `affiliations::cut_marked_contact`) and 56 % of 1901.01540's (1,202, AASTeX's `\affiliation` and the Cut); 2005.08745's
//! 592 are labelled, so the Cut leaves them at once. The reading is a value of the state beside the queue, so an
//! assignment journal rolls both back together (a rehearsal) and it goes with the state; an entry is read once, when a
//! reader first finds it queued. Perl has no counterpart, as it reads no names at a contact: this is only a cache of what
//! the Cut's and the marks' readers would read from the queue, so it never changes an answer.

use std::rc::Rc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::*;

/// The state key of the queue's [`QueueReading`].
const QUEUE_READING: &str = "frontmatter_raw_reading";

/// The readings of the first `read` entries of the frontmatter queue. A name's marks are told by the meaning of its
/// commands when they are read (`\let\up\textsuperscript` after `\author{Ann\up{1}}`), so the reading also holds what
/// those commands meant, and is read again in full when one means otherwise ([`QueueReading::meanings_hold`]).
#[derive(Clone, Default)]
struct QueueReading {
  read:         usize,
  /// The entries read so far, a reading again counted anew (the block's count in the `--debug=frontmatter` log, which
  /// `perfect_kernel_batchc1` pins).
  entries_read: usize,
  author_marks: Rc<Vec<Vec<Vec<Token>>>>,
  name_marks:   Rc<(Vec<String>, Vec<String>)>,
  /// The queue positions (from 1) of the authors requesting each label, ascending.
  requests:     FxHashMap<String, Vec<usize>>,
  /// The authors' commands, each once, and whether it means a mark (`is_mark_command`).
  commands:     Vec<(Token, bool)>,
  /// The commands in the authors' mark operands, each once, and whether it is a symbol mark (`is_symbol_mark`).
  symbols:      Vec<(Token, bool)>,
  seen:         FxHashSet<(Catcode, SymStr)>,
  seen_symbols: FxHashSet<SymStr>,
}

impl QueueReading {
  /// Does every command of the names still mean what it did when they were read?
  fn meanings_hold(&self) -> bool {
    self
      .commands
      .iter()
      .all(|(t, mark)| is_mark_command(t) == *mark)
      && self
        .symbols
        .iter()
        .all(|(t, symbol)| is_symbol_mark(std::slice::from_ref(t)) == *symbol)
  }

  fn read_entries(&mut self, entries: &[RawFrontmatter]) {
    self.entries_read += entries.len();
    for (tag, attr, command) in entries {
      if tag != "ltx:creator" {
        continue;
      }
      let (altaffil, sup) = name_marks_of(command);
      let marks = Rc::make_mut(&mut self.name_marks);
      marks.0.extend(altaffil);
      marks.1.extend(sup);
      if attr.get("role").map(String::as_str) != Some("author") {
        continue;
      }
      for t in command.unlist_ref() {
        if matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)
          && self.seen.insert((t.get_catcode(), t.get_sym()))
        {
          self.commands.push((*t, is_mark_command(t)));
        }
      }
      let operands = author_mark_operands(command.unlist_ref());
      self.read_requests(&operands);
      Rc::make_mut(&mut self.author_marks).push(operands);
    }
  }

  /// The labels the next author's mark `operands` request, as `requesting_authors` reads them.
  fn read_requests(&mut self, operands: &[Vec<Token>]) {
    let position = self.author_marks.len() + 1;
    for operand in operands {
      for t in operand {
        if t.get_catcode() == Catcode::CS && self.seen_symbols.insert(t.get_sym()) {
          self
            .symbols
            .push((*t, is_symbol_mark(std::slice::from_ref(t))));
        }
      }
      let spellings = std::iter::once(operand.clone()).chain(label_pieces(operand));
      for spelling in spellings {
        for label in clean_frontmatter_labels(&Tokens::new(spelling).to_string(), "affiliation") {
          let requesters = self.requests.entry(label).or_default();
          if requesters.last() != Some(&position) {
            requesters.push(position);
          }
        }
      }
    }
  }
}

/// The reading kept in the state, if any.
fn kept_queue_reading() -> Option<Rc<QueueReading>> {
  with_value(QUEUE_READING, |v| match v {
    Some(Stored::Opaque(reading)) => Rc::clone(reading).downcast::<QueueReading>().ok(),
    _ => None,
  })
}

/// The queue's reading, its new entries read.
fn queue_reading() -> Rc<QueueReading> {
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.len(),
    _ => 0,
  });
  let kept = kept_queue_reading();
  let holds = kept.as_ref().is_some_and(|kept| kept.meanings_hold());
  if holds && let Some(kept) = kept.as_ref().filter(|kept| kept.read == queued) {
    return Rc::clone(kept);
  }
  // (given up by the state, so that it is extended in place unless a journal holds it too)
  remove_value(QUEUE_READING);
  let mut reading = QueueReading::default();
  if let Some(kept) = kept {
    if holds && kept.read <= queued {
      reading = Rc::unwrap_or_clone(kept);
    } else {
      reading.entries_read = kept.entries_read;
    }
  }
  let from = reading.read;
  with_value("frontmatter_raw", |v| {
    if let Some(Stored::FrontmatterRaw(queue)) = v {
      reading.read_entries(&queue[from..]);
    }
  });
  reading.read = queued;
  let reading = Rc::new(reading);
  if queued > 0 {
    assign_value(
      QUEUE_READING,
      Stored::Opaque(Rc::clone(&reading) as Rc<dyn std::any::Any>),
      Some(Scope::Global),
    );
  }
  reading
}

/// The queue lost entries (a flush took it, `dequeue_front_matter` removed some): what was read of it is no more.
pub(in crate::base_utilities) fn forget_queue_reading() {
  if let Some(reading) = kept_queue_reading() {
    DebugFeature!(
      "frontmatter",
      "FRONT the queue's reading: {} entries read for {} queued",
      reading.entries_read,
      reading.read
    );
    remove_value(QUEUE_READING);
  }
}

/// The mark operands of each queued author, in queue order: the n-th digests with the label `author:n`
/// (`num_ltx:creator{author}`), by which an address goes to it — as the count starts from no author, which holds
/// unless authors an earlier flush digested are still kept (an appending `\author` after `\maketitle`).
pub(in crate::base_utilities) fn queued_author_marks() -> Rc<Vec<Vec<Vec<Token>>>> {
  Rc::clone(&queue_reading().author_marks)
}

/// The queue positions of the authors whose marks request the label a mark's `operand` sets, as `requesting_authors`
/// reads them.
pub(in crate::base_utilities) fn queued_requesters(operand: &[Token]) -> Vec<usize> {
  let Some(label) = mark_label(operand) else {
    return Vec::new();
  };
  queue_reading()
    .requests
    .get(&label)
    .cloned()
    .unwrap_or_default()
}

/// The marks the queued authors' names show, as label keys (`relocate::name_marks_of`): the items of AASTeX 5's
/// `\altaffilmark{…}` (which request `altaffil:` labels), and those of their superscripts (`affiliation:` labels).
pub fn queued_name_marks() -> Rc<(Vec<String>, Vec<String>)> {
  Rc::clone(&queue_reading().name_marks)
}
