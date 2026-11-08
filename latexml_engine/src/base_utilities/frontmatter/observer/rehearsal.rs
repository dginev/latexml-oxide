//! The observer's rehearsal: a block's stamped tokens digested as the article `\author` line prints them, with
//! Layer-B event bindings (Perl's `@authorsplits`, Base_Utility.pool.ltxml:676-683: `\and` a column, `\\` a row; the
//! notes and contacts as events), through the speculative-run engine (sect05.rs `speculative_title_page_run`):
//! assignments rolled back (`AssignmentJournalGuard`, a meaning's fallback standing with it: FM S2a.2 r9, 2409.10200),
//! diagnostics held (`DiagnosticsHold`), terminal output muted, the frontmatter frozen (`lx_replaying_class_setter`:
//! nothing is queued), the field ledger put back and the boxes it stamped unstamped (`FieldLedgerScope`: a register's
//! box `\usebox` hands back, FM S2a.2 r9, 2405.14627, 2603.26792), progress notes silenced (an `\input` in the field:
//! `SilenceProgressNotesGuard`). The
//! digested list is handed to a callback before it is discarded. Not LaTeX's `\begin{tabular}[t]{c}` around the
//! line: no alignment machinery runs, and a stray `&` is digested as a whatsit that errs only when built into the
//! document (tex_tables.rs `&`), never here: nothing is held for it, and the lexer reads it as a column separator.
//! What its boxes do not say is noted beside them (`field_notes::FieldNotes`): the definitions made, the errors held, by the
//! field position of the invocation.

use std::cell::Cell;

use latexml_core::{
  definition::origin::{DefinitionOrigin, with_origin},
  token::FieldLedgerScope,
};

use super::{
  events::{
    BreakKind, EmailMeaning, Event, HeldError, break_events, declaration_events, in_block_content,
  },
  field_notes::{
    EnvironmentStep, FieldCreator, FieldNotesHookGuard, field_notes_begin, field_notes_end,
    note_creator_call,
  },
  ledger::{SkipReason, email_meaning},
  policy::TitleMarks,
  *,
};

thread_local! {
  static REHEARSING: Cell<bool> = const { Cell::new(false) };
}

/// Whether a rehearsal is digesting now (the ledger records nothing of it).
pub(in crate::base_utilities) fn rehearsing() -> bool { REHEARSING.with(Cell::get) }

/// No rehearsal is digesting (a conversion's start: one a panic left marked on a reused thread is over).
pub(in crate::base_utilities) fn reset_rehearsing() {
  REHEARSING.with(|r| r.set(false));
  field_notes_end();
  reset_rehearsal_budget();
}

/// The most tokens one rehearsal reads, as many as a run of the title-page check's (`title_page_check::CHECK_RUN_READS_DEFAULT`). By the
/// data (the census, `FRONTMATTER_SINGLE_ENGINE_PLAN.md` "What a rollback leaves"): popZ01's 1,707 papers rehearsing read 166 at the median in their dearest rehearsal, 8,417 at the 99th percentile and
/// 3,359,702 at most (2608.18287; 2011.00785 3,314,577); by what it costs, a looping block reads this in about two seconds.
const REHEARSAL_READS_DEFAULT: usize = 8_000_000;

/// The most tokens all the rehearsals of a conversion read, in [`REHEARSAL_READS_DEFAULT`]s: a paper rehearses per call. popZ01 reads 731 in a
/// conversion at the median, 20,444 at the 99th percentile, and the most 3,360,403 (2608.18287; 2011.00785 3,314,871). The many-author witnesses are far under it:
/// 1901.01540's 2,637 rehearsals read 118,996 (1,202 authors), 2103.04797's 2,136 read 92,093 (975), 2005.08745's 668 read 13,009 (592).
const REHEARSALS_PER_CONVERSION: usize = 3;

thread_local! {
  /// The reads one rehearsal may make, this conversion's ([`rehearsal_budget`]).
  static RUN_READS: Cell<usize> = const { Cell::new(REHEARSAL_READS_DEFAULT) };
  /// The reads the conversion's rehearsals may still make.
  static READS_LEFT: Cell<usize> =
    const { Cell::new(REHEARSAL_READS_DEFAULT * REHEARSALS_PER_CONVERSION) };
  /// How many rehearsals were abandoned or never made over budget.
  static LEFT_OUT: Cell<usize> = const { Cell::new(0) };
}

/// A new conversion's budget: [`REHEARSAL_READS_DEFAULT`], or `LATEXML_REHEARSAL_READ_BUDGET` (0: none).
fn reset_rehearsal_budget() {
  let reads = std::env::var("LATEXML_REHEARSAL_READ_BUDGET")
    .ok()
    .and_then(|reads| reads.trim().parse::<usize>().ok())
    .unwrap_or(REHEARSAL_READS_DEFAULT);
  let reads = if reads == 0 { usize::MAX >> 2 } else { reads };
  RUN_READS.set(reads);
  READS_LEFT.set(reads.saturating_mul(REHEARSALS_PER_CONVERSION));
  LEFT_OUT.set(0);
}

/// How many tokens the next rehearsal may read: a block's text may loop where nothing prints it (a first `\author` a second
/// replaces, a class that drops it, `\author{First\loop}`), and a resource limit met here ends a conversion pdflatex finishes.
/// None once the conversion's reads are spent. What is left out so is said once ([`rehearsal_budget_warning`]).
pub(in crate::base_utilities) fn rehearsal_budget() -> Option<usize> {
  let left = READS_LEFT.get();
  (left != 0).then(|| left.min(RUN_READS.get()))
}

/// The Warning that says what no rehearsal read, when the document closes: its creators are read as they print, not from a
/// reading of the block (`SkipReason::OverBudget`).
pub(in crate::base_utilities) fn rehearsal_budget_warning() {
  let left_out = LEFT_OUT.replace(0);
  if left_out > 0 {
    Warn!(
      "frontmatter",
      "rehearsal_budget",
      s!(
        "The rehearsal gave up on {left_out} frontmatter call(s) that take more than {} reads to digest, or past the conversion's {}: they are read as they print",
        RUN_READS.get(),
        RUN_READS.get().saturating_mul(REHEARSALS_PER_CONVERSION)
      )
    );
  }
}

/// The event constructors of the Layer-B bindings ([`install_event_bindings`]): whatsits the lexer reads
/// (`events::lex_events`), never absorbed (a rehearsal's list is discarded), so they construct nothing. The `\and`
/// family keeps which command it was (`\AND` a new row in NeurIPS's and ICLR's `\author`); `\thanks`, `\footnote` and
/// `\footnotemark` count the footnote steps they make in LaTeX (`FOOTNOTE_STEPS`), never stepping `\c@footnote`.
pub(in crate::base_utilities) fn load() -> Result<()> {
  // (each reverting as the command it is bound for, `alias`: a text made of an argument holding one — an argument read
  // verbatim, svjour3's `\email{yjansson@kth.se \and tony@kth.se}`, 2106.06418; a link's target, lni's `\orcid`,
  // 2106.11154, 1401.5951 — keeps its `\and`, `\\` or `\par`, never the rehearsal's name: review revN2)
  DefConstructor!("\\lx@fm@obs@and", "", alias => "\\and");
  DefConstructor!("\\lx@fm@obs@And", "", alias => "\\And");
  DefConstructor!("\\lx@fm@obs@AND", "", alias => "\\AND");
  DefConstructor!("\\lx@fm@obs@break OptionalMatch:* []", "", alias => "\\\\");
  // (its own, for the Break to say so; a contact's raw argument holding it reads it as the break it is, `events::raw_lines`:
  // 2106.11154's `\orcid`)
  DefConstructor!("\\lx@fm@obs@par OptionalMatch:* []", "", alias => "\\par");
  // `\thanks` and `\footnote` (inside `\maketitle`, `\footnote` is `\thanks`): latex.ltx `\thanks` = `\footnotemark`
  // + the text, a step of the footnote counter unless an optional number is given
  DefMacro!("\\lx@fm@obs@thanks []", sub[(number)] { note_step(T_CS!("\\lx@fm@obs@note"), number) });
  // (the two tell apart what they make in an author's content, `digest_frontmatter_item`'s `Let`: `Event::footnote`)
  DefMacro!("\\lx@fm@obs@footnote []", sub[(number)] { note_step(T_CS!("\\lx@fm@obs@fnnote"), number) });
  // (its `printed` property the mark it shows on the name, `\thefootnote` at its number — the given one, or the counter's
  // value the `\lx@fm@obs@thanks` step reached —, as `\lx@fm@obs@fnmark` prints its: a name's note mark is the request
  // an entry led by the same `\footnotemark` answers, `Shun'ya Mizoguchi\footnote[1]{…}` under
  // `\affiliation{\footnotemark[1]Theory Center, …}`, 1607.07280 — FM S2a.2 r7)
  DefConstructor!("\\lx@fm@obs@note []{}", "", properties => sub[args] { note_properties(args.first().and_then(|a| a.as_ref())) });
  DefConstructor!("\\lx@fm@obs@fnnote []{}", "", properties => sub[args] { note_properties(args.first().and_then(|a| a.as_ref())) });
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
    // (read quietly too: a bare `\footnotemark` under a class whose `\@fnsymbol` reads `\thefootnote` must not end the
    // block's rehearsal — review revfms2a2_7 finding 4, jpconf.cls:153)
    let printed = note_mark_quietly(value)?;
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
  // (reverting as `\email`, as the `\and` family does: a note's text holding one, `\thanks{\email{…}}`, 1401.5951)
  DefConstructor!("\\lx@fm@obs@email [] Semiverbatim", "", alias => "\\email");
  // (an e-mail call by the class's binding printing in place, and by no binding's meaning: `ledger::email_meaning`)
  DefConstructor!("\\lx@fm@obs@email@inplace [] Semiverbatim", "", alias => "\\email");
  DefConstructor!("\\lx@fm@obs@email@own [] Semiverbatim", "", alias => "\\email");
  DefConstructor!("\\lx@fm@obs@url Semiverbatim", "");
  DefConstructor!("\\lx@fm@obs@href Semiverbatim {}", "");
  DefConstructor!("\\lx@fm@obs@orcid [] Semiverbatim", "");
  // (in a group of its own, as the API digests its entry, `digest_frontmatter_item`: a switch in one affiliation is
  // no font of the next one's or of the names after it — `\affil{\itshape $^1$…}\affil{\textit{$^3$…}}`, whose
  // second's italic read as no change of its own, `events::entry_font`; review rev94b r3 B1)
  DefConstructor!("\\lx@fm@obs@affil []{}", "", bounded => true);
  // (the affiliation of a raw file's in-place setter the API took, `annotate=run`: `frontmatter_stores`'
  // `reroute_in_place_contact_setters`, whose rerouted setter keeps the file's own as `\lx@class@contact@<name>`)
  DefConstructor!("\\lx@fm@obs@affil@run []{}", "", bounded => true);
  // (its options undigested, as `\lx@annotate@frontmatter` reads keyvals: a private key's `_` digested as text is a Script
  // error held on the block, the paper's writer kept out)
  DefConstructor!("\\lx@fm@obs@annotate {}{} OptionalUndigested {}", "");
  // (a binding's marked list, `\lx@add@affiliation@marked`, is an in-block affiliation: the marks and lines of its
  // content read as digested, not cut by spelling and handed to the frozen API — `\IEEEauthorblockA{\IEEEauthorrefmark{6}
  // Univ X\\ \IEEEauthorrefmark{2} Univ Y}` in a plain `\author`, 2603.28168, 1405.3507)
  DefConstructor!("\\lx@fm@obs@affil@marked OptionalUndigested {}", "", bounded => true);
  // (a binding's own creator inside a block prints its name there: PoS's `\speaker{R. Cassano}$^1$`, PoS.cls:681, 1412.5940,
  // 1509.01480, 1211.4334 — the frozen API printed nothing, and the name's mark led the line. Not the `\lx@add@author` of an
  // `\author` nested in a block, a call the class stores and prints no name of there: 2012.14750)
  // (its keyvals, read as the API reads them, for the Name it prints, which says them nowhere: `\speaker`'s `role=speaker`,
  // 0910.5686, 1007.2355, 1111.2428, 1111.6944 — written `role=author`; ams `\contrib`'s `name=`)
  DefMacro!("\\lx@fm@obs@creator OptionalKeyVals:Frontmatter {}", sub[(keys, name)] {
    if let Some(keys) = &keys {
      note_creator_call(keys, &name)?;
    }
    Ok(name)
  });
  Ok(())
}

/// A `\thanks` or `\footnote`'s step of the footnote counter (unless it gives a number), then its constructor `note`.
fn note_step(note: Token, number: Option<Tokens>) -> Result<Tokens> {
  if number.is_none() {
    FOOTNOTE_STEPS.with(|steps| steps.set(steps.get().saturating_add(1)));
  }
  let mut out = vec![note];
  if let Some(number) = number {
    out.push(T_OTHER!("["));
    out.push(T_BEGIN!());
    out.extend(number.unlist());
    out.push(T_END!());
    out.push(T_OTHER!("]"));
  }
  Ok(Tokens::new(out))
}

/// The properties of a note's constructor: the mark it shows on the name.
fn note_properties(number: Option<&Digested>) -> Result<SymHashMap<Stored>> {
  let given = number.map(|number| {
    let mut text = String::new();
    ink_text(number, &mut text);
    text.trim().to_string()
  });
  let value = match given {
    Some(text) => match text.parse::<i64>() {
      Ok(value) => value,
      Err(_) => return Ok(stored_map!("printed" => Stored::String(pin(&text)))),
    },
    None => FOOTNOTE_BASE
      .with(Cell::get)
      .saturating_add(FOOTNOTE_STEPS.with(Cell::get)),
  };
  let printed = note_mark_quietly(value)?;
  Ok(stored_map!("printed" => Stored::String(pin(&printed))))
}

/// The footnote mark `\thefootnote` prints with the counter at `value`, in a group of its own (the rehearsal steps
/// nothing); the number as it is where the format has no LaTeX footnote counter.
fn printed_footnote_mark(value: i64) -> Result<String> {
  if !FOOTNOTE_BASE_DEFINED.with(Cell::get) {
    return Ok(value.to_string());
  }
  let mut print = vec![T_BEGIN!(), T_CS!("\\c@footnote")];
  print.extend(Explode!(value));
  print.extend([T_CS!("\\relax"), T_CS!("\\thefootnote"), T_END!()]);
  let mut text = String::new();
  ink_text(&digest(Tokens::new(print))?, &mut text);
  Ok(text.trim().to_string())
}

/// The mark a note shows ([`printed_footnote_mark`]), read for the resolver alone: its diagnostics set aside — the
/// conversion's own `\thanks` raises what the document raises —, and a print that fails unwound where it stopped, the
/// note then showing no mark: a mark past `\@fnsymbol`'s nine ("Counter too large", eleven `\thanks` in one
/// `\author`) must not hold the block's rehearsal, nor a runaway print end it (jpconf.cls:153 `\def\@fnsymbol#1{\ifnum
/// \thefootnote=99…}` under the rehearsal's `\thefootnote`, `\@fnsymbol\c@footnote`, recurses: 1105.2659's
/// `\footnote[3]{Speaker}` left 12,000 conditionals open and the conversion Fatal). A resource Fatal that is no runaway
/// stays one.
fn note_mark_quietly(value: i64) -> Result<String> {
  let hold = util::logger::DiagnosticsHold::begin();
  let latched_before = resource_fatal_latched();
  let depth = get_frame_depth();
  let boxes = box_lists_mark();
  // (the tokens being invoked and the conditionals put back however the print ends, after a failed print's unwinding:
  // `StomachMark`)
  let stomach_mark = StomachMark::take();
  let printed = printed_footnote_mark(value);
  let failed = hold.errors_raised() > 0;
  match printed {
    Ok(printed) => {
      stomach_mark.restore();
      hold.discard();
      Ok(if failed { String::new() } else { printed })
    },
    Err(err) => {
      if resource_fatal_latched() && !latched_before && !unlatch_runaway_fatal() {
        hold.commit();
        return Err(err);
      }
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
      unwind_box_lists_to(boxes);
      stomach_mark.restore();
      hold.discard();
      unwound?;
      Ok(String::new())
    },
  }
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
  ("\\par", "\\lx@fm@obs@par"),
  ("\\thanks", "\\lx@fm@obs@thanks"),
  ("\\footnote", "\\lx@fm@obs@footnote"),
  ("\\footnotetext", "\\lx@fm@obs@notetext"),
  ("\\footnotemark", "\\lx@fm@obs@footnotemark"),
  ("\\label", "\\lx@fm@obs@label"),
  // (inside author content `\inst` is the affiliation request, `\lx@author@withinst`)
  ("\\inst", "\\lx@fm@obs@mark"),
  ("\\lx@annotate@frontmatter", "\\lx@fm@obs@annotate"),
  ("\\lx@request@frontmatter@annotation", "\\lx@fm@obs@request"),
];

/// What a block's own API calls are in its rehearsal, read as digested: an API call's content, rehearsed alone, has no names
/// of its own (a `\lx@add@creator` a class's address holds prints nothing there, 2203.02446, 1709.02726, 2411.10888).
const BOUND_IN_A_BLOCK: &[(&str, &str)] = &[
  ("\\lx@add@affiliation@marked", "\\lx@fm@obs@affil@marked"),
  ("\\lx@add@creator", "\\lx@fm@obs@creator"),
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
  // (the ACM SIG classes' column, a raw class's tabular juggle: sig-alternate.cls:283 `\def\alignauthor{\end{tabular}\hfil
  // \begin{tabular}[t]{c}}`, four errors held on the block outside its tabular and every column read as one, 1503.06772,
  // 1502.04662, 1305.7403; a binding that declares it keeps what it prints, `Policy::block_breaks`, OmniBus)
  ("\\alignauthor", "\\lx@fm@obs@and", false),
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
/// declared [`TITLE_FOOTNOTE_HOOK`] when there is one; else, as the block's conventions say (`title_marks`), the
/// class's own `\thefootnote` (report's and book's title page: arabic) or the kernel `\maketitle`'s `\@fnsymbol` of the
/// counter (when the format has it).
fn install_event_bindings(
  title_marks: TitleMarks,
  block_breaks: &[(&str, BreakKind)],
) -> Result<()> {
  for (cs, target) in ALWAYS_BOUND {
    let_i(&T_CS!(*cs), &T_CS!(*target), None);
  }
  if in_block_content() {
    for (cs, target) in BOUND_IN_A_BLOCK {
      let_i(&T_CS!(*cs), &T_CS!(*target), None);
    }
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
    if !block_breaks.iter().any(|(declared, _)| declared == cs)
      && lookup_definition(&T_CS!(*cs))?
        .is_some_and(|defn| !needs_argument || defn.get_num_args() > 0)
    {
      // (an e-mail call is read with whose meaning it has here, when the note is rehearsed — at the flush,
      // `\maketitle`, not at the call (review rev92a_r4 B1, OD #913): `\lx@fm@obs@email` a binding's, the `@inplace`
      // and `@own` variants the class's printing in place and no binding's — review rev92a_r3 B1)
      let target = if *target == "\\lx@fm@obs@email" {
        match email_meaning(cs) {
          EmailMeaning::Binding => *target,
          EmailMeaning::InPlace => "\\lx@fm@obs@email@inplace",
          EmailMeaning::Own => "\\lx@fm@obs@email@own",
        }
      } else if *target == "\\lx@fm@obs@affil"
        && lookup_definition(&T_CS!(&s!("\\lx@class@contact@{}", &cs[1..])))?.is_some()
      {
        "\\lx@fm@obs@affil@run"
      } else {
        *target
      };
      let_i(&T_CS!(*cs), &T_CS!(target), None);
    }
  }
  Ok(())
}

/// What a rehearsal gave.
#[derive(Debug)]
pub(in crate::base_utilities) enum Rehearsal<R> {
  /// The callback's value on the digested list, the errors and warnings the rehearsal raised (held, then discarded with
  /// it), and what its boxes do not say.
  Done {
    value:        R,
    errors:       usize,
    warnings:     usize,
    /// the definitions made in the tokens' positions ([`declaration_events`]; none for tokens without positions)
    declarations: Vec<Event>,
    /// the block breaks its binding declares, by the positions they were called at ([`break_events`]; `Policy::block_breaks`)
    breaks:       Vec<Event>,
    /// the errors held, by the positions they were raised at, in order (none positioned for tokens without positions)
    held:         Vec<HeldError>,
    /// the environments its tokens open and close ([`EnvironmentStep`])
    environments: Vec<EnvironmentStep>,
    /// the creator calls its own expansion made, which no box says the keyvals of ([`FieldCreator`])
    creators:     Vec<FieldCreator>,
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
  block_breaks: &[(&str, BreakKind)],
  inspect: impl FnOnce(&Digested) -> R,
) -> Result<Rehearsal<R>> {
  // (read before anything is changed: the counter the bare `\footnotemark`s print from)
  let footnotes = lookup_definition(&T_CS!("\\c@footnote"))?.is_some()
    && lookup_definition(&T_CS!("\\thefootnote"))?.is_some();
  let counter = if footnotes {
    counter_value("footnote")?.value_of()
  } else {
    0
  };
  // (and the steps the blocks rehearsed before made, while the conversion has not stepped the counter since: notes a
  // binding keeps as contacts — revtex's `\author{Ann\footnote{…}}`, one `\author` a block — print `\thefootnote` at
  // the counter TeX stepped for each, ∗, †, ‡ across the authors, 1608.06509; a counter the conversion moved itself,
  // stepped for the notes it made, starts the count afresh)
  let carried = if footnotes && lookup_int("lx_fm_obs_counter_seen") == counter {
    lookup_int("lx_fm_obs_steps_carried")
  } else {
    0
  };
  let base = counter + carried;
  let Some(journal) = begin_assignment_journal() else {
    return Ok(Rehearsal::Skipped(SkipReason::JournalUnavailable));
  };
  let Some(budget) = rehearsal_budget() else {
    journal.rollback();
    LEFT_OUT.set(LEFT_OUT.get() + 1);
    return Ok(Rehearsal::Skipped(SkipReason::OverBudget));
  };
  let _speculative = speculative_digestion();
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
  begin_meaning_log();
  let resource_fatal_before = resource_fatal_latched();
  let muted = SuppressLogOutputGuard::new(true);
  // (an `\input` in the field opens its file again: its `(Processing content …` note is not the conversion's)
  let silent = SilenceProgressNotesGuard::new();
  let depth = get_frame_depth();
  let boxes = box_lists_mark();
  // (the tokens being invoked put back however the run ends; the conditionals are the sequence's, put back by the
  // shadow after the last block — a block's run may open one the next block's closes, as TeX reads the list in one
  // pass, 2606.17718 — and only those of a failed run here: `StomachMark`)
  let tokens_depth = token_stack_depth();
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
  let _hook = FieldNotesHookGuard;
  let if_count_before = if_count();
  let budget = read_budget(budget);
  let result = with_origin(DefinitionOrigin::File, || {
    reading_from_mouth(Mouth::default(), || -> Result<Digested> {
      bgroup();
      install_event_bindings(title_marks, block_breaks)?;
      begin_mode("text")?;
      field_notes_begin(&tokens, block_breaks);
      let digested = digest(tokens)?;
      end_mode("text")?;
      egroup()?;
      Ok(digested)
    })
  });
  // (off before anything else reads: closing what the run left open is no part of it)
  let budget = budget.end();
  READS_LEFT.set(READS_LEFT.get().saturating_sub(budget.reads));
  if budget.over {
    set_if_count(if_count_before);
  }
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
  truncate_token_stack(tokens_depth);
  expire_current_token();
  let notes = field_notes_end();
  let value = match &result {
    Ok(digested) => Some((
      inspect(digested),
      declaration_events(&notes.declarations),
      break_events(&notes.breaks),
    )),
    Err(_) => None,
  };
  REHEARSING.with(|r| r.set(false));
  drop(ledger_scope);
  drop(silent);
  drop(muted);
  let errors = hold.errors_raised();
  let warnings = hold.warnings_raised();
  // (each held message is stripped of its colors to be read: only for a block that held an error)
  let held = if errors > 0 {
    held_errors(&notes.errors, errors, hold.held_errors())
  } else {
    Vec::new()
  };
  let undone = journal.rollback();
  for (flag, value) in FLAGS.iter().zip(saved) {
    assign_value(flag, value, Some(Scope::Global));
  }
  // (past the hold's end: `hold` keeps it otherwise)
  let reads = budget.reads;
  if footnotes {
    let steps = carried + FOOTNOTE_STEPS.with(Cell::get);
    assign_value(
      "lx_fm_obs_counter_seen",
      Stored::from(counter),
      Some(Scope::Global),
    );
    assign_value(
      "lx_fm_obs_steps_carried",
      Stored::from(steps),
      Some(Scope::Global),
    );
  }
  // (a block's own, never a Fatal, even when its assignments could not be undone: its diagnostics and those stay, as a failed
  // run's: `sect05.rs` `speculative_run`)
  if budget.over {
    if undone {
      hold.discard();
    } else {
      hold.commit();
    }
    DebugFeature!("frontmatter", "FRONT a rehearsal read {reads} tokens");
    LEFT_OUT.set(LEFT_OUT.get() + 1);
    unwound?;
    unwind_box_lists_to(boxes);
    while if_stack_depth() > conditionals {
      pop_if_frame();
    }
    return Ok(Rehearsal::Skipped(SkipReason::OverBudget));
  }
  if let Err(err) = result {
    if !undone || (resource_fatal_latched() && !resource_fatal_before && !unlatch_runaway_fatal()) {
      hold.commit();
      return Err(err);
    }
    hold.discard();
    DebugFeature!("frontmatter", "FRONT a rehearsal read {reads} tokens");
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
  DebugFeature!("frontmatter", "FRONT a rehearsal read {reads} tokens");
  unwound?;
  Ok(match value {
    Some((value, declarations, breaks)) if undone => Rehearsal::Done {
      value,
      errors,
      warnings,
      declarations,
      breaks,
      held,
      environments: notes.environments,
      creators: notes.creators,
    },
    _ => Rehearsal::Skipped(SkipReason::NotUndone),
  })
}

/// The errors a rehearsal held: each of its `errors` raised, in order, at the position noted (`noted`: the invocations
/// that raised them, each with how many), the rest past the last field token, each with its log line.
fn held_errors(noted: &[(u16, u16, usize)], errors: usize, lines: Vec<String>) -> Vec<HeldError> {
  let spans = noted
    .iter()
    .flat_map(|&(start, end, n)| std::iter::repeat_n(Some((start, end)), n))
    .chain(std::iter::repeat(None));
  let mut lines = lines.into_iter();
  spans
    .take(errors)
    .map(|span| HeldError {
      span,
      message: lines.next().unwrap_or_default(),
    })
    .collect()
}
