//! Inserting the front matter into the document: creators, contacts, late frontmatter, unanswered marks.

use super::*;

pub fn insert_frontmatter(document: &mut Document) -> Result<()> {
  if lookup_bool("frontmatter_done") {
    return Ok(());
  }
  // A raw class's title-page stores in a document without `\maketitle` (frontmatter_stores.rs).
  crate::frontmatter_stores::harvest_stores(false)?;
  digest_front_matter()?; // If needed
  let frontmatter_elements_set: HashSet<String> = FRONTMATTER_ELEMENTS
    .iter()
    .map(ToString::to_string)
    .collect();

  // Collect the frontmatter hash keys via with_value — we only need the
  // key set here; the full HashTagData clone previously happened just
  // to call .keys().cloned().collect(). The hash itself is consumed a
  // few lines below via remove_value, so no iteration on the borrow
  // survives past this closure.
  let set_keys: Vec<String> = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.keys().cloned().collect(),
    _ => Vec::new(),
  });
  if set_keys.is_empty() {
    return Ok(());
  }

  // If doc ONLY has abstract as frontmatter, defer until abstract's document location
  if set_keys.len() == 1 && set_keys[0] == "ltx:abstract" && !lookup_bool("frontmatter_deferred") {
    assign_value("frontmatter_deferred", true, Some(Scope::Global));
    return Ok(());
  }

  // OK, we're placing FrontMatter here, now.
  assign_value("frontmatter_done", true, Some(Scope::Global));
  remove_frontmatter_marks(document);

  // Remove frontmatter and replace with empty
  let mut frontmatter = match remove_value("frontmatter") {
    Some(Stored::HashTagData(frnt)) => frnt,
    _ => return Ok(()),
  };
  assign_value(
    "frontmatter",
    Stored::HashTagData(HashMap::default()),
    Some(Scope::Global),
  );

  // Order: first go through frontmatter_elements order, then any custom keys
  let custom_keys: Vec<String> = frontmatter
    .keys()
    .filter(|key| !frontmatter_elements_set.contains(key.as_str()))
    .map(ToString::to_string)
    .collect();
  let mut all_keys: Vec<String> = FRONTMATTER_ELEMENTS
    .iter()
    .map(ToString::to_string)
    .collect();
  all_keys.extend(custom_keys);

  // A `\maketitle` run inside a box capture (ltx-talk.cls:515 frames, unifront,
  // `\parbox{…}{\maketitle}`) no longer reaches here trapped: `place_frontmatter`
  // always inserts through a root-level `_Capture_` (#262), so the frontmatter lands
  // at the head as real elements. (Before 56gf a current-point flush degraded each
  // entry to `ltx:text class="ltx_<name>"` inside the box.) Guard
  // `perfect_kernel_batch54::maketitle_inside_a_box_goes_to_the_head`.
  for key in &all_keys {
    if let Some(list) = frontmatter.remove(key) {
      // Dubious, but assures that frontmatter appears in text mode...
      document.set_box_to_absorb(
        Tbox::new(
          pin!(""),
          lookup_font(),
          None,
          Tokens!(T_SPACE!()),
          SymHashMap::default(),
        )
        .into(),
      );
      for item in list {
        insert_frontmatter_entry(document, &item)?;
      }
      document.expire_box_to_absorb();
    }
  }
  coalesce_empty_creators(document)?;
  distribute_upfront_contacts(document)?;
  let answered = answered_labels(document);
  relocate_annotations(document)?;
  respell_emails(document)?;
  show_unanswered_marks(document, &answered)?;
  drop_empty_contacts(document)?;
  // (FM S2b: a written family's paper, where it differs from the observer's reading by the writer's operations alone,
  // written as the reading has it — under the shadow switch only; OXIDIZED_DESIGN_DIVERGENCES #837 "S2b")
  write_observed_frontmatter(document)?;
  // (the authors as written: a name the writer re-parted holds one person, OD #928)
  flag_merged_creators(document)?;
  Ok(())
}

/// A contact of no content — a template's `\curraddr{}`, `\urladdr{}`, `\dedicatory{}`, aastex's `\nocollaboration` —
/// is no contact: its name label printed over nothing ("Current address:"), where pdflatex prints nothing (58 of the
/// 2,973 ams census papers: 2206.14950, 2203.07822, 2001.01741, 2003.03018; 7 of the 688 aas). Removed once every entry
/// is placed — an empty entry still counts in today's pile (`perfect_kernel_batchfms2a2r4::an_empty_entry_is_in_todays_pile`)
/// — with a nameless creator it leaves empty. Guard `perfect_kernel_batch81::empty_contacts_print_nothing`.
fn drop_empty_contacts(document: &mut Document) -> Result<()> {
  // (empty: no text, and no element but an empty `text` or a `break` — an icon, a link, a formula is content)
  let empty = |contact: &Node| {
    contact.get_content().trim().is_empty()
      && element_nodes(contact).iter().all(|child| {
        matches!(child.get_name().as_str(), "text" | "break") && element_nodes(child).is_empty()
      })
  };
  for creator in document.findnodes("//ltx:creator", None) {
    let dropped: Vec<Node> = element_nodes(&creator)
      .into_iter()
      .filter(|child| child.get_name() == "contact" && empty(child))
      .collect();
    if dropped.is_empty() {
      continue;
    }
    for contact in dropped {
      document.remove_node(contact);
    }
    // (a nameless creator holding only them is no creator either; the scope is the creators this pass emptied)
    if element_nodes(&creator).is_empty() {
      document.remove_node(creator);
    }
  }
  Ok(())
}

/// FIDELITY_CHARTER §4, "ownership by evidence, never by bare position": an e-mail contact one address alone fills,
/// on an author it does not spell, goes to the one author it spells — an `\email` call placed by its `annotate=last`
/// after a one-call author list (AASTeX 5's `\author{Ann Able and Bob Baker}\email{able@x.org}`: 0911.3506
/// `bourouaine@mps.mpg.de` on Eckart Marsch, 1506.04035, 1203.1317, 0904.0560 — 92 of the 688 aas census papers); a copy
/// the author it spells holds already is dropped. An address that spells no one, or several, stays (`spelled_owner`).
/// Guard `perfect_kernel_batch81::an_email_call_goes_to_the_author_it_spells`.
fn respell_emails(document: &mut Document) -> Result<()> {
  let authors = document.findnodes("//ltx:creator[@role='author'][ltx:personname]", None);
  if authors.len() < 2 {
    return Ok(());
  }
  let keys: Vec<NameKeys> = authors
    .iter()
    .map(|author| {
      let name = document
        .findnode("ltx:personname", Some(author))
        .map(|person| person.get_content())
        .unwrap_or_default();
      NameKeys::of(&Tokens::new(ExplodeText!(&name)))
    })
    .collect();
  let mut moves: Vec<(Node, usize)> = Vec::new();
  for (at, author) in authors.iter().enumerate() {
    for contact in document.findnodes("ltx:contact[@role='email']", Some(author)) {
      // (one a mark's label placed — a marked e-mail line — is no bare position: `*baker@x.org` under `A. Able*`; a fuzzy
      // label, a name, that placed it nowhere left it at the call's position: `\correspondingauthor{…}\email{…}`,
      // 2003.00016, 2003.11098, 2204.06937)
      if contact
        .get_attribute("_label")
        .is_some_and(|label| !label.is_empty() && !label.starts_with("fuzzy:"))
      {
        continue;
      }
      // (its one address, a lead word aside: `\email{Email: pdrew@utexas.edu}`, 2003.00016)
      let text = contact.get_content();
      if text.matches('@').count() != 1 {
        continue;
      }
      let Some(address) = text
        .split_whitespace()
        .find(|word| word.contains('@'))
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
      else {
        continue;
      };
      if let Some(owner) = spelled_owner(&keys, address)
        && owner != at
      {
        moves.push((contact, owner));
      }
    }
  }
  for (contact, owner) in moves {
    let address = contact.get_content();
    let held = document
      .findnodes("ltx:contact[@role='email']", Some(&authors[owner]))
      .iter()
      .any(|c| c.get_content().trim() == address.trim());
    if !held {
      let mut target = authors[owner].clone();
      document.append_clone(&mut target, vec![contact.clone()])?;
    }
    document.remove_node(contact);
  }
  Ok(())
}

/// Remove author creators whose `<ltx:personname>` is empty, moving any contacts they
/// carry to the preceding real creator (else the first following one).
///
/// A flat comma author list with interspersed `\IEEEmembership{…}`/`\thanks{…}` — e.g.
/// `\author{Alice, \IEEEmembership{…}, and Bob, …\thanks{…}}` (html_feedback#4539,
/// witness 2508.00603) — comma/" and "-splits into pieces where the membership pieces
/// digest to nothing, surfacing as empty `<ltx:personname/>` creators; a trailing
/// `\thanks` then strands its affil/email on a nameless creator. This coalesces those:
/// contactless empties are dropped; a contact-bearing empty's contacts move to the
/// preceding real author. `\footnotemark`-note markers keep a personname non-empty
/// (2507.06670 "Yu Zhang"), so real authors are untouched.
///
/// Moves every annotation, `<ltx:contact>`, `<ltx:note>` and `<ltx:rdf>`: an author `\thanks` is a
/// marked `<ltx:note role="thanks">` (OXIDIZED_DESIGN #156), so a trailing `\thanks` on
/// a nameless comma-split creator would otherwise be dropped with the empty creator —
/// witness 1510.02728 (`\author{Sani,~\IEEEmembership{…} Vosoughi,~\IEEEmembership{…}%
/// \thanks{…NSF…}}`), where the note must land on the last real author, as a contact did;
/// and an author's metadata (OXIDIZED_DESIGN #479: an AASTeX 7 `\email` after an `\author` whose name prints nothing).
fn coalesce_empty_creators(document: &mut Document) -> Result<()> {
  let creators = document.findnodes("//ltx:creator[@role='author']", None);
  let mut to_remove: Vec<Node> = Vec::new();
  let mut last_real: Option<Node> = None;
  // Annotations (contacts + notes) of leading empties (before any real author), held
  // until the first real one.
  let mut orphan_annotations: Vec<Node> = Vec::new();
  for creator in creators {
    if creator_personname_empty(document, &creator) {
      let annotations: Vec<Node> = creator
        .get_child_nodes()
        .into_iter()
        .filter(|c| {
          c.get_type() == Some(NodeType::ElementNode)
            && with(document::get_node_qname(c), |q| {
              q == "ltx:contact" || q == "ltx:note" || q == "ltx:rdf"
            })
        })
        .collect();
      if let Some(ref mut prev) = last_real {
        if !annotations.is_empty() {
          document.append_clone(prev, annotations)?;
        }
      } else {
        orphan_annotations.extend(annotations);
      }
      to_remove.push(creator);
    } else {
      if !orphan_annotations.is_empty() {
        let mut first = creator.clone();
        document.append_clone(&mut first, std::mem::take(&mut orphan_annotations))?;
      }
      last_real = Some(creator);
    }
  }
  for creator in to_remove {
    document.remove_node(creator);
  }
  Ok(())
}

/// Redistribute a mis-piled contact block back to its authors (surpass-Perl).
///
/// The amsart idiom `\author{A}\author{B}\author{C}` followed by paired
/// `\address{}\email{}` declares every author up front, then all the contacts.
/// LaTeXML's default "attach a contact to the preceding creator" then bunches
/// EVERY address+email under the LAST author (arXiv:2308.06214v1,
/// arXiv/html_feedback#46); Perl 0.8.8 does the same (SHARED limitation).
///
/// Fix ONLY the clean, unambiguous pile — an `N × m` grid: the other `N-1`
/// authors carry no contact, the last author's `K` contacts split evenly
/// (`K = N·m`) into a role-periodic sequence (`role[i] == role[i+m]`), so group
/// `j` is handed to author `j`. Any irregular pile — a heterogeneous role
/// sequence, per-author counts that differ, or contacts already spread across
/// authors (the interleaved idiom, which is correct as-is) — fails the gate and
/// is left EXACTLY as Perl attached it. Mirrors the "distribute-when-clean, else
/// keep prior" rule of the shared-email splitter (OXIDIZED_DESIGN #52(j)).
fn distribute_upfront_contacts(document: &mut Document) -> Result<()> {
  let creators = document.findnodes("//ltx:creator[@role='author']", None);
  let n = creators.len();
  if n < 2 {
    return Ok(());
  }
  let contacts_of = |c: &Node| -> Vec<Node> {
    c.get_child_nodes()
      .into_iter()
      .filter(|x| {
        x.get_type() == Some(NodeType::ElementNode)
          && with(document::get_node_qname(x), |q| q == "ltx:contact")
      })
      .collect()
  };
  // Signature: only the LAST author carries any contacts.
  for creator in &creators[..n - 1] {
    if !contacts_of(creator).is_empty() {
      return Ok(());
    }
  }
  // …and no other author is to receive one of their kind: an author whose mark requests a label a pending contact of a
  // role the last author's contacts hold carries gets it from `relocate_annotations`, after this — the last author's
  // contacts are then the run's, not a pile (`\author{Ann Able$^{1}$}\author{Bob Baker}\affil{$^1$Univ X}\affil{Univ Y}
  // \affil{Univ W}` handed Univ Y to Ann and kept Univ W alone on Bob; 77n r5 review probes g01-g06, m11). A pending
  // contact of another role leaves the pile one (2605.30274's e-mail lines on the last of two authors whose affiliations
  // are mark-picked).
  let last_roles: HashSet<String> = contacts_of(&creators[n - 1])
    .iter()
    .map(|contact| contact.get_attribute("role").unwrap_or_default())
    .collect();
  let mut pending_labels: HashSet<String> = HashSet::default();
  for pending in document.findnodes(".//*[@role='pending']", None) {
    for note in element_nodes(&pending) {
      if !last_roles.contains(&note.get_attribute("role").unwrap_or_default()) {
        continue;
      }
      let labels = [
        note.get_attribute("_label").unwrap_or_default(),
        pending.get_attribute("_annotations").unwrap_or_default(),
      ];
      for label in labels.iter().flat_map(|labels| labels.split(',')) {
        if !label.is_empty() {
          pending_labels.insert(label.to_string());
        }
      }
    }
  }
  let answered_later = |creator: &Node| {
    creator
      .get_attribute("_annotations")
      .unwrap_or_default()
      .split(',')
      .any(|label| pending_labels.contains(label))
  };
  if creators[..n - 1].iter().any(answered_later) {
    return Ok(());
  }
  let last = creators[n - 1].clone();
  let contacts = contacts_of(&last);
  let k = contacts.len();
  if k == 0 || k % n != 0 {
    return Ok(());
  }
  let m = k / n;
  // Role sequence must be periodic with period m (every group has the same
  // role pattern), else the pile is not a clean grid — leave it to Perl's rule.
  let roles: Vec<String> = contacts
    .iter()
    .map(|x| x.get_attribute("role").unwrap_or_default())
    .collect();
  if (0..k - m).any(|i| roles[i] != roles[i + m]) {
    return Ok(());
  }
  // Group j (contacts[j*m .. (j+1)*m]) -> author j. The last group stays put;
  // the earlier groups clone onto their author and the originals are removed.
  let mut to_remove: Vec<Node> = Vec::new();
  for (j, creator) in creators.iter().enumerate().take(n - 1) {
    let group: Vec<Node> = contacts[j * m..(j + 1) * m].to_vec();
    let mut target = creator.clone();
    document.append_clone(&mut target, group.clone())?;
    to_remove.extend(group);
  }
  for node in to_remove {
    document.remove_node(node);
  }
  Ok(())
}

/// True if `creator`'s `<ltx:personname>` carries no name — no element child and no
/// non-whitespace text (a bare `<ltx:personname/>`), or no personname at all. A
/// personname with a real name (text) or a `<ltx:text>`/`<ltx:note>` child is NOT empty.
fn creator_personname_empty(document: &mut Document, creator: &Node) -> bool {
  match document.findnode("ltx:personname", Some(creator)) {
    None => true,
    Some(person) => !person.get_child_nodes().iter().any(|c| {
      c.get_type() == Some(NodeType::ElementNode)
        || (c.get_type() == Some(NodeType::TextNode) && !c.get_content().trim().is_empty())
    }),
  }
}

/// Streaming: insert frontmatter that arrived AFTER the in-document insertion
/// point had already been absorbed.
///
/// Eager conversion runs every `insert_frontmatter` with digestion complete
/// (build starts after digestion ends), so the `\maketitle`-time insertion
/// sees the whole document's frontmatter. Interleaved pass 1 runs it
/// mid-digestion — with the REAL builder navigation, so paragraph splitting
/// and ids come out exactly as eager — but anything queued after that moment
/// (an abstract following `\maketitle` in an AMS-style document; sweep
/// witnesses tests/structure/amsarticle.tex, titlepage.tex) misses the
/// consumption. Those late arrivals re-populate the `frontmatter` state hash;
/// this inserts them at their canonical slot — after the last frontmatter
/// element already placed — by re-running `insert_frontmatter` there
/// (element order within it is canonical, not arrival order). A late abstract then
/// goes behind the cover printed before its marker, as in the eager build.
pub fn insert_late_frontmatter(document: &mut Document) -> Result<()> {
  debug_assert!(!document.root_after_open_deferred());
  let has_late = with_value(
    "frontmatter",
    |v| matches!(v, Some(Stored::HashTagData(h)) if !h.is_empty()),
  ) || with_value(
    "frontmatter_raw",
    |v| matches!(v, Some(Stored::FrontmatterRaw(q)) if !q.is_empty()),
  );
  if !has_late || !lookup_bool("frontmatter_done") {
    // Nothing late, or nothing was ever inserted early — in the latter case
    // the root's deferred late hook performs the one-and-only insertion.
    for marker in document.findnodes(&s!("//{LATE_ABSTRACT_MARK}"), None) {
      document.remove_node(marker);
    }
    return Ok(());
  }
  // Anchor: the late elements' CANONICAL slot among the frontmatter already
  // placed — `insert_frontmatter` emits in `FRONTMATTER_ELEMENTS` order, so a
  // late abstract belongs between an existing date and existing keywords
  // (sweep witness tests/structure/amsarticle.tex), not after everything.
  // ROOT CHILDREN only: `//ltx:title` would also match every SECTION's
  // title, and the last of those anchors the insertion deep inside the body.
  // If several late kinds straddle an existing kind they clump at the first
  // late kind's slot — the byte-parity gate guards the day that case
  // materializes.
  let existing = document.findnodes(
    &FRONTMATTER_ELEMENTS
      .iter()
      .map(|q| format!("/ltx:document/{q}"))
      .collect::<Vec<_>>()
      .join(" | "),
    None,
  );
  let Some(last) = existing.last() else {
    return Ok(());
  };
  let rank = |qname: &str| {
    FRONTMATTER_ELEMENTS
      .iter()
      .position(|q| *q == qname)
      .unwrap_or(usize::MAX)
  };
  let min_late_rank = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(h)) => h.keys().map(|k| rank(k)).min(),
    _ => None,
  })
  .unwrap_or(usize::MAX);
  let displaced = existing.iter().find(|node| {
    let qname = format!("ltx:{}", node.get_name());
    rank(&qname) > min_late_rank
  });
  let savenode = document.get_node().clone();
  let wrapper = match displaced {
    // Canonically later frontmatter exists: insert right before it.
    Some(next) => document.insert_element_before(next, "ltx:_Capture_", None)?,
    // Everything placed is canonically earlier: insert after the last.
    None => match last.get_next_sibling() {
      Some(next) => document.insert_element_before(&next, "ltx:_Capture_", None)?,
      None => {
        let mut parent = last
          .get_parent()
          .expect("a frontmatter element always has a parent");
        document.open_element_at(&mut parent, "ltx:_Capture_", None, None)?
      },
    },
  };
  document.set_node(&wrapper);
  assign_value("frontmatter_done", false, Some(Scope::Global));
  insert_frontmatter(document)?;
  document.unwrap_nodes(wrapper)?;
  document.set_node(&savenode);
  // A late abstract's marker (`\lx@frontmatter@mark`): the abstract, now placed, goes behind the cover printed before
  // it, as in the eager build (OXIDIZED_DESIGN_DIVERGENCES #903), each in turn; every such marker goes.
  let late_marks = document.findnodes(&s!("//{LATE_ABSTRACT_MARK}"), None);
  for marker in &late_marks {
    place_abstract_after_late_mark(document, marker)?;
  }
  for marker in late_marks {
    document.remove_node(marker);
  }
  Ok(())
}

/// Insert one frontmatter entry `[tag, {attr}, @content]`.
/// Perl: insertFrontMatter_rec($document, $item) for the ARRAY case.
pub(in crate::base_utilities) fn insert_frontmatter_entry(
  document: &mut Document,
  entry: &TagData,
) -> Result<()> {
  let TagData { tag, attr, content } = entry;
  DebugFeature!("frontmatter", "FRONT Inserting {}", show_frontmatter(entry));
  // token-locators: frontmatter elements (e.g. <ltx:title> from `\title{…}`)
  // are opened here, far from their source, around content that was digested
  // and stored back at `\lx@add@frontmatter` time. open_element would otherwise
  // stamp them with no/last locator (→ the whole-document fallback in clients).
  // Recover the deferred content's span and stamp the element with it.
  #[cfg(feature = "token-locators")]
  if let Some(TagContent::Box(stuff)) = content.iter().find(|c| matches!(c, TagContent::Box(_))) {
    document.set_current_box_locator(latexml_core::definition::constructor::child_span(stuff));
  }
  // Perl: font => $stuff[0]->getFont, _force_font => 'true' when the tag
  // can have a font attribute and there is content.
  let mut attributes: HashMap<String, String> = attr.clone();
  let mut font: Option<Font> = None;
  if !content.is_empty()
    && document::can_have_attribute(tag, "font")
    && let Some(TagContent::Box(first)) = content.first()
    && let Ok(Some(f)) = first.get_font()
  {
    font = Some((*f).clone());
    attributes.insert("_force_font".to_string(), "true".to_string());
  }
  let from_store = attributes.contains_key("_store");
  let opened = document.open_element(tag, Some(attributes), font.as_ref())?;
  for item in content {
    insert_frontmatter_rec(document, item)?;
  }
  drop_trailing_paragraph_breaks(document, &opened);
  // Scoped close (OXIDIZED_DESIGN #202): a deferred body that ran to the end
  // of the input carries the `\end{document}` whatsit (an unbalanced
  // `\abstract{…`, #207); absorbing it here closes the document — and this
  // element with it — so there is nothing left of ours to close.
  document.close_element_if_open(tag)?;
  // A raw class's store whose value typesets nothing (a default that only warns,
  // umich-thesis.cls:71) carries no datum: its element is not kept (frontmatter_stores.rs).
  if from_store && opened.get_content().trim().is_empty() && opened.get_child_elements().is_empty()
  {
    document.remove_node(opened);
  }
  // At this time, the frontmatter element should really carry the actual literal values intended.
  // (Perl PR #2767 disables the former empty-element pruning here.)
  Ok(())
}

/// Perl: insertFrontMatter_rec($document, $item).
fn insert_frontmatter_rec(document: &mut Document, item: &TagContent) -> Result<()> {
  match item {
    TagContent::Entry(entry) => insert_frontmatter_entry(document, entry)?,
    // Otherwise, assume some sort of Box
    TagContent::Box(digested) => document.absorb(digested, None)?,
    // Perl absorbs the literal 'place_keeper' string for unfilled entries
    TagContent::PlaceKeeper => {
      document.absorb(&Digested::from(String::from("place_keeper")), None)?
    },
  }
  Ok(())
}

/// The labels the frontmatter entries carry (`_label`), each also without its prefix, as [`relocate_annotations`]
/// matches them (`institute:1` answers `affiliation:1`), and every mark an affiliation printed (`_bymark`: llncs's
/// `\institute{$^{1,3}$ Univ A}` answers a `\inst{3}` too, though only its first mark labels it): what an author's mark
/// request can find. A creator's own role-sequence label (`author:N`) answers no mark.
fn answered_labels(document: &mut Document) -> HashSet<String> {
  let mut answered = HashSet::default();
  for node in document.findnodes(".//*[@_bymark]", None) {
    for mark in node.get_attribute("_bymark").unwrap_or_default().split(',') {
      if !mark.is_empty() {
        answered.insert(mark.to_string());
        answered.insert(s!("affiliation:{mark}"));
      }
    }
  }
  for node in document.findnodes(".//*[@_label][not(self::ltx:creator)]", None) {
    for label in node.get_attribute("_label").unwrap_or_default().split(',') {
      // (nor does a contact's placement by that label, `label=author:N`: 2403.00801's `jiawei2020@…` placed on the
      // second author answered every mark "2")
      if label.is_empty() || label.starts_with("author:") {
        continue;
      }
      answered.insert(label.to_string());
      if let Some(pos) = label.find(':') {
        answered.insert(label[pos + 1..].to_string());
      }
    }
  }
  answered
}

/// An author's affiliation mark that no entry answers stays in the name, as LaTeX prints it (`Ann Able$^*$ and Bob
/// Bee$^1$` with no `$^1$` line; `Gus Gray\IEEEauthorrefmark{1}` whose legend is a `\thanks`, 2408.02464): read as
/// a request, it was taken out of the name to link an affiliation that is not there. A mark the name already shows
/// is not repeated. (Perl drops it too; repro sectioning-frontmatter/author_unmatched_numeric_mark.)
fn show_unanswered_marks(document: &mut Document, answered: &HashSet<String>) -> Result<()> {
  for creator in document.findnodes(".//ltx:creator[@_annotations]", None) {
    let Some(mut personname) = document
      .findnodes("ltx:personname", Some(&creator))
      .into_iter()
      .next()
    else {
      continue;
    };
    let shown = shown_marks(document, &creator);
    let shown_sups: Vec<String> = document
      .findnodes("ltx:personname//ltx:sup", Some(&creator))
      .iter()
      .map(|sup| sup.get_content())
      .collect();
    // (a symbol mark nothing answers is shown too — 1807.01584, 1805.02699, 2203.01270 — except one that placed one of
    // this author's contacts and was dropped from it, `$^\dagger$baker@y.edu`: answered on this author only,
    // `answered_symbol_key`; one the author's email shows as well stays on the name too, as printed: 1910.00401)
    let answered_here: Vec<String> = document
      .findnodes(".//*[@_answers]", Some(&creator))
      .iter()
      .flat_map(|node| {
        node
          .get_attribute("_answers")
          .unwrap_or_default()
          .split(',')
          .map(str::to_string)
          .collect::<Vec<_>>()
      })
      .collect();
    let text_marks: Vec<String> = creator
      .get_attribute("_textmarks")
      .unwrap_or_default()
      .split(',')
      .map(str::to_string)
      .collect();
    let picture_marks = creator.get_attribute("_picturemarks").unwrap_or_default();
    let plain_marks = creator.get_attribute("_plainmarks").unwrap_or_default();
    let mut marks: Vec<String> = Vec::new();
    for label in creator
      .get_attribute("_annotations")
      .unwrap_or_default()
      .split(',')
    {
      let Some(mark) = label.strip_prefix("affiliation:") else {
        continue;
      };
      if answered.contains(label) || answered.contains(mark) {
        continue;
      }
      // (a label only a picture prints is no mark of the name: the picture is dropped from it, and its text with it —
      // the TikZ ORCID icon's "iD", `\inst{1\myorcid{…}}`; review rev92b_r3 S1, as rounds 1-2 and Perl showed nothing.
      // Per request: a label the name also requests outside a picture is shown, `Ann Able$^{\circled{b}}$$^{b}$`'s b,
      // in either order — review rev92b_r4 B1, where the picture's hid it)
      if picture_marks.split(',').any(|m| m == label) && !plain_marks.split(',').any(|m| m == label)
      {
        continue;
      }
      let Some(mut text) = unanswered_mark_text(mark) else {
        continue;
      };
      // (the glyph follows the mark's mode: a `*` requested in text is `*`, in math `∗`)
      if mark == "*" && text_marks.iter().any(|m| m == label) {
        text = "*".to_string();
      }
      if symbol_mark_key(mark).is_some_and(|key| answered_here.contains(&key)) {
        continue;
      }
      // (a symbol the name shows in a superscript of its own is shown: a `$^*$`, `\lx@frontmatter@keepsup`, also
      // requests; `*` and `∗` are one mark)
      let unify = |text: &str| text.replace('\u{2217}', "*");
      let symbol_shown = !text.chars().all(char::is_alphanumeric)
        && shown_sups
          .iter()
          .any(|sup| unify(sup).contains(&unify(&text)));
      if symbol_shown || shown.contains(&text) || marks.contains(&text) {
        continue;
      }
      marks.push(text);
    }
    if marks.is_empty() {
      continue;
    }
    let mut sup = Node::new("sup", None, document.get_document())?;
    if let Some(namespace) = personname.get_namespace() {
      let _ = sup.set_namespace(&namespace);
    }
    sup.append_text(&marks.join(","))?;
    personname.add_child(&mut sup)?;
  }
  Ok(())
}

/// Add `label` (comma separated) to the requests of the frontmatter entry being digested (`_annotations`), and, for a
/// superscript requested in text (`\textsuperscript{1,*}`, not `$^{1,*}$`: `superscript`), to its text-mode marks
/// (`_textmarks`): such a `*` prints as `*`, a math one as `∗` (`show_unanswered_marks`). A mark command (`\inst`,
/// `\IEEEauthorrefmark`) sets its mark in math (llncs.cls `\def\inst#1{\unskip$^{#1}$}`), whatever mode it is read in.
/// The labels only a picture prints (`pictured`, [`frontmatter_label_request`]) are its picture marks
/// (`_picturemarks`), the others its plain marks (`_plainmarks`): a picture mark the entry never requests plainly is
/// never shown unanswered.
pub(in crate::base_utilities) fn add_annotation_labels(
  label: String,
  superscript: bool,
  pictured: Vec<String>,
) {
  let in_text = superscript && !lookup_bool("IN_MATH");
  with_pending_entry_attr(move |attr| {
    let mut append = |key: &str, new: &mut dyn Iterator<Item = &str>| {
      let mut marks = attr.get(key).cloned().unwrap_or_default();
      let before = marks.len();
      for mark in new {
        if !marks.is_empty() {
          marks.push(',');
        }
        marks.push_str(mark);
      }
      if marks.len() != before {
        attr.insert(key.to_string(), marks);
      }
    };
    append("_picturemarks", &mut pictured.iter().map(String::as_str));
    append(
      "_plainmarks",
      &mut label
        .split(',')
        .filter(|key| !key.is_empty() && !pictured.iter().any(|p| p == key)),
    );
    let labels = attr.get("_annotations").cloned().unwrap_or_default();
    let newval = if labels.is_empty() {
      label.clone()
    } else {
      s!("{labels},{label}")
    };
    DebugFeature!("frontmatter", "FRONT add annotation label {label}");
    attr.insert("_annotations".to_string(), newval);
    if in_text {
      let text_marks = attr.get("_textmarks").cloned().unwrap_or_default();
      let newval = if text_marks.is_empty() {
        label.clone()
      } else {
        s!("{text_marks},{label}")
      };
      attr.insert("_textmarks".to_string(), newval);
    }
  });
}

/// How an unanswered affiliation mark prints: a number or letter as it is, a superscript footnote symbol (`$^{\dag}$`,
/// which labels affiliations as a mark, `is_symbol_mark`) as its glyph — a label keyed by its printed text is that glyph
/// already (`\inst{1,\star}`'s `⋆`, 2105.03212; refactor stage 4, OXIDIZED_DESIGN_DIVERGENCES #919), one keyed by its
/// spelling (the fallback) keeps the control sequence; `None` for anything else (a fuzzy or composite label).
pub(in crate::base_utilities) fn unanswered_mark_text(mark: &str) -> Option<String> {
  // (its characters, each combining mark part of the one before it: `\={x}`'s `x̄` is a letter — review rev92b_r2 B1)
  let bases: Vec<char> = mark.chars().filter(|c| !is_combining_mark(*c)).collect();
  let marked = !mark.starts_with(is_combining_mark);
  if marked && !bases.is_empty() && bases.iter().all(|c| c.is_alphanumeric()) {
    return Some(mark.to_string());
  }
  let glyph = match mark {
    "\\dag" | "\\dagger" => "\u{2020}",
    "\\ddag" | "\\ddagger" => "\u{2021}",
    // (pdflatex sets a math `*` as `∗`; one requested in text stays `*`, `show_unanswered_marks`)
    "\\ast" | "*" => "\u{2217}",
    "\\star" => "\u{22C6}",
    "\\S" => "\u{A7}",
    "\\P" => "\u{B6}",
    "\\bullet" => "\u{2219}",
    "\\sharp" => "\u{266F}",
    "\\|" => "\u{2016}",
    "\\#" | "#" => "#",
    "\u{2020}" | "\u{2021}" | "\u{2217}" | "\u{A7}" | "\u{B6}" => mark,
    // (one printed symbol, `queue_digest::is_printed_symbol` — no letter, digit, space, ASCII, format character or
    // combining mark: `\up{\!1,2}`'s zero-width space is none, 2209.14855 —, with the marks on it; a run of several is
    // a whole-run label, its symbols requested apart, `parted_printed_label`)
    _ if marked && matches!(bases.as_slice(), [c] if is_printed_symbol(*c)) => mark,
    _ => return None,
  };
  Some(glyph.to_string())
}

/// What a frontmatter annotation says: its text, or an `ltx:rdf`'s `content`/`resource` (OXIDIZED_DESIGN #479), which
/// has no text.
pub(in crate::base_utilities) fn annotation_value(note: &Node) -> String {
  if note.get_name() == "rdf" {
    note
      .get_attribute("content")
      .or_else(|| note.get_attribute("resource"))
      .unwrap_or_else(|| note.get_content())
  } else {
    note.get_content()
  }
}
