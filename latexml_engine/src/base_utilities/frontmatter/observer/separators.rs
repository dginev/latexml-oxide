//! A binding's query on an argument's author separators, by meaning (Option C step P2-0, OXIDIZED_DESIGN_DIVERGENCES #995):
//! the `\and`-family separators an argument holds at its own level, as the rehearsal digests it to events
//! (`EventKind::Sep(SepKind::And)` at nest 0: literal, held in a macro of the document's, a `\let` copy, a `\csname
//! and\endcsname`), each with whether its meaning *is* the kernel separator's. The rehearsal binds `\and`, `\And` and
//! `\AND` to its separator event whatever they mean here (`rehearsal::ALWAYS_BOUND`), so a `Sep` alone does not say the
//! class parts names there (trap T8 of the P2 plan): a class's `\and` defined where the binding calls may print a word or
//! a `&` (smfart's, 2508.07946), and a binding's own definition stands for whatever its class prints. The meaning is read
//! outside the rehearsal, the one in force at the call site, where the binding calls the query (at `\author`,
//! `\institute`): a class that defines or `\let`s `\and` only inside its title macro is read as the kernel's there, as
//! `frontmatter_stores::kernel_and_list` reads it — jmlr.cls's word `\and` (jmlr.cls:336, inside `\jmlrpreauthor`,
//! which its `\maketitle` runs) answers `kernel` before `\maketitle` (RED repro `separators_query_jmlr_title_time_and_red`;
//! a reading of the title-time meaning is a mechanism of its own). The query rehearses from its own allowance (`rehearsal::for_a_query`),
//! never the reading's budget nor its count of calls left out.
//!
//! Two front-ends of the one query: [`argument_separators`] for a binding's Rust code, and `\lx@fm@separators{<tokens>}`
//! for a binding's macro text, expanding to the meanings' names in order, comma-separated (`kernel`, `binding`, `raw`),
//! empty for none, `unread` where no rehearsal could be made or it left a group open.

use latexml_core::{definition::origin::DefinitionOrigin, token::AUTHOR_PROV_POS_MASK};

use super::{
  events::{Event, EventKind, SepKind, lex_events},
  field_notes::EnvironmentStep,
  ledger::field_stamped,
  policy::TitleMarks,
  rehearsal::{Rehearsal, for_a_query, rehearse},
  *,
};

/// What a separator means where the query is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SepMeaning {
  /// the kernel's `\and` (`latex_constructs::sect05`'s constructor, which `\And` and `\AND` are `\let` to there, as are a
  /// binding's `\let` copies: `inst_support_sty.rs:66` `\AND`, `ijcai_sty.rs:33-34` `\AND`/`\And`, `epl_cls.rs:109`
  /// `\And` — their definition is `\and`'s own, so they read `Kernel`, not `Binding`): the
  /// kernel's author-line column, which the blocks' readings part names at. The definition must be `\and`'s own (its
  /// `get_cs()`), a kernel layer's: narrower than the pool-constructor test `frontmatter_stores::kernel_and_list` made
  /// before P2-6, which now asks this (`texvc_sty.rs:132`'s `\let\and\land` is `raw`, no kernel separator; that test
  /// is the gate's reading only where the query answers `None`)
  Kernel,
  /// a binding's own definition (its code, a fallback class's, a stand-in's or a borrowing's,
  /// `DefinitionOrigin::is_binding_made`), whatever it prints: aas_support's real `\and` (`aas_support_sty.rs:358`), the
  /// stand-ins for a class's column break printing a space or nothing (colm2025's `\And`/`\AND`, jmlr's and jmlr2e's
  /// `\And`, jair's `\hskip`, neurips's no-ops), a word (ims_support's `\AND`), a `&` (elsart_support's `\AND`), and
  /// amsmath's math `\And` (no separator at all). So `Binding` alone says neither "a separator" nor "none": the P2
  /// callers' gates ask kernel or not (`kernel_and_list`'s swap, aa, l_aa, ecai, authblk), which needs no split; a caller
  /// that must tell them apart reads its own binding's definition
  Binding,
  /// anything else: a raw class's, a package's or the document's own definition (a word, a `&`, a space), or no
  /// separator's meaning at all (`\relax`, undefined)
  Raw,
}

impl SepMeaning {
  /// The name `\lx@fm@separators` gives it.
  pub fn name(self) -> &'static str {
    match self {
      SepMeaning::Kernel => "kernel",
      SepMeaning::Binding => "binding",
      SepMeaning::Raw => "raw",
    }
  }
}

/// One separator of an argument at its own level ([`argument_separators`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgSeparator {
  /// the command of the family the rehearsal read (`\and`, `\And`, `\AND`: `SepKind::And`)
  pub command: &'static str,
  /// the meaning of the token that printed it, where the query is made
  pub meaning: SepMeaning,
  /// its own tokens in the argument, `start..end` (a literal `\and`, a macro holding only the separator, a `\let` copy,
  /// a `\csname and\endcsname`; a `\relax` before it, which prints nothing; never an environment closed right before
  /// it — an `\end{tabular}`, `\end{minipage}`, a nested table, a user environment — which is the environment's,
  /// `own_run`), where they print nothing else: no name, mark, contact or note's span meets them, the
  /// criterion of the emitter's cut (`emit::cut_at_separators`, #993). `None` where the span is unread or the tokens
  /// print ink too (a macro holding names and the separator together, `\newcommand\aut{Ann \and Bob}`): no cut by
  /// tokens parts them, the reading's cut does. Its callers part an argument there: P2-4's eptcs cells
  /// (OXIDIZED_DESIGN_DIVERGENCES #991 (8)) and the queue's name reader (`queue_reading::entry_names`, #993 P2-1b)
  pub span:    Option<(usize, usize)>,
}

/// The kinds of meaning a definition origin says, for a meaning that is not the kernel separator's.
fn made_by(origin: DefinitionOrigin) -> SepMeaning {
  if origin.is_binding_made() {
    SepMeaning::Binding
  } else {
    SepMeaning::Raw
  }
}

/// The kernel layer's separator, `latex_constructs/sect05.rs`'s `\and` constructor: the definition a kernel meaning must be
/// (`\And` and `\AND` are `\let` to it), the one `frontmatter_stores::kernel_and_list` asks for (P2-6).
const KERNEL_SEPARATOR: &str = "\\and";

/// Whether `definition` is [`KERNEL_SEPARATOR`]'s (a `\let` copy's too: its own cs) — a whatsit of it prints no text
/// (`util_text::printed_own_text`).
pub(in crate::base_utilities) fn is_kernel_separator(definition: &dyn Definition) -> bool {
  *definition.get_cs() == T_CS!(KERNEL_SEPARATOR)
}

/// What `token` means now ([`SepMeaning`]): the kernel separator is the [`KERNEL_SEPARATOR`] constructor a kernel layer
/// made (a pool, a dump, a format: `is_latexml_owned` and not `is_binding_made`).
fn meaning_of(token: &Token) -> SepMeaning {
  match lookup_meaning(token) {
    Some(Stored::Constructor(defn)) => {
      let kernel = defn.origin.is_latexml_owned() && !defn.origin.is_binding_made();
      if kernel && is_kernel_separator(defn.as_ref()) {
        SepMeaning::Kernel
      } else {
        made_by(defn.origin)
      }
    },
    Some(Stored::Expandable(defn)) => made_by(defn.get_origin()),
    Some(Stored::Primitive(defn)) => made_by(defn.origin),
    Some(Stored::Conditional(defn)) => made_by(defn.origin),
    _ => SepMeaning::Raw,
  }
}

/// The token whose meaning a separator has, its own run `span` ([`own_run`]): the one token the span covers (a literal `\and`,
/// a `\let` copy, an active character) where it is no macro; else the family's command the rehearsal read (`command`) —
/// a macro holding the separator, whose call the span covers (`\newcommand\aut{Ann \and Bob}`), a class's macro `\and`
/// itself or a copy of it (`rehearsal::let_copies_of_bound_commands` binds a copy only where it means what the command
/// means), a `\csname and\endcsname` (which names the command), a span not read.
fn printing_token(content: &[Token], span: Option<(u16, u16)>, command: &'static str) -> Token {
  let run = span.and_then(|(s, t)| content.get(usize::from(s)..usize::from(t)));
  match run {
    Some([one]) if !matches!(lookup_meaning(one), Some(Stored::Expandable(_))) => *one,
    _ => T_CS!(command),
  }
}

/// A separator's span `span` past the environments closed right before it (`environments`, the rehearsal's steps): its
/// own run of the argument. An invocation making no box leaves its cover for the next box (core
/// `token::field_cover_pending_merge`, OD #715), so a separator's box covers an `\end{tabular}` glued to it
/// (`\end{tabular}\and`, `\end{tabular}%⏎\and`; `\end{tabular}\relax\and` covers the `\relax` too, which is the
/// separator's as a font switch before a name is the name's). Those tokens close the table and are the environment's,
/// as the emitter's balanced runs take them (`emitter::Field::balanced`), not the separator's. A step reaching the span's
/// end leaves it whole: the separator keeps a token.
fn own_run(environments: &[EnvironmentStep], span: Option<(u16, u16)>) -> Option<(u16, u16)> {
  let (mut s, t) = span?;
  while let Some(step) = environments
    .iter()
    .find(|step| !step.opens && step.start <= s && s < step.end && step.end < t)
  {
    s = step.end;
  }
  Some((s, t))
}

/// A separator's own run `run` ([`own_run`]) as positions, where no ink event's span meets it — no name, mark, contact
/// or note prints from its tokens ([`ArgSeparator::span`]).
fn own_tokens(events: &[Event], run: Option<(u16, u16)>) -> Option<(usize, usize)> {
  let (s, t) = run?;
  let inks = |e: &&Event| {
    matches!(
      e.kind,
      EventKind::Name | EventKind::Mark | EventKind::Contact | EventKind::Note
    )
  };
  let met = events
    .iter()
    .filter(inks)
    .any(|ink| ink.span.is_some_and(|(a, b)| a < t && b > s));
  (!met).then_some((usize::from(s), usize::from(t)))
}

/// The `\and`-family separators `arg` holds at its own level (nest 0: not one a layout container of it holds, a
/// `tabular`, a `\parbox`, nor a note's), in order, each with what it means where the query is made ([`SepMeaning`]).
/// `None` where no rehearsal could be made — a call inside one (no journal opens), over the queries' allowance, an
/// argument past the field's positions, a run whose assignments could not be undone (`Rehearsal::Skipped`) — or where
/// the run left a group open (a `\begingroup` with no `\endgroup`: no kernel author line holds one, so its separators
/// are unknown, not a list). What a rehearsal leaves for the reading, each put back (by reading `rehearse`'s writes to
/// its thread-locals and its global assignments): the reads it spends and the calls it leaves out
/// (`READS_LEFT`, `LEFT_OUT`: the queries' own allowance instead, `rehearsal::for_a_query`), the footnote steps it carries
/// for the next block's rehearsal (`lx_fm_obs_counter_seen`, `lx_fm_obs_steps_carried`, restored here), its flags and its
/// assignments (restored and rolled back by `rehearse`; a run whose assignments could not be undone keeps them, as every
/// rehearsal's does, `SkipReason::NotUndone`, and answers `None`).
/// OXIDIZED_DESIGN_DIVERGENCES #995.
pub fn argument_separators(arg: &Tokens) -> Result<Option<Vec<ArgSeparator>>> {
  separators_of(arg, false)
}

/// [`argument_separators`] at every level of `arg`, a layout container's too (an `\mbox`, a `\parbox`, a note's): the
/// `\and`s a class runs wherever they stand. llncs's `\institute` gate (`llncs_cls.rs` `\lx@llncs@institute@dispatch`,
/// Option C P2-3): `\institutename` numbers an institute at every `\and` it runs, one inside a box too
/// (llncs.cls:870-886; `Univ A \mbox{(North \and South)}` prints "1 Univ A (North ²South)").
pub fn argument_separators_at_every_level(arg: &Tokens) -> Result<Option<Vec<ArgSeparator>>> {
  separators_of(arg, true)
}

/// An argument as a query's rehearsal reads it ([`read_argument`]): its events and the environments it opens and closes.
pub(in crate::base_utilities) struct ArgumentReading {
  pub events:       Vec<Event>,
  pub environments: Vec<EnvironmentStep>,
}

/// The one rehearsal of an argument the queries read (separators, marks; OXIDIZED_DESIGN_DIVERGENCES #995, Option C
/// F0): `None` where no rehearsal could be made or the run left a group open ([`argument_separators`]).
pub(in crate::base_utilities) fn read_argument(arg: &Tokens) -> Result<Option<ArgumentReading>> {
  let content = arg.unlist_ref();
  if content.len() > usize::from(AUTHOR_PROV_POS_MASK) {
    return Ok(None);
  }
  const CARRIED: [&str; 2] = ["lx_fm_obs_counter_seen", "lx_fm_obs_steps_carried"];
  let carried = CARRIED.map(lookup_int);
  // (from the queries' own allowance, never the reading's budget nor its count of calls left out: `for_a_query`)
  let read = for_a_query(|| {
    rehearse(
      field_stamped(2, 0, content),
      TitleMarks::Fnsymbol,
      &[],
      lex_events,
    )
  })?;
  for (key, value) in CARRIED.iter().zip(carried) {
    if lookup_int(key) != value {
      assign_value(key, Stored::from(value), Some(Scope::Global));
    }
  }
  // (a run that left a group open read no event past where it opened — `\begingroup` and no `\endgroup`: unknown, not
  // "no separator")
  let Rehearsal::Done {
    value: events,
    left_open: false,
    environments,
    ..
  } = read
  else {
    return Ok(None);
  };
  Ok(Some(ArgumentReading { events, environments }))
}

/// [`argument_separators`], at `arg`'s own level or (`every_level`) at all of them.
fn separators_of(arg: &Tokens, every_level: bool) -> Result<Option<Vec<ArgSeparator>>> {
  let Some(ArgumentReading { events, environments }) = read_argument(arg)? else {
    return Ok(None);
  };
  let content = arg.unlist_ref();
  let separators = events.iter().filter_map(|e| match e.kind {
    EventKind::Sep(SepKind::And(command)) if every_level || e.nest == 0 => {
      let run = own_run(&environments, e.span);
      Some(ArgSeparator {
        command,
        meaning: meaning_of(&printing_token(content, run, command)),
        span: own_tokens(&events, run),
      })
    },
    _ => None,
  });
  Ok(Some(separators.collect()))
}

/// The TeX front-end, `\lx@fm@separators{<tokens>}`: [`argument_separators`]' meanings by name, comma-separated.
pub(in crate::base_utilities) fn load() -> Result<()> {
  DefMacro!("\\lx@fm@separators{}", sub[(arg)] {
    let arg: Tokens = arg;
    let names = match argument_separators(&arg)? {
      Some(separators) => separators
        .iter()
        .map(|separator| separator.meaning.name())
        .collect::<Vec<_>>()
        .join(","),
      None => String::from("unread"),
    };
    Ok(mouth::tokenize_internal(TeXString::assembled(names)))
  });
  Ok(())
}

/// The own runs ([`ArgSeparator::span`]) of `arg`'s separators that mean the kernel's `\and` ([`SepMeaning::Kernel`]):
/// written, held in a macro of the document's that holds only the separator, or a `\let` copy, as the query reads them
/// by meaning ([`argument_separators`]), each a run of whole units of `arg`'s top level (no group or environment holds
/// it, [`tokens_unit_end`]), in order and apart; `None` where the query reads nothing (no rehearsal made, over its allowance, a
/// group left open), for which a caller cuts by spelling. A separator of another meaning — a class's, a binding's or the
/// document's renewed `\and`, amsmath's `\And` — is no run here (trap T8: what it prints is its caller's to say), nor
/// one a macro holds together with ink (no span: the reading cuts that one at its `Sep(And)`, #993). The cut two bindings'
/// routes share (Option C P2, OXIDIZED_DESIGN_DIVERGENCES #995): eptcs's cells (P2-4, `eptcs_cls.rs`), the
/// `\institute` lists of inst_support, aa and llncs and llncs's marked `\author` list ([`split_at_kernel_separators`],
/// P2-2, P2-3).
pub fn kernel_separator_runs(arg: &Tokens) -> Result<Option<Vec<(usize, usize)>>> {
  let Some(separators) = argument_separators(arg)? else {
    return Ok(None);
  };
  let tokens = arg.unlist_ref();
  let units = tokens_top_level_units(tokens);
  let mut runs = Vec::new();
  let mut from = 0;
  for (start, end) in separators
    .iter()
    .filter(|found| found.meaning == SepMeaning::Kernel)
    .filter_map(|found| found.span)
  {
    if start < from || !units.contains(&start) || !units.contains(&end) {
      continue;
    }
    runs.push((start, end));
    from = end;
  }
  Ok(Some(runs))
}

/// `arg` split as [`split_tokens_delimited`] splits it at `delims`, cut first at each [`kernel_separator_runs`] run:
/// each segment between the runs split by spelling as it is, never a token rewritten (a run is cut where it stands,
/// eptcs's cells' way), the segment a run ends keeping its last piece even when empty ([`split_tokens_segment`]: the
/// split of the whole list keeps a piece a delimiter ends) and each segment's first piece the run before it as its
/// delimiter; and the number of runs cut that hold no delimiter of `delims` by spelling (`split_tokens`' match,
/// [`delimiter_token_matches`]): the cuts by meaning alone — a macro holding only `\and`, a `\let` copy of the kernel's
/// `\and` where the document renewed `\and`. A run is cut inside balanced parentheses too, which the spelling split
/// protects, as TeX's column break parts them (`Univ A (North \myand South)` after a document's `\let\myand\and`), and
/// a run the spelling also cuts is cut once. The spelling's cuts in the segments stay, whatever their token means (a
/// document's renewed `\and`, an undefined `\And`: trap T8, the cut the split made before, which aa.cls's own
/// title-time `\and` keeps, aa.cls:866), as do its trimming and its protections inside a segment. Where the query reads
/// nothing or finds no run, the spelling split of `arg` alone. Option C P2-2, P2-3 (OXIDIZED_DESIGN_DIVERGENCES #995):
/// the `\institute` lists of inst_support (`\lx@splitting@by@meaning`), aa (`aa_cls.rs` `\lx@aa@institute@dispatch`)
/// and llncs (`llncs_cls.rs` `\lx@llncs@institute@dispatch`), one contact call per piece, and llncs's marked `\author`
/// list (`\lx@llncs@author@split`), one author entry per piece.
pub fn split_at_kernel_separators(
  arg: Tokens,
  delims: Vec<SplitDelim>,
) -> Result<(DelimitedPieces, usize)> {
  let runs = kernel_separator_runs(&arg)?.unwrap_or_default();
  Ok(split_at_runs(arg, delims, runs))
}

/// [`split_at_kernel_separators`] at the kernel separator runs `runs` a caller kept.
pub(in crate::base_utilities) fn split_at_runs(
  arg: Tokens,
  delims: Vec<SplitDelim>,
  runs: Vec<(usize, usize)>,
) -> (DelimitedPieces, usize) {
  if runs.is_empty() {
    return (split_tokens_delimited(arg, delims), 0);
  }
  let spelled = |t: &Token| {
    delims
      .iter()
      .any(|d| matches!(d, SplitDelim::Token(d) if delimiter_token_matches(d, t)))
  };
  let tokens = arg.unlist_ref();
  let mut pieces = Vec::new();
  let mut by_meaning = 0;
  let mut before: Vec<Token> = Vec::new();
  let mut from = 0;
  // (each run, then the end of the list)
  for run in runs.into_iter().map(Some).chain([None]) {
    let (to, ended_by_cut) = run.map_or((tokens.len(), false), |(start, _)| (start, true));
    let segment = Tokens::new(tokens[from..to].to_vec());
    let mut split = split_tokens_segment(segment, delims.clone(), ended_by_cut);
    if let Some((first, _)) = split.first_mut() {
      *first = std::mem::take(&mut before);
    }
    pieces.extend(split);
    if let Some((start, end)) = run {
      if !tokens[start..end].iter().any(spelled) {
        by_meaning += 1;
      }
      before = tokens[start..end].to_vec();
      from = end;
    }
  }
  (pieces, by_meaning)
}

/// Whether `arg` holds a token its spelled `delims` do not match that could mean the kernel's `\and` here: a copy of it
/// ([`SepMeaning::Kernel`]) or a macro no binding made (the kernel's `\csname` too). Without one,
/// [`kernel_separator_runs`] finds no run the spelling does not cut, and no rehearsal is needed.
pub(in crate::base_utilities) fn may_hold_kernel_separator(
  arg: &Tokens,
  delims: &[SplitDelim],
) -> bool {
  arg.unlist_ref().iter().any(|t| {
    matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)
      && !delims
        .iter()
        .any(|d| matches!(d, SplitDelim::Token(d) if delimiter_token_matches(d, t)))
      && match lookup_meaning(t) {
        Some(Stored::Expandable(defn)) => !defn.get_origin().is_binding_made(),
        _ => meaning_of(t) == SepMeaning::Kernel,
      }
  })
}
