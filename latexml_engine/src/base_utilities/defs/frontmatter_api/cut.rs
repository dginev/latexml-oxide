//! The frontmatter API's Cut step (`\lx@add@frontmatter`'s `cut_marked_contact`): a printed list of contact entries cut at
//! its marks, one queued contact per entry, and the places of the uncut copies a binding printed earlier. Moved unchanged
//! from `author_block/affiliations.rs` (Z18a): the API's own step, which every binding's contact goes through. The cutter
//! it shares with the bindings' own lists (`entry_calls`, `printed_marks_as_marks`, `cut_list_key`, …) moved beside it,
//! to `affiliations.rs` (Z18j).

use super::*;
use crate::{base_utilities::defs::frontmatter_fields::text_tokens, frontmatter_stores::cs_in};

/// The frontmatter API's Cut (FRONTMATTER_SINGLE_ENGINE_PLAN "The target model", placement step 0: "one item per
/// printed entry; marked lists cut at their marks"), for every binding: an affiliation, alternate affiliation or address
/// contact (`\lx@annotate@frontmatter{ltx:creator}{ltx:contact}[role=…]{…}`, which `\lx@add@affiliation`,
/// `\lx@add@address`, `\lx@add@altaffiliation` and `\lx@add@contact` queue through) whose content holds two or more
/// entries each led by a printed mark that a queued name requests is queued as one contact per entry, each labelled by
/// its mark (`\lx@affiliation@withsup`), so the names showing a mark take its entry (step 1) and an entry no name
/// requests lands once at the block (step 4, OD #159). The calls, or `None` for a contact to queue as it is: any other
/// role; a call whose binding placed it (`label=`, `_unsplit`); a list already cut (its content `\lx@affiliation@withsup`);
/// content no mark leads, or holding fewer than two requested marks (`Joint Institute, \textsuperscript{1}Univ A and
/// \textsuperscript{2}Univ B` is one entry; `$^1$Univ A` one).
///
/// The cutter is the author blocks' ([`affiliation_calls`]: lines at `\\`, `\quad` and the `\and` family, a line at each
/// space-preceded mark, a wrapper repeated on each piece, an address line placed as the list's), and every spelling of
/// a mark is read by the shared readers first: lines set in one-argument line wrappers, a one-column tabular's rows and
/// commands set side by side whose arguments marks lead (`relocate::wrapped_lines_as_lines`: 2606.28962, 2006.16977;
/// OD #964), a superscript on a spacing nucleus, in `\( … \)` or in a box of its own
/// (`relocate::spaced_marks_as_marks`, `spaced_mark_at`: `${}^{a}$`, `${\,\!}^1$`, `\mbox{${\ }^{1}$}`), and a class's
/// `\inst{n}` (aa.cls:759 `\def\inst#1{\unskip\hbox{$^{#1}$}}`, mem.cls:318, llncs) written as the
/// `\textsuperscript{n}` it prints. Witnesses (census127 cluster 2): acmart's `\institution{$^1$ Tsinghua University,
/// China; $^2$ Tencent Inc., China}` (2602.11605), `\institution{\textsuperscript{1}… \quad \textsuperscript{2}…}`
/// (2506.18881, cidr's 2408.03013), a raw class's `\affil`/`\institute` store (asp2006's 1002.4968, mem's
/// astro-ph0701509: 40 authors each given all 16 institutes), aa's sole `\institute{\inst{1}…\\ \inst{2}…\\
/// \inst{*}\email{…}}` (2410.00577). OXIDIZED_DESIGN_DIVERGENCES #954.
pub(in crate::base_utilities) fn cut_marked_contact(
  parenttag: &Tokens,
  tag: &Tokens,
  kv: Option<&KeyVals>,
  content: &Tokens,
) -> Result<Option<Tokens>> {
  if tag.to_string() != "ltx:contact" {
    return Ok(None);
  }
  let Some(kv) = kv else {
    return Ok(None);
  };
  let role = kv
    .get_value("role")
    .map(ToString::to_string)
    .unwrap_or_default();
  if !CUT_ROLES.contains(&role.as_str())
    || kv.get_value("label").is_some()
    || kv.get_value("_unsplit").is_some()
  {
    return Ok(None);
  }
  cut_list(parenttag, tag, &role, kv.revert()?, content, &API_CUT)
}

/// The Cut of `content`, a contact of `role` annotating `parenttag` with the call's attributes `call_attr`, as `parameters`
/// read its list ([`cut_marked_contact`]); `None` for a list they leave whole.
pub(in crate::base_utilities) fn cut_list(
  parenttag: &Tokens,
  tag: &Tokens,
  role: &str,
  call_attr: Tokens,
  content: &Tokens,
  parameters: &CutParameters,
) -> Result<Option<Tokens>> {
  // (the content inside the affiliation's own wrapper, `\lx@add@affiliation`'s `\lx@affiliation@withinst{…}`, which
  // each entry keeps)
  let (withinst, list) = within_inst(content);
  if list.unlist_ref().iter().find(|t| **t != T_SPACE!())
    == Some(&T_CS!("\\lx@affiliation@withsup"))
  {
    return Ok(None);
  }
  let list = printed_marks_as_marks(list);
  let requested = requested_entry_labels(&list, "cut_list")?;
  if !parameters.every_mark && requested.len() < 2 {
    return Ok(None);
  }
  // (one printed list, one Cut: a list repeated after each author's name, 2312.02779's five identical
  // `\affiliation{{$^1$}…Belfast…\\ {$^2$}…Trinity…}`, is cut once — a later copy adds no entries, as the cut one's
  // reach every author requesting their marks by label, and the copies queued before the cut, uncut while too few names
  // were queued to request two of its marks, give way to it once it is cut: review rev99a2 B1, [`place_uncut_copies`])
  let key = cut_list_key(parenttag, role, &list);
  if cut_list_queued(&key) {
    place_uncut_copies(parenttag, role, &key, &requested)?;
    DebugFeature!(
      "frontmatter",
      "FRONT cut {role}: a copy of a list already cut"
    );
    return Ok(Some(Tokens::new(Vec::new())));
  }
  // (the parts after a wrapper that holds the entries, `\institution{$^1$A \quad $^2$B}\country{X}`: an address part
  // every entry shares, repeated on each as the opening declarations are; a part holding a link, `\country{Project page:
  // \href{…}{…}}`, the block's, once — review rev99a S3, 2506.18881)
  let (list, closing, block_parts) = if parameters.shared_parts {
    shared_closing_parts(list)
  } else {
    (list, Vec::new(), Vec::new())
  };
  let list = if parameters.trim_closing_separator {
    Tokens::new(trim_trailing_separator(list.unlist()))
  } else {
    list
  };
  // (the declarations the list opens with, in force on every entry as on the whole contact — `\itshape $^1$Univ A\\
  // $^2$Univ B`, `{\bfseries $^3$Lab C\\ $^4$Lab D}`: each entry keeps them, as the observer writer's cut keeps the font
  // open before a piece, review rev94b r3 B1)
  let opening = notes_lead_declarations(&list);
  // (the call's own attributes, without the ones its marks replace)
  let mut attr: Vec<Token> = Vec::new();
  for item in split_tokens(call_attr, vec![T_OTHER!(",").into()]) {
    let key = item.to_string();
    let key = key.split('=').next().unwrap_or_default().trim();
    if key.is_empty() || parameters.dropped_keys.contains(&key) {
      continue;
    }
    if !attr.is_empty() {
      attr.push(T_OTHER!(","));
    }
    attr.extend(item.unlist());
  }
  // (and the list each entry came from, [`CUT_LIST`])
  let attr = Some(with_key(Some(Tokens::new(attr)), s!("{CUT_LIST}={key}")));
  let (cut, entries) = (std::cell::Cell::new(0), std::cell::Cell::new(0));
  let add = |attr: Option<Tokens>, entry: Tokens| {
    let (marked, attr) = mark_shown_attr(attr, &entry, "true");
    cut.set(cut.get() + usize::from(marked));
    entries.set(entries.get() + 1);
    let entry = with_closing(with_opening(entry, &opening), &closing);
    annotate_call(parenttag, tag, attr, entry, withinst)
  };
  let call = EntryCall {
    add:                    &add,
    labelled_address_lines: parameters.labelled_address_lines,
  };
  let mut calls = entry_calls(&call, attr.clone(), attr.clone(), None, list, true, true)?;
  // (one entry no mark labels, as the cutter reads them: the list stays whole, as the call cut again would be without end
  // — acmart's `\institution{$^1$…}`; unmarked lines are each a call, cond-mat0206329. Nor is a list cut whose requested
  // entries the cutter's lines do not part: the observer reads it whole by its marks — a list in a `\parbox`, 2608.21425)
  if (cut.get() == 0 && entries.get() < 2) || (!parameters.every_mark && cut.get() < 2) {
    return Ok(None);
  }
  place_uncut_copies(parenttag, role, &key, &requested)?;
  for part in block_parts {
    let block = with_key(attr.clone(), "label=addresses:block".to_string());
    calls.extend(annotate_call(parenttag, tag, Some(block), part, withinst)?);
  }
  DebugFeature!(
    "frontmatter",
    "FRONT cut {role} at its marks: {}",
    Tokens::new(calls.clone())
  );
  Ok(Some(Tokens::new(calls)))
}

/// `_markshown`'s value for an entry that shows its mark where the names show theirs (the binding lists' cutter,
/// `affiliation_calls`: `emitter::names_show_marks`); the Cut's show theirs (`true`). Read at `\lx@annotate@frontmatter@now`.
pub(super) const MARK_SHOWN_WITH_NAMES: &str = "names";

/// Whether a cut `entry` is marked (`\lx@affiliation@withsup{…}` whole), and `attr` with `_markshown={shown}` where it is
/// — but not on an address line, whose address becomes the e-mail contact of the authors showing its mark, which leaves the
/// mark no entry (aa's `\inst{*}\email{…}`, 2410.00577).
pub(super) fn mark_shown_attr(
  attr: Option<Tokens>,
  entry: &Tokens,
  shown: &str,
) -> (bool, Option<Tokens>) {
  let v = entry.unlist_ref();
  let marked = v.len() >= 3
    && v[0] == T_CS!("\\lx@affiliation@withsup")
    && v[1].get_catcode() == Catcode::BEGIN
    && skip_group(v, 1) == v.len();
  let address = || {
    leading_line_label(&Tokens::new(v[2..v.len() - 1].to_vec()))
      .is_some_and(|(_, _, rest)| line_is_email_list(&rest))
  };
  if !marked || address() {
    return (marked, attr);
  }
  (marked, Some(with_key(attr, s!("_markshown={shown}"))))
}

/// `attr` with `key` after its own keys.
fn with_key(attr: Option<Tokens>, key: String) -> Tokens {
  let mut keys = attr.map(Tokens::unlist).unwrap_or_default();
  if !keys.is_empty() {
    keys.push(T_OTHER!(","));
  }
  keys.extend(mouth::tokenize_internal(TeXString::assembled(key)).unlist());
  Tokens::new(keys)
}

/// The state key prefix of the place (queue ordinal) an author's uncut copy of a cut list held, `{CUT_COPY_PLACE}{key}:
/// author:{n}` ([`place_uncut_copies`], `\lx@cut@copy@place`).
pub(in crate::base_utilities) const CUT_COPY_PLACE: &str = "lx_cut_copy_place:";

/// The tag of the queue entry that stands where an uncut copy of a cut list was ([`place_uncut_copies`]): it makes no
/// element, only records the copy's place.
const CUT_COPY_TAG: &str = "lx:cut-copy";

/// Does the queue hold the entries of the printed list `key`, cut already ([`CUT_LIST`])?
fn cut_list_queued(key: &str) -> bool {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .any(|entry| entry.1.get(CUT_LIST).map(String::as_str) == Some(key)),
    _ => false,
  })
}

/// The queued copies of the printed list `key` (of a contact of `role` annotating `parenttag`, its entries' labels the
/// names request `requested`, [`requested_entry_labels`]) that the Cut left uncut — queued while too few names were
/// queued to request two of its marks
/// (amsart's `\author{Ann Able${}^{1}$}\address{${}^{1}$Univ A\\ ${}^{2}$Univ B}\author{Bob Baker${}^{2}$}` and the same
/// `\address` again: review rev99a2 d2; 2312.02779's REVTeX `\affiliation` after each name) — give way to its entries:
/// each is taken out of the queue when every name it reaches requests one of the list's marks, so each of them takes
/// its entry by its mark instead of the whole list; an entry stands in its place that records it for those names, so
/// the entry each takes keeps the copy's place among its contacts (user ruling R8, print order:
/// `relocate::at_queue_position`). The names a copy reaches are its run's — those queued after the last contact of its
/// role before it, which covers one placed on the name before it and a run's (`annotate=run`, `new`) — or every name
/// queued before it for one annotating them all (`annotate=all`, a number); a copy reaching a name that requests none of
/// the list's marks, or no name, stays as it is (no name loses the list it prints).
fn place_uncut_copies(
  parenttag: &Tokens,
  role: &str,
  key: &str,
  requested: &[(String, Vec<usize>)],
) -> Result<()> {
  // (the authors requesting one of the list's marks, as the Cut's decision reads them)
  let requesters: Vec<usize> = requested
    .iter()
    .flat_map(|(_, at)| at.iter().copied())
    .collect();
  let queue: Vec<RawFrontmatter> = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.clone(),
    _ => Vec::new(),
  });
  let mut removed: Vec<Tokens> = Vec::new();
  let mut places: Vec<(usize, Tokens)> = Vec::new();
  {
    let mut authors: Vec<(usize, usize)> = Vec::new();
    let mut last_of_role: Option<usize> = None;
    for (at, (tag, attr, command)) in queue.iter().enumerate() {
      if tag == "ltx:creator" && attr.get("role").map(String::as_str) == Some("author") {
        authors.push((at, authors.len() + 1));
        continue;
      }
      if tag != "ltx:contact" || attr.get("role").map(String::as_str) != Some(role) {
        continue;
      }
      let since = last_of_role.replace(at);
      let v = command.unlist_ref();
      if attr.contains_key("label")
        || attr.contains_key("_unsplit")
        || attr.contains_key(CUT_LIST)
        || v.first() != Some(&T_CS!("\\lx@annotate@frontmatter@now"))
        || !annotates(v, parenttag)
      {
        continue;
      }
      let (_, copy) = within_inst(&last_group(command));
      if cut_list_key(parenttag, role, &printed_marks_as_marks(copy)) != key {
        continue;
      }
      let all = attr
        .get("annotate")
        .is_some_and(|a| a == "all" || a.parse::<usize>().is_ok());
      let reach: Vec<&(usize, usize)> = authors
        .iter()
        .filter(|(position, _)| all || since.is_none_or(|since| *position > since))
        .collect();
      if reach.is_empty() || !reach.iter().all(|(_, n)| requesters.contains(n)) {
        continue;
      }
      let names: Vec<String> = reach.iter().map(|(_, n)| n.to_string()).collect();
      DebugFeature!(
        "frontmatter",
        "FRONT cut {role}: an uncut copy of the list gives way to it, for authors {}",
        names.join(",")
      );
      removed.push(last_group(command));
      let mut place = vec![T_CS!("\\lx@cut@copy@place"), T_BEGIN!()];
      place.extend(text_tokens(key));
      place.extend([T_END!(), T_BEGIN!()]);
      place.extend(text_tokens(&names.join(",")));
      place.extend([T_END!(), T_BEGIN!()]);
      place.extend(text_tokens(attr.get("_zone").map_or("", String::as_str)));
      place.push(T_END!());
      places.push((at, Tokens::new(place)));
    }
  }
  if places.is_empty() {
    return Ok(());
  }
  // (each copy's queue entry, the same one: nothing was queued since it was read)
  with_value_mut("frontmatter_raw", |val_opt| {
    if let Some(&mut Stored::FrontmatterRaw(ref mut queue)) = val_opt {
      for (at, place) in places {
        if let Some(entry) = queue.get_mut(at) {
          *entry = (CUT_COPY_TAG.to_string(), TagAttrs::default(), place);
        }
      }
    }
  });
  // (the frontmatter observer's ledger: the calls taken out, as a clear's, `ledger::record_clear`)
  for content in removed {
    record_cut_copy(role, &content);
  }
  Ok(())
}

/// Does the queued `\lx@annotate@frontmatter@now{parenttag}…` call `command` annotate `parenttag`?
fn annotates(command: &[Token], parenttag: &Tokens) -> bool {
  if command
    .get(1)
    .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
  {
    return false;
  }
  let end = skip_group(command, 1);
  end > 2 && command[2..end - 1] == *parenttag.unlist_ref()
}

/// The declarations `list` opens with before its first entry's mark (`\itshape`, `\small`, `\color{red}`:
/// `notes::lead_declarations`), its leading spaces and groups past.
fn notes_lead_declarations(list: &Tokens) -> Vec<Token> {
  let v = list.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!()).unwrap_or(v.len());
  lead_declarations(&v[start..])
}

/// `entry` (an entry of the Cut, `\lx@affiliation@withsup{…}` or the line itself) with the list's opening declarations
/// at its head, where it does not open with them already (the first entry).
fn with_opening(entry: Tokens, opening: &[Token]) -> Tokens {
  if opening.is_empty() {
    return entry;
  }
  let v = entry.unlist_ref();
  let wrapped = v.len() >= 3
    && v[0] == T_CS!("\\lx@affiliation@withsup")
    && v[1].get_catcode() == Catcode::BEGIN
    && skip_group(v, 1) == v.len();
  let inner = if wrapped { &v[2..v.len() - 1] } else { &v[..] };
  let lead = inner
    .iter()
    .position(|t| *t != T_SPACE!())
    .unwrap_or(inner.len());
  if inner[lead..].starts_with(opening) {
    return entry;
  }
  let mut out: Vec<Token> = Vec::with_capacity(v.len() + opening.len() + 1);
  if wrapped {
    out.extend_from_slice(&v[..2]);
  }
  out.extend_from_slice(opening);
  out.extend_from_slice(&inner[lead..]);
  if wrapped {
    out.push(v[v.len() - 1]);
  }
  Tokens::new(out)
}

/// `entry` (an entry of the Cut, `\lx@affiliation@withsup{…}` or the line itself) with the parts every entry of the list
/// shares at its end ([`shared_closing_parts`]), inside the mark's wrapper as [`with_opening`] puts the declarations.
fn with_closing(entry: Tokens, closing: &[Token]) -> Tokens {
  if closing.is_empty() {
    return entry;
  }
  let mut v = entry.unlist();
  let wrapped = v.len() >= 3
    && v[0] == T_CS!("\\lx@affiliation@withsup")
    && v[1].get_catcode() == Catcode::BEGIN
    && skip_group(&v, 1) == v.len();
  // (an entry that prints nothing, the empty piece before a list's first mark, `\institution{ $^1$…}`, stays as it is)
  let inner = if wrapped { &v[2..v.len() - 1] } else { &v[..] };
  if !inner
    .iter()
    .any(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) || is_author_mark(t))
  {
    return Tokens::new(v);
  }
  let close = if wrapped { v.pop() } else { None };
  v.extend_from_slice(closing);
  v.extend(close);
  Tokens::new(v)
}

/// `list` led by a wrapper holding its marked entries and followed by parts of its own (`\institution{$^1$A \quad
/// $^2$B}\country{X}`, acmart's address parts; pdflatex prints "X" on a line under both entries): the wrapper alone, the
/// parts every entry shares (an address part), and the parts the block holds once (one holding a link or an address,
/// `\country{Project page: \href{…}{…}}`, 2506.18881: no author's address); a part whose argument is spacing alone (a
/// space, a tie, `\relax`, a spacing command) is none of them.
/// Each part is a command and its argument, none a mark; any other list is returned as it is.
fn shared_closing_parts(list: Tokens) -> (Tokens, Vec<Token>, Vec<Tokens>) {
  let v = list.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!()).unwrap_or(v.len());
  let wrapper = v.get(start).is_some_and(|t| {
    t.get_catcode() == Catcode::CS && !is_mark_command(t) && !cs_in(t, &LINE_BOXES)
  }) && v
    .get(start + 1)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN);
  if !wrapper {
    return (list, Vec::new(), Vec::new());
  }
  let mut after = skip_group(v, start + 1);
  // (the wrappers set side by side, one line each, `\institution{$^1$A}\\ \institution{$^2$B}` as
  // `relocate::wrapped_lines_as_lines` reads them, hold the entries alike: 2006.16977)
  loop {
    let next = (after..v.len())
      .find(|&i| v[i] != T_SPACE!())
      .unwrap_or(v.len());
    let line = (next + 1..v.len())
      .find(|&i| v[i] != T_SPACE!())
      .unwrap_or(v.len());
    if v.get(next) == Some(&T_CS!("\\\\"))
      && v.get(line) == v.get(start)
      && v
        .get(line + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      after = skip_group(v, line + 1);
    } else {
      break;
    }
  }
  let mut parts: Vec<Vec<Token>> = Vec::new();
  let mut i = after;
  while i < v.len() {
    if v[i] == T_SPACE!() {
      i += 1;
      continue;
    }
    if v[i].get_catcode() != Catcode::CS
      || is_mark_command(&v[i])
      || v
        .get(i + 1)
        .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
    {
      return (list, Vec::new(), Vec::new());
    }
    let end = skip_group(v, i + 1);
    let part = &v[i..end];
    if part.iter().any(is_author_mark) {
      return (list, Vec::new(), Vec::new());
    }
    parts.push(part.to_vec());
    i = end;
  }
  if parts.is_empty() {
    return (list, Vec::new(), Vec::new());
  }
  let links = |part: &[Token]| {
    part
      .iter()
      .any(|t| *t == T_OTHER!("@") || cs_in(t, &["\\href", "\\url", "\\email", "\\mailto"]))
  };
  // (a part whose argument is spacing alone, `\country{~}` written to satisfy the class, is no entry's: repeated, or
  // left on the last entry, it printed ", ," there — 2605.18872; any other command in it may print, a macro's text
  // `\city{\cty}`, so the part is shared: review rev99a2 S-a, probe s3d)
  let inert = |part: &[Token]| {
    part[1..].iter().all(|t| {
      matches!(t.get_catcode(), Catcode::BEGIN | Catcode::END)
        || *t == T_CS!("\\relax")
        || is_spacing(t)
        || is_spacing_command(t)
    })
  };
  let (mut shared, mut block) = (Vec::new(), Vec::new());
  for part in parts {
    if inert(&part) {
      continue;
    } else if links(&part) {
      block.push(Tokens::new(part));
    } else {
      shared.extend(part);
    }
  }
  (Tokens::new(v[..after].to_vec()), shared, block)
}

/// The `\lx@annotate@frontmatter{parenttag}{tag}[attr]{entry}` call of an entry of the Cut ([`cut_marked_contact`]), in
/// the affiliation's own wrapper when the list was (`withinst`).
fn annotate_call(
  parenttag: &Tokens,
  tag: &Tokens,
  attr: Option<Tokens>,
  entry: Tokens,
  withinst: bool,
) -> Result<Vec<Token>> {
  let entry = if withinst {
    Invocation!(T_CS!("\\lx@affiliation@withinst"), vec![Some(entry)])
  } else {
    entry
  };
  let mut call = vec![T_CS!("\\lx@annotate@frontmatter"), T_BEGIN!()];
  call.extend(parenttag.unlist_ref().iter().copied());
  call.extend([T_END!(), T_BEGIN!()]);
  call.extend(tag.unlist_ref().iter().copied());
  call.push(T_END!());
  if let Some(attr) = attr {
    call.push(T_OTHER!("["));
    call.extend(attr.unlist());
    call.push(T_OTHER!("]"));
  }
  call.push(T_BEGIN!());
  call.extend(entry.unlist());
  call.push(T_END!());
  Ok(call)
}
