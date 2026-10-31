//! The font in effect where the writer cuts an entry out of a call's content (review rev94b r3 B1): read from the
//! rehearsal's digested boxes, not from the tokens' shapes. The writer makes a cut entry from its own tokens alone, so
//! whatever font the content set before them — a switch at the head of the block (`\itshape`, `\small`,
//! `\color{red}`, `\bfseries`), a group or a font command's argument still open (`{\itshape …\\ …}`,
//! `\textit{…\\ …}`, `\begin{itshape}…\end{itshape}`) — is lost unless the entry carries it: each entry carries the
//! font in effect at its first token, against the call's own, and the writer digests the entry under it
//! (`writer::plan::entry_font_tokens`). A container other than a group (an `\underline`, an `\emph`, an `\href`) holding
//! the ink of two entries wraps each of them again (`writer::container_wrap`: each entry keeps its markup). A font the
//! boxes cannot tell, or a container the writer cannot wrap again exactly, never undoes the cut (review rev94b r4 B1:
//! attribution beats typography): the entry is cut as before fonts were read, with the switches opening the group it
//! sits in (`writer::span_tokens`' `own_font`).

use std::{
  collections::{BTreeMap, BTreeSet},
  rc::Rc,
};

use latexml_core::{document::FIELD_PROV_PROPERTY, token::field_prov_pos};

use super::*;

/// A digested content's text ink by position: each character box's font (math aside), its text, and the spans of the
/// whatsits holding it. A box is read at its own token's position, never at the cover of the expansion that made it
/// (`box_span`'s field-ledger cover: `\textbf{Lab C}`'s letters covered from `\textbf`, read as the font there).
#[derive(Default)]
pub(in crate::base_utilities) struct InkFonts {
  at:       BTreeMap<u16, InkBox>,
  /// each whatsit's font (math aside) at its own token's position: a line break's is the font in effect where it
  /// breaks, which the next entry starts in when nothing between them switches ([`entry_font`])
  whatsits: BTreeMap<u16, Rc<Font>>,
}

/// The containers a cut entry is wrapped in again (`writer::container_wrap`): the tokens opening each, outermost first
/// (`\emph{`, `\textcolor{red}{`), each closed by one end-group after the entry's tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct Wrap {
  pub open:   Vec<Token>,
  pub closes: usize,
}

/// A character box's font, its text, and the spans of the whatsits holding it ([`InkFonts`]).
type InkBox = (Rc<Font>, String, Vec<(u16, u16)>);

impl InkFonts {
  pub(in crate::base_utilities) fn of(d: &Digested) -> Self {
    let mut ink = InkFonts::default();
    ink.walk(d, &mut Vec::new());
    ink
  }

  fn walk(&mut self, d: &Digested, within: &mut Vec<(u16, u16)>) {
    match d.data() {
      DigestedData::TBox(b) => {
        let text = b
          .borrow()
          .get_string()
          .map(|s| s.into_owned())
          .unwrap_or_default();
        if text.chars().any(|c| !c.is_whitespace())
          && let Ok(Some(font)) = d.get_font()
          && font.get_mathstyle().is_none()
          && let Some(Stored::Int(prov)) = d.get_property(FIELD_PROV_PROPERTY).as_deref()
          && let Ok(prov) = u16::try_from(*prov)
        {
          self
            .at
            .entry(field_prov_pos(prov))
            .or_insert_with(|| (font, text, within.clone()));
        }
      },
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          self.walk(child, within);
        }
      },
      DigestedData::Whatsit(w) => {
        if let Ok(Some(font)) = d.get_font()
          && font.get_mathstyle().is_none()
          && let Some(Stored::Int(prov)) = d.get_property(FIELD_PROV_PROPERTY).as_deref()
          && let Ok(prov) = u16::try_from(*prov)
        {
          self.whatsits.entry(field_prov_pos(prov)).or_insert(font);
        }
        let span = deep_span(d);
        if let Some(span) = span {
          within.push(span);
        }
        let w = w.borrow();
        for arg in w.get_args().iter().flatten() {
          self.walk(arg, within);
        }
        if let Ok(Some(body)) = w.get_body() {
          self.walk(&body, within);
        }
        if span.is_some() {
          within.pop();
        }
      },
      _ => {},
    }
  }

  /// The whatsits each holding the ink of two or more of the entries `spans` (an `\underline{$^1$Univ A\\ $^2$Univ B}`),
  /// with the entries (indices into `spans`) whose ink each holds: the containers a cut entry is re-wrapped in
  /// (`writer::container_wrap`), outermost first.
  pub(in crate::base_utilities) fn holders(
    &self,
    spans: &[(u16, u16)],
  ) -> Vec<((u16, u16), BTreeSet<usize>)> {
    let mut held: BTreeMap<(u16, u16), BTreeSet<usize>> = BTreeMap::new();
    for (pos, (_, _, within)) in &self.at {
      let Some(entry) = spans.iter().position(|(s, e)| s <= pos && pos < e) else {
        continue;
      };
      for w in within {
        held.entry(*w).or_default().insert(entry);
      }
    }
    let mut holders: Vec<_> = held
      .into_iter()
      .filter(|(_, entries)| entries.len() > 1)
      .collect();
    holders.sort_by_key(|((s, e), _)| (*s, std::cmp::Reverse(*e)));
    holders
  }
}

/// Where the font is read from ([`entry_font`]): the content's digested ink, the font its call set it in, and the
/// position of the content's first token (what the backward scan stops at).
pub(in crate::base_utilities) struct FontScope {
  pub ink:   InkFonts,
  pub root:  Rc<Font>,
  pub start: usize,
}

/// The font changes in effect at `content[from]`, against the call's own font (`scope.root`) — `Font::default()`
/// when none —, read from the digested ink: the font of the first text character from there on with nothing between
/// but spaces, math shifts, scripts, braces it opens and what a mark, a line break or a separator prints (`$^2$Univ
/// B`); else of the last one before it with nothing between of any other kind and no group closing (`Town\\
/// $^2$\textbf{Univ B}`); else, nothing before it in the content of any other kind, the call's own. `None` when the
/// boxes cannot tell (a switch between: `\itshape $^1$\textbf{Univ A}`). `base` is the position of `content[0]`;
/// `events` the content's, whose marks, breaks and separators are no switch.
pub(in crate::base_utilities) fn entry_font(
  content: &[Token],
  base: usize,
  from: usize,
  scope: &FontScope,
  events: &[Event],
) -> Option<Rc<Font>> {
  let pos = |i: usize| u16::try_from(base + i).ok();
  let printed = |p: u16| printed_by_event(events, p);
  // (a character box made from the token itself)
  let ink = |i: usize| {
    let t = &content[i];
    if !matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
      return None;
    }
    pos(i)
      .filter(|&p| !printed(p))
      .and_then(|p| scope.ink.at.get(&p))
      .filter(|(_, text, _)| t.with_str(|s| text.starts_with(s)))
      .map(|(font, ..)| font.clone())
  };
  let neutral = |i: usize| neutral_token(content, base, i, events);
  // (a line break's own box at the token: its font)
  let break_font = |i: usize| {
    let p = pos(i)?;
    events
      .iter()
      .any(|e| e.kind == EventKind::Break && e.span.is_some_and(|(s, t)| s <= p && p < t))
      .then(|| scope.ink.whatsits.get(&p).cloned())
      .flatten()
  };
  let found = (|| {
    let mut depth = 0i32;
    for (i, t) in content.iter().enumerate().skip(from) {
      if let Some(font) = ink(i) {
        return Some(font);
      }
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END if depth > 0 => depth -= 1,
        _ if neutral(i) => {},
        _ => break,
      }
    }
    // (a group closed before it sets nothing after it: passed whole; a line break prints in the font in effect there,
    // which nothing between it and the entry switches — `\textbf{Univ A}\\ $^2$…`, `Town~\\ $^2$…`: read from its box,
    // review rev94b r4 B1)
    let mut depth = 0i32;
    let start = scope.start.saturating_sub(base);
    for i in (start..from.min(content.len())).rev() {
      match content[i].get_catcode() {
        Catcode::END => depth += 1,
        Catcode::BEGIN if depth > 0 => depth -= 1,
        _ if depth > 0 => {},
        _ => {
          if let Some(font) = ink(i).or_else(|| break_font(i)) {
            return Some(font);
          }
          if !(content[i].get_catcode() == Catcode::BEGIN || neutral(i)) {
            return None;
          }
        },
      }
    }
    Some(scope.root.clone())
  })()?;
  Some(Rc::new(font_changes(&scope.root, &found)))
}

/// Whether the content position `p` is printed by a mark, a line break or a separator of `events` — no switch.
fn printed_by_event(events: &[Event], p: u16) -> bool {
  events.iter().any(|e| {
    matches!(
      e.kind,
      EventKind::Mark | EventKind::Break | EventKind::Sep(_)
    ) && e.span.is_some_and(|(s, t)| s <= p && p < t)
  })
}

/// Whether `content[i]` (`content[0]` at position `base`) is no switch: nothing of its own — a space, a math shift,
/// LaTeX's `\(` and `\)` spelling one (`\protect`ed), a script — or what a mark, a break or a separator of `events`
/// prints.
fn neutral_token(content: &[Token], base: usize, i: usize, events: &[Event]) -> bool {
  let t = &content[i];
  matches!(
    t.get_catcode(),
    Catcode::SPACE | Catcode::MATH | Catcode::SUPER | Catcode::SUB
  ) || (t.get_catcode() == Catcode::CS
    && t.with_cs_name(|n| matches!(n, "\\(" | "\\)" | "\\protect")))
    || u16::try_from(base + i).is_ok_and(|p| printed_by_event(events, p))
}

/// Whether nothing in `content[from..to]` switches a font ([`neutral_token`], braces aside): a container's argument up
/// to its first entry (`writer::container_wrap`).
pub(in crate::base_utilities) fn switch_free(
  content: &[Token],
  base: usize,
  from: usize,
  to: usize,
  events: &[Event],
) -> bool {
  (from..to.min(content.len())).all(|i| {
    matches!(content[i].get_catcode(), Catcode::BEGIN | Catcode::END)
      || neutral_token(content, base, i, events)
  })
}

/// The changes making `from` into `to`: each of the family, series, shape, size and colours that differ.
fn font_changes(from: &Font, to: &Font) -> Font {
  Font {
    family: (to.family != from.family)
      .then(|| to.family.clone())
      .flatten(),
    series: (to.series != from.series)
      .then(|| to.series.clone())
      .flatten(),
    shape: (to.shape != from.shape).then(|| to.shape.clone()).flatten(),
    size: (to.size != from.size).then_some(to.size).flatten(),
    color: (to.color != from.color).then_some(to.color).flatten(),
    bg: (to.bg != from.bg).then_some(to.bg).flatten(),
    ..Font::default()
  }
}

/// Whether `font` changes anything ([`font_changes`]).
pub(in crate::base_utilities) fn changes_font(font: &Font) -> bool {
  font.family.is_some()
    || font.series.is_some()
    || font.shape.is_some()
    || font.size.is_some()
    || font.color.is_some()
    || font.bg.is_some()
}
