//! The cut of an entry's class e-mail calls, which the frontmatter emitter writes each by its owner's label
//! ([`entry_email_cut`], [`shared_call_item`]), and the text rules it reads a call by ([`is_call_separator`],
//! [`is_whole_address`]); moved unchanged from `shared_row/` when the old row split (`share_row_contacts`, P27b) was
//! deleted, the emitter writing a flush that holds a class's row contact (OXIDIZED_DESIGN_DIVERGENCES #901).

use super::*;

/// The notes whose argument is no line of the contact: an e-mail in a `\thanks` stays in the note ([`email_calls_in`]).
const NOTE_COMMANDS: [&str; 3] = ["\\thanks", "\\footnote", "\\footnotetext"];

/// The separators that print nothing of their own before an e-mail call, which go with it: spaces, commas, semicolons,
/// a line break, spacing, a space factor's `\@` (`USA;\@ Email: \email{…}`, 2602.18622).
pub(in crate::base_utilities) fn is_call_separator(t: &Token) -> bool {
  matches!(t.get_catcode(), Catcode::SPACE)
    || *t == T_OTHER!(",")
    || *t == T_OTHER!(";")
    || *t == T_ACTIVE!('~')
    || (t.get_catcode() == Catcode::CS
      && t.with_str(|name| {
        matches!(
          name,
          "\\\\" | "\\newline" | "\\quad" | "\\qquad" | "\\ " | "\\enspace" | "\\hfill" | "\\@"
        )
      }))
}

/// The end of the spacing at `tokens[from..to]`'s start that prints nothing of its own: separators
/// ([`is_call_separator`]) and `\hspace{…}`, `\hspace*{…}`.
fn past_spacing(tokens: &[Token], mut from: usize, to: usize) -> usize {
  loop {
    if from < to && is_call_separator(&tokens[from]) {
      from += 1;
    } else if from < to && tokens[from] == T_CS!("\\hspace") {
      let mut arg = from + 1;
      if tokens[arg..to].first() == Some(&T_OTHER!("*")) {
        arg += 1;
      }
      if arg >= to || tokens[arg].get_catcode() != Catcode::BEGIN {
        return from;
      }
      from = skip_group(tokens, arg).min(to);
    } else {
      return from;
    }
  }
}

/// The argument of the note command at `tokens[at]` ([`NOTE_COMMANDS`]), within `..to`: where its group opens (past
/// the spaces and an optional `[…]`), and where it ends; `None` when `tokens[at]` is no note command.
fn note_argument(tokens: &[Token], at: usize, to: usize) -> Option<(usize, usize)> {
  let t = tokens.get(at)?;
  if t.get_catcode() != Catcode::CS || !t.with_str(|name| NOTE_COMMANDS.contains(&name)) {
    return None;
  }
  let mut next = at + 1;
  while tokens.get(next) == Some(&T_SPACE!()) {
    next += 1;
  }
  if tokens.get(next) == Some(&T_OTHER!("[")) {
    next = tokens[next..to]
      .iter()
      .position(|t| *t == T_OTHER!("]"))
      .map_or(to, |end| next + end + 1);
  }
  Some((next, skip_group(tokens, next).min(to)))
}

/// One e-mail call of an entry ([`email_calls_in`]): its span, the separators before it with it; where its
/// command stands; the command; its argument; whether it is written inside a note of the contact (`\thanks{Mail me at
/// \email{…}}`); the e-mail label printed before it, which its span takes ([`email_label_before`]).
struct EmailCall {
  start:   usize,
  at:      usize,
  end:     usize,
  cs:      Token,
  arg:     Vec<Token>,
  in_note: bool,
  label:   Option<Vec<Token>>,
}

/// The e-mail calls in `tokens[from..to]`: each class e-mail command ([`is_email_command`]) and its argument group, with
/// the separators before it ([`is_call_separator`]); a note's are taken apart (`in_note`) when `notes` is set, else
/// the note is passed over.
fn email_calls_in(
  tokens: &[Token],
  from: usize,
  to: usize,
  notes: bool,
  in_note: bool,
  calls: &mut Vec<EmailCall>,
) -> Result<()> {
  let mut at = from;
  while at < to {
    let t = tokens[at];
    if let Some((next, end)) = note_argument(tokens, at, to) {
      if notes && next < end && tokens[next].get_catcode() == Catcode::BEGIN {
        email_calls_in(tokens, next + 1, end - 1, notes, true, calls)?;
      }
      at = end;
      continue;
    }
    if t.get_catcode() == Catcode::CS && is_email_command(&t)? {
      let mut arg = at + 1;
      while tokens.get(arg) == Some(&T_SPACE!()) {
        arg += 1;
      }
      if arg < to && tokens[arg].get_catcode() == Catcode::BEGIN {
        let end = skip_group(tokens, arg).min(to);
        let mut start = at;
        while !in_note && start > from && is_call_separator(&tokens[start - 1]) {
          start -= 1;
        }
        // (and the label printed before it, which names the call's contacts: `Email: \email{…}`, 2102.12877)
        let mut label_tokens = None;
        if !in_note && let Some(label) = email_label_before(tokens, from, start) {
          label_tokens = Some(tokens[label..start].to_vec());
          start = label;
          while start > from && is_call_separator(&tokens[start - 1]) {
            start -= 1;
          }
        }
        calls.push(EmailCall {
          start,
          at,
          end,
          cs: t,
          arg: tokens[arg + 1..end - 1].to_vec(),
          in_note,
          label: label_tokens,
        });
        at = end;
        continue;
      }
    }
    at += 1;
  }
  Ok(())
}

/// An affiliation-kind entry whose class e-mail calls the frontmatter emitter places itself (the reading's inner
/// contacts, `Resolver::entry_emails`, OXIDIZED_DESIGN_DIVERGENCES #980), as the emitter writes it: its `command` with
/// each call's unit cut by the cut the old row split (`row_email_calls`, deleted at P27b) cut a row contact's by
/// ([`unit_cut`], [`kept_without`], each call read by [`read_call`]) — the separators before it back to the text or unit
/// before it (one at the content's start takes those after it: `\email{a}, Univ X`), the e-mail label it is named by
/// (`Email: \email{…}`, [`email_label_before`]), the label of its own the rule recognises ([`call_label`]: `Corresponding
/// author: \email{…}`, `\email{…} (Corresponding author)`) — so the entry keeps only its own text: `\institute{Univ X\\
/// \email{p1@x.edu}, \email{p2@x.edu}}` under `\inst{1}` marks is "Univ X", it had kept "Univ X ," (`…; Email: ;`,
/// `Unity Technologies Corresponding author:`). Each call stays where it was written, so it digests inside the entry and
/// the Link step places its addresses among the entry's carriers (`insert::link_entry_addresses`); a labelled unit, and
/// an unlabelled one after the first that reads as whole addresses, is written as the row's e-mail is, a label of its
/// own in its content or its name, by the entry's `label` (`\lx@row@email`, `\lx@row@email@named`) — a later call in printed
/// braces too (`\email{\{b2@x.edu\}}`, [`single_braced`]: it had stayed on the last author, review revZ18c S2); any other
/// as written.
/// With `only` (the entry's inner contacts written as items of their own, `\lx@frontmatter@itemonly`) the units are
/// cut out whole, a label of the call's aside, which stays the entry's text — with `shared` (a `_share=authors` entry,
/// llncs's sole `\institute`) the label goes with the call, its item carrying it ([`shared_call_item`]). A call inside
/// a note stays in it. `None` when the content holds no class e-mail call outside a note.
/// Witnesses (Z18b's llncs reroute, the units left in the entry): 1709.03776, 1709.08027, 2006.03363, 2008.02173,
/// 2009.11534, 2102.12877, 2207.10888, 2406.08136, 2407.18381, 2410.06985, 2501.02196, 2509.05139, 2510.06692,
/// 2609.00838, cs0606057.
pub(in crate::base_utilities) fn entry_email_cut(
  command: &Tokens,
  label: &str,
  only: bool,
  shared: bool,
) -> Result<Option<Tokens>> {
  let tokens = command.unlist_ref();
  let Some((open, close)) = content_group(tokens) else {
    return Ok(None);
  };
  let mut calls: Vec<EmailCall> = Vec::new();
  email_calls_in(tokens, open + 1, close, false, false, &mut calls)?;
  if calls.iter().all(|call| call.in_note) {
    return Ok(None);
  }
  let (inner_open, inner_close) = wrapped_span(tokens, open + 1, close);
  let lines = content_lines(tokens, inner_open, inner_close);
  let first_text = first_text_line(tokens, &lines);
  let label_tokens = || mouth::tokenize_internal(TeXString::assembled(label.to_string()));
  // (each unit: its span, and what it is written as)
  let mut units: Vec<(usize, usize, Vec<Token>)> = Vec::new();
  for (place, &(_, start, end)) in lines.iter().enumerate() {
    for call in calls
      .iter()
      .filter(|call| !call.in_note && (start..end).contains(&call.at))
    {
      let floor = units.last().map_or(inner_open, |unit| unit.1);
      let cut = unit_cut(tokens, call, (floor, start, end), Some(place) == first_text);
      // (an address the reader reads whole, no template's: the entry's to place, its call after the first written by the
      // entry's label, which it would not inherit — a call digested after another inside an entry takes no label
      // (Base_Utility.pool.ltxml:497-501 inherits from the last stub only), so it had stayed on the last author as their
      // own call: `\institute{Univ X\\ \email{a1@x.edu}, \email{b2@x.edu}, \email{c3@x.edu}}` gave Cid Cole b2@ and c3@)
      let reads_whole = read_call(call)?
        .is_some_and(|list| list.iter().all(|address| !is_template_address(address)));
      let written = if only {
        Vec::new()
      } else if !cut.labelled() && call.label.is_none() && (units.is_empty() || !reads_whole) {
        tokens[call.at..call.end].to_vec()
      } else {
        let whole = cut.labelled_address(tokens, call)?;
        match cut.name(call) {
          Some(name) => Invocation!(T_CS!("\\lx@row@email@named"), vec![
            Some(label_tokens()),
            Some(Tokens::new(name.clone())),
            Some(Tokens::new(whole))
          ])
          .unlist(),
          None => Invocation!(T_CS!("\\lx@row@email"), vec![
            Some(label_tokens()),
            Some(Tokens::new(whole))
          ])
          .unlist(),
        }
      };
      // (with `only`, the address written apart, its item named by no label of the entry's: a label stays in the entry,
      // the call alone leaving it — 2606.02634's `Norway\\ Correspondence: \email{elias.stenhede@ahus.no}`, whose address
      // is Elias Stenhede's in another entry; the label had gone with the call, a word lost)
      // — not a shared row entry's, whose labels go with the call to its item (`shared_call_item`), as the row's split
      // gave its unit to the owner or the block, never every author's (85b's `llncs_email_label_goes_with_the_call`, 104d)
      if only && !shared && (cut.labelled() || call.label.is_some()) {
        units.push((call.at, call.end, written));
        continue;
      }
      units.push((cut.start, cut.end, written));
    }
  }
  if units.is_empty() {
    return Ok(None);
  }
  // (a line break among the separators a unit takes stays where text the entry keeps comes after the unit: that text
  // stays on a line of its own — `Shanghai AI Laboratory \\ \email{…} \url{…}`, 2310.01324, whose link line the
  // entry's Kind step reads apart)
  let taken = |at: usize| {
    units
      .iter()
      .any(|(start, end, _)| (*start..*end).contains(&at))
  };
  let kept_after =
    |end: usize| (end..inner_close).any(|at| !taken(at) && !is_call_separator(&tokens[at]));
  let breaks: Vec<Option<Token>> = units
    .iter()
    .map(|(start, end, _)| {
      tokens[*start..*end]
        .iter()
        .find(|t| is_line_break(t))
        .copied()
        .filter(|_| kept_after(*end))
    })
    .collect();
  let edits: Vec<CutEdit> = units
    .into_iter()
    .zip(breaks)
    .map(|((start, end, written), line_break)| {
      (
        start,
        end,
        line_break.into_iter().chain(written).collect(),
        true,
      )
    })
    .collect();
  Ok(Some(Tokens::new(kept_without(
    tokens,
    edits,
    (inner_open, inner_close),
  ))))
}

/// A shared row entry's e-mail item as the row's split wrote its unit (`row_email_calls`' `call_unit`, deleted at
/// P27b; [`unit_cut`]): the e-mail label printed before the call, which names it, and the address as the class sets it with the label of its
/// own before and after it ([`UnitCut::labelled_address`]).
type SharedItem = (Option<String>, Vec<Token>);

/// The item of a shared row entry's e-mail call (llncs's sole `\institute`, `_share=authors`, its inner contacts written
/// as items of their own, [`entry_email_cut`]'s `only` with `shared`) carrying the labels [`entry_email_cut`] cuts from the
/// entry with the call, as the row's split gave its unit: the e-mail label printed before the call, its name
/// (`USA;\@ Email: \email{\{aable,bbaker,ccole\}@a.edu}`, each part named "Email: ", 104d; 2102.12877, 2602.18622), and
/// the label of its own the rule recognises ([`call_label`]: `Corresponding author email: \email{zz@x.edu}`, the 85b
/// re-review's s23; `Corresponding author:`, 2410.06985), in its content — to the address's owner or the block, never
/// every author's. `written` is the item's e-mail call as the emitter writes it (the call, or one address of its list);
/// the call is the one of `content` whose addresses hold `address` ([`read_call`]), else its only call. `None` when no
/// call is found, or it carries no label.
pub(in crate::base_utilities) fn shared_call_item(
  content: &Tokens,
  address: &str,
  written: &Tokens,
) -> Result<Option<SharedItem>> {
  let tokens = content.unlist_ref();
  let mut calls: Vec<EmailCall> = Vec::new();
  email_calls_in(tokens, 0, tokens.len(), false, false, &mut calls)?;
  calls.retain(|call| !call.in_note);
  let (inner_open, inner_close) = wrapped_span(tokens, 0, tokens.len());
  let lines = content_lines(tokens, inner_open, inner_close);
  let first_text = first_text_line(tokens, &lines);
  let wanted = address.trim().to_lowercase();
  let only_call = calls.len() == 1;
  let mut floor = inner_open;
  for (place, &(_, start, end)) in lines.iter().enumerate() {
    for call in calls.iter().filter(|call| (start..end).contains(&call.at)) {
      let cut = unit_cut(tokens, call, (floor, start, end), Some(place) == first_text);
      floor = cut.end;
      let holds =
        read_call(call)?.is_some_and(|list| list.iter().any(|a| a.to_lowercase() == wanted));
      if !holds && !only_call {
        continue;
      }
      let name = cut.name(call).map(|name| {
        let printed = s!("{}", Tokens::new(name.clone()).untex_string());
        printed.trim().to_string()
      });
      if name.is_none() && !cut.labelled() {
        return Ok(None);
      }
      // (the written call's own address, the one of its list the item is)
      let item_tokens = written.unlist_ref();
      let mut own: Vec<EmailCall> = Vec::new();
      email_calls_in(item_tokens, 0, item_tokens.len(), false, false, &mut own)?;
      let Some(item) = own.into_iter().next() else {
        return Ok(None);
      };
      return Ok(Some((name, cut.labelled_address(tokens, &item)?)));
    }
  }
  Ok(None)
}

/// The addresses a class e-mail call's reader reads from it, each whole ([`email_addresses`], [`is_whole_address`]); one
/// address in printed braces is that address, set as written (`\{fatima.iribu@gmail.com\}`, 2410.04091,
/// [`single_braced`]). `None` when the call is not read so. The entry's cut ([`entry_email_cut`]) and its item
/// ([`shared_call_item`]) read a call by it, as the row's split did (`row_email_calls`, deleted at P27b).
fn read_call(call: &EmailCall) -> Result<Option<Vec<String>>> {
  let read_tokens = |tokens: &[Token]| -> Result<Option<Vec<String>>> {
    Ok(
      email_addresses(&Invocation!(call.cs, vec![Some(Tokens::new(
        tokens.to_vec()
      ))]))
      .filter(|list| list.iter().all(|address| is_whole_address(address))),
    )
  };
  if let Some(list) = read_tokens(&call.arg)? {
    return Ok(Some(list));
  }
  Ok(match single_braced(&call.arg) {
    Some(inner) => read_tokens(inner)?.filter(|list| list.len() == 1),
    None => None,
  })
}

/// The first of a contact's `lines` holding any text ([`content_lines`]).
fn first_text_line(tokens: &[Token], lines: &[(usize, usize, usize)]) -> Option<usize> {
  lines.iter().position(|(_, start, end)| {
    tokens[*start..*end]
      .iter()
      .any(|t| t.get_catcode() != Catcode::SPACE)
  })
}

/// One e-mail call's unit cut ([`unit_cut`]): its span, the separators before it back to `floor` with it, and the
/// labels of its own the rule recognises ([`call_label`]).
struct UnitCut {
  start:  usize,
  end:    usize,
  before: Option<(usize, usize, usize)>,
  after:  Option<(usize, usize, usize)>,
}

/// The unit of the call `call` on the line `tokens[start..end]`, no earlier than `floor` (the unit before it): the call
/// with a label of its own before it (`Corresponding author: \email{…}`) or a parenthetical after it (`\email{…}
/// (Corresponding author),`, [`call_label`]), and the separators before it back to the text or unit before it (`Univ A
/// \\ \email{…}` leaves "Univ A"). The cut of the entry's ([`entry_email_cut`]) and
/// of its item ([`shared_call_item`]), as of the row's split (`row_email_calls`, deleted at P27b).
fn unit_cut(
  tokens: &[Token],
  call: &EmailCall,
  (floor, start, end): (usize, usize, usize),
  first: bool,
) -> UnitCut {
  let (before, after) = call_label(tokens, call, (floor, start, end), first);
  let mut first_taken = before.map_or(call.start, |(taken, ..)| taken.min(call.start));
  while first_taken > floor && is_call_separator(&tokens[first_taken - 1]) {
    first_taken -= 1;
  }
  UnitCut {
    start: first_taken.max(floor),
    end: after.map_or(call.end, |(_, _, end)| end),
    before,
    after,
  }
}

impl UnitCut {
  /// Does the unit hold a label of the call's own?
  fn labelled(&self) -> bool { self.before.is_some() || self.after.is_some() }

  /// The e-mail label printed before the call that names it (`Email:`), unless a label of its own before it is in its
  /// text already (`Corresponding author, Email:`).
  fn name<'c>(&self, call: &'c EmailCall) -> Option<&'c Vec<Token>> {
    call.label.as_ref().filter(|_| self.before.is_none())
  }

  /// The call's address as the class sets it ([`email_command_address`]), with its label of its own before it (a tie a
  /// space, as printed: `Corresponding~author:`) and its parenthetical after it.
  fn labelled_address(&self, tokens: &[Token], call: &EmailCall) -> Result<Vec<Token>> {
    let mut whole: Vec<Token> = Vec::new();
    if let Some((_, from, to)) = self.before {
      whole.extend(
        tokens[from..to]
          .iter()
          .map(|t| if *t == T_ACTIVE!('~') { T_SPACE!() } else { *t }),
      );
      whole.push(T_SPACE!());
    }
    whole.extend(email_command_address(&call.cs, Tokens::new(call.arg.clone()))?.unlist());
    if let Some((from, to, _)) = self.after {
      whole.push(T_SPACE!());
      whole.extend_from_slice(&tokens[from..to]);
    }
    Ok(whole)
  }
}

/// A span cut out of a contact ([`kept_without`]): its start and end, what is written in its place, and whether it is a
/// unit, which at the content's start takes the separators after it too.
type CutEdit = (usize, usize, Vec<Token>, bool);

/// The contact `tokens` with each of `edits` (in order) cut out and its tokens written in its place. A unit at the start
/// of the content `inner` (or right after a group's open) takes the separators after it as well: they may be the next
/// unit's own (`\email{a}, \email{b}`). The reassembly of the entry's cut ([`entry_email_cut`]), as of the
/// row's split (`row_email_calls`, deleted at P27b).
fn kept_without(
  tokens: &[Token],
  edits: Vec<CutEdit>,
  (inner_open, inner_close): (usize, usize),
) -> Vec<Token> {
  let mut kept: Vec<Token> = Vec::with_capacity(tokens.len());
  let mut from = 0;
  for (start, end, written, unit) in edits {
    kept.extend_from_slice(&tokens[from.min(start)..start]);
    from = end.max(from);
    let at_start = unit
      && (start <= inner_open
        || kept
          .last()
          .is_some_and(|t| t.get_catcode() == Catcode::BEGIN));
    kept.extend(written);
    if at_start {
      while from < inner_close && is_call_separator(&tokens[from]) {
        from += 1;
      }
    }
  }
  kept.extend_from_slice(&tokens[from..]);
  kept
}

/// The span of `tokens[from..to]` its lines are in: within the group of the one command that wraps the whole of it
/// (`\lx@affiliation@withinst{…}`, as `\lx@add@affiliation` gives an institute), spaces aside; else the span itself.
fn wrapped_span(tokens: &[Token], from: usize, to: usize) -> (usize, usize) {
  let (mut first, mut last) = (from, to);
  while first < last && tokens[first].get_catcode() == Catcode::SPACE {
    first += 1;
  }
  while last > first && tokens[last - 1].get_catcode() == Catcode::SPACE {
    last -= 1;
  }
  if first + 1 < last
    && tokens[first].get_catcode() == Catcode::CS
    && tokens[first + 1].get_catcode() == Catcode::BEGIN
    && skip_group(tokens, first + 1) == last
  {
    return wrapped_span(tokens, first + 2, last - 1);
  }
  (from, to)
}

/// Is `t` a line break of a contact's lines ([`content_lines`])?
fn is_line_break(t: &Token) -> bool {
  t.get_catcode() == Catcode::CS
    && t.with_str(|name| matches!(name, "\\\\" | "\\newline" | "\\par"))
}

/// The lines of `tokens[from..to]`, parted by a line break outside any group ([`is_line_break`], with `\\`'s `*` and
/// `[…]`): each line's start with the break before it, its own start, and its end. The lines cover the span.
fn content_lines(tokens: &[Token], from: usize, to: usize) -> Vec<(usize, usize, usize)> {
  let mut lines = Vec::new();
  let (mut sep, mut start, mut depth) = (from, from, 0usize);
  let mut at = from;
  while at < to {
    match tokens[at].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && is_line_break(&tokens[at]) => {
        lines.push((sep, start, at));
        sep = at;
        at += 1;
        if tokens[at..to].first() == Some(&T_OTHER!("*")) {
          at += 1;
        }
        if tokens[at..to].first() == Some(&T_OTHER!("[")) {
          at = tokens[at..to]
            .iter()
            .position(|t| *t == T_OTHER!("]"))
            .map_or(to, |end| at + end + 1);
        }
        start = at;
        continue;
      },
      _ => {},
    }
    at += 1;
  }
  lines.push((sep, start, to));
  lines
}

/// A call's label before it (where the spacing before it starts, and its span) and parenthetical after it (its span,
/// and where the punctuation after it ends), [`call_label`].
type CallLabel = (Option<(usize, usize, usize)>, Option<(usize, usize, usize)>);

/// The label of a line's own that goes with an e-mail call on the line `tokens[start..end]` ([`unit_cut`]), no
/// earlier than `floor` (the unit before it): a short label before it ending in a colon (`Corresponding author:`,
/// 2410.06985, 2609.00838; `Contact:`, 2509.05139), where the spacing before it starts and its span; and a
/// parenthetical of words after it (`(Corresponding author)`, 2608.24697; `(Univ Y)`, the r3 re-review's n23), its
/// span and where it ends with the punctuation after it. The label before it, read as printed (a font's group, a `~`,
/// a spacing command: `\textit{Corresponding author:}`, `Corresponding~author:`, `\hspace{1cm}Contact:`, the 85b r3
/// re-review's n14-n16), is one to four words of letters and hyphens, the first a contact's ([`names_a_contact`]:
/// corresponding, correspondence, contact, author, e-mail, mail, address), from the line's start — not the first
/// line's, the institute's own — or from a `,` or `;`, the spacing before it aside ([`past_spacing`]: the label is
/// set from its first word). Any other text beside a call is the line's, the institute's as before: `… F-76000 Rouen,
/// France. \email{…}` (2609.24668), `\hspace{0.5cm}` (2310.06847), and text that only ends in a colon (`Department of
/// Computer Science:`, `Dept. of CS:`, `University of X, Robotics Lab:`, `Lab of A. B. Smith:`: the r3 re-review's n09,
/// n10, n12, n35). So only the label forms the rule recognises go with the call; a label of another form (prose
/// without a colon, `Corresponding author email \email{…}`; a label word the rule does not know, `Reach me at:
/// \email{…}`; one after a `.`, or after other text on its line) stays in the shared institute, every author's — a
/// residual.
fn call_label(
  tokens: &[Token],
  call: &EmailCall,
  (floor, start, end): (usize, usize, usize),
  first: bool,
) -> CallLabel {
  let floor = floor.max(start);
  let is_word = |t: &Token| t.get_catcode() == Catcode::LETTER || *t == T_OTHER!("-");
  // (words of a label, as printed: letters, hyphens and spaces, the first naming a contact)
  let is_label = |printed: &str| {
    let words: Vec<&str> = printed.split_whitespace().collect();
    (1..=4).contains(&words.len())
      && names_a_contact(words[0])
      && printed
        .chars()
        .all(|c| c.is_alphabetic() || c == '-' || c.is_whitespace())
  };
  let mut before = None;
  // (the piece before the call from the line's start — not the first line's, the institute's own — or from the last
  // `,` or `;` outside any group, the other calls of the line aside, read as printed: through a font's group, a `~`, a
  // spacing command, `\textit{Corresponding author:}`, `Corresponding~author:`, `\hspace{1cm}Contact:`)
  let mut colon = call.at;
  while colon > floor && tokens[colon - 1] == T_SPACE!() {
    colon -= 1;
  }
  let (mut from, mut depth) = (None, 0usize);
  for at in (floor..colon).rev() {
    match tokens[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => depth = depth.saturating_sub(1),
      _ if depth == 0 && [T_OTHER!(","), T_OTHER!(";")].contains(&tokens[at]) => {
        from = Some(at + 1);
        break;
      },
      _ => {},
    }
  }
  let from = from.or((floor == start && !first).then_some(start));
  if let Some(taken) = from {
    // (the label without the spacing before it, which prints nothing of its own: `\hspace{1cm}Contact:`)
    let from = past_spacing(tokens, taken, colon);
    let printed = visible_name_text(&tokens[from..colon]);
    // (an `Email:` label alone is the contact's own name, which the call's start already takes, [`email_label_before`])
    if let Some(words) = printed.trim_end().strip_suffix(':')
      && is_label(words)
      && call.start > from
    {
      before = Some((taken, from, colon));
    }
  }
  let mut open = call.end;
  while open < end && tokens[open] == T_SPACE!() {
    open += 1;
  }
  let mut after = None;
  if open < end && tokens[open] == T_OTHER!("(") {
    let close = (open + 1..end).find(|at| tokens[*at] == T_OTHER!(")"));
    if let Some(close) = close
      && tokens[open + 1..close]
        .iter()
        .all(|t| is_word(t) || *t == T_SPACE!())
      && tokens[open + 1..close].iter().any(is_word)
    {
      let mut past = close + 1;
      while past < end && [T_OTHER!(","), T_OTHER!(";"), T_OTHER!(".")].contains(&tokens[past]) {
        past += 1;
      }
      after = Some((open, close + 1, past));
    }
  }
  (before, after)
}

/// Is `address` a template's, the address form a paper gives no author (`{firstname.lastname}@x.edu`,
/// `first.last@x.edu`, `name.surname@x.edu`; the 85b re-review's s08, s09; 2102.12877, 2109.05583)?
fn is_template_address(address: &str) -> bool {
  let local = address
    .split_once('@')
    .map_or(address, |(local, _)| local)
    .to_lowercase();
  let words: Vec<&str> = local
    .split(|c: char| !c.is_alphanumeric())
    .filter(|word| !word.is_empty())
    .collect();
  words.iter().any(|word| {
    [
      "firstname",
      "lastname",
      "forename",
      "surname",
      "givenname",
      "familyname",
    ]
    .contains(word)
  }) || words == ["first", "last"]
}

/// Whether `address`, read from an e-mail call ([`email_addresses`]), is an address whole: a local part and a domain of
/// labels parted by dots, nothing of the line around it — not a piece of a list the reader cut at its commas
/// (`[simon.barbarit--gaboriau@insa-, …, samia.ainouz@insa-]rouen.fr`, 2609.24668), which the call keeps as written.
pub(in crate::base_utilities) fn is_whole_address(address: &str) -> bool {
  let Some((local, domain)) = address.rsplit_once('@') else {
    return false;
  };
  !local.is_empty()
    && !local.contains(['[', ']', '(', ')', '<', '>', '@'])
    && domain.contains('.')
    && domain.split('.').all(|label| {
      !label.is_empty()
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label.chars().all(|c| c.is_alphanumeric() || c == '-')
    })
}

/// The tokens inside the printed braces of `arg` when they are the whole of it (`\{fatima.iribu@gmail.com\}`, spaces
/// aside), and no other printed brace stands within.
fn single_braced(arg: &[Token]) -> Option<&[Token]> {
  let is = |t: &Token, names: [&str; 2]| {
    t.get_catcode() == Catcode::CS && t.with_str(|name| names.contains(&name))
  };
  let open = ["\\{", "\\lbrace"];
  let close = ["\\}", "\\rbrace"];
  let first = arg.iter().position(|t| t.get_catcode() != Catcode::SPACE)?;
  let last = arg
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)?;
  if first >= last || !is(&arg[first], open) || !is(&arg[last], close) {
    return None;
  }
  let inner = &arg[first + 1..last];
  (!inner.iter().any(|t| is(t, open) || is(t, close))).then_some(inner)
}

/// The start of an e-mail label ending at `tokens[..end]` (`Email:`, `E-mail:`, `Emails:`, the colon after the word),
/// no earlier than `from`, printed before an e-mail call it names, which the call's own contact name says.
fn email_label_before(tokens: &[Token], from: usize, end: usize) -> Option<usize> {
  if end <= from || tokens[end - 1] != T_OTHER!(":") {
    return None;
  }
  let mut start = end - 1;
  while start > from
    && (tokens[start - 1].get_catcode() == Catcode::LETTER || tokens[start - 1] == T_OTHER!("-"))
  {
    start -= 1;
  }
  let word: String = tokens[start..end - 1]
    .iter()
    .map(Token::to_string)
    .collect();
  (start < end - 1 && is_email_label(&word)).then_some(start)
}

/// The span of a queued command's content, its last group (`\lx@add@frontmatter@now{ltx:creator}[role=author]{…}`,
/// `\lx@annotate@frontmatter@now{ltx:creator}{ltx:contact}[…]{…}`): the places of its braces.
fn content_group(tokens: &[Token]) -> Option<(usize, usize)> {
  let close = tokens.len().checked_sub(1)?;
  if tokens[close].get_catcode() != Catcode::END {
    return None;
  }
  let mut depth = 0usize;
  for at in (0..=close).rev() {
    match tokens[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => {
        depth -= 1;
        if depth == 0 {
          return Some((at, close));
        }
      },
      _ => {},
    }
  }
  None
}
