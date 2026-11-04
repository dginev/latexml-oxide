//! Relocating author annotations to their creators; shown marks and superscript scripts.

use super::*;

/// Find all dummy frontmatter entries (role "pending") containing unattached annotations
/// and attempt to attach to the appropriate frontmatter, based on the identifying labels.
/// Perl: relocateAnnotations($document).
pub(in crate::base_utilities) fn relocate_annotations(document: &mut Document) -> Result<()> {
  // Find dummy frontmatter elements containing not-yet-attached annotations
  let pending_nodes = document.findnodes(".//*[@role='pending']", None);
  if pending_nodes.is_empty() {
    return Ok(());
  }
  // Collect the frontmatter that have attachment labels
  let mut labeltable: HashMap<String, Vec<Node>> = HashMap::default();
  // fallback: Same, but without prefix
  let mut unlabeltable: HashMap<String, Vec<Node>> = HashMap::default();
  for target in document.findnodes(".//*[@_annotations]", None) {
    let target_role = target.get_attribute("role").unwrap_or_default();
    if target_role == "pending" {
      continue;
    }
    // Dedup labels PER TARGET: a creator that cites the same annotation label
    // from several of its authors (LLNCS `\author{A\inst{1} B\inst{2,1}}` — both
    // A and B cite institution 1) must receive that affiliation ONCE, not once
    // per citing author. Each duplicate label would otherwise push `target`
    // again, and the pending note gets cloned into it per push. Witness arXiv
    // 2603.23669: two-author creators rendered "Mila … Mila … McGill … McGill".
    let annotations = target.get_attribute("_annotations").unwrap_or_default();
    let mut seen: HashSet<&str> = HashSet::default();
    for label in annotations.split(',') {
      if label.is_empty() || !seen.insert(label) {
        continue;
      }
      labeltable
        .entry(label.to_string())
        .or_default()
        .push(target.clone());
      // Misuse of labelling macros can lead to a prefix mismatch, so fall back to
      // a prefix-stripped ("noprefix") index — e.g. an affiliation labelled
      // `institute:1` can still find an author who cited `affiliation:1`.
      // BUT a creator's OWN role-sequence label (`author:N` / `editor:N` /
      // `translator:N`, auto-assigned in `\lx@add@frontmatter@now`) must NEVER
      // enter that fallback: it would let a SHARED affiliation's `affiliation:1`
      // bind to the first author's `author:1` purely by number, stranding one
      // shared `\institute` on author 1 (Perl does exactly this — arXiv:2402.19043
      // / WDM, 5 authors + one `\institute`, no `\inst`; OXIDIZED_DESIGN #159). A
      // genuine per-author affiliation instead matches EXACTLY via `labeltable`
      // (the `affiliation:1` that `\inst{1}` requests), which is untouched here.
      if let Some(pos) = label.find(':')
        && label[..pos] != target_role
      {
        unlabeltable
          .entry(label[pos + 1..].to_string())
          .or_default()
          .push(target.clone());
      }
    }
  }
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  // the marks each author's name shows (`$^{a,b}$`: two), read where an institute labelled by a mark asks (below)
  let mut author_marks: Option<Vec<Vec<String>>> = None;
  // Only a label a mark set is answered by the authors showing that mark, as the mark's own keys (`_bymark`, read as the
  // authors' are): a number `labelseq` gave is no mark (aa's one `\institute{{1} Univ A \\ {2} Univ B …}` is one
  // institute, `affiliation:1`, under authors marked 1,2 / 2,3: astro-ph0305539).
  let by_mark: HashMap<String, Vec<String>> = pending_nodes
    .iter()
    .filter_map(|pending| {
      let keys = pending.get_attribute("_bymark")?;
      let label = pending.get_attribute("_annotations")?;
      Some((label, keys.split(',').map(str::to_string).collect()))
    })
    .collect();
  // A label's targets: the exact index, then the prefix-stripped fallbacks for a misused prefix — creators only, the
  // annotations a prefix is misused on: a title's `tnote:t1` must not take an author's orphaned `fn:t1` (63h review,
  // `elsarticle_orphan_fntext_stays_off_the_title`).
  let indexed_targets = |label: &str| -> Option<Vec<Node>> {
    if let Some(targets) = labeltable.get(label) {
      return Some(targets.clone());
    }
    let noprefix = label
      .find(':')
      .map(|pos| &label[pos + 1..])
      .unwrap_or(label);
    [
      unlabeltable.get(label),
      labeltable.get(noprefix),
      unlabeltable.get(noprefix),
    ]
    .into_iter()
    .flatten()
    .map(|targets| {
      targets
        .iter()
        .filter(|target| target.get_name() == "creator")
        .cloned()
        .collect::<Vec<_>>()
    })
    .find(|targets| !targets.is_empty())
  };
  // A superscript mark and a bracketed label printed as the same superscript are one label (authblk sets
  // `\author[1]{Bob}` and `\affil[1]{Univ X}` as a `\textsuperscript{1}`, `\AB@authnote`/`\AB@affilnote`,
  // authblk.sty:98-99, :142, :162): the label a mark set (`\affil{$^1$Univ X}`, `affiliation:1` for the authors
  // showing that mark) also reaches the authors whose label is its mark (`\author[1]{Bob}`, `1`), and a bracketed
  // label (`\affil[1]{Univ X}`, `1`) the authors requesting it by a mark (`\author{Bob$^{1}$}`, whose answered mark
  // the name no longer shows; `\inst{1}`: `affiliation:1`): Bob lost Univ X to the author whose spelling matched
  // (77n r6 review probes f2, f7). Only authblk's names and `\inst` request `affiliation:N` from a printed label, so
  // a class that renumbers its labels (elsarticle prints `\author[1]` as "a") is untouched (r7 review: elsarticle,
  // cas, llncs, revtex, aastex, IEEE probes and 12 elsarticle/cas papers unchanged).
  let find_targets = |label: &str| -> Option<Vec<Node>> {
    let mut targets = indexed_targets(label).unwrap_or_default();
    let respelt: Vec<Node> = if let Some(keys) = by_mark.get(label) {
      keys
        .iter()
        .filter_map(|key| labeltable.get(key.as_str()))
        .flatten()
        .filter(|target| target.get_name() == "creator")
        .cloned()
        .collect()
    } else if !label.is_empty() && !label.contains(':') {
      labeltable
        .get(&s!("affiliation:{label}"))
        .into_iter()
        .flatten()
        .filter(|target| target.get_name() == "creator")
        .cloned()
        .collect()
    } else {
      Vec::new()
    };
    for target in respelt {
      if !targets.contains(&target) {
        targets.push(target);
      }
    }
    (!targets.is_empty()).then_some(targets)
  };
  // An orphan no one cites is kept all the same, as LaTeX prints it. An institute-level contact, or a thanks note
  // (svjour3's `\thankstext` without its `\thanksref`; an EPJ institute's own note, 63g review, repro
  // svjour3_epj_title_and_institute_notes), is shared as a matter of course. A person's own note (elsarticle's
  // `\fntext` without its `\fnref`) lost the link to its person, which is warned about; one placed on the block (the
  // `addresses:block` label, the shared creator below) lost none: the evidence gave it to the block (ijcai's `\emails
  // {$^{*}$Corresponding author}` under two starred names, 2609.15094; OXIDIZED_DESIGN_DIVERGENCES #908).
  let shared_orphan = |note: &Node| {
    let role = note.get_attribute("role").unwrap_or_default();
    is_shared_contact_role(&role)
      || role == "thanks"
      || note.get_attribute("_label").as_deref() == Some("addresses:block")
  };
  // Beyond-Perl (OXIDIZED_DESIGN #159): an orphan's label is one institute with what inherited its label (its
  // `\email`/`\url`), and it goes to the author the evidence names, else is shared:
  // - an institute labelled by a mark (`\affiliation{$^{a}$Univ A}`, an `affiliation:a` label) is the authors' whose
  //   names show that mark (`Ann Able$^{a}$`, a visible `<ltx:sup>` the author line kept), as the reader pairs them
  //   (revtex 2011.01984, 2301.08449; repro revtex_marked_affiliations_link_by_mark); the rest are then not paired by
  //   position;
  // - an institute's `\at` names (svjour3 `fuzzy:` label) that missed the exact name match are the authors with those
  //   surnames, when each names one (`A.M.Bykov \at …` for "Andrei Bykov", 1205.2208; `Olivier Augereau, Koichi Kise,
  //   and Motoi Iwata \at …`, 1811.03214, every one of them), in any order; unless an unlabelled
  //   piece sits among them, svjour3's `\institute{A \and B \at X}` where a name-only piece shares the next institute
  //   (1709.00485, 2003.07295; repro svjour3_name_only_piece_shares_the_next_institute);
  // - numbered institutes (`affiliation:N`, no `\inst`), as many as the authors, are the i-th author's: Perl's numeric
  //   fallback and the grid `distribute_upfront_contacts` pairs (`\author{A \and B}` + `\institute{X \and Y}`);
  // - a sole author's orphans are that author's, as Perl attaches them; its institutes only where no institute is
  //   linked otherwise (an author marked for one institute is not given the one no one marks,
  //   `author_block_orphan_mark_is_kept`).
  // Any other orphan is shared, gathered into ONE trailing name-LESS `<ltx:creator role="author">` below the whole
  // author row, where Perl warns and drops it, or strands one shared `\institute` on author 1 (arXiv:2402.19043, WDM: 5
  // authors + one `\institute`). The ar5iv theme breaks a last-position shared affiliation out to full width. We reuse
  // the first such pending stub in place: it is already a `<creator>` right after the last real author.
  let mut orphan_labels: Vec<(String, bool)> = Vec::new();
  let mut linked_institute = false;
  for pending in &pending_nodes {
    for note in element_nodes(pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      let institute = is_institute_role(&note.get_attribute("role").unwrap_or_default());
      if label.is_empty() {
        continue;
      } else if find_targets(&label).is_some() {
        linked_institute |= institute;
      } else if let Some(entry) = orphan_labels.iter_mut().find(|(known, _)| *known == label) {
        entry.1 |= institute;
      } else {
        orphan_labels.push((label, institute));
      }
    }
  }
  let mut owners: HashMap<String, Vec<Node>> = HashMap::default();
  for (label, institute) in orphan_labels.iter() {
    let Some(keys) = by_mark.get(label) else {
      continue;
    };
    if !*institute {
      continue;
    }
    let author_marks = author_marks.get_or_insert_with(|| {
      authors
        .iter()
        .map(|author| shown_marks(document, author))
        .collect()
    });
    let marked: Vec<Node> = (0..authors.len())
      .filter(|&i| keys.iter().any(|key| author_marks[i].contains(key)))
      .map(|i| authors[i].clone())
      .collect();
    if !marked.is_empty() {
      owners.insert(label.clone(), marked);
    }
  }
  let linked_institute = linked_institute || !owners.is_empty();
  let unaffiliated = !linked_institute
    && authors.iter().all(|author| {
      element_nodes(author).iter().all(|child| {
        child.get_name() != "contact"
          || !is_institute_role(&child.get_attribute("role").unwrap_or_default())
      })
    });
  let institutes: Vec<&String> = orphan_labels
    .iter()
    .filter(|(label, institute)| *institute && !owners.contains_key(label))
    .map(|(label, _)| label)
    .collect();
  let numbered = |label: &str| {
    label
      .strip_prefix("affiliation:")
      .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
  };
  if !institutes.iter().any(|label| numbered(label)) {
    let surnames: Vec<Option<String>> = authors
      .iter()
      .map(|author| {
        document
          .findnode("ltx:personname", Some(author))
          .and_then(|person| surname_key(&person.get_content()))
      })
      .collect();
    // (any fuzzy orphan, an institute's or a contact's: `\correspondingauthor{Weipeng Lin, Lin Tang}`'s correspondent is
    // both authors', 2408.02230, 2404.14129, 2410.19543, 2008.13733 — it had been orphaned with a Warning)
    let fuzzy_orphans: Vec<&String> = orphan_labels
      .iter()
      .filter(|(label, _)| label.starts_with("fuzzy:") && !owners.contains_key(label))
      .map(|(label, _)| label)
      .collect();
    for label in &fuzzy_orphans {
      let Some(names) = label.strip_prefix("fuzzy:") else {
        continue;
      };
      // each name the piece gives before its `\at` (a list: commas, " and ", " \& "; spaces are `_` in a label)
      let mut named: Vec<Node> = Vec::new();
      let each_named_once = names
        .split([',', '&'])
        .flat_map(|part| part.split("_and_"))
        .map(|name| name.trim_matches('_').trim_start_matches("and_"))
        .filter(|name| !name.is_empty())
        .all(|name| {
          let key = surname_key(name);
          let matches: Vec<usize> = (0..authors.len())
            .filter(|&i| key.is_some() && surnames[i] == key)
            .collect();
          if let [i] = matches.as_slice() {
            if !named.contains(&authors[*i]) {
              named.push(authors[*i].clone());
            }
            true
          } else {
            false
          }
        });
      if each_named_once && !named.is_empty() {
        owners.insert((*label).clone(), named);
      }
    }
  } else if unaffiliated
    && authors.len() > 1
    && institutes.len() == authors.len()
    && institutes.iter().all(|label| numbered(label))
  {
    owners.extend(
      institutes
        .iter()
        .map(|label| (*label).clone())
        .zip(authors.iter().map(|author| vec![author.clone()])),
    );
  }
  if let [sole] = authors.as_slice() {
    for (label, institute) in &orphan_labels {
      if !*institute || unaffiliated {
        owners
          .entry(label.clone())
          .or_insert_with(|| vec![sole.clone()]);
      }
    }
  }
  let mut shared_creator: Option<Node> = None;
  // An orphan's owners among the authors (below), by its label and role.
  // (several owners for an institute, or for a person's note whose fuzzy label names them each — a correspondent
  // naming two; not an institute's `\email`/`\url` that inherited its label, which stays shared, 1205.6171)
  let owned_by = |label: &str, note: &Node| {
    let role = note.get_attribute("role").unwrap_or_default();
    owners.get(label).filter(|named| {
      named.len() == 1
        || is_institute_role(&role)
        || (label.starts_with("fuzzy:") && !is_shared_contact_role(&role))
    })
  };
  // The annotations an author takes, by kind and text, with their labels: a copy of one under another label that no
  // author takes is no orphan to share (`\address{$^{1,5}$Lab …}` with no author marked 5, osa_journal_sty.rs
  // `lines_per_label`, 1101.5841). Read before placing any, as the orphan copy may come first (`$^{5,1}$`).
  let mut taken: Vec<(String, Option<String>, String, String)> = Vec::new();
  for pending in &pending_nodes {
    for note in element_nodes(pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      let content = annotation_value(&note).trim().to_string();
      if !label.is_empty()
        && !content.is_empty()
        && (find_targets(&label).is_some() || owned_by(&label, &note).is_some())
      {
        taken.push((note.get_name(), note.get_attribute("role"), content, label));
      }
    }
  }
  let mut held = HeldAnnotations::new(&authors);
  for mut pending in pending_nodes {
    let mut promote_this = false;
    // the stub's notes placed elsewhere, which a promoted stub must not keep a second time
    let mut placed: Vec<Node> = Vec::new();
    for note in element_nodes(&pending) {
      let label = note.get_attribute("_label").unwrap_or_default();
      // An annotation given before any author with no label to find one by (cas's `\affiliation{…}` above the first
      // `\author`, which cas-common.sty:1252-1290 prints under the authors) is an orphan like the labelled ones below:
      // a sole author's, else shared. Perl drops it unplaced. Guard:
      // `perfect_kernel_batch63::cas_affiliation_before_author`.
      let unlabelled = label.is_empty();
      // (an empty one is nothing to keep: it made a nameless creator of its own; nor one an author already holds — a
      // store set between two `\maketitle`s is handed on to the authors a later `\author` names, and queued too:
      // `perfect_kernel_batch59::store_set_before_a_superseding_author`)
      let empty = annotation_value(&note).trim().is_empty() && element_nodes(&note).is_empty();
      // (metadata, an `ltx:rdf`, is held by an author who has its value in any annotation: 2608.17008's `\email{x}` ahead
      // of the authors, then `\email[show]{x}` on the first, is that author's displayed address: `HeldAnnotations`)
      if unlabelled && (pending.get_name() != "creator" || empty || held.holds(&authors, &note)) {
        continue;
      }
      let targets = if unlabelled {
        match authors.as_slice() {
          [author] => Some(vec![author.clone()]),
          _ => None,
        }
      } else {
        find_targets(&label)
      };
      if let Some(targets) = targets {
        placed.push(note.clone());
        for target in targets {
          // (nor twice under two labels: an entry marked with several labels is one copy a label — OSA's
          // `\address{$^{1,2}$Univ X}`, osa_journal_sty.rs `lines_per_label`, 1406.0824 — which reaches an author
          // marked with two of them once; two entries under one label are two, as the PDF prints them; an annotation
          // without text, an icon's link, is compared by nothing. Guards
          // `perfect_kernel_batch64u::osa_several_labels_one_line_each`,
          // `perfect_kernel_batch61::marked_affiliation_review_shapes`)
          let (role, content) = (note.get_attribute("role"), annotation_value(&note));
          if !content.trim().is_empty()
            && element_nodes(&target).iter().any(|c| {
              c.get_name() == note.get_name()
                && c.get_attribute("role") == role
                && c.get_attribute("_label").is_some_and(|l| l != label)
                && annotation_value(c).trim() == content.trim()
            })
          {
            continue;
          }
          DebugFeature!("frontmatter", "FRONT Moving annotation for {label}");
          let mut target = target;
          document.append_clone(&mut target, vec![note.clone()])?;
          contact_in_title_as_note(document, &target)?;
          at_queue_position(&target);
          held.touched(&target);
        }
      } else {
        let content = annotation_value(&note);
        if !unlabelled
          && owned_by(&label, &note).is_none()
          && taken.iter().any(|(name, role, text, other)| {
            *name == note.get_name()
              && *role == note.get_attribute("role")
              && *text == content.trim()
              && *other != label
          })
        {
          placed.push(note.clone());
          continue;
        }
        // (none for an orphan whose fuzzy label names its owners: a correspondent naming two authors, placed below; a
        // sole author's orphan still warns, 63h)
        let named_by_label = label.starts_with("fuzzy:") && owned_by(&label, &note).is_some();
        if !unlabelled && !shared_orphan(&note) && !named_by_label {
          let mut known: Vec<&String> = labeltable.keys().collect();
          known.sort();
          Warn!(
            "unexpected",
            "annotation",
            s!("Orphaned frontmatter annotation couldn't find target for label={label}"),
            s!(
              "known labels={}",
              known
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(",")
            )
          );
        }
        // (an institute several authors share is each one's; its `\email`/`\url` names one of them, which, unknown,
        // stays shared: `kono@rice.edu` under one `L. Ren, Q. Zhang, S. Nanot, and J. Kono \at …` piece, 1205.6171)
        if let Some(named) = owned_by(&label, &note) {
          placed.push(note.clone());
          DebugFeature!(
            "frontmatter",
            "FRONT Giving orphaned annotation {label} to its authors"
          );
          for owner in named {
            let mut owner = owner.clone();
            document.append_clone(&mut owner, vec![note.clone()])?;
            at_queue_position(&owner);
            held.touched(&owner);
          }
        } else {
          DebugFeature!(
            "frontmatter",
            "FRONT Sharing orphaned annotation {label} below authors"
          );
          match &mut shared_creator {
            // First orphan: promote THIS pending stub in place into the trailing shared creator (its note already
            // lives inside it).
            None => {
              promote_this = true;
            },
            // Later orphans (e.g. the institute's `\email`) join it.
            Some(sc) => {
              let mut sc = sc.clone();
              document.append_clone(&mut sc, vec![note.clone()])?;
              held.touched(&sc);
            },
          }
        }
      }
    }
    // (what follows changes the stub, and so any author holding it)
    held.touched(&pending);
    if promote_this {
      // pending → real trailing creator (role=pending would otherwise be dropped
      // with the node; the `_annotations`/`_label` bookkeeping attrs are stripped
      // at serialization). Name-LESS: it carries no `<ltx:personname>`.
      document.set_attribute(&mut pending, "role", "author")?;
      for note in placed {
        document.remove_node(note);
      }
      // below the authors, where a shared note prints: a stub queued before the first author stood above them
      // (cas's `\affiliation{…}` above its `\author`s; aastex's `\correspondingauthor`+`\email` first, 2408.07136)
      if let Some(last) = authors.last() {
        move_content_free_after(std::slice::from_ref(&pending), last.clone())?;
      }
      held.touched(&pending);
      shared_creator = Some(pending);
    } else {
      document.remove_node(pending);
    }
  }
  Ok(())
}

/// A kind of annotation [`HeldAnnotations`] reads: an element name and role, or (`None`) a contact or metadata of any role.
type AnnotationKind = Option<(String, Option<String>)>;

/// The annotations the authors hold, for [`relocate_annotations`]'s test of an unlabelled annotation: is one of the same
/// kind (element name and role) and text on an author, or — for metadata, an `ltx:rdf` — a contact or metadata of its
/// text? Each author's values of one kind are read the first time that kind is asked about (an author's element
/// children, their text trimmed), and read again after anything changed the author or what it holds (`touched`): the
/// answer is the one reading every author's children at the time of the question gives, and an author's children are
/// read once per kind, not once per question.
struct HeldAnnotations {
  /// each author's position, by node (`Node::to_hashable`)
  position: HashMap<usize, usize>,
  /// per author: the trimmed texts of its children of each kind read so far (`None`: a contact or metadata, any role)
  read:     Vec<HashMap<AnnotationKind, HashSet<String>>>,
}

impl HeldAnnotations {
  fn new(authors: &[Node]) -> Self {
    HeldAnnotations {
      position: authors
        .iter()
        .enumerate()
        .map(|(i, author)| (author.to_hashable(), i))
        .collect(),
      read:     vec![HashMap::default(); authors.len()],
    }
  }

  /// Does an author hold `note`'s kind with `note`'s text (or, `note` being metadata, a contact or metadata of it)?
  fn holds(&mut self, authors: &[Node], note: &Node) -> bool {
    let name = note.get_name();
    let kind = Some((name.clone(), note.get_attribute("role")));
    let metadata = name == "rdf";
    let value = annotation_value(note);
    let value = value.trim();
    authors.iter().enumerate().any(|(i, author)| {
      self.values(i, author, &kind).contains(value)
        || (metadata && self.values(i, author, &None).contains(value))
    })
  }

  /// The trimmed texts of author `i`'s element children of `kind`, read once.
  fn values(&mut self, i: usize, author: &Node, kind: &AnnotationKind) -> &HashSet<String> {
    self.read[i].entry(kind.clone()).or_insert_with(|| {
      element_nodes(author)
        .iter()
        .filter(|c| match kind {
          Some((name, role)) => c.get_name() == *name && c.get_attribute("role") == *role,
          None => matches!(c.get_name().as_str(), "contact" | "rdf"),
        })
        .map(|c| annotation_value(c).trim().to_string())
        .collect()
    })
  }

  /// `node` changed (an annotation placed on it, renamed in it, removed from it, or it moved): the authors it is or
  /// sits in are read again.
  fn touched(&mut self, node: &Node) {
    let mut cur = Some(node.clone());
    while let Some(n) = cur {
      if let Some(&i) = self.position.get(&n.to_hashable()) {
        self.read[i].clear();
      }
      cur = n.get_parent();
    }
  }
}

/// A labelled annotation relocated into an element that holds no contact — a title, whose model has none (aastex's
/// title footnote `\title{..\altaffilmark{1}}` + `\altaffiltext{1}`, 0704.0478, 1001.2402; elsarticle's
/// `\tnoteref`/`\tnotetext`) — is that element's note: the frontmatter thanks note `\lx@add@thanks` makes, as LaTeX
/// prints it (a footnote marked on the title). Perl clones the `ltx:contact` as it is, schema-invalid there.
fn contact_in_title_as_note(document: &mut Document, target: &Node) -> Result<()> {
  let Some(clone) = target.get_last_child() else {
    return Ok(());
  };
  if clone.get_type() != Some(NodeType::ElementNode)
    || document::get_node_qname(&clone) != pin_static("ltx:contact")
    || document::can_contain_qsym(document::get_node_qname(target), pin_static("ltx:contact"))
  {
    return Ok(());
  }
  let kind = classify_thanks(&clone.get_content());
  let mut note = document.rename_node(clone, "ltx:note", false)?;
  document.set_attribute(&mut note, "role", "thanks")?;
  document.add_class(&mut note, &s!("ltx_note_frontmatter ltx_thanks_{kind}"))
}

/// The annotation attribute holding its place in the frontmatter queue (set by `\lx@annotate@frontmatter@now`, in the
/// order the queue is digested): [`at_queue_position`].
pub(in crate::base_utilities) const QUEUE_ORDINAL: &str = "_queued";

/// The state counter of [`QUEUE_ORDINAL`].
pub(in crate::base_utilities) const QUEUE_ORDINAL_COUNTER: &str = "lx_frontmatter_queue_ordinal";

/// The print zones of an author block, each an offset added to an annotation's queue place ([`QUEUE_ORDINAL`]), so a
/// creator's contacts are ordered by zone, then by queue (user ruling R8, print order): the names' zone (a contact the
/// class prints with the names, above the block's entries: elsarticle.cls:665 `\@address` adds an unlabelled `\address`
/// to `\elsauthors`, 1710.03753), the block's (the default: an entry printed under the names), and the footer's (a
/// contact printed as a footnote, after every entry of the block, whenever it was queued: revtex's `\email`, 2312.02779;
/// aa's `\mail`; any contact made inside a name's note, `Ann Able$^{1}$\thanks{\email{…}}`). A binding declares a
/// contact's zone where its class prints it apart from the block (`_zone=names`, `_zone=footer`: [`zone_offset`]); the
/// API names no class.
pub(in crate::base_utilities) const NAMES_ZONE: i64 = 0;
/// The block's zone ([`NAMES_ZONE`]).
pub(in crate::base_utilities) const BLOCK_ZONE: i64 = 1 << 40;
/// The footer's zone ([`NAMES_ZONE`]).
pub(in crate::base_utilities) const FOOTER_ZONE: i64 = 1 << 41;

/// The offset of the print zone a contact's `_zone` key declares ([`NAMES_ZONE`]): `names`, `footer`, else the block's.
pub(in crate::base_utilities) fn zone_offset(zone: Option<&str>) -> i64 {
  match zone.map(str::trim) {
    Some("names") => NAMES_ZONE,
    Some("footer") => FOOTER_ZONE,
    _ => BLOCK_ZONE,
  }
}

/// The roles of the annotations printed in the author block itself, under or beside the names: an affiliation, an
/// address ([`at_queue_position`]).
const BLOCK_ROLES: [&str; 2] = ["affiliation", "address"];

/// The annotation just appended to `target` (its last element child) moved to the place its call had in the queue:
/// before the first of `target`'s annotations queued after it ([`QUEUE_ORDINAL`]). Rust-only (user ruling R8, print
/// order: a creator's contacts follow the order the PDF prints them; OXIDIZED_DESIGN_DIVERGENCES #954): an annotation
/// placed by its label is appended once every annotation is digested, after the ones attached when they were digested
/// — amsart's `\address{${}^{1}$Univ A\\ ${}^{2}$Univ B}` cut at its marks went after the `\email` that follows it
/// (2509.03485), acmart's marked `\affiliation` after its `\email` (2112.12060, 2209.09217), REVTeX 3's marked institute
/// after the abstract's `\address` lines (hep-th9805210), an aa institute after the address its name's
/// `\thanks{\email{…}}` prints in the footer (2212.07438, webofc 2409.01693). Only an entry the block prints (an
/// affiliation, an address: [`BLOCK_ROLES`]): a note a reference labels — elsarticle's `\cortext`/`\fntext`, aastex's
/// `\altaffiltext` — prints in the footer, after the block, where it is appended.
pub(in crate::base_utilities) fn at_queue_position(target: &Node) {
  let place = |node: &Node| {
    node
      .get_attribute(QUEUE_ORDINAL)
      .and_then(|n| n.parse::<i64>().ok())
  };
  let children = element_nodes(target);
  let Some(mut moved) = children.last().cloned() else {
    return;
  };
  let Some(mut at) = place(&moved) else {
    return;
  };
  // (an entry of a printed list the Cut cut, which an uncut copy of the list printed for this author earlier, keeps
  // that copy's place: `affiliations::place_uncut_copies`, `\lx@cut@copy@place`)
  if let Some(key) = moved.get_attribute(CUT_LIST)
    && let Some(copy) = target
      .get_attribute("_annotations")
      .unwrap_or_default()
      .split(',')
      .filter(|label| label.starts_with("author:"))
      .find_map(
        |label| match lookup_value(&s!("{CUT_COPY_PLACE}{key}:{label}")) {
          Some(Stored::Int(copy)) => Some(copy),
          _ => None,
        },
      )
  {
    at = copy;
    let _ = moved.set_attribute(QUEUE_ORDINAL, &at.to_string());
  }
  if !moved
    .get_attribute("role")
    .is_some_and(|role| BLOCK_ROLES.contains(&role.as_str()))
  {
    return;
  }
  // (a collaboration is the names row's, printed before every entry under it, whenever its call came: `\author{Ann
  // Able$^{1}$ \and The XYZ Collaboration \\ $^{1}$Univ X}`, `perfect_kernel_batch97d`)
  if let Some(mut later) = children
    .iter()
    .find(|c| {
      **c != moved
        && place(c).is_some_and(|p| p > at)
        && c.get_attribute("role").as_deref() != Some("collaboration")
    })
    .cloned()
  {
    let _ = later.add_prev_sibling(&mut moved);
  }
}

/// The marks a creator's name shows: each `<ltx:sup>`'s (`\textsuperscript{a}`), and each formula's that is a mark
/// (`$^{a}$`, `$^{1,2}$`, `$\,{}^{a,b}$` — 2311.14432 — `$\rm ^{1}$`: [`fm_read_formula`], which reads the formula as
/// digested, before the math is rewritten into an `<ltx:sup>`), from their printed text ([`printed_marks`]). A script on
/// a base (`$\text{Leon Bello}^{1,3}$`) or before printed ink (`${\rm ^{3}He}$`) shows none.
pub(in crate::base_utilities) fn shown_marks(
  document: &mut Document,
  creator: &Node,
) -> Vec<String> {
  let mut marks: Vec<String> = document
    .findnodes("ltx:personname//ltx:sup", Some(creator))
    .iter()
    .flat_map(|sup| printed_marks(&sup.get_content()))
    .collect();
  for math in document.findnodes("ltx:personname//ltx:Math", Some(creator)) {
    if let FmFormulaMarks::Marks(scripts) = fm_read_formula(&math) {
      marks.extend(scripts.iter().flat_map(|script| printed_marks(script)));
    }
  }
  marks
}

/// The text of the braced group `text` opens (just past its `{`), to its matching `}`, and what follows it.
fn braced_group(text: &str) -> (&str, &str) {
  let mut depth = 0usize;
  for (i, c) in text.char_indices() {
    match c {
      '{' => depth += 1,
      '}' if depth == 0 => return (&text[..i], &text[i + 1..]),
      '}' => depth -= 1,
      _ => {},
    }
  }
  (text, "")
}

/// The letter or number marks of a superscript's script: each item of its list at its top-level commas (`2,\dagger`:
/// the 2), past braces and font and size switches (`\rm a`, `\mathrm{a}`, `{\rm 2}`, `\footnotesize 1`: the a, the 2,
/// the 1), and before a symbol or closing parenthesis after it (`2*`, `1\dagger`, `1†`, `1)`: the 2, the 1; the `1)` of
/// 2306.17039, as `clean_frontmatter_labels` reads its label). A symbol alone or other markup (`\footnote{…}`) is no
/// mark. Both the marks an affiliation is labelled by (from its spelling, here) and the marks an author shows (from the
/// printed text, [`printed_marks`]) are split so (63i review). The spelling's readers are the token-only callers (the
/// `\lx@frontmatter@sup` label keys, revtex's group rule): the frontmatter refactor's later stages
/// (`docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md`).
pub(in crate::base_utilities) fn script_marks(script: &str) -> Vec<String> {
  split_marks(script, past_mark_switches)
}

/// The letter or number marks of a superscript's printed text (an `<ltx:sup>`'s, a mark formula's scripts': `1,2`,
/// `1∗`, `[1],[2]`, `a)`), read as [`script_marks`] reads a script's spelling, items and all.
pub(in crate::base_utilities) fn printed_marks(text: &str) -> Vec<String> {
  split_marks(text, str::trim_start)
}

/// A script's marks ([`script_marks`]): `past` skips what opens the script and each item before its mark (the
/// spelling's switches and spacing, [`past_mark_switches`]; the printed text's spaces).
fn split_marks(script: &str, past: fn(&str) -> &str) -> Vec<String> {
  // (the whole script under its switches, `\footnotesize\ensuremath{1,2}`: the 1 and the 2; 2411.14110)
  let script = past(script);
  let mut items = Vec::new();
  let (mut depth, mut start) = (0usize, 0usize);
  for (i, c) in script.char_indices() {
    match c {
      '{' => depth += 1,
      '}' => depth = depth.saturating_sub(1),
      ',' if depth == 0 => {
        items.push(&script[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  items.push(&script[start..]);
  // (a number range is each number of it, `1-4`, `1--4`, the printed `1–4`/`1−4`: `queue_digest::spelled_ranges`, the one
  // reader of a range — Bauer's `$^{1-4}$`, 2012.15817)
  let items: Vec<String> = items
    .into_iter()
    .flat_map(|item| {
      let cleaned: String = past(item)
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '{' && *c != '}')
        .collect();
      let spelled = spelled_ranges(vec![cleaned.clone()]);
      if spelled.len() > 1 {
        spelled
      } else {
        vec![item.to_string()]
      }
    })
    .collect();
  items
    .iter()
    .map(String::as_str)
    .filter_map(|item| {
      // (a number in brackets or parentheses is the number: `$^{[1],[2]}$` against the line's `$^1$`, 1803.06822; so is
      // one after a footnote symbol glued to it, `\textsuperscript{*3}`, 2412.01183)
      let item: String = past(item)
        .trim_start_matches(|c: char| {
          matches!(c, '[' | '(' | '*') || (!c.is_ascii() && !c.is_alphanumeric())
        })
        .chars()
        .filter(|c| !matches!(c, '{' | '}') && !c.is_whitespace())
        .collect();
      let lead: String = item.chars().take_while(|c| c.is_alphanumeric()).collect();
      let rest = &item[lead.len()..];
      let symbol_after = rest.is_empty()
        || rest.starts_with(['*', '\\', ')', ']'])
        || rest.starts_with(|c: char| !c.is_ascii() && !c.is_alphanumeric());
      (!lead.is_empty() && symbol_after).then_some(lead)
    })
    .collect()
}

/// The font and size switches, and the math wrapper, a mark is printed under (`{\rm 2}`, `\mathrm{a}`,
/// `\textsuperscript{\footnotesize\ensuremath{1}}`, 2502.16662 and 2411.14110's `\IEEEauthorrefmark`), which neither
/// an affiliation's mark nor its leading text is: for the readers that have only the tokens ([`script_marks`],
/// `notes::leads_with_mark`); a formula read from the document is read from its structure (`observer::marks`).
pub(in crate::base_utilities) const MARK_SWITCHES: [&str; 32] = [
  "\\rm",
  "\\it",
  "\\bf",
  "\\sl",
  "\\sf",
  "\\tt",
  "\\em",
  "\\mathrm",
  "\\mathit",
  "\\mathbf",
  "\\mathsf",
  "\\textrm",
  "\\textit",
  "\\textbf",
  "\\textsf",
  "\\textup",
  "\\textnormal",
  "\\itshape",
  "\\upshape",
  "\\bfseries",
  "\\rmfamily",
  "\\normalfont",
  "\\tiny",
  "\\scriptsize",
  "\\footnotesize",
  "\\small",
  "\\normalsize",
  "\\large",
  "\\Large",
  "\\ensuremath",
  "\\scriptstyle",
  "\\textstyle",
];

/// The spacing a mark is set apart with (`{}^{a,\;b}`, `{}^{\rm b,\ c}`, `\,{}^{b,c}`: 1307.6166, hep-th0410034,
/// 2311.14432), which no mark is: the control words; the control symbols `\,` `\;` `\:` `\!` `\ ` and the tie besides.
/// For the token readers, as [`MARK_SWITCHES`].
const MARK_SPACING: [&str; 4] = ["\\quad", "\\qquad", "\\thinspace", "\\enspace"];

/// `text` past the switches it opens with ([`MARK_SWITCHES`]) and the spacing ([`MARK_SPACING`]), with the braces and
/// spaces around them.
fn past_mark_switches(text: &str) -> &str {
  let mut rest = text;
  loop {
    rest = rest.trim_start_matches(|c: char| c == '{' || c == '~' || c.is_whitespace());
    let Some(after) = rest.strip_prefix('\\') else {
      return rest;
    };
    if let Some(symbol) = after.chars().next()
      && matches!(symbol, ',' | ';' | ':' | '!' | ' ')
    {
      rest = &after[1..];
      continue;
    }
    let len = after
      .find(|c: char| !c.is_ascii_alphabetic())
      .unwrap_or(after.len());
    if len == 0
      || !(MARK_SWITCHES.contains(&&rest[..=len]) || MARK_SPACING.contains(&&rest[..=len]))
    {
      return rest;
    }
    rest = &after[len..];
  }
}

/// The marks of each superscript in a TeX source string, one list per superscript (`Ann Able$^{1,2}$ and Bob$^{1}$`
/// gives [1, 2] and [1]; `^a` gives [a]; `\textsuperscript{b}` gives [b]).
fn superscript_scripts(tex: &str) -> Vec<Vec<String>> {
  const TEXTSUP: &str = "\\textsuperscript";
  let mut scripts = Vec::new();
  let mut rest = tex;
  loop {
    // (a `^` that is no `\^` accent: `Ren\^{e}`)
    let caret = rest
      .char_indices()
      .find(|&(i, c)| c == '^' && !rest[..i].ends_with('\\'))
      .map(|(i, _)| i);
    let (at, skip) = match (caret, rest.find(TEXTSUP)) {
      (Some(caret), Some(text)) if text < caret => (text, TEXTSUP.len()),
      (Some(caret), _) => (caret, 1),
      (None, Some(text)) => (text, TEXTSUP.len()),
      (None, None) => break,
    };
    rest = rest[at + skip..].trim_start();
    if let Some(inner) = rest.strip_prefix('{') {
      let (script, after) = braced_group(inner);
      scripts.push(script_marks(script));
      rest = after;
    } else if let Some(c) = rest.chars().next() {
      scripts.push(script_marks(&c.to_string()));
      rest = &rest[c.len_utf8()..];
    }
  }
  scripts
}

/// The distinct marks of the superscripts in `content`, read from its TeX spelling (`$^1$X $^2$Y`: 1 and 2).
pub fn content_marks(content: &Tokens) -> Vec<String> {
  let mut marks: Vec<String> = Vec::new();
  for mark in superscript_scripts(content.clone().untex_string().as_ref()).concat() {
    if !marks.contains(&mark) {
      marks.push(mark);
    }
  }
  marks
}

/// Is the affiliation `content` the group's, as revtex's group rule gives it? So it is when an author was queued since
/// a previous affiliation (interleaved `\author`/`\affiliation` groups, whatever their marks say: a mark no author shows
/// is still its group's, as the PDF pairs it), and when the first affiliation carries one mark that every author
/// queued shows (`\author{A$^{1}$ and B$^{1,2}$}\affiliation{$^{1}$…}`, 0808.2763). Authors listed first and their marked
/// affiliations after them are no such group, and an affiliation holding several marked lines (`$^1$… \\ $^2$…`,
/// 1205.4587) is a marked list (63i review).
pub fn queued_group_shares_the_mark(content: &Tokens) -> bool {
  group_shares_the_mark(content, false)
}

/// [`queued_group_shares_the_mark`], an interleaved group only where the affiliation before the authors itself followed
/// an author (`\author{A}\affiliation{X}\author{B}…`), not names after an affiliation given before the first
/// `\author` (2510.16137, 2209.05548: those names' marked lines are a marked list's): the group a binding declares to
/// the frontmatter observer (revtex's `_grouped=true`, FM S2b).
pub fn interleaved_group_shares_the_mark(content: &Tokens) -> bool {
  group_shares_the_mark(content, true)
}

/// Do the names of the run an affiliation continues request affiliations by their marks — a superscript or mark
/// command after the name (`Ann Able$^{1}$`, `\textsuperscript{1}`; `marked_after_name`) that is no footnote symbol
/// (`Ann Able$^{*}$`, the corresponding author's, requests none)? The run is the authors queued before the
/// affiliations that end the queue, back to the affiliation before them. AASTeX 5's `\altaffilmark{1}` requests an
/// `\altaffiltext` (label `altaffil:1`), no affiliation: a line its number labels goes by it (aas_support_sty.rs
/// `\affiliation`, [`leading_line_label`]), and an unlabelled line stays with the row (user ruling R1, 2026-11-01:
/// 1210.4079's `Pavel\altaffilmark{1} and D. P. Clemens` over two `\affil` lines of one address). A binding whose group
/// rule is `annotate=run` declares `annotate=new` for a run whose names do request affiliations, as a convention may not
/// give them an unlabelled line as a group (1202.0629's `$^{n}$` names over unnumbered lines; aas_support_sty.rs).
pub fn run_names_show_a_mark() -> bool {
  let numbered = |name: &Tokens| {
    marked_after_name(name)
      && author_mark_operands(name.unlist_ref())
        .iter()
        .any(|operand| !is_symbol_mark(operand))
  };
  let mut any = false;
  for_run_names(|name| any |= numbered(name));
  any
}

/// The key of the mark a macro stands for: a macro without parameters whose body leads with a superscript mark
/// (`\edef\MSU{$^{1}$ }`, hep-ph0301204's names and `\address` lines; [`leading_line_label`]).
fn macro_mark(t: &Token) -> Option<String> {
  if t.get_catcode() != Catcode::CS {
    return None;
  }
  let def = lookup_definition(t).ok()??;
  if !def.is_expandable()
    || def
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
  {
    return None;
  }
  let Some(ExpansionBody::Tokens(body)) = def.get_expansion() else {
    return None;
  };
  match leading_line_label(body)? {
    (label, false, _) => Some(label),
    _ => None,
  }
}

/// A line led by a macro that stands for a mark (`macro_mark`: `\address{\MSU Moscow State University, …}`): the
/// mark's key and the line without the macro.
pub fn leading_macro_mark(content: &Tokens) -> Option<(String, Tokens)> {
  let tokens = content.unlist_ref();
  let at = tokens.iter().position(|t| *t != T_SPACE!())?;
  let key = macro_mark(&tokens[at])?;
  let rest = tokens[at + 1..]
    .iter()
    .copied()
    .skip_while(|t| *t == T_SPACE!())
    .collect();
  Some((key, Tokens::new(rest)))
}

/// Do the names of the run an affiliation continues carry macro marks (`macro_mark`), and does one lead a name — before
/// its first letter, or right after a comma: a mark written after the comma (`M.~Osipenko,\MSU$^,$\INFNGE\ G.~Ricco,\INFNGE\ …`,
/// hep-ph0301204), which the comma split puts on the next name, so no name's own marks can be read? The run as
/// [`run_names_show_a_mark`] reads it.
pub fn run_names_macro_marks() -> (bool, bool) {
  let (mut marked, mut leading) = (false, false);
  for_run_names(|name| {
    let tokens = name.unlist_ref();
    let first_letter = tokens
      .iter()
      .position(|t| t.get_catcode() == Catcode::LETTER)
      .unwrap_or(tokens.len());
    for (i, t) in tokens.iter().enumerate() {
      if macro_mark(t).is_some() {
        marked = true;
        // (before the name's first letter, or right after a comma: one `\author` call queues its whole list)
        let after_comma =
          tokens[..i].iter().rev().find(|t| **t != T_SPACE!()) == Some(&T_OTHER!(","));
        leading |= i < first_letter || after_comma;
      }
    }
  });
  (marked, leading)
}

/// Calls `each` on every name of the run an affiliation continues (`queued_name`: the name, not the call around it):
/// the authors queued before the affiliations that end the queue, back to the affiliation before them.
fn for_run_names(mut each: impl FnMut(&Tokens)) {
  with_value("frontmatter_raw", |v| {
    if let Some(Stored::FrontmatterRaw(queue)) = v {
      queue
        .iter()
        .rev()
        .skip_while(|entry| entry.0 != "ltx:creator")
        .take_while(|entry| {
          !(entry.0 == "ltx:contact"
            && entry.1.get("role").map(String::as_str) == Some("affiliation"))
        })
        .filter(|entry| entry.0 == "ltx:creator")
        .for_each(|entry| each(&queued_name(&entry.2)));
    }
  });
}

/// A mark's printed number or letters as a label key: its letters and digits only (`1`, `{1}`, `\hbox{ }2` → `2`);
/// a mark of neither, its spelling (`*`).
fn mark_key(text: &str) -> String {
  let key = alphanumeric_key(text);
  if key.is_empty() {
    text.trim().to_string()
  } else {
    key
  }
}

fn alphanumeric_key(text: &str) -> String {
  mark_spelling_printed(text)
    .chars()
    .filter(|c| c.is_alphanumeric())
    .collect()
}

/// What a mark's spelling prints: the spelling without its control words and its braces, no part of the mark (`\hbox{
/// }2` → ` 2`) — the one reading of a written mark the binding's keys (`mark_key`) and the frontmatter observer's
/// request labels (`observer::events`, `\lx@request@frontmatter@annotation`) share: astro-ph0105133's `Yu-Qing Lou
/// \altaffilmark{1,\hbox{ }2}` requests `altaffil:1` and `altaffil:2`, never a label `\hbox` (review rev126a B3).
pub fn mark_spelling_printed(text: &str) -> String {
  let mut printed = String::new();
  let mut chars = text.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\' {
      // (a control word, `\hbox`, is no part of the mark)
      while chars.next_if(|c| c.is_ascii_alphabetic()).is_some() {}
    } else if c != '{' && c != '}' {
      printed.push(c);
    }
  }
  printed
}

/// The label an affiliation line leads with, whether it was typed by hand, and the line without it: a superscript
/// mark (`$^{1}$`, `$^1$`, `\textsuperscript{1}` or another mark command, `\IEEEauthorrefmark{1}`, or spelled
/// otherwise, `spaced_mark_at`: `${}^{1}$`, `\( ^{1} \)`) or a
/// hand-printed number (`1 Univ A`, `2) Univ B`, `3. Univ C`), behind the spaces and the font and size switches it is set in (`\scriptsize 2) Osservatorio …`, kept). `None` for a
/// line leading with neither (user ruling R2, 2026-11-01: a hand-printed leading number is a label wherever the names
/// request numbers).
pub fn leading_line_label(content: &Tokens) -> Option<(String, bool, Tokens)> {
  let tokens = content.unlist_ref();
  let mut at = 0;
  while let Some(t) = tokens.get(at) {
    let switch = t.get_catcode() == Catcode::CS && t.with_str(|s| MARK_SWITCHES.contains(&s));
    if *t == T_SPACE!() || switch {
      at += 1;
    } else {
      break;
    }
  }
  let prefix = &tokens[..at];
  let rebuilt = |label: String, typed: bool, from: usize| {
    let mut rest = prefix.to_vec();
    rest.extend(
      tokens[from..]
        .iter()
        .copied()
        .skip_while(|t| *t == T_SPACE!()),
    );
    (!label.is_empty()).then(|| (label, typed, Tokens::new(rest)))
  };
  let first = *tokens.get(at)?;
  // (a superscript spelled otherwise — on a spacing nucleus, in `\( … \)`, in a box of its own: `spaced_mark_at`)
  if let Some((operand, next)) = spaced_mark_at(tokens, at) {
    return rebuilt(
      mark_key(Tokens::new(operand).untex_string().as_ref()),
      false,
      next,
    );
  }
  // (a superscript: `$^{1}$`, `\textsuperscript{1}`)
  if first.get_catcode() == Catcode::MATH && tokens.get(at + 1) == Some(&T_SUPER!()) {
    let (operand, next) = sup_operand_at(tokens, at + 2)?;
    if tokens
      .get(next)
      .is_none_or(|t| t.get_catcode() != Catcode::MATH)
    {
      return None;
    }
    return rebuilt(
      mark_key(Tokens::new(operand).untex_string().as_ref()),
      false,
      next + 1,
    );
  }
  // (a mark command or an alias of one, `is_mark_command`: IEEEtran's `\IEEEauthorrefmark{1}\{ms1801,
  // bob\}@x.edu` address line as `\textsuperscript{1}`'s, 2301.03018, 2510.06608; read literally, its addresses
  // stayed an affiliation of every mark-1 author)
  if is_mark_command(&first) {
    let (operand, next) = sup_operand_at(tokens, at + 1)?;
    return rebuilt(
      mark_key(Tokens::new(operand).untex_string().as_ref()),
      false,
      next,
    );
  }
  // (a number typed by hand, followed by `)`, `.` or a space)
  let digit = |t: &Token| {
    t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit()))
  };
  let digits = tokens[at..].iter().take_while(|t| digit(t)).count();
  if digits == 0 {
    return None;
  }
  let label: String = tokens[at..at + digits]
    .iter()
    .map(ToString::to_string)
    .collect();
  match tokens.get(at + digits) {
    Some(t) if *t == T_OTHER!(")") || *t == T_OTHER!(".") => rebuilt(label, true, at + digits + 1),
    Some(t) if *t == T_SPACE!() => rebuilt(label, true, at + digits),
    _ => None,
  }
}

/// The marks the queued authors' names show, as label keys (`mark_key`): the items of AASTeX 5's `\altaffilmark{…}`
/// (which request `altaffil:` labels), and those of their superscripts (`affiliation:` labels).
pub fn queued_name_marks() -> (Vec<String>, Vec<String>) {
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => {
      let (mut altaffil, mut sup) = (Vec::new(), Vec::new());
      for entry in queue.iter().filter(|entry| entry.0 == "ltx:creator") {
        let tokens = entry.2.unlist_ref();
        for (i, t) in tokens.iter().enumerate() {
          if *t == T_CS!("\\altaffilmark")
            && let Some((operand, _)) = sup_operand_at(tokens, i + 1)
          {
            altaffil.extend(
              Tokens::new(operand)
                .untex_string()
                .as_ref()
                .split(',')
                .map(mark_key)
                .filter(|key| !key.is_empty()),
            );
          }
        }
        sup.extend(
          superscript_scripts(&entry.2.to_string())
            .concat()
            .iter()
            .map(|m| mark_key(m)),
        );
      }
      (altaffil, sup)
    },
    _ => (Vec::new(), Vec::new()),
  })
}

/// How many of the marks the queued authors' names show as superscripts (`queued_name_marks`) `content` prints, each
/// once: the labelled lines an affiliation block holds for the names (`\address{\parbox{\linewidth}{$^{1}$Univ A\\
/// $^{2}$Univ B}}`: 2).
pub fn requested_marks_held(content: &Tokens) -> usize {
  let (_, requested) = queued_name_marks();
  let mut held: Vec<String> = Vec::new();
  for mark in content_marks(content) {
    let key = mark_key(&mark);
    if requested.contains(&key) && !held.contains(&key) {
      held.push(key);
    }
  }
  held.len()
}

/// The items of `tokens` separated by `\\` at its top level, outside every group and `\begin`…`\end` environment.
pub fn split_top_level_lines(tokens: &[Token]) -> Vec<&[Token]> {
  let (mut items, mut start, mut depth, mut envs) = (Vec::new(), 0, 0usize, 0usize);
  for (i, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if *t == T_CS!("\\begin") => envs += 1,
      _ if *t == T_CS!("\\end") => envs = envs.saturating_sub(1),
      _ if depth == 0 && envs == 0 && t.defined_as(&T_CS!("\\\\")) => {
        items.push(&tokens[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  items.push(&tokens[start..]);
  items
}

/// LaTeX's line macros, each call a line of its own whatever stands beside it: a box as wide as the line
/// (latex.ltx:16403-16406 `\def\@@line{\hb@xt@\hsize}`, `\centerline[1]{\@@line{\hss#1\hss}}`, `\leftline`,
/// `\rightline`; plain.tex:575-577 the same through `\line`, which LaTeX makes the picture command).
const LINE_MACROS: [&str; 3] = ["\\centerline", "\\leftline", "\\rightline"];
/// The boxes a line may be set in (latex.ltx `\mbox`, the primitive `\hbox`): a line only when one is the whole line, as
/// two set side by side are one line — unless marks lead them, the first and another, which part the entries as they
/// part them on one line, a box no mark leads going on with the entry before it (`\mbox{$^{1}$Univ A}\mbox{$^{2}$Univ
/// B,}\mbox{Town}`, 1412.5812).
pub(in crate::base_utilities) const LINE_BOXES: [&str; 2] = ["\\mbox", "\\hbox"];

/// The lines a line of an affiliation block sets with one-argument line wrappers ([`LINE_MACROS`]: one line each;
/// [`LINE_BOXES`]: one box that is the whole line, or boxes marks lead), each the wrapper's argument; `None` for a line
/// holding anything else beside them but spacing.
fn wrapped_line_pieces(line: &[Token]) -> Option<Vec<Vec<Token>>> {
  let mut pieces: Vec<&[Token]> = Vec::new();
  let mut boxed = false;
  let mut i = 0;
  loop {
    while i < line.len() && (line[i] == T_CS!("\\par") || line[i] == T_CS!("\\noindent")) {
      i += 1;
    }
    let past = i + leading_spacing_end(&line[i..]);
    if past > i {
      i = past;
      continue;
    }
    let Some(t) = line.get(i) else {
      break;
    };
    let line_macro = LINE_MACROS.iter().any(|cs| *t == T_CS!(*cs));
    let line_box = LINE_BOXES.iter().any(|cs| *t == T_CS!(*cs));
    if !line_macro && !line_box {
      return None;
    }
    let mut open = i + 1;
    while line.get(open) == Some(&T_SPACE!()) {
      open += 1;
    }
    if line
      .get(open)
      .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
    {
      return None;
    }
    let end = skip_group(line, open);
    if end > line.len() || line[end - 1].get_catcode() != Catcode::END {
      return None;
    }
    pieces.push(&line[open + 1..end - 1]);
    boxed |= line_box;
    i = end;
  }
  if pieces.is_empty() {
    return None;
  }
  if !boxed || pieces.len() == 1 {
    return Some(pieces.into_iter().map(<[Token]>::to_vec).collect());
  }
  // (boxes side by side: entries where marks lead them, a box no mark leads the entry before it, a space between)
  let leads: Vec<bool> = pieces
    .iter()
    .map(|piece| leads_with_mark(&Tokens::new(piece.to_vec())))
    .collect();
  if !leads[0] || leads.iter().filter(|lead| **lead).count() < 2 {
    return None;
  }
  let mut entries: Vec<Vec<Token>> = Vec::new();
  for (piece, lead) in pieces.into_iter().zip(leads) {
    match entries.last_mut() {
      Some(entry) if !lead => {
        entry.push(T_SPACE!());
        entry.extend_from_slice(piece);
      },
      _ => entries.push(piece.to_vec()),
    }
  }
  Some(entries)
}

/// An affiliation block whose lines are set with one-argument line wrappers (`\address{\centerline{$^{1}$Univ A}
/// \centerline{$^{2}$Univ B}}`, the D0 author lists of hep-ex0409040 and hep-ex0612013; `\mbox{$^{1}$Univ A}\\ …`)
/// written as the lines they print, `\\` between them — so its lines are read as lines, each by its leading label —
/// when a wrapped line leads with a mark ([`leads_with_mark`]); `None` otherwise (`\address{\centerline{Univ A}}` keeps
/// its wrapper). The wrapper's alignment is dropped with it, as a `\\` line's is none. So are the rows of a one-column
/// `tabular` that is the whole line (`tabular_rows`: `\IEEEauthorblockA{\begin{tabular}{c} $^{1}$ Univ A \\ $^{2}$
/// Univ B \\ …\end{tabular}}`, 2606.28962; revtex's 0911.3234), and the commands set side by side whose arguments marks
/// lead, each with the commands after it, the command kept (`side_by_side_wrapper_lines`: acmart's
/// `\institution{$^1$Univ A}\institution{$^2$Univ B}`, 2006.16977).
pub fn wrapped_lines_as_lines(content: &Tokens) -> Option<Tokens> {
  let mut lines: Vec<Vec<Token>> = Vec::new();
  let mut labelled = false;
  let mut unwrapped = false;
  for line in split_top_level_lines(content.unlist_ref()) {
    match wrapped_line_pieces(line)
      .or_else(|| tabular_rows(line))
      .or_else(|| side_by_side_wrapper_lines(line))
    {
      Some(pieces) => {
        unwrapped = true;
        labelled |= pieces.iter().any(|piece| {
          let piece = Tokens::new(piece.clone());
          leads_with_mark(&piece) || marker_leads(&piece)
        });
        lines.extend(pieces);
      },
      None => lines.push(line.to_vec()),
    }
  }
  if !unwrapped || !labelled {
    return None;
  }
  let mut out: Vec<Token> = Vec::new();
  for (at, line) in lines.into_iter().enumerate() {
    if at > 0 {
      out.push(T_CS!("\\\\"));
    }
    out.extend(line);
  }
  Some(Tokens::new(out))
}

/// The rows of a one-column `tabular` that is the whole of `line` (`\begin{tabular}{c} $^{1}$ Univ A \\ $^{2}$ Univ B
/// \\ \{…\}@… \\ \vspace{-0.3cm} \end{tabular}`, an `\IEEEauthorblockA`'s, 2606.28962: pdflatex prints each row a line,
/// "1 Univ A / 2 Univ B / …"), each row's content; `None` for any other line, or a tabular with a column break (`&`: cells
/// side by side, whose rows are no lines of one list).
fn tabular_rows(line: &[Token]) -> Option<Vec<Vec<Token>>> {
  let past_spacing = |mut i: usize| {
    while line
      .get(i)
      .is_some_and(|t| *t == T_SPACE!() || *t == T_CS!("\\par"))
    {
      i += 1;
    }
    i
  };
  // (`{tabular}` after `\begin` or `\end`: where its name group ends)
  let names_tabular = |at: usize| {
    let open = past_spacing(at);
    (line.get(open)?.get_catcode() == Catcode::BEGIN).then_some(())?;
    let end = skip_group(line, open);
    (Tokens::new(line.get(open + 1..end.checked_sub(1)?)?.to_vec()).to_string() == "tabular")
      .then_some(end)
  };
  let begin = past_spacing(0);
  if line.get(begin) != Some(&T_CS!("\\begin")) {
    return None;
  }
  let mut i = past_spacing(names_tabular(begin + 1)?);
  // (its position, `[t]`, and its column spec)
  if line.get(i) == Some(&T_OTHER!("[")) {
    i += line[i..].iter().position(|t| *t == T_OTHER!("]"))? + 1;
    i = past_spacing(i);
  }
  if line.get(i)?.get_catcode() != Catcode::BEGIN {
    return None;
  }
  let body_start = skip_group(line, i);
  let end = line.iter().rposition(|t| *t == T_CS!("\\end"))?;
  if end < body_start || past_spacing(names_tabular(end + 1)?) != line.len() {
    return None;
  }
  let body = &line[body_start..end];
  let mut depth = 0usize;
  for t in body {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      Catcode::ALIGN if depth == 0 => return None,
      _ => {},
    }
  }
  Some(
    split_top_level_lines(body)
      .into_iter()
      .map(<[Token]>::to_vec)
      .collect(),
  )
}

/// The lines a line of one-argument commands set side by side prints as a marked list's entries
/// (`\institution{$^1$Univ A}\institution{$^2$Univ B}\country{X}`, acmart's address parts, where pdflatex prints
/// "1 Univ A / 2 Univ B / X": 2006.16977): a line at each command whose argument a mark leads (`notes::marker_leads`),
/// holding it and the commands after it up to the next, each command kept — as each piece of one wrapper the Cut parts
/// at its marks keeps it (`notes::split_wrapped_affiliation_marks`). `None` unless marks lead the first command's
/// argument and another's, or for a line holding anything but such commands and spacing: a mark command, a line macro
/// or box ([`wrapped_line_pieces`]), a note's command (`\thanks`, `\footnote`).
fn side_by_side_wrapper_lines(line: &[Token]) -> Option<Vec<Vec<Token>>> {
  let mut lines: Vec<Vec<Token>> = Vec::new();
  let mut marked = 0;
  let mut i = 0;
  while i < line.len() {
    if is_spacing(&line[i]) {
      i += 1;
      continue;
    }
    let t = line[i];
    let command = t.get_catcode() == Catcode::CS
      && !is_mark_command(&t)
      && !LINE_MACROS.iter().any(|cs| t == T_CS!(*cs))
      && !LINE_BOXES.iter().any(|cs| t == T_CS!(*cs))
      && !t.with_str(is_name_note_command)
      && line
        .get(i + 1)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN);
    if !command {
      return None;
    }
    let end = skip_group(line, i + 1);
    let leads = marker_leads(&Tokens::new(line[i..end].to_vec()));
    match lines.last_mut() {
      Some(last) if !leads => last.extend_from_slice(&line[i..end]),
      None if !leads => return None,
      _ => {
        marked += 1;
        lines.push(line[i..end].to_vec());
      },
    }
    i = end;
  }
  (marked >= 2).then_some(lines)
}

/// The spacing a superscript's empty nucleus may hold, which prints no glyph: the spacing commands
/// (`email_block_calls::is_spacing`), `\!` and `\negthinspace` (`${\,\!}^{1}$`, 2112.13115).
fn is_nucleus_spacing(t: &Token) -> bool {
  is_spacing(t) || *t == T_CS!("\\!") || *t == T_CS!("\\negthinspace")
}

/// The token readers' one reading of a superscript mark spelled other than `$^{1}$`/`\textsuperscript{1}` at `at`, as the
/// digested formula prints it — a superscript on a spacing nucleus, `${\,\!}^{1}$`, `${}^1$`, `${\ }^{1}$`, `$~^{1}$`
/// (2112.13115, 1211.5032), the script braced whole, `${^1}$`, one in `\( … \)` with or without one, `\( ^{1} \)`, or a
/// box holding only a mark,
/// `\mbox{${\ }^{1}$}` (0807.4407): the mark's operand and where the mark ends; `None` for any other token there (a
/// plain `$^{1}$` included, which every reader reads already). Shared by `notes::leads_with_mark`,
/// [`leading_line_label`], [`requested_mark_lines`] and [`spaced_marks_as_marks`] (review rev98e S2: one reader of
/// the line's leading printed label, not a spelling per binding).
pub(in crate::base_utilities) fn spaced_mark_at(
  tokens: &[Token],
  at: usize,
) -> Option<(Vec<Token>, usize)> {
  let past_spaces = |mut i: usize| {
    while tokens.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    i
  };
  let t = tokens.get(at)?;
  if LINE_BOXES.iter().any(|cs| *t == T_CS!(*cs)) {
    let open = past_spaces(at + 1);
    if tokens.get(open)?.get_catcode() != Catcode::BEGIN {
      return None;
    }
    let end = skip_group(tokens, open);
    let inner = tokens.get(open + 1..end.checked_sub(1)?)?;
    let inner_at = inner.iter().position(|t| *t != T_SPACE!())?;
    let (operand, next) = spaced_mark_at(inner, inner_at).or_else(|| {
      // (a box holding a plain mark: `\mbox{$^{1}$}`)
      (inner[inner_at].get_catcode() == Catcode::MATH
        && inner.get(inner_at + 1) == Some(&T_SUPER!()))
      .then(|| sup_operand_at(inner, inner_at + 2))
      .flatten()
      .filter(|(_, next)| {
        inner
          .get(*next)
          .is_some_and(|t| t.get_catcode() == Catcode::MATH)
      })
      .map(|(operand, next)| (operand, next + 1))
    })?;
    return inner[next..]
      .iter()
      .all(|t| *t == T_SPACE!())
      .then_some((operand, end));
  }
  let inline = *t == T_CS!("\\(");
  if t.get_catcode() != Catcode::MATH && !inline {
    return None;
  }
  let open = past_spaces(at + 1);
  // (`\( ^{1} \)`: no nucleus needed)
  if inline && tokens.get(open) == Some(&T_SUPER!()) {
    let (operand, next) = sup_operand_at(tokens, open + 1)?;
    let close = past_spaces(next);
    return (tokens.get(close) == Some(&T_CS!("\\)"))).then_some((operand, close + 1));
  }
  // (the whole script braced, `${^1}$`: 2208.09667, 2308.02280's lines after their first)
  if tokens
    .get(open)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let end = skip_group(tokens, open);
    let inner = tokens.get(open + 1..end.checked_sub(1)?)?;
    let sup = inner.iter().position(|t| *t != T_SPACE!());
    if let Some(sup) = sup
      && inner[sup] == T_SUPER!()
    {
      let (operand, next) = sup_operand_at(inner, sup + 1)?;
      let close = past_spaces(end);
      let closes = |t: &Token| {
        if inline {
          *t == T_CS!("\\)")
        } else {
          t.get_catcode() == Catcode::MATH
        }
      };
      return (inner[next..].iter().all(|t| *t == T_SPACE!())
        && tokens.get(close).is_some_and(closes))
      .then_some((operand, close + 1));
    }
  }
  // (the nucleus: a group of spacing, `{\,\!}`, `{}`, or spacing unbraced, `$~^{1}$`, `$\,^{1}$`, 1211.5032)
  let end = if tokens.get(open)?.get_catcode() == Catcode::BEGIN {
    let end = skip_group(tokens, open);
    if !tokens
      .get(open + 1..end.checked_sub(1)?)?
      .iter()
      .all(is_nucleus_spacing)
    {
      return None;
    }
    end
  } else {
    let spacing = tokens[open..]
      .iter()
      .take_while(|t| is_nucleus_spacing(t))
      .count();
    if spacing == 0 {
      return None;
    }
    open + spacing
  };
  let sup = past_spaces(end);
  if tokens.get(sup) != Some(&T_SUPER!()) {
    return None;
  }
  let (operand, next) = sup_operand_at(tokens, sup + 1)?;
  let close = past_spaces(next);
  tokens
    .get(close)
    .is_some_and(|t| {
      if inline {
        *t == T_CS!("\\)")
      } else {
        t.get_catcode() == Catcode::MATH
      }
    })
    .then_some((operand, close + 1))
}

/// An affiliation block whose labels are superscripts spelled otherwise (`spaced_mark_at`: set on a spacing nucleus, in
/// `\( … \)`, in a box of their own) (`${\,\!}^1$Department of Applied
/// Mathematics …\\ and ${\,\!}^2$Department of Physics …`, 2112.13115; `\mbox{${\ }^{1}$} Department of Physics …`,
/// 0807.4407), written with the plain marks they print, `$^{1}$`, so its lines are read by their marks as a `$^{1}$`
/// block's are (review rev98e S2): when such a mark, or a plain `$^{…}$`, leads the block, each such mark of a label the
/// names request (`$^1$CITI Laboratory, INRIA, $\quad ^2$LTCI, …`, 2106.01286), and a number set in math alone at a
/// line's start (`$1$ Gazi University …\\ $2$ Ankara …`, 0708.2552, 0909.5340). `None` otherwise — a nucleus mark in a
/// block no mark leads stays as written (`${}^{3}$He`).
pub fn spaced_marks_as_marks(content: &Tokens) -> Option<Tokens> {
  spaced_marks_written(content, false)
}

/// [`spaced_marks_as_marks`], each script braced as it was written when `as_written` (`${}^1$` as `$^1$`, `${}^{1}$` as
/// `$^{1}$`): the frontmatter API's Cut shows the mark it reads, and a braced script renders otherwise
/// (`affiliations::printed_marks_as_marks`; review rev99a2 S-c, hep-ph0007234's `${}^1$`).
pub(in crate::base_utilities) fn spaced_marks_written(
  content: &Tokens,
  as_written: bool,
) -> Option<Tokens> {
  let tokens = content.unlist_ref();
  let switch = |t: &Token| MARK_SWITCHES.iter().any(|cs| *t == T_CS!(*cs));
  // (past a one-argument wrapper the block opens with, `\institution{${}^a$Univ A \quad ${}^b$Univ B}` — acmart's
  // address part, as the Cut reads it, `cut_marked_contact` — a mark command or a box of its own excepted)
  let wrapper = |i: usize| {
    let t = &tokens[i];
    t.get_catcode() == Catcode::CS
      && tokens
        .get(i + 1)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
      && !is_mark_command(t)
      && !LINE_BOXES.iter().any(|cs| *t == T_CS!(*cs))
  };
  let lead = (0..tokens.len()).find(|&i| {
    let t = &tokens[i];
    *t != T_SPACE!() && *t != T_BEGIN!() && *t != T_CS!("\\noindent") && !switch(t) && !wrapper(i)
  })?;
  // (a number set in math at a line's start, no superscript, `$1$ Gazi University …\\ $2$ Ankara University …`, is the
  // mark it labels the line with where the names request it — 0708.2552; 0909.5340's `$1$` line over `$^2$`-led ones:
  // [`math_number_mark_at`])
  let mark_at = |i: usize| {
    spaced_mark_at(tokens, i).or_else(|| {
      (i == lead || at_line_start(tokens, i))
        .then(|| math_number_mark_at(tokens, i))
        .flatten()
    })
  };
  // (or a plain mark leads it, and a mark spelled otherwise leads a later entry: IEEEtran's `$^{1}$CITI Laboratory, INRIA,
  // $\quad ^2$LTCI, … $\quad ^3$Technion`, 2106.01286)
  if mark_at(lead).is_none()
    && !(tokens[lead].get_catcode() == Catcode::MATH && tokens.get(lead + 1) == Some(&T_SUPER!()))
  {
    return None;
  }
  let (_, requested) = queued_name_marks();
  let requests = |operand: &[Token]| {
    let mut mark = vec![T_MATH!(), T_SUPER!(), T_BEGIN!()];
    mark.extend_from_slice(operand);
    mark.extend([T_END!(), T_MATH!()]);
    let marks = content_marks(&Tokens::new(mark));
    !marks.is_empty() && marks.iter().all(|m| requested.contains(&mark_key(m)))
  };
  let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
  let mut changed = false;
  let mut i = 0;
  while i < tokens.len() {
    if let Some((operand, next)) = mark_at(i)
      && requests(&operand)
    {
      // (a script of one token written without braces, `${}^1$`)
      let bare = as_written
        && operand.len() == 1
        && tokens[i..next]
          .iter()
          .rposition(|t| *t == T_SUPER!())
          .and_then(|sup| tokens[i + sup + 1..next].iter().find(|t| **t != T_SPACE!()))
          .is_some_and(|t| t.get_catcode() != Catcode::BEGIN);
      out.extend([T_MATH!(), T_SUPER!()]);
      if bare {
        out.extend(operand);
      } else {
        out.push(T_BEGIN!());
        out.extend(operand);
        out.push(T_END!());
      }
      out.push(T_MATH!());
      changed = true;
      i = next;
    } else {
      out.push(tokens[i]);
      i += 1;
    }
  }
  changed.then(|| Tokens::new(out))
}

/// A number set in math alone at `at`, `$1$`, `$ 12 $`: its digits and where the formula ends ([`spaced_marks_written`]
/// reads one at a line's start as the mark it labels the line with).
fn math_number_mark_at(tokens: &[Token], at: usize) -> Option<(Vec<Token>, usize)> {
  if tokens.get(at)?.get_catcode() != Catcode::MATH {
    return None;
  }
  let past_spaces = |mut i: usize| {
    while tokens.get(i) == Some(&T_SPACE!()) {
      i += 1;
    }
    i
  };
  let start = past_spaces(at + 1);
  let digits = tokens[start..]
    .iter()
    .take_while(|t| {
      t.get_catcode() == Catcode::OTHER && t.with_str(|s| s.chars().all(|c| c.is_ascii_digit()))
    })
    .count();
  let close = past_spaces(start + digits);
  (digits > 0
    && tokens
      .get(close)
      .is_some_and(|t| t.get_catcode() == Catcode::MATH))
  .then(|| (tokens[start..start + digits].to_vec(), close + 1))
}

/// Does a line of `tokens` start at `i` — only spaces between it and a `\\` before it, or that `\\`'s optional
/// length (`\\[5pt]`)?
fn at_line_start(tokens: &[Token], i: usize) -> bool {
  let mut j = i;
  while j > 0 && tokens[j - 1] == T_SPACE!() {
    j -= 1;
  }
  if j > 0
    && tokens[j - 1] == T_OTHER!("]")
    && let Some(open) = tokens[..j - 1].iter().rposition(|t| *t == T_OTHER!("["))
  {
    j = open;
    while j > 0 && tokens[j - 1] == T_SPACE!() {
      j -= 1;
    }
  }
  j > 0 && tokens[j - 1].defined_as(&T_CS!("\\\\"))
}

/// How many lines of an affiliation block lead with a superscript mark of a label the names request — its own lines and
/// those of the boxes, groups and environments it sets them in, split at every `\\` (`\parbox{\linewidth}{$^{1}$Univ
/// A\\ $^{2}$Univ B}`: 2; `\begin{tabular}{c} $^{1}$…\\ $^{2}$…\end{tabular}`, 0911.3234), past the spaces, font
/// switches, `\noindent`/`\nonumber`, box openers and an environment's opener with its own arguments before the mark; a mark the
/// binding's readers do not read leads a line too (`\( ^{1} \)Univ A`). The labelled lines of a multi-line block, as
/// against marks inside one line's text (`Joint Institute, \textsuperscript{1}Univ A and \textsuperscript{2}Univ B`:
/// 0), which is one entry its group shares (review rev98e S2).
pub fn requested_mark_lines(content: &Tokens) -> usize {
  let tokens = content.unlist_ref();
  let (_, requested) = queued_name_marks();
  let switch = |t: &Token| MARK_SWITCHES.iter().any(|cs| *t == T_CS!(*cs));
  let skippable = |t: &Token| {
    *t == T_SPACE!()
      || *t == T_BEGIN!()
      || *t == T_CS!("\\noindent")
      || *t == T_CS!("\\nonumber")
      || *t == T_CS!("\\par")
      || switch(t)
  };
  let mut starts = vec![0];
  starts.extend(
    tokens
      .iter()
      .enumerate()
      .filter(|(_, t)| t.defined_as(&T_CS!("\\\\")))
      .map(|(i, _)| i + 1),
  );
  starts
    .into_iter()
    .filter(|&start| {
      let mut i = start;
      loop {
        while tokens.get(i).is_some_and(skippable) {
          i += 1;
        }
        // (a box the line is set in: `\parbox[t]{\linewidth}{…}`, `\mbox{…}`, `\hbox{…}`, an environment
        // `\begin{tabular}{c}` — past its opener; 0911.3234's two labelled lines in a `tabular`)
        match tokens.get(i) {
          Some(t) if *t == T_CS!("\\begin") => {
            i += 1;
            while tokens.get(i) == Some(&T_SPACE!()) {
              i += 1;
            }
            let name_end = skip_group(tokens, i);
            let name: String = tokens
              .get(i + 1..name_end.saturating_sub(1))
              .unwrap_or_default()
              .iter()
              .map(ToString::to_string)
              .collect();
            i = name_end;
            // (the environment's own arguments: latex.ltx `tabular`/`array` [pos]{cols}, `tabular*` {width}[pos]{cols},
            // `minipage` [pos]{width}; any other none, its first group the line's)
            let groups = match name.as_str() {
              "tabular" | "array" | "minipage" => 1,
              "tabular*" => 2,
              _ => 0,
            };
            let mut taken = 0;
            while taken < groups {
              while tokens.get(i) == Some(&T_SPACE!()) {
                i += 1;
              }
              match tokens.get(i) {
                Some(t) if t.get_catcode() == Catcode::BEGIN => {
                  i = skip_group(tokens, i);
                  taken += 1;
                },
                Some(t) if *t == T_OTHER!("[") => {
                  while tokens.get(i).is_some_and(|t| *t != T_OTHER!("]")) {
                    i += 1;
                  }
                  i += 1;
                },
                _ => break,
              }
            }
          },
          Some(t) if *t == T_CS!("\\parbox") || *t == T_CS!("\\minipage") => {
            i += 1;
            if tokens.get(i) == Some(&T_OTHER!("[")) {
              while tokens.get(i).is_some_and(|t| *t != T_OTHER!("]")) {
                i += 1;
              }
              i += 1;
            }
            while tokens.get(i) == Some(&T_SPACE!()) {
              i += 1;
            }
            i = skip_group(tokens, i);
          },
          Some(t) if LINE_BOXES.iter().any(|cs| *t == T_CS!(*cs)) => i += 1,
          _ => break,
        }
      }
      let Some(t) = tokens.get(i) else {
        return false;
      };
      let operand = if let Some((operand, _)) = spaced_mark_at(tokens, i) {
        Some(operand)
      } else if t.get_catcode() == Catcode::MATH
        && tokens
          .get(i + 1..)
          .and_then(|rest| rest.iter().find(|t| **t != T_SPACE!()))
          == Some(&T_SUPER!())
      {
        let sup = i
          + 1
          + tokens[i + 1..]
            .iter()
            .take_while(|t| **t == T_SPACE!())
            .count();
        sup_operand_at(tokens, sup + 1).map(|(operand, _)| operand)
      } else if *t == T_CS!("\\textsuperscript") {
        sup_operand_at(tokens, i + 1).map(|(operand, _)| operand)
      } else {
        None
      };
      operand.is_some_and(|operand| {
        let mut mark = vec![T_MATH!(), T_SUPER!(), T_BEGIN!()];
        mark.extend(operand);
        mark.extend([T_END!(), T_MATH!()]);
        content_marks(&Tokens::new(mark))
          .iter()
          .any(|m| requested.contains(&mark_key(m)))
      })
    })
    .count()
}

fn group_shares_the_mark(content: &Tokens, alternating: bool) -> bool {
  let mut marks: Vec<String> = superscript_scripts(&content.to_string()).concat();
  marks.dedup();
  with_value("frontmatter_raw", |v| match v {
    Some(Stored::FrontmatterRaw(queue)) => {
      let is_affiliation = |entry: &RawFrontmatter| {
        entry.0 == "ltx:contact" && entry.1.get("role").map(String::as_str) == Some("affiliation")
      };
      let since: Vec<&RawFrontmatter> = queue
        .iter()
        .rev()
        .take_while(|entry| !is_affiliation(entry))
        .filter(|entry| entry.0 == "ltx:creator")
        .collect();
      let previous = queue.iter().rposition(is_affiliation);
      let follows_an_author = |at: usize| queue[..at].iter().any(|entry| entry.0 == "ltx:creator");
      if !since.is_empty() && previous.is_some_and(|at| !alternating || follows_an_author(at)) {
        return true;
      }
      let [mark] = marks.as_slice() else {
        return false;
      };
      let scripts: Vec<Vec<String>> = since
        .iter()
        .flat_map(|entry| superscript_scripts(&entry.2.to_string()))
        .filter(|script| !script.is_empty())
        .collect();
      !scripts.is_empty() && scripts.iter().all(|script| script.contains(mark))
    },
    _ => false,
  })
}

/// The surname a name ends with, to match an institute's `\at` name with its author: the last run of letters,
/// lowercased (`A.M.Bykov` and "Andrei Bykov" give "bykov"). `None` for a name without letters.
fn surname_key(name: &str) -> Option<String> {
  name
    .rsplit(|c: char| !c.is_alphabetic())
    .find(|word| !word.is_empty())
    .map(str::to_lowercase)
}

/// The contact roles that are an institute itself, which its `\email`/`\url` may follow under its label.
fn is_institute_role(role: &str) -> bool {
  matches!(role, "affiliation" | "authorblock" | "address")
}

/// Contact roles that are institute-level information shared by every author (an
/// affiliation, or an address / email / url that inherited its label), as opposed
/// to a single person's own note. When such a contact is orphaned (a shared
/// `\institute` with no per-author `\inst`), it is collected onto the trailing
/// shared creator rather than dropped or stranded on author 1 (OXIDIZED_DESIGN #159).
fn is_shared_contact_role(role: &str) -> bool {
  matches!(
    role,
    "affiliation"
      | "altaffiliation"
      | "authorblock"
      | "address"
      | "altaddress"
      | "currentaddress"
      | "email"
      | "url"
  )
}

#[cfg(test)]
mod held_annotations {
  use super::*;

  /// An element `name` with a `role` and a text, appended to `parent`.
  fn child(
    doc: &libxml::tree::Document,
    parent: &mut Node,
    name: &str,
    role: &str,
    text: &str,
  ) -> Node {
    let mut node = Node::new(name, None, doc).expect("a node");
    if !role.is_empty() {
      node.set_attribute("role", role).expect("an attribute");
    }
    node.append_text(text).expect("a text");
    parent.add_child(&mut node).expect("a child");
    node
  }

  /// The index answers as reading the authors' children at the time of the question does (perf 96p L3): an
  /// annotation of another kind or text is not held, one placed on an author afterwards is — once `touched` says so,
  /// whether it was placed on the author or inside one of its children — and one removed is no longer held.
  #[test]
  fn held_annotations_are_read_again_after_an_author_changes() {
    let doc = libxml::tree::Document::new().expect("a document");
    let mut root = Node::new("document", None, &doc).expect("a node");
    let mut doc_mut = doc.clone();
    doc_mut.set_root_element(&root);
    let mut ann = child(&doc, &mut root, "creator", "author", "");
    let mut bob = child(&doc, &mut root, "creator", "author", "");
    child(&doc, &mut ann, "contact", "affiliation", " Univ X ");
    let authors = vec![ann.clone(), bob.clone()];
    let mut stub = child(&doc, &mut root, "creator", "pending", "");
    let univ_x = child(&doc, &mut stub, "contact", "affiliation", "Univ X");
    let univ_y = child(&doc, &mut stub, "contact", "affiliation", "Univ Y");
    let email_x = child(&doc, &mut stub, "contact", "email", "Univ X");
    let rdf_y = child(&doc, &mut stub, "rdf", "", "Univ Y");
    let mut held = HeldAnnotations::new(&authors);
    assert!(
      held.holds(&authors, &univ_x),
      "same kind, same trimmed text"
    );
    assert!(!held.holds(&authors, &univ_y), "another text");
    assert!(!held.holds(&authors, &email_x), "another role");
    assert!(!held.holds(&authors, &rdf_y), "no contact of its text");
    // (placed on Bob: held once Bob is touched — a metadata's text as a contact's too)
    let mut placed = child(&doc, &mut bob, "contact", "affiliation", "Univ Y");
    held.touched(&bob);
    assert!(held.holds(&authors, &univ_y));
    assert!(held.holds(&authors, &rdf_y));
    // (changed inside one of Ann's children: Ann is read again)
    placed.unlink();
    let mut wrapper = child(&doc, &mut ann, "contact", "email", "");
    let inner = child(&doc, &mut wrapper, "text", "", "Univ X");
    held.touched(&inner);
    assert!(held.holds(&authors, &email_x));
    held.touched(&bob);
    assert!(!held.holds(&authors, &univ_y), "removed from Bob");
  }
}
