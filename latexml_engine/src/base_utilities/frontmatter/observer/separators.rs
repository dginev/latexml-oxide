//! A binding's query on an argument's author separators, by meaning (Option C step P2-0, OXIDIZED_DESIGN_DIVERGENCES #995):
//! the `\and`-family separators an argument holds at its own level, as the rehearsal digests it to events
//! (`EventKind::Sep(SepKind::And)` at nest 0: literal, held in a macro of the document's, a `\let` copy, a `\csname
//! and\endcsname`), each with whether its meaning *is* the kernel separator's. The rehearsal binds `\and`, `\And` and
//! `\AND` to its separator event whatever they mean here (`rehearsal::ALWAYS_BOUND`), so a `Sep` alone does not say the
//! class parts names there: jmlr's `\and` prints the word "and" (jmlr.cls:336), smfart's `&` (2508.07946), colm2025's,
//! jmlr2e's and neurips's a space (trap T8 of the P2 plan). The meaning is read outside the rehearsal, where the binding
//! calls the query.
//!
//! Two front-ends of the one query: [`argument_separators`] for a binding's Rust code, and `\lx@fm@separators{<tokens>}`
//! for a binding's macro text, expanding to the meanings' names in order, comma-separated (`kernel`, `binding`, `raw`),
//! empty for none, `unread` where no rehearsal could be made.

use latexml_core::{definition::origin::DefinitionOrigin, token::AUTHOR_PROV_POS_MASK};

use super::{
  events::{EventKind, SepKind, lex_events},
  ledger::field_stamped,
  policy::TitleMarks,
  rehearsal::{Rehearsal, rehearse},
  *,
};

/// What a separator means where the query is made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SepMeaning {
  /// the kernel's `\and` (`latex_constructs::sect05`'s constructor, which `\And` and `\AND` are `\let` to there): the
  /// kernel's author-line column, which the blocks' readings part names at
  Kernel,
  /// a binding's own definition (its code, a fallback class's, a stand-in's or a borrowing's,
  /// `DefinitionOrigin::is_binding_made`): what that binding says it means
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
/// `None` where no rehearsal could be made: a call inside one (no journal opens), over the budget, or an argument past
/// the field's positions. The rehearsal undoes everything it digested; the footnote steps it carries for the next
/// block's rehearsal are put back, so a query moves no reading. OXIDIZED_DESIGN_DIVERGENCES #995.
pub fn argument_separators(arg: &Tokens) -> Result<Option<Vec<ArgSeparator>>> {
  let content = arg.unlist_ref();
  if content.len() > usize::from(AUTHOR_PROV_POS_MASK) {
    return Ok(None);
  }
  const CARRIED: [&str; 2] = ["lx_fm_obs_counter_seen", "lx_fm_obs_steps_carried"];
  let carried = CARRIED.map(lookup_int);
  let read = rehearse(
    field_stamped(2, 0, content),
    TitleMarks::Fnsymbol,
    &[],
    lex_events,
  )?;
  for (key, value) in CARRIED.iter().zip(carried) {
    if lookup_int(key) != value {
      assign_value(key, Stored::from(value), Some(Scope::Global));
    }
  }
  let Rehearsal::Done { value: events, .. } = read else {
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
