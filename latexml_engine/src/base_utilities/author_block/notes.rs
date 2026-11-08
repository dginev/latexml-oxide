//! Author notes and thanks: affiliation-mark splits, leading declarations, name annotations.

use super::*;

/// Whether an author block's `line` holds authors' notes only: one or more `\thanks{…}`, `\footnote{…}` or
/// `\IEEEcompsocitemizethanks{…}` (each with an optional `[…]`), and else only spacing (spaces, `~`, `\par`). Such a
/// line prints no text where it stands, only its notes' marks: the notes are the line's above, as written without the
/// `\\` before them (`author_block/preprocess.rs` `notes_line_joins_names`; the IEEE reader's grade-and-notes line,
/// `grade_only_line`).
pub fn notes_only_line(line: &[Token]) -> bool {
  line_notes(line).is_some_and(|notes| !notes.is_empty())
}

/// A line of notes and of parts printing nothing ([`prints_nothing`]: `\thanks{…}\vspace{2mm}`) as the document's notes,
/// as an author block digests them in a contact (`document_note_call`; a note no pubnote takes as written), the rest
/// left out: the IEEEtran binding's bare run of notes between the names and their shared block
/// (`\IEEEauthorblockN{…}\thanks{…}\IEEEauthorblockA{…}`, 1508.00691, ieeetran_cls.rs `wrap_bare_author_block_text`).
/// `None` for a line holding other ink.
pub fn document_notes(line: &[Token]) -> Option<Vec<Token>> {
  let mut out = Vec::with_capacity(line.len());
  let mut rest = Vec::new();
  let mut i = 0;
  while i < line.len() {
    match note_call_end(line, i) {
      Some(end) => {
        let note = &line[i..end];
        match document_note_call(note) {
          Some(call) => out.extend(call),
          None => out.extend_from_slice(note),
        }
        i = end;
      },
      None => {
        rest.push(line[i]);
        i += 1;
      },
    }
  }
  prints_nothing(&Tokens::new(rest)).then_some(out)
}

/// The notes of a line of notes only ([`notes_only_line`]), each the span `(start, end)` of `line` from its command
/// through its closing brace; `None` when the line holds other ink.
pub(in crate::base_utilities) fn line_notes(line: &[Token]) -> Option<Vec<(usize, usize)>> {
  let spacing = |t: &Token| *t == T_SPACE!() || *t == T_ACTIVE!('~') || *t == T_CS!("\\par");
  let mut notes = Vec::new();
  let mut i = 0;
  while i < line.len() {
    if spacing(&line[i]) {
      i += 1;
    } else {
      let end = note_call_end(line, i)?;
      notes.push((i, end));
      i = end;
    }
  }
  Some(notes)
}
