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
  // the marks each author's name shows (`$^{a,b}$`: two)
  let author_marks: Vec<Vec<String>> = authors
    .iter()
    .map(|author| shown_marks(document, author))
    .collect();
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
  // `\fntext` without its `\fnref`) lost the link to its person, which is warned about.
  let shared_orphan = |note: &Node| {
    let role = note.get_attribute("role").unwrap_or_default();
    is_shared_contact_role(&role) || role == "thanks"
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
    let marked: Vec<Node> = (0..authors.len())
      .filter(|&i| *institute && keys.iter().any(|key| author_marks[i].contains(key)))
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
      // of the authors, then `\email[show]{x}` on the first, is that author's displayed address)
      let held = || {
        let (role, content) = (note.get_attribute("role"), annotation_value(&note));
        let metadata = note.get_name() == "rdf";
        authors.iter().any(|author| {
          element_nodes(author).iter().any(|c| {
            let same_kind = c.get_name() == note.get_name() && c.get_attribute("role") == role;
            (same_kind || (metadata && matches!(c.get_name().as_str(), "contact" | "rdf")))
              && annotation_value(c).trim() == content.trim()
          })
        })
      };
      if unlabelled && (pending.get_name() != "creator" || empty || held()) {
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
            },
          }
        }
      }
    }
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
      shared_creator = Some(pending);
    } else {
      document.remove_node(pending);
    }
  }
  Ok(())
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

/// The marks a creator's name shows: each `<ltx:sup>`'s (`\textsuperscript{a}`), and each formula's that is only a
/// superscript (`$^{a}$`, `$^{1,2}$`: a `<ltx:Math>` whose `tex` is `{}^{a}` until the math is rewritten into a
/// `<ltx:sup>` later).
pub(in crate::base_utilities) fn shown_marks(
  document: &mut Document,
  creator: &Node,
) -> Vec<String> {
  let mut marks: Vec<String> = document
    .findnodes("ltx:personname//ltx:sup", Some(creator))
    .iter()
    .flat_map(|sup| script_marks(sup.get_content().trim()))
    .collect();
  for math in document.findnodes("ltx:personname//ltx:Math", Some(creator)) {
    let tex = math.get_attribute("tex").unwrap_or_default();
    // (past the spacing before it: `$\,{}^{a,b}$`, 2311.14432)
    if past_mark_spacing(tex.trim())
      .trim_start_matches("{}")
      .trim_start()
      .starts_with('^')
    {
      marks.extend(superscript_scripts(&tex).concat());
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
/// mark. Both the marks an affiliation is labelled by and the marks an author shows are read so (63i review).
pub(in crate::base_utilities) fn script_marks(script: &str) -> Vec<String> {
  // (the whole script under its switches, `\footnotesize\ensuremath{1,2}`: the 1 and the 2; 2411.14110)
  let script = past_mark_switches(script);
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
  items
    .into_iter()
    .filter_map(|item| {
      // (a number in brackets or parentheses is the number: `$^{[1],[2]}$` against the line's `$^1$`, 1803.06822; so is
      // one after a footnote symbol glued to it, `\textsuperscript{*3}`, 2412.01183)
      let item: String = past_mark_switches(item)
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
/// an affiliation's mark nor its leading text is.
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
const MARK_SPACING: [&str; 4] = ["\\quad", "\\qquad", "\\thinspace", "\\enspace"];

/// `text` past the spacing it opens with ([`MARK_SPACING`], the spacing control symbols, ties and spaces).
fn past_mark_spacing(text: &str) -> &str {
  let mut rest = text;
  loop {
    rest = rest.trim_start_matches(|c: char| c == '~' || c.is_whitespace());
    let Some(after) = rest.strip_prefix('\\') else {
      return rest;
    };
    if after.starts_with([',', ';', ':', '!', ' ']) {
      rest = &after[1..];
      continue;
    }
    let len = after
      .find(|c: char| !c.is_ascii_alphabetic())
      .unwrap_or(after.len());
    if len == 0 || !MARK_SPACING.contains(&&rest[..=len]) {
      return rest;
    }
    rest = &after[len..];
  }
}

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
