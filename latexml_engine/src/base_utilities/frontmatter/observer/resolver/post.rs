//! The resolver's post-passes (OXIDIZED_DESIGN_DIVERGENCES #837), once every input is read: a sole author's
//! orphans, `labelseq=` sequences, authblk's `\AB@thenote`, the upfront contact grid, glued labels parted, a note's
//! e-mails, the pending labelled entries.

use super::*;

impl Resolver {
  /// A sole author's orphans are that author's, as Perl attaches them (OXIDIZED_DESIGN_DIVERGENCES #159, relocate.rs):
  /// amsart's `\address` or `\dedicatory` written before the `\author` (0911.5235, 1102.3253, 0712.4133); its
  /// affiliations only when it holds none (an author whose marks request one is not given the one no one marks). A note
  /// of the title (`api:title-note`) stays the block's. The sole author is the sole live one: a replacing `\author`
  /// call's author takes the block (1611.08655).
  pub(super) fn sole_author_takes_the_block(&mut self) {
    let [sole] = self.live()[..] else {
      return;
    };
    if self.out.block.is_empty() {
      return;
    }
    let affiliated = !self.out.creators[sole].affiliations.is_empty();
    let mut kept = Vec::new();
    for assoc in std::mem::take(&mut self.out.block) {
      let institute = matches!(assoc.role.as_str(), "affiliation" | "address");
      let note = matches!(
        assoc.role.as_str(),
        "thanks" | "legend" | "note" | "footnotetext"
      );
      if assoc.rule.contains("api:title-note") || (institute && affiliated) {
        kept.push(assoc);
        continue;
      }
      let rule = s!("{} owner:sole-author", assoc.rule);
      self.attach(Some(sole), Assoc { rule, ..assoc }, note);
    }
    self.out.block = kept;
  }

  /// An annotation of the new authors made before any creator is an orphan (`relocate_annotations`): nothing when an
  /// author holds one of the same role and text (2103.04797: 171 of its 188 `\affiliation`s declared ahead of the
  /// authors are given again under them), else a sole author's (an affiliation too, whatever it holds: 2401.15779's
  /// `\affiliation{https://ifisr.net}`; a note as its note, `api:note`), else the block's.
  pub(super) fn held_before_creators(&mut self) {
    let before = |a: &Assoc| a.rule.starts_with("api:no-creator");
    let held = |a: &Assoc, creators: &[&RCreator]| {
      creators.iter().any(|c| {
        c.affiliations
          .iter()
          .chain(&c.contacts)
          .chain(&c.notes)
          .any(|b| b.role == a.role && b.text.trim() == a.text.trim())
      })
    };
    let live = self.live();
    let creators: Vec<&RCreator> = live.iter().map(|&c| &self.out.creators[c]).collect();
    self
      .out
      .block
      .retain(|a| !(before(a) && held(a, &creators)));
    if let [sole] = live[..] {
      let mut kept = Vec::new();
      for assoc in std::mem::take(&mut self.out.block) {
        if before(&assoc) {
          // (a note made before the author is its note, not a contact: review revfms2a2_4 finding 5)
          let note = assoc.rule.split_whitespace().any(|t| t == "api:note");
          let rule = s!("{} owner:sole-author", assoc.rule);
          self.attach(Some(sole), Assoc { rule, ..assoc }, note);
        } else {
          kept.push(assoc);
        }
      }
      self.out.block = kept;
    }
  }

  /// The run an `annotate=new` entry of `role` reaches: back from the last author to one holding the role (Perl
  /// Base_Utility.pool.ltxml:520-536).
  pub(super) fn new_run(&self, role: &str) -> Vec<usize> {
    let mut run = Vec::new();
    for c in (0..self.out.creators.len()).rev() {
      if !run.is_empty() && self.roles[c].iter().any(|r| r == role) {
        break;
      }
      run.push(c);
    }
    run.reverse();
    run
  }

  /// A class's address list (OD #805): each address to the author it spells; else, when the list has one address per
  /// author and every spelled one is at its author's place, to the author at its place; else the group's only author,
  /// the one corresponding author, the block.
  pub(super) fn answer_sequence(&mut self) {
    let list = std::mem::take(&mut self.sequenced);
    let all = self.live();
    let spelled: Vec<Option<usize>> = list.iter().map(|(a, ..)| self.spelled(a, &all)).collect();
    // (one place per author, the places the calls': an author's two addresses in one call share it, OD #805)
    let mut places: Vec<usize> = list.iter().map(|(_, _, _, place)| *place).collect();
    places.sort_unstable();
    places.dedup();
    let place_of = |place: usize| places.iter().position(|p| *p == place);
    // (a place is among the live creators, a spelled owner an index into all of them: compared as the live creator at
    // that place — a replacing `\author` under a `SpelledElseOrder` class, its spelled first author at place 0 but index
    // 1, read the list as out of order; review revfms2a2_6 finding 1, FM S2a.2 r7)
    let in_order = places.len() == all.len()
      && list.iter().zip(&spelled).all(|((_, _, _, place), owner)| {
        owner.is_none_or(|o| place_of(*place).map(|p| all[p]) == Some(o))
      });
    for ((address, assoc, group, place), spelled) in list.into_iter().zip(spelled) {
      let (owner, rule) = match spelled {
        Some(owner) => (Some(owner), "owner:spelled"),
        None if in_order => (place_of(place).map(|p| all[p]), "owner:order"),
        None => match group
          .iter()
          .copied()
          .filter(|c| !self.replaced.contains(c))
          .collect::<Vec<_>>()
          .as_slice()
        {
          [only] => (Some(*only), "owner:only-author"),
          _ => self.corresponding(&all),
        },
      };
      self.attach(
        owner,
        Assoc {
          role: "email".into(),
          text: address,
          rule: s!("api:sequence {rule}"),
          ..assoc
        },
        false,
      );
    }
  }

  /// An `\author` call under authblk's notes, its creators from `start` on: the counter stepped at the first after an
  /// `\affil`; each unlabelled creator showing it, a labelled one showing the marks.
  pub(super) fn thenote_author(&mut self, start: usize, labelled: bool) {
    if self.thenote.newaffil {
      self.thenote.counter += 1;
      self.thenote.newaffil = false;
    }
    if labelled {
      self.thenote.labelled = true;
      return;
    }
    for c in start..self.out.creators.len() {
      self.thenote.run_of.push((c, self.thenote.counter));
    }
  }

  /// authblk's notes once every call is read: when the marks show (a label was given, or the counter passed
  /// `\c@Maxaffil`, 2 unless the document set it), each unlabelled author requests its number and each unlabelled
  /// `\affil` is labelled by its number (ruling 1: `\author[1]{Ann}\author{Bob}\affil[1]{X}\affil{Y}` prints Y under
  /// "Ann¹ Bob¹" both, `\author[a]{Ann}` under Bob alone — 2605.25017); else each stays on its run.
  pub(super) fn thenote_finish(&mut self) {
    let note = std::mem::take(&mut self.thenote);
    if note.affils.is_empty() && note.run_of.is_empty() {
      return;
    }
    let shown = note.labelled || note.counter as i64 > note.maxaffil.unwrap_or(2);
    if !shown {
      return;
    }
    for (c, number) in note.run_of {
      self.out.creators[c].requests.push(number.to_string());
      self.thenote_requests.push((c, number.to_string()));
    }
    for (number, tag) in note.affils {
      let mut moved: Option<Assoc> = None;
      for c in 0..self.out.creators.len() {
        let creator = &mut self.out.creators[c];
        let before = creator.affiliations.len();
        let mut kept = Vec::new();
        let mut removed: Vec<usize> = Vec::new();
        for (i, a) in std::mem::take(&mut creator.affiliations)
          .into_iter()
          .enumerate()
        {
          if a.rule.split_whitespace().any(|t| t == tag) {
            removed.push(i);
            moved.get_or_insert(a);
          } else {
            kept.push(a);
          }
        }
        creator.affiliations = kept;
        if removed.is_empty() || before == 0 {
          continue;
        }
        // (the attach order without them, the places after each shifted down)
        self.order[c] = self.order[c]
          .iter()
          .filter(|(aff, i)| !(*aff && removed.contains(i)))
          .map(|&(aff, i)| {
            (
              aff,
              if aff {
                i - removed.iter().filter(|r| **r < i).count()
              } else {
                i
              },
            )
          })
          .collect();
      }
      if let Some(assoc) = moved {
        let rule = s!("api:thenote ruling1:request");
        self.pending.push((vec![number.to_string()], false, Assoc {
          rule,
          label: Some(number.to_string()),
          ..assoc
        }));
      }
    }
  }

  /// Today's redistribution of a contact pile (insert.rs `distribute_upfront_contacts`, a surpass of Perl;
  /// html_feedback#46, 2308.06214; 2205.02163): when every contact went to the last of N ≥ 2 authors (amsart's
  /// `\author{A and B}` then `\address{…}\email{…}` twice), K = N·m of them, their roles periodic with period m, the
  /// j-th group of m is the j-th author's. Any other pile stays as it is.
  pub(super) fn distribute_upfront(&mut self) {
    // (the authors today's holds: a replaced `\author` call's are none of them)
    let live = self.live();
    let n = live.len();
    if n < 2 || live[..n - 1].iter().any(|&c| !self.order[c].is_empty()) {
      return;
    }
    let last_at = live[n - 1];
    // today's pile: what the last author holds — its contacts, and the entries ruling 1 labels by a leading mark that
    // today's attached to it (`piled`), in call order; one such entry on another author is no pile
    if self
      .piled
      .iter()
      .map(|(_, owners)| owners)
      .chain(self.empty_piled.iter().map(|(_, _, owners)| owners))
      .any(|owners| owners.iter().any(|&c| c != last_at))
    {
      return;
    }
    #[derive(Clone, Copy)]
    enum Item {
      Held(bool, usize),
      Piled(usize),
      Empty(usize),
    }
    let last = &self.out.creators[last_at];
    let held_assoc = |aff: bool, i: usize| {
      if aff {
        &last.affiliations[i]
      } else {
        &last.contacts[i]
      }
    };
    let mut pile: Vec<(usize, Item)> = self.order[last_at]
      .iter()
      .map(|&(aff, i)| {
        (
          held_assoc(aff, i).source.map_or(0, |s| s.0),
          Item::Held(aff, i),
        )
      })
      .collect();
    for (at, owners) in &self.piled {
      if owners.contains(&last_at) {
        pile.push((
          self.pending[*at].2.source.map_or(0, |s| s.0),
          Item::Piled(*at),
        ));
      }
    }
    for (at, (index, _, owners)) in self.empty_piled.iter().enumerate() {
      if owners.contains(&last_at) {
        pile.push((*index, Item::Empty(at)));
      }
    }
    // (a stable sort: the held ones keep their order within a call)
    pile.sort_by_key(|(index, _)| *index);
    let role_of = |item: &Item| match *item {
      Item::Held(aff, i) => held_assoc(aff, i).role.clone(),
      Item::Piled(at) => self.pending[at].2.role.clone(),
      Item::Empty(at) => self.empty_piled[at].1.clone(),
    };
    let k = pile.len();
    if k == 0 || !k.is_multiple_of(n) {
      return;
    }
    let m = k / n;
    let roles: Vec<String> = pile.iter().map(|(_, item)| role_of(item)).collect();
    if (0..k - m).any(|i| roles[i] != roles[i + m]) {
      return;
    }
    // (nor when another author is to receive one of their kind by its label: its mark requests a labelled entry of a
    // role the pile holds, which `answer_pending` gives it after this — today's gate, insert.rs `answered_later`)
    let answered_later = live[..n - 1]
      .iter()
      .map(|&c| &self.out.creators[c])
      .any(|c| {
        c.requests.iter().any(|request| {
          self
            .pending
            .iter()
            .enumerate()
            .any(|(at, (labels, _, assoc))| {
              !self.piled.iter().any(|(p, _)| *p == at)
                && roles.contains(&assoc.role)
                && labels.iter().any(|l| label_matches(request, l))
            })
        })
      });
    if answered_later {
      return;
    }
    // group j to author j, the last group staying; a piled entry moves with its group in today's, but stays ruling 1's
    let mut moved: Vec<(usize, Assoc, bool)> = Vec::new();
    let mut keep: Vec<(bool, usize)> = Vec::new();
    for (j, group) in pile.chunks(m).enumerate() {
      for (_, item) in group {
        let Item::Held(aff, i) = *item else {
          continue;
        };
        if j + 1 < n {
          let mut assoc = held_assoc(aff, i).clone();
          assoc.rule.push_str(" owner:upfront-grid");
          moved.push((j, assoc, aff));
        } else {
          keep.push((aff, i));
        }
      }
    }
    let last = &mut self.out.creators[last_at];
    let affiliations: Vec<Assoc> = keep
      .iter()
      .filter(|(aff, _)| *aff)
      .map(|&(_, i)| last.affiliations[i].clone())
      .collect();
    let contacts: Vec<Assoc> = keep
      .iter()
      .filter(|(aff, _)| !*aff)
      .map(|&(_, i)| last.contacts[i].clone())
      .collect();
    last.affiliations = affiliations;
    last.contacts = contacts;
    for (j, assoc, aff) in moved {
      if aff {
        self.out.creators[live[j]].affiliations.push(assoc);
      } else {
        self.out.creators[live[j]].contacts.push(assoc);
      }
    }
  }

  /// Glued labels parted by the marks the other side shows (ruling 1: the marks shown on a name are its requests, read
  /// as the reader reads them): a name's mark that no entry labels, but that is two or more of the entries' labels
  /// written together — `12` under entries `1` and `2` (2103.10685), `1⋆` under `1` and a symbol (2606.01911), `∗123`
  /// — requests each; an entry's label no name shows, but that is the names' marks written together (`∗∗` where the
  /// name shows `∗` beside revtex's own footnote `∗`, 1505.06831), is each of them. A label some entry or name has as
  /// written is never parted.
  pub(super) fn part_glued_labels(&mut self) {
    let bare = |l: &str| unprefixed(l).unwrap_or(l).to_string();
    let special = |l: &str| {
      ["ref:", "fuzzy:", "author:"]
        .iter()
        .any(|p| l.starts_with(p))
    };
    let entry_labels: Vec<String> = self
      .pending
      .iter()
      .flat_map(|(labels, ..)| labels.iter().filter(|l| !special(l)).map(|l| bare(l)))
      .collect();
    if entry_labels.is_empty() {
      return;
    }
    // (a number is parted into digits only when every entry's number is a digit: `12` under entries 1-4, 2103.10685 —
    // never `21` under twenty numbered entries, 1501.05343)
    let digit_entries = entry_labels
      .iter()
      .filter(|l| l.chars().all(|c| c.is_ascii_digit()))
      .all(|l| l.len() == 1);
    for c in 0..self.out.creators.len() {
      let requests = std::mem::take(&mut self.out.creators[c].requests);
      let mut parted: Vec<String> = Vec::new();
      let mut parted_any = false;
      let mut out = Vec::with_capacity(requests.len());
      for request in requests {
        let b = bare(&request);
        let prefix = &request[..request.len() - b.len()];
        if !special(&request)
          && !entry_labels.contains(&b)
          && (digit_entries || !b.chars().all(|c| c.is_ascii_digit()))
          && let Some(parts) = parted_label(&b, &entry_labels, true)
        {
          for p in parts {
            parted.push(s!("{prefix}{p}"));
            out.push(s!("{prefix}{p}"));
          }
          parted_any = true;
        } else {
          out.push(request);
        }
      }
      self.out.creators[c].requests = out;
      if parted_any && !self.out.creators[c].rule.contains("mark:parted") {
        self.out.creators[c].rule.push_str(" mark:parted");
      }
      self.parted.extend(parted.into_iter().map(|p| (c, p)));
    }
    let shown: Vec<String> = self
      .out
      .creators
      .iter()
      .flat_map(|c| c.requests.iter().filter(|r| !special(r)).map(|r| bare(r)))
      .collect();
    for (labels, _, assoc) in &mut self.pending {
      // (only a label a printed mark gave: never a class's numbering, `labelseq` — `\institute{… \and …}`'s 12th
      // entry is no glued `1` and `2`, 1005.2409 — nor a call's own `label=`)
      let printed = assoc.rule.split_whitespace().next().is_some_and(|first| {
        matches!(
          first,
          "ruling1:request" | "ruling2:symbol-line" | "api:leading-mark"
        )
      }) || assoc.rule.contains("note:legend");
      if !printed {
        continue;
      }
      let mut parted_any = false;
      let mut out = Vec::with_capacity(labels.len());
      for label in labels.drain(..) {
        let b = bare(&label);
        if !special(&label)
          && !shown.contains(&b)
          && let Some(parts) = parted_label(&b, &shown, false)
        {
          for p in parts {
            if !out.contains(&p) {
              out.push(p);
            }
          }
          parted_any = true;
        } else {
          out.push(label);
        }
      }
      *labels = out;
      if parted_any {
        assoc.rule.push_str(" ruling1:parted");
      }
    }
  }

  /// A note's links, once every name is read: each e-mail or url to the author of the note's block it spells (a url by
  /// its page, [`link_page`] — its last path segment or its host —, an e-mail link by its letters past the scheme,
  /// `link_letters`; FIDELITY_CHARTER §4: ownership by evidence — the spelling before the note's place), else, an
  /// e-mail, the author the note is on when it is the note's only link (its mark is shown on that name: the
  /// main-session decision of 2026-10-22 recorded in OXIDIZED_DESIGN_DIVERGENCES #837, narrowed by its decision 2), else
  /// none: the link stays in the note's text. A url goes to the co-author it spells as an e-mail does (main-session
  /// decision on review revfms2a2_6 finding 2: `http://www.pengxu.net/` in the note on Liang Wang is Peng Xu's,
  /// 2001.02600), and to any author only when its page spells them — a homepage, not a repository or a file under an
  /// account (main-session decision 1 on FM S2a.2 r7, review revfms2a2_7 finding 3: 2008.12970, 2511.08416, 2103.05251,
  /// 2503.02954, 2509.01952). A note its links may empty is kept, or dropped once they all became contacts (ruling 4),
  /// here — the names after it in its block read (review revfms2a2_6 finding 3).
  pub(super) fn answer_note_links(&mut self) {
    for item in std::mem::take(&mut self.note_links) {
      match item {
        NoteLink::Link { owner, assoc, fallback } => {
          self.place_note_link(owner, assoc, fallback);
        },
        NoteLink::Note {
          index,
          note,
          owner,
          rule,
          links,
          fallback,
        } => {
          let mut all_taken = true;
          for assoc in links {
            all_taken &= self.place_note_link(owner, assoc, fallback);
          }
          if all_taken {
            self.drop_emptied_note(index, &note, &rule);
          } else {
            self.keep_note(index, &note, owner, &rule);
          }
        },
      }
    }
  }

  /// One of a note's links ([`Resolver::answer_note_links`]); whether a creator took it.
  fn place_note_link(&mut self, owner: Option<usize>, assoc: Assoc, fallback: bool) -> bool {
    let candidates: Vec<usize> = match owner {
      Some(at) => (0..self.out.creators.len())
        .filter(|&c| self.group[c] == self.group[at])
        .collect(),
      None => self.live(),
    };
    let url = assoc.role == "url";
    let link = assoc
      .target
      .as_deref()
      .filter(|t| !t.trim().is_empty())
      .unwrap_or(&assoc.text)
      .trim()
      .to_string();
    // (a url linking a page, spelled by that page alone; one printing addresses is read by its letters as an e-mail is;
    // a link to an address is an e-mail, read by the address it points to, not the anchor it prints, the note's only
    // link its owner's — `\href{mailto:tsamak@clemson.edu}{tsamak}`, 2309.10007; 2008.07668, 2312.06540, 2010.15033
    // (`contact_parts`, review revfms2a2_8 finding 1, FM S2a.2 r9); `\url{brunnerj, edemaine, dylanhen,
    // wellman@mit.edu}`, 2010.09271, 1407.7882)
    // (printing addresses, not merely an `@`: `medium.com/@user` is a page — review revfms2a2_8 finding 5, FM S2a.2 r9)
    let prints_addresses = url && !addresses_in(&assoc.text).is_empty();
    let page = url && !prints_addresses;
    let letters = if page {
      link_page(&link).to_string()
    } else if url {
      link_letters(&assoc.text).to_string()
    } else {
      link
    };
    // (a url printing addresses — a group of local parts before one domain set as a link, `\url{sosk, unno, tsuboi,
    // ichi@preferred.jp}` — is no co-author's by the first spelling it holds: it goes to the name it is on only, as
    // before r7 — 1407.7882, 1810.11748, 2001.10403 went whole to one author of the group)
    let spelled = self
      .spelled(&letters, &candidates)
      .filter(|spelled| !prints_addresses || Some(*spelled) == owner);
    match spelled {
      Some(spelled) if Some(spelled) != owner => {
        let rule = s!("{} owner:spelled", assoc.rule);
        self.attach(Some(spelled), Assoc { rule, ..assoc }, false);
      },
      // (a url spelling the name it is on, not as the note's only link, says so: `owner:spelled`)
      Some(_) if url && !fallback => {
        let rule = s!("{} owner:spelled", assoc.rule);
        self.attach(owner, Assoc { rule, ..assoc }, false);
      },
      Some(_) => self.attach(owner, assoc, false),
      // (a page no name's spells is no one's, the note's only link too: `\thanks{Code: \url{https://github.com/…}}`)
      None if fallback && !page => self.attach(owner, assoc, false),
      None => return false,
    }
    true
  }

  /// The footnote texts the document made, each a note of the names showing its mark by a `\footnotemark` — the mark
  /// the PDF prints on the name and before the text (`Steven Delvaux\footnotemark[1], \quad Holger Dette\footnotemark[2]`
  /// and, after `\maketitle`, `\footnotetext[1]{Department of …}` under `\fnsymbol`: 1108.5155, 1404.2124) —, as today's
  /// `relocate_footnote` pairs them (latex_constructs/mod.rs; marks compared without their spaces, `\fnsymbol`'s `∥` and
  /// `∗` as `\@fnsymbol`'s `‖` and `*`: 2002.11721), a mark taking one text and being a mark no more: a text built
  /// after the names (made after the flush that digests their block) goes into every mark of its print not yet paired,
  /// `Seo\footnotemark[3] \and Zhang\footnotemark[3]` (1405.4582; a second `\footnotetext` printing the same `*` finds
  /// none, 2507.22611); the texts built before them (made before that flush, `\footnotetext[1]{…}` before `\maketitle`)
  /// go, joined, into the first such mark alone, the next marks finding them gone (1703.09134; 1211.5488's two
  /// `\footnotetext[1]`). A text no name's footnote mark shows is not read — by its counter value alone, today's
  /// fallback where the title's `∗` stands for the body's `1`, no mark shows: not the evidence (1404.2124's
  /// affiliations, 0909.2782) —, nor one a block read already (`Bob\footnotemark\footnotetext{…}` inside the
  /// `\author`: the note beside the name).
  pub(super) fn answer_note_texts(&mut self) {
    let folded = |text: &str| -> String {
      text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
    };
    let mark_key = |mark: &str| -> String {
      mark
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
          '\u{2217}' => '*',
          '\u{2225}' => '‖',
          c => c,
        })
        .collect()
    };
    let texts = std::mem::take(&mut self.note_texts);
    // the marks not yet paired: (creator, key), in creator order — a replaced `\author` call's none (its creators are
    // deleted with it, `resolve`: a text built before the names went to the first mark, a replaced creator's, and was
    // lost with it — `\author{A\footnotemark[1]}` twice, `\footnotetext[1]` before `\maketitle`; review revfms2a2_5
    // finding 1, 2510.26354's replacing call)
    let mut marks: Vec<(usize, String)> = self
      .fn_requests
      .iter()
      .filter(|(c, _)| !self.replaced.contains(c))
      .map(|(c, label)| (*c, mark_key(label)))
      .collect();
    marks.sort_by_key(|(c, _)| *c);
    let mut read: Vec<String> = self
      .out
      .creators
      .iter()
      .flat_map(|c| c.notes.iter().chain(&c.contacts))
      .chain(&self.out.block)
      .chain(&self.out.dropped)
      .map(|a| folded(&a.text))
      .collect();
    // (built before the names: those of one print joined, into the first mark of it)
    let mut before: Vec<(String, usize, String)> = Vec::new();
    let mut after: Vec<(String, usize, String)> = Vec::new();
    for (index, epoch, mark, text) in texts {
      let key = mark_key(&mark);
      let text = text.trim().to_string();
      if key.is_empty() || folded(&text).is_empty() || read.contains(&folded(&text)) {
        continue;
      }
      read.push(folded(&text));
      let built_before = marks
        .iter()
        .find(|(_, k)| *k == key)
        .is_some_and(|&(c, _)| {
          self
            .epoch_of
            .get(c)
            .copied()
            .flatten()
            .is_some_and(|e| epoch <= e)
        });
      if built_before {
        match before.iter_mut().find(|(k, ..)| *k == key) {
          Some((_, _, joined)) => {
            joined.push(' ');
            joined.push_str(&text);
          },
          None => before.push((key, index, text)),
        }
      } else {
        after.push((key, index, text));
      }
    }
    let mut pair = |resolver: &mut Resolver, key: &str, index: usize, text: String, first: bool| {
      let mut owners: Vec<usize> = Vec::new();
      marks.retain(|(c, k)| {
        let take = k == key && (!first || owners.is_empty()) && !owners.contains(c);
        if take {
          owners.push(*c);
        }
        !take
      });
      for owner in owners {
        // (a footnote mark the lexer parted from the mark it was glued to: ruling 1's reading, `ruling1:parted`)
        let parted = resolver
          .parted
          .iter()
          .any(|(c, label)| *c == owner && mark_key(label) == key);
        resolver.attach(
          Some(owner),
          Assoc {
            role:   "footnotetext".into(),
            text:   text.clone(),
            label:  Some(key.to_string()),
            rule:   if parted {
              "note:footnotetext-mark ruling1:parted".into()
            } else {
              "note:footnotetext-mark".into()
            },
            source: Some((index, None)),
            target: None,
          },
          true,
        );
      }
    };
    for (key, index, text) in before {
      pair(self, &key, index, text, true);
    }
    for (key, index, text) in after {
      pair(self, &key, index, text, false);
    }
  }

  /// The labelled entries, each to every creator requesting one of its labels (ruling 1; ruling 2 for a symbol), or
  /// the block's.
  pub(super) fn answer_pending(&mut self) {
    self.part_glued_labels();
    let pending = std::mem::take(&mut self.pending);
    // (the marks the names show, by kind — numbers, letters: a labelled entry of a kind no name shows is unlabelled
    // to the reader, `answer_pending`'s annotate fallback)
    let kind = |label: &str| -> Option<bool> {
      let bare = unprefixed(label).unwrap_or(label);
      if !bare.is_empty() && bare.chars().all(|c| c.is_ascii_digit()) {
        Some(true)
      } else if !bare.is_empty()
        && bare.chars().count() <= 2
        && bare.chars().all(char::is_alphabetic)
      {
        Some(false)
      } else {
        None
      }
    };
    let shown_kinds: Vec<bool> = self
      .out
      .creators
      .iter()
      .flat_map(|c| c.requests.iter().filter_map(|r| kind(r)))
      .collect();
    let live = self.live();
    for (at, (labels, symbol, assoc)) in pending.into_iter().enumerate() {
      // (a letter or number line the binding attached by its group rule (`_grouped=true`) is the group's, as today's
      // placed it: revtex's interleaved `\author{Ann$^{\mathrm a}$}\author{Bob}\affiliation{$^{\mathrm a}$Univ A}` both
      // names', `\author{Cat Cole$^2$}\affiliation{$^3$Univ B}` Cat's though no name shows 3 — 63i review r2; a symbol
      // line stays ruling 2's, 1303.7097)
      if !symbol
        && matches!(assoc.role.as_str(), "affiliation" | "address")
        && self.grouped.contains(&at)
        && let Some((_, annotated)) = self.annotated.iter().find(|(p, _)| *p == at)
        && !annotated.is_empty()
      {
        for owner in annotated.clone() {
          self.attach(
            Some(owner),
            Assoc {
              rule: s!("{} owner:group", assoc.rule),
              ..assoc.clone()
            },
            false,
          );
        }
        continue;
      }
      let mut owners: Vec<usize> = Vec::new();
      for label in &labels {
        for owner in label_owners(&self.out.creators, &live, label) {
          if !owners.contains(&owner) {
            owners.push(owner);
          }
        }
      }
      // (an entry a `\footnotemark` leads answers the same print on a name: a footnote's mark, `\footnotemark[1]` or the
      // mark a `\footnote[1]` shows — `Shun'ya Mizoguchi\footnote[1]{…}` under `\affiliation{\footnotemark[1]Theory
      // Center, …}`, 1607.07280 (FM S2a.2 r7): ruling 1, the marks shown on a name are its requests)
      // (and a footnote-symbol line's carriers are the names its symbol shows on, a note's printed mark among them
      // (ruling 2): `\author{Sudipto Bhattacharjee\footnote{…}}` prints "Bhattacharjee∗" under revtex's symbol marks,
      // the name `\affiliation{$^{\ast,\ddagger}$Department …}` labels, 1608.06509)
      let footnote_led = assoc.rule.split_whitespace().any(|t| t == "mark:footnote");
      let symbol_labelled =
        symbol || (!labels.is_empty() && labels.iter().all(|l| is_symbol_label(l)));
      // (a note's mark carries a symbol line only for a name that shows no mark of its own: there the authors' footnote
      // symbols are their labels, 1608.06509; a name showing its own marks keeps them as its requests, and its note's mark
      // is the note's — `Pramod Shukla$^\dagger$\footnote{E-mail…}` prints "Shukla†∗", the ∗ its e-mail's, not the
      // `{^\ast}` affiliation Swagata Bera shows, 2406.01694)
      if footnote_led || symbol_labelled {
        for (c, mark) in &self.note_marks {
          let carries = footnote_led || self.out.creators[*c].requests.is_empty();
          if carries && labels.contains(mark) && !owners.contains(c) {
            owners.push(*c);
          }
        }
      }
      owners.sort_unstable();
      // (a numbered affiliation answers no request a `\footnotemark` made: that number is its footnote's — `Kui
      // Wu\inst{3\footnotemark[1]}` is not Peking University's, `\institute`'s first entry, 2404.09857; a symbol line
      // is ruling 2's, every carrier of its character; an entry a `\footnotemark` leads is that footnote's print)
      if !symbol_labelled
        && !footnote_led
        && matches!(assoc.role.as_str(), "affiliation" | "address")
      {
        owners.retain(|&owner| {
          labels.iter().any(|label| {
            let asked = self.out.creators[owner]
              .requests
              .iter()
              .filter(|r| label_matches(r, label))
              .count();
            let by_footnote = self
              .fn_requests
              .iter()
              .filter(|(c, r)| *c == owner && label_matches(r, label))
              .count();
            asked > by_footnote
          })
        });
      }
      // (a `fuzzy:<name>` label — svjour's `\institute` entries after `\at`, Perl's fuzzy label of a creator, Tag
      // ltx:creator — names the creator whose name, without its marks, it spells: ruling 1)
      let (owners, assoc) = match labels.iter().find_map(|l| l.strip_prefix("fuzzy:")) {
        Some(name) if owners.is_empty() => {
          let letters = tex_letters(name);
          let key = fold_name(&letters);
          let fuzzy: Vec<usize> = self
            .out
            .creators
            .iter()
            .enumerate()
            .filter(|(_, c)| !key.is_empty() && fold_name(&c.name) == key)
            .map(|(i, _)| i)
            .collect();
          let (fuzzy, rule) = if fuzzy.is_empty() {
            (
              surname_owners(&self.out.creators, &letters),
              s!("{} ruling1:fuzzy-label fuzzy:surname", assoc.rule),
            )
          } else {
            (fuzzy, s!("{} ruling1:fuzzy-label", assoc.rule))
          };
          (fuzzy, Assoc { rule, ..assoc })
        },
        // (a label no mark requests that spells an author's whole name names that author, as a `fuzzy:` one does:
        // amsart's `\address[Martin Br\"uckerhoff]{…}` prints "(MARTIN BRÜCKERHOFF) UNIVERSITÄT MÜNSTER", its optional
        // argument the author's name, 2103.03620)
        // (the label's words, read as one: a name's words are several labels, `mark_labels`)
        None if owners.is_empty() => {
          let spelled = fold_name(&tex_letters(&labels.concat()));
          let named: Vec<usize> = self
            .out
            .creators
            .iter()
            .enumerate()
            .filter(|(_, c)| !spelled.is_empty() && fold_name(&c.name) == spelled)
            .map(|(i, _)| i)
            .collect();
          if named.is_empty() {
            (owners, assoc)
          } else {
            let rule = s!("{} ruling1:fuzzy-label", assoc.rule);
            (named, Assoc { rule, ..assoc })
          }
        },
        _ => (owners, assoc),
      };
      // (an address an entry's e-mail call printed among several, written out: to the author it spells, else the
      // block's shared creator — decision 3 of 2026-10-22, user rulings 2026-10-20 #4, #8, #10: `\institute{Univ A \\
      // \email{\{ann,support\}@x.org}}` went whole to every author of entry 1. Any author of the document, not only the
      // entry's: llncs prints its `\institute` list as one block, `\{jzhao,qfu,chu,syue\}@lincoln.ac.uk` closing entry 3
      // under entry 1's authors, 1904.07206; an entry no name requests too, `\{guyam, osherm, tomz, guykatz,
      // schapiram\}@cs.huji.ac.il` under unmarked names, 2302.05745)
      if assoc.rule.contains("ruling3:written-out") {
        let (owner, tag) = match self.spelled(&assoc.text, &self.live()) {
          Some(owner) => (Some(owner), "owner:spelled"),
          None => (None, "owner:block"),
        };
        let rule = s!("{} {tag}", assoc.rule);
        self.attach(owner, Assoc { rule, ..assoc }, false);
        continue;
      }
      let note = assoc.role == "legend";
      // (an entry labelled by a mark of a kind no name shows — `$^{1}$School of Physics, Liaoning University` under
      // names printing no number, 2407.20481, 2005.13493, 1305.3804 — is the reader's unlabelled entry: today's
      // placement by its `annotate=`, not the block)
      if owners.is_empty()
        && !symbol
        && matches!(assoc.role.as_str(), "affiliation" | "address")
        && labels
          .iter()
          .filter_map(|l| kind(l))
          .any(|k| !shown_kinds.contains(&k))
        && let Some((_, annotated)) = self.annotated.iter().find(|(p, _)| *p == at)
        && !annotated.is_empty()
      {
        for owner in annotated.clone() {
          self.attach(
            Some(owner),
            Assoc {
              rule: s!("{} label:unshown owner:annotate", assoc.rule),
              ..assoc.clone()
            },
            false,
          );
        }
        continue;
      }
      if owners.is_empty() {
        let rule = if symbol {
          "ruling2:uncarried owner:block"
        } else {
          "label:unrequested owner:block"
        };
        self.attach(
          None,
          Assoc {
            rule: s!("{} {rule}", assoc.rule),
            ..assoc
          },
          note,
        );
        continue;
      }
      for owner in owners {
        // (an owner found by the number authblk printed on it: ruling 1 on that mark)
        let by_thenote = self
          .thenote_requests
          .iter()
          .any(|(c, r)| *c == owner && labels.contains(r));
        let mut assoc = assoc.clone();
        if by_thenote && !assoc.rule.contains("ruling1:") {
          assoc.rule.push_str(" ruling1:thenote");
        }
        // (an owner found by a request its glued mark was parted into: ruling 1 on that mark)
        let by_parted = self
          .parted
          .iter()
          .any(|(c, r)| *c == owner && labels.iter().any(|l| label_matches(r, l)));
        if by_parted && !assoc.rule.contains("ruling1:parted") {
          assoc.rule.push_str(" ruling1:parted");
        }
        self.attach(Some(owner), assoc, note);
      }
    }
  }
}
