//! Author-line separators: line parts, words, `and` splits, alignment and ACM column unwrapping.

use super::*;
use crate::{
  base_utilities::defs::frontmatter_api::inst_takes_back_space, frontmatter_stores::LAYOUT,
};

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
/// An ORCID icon link glued among them goes too ([`leading_orcid_icon_end`]): `Ashley Hai Tung Tan,$^{1}$\thanks{…}
/// \orcidAT⏎Christian Wolf,$^{1,2}$\orcidChrisW` prints each icon after its name's marks, and split after the comma
/// each was the next name's ORCID (2605.03577, mnras; review rev103b B1). Not when the names open with their icons
/// (`\orcidA~Ann Able,$^{1}$\orcidB~Bob Baker`: each icon the name after it, as 2603.26792's `\href{…}{icon}~{Name}`).
pub(in crate::base_utilities) fn marks_before_glued_commas(line: Tokens) -> Tokens {
  let inst = inst_takes_back_space();
  let inst_leads = || {
    let v = line.unlist_ref();
    v.get(leading_unprinted_end(v)) == Some(&T_CS!("\\inst"))
  };
  if marker_leads(&line) || (inst && inst_leads()) {
    return line;
  }
  let icons = {
    let v = line.unlist_ref();
    // (spacing alone before it: a parameterless icon macro reads as a declaration to `leading_unprinted_end`)
    leading_orcid_icon_end(&v[leading_spacing_end(v)..]) == 0
  };
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
      let end = glued_marks_end(&src[i + 1..], inst, icons);
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
///    ([`marks_before_glued_commas`]).
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

/// [`leading_inst_marks_end`] with the ORCID icon links glued among the marks ([`leading_orcid_icon_end`], when
/// `icons`): `$^{1}$\thanks{…}\orcidAT` (2605.03577). An icon glued to the comma with no mark before it is left where
/// it stands, which of the two names it is beside telling nothing.
fn glued_marks_end(tokens: &[Token], inst: bool, icons: bool) -> usize {
  let mut i = leading_inst_marks_end(tokens, inst);
  if i == 0 || !icons {
    return i;
  }
  loop {
    let icon = leading_orcid_icon_end(&tokens[i..]);
    if icon == 0 {
      return i;
    }
    i += icon;
    i += leading_inst_marks_end(&tokens[i..], inst);
  }
}

/// The index past an ORCID icon link that opens `tokens` (0 when none): an `\href` to an orcid.org record printing no
/// letter or digit as its tokens read ([`link_texts`]: an `\includegraphics`, a box, an icon macro — what
/// `\lx@author@href` makes the ORCID contact, OXIDIZED_DESIGN_DIVERGENCES #960), orcidlink's `\orcidlink{…}`, llncs's
/// `\orcidID{…}`, or a macro whose body is one such link (`\newcommand{\orcidAT}{\href{https://orcid.org/…}{\orcidicon}}`,
/// 2605.03577; one of required arguments, `\orcidicon{0000-…}`, 2501.02380) with its arguments — after a
/// parameterless one its empty group (`\orcidA{}`). A linked name (`\href{https://orcid.org/…}{Bob Baker}`) is none.
fn leading_orcid_icon_end(tokens: &[Token]) -> usize { orcid_icon_end_within(tokens, 0) }

/// [`leading_orcid_icon_end`], `depth` macros deep.
fn orcid_icon_end_within(tokens: &[Token], depth: usize) -> usize {
  let Some(&first) = tokens.first() else {
    return 0;
  };
  let group_after = |at: usize| {
    let open = at
      + tokens[at.min(tokens.len())..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count();
    tokens
      .get(open)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      .then(|| skip_group(tokens, open))
  };
  if first == T_CS!("\\orcidlink") || first == T_CS!("\\orcidID") {
    return group_after(1).unwrap_or(0);
  }
  if first == T_CS!("\\href") {
    let ((open, close), (ink_open, ink_close)) = link_arguments(tokens, 0);
    let url = Tokens::new(tokens[open..close].to_vec()).to_string();
    let (_, printed) = link_texts(tokens, 0);
    let groups = tokens
      .get(open)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && tokens
        .get(ink_open)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN);
    return if groups
      && url.to_ascii_lowercase().contains("orcid.org/")
      && !printed.chars().any(char::is_alphanumeric)
    {
      ink_close
    } else {
      0
    };
  }
  if depth > 2 || first.get_catcode() != Catcode::CS || lookup_conditional(&first).is_some() {
    return 0;
  }
  let Ok(Some(defn)) = lookup_definition(&first) else {
    return 0;
  };
  let params = defn
    .get_parameters()
    .map_or(0, |p| p.get_parameters().len());
  if !defn.is_expandable()
    || defn
      .get_parameters()
      .is_some_and(|p| p.get_parameters().iter().any(|param| param.optional))
  {
    return 0;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return 0;
  };
  let body = body.unlist_ref();
  let start = body.iter().take_while(|t| **t == T_SPACE!()).count();
  let end = body.len() - body.iter().rev().take_while(|t| **t == T_SPACE!()).count();
  if start >= end || start + orcid_icon_end_within(&body[start..end], depth + 1) != end {
    return 0;
  }
  let mut i = 1;
  for _ in 0..params {
    match group_after(i) {
      Some(next) => i = next,
      None => return 0,
    }
  }
  if params == 0
    && tokens
      .get(i)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    && tokens
      .get(i + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::END)
  {
    i += 2;
  }
  i
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
