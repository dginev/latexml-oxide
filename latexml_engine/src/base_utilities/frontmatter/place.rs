//! Contact labels and author order, `digest_front_matter` and `place_frontmatter`.

use super::*;

/// Where the contacts a class lists apart from the names, in the order the document gives them
/// (`\lx@add@contact@placed[owner=sequence]`: jheppub's `\emailAdd`, iopart's `\ead`), queued in `commands`, go: each
/// the label of its creator, or empty to stay where `\lx@add@contact@placed` put it (the group's only author when it was
/// given, else the author block). Read against the authors of the same flush, all of them, wherever the list stands
/// among them (2108.07972's `\emailAdd{kadota.kenji@…}` before the four `\author`s):
/// 1. the author the address spells ([`address_owners`]);
/// 2. else the n-th author, the address at the n-th place — Perl's jheppub.sty.ltxml rule (`labelseq=author`) — when the
///    list is one place for each author and in their order: no address spells an author other than the one at its
///    place, as [`addresses_in_order`] reads an author block's line (1303.3040's `zhl61@` for Zhen Liu, 2011.02499's
///    `tpsg4@`/`tpsr@`). The place is the address's, or else the call's (an `\emailAdd{a, b}` of one author's two
///    addresses). 2407.07862's `eichhorn@` listed first for Astrid Eichhorn, author 2, and 1804.07528's three addresses
///    each one place on are no list in author order. The user ruled the consistent list evidence (2026-10-21);
///    [`author_order_step`] is this step, alone;
/// 3. else where `\lx@add@contact@placed` put it: the group's only author ([`placed_contact_owner`]), or the author
///    block;
/// 4. else, for the one address of the list left on the block of a multi-author flush, the ONE author marked as the
///    corresponding author — by a note naming them so, or a mark whose legend says so —, among the authors of its
///    group and, where the document places its e-mails by address block, those requesting its address's mark
///    ([`marked_corresponding_author`]; user ruling 2026-10-21: 1901.08773's `beril@` A. Kopylov's, 2510.11030's
///    `hjhspace223@` J.-H. Ha's, 2211.07892's `yoomin@` Y. M. Oh's; 2308.06532's `zyxu@` Zhaoyi Xu's, its address
///    saying "(Corresponding author)" and he alone carrying a `*`; 1802.03677's `gear8mike@` stays the block's, the
///    number of its "Corresponding author." note on two authors, and 2111.00753's `lad@`, two legend-less `*`s);
/// 5. a flush with no authors, its list given after `\maketitle` digested them (1903.05554), keeps Perl's rule.
///
/// Step 3, for iopart's `\ead`, places after the spelling with the one author requesting the mark of the address block
/// it is printed under (`mark=address`, `\lx@address@mark@set`: 2407.14305's `rockfish3141@` Hidetomo Hoshino's,
/// 2311.07087's `sgess@` Spencer Gessner's) — weaker than a list in author order, which the PDF's address blocks do not
/// contradict (1902.07874's `\ead{…, b97202056@gmail.com, …}` of five authors after `\address{$^9$ …}`, Che-Yu Chen's
/// at place 3; 1710.02251's three `\mailto`s in an `\eads` after `$^{(2)}$`), and only in a document that writes an
/// e-mail between its addresses: one whose e-mails all follow its addresses writes them where IOP's guidelines ask,
/// "after the postal address(es)", under the last address by position only (0910.0753's `wolfgang.muender@` for W
/// Münder, 1111.3344's `knarf@` for Frank Löffler, 1911.06133's `ledou@` for Pierre Le Doussal, each after the last
/// author's `\address`; `_unmarked`).
///
/// The PDF prints the list as one shared line (jheppub.sty:127-131) or each address as a block under the address before
/// it (iopart.cls:363), so the spelling, the marks and the order are the only evidence (OXIDIZED_DESIGN_DIVERGENCES #805,
/// #806, #808, #809).
pub(in crate::base_utilities) fn sequenced_contact_labels(
  commands: &[RawFrontmatter],
) -> Vec<String> {
  let keys: Vec<NameKeys> = commands
    .iter()
    .filter(|entry| {
      entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
    })
    .map(|entry| NameKeys::of(&queued_name(&entry.2)))
    .collect();
  // (a document that writes an e-mail between its addresses places each under its address, iopart.cls:363; one whose
  // addresses all come first writes its e-mails "after the postal address(es)", as IOP's guidelines ask, under the
  // last address only by position: there an address mark places nothing, `_unmarked`)
  let first_listed = commands
    .iter()
    .position(|entry| entry.0 == "ltx:contact" && entry.1.contains_key("_sequence"));
  let interleaved = first_listed.is_some_and(|first| {
    commands[first..].iter().any(|entry| {
      entry.0 == "ltx:contact" && entry.1.get("role").map(String::as_str) == Some("affiliation")
    })
  });
  // Each address with the class command's call it came from (`\lx@sequence@call`: `\emailAdd{a, b}` is one call of two
  // addresses), the author it spells, whether it is on the block where `\lx@add@contact@placed` put it, and where it
  // goes when its address mark places nothing.
  let listed: Vec<(i64, Option<usize>, bool, Option<&String>)> = commands
    .iter()
    .filter(|entry| entry.0 == "ltx:contact" && entry.1.contains_key("_sequence"))
    .map(|entry| {
      let call = entry
        .1
        .get("_sequence")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
      let spelled = match address_owners(&keys, &last_group(&entry.2)).as_slice() {
        [at] => Some(*at),
        _ => None,
      };
      let unmarked = entry.1.get("_unmarked").filter(|_| !interleaved);
      let placed = unmarked.or_else(|| entry.1.get("label"));
      let blocked = placed.map(String::as_str) == Some("addresses:block");
      (call, spelled, blocked, unmarked)
    })
    .collect();
  let owners: Vec<(i64, Option<usize>)> = listed
    .iter()
    .map(|(call, owner, ..)| (*call, *owner))
    .collect();
  let places = author_order_step(&owners, keys.len());
  let mut labels: Vec<String> = owners
    .iter()
    .enumerate()
    .map(|(at, (_, owner))| match (owner, &places) {
      (Some(owner), _) if !keys.is_empty() => s!("author:{}", owner + 1),
      (_, Some(places)) if keys.is_empty() || places[at] < keys.len() => {
        s!("author:{}", places[at] + 1)
      },
      _ => listed[at].3.cloned().unwrap_or_default(),
    })
    .collect();
  let left: Vec<usize> = (0..listed.len())
    .filter(|at| match labels[*at].as_str() {
      "" => listed[*at].2,
      label => label == "addresses:block",
    })
    .collect();
  if let ([at], true) = (left.as_slice(), keys.len() > 1) {
    // (among the authors it may be of: its author group's, and of those the ones requesting its address's mark, when
    // the document places its e-mails by address block — `_group`, `_marks`, `\lx@add@contact@placed`)
    let entry = commands
      .iter()
      .filter(|entry| entry.0 == "ltx:contact" && entry.1.contains_key("_sequence"))
      .nth(*at);
    let positions = |key: &str| -> Option<Vec<usize>> {
      let value = entry?.1.get(key)?;
      Some(
        value
          .split(';')
          .filter_map(|n| n.trim().parse::<usize>().ok())
          .filter_map(|n| n.checked_sub(1))
          .collect(),
      )
    };
    let group = positions("_group");
    let marks = positions("_marks").filter(|marks| interleaved && !marks.is_empty());
    let within: Option<Vec<usize>> = match (group, marks) {
      (Some(group), Some(marks)) => {
        Some(group.into_iter().filter(|at| marks.contains(at)).collect())
      },
      (Some(only), None) | (None, Some(only)) => Some(only),
      (None, None) => None,
    };
    let address = entry
      .map(|entry| visible_name_text(last_group(&entry.2).unlist_ref()))
      .unwrap_or_default();
    if let Some(author) = marked_corresponding_author(commands, &address, within.as_deref()) {
      labels[*at] = s!("author:{}", author + 1);
    }
  }
  labels
}

/// Step 2 of [`sequenced_contact_labels`], kept apart so a ruling can switch it off in one change: the place (from 0)
/// of each address of `listed` (its call, its known author) in the authors' order, when the list is one place for
/// each of `authors` authors and no known author is another than the one at its place; `None` when it is no such list.
/// With no authors (the list given after `\maketitle` digested them, 1903.05554), the call's place, as Perl.
fn author_order_step(listed: &[(i64, Option<usize>)], authors: usize) -> Option<Vec<usize>> {
  // The place of each address in the list, read two ways: by the call (Perl's, a call of several addresses one place),
  // and by the address (`\emailAdd{jdhb@umich.edu, elvang@umich.edu}` for two authors, 2406.03543).
  let mut by_call: Vec<usize> = Vec::with_capacity(listed.len());
  for (at, (call, _)) in listed.iter().enumerate() {
    let place = match by_call.last() {
      Some(last) if listed[at - 1].0 == *call => *last,
      Some(last) => last + 1,
      None => 0,
    };
    by_call.push(place);
  }
  let by_address: Vec<usize> = (0..listed.len()).collect();
  let in_order = |places: &[usize]| {
    let count = places.last().map_or(0, |last| last + 1);
    listed
      .iter()
      .zip(places)
      .all(|((_, owner), place)| owner.is_none_or(|owner| owner == *place))
      && count == authors
  };
  // (by the address first: a call listing several authors' addresses, `\emailAdd{divyeshsolanki98@gmail.com,
  // srijuster@gmail.com}` for Divyesh N. Solanki and Srijit Bhattacharjee, 2308.02445, is read so; by the call when an
  // author's two addresses in one call would put the next one place on, as `\emailAdd{a, b}` for one author of two;
  // by the call, as Perl, with no authors to read)
  if authors == 0 {
    Some(by_call)
  } else if in_order(&by_address) {
    Some(by_address)
  } else if in_order(&by_call) {
    Some(by_call)
  } else {
    None
  }
}

/// The one author (index among the authors of `commands`, a frontmatter flush's queue) marked as the corresponding
/// author, for the one address of a class's list left over (`address`, its text), among the authors it may be of
/// (`within`, indices; all when `None`). An author is marked
/// - by a note in their own name that names the corresponding author ([`names_correspondence`]: jheppub's
///   `\note{Corresponding author.}`, `\note[*]{…}`; `\thanks`, `\footnote`), or
/// - by a mark they carry whose legend names the corresponding author: an affiliation line's (`\author[a,*]{…}` under
///   `\affiliation[*]{Corresponding author}`, `$^{1,*}$` under `\address{$^*$ Author to whom any correspondence should
///   be addressed.}`) or a note's — its own mark (`\note[*]{…}`) or the number a jheppub-family `\note` prints its text
///   under ([`AuthorNote::mark`]): the authors who write that number on their names are the ones it is about.
///
/// A note or a legend that says something else ("Equal contribution", "Since October 1, 2015 at …", 1510.03434) marks
/// no one. Exactly one marked author among `within` gets the address; none or two and more, the author block keeps it
/// (1802.03677's "¹ Corresponding author." note, whose 1 both M.V. Smirnov and J.J. Ling carry; 2606.26596's
/// "Corresponding authors." under the `*` of two authors; user ruling 2026-10-21). A mark with no legend at all is no
/// evidence of correspondence (a bare `*`, 2111.00753's two `$^{*}$`), save where the address itself says it is the
/// corresponding author's (`\emailAdd{zyxu@gzu.edu.cn(Corresponding author)}`) and exactly one author among `within`
/// carries a `*` with no legend (2308.06532's `\author[1,*]{Zhaoyi Xu}`, "E-mail: zyxu@gzu.edu.cn(Corresponding
/// author)" in its PDF).
fn marked_corresponding_author(
  commands: &[RawFrontmatter],
  address: &str,
  within: Option<&[usize]>,
) -> Option<usize> {
  let authors: Vec<&RawFrontmatter> = commands
    .iter()
    .filter(|entry| {
      entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
    })
    .collect();
  let names: Vec<Tokens> = authors.iter().map(|entry| queued_name(&entry.2)).collect();
  let mut notes = author_notes(names.iter(), &[
    T_CS!("\\thanks"),
    T_CS!("\\footnote"),
    T_CS!("\\note"),
  ]);
  // …and each note given apart from the names, the author's before it, as it is placed (1909.06107's
  // `\author[a,1]{and V.~Sharyy}\note{Corresponding author}`; 2404.00867's `\note[*]{…}` after Paul J. Steinhardt's
  // `\author`), its mark its own (`_mark`, jheppub's `\note[*]`)
  let mut before = None;
  for entry in commands {
    match (entry.0.as_str(), entry.1.get("role").map(String::as_str)) {
      ("ltx:creator", Some("author")) => before = Some(before.map_or(0, |at: usize| at + 1)),
      ("ltx:contact", Some("note")) => {
        if let Some(owner) = before {
          notes.push(AuthorNote {
            owner,
            mark: entry.1.get("_mark").and_then(|mark| {
              clean_frontmatter_labels(mark, "affiliation")
                .into_iter()
                .next()
            }),
            numbered: false,
            names_correspondence: names_correspondence(&visible_name_text(
              last_group(&entry.2).unlist_ref(),
            )),
          });
        }
      },
      _ => {},
    }
  }
  // Each author's labels: its class's optional ones (`\author[a,*]`, `annotations=`) and its marks'.
  let author_labels: Vec<Vec<String>> = authors
    .iter()
    .zip(&names)
    .map(|(entry, name)| {
      let mut labels = entry
        .1
        .get("annotations")
        .map(|given| clean_frontmatter_labels(given, "affiliation"))
        .unwrap_or_default();
      for operand in author_mark_operands(name.unlist_ref()) {
        labels.extend(clean_frontmatter_labels(
          &Tokens::new(operand).to_string(),
          "affiliation",
        ));
      }
      labels
    })
    .collect();
  // The legends: each affiliation's label (`\affiliation[*]{…}`'s, or the line's leading mark) and whether it names the
  // corresponding author; and each note's mark (`\note[*]{…}`, the n-th `\note`'s n).
  let mut legends: Vec<(String, bool)> = commands
    .iter()
    .filter(|entry| {
      entry.0 == "ltx:contact" && entry.1.get("role").map(String::as_str) == Some("affiliation")
    })
    .filter_map(|entry| {
      let content = last_group(&entry.2);
      let inner = queued_affiliation_text(&content);
      let label = match entry.1.get("label").filter(|label| !label.is_empty()) {
        Some(label) => clean_frontmatter_labels(label, "affiliation")
          .into_iter()
          .next(),
        None if marker_leads(&inner) => {
          entry_mark(inner.unlist_ref()).and_then(|mark| mark_label(&mark))
        },
        None => None,
      }?;
      Some((
        label,
        names_correspondence(&visible_name_text(inner.unlist_ref())),
      ))
    })
    .collect();
  // (a note's counter number is its legend only where no affiliation's label is that number: the authors writing an
  // affiliation's number request the affiliation, 2005.05695, 2005.14143, 2405.09762, 1403.3797, 2101.00866; a note's
  // own mark, `\note[*]{…}`, is its legend whatever the affiliations are labelled)
  let affiliation_legends = legends.len();
  let note_legends: Vec<(String, bool)> = notes
    .iter()
    .filter_map(|note| Some((note.mark.clone()?, note.numbered, note.names_correspondence)))
    .filter(|(mark, numbered, _)| {
      !*numbered
        || !legends[..affiliation_legends]
          .iter()
          .any(|(label, _)| label == mark)
    })
    .map(|(mark, _, says)| (mark, says))
    .collect();
  legends.extend(note_legends);
  let candidate = |at: &usize| within.is_none_or(|within| within.contains(at));
  let mut marked: std::collections::BTreeSet<usize> = notes
    .iter()
    .filter(|note| note.names_correspondence)
    .map(|note| note.owner)
    .collect();
  for (at, labels) in author_labels.iter().enumerate() {
    if labels.iter().any(|label| {
      legends
        .iter()
        .any(|(legend, says)| legend == label && *says)
    }) {
      marked.insert(at);
    }
  }
  let marked: Vec<usize> = marked.into_iter().filter(candidate).collect();
  match marked.as_slice() {
    [author] => return Some(*author),
    [] => {},
    _ => return None,
  }
  // (no author marked: the address itself naming correspondence, the one author carrying a `*` no legend explains)
  if !names_correspondence(address) {
    return None;
  }
  let star = clean_frontmatter_labels("*", "affiliation");
  let starred: Vec<usize> = author_labels
    .iter()
    .enumerate()
    .filter(|(at, labels)| {
      candidate(at)
        && labels
          .iter()
          .any(|label| star.contains(label) && !legends.iter().any(|(legend, _)| legend == label))
    })
    .map(|(at, _)| at)
    .collect();
  match starred.as_slice() {
    [author] => Some(*author),
    _ => None,
  }
}

/// The content of a queued affiliation's entry inside the wrappers that read its marks (`\lx@affiliation@withinst`,
/// `\lx@affiliation@withsup`), as written.
fn queued_affiliation_text(content: &Tokens) -> Tokens {
  let mut inner = content.clone();
  loop {
    let v = inner.unlist_ref();
    let wrappers = [
      T_CS!("\\lx@affiliation@withinst"),
      T_CS!("\\lx@affiliation@withsup"),
    ];
    let wrapped = v.first().is_some_and(|t| wrappers.contains(t))
      && v.get(1).is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && skip_group(v, 1) == v.len();
    if !wrapped {
      return inner;
    }
    inner = Tokens::new(v[2..v.len() - 1].to_vec());
  }
}

/// The content of the last top-level brace group of `tokens` (a queued frontmatter command's content argument); empty
/// when there is none.
fn last_group(tokens: &Tokens) -> Tokens {
  let v = tokens.unlist_ref();
  let mut depth = 0usize;
  let mut start = None;
  let mut last = Tokens::new(Vec::new());
  for (i, t) in v.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => {
        if depth == 0 {
          start = Some(i);
        }
        depth += 1;
      },
      Catcode::END if depth > 0 => {
        depth -= 1;
        if depth == 0
          && let Some(open) = start.take()
        {
          last = Tokens::new(v[open + 1..i].to_vec());
        }
      },
      _ => {},
    }
  }
  last
}

/// Digest FrontMatter (if not already?)
/// Perl: digestFrontMatter().
pub fn digest_front_matter() -> Result<()> {
  // The frontmatter observer's rehearsal of the author blocks, under its shadow switch only (nothing it finds is used
  // yet; OXIDIZED_DESIGN_DIVERGENCES #837).
  shadow_flush()?;
  bgroup();
  // INTENTIONAL DIVERGENCE from Perl PR #2767 (KNOWN_PERL_ERRORS #30,
  // OXIDIZED_DESIGN): clear the queue BEFORE digesting. Perl digests
  // from the live queue and wipes it after the loop — but when a queued
  // entry's own content re-triggers digestFrontMatter (witness aa.cls
  // 0907.0384: `\abstract{...}{}` dispatches the 5-arg \abstract@new,
  // whose greedy params swallow the document's `\maketitle` into arg #5,
  // so the queued abstract CONTAINS \maketitle → \lx@frontmatterhere →
  // afterDigest → re-entry), Perl re-digests the same queue unboundedly:
  // PR-head Perl dies `Fatal:perl:deep_recursion ... invokeToken`, zero
  // output (verified 2026-06-04). Pre-clearing makes the nested
  // invocation see an empty queue and terminate; the same paper then
  // converts with zero errors (real LaTeX also compiles it). Everything
  // else — deferred timing, digestion order, late re-let/\def fidelity —
  // is exactly the PR's. Newly-queued entries during this digest are
  // processed by the next invocation (or the end-of-document fallback).
  let commands: Vec<RawFrontmatter> = match remove_value("frontmatter_raw") {
    Some(Stored::FrontmatterRaw(commands)) => commands,
    _ => Vec::new(),
  };
  // The entries a seal drops in this flush, whose annotations go with them ([`seal_digested_frontmatter`]) — this
  // flush's own, the outer flush's restored after it (a queued entry may flush again, `\maketitle` in an abstract).
  let outer_sealed_off = lookup_string("lx_frontmatter_sealed_off");
  assign_value(
    "lx_frontmatter_sealed_off",
    Stored::String(pin("")),
    Some(Scope::Global),
  );
  // The authors digested here are the current ones: no later `\maketitle` supersedes
  // them unless another `\author` replaces them first (`\lx@maketitle@supersede`).
  if commands.iter().any(|entry| {
    entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
  }) {
    assign_value(
      "lx_authors_superseded",
      Stored::Bool(false),
      Some(Scope::Global),
    );
  }
  // Where this flush's sequenced contacts go ([`sequenced_contact_labels`]), in the order
  // `\lx@annotate@frontmatter@now` digests them — this flush's, the outer flush's restored after it, as the seal's.
  let outer_sequence_labels = lookup_string("lx_frontmatter_sequence_labels");
  let outer_sequence_at = lookup_int("lx_frontmatter_sequence_at");
  assign_value(
    "lx_frontmatter_sequence_labels",
    Stored::String(pin(sequenced_contact_labels(&commands).join("\u{1}"))),
    Some(Scope::Global),
  );
  assign_value(
    "lx_frontmatter_sequence_at",
    Stored::Int(0),
    Some(Scope::Global),
  );
  assign_value(
    "lx_frontmatter_flushing",
    Stored::Bool(true),
    Some(Scope::Local),
  );
  // The labels the authors' marks request, for the notes digested here (`note_body`) — this call's authors only: a
  // note queued for a later call (an appending `\author` after `\maketitle`) sees that call's.
  let requested: Vec<String> = commands
    .iter()
    .filter(|entry| {
      entry.0 == "ltx:creator" && entry.1.get("role").map(String::as_str) == Some("author")
    })
    .flat_map(|entry| author_mark_operands(entry.2.unlist_ref()))
    .flat_map(|operand| clean_frontmatter_labels(&Tokens::new(operand).to_string(), "affiliation"))
    .collect();
  assign_value(
    "lx_frontmatter_requested_marks",
    Stored::String(pin(requested.join("\u{1}"))),
    Some(Scope::Local),
  );
  // The labels the mark-picked contacts digested here answer, `role` and label (a marked line's first mark,
  // `\lx@affiliation@withsup`): an author requesting one has its contact of that role even before the contact is
  // digested, which a run's `new` reads (`\lx@annotate@frontmatter@now`; `\author{Ann$^{1}$}\author{Bob$^{2}$}
  // \affil{$^1$Univ X}\author{Cat \affil{Univ Z}}\affil{$^2$Univ Y}`: Cat's `\affil` is digested with her, before
  // Univ Y, 77n r4 review probe e4). Not a `label=` (`\affil[1]`): authblk prints the unlabelled `\affil` after it
  // under its `\AB@thenote`, both authors' (`\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}`, 2605.25017's
  // e-mail line, 77n r5 review probes q10, r07), as Perl's `annotate=new` (authblk.sty.ltxml:57) gives it.
  let offered: Vec<String> = commands
    .iter()
    .filter(|entry| entry.0 == "ltx:contact")
    .flat_map(|entry| {
      let role = entry.1.get("role").cloned().unwrap_or_default();
      let tokens = entry.2.unlist_ref();
      tokens
        .iter()
        .position(|t| *t == T_CS!("\\lx@affiliation@withsup"))
        .and_then(|at| entry_mark(&tokens[at + 1..]))
        .and_then(|mark| mark_label(&mark))
        .map(move |label| s!("{role}\u{2}{label}"))
    })
    .collect();
  assign_value(
    "lx_frontmatter_offered_labels",
    Stored::String(pin(offered.join("\u{1}"))),
    Some(Scope::Local),
  );
  if !commands.is_empty() {
    let_i(
      &T_CS!("\\lx@add@frontmatter"),
      &T_CS!("\\lx@add@frontmatter@now"),
      None,
    );
    let_i(
      &T_CS!("\\lx@annotate@frontmatter"),
      &T_CS!("\\lx@annotate@frontmatter@now"),
      None,
    );
    for (tag, attr, command) in commands {
      DebugFeature!(
        "frontmatter",
        "FRONT Digesting {tag} [{:?}] {command}",
        attr
      );
      // Perl parity (review 2026-06-04, replacing the master-era
      // fatal-swallow from witness arXiv:1903.01633): a Fatal raised
      // while digesting deferred frontmatter propagates and aborts
      // the conversion with a proper `Fatal:` log line, exactly like
      // Perl's un-eval'd `$stomach->digest($command)`. The 1903.01633
      // bug was the *silent* swallow (`let _ = digest(...)` left
      // `report.fatal=true` with no log line); propagation fixes the
      // silence without diverging from Perl. Non-fatal Error!s inside
      // the digest log-and-continue in both engines as before.
      digest(command)?;
    }
  }
  // Add punctuation to all ltx:creators, now that we know how many of each role.
  let mut updates: Vec<(usize, bool)> = Vec::new(); // (index, use_conjunction)
  with_value("frontmatter", |v| {
    if let Some(Stored::HashTagData(frnt)) = v
      && let Some(list) = frnt.get("ltx:creator")
    {
      for (i, item) in list.iter().enumerate() {
        let role = item.attr.get("role").map(String::as_str).unwrap_or("");
        let num: i64 = item
          .attr
          .get("_num")
          .and_then(|n| n.parse().ok())
          .unwrap_or(0);
        if !role.is_empty() && num > 1 {
          let n = lookup_mapping_int("num_ltx:creator", role);
          updates.push((i, num >= n));
        }
      }
    }
  });
  for (index, use_conjunction) in updates {
    let separator = DigestText!(Tokens!(if use_conjunction {
      T_CS!("\\lx@author@conj")
    } else {
      T_CS!("\\lx@author@sep")
    }))?;
    // `\lx@author@conj` may digest to \hskip Whatsits (\qquad) — extract
    // text-or-spaces (see \lx@author@prefix note in the previous port).
    let text = digested_to_text(&separator)?;
    with_value_mut("frontmatter", |val_opt| {
      if let Some(&mut Stored::HashTagData(ref mut frnt)) = val_opt
        && let Some(entry) = frnt.get_mut("ltx:creator").and_then(|l| l.get_mut(index))
      {
        entry.attr.insert("before".to_string(), text.clone());
      }
    });
  }
  assign_value(
    "lx_frontmatter_sealed_off",
    Stored::String(pin(&outer_sealed_off)),
    Some(Scope::Global),
  );
  assign_value(
    "lx_frontmatter_sequence_labels",
    Stored::String(pin(&outer_sequence_labels)),
    Some(Scope::Global),
  );
  assign_value(
    "lx_frontmatter_sequence_at",
    Stored::Int(outer_sequence_at),
    Some(Scope::Global),
  );
  egroup()?;
  Ok(())
}

/// First `<ltx:p>` under `root` reached only through plain paragraph wrappers — an
/// `ltx:logical-block` (a `center`/`flushleft` environment) or `ltx:para`. A paragraph
/// inside a further box (tcolorbox, `\parbox`, minipage, tabular) is layout we do not
/// read as a title candidate (see `maybe_promote_leading_title`).
pub(in crate::base_utilities) fn first_plain_paragraph(root: &Node) -> Option<Node> {
  let q = document::get_node_qname(root);
  if q == pin_static("ltx:p") {
    return Some(root.clone());
  }
  if q != pin_static("ltx:logical-block") && q != pin_static("ltx:para") {
    return None;
  }
  root
    .get_child_nodes()
    .iter()
    .filter(|c| c.get_type() == Some(NodeType::ElementNode))
    .find_map(first_plain_paragraph)
}

/// Every `<ltx:p>` under `root` reached only through plain paragraph wrappers, in
/// document order (the uniqueness test of `maybe_promote_leading_title`).
pub(in crate::base_utilities) fn collect_plain_paragraphs(root: &Node, out: &mut Vec<Node>) {
  let q = document::get_node_qname(root);
  if q == pin_static("ltx:p") {
    out.push(root.clone());
    return;
  }
  if q != pin_static("ltx:logical-block") && q != pin_static("ltx:para") {
    return;
  }
  for c in root.get_child_nodes() {
    if c.get_type() == Some(NodeType::ElementNode) {
      collect_plain_paragraphs(&c, out);
    }
  }
}

/// True if `root` or any descendant is set in a font larger than the body size —
/// the display-title signal. During construction the human-readable `fontsize`
/// attribute does not exist yet (it is derived from `_font` in a later finalize
/// pass); read the interned `_font` id and decode it here instead. `nominal` is
/// the document body size (NOMINAL_FONT_SIZE, mirroring font::defsize), so
/// `size > nominal * 1.1` is the construction-time analogue of `fontsize` > 110%.
pub(in crate::base_utilities) fn descendant_has_display_font(
  document: &Document,
  root: &Node,
  nominal: f64,
) -> bool {
  if let Some(fontid) = root.get_attribute("_font")
    && document
      .decode_font(&fontid)
      .and_then(Font::get_size)
      .is_some_and(|size| size > nominal * 1.1)
  {
    return true;
  }
  root.get_child_nodes().iter().any(|c| {
    c.get_type() == Some(NodeType::ElementNode) && descendant_has_display_font(document, c, nominal)
  })
}

/// True if `node` has at least one element child.
pub(in crate::base_utilities) fn has_element_child(node: &Node) -> bool {
  node
    .get_child_nodes()
    .iter()
    .any(|c| c.get_type() == Some(NodeType::ElementNode))
}

/// Where a frontmatter flush lands relative to what the document already holds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterAnchor {
  /// Right after the resources, before everything else — Perl's fallback placement.
  Top,
  /// Right after the document's leading run of first-group elements (a directly built
  /// `<titlepage>`, a promoted `<title>`), so the frontmatter group stays contiguous.
  LeadingFrontmatter,
}

/// Attribute marking a root child built before `\begin{document}` (see
/// [`mark_preamble_residue`]). Internal (`_`-prefixed): stripped from the output.
const PREAMBLE_RESIDUE: &str = "_preamble";

/// Mark every element child of the document root as PREAMBLE residue. Called by the
/// `\begin{document}` constructor when the root already exists — the preamble opened
/// it, so each child was typeset where LaTeX forbids typesetting (`\@nodocument`).
/// [`place_frontmatter`] moves such nodes below the frontmatter and never wraps them
/// as a cover (OXIDIZED_DESIGN #262).
pub fn mark_preamble_residue(root: &Node) {
  for mut child in root.get_child_nodes() {
    if child.get_type() == Some(NodeType::ElementNode)
      && document::get_node_qname(&child) != pin_static("ltx:resource")
    {
      let _ = child.set_attribute(PREAMBLE_RESIDUE, "true");
    }
  }
}

/// End the paragraph the flush interrupts: close the open elements between the
/// insertion point and the document root when every one of them auto-closes (a `p`
/// in a `para`) and none is a frontmatter-group element; otherwise leave the point
/// alone. An open `{titlepage}` is not a paragraph — `\begin{titlepage}\maketitle
/// \end{titlepage}` relies on the `\unwind@titlepage` hook finding it open (closing
/// it here left an empty `<titlepage/>` behind).
fn close_auto_closable_to_root(document: &mut Document, root: &Node) -> Result<()> {
  let mut n = document.get_node().clone();
  if n.get_type() != Some(NodeType::ElementNode) {
    match n.get_parent() {
      Some(p) => n = p,
      None => return Ok(()),
    }
  }
  let mut cur = n;
  while &cur != root {
    if cur.get_type() != Some(NodeType::ElementNode)
      || !document::can_auto_close(&cur)
      || is_frontmatter_group_element(&cur)
    {
      return Ok(());
    }
    match cur.get_parent() {
      Some(p) => cur = p,
      None => return Ok(()),
    }
  }
  document.close_to_node(root, true)
}

/// Was `node` built before `\begin{document}`? ([`mark_preamble_residue`])
fn is_preamble_residue(node: &Node) -> bool { node.has_attribute(PREAMBLE_RESIDUE) }

/// The ONE frontmatter placement pass (OXIDIZED_DESIGN #247). The schema wants every
/// frontmatter-group element before all `document.body.class` content
/// (`LaTeXML-structure.rnc:34`), and `Para.class` — hence the body group — includes
/// `pagination`, `rule` and `para` (`LaTeXML-para.rnc:18-20`). Perl only ever places at
/// the current point (`\maketitle`) or at the top (fallback) and leaves whatever that
/// yields. This pass looks at the LEADING REGION — the root children between the
/// resources and the bound (the abstract's own `ltx:_Frontmatter_Capture_` position
/// marker when it has one, else everything built so far) — and classifies each node:
///
/// * a first-group element (`is_frontmatter_group_element`) stays and becomes the
///   anchor candidate;
/// * a content-free node (`node_is_content_free`: a pagebreak, an empty paragraph, an
///   ink-free empty box) or PREAMBLE residue (`is_preamble_residue`: built before
///   `\begin{document}`, where LaTeX forbids typesetting) is left in place and, if it
///   preceded the anchor, moved after the placed frontmatter;
/// * hand-typeset title-page LAYOUT — `para` / demotable `logical-block` / `pagination`
///   with visible ink, `wrap_layout` only (the abstract-only fallback: a cover typeset
///   before a `\begin{abstract}` with no `\maketitle`) — is wrapped, in its exact order,
///   into `<ltx:titlepage>` (`titlepage_model = (FrontMatter.class |
///   SectionalFrontMatter.class | Block.class)*` is the schema's container for exactly
///   that; `para`→`block`, `logical-block` demoted bottom-up as in #244);
/// * anything else — a `<TOC>`, a section, a float, a root `<rule>`, a VISIBLE body
///   paragraph on the `\maketitle` path — ends the region: it stays where it is and the
///   frontmatter is placed ahead of it, so the document still opens with its front
///   group (Perl places at the current point and strands the title; user-approved
///   surpass, OXIDIZED_DESIGN #262).
///
/// The frontmatter is inserted after the wrapped titlepage, else after the leading
/// first-group run (`FrontmatterAnchor::LeadingFrontmatter`) or at the top, via the
/// same `_Capture_` insert→fill→unwrap machinery Perl's fallback uses. Run at every
/// placement point — `\lx@frontmatterhere`, both `\lx@frontmatter@fallback` branches,
/// the `{titlepage}` environment's `after_construct` — so a leading pagebreak before a
/// built titlepage is settled when that titlepage is built, not by a later pass. The
/// bound matters: without the marker a sectionless manual's whole body would read as
/// cover (jourcl's cover LETTER, derivative's after-abstract cover). Superseded by this
/// pass: #242's content-free gate, #245's `\end{document}` relocation, #246's anchor
/// helpers. Kept beside it: the conservative title promotion (`maybe_promote_leading_title`,
/// user-ruled), `maybe_dedup_leading_title_ink` (#6924), `insert_late_frontmatter`, the
/// marker sweep in `insert_frontmatter` and the core `finalize_rec`.
///
/// Witnesses: amsldoc/tkz-doc/tzplot (#242, 14 docs), gitinfo2/gitlog/FrontespizioScudo/
/// toptesi-example-* (#245, 6), toptesi-example-magistrale/tipfr-doc/isosigns-docs/
/// bootstrapicons-docs/derivative/jourcl (#246, 16), the faketitlepage /
/// promote_center_title / svabstract fixtures, `114_streaming_structure` (eager ≡ streaming).
pub fn place_frontmatter(
  document: &mut Document,
  wrap_layout: bool,
  anchor: FrontmatterAnchor,
) -> Result<()> {
  // Drain the raw frontmatter queue first, as `insert_frontmatter` does, so
  // `frontmatter_pending` and the flush agree: the `{titlepage}` environment's
  // `after_construct` is the one placement point without its own digest hook, and
  // under `--streaming` (digestion interleaved with construction) a `\title` given
  // after `\end{titlepage}` may still sit undigested there.
  digest_front_matter()?;
  let Some(root) = document.get_document().get_root_element() else {
    return insert_frontmatter(document);
  };
  let kids: Vec<Node> = root
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  let mut i = 0;
  while i < kids.len() && document::get_node_qname(&kids[i]) == pin_static("ltx:resource") {
    i += 1;
  }
  let bound = kids[i..]
    .iter()
    .position(|n| document::get_node_qname(n) == pin_static("ltx:_Frontmatter_Capture_"))
    .map_or(kids.len(), |k| i + k);
  // The leading prefix: first-group elements and content-free nodes, in any order.
  let mut last_first: Option<Node> = None;
  let mut movers: Vec<Node> = Vec::new();
  while i < bound {
    if is_frontmatter_group_element(&kids[i]) {
      last_first = Some(kids[i].clone());
    } else if node_is_content_free(&kids[i]) || is_preamble_residue(&kids[i]) {
      movers.push(kids[i].clone());
    } else {
      break;
    }
    i += 1;
  }
  // Hand-typeset cover before the abstract → <titlepage>, in order.
  let mut titlepage: Option<Node> = None;
  if wrap_layout && i < bound {
    let end = i
      + kids[i..bound]
        .iter()
        .take_while(|k| is_titlepage_layout(k))
        .count();
    if kids[i..end].iter().any(|k| !node_is_content_free(k)) {
      // The flush ends the paragraph it interrupts before the cover moves: text right
      // before an abstract is still the open paragraph, and wrapping it open left the
      // insertion point inside the renamed `block` (`Error:malformed:ltx:document` at
      // `\end{document}`; serbian-def-cyr/proba).
      close_auto_closable_to_root(document, &root)?;
      titlepage = Some(wrap_as_titlepage(document, &root, &kids[i..end])?);
    }
  }
  // Nothing queued (or the abstract-only deferral): `insert_frontmatter` keeps its
  // side effects; a leading pagebreak before a built titlepage still settles here.
  if !frontmatter_pending() {
    insert_frontmatter(document)?;
    if let Some(tail) = last_first {
      move_content_free_after(&movers, tail)?;
    }
    return Ok(());
  }
  // Visible body content before the flush (prose or a class-drawn cover before
  // `\maketitle`, a section before a late `\title`) does not hold the frontmatter
  // back: the schema's document model puts the frontmatter group first
  // (`LaTeXML-structure.rnc:34`), so it goes to the head and that content follows
  // it in its own order. Perl places at the current point (`\maketitle`,
  // Base_Utility.pool.ltxml:918), stranding `<title>` after the body — schema-invalid
  // (user-approved surpass 2026-09-23, OXIDIZED_DESIGN #262). The flush ends the
  // paragraph it interrupts, as every flush point does in LaTeX (`\maketitle`:
  // article.cls:203 opens with `\par`, report/book with `\begin{titlepage}`;
  // `\section`; `\end{document}`) and as the current-point insertion did by
  // auto-closing: text after `\maketitle` starts a new paragraph instead of
  // continuing one that now stands after the frontmatter.
  close_auto_closable_to_root(document, &root)?;
  let after = titlepage.or(match anchor {
    FrontmatterAnchor::Top => None,
    FrontmatterAnchor::LeadingFrontmatter => last_first.clone(),
  });
  let placed = insert_frontmatter_at(document, after.as_ref())?;
  // Content-free nodes that preceded the placed run now follow it, in their order.
  let tail = match (placed.last().cloned(), last_first) {
    (Some(p), Some(f)) => Some(if node_precedes(&p, &f) { f } else { p }),
    (p, f) => p.or(f),
  };
  if let Some(tail) = tail {
    move_content_free_after(&movers, tail)?;
  }
  Ok(())
}
