//! The reading's item keys and owners ([`Key`], [`Owner`], [`creator_items`], [`block_items`], [`contact_key`]) and the
//! token slices the emitter writes an entry from: an author block's line at a span ([`block_span_tokens`]), an entry's
//! forms and lead ([`entry_forms`], [`labelled_start`], [`span_tokens`]) and its font ([`entry_font_tokens`]). What is
//! left of the S2b writer's plan (OXIDIZED_DESIGN_DIVERGENCES #837 "S2b"), whose comparison with the placed
//! frontmatter and DOM operations were deleted with it (DEL4).

use std::collections::{BTreeSet, HashMap};

use super::{
  super::{
    events::{
      Event, EventKind, FontScope, PICTURE_NOTE, Wrap, changes_font, entry_font, font_declaration,
      switch_free, text_font_command,
    },
    ledger::{ApiCall, Outcome},
    resolver::{base_letters, spelled_name},
  },
  content::{Field, is_cs},
  dom::{Kind, norm, orcid_id},
  *,
};

/// Whose an item is: a resolved creator's (its index), or the author block's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Owner {
  Creator(usize),
  Block,
}

pub(super) type Key = (Kind, String);

/// A resolved creator's items as the comparison keys them — its affiliations, contacts and notes, an item of no letter
/// or digit none ([`wanted`]).
pub(super) fn creator_items(c: &RCreator) -> Vec<(Key, &Assoc)> {
  let mut items: Vec<(Key, &Assoc)> = Vec::new();
  for a in &c.affiliations {
    items.push(((Kind::Affiliation, norm(&a.text)), a));
  }
  for a in &c.contacts {
    items.push((contact_key(a), a));
  }
  for a in &c.notes {
    items.push(((Kind::Note, note_key(a)), a));
  }
  items.retain(|(key, _)| !key.1.is_empty());
  items
}

/// A note's comparison key: its text, normalized; a picture's ([`PICTURE_NOTE`]) keyed by where it is, whatever text it
/// prints — an item of its own, written whole (OD #989): a drawing alone or a logo prints none, and two pictures printing
/// the same text are two (`Ann Able \tikz\node{Zebraword}; \tikz\node{Zebraword};` kept one, its twin's word lost: review
/// revCR8 finding 3).
pub(super) fn note_key(a: &Assoc) -> String {
  if a.role == PICTURE_NOTE
    && let Some((at, Some((start, end)))) = a.source
  {
    return s!("picture@{at}:{start}-{end}");
  }
  norm(&a.text)
}

/// A resolved creator's contact `a` as the comparison keys it ([`creator_items`]).
pub(super) fn contact_key(a: &Assoc) -> Key {
  match a.role.as_str() {
    "note" | "thanks" => (Kind::Note, norm(&a.text)),
    "orcid" => (Kind::Contact("orcid".into()), norm(orcid_id(&a.text))),
    role => (Kind::Contact(role.to_string()), norm(&a.text)),
  }
}

/// Each item of `resolved` (its creators' and its block's) by its address, with the key the emit reads its entry by
/// (`emit::item_call`'s `key.1`: [`creator_items`], [`block_items`]); what the picture check reads an item's written span
/// by (`resolver::words::Written::new`), so the two can't differ (an ORCID keyed by its identifier: review revIMG1b
/// finding 2). An item of no letter or digit has none, nor its entry.
pub(super) fn written_keys(resolved: &Resolved) -> HashMap<*const Assoc, String> {
  resolved
    .creators
    .iter()
    .flat_map(creator_items)
    .chain(block_items(resolved))
    .map(|((_, key), a)| (std::ptr::from_ref(a), key))
    .collect()
}

/// The author block's own items, as the comparison keys them ([`wanted`]).
pub(super) fn block_items(resolved: &Resolved) -> Vec<(Key, &Assoc)> {
  resolved
    .block
    .iter()
    .map(|a| {
      let key = if a.role == PICTURE_NOTE {
        note_key(a)
      } else {
        norm(&a.text)
      };
      ((Kind::of_block(&a.role), key), a)
    })
    .filter(|(key, _)| !key.1.is_empty())
    .collect()
}

/// The API call at ledger index `index`.
pub(super) fn api_call(index: usize) -> Option<ApiCall> {
  with_ledger(|ledger| match ledger.entries().get(index) {
    Some(Entry::Api(call)) => Some(call.clone()),
    _ => None,
  })
}

/// The tokens of the block at ledger index `index` an entry's `span` covers, as [`block_line_tokens`] reads them — the
/// span's own letters and digits checked against `key` when one is given (the re-part checks what it made on the
/// digested text instead, `repart`). With `mark` (a legend, whose text keeps its mark's ink: the label-text ruling)
/// from the mark it opens with; else from its own ink, the mark left out — a made affiliation carries no entry mark, as
/// today's copies hold none, and a mark's formula cut from a group the entry closes (`{\small $~^2$ Univ of
/// Colorado}`) is no balanced run: 2308.11662's second institute was no cut, its two lines one contact (review rev94a
/// B3, S3; 2404.02078, 2609.03563, 1909.05311).
pub(super) fn block_span_tokens(
  index: usize,
  span: (u16, u16),
  key: Option<&str>,
  with_mark: bool,
) -> Option<Tokens> {
  with_ledger(|ledger| {
    let Some(Entry::Author(block)) = ledger.entries().get(index) else {
      return None;
    };
    let Outcome::Events {
      events,
      formulas,
      environments,
      held,
      ..
    } = &block.outcome
    else {
      return None;
    };
    let field = Field {
      tokens: block.tokens.unlist_ref(),
      base: block.range.0,
      formulas,
      environments,
      held,
      aligned: aligned(events),
      groups: Default::default(),
    };
    // (the event printed last before the line, a layout change aside)
    let mut mark = events
      .iter()
      .rev()
      .find(|e| e.kind != EventKind::Layout && e.span.is_some_and(|(_, e)| e <= span.0))
      .filter(|e| e.kind == EventKind::Mark)
      .and_then(|e| e.span);
    // (or the mark heading the entry from inside the entry's own wrapper, `\textbf{$^2$Univ B}`: its own formula, the
    // wrapper opened again round the entry made — [`wrapped_entry_mark`])
    let wrapper = mark
      .is_none()
      .then(|| wrapped_entry_mark(events, &field, span))
      .flatten();
    if let Some((own, _)) = &wrapper {
      mark = Some(*own);
    }
    let labelled = events
      .iter()
      .find(|e| e.kind == EventKind::Contact && e.label.is_some() && e.span == Some(span));
    // (a cut entry's font, `events::entry_font`: the entry digested under it, the switches its cut starts past left out;
    // and the containers holding it and another entry, wrapped round it again: `entry_forms`)
    let font = labelled.and_then(|e| e.font.as_ref());
    let wrap = labelled.and_then(|e| e.wrap.as_deref());
    let tokens = span_tokens(
      &field,
      span,
      key,
      mark,
      labelled.map(|e| e.marks),
      with_mark,
      font.is_some(),
    )?;
    let tokens = match wrapper {
      Some((_, opening)) => {
        let mut wrapped = opening;
        wrapped.extend(
          tokens
            .unlist()
            .into_iter()
            .skip_while(|t| t.get_catcode() == Catcode::SPACE),
        );
        wrapped.push(T_END!());
        Tokens::new(wrapped)
      },
      None => tokens,
    };
    Some(entry_tokens(font, wrap, tokens))
  })
}

/// A mark heading the entry at `span` from inside the entry's own wrapper — a text font command's argument
/// (`\textbf{$^2$Tencent Hunyuan}`, 2605.25195's `\textbf{$^1$Fudan University}  \textbf{$^2$Tencent Hunyuan}\\ $^*$
/// Equal Contribution, …`) or a group opened by font declarations (`{\it $^2$Univ B}`) that closes where the entry ends —
/// in the block `field`: the Mark event's span is the wrapper's, the cover of the call around its first box, the entry
/// inside it. The mark's own print ([`Field::own_print`]: its formula, its `\textsuperscript{…}`), right before the
/// entry's ink, and the wrapper's opening tokens, to open it again round the entry made (the cut keeps its font) —
/// `None` when the event before the entry is no Mark covering it, or the wrapper holds more than the mark and the
/// entry (the kernel's cut reads such a mark as one, `calls::wrapped_mark_at`).
fn wrapped_entry_mark(
  events: &[Event],
  field: &Field,
  span: (u16, u16),
) -> Option<((u16, u16), Vec<Token>)> {
  let (content, base) = (field.tokens, field.base);
  let mark = events
    .iter()
    .rev()
    .find(|e| e.kind != EventKind::Layout && e.span.is_some_and(|(s, _)| s < span.0))
    .filter(|e| e.kind == EventKind::Mark)?;
  let (start, end) = mark.span?;
  if end < span.1 {
    return None;
  }
  let at = |p: u16| usize::from(p).checked_sub(base);
  let (open, from, to, past) = (at(start)?, at(span.0)?, at(span.1)?, at(end)?);
  // (the wrapper's closing brace, its last token: only spaces between it and the entry's end)
  let close = past.checked_sub(1)?;
  if content.get(close)?.get_catcode() != Catcode::END
    || content
      .get(to..close)?
      .iter()
      .any(|t| t.get_catcode() != Catcode::SPACE)
  {
    return None;
  }
  // (the mark's own print, right before the entry's ink and the spaces before it)
  let (formula, ink) = field.own_print(mark)?;
  if formula <= open
    || content
      .get(ink..from)?
      .iter()
      .any(|t| t.get_catcode() != Catcode::SPACE)
  {
    return None;
  }
  // (the wrapper's opening: a text font command and its brace, or a brace and the declarations after it)
  let opening = &content[open..formula];
  let space = |t: &Token| t.get_catcode() == Catcode::SPACE;
  let brace = match opening[0].get_catcode() {
    Catcode::CS
      if (text_font_command(&opening[0]) || is_cs(&opening[0], "\\emph"))
        && opening
          .get(1)
          .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
        && opening[2..].iter().all(space) =>
    {
      open + 1
    },
    Catcode::BEGIN if opening[1..].iter().all(|t| space(t) || font_declaration(t)) => open,
    _ => return None,
  };
  // (that brace closes at the wrapper's last token)
  let mut depth = 0usize;
  for (i, t) in content.iter().enumerate().take(past).skip(brace) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 && i != close {
          return None;
        }
      },
      _ => {},
    }
  }
  if depth != 0 {
    return None;
  }
  Some((
    (
      u16::try_from(base + formula).ok()?,
      u16::try_from(base + ink).ok()?,
    ),
    opening.to_vec(),
  ))
}

/// A cut entry's `tokens` in its form ([`entry_forms`]): wrapped in its containers again, digested under its font.
fn entry_tokens(font: Option<&Rc<Font>>, wrap: Option<&Wrap>, tokens: Tokens) -> Tokens {
  let tokens = match wrap {
    Some(wrap) => {
      let mut out = wrap.open.clone();
      out.extend(tokens.unlist());
      out.extend(std::iter::repeat_n(T_END!(), wrap.closes));
      Tokens::new(out)
    },
    None => tokens,
  };
  match font {
    Some(font) => entry_font_tokens(font, tokens),
    None => tokens,
  }
}

/// Each cut entry's form: the font changes its cut starts in (`events::entry_font`), and the containers holding its ink
/// and another entry's — an `\emph{$^1$Univ A, Town\\ $^2$Univ B, City}`, an `\underline{…}` (pdflatex prints both
/// lines in it) —, which it is wrapped in again ([`container_wrap`]), its font then the one in effect where the
/// outermost starts. `entries` each entry's cut start (an index into `content`, `content[0]` at position `base`) and
/// span. Exact only: a container whose entries do not read one font (a switch inside it between them), or one the
/// writer cannot wrap again ([`container_wrap`]), gives the entries it holds neither — they are cut as before fonts
/// were read (review rev94b r4 B1: attribution beats typography, a cut is never undone for its markup).
#[allow(clippy::type_complexity)]
pub(in crate::base_utilities::frontmatter::observer) fn entry_forms(
  content: &[Token],
  base: usize,
  entries: &[(usize, (u16, u16))],
  scope: &FontScope,
  events: &[Event],
) -> Vec<(Option<Rc<Font>>, Option<Rc<Wrap>>)> {
  let spans: Vec<(u16, u16)> = entries.iter().map(|(_, span)| *span).collect();
  let holders = scope.ink.holders(&spans);
  let leads: Vec<Option<Rc<Font>>> = entries
    .iter()
    .map(|(from, _)| entry_font(content, base, *from, scope, events))
    .collect();
  entries
    .iter()
    .enumerate()
    .map(|(at, &(from, (_, end)))| {
      let mine: Vec<&((u16, u16), BTreeSet<usize>)> = holders
        .iter()
        .filter(|(_, held)| held.contains(&at))
        .collect();
      if mine.is_empty() {
        return (leads[at].clone(), None);
      }
      let one_font = mine.iter().all(|(_, held)| {
        let first = held.first().and_then(|&k| leads[k].as_ref());
        first.is_some() && held.iter().all(|&k| leads[k].as_ref() == first)
      });
      let wrap = one_font
        .then(|| {
          let containers: Vec<((u16, u16), usize)> = mine
            .iter()
            .map(|(span, held)| {
              (
                *span,
                held.iter().map(|&k| entries[k].0).min().unwrap_or(from),
              )
            })
            .collect();
          let end = usize::from(end).checked_sub(base)?;
          container_wrap(content, base, &containers, from, end, events)
        })
        .flatten();
      // (the font where the outermost container starts: one the boxes cannot tell gives neither — the base's cut;
      // re-wrapped without it, `\itshape\emph{…\\…}`'s entries read italic where pdflatex prints them upright, review
      // rev94b r5 BL3)
      match wrap
        .and_then(|(wrap, outer)| Some((entry_font(content, base, outer, scope, events)?, wrap)))
      {
        Some((font, wrap)) => (Some(font), Some(Rc::new(wrap))),
        None => (None, None),
      }
    })
    .collect()
}

/// The tokens opening each container (`containers`: its span, outermost first, and its first entry's cut start) an
/// entry cut at `from..end` sits in, wrapped round it again ([`entry_forms`]), and where the outermost starts: each
/// container a command (`content[0]` at position `base`) taking its arguments as braced groups or `[…]` options, the
/// entry in one of them, with nothing switching a font between that argument's start and the container's first entry
/// (`events::switch_free`), and only switches between two containers' — `None` otherwise (a container made by a
/// macro of the document's, an environment, a box whose argument is no group).
fn container_wrap(
  content: &[Token],
  base: usize,
  containers: &[((u16, u16), usize)],
  from: usize,
  end: usize,
  events: &[Event],
) -> Option<(Wrap, usize)> {
  let close_of = |open: usize, (opening, closing): (Catcode, Option<Token>)| -> Option<usize> {
    let mut depth = 0i32;
    for (k, t) in content.iter().enumerate().skip(open) {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth < 0 {
            return None;
          }
          if depth == 0 && opening == Catcode::BEGIN {
            return Some(k);
          }
        },
        _ if depth == 0 && closing.is_some_and(|c| *t == c) => return Some(k),
        _ => {},
      }
    }
    None
  };
  let mut open = Vec::new();
  let mut outer = None;
  let mut after: Option<usize> = None;
  for (n, &((start, _), first)) in containers.iter().enumerate() {
    let at = usize::from(start).checked_sub(base)?;
    // (the innermost container's argument is switch-free up to its first entry; an outer one's holds the next
    // container, and before it nothing but font declarations — kept in the wrap, every entry printing under them, as
    // pdflatex prints every line: `\emph{\bfseries\underline{…\\…}}`, review rev94b r5 BL3 — and spaces, which the
    // wrap leaves out; anything else between prints or acts once, which a copy in every entry would repeat — a
    // `\newline`'s break, a `\quad`'s space: the base's cut, review rev94b r6 B2)
    let innermost = n + 1 == containers.len();
    if let Some(after) = after {
      let between = content.get(after..at)?;
      if !between
        .iter()
        .all(|t| t.get_catcode() == Catcode::SPACE || font_declaration(t))
      {
        return None;
      }
      open.extend(between.iter().filter(|t| t.get_catcode() != Catcode::SPACE));
    }
    if content.get(at)?.get_catcode() != Catcode::CS {
      return None;
    }
    let mut k = at + 1;
    let arg = loop {
      let t = content.get(k)?;
      match t.get_catcode() {
        Catcode::SPACE => k += 1,
        Catcode::BEGIN => {
          let close = close_of(k, (Catcode::BEGIN, None))?;
          if k < from && end <= close {
            break k;
          }
          k = close + 1;
        },
        _ if *t == T_OTHER!("[") => k = close_of(k + 1, (Catcode::OTHER, Some(T_OTHER!("]"))))? + 1,
        _ => return None,
      }
    };
    if innermost && !switch_free(content, base, arg + 1, first, events) {
      return None;
    }
    open.extend_from_slice(&content[at..=arg]);
    outer.get_or_insert(at);
    after = Some(arg + 1);
  }
  // (a kernel text font command whose argument opens with the outermost container — `\textbf{\emph{…\\…}}`: a
  // macro, `\bgroup\bfseries #1\egroup`, no whatsit holding the entries' ink — opens each entry again too, the font then
  // the one where it starts; pdflatex prints every line bold italic — review rev94b r6 S1)
  let mut outer = outer?;
  let mut closes = containers.len();
  loop {
    let mut b = outer;
    while b > 0 && content[b - 1].get_catcode() == Catcode::SPACE {
      b -= 1;
    }
    if b < 2
      || content[b - 1].get_catcode() != Catcode::BEGIN
      || !text_font_command(&content[b - 2])
      || close_of(b - 1, (Catcode::BEGIN, None)).is_none()
    {
      break;
    }
    open.splice(0..0, content[b - 2..b].iter().copied());
    closes += 1;
    outer = b - 2;
  }
  Some((Wrap { open, closes }, outer))
}

/// `tokens` digested under the font changes `font` ([`events::entry_font`]): `\lx@fm@entry@font{n}` before them, the
/// font registered as the `n`-th (`frontmatter_stores.rs`), when it changes anything.
pub(in crate::base_utilities::frontmatter::observer) fn entry_font_tokens(
  font: &Rc<Font>,
  tokens: Tokens,
) -> Tokens {
  if !changes_font(font) {
    return tokens;
  }
  let n = lookup_int("lx_fm_entry_fonts") + 1;
  assign_value("lx_fm_entry_fonts", Stored::Int(n), Some(Scope::Global));
  assign_value(
    &s!("lx_fm_entry_font_{n}"),
    Stored::Font(font.clone()),
    Some(Scope::Global),
  );
  let mut out = vec![T_CS!("\\lx@fm@entry@font"), T_BEGIN!()];
  out.extend(ExplodeText!(&n.to_string()));
  out.push(T_END!());
  out.extend(tokens.unlist());
  Tokens::new(out)
}

/// The text a text-symbol command prints: LaTeX's `\DeclareTextSymbol` glyph for the current encoding (`\OT1\i`: core
/// `mouth::text_symbol_char`, latex.ltx's `\i`, `\ss`, `\o`, `\ae`), else its meaning or its robust form's (`\i` is
/// `\protect\i␣`), a primitive whose body is that text (plain_base.rs; Perl plain_base.pool.ltxml:108, 531
/// `DefPrimitiveI`).
fn text_symbol(t: &Token) -> Option<String> {
  if let Some(c) = mouth::text_symbol_char(t) {
    return Some(c.to_string());
  }
  // (OT1's glyphs are `\chardef`s, `\OT1\i` slot 16: decoded through the encoding's font map, as the glyph prints)
  let encoding = lookup_font()?
    .get_encoding()
    .map_or_else(|| "OT1".to_string(), |e| e.to_string());
  let glyph = T_CS!(s!(
    "\\{}{}",
    encoding.trim(),
    t.with_cs_name(ToString::to_string)
  ));
  if let Some(defn) = lookup_register_definition(&glyph)
    && matches!(defn.register_type(), Some(RegisterType::CharDef))
    && let Some(code) = defn.value_of(Vec::new())
  {
    return font_decode(code.value_of() as i32, Some(encoding.trim()), None)
      .0
      .map(String::from);
  }
  let name = t.with_cs_name(ToString::to_string);
  [T_CS!(name.clone()), T_CS!(s!("{name} "))]
    .iter()
    .find_map(|cs| match lookup_meaning(cs) {
      Some(Stored::Primitive(p)) => match p.replacement {
        Some(PrimitiveBody::String(text)) => Some(with(text, ToString::to_string)),
        _ => None,
      },
      _ => None,
    })
}

/// [`block_span_tokens`] on a block's `field`: `mark` the span of the Mark event printed last before the entry,
/// `labelled` for an in-block contact's entry read from its leading marks, where those marks print
/// (`events::labelled_entries`, which checks with it at the reading that the writer can cut each entry), `own_font`
/// whether it carries the font its cut starts in (`events::entry_font`: no switch of a group it sits in is then its, the
/// font holds them).
pub(in crate::base_utilities::frontmatter::observer) fn span_tokens(
  field: &Field,
  (start, end): (u16, u16),
  key: Option<&str>,
  mark: Option<(u16, u16)>,
  labelled: Option<Option<(u16, u16)>>,
  with_mark: bool,
  own_font: bool,
) -> Option<Tokens> {
  let (content, base) = (field.tokens, field.base);
  let at = |p: u16| usize::from(p).checked_sub(base);
  let (mut from, to) = (at(start)?, at(end)?);
  // (the span's printed letters against the key's, accents folded on both sides: an accent command prints its
  // letter accented — `Ca\~nada`, the key's "cañada" —, a text symbol its own letter — `Astrof{\'\i}sica`, `\ss`, `\o`
  // —; 1009.1371, 2401.03603's `H\"ugel`)
  let letters: String = content
    .get(from..to)?
    .iter()
    .filter_map(|t| match t.get_catcode() {
      Catcode::LETTER | Catcode::OTHER => Some(t.with_str(ToString::to_string)),
      Catcode::CS => text_symbol(t),
      _ => None,
    })
    .collect();
  if key.is_some_and(|key| base_letters(&norm(&letters)) != base_letters(key)) {
    return None;
  }
  let mut floor = 0;
  // (where the group switches the entry keeps are read up to: its own start, or its mark's formula)
  let mut first = from;
  // (where the entry starts after its mark: past the mark's whole formula and the spaces after it — `$^{1}$Univ X`,
  // whose Mark spans its script —, every command between them the entry's, with the groups it takes,
  // but a line break or an alignment token: its switches (`$^1$\it Univ X`, `$^1$\small`, `$^1$\color{red}`,
  // `$^1$\textit{…}`: the font Layout event between them has no span; the made item began at its first Name and lost
  // the switch, review rev94a r5 u03), a strut, a skip, a box; `None` when a letter outside a group, a break or an
  // alignment token stands between — review rev94a r6 N3)
  let lead = |mark_end: usize| -> Option<usize> {
    let mark_end = field.formula_end(mark_end);
    let between = content.get(mark_end..from)?;
    let skip = between
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count();
    let mut depth = 0i32;
    between[skip..]
      .iter()
      .all(|t| match t.get_catcode() {
        Catcode::SPACE => true,
        Catcode::BEGIN => {
          depth += 1;
          true
        },
        Catcode::END => {
          depth -= 1;
          depth >= 0
        },
        Catcode::LETTER | Catcode::OTHER => depth > 0,
        Catcode::CS => {
          !["\\\\", "\\par", "\\newline", "\\cr", "\\crcr"]
            .iter()
            .any(|cs| is_cs(t, cs))
            && !ALIGNMENT_ONLY.iter().any(|cs| is_cs(t, cs))
        },
        _ => false,
      })
      .then_some(mark_end + skip)
  };
  let mut switched = None;
  if let Some((mark_start, mark_end)) = mark
    && let (Some(mark_start), Some(mark_end)) = (at(mark_start), at(mark_end))
    && let Some(lead) = lead(mark_end)
  {
    // (without the mark, nothing before the entry's ink is the entry's but its switches: the mark's closing shift
    // stays the mark's)
    if with_mark {
      from = mark_start;
    } else {
      if lead < from {
        switched = Some(lead);
      }
      floor = from;
    }
  } else if let Some(marks) = labelled {
    // (an in-block contact's entry read from its leading marks spans its own ink past them (`events::
    // labelled_entries`): its marks' closing shift before it is theirs — `\affil{$^1$Univ A\\ $^2$Univ B}`, review
    // rev94b B1 —, and the group and the switches it opens between them and its ink the entry's: `$^2${\itshape Univ
    // B}, City`, `$^2$\textbf{Univ B}` — review rev94b r2 B1: cut at its ink, the group was no balanced run and the
    // block's one contact stayed a nameless creator's)
    from = labelled_lead(content, from, marks.and_then(|(_, end)| at(end)));
    floor = from;
    // (and a group opened before its marks, with its switches: `{\itshape $^2$Univ B, City}` keeps the `\itshape` —
    // unless the entry carries its font, which holds them)
    first = if own_font {
      0
    } else {
      marks.and_then(|(start, _)| at(start)).unwrap_or(from)
    };
  }
  // (a group closing past the spaces after the entry: `{\it Department …, Japan }`, 2307.03384)
  let mut past = to;
  while content
    .get(past)
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    past += 1;
  }
  let piece = |from: usize, floor: usize, first: usize| {
    field
      .content_of((from, to), floor, first)
      .or_else(|| {
        (past > to)
          .then(|| field.content_of((from, past), floor, first))
          .flatten()
      })
      .and_then(|(tokens, _)| cell_tokens(content, from, &tokens))
  };
  // (from the entry's own switches when they make a balanced run, else from its ink)
  switched
    .and_then(|lead| piece(lead, lead, lead))
    .or_else(|| piece(from, floor, first))
}

/// Where an in-block contact's entry read from its leading marks starts in `content` ([`span_tokens`]): its ink at
/// `from`, or the group and switches it opens between its marks' end (`marks_end`) and its ink (`$^2${\itshape Univ B}`).
fn labelled_lead(content: &[Token], from: usize, marks_end: Option<usize>) -> usize {
  let Some(between) = marks_end.and_then(|end| content.get(end..from)) else {
    return from;
  };
  let opens_the_entry = |t: &Token| {
    matches!(
      t.get_catcode(),
      Catcode::CS | Catcode::BEGIN | Catcode::SPACE
    ) && !["\\\\", "\\newline", "\\par", "\\linebreak", "\\cr"]
      .iter()
      .any(|name| is_cs(t, name))
  };
  if !between.iter().all(opens_the_entry) {
    return from;
  }
  from - between.len()
    + between
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count()
}

/// The index in the block `field` an in-block contact's entry at `span` is cut from ([`labelled_lead`]), `marks` where
/// its leading marks print: the reading's [`events::entry_font`] reads the font at it.
pub(in crate::base_utilities::frontmatter::observer) fn labelled_start(
  field: &Field,
  span: (u16, u16),
  marks: Option<(u16, u16)>,
) -> Option<usize> {
  let from = usize::from(span.0).checked_sub(field.base)?;
  (from <= field.tokens.len()).then(|| {
    labelled_lead(
      field.tokens,
      from,
      marks.and_then(|(_, end)| usize::from(end).checked_sub(field.base)),
    )
  })
}

/// The environments whose `\\` ends a row of an alignment.
const ALIGNMENTS: &[&str] = &[
  "tabular",
  "tabular*",
  "tabularx",
  "tabulary",
  "array",
  "longtable",
  "supertabular",
];

/// What only an alignment reads: outside one, TeX's "Misplaced" errors (`\cr`, `\omit`, `\noalign`, …).
const ALIGNMENT_ONLY: &[&str] = &[
  "\\cr",
  "\\crcr",
  "\\omit",
  "\\span",
  "\\noalign",
  "\\multicolumn",
  "\\hline",
  "\\cline",
  "\\tabularnewline",
];

/// Whether `t` is what only an alignment reads: its tab, or a command of [`ALIGNMENT_ONLY`] — outside an alignment, a stray
/// one, an Error ([`cell_tokens`], `Field::content_cut`).
pub(super) fn reads_as_alignment(t: &Token) -> bool {
  t.get_catcode() == Catcode::ALIGN || ALIGNMENT_ONLY.iter().any(|cs| is_cs(t, cs))
}

/// Tokens cut from a block at `from` as an item digests them outside the block's alignment, cut at its cells: `None`
/// when they hold, outside any whole environment of their own, an alignment tab or a command only an alignment reads
/// ([`ALIGNMENT_ONLY`]) — made outside the alignment, the tab is a stray one, an Error, and the item one of two columns
/// (review rev94a r4 B2: `\author{\begin{tabular}{cc}Ann Able & Bob Baker\\ Univ X & Univ Y\end{tabular}}` made "Univ
/// X & Univ Y" one affiliation). Where the block's innermost environment or `\shortstack` open at `from` is an
/// alignment ([`ALIGNMENTS`]), an entry continued over the rows of its column is each row's tokens in a group of its
/// own, the row ends between them the line breaks they print (`$^2$Chinese Center …\\ \footnotesize\sl P. O. Box
/// …`, nucl-th0306055): a switch set in a cell stays the cell's, as the alignment's cells group them; a row end inside
/// a group of the tokens' own is no cut, so `None`.
fn cell_tokens(content: &[Token], from: usize, tokens: &Tokens) -> Option<Tokens> {
  // (an environment's name after `\begin`/`\end`: its braced letters)
  let env_name = |run: &[Token], at: usize| -> Option<String> {
    let mut i = at + 1;
    while run
      .get(i)
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      i += 1;
    }
    if run.get(i)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let mut name = String::new();
    for t in run.get(i + 1..)? {
      if t.get_catcode() == Catcode::END {
        return Some(name);
      }
      name.push_str(&t.with_str(ToString::to_string));
    }
    None
  };
  // (the containers open at `from`: each environment by name, a `\shortstack`'s argument as the group it opens)
  let mut open: Vec<(String, Option<i32>)> = Vec::new();
  let mut depth = 0i32;
  let mut stack_next = false;
  for (i, t) in content[..from.min(content.len())].iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => {
        depth += 1;
        if std::mem::take(&mut stack_next) {
          open.push(("shortstack".to_string(), Some(depth)));
        }
      },
      Catcode::END => {
        if open.last().is_some_and(|(_, d)| *d == Some(depth)) {
          open.pop();
        }
        depth -= 1;
      },
      Catcode::CS if is_cs(t, "\\shortstack") => stack_next = true,
      Catcode::CS if is_cs(t, "\\begin") => {
        if let Some(name) = env_name(content, i) {
          open.push((name, None));
        }
      },
      Catcode::CS if is_cs(t, "\\end") => {
        if let Some(name) = env_name(content, i)
          && let Some(at) = open.iter().rposition(|(n, d)| d.is_none() && *n == name)
        {
          open.truncate(at);
        }
      },
      _ => {},
    }
  }
  let in_rows = open
    .last()
    .is_some_and(|(name, _)| name == "shortstack" || ALIGNMENTS.contains(&name.as_str()));
  // (the tokens' own environments and groups: what is inside one whole is theirs)
  let run = tokens.unlist_ref();
  let (mut own, mut depth) = (0usize, 0i32);
  let mut row_ends: Vec<usize> = Vec::new();
  for (i, t) in run.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ if is_cs(t, "\\begin") && env_name(run, i).is_some() => own += 1,
      _ if is_cs(t, "\\end") && env_name(run, i).is_some() => own = own.saturating_sub(1),
      _ if own > 0 => {},
      _ if reads_as_alignment(t) => return None,
      _ if in_rows && is_cs(t, "\\\\") => {
        if depth != 0 {
          return None;
        }
        row_ends.push(i);
      },
      _ => {},
    }
  }
  if row_ends.is_empty() {
    return Some(tokens.clone());
  }
  // (each row a group, the row end and its star and optional skip between: `\\[2pt]`)
  let mut out = vec![T_BEGIN!()];
  let mut i = 0;
  while i < run.len() {
    if row_ends.contains(&i) {
      out.push(T_END!());
      out.push(run[i]);
      i += 1;
      while run
        .get(i)
        .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
      {
        i += 1;
      }
      // (the star and the skip read as `\@ifstar` and `\@ifnextchar[` read them, past the spaces before each — `\\*
      // [2pt]` —, and the spaces a cell opens with none of its text, as the column template's `\ignorespaces` takes
      // them: review rev94a r5 y01, t02, t12)
      let skip_spaces = |i: &mut usize| {
        while run
          .get(*i)
          .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
        {
          *i += 1;
        }
      };
      if run.get(i).is_some_and(|t| t.with_str(|s| s == "*")) {
        out.push(run[i]);
        i += 1;
        skip_spaces(&mut i);
      }
      if run.get(i).is_some_and(|t| t.with_str(|s| s == "[")) {
        while let Some(t) = run.get(i) {
          out.push(*t);
          i += 1;
          if t.with_str(|s| s == "]") {
            break;
          }
        }
        skip_spaces(&mut i);
      }
      out.push(T_BEGIN!());
      continue;
    }
    out.push(run[i]);
    i += 1;
  }
  out.push(T_END!());
  Some(Tokens::new(out))
}

/// Whom an address spells among `names` (`spells` of the address reader, `resolver::address_runs`): a local part alone.
pub(super) fn spells_one_of<'a>(names: &'a [&'a str]) -> impl Fn(&str) -> bool + 'a {
  move |local: &str| spelled_name(names, &s!("{local}@x.org")).is_some()
}
