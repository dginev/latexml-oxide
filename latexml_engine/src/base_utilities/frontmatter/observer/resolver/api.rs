//! The resolver's API intake (OXIDIZED_DESIGN_DIVERGENCES #837; the module's rule 10): a frontmatter API call made
//! apart from the blocks — a creator, or a contact by its `label=`, `labelseq=`, leading mark, `\ref` key or
//! `annotate=` run.

use super::*;

/// An API call's marked entries as [`Resolver::api`] cuts them ([`mark_led_pieces`]), each with the text it resolves
/// to under `role` (the e-mails and ORCIDs an affiliation's commands declare taken out, as `api` takes them): the
/// writer's map from the entries of one call to their events, and through their spans to the tokens the document
/// queued (FM S2b, `writer/`).
pub(in crate::base_utilities::frontmatter::observer) fn split_pieces(
  events: &[Event],
  role: &str,
) -> Vec<(String, Vec<Event>)> {
  let splits_contacts = matches!(role, "affiliation" | "address" | "altaffiliation");
  mark_led_pieces(events)
    .into_iter()
    .map(|piece| {
      let refs: Vec<&Event> = piece
        .iter()
        .filter(|e| {
          !(splits_contacts
            && e.kind == EventKind::Contact
            && matches!(e.role.as_deref(), Some("email" | "orcid")))
        })
        .collect();
      (events_text(&refs), piece)
    })
    .collect()
}

impl Resolver {
  /// An empty entry's authors hold its role, as today's attaches the empty contact to them (`annotate=`: the last
  /// author, all, the run of new ones, the last N) — part of today's contact pile too (`distribute_upfront`: amsart's
  /// `\email{}` before the three `\address`/`\email` pairs is a seventh contact, no grid, 1302.1274).
  fn empty_entry_holds_its_role(&mut self, index: usize, annotate: &str, role: &str) {
    let owners = self.annotate_owners(annotate, role);
    for &c in &owners {
      self.roles[c].push(role.to_string());
    }
    if !owners.is_empty() {
      self.empty_piled.push((index, role.to_string(), owners));
    }
  }

  /// The authors today's attaches an unlabelled entry of `role` to by its `annotate=` (Perl
  /// Base_Utility.pool.ltxml:520-536): all, the run of new ones, the last N, else the last one; none before any.
  pub(super) fn annotate_owners(&self, annotate: &str, role: &str) -> Vec<usize> {
    let n = self.out.creators.len();
    if n == 0 {
      Vec::new()
    } else if annotate == "all" {
      (0..n).collect()
    } else if annotate == "new" || annotate.starts_with("run") {
      self.new_run(role)
    } else if let Ok(k) = annotate.parse::<usize>() {
      (n.saturating_sub(k)..n).collect()
    } else {
      vec![n - 1]
    }
  }

  /// The authors an `annotate=run[:<cap>]` contact reaches, as `frontmatter_api.rs` records its run: the run of the
  /// contact of the role before it while no author came since (the same count of authors), else the authors back from
  /// the last to one holding the role, at most <cap> — and the run recorded for the next. Only a call today's attaches
  /// at once records it: a mark-picked line (`_withsup`) waits for its label, and its run is not taken.
  pub(super) fn record_run(&mut self, annotate: &str, role: &str) -> Vec<usize> {
    let n = self.out.creators.len();
    let cap = annotate
      .split_once(':')
      .and_then(|(_, c)| c.trim().parse::<usize>().ok())
      .filter(|c| *c >= 1);
    let run = match &self.last_run {
      Some((r, count, run)) if r == role && *count == n => run.clone(),
      _ => {
        let mut run = Vec::new();
        for c in (0..n).rev() {
          if !run.is_empty() && self.roles[c].iter().any(|r| r == role) {
            break;
          }
          if cap.is_some_and(|cap| run.len() >= cap) {
            break;
          }
          run.push(c);
        }
        run.reverse();
        run
      },
    };
    self.last_run = Some((role.to_string(), n, run.clone()));
    run
  }

  /// An API call made apart from the blocks.
  pub(super) fn api(
    &mut self,
    index: usize,
    tag: &str,
    role: &str,
    attrs: &str,
    events: &[Event],
    policy: &'static Policy,
  ) {
    let split = self.splitting;
    let withsup = attr_value(attrs, "_withsup") == Some("true");
    // its leading marks (`\affil{$^1$Univ X}`), its `\label`s, its text
    let lead_marks: Vec<&Event> = events
      .iter()
      // (a space or a break before the mark does not part it from the content: `\affiliation{ $^a$Fermilab, $^b$UC
      // Davis}`, 0902.0006)
      // nor punctuation, an acmart address part's brace (`{$^1$Gaoling School …`, 2211.15148)
      .skip_while(|e| {
        matches!(
          e.kind,
          EventKind::Sep(SepKind::Gap { .. }) | EventKind::Break | EventKind::Layout
        ) || (e.kind == EventKind::Name && !e.text.chars().any(char::is_alphanumeric))
      })
      .take_while(|e| matches!(e.kind, EventKind::Mark | EventKind::Layout))
      .filter(|e| e.kind == EventKind::Mark)
      .collect();
    let lead: Vec<String> = lead_marks
      .iter()
      .flat_map(|e| mark_labels(e.label.as_deref().unwrap_or("")))
      .collect();
    // (marks a `\footnotemark` printed: what a name's footnote mark answers — the same print, `mark:footnote`, 1607.07280)
    let lead_footnote =
      !lead_marks.is_empty() && lead_marks.iter().all(|e| printed_by_footnotemark(e));
    let keys: Vec<String> = events
      .iter()
      .filter(|e| e.kind == EventKind::Layout && e.role.as_deref() == Some("label"))
      .filter_map(|e| e.label.clone())
      .map(|key| s!("ref:{key}"))
      .collect();
    // (its Contact events, an `\email` inside an `\institute` entry, are contacts of their own, with the entry's owners;
    // an address it prints as text stays in its text, as today's: `strip_addresses` split them off and disagreed with
    // 66 + 52 papers of the 5,000)
    // (an entry of another role — an `\email`, a `\thanks` contact — is its declared role whatever it holds: a
    // `mailto:` link inside an `\email` is the e-mail, review revfms2a2 finding 10, 2506.17875)
    let splits_contacts =
      tag != "ltx:creator" && matches!(role, "affiliation" | "address" | "altaffiliation");
    // (only its e-mails and ORCIDs: a link inside an affiliation is its text, as today's — `\href{…}{Institut Langevin},
    // \href{…}{ESPCI Paris}`, 2609.39586; `Stardust Labs, Ness Ziona, Israel; \url{https://www.stardust-initiative.com}`,
    // 2605.14947; 2308.00495, 2111.07288)
    // (the contacts its commands declare, `\email`, `\orcid`: a link is text there, an e-mail address or an ORCID
    // record it points to too — amsart's `\address{… \\ \href{mailto:…}{…}}`, 2408.13873, 2305.04702)
    // (and an ORCID inside an e-mail call, another kind of contact: `\email{giulio.pascale@unina.it \orcidlink{…}}`,
    // 2402.04675 — the e-mail read "giulio.pascale@unina.it0000-0003-1680-3425")
    let split_out = |e: &Event| {
      e.kind == EventKind::Contact
        && ((splits_contacts && matches!(e.role.as_deref(), Some("email" | "orcid")))
          || (tag != "ltx:creator" && role == "email" && contact_parts(e).0 == "orcid"))
    };
    let refs: Vec<&Event> = events.iter().filter(|e| !split_out(e)).collect();
    let text = events_text(&refs);
    let role_owned = contact_role(Some(role), &text);
    let role = if tag == "ltx:contact" {
      role_owned.as_str()
    } else {
      role
    };
    let inner: Vec<Assoc> = events
      .iter()
      .filter(|e| split_out(e) && !contact_parts(e).1.is_empty())
      .map(|e| Assoc {
        role:   contact_parts(e).0,
        text:   contact_parts(e).1,
        label:  None,
        rule:   String::new(),
        source: Some((index, e.span)),
        target: link_target(e),
      })
      .collect();
    if tag == "ltx:creator" {
      // (its `annotations=`, and the marks its name shows — ruling 1: `\author{Ann Able$^{1}$}` under acmart requests
      // 1, as a block's names line reads it; 2202.06129's shape)
      let requests: Vec<String> = attr_value(attrs, "annotations")
        .map(|l| {
          l.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .chain(
          events
            .iter()
            .filter(|e| e.kind == EventKind::Mark)
            .flat_map(|e| mark_labels(e.label.as_deref().unwrap_or(""))),
        )
        .collect();
      if !text.is_empty() {
        let start = self.out.creators.len();
        self.new_creator(text, "api:creator", index, requests, index);
        if policy.thenote {
          self.thenote_author(
            start,
            attr_value(attrs, "annotations").is_some_and(|l| !l.is_empty()),
          );
        }
      }
      return;
    }
    let note = tag == "ltx:note";
    let role = if role.is_empty() { "contact" } else { role };
    // (authblk: every `\affil` makes the next `\author` step the counter; a label shows the marks)
    let thenote_affil = policy.thenote && role == "affiliation";
    if thenote_affil {
      self.thenote.newaffil = true;
      if attr_value(attrs, "label").is_some_and(|l| !l.is_empty()) {
        self.thenote.labelled = true;
      }
    }
    // (a contact printing nothing is no contact: `\IEEEauthorblockA{\\}` — but today's attaches it all the same, so the
    // authors it reaches hold its role and a later `annotate=new` entry's run stops at them: revtex's `\noaffiliation`
    // under the ADMX Collaboration, 1804.05750; an empty `\affiliation{}` before the marked one, 1507.03321)
    if text.is_empty() && inner.is_empty() && !note {
      if !split && attr_value(attrs, "label").is_none_or(str::is_empty) {
        self.empty_entry_holds_its_role(index, attr_value(attrs, "annotate").unwrap_or(""), role);
      }
      return;
    }
    // a class's `\institute{A \and B}` list numbered by its `\and`s (`labelseq=`: llncs.cls:870-885, OD #791): each
    // piece an entry of its own, numbered in turn
    if attr_value(attrs, "labelseq").is_some_and(|s| !s.is_empty())
      && policy.label_sequence
      && events
        .iter()
        .any(|e| matches!(e.kind, EventKind::Sep(SepKind::And(_))))
    {
      for piece in events.split(|e| matches!(e.kind, EventKind::Sep(SepKind::And(_)))) {
        if piece
          .iter()
          .any(|e| e.kind == EventKind::Name || e.kind == EventKind::Contact)
        {
          self.api(index, tag, role, attrs, piece, policy);
        }
      }
      return;
    }
    // an entry of several marked entries (`\affiliation{$^{*}$Univ A,\\$^{+}$Univ B}`): each entry by its own leading
    // mark, as ruling 1 reads the marks (cond-mat0510562, 1612.09425, 1809.03738, hep-ph0408045, 2509.08362,
    // 1506.00848: the whole went to the first mark's carriers)
    if !split
      && tag != "ltx:creator"
      && attr_value(attrs, "label").is_none_or(str::is_empty)
      && attr_value(attrs, "labelseq").is_none_or(|s| s.is_empty() || !policy.label_sequence)
    {
      let pieces = mark_led_pieces(events);
      if !pieces.is_empty() {
        self.splitting = true;
        for piece in pieces {
          self.api(index, tag, role, attrs, &piece, policy);
        }
        self.splitting = false;
        return;
      }
    }
    // (a class whose `\and` prints the word: the entry's text holds it, `Policy::and_word`)
    if policy.and_word
      && events
        .iter()
        .any(|e| matches!(e.kind, EventKind::Sep(SepKind::And(_))))
    {
      let worded: Vec<Event> = events
        .iter()
        .map(|e| match e.kind {
          EventKind::Sep(SepKind::And(_)) => Event {
            kind: EventKind::Sep(SepKind::Word("and".into())),
            ..e.clone()
          },
          _ => e.clone(),
        })
        .collect();
      self.api(index, tag, role, attrs, &worded, policy);
      return;
    }
    // a contact of a class's address list printed apart from the names (`owner=sequence`, recorded `_sequence=<n>`:
    // jheppub's `\emailAdd`, iopart's `\ead`; OD #805), placed once the whole list is read
    if role == "email" && attr_value(attrs, "_sequence").is_some() {
      let n = self.out.creators.len();
      let group: Vec<usize> = match n.checked_sub(1).map(|last| self.group[last]) {
        Some(last) if policy.call_is_group => (0..n).filter(|&c| self.group[c] == last).collect(),
        _ => (0..n).collect(),
      };
      let assoc = Assoc {
        role:   "email".into(),
        text:   text.clone(),
        label:  None,
        rule:   String::new(),
        source: Some((index, None)),
        target: None,
      };
      self.sequence_calls += 1;
      let place = attr_value(attrs, "_sequence")
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(self.sequence_calls);
      for address in addresses_in(&text) {
        self
          .sequenced
          .push((address, assoc.clone(), group.clone(), place));
      }
      return;
    }
    let mut labels: Vec<String> = Vec::new();
    let mut label_rule = "";
    if let Some(label) = attr_value(attrs, "label").filter(|l| !l.is_empty()) {
      labels = mark_labels(label);
      label_rule = "api:label";
    } else if let Some(seq) =
      attr_value(attrs, "labelseq").filter(|s| !s.is_empty() && policy.label_sequence)
    {
      let at = match self.sequence.iter_mut().find(|(r, _)| r == seq) {
        Some((_, n)) => {
          *n += 1;
          *n
        },
        None => {
          self.sequence.push((seq.to_string(), 1));
          1
        },
      };
      labels = vec![at.to_string()];
      labels.extend(keys.iter().cloned());
      label_rule = "api:labelseq";
    } else if !lead.is_empty() {
      labels = lead;
      label_rule = match (split, lead_footnote) {
        (true, true) => "api:leading-mark api:split-by-marks mark:footnote",
        (true, false) => "api:leading-mark api:split-by-marks",
        (false, true) => "api:leading-mark mark:footnote",
        (false, false) => "api:leading-mark",
      };
    } else if let Some(first) = events
      .iter()
      .find(|e| e.kind == EventKind::Mark && e.label.as_deref().is_some_and(|l| !l.is_empty()))
      .filter(|_| withsup)
    {
      // (a `\lx@affiliation@withsup` line's first superscript is its label wherever it stands, the later ones its text
      // — `\lx@sup@setlabel@affiliation`: `Departments of Physics$^{4}$ and Chemistry$^{5}$, Washington University`
      // goes to the authors marked 4, 0812.4065; `Calle 5 N$^\circ$ 62-00` is labelled ∘, which no one requests,
      // 2106.12584)
      labels = mark_labels(first.label.as_deref().unwrap_or(""));
      label_rule = "api:withsup-mark";
    } else if !keys.is_empty() {
      labels = keys.clone();
      label_rule = "api:ref-label";
    }
    // (an affiliation entry a symbol leads — a footnote symbol or a mark glyph — whose text names the corresponding
    // author or an equal contribution is that legend, a note of the names showing the mark, as a block's labelled line
    // reads it (`labelled_entries`): revtex's `\affiliation{$^1$Dept … \\ $^\star${corresponding author, e-mail:
    // stefano@duke.edu}}` under `Curtarolo$^{1,\star}$` was his affiliation, 0812.1531 — review revfms2a2_7 finding 8;
    // acmart's `\country{$^1$SJTU, $^2$Infinigence-AI, $^*$Equal contributions}`, 2409.11440. Only those: the call
    // declares an affiliation, and an address change is one, and a funding word is an institution's name as often —
    // `$^\dagger$Science \& Finance, Capital Fund Management, 6 Bd Haussmann, …`, 0710.0802)
    // (and only when no piece of it reads as an affiliation — what `is_legend_text`'s funding branch asks of a text, of
    // each name and contact it prints, an address and its label reading as none —, or when it is one piece its marks
    // split an entry into: a legend line nested in an entry holding institutions is no legend of the whole, IEEEtran's
    // `\IEEEauthorblockA{{*Equal Contribution}\\ \IEEEauthorblockA{Tsinghua University} \IEEEauthorblockA{National
    // Supercomputing Center in Wuxi} …}` became one legend note, both institutions out of the affiliation role,
    // 1903.06934 — review revfms2a2_8 finding 2, FM S2a.2 r9; the role change its own tag, `api:legend`; 0812.1531's
    // `corresponding author, e-mail: stefano@duke.edu` and 1902.02631's `These authors contributed equally to this
    // work \\ E-mail: …` stay legends)
    let entry_affiliation = events
      .iter()
      .filter(|e| matches!(e.kind, EventKind::Name | EventKind::Contact))
      .any(|e| reads_as_affiliation(&text_tokens(&e.text)));
    let legend = label_rule.starts_with("api:leading-mark")
      && matches!(role, "affiliation" | "address" | "altaffiliation")
      && !labels.is_empty()
      && labels
        .iter()
        .all(|l| !l.is_empty() && !l.chars().any(char::is_alphanumeric))
      && matches!(classify_thanks(&text), "correspondence" | "contribution")
      && (split || !entry_affiliation);
    let assoc = Assoc {
      role:   if legend { "legend" } else { role }.to_string(),
      text:   text.clone(),
      label:  labels.first().cloned(),
      rule:   String::new(),
      source: Some((index, None)),
      target: None,
    };
    if !labels.is_empty() {
      // (a leading mark of content no `\lx@affiliation@withsup` wraps is ruling 1's reading, not today's: today's
      // attaches the entry where its `annotate=` puts it — what its contact grid sees, `distribute_upfront`)
      let first_pending = self.pending.len();
      for contact in inner {
        // (an e-mail call printing several addresses, a braced group `\email{\{ann,bob\}@x.org}` in an `\institute`
        // entry: each address written out, to the author of the document it spells — `answer_pending`)
        let addresses = written_out(&contact);
        if addresses.is_empty() {
          self.pending.push((labels.clone(), false, Assoc {
            rule: s!("{label_rule} api:inner-contact"),
            label: labels.first().cloned(),
            ..contact
          }));
          continue;
        }
        for address in addresses {
          self.pending.push((labels.clone(), false, Assoc {
            rule: s!("{label_rule} api:inner-contact ruling3:written-out"),
            label: labels.first().cloned(),
            text: address,
            ..contact.clone()
          }));
        }
      }
      if !text.is_empty() {
        self.pending.push((labels, false, Assoc {
          rule: if legend {
            s!("{label_rule} api:legend")
          } else {
            label_rule.to_string()
          },
          ..assoc
        }));
      }
      // (a run today's records for a call it attaches at once — revtex's symbol-led `\affiliation{$^\flat$…}` —, which an
      // unmarked line after it continues: 1303.7097's `\affiliation{CONICET La Plata, …}`, every name's)
      let run = (label_rule.starts_with("api:leading-mark")
        && !withsup
        && attr_value(attrs, "annotate")
          .unwrap_or("")
          .starts_with("run"))
      .then(|| self.record_run(attr_value(attrs, "annotate").unwrap_or(""), role));
      if label_rule.starts_with("api:leading-mark") {
        let owners = run.clone().unwrap_or_else(|| {
          self.annotate_owners(attr_value(attrs, "annotate").unwrap_or(""), role)
        });
        for at in first_pending..self.pending.len() {
          if !withsup {
            self.piled.push((at, owners.clone()));
            // (where an unshown label leaves it: the run its declared `annotate=` reached, today's attachment before
            // the label moves it — revtex's `\affiliation{$^{1}$…}` under unmarked names; not a `\lx@affiliation@withsup`
            // line, labelled at once, nor a call declaring no run)
            if !attr_value(attrs, "annotate").unwrap_or("").is_empty() {
              self.annotated.push((at, owners.clone()));
            }
          }
        }
      }
      // (today's call attaches the entry to its `annotate=new` run first and moves it by its label after: the run's
      // authors hold the role, so a later unlabelled entry's run stops at them — revtex's `\affiliation{$^1$Univ A}`
      // under Tiemann, then `\affiliation{Current address: TRUMPF}` under Kahmann alone, 1506.04004)
      // (a mark the entry prints only: an entry labelled by its call, `\address[l1]`, waits for its label alone, as
      // today's — elsarticle, 1712.00945)
      let annotate = attr_value(attrs, "annotate").unwrap_or("");
      if label_rule.starts_with("api:leading-mark")
        && (annotate == "new" || annotate.starts_with("run"))
      {
        for c in run.unwrap_or_else(|| self.new_run(role)) {
          self.roles[c].push(role.to_string());
        }
      }
      return;
    }
    if text.is_empty() && inner.is_empty() {
      return;
    }
    let annotate = attr_value(attrs, "annotate").unwrap_or("");
    let n = self.out.creators.len();
    let thenote_rule: String;
    // (a note made before any author is the title's: revtex's `\title{…\thanks{…}}`, 1603.01892, 1802.05590,
    // cond-mat0502298 — no creator's, whatever the call's parent; the block's, as today's orphan note, OD #159)
    if n == 0 && (note || role == "thanks") {
      self.attach(
        None,
        Assoc {
          rule: "api:owner:block api:title-note".into(),
          ..assoc.clone()
        },
        note,
      );
      return;
    }
    let (owners, rule): (Vec<usize>, &str) = if n == 0 && !annotate.is_empty() {
      // (an annotation of the new authors, or of a number of them, before any creator reaches none, as today's
      // `\lx@annotate@frontmatter@now` counts its parents — revtex collaboration lists declaring every `\affiliation`
      // before the first `\author` to fix their numbering, 2103.04797, 0704.1604 —, so it is an orphan as
      // `relocate_annotations` keeps one: nothing when an author holds the same, else a sole author's, else the
      // block's (`held_before_creators`: 17 of 2103.04797's 188 are no author's and stay under the names; 2401.15779);
      // an unqualified one is the block's, 0709.0811)
      // (a note's rule says so, `api:note`: `held_before_creators` gives it to a sole author as a note)
      self.attach(
        None,
        Assoc {
          rule: if note {
            "api:no-creator owner:block api:note".into()
          } else {
            "api:no-creator owner:block".into()
          },
          ..assoc.clone()
        },
        note,
      );
      for contact in &inner {
        self.attach(
          None,
          Assoc {
            rule: "api:no-creator owner:block api:inner-contact".into(),
            ..contact.clone()
          },
          false,
        );
      }
      return;
    } else if n == 0 {
      (Vec::new(), "api:owner:block")
    } else if annotate == "all" {
      ((0..n).collect(), "api:annotate:all")
    } else if annotate == "new" || annotate.starts_with("run") {
      // (`new`: back from the last author to one holding the role, Perl Base_Utility.pool.ltxml:520-536; `run[:<cap>]`:
      // the same, and a second such contact before any new author the same run's, at most <cap> authors —
      // frontmatter_api.rs `annotate=run`, authblk's `\AB@thenote` group)
      let run_rule = annotate.starts_with("run");
      let run = if run_rule {
        self.record_run(annotate, role)
      } else {
        self.new_run(role)
      };
      if run_rule && thenote_affil {
        // (its `\AB@thenote`: when the marks show it is the label, `thenote_finish`)
        let tag = s!("thenote:{}", self.thenote.affils.len());
        self
          .thenote
          .affils
          .push((self.thenote.counter, tag.clone()));
        thenote_rule = s!("api:annotate:run {tag}");
        (run, thenote_rule.as_str())
      } else {
        (
          run,
          if run_rule {
            "api:annotate:run"
          } else {
            "api:annotate:new"
          },
        )
      }
    } else if let Ok(k) = annotate.parse::<usize>() {
      ((n.saturating_sub(k)..n).collect(), "api:annotate:n")
    } else if role == "email" && policy.email_owner == EmailOwner::SpelledElseOrder {
      // (the class's list printed apart from the names: placed once the whole list is read, `answer_sequence`)
      let group: Vec<usize> = if policy.call_is_group {
        let last = self.group[n - 1];
        (0..n).filter(|&c| self.group[c] == last).collect()
      } else {
        (0..n).collect()
      };
      self.sequence_calls += 1;
      for address in addresses_in(&text) {
        self
          .sequenced
          .push((address, assoc.clone(), group.clone(), self.sequence_calls));
      }
      return;
    } else {
      (vec![n - 1], "api:annotate:last")
    };
    let owners: Vec<Option<usize>> = if owners.is_empty() {
      vec![None]
    } else {
      owners.into_iter().map(Some).collect()
    };
    // (several addresses one e-mail call prints, written out: each to the author it spells, else the block's shared
    // creator — decision 3 of 2026-10-22, user rulings 2026-10-20 #4, #8, #10; `answer_pending` for a labelled entry)
    let reached = owners.iter().any(Option::is_some);
    let candidates = self.live();
    let mut kept_inner = Vec::new();
    for contact in inner {
      let addresses = written_out(&contact);
      if addresses.is_empty() || !reached {
        kept_inner.push(contact);
        continue;
      }
      for address in addresses {
        let (owner, tag) = match self.spelled(&address, &candidates) {
          Some(owner) => (Some(owner), "owner:spelled"),
          None => (None, "owner:block"),
        };
        self.attach(
          owner,
          Assoc {
            rule: s!("{rule} api:inner-contact ruling3:written-out {tag}"),
            text: address,
            ..contact.clone()
          },
          false,
        );
      }
    }
    for owner in owners {
      if !text.is_empty() || note {
        self.attach(
          owner,
          Assoc {
            rule: rule.to_string(),
            ..assoc.clone()
          },
          note,
        );
      }
      for contact in &kept_inner {
        self.attach(
          owner,
          Assoc {
            rule: s!("{rule} api:inner-contact"),
            ..contact.clone()
          },
          false,
        );
      }
    }
  }
}

/// The addresses an entry's e-mail contact prints when it prints several (a braced group of local parts before a
/// shared domain, `{ann,bob}@x.org`, or a list): each one written out (ruling 3); none for one address or another role.
fn written_out(contact: &Assoc) -> Vec<String> {
  if contact.role != "email" {
    return Vec::new();
  }
  let addresses = addresses_in(&contact.text);
  if addresses.len() > 1 {
    addresses
  } else {
    Vec::new()
  }
}
