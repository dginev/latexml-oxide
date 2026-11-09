//! What a span of a field holds (the digested refactor's Stage 3, `docs/archive/FRONTMATTER_DIGESTED_REFACTOR_2026-11-07.md`): the tokens a
//! piece, a line or an entry the reading cut is made from, read from the digested list and its spans. A box covers what
//! it prints and no more, so a span stops short of what prints nothing around it, and that is said by the digested list
//! too, not by walking tokens back from the span: the formula it starts or ends in, whole (`Field::formulas`, whatever
//! opens it: `$\rm ^{3}$`, `${}^{*}$`, `\( ^{1} \)`, `\protect\(`, 2105.07958, hep-ph0110378, 2307.12984), and a mark's own
//! print inside a wrapper whose cover its first box took ([`Field::own_print`], `\textbf{$^2$Univ B}`, 2605.25195). What
//! is left of the tokens is brace balance, a property of the content itself, as the declarations' spans clamp at it
//! (`field_notes::settle_declarations`).

use super::{
  super::{
    events::{EventKind, HeldError},
    field_notes::EnvironmentStep,
  },
  plan::reads_as_alignment,
  *,
};

/// A field's tokens as a rehearsal read them: `tokens[0]` at position `base`, and its formulas (`events::lex_with_formulas`:
/// outermost, in order).
pub(in crate::base_utilities::frontmatter::observer) struct Field<'a> {
  pub tokens:       &'a [Token],
  pub base:         usize,
  pub formulas:     &'a [Formula],
  /// the environments they open and close, as the digestion's frames say it (`field_notes::EnvironmentStep`)
  pub environments: &'a [EnvironmentStep],
  /// the errors its rehearsal held, where (`events::HeldError`; none for a call's rehearsal)
  pub held:         &'a [HeldError],
  /// whether their digestion holds an alignment ([`aligned`])
  pub aligned:      bool,
  /// their groups as the braces open them, read once for the field ([`Groups`])
  pub groups:       Groups,
}

/// A field's groups as its braces open and close them, read once as far as asked, not again from its start for each
/// piece: before token `i`, the innermost group still open — `inner[i]`, its opening brace's index plus one, 0 for none —,
/// a group's own enclosing one the innermost open before its brace (`inner[brace]`). A closing brace no group is open at
/// closes none (1507.02068's 418 names, each piece's open groups read from the block's start: about 0.7 % of the run).
#[derive(Debug, Default)]
pub(in crate::base_utilities::frontmatter::observer) struct Groups {
  inner: std::cell::RefCell<Vec<usize>>,
}

impl Groups {
  /// The opening braces in `content[from..to]` of the groups still open at `to`, in order: the innermost open group's
  /// enclosing ones down to `from`, each read once for the field.
  fn open(&self, content: &[Token], from: usize, to: usize) -> Vec<usize> {
    let to = to.min(content.len());
    let mut inner = self.inner.borrow_mut();
    if inner.is_empty() {
      inner.push(0);
    }
    while inner.len() <= to {
      let i = inner.len() - 1;
      let here = inner[i];
      let next = match content[i].group_step() {
        1 => i + 1,
        -1 => here.checked_sub(1).map_or(0, |brace| inner[brace]),
        _ => here,
      };
      inner.push(next);
    }
    let mut open = Vec::new();
    let mut at = inner[to];
    while let Some(brace) = at.checked_sub(1).filter(|&brace| brace >= from) {
      open.push(brace);
      at = inner[brace];
    }
    open.reverse();
    open
  }
}

/// Whether a digested list's events hold an alignment's cell (`events::Walker`'s `Layout{role="cell"}`): where none is, no tab of
/// it is an alignment's — a verbatim argument read it as a character (`\url{http://y.org/?c=1&d=2}`), or it was a stray one in
/// the rehearsal as everywhere. A note's events too (`Event::inner`, `Event::body`): a tabular in a `\thanks`.
pub(in crate::base_utilities::frontmatter::observer) fn aligned(events: &[Event]) -> bool {
  events.iter().any(|e| {
    (e.kind == EventKind::Layout && e.role.as_deref() == Some("cell"))
      || aligned(&e.inner)
      || aligned(&e.body)
  })
}

/// Where a call in `tokens` reads past them: a macro at their top level whose parameters are each undelimited (TeX's
/// `#1#2`, `\newcommand`'s `[2]`; a verbatim group, `\href`'s URL) or an optional one (`[…]`), by its meaning now, an
/// undelimited argument of which the tokens don't hold — the next read at a closing brace or past the last token, there
/// the wrapper's own tokens (`\lx@author@withinst`'s `\let\inst\lx@saved@inst…`, `frontmatter_fields.rs`) or the item's
/// brace. Whether it is cut between its arguments (`\def\authord#1#2{…}`'s `\authord{Anh Thu Nguyen}`, 2209.12410) or
/// the tokens end at it, none of its arguments theirs (a mark's span covering the call alone: aiaa-tc's
/// `\thanksibid{2}`, 1505.07844, 1411.3642, 1602.00849; a preamble's `\footrecall{uva}`, 2503.21719, 2407.13730,
/// 2606.19583; a URL cut out of `\href{…}{…}`, 2603.11001). A parameter of any other kind (a delimited one) is not
/// judged, nor a command inside a group of theirs (an argument a primitive may never expand: `\detokenize{\begin}`).
pub(super) fn reads_past(tokens: &[Token]) -> Option<usize> {
  let mut depth = 0i32;
  tokens.iter().enumerate().position(|(i, t)| {
    depth += t.group_step();
    depth == 0 && arguments_end(tokens, i) == Some(None)
  })
}

/// Where the call at `tokens[at]` ends, past the arguments its meaning now reads ([`reads_past`]'s parameters):
/// `Some(None)` where an undelimited one is not in the tokens, `None` for no such call (a primitive, an unexpandable
/// command, no parameters, a parameter of another kind).
pub(super) fn arguments_end(tokens: &[Token], at: usize) -> Option<Option<usize>> {
  let (plain, verbatim, hyper, optional) = (
    pin!("Plain"),
    pin!("Semiverbatim"),
    pin!("HyperVerbatim"),
    pin!("Optional"),
  );
  let Some(Stored::Expandable(meaning)) = lookup_meaning(&tokens[at]) else {
    return None;
  };
  let parameters = meaning.get_parameters()?.get_parameters();
  if parameters.is_empty()
    || parameters
      .iter()
      .any(|p| !(p.name == plain || p.name == verbatim || p.name == hyper || p.name == optional))
  {
    return None;
  }
  let mut j = at + 1;
  for parameter in parameters {
    while tokens
      .get(j)
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      j += 1;
    }
    if parameter.name == optional {
      // (an option present is read to its `]` at the top level; one absent reads nothing: the next token is peeked)
      if tokens
        .get(j)
        .is_some_and(|t| t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "["))
      {
        let mut inner = 0i32;
        loop {
          let Some(t) = tokens.get(j) else {
            return Some(None);
          };
          inner += t.group_step();
          j += 1;
          if inner == 0 && t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == "]") {
            break;
          }
        }
      }
      continue;
    }
    match tokens.get(j).map(Token::group_step) {
      None | Some(-1) => return Some(None),
      Some(1) => {
        let mut inner = 0i32;
        loop {
          let Some(t) = tokens.get(j) else {
            return Some(None);
          };
          inner += t.group_step();
          j += 1;
          if inner == 0 {
            break;
          }
        }
      },
      Some(_) => j += 1,
    }
  }
  Some(Some(j))
}

/// Where `tokens` hold a conditional cut apart, by their meanings now: an `\else`, `\or` or `\fi` whose `\if…` is not
/// theirs — digested apart, it errs ("not in a conditional"). An `\if…` whose `\fi` is not theirs is not judged: a
/// switch's name is one too (`\newif\iffull`, `\let\ifa\iftrue`), which no meaning tells apart from a test.
fn conditional_cut(tokens: &[Token]) -> Option<usize> {
  let mut open = 0usize;
  for (i, t) in tokens.iter().enumerate() {
    let kind = lookup_definition(t)
      .ok()
      .flatten()
      .and_then(|meaning| meaning.get_conditional_type());
    match kind {
      Some(ConditionalType::If) => open += 1,
      Some(ConditionalType::Else | ConditionalType::Or | ConditionalType::Fi) if open == 0 => {
        return Some(i);
      },
      Some(ConditionalType::Fi) => open -= 1,
      _ => {},
    }
  }
  None
}

/// Whether `t` is the control sequence `name`.
pub(super) fn is_cs(t: &Token, name: &str) -> bool {
  t.get_catcode() == Catcode::CS && t.with_cs_name(|n| n == name)
}

impl Field<'_> {
  /// The opening braces in `tokens[from..to]` of the groups still open at `to` ([`Groups`]).
  fn open_groups(&self, from: usize, to: usize) -> Vec<usize> {
    self.groups.open(self.tokens, from, to)
  }

  /// The formula holding `tokens[index]`: its indices, opening shift to past its closing shift.
  fn formula_at(&self, index: usize) -> Option<(usize, usize)> {
    let pos = u16::try_from(self.base + index).ok()?;
    let at = self.formulas.partition_point(|formula| formula.end <= pos);
    let formula = self.formulas.get(at)?;
    (formula.start <= pos).then_some((
      usize::from(formula.start).checked_sub(self.base)?,
      usize::from(formula.end).checked_sub(self.base)?,
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
    range: (usize, usize),
    floor: usize,
    first: usize,
  ) -> Option<(Tokens, usize)> {
    self.content_cut(range, floor, first, &[])
  }

  /// [`Self::content_of`] without the runs `cuts` (indices, each balanced: an entry's address runs, `address_cut`), `None`
  /// where one of them is partly inside the content and partly outside it.
  pub(super) fn content_cut(
    &self,
    (start, end): (usize, usize),
    floor: usize,
    first: usize,
    cuts: &[(usize, usize)],
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
    if let Some(&group) = self.open_groups(0, start).last()
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
    if cuts
      .iter()
      .any(|&(s, t)| s < end && t > start && (s < start || t > end))
    {
      return None;
    }
    if cuts.is_empty() && self.held.is_empty() {
      tokens.extend_from_slice(&content[start..end]);
    } else {
      // (a script character its rehearsal read outside math, in no formula, an error held there, is the character that rehearsal printed:
      // `\textbf{Ningkai Mo} ^1` is a text-mode `^`, which TeX errs at (Missing $) and the digestion's recovery prints;
      // digested apart as a script it erred again, once per piece (2204.01080). `\lx@fm@script@char` prints it and
      // counts the recovery, a Warning. An `_` a verbatim argument reads as a character holds no error: `\url{a_b}` stays)
      let cut = |i: &usize| cuts.iter().any(|&(s, t)| (s..t).contains(i));
      for i in (start..end).filter(|i| !cut(i)) {
        let t = content[i];
        if matches!(t.get_catcode(), Catcode::SUPER | Catcode::SUB)
          && self.formula_at(i).is_none()
          && self.held_at(i)
        {
          tokens.push(T_CS!("\\lx@fm@script@char"));
          tokens.push(T_BEGIN!());
          tokens.push(T_OTHER!(if t.get_catcode() == Catcode::SUPER {
            "^"
          } else {
            "_"
          }));
          tokens.push(T_END!());
        } else {
          tokens.push(t);
        }
      }
    }
    // (never a run holding an alignment's tab or alignment-only command outside an environment or a formula it holds whole, as
    // the digestion's frames say them (a macro pair's `\bx a&b\\ c&d\ex` is one): digested apart, it is a stray one, an Error
    // — its caller writes the item's text, counted (the rule of `plan::cell_tokens`; 1304.3138, 1504.01048, 2607.22455). Only
    // where the digestion holds an alignment ([`aligned`]): a `\url{…?c=1&d=2}` in a block of none keeps its link (review
    // revNE1 1))
    // (nor a run a macro of which reads past its end, by its meaning, which the run is digested under: `\def\authord#1#2{…}`'s
    // `\authord{Anh Thu Nguyen}`, its second argument the affiliation's piece, read the item's closing brace (an undefined
    // `\@undefined` and a group ended in the wrong mode, 2209.12410) — its caller writes the item's text, counted)
    // (nor a run holding a conditional cut apart, an `\else` whose `\if` is not the run's: `\ifFull and \else \and \fi
    // Joseph A. Simons`, whose name's run took the `\else \and \fi` the rehearsal skipped, an Error and a tabular's group
    // ended in the wrong mode (1409.0597) — its caller writes the item's text, counted)
    if let Some(at) = conditional_cut(&tokens) {
      DebugFeature!(
        "frontmatter",
        "FRONT writer: the run {start}..{end} holds {:?}, a conditional cut apart",
        tokens[at]
      );
      return None;
    }
    if let Some(call) = reads_past(&tokens) {
      DebugFeature!(
        "frontmatter",
        "FRONT writer: the run {start}..{end} holds {:?}, whose arguments run past it",
        tokens[call]
      );
      return None;
    }
    // (what it holds whole read only for a run holding such a token at all)
    let mut held: Option<Vec<(usize, usize)>> = None;
    if let Some(stray) = (start..end).find(|&i| {
      self.aligned
        && reads_as_alignment(&content[i])
        && !cuts.iter().any(|&(s, t)| (s..t).contains(&i))
        && !held
          .get_or_insert_with(|| self.held_whole(start, end))
          .iter()
          .any(|&(s, t)| (s..t).contains(&i))
    }) {
      DebugFeature!(
        "frontmatter",
        "FRONT writer: the run {start}..{end} holds an alignment's {:?} at {stray}",
        content[stray]
      );
      return None;
    }
    Some((Tokens::new(tokens), end))
  }

  /// Whether its rehearsal held an error at `tokens[index]`'s position.
  fn held_at(&self, index: usize) -> bool {
    let pos = self.base + index;
    self
      .held
      .iter()
      .filter_map(|error| error.span)
      .any(|(from, to)| usize::from(from) <= pos && pos < usize::from(to))
  }

  /// The environments (their frames' steps, paired) and the formulas the run `start..end` holds whole: their indices.
  fn held_whole(&self, start: usize, end: usize) -> Vec<(usize, usize)> {
    let at = |p: u16| usize::from(p).checked_sub(self.base);
    let mut held = Vec::new();
    let mut open = Vec::new();
    for step in self.environments {
      let (Some(from), Some(to)) = (at(step.start), at(step.end)) else {
        continue;
      };
      if from < start || from >= end {
        continue;
      }
      if step.opens {
        open.push(from);
      } else if let Some(opened) = open.pop()
        && to <= end
      {
        held.push((opened, to));
      }
    }
    let formulas = self
      .formulas
      .iter()
      .filter_map(|formula| Some((at(formula.start)?, at(formula.end)?)));
    held.extend(formulas.filter(|&(s, t)| start <= s && t <= end));
    held
  }

  /// Whether the `nth` group around the formula starting at `from`, from the innermost, closing at `close`, starts at its brace with that formula: it prints
  /// the formula alone (`events::Formula::alone_in`: `{\bf $^{1}$} Univ X`, `{$^{1}$\,} Univ X`, a mark's own group), or the formula opens it where a line
  /// starts (`Formula::opens_line_in`: `\\ \normalsize{$^{1}$ Univ X,} Moscow`, 2005.08290, the entry's own group). A group going on after a mark elsewhere is
  /// no such thing, and no cut: `Univ A, \textit{$^{2}$H} Lab`, an isotope (`perfect_kernel_batch107e::a_wrapped_isotope_is_entry_text`).
  fn starts_at_its_brace(&self, nth: u8, from: usize, close: usize) -> bool {
    let Ok(pos) = u16::try_from(self.base + from) else {
      return false;
    };
    let at = self.formulas.partition_point(|formula| formula.end <= pos);
    self.formulas.get(at).is_some_and(|formula| {
      formula.start == pos
        && usize::from(formula.end) <= self.base + close
        && (formula.alone_in >= nth || formula.opens_line_in >= nth)
    })
  }

  /// `start..end` grown to a balanced run, its shifts in pairs: back to the opening brace of a group it closes that starts there
  /// ([`Self::starts_at_its_brace`]; any other group it closes is no run), on over the closing braces right after a group it
  /// opens.
  pub(super) fn balanced(&self, start: usize, end: usize, floor: usize) -> Option<(usize, usize)> {
    let content = self.tokens;
    let (mut start, mut end) = (start, end);
    let mut before = self.open_groups(floor, start);
    let from = start;
    let mut depth = 0;
    let mut closed = 0u8;
    for (close, t) in content.iter().enumerate().take(end).skip(from) {
      depth += t.group_step();
      if depth < 0 {
        let open = before.pop()?;
        closed = closed.saturating_add(1);
        self
          .starts_at_its_brace(closed, from, close)
          .then_some(())?;
        start = open;
        depth = 0;
      }
    }
    while depth > 0 {
      (content.get(end)?.group_step() == -1).then_some(())?;
      end += 1;
      depth -= 1;
    }
    // (its environments in pairs, as the digestion's frames say them: what `\end{ee}` prints or closes holds a position no box stops at, its
    // body's `E` written in `\newenvironment{ee}{B}{E}`, an alignment's last cell, and a run with no `\end` lost them, in an Error; a `\let` copy
    // and a macro pair count, a `\begin` under `\detokenize` doesn't)
    let at = |p: u16| usize::from(p).checked_sub(self.base);
    let mut environments = 0i32;
    for step in self.environments {
      // (one a formula holds is the formula's, taken whole: `$\begin{array}{c}a\end{array}$`, review revZ4a)
      let Some(from) = at(step.start).filter(|&from| self.formula_at(from).is_none()) else {
        continue;
      };
      if from < start {
        continue;
      }
      if from >= end && environments == 0 {
        break;
      }
      environments += if step.opens { 1 } else { -1 };
      (environments >= 0).then_some(())?;
      if from >= end || environments == 0 {
        end = end.max(at(step.end)?.min(content.len()));
        // (an alignment's cell read its `\\end` and heard the name's brace next: the name is the `\\end`'s)
        if !step.opens
          && at(step.end)? == from + 1
          && content
            .get(end)
            .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
        {
          let mut depth = 0;
          while let Some(t) = content.get(end) {
            depth += t.group_step();
            end += 1;
            if depth == 0 {
              break;
            }
          }
        }
      }
    }
    (environments == 0).then_some(())?;
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
