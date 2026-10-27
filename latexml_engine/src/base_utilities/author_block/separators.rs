//! Author-line separators: line parts, words, `and` splits, alignment and ACM column unwrapping.

use super::*;

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

/// In a list of names whose marks follow them, a mark glued to the comma after a name is that name's: `Ann
/// Able,\textsuperscript{a} Bob Baker\textsuperscript{b}` gives Ann `a` (2609.25924; repro
/// sectioning-frontmatter/author_suffix_marks_after_commas_link_affiliations; Perl gives Bob both). The marks move in
/// front of the comma before the line is split, glued commas between marks merging (`Ann$^1$,$^2$,$^3$, Bob` is one
/// name with three marks). Not across a space (`Ann$^1$, $^{*}$Bob` keeps Bob's star), and not when the names open
/// with their marks (`marker_leads`: `\large $^1$Ann Able, $^2$Bob Baker`). A `\thanks` glued among them goes too
/// (`Kerstin Hötte,$^{1}$\thanks{Corresponding author: kerstin.hotte@...}`, 2609.37343). The inst convention's marks
/// go across the space too: llncs.cls:894-896 `\def\inst#1{\unskip$^{#1}$}`, `\def\fnmsep{\unskip$^,$}` (aa.cls the
/// same, `\unskip\hbox{…}`) take back the space before them, so `Clara Dehman,⏎\inst{1}\fnmsep…\thanks{…} Mario
/// Centelles,\inst{5}` prints "Clara Dehman,¹,²,³,⁴⋆ Mario Centelles,⁵": Clara's marks and note, not Mario's (2401.16957,
/// whose four affiliations and e-mail note went to Mario Centelles; [`leading_inst_marks_end`]). Only where `\inst` means
/// the convention's mark ([`inst_takes_back_space`]): beamer's `\inst` (beamerbasetitle.sty:169 `\textsuperscript{#1}`)
/// and a document's `\newcommand{\inst}[1]{$^{#1}$}` take back no space, so `Ann Able\inst{1}, \inst{2}Bob Baker` is
/// Bob's 2 there; and not when the names open with their `\inst`s (`\inst{1}Ann Able, \inst{2}Bob Baker`, each its
/// own, as `marker_leads` keeps `$^1$Ann Able, $^2$Bob Baker`; rev78f probes c01, c08, c09).
pub(in crate::base_utilities) fn marks_before_glued_commas(line: Tokens) -> Tokens {
  let inst = inst_takes_back_space();
  let inst_leads = || {
    let v = line.unlist_ref();
    v.get(leading_unprinted_end(v)) == Some(&T_CS!("\\inst"))
  };
  if marker_leads(&line) || (inst && inst_leads()) {
    return line;
  }
  let src = line.unlist();
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut pending_comma: Option<Token> = None;
  let mut depth = 0i32;
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && t == T_OTHER!(",") {
      let end = leading_inst_marks_end(&src[i + 1..], inst);
      if end > 0 {
        out.extend_from_slice(&src[i + 1..i + 1 + end]);
        pending_comma = Some(t);
        i += 1 + end;
        continue;
      }
      // (an `\inst` mark after the comma's space: the space it takes back is dropped with it)
      let gap = src[i + 1..]
        .iter()
        .take_while(|u| **u == T_SPACE!() || **u == T_ACTIVE!('~'))
        .count();
      if inst && gap > 0 && src.get(i + 1 + gap) == Some(&T_CS!("\\inst")) {
        let end = leading_inst_marks_end(&src[i + 1 + gap..], inst);
        if end > 0 {
          out.extend_from_slice(&src[i + 1 + gap..i + 1 + gap + end]);
          pending_comma = Some(t);
          i += 1 + gap + end;
          continue;
        }
      }
      // a glued comma before this one merges into it: commas matter here only as the points the line splits at
      // (the unsplit `whole_line_cs_wrapper` fallback shows one comma for the two)
      pending_comma = None;
      out.push(t);
      i += 1;
      continue;
    }
    if let Some(comma) = pending_comma.take() {
      out.push(comma);
    }
    out.push(t);
    i += 1;
  }
  if let Some(comma) = pending_comma {
    out.push(comma);
  }
  Tokens::new(out)
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
fn only_spacing(v: &[Token]) -> bool {
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

/// Whether `t` is a letter of a word: a letter, or a letter command (`\L`, `\ss`, `\dj`) or accent written in it.
fn letterlike(t: &Token) -> bool {
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
///    ([`marks_before_glued_commas`]).
pub(in crate::base_utilities) fn spaced_name_separators(line: Tokens) -> Tokens {
  let v = line.unlist_ref();
  let parts = line_parts(v);
  let single_space = |part: Option<&LinePart>| match part {
    Some(LinePart::Gap(a, b)) => b - a == 1 && (v[*a] == T_SPACE!() || v[*a] == T_ACTIVE!('~')),
    _ => false,
  };
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
        if !crate::frontmatter_stores::LAYOUT
          .iter()
          .any(|cs| v[i] == T_CS!(*cs))
        {
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

/// [`leading_marks_end`] with the inst convention's marks: `\inst{…}` (llncs.cls:894, aa.cls, webofc.cls) and the
/// `\fnmsep`/`\fnsep` between two marks (llncs.cls:896), which an author line writes after the comma as often as before
/// it (`Mario Centelles,\inst{5}\fnmsep\inst{6}`, 2401.16957).
/// Only [`leading_marks_end`] when `inst` is false: an `\inst` that is not the convention's ([`inst_takes_back_space`]).
fn leading_inst_marks_end(tokens: &[Token], inst: bool) -> usize {
  let mut i = 0;
  loop {
    let marks = leading_marks_end(&tokens[i..]);
    if marks > 0 {
      i += marks;
      continue;
    }
    if !inst {
      return i;
    }
    match tokens.get(i) {
      Some(t) if *t == T_CS!("\\inst") => match sup_operand_at(tokens, i + 1) {
        Some((_, next)) => i = next,
        None => return i,
      },
      Some(t)
        if i > 0
          && (*t == T_CS!("\\fnmsep") || *t == T_CS!("\\fnsep"))
          && leading_inst_marks_end(&tokens[i + 1..], inst) > 0 =>
      {
        i += 1
      },
      _ => return i,
    }
  }
}

/// Whether the document's `\inst` is the inst convention's mark, which takes back the space before it: its meaning starts
/// with `\unskip` (llncs.cls:894 `\def\inst#1{\unskip$^{#1}$}`, aa.cls:759, svjour3.cls, read raw), or it is the
/// request the convention's bindings give it for that definition (`\lx@request@frontmatter@annotation`: llncs_cls.rs,
/// inst_support_sty.rs, sv_support_sty.rs). Beamer's (beamerbasetitle.sty:169 `\textsuperscript{#1}`) and a document's
/// own `\newcommand{\inst}[1]{$^{#1}$}` are none.
/// Inside author content, where `\lx@author@withinst` makes `\inst` its mark reader, the one it saved decides; a class
/// that defines `\inst` only inside its title, where it prints the names (epl2.cls:725 `\def\inst##1{\unskip${}^{##1}$…}`,
/// 1201.1900), has its binding declare that meaning as `\lx@title@inst`, which decides first.
fn inst_takes_back_space() -> bool {
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

pub fn and_split(cs: Token, tokens: Tokens) -> Vec<Token> {
  // Perl: SplitTokens($tokens, T_CS('\and'))
  // Only split on \and. The meaning-based check in split_tokens also matches
  // \AND (which is Let to \and). \And is NOT split here — amsmath overrides
  // its definition with DefMath, so it stays as a text "&" separator inside
  // <personname>, matching Perl's behavior.
  split_tokens(tokens, vec![SplitDelim::Token(T_CS!("\\and"))])
    .into_iter()
    .flat_map(|t| {
      let mut with_cs = vec![cs, T_BEGIN!()];
      with_cs.extend(t.unlist());
      with_cs.push(T_END!());
      with_cs
    })
    .collect()
}

/// Whether an author block is an IJCAI sectioned one: `\affiliations` or `\emails` at its top level (the split's
/// `Until:` delimiters match nothing deeper), each undefined, `\relax` or a no-op there — the ijcai binding's
/// (`ijcai_sty.rs`), or a raw derivative's that defines them only in `\@maketitle` (ttm.sty).
pub(in crate::base_utilities) fn has_ijcai_section_marker(stuff: &Tokens) -> Result<bool> {
  let markers = [T_CS!("\\affiliations"), T_CS!("\\emails")];
  let mut depth = 0i32;
  for token in stuff.unlist_ref() {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ if depth == 0 && markers.contains(token) => {
        let separator = token.defined_as(&TOKEN_RELAX)
          || match lookup_definition_stored(token)? {
            None => true,
            Some(Stored::Expandable(e)) => {
              e.paramlist
                .as_ref()
                .is_none_or(|p| p.get_parameters().is_empty())
                && matches!(&e.expansion, Some(ExpansionBody::Tokens(t)) if t.is_empty())
            },
            Some(_) => false,
          };
        if separator {
          return Ok(true);
        }
      },
      _ => {},
    }
  }
  Ok(false)
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

/// " and~", the "and" tied to the name after it (`Luca~Varotto, Angelo~Cenedese, and~Andrea~Cavallaro`, IEEE
/// 2011.10474, 2408.09035; svjour3 1406.6147). Unsplit, the "and" was read as the first word of the last name (63j).
pub(in crate::base_utilities) fn literal_and_tie() -> SplitDelim {
  word_and_between(T_SPACE!(), T_ACTIVE!('~'))
}
/// "~and " and "~and~", the "and" tied to the name before it (`Yasuhiro Ohta\ddag~and Kenji Kajiwara`, nlin0101056).
pub(in crate::base_utilities) fn tie_and_literal() -> SplitDelim {
  word_and_between(T_ACTIVE!('~'), T_SPACE!())
}
pub(in crate::base_utilities) fn tie_and_tie() -> SplitDelim {
  word_and_between(T_ACTIVE!('~'), T_ACTIVE!('~'))
}
/// `{\ and}`, an "and" given its own space in a group (`Huw Price {\ and} Ken Wharton`, 1508.01140).
pub(in crate::base_utilities) fn group_and() -> SplitDelim {
  let mut tks = vec![T_BEGIN!(), T_CS!("\\ ")];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(T_END!());
  SplitDelim::Tokens(Tokens::new(tks))
}
/// The word "and" between `before` and `after`.
fn word_and_between(before: Token, after: Token) -> SplitDelim {
  let mut tks = vec![before];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(after);
  SplitDelim::Tokens(Tokens::new(tks))
}
/// " \& " between two names, its spaces kept.
pub(in crate::base_utilities) fn literal_ampersand() -> SplitDelim {
  SplitDelim::Tokens(Tokens::new(vec![T_SPACE!(), T_CS!("\\&"), T_SPACE!()]))
}
/// `\&` with a tie on either side (`Kai Han~\&~Yunhe Wang`, 2601.13599), as the "and" ties are.
pub(in crate::base_utilities) fn tied_ampersands() -> [SplitDelim; 3] {
  let amp = |before: Token, after: Token| {
    SplitDelim::Tokens(Tokens::new(vec![before, T_CS!("\\&"), after]))
  };
  [
    amp(T_ACTIVE!('~'), T_ACTIVE!('~')),
    amp(T_ACTIVE!('~'), T_SPACE!()),
    amp(T_SPACE!(), T_ACTIVE!('~')),
  ]
}
// Things to split authors (Perl PR #2767, Base_Utility.pool.ltxml)
// This is " and " without the spaces stripped.
pub(in crate::base_utilities) fn literal_and() -> SplitDelim {
  let mut tks = vec![T_SPACE!()];
  tks.extend(mouth::tokenize_internal("and").unlist());
  tks.push(T_SPACE!());
  SplitDelim::Tokens(Tokens::new(tks))
}
/// An alignment environment (`center`, `flushleft`, `flushright`) that wraps a whole author block, or a whole `\and`
/// group, is layout around the lines the splitter reads, not one unit of them: its content, so its names, `\\`
/// lines and marks split as written (`\author{\begin{center}Ann Able$^{1}$, Bob Baker$^{2}$ \\ $^1$Univ A
/// ...\end{center}}`). One inside a piece — a `{center}` affiliation under the names — stays whole
/// (`split_tokens_delimited`; 2609.01563, 17357). Nested wrappers unwrap in turn.
pub(in crate::base_utilities) fn unwrap_alignment_environment(tokens: Tokens) -> Tokens {
  let mut toks = tokens.unlist();
  loop {
    let start = toks.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = toks.len() - toks.iter().rev().take_while(|t| **t == T_SPACE!()).count();
    if start >= end || toks[start] != T_CS!("\\begin") {
      break;
    }
    let stream: VecDeque<Token> = toks[start + 1..end].iter().copied().collect();
    let Some(span) = environment_closes_ahead(&stream) else {
      break;
    };
    let name_end = (start + 2..end)
      .find(|&k| toks[k].get_catcode() == Catcode::END)
      .unwrap_or(start + 1);
    let name: String = toks[start + 2..name_end]
      .iter()
      .map(|t| t.to_string())
      .collect();
    // The environment must end where the tokens do, its `\end{name}` the last thing.
    if start + 1 + span != end || !matches!(name.as_str(), "center" | "flushleft" | "flushright") {
      break;
    }
    let end_cs = (name_end + 1..end)
      .rev()
      .find(|&k| toks[k] == T_CS!("\\end"))
      .unwrap_or(end);
    toks = toks[name_end + 1..end_cs].to_vec();
  }
  Tokens::new(toks)
}

/// An alignment environment (`center`, `flushleft`, `flushright`) that opens after the names and ends the block is
/// layout too: it starts a line of its own, its content the lines under the names — `\author{Ann Able and Bob Baker
/// \begin{center}Department of Physics and Astronomy\end{center}}` prints the department centred under both names, read
/// as `… Bob Baker \\ Department …`, every name's of the row (R12; 2609.01563, 2609.17357, 2609.33831 — it had been kept
/// whole inside the last personname). One opening the block is [`unwrap_alignment_environment`]'s.
pub(in crate::base_utilities) fn trailing_alignment_environment_as_lines(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  let end = toks.len() - toks.iter().rev().take_while(|t| **t == T_SPACE!()).count();
  let mut depth = 0i32;
  for start in 0..end {
    match toks[start].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth != 0
      || toks[start] != T_CS!("\\begin")
      || toks[..start].iter().all(|t| *t == T_SPACE!())
    {
      continue;
    }
    let stream: VecDeque<Token> = toks[start + 1..end].iter().copied().collect();
    let Some(span) = environment_closes_ahead(&stream) else {
      continue;
    };
    let name_end = (start + 2..end)
      .find(|&k| toks[k].get_catcode() == Catcode::END)
      .unwrap_or(start + 1);
    let name: String = toks[start + 2..name_end]
      .iter()
      .map(|t| t.to_string())
      .collect();
    if start + 1 + span != end || !matches!(name.as_str(), "center" | "flushleft" | "flushright") {
      continue;
    }
    let end_cs = (name_end + 1..end)
      .rev()
      .find(|&k| toks[k] == T_CS!("\\end"))
      .unwrap_or(end);
    let mut out = toks[..start].to_vec();
    out.push(T_CS!("\\\\"));
    out.extend_from_slice(&toks[name_end + 1..end_cs]);
    return Tokens::new(out);
  }
  Tokens::new(toks)
}

/// An author block's parameterless macro whose expansion closes the title page's tabular and opens another — the ACM
/// SIG template's `\def\sharedaffiliation{\end{tabular}\begin{tabular}{c}}`, set inside the class's own author tabular
/// (sig-alternate.cls:283's `\alignauthor` juggles the same way) — is the line break it prints between the names
/// above and the affiliation below: digested, it closed a tabular no author block opens and left one open, whose
/// lines (the shared affiliation) were lost (1704.03168, 1301.5612, 1401.6981, 0712.1016). The author separators
/// (`\alignauthor`) stay for the splitter.
pub(in crate::base_utilities) fn tabular_juggles_as_line_breaks(tokens: Tokens) -> Tokens {
  let separators = author_and_splits();
  let juggles = |t: &Token| {
    if t.get_catcode() != Catcode::CS
      || separators
        .iter()
        .any(|s| matches!(s, SplitDelim::Token(d) if d == t))
    {
      return false;
    }
    let Ok(Some(defn)) = lookup_definition(t) else {
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
    let body: Vec<Token> = body
      .unlist_ref()
      .iter()
      .filter(|t| t.get_catcode() != Catcode::SPACE)
      .copied()
      .collect();
    body.first() == Some(&T_CS!("\\end"))
      && body
        .get(1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && body[2..]
        .iter()
        .take_while(|t| t.get_catcode() != Catcode::END)
        .map(|t| t.to_string())
        .collect::<String>()
        == "tabular"
  };
  let toks = tokens.unlist();
  if !toks.iter().any(juggles) {
    return Tokens::new(toks);
  }
  Tokens::new(
    toks
      .into_iter()
      .map(|t| if juggles(&t) { T_CS!("\\\\") } else { t })
      .collect(),
  )
}

/// The ACM SIG classes' author columns (DIVERGENCES #445), read through their own commands: the brace group that
/// follows `\alignauthor` and is the whole column — sigchi.cls writes each column as one, `\alignauthor{Name\\ \affaddr{…}\\ \email{…}}` — is
/// its content, so the column's `\\` lines split (1906.01122); and an `\affaddr{…}` is its content, so a mark that
/// leads it (`\affaddr{\textsuperscript{1} Univ…}`, 1608.06253) leads the affiliation line, as Perl's OmniBus reads
/// `\affaddr` as `\address` (OmniBus.cls.ltxml:92). Only at depth 0 and only after these two commands, which no
/// TeX Live file defines: a brace group anywhere else (`\author{{Smith, Jr., John}}`) keeps its braces.
pub(in crate::base_utilities) fn unbrace_acm_author_columns(tokens: Tokens) -> Tokens {
  let toks = tokens.unlist();
  if !toks
    .iter()
    .any(|t| *t == T_CS!("\\alignauthor") || *t == T_CS!("\\affaddr"))
  {
    return Tokens::new(toks);
  }
  let mut out = Vec::with_capacity(toks.len());
  let mut depth = 0usize;
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    i += 1;
    let column = t == T_CS!("\\alignauthor");
    if depth == 0 && (column || t == T_CS!("\\affaddr")) {
      if column {
        out.push(t);
      }
      let mut k = i;
      while k < toks.len() && toks[k].get_catcode() == Catcode::SPACE {
        k += 1;
      }
      if k < toks.len() && toks[k].get_catcode() == Catcode::BEGIN {
        let mut level = 0usize;
        let mut close = None;
        for (n, u) in toks[k..].iter().enumerate() {
          match u.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => {
              level -= 1;
              if level == 0 {
                close = Some(k + n);
                break;
              }
            },
            _ => {},
          }
        }
        // A column's group is its whole content only when nothing but spaces and `\\` follow it before the next
        // author: `\alignauthor {\large Ann}\\ \affaddr{…}` is a name's font group, kept.
        let whole = |close: usize| {
          let mut n = close + 1;
          while n < toks.len()
            && (toks[n].get_catcode() == Catcode::SPACE || toks[n] == T_CS!("\\\\"))
          {
            n += 1;
          }
          n == toks.len()
            || [
              T_CS!("\\alignauthor"),
              T_CS!("\\and"),
              T_CS!("\\And"),
              T_CS!("\\AND"),
            ]
            .contains(&toks[n])
        };
        if let Some(close) = close
          && (!column || whole(close))
        {
          // The column's own `\affaddr`s are read the same way.
          out.extend(unbrace_acm_author_columns(Tokens::new(toks[k + 1..close].to_vec())).unlist());
          i = close + 1;
          continue;
        }
      }
      if !column {
        out.push(t);
      }
      continue;
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    out.push(t);
  }
  Tokens::new(out)
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
// line is known to be author names — via split_author_line — never across
// address text. (Compare author_affil_splits(): "NO comma in affiliations!!!".)
pub(in crate::base_utilities) fn author_group_splits() -> Vec<SplitDelim> {
  let mut splits = author_and_splits();
  splits.extend([T_CS!("\\quad").into(), T_CS!("\\qquad").into()]);
  splits
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
