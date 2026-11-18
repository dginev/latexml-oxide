//! A `\date` the class prints under the author block whose lines are the block's legend (OXIDIZED_DESIGN_DIVERGENCES
//! #1071): `\date{$^1$Dept A, Univ X\\ $^2$Dept B, Univ Y}` under `Ann Able$^1$ \and Bob Baker$^2$` (1512.01686,
//! 2108.10662, 1802.01334, 2305.08521, 2511.18126; 2606.00446's authblk `\date{\textsuperscript{\S}Email: …}`). Read at
//! `\maketitle` (`\lx@date@legend`, after `\lx@author@tail`), before the flush, as the author block's own lines are: each
//! line a label of the block's names leads is the contact that label's line is, an affiliation line no label leads
//! every author's, an e-mail line its addresses' owners'. A date anything is taken from is void, its leftover dropped
//! (`Info:frontmatter:date_void`, DG 2026-11-17); one with a line no rule reads stays whole. Perl: the text is the date.

use super::*;

/// A run of lines one contact call takes: its label (as printed, and the spelling a declared label is keyed by), none
/// for an unlabelled affiliation, and its lines.
type Run = (Option<(String, Option<String>)>, Vec<Token>);

/// What one line of the date is.
#[derive(Debug, Clone, PartialEq)]
enum Line {
  /// prints nothing
  Blank,
  /// a date's: `\today`, a month's name, a year on a line naming no institution
  Date,
  /// led by a label the names request: the label as printed, the spelling a binding's declared label is keyed by, the
  /// line without its label
  Labelled(String, Option<String>, Vec<Token>),
  /// e-mail addresses alone, an e-mail label before them aside: the label, the addresses
  Addresses(Option<Tokens>, Vec<String>),
  /// text no label leads, no mark inside it: whether it names an institution ([`reads_as_affiliation`])
  Text(bool),
  /// anything else — a label no name requests, a mark inside the line: the reason, for the Info that the date stays
  Kept(&'static str),
}

/// `\lx@date@legend`: the calls a date the class prints under the author block makes of its legend lines, the date
/// itself taken off the queue, void (its date lines dropped, named in an Info). Nothing where the class prints the date
/// elsewhere or not at all ([`fm_class_prints_date_under_authors`]), under a title page of the document's or a raw
/// file's own, with no author queued, where no line is a legend's, or where a line is read by no rule; then the date is
/// untouched.
pub(in crate::base_utilities) fn date_legend_calls() -> Result<Vec<Token>> {
  if !fm_class_prints_date_under_authors()
    || crate::latex_constructs::own_maketitle()
    || crate::latex_constructs::replays_class_maketitle()
  {
    return Ok(Vec::new());
  }
  let Some(store) = crate::frontmatter_stores::store_value("date")? else {
    return Ok(Vec::new());
  };
  // (the kernel `\date`'s entry, its content the store's value: `\lx@add@date@halved`)
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.iter().rposition(|entry| {
      entry.0 == "ltx:date"
        && entry.1.get("role").map(String::as_str) == Some("creation")
        && last_group(&entry.2) == store
    }),
    _ => None,
  });
  let Some(at) = queued else {
    return Ok(Vec::new());
  };
  let names = queued_position_names()?;
  if names.is_empty() {
    return Ok(Vec::new());
  }
  let (prefix, body) = unwrapped(strip_linebreak_options(store.clone()).unlist());
  let body = wrapped_lines_as_lines(&Tokens::new(body.clone())).map_or(body, Tokens::unlist);
  let lines = date_lines(&body);
  let declared = policy::unexplained_name_label_spellings();
  let mut kinds = Vec::with_capacity(lines.len());
  for (_, line) in &lines {
    kinds.push(line_kind(line, &declared)?);
  }
  if !kinds.iter().any(|kind| {
    matches!(
      kind,
      Line::Labelled(..) | Line::Addresses(..) | Line::Text(true)
    )
  }) {
    return Ok(Vec::new());
  }
  // Fail-safes: the date stays whole (DG 2026-11-17: all or nothing) where a line's owners may not be the names it
  // reads them as. (1) A binding numbering its unlabelled names (`thenote`: authblk.sty:115-117's `\AB@thenote`, the
  // `affil` counter) gives a typed label's line to every name its number is, typing that mark or not (2307.00092,
  // 2412.00057, 2404.15649): a line a number labels is read only where every name requests it, its owners then exactly
  // them (2208.04109's `$^{1,2}$`); a binding's declared spelling is keyed apart. (2) A date with a date's line whose only
  // other lines are unlabelled text holds a remark, not a legend (2601.19933's "Part of the … research program",
  // cs0703003's report number).
  let mut numbered = false;
  if policy::policy_for(
    &ledger::policy_binding(&ledger::author_binding()),
    "\\lx@add@creator",
  )
  .thenote
  {
    for kind in &kinds {
      if let Line::Labelled(printed, None, _) = kind {
        let keys = printed_label_keys(printed, "affiliation");
        // (the number is `\arabic{affil}`: a symbol label, `*`, is no name's by number)
        let is_number = |key: &String| {
          key
            .strip_prefix("affiliation:")
            .is_some_and(|label| !label.is_empty() && label.chars().all(|c| c.is_ascii_digit()))
        };
        // (the names whose typed marks request it, by the numbers they type — the queue's requests hold the binding's
        // numbers too, and a queued entry may hold several names, `Ann Able$^{1}$, Bob Baker$^{2}$`: a position is the
        // line's only where every number it types is one of the line's, and it types one)
        if keys.iter().any(is_number) {
          numbered |= queued_author_marks()?.iter().any(|marks| {
            let numbers: Vec<Vec<String>> = marks
              .iter()
              .map(|mark| {
                requested_label_keys(mark)
                  .into_iter()
                  .filter(is_number)
                  .collect()
              })
              .filter(|numbers: &Vec<String>| !numbers.is_empty())
              .collect();
            numbers.is_empty()
              || numbers
                .iter()
                .any(|numbers| !numbers.iter().any(|key| keys.contains(key)))
          });
        }
      }
    }
  }
  let remark = kinds.contains(&Line::Date)
    && !kinds
      .iter()
      .any(|kind| matches!(kind, Line::Labelled(..) | Line::Addresses(..)));
  if numbered || remark {
    let why = if numbered {
      "a typed label where the binding numbers the names"
    } else {
      "a date with unlabelled text"
    };
    Info!(
      "frontmatter",
      "date",
      s!("A \\date under the authors stays the date ({why}): {store}")
    );
    return Ok(Vec::new());
  }
  // The runs: a labelled line with the unlabelled text lines after it (its address continued, as an author block's
  // marked line is, `entry_calls`), an affiliation line with those after it; the lines kept, each with its delimiter.
  let keys: Vec<NameKeys> = names
    .iter()
    .map(|name| printed_name_keys(name, None))
    .collect::<Result<_>>()?;
  let mut calls: Vec<Token> = Vec::new();
  let mut void: Vec<Token> = Vec::new();
  let mut run: Option<Run> = None;
  let mut explained: Vec<String> = Vec::new();
  let flush = |run: &mut Option<Run>, calls: &mut Vec<Token>| -> Result<()> {
    if let Some((label, content)) = run.take() {
      calls.extend(contact_call(label.as_ref(), &prefix, content)?);
    }
    Ok(())
  };
  // (the lines no rule reads with confidence: one makes the whole date stay as written — no part of it is taken, the
  // rest kept as a date, DG 2026-11-17)
  let mut in_run = false;
  for kind in &kinds {
    let why = match kind {
      Line::Kept(why) => Some(*why),
      Line::Text(false) if !in_run => Some("no label, no institution"),
      _ => None,
    };
    if let Some(why) = why {
      Info!(
        "frontmatter",
        "date",
        s!("A \\date under the authors stays the date, a line of it unread ({why}): {store}")
      );
      return Ok(Vec::new());
    }
    in_run = match kind {
      Line::Labelled(..) | Line::Text(true) => true,
      Line::Text(false) | Line::Blank => in_run,
      _ => false,
    };
  }
  for ((delimiter, line), kind) in lines.into_iter().zip(kinds) {
    match kind {
      Line::Blank => {
        if let Some((_, content)) = run.as_mut() {
          content.extend(delimiter);
          content.extend(line);
        }
      },
      Line::Labelled(label, spelling, rest) => {
        flush(&mut run, &mut calls)?;
        if spelling.is_some() {
          explained.push(label.clone());
        }
        run = Some((Some((label, spelling)), rest));
      },
      Line::Text(_) if run.is_some() => {
        if let Some((_, content)) = run.as_mut() {
          content.extend(delimiter);
          content.extend(line);
        }
      },
      Line::Addresses(label, addresses) => {
        flush(&mut run, &mut calls)?;
        calls.extend(address_calls(label.as_ref(), &addresses, &keys)?);
      },
      Line::Text(_) => run = Some((None, line)),
      Line::Date | Line::Kept(_) => {
        flush(&mut run, &mut calls)?;
        if !void.is_empty() {
          void.push(T_SPACE!());
        }
        void.extend(line.into_iter().skip_while(|t| *t == T_SPACE!()));
      },
    }
  }
  flush(&mut run, &mut calls)?;
  if !explained.is_empty() {
    policy::explain_name_labels(&explained);
  }
  // (a date its structured lines were taken from is void, DG 2026-11-17: no date, its leftover — a `\today`, a date
  // string — dropped, named once)
  if !prints_nothing(&Tokens::new(void.clone())) {
    // (named here, so the title-page check counts none of its words again: a ruled discard, no loss)
    let words: Vec<String> = net_content_words(&visible_name_text(&void));
    note_net_counted(&words);
    Info!("frontmatter", "date_void", s!("{}", Tokens::new(void)));
  }
  with_value_mut("frontmatter_raw", |v| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = v {
      queue.remove(at);
    }
  });
  DebugFeature!(
    "frontmatter",
    "FRONT the date's legend lines: {}",
    Tokens::new(calls.clone())
  );
  Ok(calls)
}

/// `tokens` without the group or size environment around all of them, and the switches that open them, which are given
/// back first (`{\small \it $^1$Univ A \\ $^2$Univ B}`, `\footnotesize $^1$…`).
fn unwrapped(mut v: Vec<Token>) -> (Vec<Token>, Vec<Token>) {
  let mut prefix: Vec<Token> = Vec::new();
  loop {
    while v.first() == Some(&T_SPACE!()) {
      v.remove(0);
    }
    while v.last() == Some(&T_SPACE!()) {
      v.pop();
    }
    if v.first().is_some_and(|t| t.get_catcode() == Catcode::BEGIN) && skip_group(&v, 0) == v.len()
    {
      v = v[1..v.len() - 1].to_vec();
      continue;
    }
    let lead = leading_unprinted_end(&v);
    if lead == 0 {
      break;
    }
    prefix.extend(lead_declarations(&v[..lead]));
    v = v[lead..].to_vec();
  }
  (prefix, v)
}

/// The date's lines, each with the line break before it: parted at its top-level `\\` (its `*` too), `\newline`, `\par`,
/// `\break` and `\linebreak` (1801.04802's `\break`), the breaks LaTeX prints there, and at the `\quad`s and `\qquad`s
/// an affiliation list's entries are parted at (`author_affil_splits`: 2606.00446's shape, `\textsuperscript{§}Email: …
/// \quad $^\dagger$Now at Lab B`; 2503.06162's two addresses).
fn date_lines(v: &[Token]) -> Vec<(Vec<Token>, Vec<Token>)> {
  let breaks = [
    T_CS!("\\\\"),
    T_CS!("\\newline"),
    T_CS!("\\par"),
    T_CS!("\\break"),
    T_CS!("\\linebreak"),
    T_CS!("\\quad"),
    T_CS!("\\qquad"),
  ];
  let mut lines: Vec<(Vec<Token>, Vec<Token>)> = vec![(Vec::new(), Vec::new())];
  let mut depth = 0i32;
  let mut at = 0;
  while at < v.len() {
    let t = v[at];
    depth += t.group_step();
    if depth == 0 && breaks.contains(&t) {
      let mut delimiter = vec![t];
      if t == T_CS!("\\\\") && v.get(at + 1) == Some(&T_OTHER!("*")) {
        at += 1;
        delimiter.push(v[at]);
      }
      lines.push((delimiter, Vec::new()));
    } else if let Some(last) = lines.last_mut() {
      last.1.push(t);
    }
    at += 1;
  }
  lines
}

/// The full English month names a date line prints (and "Sept"), whole words.
const MONTHS: [&str; 13] = [
  "january",
  "february",
  "march",
  "april",
  "may",
  "june",
  "july",
  "august",
  "september",
  "october",
  "november",
  "december",
  "sept",
];

/// What `line` is ([`Line`]). `declared`: the labels a binding's names print that no line of its own explains, as
/// (spelling, printed mark).
fn line_kind(line: &[Token], declared: &[(String, String)]) -> Result<Line> {
  let tokens = Tokens::new(line.to_vec());
  if prints_nothing(&tokens) {
    return Ok(Line::Blank);
  }
  if line.iter().any(|t| *t == T_CS!("\\today")) {
    return Ok(Line::Date);
  }
  let text = visible_name_text(line);
  let words: Vec<String> = text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect();
  if words.iter().any(|w| MONTHS.contains(&w.as_str())) {
    return Ok(Line::Date);
  }
  // (a label the line leads with — a superscript, a mark command, a footnote symbol, a number typed by hand:
  // `leading_line_label`, the affiliation lines' one reader)
  let lead = leading_unprinted_end(line);
  let led = Tokens::new(line[lead..].to_vec());
  if let Some((label, typed, rest)) = leading_line_label(&led) {
    // (a number typed by hand, `1) Univ A`, is its label as printed: ruling R2, a label where the names request it)
    let operand = if typed {
      Some(ExplodeText!(label.as_str()))
    } else {
      leading_operand(&led)
    };
    let printed = match &operand {
      Some(operand) => match printed_label(&Tokens::new(operand.clone()))? {
        Some(printed) => printed.text,
        None => mark_spelling_printed(&Tokens::new(operand.clone()).to_string())
          .trim()
          .to_string(),
      },
      None => String::new(),
    };
    if printed.is_empty() {
      return Ok(Line::Kept("a label no reader reads"));
    }
    if let Some((spelling, _)) = declared.iter().find(|(_, mark)| *mark == printed) {
      // (keyed by its spelling, as the binding's own line is, only a spelling of symbols — `\S`, `$\dagger$` —, which
      // the line keys alike: a document's macro, `\author[1,\myD]`, keys no line of its own either, so the line stays)
      let spelled: Vec<Token> = mouth::tokenize_internal(TeXString::assembled(spelling.clone()))
        .unlist()
        .into_iter()
        .filter(|t| t.get_catcode() != Catcode::MATH)
        .collect();
      return Ok(
        if is_footnote_symbol_operand(&spelled)
          || !spelled.iter().any(|t| t.get_catcode() == Catcode::CS)
        {
          Line::Labelled(printed, Some(spelling.clone()), rest.unlist())
        } else {
          Line::Kept("a label its binding keys by a macro")
        },
      );
    }
    // (keyed as a name's request is, by what it prints: `queue_reading::author_marks`)
    let keys = printed_label_keys(&printed, "affiliation");
    let mut requested = !keys.is_empty();
    for key in &keys {
      requested &= !queued_label_requesters(key)?.is_empty();
    }
    return Ok(if requested {
      Line::Labelled(printed, None, rest.unlist())
    } else {
      Line::Kept("a label no name requests")
    });
  }
  if let Some(symbol) = leading_symbol(line) {
    let printed = match printed_label(&Tokens::new(vec![symbol]))? {
      Some(printed) => printed.text,
      None => symbol.to_string(),
    };
    let keys = printed_label_keys(&printed, "affiliation");
    let mut requested = !keys.is_empty();
    for key in &keys {
      requested &= !queued_label_requesters(key)?.is_empty();
    }
    return Ok(if requested {
      let at = line.iter().position(|t| *t == symbol).unwrap_or(0);
      Line::Labelled(printed, None, line[at + 1..].to_vec())
    } else {
      Line::Kept("a label no name requests")
    });
  }
  if author_mark_position(&tokens).is_some() {
    return Ok(Line::Kept("a mark inside the line"));
  }
  let (label, rest) = match email_line_label(&tokens) {
    Some((label, rest)) => (Some(label), rest),
    None => (None, tokens.clone()),
  };
  if let Some(addresses) = email_addresses(&rest) {
    return Ok(Line::Addresses(label, addresses));
  }
  // (an address among text that is no e-mail label, `\textbf{@} gaborit@kth.se` (1801.04802), `bartosz@\{cs.uni.wroc.pl,
  // tu-dresden.de\}` (2108.05680): a run would take both as its affiliation's, which cuts only the addresses; a wrapped
  // address line, `\small{Emails: a@x, b@y}` (1601.01378), continues its run, whose contact cuts it)
  let addresses = resolver::addresses_in(&text);
  // (an `@` the line prints that no address holds: the readers' text drops a styled one, so counted on the tokens)
  let at_signs = line
    .iter()
    .filter(|t| matches!(t.get_catcode(), Catcode::OTHER | Catcode::LETTER) && t.to_string() == "@")
    .count();
  if !addresses.is_empty() && (!resolver::only_addresses(&text) || at_signs > addresses.len()) {
    return Ok(Line::Kept("an address among other text"));
  }
  let affiliation = reads_as_affiliation(&tokens);
  // (a year on a line naming no institution is a date's)
  if !affiliation
    && words
      .iter()
      .any(|w| w.len() == 4 && matches!(w.parse::<u32>(), Ok(1800..=2100)))
  {
    return Ok(Line::Date);
  }
  Ok(Line::Text(affiliation))
}

/// The operand of the superscript or mark command `line` leads with (`$^{1}$`, `\textsuperscript{\S}`, `${}^{1}$`).
fn leading_operand(line: &Tokens) -> Option<Vec<Token>> {
  let v = line.unlist_ref();
  let at = v.iter().position(|t| *t != T_SPACE!() && !mark_switch(t))?;
  if let Some((operand, _)) = spaced_mark_at(v, at) {
    return Some(operand);
  }
  let start = if v[at].get_catcode() == Catcode::MATH && v.get(at + 1) == Some(&T_SUPER!()) {
    at + 2
  } else if is_mark_command(&v[at]) {
    at + 1
  } else {
    return None;
  };
  let start = start + v[start..].iter().take_while(|t| **t == T_SPACE!()).count();
  let end = skip_group(v, start).min(v.len());
  let operand = &v[start..end];
  Some(
    if operand
      .first()
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && operand.len() >= 2
    {
      operand[1..operand.len() - 1].to_vec()
    } else {
      operand.to_vec()
    },
  )
}

/// The footnote symbol `line` leads with in text (`* Corresponding author`, `\dag Univ B`), past spaces, math shifts
/// and braces.
fn leading_symbol(line: &[Token]) -> Option<Token> {
  line
    .iter()
    .find(|t| {
      !matches!(
        t.get_catcode(),
        Catcode::SPACE | Catcode::MATH | Catcode::BEGIN | Catcode::END
      )
    })
    .copied()
    .filter(|t| is_footnote_symbol_operand(std::slice::from_ref(t)))
}

/// The contact call of a run: a labelled line (with the lines continuing it) the contact its label's request is answered
/// by, authblk's `\affil[label]` call (`annotate=1,label={…},_labelprints=true`: Perl's labelled contact, which
/// `relocateAnnotations` pairs with the names requesting the label; the label printed with the line, which the Kind step
/// reads a legend by, `insert::kind_legend_contacts`); an affiliation no label leads every author's (`annotate=all`), as the
/// API places a line printed under the whole block. Each in the switches the date's lines were set in.
fn contact_call(
  label: Option<&(String, Option<String>)>,
  prefix: &[Token],
  content: Vec<Token>,
) -> Result<Vec<Token>> {
  let keys = match label {
    // (a binding's declared label keyed by its spelling in the call, as the binding's own line is: authblk's
    // `\affil[\S]`; a typed mark by what it prints, as the names' requests are)
    Some((printed, spelling)) => s!(
      "role=affiliation,annotate=1,label={{{}}},_labelprints=true",
      spelling.as_deref().unwrap_or(printed)
    ),
    None => s!("role=affiliation,annotate=all"),
  };
  let mut body = prefix.to_vec();
  body.extend(content);
  Ok(
    Invocation!(T_CS!("\\lx@add@contact"), vec![
      Some(mouth::tokenize_internal(TeXString::assembled(keys))),
      Some(Tokens::new(body))
    ])
    .unlist(),
  )
}

/// The state value holding the last `\address` block a class prints under the names (`\lx@address@block`).
const ADDRESS_BLOCK: &str = "lx_address_block";

/// Records `block`, the `\address` block of spconf.sty:165 / INTERSPEECH2021.sty:162-164 / Interspeech.cls :265, whose
/// note `\lx@address@block` queues whole; read at `\maketitle` by [`address_block_legend_calls`] (OD #1073).
pub(in crate::base_utilities) fn note_address_block(block: Tokens) {
  assign_value(ADDRESS_BLOCK, Stored::Tokens(block), Some(Scope::Global));
}

/// `\lx@date@legend`'s part for the recorded `\address` block (OD #1073): its queued note replaced by the calls
/// [`address_block_calls`] gives, when every line is placed; else the note stays, whole, with an Info.
pub(in crate::base_utilities) fn address_block_legend_calls() -> Result<Vec<Token>> {
  let block = match lookup_value(ADDRESS_BLOCK) {
    Some(Stored::Tokens(block)) => block,
    _ => return Ok(Vec::new()),
  };
  remove_value(ADDRESS_BLOCK);
  let queued = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.iter().rposition(|entry| {
      entry.0 == "ltx:note"
        && entry.1.get("role").map(String::as_str) == Some("address")
        && (last_group(&entry.2) == block || entry.2 == block)
    }),
    _ => None,
  });
  let Some(at) = queued else {
    return Ok(Vec::new());
  };
  let Some(calls) = address_block_calls(&block)? else {
    Info!(
      "frontmatter",
      "address",
      s!(
        "An \\address under the authors stays its note (a line no rule places): {}",
        s!("{block}")
          .split_whitespace()
          .collect::<Vec<_>>()
          .join(" ")
      )
    );
    return Ok(Vec::new());
  };
  with_value_mut("frontmatter_raw", |v| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = v {
      queue.remove(at);
    }
  });
  Ok(calls)
}

/// The calls of an `\address` block printed under the names, only when every line is placed (OD #1073; the review's
/// fail-safe, all or nothing): with a queued author; a block with no mark under names showing none every author's
/// affiliation (`\lx@add@affiliation[annotate=all]`, 2405.13379, 2401.07506); a block led by a label whose every line
/// is led by a label a queued name shows or continues such a line, `\lx@add@affiliations`'s (1707.04373); no line of
/// e-mail addresses in either. Else `None`: trailing
/// marks (2407.15458, 2401.06806, 2211.09383), an unmarked block under marked names (2509.13723), a label no name shows
/// (2203.05882), no queued author (1909.01754), an address no rule places (2102.09114, 2407.20176, 1802.09371).
fn address_block_calls(block: &Tokens) -> Result<Option<Vec<Token>>> {
  if queued_position_names()?.is_empty() {
    return Ok(None);
  }
  // (a name's mark is a label: a footnote symbol, `\sthanks`'s ∗, labels no line)
  let names_marked = queued_author_marks()?
    .iter()
    .flatten()
    .any(|mark| mark.chars().any(char::is_alphanumeric));
  let block_marked = block.unlist_ref().iter().any(is_author_mark);
  let labelled = if !names_marked && !block_marked {
    false
  } else if names_marked && leads_with_label(block, "address block")? {
    true
  } else {
    return Ok(None);
  };
  let (_, body) = unwrapped(strip_linebreak_options(block.clone()).unlist());
  let body = wrapped_lines_as_lines(&Tokens::new(body.clone())).map_or(body, Tokens::unlist);
  let mut in_run = false;
  for (_, line) in date_lines(&body) {
    match line_kind(&line, &[])? {
      // (an e-mail line keeps the note whole: placed by `listed_address_owners`, 1802.09371's `aline.roumy@` under the
      // names "Aline", "Roumy" landed on a nameless creator; row R)
      Line::Addresses(..) => return Ok(None),
      // (a number the line opens with, typed and no mark, continues a labelled line: a street number, 1707.04373's
      // "453 F. Paul Anderson Tower")
      Line::Kept(_)
        if in_run
          && !line.iter().any(is_author_mark)
          && leading_line_label(&Tokens::new(line.clone())).is_some() => {},
      Line::Kept(_) => return Ok(None),
      Line::Labelled(..) if labelled => in_run = true,
      Line::Labelled(..) => return Ok(None),
      Line::Blank => {},
      Line::Text(_) | Line::Date if !labelled || in_run => {},
      Line::Text(_) | Line::Date => return Ok(None),
    }
  }
  let mut calls = if labelled {
    vec![T_CS!("\\lx@add@affiliations")]
  } else {
    mouth::tokenize_internal("\\lx@add@affiliation[annotate=all]").unlist()
  };
  calls.push(T_BEGIN!());
  calls.extend(block.unlist_ref().iter().copied());
  calls.push(T_END!());
  Ok(Some(calls))
}

/// The e-mail contacts of a class's e-mail line printed under the names (spconf's and Interspeech.cls's `\email`, OD
/// #1073; 2405.13379): each address its owner's by [`listed_address_owners`], when every address has one among the
/// authors queued so far; else `None`, the line kept whole by the caller.
pub(in crate::base_utilities) fn listed_email_calls(line: &Tokens) -> Result<Option<Vec<Token>>> {
  let Some(addresses) = email_addresses(line) else {
    return Ok(None);
  };
  let mut keys = Vec::new();
  for name in queued_position_names()?.iter() {
    keys.push(printed_name_keys(name, None)?);
  }
  if keys.is_empty() || listed_address_owners(&addresses, &keys).contains(&None) {
    return Ok(None);
  }
  Ok(Some(address_calls(None, &addresses, &keys)?))
}

/// The e-mail contacts of a line of addresses no label leads, each its owner's by [`listed_address_owners`]: the author
/// it spells, else the author at its place in a list of one address per author, else the author block's. The line's
/// e-mail label names each contact.
fn address_calls(
  label: Option<&Tokens>,
  addresses: &[String],
  keys: &[NameKeys],
) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  for (address, owner) in addresses.iter().zip(listed_address_owners(addresses, keys)) {
    let owner = owner.map_or_else(|| s!("addresses:block"), |at| s!("author:{}", at + 1));
    let mut keyvals =
      mouth::tokenize_internal(TeXString::assembled(s!("role=email,label={{{owner}}}"))).unlist();
    if let Some(label) = label {
      keyvals.extend(mouth::tokenize_internal(",name=").unlist());
      keyvals.push(T_BEGIN!());
      keyvals.extend(label.unlist_ref().iter().copied());
      keyvals.push(T_ACTIVE!('~'));
      keyvals.push(T_END!());
    }
    calls.extend(
      Invocation!(T_CS!("\\lx@annotate@frontmatter"), vec![
        Some(mouth::tokenize_internal("ltx:creator")),
        Some(mouth::tokenize_internal("ltx:contact")),
        Some(Tokens::new(keyvals)),
        Some(Tokens::new(
          address
            .chars()
            .map(|c| if c == '_' {
              T_CS!("\\_")
            } else {
              T_OTHER!(c.to_string())
            })
            .collect(),
        ))
      ])
      .unlist(),
    );
  }
  Ok(calls)
}

/// A note the PDF prints with no mark, in the abstract, whose text is e-mail addresses alone (an e-mail label before
/// them aside, a braced group of local parts written out): the authors' e-mail contacts, each the author it spells
/// (`spelled_owners`; a `\date` line of addresses also takes the list's order, `listed_address_owners`), OD #1071
/// (2506.06941's `\footnotetext{\{p\_shojaee, imirzadeh, …\}@apple.com}` after `\maketitle`, an unmarked footnote line
/// under the abstract). Only where every address spells its author: else the note stays as it is, with an Info. Run as
/// the note closes (`ltx:note`'s `after_close`), after `relocate_footnote`, so a text a mark pairs with is no longer here.
pub fn address_note_contacts(document: &mut Document, node: &mut Node) -> Result<()> {
  // (not as a frontmatter flush digests it or places it: a frontmatter entry is the flush's)
  // (an abstract the insertion places is the body's: its notes are read; any other entry is the flush's)
  if lookup_bool("lx_frontmatter_flushing")
    || (lookup_bool(FRONTMATTER_INSERTING)
      && document
        .findnodes("ancestor::ltx:abstract", Some(node))
        .is_empty())
  {
    return Ok(());
  }
  let role = node.get_attribute("role").unwrap_or_default();
  if !matches!(role.as_str(), "footnotetext" | "footnote" | "thanks")
    || node
      .get_attribute("mark")
      .is_some_and(|mark| !mark.trim().is_empty())
    || node.get_parent().is_none()
    || document
      .findnodes("ancestor::ltx:abstract", Some(node))
      .is_empty()
  {
    return Ok(());
  }
  // (its text, not the body footnote's number tags: "footnote" glued to the first address, review probe f2)
  let text: String = node
    .get_child_nodes()
    .iter()
    .filter(|child| child.get_name() != "tags")
    .map(Node::get_content)
    .collect();
  let addresses = resolver::addresses_in(&text);
  if addresses.is_empty() || !resolver::only_addresses(&text) {
    return Ok(());
  }
  let creators: Vec<Node> =
    document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  let keys: Vec<NameKeys> = creators
    .iter()
    .map(|creator| {
      let name = document
        .findnodes("ltx:personname", Some(creator))
        .first()
        .map(Node::get_content)
        .unwrap_or_default();
      NameKeys::of_printed(&name)
    })
    .collect();
  // (each the author it spells: a note's place among the abstract's notes is no author's order, so no positional
  // guess, as a `\date` line under the names may make — review probe f2)
  let owners: Vec<Option<usize>> = addresses
    .iter()
    .map(|address| match spelled_owners(&keys, address).as_slice() {
      [at] => Some(*at),
      _ => None,
    })
    .collect();
  if keys.is_empty() || owners.iter().any(Option::is_none) {
    Info!(
      "frontmatter",
      "note",
      s!(
        "An unmarked note of e-mail addresses stays a note (not every address spells an author): {text}"
      )
    );
    return Ok(());
  }
  let name = get_frontmatter_name(None, "ltx:contact", "email")?;
  let saved = document.get_node().clone();
  for (address, owner) in addresses.iter().zip(owners) {
    let Some(creator) = owner.and_then(|at| creators.get(at)) else {
      continue;
    };
    document.set_node(creator);
    let mut attributes = HashMap::default();
    attributes.insert("role".to_string(), "email".to_string());
    if let Some(name) = name.as_ref() {
      attributes.insert("name".to_string(), name.clone());
    }
    let mut contact = document.open_element("ltx:contact", Some(attributes), None)?;
    contact.append_text(address).ok();
    document.close_element("ltx:contact")?;
    // (after what the creator holds: its own lines first, as the PDF prints the note under the abstract)
    contact.unlink_node();
    let mut creator = creator.clone();
    creator.add_child(&mut contact).ok();
  }
  document.set_node(&saved);
  document.remove_node(node.clone());
  Ok(())
}
