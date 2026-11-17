//! aaai.sty and its years (aaai16-aaai2027, the versioned fallback strips each to `aaai`): the paper's copy read raw, the
//! marks its `\maketitle` sets declared, the author notes only its `\@maketitle` defines (2501.02471, 2511.07192,
//! 2103.05277, 2111.06599, 2006.08767, 2609.00420; OD #1053).
use std::cell::{Cell, RefCell};

use latexml_core::definition::origin::{DefinitionOrigin, with_origin};
use latexml_package::{
  engine::{base_utilities::rehearsal_footnote_value, latex_constructs::title_note_number},
  prelude::*,
  shipped_class::{FNSYMBOL_TITLE_MARKS, input_shipped_package_title_marks},
};

LoadDefinitions!({
  // As AAAI's: each prints its footnote once, "These authors contributed equally." and "Corresponding author.", the
  // first use making the note, a later one its `\footnotemark[<n>]` (2501.02471, 2508.07026, 2511.17927, 2511.15266,
  // 2511.15807; `aaai_note`, OD #1053); in a document where a use is followed by ".", each use its own note
  // (2509.03329). As stubs (`DefinitionOrigin::Stub`): the document's or a raw class's own `\newcommand` still defines
  // them (copernicus.cls:1673 `\newcommand\equalcontrib[1]`, its equal-contribution note on the affiliations).
  AAAI_FIRST_NOTES.with(|notes| notes.borrow_mut().clear());
  AAAI_DOTTED.set(false);
  DefMacro!("\\lx@aaai@note{}{}", sub[(key, text)] { aaai_note(&key.to_string(), text) });
  DefMacro!("\\lx@aaai@noted{}{}{}", sub[(key, field, before)] {
    let number = |tokens: &Tokens| tokens.to_string().trim().parse::<i64>().unwrap_or(0);
    let (field, before) = (number(&field), number(&before));
    let now = title_note_number()?;
    if now > before {
      note_aaai_first(&key.to_string(), u16::try_from(field).unwrap_or(0), now);
    }
    Ok(Tokens::new(Vec::new()))
  });
  with_origin(DefinitionOrigin::Stub, || -> Result<()> {
    DefMacro!(
      "\\equalcontrib",
      r"\lx@aaai@note{equalcontrib}{These authors contributed equally.}"
    );
    DefMacro!(
      "\\corresponding",
      r"\lx@aaai@note{corresponding}{Corresponding author.}"
    );
    Ok(())
  })?;
  // (after the stubs: a copy's own `\equalcontrib` replaces them, revMTREDUCE S3)
  input_shipped_package_title_marks(FNSYMBOL_TITLE_MARKS)?;
});

thread_local! {
  /// AAAI's notes made once (`\equalcontrib`, `\corresponding`): each kind's first use, by its position in the author
  /// field (`Token::prov`, 0 for none), and the footnote number its note took ([`aaai_note`]).
  static AAAI_FIRST_NOTES: RefCell<Vec<(String, u16, i64)>> = const { RefCell::new(Vec::new()) };
  /// A use followed by "." was read ([`aaai_note`]): the emitter's cut at ". " gives the next name the use before it,
  /// so every use is then a note of its own, as before the first-use reading (c1/c6/c7 of review revPRESBXb, 2509.03329).
  static AAAI_DOTTED: Cell<bool> = const { Cell::new(false) };
}

/// `\lx@aaai@note{<kind>}{<text>}`: the note `<text>` at the kind's first use, `\footnotemark[<n>]` after it. The rehearsal
/// tells the first by its position in the field (`Token::prov`); a use digested with none (a name the emitter digests
/// again, an `\affiliations` field) is a later one once the kind's note is made (OD #1053).
/// Once a use is followed by "." ([`AAAI_DOTTED`]), every use is `\thanks{<text>}`.
fn aaai_note(kind: &str, text: Tokens) -> Result<Tokens> {
  let dotted =
    peek_token()?.is_some_and(|t| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "."));
  if dotted || AAAI_DOTTED.get() {
    // (the fail-safe: no note lost and no name given another's mark; each name's note marked as its own)
    AAAI_DOTTED.set(true);
    let mut note = vec![T_CS!("\\thanks"), T_BEGIN!()];
    note.extend(text.unlist());
    note.push(T_END!());
    return Ok(Tokens::new(note));
  }
  let field = get_current_token().map_or(0, |t| t.prov);
  let now = title_note_number()?;
  let first = AAAI_FIRST_NOTES.with(|notes| {
    notes
      .borrow()
      .iter()
      .find(|(k, at, _)| k == kind && (field == 0 || (*at != 0 && *at < field)))
      .map(|(_, _, number)| *number)
  });
  if let Some(number) = first {
    // (a name digested again carries the kind's mark once: a segment that also holds the name before's,
    // `Bernal\equalcontrib. Raigoso\equalcontrib`, 2509.03329)
    let marked = s!("lx_aaai_marked@{kind}");
    if field == 0 && rehearsal_footnote_value().is_none() {
      if lookup_bool(&marked) {
        return Ok(Tokens::new(Vec::new()));
      }
      assign_value(&marked, true, None);
    }
    let mut mark = vec![T_CS!("\\footnotemark"), T_OTHER!("[")];
    mark.extend(Explode!(number));
    mark.push(T_OTHER!("]"));
    return Ok(Tokens::new(mark));
  }
  let before = now;
  let mut note = vec![T_CS!("\\thanks"), T_BEGIN!()];
  note.extend(text.unlist());
  note.extend([T_END!(), T_CS!("\\lx@aaai@noted"), T_BEGIN!()]);
  note.extend(Explode!(kind));
  note.extend([T_END!(), T_BEGIN!()]);
  note.extend(Explode!(field));
  note.extend([T_END!(), T_BEGIN!()]);
  note.extend(Explode!(before));
  note.push(T_END!());
  Ok(Tokens::new(note))
}

/// The note of `kind` made at field position `field` (0: none) took number `number` ([`aaai_note`]): kept unless one is
/// kept for an earlier position, or for any where this one has none; at the same position, a rehearsal's count replaces
/// the one before (the last rehearsal's base counts the title's notes digested ahead of the names: "Title∗", "Ann†",
/// "Bob†"), a later run's (the title-page check's) does not.
fn note_aaai_first(kind: &str, field: u16, number: i64) {
  let rehearsing = rehearsal_footnote_value().is_some();
  AAAI_FIRST_NOTES.with(|notes| {
    let mut notes = notes.borrow_mut();
    if notes.iter().any(|(k, at, _)| {
      k == kind && (field == 0 || (*at != 0 && (*at < field || (*at == field && !rehearsing))))
    }) {
      return;
    }
    notes.retain(|(k, ..)| k != kind);
    notes.push((kind.to_string(), field, number));
  });
}
