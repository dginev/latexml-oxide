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
  /// Base_Utility.pool.ltxml:520-536), or made before the last contact of the role that bounds a run.
  pub(super) fn new_run(&self, role: &str) -> Vec<usize> {
    let mut run = Vec::new();
    for c in (0..self.out.creators.len()).rev() {
      if !run.is_empty() && self.ends_a_run(c, role) {
        break;
      }
      run.push(c);
    }
    run.reverse();
    run
  }

  /// Whether creator `c` ends the run of a later unlabelled contact of `role` walking back to it: it holds the role,
  /// or it was made before the last contact of the role that bounds a run (`run_from`) — a block's contact, labelled
  /// or not, or a call a printed mark labels, which waits for its label and so gives its owner no role yet: `\author{ANN
  /// ABLE$^1$ \affil{$^1$Univ A} BOB BAKER \affil{Univ B}}` in a class printing `\affil` in place gave Ann Univ B
  /// too, as IEEEtran's `\IEEEauthorblockN{Ann Able\IEEEauthorrefmark{1}}\IEEEauthorblockA{\IEEEauthorrefmark{1}Univ
  /// X}\IEEEauthorblockN{Bob Baker and Cid Cole}\IEEEauthorblockA{Univ Y}` gave her Univ Y (review rev94a r6 B1). A
  /// call's `label=` sets no bound: authblk's `\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}` prints
  /// Univ Y under both (2605.25017). With no name made since that contact the bound is none: every author of the run
  /// is answered, and the contact is the whole run's as today's gives it (authblk's `\author{Ann$^{1}$}\author{Bob$^{2}$}
  /// \affil{$^1$Univ X}\affil{$^2$Univ Y}\affil{Univ Z}`, ptephy's `\affil{}{Univ Z}`; 77n r5 review probes r01, t02).
  pub(super) fn ends_a_run(&self, c: usize, role: &str) -> bool {
    self.roles[c].iter().any(|r| r == role)
      || self
        .run_from
        .get(role)
        .is_some_and(|from| c < *from && *from < self.out.creators.len())
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

  /// The e-mail calls a document's own note makes with a binding's meaning (`ltx:pubnote`, `Resolver::api`; one the
  /// document's own `\email` makes stays the note's text, review rev92a_r3 B1), once every name is read, by where the
  /// binding prints each (review rev92a_r2 B1):
  /// - routed (another binding's than the class's printing in place, `Policy::email_in_place`): the binding routes the
  ///   call to the current author, whose own call it is (FIDELITY_CHARTER §4, #420): the whole call, every address in
  ///   it, is the current author's (`api:annotate:last owner:own-call`, as the binding's `\email` outside a note; never
  ///   parted by order, review rev92a_r3 S2) — amsart's `\thanks{Correspondence to Ann Able, \email{carl@z.org}}`
  ///   prints "Email address: carl@z.org" under Bob's address, as today's, and `\thanks{Contact: \email{p@x.org,
  ///   q@y.org}}` both; with no live author current (REVTeX's `\preprint{Contact Ann Able, \email{…}}` before the
  ///   names, which prints the address nowhere; amsart's `\thanks{Contact: \email{p@x.org, q@y.org}}` before every
  ///   `\author`), the block's, the whole call (`owner:block`, as today's; review rev92a_r4 S1: the order step parted
  ///   it, p@x.org onto Ann Able);
  /// - in place (aa's `\offprints`, llncs's `\thanks`): FIDELITY_CHARTER §4's chain exactly (`place_line_contacts`
  ///   under [`Chain::DocumentNote`]: the author it spells, the authors' order, the only author, the corresponding
  ///   author, the block — no elimination, review rev92a_r1 B2) over the live authors, the name the note prints
  ///   before the address the corresponding step's evidence ([`Resolver::note_named`]). A&A's `\offprints{B. Baker,
  ///   \email{…}}` is Baker's (0711.1152, 0802.0707), as today's; `\offprints{O. Kochukhov, \email{oleg@…}}` O.
  ///   Kochukhov's by the name (0903.3512; 1205.5105, 1411.3255); `\offprints{the editors, \email{office@…}}` and
  ///   `\offprints{J. Smith, \email{…}}` under Adam Smith the block's.
  pub(super) fn answer_document_notes(&mut self) {
    let candidates = self.live();
    for note in std::mem::take(&mut self.document_emails) {
      let Some(source) = note.assoc.source else {
        continue;
      };
      let anchor = note.assoc.target.is_some().then_some(&note.assoc);
      let text = note
        .assoc
        .target
        .as_deref()
        .unwrap_or(&note.assoc.text)
        .to_string();
      if !note.in_place {
        // (one author's call: the current author's, or the block's whole when no live author is current — never
        // parted by order, review rev92a_r4 S1)
        let current = note.current.filter(|c| candidates.contains(c));
        self.attach(
          current,
          Assoc {
            role:   "email".into(),
            text:   note.assoc.text.clone(),
            label:  None,
            rule:   if current.is_some() {
              "api:document-note api:annotate:last owner:own-call".into()
            } else {
              "api:document-note owner:block".into()
            },
            source: Some(source),
            target: anchor.and_then(|a| a.target.clone()),
          },
          false,
        );
        continue;
      }
      let named =
        (!fold_name(&note.printed).is_empty()).then(|| self.note_named(&note.printed, &candidates));
      self.place_line_contacts(
        &text,
        "email",
        &candidates,
        source,
        anchor,
        Chain::DocumentNote { named: named.as_deref() },
      );
    }
  }

  /// The creators among `candidates` (the live ones, review rev92a_r2 S1) a document note's text before an address
  /// names ([`note_pieces`]): a piece names a creator only when it is that creator's name, every word agreeing
  /// ([`names_agree`]), and no one when it holds any other word; every creator some piece names, in order — one is the
  /// note's designation, none or several the block's (`place_line_contacts`).
  pub(super) fn note_named(&self, printed: &str, candidates: &[usize]) -> Vec<usize> {
    let names: Vec<(usize, Vec<(String, bool)>)> = candidates
      .iter()
      .map(|&c| (c, name_words(&tex_letters(&self.out.creators[c].name))))
      .collect();
    let mut named = Vec::new();
    for piece in note_pieces(&tex_letters(printed)) {
      let words = name_words(&piece);
      for (c, name) in &names {
        if names_agree(&words, name) && !named.contains(c) {
          named.push(*c);
        }
      }
    }
    named
  }

  /// The creators a name written in TeX names, as a `fuzzy:` label names them (ruling 1): the one whose name, its
  /// marks aside, it spells whole (letters folded, [`fold_name`]); else each of its names (commas, `&`, "and") the one
  /// creator whose surname is its last word ([`surname_owners`]: svjour3's `B. Wickramanayake \at …`, 2011.14024);
  /// none when a name names no one or several. (A document note's name is read by the stricter [`Resolver::note_named`],
  /// every word agreeing: a label's surname reading is today's, OD #159.)
  pub(super) fn named_creators(&self, name: &str) -> Vec<usize> {
    let letters = tex_letters(name);
    let key = fold_name(&letters);
    let whole: Vec<usize> = self
      .out
      .creators
      .iter()
      .enumerate()
      .filter(|(_, c)| !key.is_empty() && fold_name(&c.name) == key)
      .map(|(i, _)| i)
      .collect();
    if whole.is_empty() {
      surname_owners(&self.out.creators, &letters)
    } else {
      whole
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

  /// FIDELITY_CHARTER §4, "ownership by evidence, never by bare position": an e-mail one address alone fills, placed by
  /// its call's position (`api:annotate:*`, the upfront grid's included), goes to the one author its address spells — an
  /// AASTeX 5 `\email{able@x.org}` after `\author{Ann Able and Bob Baker}` (0911.3506, 1506.04035: 92 of the 688 aas
  /// census papers); a copy that author holds already is dropped; one a label placed stays. After the grid and the
  /// labels, as today's `insert::respell_emails`.
  pub(super) fn respell_emails(&mut self) {
    let live = self.live();
    if live.len() < 2 {
      return;
    }
    let mut moves: Vec<(usize, usize, usize)> = Vec::new();
    for &c in &live {
      for (i, a) in self.out.creators[c].contacts.iter().enumerate() {
        let text = a.text.trim();
        if a.role != "email"
          || a.label.is_some()
          || !a.rule.contains("api:annotate")
          || text.matches('@').count() != 1
        {
          continue;
        }
        if let [address] = addresses_in(text).as_slice()
          && let Some(owner) = self.spelled(address, &live)
          && owner != c
        {
          moves.push((c, i, owner));
        }
      }
    }
    let mut taken: Vec<(usize, Assoc)> = Vec::new();
    for &(c, i, owner) in moves.iter().rev() {
      taken.push((owner, self.out.creators[c].contacts.remove(i)));
    }
    for (owner, mut assoc) in taken.into_iter().rev() {
      let held = self.out.creators[owner]
        .contacts
        .iter()
        .any(|x| x.role == "email" && x.text.trim() == assoc.text.trim());
      if !held {
        assoc.rule.push_str(" owner:spelled");
        self.out.creators[owner].contacts.push(assoc);
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
    let mut compound = Vec::new();
    for (at, (labels, _, assoc)) in self.pending.iter_mut().enumerate() {
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
          // (a compound of distinct footnote symbols names the names carrying them all, as the line prints it beside
          // them — `$^{\dagger\ddagger}$ Project lead` under `Ann Able$^{\dagger}$` and `Bob Baker$^{\dagger\ddagger}$`
          // is Bob Baker's alone, 2602.19190; `∗∗` beside revtex's own `∗` is the one symbol, 1505.06831)
          let mut distinct = parts.clone();
          distinct.sort();
          distinct.dedup();
          if distinct.len() > 1 && distinct.iter().all(|p| is_symbol_label(p)) {
            compound.push(at);
          }
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
    self.compound.extend(compound);
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
    let mut title_notes: Vec<(String, usize, String)> = Vec::new();
    for (index, epoch, mark, text) in texts {
      let key = mark_key(&mark);
      let text = text.trim().to_string();
      if key.is_empty() || folded(&text).is_empty() || read.contains(&folded(&text)) {
        continue;
      }
      read.push(folded(&text));
      // (a text only the title's footnote mark shows is the title's note, as an entry only the title's marks request
      // is: emulateapj's `\title{…\footnotemark[1]}` with `\footnotetext[1]{Based on observations …}`, 1111.4983,
      // astro-ph0701636)
      if !marks.iter().any(|(_, k)| *k == key)
        && self.title_fn_marks.iter().any(|m| mark_key(m) == key)
      {
        title_notes.push((key, index, text));
        continue;
      }
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
    for (key, index, text) in title_notes {
      self.attach(
        None,
        Assoc {
          role: "note".into(),
          text,
          label: Some(key),
          rule: "note:footnotetext-mark api:title-note owner:title".into(),
          source: Some((index, None)),
          target: None,
        },
        true,
      );
    }
  }

  /// The labelled entries, each to every creator requesting one of its labels (ruling 1; ruling 2 for a symbol), or
  /// the block's.
  pub(super) fn answer_pending(&mut self) {
    self.part_glued_labels();
    let pending = std::mem::take(&mut self.pending);
    // (a block line's e-mail entry is one print of its addresses (`labelled_entries` pends each apart, with the entry's
    // labels and source): each address placed among the entry's carriers by FIDELITY_CHARTER §4's chain,
    // `print_address_owners` — not copied onto every carrier; the print and the address's place in it)
    let mut prints: Vec<Vec<String>> = Vec::new();
    let mut print_of: Vec<Option<(usize, usize)>> = Vec::with_capacity(pending.len());
    for (at, (labels, _, assoc)) in pending.iter().enumerate() {
      let line_email = assoc.role == "email"
        && matches!(
          assoc.rule.split_whitespace().next(),
          Some("ruling1:request" | "ruling2:symbol-line")
        );
      let previous = at
        .checked_sub(1)
        .and_then(|before| print_of[before].map(|p| (before, p.0)))
        .filter(|&(before, _)| {
          pending[before].0 == *labels && pending[before].2.source == assoc.source
        });
      match previous {
        Some((_, print)) if line_email => {
          print_of.push(Some((print, prints[print].len())));
          prints[print].push(assoc.text.clone());
        },
        _ if line_email => {
          print_of.push(Some((prints.len(), 0)));
          prints.push(vec![assoc.text.clone()]);
        },
        _ => print_of.push(None),
      }
    }
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
    let mut inner_emails: Vec<(Vec<usize>, Assoc)> = Vec::new();
    let mut shared_emails: Vec<Assoc> = Vec::new();
    let unpicked = self.share_unpicked(&live);
    // (each block line entry's `\and` column, and per label the columns of a block whose lines hold an entry of it:
    // a label with entries in two columns labels each column's own names — `Ann Able$^\dagger$ … \\ $^\dagger$ Univ A
    // \and Cat Cole$^\dagger$ \\ $^\dagger$ Univ C` gives Univ C to Cat Cole alone, as pdflatex sets each column apart;
    // an entry of a label no other column holds is every carrier's, wherever its column)
    let entry_column = |assoc: &Assoc| -> Option<(usize, usize)> {
      let (block, Some((start, _))) = assoc.source? else {
        return None;
      };
      self.entry_column.get(&(block, start)).map(|&c| (block, c))
    };
    let columns: Vec<Option<(usize, usize)>> =
      pending.iter().map(|(_, _, a)| entry_column(a)).collect();
    let label_columns = |block: usize, label: &str| -> usize {
      let mut seen: Vec<usize> = pending
        .iter()
        .zip(&columns)
        .filter(|((labels, ..), column)| {
          column.is_some_and(|(b, _)| b == block) && labels.iter().any(|l| l == label)
        })
        .filter_map(|(_, column)| column.map(|(_, c)| c))
        .collect();
      seen.sort_unstable();
      seen.dedup();
      seen.len()
    };
    let scoped: Vec<Vec<String>> = pending
      .iter()
      .zip(&columns)
      .map(|((labels, ..), column)| match column {
        Some((block, _)) => labels
          .iter()
          .filter(|l| label_columns(*block, l) > 1)
          .cloned()
          .collect(),
        None => Vec::new(),
      })
      .collect();
    for (at, (labels, symbol, assoc)) in pending.into_iter().enumerate() {
      // (a class's one institute no `\inst{n}` picks is every author's, its e-mail lines placed each by the chain:
      // `share_unpicked`, `answer_shared_emails`)
      if let Some(&(_, marked)) = self.shared.iter().find(|(p, _)| *p == at)
        && unpicked
        && !marked
      {
        if assoc.role == "email" {
          shared_emails.push(assoc);
        } else {
          for &owner in &live {
            self.attach(
              Some(owner),
              Assoc {
                rule: s!("{} api:share:authors", assoc.rule),
                ..assoc.clone()
              },
              false,
            );
          }
        }
        continue;
      }
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
          // (a label several columns answer: this entry's column's names only)
          if scoped[at].contains(label) && self.column[owner] != columns[at] {
            continue;
          }
          if !owners.contains(&owner) {
            owners.push(owner);
          }
        }
      }
      // (a `_share=authors` entry's number is not printed — llncs.cls:878-883 numbers its one institute only beside
      // another, aa.cls:888 `\ifnum\aa@nbinstitutes=1` alike —, so only the class's own `\inst{n}` requests it, never a
      // mark typed by hand: `Ann Able$^{1,2}$` under `\institute{{1} Univ A \\ {2} Univ B}` is no request of the whole,
      // the block's, as the binding declares (llncs_cls.rs `\lx@llncs@institute@sole`, aa_cls.rs
      // `\lx@aa@institute@sole`; astro-ph0305539's shape, perfect_kernel_batch63's 63i guard, its aa twin in
      // perfect_kernel_batch92c))
      if self.shared.iter().any(|(p, _)| *p == at) {
        owners.retain(|&c| {
          self
            .class_requests
            .iter()
            .any(|(o, q)| *o == c && labels.iter().any(|l| label_matches(q, l)))
        });
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
      // (a compound of footnote symbols parted: the names carrying every one of them)
      if self.compound.contains(&at) {
        owners.retain(|&c| {
          labels.iter().all(|l| {
            self.out.creators[c]
              .requests
              .iter()
              .any(|r| label_matches(r, l))
          })
        });
      }
      owners.sort_unstable();
      // (a numbered affiliation answers no request a `\footnotemark` made: that number is its footnote's — `Kui
      // Wu\inst{3\footnotemark[1]}` is not Peking University's, `\institute`'s first entry, 2404.09857; a symbol line
      // is ruling 2's, every carrier of its character; an entry a `\footnotemark` leads is that footnote's print)
      // (a creator's own role-sequence label, `author:N`, names it exactly and asks no mark of it: a rerouted setter's
      // `\address[key]{…}` the `\author[key]` gave, `frontmatter_stores.rs` `optional_contact`, 0902.1737, 2303.16316)
      if !symbol_labelled
        && !footnote_led
        && matches!(assoc.role.as_str(), "affiliation" | "address")
        && !labels.iter().all(|label| label.starts_with("author:"))
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
          let key = fold_name(&tex_letters(name));
          let fuzzy = self.named_creators(name);
          // (named by surname when the label spells no name whole)
          let whole = fuzzy
            .iter()
            .any(|&c| !key.is_empty() && fold_name(&self.out.creators[c].name) == key);
          let rule = if whole {
            s!("{} ruling1:fuzzy-label", assoc.rule)
          } else {
            s!("{} ruling1:fuzzy-label fuzzy:surname", assoc.rule)
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
      // (an entry's e-mail call printing one address is not its label's (the gate study's G3): placed once every
      // entry is read, by FIDELITY_CHARTER §4's chain — `answer_inner_emails`; an entry no live name carries keeps the
      // unrequested entry's placement below unless the address spells an author)
      if is_inner_email(&assoc)
        && (owners.iter().any(|c| live.contains(c)) || self.spelled(&assoc.text, &live).is_some())
      {
        inner_emails.push((owners, assoc));
        continue;
      }
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
      // (an entry only the title's marks request is the title's note, the document's, as today's keeps it in the title:
      // aastex's `\title{…\altaffilmark{1}}` with `\altaffiltext{1}{Based on observations …}`, 0801.1670, 1006.1591)
      if owners.is_empty()
        && labels
          .iter()
          .any(|l| self.title_requests.iter().any(|r| label_matches(r, l)))
      {
        self.attach(
          None,
          Assoc {
            role: "note".into(),
            rule: s!("{} api:title-note owner:title", assoc.rule),
            ..assoc
          },
          true,
        );
        continue;
      }
      // (a contact call — an `\email`, a URL, an ORCID — led by a footnote symbol no name carries is outside ruling 2,
      // which labels affiliation and legend lines (user ruling 2026-10-21 #2): it stays by its `annotate=`, as today's —
      // amsart's `\email{$^\dagger$ ann@a.edu}` after Ann's `\author` is hers, not a nameless creator's)
      // (its placement the one today's gave it, declared or the default: `piled`, api.rs)
      if owners.is_empty()
        && symbol_labelled
        && matches!(assoc.role.as_str(), "email" | "url" | "orcid")
        && let Some((_, placed)) = self.piled.iter().find(|(p, _)| *p == at)
        && !placed.is_empty()
      {
        for owner in placed.clone() {
          self.attach(
            Some(owner),
            Assoc {
              rule: s!("{} ruling2:uncarried owner:annotate", assoc.rule),
              ..assoc.clone()
            },
            false,
          );
        }
        continue;
      }
      // (a footnote-symbol line under names that show no symbol mark at all is a mark of a kind no name shows too:
      // ruling 2 gives the line to the names carrying its symbol, and none does, so it keeps its call's placement, as
      // an unshown number does below — 1610.06222's `\address{$^*$Also affiliated with …}`, `\address{$\dag$Also
      // affiliated with FAMNIT …}` under four unmarked names stayed Verret's, not a nameless creator's)
      if owners.is_empty()
        && symbol_labelled
        && matches!(assoc.role.as_str(), "affiliation" | "address")
        && !live.iter().any(|&c| {
          self.out.creators[c]
            .requests
            .iter()
            .any(|r| is_symbol_label(r))
        })
        && let Some((_, placed)) = self.piled.iter().find(|(p, _)| *p == at)
        && !placed.is_empty()
      {
        for owner in placed.clone() {
          self.attach(
            Some(owner),
            Assoc {
              rule: s!(
                "{} ruling2:uncarried label:unshown owner:annotate",
                assoc.rule
              ),
              ..assoc.clone()
            },
            false,
          );
        }
        continue;
      }
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
      // (an address no name requests that opens with one author's whole name is that author's, as the page reads it:
      // `$^1$Lior Alon, School of Mathematics, …` under unmarked names, 2106.06096 — today's output gives it them too)
      if owners.is_empty()
        && !symbol
        && matches!(assoc.role.as_str(), "affiliation" | "address")
        && let Some(head) = assoc.text.split(',').next().map(fold_name)
        && let [owner] = live
          .iter()
          .copied()
          .filter(|&c| !head.is_empty() && fold_name(&self.out.creators[c].name) == head)
          .collect::<Vec<_>>()
          .as_slice()
      {
        let rule = s!("{} label:unrequested owner:named", assoc.rule);
        self.attach(Some(*owner), Assoc { rule, ..assoc }, false);
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
      // (one of a block line's addresses: the chain's owner over every author and its entry's carriers)
      let (owners, assoc) = match print_of[at] {
        Some((print, place)) => {
          match self.print_address_owners(&prints[print], place, &owners, &live) {
            (_, None) => (owners, assoc),
            (narrowed, Some(tag)) => {
              let rule = s!("{} {tag}", assoc.rule);
              (narrowed, Assoc { rule, ..assoc })
            },
          }
        },
        _ => (owners, assoc),
      };
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
        // (an owner found by the `\ref` key its mark prints, among a numbered entry's labels: ruling 1 on that mark, as
        // an entry its keys alone label is (`api:ref-label`) — `\inst{\ref{inst1},\ref{inst2}}` under `\institute{… \\
        // \email{…} \label{inst1} \and …}`, whose entry 1 today's left on a nameless creator, 1302.1442, 1707.06238;
        // not the entry's inner e-mail, which its every carrier holding is no reading of the page: the gate study's G3)
        let by_ref = self.out.creators[owner]
          .requests
          .iter()
          .any(|r| r.starts_with("ref:") && labels.contains(r));
        if by_ref
          && !assoc.rule.contains("api:ref-label")
          && !assoc.rule.contains("api:inner-contact")
        {
          assoc.rule.push_str(" api:ref-label");
        }
        // (an owner a numbered entry (`labelseq=`) finds by a mark printed by hand — `$^{1}$`, `\textsuperscript{1}`, a
        // paper's mark macro, read off the digested list (`observer::marks`) —, not by the class's own request
        // (`\inst{1}`, `class_requests`): ruling 1, the marks shown on a name are its requests — aa's `Ann Able$^{1}$`
        // under `\institute{Univ X \and Univ Y}`, whose numbers the class prints before each entry (aa.cls v6.1
        // :319-341), which today's left on a nameless creator, the marks in the names: 1009.6047, 0904.2180 (aa class C
        // of the stage-6 gate study, 13 papers), 2207.06953 (llncs class H))
        // (not an entry holding marks of its own no split read: its number is no reading of them — 1507.05963's
        // institute before its line-led split read a contacts line)
        let by_hand = assoc.rule.split_whitespace().next() == Some("api:labelseq")
          && !self.marked_entries.contains(&at)
          && self.out.creators[owner].requests.iter().any(|r| {
            labels.iter().any(|l| label_matches(r, l))
              && !self
                .class_requests
                .iter()
                .any(|(c, q)| *c == owner && q == r)
              && !self.fn_requests.iter().any(|(c, q)| *c == owner && q == r)
          });
        // (a `\ref` key's owner says so already, `api:ref-label`)
        if by_hand && !assoc.rule.contains("ruling1:") && !assoc.rule.contains("api:ref-label") {
          assoc.rule.push_str(" ruling1:request");
        }
        // (an API entry its call labels — aastex's `\altaffiltext{5}{…}` — to the names whose marks request it: ruling 1,
        // user 2026-10-21 #1, as a labelled line inside an author block is; 0707.2093's `Woods\altaffilmark{5,6}`, whose
        // two entries today's gave the last author)
        // (not the kernel's own `author:N` label, the N-th author's: no mark asked for it)
        if assoc.rule.split_whitespace().next() == Some("api:label")
          && !assoc.rule.contains("ruling1:")
          && labels.iter().any(|l| !l.starts_with("author:"))
        {
          assoc.rule.push_str(" ruling1:request");
        }
        self.attach(Some(owner), assoc, note);
      }
    }
    let mut placed = self.answer_inner_emails(inner_emails);
    placed.extend(self.answer_shared_emails(shared_emails));
    self.place_emails_left(placed);
  }

  /// The owners of the address at `place` of a block line's e-mail entry (`addresses`, the one print; `carriers`, the
  /// names requesting its label), by FIDELITY_CHARTER §4's chain as an unmarked line's (`place_line_contacts`): the
  /// author it spells (`owner:spelled`) — spelled against every live author (`live`), so an address spelling a name
  /// that does not carry the mark is that name's (`Ann Able$^\dagger$, Bob Baker, Cat Cole$^\dagger$ …
  /// $^\dagger$\texttt{bob@x.org, cat@x.org}`: bob@ Bob Baker's, review rev93a r2 c10, d1-d4; one carrier alike, c12),
  /// and against the carriers when the authors tie; else, the print one address per carrier that every spelled address
  /// agrees with, the carrier at its place (`owner:order`; an address spelling a non-carrier agrees with no place);
  /// else every carrier's, as the symbol-line ruling gives an unspelled address on the line (FIDELITY_CHARTER §4,
  /// 2026-10-30, "every symbol carrier": a spelled address beside it does not send it to the block; no tag, the label's
  /// reading). One carrier: only an address spelling another author leaves it. The label's carriers stay the
  /// candidates of the order: `$^\dagger$\texttt{\{ann, cat\}@x.org}` under Ann Able† and Cat Cole† is ann@ Ann Able's
  /// and cat@ Cat Cole's, not both each (review rev93a B1: 2409.11430's ‡ and § groups, 2410.18711's `$^{*}$Email:
  /// jtliu@…; wuxin\_1134@…; wuzhenhua@…`, 2506.19548's `\{devesh, minesh\}@…`, 2510.10618's `\texttt{dawnkun1993@…,
  /// chenma@…}`, the second in author order).
  fn print_address_owners(
    &self,
    addresses: &[String],
    place: usize,
    carriers: &[usize],
    live: &[usize],
  ) -> (Vec<usize>, Option<&'static str>) {
    let spelled: Vec<Option<usize>> = addresses
      .iter()
      .map(|a| self.spelled(a, live).or_else(|| self.spelled(a, carriers)))
      .collect();
    match spelled[place] {
      Some(owner) if carriers.len() > 1 || !carriers.contains(&owner) => {
        return (vec![owner], Some("owner:spelled"));
      },
      _ if carriers.len() < 2 => return (carriers.to_vec(), None),
      _ => {},
    }
    let in_order = addresses.len() == carriers.len()
      && !lists_local_parts(&addresses[place])
      && spelled
        .iter()
        .zip(carriers)
        .all(|(s, c)| s.is_none_or(|s| s == *c));
    if in_order {
      return (vec![carriers[place]], Some("owner:order"));
    }
    (carriers.to_vec(), None)
  }

  /// Whether no mark picks a `_share=authors` entry (llncs.cls:868-886: one institute, no `\and` in it, printed
  /// unnumbered under the names, every author's — OD #901, today's `share_row_contacts`/`marks_pick`): no live name
  /// shows the class's own request of a number (`\inst{1}`, `\inst{2}`: [`Resolver::class_requests`]; `\inst{*}`, an
  /// empty `\inst{}`, a hand-typed `$^{1}$` and a paper's mark macro pick nothing: 2207.04574, 2604.20229, 2607.02596,
  /// 1804.05870), and no name shows a number other than 1 by hand (`Ann Able$^{1,2}$`: the one institute is several,
  /// numbered by hand). An entry holding a mark of its own is picked too (`answer_pending`, 2608.21425).
  fn share_unpicked(&self, live: &[usize]) -> bool {
    let number = |label: &str| {
      let bare = unprefixed(label).unwrap_or(label);
      !bare.is_empty() && bare.bytes().all(|b| b.is_ascii_digit())
    };
    !live.iter().any(|&c| {
      self
        .class_requests
        .iter()
        .any(|(o, l)| *o == c && number(l))
        || self.out.creators[c].requests.iter().any(|r| {
          number(r)
            && unprefixed(r).unwrap_or(r) != "1"
            && !self.class_requests.iter().any(|(o, l)| *o == c && l == r)
        })
    })
  }

  /// The e-mail calls of a `_share=authors` entry no mark picks (`share_unpicked`: llncs's one `\institute{Univ X \\
  /// \email{…}}`), every address placed by FIDELITY_CHARTER §4's chain over the live authors, as today's places a
  /// shared row's lines (`shared_row::row_email_calls`, OD #901): the author it spells; else, the entry's addresses one
  /// per author that every spelled one agrees with, the author at its place (`owner:order`); else, one author, that
  /// author; else the block (a template's `{firstname.lastname}@x.edu`, `owner:pattern`) — the one address left of
  /// several authors the corresponding author's ([`Resolver::place_emails_left`]). No elimination.
  /// Witnesses 1912.11606, 2302.05745 (llncs class S of the stage-6 gate study).
  fn answer_shared_emails(&mut self, emails: Vec<Assoc>) -> Vec<EmailPlacement> {
    let live = self.live();
    let flat: Vec<(Assoc, String)> = emails
      .into_iter()
      .flat_map(|assoc| {
        let addresses = addresses_in(&assoc.text);
        if addresses.is_empty() {
          vec![(assoc.clone(), assoc.text.clone())]
        } else {
          addresses
            .into_iter()
            .map(|address| (assoc.clone(), address))
            .collect()
        }
      })
      .collect();
    let spelled: Vec<Option<usize>> = flat
      .iter()
      .map(|(_, address)| self.spelled(address, &live))
      .collect();
    let in_order = flat.len() == live.len()
      && live.len() >= 2
      && flat.iter().all(|(_, address)| !lists_local_parts(address))
      && spelled
        .iter()
        .zip(&live)
        .all(|(s, c)| s.is_none_or(|s| s == *c));
    flat
      .into_iter()
      .zip(spelled)
      .enumerate()
      .map(|(at, ((assoc, address), spelled))| {
        let (owner, tag) = match spelled {
          Some(owner) => (Some(owner), "owner:spelled"),
          None if in_order => (Some(live[at]), "owner:order"),
          None => match live.as_slice() {
            [only] => (Some(*only), "owner:only-author"),
            _ if is_pattern_address(&address) => (None, "owner:pattern"),
            _ => (None, "owner:block"),
          },
        };
        let rule = s!("{} api:share:authors", assoc.rule);
        EmailPlacement {
          assoc: Assoc { rule, text: address, ..assoc },
          owner,
          tag,
          candidates: live.clone(),
        }
      })
      .collect()
  }

  /// The e-mail calls inside labelled entries that print one address (an `\email` inside an `\institute` entry, the
  /// gate study's G3: llncs's `\institute{Univ X\\ \email{ann.able@x.org} \and Univ Y\\ \email{info@y.org}}`), each with
  /// the entry's carriers — the names requesting its label. The address is not the label's: llncs prints its
  /// `\institute` list as one block under the names, the address once, under its institute (1606.03774, 2204.03161;
  /// aa's 0901.4618, 1401.5494: `nuccio.lanza@`, `lfossati@`), and today's copies it onto every carrier (Perl too). So
  /// FIDELITY_CHARTER §4's chain, exactly, never bare position and no elimination: the author it spells, any live
  /// author of the document (an entry's address may spell an author of another entry, as `ruling3:written-out` reads
  /// one); else the entry's only carrier; else the block, once — a group's shared address (`info@y.org` under an
  /// institute two names carry, spelling neither), a template (`\{first.last\}@x.org`, `owner:pattern`: 2408.14427) —,
  /// the one address left of several, of the entry's carriers the one marked the corresponding author
  /// ([`Resolver::place_emails_left`]: two addresses left are the block's, though one author carries both entries and
  /// is marked so). (An entry no live name carries, its address spelling no one, stays the unrequested entry's:
  /// `answer_pending`.) Each rule says so (`owner:inner-email`, a change tag: the writer moves it).
  fn answer_inner_emails(&mut self, inner_emails: Vec<(Vec<usize>, Assoc)>) -> Vec<EmailPlacement> {
    let live = self.live();
    inner_emails
      .into_iter()
      .map(|(owners, assoc)| {
        let carriers: Vec<usize> = owners.into_iter().filter(|c| live.contains(c)).collect();
        let (owner, tag) = match (self.spelled(&assoc.text, &live), carriers.as_slice()) {
          (Some(owner), _) => (Some(owner), "owner:spelled"),
          (None, [only]) => (Some(*only), "owner:only-carrier"),
          (None, _) if is_pattern_address(&assoc.text) => (None, "owner:pattern"),
          (None, _) => (None, "owner:block"),
        };
        let rule = s!("{} owner:inner-email", assoc.rule);
        EmailPlacement {
          assoc: Assoc { rule, ..assoc },
          owner,
          tag,
          candidates: carriers,
        }
      })
      .collect()
  }

  /// FIDELITY_CHARTER §4's corresponding step, once over the e-mail addresses the post-pass placed (the entries'
  /// one-address calls, [`Resolver::answer_inner_emails`]; a shared entry's lines, [`Resolver::answer_shared_emails`]),
  /// then each attached by its owner: the ONE address left on the block of a multi-author paper — no other of them left,
  /// none on the block already — goes to the one author of its candidates marked the corresponding author (a
  /// "Corresponding author" note: [`Resolver::corresponding`]) who holds no other address; a template
  /// (`owner:pattern`) and a list of local parts the reader could not part (`(npalmer|pwg)@…`, `lists_local_parts`)
  /// stay the block's, every author's address and no one's own — and count as left on it: a template under Univ X and
  /// `office@y.org` under Univ Y are two left, each the block's (review rev92b_r2 S1: `office@y.org` went to the
  /// corresponding author). Two or more left are each the block's, once (`info@x.org` and `office@y.org` under the two
  /// institutes a corresponding Ann Able and Bob Baker both carry: not both Ann Able's).
  fn place_emails_left(&mut self, mut placed: Vec<EmailPlacement>) {
    let left: Vec<usize> = (0..placed.len())
      .filter(|&at| placed[at].owner.is_none())
      .collect();
    let on_block = self.out.block.iter().any(|assoc| assoc.role == "email");
    if let [at] = left.as_slice()
      && placed[*at].tag == "owner:block"
      && !on_block
      && self.live().len() > 1
      && !lists_local_parts(&placed[*at].assoc.text)
      && let (Some(author), "owner:corresponding") = self.corresponding(&placed[*at].candidates)
      && placed.iter().all(|p| p.owner != Some(author))
      && !self.out.creators[author]
        .contacts
        .iter()
        .any(|assoc| assoc.role == "email")
    {
      placed[*at].owner = Some(author);
      placed[*at].tag = "owner:corresponding";
    }
    for EmailPlacement { assoc, owner, tag, .. } in placed {
      let rule = s!("{} {tag}", assoc.rule);
      self.attach(owner, Assoc { rule, ..assoc }, false);
    }
  }
}

/// An e-mail address the post-pass placed ([`Resolver::place_emails_left`]): its contact (the rule its chain wrote
/// before the owner's tag), its owner by the chain so far (`None` the block), the owner's tag, and the authors the
/// corresponding step may pick among (an entry's carriers, a shared entry's live authors).
struct EmailPlacement {
  assoc:      Assoc,
  owner:      Option<usize>,
  tag:        &'static str,
  candidates: Vec<usize>,
}

/// Whether a pending entry is an e-mail call an entry made that prints one address (`api:inner-contact`; one printing
/// several is written out, `ruling3:written-out`).
fn is_inner_email(assoc: &Assoc) -> bool {
  assoc.role == "email"
    && assoc
      .rule
      .split_whitespace()
      .any(|tag| tag == "api:inner-contact")
    && !assoc.rule.contains("ruling3:written-out")
}
