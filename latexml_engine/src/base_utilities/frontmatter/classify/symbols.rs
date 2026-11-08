//! The text rules on printed ink and the token primitives of `author_block/symbols.rs`, moved unchanged (MV1,
//! user ruling 2026-11-08 "knowledge in, driver out").

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

/// A mark's operand of a number or letter glued to a footnote symbol, after it or before it (`1*`, `a\dagger`, `2\ast`;
/// `*1`, `\dagger 2`, 2505.22662's `\textsuperscript{*1}`; the symbol braced, `1{\dagger}`, 2605.01693): the mark and
/// the symbol ([`rewrite_symbol_superscripts`], which writes them in the order they print).
pub(in crate::base_utilities) fn split_glued_symbol(
  operand: &[Token],
) -> Option<(Vec<Token>, Vec<Token>)> {
  let is_mark = |t: &Token| {
    t.get_catcode() == Catcode::LETTER
      || (t.get_catcode() == Catcode::OTHER
        && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit())))
  };
  let lead = operand.iter().take_while(|t| is_mark(t)).count();
  let (mark, sym) = if lead > 0 {
    (&operand[..lead], &operand[lead..])
  } else {
    let tail = operand.iter().rev().take_while(|t| is_mark(t)).count();
    let at = operand.len() - tail;
    (&operand[at..], &operand[..at])
  };
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
  (!mark.is_empty() && !unbraced.is_empty() && is_symbol_mark(unbraced))
    .then(|| (mark.to_vec(), sym.to_vec()))
}

/// A part of a mark's operand as it prints ([`glued_marks`]).
pub(in crate::base_utilities) enum GluedPart {
  /// a run of the marks it requests, comma-parted as written; one after a symbol keeps the comma it prints (`,2,3` of
  /// `$^{1*,2,3}$`, which requests 2 and 3)
  Marks(Vec<Token>),
  /// a footnote symbol
  Symbol(Vec<Token>),
}

/// [`split_glued_symbol`] over an operand's comma list, a piece of which glues a symbol to its number or letter
/// (`$^{1,2*}$`, `$^{*1,2}$`, whose printed star was lost, the label read by its number alone; `$^{1*,2,3}$`, 2609.14365);
/// a symbol alone beside them is such a piece's too (`1*,\dagger`): the marks and the symbols, each in the place it prints.
/// `None` when no piece glues one: `$^{1,*}$`, read as it is, keeps its math `∗`.
pub(in crate::base_utilities) fn glued_marks(operand: &[Token]) -> Option<Vec<GluedPart>> {
  let mut parts: Vec<GluedPart> = Vec::new();
  let (mut any_glued, mut any_mark) = (false, false);
  for piece in operand.split(|t| *t == T_OTHER!(",")) {
    let piece: Vec<Token> = piece
      .iter()
      .filter(|t| **t != T_SPACE!())
      .copied()
      .collect();
    let (mark, symbol) = if let Some((mark, symbol)) = split_glued_symbol(&piece) {
      any_glued = true;
      (mark, Some(symbol))
    } else if is_symbol_mark(&piece) {
      (Vec::new(), Some(piece))
    } else {
      (piece, None)
    };
    // (a symbol is printed before its piece's mark only where it is written so and nothing stands before it, `*1`)
    let symbol_first = parts.is_empty()
      && !mark.is_empty()
      && symbol
        .as_ref()
        .is_some_and(|symbol| operand.starts_with(symbol));
    if symbol_first && let Some(symbol) = symbol.clone() {
      parts.push(GluedPart::Symbol(symbol));
    }
    if !mark.is_empty() {
      any_mark = true;
      match parts.last_mut() {
        Some(GluedPart::Marks(marks)) => {
          marks.push(T_OTHER!(","));
          marks.extend(mark);
        },
        // (after a symbol of an earlier piece the comma between them prints, `1∗,2`; `*1`'s own symbol has none)
        Some(GluedPart::Symbol(_)) if !symbol_first => {
          parts.push(GluedPart::Marks(
            std::iter::once(T_OTHER!(",")).chain(mark).collect(),
          ));
        },
        _ => parts.push(GluedPart::Marks(mark)),
      }
    }
    if !symbol_first && let Some(symbol) = symbol {
      parts.push(GluedPart::Symbol(symbol));
    }
  }
  (any_glued && any_mark).then_some(parts)
}

/// Is `operand` an ordinal suffix, `st`, `nd`, `rd` or `th` in any case, a font wrapper around it aside (`$^{\rm th}$`,
/// `\textsuperscript{\mathrm{nd}}`)? Shared by [`rewrite_ordinal_superscripts`] and `relocate::math_number_mark_at`
/// (`$3$rd floor` is no mark: review rev104c S1).
pub(in crate::base_utilities) fn is_ordinal_suffix(operand: &[Token]) -> bool {
  const FONT_WRAPPERS: &[&str] = &[
    "\\rm", "\\mathrm", "\\textrm", "\\text", "\\mbox", "\\textup", "\\mathup", "\\it", "\\mathit",
    "\\textit", "\\bf", "\\mathbf", "\\textbf", "\\sf", "\\mathsf", "\\textsf",
  ];
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
}
