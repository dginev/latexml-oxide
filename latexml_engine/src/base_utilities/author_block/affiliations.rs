//! Author-block affiliations: the `\lx@add@affiliation` calls, address runs and their placement, contact owners.

use super::*;

/// The `\lx@add@affiliation` calls for an affiliation list: one per line or `\and`-separated
/// entry, each with its superscript marker as its label when the list has markers (Perl
/// Base_Utility.pool.ltxml:742-753, the body of `\lx@add@affiliations` after its dequeue). `attr`
/// is passed to each entry, `marked_attr` instead when the list has markers and `marks_are_labels` — and there
/// `unmarked_attr`, when given, to an entry no mark labels (no mark in it, no footnote symbol leading it): llncs's one
/// `\institute`'s `labelseq=affiliation,_share=authors`, its first line every author's as the class prints it (shape
/// (c), `\lx@llncs@institute@sup`). `authors_queued` says every author is queued, so an unmarked line may be placed by
/// the marks: an address run by its authors' positions, another line with the affiliation before it. An `\author`
/// call's tail (`author_tail_calls`) may come before later authors, and then its unmarked lines are its own creators'
/// rows, as each was before.
pub fn affiliation_calls(
  attr: Option<Tokens>,
  marked_attr: Option<Tokens>,
  unmarked_attr: Option<Tokens>,
  stuff: Tokens,
  marks_are_labels: bool,
  authors_queued: bool,
) -> Result<Vec<Token>> {
  let add = |attr: Option<Tokens>, entry: Tokens| {
    Ok(Invocation!(T_CS!("\\lx@add@affiliation"), vec![attr, Some(entry)]).unlist())
  };
  entry_calls(
    &EntryCall {
      add:                    &add,
      labelled_address_lines: false,
    },
    attr,
    marked_attr,
    unmarked_attr,
    stuff,
    marks_are_labels,
    authors_queued,
  )
}

/// How [`entry_calls`] adds an entry: the call made of its attributes and content, and whether an address line behind
/// an e-mail label is an address line of the list (the frontmatter API's Cut, [`cut_marked_contact`]) or text of the
/// entry above it (a binding's own list).
struct EntryCall<'a> {
  add:                    &'a dyn Fn(Option<Tokens>, Tokens) -> Result<Vec<Token>>,
  labelled_address_lines: bool,
}

/// [`affiliation_calls`], each entry's call made by `add` (its attributes, its content) — the frontmatter API's Cut
/// ([`cut_marked_contact`]) adds an entry with the call that held the list, its role and wrapper kept.
fn entry_calls(
  call: &EntryCall,
  attr: Option<Tokens>,
  marked_attr: Option<Tokens>,
  unmarked_attr: Option<Tokens>,
  stuff: Tokens,
  marks_are_labels: bool,
  authors_queued: bool,
) -> Result<Vec<Token>> {
  let EntryCall { add, labelled_address_lines } = *call;
  let mut calls: Vec<Token> = Vec::new();
  // Consume `\\[len]` row-break optionals before splitting (KNOWN_PERL_ERRORS #75).
  let stuff = rewrite_ordinal_superscripts(strip_linebreak_options(unalias_mark_commands(stuff)));
  let with_sup = marks_are_labels && author_mark_position(&stuff).is_some();
  // In a marked list, the affiliations it names, each a mark-led line (or a piece of one) with the unmarked lines
  // that follow it.
  let mut marked_entries: Vec<Vec<Token>> = Vec::new();
  // The runs of address lines, placed once the whole list is read (`place_address_runs`); whether a run is being
  // read, and whether the line just read was an entry or its continuation.
  let mut address_runs: Vec<AddressRun> = Vec::new();
  let mut in_run = false;
  let mut follows_entry = false;
  for (delimiter, line) in split_tokens_delimited(stuff, affil_splits()) {
    // Skip empty segments (e.g. a trailing \\ or a line that was wholly
    // consumed by an \email/\url) so they don't become blank affiliations.
    // Mirrors \lx@add@authors. So is a line that prints nothing ([`prints_nothing`]: IEEEtran's
    // `\end{@IEEEauthorhalign}\hfill\strut`, 2309.15562, an empty affiliation of the last author).
    if line.is_empty() || prints_nothing(&line) {
      continue;
    }
    if with_sup {
      let across_and = delimiter.iter().any(|t| {
        author_and_splits()
          .iter()
          .any(|d| matches!(d, SplitDelim::Token(a) if a == t))
      });
      // (in a list the frontmatter API cuts, an address line behind an e-mail label is one too, its addresses the run's:
      // `email: agupta@…, orion@…` closing an aa `\institute`, astro-ph0405186 — its label the contacts' own name,
      // `email_line_calls`; a binding's own list keeps such a line in the entry above it, which the observer's writer
      // reads: IEEEtran's `\IEEEauthorblockA{$^\dag$…\\ … \\ E-mail: …}`, 2308.05546)
      let address_line = if labelled_address_lines {
        line_addresses(&line).is_some()
      } else {
        line_is_email_list(&line)
      };
      if authors_queued && author_mark_position(&line).is_none() && address_line {
        // (`\and` separates entries, and runs: `Univ A \\ a@x \and b@y` is two)
        if !in_run || across_and {
          let under = (follows_entry && !across_and).then(|| {
            let entry = marked_entries.len() - 1;
            (entry, marked_entries[entry].len())
          });
          address_runs.push(AddressRun { lines: Vec::new(), under });
          in_run = true;
        }
        if let Some(run) = address_runs.last_mut() {
          run.lines.push((delimiter, line));
        }
        follows_entry = false;
        continue;
      }
      in_run = false;
      // An unmarked line continues the affiliation before it, with its `\\` put back, as an unmarked line continues
      // an entry in `\lx@add@authors`: `$^1$Department\\University` is one affiliation; on its own, an unlabelled
      // line was placed by position, not by mark (2609.19448; repro
      // sectioning-frontmatter/marked_affiliation_continuation_lines_stay_with_it). Not across `\and`, which
      // separates entries there too (DIVERGENCES #52(g)), nor a footnote-symbol legend (`* Equal contribution`,
      // `$\dagger$ Corresponding author`), which belongs to the whole block (llncs `\institute`, 2609.06094).
      if authors_queued
        && author_mark_position(&line).is_none()
        && !across_and
        && !starts_with_footnote_symbol(&line)
        && let Some(last) = marked_entries.last_mut()
      {
        last.extend(delimiter);
        last.extend(line.unlist());
        follows_entry = true;
        continue;
      }
      // The superscript markers ARE the affiliation labels here, so drop any
      // caller-supplied labelseq: applying both double-labels each affiliation
      // and duplicates it onto every \inst{n} author. Mirrors \lx@add@authors,
      // which likewise passes no attr in its with-superscript branch.
      // A line that starts with a mark and names several marked institutions (`$^1$Univ A; $^2$Univ B`)
      // is split at each whitespace-preceded mark, as `\lx@add@authors` splits its marker-led affiliation
      // lines: one label per affiliation, else the last mark labelled them all (2609.22690, 21347). A
      // line with text before its first mark (`Univ X \\ Present address: $^2$Univ Y`'s second line)
      // stays whole, as there.
      let segs = if marker_leads(&line) {
        split_wrapped_affiliation_marks(line)
      } else {
        vec![line]
      };
      for seg in segs {
        if spacing_end(seg.unlist_ref()) == 0 {
          continue;
        }
        marked_entries.push(seg.unlist());
        follows_entry = true;
      }
    } else {
      calls.extend(add(attr.clone(), line)?);
    }
  }
  let email_calls = place_address_runs(address_runs, &mut marked_entries, 0)?;
  // The separator between two affiliations (`$^1$Univ A, $^2$Univ B`) is trimmed from each entry but the last once
  // its address runs are in place: an entry is whole only then, and a run spliced at the length the entry had when the
  // run was read (`AddressRun::under`) would otherwise land past the end of an entry trimmed before it
  // (`\IEEEauthorrefmark{2}…Nankai University,\\ \{…\}@mail.nankai.edu.cn;\\ \IEEEauthorrefmark{3}…`, 2509.13772: a
  // panic, "range end index 44 out of range for slice of length 43"; repro
  // sectioning-frontmatter/affiliation_address_run_after_a_trailing_comma).
  let last = marked_entries.len().saturating_sub(1);
  for entry in &mut marked_entries[..last] {
    *entry = trim_trailing_separator(std::mem::take(entry));
  }
  for entry in marked_entries {
    // (an entry no mark labels, in a list whose caller says how such an entry is placed: as that call places it, not
    // labelled by a mark — llncs's `\institute{Univ A, Town \\ \textsuperscript{*}Equal contribution}`, whose `Univ A,
    // Town` went to the last author alone, every author's in pdflatex: dev20k 2603.18089, 2608.22772, 2207.13038,
    // 2609.31726, 1905.07852)
    let entry = Tokens::new(entry);
    if let Some(unmarked) = &unmarked_attr
      && author_mark_position(&entry).is_none()
      && !starts_with_footnote_symbol(&entry)
    {
      calls.extend(add(Some(unmarked.clone()), entry)?);
      continue;
    }
    let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(entry)]);
    calls.extend(add(marked_attr.clone(), withsup)?);
  }
  calls.extend(email_calls);
  Ok(calls)
}

/// The contact roles the frontmatter API's Cut reads ([`cut_marked_contact`]): the ones a printed list of entries is
/// labelled by marks in (an affiliation, an alternate one, an address).
const CUT_ROLES: [&str; 3] = ["affiliation", "altaffiliation", "address"];

/// The keys a Cut entry does not keep from the call that held its list: the marks are its labels, so a declared
/// sequence (`labelseq`) or sharing (`_share=authors`) no longer places it (FRONTMATTER_SINGLE_ENGINE_PLAN step 1
/// before step 3: no convention outranks a mark, rulings R1, R3).
const CUT_DROPPED_KEYS: [&str; 2] = ["labelseq", "_share"];

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
/// a mark is read by the shared readers first: lines set in one-argument line wrappers
/// (`relocate::wrapped_lines_as_lines`), a superscript on a spacing nucleus, in `\( … \)` or in a box of its own
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
  // (the content inside the affiliation's own wrapper, `\lx@add@affiliation`'s `\lx@affiliation@withinst{…}`, which
  // each entry keeps)
  let within = content.unlist_ref();
  let withinst = within.len() >= 3
    && within[0] == T_CS!("\\lx@affiliation@withinst")
    && within[1].get_catcode() == Catcode::BEGIN
    && skip_group(within, 1) == within.len();
  let list = if withinst {
    Tokens::new(within[2..within.len() - 1].to_vec())
  } else {
    content.clone()
  };
  if list.unlist_ref().iter().find(|t| **t != T_SPACE!())
    == Some(&T_CS!("\\lx@affiliation@withsup"))
  {
    return Ok(None);
  }
  let list = printed_marks_as_marks(list);
  if !holds_requested_marked_entries(&list) {
    return Ok(None);
  }
  // (the declarations the list opens with, in force on every entry as on the whole contact — `\itshape $^1$Univ A\\
  // $^2$Univ B`, `{\bfseries $^3$Lab C\\ $^4$Lab D}`: each entry keeps them, as the observer writer's cut keeps the font
  // open before a piece, review rev94b r3 B1)
  let opening = notes_lead_declarations(&list);
  // (the call's own attributes, without the ones its marks replace)
  let mut attr: Vec<Token> = Vec::new();
  for item in split_tokens(kv.revert()?, vec![T_OTHER!(",").into()]) {
    let key = item.to_string();
    let key = key.split('=').next().unwrap_or_default().trim();
    if key.is_empty() || CUT_DROPPED_KEYS.contains(&key) {
      continue;
    }
    if !attr.is_empty() {
      attr.push(T_OTHER!(","));
    }
    attr.extend(item.unlist());
  }
  let attr = Some(Tokens::new(attr));
  let add = |attr: Option<Tokens>, entry: Tokens| {
    // (each marked entry shows the mark that labels it, `_markshown`: `\lx@annotate@frontmatter@now` — but an address
    // line, whose address becomes the e-mail contact of the authors showing its mark, which leaves the mark no entry:
    // aa's `\inst{*}\email{…}`, 2410.00577)
    let v = entry.unlist_ref();
    let marked = v.len() >= 3
      && v[0] == T_CS!("\\lx@affiliation@withsup")
      && v[1].get_catcode() == Catcode::BEGIN
      && skip_group(v, 1) == v.len();
    let attr = if marked
      && leading_line_label(&Tokens::new(v[2..v.len() - 1].to_vec()))
        .is_none_or(|(_, _, rest)| !line_is_email_list(&rest))
    {
      let mut shown = attr.map(Tokens::unlist).unwrap_or_default();
      if !shown.is_empty() {
        shown.push(T_OTHER!(","));
      }
      shown.extend(mouth::tokenize_internal("_markshown=true").unlist());
      Some(Tokens::new(shown))
    } else {
      attr
    };
    let entry = with_opening(entry, &opening);
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
  };
  let call = EntryCall {
    add:                    &add,
    labelled_address_lines: true,
  };
  let calls = entry_calls(&call, attr.clone(), attr, None, list, true, true)?;
  DebugFeature!(
    "frontmatter",
    "FRONT cut {role} at its marks: {}",
    Tokens::new(calls.clone())
  );
  Ok(Some(Tokens::new(calls)))
}

/// `list` with the marks spelled otherwise written as the plain marks they print, for the Cut's readers
/// ([`cut_marked_contact`]): lines in one-argument line wrappers as lines (`relocate::wrapped_lines_as_lines`), a
/// superscript spelled otherwise as `$^{…}$` (`relocate::spaced_marks_as_marks`), a class's `\inst{n}` as
/// `\textsuperscript{n}`.
fn printed_marks_as_marks(list: Tokens) -> Tokens {
  // (a list wholly in one group, `{\bfseries $^3$Lab C\\ $^4$Lab D}`, is its content: the contact is a group of its own)
  let v = list.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!()).unwrap_or(v.len());
  let end = v
    .iter()
    .rposition(|t| *t != T_SPACE!())
    .map_or(0, |i| i + 1);
  let list = if start < end
    && v[start].get_catcode() == Catcode::BEGIN
    && skip_group(v, start) == end
    && split_top_level_lines(&v[start + 1..end - 1]).len() > 1
  {
    Tokens::new(v[start + 1..end - 1].to_vec())
  } else {
    list
  };
  let list = wrapped_lines_as_lines(&list).unwrap_or(list);
  let list = spaced_marks_as_marks(&list).unwrap_or(list);
  if !list.unlist_ref().contains(&T_CS!("\\inst")) {
    return list;
  }
  Tokens::new(
    list
      .unlist()
      .into_iter()
      .map(|t| {
        if t == T_CS!("\\inst") {
          T_CS!("\\textsuperscript")
        } else {
          t
        }
      })
      .collect(),
  )
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

/// Does `list` hold two or more entries, cut as [`affiliation_calls`] cuts them, led by marks of distinct labels that
/// queued names request ([`requesting_authors`]), its first printed line led by a mark ([`marker_leads`])?
fn holds_requested_marked_entries(list: &Tokens) -> bool {
  let authors = queued_author_marks();
  if authors.is_empty() {
    return false;
  }
  let lines: Vec<Tokens> = split_tokens_delimited(list.clone(), affil_splits())
    .into_iter()
    .map(|(_, line)| line)
    .filter(|line| !line.is_empty() && !prints_nothing(line))
    .collect();
  if !lines.first().is_some_and(marker_leads) {
    return false;
  }
  let mut labels: Vec<String> = Vec::new();
  for line in lines.into_iter().filter(marker_leads) {
    for piece in split_wrapped_affiliation_marks(line) {
      if let Some(mark) = entry_mark(piece.unlist_ref())
        && !requesting_authors(&authors, &mark).is_empty()
        && let Some(label) = mark_label(&mark)
        && !labels.contains(&label)
      {
        labels.push(label);
      }
    }
  }
  labels.len() >= 2
}

/// A class-numbered institute list as its title page prints it: llncs.cls:870-885 `\institutename` (copied by
/// preprint.cls:233-256, 2405.16615) redefines `\and` to step the institute counter and print `$^{\theinst}$` before the
/// next entry, the first numbered 1 — so the n-th `\and` entry of the `\@institute` store is affiliation n, which the
/// authors' numeric marks pick (`$^{1}$`, `$^2$`). `stuff` with each entry led by its number as a superscript, when the
/// class carries that `\institutename` (a counter stepped by its `\and`), the store holds at least two `\and` entries and
/// no mark of its own, and the queued authors carry marks that are all numbers; else `None` (the entries annotate every
/// author, `\lx@add@store@affiliations`). Was each author given every institute (2405.16615; SYNC_STATUS row 153).
pub fn class_numbered_institutes(stuff: &Tokens) -> Option<Tokens> {
  let numbers_its_entries = matches!(lookup_definition_stored(&T_CS!("\\institutename")),
    Ok(Some(Stored::Expandable(definition)))
      if matches!(definition.get_expansion(), Some(ExpansionBody::Tokens(body))
        if body.unlist_ref().windows(2).any(|w| w[0] == T_CS!("\\def") && w[1] == T_CS!("\\and"))
          && body.unlist_ref().contains(&T_CS!("\\stepcounter"))));
  if !numbers_its_entries || author_mark_position(stuff).is_some() {
    return None;
  }
  let marks: Vec<String> = queued_author_marks()
    .into_iter()
    .flatten()
    .map(|operand| Tokens::new(operand).to_string())
    .collect();
  let numeric = |mark: &String| {
    mark
      .split(',')
      .all(|piece| !piece.trim().is_empty() && piece.trim().chars().all(|c| c.is_ascii_digit()))
  };
  if marks.is_empty() || !marks.iter().all(numeric) {
    return None;
  }
  let entries = split_tokens(stuff.clone(), vec![T_CS!("\\and").into()]);
  if entries.len() < 2 {
    return None;
  }
  let mut out: Vec<Token> = Vec::new();
  for (n, entry) in entries.into_iter().enumerate() {
    if n > 0 {
      out.push(T_CS!("\\and"));
    }
    out.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
    out.extend(mouth::tokenize_internal(TeXString::assembled((n + 1).to_string())).unlist());
    out.push(T_END!());
    out.extend(entry.unlist());
  }
  Some(Tokens::new(out))
}

/// Whether an affiliation a class gives after its authors is a marked list ([`affiliation_calls`]): it has a mark, and the
/// queued authors carry marks — or none is queued yet, when the marks are labels for the authors to come (an affiliation
/// without its label had no one to go to).
pub(in crate::base_utilities) fn affiliation_is_marked(stuff: &Tokens) -> bool {
  (queued_creators_have_marks() || queued_creator_count() == 0)
    && author_mark_position(stuff).is_some()
}

/// The calls for an unmarked address shared by the authors before it (`\lx@add@affiliation@shared`): each of its
/// entries (separated by the `\and` family, `\And` the column break of the conference templates) one affiliation,
/// attributed by `attr` — one for entries printed alike (1302.5348's `\UMDAddress \\ … \And \UMDAddress \\ …`) —
/// without its e-mail lines, whose addresses, all of them, are one run placed as a marked list's are
/// ([`place_address_runs`]: one for each author of the row, in their order, else the block's, one contact an address —
/// the row the authors queued after the last affiliation, as `annotate=run` attaches one: 2302.10910's four e-mails of
/// a later row go to its four authors, not to the block for not being one for each queued author) — not lines of the
/// affiliation (1806.02924's `Carnegie Mellon University\\ \{asuggala, adarshp, vaishnavh, pradeepr\}@cs.cmu.edu`,
/// 1311.1894's `\texttt{\small \{honda,takemura\}@…}`), as the authors' own addresses are in a paired block
/// (2610.11580).
pub(in crate::base_utilities) fn shared_affiliation_calls(
  attr: Option<Tokens>,
  stuff: Tokens,
) -> Result<Vec<Token>> {
  let authors_queued = queued_creator_count() > 0;
  let mut entries: Vec<Vec<Token>> = Vec::new();
  let mut run = AddressRun { lines: Vec::new(), under: None };
  for entry in split_tokens(strip_linebreak_options(stuff), author_and_splits()) {
    let mut kept: Vec<Token> = Vec::new();
    for (delimiter, line) in split_tokens_delimited(entry, author_line_breaks()) {
      if authors_queued && line_is_email_list(&line) {
        run.lines.push((delimiter, line));
      } else if spacing_end(line.unlist_ref()) > 0 {
        if !kept.is_empty() {
          kept.extend(delimiter);
        }
        kept.extend(line.unlist());
      }
    }
    let lead = kept
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count();
    kept.drain(..lead);
    while kept
      .last()
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      kept.pop();
    }
    if !kept.is_empty() && !entries.contains(&kept) {
      entries.push(kept);
    }
  }
  let mut calls: Vec<Token> = Vec::new();
  for entry in entries {
    calls.extend(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        attr.clone(),
        Some(Tokens::new(entry))
      ])
      .unlist(),
    );
  }
  if !run.lines.is_empty() {
    calls.extend(place_address_runs(vec![run], &mut [], queued_row_start())?);
  }
  Ok(calls)
}

/// The addresses of an address line of a marked list: a line of addresses only (`email_addresses`), or one behind an
/// e-mail label (`email: a@x, b@y`: `split_email_label`), whose label `email_line_calls` keeps as the contacts' name.
pub(in crate::base_utilities) fn line_addresses(line: &Tokens) -> Option<Vec<String>> {
  email_addresses(line).or_else(|| match split_email_label(line) {
    (Some(_), rest) => email_addresses(&rest),
    _ => None,
  })
}

/// A run of consecutive address lines in a marked affiliation list, with the entry it follows (when the line before
/// it was that entry, not across `\and`) and the length that entry had when the run was read: where a continuation
/// goes.
struct AddressRun {
  lines: Vec<(Vec<Token>, Tokens)>,
  under: Option<(usize, usize)>,
}

/// The `\lx@add@email` calls for the address runs of a marked affiliation list (OXIDIZED_DESIGN #52(j)); a run that
/// stays with its affiliation is spliced into that entry. An address claims an author only where the list says
/// whose it is:
/// - when the list puts addresses under its affiliations — it has one, or a run sits between two (2609.19448
///   `$^1$Department\\University\\\texttt{david.abel@ed.ac.uk}\\[2ex]$^2$…`) — one address under an affiliation
///   exactly one author requests is that author's; any other run under an affiliation continues it, as the PDF prints
///   it, claiming no author (an affiliation several authors share, or several addresses under one author's, which
///   may be everyone's);
/// - else the run is the block's: one address for each author goes to the authors in order (llncs
///   `\institute{$^1$A \\ $^2$B \\ \email{a@x} \\ \email{b@y}}`), any other is one contact an address on the shared
///   creator below the authors (#159) — by order, 2609.06094's four addresses for six authors had given Jindong
///   Gu's to Runjia Li, and by the mark of the affiliation they follow, three of them to the wrong author.
///
/// `row_start` is the queue position the authors the list belongs to start after (0: every queued author).
fn place_address_runs(
  runs: Vec<AddressRun>,
  entries: &mut [Vec<Token>],
  row_start: usize,
) -> Result<Vec<Token>> {
  let authors = queued_author_marks();
  let row_start = row_start.min(authors.len());
  // (a legend line, `* Equal contribution`, is an entry with no mark; nor does a footnote-symbol mark after a run,
  // `$^\dagger$ Corresponding author`, show the run sits under an affiliation — as an iopart `\address`'s one mark it
  // still labels one)
  let marks: Vec<Option<Vec<Token>>> = entries.iter().map(|entry| entry_mark(entry)).collect();
  let affiliation_follows = |entry: usize| {
    marks[entry + 1..]
      .iter()
      .flatten()
      .any(|mark| !is_symbol_mark(mark))
  };
  let under_affiliations = marks.iter().flatten().count() == 1
    || runs.iter().any(|run| {
      run
        .under
        .is_some_and(|(entry, _)| affiliation_follows(entry))
    });
  let mut placed: Vec<Vec<Token>> = Vec::new();
  // In reverse, so a continuation spliced into an entry leaves the places of the runs before it there.
  for run in runs.into_iter().rev() {
    let counts: Vec<usize> = run
      .lines
      .iter()
      .map(|(_, line)| line_addresses(line).map_or(0, |a| a.len()))
      .collect();
    let total: usize = counts.iter().sum();
    let mut calls = Vec::new();
    match run.under {
      Some((entry, at)) if under_affiliations => {
        let requesting = entry_mark(&entries[entry])
          .map_or_else(Vec::new, |mark| requesting_authors(&authors, &mark));
        if let ([author], 1) = (requesting.as_slice(), total) {
          for (_, line) in run.lines {
            calls.extend(email_line_calls(
              line,
              AddressPlacement::Authors(vec![*author]),
            )?);
          }
        } else {
          let continuation: Vec<Token> = run
            .lines
            .into_iter()
            .flat_map(|(delimiter, line)| delimiter.into_iter().chain(line.unlist()))
            .collect();
          entries[entry].splice(at..at, continuation);
        }
      },
      _ if total == authors.len() - row_start && run_in_author_order(&run.lines, row_start) => {
        let mut positions = row_start + 1..;
        for ((_, line), count) in run.lines.into_iter().zip(counts) {
          let authors = positions.by_ref().take(count).collect();
          calls.extend(email_line_calls(line, AddressPlacement::Authors(authors))?);
        }
      },
      _ => {
        // (an address the run's place leaves to no author still goes to the one it spells, before the block: the
        // placement order's step 2 before step 4 — aa's `\institute{$^{1}$Univ X\\ $^{2}$Univ Y\\ \email{ann.able@y.org}}`
        // under three names, 1507.05963, cut at its marks by the frontmatter API, OXIDIZED_DESIGN_DIVERGENCES #954)
        let keys = queued_author_name_keys();
        for (_, line) in run.lines {
          let owners: Vec<Option<usize>> = line_addresses(&line)
            .unwrap_or_default()
            .iter()
            .map(|address| spelled_owner(&keys, address).map(|owner| owner + 1))
            .collect();
          let placement = if owners.iter().any(Option::is_some) {
            AddressPlacement::Settled(owners)
          } else {
            AddressPlacement::Shared
          };
          calls.extend(email_line_calls(line, placement)?);
        }
      },
    }
    placed.push(calls);
  }
  Ok(placed.into_iter().rev().flatten().collect())
}

/// The queue positions of the authors whose marks request the affiliation label `mark` sets. An author's mark reads as
/// its spelling (a symbol answered by its own line: OSA's `A. Able,$^{1,*}$` and `$^*$e-mail: …`, guard
/// `perfect_kernel_batch64u::osa_email_line_mark_shapes`) and as its request reads it ([`label_pieces`]): a glued
/// `1*`/`1\dagger` asks for label 1 too (77n r7 review probe g8: `Bob$^{2*}$` answered by `\affil{$^2$Y}`, not by
/// authblk's run).
pub(in crate::base_utilities) fn requesting_authors(
  authors: &[Vec<Vec<Token>>],
  mark: &[Token],
) -> Vec<usize> {
  let Some(label) = mark_label(mark) else {
    return Vec::new();
  };
  authors
    .iter()
    .enumerate()
    .filter(|(_, marks)| {
      marks.iter().any(|operand| {
        let reads = |operand: Vec<Token>| {
          clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation")
            .contains(&label)
        };
        reads(operand.clone()) || label_pieces(operand).is_some_and(reads)
      })
    })
    .map(|(i, _)| i + 1)
    .collect()
}

/// The name a queued author entry holds: the content of its `\lx@personname` (inside `\lx@author@withinst`), not the
/// `\lx@add@frontmatter@now{ltx:creator}[role=author]` around it, whose words read as surnames ("author" spelled in
/// `Corresponding author: …`; guard `perfect_kernel_batch64u::osa_style_address_email_line`).
pub(in crate::base_utilities) fn queued_name(entry: &Tokens) -> Tokens {
  let tokens = entry.unlist_ref();
  for cs in ["\\lx@author@withinst", "\\lx@personname"] {
    if let Some(at) = tokens.iter().position(|t| *t == T_CS!(cs))
      && tokens
        .get(at + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      let mut depth = 0usize;
      for (j, t) in tokens.iter().enumerate().skip(at + 1) {
        match t.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth == 0 {
              return Tokens::new(tokens[at + 2..j].to_vec());
            }
          },
          _ => {},
        }
      }
    }
  }
  entry.clone()
}

/// Whether the queued authors' names (`queued_name`) print every word `text` prints — each word one of a name's, or
/// an initial of one, accents aside (`visible_name_text`), or the word "and" between names: `text` then prints no
/// word the authors do not. True for a `text` printing no word; else false when no author is queued. svjour3's
/// `\institute{F. Author \at …}` piece names its authors before the address (`sv_support_sty.rs`): a piece writing
/// only their names adds nothing to the XML (`B. Baker, and C. Cole \at …`, batch 63's piece naming two), one writing
/// more (`\textit{Contact author:} Erwan {Le Merrer}`, 1711.01894; `Xin-long Luo, Corresponding author`, 2107.13864;
/// the whole name `Leonardo Anjoletto Ferreira` of `Leonardo A. Ferreira`, 1705.01399) prints words no name does.
pub fn queued_author_name_prints(text: &Tokens) -> bool {
  let written = printed_words(text);
  if written.is_empty() {
    return true;
  }
  let names: Vec<String> = queued_author_name_words().into_iter().flatten().collect();
  !names.is_empty()
    && written.iter().all(|w| {
      w == "and"
        || names
          .iter()
          .any(|o| o == w || (w.chars().count() == 1 && o.starts_with(w.as_str())))
    })
}

/// How many queued authors' names (`queued_name`) `text` spells whole: each author whose name's words (as
/// [`queued_author_name_prints`] reads them) stand together, in order, among the words `text` prints. svjour3's
/// `\institute{\begin{itemize}\item[\Letter] Ann Able \\ \email{…} \item[] Bob Baker \\ \email{…} \at \item[1]
/// Univ X …}` names its list's authors before the one `\at` (2312.14898): two or more names it no one by.
pub fn queued_author_names_spelled(text: &Tokens) -> usize {
  let written = printed_words(text);
  queued_author_name_words()
    .into_iter()
    .filter(|name| {
      !name.is_empty()
        && written
          .windows(name.len())
          .any(|run| run == name.as_slice())
    })
    .count()
}

/// The words `tokens` print as a name-shape test reads them ([`visible_name_text`]), accents aside, lowercased.
fn printed_words(tokens: &Tokens) -> Vec<String> {
  resolver::base_letters(&visible_name_text(tokens.unlist_ref()))
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect()
}

/// Each queued author's name ([`queued_name`]) as its [`printed_words`], in queue order.
fn queued_author_name_words() -> Vec<Vec<String>> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .map(|entry| printed_words(&queued_name(&entry.2)))
      .collect(),
    _ => Vec::new(),
  })
}

/// The keys ([`NameKeys`]) of each queued author's name ([`queued_name`]), in queue order (the n-th digests
/// with `author:n`).
pub(in crate::base_utilities) fn queued_author_name_keys() -> Vec<NameKeys> {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .filter(|entry| {
        entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
      })
      .map(|entry| NameKeys::of(&queued_name(&entry.2)))
      .collect(),
    _ => Vec::new(),
  })
}

/// Where a contact placed apart from the names goes ([`placed_contact_owner`]).
pub(in crate::base_utilities) enum ContactOwner {
  /// to the authors requesting the mark's label, which pairs them as an affiliation's mark does (and so the mark,
  /// answered, is not left standing on the name)
  Mark(String),
  /// to the author at this queue position (from 1)
  Author(usize),
  /// to the whole block, the shared creator below the authors (#159)
  Block,
}

/// The owner of a contact placed apart from the names (`\lx@add@contact@placed`), in this order:
/// 1. the mark written on the contact itself (`$^*$x@y`), when one queued author requests it — the contact's own mark
///    is the strongest evidence, stronger than the spelling (`*b@x` under `A$^*$` and `B` is A's); when several
///    request it, the one of them the address spells (1312.7046);
/// 2. the author whose surname the address's local part spells ([`spelled_owners`]: 0901.2914);
/// 3. the only author of the current group (`\lx@frontmatter@group@start`; the whole queue when no class marks
///    groups);
/// 4. else the block's.
///
/// An author block's labelled email lines rank otherwise ([`block_email_calls`]): there the spelling outranks the mark
/// of a correspondence NOTE before the line (1710.00802's `aleder@` under another author's "Corresponding Author"),
/// a mark on a nearby note being weaker evidence than one on the address.
///
/// A `weak` mark is not written on the contact but on the address block the class prints it under (iopart's `\ead`
/// after an `\address`, `\lx@address@mark@set`): it places only an address that spells no one (1111.5264's `johnwc@`
/// under `$^6$`, which two authors request, is J W Conklin's `$^1$`; 2407.14305's `rockfish3141@` under `$^1$ Waseda`,
/// Hidetomo Hoshino's alone).
pub(in crate::base_utilities) fn placed_contact_owner(
  mark: &Tokens,
  content: &Tokens,
  weak: bool,
) -> ContactOwner {
  let authors = queued_author_marks();
  let keys = queued_author_name_keys();
  let spelled: Vec<usize> = address_owners(&keys, content)
    .into_iter()
    .map(|at| at + 1)
    .collect();
  let requesting = if mark.is_empty() || (weak && !spelled.is_empty()) {
    Vec::new()
  } else {
    requesting_authors(&authors, mark.unlist_ref())
  };
  if let [author] = requesting.as_slice() {
    // (a footnote-symbol mark stays shown on the name and pairs by no label of its own: the author's position)
    return match mark_label(mark.unlist_ref()) {
      Some(label) if !is_symbol_mark(mark.unlist_ref()) => ContactOwner::Mark(label),
      _ => ContactOwner::Author(*author),
    };
  }
  if let [author] = requesting
    .iter()
    .copied()
    .filter(|author| spelled.contains(author))
    .collect::<Vec<_>>()
    .as_slice()
  {
    return ContactOwner::Author(*author);
  }
  if let [author] = spelled.as_slice() {
    return ContactOwner::Author(*author);
  }
  let first = frontmatter_group_first();
  if authors.len() == first {
    return ContactOwner::Author(first);
  }
  ContactOwner::Block
}

/// `text` as tokens of its characters, each as it is: an ASCII letter a letter, anything else other (no `#` parameter, no
/// `%` comment, no `{`/`}` group: a keyval value that is text).
pub(in crate::base_utilities) fn text_tokens(text: &str) -> Vec<Token> {
  text
    .chars()
    .map(|c| {
      let c = c.to_string();
      if c.chars().all(|c| c.is_ascii_alphabetic()) {
        T_LETTER!(&c)
      } else if c == " " {
        T_SPACE!()
      } else {
        T_OTHER!(&c)
      }
    })
    .collect()
}

/// The key a footnote-symbol mark (`$^\dagger$` on `$^\dagger$baker@y.edu`) answers on the author it places a contact
/// for ([`symbol_mark_key`]). `None` for any other mark.
pub(in crate::base_utilities) fn answered_symbol_key(mark: &[Token]) -> Option<String> {
  if mark.is_empty() || !is_symbol_mark(mark) {
    return None;
  }
  let label = mark_label(mark)?;
  symbol_mark_key(label.strip_prefix("affiliation:").unwrap_or(&label))
}

/// A symbol mark's key, made of its glyph (`unanswered_mark_text`: `\dag` and `\dagger` alike, `\ast` and `*` alike)
/// as hexadecimal code points (`sym-2020`): only letters, digits and `-`, so it passes through a keyval's tokenization
/// as it is (a `#`, `\#`'s glyph, read back as a parameter token raised `Error:misdefined:#`).
pub(in crate::base_utilities) fn symbol_mark_key(mark: &str) -> Option<String> {
  let glyph = unanswered_mark_text(mark)?.replace('\u{2217}', "*");
  (!glyph.chars().all(char::is_alphanumeric)).then(|| {
    s!(
      "sym-{}",
      glyph
        .chars()
        .map(|c| format!("{:x}", c as u32))
        .collect::<Vec<_>>()
        .join("-")
    )
  })
}

/// The queue position of the current author group's first author (`\lx@frontmatter@group@start`; 1 when no class marks
/// groups).
pub(in crate::base_utilities) fn frontmatter_group_first() -> usize {
  match lookup_value("lx_frontmatter_group_first") {
    Some(Stored::Number(n)) => usize::try_from(n.value_of()).unwrap_or(1).max(1),
    _ => 1,
  }
}

/// Where the addresses of a line in an author block go.
pub(in crate::base_utilities) enum AddressPlacement {
  /// The n-th address to the n-th author, counted over the emails so far (`labelseq=author`).
  Sequence,
  /// Each address to the author at that queue position, by the `author:N` label the creator digests with.
  Authors(Vec<usize>),
  /// Each address a name settles to the author at that queue position, as [`AddressPlacement::Authors`]; each other
  /// the block's, as [`AddressPlacement::Shared`] places it (2609.15028's `qq@…`-like address no name spells beside
  /// `zihanxue@…`, `serhii@…`; user ruling R10 2026-10-20). The line itself when no address is settled.
  Settled(Vec<Option<usize>>),
  /// The block's, on the shared creator below the authors (#159): its label names no creator. One contact an address
  /// when the line lists several (1802.07182's `\{jrr41,$\,$wct23,…\}@cam.ac.uk`, whose raw line a Semiverbatim
  /// contact showed as `{jrr41,$ $wct23,…}`), else the line.
  Shared,
  /// The line's, no address of which a name settles, to every author at these queue positions, those carrying the marks
  /// that led it (`$^*$zq77@x.edu` under `Ann Able$^{1,*}$`; main-session decision 3 on the 77p r11 questions;
  /// 2605.01818's `$^*$nbes@itu.dk` on the ∗ author).
  Marked(Vec<usize>),
}
