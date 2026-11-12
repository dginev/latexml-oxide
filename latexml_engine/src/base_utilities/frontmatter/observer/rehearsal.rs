//! The observer's rehearsal: a block's stamped tokens digested as the article `\author` line prints them, with
//! Layer-B event bindings (Perl's `@authorsplits`, Base_Utility.pool.ltxml:676-683: `\and` a column, `\\` a row; the
//! notes and contacts as events), through the speculative-run engine (sect05.rs `title_page_probe_run`):
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

use std::{
  cell::{Cell, RefCell},
  ops::Range,
};

use latexml_core::{
  definition::origin::{DefinitionOrigin, with_origin},
  token::FieldLedgerScope,
};

use super::{
  events::{
    BreakKind, EmailMeaning, Event, HeldError, break_events, declaration_events, in_block_content,
  },
  field_notes::{
    EnvironmentStep, FieldCreator, FieldNotesHookGuard, HeldExpansion, PrintingCall,
    field_notes_begin, field_notes_end, note_creator_call,
  },
  ledger::{SkipReason, Unsaid, annotated_parent, email_meaning},
  policy::TitleMarks,
  *,
};

thread_local! {
  static REHEARSING: Cell<bool> = const { Cell::new(false) };
  /// the entries the rehearsal's run would have queued besides its creators ([`note_frozen_entry`])
  static FROZEN_ENTRIES: RefCell<Vec<RawFrontmatter>> = const { RefCell::new(Vec::new()) };
}

/// Note, while a rehearsal digests, an entry `tag` its run would queue (`command`, the frozen frontmatter queues none:
/// `queue::queue_front_matter`) that makes no creator nor annotates one (`resolver::creator_call`): a frontmatter setter run
/// inside the block, a `\date` in `\author` (2503.02741), which `Rehearsal::Done::queued` says, as the queue would hold
/// it (the emitter queues it at the flush: `emitter::emit::recover_queued`).
pub(in crate::base_utilities) fn note_frozen_entry(
  tag: &str,
  attr: Option<&KeyVals>,
  command: &Tokens,
) {
  if rehearsing() && !resolver::creator_call(tag, &annotated_parent(command)) {
    let entry = (tag.to_string(), queued_attributes(attr), command.clone());
    FROZEN_ENTRIES.with(|entries| entries.borrow_mut().push(entry));
  }
}

/// Note, while a rehearsal digests, an entry `tag` its run digests where it stands (`\lx@add@frontmatter@until`: an
/// abstract or keywords environment), undone with the run, which no queue holds: noted with no command, never queued again
/// (`emitter::emit::recover_queued`). REVTeX 3's `\begin{abstract}` inside an `\address`'s minipage (cond-mat9801109).
pub(in crate::base_utilities) fn note_frozen_digest(tag: &str) {
  if rehearsing() {
    let entry = (
      tag.to_string(),
      TagAttrs::default(),
      Tokens::new(Vec::new()),
    );
    FROZEN_ENTRIES.with(|entries| entries.borrow_mut().push(entry));
  }
}

/// Whether a rehearsal is digesting now (the ledger records nothing of it).
pub(in crate::base_utilities) fn rehearsing() -> bool { REHEARSING.with(Cell::get) }

/// Whether the frontmatter's rehearsal is reading the author fields now: where a binding's command whose meaning is a
/// frontmatter fact makes the reading's event (IEEEtran's `\IEEEmembership`, a Contact the resolver gives the name it follows).
pub fn rehearsing_author_fields() -> bool { rehearsing() }

/// Whether the frontmatter is reading its author fields now: a rehearsal (the reading's events) or the flush that digests
/// what the emitter wrote from them (`lx_frontmatter_flushing`, `place.rs`). A binding's command printing a frontmatter fact
/// only there asks it (IEEEtran's `\IEEEmembership`; OXIDIZED_DESIGN_DIVERGENCES #1016).
pub fn reading_author_fields() -> bool { rehearsing() || lookup_bool("lx_frontmatter_flushing") }

/// No rehearsal is digesting (a conversion's start: one a panic left marked on a reused thread is over), nor is a class's
/// own e-mail command being printed for its label (one a panic inside the print left set would turn off every later
/// conversion's label reading on the thread: [`own_email_properties`], review revX2 4).
pub(in crate::base_utilities) fn reset_rehearsing() {
  REHEARSING.with(|r| r.set(false));
  READING_OWN_EMAIL.set(false);
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
  /// The reads the content check's own rehearsals may still make ([`for_the_check`]): one rehearsal's, apart from the
  /// conversion's.
  static CHECK_READS_LEFT: Cell<usize> = const { Cell::new(REHEARSAL_READS_DEFAULT) };
  /// The reads the argument queries (separators, marks) may still make, one shared allowance ([`for_a_query`]): one
  /// rehearsal's, apart from the conversion's and the check's.
  static QUERY_READS_LEFT: Cell<usize> = const { Cell::new(REHEARSAL_READS_DEFAULT) };
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
  CHECK_READS_LEFT.set(reads);
  QUERY_READS_LEFT.set(reads);
  LEFT_OUT.set(0);
}

/// Run `rehearsals` that read for the frontmatter content check alone (`reading::read_pending`'s calls a block's own
/// expansion made, `ApiCall::inside_events`) from the check's own allowance, one rehearsal's reads ([`RUN_READS`]) for
/// the conversion, never from the reads the output's rehearsals may make: a check never moves what is written (a later
/// call's or a later flush's block over budget for it: revtex's `\collaboration{…}` after an `\author`, its block's own
/// call, read before the next author's `\affiliation`, which lost its rehearsal). One it gives up on is in no budget
/// Warning: nothing written depends on it, and the check counts that call by its content's text (`ApiCall::unread`).
pub(in crate::base_utilities) fn for_the_check<R>(rehearsals: impl FnOnce() -> R) -> R {
  apart_from_the_reading(&CHECK_READS_LEFT, rehearsals)
}

/// Run an argument query (`separators::read_argument`, which the separators and the marks queries share) from the
/// queries' own allowance, one rehearsal's reads ([`RUN_READS`]) for the conversion, never from the reads the output's
/// rehearsals may make nor in their count of calls left out: a query moves no reading (four costly queries before
/// `\maketitle` once spent a small budget, and the block they preceded was written whole, its two authors one creator).
/// One it gives up on answers `None`, which its caller reads by spelling as before the query (the marks query's
/// fallback counted), in no budget Warning.
pub(in crate::base_utilities) fn for_a_query<R>(rehearsals: impl FnOnce() -> R) -> R {
  apart_from_the_reading(&QUERY_READS_LEFT, rehearsals)
}

/// Run `rehearsals` with `allowance` standing in for the conversion's reads ([`READS_LEFT`]), the count of calls left
/// out ([`LEFT_OUT`]) put back after: [`for_the_check`], [`for_a_query`].
fn apart_from_the_reading<R>(
  allowance: &'static std::thread::LocalKey<Cell<usize>>,
  rehearsals: impl FnOnce() -> R,
) -> R {
  let left_out = LEFT_OUT.get();
  READS_LEFT.set(allowance.replace(READS_LEFT.get()));
  let read = rehearsals();
  allowance.set(READS_LEFT.replace(allowance.get()));
  LEFT_OUT.set(left_out);
  read
}

/// How many tokens the next rehearsal may read: a block's text may loop where nothing prints it (a first `\author` a second
/// replaces, a class that drops it, `\author{First\loop}`), and a resource limit met here ends a conversion pdflatex finishes.
/// None once the conversion's reads are spent. What is left out so is said once ([`rehearsal_budget_warning`]).
pub(in crate::base_utilities) fn rehearsal_budget() -> Option<usize> {
  let left = READS_LEFT.get();
  (left != 0).then(|| left.min(RUN_READS.get()))
}

/// The Warning that says what no rehearsal read, when the document closes (`SkipReason::OverBudget`): an author block
/// among them is written whole (Perl's whole-block creator, `writer::writer_stays_out` (deleted)) and any other call's words are not
/// written — the content check counts them (`resolver::placed::content_check`, `ApiCall::unread`).
pub(in crate::base_utilities) fn rehearsal_budget_warning() {
  let left_out = LEFT_OUT.replace(0);
  if left_out > 0 {
    let read = "an author block among them is written whole, as Perl's whole-block creator, and any other call's words \
                are not written, the frontmatter content check counting them";
    Warn!(
      "frontmatter",
      "rehearsal_budget",
      s!(
        "The rehearsal gave up on {left_out} frontmatter call(s) that take more than {} reads to digest, or past the conversion's {}: {read}",
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
  // `\affiliation{\footnotemark[1]Theory Center, …}`, 1607.07280 — FM S2a.2 r7). Its text is text, read in text mode as
  // the note's content is in the document (Perl's `\lx@annotate@frontmatter` digests it at the frontmatter, `DigestText`;
  // LaTeX sets a footnote's text in a paragraph of its own): a note set in a formula, `Tianming Wei$^{2,3}\thanks{The
  // first two authors contributed equally}$` (2407.15815; 2603.20851's `$^1\thanks{Corresponding Author}$`), kept its
  // words' spaces, which math mode skipped ("Thefirsttwoauthorscontributedequally").
  DefConstructor!("\\lx@fm@obs@note []{}", "", bounded => true, mode => "text", properties => sub[args] { note_properties(args.first().and_then(|a| a.as_ref())) });
  DefConstructor!("\\lx@fm@obs@fnnote []{}", "", bounded => true, mode => "text", properties => sub[args] { note_properties(args.first().and_then(|a| a.as_ref())) });
  // `\footnotetext`: the text of a mark printed elsewhere, no step
  DefConstructor!("\\lx@fm@obs@notetext []{}", "", bounded => true, mode => "text");
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
  // (the label the class's own command prints before the address is read as it prints: `own_email_properties`; its
  // address read as the class's macro reads its `#1`, unexpanded before it is digested — sig-alternate.cls:462 on
  // `\email{\fontsize{11pt}{12pt}\selectfont …}`, whose `\@defaultunits` expanded ahead missed its `\@nnil` and `\@nil`
  // ran undefined, 1802.01191: `UnexpandedSemiverbatim`)
  DefConstructor!("\\lx@fm@obs@email@own [] UnexpandedSemiverbatim", "", alias => "\\email",
    properties => sub[args] { own_email_properties(OWN_EMAIL[0].2, args.get(1).and_then(|a| a.as_ref())) });
  DefConstructor!("\\lx@fm@obs@emailAdd@own [] UnexpandedSemiverbatim", "", alias => "\\email",
    properties => sub[args] { own_email_properties(OWN_EMAIL[1].2, args.get(1).and_then(|a| a.as_ref())) });
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
  // (a creator call an API call's content makes, outside a block — amsart's `\thanks{… \author{Bob}\address{Univ B}
  // \email{bob@b.org}}`, 0807.4720: Perl's digestion makes the creator there, and every `\address`/`\email` after it is
  // that creator's, `\lx@annotate@frontmatter`, Base_Utility.pool.ltxml:520-531): the queueing sink, whichever meaning
  // it has, called as it is, after the event it leaves, `EventKind::ApiCall` "ltx:creator" (`Resolver::api`'s
  // `ltx:pubnote`)
  DefConstructor!("\\lx@fm@obs@creatorcall", "");
  DefMacro!("\\lx@fm@obs@add OptionalUndigested {}", sub[(keys, tag)] {
    let mut out = Vec::new();
    if tag.to_string().trim() == "ltx:creator" {
      out.push(T_CS!(CREATOR_CALL));
    }
    out.push(T_CS!(SINK_MEANING));
    if let Some(keys) = keys {
      out.push(T_OTHER!("["));
      out.extend(keys.unlist());
      out.push(T_OTHER!("]"));
    }
    out.push(T_BEGIN!());
    out.extend(tag.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
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
  print.extend([T_CS!("\\relax"), T_CS!(THEFOOTNOTE), T_END!()]);
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
  print_quietly(|| printed_footnote_mark(value))
}

/// The text `print` reads from what it digests ([`note_mark_quietly`]'s reading, any print's): its diagnostics set
/// aside, a print that fails unwound where it stopped and read as nothing, a resource Fatal that is no runaway kept.
fn print_quietly(print: impl FnOnce() -> Result<String>) -> Result<String> {
  let hold = util::logger::DiagnosticsHold::begin();
  let latched_before = resource_fatal_latched();
  let depth = get_frame_depth();
  let boxes = box_lists_mark();
  // (the tokens being invoked and the conditionals put back however the print ends, after a failed print's unwinding:
  // `StomachMark`)
  let stomach_mark = StomachMark::take();
  let printed = print();
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

/// Whether digested material holds a frontmatter API call, as the rehearsal binds it (`\lx@annotate@frontmatter`).
fn calls_the_api(d: &Digested) -> bool {
  match d.data() {
    DigestedData::List(l) => l.borrow().boxes.iter().any(calls_the_api),
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      w.get_definition().get_cs_name() == "\\lx@fm@obs@annotate"
        || w.get_args().iter().flatten().any(calls_the_api)
        || w
          .get_body()
          .ok()
          .flatten()
          .is_some_and(|body| calls_the_api(&body))
    },
    _ => false,
  }
}

/// The class's own e-mail commands the rehearsal reads (`EmailMeaning::Own`: the document's or a raw class's meaning, no
/// binding's): each command, the binding it is read by, and where its own meaning is kept while the block is rehearsed,
/// for the binding to print ([`own_email_properties`]).
const OWN_EMAIL: [(&str, &str, &str); 2] = [
  (
    "\\email",
    "\\lx@fm@obs@email@own",
    "\\lx@fm@obs@email@print",
  ),
  (
    "\\emailAdd",
    "\\lx@fm@obs@emailAdd@own",
    "\\lx@fm@obs@emailAdd@print",
  ),
];

/// The properties of a class's own e-mail call: the label its command prints before its address (`name`), as it prints
/// it — the class's meaning (kept at `print`) digested on the call's address, quietly ([`print_quietly`]), the text its
/// ink holds before the address. jfm/jpp's `\def\email#1{Email address for correspondence: #1}` (2603.18265, 2310.05678,
/// 1805.11102, 1705.00517, 2401.17126, 2110.10346, 2608.21737): the contact's `name` "Email address for correspondence:",
/// the note's label as the class prints it (FIDELITY_CHARTER: label text is markup, never dropped). None where the
/// command prints nothing before the address, or prints no address at the call (amsart-like classes store it for a
/// list at the end: AIMS.cls's `\g@addto@macro\addresses`, 1903.05770), and for a meaning LaTeXML made.
fn own_email_properties(print: &str, address: Option<&Digested>) -> Result<SymHashMap<Stored>> {
  let Some(address) = address else {
    return Ok(SymHashMap::default());
  };
  let mut text = String::new();
  ink_text(address, &mut text);
  let address = text.trim().to_string();
  // (a raw file's or the document's meaning, which prints as it is: never a binding's guess, whose meaning is the
  // frontmatter API's call — elsarticle's fallback `\email` = `\lx@add@contact[role=email]{#1}`, unlocked)
  let prints_itself =
    lookup_definition(&T_CS!(print))?.is_some_and(|defn| !defn.get_origin().is_latexml_owned());
  if address.is_empty() || !prints_itself {
    return Ok(SymHashMap::default());
  }
  // (never inside its own print: a meaning calling the command again, bound here to this binding, reads no label)
  if READING_OWN_EMAIL.replace(true) {
    return Ok(SymHashMap::default());
  }
  // (a `\thanks` the class's command prints steps no mark of the block's notes)
  let steps = FOOTNOTE_STEPS.with(Cell::get);
  let printed = print_quietly(|| {
    let mut tokens = vec![T_BEGIN!(), T_CS!(print), T_BEGIN!()];
    tokens.extend(Explode!(address.as_str()));
    tokens.extend([T_END!(), T_END!()]);
    let printed = digest(Tokens::new(tokens))?;
    // (a print reaching the frontmatter API is a binding's call, read as one, never the class's own label: cej-simone2's
    // `\email[1][E-mail: ]{\begingroup\sanitize@url\@email{#1}}`, its `\@email` revtex's `\lx@add@email`, 1608.07191)
    if calls_the_api(&printed) {
      return Ok(String::new());
    }
    let mut text = String::new();
    ink_text(&printed, &mut text);
    Ok(text)
  });
  FOOTNOTE_STEPS.with(|s| s.set(steps));
  READING_OWN_EMAIL.set(false);
  let label = printed?
    .split_once(address.as_str())
    .map(|(before, _)| before.trim().to_string())
    .filter(|label| label.chars().any(char::is_alphabetic));
  Ok(match label {
    Some(label) => stored_map!("name" => Stored::String(pin(&label))),
    None => SymHashMap::default(),
  })
}

thread_local! {
  /// Whether a class's own e-mail command is being printed for its label ([`own_email_properties`]).
  static READING_OWN_EMAIL: Cell<bool> = const { Cell::new(false) };
  /// The footnote steps the block being rehearsed made so far (`\thanks`, `\footnote` and bare `\footnotemark`s),
  /// from 0 at each rehearsal.
  static FOOTNOTE_STEPS: Cell<i64> = const { Cell::new(0) };
  /// `\c@footnote` at the rehearsal's start, and whether the format has it (and `\thefootnote`).
  static FOOTNOTE_BASE: Cell<i64> = const { Cell::new(0) };
  static FOOTNOTE_BASE_DEFINED: Cell<bool> = const { Cell::new(false) };
}

/// `\\c@footnote`'s value, `None` where the format has no such register: what the bare marks print from, which the
/// rehearsal reads at its start and the field notes after each invocation printing nothing (`field_notes::FieldNotes::
/// counter_moves`).
pub(super) fn footnote_counter() -> Option<i64> {
  match lookup_definition(&T_CS!("\\c@footnote")) {
    Ok(Some(defn)) if defn.is_register() => defn.value_of(Vec::new()).map(NumericOps::value_of),
    _ => None,
  }
}

/// The event constructor of a row break (`\\`, `\\newline`, `\\linebreak`; a tabular juggle, `install_event_bindings`).
const ROW_BREAK: &str = "\\lx@fm@obs@break";
/// The event constructor of an author separator (`\\and`; a raw class's column, `\\alignauthor`).
const AUTHOR_AND: &str = "\\lx@fm@obs@and";

/// The frontmatter API's queueing sink, and the name the rehearsal of a call outside a block keeps its meaning under while
/// `\lx@fm@obs@add` stands for it (`install_event_bindings`: a creator call an API call's content makes, 0807.4720).
const SINK: &str = "\\lx@add@frontmatter";
const SINK_MEANING: &str = "\\lx@fm@obs@add@meaning";
/// The event constructor `\lx@fm@obs@add` leaves before a creator call (`events`' `EventKind::ApiCall` "ltx:creator").
const CREATOR_CALL: &str = "\\lx@fm@obs@creatorcall";

/// The commands the rehearsal binds to the event constructors, always (the article `\author` line's own vocabulary,
/// and the frontmatter API's queueing sink, frozen otherwise), and whether a `\let` copy of the command is bound as it
/// is, by its meaning ([`let_copies_of_bound_commands`]): the author line's separators, its row break and its notes. TeX's
/// `\let\nx\and` makes `\nx` the separator, which pdflatex and Perl part the names at (OXIDIZED_DESIGN_DIVERGENCES #994).
const ALWAYS_BOUND: &[(&str, &str, bool)] = &[
  ("\\and", AUTHOR_AND, true),
  ("\\And", "\\lx@fm@obs@And", true),
  ("\\AND", "\\lx@fm@obs@AND", true),
  ("\\\\", ROW_BREAK, true),
  ("\\newline", ROW_BREAK, false),
  ("\\linebreak", ROW_BREAK, false),
  ("\\par", "\\lx@fm@obs@par", false),
  ("\\thanks", "\\lx@fm@obs@thanks", true),
  ("\\footnote", "\\lx@fm@obs@footnote", true),
  ("\\footnotetext", "\\lx@fm@obs@notetext", false),
  ("\\footnotemark", "\\lx@fm@obs@footnotemark", false),
  ("\\label", "\\lx@fm@obs@label", false),
  // (inside author content `\inst` is the affiliation request, `\lx@author@withinst`)
  ("\\inst", "\\lx@fm@obs@mark", false),
  ("\\lx@annotate@frontmatter", "\\lx@fm@obs@annotate", false),
  (
    "\\lx@request@frontmatter@annotation",
    "\\lx@fm@obs@request",
    false,
  ),
];

/// How many of the document's own macros [`let_copies_of_bound_commands`] reads into, at most, from one block.
const COPY_SEARCH_MACROS: usize = 256;

/// The `\let` copies of an [`ALWAYS_BOUND`] command bound by its meaning that the block's tokens reach — spelled in it,
/// or held in a macro of the document's own it spells (`\newcommand\aut{Ann \nx Bob}`, `\author{\aut}`) — each with the
/// event constructor the command is bound to: a command whose meaning is a definition made for the command (`defined_as`
/// it, so a `\relax` or an undefined name is no copy of a separator `\let` to `\relax` or left undefined), whatever the
/// command means now — as the command itself is bound by its name, whatever its definition. A copy taken before the
/// document renews the command keeps the command's separator or note in TeX (`\let\nx\and\renewcommand\and{…}`, two
/// columns in pdflatex); a copy of a renewed `\and` is bound as the renewed `\and` is (read as its
/// macro, a `\renewcommand\and{\\}`'s copy would make the second name the first's affiliation). Read before the
/// bindings change any meaning; a name bound by its own name, or one the binding declares a block break, is left to
/// those. The kernel's `\and` is a constructor printing " and " (`sect05.rs`): bound by name only, the copy kept it,
/// the names merged into one creator (OC1's `\let` probe; article, eptcs, IEEEtran journal).
fn let_copies_of_bound_commands(
  tokens: &Tokens,
  block_breaks: &[(&str, BreakKind)],
) -> Vec<(Token, &'static str)> {
  let mut found = Vec::new();
  let commands: Vec<(Token, &'static str)> = ALWAYS_BOUND
    .iter()
    .filter(|(.., by_meaning)| *by_meaning)
    .map(|(cs, target, _)| (T_CS!(*cs), *target))
    .collect();
  // (the distinct commands read: a block's and its macros', which may spell thousands)
  let mut seen: HashSet<Token> = HashSet::default();
  let mut pending: Vec<Token> = tokens.unlist_ref().iter().rev().copied().collect();
  let mut macros = 0;
  while let Some(t) = pending.pop() {
    if !t.is_executable() || !seen.insert(t) {
      continue;
    }
    let by_name =
      t.with_cs_name(|name| bound_by_name(name) || block_breaks.iter().any(|(cs, _)| *cs == name));
    if by_name {
      continue;
    }
    if let Some((_, target)) = commands.iter().find(|(command, _)| t.defined_as(command)) {
      found.push((t, *target));
      continue;
    }
    if macros < COPY_SEARCH_MACROS
      && let Ok(Some(defn)) = lookup_definition(&t)
      && defn.is_expandable()
      && defn.get_origin() == DefinitionOrigin::Document
      && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
    {
      macros += 1;
      pending.extend(body.unlist_ref().iter().rev().copied());
    }
  }
  found
}

/// Whether the rehearsal binds the command `name` by its name ([`ALWAYS_BOUND`], [`BOUND_WHEN_DEFINED`]).
fn bound_by_name(name: &str) -> bool {
  ALWAYS_BOUND.iter().any(|(cs, ..)| *cs == name)
    || BOUND_WHEN_DEFINED.iter().any(|(cs, ..)| *cs == name)
}

/// Whether a rehearsal may give `t` a meaning of its own ([`install_event_bindings`]): what it prints there is not
/// what its definition here says.
pub(super) fn rebinds(t: &Token) -> bool {
  t.with_cs_name(|name| {
    bound_by_name(name)
      || BOUND_IN_A_BLOCK.iter().any(|(cs, _)| *cs == name)
      || [SINK, SINK_MEANING, THEFOOTNOTE].contains(&name)
  })
}

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
/// argument itself, its macro taking none. A definition reading more required arguments than the binding is not that
/// contact either (pnastwo's `\affil#1#2`, [`binding_required_arguments`]; 1312.3323).
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
  ("\\alignauthor", AUTHOR_AND, false),
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

/// The footnote mark's command, which the rehearsal binds to the title's marks ([`install_event_bindings`]).
const THEFOOTNOTE: &str = "\\thefootnote";

/// Bind the Layer-B event commands, in the rehearsal's group, and the title's footnote marks: `\thefootnote` the
/// declared [`TITLE_FOOTNOTE_HOOK`] when there is one; else, as the block's conventions say (`title_marks`), the
/// class's own `\thefootnote` (report's and book's title page: arabic) or the kernel `\maketitle`'s `\@fnsymbol` of the
/// counter (when the format has it).
///
/// First, a tabular juggle (`fallback::tabular_juggle_between`: `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}`
/// in sig-alternate's `\author`, 0712.1016, 1704.03168) is the column it prints, as the juggle written inline is
/// ([`inline_juggles_as_columns`]), or the row break it prints between (1301.5612's `\end{tabular}\newline
/// \begin{tabular}{c}`; the deleted old driver wrote every juggle `\\`): digested as
/// defined, it closed a tabular the line never opened and left one open, whose lines no box held — lost with no Warning,
/// the content check reading only what the events hold. A column after the names holding none is their shared
/// affiliation (the reading's, `resolver::blocks`). A juggle reopens the tabular it closes, so a macro that only closes one (an
/// `\\affiliation`'s `\\def\\ex{\\end{tabular}}` closing the tabular its `\\bx` opened, `perfect_kernel_batchz4`; an
/// `\\author`'s `\\etab` closing the block's own) keeps its meaning, in any rehearsal; a command the binding declares a
/// block break keeps its meaning. Before any of it, a `\let` copy of a separator or note command the block reaches is
/// bound as the command is ([`let_copies_of_bound_commands`]: `\let\nx\and` parts the names).
fn install_event_bindings(
  tokens: &Tokens,
  title_marks: TitleMarks,
  block_breaks: &[(&str, BreakKind)],
) -> Result<()> {
  for (copy, target) in let_copies_of_bound_commands(tokens, block_breaks) {
    let_i(&copy, &T_CS!(target), None);
  }
  for t in tokens.unlist_ref() {
    let declared = || t.with_cs_name(|name| block_breaks.iter().any(|(cs, _)| *cs == name));
    if let Some(between) = tabular_juggle_between(t)
      && !declared()
    {
      let target = if prints_a_row_break(&between) {
        ROW_BREAK
      } else {
        AUTHOR_AND
      };
      let_i(t, &T_CS!(target), None);
    }
  }
  for (cs, target, _) in ALWAYS_BOUND {
    let_i(&T_CS!(*cs), &T_CS!(*target), None);
  }
  if in_block_content() {
    for (cs, target) in BOUND_IN_A_BLOCK {
      let_i(&T_CS!(*cs), &T_CS!(*target), None);
    }
  } else {
    let_i(&T_CS!(SINK_MEANING), &T_CS!(SINK), None);
    let_i(&T_CS!(SINK), &T_CS!("\\lx@fm@obs@add"), None);
  }
  if lookup_definition(&T_CS!(TITLE_FOOTNOTE_HOOK))?.is_some() {
    let_i(&T_CS!(THEFOOTNOTE), &T_CS!(TITLE_FOOTNOTE_HOOK), None);
  } else if title_marks == TitleMarks::Fnsymbol
    && lookup_definition(&T_CS!("\\@fnsymbol"))?.is_some()
  {
    let_i(&T_CS!(THEFOOTNOTE), &T_CS!("\\lx@fm@obs@thefootnote"), None);
  }
  for (cs, target, needs_argument) in BOUND_WHEN_DEFINED {
    if !block_breaks.iter().any(|(declared, _)| declared == cs)
      && lookup_definition(&T_CS!(*cs))?.is_some_and(|defn| {
        (!needs_argument || defn.get_num_args() > 0)
          && required_arguments(&*defn) <= binding_required_arguments(target)
      })
    {
      // (an e-mail call is read with whose meaning it has here, when the note is rehearsed — at the flush,
      // `\maketitle`, not at the call (review rev92a_r4 B1, OD #913): `\lx@fm@obs@email` a binding's, the `@inplace`
      // and `@own` variants the class's printing in place and no binding's — review rev92a_r3 B1)
      let target = if *target == "\\lx@fm@obs@email" {
        match email_meaning(cs) {
          EmailMeaning::Binding => *target,
          EmailMeaning::InPlace => "\\lx@fm@obs@email@inplace",
          EmailMeaning::Own => OWN_EMAIL
            .iter()
            .find(|(own, ..)| own == cs)
            .map_or(OWN_EMAIL[0].1, |(_, target, _)| *target),
        }
      } else if *target == "\\lx@fm@obs@affil"
        && lookup_definition(&T_CS!(&s!("\\lx@class@contact@{}", &cs[1..])))?.is_some()
      {
        "\\lx@fm@obs@affil@run"
      } else {
        *target
      };
      // (a class's own e-mail command kept beside its binding, which prints it to read its label:
      // `own_email_properties`; never the binding itself, bound again)
      if let Some((_, own, print)) = OWN_EMAIL.iter().find(|(_, own, _)| *own == target)
        && lookup_definition(&T_CS!(*cs))?
          .is_some_and(|defn| defn.get_cs().as_ref() != &T_CS!(*own))
      {
        let_i(&T_CS!(*print), &T_CS!(*cs), None);
      }
      let_i(&T_CS!(*cs), &T_CS!(target), None);
    }
  }
  Ok(())
}

/// How many arguments `defn` reads that are not optional (a `\newcommand`'s `[…]` default, a `[]` parameter): the
/// arguments a call of it must carry.
fn required_arguments(defn: &dyn Definition) -> usize {
  defn.get_parameters().map_or(0, |params| {
    params
      .get_parameters()
      .iter()
      .filter(|param| !param.novalue && !param.optional)
      .count()
  })
}

/// The required arguments the rehearsal's binding `target` reads ([`BOUND_WHEN_DEFINED`]). A document command that reads
/// more is not that contact and keeps its meaning: pnastwo.cls:368's `\affil#1#2` (the affiliation's number, then its
/// text, stored for `\@maketitle` to print — the call prints only the mark), bound to the one-argument affiliation,
/// was read as the affiliation "1" with `{Inst}` left in the name, and the emitter's item `\affil{1}`, digested as
/// defined, read its second argument past the item's end — `\gdef`ing `\affil1` to `\let`, which, printed by
/// `\@maketitle`, ate the closing braces of the title-page deposit: a group left open after `\maketitle`, every body
/// `\footnote` in it gobbled, the deposit (the contributor line, the printed affiliations) dropped for its error
/// (1312.3323, 1404.2960, 0903.4054, 0907.3600; `perfect_kernel_batchcl6`).
fn binding_required_arguments(target: &str) -> usize {
  lookup_definition(&T_CS!(target))
    .ok()
    .flatten()
    .map_or(usize::MAX, |defn| required_arguments(&*defn))
}

/// Whether `between`, what a tabular juggle prints between its close and its reopen, holds a row break the rehearsal
/// binds (`\\`, `\\[2mm]`, `\newline`, `\linebreak`) or a vertical skip (`\vskip 2mm`: the paragraph's end, the
/// tabulars stacked in the PDF, review revI1 finding 3): the juggle is that break, else a column. `\vspace{2mm}` in a
/// paragraph is no break (latex.ltx `\@vspace`: a `\vadjust` after the line, two columns in the PDF).
fn prints_a_row_break(between: &[Token]) -> bool {
  between.iter().any(|t| *t == T_CS!(VERTICAL_SKIP)) || prints_a_bound_break(between)
}

/// TeX's vertical skip, which ends a paragraph (tex.web §1094 `head_for_vmode`) and reads a glue spec (§461).
const VERTICAL_SKIP: &str = "\\vskip";

/// Whether `between` holds a row break the rehearsal binds (`\\`, `\newline`, `\linebreak`), the break event it
/// digests to.
fn prints_a_bound_break(between: &[Token]) -> bool {
  between.iter().any(|t| {
    ALWAYS_BOUND
      .iter()
      .any(|(cs, target, _)| *target == ROW_BREAK && *t == T_CS!(*cs))
  })
}

/// The juggle written inline: an `\end{tabular}` closing a tabular the block never opened, reopened after nothing but
/// spaces and commands with their `[…]` and `{…}` arguments (what a class prints between, `\hfil`, `\newline`,
/// `\\[2mm]`, `\vspace{2mm}`), the reopening `\begin{tabular}[…]{…}` its part. With no line break printed between it is the column `\alignauthor` is (sig-alternate.cls:274 `\end{tabular}%
/// \begin{tabular}[t]{p{\auwidth}}\centering`, bound to an author separator, [`install_event_bindings`]): 1401.6981
/// writes its second author's column so, `\end{tabular}\begin{tabular}[t]{p{1.3\auwidth}}\centering Gianmarco De
/// Francisci Morales…`, three columns in the PDF. With one printed between (`\end{tabular}\newline\begin{tabular}{c}`,
/// 1301.5612's macro written out; `\\[2mm]`, review revG3 finding 2, "Univ B" lost with no Warning before; a `\vskip`,
/// [`prints_a_row_break`]) that break stands; a skip in the line is no break (`\hskip 1em`, `\vspace{2mm}`, two columns
/// in the PDF). Digested as written it closed the title page's tabular and left
/// one open, whose lines no box held — at `=2` authors 2 and 3 of 1401.6981 lost with no Warning. A close of a tabular
/// the block opened (`\begin{tabular}…\end{tabular}`) or one no reopen follows keeps its meaning; with the emitter's
/// switch on.
fn inline_juggles_as_columns(tokens: Tokens) -> Tokens {
  let (end, begin) = environment_commands();
  let toks = tokens.unlist_ref();
  if !toks.contains(&end) {
    return tokens;
  }
  let optional = |k: usize| past_optional(toks, k);
  let braced = |k: usize| past_braced(toks, k);
  // The span past the reopen's `\begin{tabular}[…]{…}`, if one follows `at` after only spaces and commands, each with
  // the `[…]` and `{…}` arguments right after it (a bare group, `\end{tabular} {Extra Words} \begin{tabular}`, is no
  // command's: the juggle is read as written, review revH1 finding 2).
  let argument = |k: usize| {
    optional(k)
      .or_else(|| braced(k))
      .filter(|&(first, _)| first == k)
      .map(|(_, end)| end)
  };
  let (space, command) = (Catcode::SPACE, Catcode::CS);
  let (glue, dimen) = ([T_CS!("\\hskip"), T_CS!(VERTICAL_SKIP)], T_CS!("\\kern"));
  let reopen = |at: usize| -> Option<(usize, usize)> {
    let mut start = at;
    loop {
      let c = toks.get(start)?.get_catcode();
      if c != space && (c != command || toks[start] == begin) {
        break;
      }
      let cs = toks[start];
      start += 1;
      if c == command {
        // What the command reads past spaces, as TeX reads it: `\\`'s `[…]` (its `*` right after it), by `\@ifstar` and
        // `\@ifnextchar` (latex.ltx `\@normalcr`/`\@xnewline`, `\@tabularcr`/`\@xtabularcr`; `\\ [2mm]`, review
        // revH4 finding 1, "Univ B" lost with no Warning before), and a glue or dimen spec written out
        // (`\hskip 1em`, h3e). A `{…}` comes right after its command (`argument`).
        if let Some(k) = past_line_break(toks, start - 1) {
          start = k;
        } else if glue.contains(&cs) {
          start = past_glue(toks, start).unwrap_or(start);
        } else if cs == dimen {
          start = past_dimen(toks, start, false).unwrap_or(start);
        }
        while let Some(k) = argument(start) {
          start = k;
        }
      }
    }
    Some((start, past_tabular_open(toks, start)?))
  };
  let mut out = Vec::with_capacity(toks.len());
  let mut opened = 0usize;
  let mut i = 0;
  while i < toks.len() {
    if let Some(len) = environment_call_len(&toks[i..], begin, "tabular") {
      opened += 1;
      out.extend_from_slice(&toks[i..i + len]);
      i += len;
    } else if let Some(len) = environment_call_len(&toks[i..], end, "tabular") {
      match (opened, reopen(i + len)) {
        (0, Some((start, past))) => {
          // (what the commands between print goes with the column before, as the old driver's output orders it:
          // `\mbox{Extra Words}` after Univ A, not the next column's first line, Bob Two lost with 2 Errors before —
          // review revH1 finding 2; the writer still writes the column's source, `\end{tabular}` in it, so the words
          // between are lost, counted by the content check: OD #980 (5))
          let between = &toks[i + len..start];
          out.extend_from_slice(between);
          // (a bound break between is the break event itself; a vertical skip, which digests to none, is followed
          // by one: `\vskip 2mm`, review revI1 finding 3, read as a column before)
          if !prints_a_bound_break(between) {
            let target = if prints_a_row_break(between) {
              ROW_BREAK
            } else {
              AUTHOR_AND
            };
            out.push(Token {
              prov: toks[i].prov,
              ..T_CS!(target)
            });
          }
          i = past;
        },
        _ => {
          opened = opened.saturating_sub(1);
          out.extend_from_slice(&toks[i..i + len]);
          i += len;
        },
      }
    } else {
      out.push(toks[i]);
      i += 1;
    }
  }
  Tokens::new(out)
}

/// The position past a glue spec written out from `at` (TeX's `scan_glue`, tex.web §461: a dimen, then `plus` and
/// `minus` ones that may be infinite), if one is; a register or anything else is not read.
fn past_glue(toks: &[Token], at: usize) -> Option<usize> {
  let mut k = past_dimen(toks, at, false)?;
  for word in ["plus", "minus"] {
    if let Some(after) = past_keyword(toks, k, word) {
      k = past_dimen(toks, after, true)?;
    }
  }
  Some(k)
}

/// The position past a dimen written out from `at`: optional signs, digits with a `.` or `,`, and a unit (tex.web
/// §448-458: `true`, a physical or font unit, `fil`…`filll` where `infinite`); the optional space after it is the
/// caller's, which skips spaces.
fn past_dimen(toks: &[Token], at: usize, infinite: bool) -> Option<usize> {
  let other = |k: usize| {
    let t = toks.get(k).filter(|t| t.get_catcode() == Catcode::OTHER);
    t.and_then(|t| char::from_u32(t.get_charcode()))
  };
  let mut k = non_space(toks, at)?;
  while matches!(other(k), Some('+' | '-')) {
    k = non_space(toks, k + 1)?;
  }
  let digits = k;
  while matches!(other(k), Some('0'..='9' | '.' | ',')) {
    k += 1;
  }
  if k == digits {
    return None;
  }
  if infinite && let Some(mut k) = past_keyword(toks, k, "fil") {
    for _ in 0..2 {
      k = past_keyword(toks, k, "l").unwrap_or(k);
    }
    return Some(k);
  }
  let k = past_keyword(toks, k, "true").unwrap_or(k);
  [
    "em", "ex", "pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp", "px",
  ]
  .iter()
  .find_map(|unit| past_keyword(toks, k, unit))
}

/// The position past the keyword `word` read from `at` after spaces, its letters in either case, if it is there
/// (tex.web §407 `scan_keyword`; a command's code is no letter's).
fn past_keyword(toks: &[Token], at: usize, word: &str) -> Option<usize> {
  let first = non_space(toks, at)?;
  let mut k = first;
  for c in word.chars() {
    let t = char::from_u32(toks.get(k)?.get_charcode());
    if t.is_none_or(|t| t.to_ascii_lowercase() != c) {
      return None;
    }
    k += 1;
  }
  Some(k)
}

/// The first position from `at` that holds no space, if one does.
fn non_space(toks: &[Token], at: usize) -> Option<usize> {
  (at..toks.len()).find(|&k| toks[k].get_catcode() != Catcode::SPACE)
}

/// Where a `\\` at `at` and what it reads after it end, if one is there: its `*` right after it only, as the binding
/// reads it (`\lx@fm@obs@break OptionalMatch:*` and Perl's `readMatch`, Gullet.pm:604: `\\ *[2mm]` read past its space
/// ended a mode it never opened, 1 Error at `=2`, review revI1 finding 2), then its `[…]` past spaces, by `\@ifstar` and
/// `\@ifnextchar` (latex.ltx `\@normalcr`/`\@xnewline`, `\@tabularcr`/`\@xtabularcr`). Both print nothing: the juggle's
/// reopen reads past them, and the content check reads no word in them (`ledger::content_text`, `\\[2mm]`, review
/// revCK1b finding 5).
pub(super) fn past_line_break(toks: &[Token], at: usize) -> Option<usize> {
  if *toks.get(at)? != T_CS!("\\\\") {
    return None;
  }
  let mut at = at + 1;
  if toks.get(at) == Some(&T_OTHER!("*")) {
    at += 1;
  }
  Some(past_optional(toks, at).map_or(at, |(_, end)| end))
}

/// Where a tabular's own call at `at` ends, and the spans in it that print: `\begin{tabular}` with its `[…]` and its
/// column spec `{…}` (the juggle's reopen, [`inline_juggles_as_columns`]), or `\end{tabular}`. The call prints nothing
/// but the text its spec places ([`spec_printed`]); the content check reads only those spans (`ledger::content_text`:
/// `\begin{tabular}{c}` counted "c" and "tabular" lost, review revI2 finding 1; `{@{Inst }c}` prints "Inst", review
/// revI3 finding 1).
pub(super) fn past_tabular_call(toks: &[Token], at: usize) -> Option<(usize, Vec<Range<usize>>)> {
  if let Some(len) = environment_call_len(&toks[at..], environment_commands().0, "tabular") {
    return Some((at + len, Vec::new()));
  }
  let (spec, end) = tabular_open_spec(toks, at)?;
  let mut printed = Vec::new();
  spec_printed(toks, spec + 1..end - 1, &mut printed);
  Some((end, printed))
}

/// The spans a column spec's tokens `spec` print (latex.ltx `\@mkpream`, array.sty `\@mkpream`): the text of `@{…}`
/// and `!{…}` between columns and of `>{…}` and `<{…}` in each cell (pdflatex: `{>{Prof }c<{ Dr}!{ or }c}` prints
/// "Prof Dept Dr or"), and those in a `*{n}{…}`'s repeated spec; a column letter, its argument (`p{3cm}`) and the
/// count `n` print nothing. `\@mkpream` compares the spec's characters by code (`\if`), so a letter `@` under
/// `\makeatletter` is read as an other one (review revI4 finding 2: `{@{Inst }c}` there printed "Inst" uncounted).
fn spec_printed(toks: &[Token], spec: Range<usize>, printed: &mut Vec<Range<usize>>) {
  let mut k = spec.start;
  while k < spec.end {
    let token = &toks[k];
    let group = |from: usize| past_braced(&toks[..spec.end], from);
    if matches!(token.get_catcode(), Catcode::OTHER | Catcode::LETTER)
      && let Some(c @ ('@' | '!' | '>' | '<' | '*')) = char::from_u32(token.get_charcode())
      && let Some((open, close)) = group(k + 1)
    {
      k = close;
      if c != '*' {
        printed.push(open + 1..close - 1);
      } else if let Some((open, close)) = group(k) {
        spec_printed(toks, open + 1..close - 1, printed);
        k = close;
      }
    } else if let Some((_, close)) = group(k).filter(|&(open, _)| open == k) {
      k = close;
    } else {
      k += 1;
    }
  }
}

/// `\end` and `\begin`, the environment commands a tabular's call reads.
fn environment_commands() -> (Token, Token) { (T_CS!("\\end"), T_CS!("\\begin")) }

/// Where `\begin{tabular}[…]{…}` at `at` ends, if one starts there.
fn past_tabular_open(toks: &[Token], at: usize) -> Option<usize> {
  tabular_open_spec(toks, at).map(|(_, end)| end)
}

/// Where the column spec `{…}` of a `\begin{tabular}[…]{…}` at `at` starts, and where the call ends, if one starts
/// there.
fn tabular_open_spec(toks: &[Token], at: usize) -> Option<(usize, usize)> {
  let k = at + environment_call_len(&toks[at..], environment_commands().1, "tabular")?;
  let k = past_optional(toks, k).map_or(k, |(_, end)| end);
  past_braced(toks, k)
}

/// Where the `[…]` that starts at `at` after spaces starts, and the position past it, if one does.
fn past_optional(toks: &[Token], at: usize) -> Option<(usize, usize)> {
  past_group(toks, at, |t| *t == T_OTHER!("["), |t| *t == T_OTHER!("]"))
}

/// Where the `{…}` that starts at `at` after spaces starts, and the position past it, if one does.
fn past_braced(toks: &[Token], at: usize) -> Option<(usize, usize)> {
  past_group(
    toks,
    at,
    |t| t.get_catcode() == Catcode::BEGIN,
    |t| t.get_catcode() == Catcode::END,
  )
}

/// Where the balanced `open`…`close` group that starts at `at` after spaces starts, and the position past it, if one
/// does.
fn past_group(
  toks: &[Token],
  at: usize,
  open: impl Fn(&Token) -> bool,
  close: impl Fn(&Token) -> bool,
) -> Option<(usize, usize)> {
  let first = non_space(toks, at)?;
  if !open(&toks[first]) {
    return None;
  }
  let mut level = 0usize;
  for (k, t) in toks.iter().enumerate().skip(first) {
    if open(t) {
      level += 1;
    } else if close(t) {
      level -= 1;
      if level == 0 {
        return Some((first, k + 1));
      }
    }
  }
  None
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
    /// whether the run left a group open — its frames not back where they started, the mode it switched to not ended (an
    /// `\end{tabular}\begin{tabular}{c}` juggle in the line, 1401.6981): what it digested past that point is in no box of
    /// the list, so no event says it, and the content check reads the block's content too (`resolver::placed::content_check`)
    left_open:    bool,
    /// what the block's own digestion did that no event says (`field_notes::FieldNotes::counter_moves`, `printing`)
    unsaid:       Option<Box<Unsaid>>,
    /// the entries the run queued besides its creators ([`note_frozen_entry`]): a frontmatter setter run inside the block,
    /// a `\date` in `\author` (2503.02741), undone with the run
    queued:       Vec<RawFrontmatter>,
    /// the expansions of the document's macros its tokens call (`field_notes::HeldExpansion`)
    expansions:   Vec<HeldExpansion>,
  },
  /// No rehearsal was made or kept, and why: no journal could open (one is already open), its assignments could not be
  /// undone, or it ended in a Fatal error of its own (discarded with it).
  Skipped(SkipReason),
}

/// The footnote steps the blocks rehearsed so far carry ([`rehearse`]), to read a block again from where it began
/// (`reading::unfold_held_names`).
pub(in crate::base_utilities) fn footnotes_carried() -> (i64, i64) {
  (
    lookup_int("lx_fm_obs_counter_seen"),
    lookup_int("lx_fm_obs_steps_carried"),
  )
}

/// Set what [`footnotes_carried`] reads.
pub(in crate::base_utilities) fn set_footnotes_carried((seen, carried): (i64, i64)) {
  assign_value(
    "lx_fm_obs_counter_seen",
    Stored::from(seen),
    Some(Scope::Global),
  );
  assign_value(
    "lx_fm_obs_steps_carried",
    Stored::from(carried),
    Some(Scope::Global),
  );
}

/// Digest `tokens` (a block's stamped tokens, `ledger::field_stamped`) under the event bindings and hand the
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
  let footnotes = footnote_counter().is_some() && lookup_definition(&T_CS!(THEFOOTNOTE))?.is_some();
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
  // reading after the last block — a block's run may open one the next block's closes, as TeX reads the list in one
  // pass, 2606.17718 — and only those of a failed run here: `StomachMark`)
  let tokens_depth = token_stack_depth();
  let conditionals = if_stack_depth();
  let ledger_scope = FieldLedgerScope::begin();
  FROZEN_ENTRIES.with(|entries| entries.borrow_mut().clear());
  REHEARSING.with(|r| r.set(true));
  FOOTNOTE_STEPS.with(|steps| steps.set(0));
  // The whole group in a reading context of its own: what its end unreads (an `\aftergroup` of the block's,
  // `pop_stack_frame_with`) lands in this mouth, closed with whatever is left (`reading_from_mouth`'s forced close),
  // never in the document's input; and the token the group's end and its diagnostics see is the rehearsal's own, not
  // the document's current one (a `}` closing an alignment template's brace, `egroup`'s `close_template_brace`).
  // Guards: the repro `sectioning-frontmatter/observer_rehearsal_aftergroup_stays_inside` (its `\aftergroup\leak`
  // printed "LEAKED" into the body before this wrap) and `perfect_kernel_batchfms2a1::
  // the_rehearsal_leaves_nothing_behind` (no `LEAKED` in any adversarial shape's output, each shape's body text and
  // counts pinned).
  local_current_token(T_CS!("\\relax"));
  let _hook = FieldNotesHookGuard;
  let if_count_before = if_count();
  let budget = read_budget(budget);
  let tokens = inline_juggles_as_columns(tokens);
  let result = with_origin(DefinitionOrigin::File, || {
    reading_from_mouth(Mouth::default(), || -> Result<Digested> {
      bgroup();
      install_event_bindings(&tokens, title_marks, block_breaks)?;
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
  let left_open = get_frame_depth() > depth;
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
  let queued = FROZEN_ENTRIES.with(|entries| std::mem::take(&mut *entries.borrow_mut()));
  let undone = journal.rollback();
  // (a printing call's definitions that the block introduces, undefined again once undone: the class's own state such a call
  // sets — PoS's `\global\@speakertrue` in a paper's `\speaker`, 1009.6192; manyfoot's `\@thefnmark`, 2511.01971 — is the
  // class's to read, as `\maketitle` does)
  let printing: Vec<PrintingCall> = notes
    .printing
    .into_iter()
    .filter_map(|mut call| {
      call.definees.retain(|&key| !has_meaning_key(key));
      (!call.definees.is_empty()).then_some(call)
    })
    .collect();
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
  // run's: `sect05.rs` `speculative_run_read`)
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
      left_open,
      unsaid: Unsaid {
        counter_moves: notes.counter_moves,
        printing,
      }
      .boxed(),
      queued,
      expansions: notes.expansions,
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
