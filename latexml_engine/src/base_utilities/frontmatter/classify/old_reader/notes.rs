//! The IEEEtran binding's bare run of notes as the document's (`document_notes`, `ieeetran_cls.rs`). Moved unchanged by Z18m, which deleted `author_block/`, from `author_block/notes.rs`.

use super::*;

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
