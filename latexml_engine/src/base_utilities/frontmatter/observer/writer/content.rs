//! What a span of a field holds (the digested refactor's Stage 3, `FRONTMATTER_DIGESTED_REFACTOR.md`): the tokens a
//! piece, a line or an entry the reading cut is made from, read from the digested list and its spans. A box covers what
//! it prints and no more, so a span stops short of what prints nothing around it, and that is said by the digested list
//! too, not by walking tokens back from the span: the formula it starts or ends in, whole (`Field::formulas`, whatever
//! opens it: `$\rm ^{3}$`, `${}^{*}$`, `\( ^{1} \)`, `\protect\(`, 2105.07958, hep-ph0110378, 2307.12984), and a mark's own
//! print inside a wrapper whose cover its first box took ([`Field::own_print`], `\textbf{$^2$Univ B}`, 2605.25195). What
//! is left of the tokens is brace balance, a property of the content itself, as the declarations' spans clamp at it
//! (`field_notes::balance_declarations`).

use super::*;

/// A field's tokens as a rehearsal read them: `tokens[0]` at position `base`, and the extents of its formulas
/// (`events::lex_with_formulas`: outermost, in order).
pub(in crate::base_utilities::frontmatter::observer) struct Field<'a> {
  pub tokens:   &'a [Token],
  pub base:     usize,
  pub formulas: &'a [(u16, u16)],
}

/// Whether `t` is the control sequence `name`.
pub(super) fn is_cs(t: &Token, name: &str) -> bool {
  t.get_catcode() == Catcode::CS && t.with_cs_name(|n| n == name)
}

/// The opening braces in `content[from..to]` of the groups still open at `to`.
fn open_groups(content: &[Token], from: usize, to: usize) -> Vec<usize> {
  let mut open = Vec::new();
  for (i, t) in content.iter().enumerate().take(to).skip(from) {
    match t.group_step() {
      1 => open.push(i),
      -1 => {
        open.pop();
      },
      _ => {},
    }
  }
  open
}

impl Field<'_> {
  /// The formula holding `tokens[index]`: its indices, opening shift to past its closing shift.
  fn formula_at(&self, index: usize) -> Option<(usize, usize)> {
    let pos = u16::try_from(self.base + index).ok()?;
    let at = self.formulas.partition_point(|&(_, end)| end <= pos);
    let &(start, end) = self.formulas.get(at)?;
    (start <= pos).then_some((
      usize::from(start).checked_sub(self.base)?,
      usize::from(end).checked_sub(self.base)?,
    ))
  }

  /// `index`, or where the formula holding it starts.
  pub(super) fn formula_start(&self, index: usize) -> usize {
    self.formula_at(index).map_or(index, |(start, _)| start)
  }

  /// `index` (the end of a span), or past the formula holding the token before it.
  pub(super) fn formula_end(&self, index: usize) -> usize {
    index
      .checked_sub(1)
      .and_then(|last| self.formula_at(last))
      .map_or(index, |(_, end)| end.max(index))
  }

  /// A Mark's own print as indices, where its span is wider or narrower: the whole formula it is in (the first box in a
  /// wrapper takes the cover of the call around it: `\textbf{$^2$Univ B}`'s span is the wrapper's), or the
  /// `\textsuperscript{…}` call to its closing brace. `None` for a mark of neither.
  pub(super) fn own_print(&self, mark: &Event) -> Option<(usize, usize)> {
    let index = |p: u16| usize::from(p).checked_sub(self.base);
    if let Some(formula) = mark.formula.filter(|f| f.base.is_some()) {
      let (start, end) = formula.span?;
      return Some((index(start)?, index(end)?));
    }
    let call = index(mark.own_at?)?;
    let mut depth = 0;
    for (k, t) in self.tokens.iter().enumerate().skip(call + 1) {
      if depth == 0 && t.group_step() != 1 {
        return None;
      }
      depth += t.group_step();
      if depth == 0 {
        return Some((call, k + 1));
      }
    }
    None
  }

  /// The content of the span `start..end` (indices), past `floor` (where the piece before ended): the balanced run of
  /// tokens holding it, and where that ends — with the switches that open the group it sits in, when it sits inside one
  /// (`{\bf $^1$Univ A, $^2$Univ B}`: each piece keeps the `\bf`, unless `first` is 0: a piece carrying the font it
  /// starts in, `events::entry_font`) —, or `None` when no balanced run holds it.
  ///
  /// The run takes each formula the span starts or ends in whole, a mark's own font group from its opening brace
  /// (`{\bf $^1$} Univ A`) and the closing braces right after a group it opens.
  pub(super) fn content_of(
    &self,
    (start, end): (usize, usize),
    floor: usize,
    first: usize,
  ) -> Option<(Tokens, usize)> {
    let content = self.tokens;
    if start > end || end > content.len() || start < floor {
      return None;
    }
    let (mut start, mut end) = (start, end);
    // (a formula the piece before ended in stays as it was, its other end past `floor`: the mark's closing shift is the
    // mark's, `$^1 Univ$`)
    if let Some((s, e)) = self.formula_at(start).filter(|&(s, _)| s >= floor) {
      start = s;
      end = end.max(e);
    }
    if let Some((s, e)) = end
      .checked_sub(1)
      .and_then(|last| self.formula_at(last))
      .filter(|&(s, _)| s >= floor)
    {
      start = start.min(s);
      end = end.max(e);
    }
    end = end.min(content.len());
    let (start, end) = self.balanced(start, end, floor)?;
    // (the group the pieces sit in: its opening brace, and the switches before the first piece in it)
    let mut tokens = Vec::new();
    if let Some(&group) = open_groups(content, 0, start).last()
      && group < first
    {
      let switches = &content[group + 1..first.min(start)];
      if switches
        .iter()
        .all(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::SPACE))
      {
        tokens.extend_from_slice(switches);
      }
    }
    tokens.extend_from_slice(&content[start..end]);
    Some((Tokens::new(tokens), end))
  }

  /// Whether the group from `open` to `close` holds only switches and the formula starting at `from`: `{\bf $^{1}$} Univ X`, a
  /// mark's own font group. A group going on after the mark is no such thing, and no cut: `\textit{$^{2}$H} Lab`, an isotope
  /// (`perfect_kernel_batch107e::a_wrapped_isotope_is_entry_text`).
  fn holds_only_a_formula(&self, open: usize, from: usize, close: usize) -> bool {
    let content = self.tokens;
    let Some((start, end)) = self.formula_at(from) else {
      return false;
    };
    start == from
      && end <= close
      && content[open + 1..from]
        .iter()
        .all(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::SPACE))
      && content[end..close]
        .iter()
        .all(|t| t.get_catcode() == Catcode::SPACE)
  }

  /// `start..end` grown to a balanced run, its shifts in pairs: back to the opening brace of a mark's own font group it closes
  /// ([`Self::holds_only_a_formula`]; any other group it closes is no run), on over the closing braces right after a group it
  /// opens.
  fn balanced(&self, start: usize, end: usize, floor: usize) -> Option<(usize, usize)> {
    let content = self.tokens;
    let (mut start, mut end) = (start, end);
    let mut before = open_groups(content, floor, start);
    let from = start;
    let mut depth = 0;
    for (close, t) in content.iter().enumerate().take(end).skip(from) {
      depth += t.group_step();
      if depth < 0 {
        let open = before.pop()?;
        self.holds_only_a_formula(open, from, close).then_some(())?;
        start = open;
        depth = 0;
      }
    }
    while depth > 0 {
      (content.get(end)?.group_step() == -1).then_some(())?;
      end += 1;
      depth -= 1;
    }
    // (a formula the digested list did not say, its shifts in pairs all the same)
    let run = &content[start..end];
    let spelled = |name| run.iter().filter(|t| is_cs(t, name)).count();
    let shifts = run
      .iter()
      .filter(|t| t.get_catcode() == Catcode::MATH)
      .count();
    if spelled("\\(") != spelled("\\)") {
      return None;
    }
    if shifts % 2 == 1 {
      (content.get(end)?.get_catcode() == Catcode::MATH).then_some(())?;
      end += 1;
    }
    Some((start, end))
  }
}
