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
  // (the lines a list sets in wrappers, a one-column tabular's rows, commands side by side, read as lines, as the
  // frontmatter API's Cut reads them — `relocate::wrapped_lines_as_lines`: IEEEtran's `\IEEEauthorblockA{\begin{tabular}{c}
  // $^{1}$ Univ A \\ $^{2}$ Univ B \\ …\end{tabular}}`, 2606.28962)
  let stuff = wrapped_lines_as_lines(&stuff).unwrap_or(stuff);
  let stuff = spaced_marks_as_marks(&stuff).unwrap_or(stuff);
  let stuff = rewrite_ordinal_superscripts(strip_linebreak_options(unalias_mark_commands(stuff)));
  let stuff = blank_line_marks_dropped(stuff);
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
/// the last name). Read by the author block's reader (`dispatch::add_authors_calls_sectioned`) and by every list's cutter
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

/// The contact roles the frontmatter API's Cut reads ([`cut_marked_contact`]): the ones a printed list of entries is
/// labelled by marks in (an affiliation, an alternate one, an address).
pub(in crate::base_utilities) const CUT_ROLES: [&str; 3] =
  ["affiliation", "altaffiliation", "address"];

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
  // (the content inside the affiliation's own wrapper, `\lx@add@affiliation`'s `\lx@affiliation@withinst{…}`, which
  // each entry keeps)
  let (withinst, list) = within_inst(content);
  if list.unlist_ref().iter().find(|t| **t != T_SPACE!())
    == Some(&T_CS!("\\lx@affiliation@withsup"))
  {
    return Ok(None);
  }
  let list = printed_marks_as_marks(list);
  if !requested_marked_entries_in(&list) {
    return Ok(None);
  }
  // (one printed list, one Cut: a list repeated after each author's name, 2312.02779's five identical
  // `\affiliation{{$^1$}…Belfast…\\ {$^2$}…Trinity…}`, is cut once — a later copy adds no entries, as the cut one's
  // reach every author requesting their marks by label, and the copies queued before the cut, uncut while too few names
  // were queued to request two of its marks, give way to it: review rev99a2 B1, [`place_uncut_copies`])
  let key = cut_list_key(parenttag, &role, &list);
  place_uncut_copies(parenttag, &role, &key, &list);
  if cut_list_queued(&key) {
    DebugFeature!(
      "frontmatter",
      "FRONT cut {role}: a copy of a list already cut"
    );
    return Ok(Some(Tokens::new(Vec::new())));
  }
  // (the parts after a wrapper that holds the entries, `\institution{$^1$A \quad $^2$B}\country{X}`: an address part
  // every entry shares, repeated on each as the opening declarations are; a part holding a link, `\country{Project page:
  // \href{…}{…}}`, the block's, once — review rev99a S3, 2506.18881)
  let (list, closing, block_parts) = shared_closing_parts(list);
  // (nothing follows the list's last entry: a separator closing it is none of its text, `…Paris,⏎}`, 2009.04283)
  let list = Tokens::new(trim_trailing_separator(list.unlist()));
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
  // (and the list each entry came from, [`CUT_LIST`])
  if !attr.is_empty() {
    attr.push(T_OTHER!(","));
  }
  attr.extend(mouth::tokenize_internal(TeXString::assembled(s!("{CUT_LIST}={key}"))).unlist());
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
    let entry = with_closing(with_opening(entry, &opening), &closing);
    annotate_call(parenttag, tag, attr, entry, withinst)
  };
  let call = EntryCall {
    add:                    &add,
    labelled_address_lines: true,
  };
  let mut calls = entry_calls(&call, attr.clone(), attr.clone(), None, list, true, true)?;
  for part in block_parts {
    let mut block = attr.clone().map(Tokens::unlist).unwrap_or_default();
    if !block.is_empty() {
      block.push(T_OTHER!(","));
    }
    block.extend(mouth::tokenize_internal("label=addresses:block").unlist());
    calls.extend(annotate_call(
      parenttag,
      tag,
      Some(Tokens::new(block)),
      part,
      withinst,
    )?);
  }
  DebugFeature!(
    "frontmatter",
    "FRONT cut {role} at its marks: {}",
    Tokens::new(calls.clone())
  );
  Ok(Some(Tokens::new(calls)))
}

/// The Kind step for the addresses printed inside an affiliation-kind entry (FRONTMATTER_SINGLE_ENGINE_PLAN "The structure
/// model"; census127 cluster 1, the address family), at the frontmatter API's one call site after the Cut
/// (`\lx@annotate@frontmatter`, each piece of the Cut passing once): an affiliation, alternate affiliation or address
/// contact ([`CUT_ROLES`]) holding a line of e-mail addresses — a list (`\{aable, bbaker, ccole\}@a.edu`, `a@x, b@y`),
/// one behind an e-mail label (`Email: a@x, b@y`, its label the contacts' name), an e-mail command's (`\email{…}`), or
/// the rest of the entry's first line behind the mark that labels it (`{\tt $^1$\{aable, ccole\}@a.edu}`, the marked
/// entry the author block cut it as) — is that entry's content without those lines, each address an e-mail call digested
/// inside it ([`AddressPlacement::Inner`], `email_line_calls`): an e-mail of the entry's, whose owners the Link step
/// gives (`email_owners::entry_address_owners`, by the `_entry` key `\lx@annotate@frontmatter@now` gives the entry and
/// its inner contacts). An entry of addresses alone stays as their carrier, empty, and is dropped once they are placed
/// (`insert::drop_empty_contacts`). `None` for any other contact, or one holding no such line. The lines are read by the
/// author block's reader of an address line ([`line_addresses`]): an address inside a line of other words (`Univ X,
/// Town; ann@x.org`) is the frontmatter observer's text reading's alone (`Resolver::take_out_entry_addresses`), and a
/// line these tokens do not show — a document's macro, `\authorcr`, a connective "and" — the flush's Kind step's, which
/// reads it as printed (`insert::kind_address_lines`, OD #966; 2608.22602, 2111.01631, 2302.02638). A
/// template (`\{firstname.lastname\}@x.org`, [`is_pattern_address`]) or a list the reader could not part
/// ([`lists_local_parts`]) stays where it prints. Witnesses: authblk's `\affil{… \\ \{…\}@x}` (2502.01754) and
/// `\affil[ ]{\texttt{\{…\}@x}}` (2602.21641, 2307.02047), the kernel's `{\tt $^1$\{…\}@x}` lines (1901.09107), llncs's
/// `\email{\{ivan|carolin\}@x}` (0902.3918, 1909.00276), aaspp4's `$^3$…;\\ Email: dzg@, xywang@, hyf@`
/// (astro-ph0512555).
pub(in crate::base_utilities) fn kind_entry_addresses(
  tag: &Tokens,
  kv: Option<&KeyVals>,
  content: &Tokens,
) -> Result<Option<Tokens>> {
  if tag.to_string() != "ltx:contact"
    || !content
      .unlist_ref()
      .iter()
      .any(|t| t.with_str(|s| s == "@"))
  {
    return Ok(None);
  }
  let role = kv
    .and_then(|kv| kv.get_value("role"))
    .map(ToString::to_string)
    .unwrap_or_default();
  // (an entry a class shares among every author, llncs's one `\institute`, `_share=authors`: its lines of addresses are
  // the shared row's, placed by the same chain where the row is read — `shared_row::row_email_calls`, OD #901;
  // 1103.2027)
  if !CUT_ROLES.contains(&role.as_str()) || kv.is_some_and(|kv| kv.get_value("_share").is_some()) {
    return Ok(None);
  }
  let (withinst, inner) = within_inst(content);
  let (withsup, inner) = within_wrapper(&inner, "\\lx@affiliation@withsup");
  // (a footnote of the entry's holding addresses alone, behind a label or none — REVTeX's `\affiliation{Univ A,
  // Town\footnote{Electronic address: aable@a.edu}}`, the note the class's own `\email` prints (revtex4-2.cls:2404):
  // its addresses the entry's, as an address line's; Perl keeps the footnote a `<pubnote>` of the document,
  // Base_Utility.pool.ltxml:331 — 1002.3223, cond-mat0202268)
  let (inner, footnotes) = take_address_footnotes(inner);
  // (a line the reader takes out: its addresses read, none a template or a list it could not part, and its groups
  // balanced, so the entry left is)
  // (a line of e-mail commands is e-mails already, the class's own contacts with the label it prints — llncs's
  // `\email{…}` "E-mail", 2008.05861's `\email{\{jlwang,yhliu\}@…}, \email{jianbo@…}` —, digested inside the entry
  // and so its addresses: left as it is)
  // (a line holding any other command — an ORCID, a note, a symbol — is no line of addresses alone: it stays in the
  // entry as it prints, its addresses the text reading's, [`prints_addresses_only`]; 2308.02269's
  // `\url{{arim,sue}@ist.hokudai.ac.jp}\orcidID{0000-0002-2701-0271}` had read the ORCID's digits into the domain and
  // lost the ORCID, review rev100b B1)
  let addresses_of = |line: &Tokens| {
    let command = line
      .unlist_ref()
      .iter()
      .any(|t| t.get_catcode() == Catcode::CS && is_email_wrapper(t));
    (!command && prints_addresses_only(line))
      .then(|| line_addresses(line))
      .flatten()
      .filter(|addresses| {
        balanced(line.unlist_ref())
          && addresses
            .iter()
            .all(|a| !is_pattern_address(a) && !lists_local_parts(a))
      })
  };
  let mut kept: Vec<(Vec<Token>, Vec<Token>)> = Vec::new();
  let mut address_lines: Vec<Tokens> = Vec::new();
  // (for each address line, whether no line the entry keeps prints before it)
  let mut leading: Vec<bool> = Vec::new();
  let mut first = true;
  // (whether the entry prints anything but addresses, its mark aside)
  let mut prints_else = false;
  for (delimiter, line) in split_tokens_delimited(inner, affil_splits()) {
    let printed = !line.is_empty() && !prints_nothing(&line);
    // (the first line of a marked entry: the mark that labels it kept, the addresses behind it the entry's)
    if printed
      && first
      && withsup
      && let Some((mark, rest)) = mark_led_addresses(&line)
      && addresses_of(&rest).is_some()
    {
      kept.push((delimiter, mark));
      address_lines.push(rest);
      leading.push(false);
      first = false;
      continue;
    }
    // (a line a mark leads is no address line: its mark is no letter of an address, `$^2$bob@y.org`)
    if printed && author_mark_position(&line).is_none() && addresses_of(&line).is_some() {
      address_lines.push(line);
      leading.push(!prints_else);
      first = false;
      continue;
    }
    first &= !printed;
    prints_else |= printed;
    kept.push((delimiter, line.unlist()));
  }
  if address_lines.is_empty() && footnotes.is_empty() {
    return Ok(None);
  }
  let mut entry: Vec<Token> = Vec::new();
  for (delimiter, line) in kept {
    if !entry.is_empty() {
      entry.extend(delimiter);
    }
    entry.extend(line);
  }
  let mut entry = trim_trailing_separator(entry);
  // (an entry of addresses alone that the class names, `\mail{…}`'s "Correspondence to", REVTeX's
  // `\altaffiliation[E-mail address: ]{…}`: its name the e-mails' — where no label of their own names them; 0709.0811,
  // astro-ph0108379, cond-mat0703234, 1402.3092)
  let entry_name: Option<Vec<Token>> = (!prints_else && footnotes.is_empty())
    .then(|| kv.and_then(|kv| kv.revert().ok()))
    .flatten()
    .and_then(|attrs| {
      split_tokens(attrs, vec![T_OTHER!(",").into()])
        .into_iter()
        .find(|item| item.to_string().trim_start().starts_with("name="))
        .map(Tokens::unlist)
    })
    // (not the role's own default, `\lx@contact@altaffiliation@name` — "Alternate Affiliation:" names no e-mail, 0812.4856
    // —, nor an empty one, which names nothing: aa's `\mail` copy, 97b)
    .filter(|name| {
      let value = name.iter().skip_while(|t| **t != T_OTHER!("=")).skip(1);
      !name
        .iter()
        .any(|t| t.get_catcode() == Catcode::CS && t.with_str(|s| s.starts_with("\\lx@contact@")))
        && value.clone().any(|t| {
          matches!(
            t.get_catcode(),
            Catcode::LETTER | Catcode::OTHER | Catcode::CS
          )
        })
    });
  // (each e-mail the Kind step made says so, `ENTRY_KINDED`: an e-mail command the author wrote inside the entry is
  // placed by its own call where it holds one address, `insert::link_entry_addresses`)
  let kinded = mouth::tokenize_internal(TeXString::assembled(s!("{ENTRY_KINDED}=true"))).unlist();
  // (a line printed before every line the entry keeps, its e-mails placed before the entry: user ruling R8,
  // `\IEEEauthorblockA{\href{mailto:ann@a.org}{ann@a.org}\\ \url{https://orcid.org/…}\\ …}`, 2206.10577 — review
  // rev107c N1, `insert::LINE_FIRST`)
  let kinded_first = mouth::tokenize_internal(TeXString::assembled(s!(
    "{ENTRY_KINDED}=true,{LINE_FIRST}=true"
  )))
  .unlist();
  for (line, leading) in address_lines.into_iter().zip(leading) {
    let kinded = if leading && prints_else {
      &kinded_first
    } else {
      &kinded
    };
    // (the class's name before the line's own label where both print: aa's `\mail{e-mail: x}`, "Correspondence to
    // e-mail:", 0805.1001; REVTeX's `\altaffiliation[Authors for Correspondence: ]{Email: …}`, 2208.10784)
    let (label, rest) = split_email_label(&line);
    let calls = match (&entry_name, label) {
      (Some(name), None) => {
        optioned_email_calls(email_line_calls(line, AddressPlacement::Inner)?, name)
      },
      (Some(name), Some(label)) if line_is_email_list(&rest) && !line_is_email_list(&line) => {
        let value: Vec<Token> = name
          .iter()
          .copied()
          .skip_while(|t| *t != T_OTHER!("="))
          .skip(1)
          .filter(|t| !matches!(t.get_catcode(), Catcode::BEGIN | Catcode::END))
          .collect();
        let mut both = value;
        if both.last().is_some_and(|t| !is_spacing(t)) {
          both.push(T_SPACE!());
        }
        both.extend(label.unlist());
        named_email_calls(
          email_line_calls(rest, AddressPlacement::Inner)?,
          &Tokens::new(both),
        )
      },
      _ => email_line_calls(line, AddressPlacement::Inner)?,
    };
    entry.extend(optioned_email_calls(calls, kinded));
  }
  for (label, line) in footnotes {
    let calls = email_line_calls(line, AddressPlacement::Inner)?;
    let calls = match label {
      Some(label) => named_email_calls(calls, &label),
      None => calls,
    };
    entry.extend(optioned_email_calls(calls, &kinded));
  }
  let mut entry = Tokens::new(entry);
  if withsup {
    entry = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(entry)]);
  }
  if withinst {
    entry = Invocation!(T_CS!("\\lx@affiliation@withinst"), vec![Some(entry)]);
  }
  DebugFeature!(
    "frontmatter",
    "FRONT kind {role}: its address lines the entry's e-mails: {entry}"
  );
  Ok(Some(entry))
}

/// The attribute naming the affiliation-kind entry an e-mail was digested inside, and the entry itself (its key, a count
/// of such entries: `\lx@annotate@frontmatter@now`), by which the Link step reads the entry's addresses and carriers
/// (`insert::link_entry_addresses`; OXIDIZED_DESIGN_DIVERGENCES #956).
pub(in crate::base_utilities) const ENTRY_KEY: &str = "_entry";

/// The attribute saying a footnote symbol leads the entry [`ENTRY_KEY`] names (`$^\dagger$…`), whose addresses no
/// carrier spells are every carrier's (FIDELITY_CHARTER §4, "every symbol carrier").
pub(in crate::base_utilities) const ENTRY_SYMBOL: &str = "_entrysymbol";

/// The attribute (and `\lx@add@contact` option, a declared `Frontmatter` key) of a contact whose label its binding
/// prints before its line, as written: authblk's `\affil[x]` (`\AB@affilnote`, authblk.sty:142), jheppub's
/// `\affiliation[x]` (`\item[$^{#1}$]`), cas's letter (`cas_labelled_entry`). The Kind step for a legend gives a note
/// only a mark that prints (`insert::kind_legend_contacts`, OXIDIZED_DESIGN_DIVERGENCES #961): a binding's key
/// (icml's `\icmlaffiliation{intern}{…}`) is none.
pub(in crate::base_utilities) const LABEL_PRINTS: &str = "_labelprints";

/// The attribute of a contact labelled by its place among its role's entries (`labelseq`: svjour's and llncs's
/// `\institute{… \and …}`, aa's): the label `\lx@annotate@frontmatter@now` gave it, a number no mark printed — the
/// Kind step for a legend reads it as a mark only where a carrier's name shows or requests it
/// (`insert::kind_legend_contacts`, OXIDIZED_DESIGN_DIVERGENCES #961; 1507.04774's svjour_blank prints none). Any other
/// `affiliation:K` label was read off a printed mark (the marks path, the Cut, the observer's writer).
pub(in crate::base_utilities) const SEQUENCE_LABEL: &str = "_seqlabel";

/// The attribute of an e-mail the Kind step made of an entry's line or note ([`kind_entry_addresses`]), as against an
/// e-mail command the author wrote inside the entry, which its own call places where it holds one address
/// (FIDELITY_CHARTER §4 #420; `insert::link_entry_addresses`).
pub(in crate::base_utilities) const ENTRY_KINDED: &str = "_kinded";

/// The state key naming the affiliation-kind entry whose content is being digested (`<key>:symbol` or `<key>:`), whose
/// e-mails take its [`ENTRY_KEY`] (and its [`ENTRY_SYMBOL`]); empty outside one.
pub(in crate::base_utilities) const ENTRY_DIGESTING: &str = "lx_frontmatter_entry_digesting";

/// The state key counting the affiliation-kind entries given an [`ENTRY_KEY`].
pub(in crate::base_utilities) const ENTRY_KEY_COUNTER: &str = "lx_frontmatter_entry_key";

/// Whether a footnote symbol leads an entry's content (in the affiliation's own wrappers): a text symbol (`* Equal
/// contribution`, `\dag …`, [`starts_with_footnote_symbol`]) or a superscript one (`$^\dagger$Univ X`).
pub(in crate::base_utilities) fn led_by_footnote_symbol(content: &Tokens) -> bool {
  let (_, inner) = within_inst(content);
  let (_, inner) = within_wrapper(&inner, "\\lx@affiliation@withsup");
  starts_with_footnote_symbol(&inner)
    || (marker_leads(&inner)
      && entry_mark(inner.unlist_ref()).is_some_and(|mark| is_symbol_mark(&mark)))
}

/// `content` without the notes (`\footnote{…}`, `\thanks{…}`, an optional mark aside) that print a line of addresses
/// alone, behind a label or none, and those lines, each with the label its contacts are named by when the line's own
/// reader does not read it ([`note_addresses`]; [`kind_entry_addresses`]).
fn take_address_footnotes(content: Tokens) -> (Tokens, Vec<(Option<Tokens>, Tokens)>) {
  let v = content.unlist_ref();
  let note_cs = [T_CS!("\\footnote"), T_CS!("\\thanks")];
  if !v.iter().any(|t| note_cs.contains(t)) {
    return (content, Vec::new());
  }
  let mut kept: Vec<Token> = Vec::with_capacity(v.len());
  let mut lines: Vec<(Option<Tokens>, Tokens)> = Vec::new();
  let mut i = 0;
  while i < v.len() {
    if note_cs.contains(&v[i]) {
      let mut open = i + 1;
      while v.get(open) == Some(&T_SPACE!()) {
        open += 1;
      }
      // (its optional mark, `\footnote[2]{…}`)
      if v.get(open) == Some(&T_OTHER!("["))
        && let Some(close) = v[open..].iter().position(|t| *t == T_OTHER!("]"))
      {
        open += close + 1;
        while v.get(open) == Some(&T_SPACE!()) {
          open += 1;
        }
      }
      if v
        .get(open)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      {
        let end = skip_group(v, open);
        if end <= v.len()
          && end >= open + 2
          && let Some(line) = note_addresses(&Tokens::new(v[open + 1..end - 1].to_vec()))
        {
          lines.push(line);
          i = end;
          continue;
        }
      }
    }
    kept.push(v[i]);
    i += 1;
  }
  (Tokens::new(kept), lines)
}

/// The addresses line a note prints when it prints addresses alone: one [`line_addresses`] reads (a list, one behind
/// an e-mail label), or one behind a label ending in an e-mail label's words before its colon (`Corresponding authors
/// e-mail: cecilia.pennetta@unile.it`, cond-mat0202268; REVTeX's `Electronic address: …`, 1002.3223) — the label then
/// given apart, the contacts' name. None a template or a list the reader could not part.
fn note_addresses(note: &Tokens) -> Option<(Option<Tokens>, Tokens)> {
  let real = |addresses: Vec<String>| {
    addresses
      .iter()
      .all(|a| !is_pattern_address(a) && !lists_local_parts(a))
  };
  if line_addresses(note).is_some_and(real) {
    return Some((None, note.clone()));
  }
  let v = note.unlist_ref();
  let colon = v.iter().position(|t| *t == T_OTHER!(":"))?;
  let label: Vec<Token> = v[..colon]
    .iter()
    .copied()
    .skip_while(|t| *t == T_SPACE!())
    .collect();
  let text = visible_name_text(&label);
  let words: Vec<&str> = text.split_whitespace().collect();
  // (a label of a few words — `Corresponding authors e-mail`; a sentence before its colon is the note's text, `X. Zhang
  // and L. Duan are with …, Singapore. E-mail`, 1710.05616)
  let labelled = words.len() <= 4
    && (1..=2.min(words.len())).any(|n| is_email_label(&words[words.len() - n..].join(" ")));
  let rest = Tokens::new(
    v[colon + 1..]
      .iter()
      .copied()
      .skip_while(is_spacing)
      .collect(),
  );
  (labelled && email_addresses(&rest).is_some_and(real)).then(|| {
    let mut label = label;
    label.push(T_OTHER!(":"));
    (Some(Tokens::new(label)), rest)
  })
}

/// Whether `tokens` closes every group it opens, none before it opens it.
fn balanced(tokens: &[Token]) -> bool {
  let mut depth = 0usize;
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => return false,
      Catcode::END => depth -= 1,
      _ => {},
    }
  }
  depth == 0
}

/// The commands a line of addresses alone prints them with, beside the font and size commands
/// ([`NAME_TEXT_COMMANDS`], [`FONT_SWITCH`]) and the spacing ([`is_spacing`], [`is_spacing_command`],
/// [`is_line_separator`]): the url and link commands, the printed braces, underscore and bar of a list of local parts
/// ([`email_addresses`]), and the boxes and no-ops round them.
const ADDRESS_LINE_COMMANDS: &[&str] = &[
  "\\url",
  "\\href",
  "\\nolinkurl",
  "\\path",
  "\\{",
  "\\}",
  "\\_",
  "\\lbrace",
  "\\rbrace",
  "\\textbraceleft",
  "\\textbraceright",
  "\\textunderscore",
  "\\mid",
  "\\vert",
  "\\mathtt",
  "\\texttt",
  // (a text command printing its argument: `\text{sgadgil@stanford.edu}`, 2011.11850; `\verb"ndr@…"`, astro-ph0002160)
  "\\text",
  "\\verb",
  "\\protect",
  "\\relax",
  "\\mbox",
  "\\hbox",
  "\\par",
  "\\noindent",
  "\\quad",
  "\\smallskip",
  "\\medskip",
  "\\bigskip",
];

/// Whether `line` holds no command but those a line of addresses alone prints them with ([`ADDRESS_LINE_COMMANDS`], the
/// font, size and spacing commands; a command whose arguments print no letter, `\color{…}`, `\hspace{…}`, aside:
/// [`without_non_printing_arguments`]). Any other — `\orcidID{…}` (2308.02269), `\thanks{…}`, `$^\dagger$` — prints
/// more than the addresses, and the line stays in its entry as it prints (review rev100b B1).
fn prints_addresses_only(line: &Tokens) -> bool {
  without_non_printing_arguments(line.unlist_ref())
    .iter()
    .all(|t| {
      t.get_catcode() != Catcode::CS
        || is_spacing(t)
        || is_spacing_command(t)
        || is_line_separator(t)
        || t.with_str(|s| {
          NAME_TEXT_COMMANDS.contains(&s)
            || FONT_SWITCH.is_match(s)
            || ADDRESS_LINE_COMMANDS.contains(&s)
        })
    })
}

/// A line led by a printed mark (`$^1$…`, `\textsuperscript{1}…`, the font switches before it kept: `{\tt $^1$\{aable,
/// ccole\}@a.edu}`, a group round the whole line read inside), parted into the mark and what follows it, each in the
/// line's group: `None` for a line no printed mark leads (`relocate::leading_line_label`; a number typed by hand is no
/// mark here).
fn mark_led_addresses(line: &Tokens) -> Option<(Vec<Token>, Tokens)> {
  let v = line.unlist_ref();
  let grouped = v.len() >= 2 && v[0].get_catcode() == Catcode::BEGIN && skip_group(v, 0) == v.len();
  let inner = if grouped { &v[1..v.len() - 1] } else { v };
  let (_, typed, rest) = leading_line_label(&Tokens::new(inner.to_vec()))?;
  if typed {
    return None;
  }
  // (the rest is the switches before the mark and the tokens after it, those a tail of the line)
  let lead = inner
    .iter()
    .take_while(|t| {
      **t == T_SPACE!()
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| MARK_SWITCHES.contains(&s)))
    })
    .count();
  let tail = &rest.unlist_ref()[lead.min(rest.unlist_ref().len())..];
  let mark = &inner[..inner.len().checked_sub(tail.len())?];
  let wrap = |tokens: Vec<Token>| {
    if grouped {
      let mut out = vec![v[0]];
      out.extend(tokens);
      out.push(v[v.len() - 1]);
      out
    } else {
      tokens
    }
  };
  let mut address = inner[..lead].to_vec();
  address.extend_from_slice(tail);
  Some((wrap(mark.to_vec()), Tokens::new(wrap(address))))
}

/// The content of `content` inside the one-argument wrapper `cs` round all of it (`\lx@affiliation@withsup{…}`), and
/// whether it was.
fn within_wrapper(content: &Tokens, cs: &str) -> (bool, Tokens) {
  let v = content.unlist_ref();
  if v.len() >= 3
    && v[0] == T_CS!(cs)
    && v[1].get_catcode() == Catcode::BEGIN
    && skip_group(v, 1) == v.len()
  {
    (true, Tokens::new(v[2..v.len() - 1].to_vec()))
  } else {
    (false, content.clone())
  }
}

/// The content of a contact inside the affiliation's own wrapper (`\lx@affiliation@withinst{…}`, which
/// `\lx@add@affiliation` puts around it), and whether it was in it.
fn within_inst(content: &Tokens) -> (bool, Tokens) {
  let within = content.unlist_ref();
  let withinst = within.len() >= 3
    && within[0] == T_CS!("\\lx@affiliation@withinst")
    && within[1].get_catcode() == Catcode::BEGIN
    && skip_group(within, 1) == within.len();
  if withinst {
    (true, Tokens::new(within[2..within.len() - 1].to_vec()))
  } else {
    (false, content.clone())
  }
}

/// The attribute of an entry of the Cut ([`cut_marked_contact`]) naming the printed list it came from
/// ([`cut_list_key`]): a copy of the list queued again adds no entries ([`cut_list_queued`]), and an entry relocated to
/// an author keeps the place the author's own uncut copy had (`relocate::at_queue_position`, [`CUT_COPY_PLACE`]).
pub(in crate::base_utilities) const CUT_LIST: &str = "_cutlist";

/// The state key prefix of the place (queue ordinal) an author's uncut copy of a cut list held, `{CUT_COPY_PLACE}{key}:
/// author:{n}` ([`place_uncut_copies`], `\lx@cut@copy@place`).
pub(in crate::base_utilities) const CUT_COPY_PLACE: &str = "lx_cut_copy_place:";

/// The tag of the queue entry that stands where an uncut copy of a cut list was ([`place_uncut_copies`]): it makes no
/// element, only records the copy's place.
const CUT_COPY_TAG: &str = "lx:cut-copy";

/// The key of a printed list the Cut reads (`list`, its marks written as the plain marks they print,
/// [`printed_marks_as_marks`]) of a contact of `role` annotating `parenttag`: two calls holding the same list have the
/// same key.
fn cut_list_key(parenttag: &Tokens, role: &str, list: &Tokens) -> String {
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

/// Does the queue hold the entries of the printed list `key`, cut already ([`CUT_LIST`])?
fn cut_list_queued(key: &str) -> bool {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue
      .iter()
      .any(|entry| entry.1.get(CUT_LIST).map(String::as_str) == Some(key)),
    _ => false,
  })
}

/// The queued copies of the printed list `key` (of a contact of `role` annotating `parenttag`, its entries led by the
/// marks of `list`) that the Cut left uncut — queued while too few names were queued to request two of its marks
/// (amsart's `\author{Ann Able${}^{1}$}\address{${}^{1}$Univ A\\ ${}^{2}$Univ B}\author{Bob Baker${}^{2}$}` and the same
/// `\address` again: review rev99a2 d2; 2312.02779's REVTeX `\affiliation` after each name) — give way to its entries:
/// each is taken out of the queue when every name it reaches requests one of the list's marks, so each of them takes
/// its entry by its mark instead of the whole list; an entry stands in its place that records it for those names, so
/// the entry each takes keeps the copy's place among its contacts (user ruling R8, print order:
/// `relocate::at_queue_position`). The names a copy reaches are its run's — those queued after the last contact of its
/// role before it, which covers one placed on the name before it and a run's (`annotate=run`, `new`) — or every name
/// queued before it for one annotating them all (`annotate=all`, a number); a copy reaching a name that requests none of
/// the list's marks, or no name, stays as it is (no name loses the list it prints).
fn place_uncut_copies(parenttag: &Tokens, role: &str, key: &str, list: &Tokens) {
  let marks = requested_entry_marks(list);
  let requests = |operands: &[Vec<Token>]| {
    marks
      .iter()
      .any(|mark| !requesting_authors(&[operands.to_vec()], mark).is_empty())
  };
  let queue: Vec<RawFrontmatter> = with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => queue.clone(),
    _ => Vec::new(),
  });
  let mut removed: Vec<Tokens> = Vec::new();
  let mut places: Vec<(usize, Tokens)> = Vec::new();
  {
    let mut authors: Vec<(usize, usize, Vec<Vec<Token>>)> = Vec::new();
    let mut last_of_role: Option<usize> = None;
    for (at, (tag, attr, command)) in queue.iter().enumerate() {
      if tag == "ltx:creator" && attr.get("role").map(String::as_str) == Some("author") {
        authors.push((
          at,
          authors.len() + 1,
          author_mark_operands(command.unlist_ref()),
        ));
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
      let reach: Vec<&(usize, usize, Vec<Vec<Token>>)> = authors
        .iter()
        .filter(|(position, ..)| all || since.is_none_or(|since| *position > since))
        .collect();
      if reach.is_empty() || !reach.iter().all(|(.., operands)| requests(operands)) {
        continue;
      }
      let names: Vec<String> = reach.iter().map(|(_, n, _)| n.to_string()).collect();
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
    return;
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
  if fm_shadow_enabled() {
    for content in removed {
      record_cut_copy(role, &content);
    }
  }
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
  let list = spaced_marks_written(&list, true).unwrap_or(list);
  // (`\inst` by its meaning, the one-argument mark it prints: `separators::inst_prints_its_mark`)
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
    t.get_catcode() == Catcode::CS
      && !is_mark_command(t)
      && !LINE_BOXES.iter().any(|cs| *t == T_CS!(*cs))
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
    part.iter().any(|t| {
      *t == T_OTHER!("@")
        || ["\\href", "\\url", "\\email", "\\mailto"]
          .iter()
          .any(|cs| *t == T_CS!(*cs))
    })
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

/// The frontmatter API's Cut decision (`cut_marked_contact`), the one predicate every reader of a store's marked
/// entries asks: does `list` hold two or more entries, cut as [`affiliation_calls`] cuts them, led by marks of distinct
/// labels that queued names request (`requesting_authors`), its first printed line led by a mark (`marker_leads`) —
/// its marks read as the Cut reads them, whatever their spelling (`printed_marks_as_marks`)?
pub fn holds_requested_marked_entries(list: &Tokens) -> bool {
  requested_marked_entries_in(&printed_marks_as_marks(list.clone()))
}

/// [`holds_requested_marked_entries`] of a list whose marks are written as the plain marks they print.
fn requested_marked_entries_in(list: &Tokens) -> bool { requested_entry_marks(list).len() >= 2 }

/// The marks, of distinct labels, leading the entries of `list` (its marks written as the plain marks they print) that
/// queued names request — none when its first printed line is not led by a mark ([`requested_marked_entries_in`]).
fn requested_entry_marks(list: &Tokens) -> Vec<Vec<Token>> {
  let authors = queued_author_marks();
  if authors.is_empty() {
    return Vec::new();
  }
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
        && !requesting_authors(&authors, &mark).is_empty()
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
  for operand in queued_author_marks().into_iter().flatten() {
    for piece in Tokens::new(operand).to_string().split(',') {
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
      // `email_owners::entry_address_owners`, OXIDIZED_DESIGN_DIVERGENCES #956; it no longer stays in the entry's text,
      // 2609.19448's one address under one author's affiliation still that author's)
      Some((entry, at)) if under_affiliations => {
        let requesting = entry_mark(&entries[entry])
          .map_or_else(Vec::new, |mark| requesting_authors(&authors, &mark));
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
  /// Each address its own contact, with no placement of its own: an address line of an affiliation-kind entry, digested
  /// inside it, the entry's address — its owners the entry's Link step gives ([`kind_entry_addresses`],
  /// `email_owners::entry_address_owners`).
  Inner,
}
