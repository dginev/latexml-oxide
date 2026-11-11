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
  events::{EventKind, SepKind, lex_events},
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
  /// `get_cs()`), a kernel layer's: narrower than `frontmatter_stores::kernel_and_list`, which takes any pool constructor —
  /// `texvc_sty.rs:132`'s `\let\and\land` is `raw` here and kernel there (P2-6, which swaps that gate, meets it)
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
/// (`\And` and `\AND` are `\let` to it), as `frontmatter_stores::kernel_and_list` reads it.
const KERNEL_SEPARATOR: &str = "\\and";

/// What `token` means now ([`SepMeaning`]): the kernel separator is the [`KERNEL_SEPARATOR`] constructor a kernel layer
/// made (a pool, a dump, a format: `is_latexml_owned` and not `is_binding_made`).
fn meaning_of(token: &Token) -> SepMeaning {
  match lookup_meaning(token) {
    Some(Stored::Constructor(defn)) => {
      let kernel = defn.origin.is_latexml_owned() && !defn.origin.is_binding_made();
      if kernel && *defn.get_cs() == T_CS!(KERNEL_SEPARATOR) {
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

/// The token whose meaning a separator has, its span `span` of `content`: the one token the span covers (a literal `\and`,
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
    ..
  } = read
  else {
    return Ok(None);
  };
  let separators = events.iter().filter_map(|e| match e.kind {
    EventKind::Sep(SepKind::And(command)) if e.nest == 0 => Some(ArgSeparator {
      command,
      meaning: meaning_of(&printing_token(content, e.span, command)),
    }),
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
