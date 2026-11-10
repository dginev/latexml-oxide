//! The text rules on printed ink and the token primitives of `author_block/separators.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;
use crate::frontmatter_stores::LAYOUT;

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

/// The font commands whose argument `wrapper_over_lines_as_lines` gives each line: [`NAME_TEXT_COMMANDS`]' `\text…` ones
/// and `\emph` — and that wrap one entry of a run (`notes::wrapped_mark_at`).
pub(in crate::base_utilities) fn is_text_font_command(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && t.with_str(|cs| {
      NAME_TEXT_COMMANDS.contains(&cs) && (cs.starts_with("\\text") || cs == "\\emph")
    })
}

/// Perl: positionOf($tokens, @delims) — Base_Utility.pool.ltxml (PR #2767).
/// Find the position of a Token from `delims` within `tokens`.
/// 1 based, so None == token not present.
pub fn position_of(tokens: &Tokens, delims: &[Token]) -> Option<usize> {
  for (i, t) in tokens.unlist_ref().iter().enumerate() {
    if delims.contains(t) {
      return Some(i + 1);
    }
  }
  None
}

// Things to split authors (Perl PR #2767, Base_Utility.pool.ltxml)
// This is " and " without the spaces stripped.
pub(in crate::base_utilities) fn literal_and() -> SplitDelim {
  let mut tks = vec![T_SPACE!()];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(T_SPACE!());
  SplitDelim::Tokens(Tokens::new(tks))
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

// (moved unchanged from `author_block/separators.rs` by Z18m, which deleted `author_block/`: Author-line separators:
// line parts, words, `and` splits, alignment and ACM column unwrapping.)
/// If `tokens` (spaces around it aside) is one brace group, its opening declarations ([`opening_declarations`]:
/// `\bf`, `\small`) and the rest of its content.
pub(in crate::base_utilities) fn whole_line_group(tokens: &Tokens) -> Option<(Vec<Token>, Tokens)> {
  let v = tokens.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!())?;
  let end = v.iter().rposition(|t| *t != T_SPACE!())?;
  if end <= start || v[start].get_catcode() != Catcode::BEGIN || skip_group(v, start) != end + 1 {
    return None;
  }
  let inner = Tokens::new(v[start + 1..end].to_vec());
  let declarations = opening_declarations(&inner);
  let rest = Tokens::new(inner.unlist_ref()[declarations.len()..].to_vec());
  Some((declarations, rest))
}

/// What an author line's top level is made of, for [`spaced_name_separators`]: the spacing or line breaks between two
/// words (a gap), the word "and" or `\&`, a brace group, a math span, a span the split keeps whole (balanced
/// parentheses, an environment), or one token.
#[derive(Clone, Copy, PartialEq)]
enum LinePart {
  Gap(usize, usize),
  And(usize),
  Ampersand(usize),
  Group(usize, usize),
  Math(usize, usize),
  Span(usize, usize),
  Token(usize),
}

/// The parts of an author line's top level ([`LinePart`]).
fn line_parts(v: &[Token]) -> Vec<LinePart> {
  let and = [T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")];
  let mut parts = Vec::with_capacity(v.len());
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    let gap = separator_gap_end(v, i, true);
    let (part, end) = if gap > i {
      (LinePart::Gap(i, gap), gap)
    } else if t.get_catcode() == Catcode::BEGIN {
      let end = skip_group(v, i).min(v.len());
      (LinePart::Group(i, end), end)
    } else if t.defined_as(&T_MATH!()) {
      let end = (i + 1..v.len())
        .find(|&j| v[j].defined_as(&T_MATH!()))
        .map_or(v.len(), |j| j + 1);
      (LinePart::Math(i, end), end)
    } else if v[i..].starts_with(&and) {
      (LinePart::And(i), i + 3)
    } else if t == T_CS!("\\&") {
      (LinePart::Ampersand(i), i + 1)
    } else if t == T_OTHER!("(")
      && paren_closes_ahead(&v[i + 1..].iter().copied().collect::<VecDeque<Token>>())
    {
      let mut level = 0i32;
      let mut end = i;
      while end < v.len() {
        if v[end] == T_OTHER!("(") {
          level += 1;
        } else if v[end] == T_OTHER!(")") {
          level -= 1;
        }
        end += 1;
        if level == 0 {
          break;
        }
      }
      (LinePart::Span(i, end), end)
    } else if t == T_CS!("\\begin")
      && let Some(span) =
        environment_closes_ahead(&v[i + 1..].iter().copied().collect::<VecDeque<Token>>())
    {
      (LinePart::Span(i, i + 1 + span), i + 1 + span)
    } else {
      (LinePart::Token(i), i + 1)
    };
    parts.push(part);
    i = end;
  }
  parts
}

/// Whether the part before a separator word ends the word before it: a gap; a control word that is no letter (`\L`,
/// `\ss`) or accent ([`parts_words`]); a note's or mark's argument (`Dan Dorn\thanks{…}and Eve Ess`, jfm 2005.00317),
/// or a group whose text does not end in a letter or a control symbol (`{\bf Ann Able}and` prints "Ann Ableand",
/// `{Rol\-}and` "Roland"); a math span holding only a superscript, a mark (`Bao-zhi Sun$^\ast$\&Shao-Ming Fei`,
/// 1707.03934); a span the split keeps whole.
fn ends_a_word(v: &[Token], parts: &[LinePart], at: usize) -> bool {
  match parts[at] {
    LinePart::Gap(..) | LinePart::Span(..) => true,
    LinePart::Token(i) => parts_words(&v[i]),
    LinePart::Group(start, end) => {
      let owner = at.checked_sub(1).and_then(|before| match parts[before] {
        LinePart::Token(k) if v[k].get_catcode() == Catcode::CS => Some(v[k]),
        _ => None,
      });
      if let Some(owner) = owner {
        if is_name_annotation(&owner) || is_footnote_mark(&owner) {
          return true;
        }
        if letterlike(&owner) {
          return false;
        }
      }
      // (the group's last token: a letter, a letter command, a control symbol printing no space, or an inner group's
      // end continue the word)
      let last = v[start + 1..end.saturating_sub(1)]
        .iter()
        .rev()
        .find(|t| **t != T_SPACE!());
      !last.is_some_and(|t| {
        letterlike(t)
          || t.get_catcode() == Catcode::END
          || (t.get_catcode() == Catcode::CS
            && !parts_words(t)
            && separator_gap_end(std::slice::from_ref(t), 0, false) == 0)
      })
    },
    LinePart::Math(start, end) => {
      v.get(start + 1) == Some(&T_SUPER!())
        && sup_operand_at(v, start + 2).is_some_and(|(_, next)| {
          let mut next = next;
          while v.get(next) == Some(&T_SPACE!()) {
            next += 1;
          }
          next + 1 == end
        })
    },
    LinePart::And(_) | LinePart::Ampersand(_) => false,
  }
}

/// Whether the part after a separator word begins the next one: a gap (`Eve Ess and\;Fay Fox`, 2502.11348), or a
/// control word that is no letter or accent ([`parts_words`]: `and\rm`, not `and\-`).
fn begins_a_word(v: &[Token], part: LinePart) -> bool {
  match part {
    LinePart::Gap(..) => true,
    LinePart::Token(i) => parts_words(&v[i]),
    _ => false,
  }
}

/// An author line's name separators as the split reads them, at its top level (the split's re-reads of a wrapper's
/// content see the content's own):
///  - the word "and" between two word boundaries, and `\&` after one, written ` and `, ` \& `, the line
///    breaks and spacing commands beside them dropped. TeX skips the spaces after a control word, so `\space and`,
///    `\break and`, `\ns and` hold no space token; nor do `\thanks{…}and`, `$^\ast$\&`, `and\;`, svmult's `and\\` —
///    each "and" the author wrote between two names, whether pdflatex prints it apart or, after a control word, glued to
///    it ("ATLAS∗and", OXIDIZED_DESIGN_DIVERGENCES #765: a reading of the author's intent) (1311.3650, 1503.04190, 1505.06278, 2005.00317, 1912.02168,
///    2509.22118, 1504.03105, 1607.04619, 2004.02831, 2404.12084, 2502.11348, astro-ph9704233, 1707.03934, 2401.17237,
///    2506.15327, 2509.11693, 2403.07875, 2512.18788, 2603.27055). Perl's only such separator is " and " between
///    spaces (Base_Utility.pool.ltxml:678), so it keeps these names one author (OXIDIZED_DESIGN_DIVERGENCES #765). A
///    letter or a letter or accent command beside the word keeps it in its name ("Rand", "Anders", `Zo\"{e}and`), and
///    "AT\&T" stays one word;
///  - a comma at the edge of a math span that holds only a superscript mark, printed between the names, is written
///    outside it, where the split sees it: `Wittor$^{1}, $Brüggen` (1610.00129), `Iacobuta$,^{5,6,7}$ Mabel`
///    (2108.04461), `Manoj$^{2},$ Chandra` (2402.11182) — a mark after the comma then goes to the name before it
///    (the reading's `resolver/lines.rs` `mark_goes_next`).
pub(in crate::base_utilities) fn spaced_name_separators(line: Tokens) -> Tokens {
  let v = line.unlist_ref();
  let parts = line_parts(v);
  let single_space = |part: Option<&LinePart>| match part {
    Some(LinePart::Gap(a, b)) => b - a == 1 && (v[*a] == T_SPACE!() || v[*a] == T_ACTIVE!('~')),
    _ => false,
  };
  // (a declaration's name read in place: no symbol interned per declaration for every token of the line)
  let layout = |t: &Token| t.get_catcode() == Catcode::CS && t.with_str(|cs| LAYOUT.contains(&cs));
  let mut out: Vec<Token> = Vec::with_capacity(v.len() + 4);
  let mut changed = false;
  // the gap a separator just written took in (dropped), and the length of the gap last written, at the end of `out`
  let mut skip_next_gap = false;
  let mut written_gap: Option<usize> = None;
  // the place and length in `out` of the gap last written when only declarations that typeset nothing follow it
  // (`\sc Ann Able \rm and \sc Bob Baker`: the gap before `\rm` is the separator's too, no name's; not before
  // `\LaTeX`, which prints a word)
  let mut gap_before_layout: Option<(usize, usize)> = None;
  for (at, part) in parts.iter().enumerate() {
    let (word, start, len) = match *part {
      LinePart::And(i) => (true, i, 3),
      LinePart::Ampersand(i) => (false, i, 1),
      LinePart::Gap(start, end) => {
        if std::mem::take(&mut skip_next_gap) {
          (written_gap, gap_before_layout) = (None, None);
        } else {
          gap_before_layout = Some((out.len(), end - start));
          out.extend_from_slice(&v[start..end]);
          written_gap = Some(end - start);
        }
        continue;
      },
      LinePart::Math(start, end) => {
        (skip_next_gap, written_gap, gap_before_layout) = (false, None, None);
        if let Some(moved) = comma_out_of_mark_math(&v[start..end]) {
          out.extend(moved);
          changed = true;
        } else {
          out.extend_from_slice(&v[start..end]);
        }
        continue;
      },
      LinePart::Group(start, end) | LinePart::Span(start, end) => {
        (skip_next_gap, written_gap, gap_before_layout) = (false, None, None);
        out.extend_from_slice(&v[start..end]);
        continue;
      },
      LinePart::Token(i) => {
        (skip_next_gap, written_gap) = (false, None);
        if !layout(&v[i]) {
          gap_before_layout = None;
        }
        out.push(v[i]);
        continue;
      },
    };
    // ("and" needs both boundaries; `\&`, no letter, only the one before it: `$^\ast$\&Shao-Ming Fei`, but `AT\&T`)
    let before = at.checked_sub(1).is_some_and(|b| ends_a_word(v, &parts, b));
    let after = parts
      .get(at + 1)
      .is_some_and(|next| begins_a_word(v, *next));
    let separates = before && (after || !word);
    // (between two plain spaces or ties the split's own separators match as written)
    let plain = at
      .checked_sub(1)
      .is_some_and(|b| single_space(parts.get(b)))
      && single_space(parts.get(at + 1));
    if !separates || plain {
      (skip_next_gap, written_gap, gap_before_layout) = (false, None, None);
      out.extend_from_slice(&v[start..start + len]);
      continue;
    }
    changed = true;
    if let Some(gap) = written_gap.take() {
      out.truncate(out.len() - gap);
    } else if let Some((place, gap)) = gap_before_layout {
      out.drain(place..place + gap);
    }
    gap_before_layout = None;
    out.push(T_SPACE!());
    out.extend_from_slice(&v[start..start + len]);
    out.push(T_SPACE!());
    skip_next_gap = true;
  }
  if changed { Tokens::new(out) } else { line }
}

/// A math span holding only a superscript mark and a comma at its edge (`$^{1}, $`, `$,^{5}$`), with the comma written
/// outside it: `$^{1}$,`, `,$^{5}$`. `None` for any other span.
fn comma_out_of_mark_math(span: &[Token]) -> Option<Vec<Token>> {
  let (open, close) = (*span.first()?, *span.last()?);
  if span.len() < 4 || !close.defined_as(&T_MATH!()) {
    return None;
  }
  let inner: Vec<Token> = span[1..span.len() - 1]
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  let comma = T_OTHER!(",");
  let mark_only = |mark: &[Token]| {
    mark.first() == Some(&T_SUPER!())
      && sup_operand_at(mark, 1).is_some_and(|(_, next)| next == mark.len())
  };
  if inner.last() == Some(&comma) && mark_only(&inner[..inner.len() - 1]) {
    let mut out = vec![open];
    out.extend_from_slice(&inner[..inner.len() - 1]);
    out.extend([close, comma]);
    Some(out)
  } else if inner.first() == Some(&comma) && mark_only(&inner[1..]) {
    let mut out = vec![comma, open];
    out.extend_from_slice(&inner[1..]);
    out.push(close);
    Some(out)
  } else {
    None
  }
}

// GROUP-level separators for the no-marker author heuristic (OXIDIZED_DESIGN
// #52): the \and family plus \quad/\qquad, but NOT the comma and NOT the literal
// " and ". A comma separates NAMES within one author line ("Alice, Bob") and the
// tokens WITHIN a multi-part address ("Laurel, MD 20723") — so splitting author
// *groups* on it shreds addresses into fake authors and turns the following
// \email line into a bogus affiliation (witness arXiv:2606.00315, NeurIPS
// idiom). Perl's @authorsplits (Base_Utility.pool.ltxml L679) includes the comma
// and shares this mis-split; excluding it here is a deliberate surpass-Perl
// divergence. Name-level comma/" and " splitting still happens, but only AFTER a
// line is known to be author names — in the reading's names line (`resolver/lines.rs`
// `names_line`) — never across address text. (Compare author_affil_splits(): "NO comma in affiliations!!!".)
pub(in crate::base_utilities) fn author_group_splits() -> Vec<SplitDelim> {
  let mut splits = author_and_splits();
  splits.extend([T_CS!("\\quad").into(), T_CS!("\\qquad").into()]);
  splits
}

/// Whether an author argument holds a separator the kernel's author read parts it at: as that read sees it
/// (`normalize_hspace_separators`: `\hspace`, `\hfill` and a spacing run between two names as `\quad`; the space a name
/// sets before its own note or mark none) it splits at `author_group_splits` or at the line breaks of
/// `author_affil_splits` into two pieces or more that are not empty. [`split_tokens`] leaves a note's argument, a group
/// and math alone. For a binding that reads an `\author` with none as one name: authblk's.
pub fn has_author_block_separator(stuff: &Tokens) -> bool {
  let separators = author_group_splits()
    .into_iter()
    .chain(author_affil_splits())
    .collect();
  split_tokens(normalize_hspace_separators(stuff.clone()), separators)
    .iter()
    .filter(|piece| !piece.is_empty())
    .count()
    > 1
}
