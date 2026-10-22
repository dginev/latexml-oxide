//! The observer's rehearsal: a block's stamped tokens digested as the article `\author` line prints them, with
//! Layer-B event bindings (Perl's `@authorsplits`, Base_Utility.pool.ltxml:676-683: `\and` a column, `\\` a row; the
//! notes and contacts as events), through the speculative-run engine (sect05.rs `speculative_title_page_run`):
//! assignments rolled back (`AssignmentJournalGuard`), diagnostics held (`DiagnosticsHold`), terminal output muted, the
//! frontmatter frozen (`lx_replaying_class_setter`: nothing is queued), the field ledger put back
//! (`FieldLedgerScope`), progress notes silenced (an `\input` in the field: `SilenceProgressNotesGuard`). The
//! digested list is handed to a callback before it is discarded. Not LaTeX's `\begin{tabular}[t]{c}` around the
//! line: no alignment machinery runs, and a stray `&` is digested as a whatsit that errs only when built into the
//! document (tex_tables.rs `&`), never here: nothing is held for it, and the lexer reads it as a column separator.

use std::cell::Cell;

use latexml_core::{
  definition::origin::{DefinitionOrigin, with_origin},
  token::FieldLedgerScope,
};

use super::{ledger::SkipReason, policy::TitleMarks, *};

thread_local! {
  static REHEARSING: Cell<bool> = const { Cell::new(false) };
}

/// Whether a rehearsal is digesting now (the ledger records nothing of it).
pub(in crate::base_utilities) fn rehearsing() -> bool { REHEARSING.with(Cell::get) }

/// No rehearsal is digesting (a conversion's start: one a panic left marked on a reused thread is over).
pub(in crate::base_utilities) fn reset_rehearsing() { REHEARSING.with(|r| r.set(false)); }

/// The event constructors of the Layer-B bindings ([`install_event_bindings`]): whatsits the lexer reads
/// (`events::lex_events`), never absorbed (a rehearsal's list is discarded), so they construct nothing. The `\and`
/// family keeps which command it was (`\AND` a new row in NeurIPS's and ICLR's `\author`); `\thanks`, `\footnote` and
/// `\footnotemark` count the footnote steps they make in LaTeX (`FOOTNOTE_STEPS`), never stepping `\c@footnote`.
pub(in crate::base_utilities) fn load() -> Result<()> {
  DefConstructor!("\\lx@fm@obs@and", "");
  DefConstructor!("\\lx@fm@obs@And", "");
  DefConstructor!("\\lx@fm@obs@AND", "");
  DefConstructor!("\\lx@fm@obs@break OptionalMatch:* []", "");
  // `\thanks` and `\footnote` (inside `\maketitle`, `\footnote` is `\thanks`): latex.ltx `\thanks` = `\footnotemark`
  // + the text, a step of the footnote counter unless an optional number is given
  DefMacro!("\\lx@fm@obs@thanks []", sub[(number)] {
    if number.is_none() {
      FOOTNOTE_STEPS.with(|steps| steps.set(steps.get().saturating_add(1)));
    }
    let mut out = vec![T_CS!("\\lx@fm@obs@note")];
    if let Some(number) = number {
      out.push(T_OTHER!("["));
      out.push(T_BEGIN!());
      out.extend(number.unlist());
      out.push(T_END!());
      out.push(T_OTHER!("]"));
    }
    Ok(Tokens::new(out))
  });
  DefConstructor!("\\lx@fm@obs@note []{}", "");
  // `\footnotetext`: the text of a mark printed elsewhere, no step
  DefConstructor!("\\lx@fm@obs@notetext []{}", "");
  // `\footnotemark[<n>]` prints `\thefootnote` at <n> (latex.ltx `\@xfootnotemark`), a bare `\footnotemark` at the
  // counter's next value (`\stepcounter\@mpfn`): its `printed` property is that print, made with the counter set in a
  // group of its own, to <n> or to its value at the rehearsal's start plus the steps the block made up to it, so the
  // rehearsal steps nothing (the label is what the PDF prints when the counter at the flush is the one at
  // `\maketitle`). A constructor, as the command it stands for: its box covers its optional argument's positions. A
  // given `[<n>]` is read with `\c@footnote` at the rehearsal's start plus the steps the block made before it (local to
  // the mark), as LaTeX reads it after the title's notes stepped the counter: `\footnotemark[\arabic{footnote}]`
  // prints the mark of the last `\thanks` (2201.11183, 2111.13260; review rev78o2 finding 3), not of the unstepped
  // counter.
  DefMacro!("\\lx@fm@obs@footnotemark []", sub[(number)] {
    // (the constructor token carries the field position of the `\footnotemark` it stands for, so the mark's box covers
    // it as the command's own would)
    let prov = get_current_token().map_or(0, |t| t.prov);
    let mark = Token { prov, ..T_CS!("\\lx@fm@obs@fnmark") };
    let Some(number) = number else {
      return Ok(Tokens::new(vec![mark]));
    };
    let mut out = Vec::new();
    let counted = FOOTNOTE_BASE_DEFINED.with(Cell::get);
    if counted {
      let value = FOOTNOTE_BASE
        .with(Cell::get)
        .saturating_add(FOOTNOTE_STEPS.with(Cell::get));
      out.extend([T_CS!("\\begingroup"), T_CS!("\\c@footnote"), T_OTHER!("=")]);
      out.extend(Explode!(value));
      out.push(T_CS!("\\relax"));
    }
    out.extend([mark, T_OTHER!("["), T_BEGIN!()]);
    out.extend(number.unlist());
    out.extend([T_END!(), T_OTHER!("]")]);
    if counted {
      out.push(T_CS!("\\endgroup"));
    }
    Ok(Tokens::new(out))
  });
  DefConstructor!("\\lx@fm@obs@fnmark []", "", properties => sub[args] {
    let given = args.first().and_then(|a| a.as_ref()).map(|number| {
      let mut text = String::new();
      ink_text(number, &mut text);
      text.trim().to_string()
    });
    let value = match given {
      Some(text) => match text.parse::<i64>() {
        Ok(value) => value,
        // (not a number: TeX's "Missing number"; the mark is what was given)
        Err(_) => return Ok(stored_map!("printed" => Stored::String(pin(&text)))),
      },
      None => {
        let steps = FOOTNOTE_STEPS.with(|steps| {
          steps.set(steps.get().saturating_add(1));
          steps.get()
        });
        FOOTNOTE_BASE.with(Cell::get).saturating_add(steps)
      },
    };
    let printed = if FOOTNOTE_BASE_DEFINED.with(Cell::get) {
      let mut print = vec![T_BEGIN!(), T_CS!("\\c@footnote")];
      print.extend(Explode!(value));
      print.extend([T_CS!("\\relax"), T_CS!("\\thefootnote"), T_END!()]);
      let mut text = String::new();
      ink_text(&digest(Tokens::new(print))?, &mut text);
      text.trim().to_string()
    } else {
      // (no LaTeX footnote counter: the number printed as it is)
      value.to_string()
    };
    Ok(stored_map!("printed" => Stored::String(pin(&printed))))
  });
  // the marks the kernel `\maketitle` prints (latex.ltx/article.cls `\renewcommand\thefootnote{\@fnsymbol\c@footnote}`),
  // unless a binding declares its title's own (`TITLE_FOOTNOTE_HOOK`)
  DefMacro!("\\lx@fm@obs@thefootnote", "\\@fnsymbol\\c@footnote");
  DefConstructor!("\\lx@fm@obs@mark {}", "");
  // a binding's request of an annotation, printing nothing (elsarticle's `\corref{cor1}`, `\fnref{fn1}`:
  // `\lx@request@frontmatter@annotation[cor]{cor1}`, elsart_support_core_sty.rs:58-60): a mark labelled
  // `<prefix>:<label>` (S2a.2)
  DefConstructor!("\\lx@fm@obs@request []{}", "");
  DefConstructor!("\\lx@fm@obs@label Semiverbatim", "");
  DefConstructor!("\\lx@fm@obs@email [] Semiverbatim", "");
  DefConstructor!("\\lx@fm@obs@url Semiverbatim", "");
  DefConstructor!("\\lx@fm@obs@href Semiverbatim {}", "");
  DefConstructor!("\\lx@fm@obs@orcid [] Semiverbatim", "");
  DefConstructor!("\\lx@fm@obs@affil []{}", "");
  DefConstructor!("\\lx@fm@obs@annotate {}{}[]{}", "");
  Ok(())
}

thread_local! {
  /// The footnote steps the block being rehearsed made so far (`\thanks`, `\footnote` and bare `\footnotemark`s),
  /// from 0 at each rehearsal.
  static FOOTNOTE_STEPS: Cell<i64> = const { Cell::new(0) };
  /// `\c@footnote` at the rehearsal's start, and whether the format has it (and `\thefootnote`).
  static FOOTNOTE_BASE: Cell<i64> = const { Cell::new(0) };
  static FOOTNOTE_BASE_DEFINED: Cell<bool> = const { Cell::new(false) };
}

/// The commands the rehearsal binds to the event constructors, always (the article `\author` line's own vocabulary,
/// and the frontmatter API's queueing sink, frozen otherwise).
const ALWAYS_BOUND: &[(&str, &str)] = &[
  ("\\and", "\\lx@fm@obs@and"),
  ("\\And", "\\lx@fm@obs@And"),
  ("\\AND", "\\lx@fm@obs@AND"),
  ("\\\\", "\\lx@fm@obs@break"),
  ("\\newline", "\\lx@fm@obs@break"),
  ("\\linebreak", "\\lx@fm@obs@break"),
  ("\\par", "\\lx@fm@obs@break"),
  ("\\thanks", "\\lx@fm@obs@thanks"),
  ("\\footnote", "\\lx@fm@obs@thanks"),
  ("\\footnotetext", "\\lx@fm@obs@notetext"),
  ("\\footnotemark", "\\lx@fm@obs@footnotemark"),
  ("\\label", "\\lx@fm@obs@label"),
  // (inside author content `\inst` is the affiliation request, `\lx@author@withinst`)
  ("\\inst", "\\lx@fm@obs@mark"),
  ("\\lx@annotate@frontmatter", "\\lx@fm@obs@annotate"),
  ("\\lx@request@frontmatter@annotation", "\\lx@fm@obs@request"),
];

/// The contact and mark commands the rehearsal binds when the document has them (an undefined one stays undefined: an
/// error, held, as pdflatex's), and whether the command's own definition must take an argument for it: a contact
/// command taking none is a switch, not that contact (jmlr2e's `\email`, `\name Ann \email ann@x.org`, a font change
/// a one-argument binding read as the e-mail "a": 2412.18293, 2305.17297; S2a.2 r2) — but url.sty's `\url` reads its
/// argument itself, its macro taking none.
const BOUND_WHEN_DEFINED: &[(&str, &str, bool)] = &[
  ("\\email", "\\lx@fm@obs@email", true),
  ("\\emailAdd", "\\lx@fm@obs@email", true),
  ("\\url", "\\lx@fm@obs@url", false),
  ("\\homepage", "\\lx@fm@obs@url", false),
  ("\\href", "\\lx@fm@obs@href", false),
  ("\\orcidlink", "\\lx@fm@obs@orcid", true),
  ("\\orcid", "\\lx@fm@obs@orcid", true),
  ("\\affil", "\\lx@fm@obs@affil", true),
  ("\\affiliation", "\\lx@fm@obs@affil", true),
  ("\\address", "\\lx@fm@obs@affil", true),
  ("\\institute", "\\lx@fm@obs@affil", true),
  ("\\IEEEauthorrefmark", "\\lx@fm@obs@mark", true),
];
/// The hook a binding declares its title's footnote marks with — the kernel's own, `\lx@title@thefootnote`
/// (`latex_constructs::TITLE_THEFOOTNOTE`), which also marks the notes the title block prints: defined, the
/// rehearsal's `\thefootnote` is it (what a bare `\footnotemark` in the block prints); undefined, the kernel
/// `\maketitle`'s `\@fnsymbol\c@footnote`. Article, report and book define it (`declare_fnsymbol_title_block`); a class
/// numbering its title's marks otherwise defines it — revtex4-2.cls:3040-3041 `\frontmatter@thefootnote` is
/// `\@fnsymbol` by default and `\@alph` only under its `altaffilletter` option (:5978 `\@booleanfalse
/// \altaffilletter@sw`), so a revtex binding would `\def\lx@title@thefootnote{\frontmatter@thefootnote}`. Guards
/// `perfect_kernel_batchfms2a1::title_footnote_hook_overrides_fnsymbol`,
/// `perfect_kernel_batch78o::declared_title_thefootnote_prints_the_marks_the_observer_labels`.
const TITLE_FOOTNOTE_HOOK: &str = crate::latex_constructs::TITLE_THEFOOTNOTE;

/// Bind the Layer-B event commands, in the rehearsal's group, and the title's footnote marks: `\thefootnote` the
/// declared [`TITLE_FOOTNOTE_HOOK`] when there is one; else, as the block's policy entry says (`title_marks`), the
/// class's own `\thefootnote` (report's and book's title page: arabic) or the kernel `\maketitle`'s `\@fnsymbol` of the
/// counter (when the format has it).
fn install_event_bindings(title_marks: TitleMarks) -> Result<()> {
  for (cs, target) in ALWAYS_BOUND {
    let_i(&T_CS!(*cs), &T_CS!(*target), None);
  }
  if lookup_definition(&T_CS!(TITLE_FOOTNOTE_HOOK))?.is_some() {
    let_i(&T_CS!("\\thefootnote"), &T_CS!(TITLE_FOOTNOTE_HOOK), None);
  } else if title_marks == TitleMarks::Fnsymbol
    && lookup_definition(&T_CS!("\\@fnsymbol"))?.is_some()
  {
    let_i(
      &T_CS!("\\thefootnote"),
      &T_CS!("\\lx@fm@obs@thefootnote"),
      None,
    );
  }
  for (cs, target, needs_argument) in BOUND_WHEN_DEFINED {
    if lookup_definition(&T_CS!(*cs))?
      .is_some_and(|defn| !needs_argument || defn.get_num_args() > 0)
    {
      let_i(&T_CS!(*cs), &T_CS!(*target), None);
    }
  }
  Ok(())
}

/// What a rehearsal gave.
#[derive(Debug)]
pub(in crate::base_utilities) enum Rehearsal<R> {
  /// The callback's value on the digested list, and the errors and warnings the rehearsal raised (held, then
  /// discarded with it).
  Done {
    value:    R,
    errors:   usize,
    warnings: usize,
  },
  /// No rehearsal was made or kept, and why: no journal could open (one is already open), its assignments could not be
  /// undone, or it ended in a Fatal error of its own (discarded with it).
  Skipped(SkipReason),
}

/// Digest `tokens` (a block's stamped tokens, `ledger::stamped_block_tokens`) under the event bindings and hand the
/// digested list to `inspect` before discarding it; nothing of it stays. A resource Fatal (the deadline, the memory
/// budget, a runaway limit) propagates, as does one whose run could not be undone (Fatal stays Fatal); any other Fatal
/// is the rehearsal's alone, discarded with its diagnostics.
pub(in crate::base_utilities) fn rehearse<R>(
  tokens: Tokens,
  title_marks: TitleMarks,
  inspect: impl FnOnce(&Digested) -> R,
) -> Result<Rehearsal<R>> {
  // (read before anything is changed: the counter the bare `\footnotemark`s print from)
  let footnotes = lookup_definition(&T_CS!("\\c@footnote"))?.is_some()
    && lookup_definition(&T_CS!("\\thefootnote"))?.is_some();
  let base = if footnotes {
    counter_value("footnote")?.value_of()
  } else {
    0
  };
  let Some(journal) = begin_assignment_journal() else {
    return Ok(Rehearsal::Skipped(SkipReason::JournalUnavailable));
  };
  FOOTNOTE_BASE_DEFINED.with(|defined| defined.set(footnotes));
  FOOTNOTE_BASE.with(|cell| cell.set(base));
  const FLAGS: [&str; 3] = [
    "lx_replaying_class_setter",
    "lx_depositing_frontmatter_fields",
    "lx_depositing_class_maketitle",
  ];
  let saved: Vec<bool> = FLAGS.iter().map(|flag| lookup_bool(flag)).collect();
  for flag in FLAGS {
    assign_value(flag, true, Some(Scope::Global));
  }
  let hold = util::logger::DiagnosticsHold::begin();
  let resource_fatal_before = resource_fatal_latched();
  let muted = SuppressLogOutputGuard::new(true);
  // (an `\input` in the field opens its file again: its `(Processing content …` note is not the conversion's)
  let silent = SilenceProgressNotesGuard::new();
  let depth = get_frame_depth();
  let boxes = box_lists_mark();
  let conditionals = if_stack_depth();
  let ledger_scope = FieldLedgerScope::begin();
  REHEARSING.with(|r| r.set(true));
  FOOTNOTE_STEPS.with(|steps| steps.set(0));
  // The whole group in a reading context of its own: what its end unreads (an `\aftergroup` of the block's,
  // `pop_stack_frame_with`) lands in this mouth, closed with whatever is left (`reading_from_mouth`'s forced close),
  // never in the document's input; and the token the group's end and its diagnostics see is the rehearsal's own, not
  // the document's current one (a `}` closing an alignment template's brace, `egroup`'s `close_template_brace`).
  // Guards: the repro `sectioning-frontmatter/observer_rehearsal_aftergroup_stays_inside` (its `\aftergroup\leak`
  // printed "LEAKED" into the body before this wrap) and `perfect_kernel_batchfms2a1::
  // shadow_switch_leaves_the_xml_unchanged` (XML and log the same with the switch on and off over the adversarial
  // shapes).
  local_current_token(T_CS!("\\relax"));
  let result = with_origin(DefinitionOrigin::File, || {
    reading_from_mouth(Mouth::default(), || -> Result<Digested> {
      bgroup();
      install_event_bindings(title_marks)?;
      begin_mode("text")?;
      let digested = digest(tokens)?;
      end_mode("text")?;
      egroup()?;
      Ok(digested)
    })
  });
  let unwound = reading_from_mouth(Mouth::default(), || {
    new_local_box_list();
    while get_frame_depth() > depth {
      let before = get_frame_depth();
      let nobox = lookup_bool("groupNonBoxing");
      if pop_stack_frame(nobox).is_err() || get_frame_depth() >= before {
        break;
      }
    }
    expire_local_box_list();
    Ok(())
  });
  expire_current_token();
  let value = match &result {
    Ok(digested) => Some(inspect(digested)),
    Err(_) => None,
  };
  REHEARSING.with(|r| r.set(false));
  drop(ledger_scope);
  drop(silent);
  drop(muted);
  let errors = hold.errors_raised();
  let warnings = hold.warnings_raised();
  let undone = journal.rollback();
  for (flag, value) in FLAGS.iter().zip(saved) {
    assign_value(flag, value, Some(Scope::Global));
  }
  if let Err(err) = result {
    if !undone || (resource_fatal_latched() && !resource_fatal_before && !unlatch_runaway_fatal()) {
      hold.commit();
      return Err(err);
    }
    hold.discard();
    unwound?;
    unwind_box_lists_to(boxes);
    while if_stack_depth() > conditionals {
      pop_if_frame();
    }
    return Ok(Rehearsal::Skipped(SkipReason::Fatal));
  }
  if undone {
    hold.discard();
  } else {
    // (its diagnostics and its global assignments stay, as a speculative run's that could not be undone)
    hold.commit();
  }
  unwound?;
  Ok(match value {
    Some(value) if undone => Rehearsal::Done { value, errors, warnings },
    _ => Rehearsal::Skipped(SkipReason::NotUndone),
  })
}
