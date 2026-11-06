//! A rehearsal's notes on what its boxes do not hold ([`FieldNotes`]): the definitions and the errors of a frontmatter
//! field's tokens, by position, taken on each field token's invocation (core `token::FieldNotesHook`).

use std::cell::RefCell;

use latexml_core::{
  common::{
    arena::{self, SymStr},
    error::REPORT,
    store::Stored,
  },
  definition::origin::DefinitionOrigin,
  digested::{Digested, DigestedData},
  document::FIELD_FORMULA_START_PROPERTY,
  gullet::pushback_is_empty,
  state,
  token::{
    FieldCover, TOKEN_EXPANDAFTER, Token, field_cover_pending, field_prov_pos, set_field_notes_hook,
  },
  tokens::Tokens,
};

/// A definition made while a rehearsal digests a field's tokens ([`FieldNotes`]): what no box says, a definition printing
/// nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDeclaration {
  /// where it starts: the position of the field's token that made it, a definer or the call of a macro whose expansion is
  /// the definer, or before it what leads to it ([`declaration_start`]: `\global`, `\long`, `\expandafter`)
  pub start:    u16,
  /// one past the last position that invocation read
  pub end:      u16,
  /// the meanings it assigned, in order, each once
  pub definees: Vec<SymStr>,
  /// whether every one of them was assigned globally
  pub global:   bool,
  /// the groups open around it, in the field
  pub depth:    usize,
  /// its call may still be being read ([`field_notes_after`])
  open:         bool,
  /// the position of the invocation that made it, which a macro's expansion shares
  call:         u16,
}

/// A rehearsal's notes on what its boxes do not hold, by the field position of the invocation (the stomach's, right after
/// it read what it reads: `field_notes_after`), open from [`field_notes_begin`] to [`field_notes_end`].
#[derive(Debug, Default)]
pub struct FieldNotes {
  /// the definitions made by invocations that made no box ([`FieldDeclaration`])
  pub declarations: Vec<FieldDeclaration>,
  /// the positions `start..end` of the invocations during which errors were raised, each with how many
  pub errors:       Vec<(u16, u16, usize)>,
  /// the positions the field's tokens end at (the reach of the field's last invocation)
  end:              u16,
  /// the errors counted so far, and the groups open when the notes began
  errors_seen:      usize,
  depth:            usize,
  /// the meaning log's last read, an empty buffer between two reads (`state::swap_meaning_log`)
  assigned:         Vec<(SymStr, bool)>,
  /// the field's tokens by position, what was called there ([`field_declares`]), the first at each
  field:            Vec<Option<Token>>,
}

thread_local! {
  /// The notes being taken ([`field_notes_begin`]).
  static FIELD_NOTES: RefCell<FieldNotes> = RefCell::default();
}

/// Begin the notes of the tokens about to be digested, `tokens` the field's (a rehearsal's; rehearsals are not nested,
/// their journals not being: `state::begin_assignment_journal`). Their positions' end bounds what the last of them read.
pub fn field_notes_begin(tokens: &Tokens) {
  let mut field: Vec<Option<Token>> = Vec::new();
  for token in tokens.unlist_ref().iter().filter(|t| t.prov != 0) {
    let pos = usize::from(field_prov_pos(token.prov));
    if field.len() <= pos {
      field.resize(pos + 1, None);
    }
    field[pos].get_or_insert(*token);
  }
  let end = u16::try_from(field.len()).unwrap_or(u16::MAX);
  FIELD_NOTES.with(|notes| {
    *notes.borrow_mut() = FieldNotes {
      end,
      field,
      errors_seen: REPORT.borrow().error,
      depth: state::get_frame_depth(),
      ..FieldNotes::default()
    };
  });
  // (what the rehearsal bound before its tokens is none of theirs)
  state::swap_meaning_log(&mut Vec::new());
  set_field_notes_hook(Some(field_notes_after));
}

/// End the notes ([`field_notes_begin`]).
pub fn field_notes_end() -> FieldNotes {
  set_field_notes_hook(None);
  FIELD_NOTES.with(RefCell::take)
}

/// Whether `token`, invoked at the field position `pos`, stands for what the paper declares: the field's own token there, a
/// definer invoked directly (`\def`, `\let`, `\newcommand`), or the call there of a macro the paper or a file it read
/// defined, whose expansion `token` is. A kernel's or a binding's macro assigning meanings on its own account (a font
/// switch's `\f@series`, `\centering`'s `\\`, an environment's `\@currenvir`) declares nothing of the paper's.
fn field_declares(token: &Token, pos: u16, field: &[Option<Token>]) -> bool {
  let Some(Some(call)) = field.get(usize::from(pos)) else {
    return false;
  };
  if call == token {
    return !matches!(state::lookup_meaning(call), Some(Stored::Expandable(_)));
  }
  match state::lookup_meaning(call) {
    Some(Stored::Expandable(definition)) => matches!(
      latexml_core::common::object::Object::get_origin(&*definition),
      DefinitionOrigin::Document | DefinitionOrigin::File
    ),
    _ => false,
  }
}

/// Whether the meaning `key` is the kernel's own state of font selection, which a font switch assigns (`\fontshape`'s
/// `\f@shape`, `\selectfont`'s `\font@name` and the font it loads, `\OT1/cmr/m/it/10`): no declaration of the paper's. The
/// call alone does not tell: `\reinstall@nfss@defs` defines `\itshape`, `\scshape` and their kin again at
/// `\begin{document}`, so they read as the document's.
fn is_font_state(key: SymStr) -> bool {
  arena::with(key, |name| {
    name == "\\font@name"
      || name.starts_with("\\f@")
      || name.ends_with("f@size")
      || name.matches('/').count() >= 3
  })
}

/// Where the declaration made by the invocation at `pos` starts: at what leads to it (`\global`, `\long`, prefixes making no
/// box; `\expandafter`, read by the gullet), since a content read of its span without them would be another definition, or
/// run `\csname` on its own (`\expandafter\def\csname ww\endcsname{1}`, repro `event_declaration_span_holds_its_lead`).
/// Nothing else leads to it: not an earlier declaration, a font switch, or a conditional, whose `\fi` is past the span.
fn declaration_start(pos: u16, field: &[Option<Token>]) -> u16 {
  let leads = |t: &Token| match state::lookup_meaning(t) {
    Some(Stored::Primitive(meaning)) => meaning.is_prefix,
    Some(Stored::Expandable(meaning)) => meaning.cs == *TOKEN_EXPANDAFTER,
    _ => false,
  };
  let mut start = usize::from(pos);
  while start > 0
    && field
      .get(start - 1)
      .copied()
      .flatten()
      .as_ref()
      .is_some_and(leads)
  {
    start -= 1;
  }
  u16::try_from(start).unwrap_or(pos)
}

/// Note the invocation of the field token `token` that read as far as `cover` and made `result`: the errors raised since
/// the last one, and, making no box, the meanings it declared ([`field_declares`]), as far as its call read — a macro's
/// arguments, the pending cover (its expansion's tokens inherit its position). A box made inside the reach of a declaration
/// still open (a macro printing ink, its internal `\let`s) takes it back: that box's span holds the call, and a
/// declaration of it would print it twice. The first token read past its reach closes it.
fn field_notes_after(token: &Token, cover: FieldCover, result: &[Digested]) {
  let pos = field_prov_pos(token.prov);
  // (the opening shift's invocation holds the box its closing shift made: the formula starts here)
  for digested in result {
    if matches!(digested.data(), DigestedData::Whatsit(w) if w.borrow().is_math()) {
      digested.try_set_property(FIELD_FORMULA_START_PROPERTY, i64::from(pos));
    }
  }
  // (nearly every invocation: no error, nothing assigned, no declaration its reach could close)
  let errors = REPORT.borrow().error;
  let quiet = FIELD_NOTES.with(|notes| {
    let notes = notes.borrow();
    errors == notes.errors_seen
      && !notes
        .declarations
        .iter()
        .any(|declaration| declaration.open)
  });
  if quiet && state::meaning_log_is_empty() {
    return;
  }
  let mut assigned = FIELD_NOTES.with(|notes| std::mem::take(&mut notes.borrow_mut().assigned));
  state::swap_meaning_log(&mut assigned);
  let pending = field_cover_pending();
  let pending_here = pending.field == cover.field && pending.start <= pos && pos <= pending.end;
  let reach = if pending_here {
    pending.end.max(cover.end)
  } else {
    cover.end
  };
  let drained = pushback_is_empty();
  let depth = (drained || !assigned.is_empty()).then(state::get_frame_depth);
  FIELD_NOTES.with(|notes| {
    let notes = &mut *notes.borrow_mut();
    // (the field's last token, outside any group: nothing left in the input shows how far it read, which is to the end)
    let reach = if drained && depth == Some(notes.depth) {
      reach.max(notes.end)
    } else {
      reach
    };
    if errors > notes.errors_seen {
      notes
        .errors
        .push((pos, reach.max(pos + 1), errors - notes.errors_seen));
    }
    notes.errors_seen = errors;
    notes.declarations.retain_mut(|declaration| {
      if declaration.open {
        if pos >= declaration.end || (pending.field == cover.field && pending.end > declaration.end)
        {
          declaration.open = false;
        } else if !result.is_empty() && declaration.start <= pos {
          return false;
        }
      }
      true
    });
    if !result.is_empty() || assigned.is_empty() || !field_declares(token, pos, &notes.field) {
      return;
    }
    assigned.retain(|(key, _)| !is_font_state(*key));
    if assigned.is_empty() {
      return;
    }
    let global = assigned.iter().all(|(_, global)| *global);
    let definees = assigned.drain(..).map(|(key, _)| key);
    match notes.declarations.last_mut() {
      // (a macro's expansion defining several, one after another: one call's)
      Some(last) if last.open && last.call == pos => {
        last.end = last.end.max(reach);
        last.global &= global;
        for key in definees {
          if !last.definees.contains(&key) {
            last.definees.push(key);
          }
        }
      },
      _ => {
        let start = declaration_start(pos, &notes.field);
        let mut once = Vec::new();
        for key in definees {
          if !once.contains(&key) {
            once.push(key);
          }
        }
        notes.declarations.push(FieldDeclaration {
          start,
          end: reach.max(pos + 1),
          definees: once,
          global,
          depth: depth.unwrap_or(0).saturating_sub(notes.depth),
          open: true,
          call: pos,
        });
      },
    }
  });
  assigned.clear();
  FIELD_NOTES.with(|notes| notes.borrow_mut().assigned = assigned);
}
