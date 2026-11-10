//! A class's title code read where it reads `\@author`: the definitions in force there, replayed for the author content
//! (ERR2, OXIDIZED_DESIGN_DIVERGENCES #983), and the statements a tabular juggle reads by their meaning
//! (`fallback::title_code_juggle`). It reads the class's code, not the old reader's: moved unchanged by Z18m from
//! `defs/frontmatter_fields/names.rs`, where ERR2 put it beside the moved field readers; `\lx@add@author`'s replay
//! (`defs/frontmatter_fields.rs`), `fallback.rs` and the emitter's held-undefined test (`emitter/content.rs`, with
//! `reads_arguments`, PC3's) import it from here.

use super::*;

/// The definitions in force where a class's title code reads `\@author`, to replay for the author content: its
/// statements ([`title_code_author_statements`]) but one that closes a group it did not open
/// ([`closes_only_its_own_groups`]: 2601.13359's `\def\affiliations{\egroup\par\Large\bgroup\rm}`, `\let\x\egroup`, a
/// tabular juggle `fallback::title_code_juggle` reads as a separator by its meaning) — the title code's layout between
/// its own groups, which the author content is not inside.
pub(in crate::base_utilities) fn title_code_author_definitions() -> Vec<(Token, Vec<Token>)> {
  title_code_author_statements()
    .into_iter()
    .filter(|(_, statement)| replayable(statement))
    .collect()
}

/// Whether the title-code statement `statement` ([`statements_at_author`]) is replayed for the author content: it closes
/// only the groups it opens ([`closes_only_its_own_groups`]).
pub(in crate::base_utilities) fn replayable(statement: &[Token]) -> bool {
  closes_only_its_own_groups(statement.get(2..).unwrap_or_default())
}

/// The definition statements in force where a class's title code reads `\@author`: those of the groups still open there
/// in its `\@maketitle` (else a dropped `\maketitle`'s) body — `\def`, `\gdef`, `\let`, `\newcommand`, `\providecommand`,
/// `\newcounter` (its definee `\c@<name>`) — with a global one kept past its group, in source order; each with its
/// definee ([`statements_at_author`]). Where a
/// binding keeps its own `\@maketitle` while the paper's copy of the file it stands in for carries another
/// (`state::copy_kept_meaning`, the copy's overlay: neurips_2022.sty:285-293's `\def\NewAnd`, 2205.14141, 2205.14539),
/// the copy's title code is the one pdflatex runs: its statements follow, a definee the live code defines first kept as
/// that defines it.
pub(in crate::base_utilities) fn title_code_author_statements() -> Vec<(Token, Vec<Token>)> {
  let at_author = |meaning: Option<Stored>| match meaning {
    Some(Stored::Expandable(definition)) => match definition.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => statements_at_author(body.unlist_ref()),
      _ => None,
    },
    _ => None,
  };
  let mut definitions = [T_CS!("\\@maketitle"), T_CS!("\\lx@dropped@maketitle")]
    .iter()
    .find_map(|cs| at_author(lookup_meaning(cs)))
    .unwrap_or_default();
  if let Some(copys) = at_author(copy_kept_meaning(&T_CS!("\\@maketitle"))) {
    for (definee, statement) in copys {
      if !definitions.iter().any(|(d, _)| *d == definee) {
        definitions.push((definee, statement));
      }
    }
  }
  definitions
}

/// The definition statements of `body`'s groups open at its first `\@author` ([`title_code_author_statements`]),
/// in source order; `None` when it reads no `\@author`. A macro the body defines whose statement holds `\@author` (or
/// such a macro) reads it where it is used: its first use is the reading point, aaai2026.sty:73's `\def\theauthors{\if
/// T\showauthors@on\@author\else …\fi}`, used at :98 inside the `\sbox{\titlearea}` group that defines `\coo` and
/// `\equalcontrib` (2508.00545, 2508.04047, 2508.04749, 2508.05996, 2508.08170). Groups are brace groups, `\bgroup`/`\egroup`,
/// `\begingroup`/`\endgroup` and environments. A statement runs locally where it is replayed: a `\gdef` is a `\def`
/// there, and the `\global` before a definition is left out. `\edef`/`\xdef` are not replayed (expanded now, not at
/// the title code), nor `\renewcommand` (of a command already defined, which stands). A statement whose shape is not
/// read whole (a `\def` without its body's `{`, a `\newcommand` without a body) is left out. One that closes a group it
/// did not open is kept, for its readers (a tabular juggle, `fallback::title_code_juggle`), and its tokens are read on as
/// the title code's own (its group steps counted).
pub(in crate::base_utilities) fn statements_at_author(
  body: &[Token],
) -> Option<Vec<(Token, Vec<Token>)>> {
  // the index after the balanced group starting at `i` (a BEGIN), else after the one token there
  let group_end = |i: usize| -> usize {
    if body.get(i).map(|t| t.get_catcode()) != Some(Catcode::BEGIN) {
      return (i + 1).min(body.len());
    }
    let mut depth = 0usize;
    for (j, t) in body.iter().enumerate().skip(i) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return j + 1;
          }
        },
        _ => {},
      }
    }
    body.len()
  };
  let skip_spaces = |mut j: usize| -> usize {
    while body.get(j) == Some(&T_SPACE!()) {
      j += 1;
    }
    j
  };
  // the index after the `[…]` options from `j` on, each with the spaces after it
  let past_options = |mut j: usize| -> usize {
    while body.get(j) == Some(&T_OTHER!("[")) {
      while j < body.len() && body[j] != T_OTHER!("]") {
        j += 1;
      }
      j = skip_spaces(j + 1);
    }
    j
  };
  // each statement with the group it is made in (None: global)
  let mut statements: Vec<(Token, Vec<Token>, Option<usize>)> = Vec::new();
  let mut open: Vec<usize> = vec![0];
  let mut groups = 0usize;
  // (the macros the body defines that read `\@author`: their first use reads it)
  let mut readers: Vec<Token> = Vec::new();
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    let name = t.with_str(str::to_string);
    let opens = t.get_catcode() == Catcode::BEGIN
      || matches!(name.as_str(), "\\bgroup" | "\\begingroup" | "\\begin");
    let closes = t.get_catcode() == Catcode::END
      || matches!(name.as_str(), "\\egroup" | "\\endgroup" | "\\end");
    if opens {
      groups += 1;
      open.push(groups);
      i += 1;
      continue;
    }
    if closes {
      if open.len() > 1 {
        open.pop();
      }
      i += 1;
      continue;
    }
    if t == T_CS!("\\@author") || readers.contains(&t) {
      return Some(
        statements
          .into_iter()
          .filter(|(_, _, group)| group.is_none_or(|g| open.contains(&g)))
          .map(|(definee, statement, _)| (definee, statement))
          .collect(),
      );
    }
    let global = name == "\\gdef" || (i > 0 && body[i - 1] == T_CS!("\\global"));
    // the definee, where the statement ends, and its first token as replayed
    let read: Option<(Option<Token>, usize, Token)> = match name.as_str() {
      "\\def" | "\\gdef" if t.get_catcode() == Catcode::CS => {
        // the parameter text runs to the body's `{`
        let mut j = i + 2;
        while j < body.len() && body[j].get_catcode() != Catcode::BEGIN {
          j += 1;
        }
        (j < body.len()).then(|| (body.get(i + 1).copied(), group_end(j), T_CS!("\\def")))
      },
      "\\let" if t.get_catcode() == Catcode::CS => {
        let mut j = skip_spaces(i + 2);
        if body.get(j) == Some(&T_OTHER!("=")) {
          j += 1;
        }
        j = skip_spaces(j);
        body
          .get(j)
          .filter(|target| !matches!(target.get_catcode(), Catcode::BEGIN | Catcode::END))
          .map(|_| (body.get(i + 1).copied(), j + 1, t))
      },
      // (a counter the title code declares for its own macros: aaai2026.sty:77's `\newcounter{corfn}`, which its `\co` reads
      // — the definee `\c@corfn`, replayed only while undefined, so a second pass declares nothing; 2508.04047)
      "\\newcounter" if t.get_catcode() == Catcode::CS => {
        let j = skip_spaces(i + 1);
        let end = group_end(j);
        let name: String = (body.get(j + 1..end.saturating_sub(1)).unwrap_or_default())
          .iter()
          .map(ToString::to_string)
          .collect();
        (!name.is_empty()).then(|| (Some(T_CS!(&s!("\\c@{name}"))), past_options(end), t))
      },
      "\\newcommand" | "\\providecommand" if t.get_catcode() == Catcode::CS => {
        let mut j = i + 1;
        if body.get(j) == Some(&T_OTHER!("*")) {
          j += 1;
        }
        j = skip_spaces(j);
        let definee = match body.get(j) {
          Some(b) if b.get_catcode() == Catcode::BEGIN => body.get(j + 1).copied(),
          other => other.copied(),
        };
        j = past_options(skip_spaces(group_end(j)));
        body
          .get(j)
          .filter(|b| b.get_catcode() != Catcode::END)
          .map(|_| (definee, group_end(j), t))
      },
      _ => None,
    };
    match read {
      Some((Some(definee), stop, head))
        if matches!(definee.get_catcode(), Catcode::CS | Catcode::ACTIVE) && stop <= body.len() =>
      {
        let mut statement = vec![head];
        statement.extend_from_slice(&body[i + 1..stop]);
        let own = closes_only_its_own_groups(&body[i + 2..stop]);
        if own
          && statement[2..]
            .iter()
            .any(|u| *u == T_CS!("\\@author") || readers.contains(u))
        {
          readers.push(definee);
        }
        statements.push((
          definee,
          statement,
          (!global).then(|| *open.last().unwrap_or(&0)),
        ));
        // (one closing a group it did not open is read on, its group steps the title code's)
        i = if own { stop } else { i + 1 };
      },
      _ => i += 1,
    }
  }
  None
}

/// Whether `tokens` (a definition's meaning) close only the groups and environments they open (`\bgroup`,
/// `\begingroup`, `\begin`): an author separator `\def\sep{\end{tabular}\begin{tabular}{c}}` does not.
fn closes_only_its_own_groups(tokens: &[Token]) -> bool {
  let mut depth = 0usize;
  for t in tokens {
    if t.get_catcode() != Catcode::CS {
      continue;
    }
    match t.with_str(str::to_string).as_str() {
      "\\bgroup" | "\\begingroup" | "\\begin" => depth += 1,
      "\\egroup" | "\\endgroup" | "\\end" => match depth.checked_sub(1) {
        Some(d) => depth = d,
        None => return false,
      },
      _ => {},
    }
  }
  true
}

/// Whether the title-code statement `statement` ([`title_code_author_statements`]) defines
/// `definee` reading arguments: a parameter in a `\def`'s parameter text (pazhastl.cls:112's
/// `\def\address##1{\textsuperscript{##1}}`), or a `\newcommand`'s `[n]` past its definee. Its argument prints what the
/// rehearsal, which runs without the title code's definitions, read as the block's own (`\address{1}`'s "1", a mark), so
/// the command is no undefined token for a piece to take alone (`emitter::content::Field::held_undefined_at`, PC3's;
/// Z18m moved it here with this reader); a parameterless one
/// (aaai2026.sty's `\co`, a note; a `\let`, a `\newcounter`'s `\c@x`) is.
pub(in crate::base_utilities::frontmatter) fn reads_arguments(
  definee: &Token,
  statement: &[Token],
) -> bool {
  let Some(at) = statement.iter().skip(1).position(|t| t == definee) else {
    return false;
  };
  // (its parameter text up to the body's group, past a braced definee's closing brace: a `\newcommand{\x}`'s `[1]`)
  statement[at + 2..]
    .iter()
    .skip_while(|t| t.group_step() == -1)
    .take_while(|t| t.group_step() != 1)
    .any(|t| t.get_catcode() == Catcode::PARAM || *t == T_OTHER!("["))
}

#[cfg(test)]
mod tests {
  use latexml_core::state::{State, StateOptions, set_state};

  use super::*;

  fn setup() {
    // T_CS!/T_BEGIN! etc. and tokenize_internal need a thread-local State for
    // string interning.
    set_state(State::new(StateOptions::default()));
  }
  fn tk(s: &'static str) -> Tokens { mouth::tokenize_internal(s) }

  /// The definitions of a title code `body` replayed for the author content, with their definees
  /// (`statements_at_author`, `replayable`).
  fn definitions_at_author(body: &'static str) -> Option<Vec<(Token, Vec<Token>)>> {
    statements_at_author(tk(body).unlist_ref()).map(|statements| {
      statements
        .into_iter()
        .filter(|(_, statement)| replayable(statement))
        .collect()
    })
  }

  /// The replayed statements of a title code `body`, as strings.
  fn replayed(body: &'static str) -> Vec<String> {
    definitions_at_author(body)
      .expect("reads \\@author")
      .into_iter()
      .map(|(_, statement)| Tokens::new(statement).to_string())
      .collect()
  }

  #[test]
  fn title_code_definitions_open_at_author() {
    setup();
    // a spaced `\newcommand [1] {…}` is read whole; `\let`; the last of two `\def`s comes last; a group closed before
    // `\@author` is not replayed
    assert_eq!(
      replayed(
        r"{\let\addr\itshape\newcommand\note [1] {[#1]}\def\dup{a}{\def\dup{b}\begingroup\def\gone{G}\endgroup\@author}}"
      ),
      [
        r"\let\addr\itshape",
        r"\newcommand\note[1] {[#1]}",
        r"\def\dup{a}",
        r"\def\dup{b}"
      ]
    );
  }

  #[test]
  fn title_code_definitions_global_local_and_skipped() {
    setup();
    // a `\gdef` (and `\global\def`) made in a closed group is kept, replayed as a local `\def`; `\edef`,
    // `\renewcommand` and a definition closing a group it did not open (2601.13359) are not replayed
    assert_eq!(
      replayed(
        r"{\gdef\g{1}\global\def\h{2}}{\edef\e{3}\renewcommand\r{4}\def\affiliations{\egroup\par\bgroup}\let\x\egroup\def\sep{\end{tabular}\begin{tabular}{c}}\def\ok{\begin{center}c\end{center}}\@author}"
      ),
      [
        r"\def\g{1}",
        r"\def\h{2}",
        r"\def\ok{\begin{center}c\end{center}}"
      ]
    );
    assert!(definitions_at_author(r"{\def\name{N}}").is_none());
  }

  #[test]
  fn title_code_counters_replay_by_their_count_register() {
    setup();
    // aaai2026.sty:77's `\newcounter{corfn}`, which its `\co` reads (2508.04047): replayed, its definee `\c@corfn` (so
    // only while that is undefined), the optional `[within]` with it
    let definitions = definitions_at_author(
      r"\newcounter{corfn}\setcounter{corfn}{0}\newcounter {eqfn}[section]\def\co{\value{corfn}}\@author",
    )
    .expect("reads \\@author");
    assert_eq!(
      definitions
        .iter()
        .map(|(definee, statement)| (
          definee.to_string(),
          Tokens::new(statement.clone()).to_string()
        ))
        .collect::<Vec<_>>(),
      [
        (r"\c@corfn".to_string(), r"\newcounter{corfn}".to_string()),
        (
          r"\c@eqfn".to_string(),
          r"\newcounter{eqfn}[section]".to_string()
        ),
        (r"\co".to_string(), r"\def\co{\value{corfn}}".to_string())
      ]
    );
  }
}
