//! Author-block line kinds: apart and row line calls, welded layouts, symbol-labelled institutions, URL contacts.

use super::*;

/// Line role within a superscript-labeled author block (see `\lx@add@authors`).
#[derive(Clone, Copy, PartialEq)]
pub(in crate::base_utilities) enum AuthorLineKind {
  Author,
  Affiliation,
  Email,
  /// An email line behind a label or the class's email command (`Email: \email{x}`, [`labelled_emails`]).
  LabelledEmail,
  /// A line that is only a link ([`is_link_line`]): a URL contact, or an affiliation when the link prints an institution
  /// ([`is_url_line`], [`url_contact_call`]).
  Link,
}

/// The creator a link line of an author block goes to ([`url_contact_call`]).
#[derive(Clone, Debug, PartialEq)]
pub(in crate::base_utilities) enum LinkOwner {
  /// the whole block's (`label=addresses:block`, the shared creator of OXIDIZED_DESIGN #159; a sole author's)
  Block,
  /// the author being made, the last of the line's group
  Current,
  /// the line's group of that many names, as the group's affiliation lines are (`annotate=N`)
  Group(usize),
  /// the author a label of its own names (`label=…`, [`placed_label`]), given once the block's marked affiliations are:
  /// a marked block's link after them in its contacts, as the column prints it (the r5 re-review's u14)
  Author(String),
}

/// Whose a link line of a marked author block is ([`url_contact_call`]): in an `\and` group of authors while other
/// groups hold authors, that group's (its one author's, or its names' as its affiliation lines are), the link printed in
/// that column; else the block's when the call makes
/// the whole author list (`replace`, the document's `\author`), the project's page of 2605.06223, 2602.08439,
/// 2604.12896 (user ruling 2026-10-20, batch 77p r7); else, a per-author class's call, its author's. `kinds` are the
/// block's entries, `group_starts` where each group's begin, `index` the line's entry.
pub(in crate::base_utilities) fn link_line_owner(
  kinds: &[AuthorLineKind],
  group_starts: &[usize],
  index: usize,
  replace: bool,
) -> LinkOwner {
  let (start, end) = group_bounds(group_starts, kinds.len(), index);
  let authors = |kinds: &[AuthorLineKind]| {
    kinds
      .iter()
      .filter(|kind| **kind == AuthorLineKind::Author)
      .count()
  };
  let here = authors(&kinds[start..end.max(start)]);
  if here == 1 && authors(kinds) > here {
    LinkOwner::Current
  } else if here > 1 && authors(kinds) > here {
    // (a column of several names: theirs, as the column's affiliation lines are, `Ann Able$^1$, Bob Baker$^1$ \\
    // $^1$Univ A \\ \url{…} \and Cat Cole$^2$ …`, the r5 re-review's u6m; ruling 9)
    LinkOwner::Group(here)
  } else if replace {
    LinkOwner::Block
  } else {
    LinkOwner::Current
  }
}

/// The calls placing a line under a row of names parted by space whose footnote symbols label the institutions under
/// them (a group of the unmarked branch kept from the names before it by its lines' marks, and one of its lines an
/// institution a symbol labels, [`labelled_institution`]: `Dinghong Song$^{\dagger}$ \ \ \ \ Zhiwei Xu$^\mathsection$ …
/// Dong Li$^{\dagger}$ \\ $^\mathsection$ Tsinghua University \ \ \ \ $^\dagger$ University of California, Merced \\
/// \texttt{dinghongsong21@gmail.com}`, 2601.02680, 2505.23260, 2608.12337, the r5 re-review's r10, r11, r14, r16, r18):
/// the lines print under the whole row, not under its last name. `row` is the row's names (indices into `name_keys`),
/// `name_symbols` the symbols each name of it carries (none for a name of another row: an `\and` column's carriers are
/// no part of it, the r6 re-review's x6, w28). Each address of an e-mail line goes to the name it spells anywhere in
/// the block, and one no name spells to the block (`label=addresses:block`, the shared creator of #159) — no
/// elimination; a link line is the block's; an institution a symbol labels goes to every name carrying it
/// (`label=author:N`), without the symbol, each such institution of the line its own (`$^\mathsection$ Tsinghua
/// University \ \ \ \ $^\dagger$ …`). An unmarked line goes on with the institution above it only where that line is
/// one such institution alone, ending in a comma, and it reads as an institution too (`continuing`, the carriers of
/// that one: `$^{\dagger}$State Key Laboratory …, \\ School of Computer Science, Peking University`, 2608.12337); an
/// unmarked institution otherwise is every name's of the row, as it prints under the whole row (user ruling R12
/// 2026-10-20: `… \\ Department of Physics`, the r6 re-review's y8); a legend goes to the names carrying its symbol;
/// any other piece — a note, a place, a symbol no name of the row carries, an address line no list reads — is the
/// block's (`Equal contribution`, the r6 re-review's y7; 2607.16926's address line under all five names) —
/// [`row_line_calls`]. None lands on one name for its place (batch 77p r9, r10). A line that prints nothing is none.
pub(in crate::base_utilities) fn apart_line_calls(
  line: &Tokens,
  name_keys: &[NameKeys],
  name_symbols: &[Vec<String>],
  row: std::ops::RangeInclusive<usize>,
  placed: &mut Vec<Option<String>>,
  bodies: &mut [Vec<Token>],
  continuing: &mut Continuing,
) -> Result<Vec<Token>> {
  // (a name's label of its own, given to it in its content, [`placed_label`])
  let mut author = |owner: usize| -> Result<String> {
    let (label, call) = placed_label(placed, owner)?;
    if let Some(body) = bodies.get_mut(owner) {
      body.extend(call.unwrap_or_default());
    }
    Ok(s!("label={label}"))
  };
  let block = s!("label=addresses:block");
  let mut calls = Vec::new();
  if prints_nothing(line) {
    return Ok(calls);
  }
  // (the carriers the line above leaves this one to go on with: none unless set again below)
  let above = std::mem::take(&mut continuing.carriers);
  // (a mark after an address is no letter of it: `\texttt{mazeyuwhu@gmail.com}$^{\dagger}$`, 2109.13296, the address
  // of the name it spells; the r7 re-review's findings 1 and 9)
  let (addresses_line, trailing_carriers) = match before_trailing_symbol_mark(line.unlist_ref()) {
    Some((text, marks)) => (
      Tokens::new(text.to_vec()),
      symbol_carriers(&symbol_labels(&Tokens::new(marks.to_vec())), name_symbols),
    ),
    None => (line.clone(), Vec::new()),
  };
  if let Some((label, addresses)) = line_emails(&addresses_line)? {
    // (a line no address of which a name spells goes to the names carrying its marks, the marks no part of its addresses
    // — [`marked_email_line`] for those that lead it; main-session decision 3 on the 77p r11 questions, the r8
    // re-review's finding 6, probes s10, t4 —, else to the block; a spelled address keeps the mark it shows, 65e)
    let unspelled = addresses
      .iter()
      .all(|address| spelled_owner(name_keys, &address_text(&address.address)).is_none());
    let (label, addresses, carriers) = match (unspelled, marked_email_line(line, name_symbols)?) {
      (true, Some((stripped, carriers))) => match line_emails(&stripped)? {
        Some((label, addresses)) => (label, addresses, carriers),
        None => (label, addresses, Vec::new()),
      },
      (true, None) => (label, addresses, trailing_carriers),
      (false, _) => (label, addresses, Vec::new()),
    };
    for address in addresses {
      let owners = match spelled_owner(name_keys, &address_text(&address.address)) {
        Some(owner) => vec![owner],
        None => carriers.clone(),
      };
      if owners.is_empty() {
        calls.extend(email_calls(label.as_ref(), vec![address], Some(&block))?);
      } else {
        for owner in owners {
          let placement = author(owner)?;
          calls.extend(email_calls(
            label.as_ref(),
            vec![EmailAddress {
              address: address.address.clone(),
              call:    address.call.clone(),
            }],
            Some(&placement),
          )?);
        }
      }
    }
    return Ok(calls);
  }
  if is_url_line(line) {
    return url_contact_call(line.clone(), LinkOwner::Block, true);
  }
  calls.extend(row_line_calls(
    line,
    name_symbols,
    row,
    placed,
    bodies,
    &above,
    continuing,
  )?);
  Ok(calls)
}

/// An e-mail line's label and addresses: a labelled one's ([`line_labelled_emails`]), else a bare one's
/// ([`bare_email_addresses`]); `None` for any other line.
pub(in crate::base_utilities) fn line_emails(
  line: &Tokens,
) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  Ok(match line_labelled_emails(line)? {
    Some(found) => Some(found),
    None => bare_email_addresses(line)?.map(|addresses| {
      (
        None,
        addresses
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
          .collect(),
      )
    }),
  })
}

/// `line` without the footnote symbol marks that lead it ([`after_symbol_mark`], each of a run).
pub(in crate::base_utilities) fn without_leading_symbol_marks(line: &Tokens) -> Tokens {
  let mut after_marks: &[Token] = line.unlist_ref();
  while after_symbol_mark(after_marks).len() < after_marks.len() {
    after_marks = after_symbol_mark(after_marks);
  }
  Tokens::new(after_marks.to_vec())
}

/// An e-mail line its footnote symbol marks lead (`$^\dagger$zq77@x.edu`), when names of the row (`name_symbols`, the
/// symbols each carries, none for a name of another row) carry them: the line without them, and those names, whose an
/// address of it no name spells is (main-session decision 3 on the 77p r11 questions, as a marked line ties an
/// affiliation to its carriers, 63i; the r8 re-review's finding 6, probes s10, t4; as the numbered branch places one,
/// [`AddressPlacement::Marked`]: 2605.01818). `None` for any other line; a mark no name carries stays, as it prints.
pub(in crate::base_utilities) fn marked_email_line(
  line: &Tokens,
  name_symbols: &[Vec<String>],
) -> Result<Option<(Tokens, Vec<usize>)>> {
  let stripped = without_leading_symbol_marks(line);
  if stripped.unlist_ref().len() == line.unlist_ref().len() {
    return Ok(None);
  }
  let carriers = symbol_carriers(&leading_symbol_labels(line), name_symbols);
  Ok(
    (!carriers.is_empty()
      && (line_labelled_emails(&stripped)?.is_some()
        || bare_email_addresses(&stripped)?.is_some()))
    .then_some((stripped, carriers)),
  )
}

/// The names a line under a row of symbol-marked names goes on with, from the line above it ([`row_line_calls`]).
pub(in crate::base_utilities) struct Continuing {
  /// the carriers of the institution above, which the line goes on with; none unless set again by the line
  pub(in crate::base_utilities) carriers:      Vec<usize>,
  /// whether the group's labelled institutions run on over unmarked lines ([`welded_layout`])
  pub(in crate::base_utilities) welded:        bool,
  /// the group's lines holding an institution a symbol labels yet to come ([`labelled_line`]): past the last, a row
  /// with an unmarked name welds no line (user ruling R12)
  pub(in crate::base_utilities) labelled_left: usize,
}

/// Whether `line` holds an institution a footnote symbol of the names (`name_symbols`) labels
/// ([`labelled_institution`]).
pub(in crate::base_utilities) fn labelled_line(
  line: &Tokens,
  name_symbols: &[Vec<String>],
) -> bool {
  symbol_led_segments(line)
    .iter()
    .any(|segment| labelled_institution(segment, name_symbols).is_some())
}

/// Whether an unmarked line may go on with the labelled institution above it where a group welds them
/// ([`welded_layout`]): it prints, holds no address (`@`) and is no link line, is no note — correspondence, a
/// contribution, an address, funding ([`classify_thanks`]) — and reads as an institution ([`reads_as_institution`]:
/// `Ball State University`) or a place ([`reads_as_place`]: `Muncie, IN 47306`), 2602.05059. Not `Corresponding Author:
/// \texttt{…}` (2604.02904), `Email: (…)@tu-bs.de, boche@tum.de` (2602.03421), `Preprint. Under review.`, `Work done
/// during an internship at Google`: the 77y review's finding 3, probes c1, c3, x4.
pub(in crate::base_utilities) fn weldable_line(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  !prints_nothing(line)
    && !text.contains('@')
    && !is_link_line(line)
    && classify_thanks(&text) == "note"
    && (reads_as_institution(line) || reads_as_place(line))
}

/// Whether `line` reads as a place: no name ([`name_count`]), no address, label (`:`) or note ([`classify_thanks`]),
/// parts a comma parts, each word of them opening with a capital or a digit, or a particle ([`NAME_PARTICLES`])
/// (`Muncie, IN 47306`, `Braunschweig, Germany`, `Hong Kong SAR, China`, `Campus de Santiago, 3810-183 Aveiro,
/// Portugal`, 2605.19581).
fn reads_as_place(line: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim();
  text.contains(',')
    && !text.contains(['@', ':'])
    && name_count(text) == 0
    && classify_thanks(text) == "note"
    && text.split_whitespace().all(|word| {
      let word = word.trim_matches(|c: char| !c.is_alphanumeric());
      word.is_empty()
        || NAME_PARTICLES.contains(&word)
        || word
          .chars()
          .next()
          .is_some_and(|c| c.is_uppercase() || c.is_ascii_digit())
    })
}

/// Whether the lines under a row of symbol-marked names (`affils`, the names' symbols `name_symbols`) set each labelled
/// institution over unmarked lines of its own: an unmarked line that may go on with one ([`weldable_line`]: an
/// institution or a place) stands between two lines holding an institution a symbol labels ([`labelled_line`]) —
/// `$^\dagger$Department of Computer Science \\ Ball State University \\ Muncie, IN 47306 \\ … \\ $^\ddagger$School of
/// Artificial Intelligence …`, 2602.05059 (the r8 re-review's finding 5, probe t7). There such an unmarked line goes on
/// with the institution above it, as a numbered block welds it (x12, main-session decision 3 on the 77p r10 questions)
/// — a note, an address or a link line not (the 77y review's finding 3: 2604.02904, 2602.03421), nor, in a row with an
/// unmarked name, a line past the last labelled institution, which is every name's of the row (R12; probe c4); else
/// an unmarked institution under the last labelled one is every name's of the row (user ruling R12: `$^\dagger$Univ A
/// \\ $^\ddagger$Univ B \\ Department of Physics`, probe y8).
pub(in crate::base_utilities) fn welded_layout(
  affils: &[Tokens],
  name_symbols: &[Vec<String>],
) -> bool {
  let (mut seen_labelled, mut unmarked_since) = (false, false);
  for line in affils {
    if labelled_line(line, name_symbols) {
      if unmarked_since {
        return true;
      }
      seen_labelled = true;
    } else if seen_labelled && symbol_marks(line).is_empty() && weldable_line(line) {
      unmarked_since = true;
    }
  }
  false
}

/// The calls placing a line under a row of names that is no address and no link ([`apart_line_calls`]; and a line
/// under a row whose only symbol line is a legend, 2609.24299, 2603.06578): each symbol-led piece of it
/// ([`symbol_led_segments`]) that is an institution its symbol labels to the row's names carrying the symbol
/// (`name_symbols`, by index, none for a name of another row), without it; a legend (`$^*$Corresponding author: …`) to
/// the names carrying its symbol, whole, its mark naming them; an unmarked institution ([`reads_as_institution`]) to
/// the carriers of the line above when that was one labelled institution alone ending in a comma (`above`), else to
/// every name of the `row` (user ruling R12 2026-10-20); anything else — a note, a place, a symbol no name of the row
/// carries — to the block (`label=addresses:block`, the shared creator of #159), never to one name for its place. Where
/// the group's labelled institutions run on over unmarked lines (`continuing.welded`, [`welded_layout`]), an unmarked
/// institution or place ([`weldable_line`]) goes on with the carriers of the line above (2602.05059) — a note, an
/// address or a link line not (2604.02904, 2602.03421) — and past the group's last labelled institution only where
/// every name of the row carries a mark: under a row with an unmarked name an unmarked line under it goes on with it
/// only as one under an institution ending in a comma does, else it is every name's of the row (R12, literally; the
/// 77y review's finding 3, probe c4; the rev77y2 re-review's finding 1, probe w1). `continuing` is set to the carriers
/// a line under this one goes on with.
pub(in crate::base_utilities) fn row_line_calls(
  line: &Tokens,
  name_symbols: &[Vec<String>],
  row: std::ops::RangeInclusive<usize>,
  placed: &mut Vec<Option<String>>,
  bodies: &mut [Vec<Token>],
  above: &[usize],
  continuing: &mut Continuing,
) -> Result<Vec<Token>> {
  let mut calls = Vec::new();
  let mut affiliation = |owner: Option<usize>, content: Tokens| -> Result<Vec<Token>> {
    let placement = match owner {
      Some(owner) => {
        // (a name's label of its own, given to it in its content, [`placed_label`])
        let (label, call) = placed_label(placed, owner)?;
        if let Some(body) = bodies.get_mut(owner) {
          body.extend(call.unwrap_or_default());
        }
        s!("label={label}")
      },
      None => s!("label=addresses:block"),
    };
    Ok(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        Some(mouth::tokenize_internal(TeXString::assembled(placement))),
        Some(content)
      ])
      .unlist(),
    )
  };
  let segments = symbol_led_segments(line);
  let alone = segments.len() == 1;
  if labelled_line(line, name_symbols) {
    continuing.labelled_left = continuing.labelled_left.saturating_sub(1);
  }
  // (a line past the group's last labelled institution welds on only where every name of the row carries a mark: one
  // under a row with an unmarked name is every name's of the row, R12 — the 77y review's finding 3, probe c4)
  let welds = continuing.welded
    && (continuing.labelled_left > 0
      || row.clone().all(|at| {
        name_symbols
          .get(at)
          .is_some_and(|symbols| !symbols.is_empty())
      }));
  for segment in segments {
    // (a mark anywhere in it, a group around it or after its text, makes it no unmarked line: the r7 re-review's
    // finding 1, 2606.28166, 2602.16540, 2109.13296)
    let unmarked = symbol_marks(&segment).is_empty();
    let (owners, content): (Vec<usize>, Tokens) = match labelled_institution(&segment, name_symbols)
    {
      Some((carriers, rest)) => {
        // (one institution alone on its line, ending in a comma, goes on on the line under it; so does any, where the
        // group's labelled institutions run on over unmarked lines, [`welded_layout`], but the last of them in a row
        // with an unmarked name, `welds`: `$^\ddagger$School of Law, University B \\ Department of Physics, University
        // C` gives the last line to every name of the row, R12 — the rev77y2 re-review's finding 1, probe w1)
        if alone
          && (welds
            || visible_name_text(rest.unlist_ref())
              .trim_end()
              .ends_with(','))
        {
          continuing.carriers = carriers.clone();
        }
        (carriers, rest)
      },
      // (an unmarked line under a labelled institution whose group runs its institutions on over unmarked lines goes on
      // with it, a place too, and so does the line under it: `$^\dagger$Department of Computer Science \\ Ball State
      // University \\ Muncie, IN 47306`, 2602.05059 — the x12 weld of a numbered block, main-session decision 3 on
      // the 77p r10 questions, applied to symbol rows; the r8 re-review's finding 5, probe t7) — an institution or a
      // place, no note, address or link line (`Corresponding Author: \texttt{…}`, 2604.02904; `Email: (…)@tu-bs.de,
      // boche@tum.de`, 2602.03421: the 77y review's finding 3)
      None if unmarked && alone && welds && !above.is_empty() && weldable_line(&segment) => {
        continuing.carriers = above.to_vec();
        (above.to_vec(), segment)
      },
      // (and an unmarked piece before the labelled institutions of its line, no note and no address, is the
      // institution of the names no mark sends elsewhere, as it prints in their row: `Uber Technologies \ \ \
      // $^\dagger$University of British Columbia`, 2603.07345 — 77p r11; not one `\quad` parts onto a line of its own,
      // which reads alone: 2509.20478's `UC Berkeley \quad $^\dagger$\unskip Princeton University`)
      None
        if unmarked
          && (reads_as_institution(&segment)
            || link_prints_an_institution(&segment)
            || (!alone && institution_beside_labelled(&segment))) =>
      {
        if alone && !above.is_empty() {
          (above.to_vec(), segment)
        } else {
          (row.clone().collect(), segment)
        }
      },
      // (a marked piece no institution reads: the names carrying every mark of it, else the block's — never the row's)
      None if !unmarked => (
        symbol_carriers(&symbol_labels(&segment), name_symbols),
        segment,
      ),
      None => (Vec::new(), segment),
    };
    if owners.is_empty() {
      calls.extend(affiliation(None, content)?);
    } else {
      for owner in owners {
        calls.extend(affiliation(Some(owner), content.clone())?);
      }
    }
  }
  Ok(calls)
}

/// Whether an unmarked piece of a line of symbol-led pieces ([`symbol_led_segments`]) reads as their fellow
/// institution: letters, no address, no note — correspondence, a contribution, funding ([`classify_thanks`]) — and no link
/// line (`Uber Technologies` beside `$^\dagger$University of British Columbia`, 2603.07345).
fn institution_beside_labelled(segment: &Tokens) -> bool {
  let text = visible_name_text(segment.unlist_ref());
  has_name_letter(segment.unlist_ref())
    && !text.contains('@')
    && classify_thanks(&text) == "note"
    && !is_link_line(segment)
}

/// Whether the institution `under` goes on with the words of `line` above it as one name of one institution, no
/// institution of its own under a line of names: it is one generic institution noun ([`NON_NAME_WORDS`]: `Shanghai Jiao
/// Tong \\ University`), opens with a connective (`… \\ of Science and Technology`), or holds every word of the line
/// (`Hong Kong \\ The Hong Kong University of Science and Technology`) — the rev77y2 re-review's finding 4, probes k12,
/// k17. An institution's own name is none (`Cat Cole \\ CERN`, `\\ RIKEN`, `\\ the University of Tokyo`, `\\ eBay
/// Inc.`, `\\ Max-Planck-Institut`: the rev77y3 re-review's finding 1, probes k20, k26, k31, k40-k42; rev77y4's, c2,
/// c5). A name over a one-word generic noun or a connective-led institution of its own (`Cat Cole \\ Physics`, `\\ at
/// Univ C`) is read as the line's continuation — a SYNC residual (rev77y4 probes c1, c3, c4, c8, k27, k30).
pub(in crate::base_utilities) fn continues_the_line(line: &Tokens, under: &Tokens) -> bool {
  let text = visible_name_text(line.unlist_ref());
  let under = visible_name_text(under.unlist_ref());
  let words: Vec<&str> = under.split_whitespace().collect();
  // (the whole word, never a hyphen part of an institution's own name: `Max-Planck-Institut`, `Helmholtz-Zentrum` — the
  // rev77y4 re-review's finding 1, probes c2, c5)
  let generic_noun = |word: &str| {
    word_forms(word)
      .next()
      .is_some_and(|bare| NON_NAME_WORDS.contains(&bare.as_str()))
  };
  matches!(words.as_slice(), [word] if generic_noun(word))
    || words
      .first()
      .is_some_and(|word| ["of", "and", "for", "at", "in"].contains(word))
    || text.split_whitespace().all(|word| words.contains(&word))
}

/// Whether `under` is an e-mail line holding an address the names of `line` spell ([`spelled_owners`]): `Cat Cole \\
/// \texttt{ccole@x.edu}` — the rev77y2 re-review's finding 4, probe k15.
pub(in crate::base_utilities) fn addresses_spelling(line: &Tokens, under: &Tokens) -> bool {
  let keys = [NameKeys::of(line)];
  email_addresses(under).is_some_and(|addresses| {
    addresses
      .iter()
      .any(|address| !spelled_owners(&keys, address).is_empty())
  })
}

/// Whether `line` reads as an institution: as an affiliation ([`reads_as_affiliation`]: an institution word, no name,
/// no `@`), and no note — correspondence, a contribution, an address, funding ([`classify_thanks`]).
pub(in crate::base_utilities) fn reads_as_institution(line: &Tokens) -> bool {
  reads_as_affiliation(line) && classify_thanks(&visible_name_text(line.unlist_ref())) == "note"
}

/// The footnote symbols [`symbol_marks`] reads in `tokens`, one each of a run of distinct ones (`$^{\dagger\ddagger}$`:
/// two, 2608.12337), as the labels they request are read ([`clean_frontmatter_labels`]).
pub(in crate::base_utilities) fn symbol_labels(tokens: &Tokens) -> Vec<String> {
  symbol_marks(tokens)
    .iter()
    .flat_map(|mark| clean_frontmatter_labels(mark, ""))
    .collect()
}

/// A piece of a line ([`symbol_led_segments`]) that is an institution its footnote symbol labels: led by a symbol mark
/// some names carry (`name_symbols`), the rest no note — not correspondence, a contribution, an address or funding
/// ([`classify_thanks`]), and no e-mail address (`$^\mathsection$ Tsinghua University`, `$^{\ddagger}$LLM-Core Xiaomi`,
/// 2601.02680, 2608.12337; not `$^*$ Corresponding author`) —: the names carrying the symbol, and the piece without it.
pub(in crate::base_utilities) fn labelled_institution(
  segment: &Tokens,
  name_symbols: &[Vec<String>],
) -> Option<(Vec<usize>, Tokens)> {
  let after = after_symbol_mark(segment.unlist_ref());
  let (rest, marks) = if after.len() == segment.unlist_ref().len() {
    // (a mark after its text labels it as one before it does: `Carnegie Mellon University, …$^{\dagger}$`, 2109.13296;
    // not a text holding marks of its own, a row of marked names, `\textbf{~Pavan Thodima$^{\dagger}$, Aniket
    // Rege$^{\dagger}$}`, 2601.18077, whose last mark is a name's, no line's label)
    let (text, marks) = before_trailing_symbol_mark(segment.unlist_ref())?;
    if !symbol_marks(&Tokens::new(text.to_vec())).is_empty() {
      return None;
    }
    (
      Tokens::new(text.to_vec()),
      symbol_labels(&Tokens::new(marks.to_vec())),
    )
  } else {
    (Tokens::new(after.to_vec()), leading_symbol_labels(segment))
  };
  if marks.is_empty() {
    return None;
  }
  let text = visible_name_text(rest.unlist_ref());
  // (under the names, a symbol-led line is no name: `$^{\ddagger}$LLM-Core Xiaomi` reads as one)
  let institution = reads_as_affiliation(&rest)
    || (classify_thanks(&text) == "note"
      && !text.contains('@')
      && has_name_letter(rest.unlist_ref()));
  let carriers = symbol_carriers(&marks, name_symbols);
  (institution && !carriers.is_empty()).then_some((carriers, rest))
}

/// The labels of the footnote symbol marks that lead `segment` before its text ([`after_symbol_mark`], each mark of a
/// run, [`symbol_marks`]; one each of a compound mark's symbols, [`clean_frontmatter_labels`]: `$^{*\dag}$Project lead:
/// …` gives `*` and `\dag`, 2602.19190).
pub(in crate::base_utilities) fn leading_symbol_labels(segment: &Tokens) -> Vec<String> {
  // (each symbol, not a run whole: [`clean_frontmatter_labels`] gives both)
  let mut labels: Vec<String> = Vec::new();
  let mut v: &[Token] = segment.unlist_ref();
  loop {
    let rest = after_symbol_mark(v);
    if rest.len() == v.len() {
      break;
    }
    let lead = Tokens::new(v[..v.len() - rest.len()].to_vec());
    // (the run whole is the first label of several; a doubled mark, `$^{**}$`, `$^{\dagger\dagger}$`, is one label,
    // its own: the r7 re-review's finding 3, probes n1, n2 — a comma list, `$^{\dagger,\ddagger}$`, has no run whole,
    // its first label a symbol of its own: the r8 re-review's finding 10)
    for label in symbol_marks(&lead).iter().flat_map(|mark| {
      let labels = clean_frontmatter_labels(mark, "");
      let skip = usize::from(
        labels.len() > 1
          && labels
            .first()
            .is_some_and(|run| SYMBOL_COMMAND_RUN.is_match(&run.replace('_', " "))),
      );
      labels.into_iter().skip(skip)
    }) {
      if !labels.contains(&label) {
        labels.push(label);
      }
    }
    v = rest;
  }
  labels
}

/// The names (by index into `name_symbols`, the symbols each carries) carrying every one of `marks`: a line's compound
/// mark names those marked with all its symbols (`$^{*\dag}$Project lead: Yi Yang` is Yi Yang's, `$^{*\dag}$`, not
/// every starred name's, 2602.19190), a single symbol every name carrying it (`$^\dagger$` reaches a name marked
/// `$^{\dagger\ddagger}$`, 2608.12337).
fn symbol_carriers(marks: &[String], name_symbols: &[Vec<String>]) -> Vec<usize> {
  (0..name_symbols.len())
    .filter(|at| !marks.is_empty() && marks.iter().all(|mark| name_symbols[*at].contains(mark)))
    .collect()
}

/// The pieces of `line` each footnote symbol mark ([`keepsup`]) leads, at the top level after a space or spacing
/// (`$^\mathsection$ Tsinghua University \ \ \ \ $^\dagger$ University of California, Merced`: two), without the spacing
/// around them; the line whole when none does.
pub(in crate::base_utilities) fn symbol_led_segments(line: &Tokens) -> Vec<Tokens> {
  // (through a group around the whole line, and at a group a mark opens: `{\small $^{\dagger}$\,University of
  // Toronto \qquad $^{\ddagger}$\,EPFL}`, 2606.28166; the r7 re-review's finding 1, probes g1, g3)
  let v = unwrapped(line.unlist_ref());
  let mut cuts = vec![0];
  let mut depth = 0usize;
  for (i, t) in v.iter().enumerate() {
    let after_space = i > 0 && (v[i - 1] == T_SPACE!() || is_spacing(&v[i - 1]));
    if depth == 0
      && after_space
      && (*t == T_CS!("\\lx@frontmatter@keepsup")
        || (t.get_catcode() == Catcode::BEGIN
          && after_leading_markup(&v[i..]).first() == Some(&T_CS!("\\lx@frontmatter@keepsup"))))
    {
      cuts.push(i);
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
  }
  cuts.push(v.len());
  // (a piece of marks alone is no piece: the mark after a space ends the text before it, which it labels,
  // [`before_trailing_symbol_mark`] — `Univ A $^{\dagger}$`, the r8 re-review's finding 7, probe s3)
  let marks_only = |piece: &[Token]| {
    let mut after = piece;
    while after_symbol_mark(after).len() < after.len() {
      after = after_symbol_mark(after);
    }
    after.len() < piece.len() && spacing_end(after) == 0
  };
  let mut at = 1;
  while at + 1 < cuts.len() {
    if marks_only(&v[cuts[at]..cuts[at + 1]]) {
      cuts.remove(at);
    } else {
      at += 1;
    }
  }
  cuts
    .windows(2)
    .filter_map(|pair| {
      let piece = &v[pair[0]..pair[1]];
      let piece = &piece[leading_spacing_end(piece)..];
      let mut end = spacing_end(piece);
      // (and the run that parts it from the next, `\ \ \ \ `)
      while end > 0 && is_spacing(&piece[end - 1]) {
        end -= 1;
      }
      let piece = unwrapped(&piece[..end]);
      (!piece.is_empty()).then(|| Tokens::new(piece.to_vec()))
    })
    .collect()
}

/// Whether a name's content holds a command that makes its creator itself (`\lx@add@creator`: PoS's `\speaker{Piotr
/// Korcyl}`, 1307.5080), which a label of the `\lx@add@author` around it would not reach ([`placed_label`]): a contact of
/// that name is placed in its content, as the name is made.
pub(in crate::base_utilities) fn makes_its_creator(name: &[Token]) -> Result<bool> {
  for t in name.iter().filter(|t| t.get_catcode() == Catcode::CS) {
    if let Some(def) = lookup_definition(t)?
      && let Some(ExpansionBody::Tokens(body)) = def.get_expansion()
      && body.unlist_ref().first() == Some(&T_CS!("\\lx@add@creator"))
    {
      return Ok(true);
    }
  }
  Ok(false)
}

/// The label of the author `author` of a block (an index among its names) that a contact placed on it after the names
/// are made reaches it by, `labels` holding those given so far: a new one, `author:placed<N>` (counted over the
/// document, so no two blocks share one; `author:` so no misprefixed institute label's number reaches it), given to the
/// author by `\lx@author@placed@label` in its content (`label`, the call to add there, `Some` when new). Not the
/// `author:N` sequence label: a block whose content makes creators of its own (PoS's `\speaker`, 1307.5080) numbers them
/// otherwise.
pub(in crate::base_utilities) fn placed_label(
  labels: &mut Vec<Option<String>>,
  author: usize,
) -> Result<(String, Option<Vec<Token>>)> {
  if labels.len() <= author {
    labels.resize(author + 1, None);
  }
  if let Some(label) = &labels[author] {
    return Ok((label.clone(), None));
  }
  let count = lookup_int("lx_author_placed_labels") + 1;
  assign_value(
    "lx_author_placed_labels",
    Stored::Int(count),
    Some(Scope::Global),
  );
  let label = s!("author:placed{count}");
  labels[author] = Some(label.clone());
  let call = Invocation!(T_CS!("\\lx@author@placed@label"), vec![Some(
    mouth::tokenize_internal(TeXString::assembled(label.clone()))
  )])
  .unlist();
  Ok((label, Some(call)))
}

/// Where the `\and` group of a marked author block's entry `index` begins and ends among its `count` entries
/// (`group_starts`, where each begins).
pub(in crate::base_utilities) fn group_bounds(
  group_starts: &[usize],
  count: usize,
  index: usize,
) -> (usize, usize) {
  let group = group_starts
    .iter()
    .rposition(|start| *start <= index)
    .unwrap_or(0);
  let start = group_starts.get(group).copied().unwrap_or(0);
  let end = group_starts.get(group + 1).copied().unwrap_or(count);
  (start, end.max(start))
}

/// The call making a link line of an author block ([`is_link_line`]) a URL contact (`\lx@add@contact[role=url]`, as a
/// class's `\homepage` is), not an affiliation: the line is its content, a label before the link with it (`Demo
/// Code:~\url{…}`, 2602.09509), in the author block's markup as an affiliation line is (`\lx@affiliation@withinst`);
/// placed as `owner` says (user ruling 2026-10-20, batch 77p r7). A link printing an institution
/// ([`is_url_line`] false: `\href{https://launchsafe.com}{\textbf{LaunchSafe Research}}`, 2606.26479) is an affiliation
/// placed so, the link inside it (`url` false; the r5 re-review's u38).
pub(in crate::base_utilities) fn url_contact_call(
  line: Tokens,
  owner: LinkOwner,
  url: bool,
) -> Result<Vec<Token>> {
  let placement = match owner {
    LinkOwner::Block => s!("label=addresses:block"),
    LinkOwner::Current => String::new(),
    LinkOwner::Group(names) => s!("annotate={names}"),
    LinkOwner::Author(label) => s!("label={label}"),
  };
  if !url {
    return Ok(
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![
        (!placement.is_empty()).then(|| mouth::tokenize_internal(TeXString::assembled(placement))),
        Some(line)
      ])
      .unlist(),
    );
  }
  let keyvals = if placement.is_empty() {
    s!("role=url")
  } else {
    s!("role=url,{placement}")
  };
  let mut content = mouth::tokenize_internal(
    r"\lx@author@markup@begin\expandafter\lx@author@markup@run\expandafter{\lx@author@markup@unprovide}",
  )
  .unlist();
  content.push(T_BEGIN!());
  // (a link that prints nothing is none of it: 2601.02680's `\url{}` after its repository link)
  let v = line.unlist_ref();
  let mut i = 0;
  while i < v.len() {
    if v[i] == T_CS!("\\url") || v[i] == T_CS!("\\href") {
      let (_, (_, after_second)) = link_arguments(v, i);
      let (open, close) = link_printed_argument(v, i);
      let end = if v[i] == T_CS!("\\href") {
        after_second
      } else {
        close
      };
      if v[open..close].iter().all(|t| {
        matches!(
          t.get_catcode(),
          Catcode::BEGIN | Catcode::END | Catcode::SPACE
        )
      }) {
        i = end.max(i + 1);
        continue;
      }
    }
    content.push(v[i]);
    i += 1;
  }
  content.push(T_END!());
  Ok(
    Invocation!(T_CS!("\\lx@add@contact"), vec![
      Some(mouth::tokenize_internal(TeXString::assembled(keyvals))),
      Some(Tokens::new(content))
    ])
    .unlist(),
  )
}
