//! Author-block affiliations: the `\lx@add@affiliation` calls, address runs and their placement, contact owners.

use super::*;

/// The `\lx@add@affiliation` calls for an affiliation list: one per line or `\and`-separated
/// entry, each with its superscript marker as its label when the list has markers (Perl
/// Base_Utility.pool.ltxml:742-753, the body of `\lx@add@affiliations` after its dequeue). `attr`
/// is passed to each entry, `marked_attr` instead when the list has markers and `marks_are_labels`.
/// `authors_queued` says every author is queued, so an unmarked line may be placed by the marks: an
/// address run by its authors' positions, another line with the affiliation before it. An `\author`
/// call's tail (`author_tail_calls`) may come before later authors, and then its unmarked lines are
/// its own creators' rows, as each was before.
pub fn affiliation_calls(
  attr: Option<Tokens>,
  marked_attr: Option<Tokens>,
  stuff: Tokens,
  marks_are_labels: bool,
  authors_queued: bool,
) -> Result<Vec<Token>> {
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
      if authors_queued && author_mark_position(&line).is_none() && line_is_email_list(&line) {
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
      calls.extend(
        Invocation!(T_CS!("\\lx@add@affiliation"), vec![
          attr.clone(),
          Some(line)
        ])
        .unlist(),
      );
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
    let withsup = Invocation!(T_CS!("\\lx@affiliation@withsup"), vec![Some(Tokens::new(
      entry
    ))]);
    calls.extend(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        marked_attr.clone(),
        Some(withsup)
      ])
      .unlist(),
    );
  }
  calls.extend(email_calls);
  Ok(calls)
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
      .map(|(_, line)| email_addresses(line).map_or(0, |a| a.len()))
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
        for (_, line) in run.lines {
          calls.extend(email_line_calls(line, AddressPlacement::Shared)?);
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
