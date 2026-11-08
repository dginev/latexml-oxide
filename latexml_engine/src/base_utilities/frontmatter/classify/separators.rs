//! The text rules on printed ink and the token primitives of `author_block/separators.rs`, moved unchanged (MV1,
//! user ruling 2026-11-08 "knowledge in, driver out").

use super::*;

/// The line breaks an author line's printed separator may stand beside, with their optional `*` and `[…]`: a break
/// before the word or after it is no name's (`Ann Able\orcidID{…} and\\ Bob Baker`, svmult; `Badr~Kaoui$^{3,4}$\break
/// and Jens~Harting`, jfm 1311.3650; `\linebreak and`, mdpi 2509.22118), nor is a control return — a line ending in `\`,
/// which latex.ltx:558 `\def\^^M{\ }` prints as a space (2004.02831, 2404.12084).
pub(in crate::base_utilities) const SEPARATOR_LINE_BREAKS: [&str; 5] =
  ["\\\\", "\\break", "\\linebreak", "\\allowbreak", "\\\r"];

/// The index past the spacing that starts at `v[i]` (`i` when none): spaces, ties, space commands and lengths
/// ([`leading_spacing_end`]), the line breaks of [`SEPARATOR_LINE_BREAKS`], and — when `macros` — a parameterless
/// macro whose body is only spacing (jfm.cls:1025 `\newcommand\ns{\kern.5em\relax}`: `Chang Liu \ns and Dennice F.
/// Gayme`, 1912.02168).
pub(in crate::base_utilities) fn separator_gap_end(
  v: &[Token],
  mut i: usize,
  macros: bool,
) -> usize {
  while i < v.len() {
    let past = i + leading_spacing_end(&v[i..]);
    if past > i {
      i = past;
      continue;
    }
    let t = v[i];
    if SEPARATOR_LINE_BREAKS.iter().any(|cs| t == T_CS!(*cs)) {
      i += 1;
      if t == T_CS!("\\\\") && v.get(i) == Some(&T_OTHER!("*")) {
        i += 1;
      }
      if (t == T_CS!("\\\\") || t == T_CS!("\\linebreak"))
        && v.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = v[i..].iter().position(|u| *u == T_OTHER!("]"))
      {
        i += close + 1;
      }
    } else if macros && prints_only_spacing(&t) {
      i += 1;
    } else {
      break;
    }
  }
  i
}

/// Whether `cs` is a parameterless macro whose body prints only spacing (`\kern.5em\relax`, `\hskip1em`, `\quad`).
fn prints_only_spacing(cs: &Token) -> bool {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return false;
  }
  let Ok(Some(defn)) = lookup_definition(cs) else {
    return false;
  };
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  let body = body.unlist_ref();
  body.iter().any(|t| {
    !matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) && *t != T_CS!("\\relax")
  }) && only_spacing(body)
}

/// Whether `v` holds only spacing ([`separator_gap_end`]), `\relax` and groups of the same (`{\kern.5em}`); a spacing
/// command's braced argument is its own (`\hspace{1em}`, `\rule{0pt}{1ex}`).
pub(in crate::base_utilities) fn only_spacing(v: &[Token]) -> bool {
  let mut i = 0;
  while i < v.len() {
    let past = separator_gap_end(v, i, false);
    if past > i {
      i = past;
    } else if v[i] == T_CS!("\\relax") {
      i += 1;
    } else if v[i].get_catcode() == Catcode::BEGIN {
      let end = skip_group(v, i);
      if end > v.len()
        || end < i + 2
        || v[end - 1].get_catcode() != Catcode::END
        || !only_spacing(&v[i + 1..end - 1])
      {
        return false;
      }
      i = end;
    } else {
      return false;
    }
  }
  true
}

/// Whether `t` is a letter of a word: a letter, or a letter command (`\L`, `\ss`, `\dj`) or accent written in it.
pub(in crate::base_utilities) fn letterlike(t: &Token) -> bool {
  t.get_catcode() == Catcode::LETTER
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|s| letter_command(s).is_some() || ACCENT_COMMANDS.contains(&s)))
}

/// Whether `t` is a control word that is no letter of the word beside it: a name of two characters or more, or of one
/// letter (tex.web §354-356 — a control symbol's name is one non-letter), after which TeX skips the spaces, so the
/// command itself parts the words (`\rm`, `\break`, `\P`). A control symbol skips none: `\-` (a discretionary
/// hyphen) and `\/` (an italic correction) print inside a word (`Rol\-and Smith`, `Bertr\/and Smith` print "Roland
/// Smith", "Bertrand Smith"); a spacing one (`\,`, `\ `) is a gap of its own ([`separator_gap_end`]).
pub(in crate::base_utilities) fn parts_words(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && !letterlike(t)
    && t.with_str(|s| {
      let mut name = s.strip_prefix('\\').unwrap_or(s).chars();
      match (name.next(), name.next()) {
        (Some(_), Some(_)) => true,
        (Some(c), None) => c.is_alphabetic(),
        _ => false,
      }
    })
}

/// The end of the marks a name's tokens open with, with no space before them: `\textsuperscript{a}`, `$^{a}$`, the
/// shown symbol mark `\lx@frontmatter@keepsup{*}\lx@frontmatter@symbolmark{*}`, and a `\thanks{...}` or
/// `\footnote{...}`, whose mark prints with them (0 when none).
pub(in crate::base_utilities) fn leading_marks_end(tokens: &[Token]) -> usize {
  let mut i = 0;
  while let Some(&first) = tokens.get(i) {
    let next = if first == T_CS!("\\textsuperscript")
      || first == T_CS!("\\lx@frontmatter@keepsup")
      || first == T_CS!("\\lx@frontmatter@symbolmark")
      // aastex's mark (aas_support: an affiliation request), written after the comma as often as before it
      // (`Todd M. Tripp,\altaffilmark{2,3} Bart P. Wakker,\altaffilmark{4}`, astro-ph0302534)
      || first == T_CS!("\\altaffilmark")
    {
      sup_operand_at(tokens, i + 1).map(|(_, next)| next)
    } else if (first == T_CS!("\\thanks") || first == T_CS!("\\footnote"))
      && tokens
        .get(i + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      sup_operand_at(tokens, i + 1).map(|(_, next)| next)
    } else if first == T_MATH!() && tokens.get(i + 1) == Some(&T_SUPER!()) {
      sup_operand_at(tokens, i + 2)
        .filter(|(_, next)| tokens.get(*next) == Some(&T_MATH!()))
        .map(|(_, next)| next + 1)
    } else if first == T_OTHER!("*") {
      // a run of bare asterisks, a footnote mark written after the comma (`S. F. Pereira,* Z. Y. Ou,** and …`,
      // quant-ph0003094)
      Some(
        i + tokens[i..]
          .iter()
          .take_while(|t| **t == T_OTHER!("*"))
          .count(),
      )
    } else {
      None
    };
    match next {
      Some(next) => i = next,
      None => break,
    }
  }
  i
}

/// The font commands whose argument [`wrapper_over_lines_as_lines`] gives each line: [`NAME_TEXT_COMMANDS`]' `\text…` ones
/// and `\emph` — and that wrap one entry of a run (`notes::wrapped_mark_at`).
pub(in crate::base_utilities) fn is_text_font_command(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && t.with_str(|cs| {
      NAME_TEXT_COMMANDS.contains(&cs) && (cs.starts_with("\\text") || cs == "\\emph")
    })
}

/// The `\and` family only — the HARD author boundary the superscript-marker
/// branch groups on FIRST, so a marker-less line never merges into an author
/// from a previous `\and` group (html_feedback#1021 F2; OXIDIZED_DESIGN #52(g)).
/// With it the ACM SIG classes' `\alignauthor`, which opens each author's column (sig-alternate.cls, sigchi.cls:
/// `\author{\alignauthor Name\\ \affaddr{…}\\ \email{…} \alignauthor …}`): split here, its raw definition — a
/// tabular juggle for the class's title page — never runs inside a name. Perl's OmniBus makes it empty
/// (OmniBus.cls.ltxml:76) and its `@authorsplits` lacks it; since the arXiv profile runs these shipped classes raw
/// (DIVERGENCES #444) their authors were lost (1605.02827, 2003.09061, 1906.01122).
pub(in crate::base_utilities) fn author_and_splits() -> Vec<SplitDelim> {
  vec![
    T_CS!("\\and").into(),
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    T_CS!("\\alignauthor").into(),
  ]
}

// Things to split author & affiliation mix; NO comma in affiliations!!!
pub(in crate::base_utilities) fn author_affil_splits() -> Vec<SplitDelim> {
  vec![
    T_CS!("\\and").into(),
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    // Beyond-Perl: the literal word " and " is deliberately NOT a line-level
    // delimiter here (Perl's @authoraffilsplits includes it). This split runs
    // BEFORE author/affiliation classification, so splitting on " and " shreds
    // institution names that contain it — "Princeton Language and Intelligence"
    // → "Princeton Language" + "Intelligence, …" (re-joined without a space).
    // Authors written "Alice and Bob" still split: literal " and " is applied in
    // the author arm (comma_split_author_line), AFTER a line is known to be an
    // author line, so affiliations are never fragmented. Witness 2605.00347.
    T_CS!("\\quad").into(),
    T_CS!("\\qquad").into(),
    T_CS!("\\\\").into(),
    // latex.ltx:9256 `\DeclareRobustCommand\newline{\@normalcr\relax}`: the line break `\\` is (2401.14196's
    // `Ann Able$^{1}$, Bob Baker$^{1,2}$ \newline Cat Cole$^{1}$`)
    T_CS!("\\newline").into(),
  ]
}
