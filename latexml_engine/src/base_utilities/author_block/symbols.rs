//! Footnote-symbol marks and the superscript rewrites (symbols, ordinals, in-word).

use super::*;

/// The footnote-SYMBOL characters (`\fnsymbol`-style) and control sequences that,
/// as a superscript on an author, mark an equal-contribution / corresponding /
/// note relation — NEVER an affiliation (which is numbered). Kept deliberately to
/// pure symbols so a numeric (`1`) or lettered (`a`) affiliation mark is never
/// misread as a note. Mirrors the note-vs-affiliation split already documented in
/// `starts_with_affiliation_mark`. The ptephy binding normalizes the same symbols (latexml_contrib ptephy_cls.rs
/// `footnote_symbol`): keep the two lists in step.
pub(in crate::base_utilities) fn is_footnote_symbol_operand(sym: &[Token]) -> bool {
  use FOOTNOTE_SYMBOL_CS as SYMBOL_CS;
  const SYMBOL_CHARS: &[&str] = &[
    "*", "\u{2217}", "\u{2020}", "\u{2021}", "\u{A7}", "\u{B6}", "\u{22C6}", "\u{2605}",
    "\u{2022}", "\u{25E6}", "\u{2666}",
  ];
  let mut saw_symbol = false;
  for t in sym {
    match t.get_catcode() {
      Catcode::SPACE => continue,
      Catcode::CS => {
        if !t.with_str(|s| SYMBOL_CS.contains(&s)) && !is_symbol_command(t, 1) {
          return false;
        }
        saw_symbol = true;
      },
      Catcode::OTHER | Catcode::LETTER => {
        if !t.with_str(|s| SYMBOL_CHARS.contains(&s)) {
          return false;
        }
        saw_symbol = true;
      },
      _ => return false,
    }
  }
  saw_symbol
}

/// The footnote-symbol commands ([`is_footnote_symbol_operand`]).
const FOOTNOTE_SYMBOL_CS: &[&str] = &[
  "\\ast",
  "\\star",
  "\\dagger",
  "\\ddagger",
  "\\S",
  "\\P",
  "\\bullet",
  "\\diamond",
  "\\circ",
  "\\sharp",
  "\\|",
  "\\#",
];

/// Whether `t` is a footnote-symbol command beyond [`FOOTNOTE_SYMBOL_CS`]: one LaTeX's own footnote marks print — a
/// command in the body of `\@fnsymbol` (latex.ltx `\def\@fnsymbol#1{\ifcase#1\or \TextOrMath\textasteriskcentered *\or
/// \TextOrMath \textdagger \dagger\or … \TextOrMath \textsection \mathsection\or …}`, as the document's class may
/// redefine it) that prints a symbol, a math symbol or a primitive printing one (`\mathsection`, `\textparagraph`),
/// not the conditionals around them; or a macro without parameters whose body is only such commands
/// (`\newcommand{\eq}{\mathsection}`), `depth` macros deep. A superscript holding only such commands is a symbol mark,
/// as one holding `\dagger` is (main-session decision 2026-10-20, batch 77p r6: 2601.02680's `$^\mathsection$` and
/// `$^{\mathsection}$`, beside its `$^{\dagger}$`). Letters and digits are none, nor is a text-mode `\dag`/`\ddag`,
/// the affiliation mark it is today ([`is_symbol_mark`]); and a math symbol outside `\@fnsymbol` (a suit, `\clubsuit`)
/// stays the affiliation mark it labels its lines with (2601.05633's `$^{\clubsuit \heartsuit}$`).
fn is_symbol_command(t: &Token, depth: usize) -> bool {
  if t.get_catcode() != Catcode::CS || *t == T_CS!("\\dag") || *t == T_CS!("\\ddag") {
    return false;
  }
  // (a command that prints a symbol: a math symbol, or a primitive printing a character that is no letter or digit)
  let prints_a_symbol = |u: &Token| match lookup_definition_stored(u) {
    Ok(Some(Stored::MathPrimitive(math))) => math.nargs.unwrap_or(0) == 0,
    Ok(Some(Stored::Primitive(primitive))) => matches!(
      &primitive.replacement,
      Some(PrimitiveBody::String(text))
        if with(*text, |text| {
          !text.is_empty() && !text.chars().any(|c| c.is_alphanumeric() || c.is_whitespace())
        })
    ),
    _ => false,
  };
  let in_fnsymbol = |u: &Token| {
    matches!(lookup_definition_stored(&T_CS!("\\@fnsymbol")),
      Ok(Some(Stored::Expandable(fnsymbol)))
        if matches!(fnsymbol.get_expansion(), Some(ExpansionBody::Tokens(body))
          if body.unlist_ref().contains(u)))
  };
  if in_fnsymbol(t) && prints_a_symbol(t) {
    return true;
  }
  depth > 0
    && matches!(lookup_definition_stored(t), Ok(Some(Stored::Expandable(macro_def)))
    if macro_def
      .get_parameters()
      .is_none_or(|p| p.get_parameters().is_empty())
      && matches!(macro_def.get_expansion(), Some(ExpansionBody::Tokens(body))
        if body.unlist_ref().iter().any(|u| *u != T_SPACE!())
          && body.unlist_ref().iter().all(|u| {
            *u == T_SPACE!()
              || u.with_str(|s| FOOTNOTE_SYMBOL_CS.contains(&s))
              || is_symbol_command(u, depth - 1)
          })))
}

/// Is a mark's operand a footnote symbol (`*`, `\dagger`, `\dag`), an equal-contribution or correspondence legend's
/// mark rather than an affiliation number?
pub(in crate::base_utilities) fn is_symbol_mark(operand: &[Token]) -> bool {
  is_footnote_symbol_operand(operand)
    || matches!(
      operand.iter().filter(|t| **t != T_SPACE!()).collect::<Vec<_>>()[..],
      [t] if *t == T_CS!("\\dag") || *t == T_CS!("\\ddag")
    )
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

/// A legend-shaped `line` that is an affiliation its symbol labels: led by a symbol mark [`keepsup`] made
/// (`$^{\diamond}$ National University of Singapore`), the symbol one an author of the block carries (`carried`, the
/// authors' [`symbol_marks`]), and the rest reading as an affiliation ([`reads_as_affiliation`]): the line with that mark
/// written back as the superscript it was, which `\lx@affiliation@withsup` labels the affiliation by, so it goes to the
/// authors carrying the symbol as a numbered one goes to the authors carrying the number (2601.05633's ⋄ line under its ♡
/// line, to Yang Zhang, Zhiyong Huang and Tat-Seng Chua, not the last author; batch 77p r6). A legend
/// (`$^{*}$ Equal contribution`) or a symbol no author carries stays as it is.
pub(in crate::base_utilities) fn symbol_labelled_affiliation(
  line: &Tokens,
  carried: &[String],
) -> Option<Tokens> {
  let v = line.unlist_ref();
  let lead = v.len() - after_leading_markup(&v[leading_spacing_end(v)..]).len();
  if v.get(lead) != Some(&T_CS!("\\lx@frontmatter@keepsup")) {
    return None;
  }
  let shown_end = skip_group(v, lead + 1);
  if v.get(shown_end) != Some(&T_CS!("\\lx@frontmatter@symbolmark")) {
    return None;
  }
  let end = skip_group(v, shown_end + 1);
  let mark = symbol_marks(&Tokens::new(v[shown_end..end].to_vec()));
  let rest = Tokens::new(v[end..].to_vec());
  if !mark.first().is_some_and(|mark| carried.contains(mark)) || !reads_as_affiliation(&rest) {
    return None;
  }
  let mut out = v[..lead].to_vec();
  out.push(T_CS!("\\textsuperscript"));
  out.extend_from_slice(&v[lead + 1..shown_end]);
  out.extend(rest.unlist());
  Some(Tokens::new(out))
}

/// Whether `line` is an institution no mark leads ([`reads_as_affiliation`]): not led, past its spacing and font
/// commands, by a footnote symbol or the visible superscript [`rewrite_symbol_superscripts`] makes one — a symbol-led line
/// is its symbol's carriers', or a legend (2604.02578's `\textsuperscript{*}Department of Computer Science` under names
/// marked `*†`, `†‡`, `*†‡`; 2604.03394's `\small ${\dag}$ Department …`) — nor by math (`$(a)$Univ A`, a mark no author
/// requests); no note naming the corresponding author; and an institution by a word naming one, not by a role or contact
/// word alone (`ROLE_WORDS`: "This research was supported by …", "for the ATLAS Collaboration", "Corresponding author:
/// Professor …", the rev77t review's r4, r6, r8).
pub(in crate::base_utilities) fn unmarked_institution_line(line: &Tokens) -> bool {
  /// The words [`NON_NAME_WORDS`] holds for a role or a contact, not an institution.
  const ROLE_WORDS: &[&str] = &[
    "research",
    "professor",
    "collaboration",
    "member",
    "team",
    "email",
    "e-mail",
  ];
  let v = line.unlist_ref();
  let lead = after_leading_markup(&v[leading_spacing_end(v)..]);
  let text = visible_name_text(v);
  reads_as_affiliation(line)
    && lead.first() != Some(&T_CS!("\\lx@frontmatter@keepsup"))
    && lead.first().map(Token::get_catcode) != Some(Catcode::MATH)
    && !starts_with_footnote_symbol(&Tokens::new(lead.to_vec()))
    && !names_correspondence(&text)
    && text.split_whitespace().any(|word| {
      non_name_word(word) && !word_forms(word).any(|form| ROLE_WORDS.contains(&form.as_str()))
    })
}

/// A line led by a mark in parentheses set in math, `$(a)$Department of Physics, …`, whose letter or number an author of
/// the block requests as its superscript mark (`requested`: `Etsuko Itou$^a$`): the line with that mark written as the
/// superscript `\lx@affiliation@withsup` labels the affiliation by, so it goes to the authors carrying it as a `$^a$`-led
/// line does (1011.0516, PoS.cls, five such lines welded into the last name). A parenthesized lead no author requests
/// stays the line's text.
pub(in crate::base_utilities) fn parenthesized_mark_line(
  line: &Tokens,
  requested: &[String],
) -> Option<Tokens> {
  let v = line.unlist_ref();
  let lead = leading_spacing_end(v);
  let rest = &v[lead..];
  let close = rest
    .iter()
    .skip(1)
    .position(|t| t.get_catcode() == Catcode::MATH)?
    + 1;
  if rest.first().map(Token::get_catcode) != Some(Catcode::MATH) {
    return None;
  }
  let inner: Vec<Token> = rest[1..close]
    .iter()
    .copied()
    .filter(|t| *t != T_SPACE!())
    .collect();
  let [open, label @ .., shut] = inner.as_slice() else {
    return None;
  };
  if *open != T_OTHER!("(")
    || *shut != T_OTHER!(")")
    || label.is_empty()
    || !label.iter().all(|t| {
      matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)
        && t.with_str(|s| s.chars().all(char::is_alphanumeric))
    })
  {
    return None;
  }
  let mark = Tokens::new(label.to_vec()).to_string();
  if !requested.contains(&mark) {
    return None;
  }
  let mut out = v[..lead].to_vec();
  out.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
  out.extend_from_slice(label);
  out.push(T_END!());
  out.extend_from_slice(&rest[close + 1..]);
  Some(Tokens::new(out))
}

/// Whether `line` is a legend: a footnote symbol ([`starts_with_footnote_symbol`]), also as the visible superscript
/// [`rewrite_symbol_superscripts`] makes it (`\textsuperscript{*}\textit{Equal contribution.}`, 2509.20748), and text
/// after it.
pub(in crate::base_utilities) fn is_legend_line(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let v = after_leading_markup(&v[leading_spacing_end(v)..]);
  (starts_with_footnote_symbol(&Tokens::new(v.to_vec()))
    || v.first() == Some(&T_CS!("\\lx@frontmatter@keepsup")))
    && has_name_letter(v)
}

/// Rewrite footnote-SYMBOL author superscripts — `$^{*}$`, `${}^{\dagger}$`,
/// `\textsuperscript{\ddagger}`, a bare `^{*}` — onto the visible
/// `\lx@frontmatter@keepsup` sentinel, BEFORE author-block branch selection.
///
/// Two effects, both wanted (OXIDIZED_DESIGN #52; witness arXiv:2506.06941, where
/// Iman Mirzadeh's literal `$^{*}$` was silently dropped):
///   * the mark no longer counts as an affiliation superscript, so a block whose
///     ONLY superscript is such a note-mark takes the clean no-marker author
///     branch instead of the affiliation-linking one;
///   * the symbol renders as a real superscript instead of being consumed into an
///     `affiliation:*` label that matches no affiliation and is discarded.
///
/// A number or letter glued to such a symbol (`$^{1*}$`, `\textsuperscript{a\dagger}`) is the affiliation mark it is
/// and the symbol after it (`split_glued_symbol`): the mark requests affiliation 1, the symbol is shown, as for
/// `$^{1,*}$` (else the one request `affiliation:1*` matched no affiliation, and affiliation 1 went to no one; authblk's
/// `\author{Ann$^{1*}$}`, 77n r3; the neurips fixture's Wenqi Shao, 2507.06670). The symbol is shown as written, a text
/// `*` as for a lone `$^{*}$`, where the unsplit `$^{1,*}$` keeps its math `∗` (2210.14772, 2508.09827).
///
/// NUMERIC/lettered affiliation marks (`$^{1}$`, `\textsuperscript{a}`) are left
/// untouched, so affiliation linking is unaffected. Only the specific
/// superscript-mark token shapes are matched; a superscript inside real math
/// (`$x^2$`) is not (its base is not empty), so math content is preserved.
pub(in crate::base_utilities) fn rewrite_symbol_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let mut i = 0;
  // Read a superscript operand at `src[k]`: a braced `{...}` group, or a single
  // token. Returns (operand tokens, index past the operand) or None.
  let read_operand = |k: usize| -> Option<(Vec<Token>, usize)> {
    if k >= n {
      return None;
    }
    if src[k] == T_BEGIN!() {
      let mut depth = 1usize;
      let mut m = k + 1;
      let mut inner = Vec::new();
      while m < n && depth > 0 {
        if src[m] == T_BEGIN!() {
          depth += 1;
        } else if src[m] == T_END!() {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        inner.push(src[m]);
        m += 1;
      }
      if depth == 0 {
        Some((inner, m + 1))
      } else {
        None
      }
    } else {
      Some((vec![src[k]], k + 1))
    }
  };
  while i < n {
    let t = src[i];
    // `\textsuperscript{sym}` — always braced in practice.
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((sym, next)) = read_operand(i + 1)
      && is_footnote_symbol_operand(&sym)
    {
      out.extend(keepsup(sym));
      i = next;
      continue;
    }
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((operand, next)) = read_operand(i + 1)
      && let Some((mark, sym)) = split_glued_symbol(&operand)
    {
      out.extend([t, T_BEGIN!()]);
      out.extend(mark);
      out.push(T_END!());
      out.extend(keepsup(sym));
      i = next;
      continue;
    }
    // `$ [ {} ] ^ sym $` — a math span whose whole content is a superscript on an
    // empty base. Rewrite the entire span (delimiters included) to a text sup.
    if t == T_MATH!()
      && let Some(close) = (i + 1..n).find(|&m| src[m] == T_MATH!())
    {
      // optional empty base `{}` before the superscript
      let p = if i + 2 < close && src[i + 1] == T_BEGIN!() && src[i + 2] == T_END!() {
        i + 3
      } else {
        i + 1
      };
      if p < close
        && src[p] == T_SUPER!()
        && let Some((sym, next)) = read_operand(p + 1)
        && next == close
        && is_footnote_symbol_operand(&sym)
      {
        out.extend(keepsup(sym));
        i = close + 1;
        continue;
      }
      if p < close
        && src[p] == T_SUPER!()
        && let Some((operand, next)) = read_operand(p + 1)
        && next == close
        && let Some((mark, sym)) = split_glued_symbol(&operand)
      {
        out.extend_from_slice(&src[i..=p]);
        out.push(T_BEGIN!());
        out.extend(mark);
        out.extend([T_END!(), T_MATH!()]);
        out.extend(keepsup(sym));
        i = close + 1;
        continue;
      }
    }
    // Only the math-delimited (`$…$`) and `\textsuperscript{…}` spellings are
    // rewritten — the correct ways to write a text superscript. A bare `^` is NOT
    // matched here: outside math it is invalid LaTeX, and a `^` reached mid-scan is
    // the superscript operator of real inline math on a non-empty base (`$a^{*}$`),
    // which must be left intact.
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// A mark's operand of a number or letter glued to a footnote symbol after it (`1*`, `a\dagger`, `2\ast`; the symbol
/// braced, `1{\dagger}`, 2605.01693): the mark and the symbol ([`rewrite_symbol_superscripts`]).
pub(in crate::base_utilities) fn split_glued_symbol(
  operand: &[Token],
) -> Option<(Vec<Token>, Vec<Token>)> {
  let lead = operand
    .iter()
    .take_while(|t| {
      t.get_catcode() == Catcode::LETTER
        || (t.get_catcode() == Catcode::OTHER
          && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit())))
    })
    .count();
  let sym = &operand[lead..];
  // (a group holding the symbol alone is the symbol)
  let unbraced = match sym {
    [open, inner @ .., close]
      if open.get_catcode() == Catcode::BEGIN
        && close.get_catcode() == Catcode::END
        && skip_group(sym, 0) == sym.len() =>
    {
      inner
    },
    _ => sym,
  };
  (lead > 0 && !unbraced.is_empty() && is_symbol_mark(unbraced))
    .then(|| (operand[..lead].to_vec(), sym.to_vec()))
}

/// `\lx@frontmatter@keepsup{<sym>}` as a token list: a superscript shown as it is, requesting nothing.
fn visible_sup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = vec![T_CS!("\\lx@frontmatter@keepsup"), T_BEGIN!()];
  v.extend(sym);
  v.push(T_END!());
  v
}

/// The footnote-symbol marks [`keepsup`] wrote into `tokens` (the `\lx@frontmatter@symbolmark` operands, as text), at
/// any depth: `Ann Able$^{\dagger}$` and `$^{\dagger}$Univ A` both hold `\dagger`.
pub(in crate::base_utilities) fn symbol_marks(tokens: &Tokens) -> Vec<String> {
  let v = tokens.unlist_ref();
  let mut marks = Vec::new();
  for (i, t) in v.iter().enumerate() {
    if *t == T_CS!("\\lx@frontmatter@symbolmark") && v.get(i + 1) == Some(&T_BEGIN!()) {
      let mut depth = 0usize;
      let mut operand: Vec<Token> = Vec::new();
      for &u in &v[i + 1..] {
        match u.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => depth -= 1,
          _ => {},
        }
        if depth == 0 {
          break;
        }
        if !(depth == 1 && u.get_catcode() == Catcode::BEGIN) {
          operand.push(u);
        }
      }
      let mark = Tokens::new(operand).to_string();
      let mark = mark.trim();
      if !mark.is_empty() {
        marks.push(mark.to_string());
      }
    }
  }
  marks
}

/// A footnote-symbol author mark: shown (`visible_sup`), and requesting the affiliation of its symbol
/// (`\lx@frontmatter@symbolmark`).
pub(in crate::base_utilities) fn keepsup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = visible_sup(sym.clone());
  v.extend([T_CS!("\\lx@frontmatter@symbolmark"), T_BEGIN!()]);
  v.extend(sym);
  v.push(T_END!());
  v
}

/// Rewrite an ordinal suffix superscript after a numeral (`5\textsuperscript{th}`, `21$^{st}$`) onto a visible
/// superscript: it is the number's, not an affiliation mark, so a marked list is not split at it nor labelled by it
/// (repro sectioning-frontmatter/affiliation_line_superscript_ordinal_is_not_a_mark; iopart's
/// `\address{$^1$Univ A, 5 \textsuperscript{th} floor}`, 2609.01831).
pub(in crate::base_utilities) fn rewrite_ordinal_superscripts(tokens: Tokens) -> Tokens {
  // The suffix as written, a font wrapper around it aside (`$^{\rm th}$`, `\textsuperscript{\mathrm{nd}}`).
  const FONT_WRAPPERS: &[&str] = &[
    "\\rm", "\\mathrm", "\\textrm", "\\text", "\\mbox", "\\textup", "\\mathup", "\\it", "\\mathit",
    "\\textit", "\\bf", "\\mathbf", "\\textbf", "\\sf", "\\mathsf", "\\textsf",
  ];
  let is_ordinal = |operand: &[Token]| {
    let text: String = operand
      .iter()
      .filter(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
      .map(|t| t.with_str(str::to_lowercase))
      .collect();
    operand.iter().all(|t| match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER | Catcode::SPACE | Catcode::BEGIN | Catcode::END => true,
      Catcode::CS => FONT_WRAPPERS.iter().any(|w| *t == T_CS!(*w)),
      _ => false,
    }) && matches!(text.as_str(), "st" | "nd" | "rd" | "th")
  };
  let is_digit = |t: &Token| {
    t.get_catcode() == Catcode::OTHER
      && t.with_str(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
  };
  let after_numeral = |out: &[Token]| {
    out
      .iter()
      .rev()
      .find(|t| **t != T_SPACE!())
      .is_some_and(is_digit)
  };
  // The suffix is shown in text: a math-only wrapper takes its text form (`$^{\mathrm{th}}$`).
  let text_mode = |operand: Vec<Token>| -> Vec<Token> {
    operand
      .into_iter()
      .map(|t| {
        if t == T_CS!("\\mathrm") {
          T_CS!("\\textrm")
        } else if t == T_CS!("\\mathup") {
          T_CS!("\\textup")
        } else if t == T_CS!("\\mathit") {
          T_CS!("\\textit")
        } else if t == T_CS!("\\mathbf") {
          T_CS!("\\textbf")
        } else if t == T_CS!("\\mathsf") {
          T_CS!("\\textsf")
        } else {
          t
        }
      })
      .collect()
  };
  let src = tokens.unlist();
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    if t == T_CS!("\\textsuperscript")
      && let Some((operand, next)) = sup_operand_at(&src, i + 1)
      && is_ordinal(&operand)
      && after_numeral(&out)
    {
      out.extend(visible_sup(text_mode(operand)));
      i = next;
      continue;
    }
    if t == T_MATH!() {
      // `$^{th}$` after a numeral, or `$5^{th}$` with the numeral inside: the digits stay, the suffix is shown.
      let digits_end = (i + 1..src.len())
        .find(|&k| !is_digit(&src[k]))
        .unwrap_or(src.len());
      if src.get(digits_end) == Some(&T_SUPER!())
        && let Some((operand, next)) = sup_operand_at(&src, digits_end + 1)
        && src.get(next) == Some(&T_MATH!())
        && is_ordinal(&operand)
        && (digits_end > i + 1 || after_numeral(&out))
      {
        out.extend_from_slice(&src[i + 1..digits_end]);
        out.extend(visible_sup(text_mode(operand)));
        i = next + 1;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// Rewrite a superscript inside a word, a letter on each side of it (`Craig M$^{\mathrm{c}}$Rae`, 2505.09041;
/// `M\textsuperscript{c}Donald`; `I$^2$R`, `NLP$^2$C`), onto a visible superscript: it is the name's spelling, not an
/// affiliation mark, which follows a name — read as one, its letter was a request and left the name ("Craig MRae", the
/// "c" shown after it). Not a mark list (`$^{1,3}$`: a list of marks is never spelling) nor a mark glued to the word
/// "and" after it (`Hiroshi Ito$^{1,3}$and Tetsuya Ogata`, 2605.00471; `Bipasha Kundu$^{1}$and Cristian Linte`,
/// 2604.27101): those are the name's marks, which kept as its text lost their affiliations (77n r3 review).
pub(in crate::base_utilities) fn rewrite_in_word_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let letter = |t: Option<&Token>| t.is_some_and(|t| t.get_catcode() == Catcode::LETTER);
  // (the operand a mark's: a list of marks, or the word after it "and", the separator before the next name; `Andrew`
  // is a word, 77n r4 review probe a8. Only "and" itself: `Ann Able$^{1}$And Bob Baker$^{2}$`, `AND`, stays one name,
  // which `flag_merged_creators` reports — read as a mark, its "And" merged the names silently, 77n r5 review probes
  // a1, a6)
  let marks = |operand: &[Token], after: usize| {
    operand.contains(&T_OTHER!(","))
      || (src.get(after..after + 3) == Some(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")][..])
        && !letter(src.get(after + 3)))
  };
  // (the letters before it a word, not a length's unit: `\kern2pt$^{1}$Univ A` is a spacing, then a mark; 2407.15815)
  let after_a_word = |out: &[Token]| {
    let word = out
      .iter()
      .rev()
      .take_while(|t| t.get_catcode() == Catcode::LETTER)
      .count();
    word > 0
      && out[..out.len() - word].last().is_none_or(|t| {
        !(t.get_catcode() == Catcode::OTHER
          && t.with_str(|s| {
            s.chars()
              .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
          }))
      })
  };
  let mut out: Vec<Token> = Vec::with_capacity(src.len());
  let mut i = 0;
  while i < src.len() {
    let t = src[i];
    if after_a_word(&out) {
      if t == T_CS!("\\textsuperscript")
        && let Some((operand, next)) = sup_operand_at(&src, i + 1)
        && letter(src.get(next))
        && !marks(&operand, next)
      {
        out.extend(visible_sup(operand));
        i = next;
        continue;
      }
      if t == T_MATH!()
        && src.get(i + 1) == Some(&T_SUPER!())
        && let Some((operand, next)) = sup_operand_at(&src, i + 2)
        && src.get(next) == Some(&T_MATH!())
        && letter(src.get(next + 1))
        && !marks(&operand, next + 1)
      {
        let mut math = vec![T_MATH!()];
        math.extend(operand);
        math.push(T_MATH!());
        out.extend(visible_sup(math));
        i = next + 1;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}
