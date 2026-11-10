//! The frontmatter API's entry cutter: the `\lx@add@affiliation` calls for a printed list (`entry_calls`), address
//! runs and their placement, the marks a list's entries are cut at, and the marked/shared affiliation gates
//! (`\lx@add@affiliation@marked`/`@shared`) and class-numbered institutes the bindings and `frontmatter_stores` call. Moved
//! unchanged from `author_block/affiliations.rs` (Z18j): the Cut's own closure, rerouted onto the reading by Option C
//! P2-P4 (FRONTMATTER_SINGLE_ENGINE_PLAN, row "`author_block/`'s survivors").

use super::*;
use crate::base_utilities::defs::frontmatter_fields::queued_author_name_keys;

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
pub(in crate::base_utilities) struct EntryCall<'a> {
  pub(in crate::base_utilities) add: &'a dyn Fn(Option<Tokens>, Tokens) -> Result<Vec<Token>>,
  pub(in crate::base_utilities) labelled_address_lines: bool,
}

/// [`affiliation_calls`], each entry's call made by `add` (its attributes, its content) — the frontmatter API's Cut
/// ([`cut_marked_contact`]) adds an entry with the call that held the list, its role and wrapper kept.
pub(in crate::base_utilities) fn entry_calls(
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
  // (the lines a list sets in wrappers, a one-column tabular's rows, commands side by side, read as lines, as the
  // frontmatter API's Cut reads them — `relocate::wrapped_lines_as_lines`: IEEEtran's `\IEEEauthorblockA{\begin{tabular}{c}
  // $^{1}$ Univ A \\ $^{2}$ Univ B \\ …\end{tabular}}`, 2606.28962)
  let stuff = wrapped_lines_as_lines(&stuff).unwrap_or(stuff);
  let stuff = spaced_marks_as_marks(&stuff).unwrap_or(stuff);
  let stuff = rewrite_ordinal_superscripts(strip_linebreak_options(unalias_mark_commands(stuff)));
  let stuff = blank_line_marks_dropped(stuff);
  let stuff = glued_line_marks_as_marks(stuff, true);
  let with_sup = marks_are_labels && author_mark_position(&stuff).is_some();
  // In a marked list, the affiliations it names, each a mark-led line (or a piece of one) with the unmarked lines
  // that follow it.
  let mut marked_entries: Vec<Vec<Token>> = Vec::new();
  // The runs of address lines, placed once the whole list is read (`place_address_runs`); whether a run is being
  // read, and whether the line just read was an entry or its continuation.
  let mut address_runs: Vec<AddressRun> = Vec::new();
  let mut in_run = false;
  let mut follows_entry = false;
  // (the notes of the list's address lines, the document's: [`address_line_notes`])
  let mut address_notes: Vec<Token> = Vec::new();
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
      let is_address_line = |line: &Tokens| {
        if labelled_address_lines {
          line_addresses(line).is_some()
        } else {
          line_is_email_list(line)
        }
      };
      // (…and one carrying notes, its notes the document's, after the list's calls: [`address_line_notes`],
      // `\institute{$^{1}$Univ X \\ $^{2}$Univ Y \\ \texttt{a@x, b@y}\thanks{…}}`, as 2605.23553's author block)
      let line = match (authors_queued && author_mark_position(&line).is_none())
        .then(|| address_line_notes(&line, |rest| Ok(is_address_line(rest))))
        .transpose()?
        .flatten()
      {
        Some((rest, notes)) => {
          address_notes.extend(notes);
          rest
        },
        None => line,
      };
      let address_line = is_address_line(&line);
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
  calls.extend(address_notes);
  Ok(calls)
}

/// `stuff` without the marks opening a line that print no label — a superscript of spacing alone (`$^{\;}$`, `$^{~}$`,
/// `\textsuperscript{\,}`), braces, spaces and `\relax` aside: such a line is unmarked, so it continues the entry above
/// it as any unmarked line does, not an entry of its own (mnras 2303.08793's `$^{8}$Institute for Quantum Computing …,
/// Waterloo, \\ $^{\;}$Ontario N2L 3G1, Canada`, which pdflatex prints under entry 8 with no label; the line had gone to
/// the last name). Read by every list's cutter (and, until DEL2, by the old author block reader `dispatch::add_authors_calls_sectioned`)
/// ([`entry_calls`]). A blank mark elsewhere is left to the mark readers (`email_lines::drop_empty_marks`).
pub(in crate::base_utilities) fn blank_line_marks_dropped(stuff: Tokens) -> Tokens {
  let v = stuff.unlist_ref();
  let blank = |range: &[Token]| {
    range.iter().all(|t| {
      matches!(
        t.get_catcode(),
        Catcode::BEGIN | Catcode::END | Catcode::SPACE
      ) || *t == T_CS!("\\relax")
        || is_spacing(t)
    })
  };
  // (the mark at `i`, a line's first token: where it ends when it prints no label)
  let blank_mark_end = |i: usize| -> Option<usize> {
    let (operand, closing) =
      if v[i].get_catcode() == Catcode::MATH && v.get(i + 1) == Some(&T_SUPER!()) {
        (i + 2, true)
      } else if is_mark_command(&v[i]) {
        (
          i + 1 + v[i + 1..].iter().take_while(|t| **t == T_SPACE!()).count(),
          false,
        )
      } else {
        return None;
      };
    if v.get(operand)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let close = skip_group(v, operand);
    if close > v.len() || !blank(&v[operand..close]) {
      return None;
    }
    if closing {
      (v.get(close)?.get_catcode() == Catcode::MATH).then_some(close + 1)
    } else {
      Some(close)
    }
  };
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut line_start = true;
  let mut i = 0;
  let mut dropped = false;
  while i < v.len() {
    let t = v[i];
    if line_start && t == T_SPACE!() {
      out.push(t);
      i += 1;
      continue;
    }
    if line_start && let Some(end) = blank_mark_end(i) {
      i = end;
      dropped = true;
      line_start = false;
      continue;
    }
    line_start = t == T_CS!("\\\\") || t == T_CS!("\\newline");
    out.push(t);
    i += 1;
  }
  if dropped { Tokens::new(out) } else { stuff }
}

/// The key of a printed list the Cut reads (`list`, its marks written as the plain marks they print,
/// [`printed_marks_as_marks`]) of a contact of `role` annotating `parenttag`: two calls holding the same list have the
/// same key.
pub(in crate::base_utilities) fn cut_list_key(
  parenttag: &Tokens,
  role: &str,
  list: &Tokens,
) -> String {
  use std::hash::{Hash, Hasher};
  let mut hasher = std::hash::DefaultHasher::new();
  (parenttag.to_string(), role, list.to_string()).hash(&mut hasher);
  s!("{:016x}", hasher.finish())
}

/// The attribute naming an affiliation list a binding's own cutter cuts into its marked entries
/// (`\lx@add@affiliation@marked`, [`affiliation_calls`]), as the frontmatter API's Cut names its entries ([`CUT_LIST`]):
/// one printed list is cut once, by whichever cutter comes first — revtex's `\affiliation{{$^1$}Univ A\\ {$^2$}Univ B}`
/// after the first name, cut by the binding, and the same `\affiliation` after the later names, which the Cut would cut
/// again (review rev99a2 d1).
pub(in crate::base_utilities) fn binding_cut_list_attr(list: &Tokens) -> Tokens {
  let key = cut_list_key(
    &mouth::tokenize_internal("ltx:creator"),
    "affiliation",
    &printed_marks_as_marks(list.clone()),
  );
  mouth::tokenize_internal(TeXString::assembled(s!("{CUT_LIST}={key}")))
}

/// `list` with the marks spelled otherwise written as the plain marks they print, for the Cut's readers
/// ([`cut_marked_contact`]): lines in one-argument line wrappers as lines (`relocate::wrapped_lines_as_lines`), a
/// superscript spelled otherwise as `$^{…}$` (`relocate::spaced_marks_as_marks`), a class's `\inst{n}` as
/// `\textsuperscript{n}`.
pub(in crate::base_utilities) fn printed_marks_as_marks(list: Tokens) -> Tokens {
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
  let list = spaced_marks_written(&list, true).unwrap_or(list);
  // (`\inst` by its meaning, the one-argument mark it prints: `line_marks::inst_prints_its_mark`)
  if !list.unlist_ref().contains(&T_CS!("\\inst")) || !inst_prints_its_mark() {
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

/// The frontmatter API's Cut decision (`cut_marked_contact`), the one predicate every reader of a store's marked
/// entries asks: does `list` hold two or more entries, cut as [`affiliation_calls`] cuts them, led by marks of distinct
/// labels that queued names request (`requesting_authors`), its first printed line led by a mark (`marker_leads`) —
/// its marks read as the Cut reads them, whatever their spelling (`printed_marks_as_marks`)?
pub fn holds_requested_marked_entries(list: &Tokens) -> bool {
  requested_marked_entries_in(&printed_marks_as_marks(list.clone()))
}

/// [`holds_requested_marked_entries`] of a list whose marks are written as the plain marks they print.
pub(in crate::base_utilities) fn requested_marked_entries_in(list: &Tokens) -> bool {
  requested_entry_marks(list).len() >= 2
}

/// The marks, of distinct labels, leading the entries of `list` (its marks written as the plain marks they print) that
/// queued names request — none when its first printed line is not led by a mark ([`requested_marked_entries_in`]).
pub(in crate::base_utilities) fn requested_entry_marks(list: &Tokens) -> Vec<Vec<Token>> {
  let lines: Vec<Tokens> = split_tokens_delimited(list.clone(), affil_splits())
    .into_iter()
    .map(|(_, line)| line)
    .filter(|line| !line.is_empty() && !prints_nothing(line))
    .collect();
  if !lines.first().is_some_and(marker_leads) {
    return Vec::new();
  }
  let mut labels: Vec<String> = Vec::new();
  let mut marks: Vec<Vec<Token>> = Vec::new();
  for line in lines.into_iter().filter(marker_leads) {
    for piece in split_wrapped_affiliation_marks(line) {
      if let Some(mark) = entry_mark(piece.unlist_ref())
        && !queued_requesters(&mark).is_empty()
        && let Some(label) = mark_label(&mark)
        && !labels.contains(&label)
      {
        labels.push(label);
        marks.push(mark);
      }
    }
  }
  marks
}

/// A class-numbered institute list as its title page prints it: llncs.cls:870-885 `\institutename` (copied by
/// preprint.cls:233-256, 2405.16615) redefines `\and` to step the institute counter and print `$^{\theinst}$` before the
/// next entry, the first numbered 1 — so the n-th `\and` entry of the `\@institute` store is affiliation n, which the
/// authors' numeric marks pick (`$^{1}$`, `$^2$`). `stuff` with each entry led by its number as a superscript, when the
/// class numbers the entries of its `store` (named by its control sequence, `\@institute`), the store holds at least two
/// `\and` entries and no mark of its own, and the queued authors' marks request numbers ([`numbered_marks_requested`]);
/// else `None` (the entries annotate every author, `\lx@add@store@affiliations`). Whether the class numbers them is
/// what its own title page printed in the speculative run frontmatter_stores.rs `note_and_numbering` made of the store
/// (`lx_and_numbered_<store>`: the sentinel entries each printed after its number, in `\and` order — sab.cls:404-426's
/// and edpsci.cls's `\refstepcounter{inst}`, 2308.05831, 2308.05830, 2501.08874, baaa.cls 2206.02830, comsis2.cls
/// 2207.09249; batch 104b); with no such run (no store, none could be made), the class's `\institutename` read as
/// before: a `\def\and` stepping a counter (`\stepcounter`). Was each author given every institute (2405.16615; SYNC_STATUS
/// row 153; the sab family, SYNC_STATUS row "104b").
pub fn class_numbered_institutes(stuff: &Tokens, store: Option<&str>) -> Option<Tokens> {
  let run = store.and_then(|store| match lookup_value(&s!("lx_and_numbered_{store}")) {
    Some(Stored::Bool(numbered)) => Some(numbered),
    _ => None,
  });
  let numbers_its_entries = run.unwrap_or_else(|| {
    matches!(lookup_definition_stored(&T_CS!("\\institutename")),
    Ok(Some(Stored::Expandable(definition)))
      if matches!(definition.get_expansion(), Some(ExpansionBody::Tokens(body))
        if body.unlist_ref().windows(2).any(|w| w[0] == T_CS!("\\def") && w[1] == T_CS!("\\and"))
          && body.unlist_ref().contains(&T_CS!("\\stepcounter"))))
  });
  if !numbers_its_entries || author_mark_position(stuff).is_some() || !numbered_marks_requested() {
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

/// Do the queued authors' marks request numbered entries: each mark a comma list of numbers (`\inst{1,2}`), at least
/// one, beside which only symbols stand — a legend's `\textsuperscript{\dag}`, `*` (comsis2.cls's names, 2207.09249), an
/// empty piece (edpsci's `\inst{1,2}$^{,}$`, 2501.08874) —, never a letter label (`\inst{a}`), which no number answers?
pub fn numbered_marks_requested() -> bool {
  let mut numbers = false;
  for operand in queued_author_marks().iter().flatten() {
    for piece in Tokens::new(operand.clone()).to_string().split(',') {
      let piece = piece.trim();
      if piece.is_empty() || piece.starts_with('\\') || !piece.chars().any(|c| c.is_alphanumeric())
      {
        continue;
      }
      if !piece.chars().all(|c| c.is_ascii_digit()) {
        return false;
      }
      numbers = true;
    }
  }
  numbers
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
///   exactly one author requests is that author's; any other run under an affiliation goes back into that entry, whose
///   Kind step takes it out as the entry's e-mails and whose Link step places them (the author each spells, the entry's
///   only carrier, …, else the block once: [`kind_entry_addresses`], OD #956);
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
      // (a run under an affiliation is that entry's: it goes back into it, whose Kind step takes its addresses out as
      // the entry's e-mails and whose Link step places them — the author each spells, the entry's only carrier, every
      // carrier of a symbol-led one, one per carrier in order, else the block once: `kind_entry_addresses`,
      // `owners::entry_address_owners`, OXIDIZED_DESIGN_DIVERGENCES #956; it no longer stays in the entry's text,
      // 2609.19448's one address under one author's affiliation still that author's)
      Some((entry, at)) if under_affiliations => {
        let requesting =
          entry_mark(&entries[entry]).map_or_else(Vec::new, |mark| queued_requesters(&mark));
        // (one address under an affiliation one author requests is that author's, its line made the contact here: an
        // e-mail command a raw class's store holds, `\email{hyf@…}` under `\inst{1}`'s institute, prints nowhere if
        // left in the entry — astro-ph0701846)
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

/// Where the addresses of a line in an author block go.
pub(in crate::base_utilities) enum AddressPlacement {
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
  /// Each address its own contact, with no placement of its own: an address line of an affiliation-kind entry, digested
  /// inside it, the entry's address — its owners the entry's Link step gives ([`kind_entry_addresses`],
  /// `owners::entry_address_owners`).
  Inner,
}
