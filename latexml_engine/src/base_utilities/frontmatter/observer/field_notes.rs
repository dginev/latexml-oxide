//! A rehearsal's notes on what its boxes do not hold ([`FieldNotes`]): the definitions and the errors of a frontmatter
//! field's tokens, by position, taken on each field token's invocation (core `token::FieldNotesHook`).

use std::{cell::RefCell, rc::Rc};

use latexml_core::{
  T_CS,
  common::{arena::SymStr, error::REPORT, object::Object, store::Stored},
  definition::{ExpansionBody, expandable::Expandable, origin::DefinitionOrigin},
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
  /// where the prefixes waiting for their command start (`\global`, `\long`). With the `\expandafter`s expanded right before
  /// it they lead to a declaration ([`read_lead`]), since a content read of its span without them would be another definition,
  /// or run `\csname` on its own (`\expandafter\def\csname ww\endcsname{1}`, repro `event_declaration_span_holds_its_lead`).
  /// Invocations, not positions: an earlier declaration, a font switch or a conditional ends it, and what those read is theirs
  /// (`\let\xa\global\def\y{1}`, `\ifx\global\global\def\y{1}\fi`)
  lead:             Option<u16>,
  /// the same, as the invocation being noted reads it
  lead_read:        Option<u16>,
  /// the last run of one field token expanded since the last invocation, where it starts (an `\expandafter` chain's), and that of
  /// the invocation being noted: the expansions of a block of a thousand names, no list of them
  expanded:         Option<(u16, Token)>,
  read_expanded:    Option<(u16, Token)>,
  /// whether a declaration's call may still be being read ([`FieldDeclaration`] `open`)
  any_open:         bool,
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

/// End the notes ([`field_notes_begin`]), each declaration's span a run of the field a replay can read
/// ([`settle_declarations`]).
pub fn field_notes_end() -> FieldNotes {
  set_field_notes_hook(None);
  let mut notes = FIELD_NOTES.with(RefCell::take);
  if !notes.declarations.is_empty() {
    settle_declarations(&mut notes);
  }
  notes
}

/// The spans of a block's declarations as runs the emitter can replay, in order, none reading what another reads. Where the
/// invocation's reach is true of the reading but not of the content:
/// - **A body's definition.** One in the body of a macro defined in the field runs at the macro's call, its tokens still at their
///   own positions, inside the braces of what defined them: `\def\m{\newcount\qq}\m`'s `\qq` read [3,7] over `\m`'s [0,6]. It starts
///   past that definition, at the call; with nothing left of its own (`\def\x{\def\y{1}\def\z{2}}\x`'s `\y`) its names go with the next,
///   which holds the call, or the last.
/// - **A peek.** A macro testing the next token read as far as it (`\mstar\mfile`, whose `\@ifstar` peeked at `\mfile`, itself a
///   declaration's call): outside the braces of the earlier, the later one holds its own call, and the earlier ends there.
/// - **Brace balance.** A definer whose next token is no later one of the field read as far as the call around it read: the end of an
///   argument, `\textbf{\def\x{1}}`, `\AtBeginDocument{\def\x{1}\def\y{2}}`'s last, holds a closing brace of what opened before it. The
///   span ends before it.
#[cold]
#[inline(never)]
fn settle_declarations(notes: &mut FieldNotes) {
  let field = std::mem::take(&mut notes.field);
  let step = |pos: u16| {
    field
      .get(usize::from(pos))
      .copied()
      .flatten()
      .map_or(0, |t| t.group_step())
  };
  let mut settled: Vec<FieldDeclaration> = Vec::with_capacity(notes.declarations.len());
  let mut waiting: Vec<SymStr> = Vec::new();
  let add = |to: &mut Vec<SymStr>, names: Vec<SymStr>| {
    for name in names {
      if !to.contains(&name) {
        to.push(name);
      }
    }
  };
  for mut declaration in notes.declarations.drain(..) {
    if let Some(last) = settled.last_mut()
      && declaration.start < last.end
    {
      let inside = (last.start..declaration.start).map(step).sum::<i32>() > 0;
      if inside {
        declaration.start = last.end;
      } else if declaration.start > last.start {
        last.end = declaration.start;
      } else {
        add(&mut last.definees, declaration.definees);
        continue;
      }
    }
    if declaration.end <= declaration.start {
      waiting.append(&mut declaration.definees);
      continue;
    }
    if !waiting.is_empty() {
      let mine = std::mem::replace(&mut declaration.definees, std::mem::take(&mut waiting));
      add(&mut declaration.definees, mine);
    }
    settled.push(declaration);
  }
  for declaration in &mut settled {
    let mut depth = 0i32;
    for pos in declaration.start..declaration.end {
      depth += step(pos);
      if depth < 0 {
        declaration.end = pos;
        break;
      }
    }
  }
  if let Some(last) = settled.last_mut() {
    add(&mut last.definees, waiting);
  }
  notes.declarations = settled;
  notes.field = field;
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

/// `token`'s meaning when a macro.
fn macro_of(token: &Token) -> Option<Rc<Expandable>> {
  match state::lookup_meaning(token) {
    Some(Stored::Expandable(meaning)) => Some(meaning),
    _ => None,
  }
}

/// What the field hands a call at `pos` reading up to `reach`: its arguments' tokens.
fn handed_to_a_call(field: &[Option<Token>], pos: u16, reach: u16) -> &[Option<Token>] {
  field
    .get(usize::from(pos) + 1..usize::from(reach).min(field.len()))
    .unwrap_or(&[])
}

/// The names a macro the document defined holds in its body, and those a macro of the same kind in its body holds: what a kernel's or
/// a binding's macro it calls may be defining for it (`\newcommand\m{\newcount\qq}`, `\newcommand\m{\@ifundefined{qq}{\def\qq{1}}{}}`).
/// A name built at run time is in no body (`\@namedef{qq}{1}`). None for a macro of a `.sty` or `.cls` it loaded: a raw package's body names its own
/// state, which a run of the kernel's assigns for it (`\makecell`'s `\tabg@pe`, `\t@bset`, 2203.03195, 2502.11387).
#[cold]
#[inline(never)]
fn names_in_a_papers_macro(call: &Rc<Expandable>) -> Vec<SymStr> {
  let mut names: Vec<SymStr> = Vec::new();
  if call.get_origin() != DefinitionOrigin::Document {
    return names;
  }
  let mut bodies = vec![Rc::clone(call)];
  let mut read = 0usize;
  while let Some(macro_) = bodies.pop() {
    let Some(ExpansionBody::Tokens(body)) = macro_.expansion.as_ref() else {
      continue;
    };
    for token in body.unlist_ref() {
      let key = state::meaning_key(token);
      if names.contains(&key) {
        continue;
      }
      names.push(key);
      if let Some(meaning) = macro_of(token)
        && meaning.get_origin() == DefinitionOrigin::Document
        && read < MACROS_READ_FOR_NAMES
      {
        read += 1;
        bodies.push(meaning);
      }
    }
  }
  names
}

/// How many macros [`names_in_a_papers_macro`] reads, past which a name is too far from the call to be its.
const MACROS_READ_FOR_NAMES: usize = 64;

/// Keep of what `token`, invoked at the field position `pos` and reading up to `reach`, assigned what the paper declares. The
/// field's own token there, a definer invoked directly (`\def`, `\let`, `\newcommand`), declares what it assigns. So does the
/// expansion of a call there of a macro the paper or a file it read defined, unless it names nothing. Within the run of a kernel's or
/// a binding's macro that call reached (`inside`, [`close_runs`]) the kernel assigns on its own account (`\size@update`,
/// `\@let@token`), so only what the call's body or its arguments name is the paper's: 2608.17767's `\newcommand\lxaff{\small\itshape}`
/// declares nothing, `\itshape` being made by `\reinstall@nfss@defs` at `\begin{document}`; `\newcommand\m{\newcount\qq}`
/// and `\newcommand\m{\@ifundefined{qq}{\def\qq{1}}{}}` declare `\qq` ([`names_in_a_papers_macro`]). A kernel's or a binding's
/// macro called there assigns on its own account likewise (a font switch's `\f@series`, `\centering`'s `\\`, an environment's
/// `\@currenvir`): only the names the field handed it are the paper's (`\newcount\nn`, `\newsavebox\bb`, whose arguments the
/// expansion reads).
#[cold]
#[inline(never)]
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
      let handed = handed_to_a_call(field, pos, reach);
      assigned.retain(|assignment| {
        handed
          .iter()
          .flatten()
          .any(|t| state::meaning_key(t) == assignment.key)
      });
    },
    Some(Stored::Expandable(definition)) => {
      if inside {
        let named = names_in_a_papers_macro(&definition);
        let handed = handed_to_a_call(field, pos, reach);
        assigned.retain(|assignment| {
          named.contains(&assignment.key)
            || handed
              .iter()
              .flatten()
              .any(|t| state::meaning_key(t) == assignment.key)
        });
      } else if names_nothing(token) {
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

/// What `test` says of `token`'s meaning when a macro; false else.
fn with_expandable(token: &Token, test: impl Fn(&Expandable) -> bool) -> bool {
  macro_of(token).is_some_and(|meaning| test(&meaning))
}

/// Note the expansion of the field token `token`, its tokens about to go on the input: a kernel's or a binding's macro a paper's
/// macro reached opens a run ([`keep_declared`]), its arguments of the field's own positions none of it (`\mbox{\def\x{1}}`,
/// `\parbox`); a block break its binding declares is noted; it is kept for what leads to the next invocation ([`read_lead`]).
/// One for every tie of a block of a thousand names (1507.02068): a meaning is looked up only for a token no field holds there.
fn field_expansion(token: &Token) {
  let pos = field_prov_pos(token.prov);
  FIELD_NOTES.with(|notes| {
    let notes = &mut *notes.borrow_mut();
    let own = notes
      .field
      .get(usize::from(pos))
      .copied()
      .flatten()
      .as_ref()
      == Some(token);
    let owned = !own && with_expandable(token, |meaning| meaning.get_origin().is_latexml_owned());
    if !notes.kernel_runs.is_empty() || owned {
      let held = pushback_len();
      close_runs(notes, held);
      if owned && notes.kernel_runs.last().is_none_or(|(at, _)| *at != pos) {
        notes.kernel_runs.push((pos, held));
      }
    }
    if !notes.block_breaks.is_empty()
      && let Some((_, kind)) = notes
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
    notes.expanded = match notes.expanded {
      Some((start, last)) if last == *token => Some((start.min(pos), last)),
      _ => Some((pos, *token)),
    };
  });
}

/// Where what leads to an invocation starts ([`FieldNotes::lead`]): the prefixes waiting for it, and the `\expandafter`s last expanded
/// before it, the inner ones first (`\expandafter\def\expandafter\y…`); any other expansion ends those, a prefix waiting past it.
fn read_lead(waiting: Option<u16>, expanded: Option<(u16, Token)>) -> Option<u16> {
  match expanded {
    Some((pos, token)) if with_expandable(&token, |meaning| meaning.cs == *TOKEN_EXPANDAFTER) => {
      Some(waiting.map_or(pos, |lead| lead.min(pos)))
    },
    _ => waiting,
  }
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
  // (a letter of a name, nearly every invocation: it makes a box, in no run, led by nothing, no error, no declaration its reach
  // could close, nothing assigned)
  let errors = REPORT.borrow().error;
  if !result.is_empty()
    && state::meaning_log_is_empty()
    && FIELD_NOTES.with(|notes| {
      let notes = notes.borrow();
      notes.kernel_runs.is_empty()
        && notes.expanded.is_none()
        && notes.lead.is_none()
        && !notes.any_open
        && errors == notes.errors_seen
    })
  {
    return;
  }
  // (what it is read inside of and led by, as the last left them)
  let waiting = result.is_empty() && state::has_tex_prefixes();
  let (inside, quiet) = FIELD_NOTES.with(|notes| {
    let notes = &mut *notes.borrow_mut();
    if !notes.kernel_runs.is_empty() {
      close_runs(notes, pushback_len());
    }
    let inside = notes.kernel_runs.last().is_some_and(|(at, _)| *at == pos);
    // (the expansions before it, read at a declaration or by a prefix; the prefixes waiting for it, then for the next)
    notes.read_expanded = notes.expanded.take();
    notes.lead_read = notes.lead.take();
    if waiting {
      notes.lead = read_lead(notes.lead_read, notes.read_expanded).or(Some(pos));
    }
    (inside, errors == notes.errors_seen && !notes.any_open)
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
  // (a field's own token whose next token is no later one of the field read what the call around it read, which shows no more:
  // `\newcommand\pass[1]{#1\relax}\pass{\def\x{1}}`, `\textbf{\def\x{1}}`, the last of `\AtBeginDocument{\def\x{1}\def\y{2}}`)
  let pending_here = (!own || cover.end <= pos + 1)
    && pending.field == cover.field
    && pending.start <= pos
    && pos <= pending.end;
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
    if notes.any_open {
      notes.declarations.retain_mut(|declaration| {
        if declaration.open {
          if pos >= declaration.end
            || (pending.field == cover.field && pending.end > declaration.end)
          {
            declaration.open = false;
          } else if !result.is_empty() && declaration.start <= pos {
            return false;
          }
        }
        true
      });
      notes.any_open = notes
        .declarations
        .iter()
        .any(|declaration| declaration.open);
    }
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
          start: read_lead(notes.lead_read, notes.read_expanded).unwrap_or(pos),
          end: reach.max(pos + 1),
          definees: once,
          global,
          depth: depth.unwrap_or(0).saturating_sub(notes.depth),
          open: true,
          call: pos,
        });
        notes.any_open = true;
      },
    }
  });
  assigned.clear();
  FIELD_NOTES.with(|notes| notes.borrow_mut().assigned = assigned);
}
