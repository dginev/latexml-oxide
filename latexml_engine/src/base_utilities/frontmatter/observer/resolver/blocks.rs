//! The resolver's owner chain for an author block (OXIDIZED_DESIGN_DIVERGENCES #837; the module's rules 5-9): the
//! block's lines by type, labelled entries to the names requesting them, unlabelled lines to the row above, a
//! line's contacts by their own mark, spelling, order, the group's only author or elimination, and its notes.

use super::*;

/// The chain [`Resolver::place_line_contacts`] places a print's addresses by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Chain<'a> {
  /// A line of a block (or an entry's e-mail call): the spelled addresses first, then — the one address left, and
  /// the one candidate left without an address — by elimination (R10), then the rest of the chain.
  Line,
  /// A document's own note (`ltx:pubnote`, `answer_document_notes`): FIDELITY_CHARTER §4 exactly — the author it
  /// spells, the authors' order, the only author, the corresponding author, the block; no elimination, so an address
  /// that spells no one is not guessed onto an author (review rev92a_r1 B2: aa's `\offprints{the editors,
  /// \email{office@y.org}}`). `named` is the creators the name the note prints before the address names, when the
  /// class prints the address in the note after it ([`Resolver::note_named`], `Policy::email_in_place`): A&A's "Send
  /// offprint requests to: O. Kochukhov, e-mail: …" designates the corresponding author (main-session decision on 92a
  /// r1 question a, 0903.3512), so one creator named is the corresponding step's owner, and none or several the
  /// block's — prose naming no one ("Supported by NSF.") too (an open design point, OD #913); a note printing nothing
  /// before the address (`named` `None`) leaves the step to the creators' own "Corresponding author" notes. (A call
  /// the class prints elsewhere, routed, is one author's whole call and never reaches the chain: review rev92a_r4 S1.)
  DocumentNote { named: Option<&'a [usize]> },
}

impl Resolver {
  /// Contacts of `text` (one per printed address, ruling 3; the text whole when it holds none), owned by `chain`; each
  /// rule led by `api:document-note` for a document's note.
  pub(super) fn place_line_contacts(
    &mut self,
    text: &str,
    role: &str,
    candidates: &[usize],
    source: (usize, Option<(u16, u16)>),
    anchor: Option<&Assoc>,
    chain: Chain<'_>,
  ) {
    let led = |rule: String| match chain {
      Chain::Line => rule,
      Chain::DocumentNote { .. } => s!("api:document-note {rule}"),
    };
    // (with the emitter's switch on (`emitting`; the switch-off output stays as it was, brief LO1) by the address reader,
    // which reads a list of local parts sharing one domain where it opens the line or a part spells an author: `{\it
    // pzou94, ozel, suresh@gwu.edu}` under three names, 2003.14069 — two of them nameless notes; spelling assigns them,
    // position where it fails, below)
    let addresses = addresses_in(text);
    let addresses = if self.emitting {
      let spells = |local: &str| self.spelled(&s!("{local}@x.org"), candidates).is_some();
      let read: Vec<String> = address_runs(text, &spells)
        .into_iter()
        .flat_map(|run| run.addresses)
        .collect();
      // (where it reads every address the plain reading does and more; a tagged address it leaves where it prints,
      // `ann@a.org (Ann)`, is one all the same)
      if read.len() > addresses.len() && addresses.iter().all(|a| read.contains(a)) {
        read
      } else {
        addresses
      }
    } else {
      addresses
    };
    if addresses.is_empty() {
      let (owner, rule) = match candidates {
        [only] => (Some(*only), "owner:only-author"),
        _ => (None, "owner:block"),
      };
      self.attach(
        owner,
        Assoc {
          role:   role.to_string(),
          text:   text.to_string(),
          label:  None,
          rule:   led(rule.into()),
          source: Some(source),
          target: None,
        },
        false,
      );
      return;
    }
    let spelled: Vec<Option<usize>> = addresses
      .iter()
      .map(|a| self.spelled(a, candidates))
      .collect();
    let unspelled = spelled.iter().filter(|s| s.is_none()).count();
    let left: Vec<usize> = candidates
      .iter()
      .copied()
      .filter(|c| !spelled.contains(&Some(*c)))
      .filter(|&c| {
        !self.out.creators[c]
          .contacts
          .iter()
          .any(|a| a.role == "email")
      })
      .collect();
    // (a list with one address per author that every spelled address agrees with is in author order: FIDELITY_CHARTER
    // §4, 2026-10-21, OD #805 — `{qkv, riven, jiyicheng.cs, lihuan.cs, cwang85}@zju.edu.cn` under five names, 2603.19610;
    // 2512.22629, 2305.12998, 2105.11174, 2412.09666)
    // (and no address the block prints elsewhere spells one of them: `\{xx,yy\}@x.edu` under two names, one of whom
    // `ann.able@gmail.com` on the next line spells, is no one-per-author list — three addresses for two authors, the
    // two unspelled the block's, as the old reader's 77p r9 held them)
    let in_order = addresses.len() == candidates.len()
      && candidates.len() >= 2
      && spelled
        .iter()
        .zip(candidates)
        .all(|(s, c)| s.is_none_or(|s| s == *c))
      && self
        .block_addresses
        .iter()
        .filter(|a| !addresses.contains(a))
        .all(|a| self.spelled(a, candidates).is_none());
    let address_count = addresses.len();
    // (a link printing one address: its anchor the contact's text, its address the target — position 2, review
    // revfms2a2_9 finding 5; 1607.00911's `\href{mailto: edremets@…}{* edremets@…}`)
    let anchor = anchor.filter(|_| address_count == 1);
    for (at, (address, spelled)) in addresses.into_iter().zip(spelled).enumerate() {
      let (owner, rule) = match spelled {
        // (a document's note: the chain's first step, a ruling's change of today's — `api:note-spelled`)
        Some(owner) if matches!(chain, Chain::DocumentNote { .. }) => {
          (Some(owner), "owner:spelled api:note-spelled")
        },
        Some(owner) => (Some(owner), "owner:spelled"),
        None if in_order => (Some(candidates[at]), "owner:order"),
        None
          if chain == Chain::Line && unspelled == 1 && candidates.len() > 1 && left.len() == 1 =>
        {
          (Some(left[0]), "owner:elimination")
        },
        None => match candidates {
          [only] => (Some(*only), "owner:only-author"),
          // (an address pattern, `\{FirstName.LastName\}@SIT.Fraunhofer.de` under three names, is every author's
          // address and no one's own: the block's, as today's — not the corresponding author's, 2006.12418; where
          // order or elimination places it, as today's too, 2110.05663)
          _ if is_pattern_address(&address) => (None, "owner:pattern"),
          // (the corresponding author's address is one printed alone: an address left over in a print of several is
          // another author's — `{hong3924.cs10, evan20010126.cs12, …}@nycu.edu.tw` under a corresponding Ching-Chun
          // Huang, 2512.08223; `{congjing, ryanflin}@uw.edu chris20030909@gmail.com`, 2602.04785; 2604.14972)
          _ if address_count > 1 => (None, "owner:block"),
          // (an address two or more names spell is the corresponding author's, or the author a document's note names,
          // only where they are one of them, else the block's, as the old reader holds it for both
          // (`author_block/email_block_calls.rs` `spelled`, `owners.marked`): `wang@x.org` under X. Wang, Y. Wang and a
          // corresponding Z. Li — `perfect_kernel_batch64::marked_block_email_line_owner`, 77p r4; A&A's
          // `\offprints{Z. Li, \email{wang@x.org}}` —
          // `perfect_kernel_batchnl1::an_address_others_spell_is_not_the_named_authors`)
          _ => {
            let found = match chain {
              Chain::DocumentNote { named: Some(named) } => match named {
                [one] if candidates.len() > 1 => (Some(*one), "owner:corresponding api:note-name"),
                _ => (None, "owner:block"),
              },
              _ => self.corresponding(candidates),
            };
            let spellers = self.spellers(&address, candidates);
            match found {
              (Some(c), _) if spellers.len() >= 2 && !spellers.contains(&c) => {
                (None, "owner:block")
              },
              found => found,
            }
          },
        },
      };
      self.attach(
        owner,
        Assoc {
          role:   "email".into(),
          text:   anchor.map_or_else(|| address.clone(), |a| a.text.clone()),
          label:  None,
          rule:   led(s!("{rule} ruling3:one-per-print")),
          source: Some(source),
          target: anchor.and_then(|a| a.target.clone()),
        },
        false,
      );
    }
  }

  /// Resolve one block (its events under `policy`).
  pub(super) fn block(
    &mut self,
    index: usize,
    policy: &'static Policy,
    cautious: bool,
    keyvals: Option<&str>,
    events: &[Event],
  ) {
    self.out.policies.push((index, policy.name));
    self.notes_marked = !policy.unmarked_notes;
    let and_inline = policy.and_inline;
    let events = &read_events(events);
    let lines = lines_of(events, and_inline);
    let block_requests: Vec<String> = keyvals
      .and_then(|kv| attr_value(kv, "annotations"))
      .map(|labels| {
        labels
          .split(',')
          .map(|l| l.trim().to_string())
          .filter(|l| !l.is_empty())
          .collect()
      })
      .unwrap_or_default();
    let group = index;
    let block_creators_start = self.out.creators.len();
    self.block_addresses = lines
      .iter()
      .flat_map(|line| addresses_in(&events_text(&line.events)))
      .collect();
    // the labels the block's later mark-led lines answer (for the prefix convention)
    let answered_below: Vec<String> = lines
      .iter()
      .skip(1)
      .filter(|line| {
        line
          .events
          .iter()
          .find(|e| {
            inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
          })
          .is_some_and(|e| e.kind == EventKind::Mark)
      })
      .flat_map(|line| entries_of(&line.events).into_iter().flat_map(|e| e.labels))
      .collect();
    // per column: the creators of its last names row
    let mut row_above: Vec<(usize, Vec<usize>)> = Vec::new();
    // per column: the creators of its run of names rows, each directly under the one before with no line answering it
    // (an unmarked institution under them is every name's of the run, user ruling R12: `A$^{1}$, B$^{2}$\\ C$^{3}$\\
    // Univ` gives Univ to all three, as today's splitter does — 1511.01102's four rows over `(The Nearby Supernova
    // Factory)`)
    let mut run_above: Vec<(usize, Vec<usize>)> = Vec::new();
    // the entry the previous line ended with, which an unmarked line under it continues (an address's second line:
    // `$^1$Dept of X \\ 1-1-1 Street, City`, 0803.3589; one affiliation, as today's `\\`-joined contact)
    let mut continues: Option<Continued> = None;
    let mut first_in_column: Vec<usize> = Vec::new();
    // the columns whose last line is a names row ending in a separator (a comma, "and"), which the next line continues
    let mut open_rows: Vec<usize> = Vec::new();
    // the columns whose names row has a line under it already (its affiliation)
    let mut answered_rows: Vec<usize> = Vec::new();
    // the columns whose last line was labelled entries, and how many entries it held
    let mut labelled_last: Vec<(usize, usize)> = Vec::new();
    // an unmarked line's affiliation whose group runs on past the line (its places, where the group ends)
    let mut open_group: Option<(u16, Vec<(usize, usize)>)> = None;
    // the places of the unmarked affiliation the line before made, which a line after a break inside the same braced
    // group continues
    let mut unmarked_before: Option<Vec<(usize, usize)>> = None;
    // the last line an address list read on to (`place_line_contacts` over the lines of one list)
    let mut skip_to = 0usize;
    // the URL lines under every name so far, each with those names: the block's if no name comes after them
    let mut block_links: Vec<(&Event, Vec<usize>)> = Vec::new();
    // the last column holding a line that reads as names: a column after it opening with an institution holds no name
    // of its own, and is the shared affiliation of every name above it
    let last_named = lines
      .iter()
      .filter(|line| reads_as_names(&events_text(&line.events)))
      .map(|line| line.column)
      .max();
    for (n, line) in lines.iter().enumerate() {
      if n > 0 && n <= skip_to {
        continue;
      }
      self.line_column = Some((index, line.column));
      for span in line.events.iter().filter_map(|e| e.span) {
        self.entry_column.insert((index, span.0), line.column);
      }
      let unmarked_above = unmarked_before.take();
      let unmarked_line_above = unmarked_above.is_some();
      let continued = open_rows.contains(&line.column);
      open_rows.retain(|c| *c != line.column);
      // (the line before in the column was labelled entries: an unmarked name under two or more of them is a new row's —
      // SYNC row 46, `$^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics`; a line under one entry goes on
      // with it, the old reader's N5 (`add_authors_calls_sectioned`'s unmarked line appended to the entry above), whatever
      // its words read as: `$^1$The Graduate Center, CUNY\\ Computer Science\\ New York, USA`, 2501.04046; `$^{1}$Institute
      // of …, Ionosphere and\\ Radio Wave Propagation\\ RAS, …`, hep-ph0511130 — each line one affiliation's)
      let entries_above = labelled_last
        .iter()
        .find(|(c, _)| *c == line.column)
        .map(|&(_, entries)| entries);
      let after_labelled = entries_above.is_some();
      let after_labelled_list = entries_above.is_some_and(|entries| entries >= 2);
      labelled_last.retain(|(c, _)| *c != line.column);
      let under_affiliation = answered_rows.contains(&line.column);
      // (a line that is no names row answers the row above; a names row below takes this back)
      if !answered_rows.contains(&line.column) && row_above.iter().any(|(c, _)| *c == line.column) {
        answered_rows.push(line.column);
      }
      // (punctuation before a mark does not lead the line: `($^1$) LSIIT …`, 1004.4809)
      let first_ink = line
        .events
        .iter()
        .find(|e| {
          inked(e) && !(e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
        })
        .copied();
      let column_first = !first_in_column.contains(&line.column);
      if column_first {
        first_in_column.push(line.column);
      }
      // (a column holding no name of its own after the last name is the shared affiliation of every name above it, under
      // every name as a link printing an institution is: sig-alternate's `\sharedaffiliation` (`\end{tabular}
      // \begin{tabular}{c}`, a column, `observer::rehearsal`) under its `\alignauthor` columns, the PDF centring it under
      // all of them — 0712.1016, 1704.03168, the last author's alone before; written inline, `A One \and Bob Two
      // \end{tabular}\begin{tabular}{c} Univ Shared`, "Univ Shared" a third creator, review revG3 findings 1 and Q3)
      let shared_column = column_first
        && line.column > 0
        && last_named.is_some_and(|named| line.column > named)
        && self.out.creators.len() > block_creators_start
        && is_institution(&events_text(&line.events));
      if shared_column {
        let every: Vec<usize> = (block_creators_start..self.out.creators.len()).collect();
        run_above.push((line.column, every.clone()));
        row_above.push((line.column, every));
        // (the names row it answers: the lines under it go on with the affiliation)
        answered_rows.push(line.column);
      }
      let above = row_above
        .iter()
        .rev()
        .find(|(c, _)| *c == line.column)
        .map(|(_, r)| r.clone());
      let above = above
        .or_else(|| row_above.last().map(|(_, r)| r.clone()))
        .unwrap_or_default();
      let run = run_above
        .iter()
        .rev()
        .find(|(c, _)| *c == line.column)
        .or_else(|| run_above.last())
        .map(|(_, r)| r.clone())
        .unwrap_or_default();
      let carried = continues.take();
      // (a line a glyph leads — no superscript, `(\S) Inst…` — whose mark no name requests is no labelled line: Perl's
      // block reader splits at superscripts alone (`positionOf($line, @authorsup_markers)`, Base_Utility.pool.ltxml:
      // 699-707), so such a line continues the entry above it, as an unmarked line does (`line:continued`), and a printed
      // mark no name requests is no label (FIDELITY_CHARTER §4) — cond-mat0309053's translators' `(\S)`, `(\dag)`,
      // `(\ddag)` institutes, no author's marks, stay in the authors' affiliation as before, not a nameless creator's)
      // (only where an entry is open above it, which the line continues — Perl's `if (@entries)`; else, an e-mail
      // entry above or none, it stays the symbol line it was: `*Corresponding Author: Ayodeji Olalekan Salau (…)` under
      // the `\textsuperscript{7}\texttt{…}` addresses is no names row, 2510.01302. With the emitter's switch on alone,
      // `emitting`)
      let unrequested_glyph = self.emitting
        && carried.is_some()
        && first_ink.is_some_and(|e| {
          e.kind == EventKind::Mark
            && e.role.as_deref() == Some("text")
            && !entries_of(&line.events)
              .iter()
              .flat_map(|entry| &entry.labels)
              .any(|label| {
                block_requests.iter().any(|r| label_matches(r, label))
                  || self
                    .out
                    .creators
                    .iter()
                    .any(|c| c.requests.iter().any(|r| label_matches(r, label)))
              })
        });
      if unrequested_glyph && let Some(carried) = carried {
        let notes = line
          .events
          .iter()
          .copied()
          .filter(|e| e.kind == EventKind::Note);
        self.continue_line(
          index,
          &carried,
          &events_text(&line.events),
          notes,
          above.last().copied(),
        );
        continues = Some(carried);
        continue;
      }
      match first_ink.map(|e| &e.kind) {
        Some(EventKind::Mark) => {
          let entries = entries_of(&line.events);
          // (or the block is that one line, its affiliations the calls after it: revtex's `\author{$^{1}$Masashi Kimura
          // and $^{2,3}$Takahiro Tanaka}`, 1805.08625; 1405.2400, 2001.00788, 0710.4115, gr-qc0606119 — its names lost)
          let prefix = n == 0
            && !entries.is_empty()
            && entries.iter().all(|e| {
              reads_as_names(&events_text(&e.events))
                && (lines.len() == 1 || e.labels.iter().any(|l| answered_below.contains(l)))
            });
          if prefix {
            let mut row = Vec::new();
            for entry in entries {
              let start = row.len();
              for name in names_of(&entry.events) {
                let c = self.new_creator(
                  name,
                  "line:names mark:prefix",
                  index,
                  [entry.labels.clone(), block_requests.clone()].concat(),
                  group,
                );
                row.push(c);
              }
              // (the entry's notes and contacts are its name's, as a names line attaches them to the name before:
              // `$^{1}$José Edgar Madriz Aguilar\thanks{E-mail address: …}`, 0710.4115, gr-qc0606119 — the notes were
              // lost)
              let owner = row[start..].last().copied();
              for event in &entry.events {
                match event.kind {
                  EventKind::Note => self.note(index, event, owner, "note:adjacent"),
                  EventKind::Contact => self.contact_event(
                    index,
                    event,
                    &owner.into_iter().collect::<Vec<_>>(),
                    "owner:adjacent",
                  ),
                  _ => {},
                }
              }
            }
            answered_rows.retain(|c| *c != line.column);
            run_above.push((line.column, row.clone()));
            row_above.push((line.column, row));
            continue;
          }
          labelled_last.push((line.column, entries.len()));
          continues = self.labelled_entries(index, entries, "");
        },
        Some(_) => {
          let text = events_text(&line.events);
          let only_contacts = line
            .events
            .iter()
            .filter(|e| inked(e))
            .all(|e| e.kind == EventKind::Contact);
          // (a column's first line that is one link printing a name is that name: `\href{https://orcid.org/…}
          // {\includegraphics{orcid}\hspace{1mm}Amanda A. Howard}`, 2311.06483)
          if column_first && only_contacts {
            let links: Vec<&Event> = line
              .events
              .iter()
              .copied()
              .filter(|e| e.kind == EventKind::Contact)
              .collect();
            if let [link] = links.as_slice()
              && link.role.as_deref() == Some("url")
              && reads_as_names(&link.text)
            {
              let c = self.new_creator(
                link.text.clone(),
                "line:names link-text",
                index,
                block_requests.clone(),
                group,
              );
              answered_rows.retain(|c| *c != line.column);
              run_above.push((line.column, vec![c]));
              row_above.push((line.column, vec![c]));
              continue;
            }
          }
          // (a later line is more names when each of its pieces reads as a name: `Triantafyllos Afouras\quad\quad
          // Himel Chowdhury …` under the first row, 2111.03635; the classify helpers on each piece, not the run of them)
          // (a line of e-mail addresses alone is never names, a column's first line too: `\And
          // \{sagghosh, …, niranjan\}@cs.stonybrook.edu`, 2210.05991)
          let address_line = !addresses_in(&text).is_empty() && only_addresses(&text);
          let names_line = !only_contacts
            && !address_line
            && ((column_first && !shared_column)
              || later_line_is_names(line, continued, under_affiliation, after_labelled_list));
          if names_line {
            answered_rows.retain(|c| *c != line.column);
            // (a separator glyph before the line's contact parts the name from it, and ends no row: `Chiara Simeoni --
            // \url{…}`, 2511.07187 — only one the line ends with opens it)
            let ends_open = |e: &&Event| match &e.kind {
              EventKind::Sep(SepKind::Comma) => true,
              EventKind::Sep(SepKind::Word(word)) => {
                !events::is_separator_word(word)
                  || line
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.kind != EventKind::Layout)
                    .is_some_and(|last| std::ptr::eq(*last, *e))
              },
              _ => false,
            };
            if line
              .events
              .iter()
              .rev()
              .find(|e| {
                !matches!(
                  e.kind,
                  EventKind::Layout | EventKind::Contact | EventKind::Note
                )
              })
              .is_some_and(ends_open)
            {
              open_rows.push(line.column);
            }
            let made = self.names_line(index, policy, cautious, line, &block_requests, group);
            // (a later column's first name is parted from the name before by the `\and` (or `&`) that opened the
            // column: `Chris Kapulkin \arxivortac{\and}{and} Peter LeFanu Lumsdaine`, 2006.13694)
            if line.column > 0
              && column_first
              && let Some(&first) = made.creators.first()
              && !self.out.creators[first].rule.contains("sep:")
            {
              self.out.creators[first].rule.push_str(" sep:column");
            }
            // (a names row directly under the column's last, no line between answering it, goes on with its run)
            let mut names_run = if under_affiliation {
              Vec::new()
            } else {
              run_above
                .iter()
                .rev()
                .find(|(c, _)| *c == line.column)
                .map(|(_, r)| r.clone())
                .unwrap_or_default()
            };
            names_run.extend(made.creators.iter().copied());
            run_above.push((line.column, names_run));
            row_above.push((line.column, made.creators));
            continue;
          }
          if only_contacts {
            for contact in line.events.iter().filter(|e| e.kind == EventKind::Contact) {
              // (an in-block contact printing addresses alone is a line of addresses, whatever command made it: e-mails by
              // the owner chain among the block's names — `\IEEEauthorblockA{hanxinchi@sjtu.edu.cn, …}` under 17 names of
              // 17 `\and`-columns, 2603.28168, no affiliation of the last column's)
              if contact.label.is_none()
                && contact.role.as_deref() == Some("affiliation")
                && !addresses_in(&contact.text).is_empty()
                && only_addresses(&contact.text)
              {
                let candidates: Vec<usize> =
                  (block_creators_start..self.out.creators.len()).collect();
                let source = (index, contact.span);
                self.place_line_contacts(
                  &contact.text,
                  "email",
                  &candidates,
                  source,
                  None,
                  Chain::Line,
                );
                continue;
              }
              // (a URL line under every name of the document's author list is the block's, its shared creator (OD #159,
              // `addresses:block`): a project's page, `Ann Able, Bob Baker \\ Univ A \\ \url{…}` — the old reader's
              // `LinkOwner::Block`, `author_block::dispatch` and `link_line_owner`, user ruling 2026-10-20, batch 77p r7:
              // 2605.06223, 2602.08439, 2604.12896, 2604.20841, 2601.17879; FIDELITY_CHARTER §4 amended 2026-11-05.
              // Whether no names come after it is known at the block's end: decided there. A link printing an
              // institution is the names' affiliation, as the old rule's exception gives it every author:
              // `\href{https://www.google.com}{Google}`, `dispatch.rs`'s `link_prints_an_institution` (`href_prints_an_institution`), user ruling R12
              // 2026-10-20, the r5 re-review's u38, review revFB2a finding 3. A lone name's URL is its own, the group's
              // only author's — FIDELITY_CHARTER §4)
              if self.emitting
                && self.block_replaces
                && above.len() > 1
                && contact_parts(contact).0 == "url"
                && above
                  .iter()
                  .copied()
                  .eq(block_creators_start..self.out.creators.len())
              {
                if contact
                  .target
                  .as_deref()
                  .is_some_and(|target| href_prints_an_institution(target, &contact.text))
                {
                  let affiliation = Event {
                    role: Some("affiliation".into()),
                    ..(*contact).clone()
                  };
                  self.contact_event(index, &affiliation, &above, "line:contact R12:row-above");
                } else {
                  block_links.push((contact, above.clone()));
                }
                continue;
              }
              self.contact_event(index, contact, &above, "line:contact R12:row-above");
            }
            continue;
          }
          // (a line whose marks trail their text is labelled entries: `Nanyang Technological University$^1$,
          // Amazon$^2$`, 2507.05750)
          // (each of its marks one a name shows on another line: a symbol after an affiliation, `Harvard
          // University$^{*}$`, points to a note, 2307.02318)
          // (shown on a name: a mark another address line prints is no name's — aistats's `\aistatsaddress{Univ A$^*$
          // \And Univ B \And Univ A$^*$}` under names showing no `*` pairs each address with its column's name, as
          // the class prints column i under author i, 77o's `aistats_note_marked_address_still_pairs_by_column`)
          let trailing = trailing_entries(&line.events);
          let shown_elsewhere = |label: &String| {
            self.out.creators[block_creators_start..]
              .iter()
              .any(|c| c.requests.iter().any(|r| label_matches(r, label)))
          };
          if !trailing.is_empty()
            && trailing
              .iter()
              .all(|e| e.labels.iter().any(shown_elsewhere))
          {
            continues = self.labelled_entries(index, trailing, " mark:trailing");
            continue;
          }
          // an unmarked line under names: an address line by the owner chain, else the row above's affiliation
          let source = (
            index,
            line.events.iter().find(|e| inked(e)).and_then(|e| e.span),
          );
          let notes: Vec<&Event> = line
            .events
            .iter()
            .copied()
            .filter(|e| e.kind == EventKind::Note)
            .collect();
          // (a line of notes only mixing notes a name's mark requests with one no mark labels: the requested notes their
          // names', the other the block's, no name's by its place — as the old reader joins such a line, OD #957
          // `notes_the_names_request`: `\\ \thanks{$^{1}$Univ A}\thanks{This work was funded by …}`, 2609.17106,
          // review rev100c S1; a line of unrequested notes alone stays the line owner's, as today's, SYNC_STATUS 100c)
          if text.is_empty() {
            let live = self.live();
            let requested = |note: &Event| {
              leading_mark(note).is_some_and(|(labels, _)| {
                labels
                  .iter()
                  .any(|l| !label_owners(&self.out.creators, &live, l).is_empty())
              })
            };
            let asked: Vec<bool> = notes.iter().map(|note| requested(note)).collect();
            let mixed = asked.contains(&true);
            // (a line of notes under a labelled entry is that entry's line, as today's one `\\`-joined contact: its notes
            // the block's, as Perl digests them in the contact (`\lx@add@pubnote@thanks`), one `<pubnote>` each — `$^{2}$Univ
            // B, City\\ \thanks{Equal contribution.}` gave the last name the thanks, review revLO2 base; with the emitter's
            // switch on alone, `emitting`)
            let entry_line = self.emitting && matches!(carried, Some(Continued::Pending(_)));
            for (note, asked) in notes.into_iter().zip(asked) {
              if mixed && !asked {
                self.note(index, note, None, "note:line-owner note:unrequested");
              } else if entry_line {
                self.note(index, note, None, "note:line-owner note:entry");
              } else {
                self.note(index, note, above.last().copied(), "note:line-owner");
              }
            }
            continue;
          }
          // (an unmarked line under a labelled entry continues it, as today's one `\\`-joined contact — a line holding an
          // address beside other text too: `$^{(1)}$Dept \\ Univ \\ City. \{ann,bob\}@upc.edu`, 2509.12185; a line of
          // addresses alone is e-mails by the owner chain, below: `$^1$Dept \\ landi@univ.trieste.it`, 0801.3376, 0810.3062)
          // (a line inside the group an unmarked line above opened is that affiliation's: `\\ \small{\textit{Dept …,
          // and\\ Institute …,\\ Facultad …}}`, one print unit, as today's one contact — 1012.1813)
          // (a line after a break inside the braced group the unmarked line above is in continues its affiliation, one
          // print unit, as today's one contact: `Xuhui Peng \\ {\em\tiny College of Mathematics …,\\ Changsha, Hunan
          // 410081, P. R. China}`, 1710.07902; `\textsl{Pure Mathematics\\University of Waterloo\\…}`, math0610612 — no
          // line of addresses alone)
          if line.group_break
            && let Some(places) = unmarked_above
            && (addresses_in(&text).is_empty() || !only_addresses(&text))
          {
            self.continue_entry(&Continued::Joined(places.clone()), &text);
            for note in notes {
              self.note(index, note, above.last().copied(), "note:line-owner");
            }
            unmarked_before = Some(places);
            continue;
          }
          // (a line under one name, the only name of its run of rows, that is the collaboration it speaks for is the
          // name's collaboration contact, no part of the name and no affiliation (row C5, the old reader's
          // `collaboration_line_before`; FIDELITY_CHARTER §4, OXIDIZED_DESIGN_DIVERGENCES #946): `{\slshape Maxime
          // Defurne$^1$}\\ {\footnotesize for the Hall A DVCS collaboration}`, 1502.03246 (review rev94a S6 joined it to
          // the name, as Perl's one personname); `Carolyn Brown$^1$\\ on behalf of the Bcool project team`, 1310.6507;
          // a bracketed line under rows of names is theirs, R12: `(The Nearby Supernova Factory)`, 1511.01102)
          if speaks_for(&text)
            && let [c] = above[..]
            && run.iter().all(|&r| r == c)
            && self.out.creators[c].affiliations.is_empty()
            && self.out.creators[c].contacts.is_empty()
          {
            // (sourced at the line's ink, its first inked event to its last: the emitter writes it in the markup it prints
            // in, `{\footnotesize …}`'s size — review revNL1 S3)
            let mut inked_spans = line
              .events
              .iter()
              .filter(|e| inked(e))
              .filter_map(|e| e.span);
            let span = inked_spans
              .clone()
              .next()
              .zip(inked_spans.next_back())
              .map(|((start, _), (_, end))| (start, end));
            self.collaboration(index, span, &[c], text.trim());
            for note in notes {
              self.note(index, note, Some(c), "note:line-owner");
            }
            continue;
          }
          let starts = source.1.map(|(start, _)| start);
          match open_group.take() {
            Some((until, places)) if starts.is_some_and(|start| start < until) => {
              self.continue_entry(&Continued::Joined(places.clone()), &text);
              open_group = Some((until, places));
              continue;
            },
            _ => {},
          }
          // (a line of addresses alone, placed below, leaves the entry open: the unmarked line after it is the entry's, as
          // today's one `\\`-joined contact, no affiliation of every name — `$^2$CNRS, LAB \\ email \tt{…} \\ To appear in …`,
          // 1209.0869; 2606.23976, 0910.1386, 1411.2744)
          let carried = if addresses_in(&text).is_empty() || !only_addresses(&text) {
            if let Some(carried) = carried {
              self.continue_line(index, &carried, &text, notes, above.last().copied());
              continues = Some(carried);
              continue;
            }
            None
          } else {
            carried
          };
          // (an institution line with its address beside it is an unmarked line, the address in its text, as a continued
          // line's is — `1. Chester F. Carlson Center …, NY 14623; JHK's email: jhk@cis.rit.edu` under marked names,
          // astro-ph0202139, whose institutions the address branch lost; its hand-written `1.` stays unread, OPEN_ROWS
          // FM-24)
          let beside: String = text
            .split_whitespace()
            .filter(|w| !w.contains('@'))
            .collect::<Vec<_>>()
            .join(" ");
          // (an address decides only its own ink: the line's other text, its label word aside, is the item it continues —
          // the unmarked line above, of its column, ending in a comma — or, under one names row and no column, every
          // author's of the row, an unmarked line as any, the address taken out of its text by the chain
          // (`take_out_entry_addresses`), as today's gives the line to them all: `Ann Able and Bob Baker\\ Institute of
          // Things, Univ.\ A,\\ Dresden, Saxony 01062, Germany (bob.baker@a.org)`, whose city went to Bob alone; 1605.03019's
          // `\quad`-parted names over `\small{IDSIA, 6928 Manno, Switzerland}{\{adam, samuli, monaldo\}@idsia.ch}`,
          // whose institute went to no one; 2203.10777)
          let runs = address_runs(&text, &|_| false);
          let labels: Vec<String> = runs
            .iter()
            .filter_map(|run| run.label.as_deref())
            .flat_map(printed_words)
            .map(str::to_lowercase)
            .collect();
          // (the address reader's runs out, and any address it leaves where it prints — a tagged one, `ann@a.org (Ann)`)
          let rest = strip_address_runs(&text, &runs)
            .split_whitespace()
            .filter(|w| !w.contains('@'))
            .collect::<Vec<_>>()
            .join(" ");
          let addresses = addresses_in(&text);
          let other_ink = !reads_as_email_label(&rest)
            && printed_words(&rest).any(|word| !labels.contains(&word.to_lowercase()));
          let continues_above = unmarked_line_above
            && n > 0
            && lines[n - 1].column == line.column
            && lines[n - 1]
              .events
              .iter()
              .rev()
              .find(|e| !matches!(e.kind, EventKind::Layout))
              .is_some_and(|e| matches!(e.kind, EventKind::Sep(SepKind::Comma)));
          // (its addresses spelling one author, the line is that author's, its other inks with it: `POC: A. Able
          // (ann.able@a.org)`, 2110.03506; `ann.able@a.org (corresponding)`, `Written::lost_inks`)
          let owners: std::collections::BTreeSet<Option<usize>> = addresses
            .iter()
            .map(|address| self.spelled(address, &run))
            .collect();
          let one_owner = matches!(owners.iter().collect::<Vec<_>>()[..], [Some(_)]);
          // (nor is a name its own address spells, a name the reading lost, any row's: `Ann Able ann@a.org\\ Bob Baker
          // bob@b.org`, the block written whole for it, `LostInk::names_fail`)
          let own_name = addresses
            .iter()
            .any(|address| spelled_name(&[rest.as_str()], address).is_some());
          let one_row =
            !run.is_empty() && lines.iter().all(|l| l.column == 0) && !after_labelled && !one_owner;
          // (with the emitter's switch on alone, `emitting`: the switch-off output stays as it was, brief LO1)
          let row_line = self.emitting
            && other_ink
            && !own_name
            && !only_addresses(&text)
            && (continues_above || one_row);
          if !addresses_in(&text).is_empty()
            && !row_line
            && (only_addresses(&text) || !is_institution(&beside))
          {
            // (a list of addresses running on over the next lines — the line ending in a comma, each next line addresses
            // alone — is one print: `E-mails: 202010106047@mail.scut.edu.cn, \\ scbingli@scut.edu.cn`, whose second
            // address spells Bing Li, leaving the first to Bo Wang by elimination, 2312.05446)
            let mut text = text.clone();
            let mut last = n;
            while lines[last]
              .events
              .iter()
              .rev()
              .find(|e| !matches!(e.kind, EventKind::Layout))
              .is_some_and(|e| matches!(e.kind, EventKind::Sep(SepKind::Comma)))
              && let Some(next) = lines.get(last + 1)
              && next.column == line.column
            {
              let next_text = events_text(&next.events);
              if addresses_in(&next_text).is_empty() || !only_addresses(&next_text) {
                break;
              }
              text = s!("{text} {next_text}");
              last += 1;
            }
            skip_to = last;
            continues = carried;
            let candidates: Vec<usize> = (block_creators_start..self.out.creators.len()).collect();
            self.place_line_contacts(&text, "email", &candidates, source, None, Chain::Line);
            // (the notes of the list's lines are the block's: a line of addresses is no one name's, so no place makes
            // its note a name's — the lines read on to were skipped after, and their notes went unread, review
            // revfms2a2_5 finding 6; 2406.07529's `\vphantom{\thanks{Equal contribution.}}` after the last address,
            // which today's XML loses. Their Contact events' addresses are the list's text, placed above)
            let notes: Vec<&Event> = lines[n..=last]
              .iter()
              .flat_map(|l| l.events.iter().copied())
              .filter(|e| e.kind == EventKind::Note)
              .collect();
            for note in notes {
              self.note(index, note, None, "note:address-line");
            }
            continue;
          }
          // (each unmarked line its own contact, as each `\\` line is today's: 0707.1113, 1404.0431; ruling 3 — but under a
          // names row whose names show marks no line answers, the lines are one affiliation of every name, as today's
          // `\\`-joined contact: `Riemann$^1$, … \\ 1- DESY in Zeuthen \\ Platanenallee 6, …`, 0903.2366, 2303.09899; a
          // footnote symbol is no such mark, `Marciniak$^{*}$ \\ Department … \\ Campus …` stays a contact a line,
          // 0707.1113)
          let joined = run.iter().any(|&c| {
            self.out.creators[c]
              .requests
              .iter()
              .any(|r| !is_symbol_label(r))
          });
          // (an address decides only its own ink, its holder too: a line the row holds whose addresses spell one of the
          // row is that one's e-mail, the line's other text the row's, as the switch off places it — 2203.10777's
          // `Dresden, Saxony 01062, Germany (martin.waltz@tu-dresden.de)` under Martin Waltz, written whole with the
          // address on int146a's LO1 (review revLO1 1); `Ann Able and Bob Baker\\ Univ A\\ Present address: Dept X, Univ Y
          // (ann.able@y.org)`, Ann's e-mail and both authors' text without it (revLO1 2). Only an address printed last on
          // its line, so the emitter cuts the rest from its tokens; with the emitter's switch on alone, `emitting`)
          // (every address printed last: no other ink between or after them — `Dresden, Germany (bob.baker@a.org); Leipzig
          // (bob@b.org)` keeps its reading, review revLO2 3; the emitter's cut ends at the first one, `prefix_tokens`)
          let own_ink = || {
            let cuts = address_cuts(&text, &runs);
            let mut read: Vec<&String> = runs.iter().flat_map(|r| &r.addresses).collect();
            read.sort_unstable();
            read.dedup();
            let inkless =
              |from: usize, to: usize| !text[from..to].chars().any(char::is_alphanumeric);
            cuts
              .last()
              .filter(|&&(_, end)| inkless(end, text.len()))
              .filter(|_| cuts.windows(2).all(|pair| inkless(pair[0].1, pair[1].0)))
              .filter(|_| {
                read.len() == addresses.len() && addresses.iter().all(|a| read.contains(&a))
              })
              .map(|_| strip_address_runs(&text, &runs))
              .filter(|rest| rest.chars().any(char::is_alphanumeric))
          };
          let held = (self.emitting && one_owner && other_ink && !own_name)
            .then(own_ink)
            .flatten();
          let text = match (owners.iter().next(), held) {
            (Some(&Some(owner)), Some(rest)) => {
              for address in &addresses {
                // (where the address prints as a contact of its own, its span: the emitter writes its tokens)
                let span = line
                  .events
                  .iter()
                  .find(|e| e.kind == EventKind::Contact && e.text.contains(address.as_str()))
                  .and_then(|e| e.span)
                  .or(source.1);
                self.attach(
                  Some(owner),
                  Assoc {
                    role:   "email".into(),
                    text:   address.clone(),
                    label:  None,
                    rule:   "line:row-address owner:spelled".into(),
                    source: Some((index, span)),
                    target: None,
                  },
                  false,
                );
              }
              rest
            },
            _ => text,
          };
          let mut places = Vec::new();
          for owner in &run {
            places.push((*owner, self.out.creators[*owner].affiliations.len()));
            self.attach(
              Some(*owner),
              Assoc {
                role:   "affiliation".into(),
                text:   text.clone(),
                label:  None,
                rule:   if joined {
                  "line:unmarked-joined R12:row-above".into()
                } else {
                  "line:unmarked R12:row-above".into()
                },
                source: Some(source),
                target: None,
              },
              false,
            );
          }
          let line_end = line
            .events
            .iter()
            .rev()
            .find_map(|e| e.span)
            .map(|(_, end)| end);
          let group_end = line
            .events
            .iter()
            .filter_map(|e| e.span)
            .map(|(_, end)| end)
            .max();
          if let (Some(line_end), Some(group_end)) = (line_end, group_end)
            && group_end > line_end
            && !places.is_empty()
          {
            open_group = Some((group_end, places.clone()));
          }
          if !places.is_empty() {
            unmarked_before = Some(places.clone());
          }
          if joined && !places.is_empty() {
            continues = Some(Continued::Joined(places));
          }
          if above.is_empty() {
            self.attach(
              None,
              Assoc {
                role:   "affiliation".into(),
                text:   text.clone(),
                label:  None,
                rule:   "line:unmarked owner:block".into(),
                source: Some(source),
                target: None,
              },
              false,
            );
          }
          for note in notes {
            self.note(index, note, above.last().copied(), "note:line-owner");
          }
        },
        None => {},
      }
    }
    for (contact, above) in block_links {
      let names = block_creators_start..self.out.creators.len();
      let owners = if above.iter().copied().eq(names) {
        &[][..]
      } else {
        &above[..]
      };
      self.contact_event(index, contact, owners, "line:contact R12:row-above");
    }
    self.line_column = None;
    self.block_addresses.clear();
  }

  /// A labelled line's entries, each pending its requests (ruling 1; ruling 2 for a symbol): an e-mail entry one
  /// contact per address, any other one affiliation or legend. The entry an unmarked line under it continues.
  pub(super) fn labelled_entries(
    &mut self,
    index: usize,
    entries: Vec<Entry<'_>>,
    extra: &str,
  ) -> Option<Continued> {
    let mut continues = None;
    for entry in entries {
      // (the closing half of a bracketed mark is no text of the entry: `($^1$) Univ A`, 1004.4809)
      let text = events_text(&entry.events)
        .trim_start_matches(|c: char| ")]-.:".contains(c) || c.is_whitespace())
        .to_string();
      // (the entry's span is its every event's, its first's start to its last ink's end, as a joined name's
      // (`joined_single_name`): the writer cuts the entry by it, its letters the entry's text — an entry of several
      // events, its words split at an inner `and` or comma, was its first piece's span alone, so the cut failed ("a block
      // line does not cut") and the whole line went to the first mark's name: `$^{1}$Univ of Science and
      // Technology~~~~$^{3}$Univ C`, dev20k 2111.07950, 1503.06795, 2504.05692, 2008.02300, 2404.01856)
      // (a note after the entry's last other ink is the block's, no name's: Perl digests it inside the entry's contact,
      // where `\thanks` is `\lx@add@pubnote@thanks` (Base_Utility.pool.ltxml:332), and prints it once, a `<pubnote>` held by
      // no author — 2504.09745's `$^{\ddagger}$Ericsson Canada\thanks{This work was supported …}`; the switch off prints it
      // once per author its contact is relocated to (two pubnotes there), the emitter's switch on before this a nameless
      // creator's note and the entry written with the note in it (review revLO2 1). The entry's span ends before it; read
      // with the emitter's switch on alone, `emitting`)
      let trailing = if self.emitting {
        entry
          .events
          .iter()
          .rev()
          .take_while(|e| !inked(e) || e.kind == EventKind::Note)
          .filter(|e| e.kind == EventKind::Note)
          .count()
      } else {
        0
      };
      // (an entry of notes alone keeps them, as before)
      let trailing = if trailing < entry.events.iter().filter(|e| inked(e)).count() {
        trailing
      } else {
        0
      };
      let notes: Vec<&Event> = entry
        .events
        .iter()
        .rev()
        .filter(|e| e.kind == EventKind::Note)
        .take(trailing)
        .copied()
        .collect();
      let span = match (
        entry.events.first().and_then(|e| e.span),
        entry
          .events
          .iter()
          .rev()
          .filter(|e| inked(e))
          .nth(trailing)
          .and_then(|e| e.span),
      ) {
        (Some((start, _)), Some((_, end))) if end >= start => Some((start, end)),
        (first, _) => first,
      };
      // (an entry a `\footnotemark` leads: what a name's footnote mark answers, `mark:footnote` — 1607.07280)
      let footnote = entry
        .events
        .iter()
        .find(|e| e.kind == EventKind::Mark)
        .is_some_and(|e| printed_by_footnotemark(e));
      let extra = if footnote {
        s!("{extra} mark:footnote")
      } else {
        extra.to_string()
      };
      let rule = if entry.symbol {
        s!("ruling2:symbol-line{extra}")
      } else {
        s!("ruling1:request{extra}")
      };
      let contacts: Vec<&Event> = entry
        .events
        .iter()
        .copied()
        .filter(|e| e.kind == EventKind::Contact)
        .collect();
      // (an entry of addresses alone is e-mails; one with other text beside its address is that text's affiliation, the
      // address in it, as a continued line's is — `1. Chester F. Carlson Center …, NY 14623; JHK's email:
      // jhk@cis.rit.edu`, astro-ph0202139, whose institutions were lost)
      let role = if !addresses_in(&text).is_empty()
        && only_addresses(&text)
        && contacts
          .iter()
          .all(|c| c.role.as_deref() != Some("affiliation"))
      {
        "email"
      // (a legend's mark is a symbol: a footnote symbol, or a mark glyph of no `\@fnsymbol` kind — `${\star}$ to whom
      // correspondence should be addressed` under `Pace,$^{2,\star}$`, 2401.00056, FM S2a.2 r7; an entry of no label
      // has none, review revfms2a2_7 finding 7)
      } else if is_legend_text(&text)
        && (entry.symbol
          || (!entry.labels.is_empty()
            && entry
              .labels
              .iter()
              .all(|l| !l.is_empty() && !l.chars().any(char::is_alphanumeric))))
      {
        "legend"
      } else {
        "affiliation"
      };
      if role == "email" {
        for address in addresses_in(&text) {
          self
            .pending
            .push((entry.labels.clone(), entry.symbol, Assoc {
              role:   "email".into(),
              text:   address,
              label:  entry.labels.first().cloned(),
              rule:   rule.clone(),
              source: Some((index, span)),
              target: None,
            }));
        }
        continues = None;
      } else {
        // (a line led by one mark printing distinct footnote symbols together names the names carrying them all:
        // `$^{\dagger\ddagger}$ Project lead` under `Ann Able$^{\dagger}$` and `Bob Baker$^{\dagger\ddagger}$` is Bob
        // Baker's alone, 2602.19190 — the old reader's 77p r10 rule; `$^{\dagger,\ddagger}$`, two marks, is either's)
        let compound = entry.symbol && entry.compound && {
          let mut symbols: Vec<&String> = entry.labels.iter().collect();
          symbols.sort_unstable();
          symbols.dedup();
          symbols.len() > 1 && symbols.iter().all(|l| is_symbol_label(l))
        };
        self
          .pending
          .push((entry.labels.clone(), entry.symbol, Assoc {
            role: role.into(),
            text,
            label: entry.labels.first().cloned(),
            rule: rule.clone(),
            source: Some((index, span)),
            target: None,
          }));
        if compound {
          self.compound.push(self.pending.len() - 1);
        }
        continues = Some(Continued::Pending(self.pending.len() - 1));
      }
      for note in notes.into_iter().rev() {
        self.note(index, note, None, "note:entry");
      }
    }
    continues
  }

  /// A line continuing the entry above it, an unmarked one or one a glyph no name requests leads: its text the entry's
  /// ([`Self::continue_entry`]); its notes, under a labelled entry, the block's, as a line of notes alone under it
  /// (`$^{2}$Univ B\\ Funded by X.\thanks{Grant 5.}`, review revLO2 base; `emitting`), else the name's above it.
  fn continue_line<'e>(
    &mut self,
    index: usize,
    carried: &Continued,
    text: &str,
    notes: impl IntoIterator<Item = &'e Event>,
    above: Option<usize>,
  ) {
    self.continue_entry(carried, text);
    let (owner, rule) = if self.emitting && matches!(carried, Continued::Pending(_)) {
      (None, "note:line-owner note:entry")
    } else {
      (above, "note:line-owner")
    };
    for note in notes {
      self.note(index, note, owner, rule);
    }
  }

  /// An unmarked line continuing the entry before it: its text appended, and the entry's lines kept
  /// ([`Resolved::continued`]).
  pub(super) fn continue_entry(&mut self, carried: &Continued, text: &str) {
    let mut sources: Vec<(Option<Source>, String)> = Vec::new();
    let mut extend = |assoc: &mut Assoc| {
      if !sources.iter().any(|(s, _)| *s == assoc.source) {
        sources.push((assoc.source, assoc.text.clone()));
      }
      assoc.text = s!("{} {text}", assoc.text);
      if !assoc.rule.contains("line:continued") {
        assoc.rule.push_str(" line:continued");
      }
    };
    match carried {
      Continued::Pending(at) => {
        if let Some((_, _, assoc)) = self.pending.get_mut(*at) {
          extend(assoc);
        }
      },
      Continued::Joined(places) => {
        for &(c, i) in places {
          if let Some(assoc) = self.out.creators[c].affiliations.get_mut(i) {
            extend(assoc);
          }
        }
      },
    }
    for (source, before) in sources {
      let Some(source) = source else {
        continue;
      };
      match self.continued.iter_mut().find(|(s, _)| *s == source) {
        Some((_, lines)) => lines.push(text.to_string()),
        None => self
          .continued
          .push((source, vec![before, text.to_string()])),
      }
    }
  }

  /// A Contact event (an `\email`, `\url`, an in-block `\affil`): a labelled one waits for its requests; an e-mail by
  /// the owner chain among `owners`; any other to `owners` (the name before, the row above).
  pub(super) fn contact_event(
    &mut self,
    index: usize,
    contact: &Event,
    owners: &[usize],
    rule: &str,
  ) {
    let (role, text) = contact_parts(contact);
    let assoc = Assoc {
      role: role.clone(),
      text,
      label: contact.label.clone(),
      rule: rule.to_string(),
      source: Some((index, contact.span)),
      target: link_target(contact),
    };
    if let Some(label) = contact.label.as_deref().filter(|_| role == "affiliation") {
      self.pending.push((mark_labels(label), false, Assoc {
        rule: "ruling1:request".into(),
        ..assoc
      }));
      return;
    }
    if owners.is_empty() {
      self.attach(
        None,
        Assoc {
          rule: "owner:block".into(),
          ..assoc
        },
        false,
      );
      return;
    }
    // (an e-mail given to a row of names, or printing several addresses, is placed address by address by the owner
    // chain, ruling 3 — `\email{sfranco@kitp.ucsb.edu, ahanany@imperial.ac.uk}` under four names, 0809.3237)
    // (a link to an address by its address, not the anchor it prints: `\href{mailto:…}{…}`, FM S2a.2 r9 — only a
    // `mailto:` link's: an `\href{http://…}{a@x.org}` is read by the address its text prints, review revfms2a2_9 finding
    // 5; a link printing one address keeps its anchor as the text, as the single-owner path does)
    let mailto = contact.target.as_deref().and_then(mailto_address);
    let addresses = mailto.unwrap_or(&assoc.text);
    if role == "email" && (owners.len() > 1 || addresses_in(addresses).len() > 1) {
      let anchor = mailto.is_some().then_some(&assoc);
      self.place_line_contacts(
        addresses,
        "email",
        owners,
        (index, contact.span),
        anchor,
        Chain::Line,
      );
      return;
    }
    for &owner in owners {
      self.attach(Some(owner), assoc.clone(), false);
    }
  }

  /// A Note event: its inner contacts to its owner when they spell it or are its only link (decision 2), the rest left in
  /// its text — its e-mails and urls placed once every name is read, to the author they spell (`answer_note_links`); the
  /// note itself, its whole text, unless its contacts emptied it and were all taken out (ruling 4), decided there when
  /// its links may empty it. The mark it shows on the name it is beside, its request of an entry led by the same
  /// `\footnotemark` (`note_marks`, 1607.07280).
  pub(super) fn note(&mut self, index: usize, note: &Event, owner: Option<usize>, rule: &str) {
    if rule == "note:adjacent"
      && self.notes_marked
      && let Some(at) = owner
      && let Some(shows) = note_shows(note)
    {
      for label in mark_labels(shows) {
        self.note_marks.push((at, label));
      }
    }
    // (what is left of its text once its contacts are taken out decides only whether they emptied it: the note keeps
    // its whole text, "Codebase is available at https://…" — review revfms2a2 finding 1; 2401.17399, 1607.05792,
    // 2603.01292)
    let mut rest = note.text.clone();
    let contacts: Vec<&Event> = note
      .inner
      .iter()
      .filter(|e| e.kind == EventKind::Contact)
      .collect();
    // (main-session decision 2 of 2026-10-22, user ruling 2026-10-20 #4 — no guessing: a note's inner contact is its
    // owner's only when it spells the owner, or is the note's only link with the note's mark shown on the owner; the rest
    // stay in the note's text. A note holding several links names others or the paper as often as its owner — a
    // co-author's homepage, a video, data records, four co-authors' ORCIDs all went to the last author: 2001.02600,
    // 1803.04996, 2504.08644, 2602.13176 —, and a class printing no note marks places a note by position alone,
    // `Policy::unmarked_notes`)
    // (a note no name holds — a line's, an address list's — has no owner to fall back to: its link goes to the author it
    // spells, or stays in its text, never the block's — main-session decision 4 on FM S2a.2 r7, review revfms2a2_7
    // finding 2; the block's url went on to a sole author by `sole_author_takes_the_block`)
    let fallback = contacts.len() == 1 && self.notes_marked && owner.is_some();
    // (each link with its target, `Assoc::target`: a url's page is what says whose it is, an e-mail link's address,
    // `place_note_link`)
    let mut links: Vec<(Assoc, bool)> = Vec::new();
    let mut all_taken = true;
    for inner in &contacts {
      // (an e-mail call of a binding's meaning is no guess: its code routes the address to the current author, the note's
      // owner, as Perl's API does, `\lx@add@email`, and the note written prints it nowhere — siamart's `\thanks{…,
      // \email{wning@vt.edu}, \email{amiedlar@vt.edu}.}`, 2511.14713; REVTeX's; main-session ruling 2026-11-08. With the
      // emitter's switch on alone, `emitting`)
      let routed = self.emitting
        && owner.is_some()
        && inner.role.as_deref() == Some("email")
        && inner.meaning == EmailMeaning::Binding;
      let (role, text) = contact_parts(inner);
      let assoc = Assoc {
        role,
        text,
        label: inner.label.clone(),
        rule: s!("{rule} ruling4:inner-contact"),
        source: Some((index, inner.span)),
        target: link_target(inner),
      };
      // (an e-mail or a url goes to the author it spells before the one the note is on — charter §4, by evidence:
      // `Chauvet\thanks{… jocelyn.chauvet@…; xavier.bry@…}`, 1908.04020; 2304.00435; a co-author's homepage, 2001.02600
      // — read once every name is, `answer_note_links`: the names after the note in its block are not read yet)
      let link = matches!(assoc.role.as_str(), "email" | "url");
      if link {
        links.push((assoc, fallback || routed));
      } else if fallback {
        self.attach(owner, assoc, false);
      } else {
        all_taken = false;
        continue;
      }
      if !inner.text.is_empty() {
        rest = rest.replacen(inner.text.as_str(), "", 1);
      }
    }
    let emptiable = !contacts.is_empty() && all_taken && rest.chars().all(|c| !c.is_alphanumeric());
    if emptiable && !links.is_empty() {
      // (whether its links all become contacts, and it is dropped, once every name is read)
      self.note_links.push(NoteLink::Note {
        index,
        note: Box::new(note.clone()),
        owner,
        rule: rule.to_string(),
        links,
      });
      return;
    }
    for (assoc, fallback) in links {
      self
        .note_links
        .push(NoteLink::Link { owner, assoc, fallback });
    }
    if emptiable {
      self.drop_emptied_note(index, note, rule);
      return;
    }
    self.keep_note(index, note, owner, rule);
  }

  /// A note its contacts emptied, all taken out (ruling 4): dropped, its text kept for the comparison.
  pub(super) fn drop_emptied_note(&mut self, index: usize, note: &Event, rule: &str) {
    self.out.dropped.push(Assoc {
      role:   "note".into(),
      text:   note.text.trim().to_string(),
      label:  note.label.clone(),
      rule:   s!("{rule} ruling4:emptied"),
      source: Some((index, note.span)),
      target: None,
    });
  }

  /// A note kept: a legend of the label its leading mark names, else a note of `owner`, its whole text. A note nothing
  /// in which prints is none (ruling 4 as the user's rulings of 2026-10-30 extend it, OXIDIZED_DESIGN_DIVERGENCES #916:
  /// `\thanks{}`, `\thanks{~}`, as the XML drops it): dropped, as an emptied one is.
  pub(super) fn keep_note(&mut self, index: usize, note: &Event, owner: Option<usize>, rule: &str) {
    if note.inkless {
      self.out.dropped.push(Assoc {
        role:   "note".into(),
        text:   note.text.trim().to_string(),
        label:  note.label.clone(),
        rule:   s!("{rule} ruling4:inkless"),
        source: Some((index, note.span)),
        target: None,
      });
      return;
    }
    let live = self.live();
    let numbered = self.emitting && !note.footnote && numbered_thanks(note);
    // (one such `\thanks` holding several marked entries, `\thanks{$^{1}$UC Berkeley, $^{2}$Toyota Motor North America,
    // $^{3}$Princeton university}`: each its own entry by its mark, as the switch off cuts it — 2509.19480, 2206.07648,
    // where the whole stayed one note of the last name. Its lexed text cut before each top-level mark (`mark_led_pieces`, the API
    // reader's cut, not the switch off's `split_marked_pieces`), so an entry's own comma stays its text, `$^{1}$Dept A, Univ A,
    // $^{2}$Univ B`; each piece led by raised marks with labels, else it stays one note, as before. Each piece is sourced at
    // its own span, its first event's start to its last's end (a mark's formula whole), so the writer writes the reading's cut, not a cut of its own:
    // `$^{1}$Univ A; $^{2}$Univ B. $^{*}$Corresponding author.`, whose third piece the writer's re-cut of the whole note lost,
    // review revG5 1)
    if numbered {
      let pieces = mark_led_pieces(&note.body);
      let entries: Option<Vec<MarkedEntry>> = pieces
        .iter()
        .map(|piece| {
          let (start, end) = events::leading_marks(piece);
          let marks: Vec<&Event> = piece[start..end]
            .iter()
            .filter(|e| e.kind == EventKind::Mark)
            .collect();
          let labels: Vec<String> = marks
            .iter()
            .flat_map(|mark| mark_labels(mark.label.as_deref().unwrap_or("")))
            .collect();
          (!marks.is_empty() && marks.iter().all(|e| raised_mark(e)) && !labels.is_empty()).then(
            || {
              let text = events_text(&piece[end..].iter().collect::<Vec<_>>());
              let first = piece.iter().find_map(Event::extent);
              let last = piece.iter().rev().find_map(Event::extent);
              let span = first.zip(last).map(|(first, last)| (first.0, last.1));
              (labels, text.trim().to_string(), span.or(note.span))
            },
          )
        })
        .collect();
      if let Some(entries) = entries.filter(|entries| entries.len() > 1) {
        // (the separator words the cut drops between two entries are the cut's, no entry's text, as an API call's
        // (`$^{1}$Univ A, $^{2}$Univ B and $^{3}$Univ C`): recorded dropped, so no word of the note reads as unwritten)
        let word = |e: &Event| match &e.kind {
          EventKind::Sep(SepKind::Word(word)) => Some(word.clone()),
          _ => None,
        };
        let mut cut: Vec<String> = note.body.iter().filter_map(word).collect();
        for kept in pieces.iter().flatten().filter_map(word) {
          if let Some(at) = cut.iter().position(|w| *w == kept) {
            cut.remove(at);
          }
        }
        if !cut.is_empty() {
          self.out.dropped.push(Assoc {
            role:   "separator".into(),
            text:   cut.join(" "),
            label:  None,
            rule:   s!("{rule} note:split-by-marks"),
            source: Some((index, note.span)),
            target: None,
          });
        }
        for (labels, text, span) in entries {
          let role = if is_legend_text(&text) {
            "legend"
          } else {
            "affiliation"
          };
          self.pending.push((labels.clone(), false, Assoc {
            role: role.into(),
            text,
            label: labels.first().cloned(),
            rule: s!("{rule} note:legend ruling1:request note:split-by-marks"),
            source: Some((index, span)),
            target: None,
          }));
        }
        return;
      }
    }
    // (a note opening with a Mark some name requests is that label's legend: `\thanks{$^{1}$ Canizaro … Center}`
    // under `Newaz$^{1,2,*}$`, 2311.16145 — the mark it prints, its leading Mark event; its owners by ruling 1)
    // (one whose text holds a later mark of other requested labels, none of the first's, is the legends of several
    // entries, no one label's: IEEEtran's `\IEEEcompsocitemizethanks{\IEEEcompsocthanksitem \Mark{1}College …
    // \IEEEcompsocthanksitem \Mark{2}College …}` was given whole to the authors of mark 1, 1802.03043 — a note of the
    // name before, as today's; a later `\textsuperscript{1,2}` sharing the first's label keeps the first's reading, as
    // today's, 2507.07006)
    if let Some((labels, legend)) = leading_mark(note)
      && labels
        .iter()
        .any(|l| !label_owners(&self.out.creators, &live, l).is_empty())
      && !note
        .inner
        .iter()
        // (in a re-routed `\thanks`, its raised marks alone: a glyph is its text, `*Corresponding authors` closing
        // `$^{4}$ Department of Mechanical Engineering, …`, 2505.06451)
        .filter(|e| e.kind == EventKind::Mark && !e.line_first && (!numbered || raised_mark(e)))
        .map(|e| mark_labels(e.label.as_deref().unwrap_or("")))
        .any(|later| {
          !later.is_empty()
            && later.iter().all(|l| !labels.contains(l))
            && later
              .iter()
              .any(|l| !label_owners(&self.out.creators, &live, l).is_empty())
        })
    {
      let symbol = labels.iter().all(|l| is_symbol_label(l));
      // (with the emitter's switch on, the switch-off procedure's kind, in its order (P2 NT; ported strictly, P2r2): a
      // `\footnote` is a note (`\lx@add@note`, frontmatter_fields.rs); a `\thanks` is an affiliation only where
      // `\lx@add@thanks` re-routes it, its content opening with `$^` or `\textsuperscript` whose first character is an
      // ASCII digit (`starts_with_affiliation_mark`, OXIDIZED_DESIGN #52) — here the note's leading Mark, a superscript
      // whose printed text opens with a digit —, else a note; an affiliation whose whole text is a legend is then a note
      // (the Kind step for a legend, `insert::kind_legend_contacts`; here `is_legend_text`). The note goes to its mark's
      // carriers, not the name before it (FIDELITY_CHARTER §4: a symbol-led line goes to the authors carrying the
      // symbol, affiliation or legend alike): `\thanks{\IEEEauthorrefmark{2} Also with …}`, 1107.5730; `\thanks{$\diamond$
      // Work conducted during an internship …}`, 2205.02029. One departure, the reading's own entry kept: a note that
      // prints e-mail addresses and is no legend stays the entry the reading made it, whose addresses go each to the
      // carrier it spells (FIDELITY_CHARTER §4's symbol-line rule; the switch-off note keeps them in its text, every
      // address on the note's owner): IEEEtran journal's `\thanks{\IEEEauthorrefmark{1}The authors are with …, E-mails:
      // a@, b@, c@}`, 1610.05419, 2405.06709 — review revP2 finding 1)
      let role = if (self.emitting && !numbered && addresses_in(&legend).is_empty())
        || is_legend_text(&legend)
        || symbol
      {
        "legend"
      } else {
        "affiliation"
      };
      self.pending.push((labels.clone(), symbol, Assoc {
        role:   role.into(),
        text:   legend.clone(),
        label:  labels.first().cloned(),
        rule:   s!("{rule} note:legend ruling1:request"),
        source: Some((index, note.span)),
        target: None,
      }));
      return;
    }
    let role = note
      .role
      .as_deref()
      .unwrap_or("thanks")
      .split(' ')
      .next()
      .unwrap_or("thanks")
      .to_string();
    let rule = rule.to_string();
    self.attach(
      owner,
      Assoc {
        role,
        text: note.text.trim().to_string(),
        label: note.label.clone(),
        rule,
        source: Some((index, note.span)),
        target: None,
      },
      true,
    );
  }
}

/// A raised mark: a superscript or `\textsuperscript`, what Perl's readers split at (`@authorsup_markers`).
fn raised_mark(e: &Event) -> bool {
  e.kind == EventKind::Mark
    && e
      .role
      .as_deref()
      .is_some_and(|r| r.starts_with("sup") || r.starts_with("math"))
}

/// A `\thanks` whose text opens with a raised number: what `\lx@add@thanks` re-routes as affiliations, each segment
/// its raised marks part it into an `\lx@affiliation@withsup` entry labelled by its mark (frontmatter_fields.rs
/// `\lx@add@thanks`, `starts_with_affiliation_mark`, OXIDIZED_DESIGN #52; `split_wrapped_affiliation_marks`). Read with
/// the emitter's switch on alone (`emitting`).
fn numbered_thanks(note: &Event) -> bool {
  note
    .inner
    .iter()
    .find(|e| e.kind == EventKind::Mark && e.line_first)
    .is_some_and(|mark| {
      raised_mark(mark)
        && mark
          .text
          .trim_start()
          .starts_with(|c: char| c.is_ascii_digit())
    })
}

/// One entry of a marked `\thanks` the reading cuts ([`Resolver::keep_note`]): its mark's labels, its text, its own span.
type MarkedEntry = (Vec<String>, String, Option<(u16, u16)>);
