//! The frontmatter API's line readers for an entry's marks: the mark-command aliases, a line's
//! notes and leading footnote symbol, and whether `\inst` prints its mark. Moved unchanged from `author_block/marks.rs`,
//! `separators.rs`, `symbols.rs` and `notes.rs` (Z18j).

use super::*;

/// `tokens` with every alias of a mark command written as the command it means ([`mark_command_meant`]), so the
/// readers of an author or affiliation block that match a command by name (`\textsuperscript`'s symbol and ordinal
/// rewrites) see the alias as the command.
pub(in crate::base_utilities) fn unalias_mark_commands(tokens: Tokens) -> Tokens {
  if !tokens.unlist_ref().iter().any(|t| {
    matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)
      && !mark_commands().contains(t)
      && is_mark_command(t)
  }) {
    return tokens;
  }
  Tokens::new(
    tokens
      .unlist()
      .into_iter()
      .map(|t| mark_command_meant(&t).unwrap_or(t))
      .collect(),
  )
}

/// Whether the document's `\inst` is the inst convention's mark, which takes back the space before it: its meaning starts
/// with `\unskip` (llncs.cls:894 `\def\inst#1{\unskip$^{#1}$}`, aa.cls:759, svjour3.cls, read raw), or it is the
/// request the convention's bindings give it for that definition (`\lx@request@frontmatter@annotation`: llncs_cls.rs,
/// inst_support_sty.rs, sv_support_sty.rs). Beamer's (beamerbasetitle.sty:169 `\textsuperscript{#1}`) and a document's
/// own `\newcommand{\inst}[1]{$^{#1}$}` are none.
/// Inside author content, where `\lx@author@withinst` makes `\inst` its mark reader, the one it saved decides; a class
/// that defines `\inst` only inside its title, where it prints the names (epl2.cls:725 `\def\inst##1{\unskip${}^{##1}$…}`,
/// 1201.1900), has its binding declare that meaning as `\lx@title@inst`, which decides first.
pub(in crate::base_utilities) fn inst_takes_back_space() -> bool {
  let first = |cs: &str| match lookup_definition(&T_CS!(cs)) {
    Ok(Some(defn)) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => body
        .unlist_ref()
        .iter()
        .find(|t| **t != T_SPACE!())
        .copied(),
      _ => None,
    },
    _ => None,
  };
  let lead = match first("\\lx@title@inst").or_else(|| first("\\inst")) {
    Some(t) if t == T_CS!("\\lx@inst@mark") => first("\\lx@saved@inst"),
    lead => lead,
  };
  lead.is_some_and(|t| t == T_CS!("\\unskip") || t == T_CS!("\\lx@request@frontmatter@annotation"))
}

/// Does the document's `\inst` print its one argument as a raised mark — the inst convention's
/// ([`inst_takes_back_space`]: aa.cls:759 `\def\inst#1{\unskip\hbox{$^{#1}$}}`, mem.cls:318, llncs), or a definition of
/// one parameter that raises it (beamerbasetitle.sty:169 `\textsuperscript{#1}`, a document's `$^{#1}$`)? By its
/// meaning, not its name: a document's `\newcommand{\inst}{Institute}` is a word (review rev99a S2).
pub(in crate::base_utilities) fn inst_prints_its_mark() -> bool {
  if inst_takes_back_space() {
    return true;
  }
  let Ok(Some(defn)) = lookup_definition(&T_CS!("\\inst")) else {
    return false;
  };
  let one_argument = defn.get_parameters().is_some_and(|p| {
    let params = p.get_parameters();
    params.len() == 1 && !params[0].optional
  });
  one_argument
    && matches!(defn.get_expansion(), Some(ExpansionBody::Tokens(body))
      if body.unlist_ref().iter().any(|t| *t == T_SUPER!() || is_mark_command(t)))
}

/// Does `line` open (after spaces, math shifts and braces) with a footnote symbol (`* Equal contribution`,
/// `$\dagger$ Corresponding author`, `\dag Corresponding author`) — a legend for the authors' symbol marks, not an
/// affiliation line? The text-mode `\dag`/`\ddag` count here only: as superscript marks (`$^\dag$`) they still label
/// affiliations (`is_footnote_symbol_operand` leaves them out, so `rewrite_symbol_superscripts` keeps them as marks).
pub(in crate::base_utilities) fn starts_with_footnote_symbol(line: &Tokens) -> bool {
  line
    .unlist_ref()
    .iter()
    .find(|t| {
      !matches!(
        t.get_catcode(),
        Catcode::SPACE | Catcode::MATH | Catcode::BEGIN | Catcode::END
      )
    })
    .is_some_and(|t| {
      *t == T_CS!("\\dag")
        || *t == T_CS!("\\ddag")
        || is_footnote_symbol_operand(std::slice::from_ref(t))
    })
}

/// The notes of a line of notes only, each the span `(start, end)` of `line` from its command
/// through its closing brace; `None` when the line holds other ink.
pub(in crate::base_utilities) fn line_notes(line: &[Token]) -> Option<Vec<(usize, usize)>> {
  let spacing = |t: &Token| *t == T_SPACE!() || *t == T_ACTIVE!('~') || *t == T_CS!("\\par");
  let mut notes = Vec::new();
  let mut i = 0;
  while i < line.len() {
    if spacing(&line[i]) {
      i += 1;
    } else {
      let end = note_call_end(line, i)?;
      notes.push((i, end));
      i = end;
    }
  }
  Some(notes)
}
