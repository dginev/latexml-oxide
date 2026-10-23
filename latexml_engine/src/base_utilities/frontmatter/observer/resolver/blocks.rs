//! The resolver's owner chain for an author block (OXIDIZED_DESIGN_DIVERGENCES #837; the module's rules 5-9): the
//! block's lines by type, labelled entries to the names requesting them, unlabelled lines to the row above, a
//! line's contacts by their own mark, spelling, order, the group's only author or elimination, and its notes.

use super::*;

impl Resolver {
  /// Contacts of `text` (one per printed address, ruling 3; the text whole when it holds none), owned by the chain:
  /// the spelled ones first, then — the one address left, and the one candidate left without an address — by
  /// elimination (R10), then the rest of the chain.
  pub(super) fn place_line_contacts(
    &mut self,
    text: &str,
    role: &str,
    candidates: &[usize],
    source: (usize, Option<(u16, u16)>),
    anchor: Option<&Assoc>,
  ) {
    let addresses = addresses_in(text);
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
          rule:   rule.into(),
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
    let in_order = addresses.len() == candidates.len()
      && candidates.len() >= 2
      && spelled
        .iter()
        .zip(candidates)
        .all(|(s, c)| s.is_none_or(|s| s == *c));
    let address_count = addresses.len();
    // (a link printing one address: its anchor the contact's text, its address the target — position 2, review
    // revfms2a2_9 finding 5; 1607.00911's `\href{mailto: edremets@…}{* edremets@…}`)
    let anchor = anchor.filter(|_| address_count == 1);
    for (at, (address, spelled)) in addresses.into_iter().zip(spelled).enumerate() {
      let (owner, rule) = match spelled {
        Some(owner) => (Some(owner), "owner:spelled"),
        None if in_order => (Some(candidates[at]), "owner:order"),
        None if unspelled == 1 && candidates.len() > 1 && left.len() == 1 => {
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
          _ => self.corresponding(candidates),
        },
      };
      self.attach(
        owner,
        Assoc {
          role:   "email".into(),
          text:   anchor.map_or_else(|| address.clone(), |a| a.text.clone()),
          label:  None,
          rule:   s!("{rule} ruling3:one-per-print"),
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
    let events = &glue_shown_runs(&split_text_marks(&join_ordinals(&orcid_link_names(events))));
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
    // the entry the previous line ended with, which an unmarked line under it continues (an address's second line:
    // `$^1$Dept of X \\ 1-1-1 Street, City`, 0803.3589; one affiliation, as today's `\\`-joined contact)
    let mut continues: Option<Continued> = None;
    let mut first_in_column: Vec<usize> = Vec::new();
    // the columns whose last line is a names row ending in a separator (a comma, "and"), which the next line continues
    let mut open_rows: Vec<usize> = Vec::new();
    // the columns whose names row has a line under it already (its affiliation)
    let mut answered_rows: Vec<usize> = Vec::new();
    // the columns whose last line was labelled entries
    let mut labelled_last: Vec<usize> = Vec::new();
    // an unmarked line's affiliation whose group runs on past the line (its places, where the group ends)
    let mut open_group: Option<(u16, Vec<(usize, usize)>)> = None;
    // the places of the unmarked affiliation the line before made, which a line after a break inside the same braced
    // group continues
    let mut unmarked_before: Option<Vec<(usize, usize)>> = None;
    // the last line an address list read on to (`place_line_contacts` over the lines of one list)
    let mut skip_to = 0usize;
    for (n, line) in lines.iter().enumerate() {
      if n > 0 && n <= skip_to {
        continue;
      }
      let unmarked_above = unmarked_before.take();
      let continued = open_rows.contains(&line.column);
      open_rows.retain(|c| *c != line.column);
      // (the line before in the column was labelled entries: an unmarked name under them is a new row's — SYNC row 46,
      // `$^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics`)
      let after_labelled = labelled_last.contains(&line.column);
      labelled_last.retain(|c| *c != line.column);
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
      let above = row_above
        .iter()
        .rev()
        .find(|(c, _)| *c == line.column)
        .map(|(_, r)| r.clone());
      let above = above
        .or_else(|| row_above.last().map(|(_, r)| r.clone()))
        .unwrap_or_default();
      let carried = continues.take();
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
            row_above.push((line.column, row));
            continue;
          }
          labelled_last.push(line.column);
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
            && (column_first
              || later_line_is_names(line, continued, under_affiliation, after_labelled));
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
            row_above.push((line.column, made.creators));
            continue;
          }
          if only_contacts {
            for contact in line.events.iter().filter(|e| e.kind == EventKind::Contact) {
              self.contact_event(index, contact, &above, "line:contact R12:row-above");
            }
            continue;
          }
          // (a line whose marks trail their text is labelled entries: `Nanyang Technological University$^1$,
          // Amazon$^2$`, 2507.05750)
          // (each of its marks one a name shows on another line: a symbol after an affiliation, `Harvard
          // University$^{*}$`, points to a note, 2307.02318)
          let trailing = trailing_entries(&line.events);
          let shown_elsewhere = |label: &String| {
            lines.iter().enumerate().any(|(m, other)| {
              m != n
                && other
                  .events
                  .iter()
                  .any(|e| e.kind == EventKind::Mark && event_labels(e).contains(label))
            })
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
          if text.is_empty() {
            for note in notes {
              self.note(index, note, above.last().copied(), "note:line-owner");
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
          let starts = source.1.map(|(start, _)| start);
          match open_group.take() {
            Some((until, places)) if starts.is_some_and(|start| start < until) => {
              self.continue_entry(&Continued::Joined(places.clone()), &text);
              open_group = Some((until, places));
              continue;
            },
            _ => {},
          }
          if let Some(carried) =
            carried.filter(|_| addresses_in(&text).is_empty() || !only_addresses(&text))
          {
            self.continue_entry(&carried, &text);
            for note in notes {
              self.note(index, note, above.last().copied(), "note:line-owner");
            }
            continues = Some(carried);
            continue;
          }
          if !addresses_in(&text).is_empty() {
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
            let candidates: Vec<usize> = (block_creators_start..self.out.creators.len()).collect();
            self.place_line_contacts(&text, "email", &candidates, source, None);
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
          let joined = above.iter().any(|&c| {
            self.out.creators[c]
              .requests
              .iter()
              .any(|r| !is_symbol_label(r))
          });
          let mut places = Vec::new();
          for owner in &above {
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
      let span = entry.events.first().and_then(|e| e.span);
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
      let role = if !addresses_in(&text).is_empty()
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
        continues = Some(Continued::Pending(self.pending.len() - 1));
      }
    }
    continues
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
      self.place_line_contacts(addresses, "email", owners, (index, contact.span), anchor);
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
    let mut links: Vec<Assoc> = Vec::new();
    let mut all_taken = true;
    for inner in &contacts {
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
        links.push(assoc);
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
        note: note.clone(),
        owner,
        rule: rule.to_string(),
        links,
        fallback,
      });
      return;
    }
    for assoc in links {
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

  /// A note kept: a legend of the label its leading mark names, else a note of `owner`, its whole text.
  pub(super) fn keep_note(&mut self, index: usize, note: &Event, owner: Option<usize>, rule: &str) {
    let live = self.live();
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
        .filter(|e| e.kind == EventKind::Mark && !e.line_first)
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
      let role = if is_legend_text(&legend) || symbol {
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
    let rule = if note.text.is_empty() && note.inner.is_empty() {
      s!("{rule} ruling4:empty-kept")
    } else {
      rule.to_string()
    };
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
