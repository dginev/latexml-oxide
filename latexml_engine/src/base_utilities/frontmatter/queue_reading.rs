//! What the frontmatter queue's names ask of their marks, read once per entry and kept with the queue.
//!
//! Every contact of an author block asks what the queued names request, so reading the names again at each call made the
//! block's cost the product of its names and its contacts: 28 % of 2103.04797's instructions (975 authors, the Cut,
//! `frontmatter_api::cut::cut_marked_contact`) and 56 % of 1901.01540's (1,202, AASTeX's `\affiliation` and the Cut); 2005.08745's
//! 592 are labelled, so the Cut leaves them at once. The reading is a value of the state beside the queue, so an
//! assignment journal rolls both back together (a rehearsal) and it goes with the state; an entry is read once, when a
//! reader first finds it queued. Perl has no counterpart, as it reads no names at a contact. Since Option C F0 an author's
//! marks are its name's digested ones (`argument_marks`), keyed by printed text (OXIDIZED_DESIGN_DIVERGENCES #995).

use std::rc::Rc;

use rustc_hash::FxHashMap;

use super::*;

/// The state key of the queue's [`QueueReading`].
const QUEUE_READING: &str = "frontmatter_raw_reading";

/// The readings of the first `read` entries of the frontmatter queue: their name marks by tokens, and the marks of the
/// authors among the first `marks_read`, one marks query per queued author ([`author_marks`]).
#[derive(Clone, Default)]
struct QueueReading {
  read:         usize,
  /// The entries read so far, a reading again counted anew (the block's count in the `--debug=frontmatter` log, which
  /// `perfect_kernel_batchc1` pins).
  entries_read: usize,
  name_marks:   Rc<(Vec<String>, Vec<String>)>,
  marks_read:   usize,
  author_marks: Rc<Vec<Vec<String>>>,
  /// The queue positions (from 1) of the authors requesting each label, ascending.
  requests:     FxHashMap<String, Vec<usize>>,
}

impl QueueReading {
  // (the names' marks by tokens, `name_marks_of`: token reading until P4c (F0 report))
  fn read_names(&mut self, entries: &[RawFrontmatter]) {
    self.entries_read += entries.len();
    for (tag, _, command) in entries {
      if tag == "ltx:creator" {
        let (altaffil, sup) = name_marks_of(command);
        let marks = Rc::make_mut(&mut self.name_marks);
        marks.0.extend(altaffil);
        marks.1.extend(sup);
      }
    }
  }

  fn read_marks(&mut self, entries: &[RawFrontmatter]) -> Result<()> {
    for (tag, attr, command) in entries {
      if tag != "ltx:creator" || attr.get("role").map(String::as_str) != Some("author") {
        continue;
      }
      let (texts, keys) = author_marks(command)?;
      let position = self.author_marks.len() + 1;
      for label in keys {
        let requesters = self.requests.entry(label).or_default();
        if requesters.last() != Some(&position) {
          requesters.push(position);
        }
      }
      Rc::make_mut(&mut self.author_marks).push(texts);
    }
    Ok(())
  }
}

/// The marks a queued author `entry`'s name ([`queued_name`]) prints at its own level outside its notes, by their printed
/// text (`argument_marks`: a document macro's mark too, `\newcommand\ua{\textsuperscript{1}}`), and the labels they
/// request, as `\lx@sup@request@affiliation` keys them (`printed_label_keys`); a `\footnotemark` requests a note, none.
/// Where the query cannot read the name, the entry's mark operands by spelling (`author_mark_operands`), counted.
fn author_marks(entry: &Tokens) -> Result<(Vec<String>, Vec<String>)> {
  let mut texts = Vec::new();
  let mut keys = Vec::new();
  if let Some(marks) = argument_marks(&queued_name(entry))? {
    for mark in marks {
      if mark.role.as_deref() == Some("footnotemark") {
        continue;
      }
      // (a cross-reference's key as `printed_label` reads it, `\ref{key}`: `events::script_mark`'s `ref:key`)
      let text = match mark.label {
        Some(label) if mark.role.as_deref().is_some_and(|r| r.ends_with(" ref")) => label
          .split(',')
          .map(|piece| match piece.trim().strip_prefix("ref:") {
            Some(key) => s!("\\ref{{{key}}}"),
            None => piece.to_string(),
          })
          .collect::<Vec<_>>()
          .join(","),
        Some(label) => label,
        // (a mark printing no label requests none: counted, as an unread name is)
        None => {
          count_unread_marks("queued_author_marks, a mark without a label");
          continue;
        },
      };
      keys.extend(printed_label_keys(&text, "affiliation"));
      texts.push(text);
    }
    return Ok((texts, keys));
  }
  count_unread_marks("queued_author_marks");
  for operand in author_mark_operands(entry.unlist_ref()) {
    let spellings = std::iter::once(operand.clone()).chain(label_pieces(&operand));
    for spelling in spellings {
      keys.extend(clean_frontmatter_labels(
        &Tokens::new(spelling).to_string(),
        "affiliation",
      ));
    }
    texts.push(Tokens::new(operand).to_string());
  }
  Ok((texts, keys))
}

/// The reading kept in the state, if any.
fn kept_queue_reading() -> Option<Rc<QueueReading>> {
  with_value(QUEUE_READING, |v| match v {
    Some(Stored::Opaque(reading)) => Rc::clone(reading).downcast::<QueueReading>().ok(),
    _ => None,
  })
}

/// The queued entries from `from` on.
fn queued_from(from: usize) -> Vec<RawFrontmatter> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.get(from..).unwrap_or_default().to_vec(),
    _ => Vec::new(),
  })
}

/// Keep `reading` in the state, beside a non-empty queue.
fn keep(reading: QueueReading) -> Rc<QueueReading> {
  let reading = Rc::new(reading);
  if reading.read > 0 {
    assign_value(
      QUEUE_READING,
      Stored::Opaque(Rc::clone(&reading) as Rc<dyn std::any::Any>),
      Some(Scope::Global),
    );
  }
  reading
}

/// The queue's reading, its new entries' name marks read.
fn queue_reading() -> Rc<QueueReading> {
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.len(),
    _ => 0,
  });
  let kept = kept_queue_reading();
  if let Some(kept) = kept.as_ref().filter(|kept| kept.read == queued) {
    return Rc::clone(kept);
  }
  // (given up by the state, so that it is extended in place unless a journal holds it too)
  remove_value(QUEUE_READING);
  let mut reading = QueueReading::default();
  if let Some(kept) = kept {
    if kept.read <= queued {
      reading = Rc::unwrap_or_clone(kept);
    } else {
      reading.entries_read = kept.entries_read;
    }
  }
  reading.read_names(&queued_from(reading.read));
  reading.read = queued;
  keep(reading)
}

/// The queue's reading, its authors' marks read too.
fn queue_marks_reading() -> Result<Rc<QueueReading>> {
  let reading = queue_reading();
  if reading.marks_read == reading.read {
    return Ok(reading);
  }
  remove_value(QUEUE_READING);
  let mut reading = Rc::unwrap_or_clone(reading);
  // (the entries taken off the queue first: a marks query digests)
  reading.read_marks(&queued_from(reading.marks_read)[..reading.read - reading.marks_read])?;
  reading.marks_read = reading.read;
  Ok(keep(reading))
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

/// The marks of each queued author, by their printed text ([`author_marks`]), in queue order: the n-th digests with
/// the label `author:n` (`num_ltx:creator{author}`), by which an address goes to it — as the count starts from no
/// author, which holds unless authors an earlier flush digested are still kept (an appending `\author` after `\maketitle`).
pub(in crate::base_utilities) fn queued_author_marks() -> Result<Rc<Vec<Vec<String>>>> {
  Ok(Rc::clone(&queue_marks_reading()?.author_marks))
}

/// Whether a queued author's name prints a mark ([`queued_author_marks`]).
pub(in crate::base_utilities) fn queued_authors_marked() -> Result<bool> {
  Ok(queued_author_marks()?.iter().any(|marks| !marks.is_empty()))
}

/// The queue positions of the authors whose marks request `label` (`affiliation:<key>`).
pub(in crate::base_utilities) fn queued_label_requesters(label: &str) -> Result<Vec<usize>> {
  Ok(
    queue_marks_reading()?
      .requests
      .get(label)
      .cloned()
      .unwrap_or_default(),
  )
}

/// The queue positions of the authors whose marks request the label a mark's `operand` sets, by its printed text as
/// `\lx@sup@setlabel@affiliation` sets it ([`set_label`]).
pub(in crate::base_utilities) fn queued_requesters(operand: &[Token]) -> Result<Vec<usize>> {
  let operand = Tokens::new(operand.to_vec());
  match set_label(&operand, printed_label(&operand)?.as_ref()) {
    Some(label) => queued_label_requesters(&label),
    None => Ok(Vec::new()),
  }
}

/// The affiliation labels the queued authors' marks request (`affiliation:<key>`), or, once a flush took the queue, those of
/// the authors it took ([`flush_requested_marks`]): what tells a script glued to a word a label (`events::names_requested`).
pub(in crate::base_utilities) fn requested_labels() -> Vec<String> {
  let queued: Vec<String> = queue_reading().requests.keys().cloned().collect();
  if !queued.is_empty() {
    return queued;
  }
  flush_requested_marks()
}

/// The marks the queued authors' names show, as label keys (`relocate::name_marks_of`): the items of AASTeX 5's
/// `\altaffilmark{…}` (which request `altaffil:` labels), and those of their superscripts (`affiliation:` labels).
pub fn queued_name_marks() -> Rc<(Vec<String>, Vec<String>)> {
  Rc::clone(&queue_reading().name_marks)
}

/// Whether the queued authors' names (`queued_name`) print every word `text` prints — each word one of a name's, or
/// an initial of one, accents aside (`visible_name_text`), or the word "and" between names: `text` then prints no
/// word the authors do not. True for a `text` printing no word; else false when no author is queued. svjour3's
/// `\institute{F. Author \at …}` piece names its authors before the address (`sv_support_sty.rs`): a piece writing
/// only their names adds nothing to the XML (`B. Baker, and C. Cole \at …`, batch 63's piece naming two), one writing
/// more (`\textit{Contact author:} Erwan {Le Merrer}`, 1711.01894; `Xin-long Luo, Corresponding author`, 2107.13864;
/// the whole name `Leonardo Anjoletto Ferreira` of `Leonardo A. Ferreira`, 1705.01399) prints words no name does.
pub fn queued_author_name_prints(text: &Tokens) -> bool {
  let written = printed_words(text);
  if written.is_empty() {
    return true;
  }
  let names: Vec<String> = queued_author_name_words().into_iter().flatten().collect();
  !names.is_empty()
    && written.iter().all(|w| {
      w == "and"
        || names
          .iter()
          .any(|o| o == w || (w.chars().count() == 1 && o.starts_with(w.as_str())))
    })
}

/// How many queued authors' names (`queued_name`) `text` spells whole: each author whose name's words (as
/// [`queued_author_name_prints`] reads them) stand together, in order, among the words `text` prints. svjour3's
/// `\institute{\begin{itemize}\item[\Letter] Ann Able \\ \email{…} \item[] Bob Baker \\ \email{…} \at \item[1]
/// Univ X …}` names its list's authors before the one `\at` (2312.14898): two or more names it no one by. A whole author
/// list queued as one entry (sv_support's `\author{A \and B}`, one `\lx@add@author@split` call the reading cuts) is
/// parted at its separators first (`entry_names`), so each of its names is counted as a name of its own.
pub fn queued_author_names_spelled(text: &Tokens) -> Result<usize> {
  let written = printed_words(text);
  let mut spelled = 0;
  for entry in queued_author_names() {
    for name in entry_names(&entry)? {
      let name = printed_words(&name);
      if !name.is_empty()
        && written
          .windows(name.len())
          .any(|run| run == name.as_slice())
      {
        spelled += 1;
      }
    }
  }
  Ok(spelled)
}

/// The names one queued author entry holds: the entry parted where the reading cuts its names, at each `\and`-family
/// separator not inside a layout container (a brace group is none) the rehearsal reads ([`argument_separators`],
/// OXIDIZED_DESIGN_DIVERGENCES #995), at its own tokens (`ArgSeparator::span`: a literal, a `\let` copy, an active
/// character, a macro holding only the separator, the document's macro `\and`; never an `\end{tabular}` glued before it,
/// which is the table's), whatever it means where the query is made: the rehearsal binds `\and` as the separator whatever
/// it means (`rehearsal::ALWAYS_BOUND`), so the reading writes a creator per piece there, and svjour3, the one class whose
/// binding asks, sets its own `\and` in its `\@maketitle` before the names print (svjour3.cls:608; svjour, svmult and
/// svproc likewise), so a document's `\renewcommand\and` before it changes no part. A separator whose tokens print ink
/// too (a macro holding names and the separator, `\newcommand\aut{Ann \and Bob}`) has no span: no part there. An entry
/// the query could not read (`None`: no rehearsal made, over its allowance, a group left open) is one name, as it is with
/// no separator: unread is not "no separator", and keeping the entry whole is the reading before any part was known.
fn entry_names(name: &Tokens) -> Result<Vec<Tokens>> {
  let Some(separators) = argument_separators(name)? else {
    return Ok(vec![name.clone()]);
  };
  let content = name.unlist_ref();
  let mut names = Vec::new();
  let mut from = 0;
  for separator in separators {
    if let Some((start, end)) = separator.span
      && let Some(name) = content.get(from..start)
    {
      names.push(Tokens::new(name.to_vec()));
      from = end;
    }
  }
  names.push(Tokens::new(
    content.get(from..).unwrap_or_default().to_vec(),
  ));
  Ok(names)
}

/// The words `tokens` print as a name-shape test reads them ([`visible_name_text`]), accents aside, lowercased.
fn printed_words(tokens: &Tokens) -> Vec<String> {
  resolver::base_letters(&visible_name_text(tokens.unlist_ref()))
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect()
}

/// Each queued author's name ([`queued_name`]) as its [`printed_words`], in queue order.
fn queued_author_name_words() -> Vec<Vec<String>> {
  queued_author_names().iter().map(printed_words).collect()
}

/// Each queued author entry's name ([`queued_name`]), in queue order.
fn queued_author_names() -> Vec<Tokens> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .map(|entry| queued_name(&entry.2))
      .collect(),
    _ => Vec::new(),
  })
}
