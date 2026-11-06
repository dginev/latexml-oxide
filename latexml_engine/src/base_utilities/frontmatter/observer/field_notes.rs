//! A rehearsal's notes on what its boxes do not hold ([`FieldNotes`]): the definitions and the errors of a frontmatter
//! field's tokens, by position, taken on each field token's invocation (core `token::FieldNotesHook`).

use std::cell::RefCell;

use latexml_core::{
  T_CS,
  common::{arena::SymStr, error::REPORT, object::Object, store::Stored},
  digested::{Digested, DigestedData},
  document::FIELD_FORMULA_START_PROPERTY,
  gullet::{pushback_is_empty, pushback_len},
  state::{self, MeaningAssigned},
  stomach::{field_token_encloses, token_stack_depth},
  token::{
    FieldCover, FieldInvocation, TOKEN_EXPANDAFTER, Token, field_cover_pending, field_prov_pos,
    set_field_notes_hook,
  },
  tokens::Tokens,
};

use super::events::BreakKind;

/// A definition made while a rehearsal digests a field's tokens ([`FieldNotes`]): what no box says, a definition printing
/// nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDeclaration {
  /// where it starts: the position of the field's token that made it, a definer or the call of a macro whose expansion is
  /// the definer, or before it what leads to it ([`FieldNotes::lead`]: `\global`, `\long`, `\expandafter`)
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

/// A call of a block break its binding declares (`Policy::block_breaks`), a command printing whatever the class makes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldBreak {
  /// the position of the call
  pub at:      u16,
  /// the command
  pub command: Token,
  /// what it breaks
  pub kind:    BreakKind,
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
  /// the calls of a block break its binding declares ([`FieldBreak`])
  pub breaks:       Vec<FieldBreak>,
  /// which commands those are
  block_breaks:     Vec<(Token, BreakKind)>,
  /// the errors counted so far, and the groups open and the tokens being invoked when the notes began
  errors_seen:      usize,
  depth:            usize,
  stack:            usize,
  /// the meaning log's last read, an empty buffer between two reads (`state::swap_meaning_log`)
  assigned:         Vec<MeaningAssigned>,
  /// the field's tokens by position, what was called there ([`keep_declared`]), the first at each
  field:            Vec<Option<Token>>,
  /// the runs of a kernel's or a binding's macro a paper's macro reached, being read ([`close_runs`]): the position they take
  /// and how many tokens the pushback held before each
  kernel_runs:      Vec<(u16, usize)>,
  /// where what leads to the next invocation starts: the prefixes waiting for their command (`\global`, `\long`) and the
  /// `\expandafter`s read right before it, since a content read of a declaration's span without them would be another
  /// definition, or run `\csname` on its own (`\expandafter\def\csname ww\endcsname{1}`, repro
  /// `event_declaration_span_holds_its_lead`). Invocations, not positions: an earlier declaration, a font switch or a
  /// conditional ends it, and what those read is theirs (`\let\xa\global\def\y{1}`, `\ifx\global\global\def\y{1}\fi`)
  lead:             Option<u16>,
}

thread_local! {
  /// The notes being taken ([`field_notes_begin`]).
  static FIELD_NOTES: RefCell<FieldNotes> = RefCell::default();
}

/// Begin the notes of the tokens about to be digested, `tokens` the field's (a rehearsal's; rehearsals are not nested,
/// their journals not being: `state::begin_assignment_journal`). Their positions' end bounds what the last of them read.
pub fn field_notes_begin(tokens: &Tokens, block_breaks: &[(&str, BreakKind)]) {
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
      stack: token_stack_depth(),
      block_breaks: block_breaks
        .iter()
        .map(|(command, kind)| (T_CS!(*command), *kind))
        .collect(),
      ..FieldNotes::default()
    };
  });
  set_field_notes_hook(Some(field_notes_after));
}

/// End the notes ([`field_notes_begin`]).
pub fn field_notes_end() -> FieldNotes {
  set_field_notes_hook(None);
  FIELD_NOTES.with(RefCell::take)
}

/// Clears the hook however the rehearsal ends, a panic under a worker's `catch_unwind` too: it is set only while
/// [`field_notes_begin`]'s notes are open.
pub struct FieldNotesHookGuard;

impl Drop for FieldNotesHookGuard {
  fn drop(&mut self) { set_field_notes_hook(None); }
}

/// Whether `token`'s meaning reads nothing of the input by its parameters: a primitive naming no definee, whatever it
/// assigns on its own account (`\selectfont`, `tex_fonts::pickup_font`: `\font@name` and the font it loads). A definer reads
/// the name it defines.
fn names_nothing(token: &Token) -> bool {
  state::lookup_meaning(token)
    .and_then(|meaning| meaning.to_definition())
    .is_none_or(|meaning| {
      meaning
        .get_parameters()
        .is_none_or(|params| params.get_parameters().is_empty())
    })
}

/// Keep of what `token`, invoked at the field position `pos` and reading up to `reach`, assigned what the paper declares. The
/// field's own token there, a definer invoked directly (`\def`, `\let`, `\newcommand`), declares what it assigns. So does the
/// expansion of a call there of a macro the paper or a file it read defined, unless it is the expansion of a kernel's or a
/// binding's macro that call reached (`inside`, [`close_runs`]: `\newcommand\lxaff{\small\itshape}`, 2608.17767's
/// `\small\itshape`, `\itshape` being made by `\reinstall@nfss@defs` at `\begin{document}`), or names nothing. A kernel's or a
/// binding's macro called there assigns on its own account (a font switch's `\f@series`, `\centering`'s `\\`, an environment's
/// `\@currenvir`), so only the names the field handed it are the paper's (`\newcount\nn`, `\newsavebox\bb`, whose arguments the
/// expansion reads).
fn keep_declared(
  token: &Token,
  pos: u16,
  reach: u16,
  inside: bool,
  field: &[Option<Token>],
  assigned: &mut Vec<MeaningAssigned>,
) {
  let Some(Some(call)) = field.get(usize::from(pos)) else {
    assigned.clear();
    return;
  };
  let direct = call == token;
  match state::lookup_meaning(call) {
    Some(Stored::Expandable(_)) if direct => assigned.clear(),
    Some(Stored::Expandable(definition)) if definition.get_origin().is_latexml_owned() => {
      let handed = field
        .get(usize::from(pos) + 1..usize::from(reach).min(field.len()))
        .unwrap_or(&[]);
      assigned.retain(|assignment| {
        handed
          .iter()
          .flatten()
          .any(|t| state::meaning_key(t) == assignment.key)
      });
    },
    Some(Stored::Expandable(_)) => {
      if inside || names_nothing(token) {
        assigned.clear();
      }
    },
    _ if direct && !names_nothing(token) => {},
    _ => assigned.clear(),
  }
}

/// Close the runs of a kernel's or a binding's macro read by now: an invocation that was the run's took a token above what the
/// pushback held before the expansion, and left it at least that (`held`, past what it read; its tokens take the position of
/// the call they came from, `token::inherit_field_prov`, so nothing else tells). One that left it below came after the run: a
/// run's last token reading on past it read what the field handed it.
fn close_runs(notes: &mut FieldNotes, held: usize) {
  while notes
    .kernel_runs
    .last()
    .is_some_and(|(_, before)| held < *before)
  {
    notes.kernel_runs.pop();
  }
}

/// Note the expansion of the field token `token`, its tokens about to go on the input: a kernel's or a binding's macro a paper's macro reached
/// opens a run ([`keep_declared`]), its arguments of the field's own positions none of it (`\mbox{\def\x{1}}`, `\parbox`); what
/// leads to a declaration ([`FieldNotes::lead`]) goes on through `\expandafter`, and past any other only with a prefix waiting.
fn field_expansion(token: &Token) {
  let pos = field_prov_pos(token.prov);
  let (expandafter, owned) = match state::lookup_meaning(token) {
    Some(Stored::Expandable(meaning)) => (
      meaning.cs == *TOKEN_EXPANDAFTER,
      meaning.get_origin().is_latexml_owned(),
    ),
    _ => (false, false),
  };
  FIELD_NOTES.with(|notes| {
    let notes = &mut *notes.borrow_mut();
    let own = notes
      .field
      .get(usize::from(pos))
      .copied()
      .flatten()
      .as_ref()
      == Some(token);
    if !notes.kernel_runs.is_empty() || (owned && !own) {
      let held = pushback_len();
      close_runs(notes, held);
      if owned && !own && notes.kernel_runs.last().is_none_or(|(at, _)| *at != pos) {
        notes.kernel_runs.push((pos, held));
      }
    }
    if let Some((_, kind)) = notes
      .block_breaks
      .iter()
      .find(|(command, _)| command == token)
    {
      notes.breaks.push(FieldBreak {
        at:      pos,
        command: *token,
        kind:    *kind,
      });
    }
    if expandafter {
      notes.lead = Some(notes.lead.map_or(pos, |lead| lead.min(pos)));
    } else if notes.lead.is_some() && !state::has_tex_prefixes() {
      notes.lead = None;
    }
  });
}

/// What the hook hears ([`token::FieldNotesHook`]).
fn field_notes_after(token: &Token, cover: FieldCover, invocation: FieldInvocation<'_>) {
  match invocation {
    FieldInvocation::Digested(result) => field_invocation(token, cover, result),
    FieldInvocation::Expanded => field_expansion(token),
  }
}

/// Note the invocation of the field token `token` that read as far as `cover` and made `result`: the errors raised since
/// the last one, and, making no box, the meanings it declared ([`field_declares`]), as far as its call read — a macro's
/// arguments, the pending cover (its expansion's tokens inherit its position). A box made inside the reach of a declaration
/// still open (a macro printing ink, its internal `\let`s) takes it back: that box's span holds the call, and a
/// declaration of it would print it twice. The first token read past its reach closes it.
fn field_invocation(token: &Token, cover: FieldCover, result: &[Digested]) {
  let pos = field_prov_pos(token.prov);
  // (the opening shift's invocation holds the box its closing shift made: the formula starts here)
  for digested in result {
    if matches!(digested.data(), DigestedData::Whatsit(w) if w.borrow().is_math()) {
      digested.try_set_property(FIELD_FORMULA_START_PROPERTY, i64::from(pos));
    }
  }
  // (what it is read inside of and led by, and what the next invocation will be: as the last left them)
  let waiting = result.is_empty() && state::has_tex_prefixes();
  let (inside, lead) = FIELD_NOTES.with(|notes| {
    let notes = &mut *notes.borrow_mut();
    if !notes.kernel_runs.is_empty() {
      close_runs(notes, pushback_len());
    }
    let inside = notes.kernel_runs.last().is_some_and(|(at, _)| *at == pos);
    let lead = notes.lead.take();
    if waiting {
      notes.lead = Some(lead.unwrap_or(pos));
    }
    (inside, lead)
  });
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
  // (the call's whole reach when its expansion invoked this: a macro's arguments. A field's own token read what its cover says,
  // an `\expandafter` before it having read on past it: `\expandafter\def\expandafter\y\expandafter{\expandafter}Ann`'s `A`)
  let pending = field_cover_pending();
  let own = FIELD_NOTES.with(|notes| {
    notes
      .borrow()
      .field
      .get(usize::from(pos))
      .copied()
      .flatten()
      .as_ref()
      == Some(token)
  });
  let pending_here =
    !own && pending.field == cover.field && pending.start <= pos && pos <= pending.end;
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
    if !result.is_empty() || assigned.is_empty() || field_token_encloses(notes.stack) {
      return;
    }
    // (an enclosing invocation still running assigned the others: `\begin{minipage}`'s, which digests its body)
    assigned.retain(|assignment| assignment.by == *token);
    keep_declared(token, pos, reach, inside, &notes.field, &mut assigned);
    if assigned.is_empty() {
      return;
    }
    let global = assigned.iter().all(|assignment| assignment.global);
    let definees = assigned.drain(..).map(|assignment| assignment.key);
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
        let mut once = Vec::new();
        for key in definees {
          if !once.contains(&key) {
            once.push(key);
          }
        }
        notes.declarations.push(FieldDeclaration {
          start: lead.unwrap_or(pos),
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
